// SPDX-License-Identifier: MPL-2.0

//! Bounded temporal calibration probe. Python owns protocol, orchestration and oracles.
use super::*;
use std::{fs, io::Write};

type SetFrame = unsafe extern "C" fn(*mut c_void, u32, u32, f32, f32, u32, u32) -> i32;
type RecordInputs = unsafe extern "C" fn(*mut c_void) -> i32;
type CopyBytes = unsafe extern "C" fn(*mut c_void, *mut u8, u32) -> i32;
type Fence = unsafe extern "C" fn(*mut c_void) -> u64;
type Memory = unsafe extern "C" fn(*mut c_void);

pub(super) struct Equipment {
    _library: libloading::Library,
    pub(super) device: ID3D12Device,
    pub(super) create: CreateHarness,
    set: SetFrame,
    pub(super) record: RecordInputs,
    pub(super) copy_inputs: CopyBytes,
    pub(super) copy_output: CopyBytes,
    pub(super) finish: Finish,
    pub(super) fence: Fence,
    pub(super) destroy: Destroy,
    pub(super) check: DeviceCheck,
    pub(super) memory: Memory,
}

impl Equipment {
    pub(super) fn acquire() -> Self {
        Self::acquire_with_diagnostics(false)
    }

    pub(super) fn acquire_with_diagnostics(proton: bool) -> Self {
        // SAFETY: The coordinator compiles the trusted helper. Signatures match
        // its C exports; the library survives all functions and COM objects.
        unsafe {
            let library =
                libloading::Library::new(std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap())
                    .unwrap();
            let debug_hr = library.get::<EnableDebug>(b"gpu_enable_debug\0").unwrap()();
            eprintln!("Temporal DirectX debug: HRESULT={debug_hr:#010x} proton={proton}");
            assert!(proton || debug_hr == 0);
            let mut raw = null_mut();
            assert_eq!(
                library
                    .get::<CreateDevice>(b"probe_create_device\0")
                    .unwrap()(&mut raw),
                0
            );
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            assert_eq!(
                library.get::<DeviceCheck>(b"gpu_require_amd\0").unwrap()(device.as_raw()),
                0
            );
            Self {
                create: *library.get(b"gpu_create\0").unwrap(),
                set: *library.get(b"temporal_calibration_set_frame\0").unwrap(),
                record: *library
                    .get(b"temporal_calibration_record_inputs\0")
                    .unwrap(),
                copy_inputs: *library.get(b"temporal_calibration_copy_inputs\0").unwrap(),
                copy_output: *library.get(b"gpu_copy_payload\0").unwrap(),
                finish: *library.get(b"gpu_finish\0").unwrap(),
                fence: *library.get(b"gpu_fence_value\0").unwrap(),
                destroy: *library.get(b"gpu_destroy\0").unwrap(),
                check: *library
                    .get(if proton {
                        b"gpu_check_device_removed\0" as &[u8]
                    } else {
                        b"gpu_check_device\0"
                    })
                    .unwrap(),
                memory: *library.get(b"temporal_calibration_memory\0").unwrap(),
                device,
                _library: library,
            }
        }
    }
}

