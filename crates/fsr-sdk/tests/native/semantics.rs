// SPDX-License-Identifier: MPL-2.0

//! Controlled comparisons, rather than a general image-quality oracle.
use super::*;

type SetFrame = unsafe extern "C" fn(*mut c_void, u32, f32, f32, u32) -> i32;
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

struct Equipment {
    _library: libloading::Library,
    device: ID3D12Device,
    runtime: Runtime,
    create: CreateHarness,
    set_frame: SetFrame,
    finish: Finish,
    copy: CopyRed,
    destroy: Destroy,
    check: DeviceCheck,
}

impl Equipment {
    fn acquire() -> Self {
        let helper_path = std::env::var_os("FSR_SDK_TEST_M6_DLL").unwrap();
        let loader = std::env::var_os("FSR_SDK_TEST_DLL").unwrap();
        // SAFETY: The opt-in runner compiles the trusted helper and stages the
        // pinned signed runtime. Every signature is paired with its C export.
        unsafe {
            let library = libloading::Library::new(helper_path).unwrap();
            let debug = *library.get::<EnableDebug>(b"m6_enable_debug\0").unwrap();
            assert_eq!(debug(), 0, "debug layer unavailable");
            let create = *library
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap();
            let mut raw = null_mut();
            assert_eq!(create(&mut raw), 0);
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            let amd = *library.get::<DeviceCheck>(b"m6_require_amd\0").unwrap();
            assert_eq!(amd(device.as_raw()), 0, "requires AMD hardware");
            Self {
                create: *library.get::<CreateHarness>(b"m6_create\0").unwrap(),
                set_frame: *library.get::<SetFrame>(b"semantic_set_frame\0").unwrap(),
                finish: *library.get::<Finish>(b"m6_finish\0").unwrap(),
                copy: *library.get::<CopyRed>(b"semantic_copy_red\0").unwrap(),
                destroy: *library.get::<Destroy>(b"m6_destroy\0").unwrap(),
                check: *library.get::<DeviceCheck>(b"m6_check_device\0").unwrap(),
                runtime: Runtime::load_unchecked(loader).unwrap(),
                device,
                _library: library,
            }
        }
    }

    fn sequence(
        &self,
        name: &str,
        reverse_motion: bool,
        wrong_jitter: bool,
        reset_last: bool,
        fresh_last: bool,
    ) -> Vec<f32> {
        let mut context = Upscaler::new(
            &self.runtime,
            &self.device,
            UpscalerOptions::new(
                Dimensions::new(320, 180).unwrap(),
                Dimensions::new(640, 360).unwrap(),
            ),
        )
        .unwrap();
        log_provider(&mut context);
        let mut frames = Vec::new();
        let mut result = vec![0.0; PIXELS];
        for (frame, &(jx, jy)) in JITTER
            .iter()
            .enumerate()
            .skip(if fresh_last { 7 } else { 0 })
        {
            let mut handle = null_mut();
            let mut raw_list = null_mut();
            let mut raw_resources = [null_mut(); 5];
            // SAFETY: Writable outputs receive an owned harness and extra COM
            // references. Upload rewriting precedes submission of this frame.
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
                assert_eq!(
                    (self.set_frame)(handle, frame as u32, jx, jy, u32::from(reverse_motion)),
                    0
                );
            }
            // SAFETY: Each helper output transfers one live COM reference.
            let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
            let resources: Vec<_> = raw_resources
                .into_iter()
                .map(|raw| {
                    assert!(!raw.is_null());
                    // SAFETY: The helper transfers one reference per resource.
                    unsafe { ID3D12Resource::from_raw(raw) }
                })
                .collect();
            let reset = frame == 0 || fresh_last || (reset_last && frame == 7);
            // SAFETY: Upload copies and transitions are already recorded;
            // this context and every external dependency survive the fence.
            let dispatch = unsafe {
                context.dispatch(UpscaleDispatch {
                    command_list: &list,
                    color: &resources[0],
                    depth: &resources[1],
                    motion_vectors: &resources[2],
                    exposure: &resources[3],
                    output: &resources[4],
                    jitter: JitterOffsetPixels::new(
                        if wrong_jitter { -jx } else { jx },
                        if wrong_jitter { -jy } else { jy },
                    )
                    .unwrap(),
                    frame_time: FrameTimeMillis::new(16.667).unwrap(),
                    camera: CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0)
                        .unwrap(),
                    reset,
                })
            };
            if let Err(error) = dispatch {
                abort_dispatch(error, frame as u32);
            }
            let mut metrics = Metrics::default();
            // SAFETY: Finish records readback, submits and fences this frame.
            let hr = unsafe { (self.finish)(handle, &mut metrics) };
            if hr < 0 {
                eprintln!("Semantic fence/readback failure: {hr:#x}");
                std::process::exit(1); // Preserve possibly in-flight dependencies.
            }
            assert_eq!(metrics.finite_pixels as usize, PIXELS);
            assert_eq!(metrics.debug_errors, 0);
            assert_eq!(metrics.device_removed, 0);
            if frame == 7 {
                // SAFETY: The fence completed; destination has exactly PIXELS
                // writable floats, as required by the paired C helper.
                assert_eq!(
                    unsafe { (self.copy)(handle, result.as_mut_ptr(), PIXELS as u32) },
                    0
                );
            }
            frames.push((handle, list, resources));
        }
        // Every submission is fenced and frame dependencies still survive.
        context.destroy().unwrap();
        for (handle, list, resources) in frames {
            drop(resources);
            drop(list);
            // SAFETY: Context destruction and GPU completion both precede
            // releasing the helper's resources, allocator and command list.
            unsafe { (self.destroy)(handle) };
        }
        // SAFETY: The device is live after all dependent teardown.
        assert_eq!(unsafe { (self.check)(self.device.as_raw()) }, 0);
        eprintln!("Semantic sequence {name}: completed");
        result
    }
}

