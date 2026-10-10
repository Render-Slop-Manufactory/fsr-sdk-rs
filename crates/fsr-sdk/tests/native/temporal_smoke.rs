// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn native_temporal_smoke() {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_M6_DLL").expect("FSR_SDK_TEST_M6_DLL"));
    assert!(std::env::var_os("FSR_SDK_TEST_M6_CONTROL").is_none());
    // SAFETY: The opt-in runner builds this trusted helper and stages pinned DLLs.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact C exports/signatures are paired with m6_gpu.cpp.
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
            *helper.get::<EnableDebug>(b"m6_enable_debug\0").unwrap(),
            *helper
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap(),
            *helper.get::<DeviceCheck>(b"m6_require_amd\0").unwrap(),
            *helper.get::<DeviceCheck>(b"m6_check_device\0").unwrap(),
            *helper.get::<CreateHarness>(b"m6_create\0").unwrap(),
            *helper
                .get::<SetTemporalFrame>(b"m6_set_temporal_frame\0")
                .unwrap(),
            *helper.get::<Finish>(b"m6_finish\0").unwrap(),
            *helper.get::<Destroy>(b"m6_destroy\0").unwrap(),
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
    let runtime = unsafe { Runtime::load_unchecked(&loader) }.unwrap();
    let mut upscaler = Upscaler::new(
        &runtime,
        &device,
        UpscalerOptions::new(
            Dimensions::new(320, 180).unwrap(),
            Dimensions::new(640, 360).unwrap(),
        ),
    )
    .unwrap();
    eprintln!("Gate B public construction: OK native_code={FFX_API_RETURN_OK:#010x}");
    log_provider(&mut upscaler);

    // Keep every harness/resource set until after normal context Drop. Waiting
    // each frame is only this test's strategy, not a wrapper requirement.
    let mut frames = Vec::with_capacity(3);
    for (frame, (jx, jy)) in [(0.25, -0.25), (-0.25, 0.25), (0.125, -0.125)]
        .into_iter()
        .enumerate()
    {
        let frame = frame as u32;
        eprintln!(
            "Gate B frame={frame}: translation={frame}px jitter=({jx},{jy}) reset={} motion=({},0) frame_time=16.667ms",
            frame == 0,
            if frame == 0 { 0.0 } else { -1.0 / 320.0 }
        );
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: A fresh harness owns its recording list/allocator and resource
        // set; it transfers an extra COM reference for every returned interface.
        let hr = unsafe {
            create_harness(
                device.as_raw(),
                &mut harness,
                &mut raw_list,
                raw_resources.as_mut_ptr(),
            )
        };
        eprintln!("Gate B frame={frame} harness creation: HRESULT={hr:#010x}");
        assert!(
            hr >= 0 && !harness.is_null() && !raw_list.is_null(),
            "HARNESS / INPUT FAILURE: frame={frame} setup: {hr:#010x}"
        );
        // SAFETY: Only not-yet-submitted color/motion uploads are rewritten.
        let hr = unsafe { set_frame(harness, frame, jx, jy) };
        assert!(
            hr >= 0,
            "HARNESS / INPUT FAILURE: frame={frame} uploads: {hr:#010x}"
        );
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
        // SAFETY: The helper recorded correct input/output transitions. This
        // upscaler, runtime, device, resources and allocator survive completion.
        let dispatch = unsafe {
            upscaler.dispatch(UpscaleDispatch::new(
                &list,
                UpscaleResources::new(
                    &resources[0],
                    &resources[1],
                    &resources[2],
                    &resources[3],
                    &resources[4],
                ),
                JitterOffsetPixels::new(jx, jy).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                frame == 0,
            ))
        };
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
        assert_eq!(
            metrics.debug_errors, 0,
            "frame={frame}: debug-layer errors/corruption"
        );
        assert_eq!(metrics.device_removed, 0, "frame={frame}: device removed");
        frames.push((harness, list, resources));
    }
    // All three submitted dispatches were fenced. Required external dependencies
    // are still alive while the ordinary D005 Drop makes its one destroy attempt.
    drop(upscaler);
    eprintln!("Gate B normal Drop completed; native destroy result is not exposed");
    // SAFETY: The caller device and all frame resource sets remain alive.
    assert_eq!(
        unsafe { check_device(device.as_raw()) },
        0,
        "post-Drop device/debug failure"
    );
    for (harness, list, resources) in frames {
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
    eprintln!("Gate B: PASSED");
}
