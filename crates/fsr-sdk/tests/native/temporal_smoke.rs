// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::upscaler::{
    DepthConvention, ExposureMode, MotionVectorJitter, MotionVectorResolution, MotionVectorScale,
    PreExposure,
};

type Profile = (
    DepthConvention,
    MotionVectorScale,
    MotionVectorResolution,
    MotionVectorJitter,
);

pub(super) fn native_temporal_smoke() {
    temporal_smoke(None, ExposureMode::Explicit, false, 0, false);
}

pub(super) fn native_depth_motion_dispatch() {
    use DepthConvention::*;
    use MotionVectorJitter::*;
    use MotionVectorResolution::*;
    use MotionVectorScale::*;
    // Eight representative profiles; controlled fixtures cover every combination.
    for profile in [
        (StandardFinite, Uv, Render, Excluded),
        (StandardFinite, Pixels, Display, Included),
        (ReversedFinite, Pixels, Render, Included),
        (ReversedFinite, Uv, Display, Excluded),
        (StandardInfinite, Uv, Render, Included),
        (StandardInfinite, Pixels, Display, Excluded),
        (ReversedInfinite, Pixels, Render, Excluded),
        (ReversedInfinite, Uv, Display, Included),
    ] {
        temporal_smoke(Some(profile), ExposureMode::Explicit, false, 0, false);
    }
    eprintln!("Depth/motion dispatch: PASSED");
}

pub(super) fn native_automatic_exposure_dispatch() {
    temporal_smoke(None, ExposureMode::Automatic, false, 0, false);
    eprintln!("Automatic exposure dispatch: PASSED");
}

pub(super) fn native_active_render_dispatch() {
    for resolution in [
        MotionVectorResolution::Render,
        MotionVectorResolution::Display,
    ] {
        for units in [MotionVectorScale::Uv, MotionVectorScale::Pixels] {
            temporal_smoke(
                Some((
                    DepthConvention::StandardFinite,
                    units,
                    resolution,
                    MotionVectorJitter::Excluded,
                )),
                ExposureMode::Explicit,
                true,
                0,
                false,
            );
        }
    }
    eprintln!("Active render dispatch: PASSED");
}

pub(super) fn native_supplied_masks_dispatch() {
    for mode in [ExposureMode::Explicit, ExposureMode::Automatic] {
        for bits in [16, 32, 48] {
            temporal_smoke(None, mode, true, bits, false);
        }
    }
    eprintln!("Supplied masks dispatch: PASSED");
}

pub(super) fn native_hdr_dispatch() {
    for mode in [ExposureMode::Explicit, ExposureMode::Automatic] {
        temporal_smoke(None, mode, false, 0, true);
    }
    eprintln!("Linear HDR dispatch: PASSED");
}

