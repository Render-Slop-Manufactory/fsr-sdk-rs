// SPDX-License-Identifier: MPL-2.0

//! Opt-in M6 recording-ahead experiment.
use super::{
    CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch, Upscaler,
    UpscalerOptions,
};
use crate::runtime::Runtime;
use std::{ffi::c_void, path::PathBuf, ptr::null_mut};
use windows::{
    Win32::Graphics::Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource},
    core::Interface,
};

#[repr(C)]
#[derive(Debug, Default)]
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

type EnableDebug = unsafe extern "C" fn() -> i32;
type CreateDevice = unsafe extern "C" fn(*mut *mut c_void) -> i32;
type CreateHarness =
    unsafe extern "C" fn(*mut c_void, *mut *mut c_void, *mut *mut c_void, *mut *mut c_void) -> i32;
type SetColorShift = unsafe extern "C" fn(*mut c_void, f32) -> i32;
type FinishBatch = unsafe extern "C" fn(*const *mut c_void, u32, *mut Metrics) -> i32;
type Destroy = unsafe extern "C" fn(*mut c_void);

struct Recording {
    harness: *mut c_void,
    list: ID3D12GraphicsCommandList,
    resources: Vec<ID3D12Resource>,
}

#[test]
#[ignore = "requires staged v2.3.0 DLLs, MSVC-built m6_gpu.dll and a hardware DX12 GPU; use tests/run-m6-recording-spike.ps1"]
fn native_recording_ahead_spike() {
    let count: usize = std::env::var("FSR_SDK_TEST_RECORD_COUNT")
        .expect("FSR_SDK_TEST_RECORD_COUNT")
        .parse()
        .expect("integer record count");
    assert!((1..=8).contains(&count));
    let mode = std::env::var("FSR_SDK_TEST_RECORD_MODE").expect("FSR_SDK_TEST_RECORD_MODE");
    assert!(mode == "same" || mode == "sibling");
    eprintln!("recording spike: mode={mode} count={count}");

    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_M6_DLL").expect("FSR_SDK_TEST_M6_DLL"));
    // SAFETY: The opt-in runner builds the local helper and stages matching SDK DLLs.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact C export names and signatures are paired with m6_gpu.cpp.
    let enable_debug = unsafe { *helper.get::<EnableDebug>(b"m6_enable_debug\0").unwrap() };
    let create_device = unsafe {
        *helper
            .get::<CreateDevice>(b"probe_create_device\0")
            .unwrap()
    };
    let create_harness = unsafe { *helper.get::<CreateHarness>(b"m6_create\0").unwrap() };
    let set_color_shift = unsafe {
        *helper
            .get::<SetColorShift>(b"m6_set_color_shift\0")
            .unwrap()
    };
    let finish_batch = unsafe {
        *helper
            .get::<FinishBatch>(b"m6_finish_recorded_batch\0")
            .unwrap()
    };
    let destroy = unsafe { *helper.get::<Destroy>(b"m6_destroy\0").unwrap() };

    // SAFETY: Enable the debug layer before creating the device.
    let hr = unsafe { enable_debug() };
    assert!(hr >= 0, "debug layer: {hr:#x}");
    let mut raw_device = null_mut();
    // SAFETY: Writable output; the helper transfers one COM reference.
    let hr = unsafe { create_device(&mut raw_device) };
    assert!(hr >= 0 && !raw_device.is_null(), "device: {hr:#x}");
    // SAFETY: The helper returned an owned ID3D12Device reference.
    let device = unsafe { ID3D12Device::from_raw(raw_device) };
    // SAFETY: The explicit trusted loader and provider match the pinned ABI.
    let runtime = unsafe { Runtime::load(&loader) }.unwrap();
    let context_count = if mode == "sibling" { count } else { 1 };
    let mut contexts = Vec::with_capacity(context_count);
    for index in 0..context_count {
        let upscaler = Upscaler::new(
            &runtime,
            &device,
            UpscalerOptions {
                max_render_size: Dimensions::new(320, 180).unwrap(),
                max_upscale_size: Dimensions::new(640, 360).unwrap(),
            },
        )
        .unwrap();
        eprintln!("created context {index}");
        contexts.push(upscaler);
    }

    let mut recordings = Vec::with_capacity(count);
    for index in 0..count {
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: The helper owns a fresh allocator, recording DIRECT list and
        // resource set, transferring one extra COM reference for each pointer.
        let hr = unsafe {
            create_harness(
                device.as_raw(),
                &mut harness,
                &mut raw_list,
                raw_resources.as_mut_ptr(),
            )
        };
        assert!(
            hr >= 0 && !harness.is_null() && !raw_list.is_null(),
            "harness {index}: {hr:#x}"
        );
        // SAFETY: The color upload belongs to this unsubmitted harness. The
        // shifted synthetic red channel stays below 1.0 for all eight cases.
        let hr = unsafe { set_color_shift(harness, index as f32 * 0.015) };
        assert!(hr >= 0, "color shift {index}: {hr:#x}");
        // SAFETY: Each non-null pointer carries a transferred COM reference.
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<ID3D12Resource> = raw_resources
            .into_iter()
            .map(|pointer| {
                assert!(!pointer.is_null());
                // SAFETY: The helper AddRef'd each returned resource.
                unsafe { ID3D12Resource::from_raw(pointer) }
            })
            .collect();
        let context = if mode == "sibling" { index } else { 0 };
        // SAFETY: The helper has recorded input transitions on this fresh list;
        // all contexts, resources and allocators remain live until the fence.
        let dispatch = unsafe {
            contexts[context].dispatch(UpscaleDispatch {
                command_list: &list,
                color: &resources[0],
                depth: &resources[1],
                motion_vectors: &resources[2],
                exposure: &resources[3],
                output: &resources[4],
                jitter: JitterOffsetPixels::new(0.25, -0.25).unwrap(),
                frame_time: FrameTimeMillis::new(16.667).unwrap(),
                camera: CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0)
                    .unwrap(),
                reset: mode == "sibling" || index == 0,
            })
        };
        if let Err(error) = dispatch {
            eprintln!("dispatch {index} failed: {error:?}");
            // Partial native recording may exist. Keep the process isolated and
            // avoid context/resource destruction before GPU completion.
            std::process::exit(1);
        }
        eprintln!("recorded dispatch {index}: OK");
        recordings.push(Recording {
            harness,
            list,
            resources,
        });
    }
    eprintln!("all dispatches recorded; submitting now");
    let handles: Vec<*mut c_void> = recordings
        .iter()
        .map(|recording| recording.harness)
        .collect();
    let mut metrics: Vec<Metrics> = (0..count).map(|_| Metrics::default()).collect();
    // SAFETY: Each closed list has a unique allocator/resource set. The helper
    // submits on one queue in order and waits for its fence before readback.
    let hr = unsafe { finish_batch(handles.as_ptr(), count as u32, metrics.as_mut_ptr()) };
    if hr < 0 {
        eprintln!("batch submit/fence/readback failed: {hr:#x}");
        std::process::exit(1);
    }
    let mut valid = true;
    let mut previous_left = None;
    for (index, result) in metrics.iter().enumerate() {
        eprintln!("GPU readback {index}: {result:?}");
        let expected = result.width * result.height;
        valid &= (result.width, result.height) == (640, 360)
            && result.device_removed == 0
            && result.debug_errors == 0
            && result.finite_pixels == expected
            && result.changed_pixels > expected / 2
            && result.max_red - result.min_red > 0.1
            && result.right_red > result.left_red + 0.05;
        if let Some(previous) = previous_left {
            valid &= result.left_red > previous + 0.005;
        }
        previous_left = Some(result.left_red);
    }
    for Recording {
        harness: _,
        list,
        resources,
    } in recordings
    {
        drop(resources);
        drop(list);
    }
    for context in contexts {
        context.destroy().unwrap();
    }
    // SAFETY: The fence completed and all Rust COM references have been dropped.
    for handle in handles {
        unsafe { destroy(handle) };
    }
    assert!(valid, "one or more batch outputs failed validation");
}
