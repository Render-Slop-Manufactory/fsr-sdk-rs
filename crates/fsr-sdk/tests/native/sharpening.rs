// SPDX-License-Identifier: MPL-2.0

//! Test-only D014 spike: inspect normally, then change only native sharpening.
//! No proposed public frame API is implemented by this experiment.
use super::*;

type SetFrame = unsafe extern "C" fn(*mut c_void, u32, f32, f32) -> i32;
type CopyBits = unsafe extern "C" fn(*mut c_void, *mut u16, u32) -> i32;
type CopyRed = unsafe extern "C" fn(*mut c_void, *mut f32, u32) -> i32;
const PIXELS: usize = 640 * 360;
const JITTER: [(f32, f32); 8] = [
    (0.25, -0.25),
    (-0.25, 0.25),
    (0.125, -0.375),
    (-0.375, 0.125),
    (0.375, -0.125),
    (-0.125, 0.375),
    (0.0625, -0.0625),
    (-0.0625, 0.0625),
];

struct Image {
    bits: Vec<u16>,
    red: Vec<f32>,
}

struct Equipment {
    _library: libloading::Library,
    device: ID3D12Device,
    runtime: Runtime,
    create: CreateHarness,
    set_frame: SetFrame,
    finish: Finish,
    copy_bits: CopyBits,
    copy_red: CopyRed,
    destroy: Destroy,
    check: DeviceCheck,
}

impl Equipment {
    fn acquire() -> Self {
        let helper = std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap();
        let loader = std::env::var_os("FSR_SDK_TEST_DLL").unwrap();
        // SAFETY: Opt-in runner builds this helper, verifies/stages trusted AMD
        // binaries and excludes GBV. Signatures match the paired C exports.
        unsafe {
            let library = libloading::Library::new(helper).unwrap();
            let debug = *library.get::<EnableDebug>(b"gpu_enable_debug\0").unwrap();
            assert_eq!(debug(), 0);
            let create = *library
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap();
            let mut raw = null_mut();
            assert_eq!(create(&mut raw), 0);
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            let amd = *library.get::<DeviceCheck>(b"gpu_require_amd\0").unwrap();
            assert_eq!(amd(device.as_raw()), 0);
            Self {
                create: *library.get::<CreateHarness>(b"gpu_create\0").unwrap(),
                set_frame: *library.get::<SetFrame>(b"sharpening_set_frame\0").unwrap(),
                finish: *library.get::<Finish>(b"gpu_finish\0").unwrap(),
                copy_bits: *library
                    .get::<CopyBits>(b"sharpening_copy_red_bits\0")
                    .unwrap(),
                copy_red: *library.get::<CopyRed>(b"semantic_copy_red\0").unwrap(),
                destroy: *library.get::<Destroy>(b"gpu_destroy\0").unwrap(),
                check: *library.get::<DeviceCheck>(b"gpu_check_device\0").unwrap(),
                runtime: Runtime::load(loader).unwrap(),
                device,
                _library: library,
            }
        }
    }

