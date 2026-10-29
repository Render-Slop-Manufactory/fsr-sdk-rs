// SPDX-License-Identifier: MPL-2.0

//! Actual allocation/view checks through both public recording families.
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

const FORMATS: [(DXGI_FORMAT, DXGI_FORMAT, u32); 12] = [
    (DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM, 14),
    (
        DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        DXGI_FORMAT_R8G8B8A8_UNORM,
        12,
    ),
    (
        DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        15,
    ),
    (
        DXGI_FORMAT_R32G32B32A32_TYPELESS,
        DXGI_FORMAT_R32G32B32A32_FLOAT,
        1,
    ),
    (
        DXGI_FORMAT_R16G16B16A16_TYPELESS,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        30,
    ),
    (
        DXGI_FORMAT_R10G10B10A2_TYPELESS,
        DXGI_FORMAT_R10G10B10A2_UNORM,
        32,
    ),
    (DXGI_FORMAT_R8G8B8A8_TYPELESS, DXGI_FORMAT_R8G8B8A8_UNORM, 9),
    (
        DXGI_FORMAT_B8G8R8A8_TYPELESS,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        13,
    ),
    (DXGI_FORMAT_R8G8B8A8_SNORM, DXGI_FORMAT_R8G8B8A8_SNORM, 11),
    (
        DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
        DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
        29,
    ),
    (DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32B32_FLOAT, 5),
    (
        DXGI_FORMAT_R32G32B32_TYPELESS,
        DXGI_FORMAT_R32G32B32_FLOAT,
        5,
    ),
];
thread_local! { static CAPTURED: RefCell<Vec<(u32,u32,u32)>> = const { RefCell::new(Vec::new()) }; }
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Paired controlled fixture supplies the descriptor for this tag;
    // only copied metadata is retained, never native resource borrows.
    let resources = unsafe {
        if (*header).r#type == FFX_API_DISPATCH_DESC_TYPE_UPSCALE {
            let d = &*header.cast::<ffxDispatchDescUpscale>();
            assert_eq!(d.flags, 0);
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
        // SAFETY: Exact exports of the paired controlled fixture.
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
        let refs = scene.reference_counts();
        let full = scene.mask();
        let mask = scene.mask();
        let full_object = com::object_from(&full);
        let mask_object = com::object_from(&mask);
        full_object.desc.set(scene.resource_object(0).desc.get());
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
        let support1 = device.format_support.get();
        let support2 = device.format_support2.get();
        for op in 0..4 {
            let (object, role, slot) = match op {
                0 => (scene.resource_object(0), ResourceRole::Color, 0),
                1 => (scene.resource_object(4), ResourceRole::Output, 4),
                2 => (scene.resource_object(0), ResourceRole::OpaqueOnlyColor, 0),
                _ => (full_object, ResourceRole::Color, 1),
            };
            macro_rules! validate {
                () => {
                    if op < 2 {
                        {
                            let recording = &scene.input();
                            context.validate_upscale_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    } else {
                        {
                            let recording = &generator();
                            context.validate_reactive_mask_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    }
                };
            }
            macro_rules! record {
                () => {
                    if op < 2 {
                        unsafe {
                            let recording = scene.input();
                            context.record(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    } else {
                        unsafe {
                            let recording = generator();
                            context.record_reactive_mask(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    }
                };
            }
            let original = object.desc.get();
            for (allocation, uav, native) in FORMATS {
                let srgb = matches!(
                    allocation,
                    DXGI_FORMAT_R8G8B8A8_UNORM_SRGB | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB
                );
                let srv = if srgb { allocation } else { uav };
                let rgb32 = srv == DXGI_FORMAT_R32G32B32_FLOAT;
                if rgb32 && op == 1 {
                    let mut desc = original;
                    desc.Format = allocation;
                    object.desc.set(desc);
                    let before = calls().calls;
                    assert_result!(validate!(), Err(invalid(role, ResourceProperty::Format)));
                    assert_result!(record!(), Err(invalid(role, ResourceProperty::Format)));
                    assert_eq!(calls().calls, before);
                    continue;
                }
                for has_uav in [false, true] {
                    if op == 1 && !has_uav {
                        continue;
                    }
                    if rgb32 && has_uav {
                        let mut desc = original;
                        desc.Format = allocation;
                        desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
                        object.desc.set(desc);
                        let before = calls().calls;
                        assert_result!(
                            validate!(),
                            Err(invalid(role, ResourceProperty::UnorderedAccess))
                        );
                        assert_result!(
                            record!(),
                            Err(invalid(role, ResourceProperty::UnorderedAccess))
                        );
                        assert_eq!(calls().calls, before);
                        continue;
                    }
                    let mut desc = original;
                    desc.Format = allocation;
                    desc.Flags = if has_uav {
                        D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
                    } else {
                        D3D12_RESOURCE_FLAG_NONE
                    };
                    object.desc.set(desc);
                    // Real sRGB views expose no typed UAV support. A query on
                    // the allocation instead of the UNORM UAV must fail here.
                    let srv_support = if op == 1 || rgb32 {
                        support1 & !D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE
                    } else {
                        support1
                    };
                    let profiles = if srgb {
                        vec![
                            (
                                srv,
                                srv_support & !D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                                D3D12_FORMAT_SUPPORT2::default(),
                            ),
                            (
                                uav,
                                D3D12_FORMAT_SUPPORT1_TEXTURE2D
                                    | D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                                support2,
                            ),
                        ]
                    } else {
                        vec![(srv, srv_support, support2)]
                    };
                    *device.format_support_overrides.borrow_mut() = profiles.clone();
                    device.format_queries.borrow_mut().clear();
                    let casts = device.casting_query_calls.get();
                    assert_result!(validate!(), Ok(()));
                    assert_result!(record!(), Ok(()));
                    let mut queries = vec![srv];
                    if srgb && has_uav {
                        queries.push(uav);
                    }
                    assert_eq!(&*device.format_queries.borrow(), &queries.repeat(2));
                    assert_eq!(
                        device.casting_query_calls.get() - casts,
                        if srgb && has_uav { 2 } else { 0 }
                    );
                    CAPTURED.with_borrow(|v| {
                        assert_eq!(
                            v[slot],
                            (
                                native,
                                if has_uav { 2 } else { 0 },
                                if op == 1 { 2 } else { 4 }
                            )
                        )
                    });
                    let native_calls = calls().calls;
                    for view in queries.iter().copied() {
                        device.format_fail_at.set(Some(view));
                        device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
                        let expected = Err(crate::error::test_inspection(
                            InspectionObject::ContextDeviceForResource(role),
                            InspectionOperation::CheckFormatSupport,
                            GET_DEVICE_FAILURE.0,
                        ));
                        assert_result!(validate!(), expected);
                        assert_result!(record!(), expected);
                    }
                    device.format_query_failure.set(None);
                    device.format_fail_at.set(None);
                    for view in queries {
                        let mut required = D3D12_FORMAT_SUPPORT1_TEXTURE2D;
                        if view == srv {
                            required |= D3D12_FORMAT_SUPPORT1_SHADER_LOAD;
                            if op != 1 && !rgb32 {
                                required |= D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE;
                            }
                        }
                        if view == uav && has_uav {
                            required |= D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW;
                        }
                        for bit in [
                            D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                            D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                            D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
                            D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                        ] {
                            if !required.contains(bit) {
                                continue;
                            }
                            let mut missing = profiles.clone();
                            missing.iter_mut().find(|p| p.0 == view).unwrap().1 &= !bit;
                            *device.format_support_overrides.borrow_mut() = missing;
                            let expected = Err(invalid(role, ResourceProperty::FormatSupport));
                            assert_result!(validate!(), expected);
                            assert_result!(record!(), expected);
                        }
                        *device.format_support_overrides.borrow_mut() = profiles.clone();
                    }
                    if has_uav {
                        let mut missing = profiles.clone();
                        missing.iter_mut().find(|p| p.0 == uav).unwrap().2 = Default::default();
                        *device.format_support_overrides.borrow_mut() = missing;
                        if op == 1 {
                            assert_result!(
                                validate!(),
                                Err(invalid(role, ResourceProperty::FormatSupport))
                            );
                            assert_result!(
                                record!(),
                                Err(invalid(role, ResourceProperty::FormatSupport))
                            );
                        } else {
                            assert_result!(validate!(), Ok(()));
                        }
                        *device.format_support_overrides.borrow_mut() = profiles.clone();
                    }
                    device.typed_casting.set(false);
                    device.casting_query_failure.set(Some(GET_DEVICE_FAILURE));
                    if srgb && has_uav {
                        let expected = Err(crate::error::test_inspection(
                            InspectionObject::ContextDeviceForResource(role),
                            InspectionOperation::CheckFormatCasting,
                            GET_DEVICE_FAILURE.0,
                        ));
                        device.format_queries.borrow_mut().clear();
                        assert_result!(validate!(), expected);
                        assert_result!(record!(), expected);
                        assert!(device.format_queries.borrow().is_empty());
                        device.casting_query_failure.set(None);
                        assert_result!(
                            validate!(),
                            Err(invalid(role, ResourceProperty::FormatSupport))
                        );
                        assert_result!(
                            record!(),
                            Err(invalid(role, ResourceProperty::FormatSupport))
                        );
                    } else {
                        assert_result!(validate!(), Ok(()));
                    }
                    device.typed_casting.set(true);
                    device.casting_query_failure.set(None);
                    assert_eq!(calls().calls, native_calls);
                    for property in [
                        ResourceProperty::ShaderReadable,
                        ResourceProperty::Device,
                        ResourceProperty::Extent,
                    ] {
                        let before = (
                            device.format_query_calls.get(),
                            device.casting_query_calls.get(),
                        );
                        match property {
                            ResourceProperty::ShaderReadable => {
                                desc.Flags |= D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
                                object.desc.set(desc);
                            }
                            ResourceProperty::Device => {
                                object.set_device(scene.foreign_device.as_raw().cast())
                            }
                            _ => {
                                desc.Width += 1;
                                object.desc.set(desc);
                            }
                        }
                        assert_result!(validate!(), Err(invalid(role, property)));
                        assert_result!(record!(), Err(invalid(role, property)));
                        assert_eq!(
                            (
                                device.format_query_calls.get(),
                                device.casting_query_calls.get()
                            ),
                            before
                        );
                        desc.Flags &= !D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
                        desc.Width = original.Width;
                        object.desc.set(desc);
                        object.set_device(scene.device.as_raw().cast());
                    }
                    assert_result!(record!(), Ok(())); // Inspection errors do not poison.
                    device.format_support_overrides.borrow_mut().clear();
                }
            }
            object.desc.set(original);
        }
        for (format, _, _) in FORMATS {
            for (index, role) in [
                (1, ResourceRole::Depth),
                (2, ResourceRole::MotionVectors),
                (3, ResourceRole::Exposure),
            ] {
                let object = scene.resource_object(index);
                let old = object.desc.get();
                let mut d = old;
                d.Format = format;
                object.desc.set(d);
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
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::Format))
                );
                object.desc.set(old);
            }
            let mut wrong = mask_desc;
            wrong.Format = format;
            mask_object.desc.set(wrong);
            assert_result!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(
                    ResourceRole::ReactiveMaskOutput,
                    ResourceProperty::Format
                ))
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
                Err(invalid(
                    ResourceRole::ReactiveMaskOutput,
                    ResourceProperty::Format
                ))
            );
            mask_object.desc.set(mask_desc);
            for role in [
                ResourceRole::ReactiveMask,
                ResourceRole::TransparencyAndCompositionMask,
            ] {
                mask_object.desc.set(wrong);
                let mut bundle = scene.input().resources().clone();
                if role == ResourceRole::ReactiveMask {
                    bundle.set_reactive_mask(Some(&mask));
                } else {
                    bundle.set_transparency_and_composition_mask(Some(&mask));
                }
                let input = UpscaleTestInputs::new(
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
                assert_result!(
                    unsafe {
                        context.record(input.command_list, &input.resources, &input.parameters)
                    },
                    Err(invalid(role, ResourceProperty::Format))
                );
                mask_object.desc.set(mask_desc);
            }
        }
        for format in [DXGI_FORMAT_R8G8B8A8_UINT, DXGI_FORMAT_R16G16B16A16_UNORM] {
            let object = scene.resource_object(0);
            let old = object.desc.get();
            let mut d = old;
            d.Format = format;
            object.desc.set(d);
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(ResourceRole::Color, ResourceProperty::Format))
            );
            assert_result!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(
                    ResourceRole::OpaqueOnlyColor,
                    ResourceProperty::Format
                ))
            );
            object.desc.set(old);
        }
        for state in [
            crate::context_policy::failed(crate::error::Operation::Upscale, 3),
            crate::context_policy::failed(crate::error::Operation::Tuning, 3),
        ] {
            context.core.eligibility = state;
            let before = device.format_query_calls.get();
            assert!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                }
                .is_err()
            );
            assert!(
                {
                    let recording = &generator();
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                }
                .is_err()
            );
            assert_eq!(device.format_query_calls.get(), before);
        }
        context.core.eligibility = Eligibility::Usable;
        let mut d = scene.resource_object(0).desc.get();
        d.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
        scene.resource_object(0).desc.set(d);
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
        assert_eq!(scene.reference_counts(), refs);
        context.destroy().unwrap();
    }};
}

#[test]
fn local_color_views_and_failures() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_color_views_and_failures() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