fn difference(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    for y in 32..328 {
        for x in 32..608 {
            let i = y * 640 + x;
            sum += (a[i] - b[i]).abs();
            count += 1;
        }
    }
    sum / count as f32
}

fn reference_error(image: &[f32]) -> f32 {
    let mut reference = vec![0.0; PIXELS];
    for y in 0..360 {
        for x in 0..640 {
            let u = (x as f32 + 0.5) / 2.0 - 14.0;
            let v = (y as f32 + 0.5) / 2.0;
            reference[y * 640 + x] = 0.5 + 0.4 * ((u + 0.37 * v) * 0.8).sin();
        }
    }
    difference(image, &reference)
}

pub(super) fn compare() {
    let equipment = Equipment::acquire();
    let correct = equipment.sequence("correct", false, false, false, false);
    let reversed = equipment.sequence("reversed motion", true, false, false, false);
    let wrong_jitter = equipment.sequence("reversed jitter", false, true, false, false);
    let reset = equipment.sequence("reset after history", false, false, true, false);
    let fresh = equipment.sequence("fresh reset", false, false, true, true);
    let motion_delta = difference(&correct, &reversed);
    let jitter_delta = difference(&correct, &wrong_jitter);
    let reset_delta = difference(&reset, &fresh);
    let history_delta = difference(&correct, &reset);
    let correct_error = reference_error(&correct);
    let reversed_error = reference_error(&reversed);
    let jitter_error = reference_error(&wrong_jitter);
    eprintln!(
        "Semantic deltas: motion={motion_delta:.8} jitter={jitter_delta:.8} reset={reset_delta:.8} history={history_delta:.8}"
    );
    eprintln!(
        "Analytic red MAE: correct={:.8} reversed_motion={:.8} reversed_jitter={:.8}",
        correct_error, reversed_error, jitter_error
    );
    // Sensitivity comparisons detect ignored/misforwarded inputs. They do not
    // guarantee a ranking across arbitrary scenes, providers or hardware.
    assert!(
        motion_delta > 0.0001,
        "motion direction did not affect reconstruction"
    );
    assert!(
        jitter_delta > 0.0001,
        "jitter direction did not affect reconstruction"
    );
    assert!(
        history_delta > 0.0001,
        "accumulated history did not affect reconstruction"
    );
    assert!(reset_delta < 0.00001, "reset retained prior history");
    assert!(
        correct_error < reversed_error,
        "reversed motion improved the analytic comparison"
    );
    assert!(
        correct_error < jitter_error,
        "reversed jitter improved the analytic comparison"
    );
    eprintln!("Gate C: PASSED");
}