    fn sequence(&mut self, scene: u32, name: &str, sharpness: Option<f32>) -> Image {
        let mut context = unsafe { self.runtime.upscaler_dx12(&self.device) }
            .create(&UpscalerOptions::new(
                Dimensions::new(320, 180).unwrap(),
                Dimensions::new(640, 360).unwrap(),
            ))
            .unwrap();
        log_provider(&mut context);
        let mut frames = Vec::new();
        let mut image = Image {
            bits: vec![0; PIXELS],
            red: vec![0.0; PIXELS],
        };
        for (frame, &(jx, jy)) in JITTER.iter().enumerate() {
            let mut handle = null_mut();
            let mut raw_list = null_mut();
            let mut raw_resources = [null_mut(); 5];
            // SAFETY: Writable outputs receive an owned harness and extra COM
            // references. The upload is rewritten before any GPU submission.
            unsafe {
                assert_eq!(
                    (self.create)(
                        self.device.as_raw(),
                        &mut handle,
                        &mut raw_list,
                        raw_resources.as_mut_ptr()
                    ),
                    0
                );
                assert!(!handle.is_null() && !raw_list.is_null());
                assert_eq!((self.set_frame)(handle, scene, jx, jy), 0);
            }
            // SAFETY: The helper transfers a live reference for each output.
            let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
            let resources: Vec<_> = raw_resources
                .into_iter()
                .map(|raw| {
                    assert!(!raw.is_null());
                    // SAFETY: This is one owned reference from gpu_create.
                    unsafe { ID3D12Resource::from_raw(raw) }
                })
                .collect();
            let dispatch = UpscaleTestInputs::new(
                &list,
                UpscaleResources::builder()
                    .color(&resources[0])
                    .depth(&resources[1])
                    .motion_vectors(&resources[2])
                    .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                    .output(&resources[4])
                    .build()
                    .unwrap(),
                JitterOffsetPixels::new(jx, jy).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                frame == 0,
            );
            // SAFETY: Normal preflight and owner/poison orchestration are shared
            // with public dispatch. Only the two already-bound native fields
            // differ. Values are fixed finite [0,1] constants; all dependencies
            // survive the fence and context destruction. Boundary states are
            // checked by readback barriers and the ordinary DX12 debug layer.
            let result = unsafe {
                context.dispatch_core(|this| {
                    let mut desc = super::super::dispatch::inspect_dispatch(
                        &self.device,
                        &this.options,
                        dispatch,
                    )?;
                    desc.enableSharpening = sharpness.is_some();
                    desc.sharpness = sharpness.unwrap_or(0.0);
                    Ok(desc)
                })
            };
            if let Err(error) = result {
                abort_dispatch(error, frame as u32);
            }
            let mut metrics = Metrics::default();
            // SAFETY: Finish records readback, submits once and fences the frame.
            let hr = unsafe { (self.finish)(handle, &mut metrics) };
            if hr < 0 {
                eprintln!("Sharpening fence/readback failure: {hr:#x}");
                std::process::exit(1); // Preserve possibly in-flight dependencies.
            }
            eprintln!(
                "SHARPEN_FRAME scene={scene} setting={name} frame={frame} finite={} changed={} debug_errors={} device_removed={:#x}",
                metrics.finite_pixels,
                metrics.changed_pixels,
                metrics.debug_errors,
                metrics.device_removed
            );
            assert_eq!((metrics.width, metrics.height), (640, 360));
            assert_eq!(metrics.finite_pixels as usize, PIXELS);
            assert_eq!(metrics.changed_pixels as usize, PIXELS);
            assert_eq!(metrics.debug_errors, 0);
            assert_eq!(metrics.device_removed, 0);
            if frame == 7 {
                // SAFETY: Completed readback and exact-sized writable buffers.
                unsafe {
                    assert_eq!(
                        (self.copy_bits)(handle, image.bits.as_mut_ptr(), PIXELS as u32),
                        0
                    );
                    assert_eq!(
                        (self.copy_red)(handle, image.red.as_mut_ptr(), PIXELS as u32),
                        0
                    );
                }
            }
            frames.push((handle, list, resources));
        }
        context.destroy().unwrap();
        for (handle, list, resources) in frames {
            drop(resources);
            drop(list);
            // SAFETY: All submitted work completed and context was destroyed.
            unsafe { (self.destroy)(handle) };
        }
        // SAFETY: Device remains live after dependent teardown.
        assert_eq!(unsafe { (self.check)(self.device.as_raw()) }, 0);
        eprintln!("SHARPEN_SEQUENCE scene={scene} setting={name} frames=8 cleanup=passed");
        if let Some(directory) = std::env::var_os("FSR_SDK_TEST_SHARPENING_OUTPUT") {
            let path = PathBuf::from(directory).join(format!("scene-{scene}-{name}.red.u16le"));
            let bytes: Vec<_> = image
                .bits
                .iter()
                .flat_map(|bits| bits.to_le_bytes())
                .collect();
            std::fs::write(path, bytes).unwrap();
        }
        image
    }
}

fn roi() -> impl Iterator<Item = usize> {
    (32..328).flat_map(|y| (32..608).map(move |x| y * 640 + x))
}

fn gradient(image: &Image) -> f64 {
    let mut sum = 0.0;
    for y in 32..328 {
        for x in 32..607 {
            let i = y * 640 + x;
            sum += (f64::from(image.red[i + 1]) - f64::from(image.red[i])).abs();
        }
    }
    sum / (296.0 * 575.0)
}

fn metrics(scene: u32, name: &str, image: &Image, off: &Image, g_off: f64) -> bool {
    let count = 296.0 * 576.0;
    let mut diff = 0.0;
    let mut changed = 0;
    let mut shift = 0.0;
    for i in roi() {
        let delta = f64::from(image.red[i]) - f64::from(off.red[i]);
        diff += delta.abs();
        changed += u32::from(delta.abs() >= 1.0 / 1024.0);
        shift += delta;
    }
    let d = diff / count;
    let c = f64::from(changed) / count;
    let g = gradient(image);
    let b = (shift / count).abs();
    let passed = d >= 0.001 && c >= 0.01 && g_off > 0.0 && g >= 1.01 * g_off && b <= 0.005;
    eprintln!(
        "SHARPEN_METRIC scene={scene} setting={name} D={d:.10} C={c:.10} G={g:.10} B={b:.10} G_off={g_off:.10} criteria={passed}"
    );
    passed
}

