// SPDX-License-Identifier: MPL-2.0

//! Role-specific view validation through both public recording routes.
use super::{
    com::{GET_DEVICE_FAILURE, Scene},
    *,
};
use crate::error::{
    DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
};
use crate::upscaler::ReactiveMaskOptions;
use std::cell::RefCell;
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

const COLORS: [(DXGI_FORMAT, u32); 4] = [
    (
        DXGI_FORMAT_R32G32B32A32_FLOAT,
        FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT,
    ),
    (
        DXGI_FORMAT_R11G11B10_FLOAT,
        FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT,
    ),
    (
        DXGI_FORMAT_R10G10B10A2_UNORM,
        FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM,
    ),
    (
        DXGI_FORMAT_R8G8B8A8_UNORM,
        FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM,
    ),
];
thread_local! { static CAPTURED: RefCell<Vec<(u32,u32,u32)>> = const { RefCell::new(Vec::new()) }; }
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: The controlled observer receives the complete descriptor matching
    // its tag and copies only format/usage/state values, retaining no borrows.
    let resources = unsafe {
        if (*header).r#type == FFX_API_DISPATCH_DESC_TYPE_UPSCALE {
            let d = &*header.cast::<ffxDispatchDescUpscale>();
            vec![d.color, d.depth, d.motionVectors, d.exposure, d.output]
        } else {
            let d = &*header.cast::<ffxDispatchDescUpscaleGenerateReactiveMask>();
            vec![d.colorOpaqueOnly, d.colorPreUpscale, d.outReactive]
        }
    };
    CAPTURED.with_borrow_mut(|out| {
        *out = resources
            .into_iter()
            .map(|r| (r.description.format, r.description.usage, r.state))
            .collect()
    });
}
fn invalid(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
    ))
}

