// SPDX-License-Identifier: MPL-2.0

use super::{
    com::{Scene, object_from},
    *,
};
use crate::{
    error::{
        DispatchValidation, InspectionObject, InspectionOperation, Operation, ResourceProperty,
        ResourceRole,
    },
    upscaler::ReactiveMaskOptions,
};
use std::{cell::Cell, ffi::c_void};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResourceObservation {
    pointer: usize,
    metadata: [u32; 9],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Observation {
    list: usize,
    resources: [ResourceObservation; 3],
    render: [u32; 2],
    scalars: [u32; 3],
    flags: u32,
    next_null: bool,
}
thread_local! { static CAPTURED: Cell<Option<Observation>> = const { Cell::new(None) }; }
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Registered only for this fixture's complete dispatch descriptors.
    let header = unsafe { &*header };
    if header.r#type != FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK {
        return;
    }
    // SAFETY: This tag is produced only by the complete production generator path.
    let d = unsafe {
        &*(std::ptr::from_ref(header).cast::<ffxDispatchDescUpscaleGenerateReactiveMask>())
    };
    CAPTURED.set(Some(Observation {
        list: d.commandList as usize,
        resources: [d.colorOpaqueOnly, d.colorPreUpscale, d.outReactive].map(|r| {
            ResourceObservation {
                pointer: r.resource as usize,
                // SAFETY: All resources are Texture2D descriptions with dimension members.
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
            }
        }),
        render: [d.renderSize.width, d.renderSize.height],
        scalars: [
            d.scale.to_bits(),
            d.cutoffThreshold.to_bits(),
            d.binaryValue.to_bits(),
        ],
        flags: d.flags,
        next_null: d.header.pNext.is_null(),
    }));
}
unsafe extern "C" fn configure_error(_: *mut *mut c_void, _: *const c_void) -> u32 {
    0xa030
}
fn counts(observer: &libloading::Library) -> [u32; 3] {
    // SAFETY: Exact matched fixture counter signatures.
    unsafe {
        [
            observer
                .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
                .unwrap()(),
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
            .calls,
            observer
                .get::<extern "C" fn() -> u32>(c"fixture_configure_calls")
                .unwrap()(),
        ]
    }
}
fn set_code(observer: &libloading::Library, code: u32) {
    // SAFETY: Matched fixture setter.
    unsafe {
        observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap()(code);
    }
}
fn options(flags: u32) -> ReactiveMaskOptions {
    let mut options = ReactiveMaskOptions::new(-0.0, -2.0, 2.0).unwrap();
    options.set_apply_tonemap(flags & 1 != 0);
    options.set_apply_inverse_tonemap(flags & 2 != 0);
    options.set_apply_threshold(flags & 4 != 0);
    options.set_use_components_max(flags & 8 != 0);
    options
}
fn request<'a>(
    scene: &'a Scene,
    full: &'a ID3D12Resource,
    output: &'a ID3D12Resource,
    region: Dimensions,
    flags: u32,
) -> ReactiveMaskTestInputs<'a> {
    ReactiveMaskTestInputs::new(
        &scene.list,
        &scene.resources[0],
        full,
        output,
        region,
        options(flags),
    )
}
fn resource_error(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
    ))
}
macro_rules! reject {
    ($context:expr,$request:expr,$expected:expr,$observer:expr,$scene:expr) => {{
        let before = counts(&$observer);
        let references = $scene.reference_counts();
        let dispatch = $request;
        assert_result!(
            $context.validate_reactive_mask_inputs(
                dispatch.command_list,
                &dispatch.resources,
                &dispatch.parameters
            ),
            Err($expected)
        );
        // SAFETY: Controlled fixture; invalid metadata rejects before FidelityFX.
        assert_result!(
            unsafe {
                $context.record_reactive_mask(
                    dispatch.command_list,
                    &dispatch.resources,
                    &dispatch.parameters,
                )
            },
            Err($expected)
        );
        assert_eq!(counts(&$observer), before);
        assert_eq!($scene.reference_counts(), references);
        assert_eq!($context.core.eligibility, Eligibility::Usable);
    }};
}
macro_rules! exercise {
    ($runtime:expr, $family:ty) => {{
        let (mut runtime, observer) = $runtime;
        // SAFETY: Exact fixture callback export; synchronously copies observations.
        unsafe {
            observer
                .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                    c"fixture_set_dispatch_observer",
                )
                .unwrap()(Some(capture));
        }
        let scene = Scene::new();
        let full = scene.mask();
        let original = object_from(&full).desc.get();
        object_from(&full).desc.set(D3D12_RESOURCE_DESC {
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            ..original
        });
        let mask = scene.mask();
        let original = object_from(&mask).desc.get();
        object_from(&mask).desc.set(D3D12_RESOURCE_DESC {
            Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            ..original
        });
        let region = Dimensions::new(960, 540).unwrap();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let references = scene.reference_counts();
        for flags in 0..16 {
            for size in [(1280, 720), (960, 540), (1, 1), (1280, 720)] {
                for color in [&full, &scene.resources[0]] {
                    let request = request(
                        &scene,
                        color,
                        &mask,
                        Dimensions::new(size.0, size.1).unwrap(),
                        flags,
                    );
                    let references = scene.reference_counts();
                    let before = counts(&observer);
                    let cloned = request.clone();
                    assert_eq!(cloned.command_list().as_raw(), scene.list.as_raw());
                    assert_eq!(
                        cloned.color_opaque_only().as_raw(),
                        scene.resources[0].as_raw()
                    );
                    assert_eq!(cloned.color_pre_upscale().as_raw(), color.as_raw());
                    assert_eq!(cloned.output().as_raw(), mask.as_raw());
                    assert_eq!(cloned.render_size(), request.render_size());
                    assert_eq!(cloned.options().scale().to_bits(), (-0.0f32).to_bits());
                    assert_result!(
                        context.validate_reactive_mask_inputs(
                            request.command_list,
                            &request.resources,
                            &request.parameters
                        ),
                        Ok(())
                    );
                    assert_eq!(counts(&observer), before);
                    assert_eq!(scene.reference_counts(), references);
                    assert_eq!(object_from(&mask).references.get(), 1);
                    assert_eq!(object_from(&full).references.get(), 1);
                    // SAFETY: Fixture observes complete descriptors, records no GPU work.
                    unsafe {
                        context.record_reactive_mask(
                            request.command_list,
                            &request.resources,
                            &request.parameters,
                        )
                    }
                    .unwrap();
                    assert_eq!(counts(&observer), [before[0], before[1] + 1, before[2]]);
                    let seen = CAPTURED.take().unwrap();
                    assert!(seen.next_null);
                    assert_eq!(seen.list, scene.list.as_raw() as usize);
                    assert_eq!(seen.flags, flags);
                    assert_eq!(seen.render, [size.0, size.1]);
                    assert_eq!(
                        seen.scalars,
                        [(-0.0f32).to_bits(), (-2.0f32).to_bits(), 2.0f32.to_bits()]
                    );
                    for (index, resource) in
                        [&scene.resources[0], color, &mask].into_iter().enumerate()
                    {
                        assert_eq!(
                            seen.resources[index],
                            ResourceObservation {
                                pointer: resource.as_raw() as usize,
                                metadata: [
                                    2,
                                    if index == 2 { 25 } else { 4 },
                                    1280,
                                    720,
                                    1,
                                    1,
                                    0,
                                    if index == 2 { 2 } else { 0 },
                                    if index == 2 { 2 } else { 4 }
                                ]
                            }
                        );
                    }
                    assert_eq!(scene.reference_counts(), references);
                }
            }
        }
        reject!(
            context,
            request(&scene, &full, &mask, Dimensions::new(1281, 720).unwrap(), 0),
            Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveRenderSize
            )),
            observer,
            scene
        );
        reject!(
            context,
            request(&scene, &full, &mask, Dimensions::new(1280, 721).unwrap(), 0),
            Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveRenderSize
            )),
            observer,
            scene
        );
        for (resource, role) in [
            (&scene.resources[0], ResourceRole::OpaqueOnlyColor),
            (&full, ResourceRole::Color),
            (&mask, ResourceRole::ReactiveMaskOutput),
        ] {
            let object = object_from(resource);
            let valid = object.desc.get();
            for (desc, property) in [
                (
                    D3D12_RESOURCE_DESC {
                        Format: DXGI_FORMAT_R32_FLOAT,
                        ..valid
                    },
                    ResourceProperty::Format,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        Width: 1279,
                        ..valid
                    },
                    ResourceProperty::Extent,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        Height: 719,
                        ..valid
                    },
                    ResourceProperty::Extent,
                ),
                (
                    D3D12_RESOURCE_DESC { Width: 0, ..valid },
                    ResourceProperty::Extent,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        Width: u64::from(u32::MAX) + 1,
                        ..valid
                    },
                    ResourceProperty::Extent,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                        ..valid
                    },
                    ResourceProperty::TextureShape,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        MipLevels: 2,
                        ..valid
                    },
                    ResourceProperty::TextureShape,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        DepthOrArraySize: 2,
                        ..valid
                    },
                    ResourceProperty::TextureShape,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 2,
                            Quality: 0,
                        },
                        ..valid
                    },
                    ResourceProperty::TextureShape,
                ),
                (
                    D3D12_RESOURCE_DESC {
                        Flags: D3D12_RESOURCE_FLAGS(
                            valid.Flags.0 | D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE.0,
                        ),
                        ..valid
                    },
                    ResourceProperty::ShaderReadable,
                ),
            ] {
                object.desc.set(desc);
                reject!(
                    context,
                    request(&scene, &full, &mask, region, 0),
                    resource_error(role, property),
                    observer,
                    scene
                );
            }
            object.desc.set(valid);
            object.query_failure.set(Some(com::QUERY_FAILURE));
            reject!(
                context,
                request(&scene, &full, &mask, region, 0),
                crate::error::test_inspection(
                    InspectionObject::Resource(role),
                    InspectionOperation::QueryIdentity,
                    com::QUERY_FAILURE.0
                ),
                observer,
                scene
            );
            object.query_failure.set(None);
            object.get_device_failure.set(Some(com::GET_DEVICE_FAILURE));
            reject!(
                context,
                request(&scene, &full, &mask, region, 0),
                crate::error::test_inspection(
                    InspectionObject::Resource(role),
                    InspectionOperation::GetDevice,
                    com::GET_DEVICE_FAILURE.0
                ),
                observer,
                scene
            );
            object.get_device_failure.set(None);
            let original_device = object.device.get();
            object.set_device(scene.foreign_device.as_raw().cast());
            reject!(
                context,
                request(&scene, &full, &mask, region, 0),
                resource_error(role, ResourceProperty::Device),
                observer,
                scene
            );
            object.set_device(original_device);
        }
        let valid = object_from(&mask).desc.get();
        object_from(&mask).desc.set(D3D12_RESOURCE_DESC {
            Flags: D3D12_RESOURCE_FLAG_NONE,
            ..valid
        });
        reject!(
            context,
            request(&scene, &full, &mask, region, 0),
            resource_error(
                ResourceRole::ReactiveMaskOutput,
                ResourceProperty::UnorderedAccess
            ),
            observer,
            scene
        );
        object_from(&mask).desc.set(valid);
        for input in [&scene.resources[0], &full] {
            object_from(&mask).set_canonical(input.as_raw().cast());
            reject!(
                context,
                request(&scene, &full, &mask, region, 0),
                resource_error(
                    ResourceRole::ReactiveMaskOutput,
                    ResourceProperty::DistinctIdentity
                ),
                observer,
                scene
            );
            object_from(&mask).set_canonical(mask.as_raw().cast());
        }
        scene
            .list_object()
            .list_type
            .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
        reject!(
            context,
            request(&scene, &full, &mask, region, 0),
            Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::CommandListType
            )),
            observer,
            scene
        );
        scene
            .list_object()
            .list_type
            .set(D3D12_COMMAND_LIST_TYPE_DIRECT);
        scene
            .list_object()
            .set_device(scene.foreign_device.as_raw().cast());
        reject!(
            context,
            request(&scene, &full, &mask, region, 0),
            Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::CommandListDevice
            )),
            observer,
            scene
        );
        scene.list_object().set_device(scene.device.as_raw().cast());
        // A successful preflight cannot cache metadata or skip reinspection.
        assert_result!(
            {
                let recording = &request(&scene, &full, &mask, region, 0);
                context.validate_reactive_mask_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        object_from(&mask).desc.set(D3D12_RESOURCE_DESC {
            Format: DXGI_FORMAT_R32_FLOAT,
            ..valid
        });
        reject!(
            context,
            request(&scene, &full, &mask, region, 0),
            resource_error(ResourceRole::ReactiveMaskOutput, ResourceProperty::Format),
            observer,
            scene
        );
        object_from(&mask).desc.set(valid);
        let mut context = *Box::new(context);
        // The same generated resource is accepted by the ordinary supplied-mask route.
        // SAFETY: No GPU work; real barriers/submission remain renderer obligations.
        unsafe {
            let recording = request(&scene, &full, &mask, Dimensions::new(1280, 720).unwrap(), 0);
            context.record_reactive_mask(
                recording.command_list,
                &recording.resources,
                &recording.parameters,
            )
        }
        .unwrap();
        let frame = scene.input();
        let mut resources = frame.resources().clone();
        resources.set_reactive_mask(Some(&mask));
        let frame = UpscaleTestInputs::new(
            &scene.list,
            resources,
            frame.jitter(),
            frame.frame_time(),
            frame.camera(),
            true,
        );
        unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) }.unwrap();
        context
            .request_tuning(
                TuningRequest::new(
                    TuningParameter::VelocityFactor,
                    TuningValue::ProviderDefault,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(scene.reference_counts(), references);
        context.destroy().unwrap();

        for (origin, code) in [(0, 7), (0, u32::MAX), (1, 7), (2, 0xa030)] {
            let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                .create(&sizes())
                .unwrap();
            set_code(&observer, code);
            let operation = match origin {
                2 => Operation::Tuning,
                1 => Operation::Upscale,
                _ => Operation::GenerateReactiveMask,
            };
            let expected = crate::test_errors::terminal(Error::native(operation, code));
            let result = if origin == 2 {
                // SAFETY: Matching synchronous Configure observer returns an error.
                unsafe {
                    observer
                        .get::<extern "C" fn(
                            Option<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32>,
                        )>(c"fixture_set_configure_observer")
                        .unwrap()(Some(configure_error));
                }
                context.request_tuning(
                    TuningRequest::new(
                        TuningParameter::VelocityFactor,
                        TuningValue::ProviderDefault,
                    )
                    .unwrap(),
                )
            } else if origin == 1 {
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
                    let recording = request(&scene, &full, &mask, region, 0);
                    context.record_reactive_mask(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                }
            };
            assert_result!(result, Err(Error::native(operation, code)));
            let before = counts(&observer);
            scene
                .list_object()
                .query_failure
                .set(Some(com::QUERY_FAILURE));
            scene
                .list_object()
                .list_type
                .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
            assert_result!(
                {
                    let recording = &request(
                        &scene,
                        &full,
                        &mask,
                        Dimensions::new(u32::MAX, 1).unwrap(),
                        0,
                    );
                    context.validate_reactive_mask_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(expected)
            );
            assert_result!(
                unsafe {
                    let recording = request(&scene, &full, &mask, region, 0);
                    context.record_reactive_mask(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(expected)
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
                Err(expected)
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
                Err(expected)
            );
            assert_result!(
                context.request_tuning(
                    TuningRequest::new(TuningParameter::VelocityFactor, TuningValue::Value(-0.0))
                        .unwrap()
                ),
                Err(expected)
            );
            assert_result!(context.provider(), Err(expected));
            assert_result!(context.resource_requirements(), Err(expected));
            assert_result!(context.gpu_memory_usage(), Err(expected));
            assert_result!(context.upscale_ratio(QualityMode::Quality), Err(expected));
            assert_result!(
                context.render_size_for_quality(region, QualityMode::Quality),
                Err(expected)
            );
            assert_result!(context.jitter_phase_count(region, region), Err(expected));
            assert_result!(
                context.jitter_offset(-1, JitterPhaseCount::new(1).unwrap()),
                Err(expected)
            );
            assert_eq!(counts(&observer), before);
            scene.list_object().query_failure.set(None);
            scene
                .list_object()
                .list_type
                .set(D3D12_COMMAND_LIST_TYPE_DIRECT);
            context.destroy().unwrap();
            set_code(&observer, 0);
            // SAFETY: Exact fixture export; remove the failed Configure callback.
            unsafe {
                observer
                    .get::<extern "C" fn(
                        Option<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32>,
                    )>(c"fixture_set_configure_observer")
                    .unwrap()(None);
            }
        }
        // Generation failure preserves the existing single-attempt owner and
        // retains its critical bundle if terminal destruction fails.
        let before = fixture_snapshot(&observer).destroys;
        let references = scene.device_object().references.get();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        set_code(&observer, 7);
        // SAFETY: Controlled fixture; native rejection records no GPU work.
        assert_result!(
            unsafe {
                let recording = request(&scene, &full, &mask, region, 0);
                context.record_reactive_mask(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::native(Operation::GenerateReactiveMask, 7))
        );
        fixture_set(&observer, 0, u32::MAX, 0);
        assert_result!(
            context.destroy(),
            Err(Error::native(Operation::Destroy, u32::MAX))
        );
        assert_eq!(fixture_snapshot(&observer).destroys, before + 1);
        assert_eq!(scene.device_object().references.get(), references + 1);
    }};
}

#[test]
fn local_generator_mapping_validation_composition_and_terminal_errors() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_generator_mapping_validation_composition_and_terminal_errors() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
