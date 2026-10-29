// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::{
    error::{DispatchValidation, Operation, ResourceProperty, ResourceRole},
    upscaler::{ColorEncodingHint, ExposureMode, InputColorSpace, PreExposure},
};
use std::cell::Cell;

thread_local! {
    static REQUIRED: Cell<u64> = const { Cell::new(15) };
    static ESTIMATE_FLAGS: Cell<Option<u32>> = const { Cell::new(None) };
    static FRAME: Cell<Option<(u32, u32, bool, bool)>> = const { Cell::new(None) };
}

unsafe extern "C" fn query(
    context: *mut *mut std::ffi::c_void,
    header: *mut std::ffi::c_void,
) -> u32 {
    // SAFETY: The controlled fixture supplies synchronous, complete descriptors.
    unsafe {
        match (*header.cast::<ffxApiHeader>()).r#type {
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2 => {
                assert!(context.is_null());
                let d = &mut *header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                ESTIMATE_FLAGS.set(Some(d.flags));
                *d.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage {
                    totalUsageInBytes: 4096,
                    aliasableUsageInBytes: 0,
                };
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS => {
                assert!(!context.is_null() && !(*context).is_null());
                let d = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                d.required_resources = REQUIRED.get();
                d.optional_resources = 48;
            }
            _ => return 0xf001,
        }
    }
    0
}

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Matched synchronous fixture ABI; no resource pointers are retained.
    let d = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    FRAME.set(Some((
        d.flags,
        d.preExposure.to_bits(),
        d.exposure.resource.is_null(),
        d.reset,
    )));
}

