// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::{
    error::{DispatchValidation, ResourceProperty, ResourceRole},
    upscaler::{ExposureMode, PreExposure},
};
use std::cell::Cell;

thread_local! {
    static ESTIMATE_FLAGS: Cell<Option<u32>> = const { Cell::new(None) };
    static FRAME: Cell<Option<(u32, u32, bool, bool)>> = const { Cell::new(None) };
}

unsafe extern "C" fn query(
    context: *mut *mut std::ffi::c_void,
    header: *mut std::ffi::c_void,
) -> u32 {
    // SAFETY: The controlled fixture supplies complete synchronous descriptors.
    // Only flags are copied; native output storage remains owned by the caller.
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
                d.required_resources = 15;
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
        // SAFETY: Exact fixture ABI; callbacks only inspect synchronous calls.
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
        for hdr in [false, true] {
            for automatic in [false, true] {
                let mut options = sizes();
                options.set_high_dynamic_range(hdr);
                options.set_dynamic_resolution(true);
                options.set_exposure_mode(if automatic {
                    ExposureMode::Automatic
                } else {
                    ExposureMode::Explicit
                });
                let flags = u32::from(hdr) | (u32::from(automatic) * 32) | 64;
                ESTIMATE_FLAGS.set(None);
                unsafe { runtime.upscaler_dx12(&scene.device) }
                    .estimate_memory(&options)
                    .unwrap();
                assert_eq!(ESTIMATE_FLAGS.take(), Some(flags));
                let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                    .create(&options)
                    .unwrap();
                assert_eq!(fixture_snapshot(&observer).flags, flags);
                options.set_high_dynamic_range(!hdr);
                assert_eq!(context.core.options.high_dynamic_range(), hdr);
                let counts = scene.reference_counts();
                // HDR retains the RGBA16F requirement. Rejection leaves the
                // context usable and does not reach the native observer.
                let original = scene.resource_object(0).desc.get();
                let mut wrong = original;
                wrong.Format = windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UINT;
                scene.resource_object(0).desc.set(wrong);
                FRAME.set(None);
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
                // SAFETY: Controlled COM/runtime fixtures record no GPU work.
                assert_result!(
                    unsafe {
                        let recording = input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(Error::Input(crate::error::InputError::Dispatch(
                        DispatchValidation::Resource {
                            role: ResourceRole::Color,
                            property: ResourceProperty::Format,
                        }
                    )))
                );
                assert!(FRAME.get().is_none());
                scene.resource_object(0).desc.set(original);
                for factor in [0.25, 1.0, 4.0] {
                    let mut frame = input();
                    frame.set_pre_exposure(PreExposure::new(factor).unwrap());
                    // SAFETY: Matched fixtures; all calls stay on this thread.
                    unsafe {
                        context.record(frame.command_list, &frame.resources, &frame.parameters)
                    }
                    .unwrap();
                    assert_eq!(FRAME.take(), Some((0, factor.to_bits(), automatic, false)));
                    assert_eq!(scene.reference_counts(), counts);
                }
                assert_eq!(context.core.eligibility, Eligibility::Usable);
                context.destroy().unwrap();
            }
        }
        assert_eq!(fixture_snapshot(&observer).destroys, 4);
    }};
}

#[test]
fn hdr_creation_estimate_and_dispatch_preserve_independent_controls() {
    check_family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    check_family!(fixture_movable_runtime(), MovableUpscaler);
}
