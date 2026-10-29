// SPDX-License-Identifier: MPL-2.0

use super::{
    com::{Scene, object_from},
    *,
};
use crate::{
    error::{
        DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
    },
    upscaler::ExposureMode,
};
use std::{cell::Cell, ffi::c_void};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MaskObservation {
    pointer: usize,
    metadata: [u32; 9],
}
type Observation = ([MaskObservation; 2], [u32; 2], bool);
thread_local! {
    static REQUIRED: Cell<u64> = const { Cell::new(15) };
    static OPTIONAL: Cell<u64> = const { Cell::new(48) };
    static CAPTURED: Cell<Option<Observation>> = const { Cell::new(None) };
}
unsafe extern "C" fn query(context: *mut *mut c_void, desc: *mut c_void) -> u32 {
    // SAFETY: Registered only for complete synchronous live requirement queries.
    unsafe {
        if context.is_null() || (*context).is_null() {
            return 0xf001;
        }
        let d = &mut *desc.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
        if d.header.r#type != FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS {
            return 0xf002;
        }
        d.required_resources = REQUIRED.get();
        d.optional_resources = OPTIONAL.get();
    }
    0
}
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: The fixture passes a complete descriptor, valid only during this call.
    let d = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    let masks = [d.reactive, d.transparencyAndComposition].map(|r| MaskObservation {
        pointer: r.resource as usize,
        // SAFETY: All three unions contain their texture dimensions.
        metadata: unsafe {
            [
                r.description.r#type,
                r.description.format,
                r.description.width.width,
                r.description.height.height,
                r.description.depth.depth,
                r.description.mipCount,
                r.description.flags,
                r.description.usage,
                r.state,
            ]
        },
    });
    CAPTURED.set(Some((
        masks,
        [d.renderSize.width, d.renderSize.height],
        d.reset,
    )));
}
fn register(observer: &libloading::Library) {
    REQUIRED.set(15);
    OPTIONAL.set(48);
    // SAFETY: Matching controlled fixture ABI; callbacks retain no addresses.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(Some(query));
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
}
fn calls(observer: &libloading::Library) -> (u32, u32) {
    // SAFETY: Exact matching fixture exports.
    unsafe {
        (
            observer
                .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
                .unwrap()(),
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
            .calls,
        )
    }
}
fn input<'a>(
    scene: &'a Scene,
    masks: [&'a ID3D12Resource; 2],
    present: u64,
    automatic: bool,
) -> UpscaleTestInputs<'a> {
    let base = scene.input();
    let mut resources = if automatic {
        UpscaleResources::builder()
            .color(&scene.resources[0])
            .depth(&scene.resources[1])
            .motion_vectors(&scene.resources[2])
            .exposure(crate::upscaler::dx12::ExposureInput::Automatic)
            .output(&scene.resources[4])
            .build()
            .unwrap()
    } else {
        base.resources().clone()
    };
    assert!(resources.reactive_mask().is_none());
    assert!(resources.transparency_and_composition_mask().is_none());
    resources.set_reactive_mask(Some(masks[0]));
    resources.set_reactive_mask((present & 16 != 0).then_some(masks[0]));
    resources.set_transparency_and_composition_mask(Some(masks[1]));
    resources.set_transparency_and_composition_mask((present & 32 != 0).then_some(masks[1]));
    assert_eq!(
        resources.reactive_mask().map(Interface::as_raw),
        (present & 16 != 0).then_some(masks[0].as_raw())
    );
    assert_eq!(
        resources
            .transparency_and_composition_mask()
            .map(Interface::as_raw),
        (present & 32 != 0).then_some(masks[1].as_raw())
    );
    UpscaleTestInputs::new(
        base.command_list(),
        resources.clone(),
        base.jitter(),
        base.frame_time(),
        base.camera(),
        false,
    )
}
fn resource_error(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
    ))
}