pub(super) fn run(input_only: bool) {
    let output =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_TEMPORAL_CALIBRATION_OUTPUT").unwrap());
    let jitter: Vec<[f32; 2]> =
        fs::read_to_string(std::env::var_os("FSR_SDK_TEST_TEMPORAL_CALIBRATION_JITTER").unwrap())
            .unwrap()
            .lines()
            .map(|line| {
                let values: Vec<f32> = line
                    .split_whitespace()
                    .map(|v| v.parse().unwrap())
                    .collect();
                assert_eq!(values.len(), 2);
                [values[0], values[1]]
            })
            .collect();
    assert_eq!(jitter.len(), 32);
    let equipment = Equipment::acquire();
    // The input-only child does not load or call AMD's runtime.
    let mut runtime = (!input_only).then(|| {
        // SAFETY: The coordinator hash/signature-checks and stages SDK v2.3.0.
        unsafe { Runtime::load(std::env::var_os("FSR_SDK_TEST_DLL").unwrap()) }.unwrap()
    });
    let arms: &[(&str, u32, bool, bool, bool, bool)] = if input_only {
        &[
            ("input-a", 0, false, false, false, false),
            ("input-b", 1, false, false, false, false),
            ("input-cut", 1, false, false, true, false),
        ]
    } else {
        &[
            ("a-correct", 0, false, false, false, false),
            ("b-correct", 1, false, false, false, false),
            ("b-reverse-motion", 1, true, false, false, false),
            ("b-negated-jitter", 1, false, true, false, false),
            ("cut-history", 1, false, false, true, false),
            ("cut-fresh", 1, false, false, true, true),
        ]
    };
    let mut metadata = fs::File::create(output.join("frames.jsonl")).unwrap();
    let mut count = 0;
    for &(arm, scene, reverse, wrong_jitter, cut, fresh) in arms {
        let mut context = runtime.as_mut().map(|runtime| {
            let mut context = unsafe { runtime.upscaler_dx12(&equipment.device) }
                .create(&UpscalerOptions::new(
                    Dimensions::new(320, 180).unwrap(),
                    Dimensions::new(640, 360).unwrap(),
                ))
                .unwrap();
            log_provider(&mut context);
            context
        });
        let frames: Vec<u32> = if input_only {
            vec![0, 1, 31, 32, 37, 40, 63]
        } else if fresh {
            (37..=40).collect()
        } else if cut {
            (0..=40).collect()
        } else {
            (0..64).collect()
        };
        let mut retained = Vec::new();
        for frame in frames {
            let [jx, jy] = jitter[frame as usize % 32];
            let (dx, dy) = if wrong_jitter { (-jx, -jy) } else { (jx, jy) };
            let reset = frame == 0 || (cut && frame == 37);
            let mut handle = null_mut();
            let mut raw_list = null_mut();
            let mut raw_resources = [null_mut(); 5];
            // SAFETY: Writable outputs receive owned helper/COM references.
            // Upload rewriting and input copies precede this frame's submission.
            unsafe {
                assert_eq!(
                    (equipment.create)(
                        equipment.device.as_raw(),
                        &mut handle,
                        &mut raw_list,
                        raw_resources.as_mut_ptr()
                    ),
                    0
                );
                assert!(!handle.is_null() && !raw_list.is_null());
                assert_eq!(
                    (equipment.set)(
                        handle,
                        scene,
                        frame,
                        jx,
                        jy,
                        u32::from(reverse),
                        u32::from(cut)
                    ),
                    0
                );
                assert_eq!((equipment.record)(handle), 0);
            }
            // SAFETY: Helper outputs transfer one live COM reference each.
            let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
            let resources: Vec<_> = raw_resources
                .into_iter()
                .map(|raw| {
                    assert!(!raw.is_null());
                    // SAFETY: Each output owns one transferred reference.
                    unsafe { ID3D12Resource::from_raw(raw) }
                })
                .collect();
            eprintln!(
                "Temporal calibration begin: arm={arm} frame={frame} phase={}",
                frame % 32
            );
            if let Some(context) = context.as_mut() {
                // SAFETY: All resource states and contents follow the protocol.
                // Dependencies survive submission, its fence and context teardown.
                if let Err(error) = unsafe {
                    let recording = UpscaleTestInputs::new(
                        &list,
                        UpscaleResources::builder()
                            .color(&resources[0])
                            .depth(&resources[1])
                            .motion_vectors(&resources[2])
                            .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                            .output(&resources[4])
                            .build()
                            .unwrap(),
                        JitterOffsetPixels::new(dx, dy).unwrap(),
                        FrameTimeMillis::new(1000.0 / 60.0).unwrap(),
                        CameraParameters::new(0.5, 20.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                        reset,
                    );
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                } {
                    abort_dispatch(error, frame);
                }
            }
            let mut metrics = Metrics::default();
            // SAFETY: Paired helper submits this frame and waits for its fence.
            let hr = unsafe { (equipment.finish)(handle, &mut metrics) };
            if hr < 0 {
                eprintln!(
                    "Temporal calibration uncertain completion: arm={arm} frame={frame} hr={hr:#x}"
                );
                std::process::exit(1);
            }
            // SAFETY: The fence query uses the retained helper object.
            let fence = unsafe { (equipment.fence)(handle) };
            if fence != 1 || metrics.device_removed != 0 {
                eprintln!(
                    "Temporal calibration unconfirmed completion: fence={fence} device={}",
                    metrics.device_removed
                );
                std::process::exit(1);
            }
            let mut inputs = vec![0u8; 320 * 180 * 16 + 4];
            // SAFETY: GPU completion precedes readback into exact-size buffers.
            assert_eq!(
                unsafe {
                    (equipment.copy_inputs)(handle, inputs.as_mut_ptr(), inputs.len() as u32)
                },
                0
            );
            let stem = format!("{arm}-{frame:03}");
            fs::write(output.join(format!("{stem}.inputs")), inputs).unwrap();
            if !input_only {
                let mut payload = vec![0u8; 640 * 360 * 8];
                // SAFETY: Same completed frame, exact RGBA16F byte count.
                assert_eq!(
                    unsafe {
                        (equipment.copy_output)(handle, payload.as_mut_ptr(), payload.len() as u32)
                    },
                    0
                );
                fs::write(output.join(format!("{stem}.rgba16f")), payload).unwrap();
            }
            writeln!(metadata, "{{\"arm\":\"{arm}\",\"scene\":{scene},\"frame\":{frame},\"phase\":{},\"image_jitter\":[{jx},{jy}],\"dispatch_jitter\":[{dx},{dy}],\"reverse_motion\":{reverse},\"camera_cut\":{cut},\"camera_xy\":[{},{}],\"reset\":{reset},\"camera\":[0.5,20,{},1],\"motion_scale\":[320,180],\"exposure\":1,\"pre_exposure\":1,\"flags\":0,\"masks\":false,\"sharpening\":false,\"frame_ms\":{},\"dispatch\":\"{}\",\"finish_hr\":{hr},\"fence\":{fence},\"finite_pixels\":{},\"changed_pixels\":{},\"debug_errors\":{},\"device_removed\":{}}}",
                frame % 32,
                if cut && frame >= 37 { 192.0 / (90.0 / (std::f64::consts::PI / 6.0).tan()) } else { 0.0 },
                if cut && frame >= 37 { 96.0 / (90.0 / (std::f64::consts::PI / 6.0).tan()) } else { 0.0 },
                std::f32::consts::FRAC_PI_3, 1000.0f32 / 60.0,
                if input_only { "not_run" } else { "ok" }, metrics.finite_pixels,
                metrics.changed_pixels, metrics.debug_errors, metrics.device_removed).unwrap();
            metadata.flush().unwrap();
            assert_eq!(metrics.debug_errors, 0);
            assert_eq!(metrics.finite_pixels, 640 * 360);
            if !input_only {
                assert_eq!(metrics.changed_pixels, 640 * 360);
            }
            retained.push((handle, list, resources));
            count += 1;
        }
        eprintln!(
            "Temporal calibration retained memory: arm={arm} frames={}",
            retained.len()
        );
        // SAFETY: Device and all frame equipment remain live after their fences.
        unsafe { (equipment.memory)(equipment.device.as_raw()) };
        if let Some(context) = context.take()
            && let Err(error) = context.destroy()
        {
            eprintln!("Temporal calibration destroy failure: {error:?}");
            std::process::exit(1);
        }
        for (handle, list, resources) in retained {
            drop(resources);
            drop(list);
            // SAFETY: Every GPU fence and context destruction precedes release.
            unsafe { (equipment.destroy)(handle) };
        }
        // SAFETY: Device is retained after all dependent equipment is released.
        unsafe {
            assert_eq!((equipment.check)(equipment.device.as_raw()), 0);
            (equipment.memory)(equipment.device.as_raw());
        }
        eprintln!("Temporal calibration cleanup complete: arm={arm}");
    }
    assert_eq!(count, if input_only { 21 } else { 301 });
    eprintln!(
        "Temporal calibration {}: COMPLETED frames={count}",
        if input_only {
            "input-only"
        } else {
            "calibration"
        }
    );
}
