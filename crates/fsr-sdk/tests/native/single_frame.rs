// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::upscaler::{ExposureInput, PreExposure};

pub(super) fn native_dispatch_readback() {
    dispatch_readback(false, None);
}

pub(super) fn proton_dispatch_readback() {
    dispatch_readback(true, None);
}

pub(super) fn native_pre_exposure_dispatch() {
    assert!(
        std::env::var_os("FSR_SDK_TEST_GPU_CONTROL").is_none(),
        "pre-exposure case requires SDK dispatch"
    );
    for value in [0.25, 1.0, 4.0] {
        dispatch_readback(false, Some(PreExposure::new(value).unwrap()));
    }
    eprintln!("Pre-exposure dispatch: PASSED");
}

fn dispatch_readback(proton: bool, pre_exposure: Option<PreExposure>) {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_GPU_DLL").expect("FSR_SDK_TEST_GPU_DLL"));
    // SAFETY: The opt-in runner compiles this trusted local helper and stages
    // matching v2.3.0 SDK DLLs; signatures below match its C exports.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Every symbol is looked up by its exact C ABI export name.
    let enable_debug = unsafe { *helper.get::<EnableDebug>(b"gpu_enable_debug\0").unwrap() };
    let create_device = unsafe {
        *helper
            .get::<CreateDevice>(b"probe_create_device\0")
            .unwrap()
    };
    // SAFETY: These exact helper exports inspect the live caller-owned device.
    let require_amd = unsafe { *helper.get::<DeviceCheck>(b"gpu_require_amd\0").unwrap() };
    let check_symbol: &[u8] = if proton {
        b"gpu_check_device_removed\0"
    } else {
        b"gpu_check_device\0"
    };
    let check_device = unsafe { *helper.get::<DeviceCheck>(check_symbol).unwrap() };
    let create_harness = unsafe { *helper.get::<CreateHarness>(b"gpu_create\0").unwrap() };
    let compare = unsafe {
        *helper
            .get::<CompareResource>(b"gpu_compare_resource\0")
            .unwrap()
    };
    let finish = unsafe { *helper.get::<Finish>(b"gpu_finish\0").unwrap() };
    let destroy = unsafe { *helper.get::<Destroy>(b"gpu_destroy\0").unwrap() };

    // SAFETY: Debug configuration occurs before device creation.
    let debug_hr = unsafe { enable_debug() };
    eprintln!("Gate A debug layer: HRESULT={debug_hr:#010x}");
    assert!(
        proton || debug_hr >= 0,
        "ENVIRONMENT BLOCKER: D3D12 debug/GPU validation unavailable: {debug_hr:#x}"
    );
    if proton {
        eprintln!(
            "Proton validation: DirectX debug HRESULT={debug_hr:#010x}; D3D12 GBV not enabled; Vulkan diagnostics recorded by launcher"
        );
        assert!(
            std::env::var_os("FSR_SDK_TEST_GPU_CONTROL").is_none(),
            "Proton case requires SDK dispatch"
        );
    }
    let mut raw_device = null_mut();
    // SAFETY: Writable output; the helper transfers one COM reference.
    let hr = unsafe { create_device(&mut raw_device) };
    eprintln!("Gate A device creation: HRESULT={hr:#010x}");
    assert!(hr >= 0 && !raw_device.is_null(), "device creation: {hr:#x}");
    // SAFETY: The helper returned an owned ID3D12Device reference.
    let device = unsafe { ID3D12Device::from_raw(raw_device) };
    // SAFETY: The device is live; the helper resolves its actual adapter LUID.
    let hr = unsafe { require_amd(device.as_raw()) };
    assert_eq!(
        hr, 0,
        "ENVIRONMENT BLOCKER: selected adapter is not AMD hardware: {hr:#010x}"
    );
    eprintln!("Single-frame: loading runtime");
    // SAFETY: The explicit trusted deployment matches the pinned ABI and D018
    // query assumptions; this child serializes calls and retains dependencies.
    let mut runtime = unsafe { Runtime::load(&loader) }.unwrap();
    let options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    assert!(
        !unsafe { runtime.upscaler_dx12(&device) }
            .providers()
            .unwrap()
            .is_empty()
    );
    let estimate = unsafe { runtime.upscaler_dx12(&device) }
        .estimate_memory(&options)
        .unwrap();
    eprintln!("Single-frame: creating public upscaler context");
    let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
        .create(&options)
        .unwrap();
    let provider = upscaler.provider().unwrap();
    let requirements = upscaler.resource_requirements().unwrap();
    let usage = upscaler.gpu_memory_usage().unwrap();
    eprintln!(
        "Gate A queries: provider={provider:?}; estimate={estimate:?}; requirements={requirements:?}; usage={usage:?}"
    );
    eprintln!("Gate A public construction: OK native_code={FFX_API_RETURN_OK:#010x}");
    log_provider(&mut upscaler);
    eprintln!("Single-frame: creating synthetic textures and list");
    let mut harness = null_mut();
    let mut raw_list = null_mut();
    let mut raw_resources = [null_mut(); 5];
    // SAFETY: The helper owns a recording DIRECT list and five resources; it
    // transfers an extra COM reference for each returned interface.
    let hr = unsafe {
        create_harness(
            device.as_raw(),
            &mut harness,
            &mut raw_list,
            raw_resources.as_mut_ptr(),
        )
    };
    assert!(hr >= 0 && !harness.is_null(), "DX12 setup: {hr:#x}");
    if let Some(factor) = pre_exposure {
        type SetPreExposure = unsafe extern "C" fn(*mut c_void, f32) -> i32;
        // SAFETY: Exact trusted helper export; this fresh harness's pending
        // colour upload is scaled before submission, with all resources live.
        unsafe {
            let set = helper
                .get::<SetPreExposure>(c"gpu_set_pre_exposure")
                .unwrap();
            assert_eq!(set(harness, factor.value()), 0);
        }
        eprintln!("Pre-exposure input RGB scaled: factor={}", factor.value());
    }
    eprintln!("Single-frame: DX12 setup complete");
    // SAFETY: Each non-null pointer carries one transferred COM reference.
    let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
    let resources: Vec<ID3D12Resource> = raw_resources
        .into_iter()
        .map(|pointer| {
            assert!(!pointer.is_null());
            // SAFETY: The C++ harness AddRef'd each returned resource.
            unsafe { ID3D12Resource::from_raw(pointer) }
        })
        .collect();
    let formats = [
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        DXGI_FORMAT_R32_FLOAT,
        DXGI_FORMAT_R16G16_FLOAT,
        DXGI_FORMAT_R32_FLOAT,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
    ];
    for index in 0..5 {
        let native = resource_for_parity(&resources[index], formats[index], index == 4).unwrap();
        let state = if index == 4 {
            FFX_API_RESOURCE_STATE_UNORDERED_ACCESS
        } else {
            FFX_API_RESOURCE_STATE_COMPUTE_READ
        };
        // SAFETY: The helper compares fields to AMD's inline DX12 converter;
        // all pointers remain live during this call.
        assert_eq!(
            unsafe { compare(resources[index].as_raw(), &native, state) },
            1,
            "DX12 conversion differs from AMD helper for resource {index}"
        );
    }
    eprintln!("Single-frame: DX12 resource parity complete");
    if std::env::var_os("FSR_SDK_TEST_GPU_CONTROL").is_some() {
        // The same uploads, barriers, list, queue and readback run under GBV,
        // without calling the FSR provider's dispatch entry point.
        let mut metrics = Metrics::default();
        // SAFETY: The helper retains the resources and waits on its fence.
        let hr = unsafe { finish(harness, &mut metrics) };
        if hr < 0 {
            eprintln!("DX12 control submit/fence/readback failed: {hr:#x}");
            std::process::exit(1);
        }
        eprintln!("DX12 control readback: {metrics:?}");
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(metrics.finite_pixels, 640 * 360);
        assert_eq!(metrics.changed_pixels, 0, "control output lost sentinel");
        assert_eq!(metrics.debug_errors, 0, "DX12 validation errors");
        assert_eq!(metrics.device_removed, 0, "device removed");
        drop(resources);
        drop(list);
        upscaler.destroy().unwrap();
        // SAFETY: Submission completed before releasing the helper's COM refs.
        unsafe { destroy(harness) };
        return;
    }
    let bindings = UpscaleResources::builder()
        .color(&resources[0])
        .depth(&resources[1])
        .motion_vectors(&resources[2])
        .exposure(ExposureInput::Texture(&resources[3]))
        .output(&resources[4])
        .build()
        .unwrap();
    let mut frame = crate::upscaler::FrameParameters::new(
        JitterOffsetPixels::new(0.25, -0.25).unwrap(),
        FrameTimeMillis::new(16.667).unwrap(),
        CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
        crate::upscaler::HistoryAction::Reset,
    );
    if let Some(factor) = pre_exposure {
        frame.set_pre_exposure(factor);
    }
    // SAFETY: Input RGB has matching pre-exposure; the helper recorded the
    // required states and retains resources/allocator/context through its fence.
    let dispatch = unsafe { upscaler.record(&list, &bindings, &frame) };
    if let Err(error) = dispatch {
        abort_dispatch(error, 0);
    }
    eprintln!("ffxDispatch: OK native_code={FFX_API_RETURN_OK:#010x}");
    let mut metrics = Metrics::default();
    // SAFETY: The helper owns the list/queue/fence and waits before readback.
    let hr = unsafe { finish(harness, &mut metrics) };
    if hr < 0 {
        eprintln!("DX12 submit/fence/readback failed: {hr:#x}");
        std::process::exit(1);
    }
    eprintln!("GPU readback: {metrics:?}");
    let expected = metrics.width * metrics.height;
    assert_eq!((metrics.width, metrics.height), (640, 360));
    assert_eq!(metrics.device_removed, 0, "device removed");
    if proton && metrics.debug_errors == u64::MAX {
        eprintln!("Proton validation: DirectX info queue unavailable; error count unavailable");
    } else {
        assert!(
            debug_hr >= 0,
            "error count cannot replace an unavailable debug layer"
        );
        assert_eq!(metrics.debug_errors, 0, "DX12 validation errors");
    }
    assert_eq!(metrics.finite_pixels, expected, "non-finite output");
    assert!(
        metrics.changed_pixels > expected / 2,
        "output stayed at sentinel"
    );
    if pre_exposure.is_none() {
        assert!(
            metrics.max_red - metrics.min_red > 0.1,
            "output lacks variation"
        );
        assert!(
            metrics.right_red > metrics.left_red + 0.05,
            "output lacks input gradient"
        );
    }
    drop(resources);
    drop(list);
    upscaler.destroy().unwrap();
    eprintln!("Gate A explicit destroy: OK native_code={FFX_API_RETURN_OK:#010x}");
    // SAFETY: GPU completion was fenced, and no Rust reference to harness
    // resources remains. The C++ owner can now release its COM references.
    unsafe { destroy(harness) };
    eprintln!("Gate A after explicit destroy and resource cleanup:");
    // SAFETY: All GPU work completed; the caller's device/runtime remain live.
    assert_eq!(
        unsafe { check_device(device.as_raw()) },
        0,
        "post-destroy device/debug failure"
    );
    if let Some(factor) = pre_exposure {
        eprintln!("Pre-exposure frame: PASSED factor={}", factor.value());
    } else if proton {
        eprintln!("Proton single-frame: PASSED");
    } else {
        eprintln!("Gate A: PASSED");
    }
}