macro_rules! exercise {
    ($runtime:expr, $owner:ty) => {{
        let (mut runtime, observer) = $runtime;
        // SAFETY: These exact exports belong to the paired controlled fixture.
        let (observe, calls, code) = unsafe {
            (
                *observer
                    .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                        c"fixture_set_dispatch_observer",
                    )
                    .unwrap(),
                *observer
                    .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                    .unwrap(),
                *observer
                    .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                    .unwrap(),
            )
        };
        observe(Some(capture));
        let scene = Scene::new();
        let device = scene.device_object();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let references = scene.reference_counts();
        let support1 = device.format_support.get();
        let support2 = device.format_support2.get();
        let before = device.format_query_calls.get();
        assert_result!(
            {
                let recording = &scene.input();
                context.validate_upscale_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        assert_eq!(device.format_query_calls.get(), before);

        for (index, role, candidates) in [
            (0, ResourceRole::Color, COLORS.to_vec()),
            (
                1,
                ResourceRole::Depth,
                vec![(DXGI_FORMAT_R16_FLOAT, FFX_API_SURFACE_FORMAT_R16_FLOAT)],
            ),
            (4, ResourceRole::Output, COLORS.to_vec()),
        ] {
            let object = scene.resource_object(index);
            let baseline = object.desc.get();
            for (format, native) in candidates {
                let mut desc = baseline;
                desc.Format = format;
                object.desc.set(desc);
                device.format_queries.borrow_mut().clear();
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                assert_eq!(&*device.format_queries.borrow(), &[format, format]);
                CAPTURED.with_borrow(|c| {
                    assert_eq!(
                        c[index],
                        (
                            native,
                            if index == 4 { 2 } else { 0 },
                            if index == 4 { 2 } else { 4 }
                        )
                    )
                });
                let native_calls = calls().calls;
                let expected = Err(crate::error::test_inspection(
                    InspectionObject::ContextDeviceForResource(role),
                    InspectionOperation::CheckFormatSupport,
                    GET_DEVICE_FAILURE.0,
                ));
                device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                device.format_query_failure.set(None);
                let bits = if index == 4 {
                    vec![
                        D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                        D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                        D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                    ]
                } else {
                    desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
                    object.desc.set(desc);
                    vec![
                        D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                        D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                        D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
                        D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                    ]
                };
                for bit in bits {
                    device.format_support.set(support1 & !bit);
                    let expected = Err(invalid(role, ResourceProperty::FormatSupport));
                    assert_result!(
                        {
                            let recording = &scene.input();
                            context.validate_upscale_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        expected
                    );
                    assert_result!(
                        unsafe {
                            let recording = scene.input();
                            context.record(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        expected
                    );
                }
                device.format_support.set(support1);
                device.format_support2.set(Default::default());
                if index == 4 {
                    let expected = Err(invalid(role, ResourceProperty::FormatSupport));
                    assert_result!(
                        {
                            let recording = &scene.input();
                            context.validate_upscale_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        expected
                    );
                    assert_result!(
                        unsafe {
                            let recording = scene.input();
                            context.record(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        expected
                    );
                } else {
                    assert_result!(
                        {
                            let recording = &scene.input();
                            context.validate_upscale_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        Ok(())
                    );
                }
                assert_eq!(calls().calls, native_calls);
                device.format_support2.set(support2);
                desc.Flags |= D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
                object.desc.set(desc);
                let before = device.format_query_calls.get();
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::ShaderReadable))
                );
                assert_eq!(device.format_query_calls.get(), before);
                desc.Flags &= !D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
                object.desc.set(desc);
                if index == 4 {
                    device
                        .format_support
                        .set(support1 & !D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE);
                } else {
                    desc.Flags = D3D12_RESOURCE_FLAG_NONE;
                    object.desc.set(desc);
                    device
                        .format_support
                        .set(support1 & !D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW);
                }
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                device.format_support.set(support1);
                // Metadata and foreign devices reject before the format query.
                desc.Width += 1;
                object.desc.set(desc);
                let before = device.format_query_calls.get();
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::Extent))
                );
                desc.Width -= 1;
                object.desc.set(desc);
                object.set_device(scene.foreign_device.as_raw().cast());
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::Device))
                );
                assert_eq!(device.format_query_calls.get(), before);
                object.set_device(scene.device.as_raw().cast());
            }
            object.desc.set(baseline);
        }

        // Every new format rejects in each unselected production role.
        for (index, role) in [
            (0, ResourceRole::Color),
            (1, ResourceRole::Depth),
            (2, ResourceRole::MotionVectors),
            (3, ResourceRole::Exposure),
            (4, ResourceRole::Output),
        ] {
            let object = scene.resource_object(index);
            let baseline = object.desc.get();
            for (format, _) in COLORS
                .into_iter()
                .chain([(DXGI_FORMAT_R16_FLOAT, FFX_API_SURFACE_FORMAT_R16_FLOAT)])
            {
                if (index == 0 || index == 4) && format != DXGI_FORMAT_R16_FLOAT
                    || index == 1 && format == DXGI_FORMAT_R16_FLOAT
                {
                    continue;
                }
                let mut desc = baseline;
                desc.Format = format;
                object.desc.set(desc);
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::Format))
                );
            }
            object.desc.set(baseline);
        }

        let mut originals = Vec::new();
        for (index, format) in [
            (0, DXGI_FORMAT_R8G8B8A8_UNORM),
            (1, DXGI_FORMAT_R16_FLOAT),
            (4, DXGI_FORMAT_R32G32B32A32_FLOAT),
        ] {
            let object = scene.resource_object(index);
            let original = object.desc.get();
            originals.push((index, original));
            let mut d = original;
            d.Format = format;
            object.desc.set(d);
        }
        device.format_queries.borrow_mut().clear();
        assert_result!(
            {
                let recording = &scene.input();
                context.validate_upscale_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        assert_eq!(
            &*device.format_queries.borrow(),
            &[
                DXGI_FORMAT_R8G8B8A8_UNORM,
                DXGI_FORMAT_R16_FLOAT,
                DXGI_FORMAT_R32G32B32A32_FLOAT
            ]
        );
        for (index, original) in originals {
            scene.resource_object(index).desc.set(original);
        }

        // Independent colour storage, not an alias of the opaque input.
        let full = scene.mask();
        let mask = scene.mask();
        let full_object = com::object_from(&full);
        let mask_object = com::object_from(&mask);
        let mut full_desc = scene.resource_object(0).desc.get();
        full_object.desc.set(full_desc);
        let mut mask_desc = mask_object.desc.get();
        mask_desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
        mask_object.desc.set(mask_desc);
        let generator = || {
            ReactiveMaskTestInputs::new(
                &scene.list,
                &scene.resources[0],
                &full,
                &mask,
                Dimensions::new(1280, 720).unwrap(),
                ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap(),
            )
        };
        for (format, _) in COLORS
            .into_iter()
            .chain([(DXGI_FORMAT_R16_FLOAT, FFX_API_SURFACE_FORMAT_R16_FLOAT)])
        {
            let mut wrong = mask_desc;
            wrong.Format = format;
            mask_object.desc.set(wrong);
            let expected = Err(invalid(
                ResourceRole::ReactiveMaskOutput,
                ResourceProperty::Format,
            ));
            assert_result!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                expected
            );
            assert_result!(
                unsafe {
                    let recording = generator();
                    context.record_reactive_mask(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                expected
            );
            mask_object.desc.set(mask_desc);
            for role in [
                ResourceRole::ReactiveMask,
                ResourceRole::TransparencyAndCompositionMask,
            ] {
                let mut input = scene.input();
                mask_object.desc.set(wrong);
                let mut bundle = input.resources().clone();
                if role == ResourceRole::ReactiveMask {
                    bundle.set_reactive_mask(Some(&mask));
                } else {
                    bundle.set_transparency_and_composition_mask(Some(&mask));
                }
                input = UpscaleTestInputs::new(
                    &scene.list,
                    bundle,
                    JitterOffsetPixels::new(0.0, 0.0).unwrap(),
                    FrameTimeMillis::new(16.0).unwrap(),
                    CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(),
                    true,
                );
                assert_result!(
                    context.validate_upscale_inputs(
                        input.command_list,
                        &input.resources,
                        &input.parameters
                    ),
                    Err(invalid(role, ResourceProperty::Format))
                );
                mask_object.desc.set(mask_desc);
            }
        }
        for (object, role) in [
            (scene.resource_object(0), ResourceRole::OpaqueOnlyColor),
            (full_object, ResourceRole::Color),
        ] {
            let baseline = object.desc.get();
            let mut wrong = baseline;
            wrong.Format = DXGI_FORMAT_R16_FLOAT;
            object.desc.set(wrong);
            assert_result!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(role, ResourceProperty::Format))
            );
            object.desc.set(baseline);
        }
        for (slot, role) in [(0, ResourceRole::OpaqueOnlyColor), (1, ResourceRole::Color)] {
            let object = if slot == 0 {
                scene.resource_object(0)
            } else {
                full_object
            };
            let baseline = object.desc.get();
            for (format, native) in COLORS {
                let mut d = baseline;
                d.Format = format;
                object.desc.set(d);
                assert_result!(
                    {
                        let recording = &generator();
                        context.validate_reactive_mask_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                assert_result!(
                    unsafe {
                        let recording = generator();
                        context.record_reactive_mask(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                CAPTURED.with_borrow(|c| assert_eq!(c[slot], (native, 0, 4)));
                device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
                let expected = Err(crate::error::test_inspection(
                    InspectionObject::ContextDeviceForResource(role),
                    InspectionOperation::CheckFormatSupport,
                    GET_DEVICE_FAILURE.0,
                ));
                let before = calls().calls;
                assert_result!(
                    {
                        let recording = &generator();
                        context.validate_reactive_mask_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                assert_result!(
                    unsafe {
                        let recording = generator();
                        context.record_reactive_mask(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                assert_eq!(calls().calls, before);
                device.format_query_failure.set(None);
                d.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
                object.desc.set(d);
                for bit in [
                    D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                    D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                    D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
                    D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                ] {
                    device.format_support.set(support1 & !bit);
                    assert_result!(
                        {
                            let recording = &generator();
                            context.validate_reactive_mask_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        Err(invalid(role, ResourceProperty::FormatSupport))
                    );
                    assert_result!(
                        unsafe {
                            let recording = generator();
                            context.record_reactive_mask(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        },
                        Err(invalid(role, ResourceProperty::FormatSupport))
                    );
                    assert_eq!(calls().calls, before);
                }
                device.format_support.set(support1);
                device.format_support2.set(Default::default());
                assert_result!(
                    unsafe {
                        let recording = generator();
                        context.record_reactive_mask(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
                device.format_support2.set(support2);
            }
            object.desc.set(baseline);
        }
        // Fixed role order identifies the first failing new pair, never an
        // anonymous context-device failure inferred from call ordering.
        full_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        full_object.desc.set(full_desc);
        let color = scene.resource_object(0);
        let mut d = color.desc.get();
        d.Format = DXGI_FORMAT_R32G32B32A32_FLOAT;
        color.desc.set(d);
        device.format_queries.borrow_mut().clear();
        assert_result!(
            {
                let recording = &generator();
                context.validate_reactive_mask_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        assert_eq!(
            &*device.format_queries.borrow(),
            &[DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM]
        );
        d.Format = DXGI_FORMAT_R16G16B16A16_FLOAT;
        color.desc.set(d);
        for (state, error) in [
            (
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                crate::test_errors::terminal(Error::native(crate::error::Operation::Upscale, 3)),
            ),
            (
                crate::context_policy::failed(crate::error::Operation::Tuning, 3),
                crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 3)),
            ),
        ] {
            context.core.eligibility = state;
            let before = device.format_query_calls.get();
            assert_result!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(error)
            );
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(error)
            );
            assert_eq!(device.format_query_calls.get(), before);
        }
        context.core.eligibility = Eligibility::Usable;
        code(0x8234);
        assert_result!(
            unsafe {
                let recording = scene.input();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::native(crate::error::Operation::Upscale, 0x8234))
        );
        assert_result!(
            unsafe {
                let recording = scene.input();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                0x8234
            )))
        );
        drop(full);
        drop(mask);
        assert_eq!(scene.reference_counts(), references);
        context.destroy().unwrap();
    }};
}

#[test]
fn local_typed_format_roles_views_and_failures() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_typed_format_roles_views_and_failures() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