macro_rules! check_family {
    ($runtime:expr, $context:ty) => {{
        let (mut runtime, observer) = $runtime;
        register(&observer);
        let scene = Scene::new();
        let reactive = scene.mask();
        let composition = scene.mask();
        let masks = [&reactive, &composition];
        for automatic in [false, true] {
            for (required, optional) in [0, 16, 32, 48]
                .into_iter()
                .flat_map(|required| [0, 16, 32, 48, u64::MAX].map(|optional| (required, optional)))
            {
                REQUIRED.set(15 | required);
                OPTIONAL.set(optional);
                let mut options = sizes();
                options.set_dynamic_resolution(true);
                if automatic {
                    options.set_exposure_mode(ExposureMode::Automatic);
                }
                let before = calls(&observer);
                let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                    .create(&options)
                    .unwrap();
                assert_eq!(calls(&observer), (before.0 + 1, before.1));
                let counts = scene.reference_counts();
                for present in [0, 16, 32, 48] {
                    let before = calls(&observer);
                    let mut frame = input(&scene, masks, present, automatic);
                    frame.set_active_render_size(Some(Dimensions::new(960, 540).unwrap()));
                    CAPTURED.set(None);
                    // SAFETY: Controlled fixtures record no GPU work.
                    let result = unsafe {
                        context.record(frame.command_list, &frame.resources, &frame.parameters)
                    };
                    let missing = required & !present;
                    if missing != 0 {
                        let role = if missing & 16 != 0 {
                            ResourceRole::ReactiveMask
                        } else {
                            ResourceRole::TransparencyAndCompositionMask
                        };
                        assert_result!(
                            result,
                            Err(Error::Input(crate::error::InputError::Dispatch(
                                DispatchValidation::MissingRequiredResource(role)
                            )))
                        );
                        assert_eq!(calls(&observer), before);
                        assert!(CAPTURED.take().is_none());
                    } else {
                        result.unwrap();
                        assert_eq!(calls(&observer), (before.0, before.1 + 1));
                        let (observed, active, reset) = CAPTURED.take().unwrap();
                        assert_eq!(active, [960, 540]);
                        assert!(!reset);
                        for (index, bit) in [16, 32].into_iter().enumerate() {
                            let expected = if present & bit == 0 {
                                MaskObservation {
                                    pointer: 0,
                                    metadata: [0; 9],
                                }
                            } else {
                                MaskObservation {
                                    pointer: masks[index].as_raw() as usize,
                                    metadata: [
                                        FFX_API_RESOURCE_TYPE_TEXTURE2D,
                                        FFX_API_SURFACE_FORMAT_R8_UNORM,
                                        1280,
                                        720,
                                        1,
                                        1,
                                        0,
                                        0,
                                        FFX_API_RESOURCE_STATE_COMPUTE_READ,
                                    ],
                                }
                            };
                            assert_eq!(observed[index], expected);
                        }
                    }
                    assert_eq!(scene.reference_counts(), counts);
                    assert_eq!(object_from(&reactive).references.get(), 1);
                    assert_eq!(object_from(&composition).references.get(), 1);
                    assert_eq!(context.core.eligibility, Eligibility::Usable);
                }
                context.destroy().unwrap();
            }
        }
        REQUIRED.set(15);
        OPTIONAL.set(0);
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let before = calls(&observer);
        let counts = scene.reference_counts();
        // An unlisted mask still undergoes COM inspection before native dispatch.
        object_from(&reactive)
            .query_failure
            .set(Some(com::QUERY_FAILURE));
        assert_result!(
            unsafe {
                let recording = input(&scene, masks, 16, false);
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(crate::error::test_inspection(
                InspectionObject::Resource(ResourceRole::ReactiveMask),
                InspectionOperation::QueryIdentity,
                com::QUERY_FAILURE.0
            ))
        );
        object_from(&reactive).query_failure.set(None);
        assert_eq!(calls(&observer), before);
        assert_eq!(scene.reference_counts(), counts);
        assert_eq!(object_from(&reactive).references.get(), 1);
        assert_eq!(context.core.eligibility, Eligibility::Usable);
        unsafe {
            let recording = input(&scene, masks, 48, false);
            context.record(
                recording.command_list,
                &recording.resources,
                &recording.parameters,
            )
        }
        .unwrap();
        let (observed, _, _) = CAPTURED.take().unwrap();
        assert_eq!(observed[0].pointer, reactive.as_raw() as usize);
        assert_eq!(observed[1].pointer, composition.as_raw() as usize);
        context.destroy().unwrap();

        // Resource failures and native-error poisoning apply even to unlisted masks.
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let counts = scene.reference_counts();
        for (mask, role) in [
            (masks[0], ResourceRole::ReactiveMask),
            (masks[1], ResourceRole::TransparencyAndCompositionMask),
        ] {
            let object = object_from(mask);
            let original = object.desc.get();
            for (property, desc) in [
                (
                    ResourceProperty::Format,
                    D3D12_RESOURCE_DESC {
                        Format: DXGI_FORMAT_R32_FLOAT,
                        ..original
                    },
                ),
                (
                    ResourceProperty::Extent,
                    D3D12_RESOURCE_DESC {
                        Width: 960,
                        ..original
                    },
                ),
                (
                    ResourceProperty::TextureShape,
                    D3D12_RESOURCE_DESC {
                        MipLevels: 2,
                        ..original
                    },
                ),
                (
                    ResourceProperty::ShaderReadable,
                    D3D12_RESOURCE_DESC {
                        Flags: D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
                        ..original
                    },
                ),
            ] {
                let before = calls(&observer);
                object.desc.set(desc);
                assert_result!(
                    unsafe {
                        let recording = input(&scene, masks, 48, false);
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(resource_error(role, property))
                );
                object.desc.set(original);
                assert_eq!(calls(&observer), before);
                assert_eq!(scene.reference_counts(), counts);
            }
            object.set_device(scene.foreign_device.as_raw().cast());
            assert_result!(
                unsafe {
                    let recording = input(&scene, masks, 48, false);
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(resource_error(role, ResourceProperty::Device))
            );
            object.set_device(scene.device.as_raw().cast());
            object.set_canonical(scene.resources[0].as_raw().cast());
            assert_result!(
                unsafe {
                    let recording = input(&scene, masks, 48, false);
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(resource_error(role, ResourceProperty::DistinctIdentity))
            );
            object.set_canonical(mask.as_raw().cast());
            assert_eq!(scene.reference_counts(), counts);
            unsafe {
                let recording = input(&scene, masks, 48, false);
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            }
            .unwrap();
        }
        // Native failure still poisons dispatch; no subsequent inspection/query occurs.
        unsafe {
            observer
                .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                .unwrap()(0xf1234567);
        }
        assert!(matches!(
            unsafe {
                let recording = input(&scene, masks, 48, false);
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::Native(crate::error::NativeError {
                operation: crate::error::Operation::Upscale,
                code: 0xf1234567
            }))
        ));
        let before = calls(&observer);
        assert_result!(
            unsafe {
                let recording = input(&scene, masks, 48, false);
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                0xf1234567
            )))
        );
        assert_result!(
            context.resource_requirements(),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                0xf1234567
            )))
        );
        assert_eq!(calls(&observer), before);
        context.destroy().unwrap();
    }};
}
#[test]
fn supplied_masks_mapping_requirements_inspection_retry_and_poison() {
    check_family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    check_family!(fixture_movable_runtime(), MovableUpscaler);
}
