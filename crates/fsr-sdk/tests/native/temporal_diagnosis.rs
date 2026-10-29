// SPDX-License-Identifier: MPL-2.0

//! Fixed diagnostic matrix. Python freezes the arms and independently audits inputs.
use super::{temporal_calibration::Equipment, *};
use std::{fs, io::Write};

type SetFrame = unsafe extern "C" fn(*mut c_void, u32, u32, f32, f32, f32, f32, u32) -> i32;
type SetCutFrame = unsafe extern "C" fn(*mut c_void, u32, u32, f32, f32, f32, f32, u32, u32) -> i32;

struct Arm {
    name: String,
    scene: u32,
    vx: f32,
    vy: f32,
    motion: u32,
    jitter: i32,
    reset_all: bool,
    start: u32,
    reset_frame: i32,
    jitter_phase: u32,
    jitter_x: i32,
    jitter_y: i32,
    dispatch_lag: u32,
}

pub(super) fn run(input_only: bool) {
    run_with_diagnostics(input_only, false);
}

pub(super) fn run_with_diagnostics(input_only: bool, proton: bool) {
    let frame_count: u32 = std::env::var("FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_FRAMES")
        .map(|value| value.parse().expect("Invalid temporal frame count"))
        .unwrap_or(64);
    assert!((64..=256).contains(&frame_count) && frame_count.is_multiple_of(32));
    let cut_frame: Option<u32> = std::env::var("FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_CUT_FRAME")
        .ok()
        .map(|value| value.parse().expect("Invalid cut frame"));
    if let Some(cut) = cut_frame {
        assert_eq!(frame_count, 128);
        assert_eq!(cut, 64);
    }
    let jitter_study = std::env::var("FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_JITTER_STUDY")
        .is_ok_and(|value| value == "1");
    assert!(!jitter_study || (frame_count == 128 && cut_frame.is_none()));
    let output = PathBuf::from(std::env::var_os("FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_OUTPUT").unwrap());
    let table =
        fs::read_to_string(std::env::var_os("FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_ARMS").unwrap())
            .unwrap();
    let arms: Vec<Arm> = table
        .lines()
        .map(|line| {
            let p: Vec<_> = line.split_whitespace().collect();
            assert_eq!(
                p.len(),
                if jitter_study {
                    11
                } else if cut_frame.is_some() {
                    9
                } else {
                    7
                }
            );
            Arm {
                name: p[0].to_owned(),
                scene: p[1].parse().unwrap(),
                vx: p[2].parse().unwrap(),
                vy: p[3].parse().unwrap(),
                motion: p[4].parse().unwrap(),
                jitter: p[5].parse().unwrap(),
                reset_all: p[6] == "1",
                start: if p.len() == 9 {
                    p[7].parse().unwrap()
                } else {
                    0
                },
                reset_frame: if p.len() == 9 {
                    p[8].parse().unwrap()
                } else {
                    -1
                },
                jitter_phase: if jitter_study {
                    p[7].parse().unwrap()
                } else {
                    0
                },
                jitter_x: p[if jitter_study { 8 } else { 5 }].parse().unwrap(),
                jitter_y: p[if jitter_study { 9 } else { 5 }].parse().unwrap(),
                dispatch_lag: if jitter_study {
                    p[10].parse().unwrap()
                } else {
                    0
                },
            }
        })
        .collect();
    assert!((1..=32).contains(&arms.len()));
    let frames: Vec<u32> = (0..frame_count)
        .filter(|frame| {
            !input_only
                || *frame <= 1
                || matches!(frame % 32, 0 | 31)
                || cut_frame.is_some_and(|cut| *frame == cut + 1)
                || (jitter_study
                    && arms
                        .iter()
                        .any(|arm| matches!((frame + arm.jitter_phase) % 32, 0 | 31)))
        })
        .collect();
    for arm in &arms {
        assert!(!arm.name.is_empty());
        assert!(
            arm.name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        );
        assert!(arm.vx.is_finite() && arm.vy.is_finite());
        assert!(arm.start < frame_count && (-1..frame_count as i32).contains(&arm.reset_frame));
        assert!(arm.jitter_phase < 32 && arm.dispatch_lag <= 1);
        assert!((-1..=1).contains(&arm.jitter_x) && (-1..=1).contains(&arm.jitter_y));
    }
    let jitter: Vec<[f32; 2]> =
        fs::read_to_string(std::env::var_os("FSR_SDK_TEST_TEMPORAL_CALIBRATION_JITTER").unwrap())
            .unwrap()
            .lines()
            .map(|line| {
                let p: Vec<f32> = line
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                assert_eq!(p.len(), 2);
                [p[0], p[1]]
            })
            .collect();
    assert_eq!(jitter.len(), 32);
    let equipment = Equipment::acquire_with_diagnostics(proton);
    // SAFETY: The staged helper has this paired C export and remains loaded.
    let library =
        unsafe { libloading::Library::new(std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap()) }
            .unwrap();
    // SAFETY: Exact signature matches temporal_diagnosis.h.
    let set = unsafe {
        *library
            .get::<SetFrame>(b"temporal_diagnosis_set_frame\0")
            .unwrap()
    };
    // SAFETY: Paired private export in the same retained helper library.
    let set_cut = cut_frame.map(|_| unsafe {
        *library
            .get::<SetCutFrame>(b"temporal_reset_set_frame\0")
            .unwrap()
    });
    let mut runtime = (!input_only).then(|| {
        // SAFETY: Coordinator checks the trusted SDK v2.3.0 deployment.
        unsafe { Runtime::load(std::env::var_os("FSR_SDK_TEST_DLL").unwrap()) }.unwrap()
    });
    let mut metadata = fs::File::create(output.join("frames.jsonl")).unwrap();
    let mut count = 0;
    for arm in &arms {
        assert!(arm.scene <= 2 && arm.motion <= 2 && (-1..=1).contains(&arm.jitter));
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
        let mut retained = Vec::new();
        for &frame in &frames {
            if frame < arm.start {
                continue;
            }
            let phase = (frame + arm.jitter_phase) % 32;
            let [jx, jy] = jitter[phase as usize];
            let [dispatch_x, dispatch_y] = jitter[((phase + 32 - arm.dispatch_lag) % 32) as usize];
            let (dx, dy) = (
                dispatch_x * arm.jitter_x as f32,
                dispatch_y * arm.jitter_y as f32,
            );
            let reset = frame == 0 || arm.reset_all || frame as i32 == arm.reset_frame;
            let mut handle = null_mut();
            let mut raw_list = null_mut();
            let mut raw_resources = [null_mut(); 5];
            // SAFETY: Writable outputs transfer helper/COM ownership; inputs are
            // rewritten and copied before submission of this fresh command list.
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
                    if let (Some(set_cut), Some(cut)) = (set_cut, cut_frame) {
                        set_cut(
                            handle, arm.scene, frame, jx, jy, arm.vx, arm.vy, arm.motion, cut,
                        )
                    } else {
                        set(handle, arm.scene, frame, jx, jy, arm.vx, arm.vy, arm.motion)
                    },
                    0
                );
                assert_eq!((equipment.record)(handle), 0);
            }
            // SAFETY: Helper transfers owned live COM interfaces.
            let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
            let resources: Vec<_> = raw_resources
                .into_iter()
                .map(|raw| {
                    assert!(!raw.is_null());
                    // SAFETY: Each output transfers one live COM reference.
                    unsafe { ID3D12Resource::from_raw(raw) }
                })
                .collect();
            eprintln!("Temporal diagnosis begin: arm={} frame={frame}", arm.name);
            if let Some(context) = context.as_mut() {
                // SAFETY: States and lifetimes follow the existing fenced profile;
                // motion/jitter/reset variants intentionally perturb image semantics.
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
            // SAFETY: Paired helper submits and fences before readback.
            let hr = unsafe { (equipment.finish)(handle, &mut metrics) };
            if hr < 0 {
                eprintln!("Temporal diagnosis uncertain completion: hr={hr:#x}");
                std::process::exit(1);
            }
            // SAFETY: Retained helper owns the fence.
            let fence = unsafe { (equipment.fence)(handle) };
            if fence != 1 || metrics.device_removed != 0 {
                std::process::exit(1);
            }
            let mut inputs = vec![0u8; 320 * 180 * 16 + 4];
            // SAFETY: Exact buffer size and CPU-observed GPU completion.
            assert_eq!(
                unsafe {
                    (equipment.copy_inputs)(handle, inputs.as_mut_ptr(), inputs.len() as u32)
                },
                0
            );
            let stem = format!("{}-{frame:03}", arm.name);
            fs::write(output.join(format!("{stem}.inputs")), inputs).unwrap();
            if !input_only {
                let mut pixels = vec![0u8; 640 * 360 * 8];
                // SAFETY: Same completed frame and exact RGBA16F size.
                assert_eq!(
                    unsafe {
                        (equipment.copy_output)(handle, pixels.as_mut_ptr(), pixels.len() as u32)
                    },
                    0
                );
                fs::write(output.join(format!("{stem}.rgba16f")), pixels).unwrap();
            }
            let cut_metadata =
                cut_frame.map_or(String::new(), |cut| format!(",\"cut_frame\":{cut}"));
            let jitter_metadata = if jitter_study {
                format!(
                    ",\"jitter_phase\":{},\"dispatch_lag\":{}",
                    arm.jitter_phase, arm.dispatch_lag
                )
            } else {
                String::new()
            };
            writeln!(metadata,"{{\"arm\":\"{}\",\"scene\":{},\"frame\":{frame},\"velocity\":[{},{}],\"motion\":{},\"image_jitter\":[{jx},{jy}],\"dispatch_jitter\":[{dx},{dy}],\"reset\":{reset},\"finish_hr\":{hr},\"fence\":{fence},\"debug_errors\":{},\"device_removed\":{},\"finite_pixels\":{},\"changed_pixels\":{},\"dispatch\":\"{}\"{cut_metadata}{jitter_metadata}}}",
                arm.name,arm.scene,arm.vx,arm.vy,arm.motion,metrics.debug_errors,metrics.device_removed,
                metrics.finite_pixels,metrics.changed_pixels,if input_only { "not_run" } else { "ok" }).unwrap();
            metadata.flush().unwrap();
            assert!(metrics.debug_errors == 0 || (proton && metrics.debug_errors == u64::MAX));
            assert_eq!(metrics.finite_pixels, 640 * 360);
            if !input_only {
                assert_eq!(metrics.changed_pixels, 640 * 360);
            }
            retained.push((handle, list, resources));
            count += 1;
        }
        // SAFETY: All frame equipment and the device remain live after fences.
        unsafe { (equipment.memory)(equipment.device.as_raw()) };
        if let Some(context) = context.take()
            && let Err(error) = context.destroy()
        {
            eprintln!("Temporal diagnosis destroy failure: {error:?}");
            std::process::exit(1);
        }
        for (handle, list, resources) in retained {
            drop(resources);
            drop(list);
            // SAFETY: Fences and context destruction precede equipment release.
            unsafe { (equipment.destroy)(handle) };
        }
        // SAFETY: Device outlives every dependent context and resource.
        assert_eq!(unsafe { (equipment.check)(equipment.device.as_raw()) }, 0);
        eprintln!("Temporal diagnosis cleanup complete: arm={}", arm.name);
    }
    assert_eq!(
        count,
        arms.iter()
            .map(|arm| frames.iter().filter(|frame| **frame >= arm.start).count())
            .sum::<usize>()
    );
    eprintln!(
        "Temporal diagnosis {}: COMPLETED frames={count}",
        if input_only { "input-only" } else { "matrix" }
    );
}