// Pixel centres use output coordinates. Bilinear sampling along the analytic
// edge normal averages 64 profiles with centres Y=80..280, X=256+0.37Y.
fn sample(image: &Image, x: f64, y: f64) -> f64 {
    let x = x - 0.5;
    let y = y - 0.5;
    let ix = x.floor() as usize;
    let iy = y.floor() as usize;
    let tx = x - ix as f64;
    let ty = y - iy as f64;
    let a = f64::from(image.red[iy * 640 + ix]) * (1.0 - tx)
        + f64::from(image.red[iy * 640 + ix + 1]) * tx;
    let b = f64::from(image.red[(iy + 1) * 640 + ix]) * (1.0 - tx)
        + f64::from(image.red[(iy + 1) * 640 + ix + 1]) * tx;
    a * (1.0 - ty) + b * ty
}

fn edge_diagnostics(name: &str, image: &Image) {
    let norm = (1.0_f64 + 0.37 * 0.37).sqrt();
    let profile: Vec<_> = (0..65)
        .map(|step| {
            let distance = -8.0 + f64::from(step) * 0.25;
            let sum: f64 = (0..64)
                .map(|i| {
                    let y = 80.0 + f64::from(i) * 200.0 / 63.0;
                    sample(
                        image,
                        256.0 + 0.37 * y + distance / norm,
                        y - 0.37 * distance / norm,
                    )
                })
                .sum();
            (distance, sum / 64.0)
        })
        .collect();
    let crossing = |level: f64| {
        profile.windows(2).find_map(|pair| {
            let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
            (y0 < level && y1 >= level).then(|| x0 + (level - y0) * (x1 - x0) / (y1 - y0))
        })
    };
    let width = crossing(0.18).zip(crossing(0.82)).map(|(a, b)| b - a);
    let min = roi()
        .map(|i| f64::from(image.red[i]))
        .fold(f64::INFINITY, f64::min);
    let max = roi()
        .map(|i| f64::from(image.red[i]))
        .fold(f64::NEG_INFINITY, f64::max);
    eprintln!(
        "SHARPEN_EDGE setting={name} width_10_90={width:?} min={min:.10} max={max:.10} undershoot={:.10} overshoot={:.10}",
        (0.1 - min).max(0.0),
        (max - 0.9).max(0.0)
    );
    for (distance, red) in profile {
        eprintln!("SHARPEN_PROFILE setting={name} distance={distance:.2} red={red:.10}");
    }
}

pub(super) fn compare() {
    let mut equipment = Equipment::acquire();
    let mut effect_passed = true;
    for scene in 0..2 {
        let off = equipment.sequence(scene, "off_a", None);
        let repeat = equipment.sequence(scene, "off_b", None);
        let zero = equipment.sequence(scene, "zero", Some(0.0));
        let medium = equipment.sequence(scene, "0.35", Some(0.35));
        let maximum = equipment.sequence(scene, "1", Some(1.0));
        let mismatches = roi().filter(|&i| off.bits[i] != repeat.bits[i]).count();
        let noise = roi()
            .map(|i| (f64::from(off.red[i]) - f64::from(repeat.red[i])).abs())
            .sum::<f64>()
            / (296.0 * 576.0);
        eprintln!(
            "SHARPEN_REPEAT scene={scene} bit_mismatches={mismatches} N={noise:.10} policy=bit_identical"
        );
        // Complete/report all settings and both scenes before failing an effect
        // criterion. An execution failure remains immediately terminal.
        effect_passed &= mismatches == 0;
        let g_off = gradient(&off);
        metrics(scene, "zero", &zero, &off, g_off);
        let medium_passed = metrics(scene, "0.35", &medium, &off, g_off);
        let maximum_passed = metrics(scene, "1", &maximum, &off, g_off);
        if scene == 0 {
            effect_passed &= medium_passed && maximum_passed;
        } else {
            for (name, image) in [
                ("off_a", &off),
                ("off_b", &repeat),
                ("zero", &zero),
                ("0.35", &medium),
                ("1", &maximum),
            ] {
                edge_diagnostics(name, image);
            }
        }
    }
    assert!(
        effect_passed,
        "predeclared sharpening criteria failed; retain original results"
    );
    eprintln!("Sharpening gate: PASSED");
}