fn temporal_smoke(
    profile: Option<Profile>,
    exposure_mode: ExposureMode,
    dynamic: bool,
    mask_bits: u32,
    hdr: bool,
) {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_GPU_DLL").expect("FSR_SDK_TEST_GPU_DLL"));
    assert!(std::env::var_os("FSR_SDK_TEST_GPU_CONTROL").is_none());
    // SAFETY: The opt-in runner builds this trusted helper and stages pinned DLLs.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact C exports/signatures are paired with gpu.cpp.
    let (
        enable_debug,
        create_device,
        require_amd,
        check_device,
        create_harness,
        set_frame,
        finish,
        destroy,
    ) = unsafe {
        (
            *helper.get::<EnableDebug>(b"gpu_enable_debug\0").unwrap(),
            *helper
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap(),
            *helper.get::<DeviceCheck>(b"gpu_require_amd\0").unwrap(),
            *helper.get::<DeviceCheck>(b"gpu_check_device\0").unwrap(),
            *helper.get::<CreateHarness>(b"gpu_create\0").unwrap(),
            *helper
                .get::<SetTemporalFrame>(b"gpu_set_temporal_frame\0")
                .unwrap(),
            *helper.get::<Finish>(b"gpu_finish\0").unwrap(),
            *helper.get::<Destroy>(b"gpu_destroy\0").unwrap(),
        )
    };
    // SAFETY: Debug configuration precedes creation of the only device.
    let hr = unsafe { enable_debug() };
    eprintln!("Gate B debug layer: HRESULT={hr:#010x}");
    assert!(
        hr >= 0,
        "ENVIRONMENT BLOCKER: debug layer unavailable: {hr:#010x}"
    );
    let mut raw_device = null_mut();
    // SAFETY: Writable output; one owned COM reference is transferred.
    let hr = unsafe { create_device(&mut raw_device) };
    eprintln!("Gate B device creation: HRESULT={hr:#010x}");
    assert!(
        hr >= 0 && !raw_device.is_null(),
        "ENVIRONMENT BLOCKER: device: {hr:#010x}"
    );
    // SAFETY: The helper transferred an owned live device interface.
    let device = unsafe { ID3D12Device::from_raw(raw_device) };
    // SAFETY: Inspect the actual device's adapter, not the machine inventory.
    let hr = unsafe { require_amd(device.as_raw()) };
    assert_eq!(
        hr, 0,
        "ENVIRONMENT BLOCKER: selected adapter is not AMD hardware: {hr:#010x}"
    );
    // SAFETY: Explicit trusted loader/provider match the pinned ABI.
    let mut runtime = unsafe { Runtime::load(&loader) }.unwrap();
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    options.set_exposure_mode(exposure_mode);
    options.set_dynamic_resolution(dynamic);
    options.set_high_dynamic_range(hdr);
    if exposure_mode == ExposureMode::Automatic || hdr {
        eprintln!(
            "Profile memory estimate: {:?}",
            unsafe { runtime.upscaler_dx12(&device) }
                .estimate_memory(&options)
                .unwrap()
        );
    }
    if let Some((depth, units, resolution, jitter)) = profile {
        options.set_depth_convention(depth);
        options.set_motion_vector_resolution(resolution);
        options.set_motion_vector_jitter(jitter);
        let estimate = unsafe { runtime.upscaler_dx12(&device) }
            .estimate_memory(&options)
            .unwrap();
        eprintln!(
            "Depth/motion profile: depth={depth:?} units={units:?} resolution={resolution:?} jitter={jitter:?} estimate={estimate:?}"
        );
    }
    let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
        .create(&options)
        .unwrap();
    eprintln!("Gate B public construction: OK native_code={FFX_API_RETURN_OK:#010x}");
    log_provider(&mut upscaler);

    // Keep every harness/resource set until after context teardown. Waiting
    // each frame is only this test's strategy, not a wrapper requirement.
    let mut frames = Vec::with_capacity(3);
    for (frame, (jx, jy)) in [(0.25, -0.25), (-0.25, 0.25), (0.125, -0.125)]
        .into_iter()
        .enumerate()
    {
        let frame = frame as u32;
        eprintln!(
            "Gate B frame={frame}: translation={frame}px jitter=({jx},{jy}) reset={} frame_time=16.667ms",
            frame == 0
        );
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: A fresh harness owns its recording list/allocator and resource
        // set; it transfers an extra COM reference for every returned interface.
        let hr = if let Some((_, _, resolution, _)) = profile {
            type CreateProfile = unsafe extern "C" fn(
                *mut c_void,
                u32,
                *mut *mut c_void,
                *mut *mut c_void,
                *mut *mut c_void,
            ) -> i32;
            // SAFETY: Exact trusted helper ABI; transfers owned COM references.
            unsafe {
                helper
                    .get::<CreateProfile>(c"gpu_create_motion_resolution")
                    .unwrap()(
                    device.as_raw(),
                    u32::from(resolution == MotionVectorResolution::Display),
                    &mut harness,
                    &mut raw_list,
                    raw_resources.as_mut_ptr(),
                )
            }
        } else {
            unsafe {
                create_harness(
                    device.as_raw(),
                    &mut harness,
                    &mut raw_list,
                    raw_resources.as_mut_ptr(),
                )
            }
        };
        eprintln!("Gate B frame={frame} harness creation: HRESULT={hr:#010x}");
        assert!(
            hr >= 0 && !harness.is_null() && !raw_list.is_null(),
            "HARNESS / INPUT FAILURE: frame={frame} setup: {hr:#010x}"
        );
        // SAFETY: Only not-yet-submitted colour/motion uploads are rewritten.
        let hr = unsafe { set_frame(harness, frame, jx, jy) };
        assert!(
            hr >= 0,
            "HARNESS / INPUT FAILURE: frame={frame} uploads: {hr:#010x}"
        );
        let pre_exposure = if hdr {
            [0.25, 1.0, 4.0][frame as usize]
        } else {
            1.0
        };
        if hdr {
            type SetHdrFrame = unsafe extern "C" fn(*mut c_void, f32) -> i32;
            // SAFETY: Exact helper ABI; writes pending uploads before submission.
            // Full colour/depth/motion/exposure copies are checked after fencing.
            unsafe {
                assert_eq!(
                    helper.get::<SetHdrFrame>(c"gpu_set_hdr_frame").unwrap()(harness, pre_exposure,),
                    0
                );
            }
            eprintln!(
                "Linear HDR frame={frame}: scene_rgb=8,4,2 pre_exposure={pre_exposure} mode={exposure_mode:?} explicit_exposure=1"
            );
        }
        if let Some((depth, units, _, jitter)) = profile {
            type SetProfile =
                unsafe extern "C" fn(*mut c_void, u32, u32, u32, u32, f32, f32, f32, f32) -> i32;
            let (previous_x, previous_y) = match frame {
                0 => (0.0, 0.0),
                1 => (0.25, -0.25),
                _ => (-0.25, 0.25),
            };
            // SAFETY: Exact trusted helper ABI; pending depth/motion uploads
            // are rewritten before any submission, while all resources are live.
            unsafe {
                let set = helper
                    .get::<SetProfile>(c"gpu_set_depth_motion_profile")
                    .unwrap();
                assert_eq!(
                    set(
                        harness,
                        u32::from(matches!(
                            depth,
                            DepthConvention::ReversedFinite | DepthConvention::ReversedInfinite
                        )),
                        u32::from(units == MotionVectorScale::Pixels),
                        u32::from(jitter == MotionVectorJitter::Included),
                        frame,
                        jx,
                        jy,
                        previous_x,
                        previous_y
                    ),
                    0
                );
            }
        }
        let active = if dynamic {
            [(320, 180), (256, 144), (288, 162)][frame as usize]
        } else {
            (320, 180)
        };
        if dynamic {
            type SetActive = unsafe extern "C" fn(*mut c_void, u32, u32, u32, u32, f32, f32) -> i32;
            // SAFETY: Exact helper ABI; writes pending uploads before submission.
            // Inactive input guards are read back and checked after fencing.
            unsafe {
                assert_eq!(
                    helper.get::<SetActive>(c"gpu_set_active_render").unwrap()(
                        harness,
                        active.0,
                        active.1,
                        u32::from(profile.is_some_and(|p| p.1 == MotionVectorScale::Pixels)),
                        frame,
                        jx,
                        jy
                    ),
                    0
                );
            }
            eprintln!(
                "Active render frame={frame}: active={active:?} allocations=320x180 output=640x360"
            );
        }
        // SAFETY: The helper transferred owned, live COM interfaces.
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<ID3D12Resource> = raw_resources
            .into_iter()
            .map(|pointer| {
                assert!(!pointer.is_null());
                // SAFETY: Each pointer carries one transferred COM reference.
                unsafe { ID3D12Resource::from_raw(pointer) }
            })
            .collect();
        let mut resource_bundle = if exposure_mode == ExposureMode::Automatic {
            UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Automatic)
                .output(&resources[4])
                .build()
                .unwrap()
        } else {
            UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                .output(&resources[4])
                .build()
                .unwrap()
        };
        let mut masks = Vec::new();
        if mask_bits != 0 {
            type CreateMasks =
                unsafe extern "C" fn(*mut c_void, u32, u32, u32, f32, f32, *mut *mut c_void) -> i32;
            let mut raw_masks = [null_mut(); 2];
            // SAFETY: Matching trusted helper ABI; records pending uploads and
            // transfers two distinct owned COM references, retained through teardown.
            unsafe {
                assert_eq!(
                    helper.get::<CreateMasks>(c"gpu_create_masks").unwrap()(
                        harness,
                        active.0,
                        active.1,
                        frame,
                        jx,
                        jy,
                        raw_masks.as_mut_ptr()
                    ),
                    0
                );
                for pointer in raw_masks {
                    assert!(!pointer.is_null());
                    masks.push(ID3D12Resource::from_raw(pointer));
                }
            }
            resource_bundle.set_reactive_mask((mask_bits & 16 != 0).then_some(&masks[0]));
            resource_bundle
                .set_transparency_and_composition_mask((mask_bits & 32 != 0).then_some(&masks[1]));
            eprintln!(
                "Supplied masks frame={frame}: bits={mask_bits} mode={exposure_mode:?} active={active:?} allocations=320x180"
            );
        }
        let mut input = UpscaleTestInputs::new(
            &list,
            resource_bundle,
            JitterOffsetPixels::new(jx, jy).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            if profile.is_some_and(|(depth, _, _, _)| depth.is_infinite()) {
                CameraParameters::new_infinite(0.1, std::f32::consts::FRAC_PI_3, 1.0).unwrap()
            } else {
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap()
            },
            frame == 0,
        );
        if let Some((_, units, _, _)) = profile {
            input.set_motion_vector_scale(units);
        }
        if dynamic {
            input.set_active_render_size(Some(Dimensions::new(active.0, active.1).unwrap()));
        }
        input.set_pre_exposure(PreExposure::new(pre_exposure).unwrap());
        // SAFETY: The helper supplies depth/motion matching the selected profile
        // and records correct transitions; all GPU dependencies survive fencing.
        let dispatch =
            unsafe { upscaler.record(input.command_list, &input.resources, &input.parameters) };
        if let Err(error) = dispatch {
            abort_dispatch(error, frame);
        }
        eprintln!("Gate B frame={frame} public dispatch: OK native_code={FFX_API_RETURN_OK:#010x}");
        let mut metrics = Metrics::default();
        // SAFETY: Readback transition/copy are recorded before Close/submission;
        // the helper fences GPU completion before CPU Map/metrics inspection.
        let hr = unsafe { finish(harness, &mut metrics) };
        if hr < 0 {
            eprintln!(
                "Gate B frame={frame} submit/fence/readback failed: HRESULT={hr:#010x}; completion may be unverified"
            );
            // Avoid ordinary cleanup if dependencies may still be in flight.
            std::process::exit(1);
        }
        eprintln!("Gate B frame={frame} GPU readback: {metrics:?}");
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(
            metrics.finite_pixels,
            230_400,
            "frame={frame}: non-finite pixels={}",
            230_400 - metrics.finite_pixels
        );
        assert!(
            metrics.changed_pixels > 0,
            "frame={frame}: expected output was not observed"
        );
        if hdr {
            assert_eq!(
                metrics.changed_pixels, 230_400,
                "HDR output sentinel remained"
            );
            // Broad signal-domain bounds for a constant HDR surface, not an
            // image-quality threshold. Clipping to [0,1] and stale sentinel
            // red channels must both fail, with all pre-exposure factors.
            assert!(
                metrics.min_red > 1.0 && metrics.max_red < 64.0,
                "HDR output lost its signal range: {metrics:?}"
            );
            eprintln!(
                "HDR output range: PASSED frame={frame} min={} max={}",
                metrics.min_red, metrics.max_red
            );
        }
        if dynamic {
            assert_eq!(
                metrics.changed_pixels, 230_400,
                "active render output sentinel remained"
            );
        }
        assert_eq!(
            metrics.debug_errors, 0,
            "frame={frame}: debug-layer errors/corruption"
        );
        assert_eq!(metrics.device_removed, 0, "frame={frame}: device removed");
        frames.push((harness, list, resources, masks));
    }
    // All three submitted dispatches were fenced. Required external dependencies
    // remain alive through the selected explicit or implicit destruction path.
    if profile.is_some() || exposure_mode == ExposureMode::Automatic || mask_bits != 0 || hdr {
        upscaler.destroy().unwrap();
        eprintln!("Profile explicit destroy: OK");
    } else {
        drop(upscaler);
        eprintln!("Gate B normal Drop completed; native destroy result is not exposed");
    }
    // SAFETY: The caller device and all frame resource sets remain alive.
    assert_eq!(
        unsafe { check_device(device.as_raw()) },
        0,
        "post-Drop device/debug failure"
    );
    for (harness, list, resources, masks) in frames {
        drop(masks);
        drop(resources);
        drop(list);
        // SAFETY: All GPU work completed; no Rust resource/list borrows remain.
        unsafe { destroy(harness) };
    }
    eprintln!("Gate B after frame-resource cleanup:");
    // SAFETY: The device/runtime are retained after all dependent cleanup.
    assert_eq!(
        unsafe { check_device(device.as_raw()) },
        0,
        "post-resource-cleanup device/debug failure"
    );
    if let Some(profile) = profile {
        eprintln!("Depth/motion profile: PASSED {profile:?}");
    } else {
        eprintln!("Gate B: PASSED");
    }
}