macro_rules! check_family {
    ($fixture:expr, $context:ty) => {{
        let (mut runtime, observer) = $fixture;
        let scene = Scene::new();
        REQUIRED.set(15);
        // SAFETY: Exact fixture signatures; callbacks only inspect synchronous calls.
        unsafe {
            observer
                .get::<extern "C" fn(
                    Option<
                        unsafe extern "C" fn(
                            *mut *mut std::ffi::c_void,
                            *mut std::ffi::c_void,
                        ) -> u32,
                    >,
                )>(c"fixture_set_query_observer")
                .unwrap()(Some(query));
            observer
                .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                    c"fixture_set_dispatch_observer",
                )
                .unwrap()(Some(capture));
        }
        for space in [InputColorSpace::Linear, InputColorSpace::NonLinear] {
            for hdr in [false, true] {
                for automatic in [false, true] {
                    let mut options = sizes();
                    options.set_input_color_space(space);
                    options.set_high_dynamic_range(hdr);
                    options.set_debug_checking(hdr);
                    options.set_debug_visualization(automatic);
                    options.set_dynamic_resolution(true);
                    options.set_exposure_mode(if automatic {
                        ExposureMode::Automatic
                    } else {
                        ExposureMode::Explicit
                    });
                    let flags = if space == InputColorSpace::NonLinear {
                        256
                    } else {
                        0
                    } | u32::from(hdr)
                        | u32::from(hdr) * 128
                        | u32::from(automatic) * 512
                        | u32::from(automatic) * 32
                        | 64;
                    ESTIMATE_FLAGS.set(None);
                    unsafe { runtime.upscaler_dx12(&scene.device) }
                        .estimate_memory(&options)
                        .unwrap();
                    assert_eq!(ESTIMATE_FLAGS.take(), Some(flags));
                    let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                        .create(&options)
                        .unwrap();
                    assert_eq!(fixture_snapshot(&observer).flags, flags);
                    options.set_input_color_space(InputColorSpace::Linear);
                    assert_eq!(context.core.options.input_color_space(), space);
                    let counts = scene.reference_counts();
                    let input = || {
                        let base = scene.input();
                        let resources = if automatic {
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
                        UpscaleTestInputs::new(
                            base.command_list(),
                            resources,
                            base.jitter(),
                            base.frame_time(),
                            base.camera(),
                            false,
                        )
                    };
                    for (hint, bits) in [
                        (ColorEncodingHint::Unspecified, 0),
                        (ColorEncodingHint::Srgb, 2),
                        (ColorEncodingHint::Pq, 4),
                    ] {
                        for allow in [false, true] {
                            let mut frame = input();
                            assert_eq!(
                                frame.color_encoding().hint(),
                                ColorEncodingHint::Unspecified
                            );
                            assert!(!frame.color_encoding().allows_context_mismatch());
                            frame.set_color_encoding(
                                crate::upscaler::ColorEncodingRequest::allow_context_mismatch(hint),
                            );
                            if !allow {
                                frame.set_color_encoding(
                                    crate::upscaler::ColorEncodingRequest::checked(hint),
                                );
                            }
                            assert_eq!(frame.color_encoding().hint(), hint);
                            assert_eq!(frame.color_encoding().allows_context_mismatch(), allow);
                            assert!(!frame.draw_debug_view());
                            frame.set_draw_debug_view(allow);
                            assert_eq!(frame.draw_debug_view(), allow);
                            frame.set_pre_exposure(PreExposure::new(0.25).unwrap());
                            FRAME.set(None);
                            // SAFETY: Controlled COM/runtime fixtures record no GPU work.
                            let result = unsafe {
                                context.record(
                                    frame.command_list,
                                    &frame.resources,
                                    &frame.parameters,
                                )
                            };
                            if !allow
                                && space == InputColorSpace::Linear
                                && hint != ColorEncodingHint::Unspecified
                            {
                                assert_result!(
                                    result,
                                    Err(Error::Input(crate::error::InputError::Dispatch(
                                        DispatchValidation::ColorEncodingContext
                                    )))
                                );
                                assert_eq!(FRAME.get(), None);
                            } else {
                                result.unwrap();
                                assert_eq!(
                                    FRAME.take(),
                                    Some((
                                        bits | u32::from(allow),
                                        0.25f32.to_bits(),
                                        automatic,
                                        false
                                    ))
                                );
                            }
                            assert_eq!(scene.reference_counts(), counts);
                        }
                    }
                    assert_eq!(context.core.eligibility, Eligibility::Usable);
                    context.destroy().unwrap();
                }
            }
        }

        REQUIRED.set(31);
        let mut masks_context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let mut no_mask = scene.input();
        no_mask.set_color_encoding(
            crate::upscaler::ColorEncodingRequest::allow_context_mismatch(ColorEncodingHint::Pq),
        );
        FRAME.set(None);
        // SAFETY: Required-mask validation must still reject the overridden frame.
        assert_result!(
            unsafe {
                masks_context.record(
                    no_mask.command_list,
                    &no_mask.resources,
                    &no_mask.parameters,
                )
            },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::MissingRequiredResource(ResourceRole::ReactiveMask)
            )))
        );
        assert_eq!(FRAME.get(), None);
        masks_context.destroy().unwrap();
        REQUIRED.set(15);
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let overridden = || {
            let mut frame = scene.input();
            frame.set_color_encoding(
                crate::upscaler::ColorEncodingRequest::allow_context_mismatch(
                    ColorEncodingHint::Pq,
                ),
            );
            frame
        };
        let counts = scene.reference_counts();
        FRAME.set(None);
        scene
            .resource_object(0)
            .query_failure
            .set(Some(com::QUERY_FAILURE));
        let mut checked = overridden();
        checked.set_color_encoding(crate::upscaler::ColorEncodingRequest::checked(
            ColorEncodingHint::Pq,
        ));
        // SAFETY: Fixtures only; validation fails before native recording.
        assert_result!(
            unsafe {
                context.record(
                    checked.command_list,
                    &checked.resources,
                    &checked.parameters,
                )
            },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ColorEncodingContext
            )))
        );
        // SAFETY: The override does not skip COM inspection.
        assert!(matches!(
            unsafe {
                let recording = overridden();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::Backend(crate::error::BackendError::Dx12Inspection(
                _
            )))
        ));
        scene.resource_object(0).query_failure.set(None);
        let original = scene.resource_object(0).desc.get();
        let mut wrong = original;
        wrong.Format = windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UINT;
        scene.resource_object(0).desc.set(wrong);
        // SAFETY: The override does not skip format validation.
        assert_result!(
            unsafe {
                let recording = overridden();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::Resource {
                    role: ResourceRole::Color,
                    property: ResourceProperty::Format
                }
            )))
        );
        scene.resource_object(0).desc.set(original);
        let base = scene.input();
        let mut missing_exposure = UpscaleTestInputs::new(
            base.command_list(),
            UpscaleResources::builder()
                .color(&scene.resources[0])
                .depth(&scene.resources[1])
                .motion_vectors(&scene.resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Automatic)
                .output(&scene.resources[4])
                .build()
                .unwrap(),
            base.jitter(),
            base.frame_time(),
            base.camera(),
            base.reset(),
        );
        missing_exposure.set_color_encoding(
            crate::upscaler::ColorEncodingRequest::allow_context_mismatch(ColorEncodingHint::Pq),
        );
        // SAFETY: The override does not skip exposure-presence validation.
        assert_result!(
            unsafe {
                context.record(
                    missing_exposure.command_list,
                    &missing_exposure.resources,
                    &missing_exposure.parameters,
                )
            },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ExposureMode
            )))
        );
        let mut oversized = overridden();
        oversized
            .set_active_render_size(Some(crate::upscaler::Dimensions::new(1281, 720).unwrap()));
        // SAFETY: The override does not skip active-size validation.
        assert_result!(
            unsafe {
                context.record(
                    oversized.command_list,
                    &oversized.resources,
                    &oversized.parameters,
                )
            },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveRenderSize
            )))
        );
        assert_eq!(FRAME.get(), None);
        assert_eq!(scene.reference_counts(), counts);
        // SAFETY: Exact controlled fixture signature; subsequent native call fails.
        unsafe {
            observer
                .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                .unwrap()(u32::MAX)
        };
        // SAFETY: Fixture records no GPU work, including on failure.
        assert_result!(
            unsafe {
                let recording = overridden();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::native(Operation::Upscale, u32::MAX))
        );
        assert_eq!(FRAME.take().unwrap().0, 4);
        // SAFETY: Poison must stop a later override before COM/native recording.
        assert_result!(
            unsafe {
                let recording = overridden();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                u32::MAX
            )))
        );
        assert_eq!(FRAME.get(), None);
        assert_eq!(scene.reference_counts(), counts);
        context.destroy().unwrap();
        assert_eq!(fixture_snapshot(&observer).destroys, 10);
    }};
}

#[test]
fn color_flags_checked_pairing_and_per_frame_override_share_both_families() {
    check_family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    check_family!(fixture_movable_runtime(), MovableUpscaler);
}
