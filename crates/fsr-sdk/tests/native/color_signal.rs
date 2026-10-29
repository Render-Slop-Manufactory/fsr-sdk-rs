// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::upscaler::{ColorEncodingHint, ExposureMode, InputColorSpace, PreExposure};

type Upload = unsafe extern "C" fn(*mut c_void, *const u8, u32, f32) -> i32;
type CopyPayload = unsafe extern "C" fn(*mut c_void, *mut u8, u32) -> i32;

fn variable(name: &str) -> String {
    let legacy = name
        .strip_prefix("FSR_SDK_COLOR_")
        .and_then(|suffix| std::env::var_os(format!("FSR_SDK_COLOUR_{suffix}")));
    resolve_variable(name, std::env::var_os(name), legacy)
}

fn resolve_variable(
    name: &str,
    current: Option<std::ffi::OsString>,
    legacy: Option<std::ffi::OsString>,
) -> String {
    assert!(
        current.is_none() || legacy.is_none() || current == legacy,
        "conflicting current and legacy values for {name}"
    );
    current
        .or(legacy)
        .unwrap_or_else(|| panic!("missing {name}"))
        .into_string()
        .unwrap_or_else(|_| panic!("non-Unicode value for {name}"))
}

pub(super) fn run() {
    let mode = variable("FSR_SDK_COLOR_ENCODING");
    let (space, hint) = match mode.as_str() {
        "linear" => (InputColorSpace::Linear, ColorEncodingHint::Unspecified),
        "unspecified" => (InputColorSpace::NonLinear, ColorEncodingHint::Unspecified),
        "srgb" => (InputColorSpace::NonLinear, ColorEncodingHint::Srgb),
        "pq" => (InputColorSpace::NonLinear, ColorEncodingHint::Pq),
        _ => panic!("invalid encoding"),
    };
    let automatic = variable("FSR_SDK_COLOR_AUTOMATIC") == "1";
    let pre: Vec<f32> = variable("FSR_SDK_COLOR_PRE_EXPOSURE")
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let exposure: Vec<f32> = variable("FSR_SDK_COLOR_EXPOSURE")
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let reset: Vec<bool> = variable("FSR_SDK_COLOR_RESET")
        .split(',')
        .map(|s| match s {
            "1" => true,
            "0" => false,
            _ => panic!("invalid reset"),
        })
        .collect();
    assert_eq!((pre.len(), exposure.len()), (3, 3));
    assert_eq!(reset.len(), 3);
    assert!(reset[0]);
    assert!(exposure.iter().all(|v| v.is_finite() && *v > 0.0));
    let inputs = PathBuf::from(variable("FSR_SDK_COLOR_INPUT"));
    let outputs = PathBuf::from(variable("FSR_SDK_COLOR_OUTPUT"));
    let loader = PathBuf::from(variable("FSR_SDK_TEST_DLL"));
    // SAFETY: Explicit opt-in runner builds the helper and stages hash-pinned AMD DLLs.
    let helper = unsafe { libloading::Library::new(variable("FSR_SDK_TEST_GPU_DLL")) }.unwrap();
    // SAFETY: All loads match the paired C++ exports in this checkout.
    let (enable, create_device, require_amd, checkpoint, create, upload, finish, copy, destroy) = unsafe {
        (
            *helper.get::<EnableDebug>(c"gpu_enable_debug").unwrap(),
            *helper.get::<CreateDevice>(c"probe_create_device").unwrap(),
            *helper.get::<DeviceCheck>(c"gpu_require_amd").unwrap(),
            *helper.get::<DeviceCheck>(c"gpu_check_device").unwrap(),
            *helper.get::<CreateHarness>(c"gpu_create").unwrap(),
            *helper.get::<Upload>(c"gpu_set_color_signal").unwrap(),
            *helper.get::<Finish>(c"gpu_finish").unwrap(),
            *helper.get::<CopyPayload>(c"gpu_copy_payload").unwrap(),
            *helper.get::<Destroy>(c"gpu_destroy").unwrap(),
        )
    };
    // SAFETY: Enable the debug layer before creating any device.
    assert_eq!(unsafe { enable() }, 0);
    let mut raw = null_mut();
    // SAFETY: Writable output receives one owned COM reference.
    assert_eq!(unsafe { create_device(&mut raw) }, 0);
    assert!(!raw.is_null());
    // SAFETY: The helper transfers an owned ID3D12Device interface.
    let device = unsafe { ID3D12Device::from_raw(raw) };
    // SAFETY: Device remains live through all helper calls.
    assert_eq!(unsafe { require_amd(device.as_raw()) }, 0);
    // SAFETY: The bounded selected signal/provider combinations are the explicit
    // subject of this isolated probe; all resource and lifetime obligations hold.
    let mut runtime = unsafe { Runtime::load(loader) }.unwrap();
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    options.set_high_dynamic_range(true);
    options.set_input_color_space(space);
    if automatic {
        options.set_exposure_mode(ExposureMode::Automatic);
    }
    let estimate = unsafe { runtime.upscaler_dx12(&device) }
        .estimate_memory(&options)
        .unwrap();
    let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
        .create(&options)
        .unwrap();
    log_provider(&mut upscaler);
    eprintln!(
        "COLOR_CONTEXT flags={} encoding={mode} automatic={automatic} estimate={estimate:?}",
        options.native_flags()
    );
    let mut retained = Vec::new();
    for (index, jitter) in [(0.25, -0.25), (-0.25, 0.25), (0.125, -0.125)]
        .into_iter()
        .enumerate()
    {
        let pixels = std::fs::read(inputs.join(format!("frame-{index}.rgba16f"))).unwrap();
        assert_eq!(pixels.len(), 320 * 180 * 8);
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut resources = [null_mut(); 5];
        // SAFETY: Fresh recording allocator/list/textures transfer owned COM references.
        assert_eq!(
            unsafe {
                create(
                    device.as_raw(),
                    &mut harness,
                    &mut raw_list,
                    resources.as_mut_ptr(),
                )
            },
            0
        );
        assert!(
            !harness.is_null() && !raw_list.is_null() && resources.iter().all(|r| !r.is_null())
        );
        // SAFETY: Exactly sized upload bytes remain valid during synchronous copying.
        assert_eq!(
            unsafe {
                upload(
                    harness,
                    pixels.as_ptr(),
                    pixels.len() as u32,
                    exposure[index],
                )
            },
            0
        );
        // SAFETY: Each interface pointer carries an independent owned COM reference.
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources = resources.map(|r| unsafe { ID3D12Resource::from_raw(r) });
        let bundle = if automatic {
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
        let mut frame = UpscaleTestInputs::new(
            &list,
            bundle,
            JitterOffsetPixels::new(jitter.0, jitter.1).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0).unwrap(),
            reset[index],
        );
        frame.set_pre_exposure(PreExposure::new(pre[index]).unwrap());
        frame.set_color_encoding(crate::upscaler::ColorEncodingRequest::checked(hint));
        eprintln!(
            "COLOR_FRAME index={index} pre={} exposure={} reset={} hint={hint:?}",
            pre[index], exposure[index], reset[index]
        );
        // SAFETY: Inputs enter compute-read, output UAV; fresh lists submit once,
        // all GPU dependencies survive fencing and context teardown below.
        if let Err(error) =
            unsafe { upscaler.record(frame.command_list, &frame.resources, &frame.parameters) }
        {
            abort_dispatch(error, index as u32);
        }
        let mut metrics = Metrics::default();
        // SAFETY: The helper records readbacks, submits once and fences completion.
        let hr = unsafe { finish(harness, &mut metrics) };
        if hr != 0 {
            eprintln!("COLOR_SIGNAL submission/fence/readback failure: {hr:#010x}");
            // Completion is not established. Exit the isolated process without
            // unwinding context/resource owners into potentially in-flight work.
            std::process::exit(1);
        }
        assert_eq!(
            (metrics.finite_pixels, metrics.changed_pixels),
            (230_400, 230_400)
        );
        assert_eq!((metrics.debug_errors, metrics.device_removed), (0, 0));
        let mut payload = vec![0u8; 640 * 360 * 8];
        // SAFETY: Fenced readback with exactly sized writable output storage.
        assert_eq!(
            unsafe { copy(harness, payload.as_mut_ptr(), payload.len() as u32) },
            0
        );
        std::fs::write(outputs.join(format!("frame-{index}.rgba16f")), payload).unwrap();
        retained.push((harness, list, resources));
    }
    upscaler.destroy().unwrap();
    // SAFETY: All work was fenced; resources and device remain live after context teardown.
    assert_eq!(unsafe { checkpoint(device.as_raw()) }, 0);
    for (harness, list, resources) in retained {
        drop(resources);
        drop(list);
        // SAFETY: No GPU work or Rust resource borrows remain for this harness.
        unsafe { destroy(harness) };
    }
    // SAFETY: Device/runtime/helper still live after releasing all frame dependencies.
    assert_eq!(unsafe { checkpoint(device.as_raw()) }, 0);
    eprintln!("COLOR_SIGNAL PASSED frames=3");
}

#[test]
fn environment_aliases_accept_either_spelling_and_matching_values() {
    for (current, legacy) in [
        (Some("pq"), None),
        (None, Some("pq")),
        (Some("pq"), Some("pq")),
    ] {
        assert_eq!(
            resolve_variable(
                "FSR_SDK_COLOR_ENCODING",
                current.map(Into::into),
                legacy.map(Into::into)
            ),
            "pq"
        );
    }
}

#[test]
#[should_panic(expected = "conflicting current and legacy values")]
fn environment_aliases_reject_conflicting_values() {
    resolve_variable(
        "FSR_SDK_COLOR_ENCODING",
        Some("pq".into()),
        Some("srgb".into()),
    );
}

#[test]
#[should_panic(expected = "missing FSR_SDK_COLOR_ENCODING")]
fn environment_aliases_reject_missing_values() {
    resolve_variable("FSR_SDK_COLOR_ENCODING", None, None);
}
