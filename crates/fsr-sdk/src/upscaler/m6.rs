// SPDX-License-Identifier: MPL-2.0

//! Isolated M6 GPU proof, now exercising the public M6b route.
//! C++ DX12 equipment lives under tests/.
use super::{
    CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch, Upscaler,
    UpscalerOptions, dispatch::resource_for_parity,
};
use crate::runtime::Runtime;
use fsr_sdk_sys::api::{
    FFX_API_RESOURCE_STATE_COMPUTE_READ, FFX_API_RESOURCE_STATE_UNORDERED_ACCESS,
    FFX_API_RETURN_OK, FfxApiResource, ffxApiHeader,
};
use std::{
    ffi::{CStr, c_char, c_void},
    path::PathBuf,
    ptr::null_mut,
};
use windows::{
    Win32::Graphics::{
        Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource},
        Dxgi::Common::{
            DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32_FLOAT,
        },
    },
    core::Interface,
};

#[repr(C)]
#[derive(Default, Debug)]
struct Metrics {
    width: u32,
    height: u32,
    finite_pixels: u32,
    changed_pixels: u32,
    min_red: f32,
    max_red: f32,
    left_red: f32,
    right_red: f32,
    debug_errors: u64,
    device_removed: i32,
}

// Test-only provider-identification query from SDK v2.3.0 ffx_api.h, like M5.
#[repr(C)]
struct ProviderQuery {
    header: ffxApiHeader,
    id: u64,
    name: *const c_char,
}

type EnableDebug = unsafe extern "C" fn() -> i32;
type CreateDevice = unsafe extern "C" fn(*mut *mut c_void) -> i32;
type CreateHarness =
    unsafe extern "C" fn(*mut c_void, *mut *mut c_void, *mut *mut c_void, *mut *mut c_void) -> i32;
type CompareResource = unsafe extern "C" fn(*mut c_void, *const FfxApiResource, u32) -> i32;
type Finish = unsafe extern "C" fn(*mut c_void, *mut Metrics) -> i32;
type Destroy = unsafe extern "C" fn(*mut c_void);

#[test]
#[ignore = "requires trusted staged v2.3.0 DLLs, MSVC-built m6_gpu.dll and a hardware DX12 GPU; use tests/run-m6.ps1 -Native"]
fn native_dispatch_readback() {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_M6_DLL").expect("FSR_SDK_TEST_M6_DLL"));
    // SAFETY: The opt-in runner compiles this trusted local helper and stages
    // matching v2.3.0 SDK DLLs; signatures below match its C exports.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Every symbol is looked up by its exact C ABI export name.
    let enable_debug = unsafe { *helper.get::<EnableDebug>(b"m6_enable_debug\0").unwrap() };
    let create_device = unsafe {
        *helper
            .get::<CreateDevice>(b"probe_create_device\0")
            .unwrap()
    };
    let create_harness = unsafe { *helper.get::<CreateHarness>(b"m6_create\0").unwrap() };
    let compare = unsafe {
        *helper
            .get::<CompareResource>(b"m6_compare_resource\0")
            .unwrap()
    };
    let finish = unsafe { *helper.get::<Finish>(b"m6_finish\0").unwrap() };
    let destroy = unsafe { *helper.get::<Destroy>(b"m6_destroy\0").unwrap() };

    // SAFETY: Debug configuration occurs before device creation.
    let debug_hr = unsafe { enable_debug() };
    assert!(
        debug_hr >= 0,
        "D3D12 debug/GPU validation unavailable: {debug_hr:#x}"
    );
    let mut raw_device = null_mut();
    // SAFETY: Writable output; the helper transfers one COM reference.
    let hr = unsafe { create_device(&mut raw_device) };
    assert!(hr >= 0 && !raw_device.is_null(), "device creation: {hr:#x}");
    // SAFETY: The helper returned an owned ID3D12Device reference.
    let device = unsafe { ID3D12Device::from_raw(raw_device) };
    eprintln!("M6a: loading runtime");
    // SAFETY: The explicit trusted loader and provider match the pinned ABI.
    let runtime = unsafe { Runtime::load(&loader) }.unwrap();
    eprintln!("M6a: creating public M5 context");
    let mut upscaler = Upscaler::new(
        &runtime,
        &device,
        UpscalerOptions {
            max_render_size: Dimensions::new(320, 180).unwrap(),
            max_upscale_size: Dimensions::new(640, 360).unwrap(),
        },
    )
    .unwrap();
    let mut provider = ProviderQuery {
        header: ffxApiHeader {
            r#type: 6,
            pNext: null_mut(),
        },
        id: 0,
        name: std::ptr::null(),
    };
    // SAFETY: This exact test-only query ABI is paired with the existing M5
    // native fixture; the context and runtime remain live while reading name.
    let query_code = unsafe { upscaler.owner.query_for_test(&raw mut provider.header) };
    assert_eq!(query_code, FFX_API_RETURN_OK, "provider query failed");
    assert!(!provider.name.is_null() && provider.id != 0);
    // SAFETY: A successful query returned a live native name pointer.
    let name = unsafe { CStr::from_ptr(provider.name) };
    eprintln!(
        "M6a provider: id={:#018x}; name={}",
        provider.id,
        name.to_string_lossy()
    );
    eprintln!("M6a: creating synthetic textures and list");
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
    eprintln!("M6a: DX12 setup complete");
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
    eprintln!("M6a: DX12 resource parity complete");
    if std::env::var_os("FSR_SDK_TEST_M6_CONTROL").is_some() {
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
    // SAFETY: The helper recorded matching transitions; all resources, the
    // allocator, context and runtime are retained until the fence completes.
    let dispatch = unsafe {
        upscaler.dispatch(UpscaleDispatch {
            command_list: &list,
            color: &resources[0],
            depth: &resources[1],
            motion_vectors: &resources[2],
            exposure: &resources[3],
            output: &resources[4],
            jitter: JitterOffsetPixels::new(0.25, -0.25).unwrap(),
            frame_time: FrameTimeMillis::new(16.667).unwrap(),
            camera: CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
            reset: true,
        })
    };
    if let Err(error) = dispatch {
        eprintln!("ffxDispatch failed: {error:?}");
        // The provider may have recorded partial work. End this isolated child
        // without running context/resource destructors before GPU completion.
        std::process::exit(1);
    }
    eprintln!("ffxDispatch: OK");
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
    assert_eq!(metrics.debug_errors, 0, "DX12 validation errors");
    assert_eq!(metrics.finite_pixels, expected, "non-finite output");
    assert!(
        metrics.changed_pixels > expected / 2,
        "output stayed at sentinel"
    );
    assert!(
        metrics.max_red - metrics.min_red > 0.1,
        "output lacks variation"
    );
    assert!(
        metrics.right_red > metrics.left_red + 0.05,
        "output lacks input gradient"
    );
    drop(resources);
    drop(list);
    upscaler.destroy().unwrap();
    // SAFETY: GPU completion was fenced, and no Rust reference to harness
    // resources remains. The C++ owner can now release its COM references.
    unsafe { destroy(harness) };
}
