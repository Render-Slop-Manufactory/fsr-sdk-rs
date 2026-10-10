// SPDX-License-Identifier: MPL-2.0

//! M9 public-API integration, under the selected 2026-10-09 protocol.
//! Image comparisons are diagnostics; the historical effect test stays separate.
use super::*;
use crate::upscaler::{NativeSharpness, Sharpening};

type SetFrame = unsafe extern "C" fn(*mut c_void, u32, f32, f32) -> i32;
type CopyBits = unsafe extern "C" fn(*mut c_void, *mut u16, u32) -> i32;
type CopyRed = unsafe extern "C" fn(*mut c_void, *mut f32, u32) -> i32;
type FenceValue = unsafe extern "C" fn(*mut c_void) -> u64;
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
    fence_value: FenceValue,
    copy_bits: CopyBits,
    copy_red: CopyRed,
    destroy: Destroy,
    check: DeviceCheck,
    output: PathBuf,
}

impl Equipment {
    fn acquire() -> Self {
        let helper = std::env::var_os("FSR_SDK_TEST_M6_DLL").unwrap();
        let loader = std::env::var_os("FSR_SDK_TEST_DLL").unwrap();
        let output = PathBuf::from(std::env::var_os("FSR_SDK_TEST_M9_OUTPUT").unwrap());
        assert!(
            output.is_dir(),
            "runner must supply a fresh output directory"
        );
        assert!(std::env::var_os("FSR_SDK_TEST_GBV").is_none());
        // SAFETY: The opt-in runner builds this helper and verifies signed SDK
        // DLLs. These signatures match its C exports; all use stays on this thread.
        unsafe {
            let library = libloading::Library::new(helper).unwrap();
            let debug = *library.get::<EnableDebug>(b"m6_enable_debug\0").unwrap();
            assert_eq!(debug(), 0);
            let create = *library
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap();
            let mut raw = null_mut();
            assert_eq!(create(&mut raw), 0);
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            let amd = *library.get::<DeviceCheck>(b"m6_require_amd\0").unwrap();
            assert_eq!(amd(device.as_raw()), 0);
            Self {
                create: *library.get::<CreateHarness>(b"m6_create\0").unwrap(),
                set_frame: *library.get::<SetFrame>(b"sharpening_set_frame\0").unwrap(),
                finish: *library.get::<Finish>(b"m6_finish\0").unwrap(),
                fence_value: *library.get::<FenceValue>(b"d013_fence_value\0").unwrap(),
                copy_bits: *library
                    .get::<CopyBits>(b"sharpening_copy_red_bits\0")
                    .unwrap(),
                copy_red: *library.get::<CopyRed>(b"semantic_copy_red\0").unwrap(),
                destroy: *library.get::<Destroy>(b"m6_destroy\0").unwrap(),
                check: *library.get::<DeviceCheck>(b"m6_check_device\0").unwrap(),
                runtime: Runtime::load_unchecked(loader).unwrap(),
                device,
                output,
                _library: library,
            }
        }
    }

    fn sequence(&self, scene: u32, arm: &str, switching: bool) -> Vec<Image> {
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
        let mut completed_frames = Vec::new();
        let mut images = Vec::new();
        for (frame, &(jx, jy)) in JITTER.iter().enumerate() {
            let mut handle = null_mut();
            let mut raw_list = null_mut();
            let mut raw_resources = [null_mut(); 5];
            // SAFETY: Writable outputs receive an owned harness and extra COM
            // references. Its uploads are rewritten before any submission.
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
            // SAFETY: The helper transfers one owned live reference per output.
            let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
            let resources: Vec<_> = raw_resources
                .into_iter()
                .map(|raw| {
                    assert!(!raw.is_null());
                    // SAFETY: This is an owned reference transferred by m6_create.
                    unsafe { ID3D12Resource::from_raw(raw) }
                })
                .collect();
            let mut dispatch = UpscaleDispatch::new(
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
            );
            let strength = if switching {
                match frame {
                    4 => Some(0.0),
                    5 => Some(0.35),
                    6 => Some(1.0),
                    _ => None,
                }
            } else {
                None
            };
            dispatch.set_sharpening(match strength {
                Some(value) => Sharpening::Native(NativeSharpness::new(value).unwrap()),
                None => Sharpening::Disabled,
            });
            // SAFETY: Fresh DIRECT list and five distinct textures use the fixed
            // signed profile and required states. Each prior frame is fenced;
            // context, list/allocator and resources survive completion/teardown.
            let recorded = unsafe { context.dispatch(dispatch) };
            if let Err(error) = recorded {
                eprintln!("M9_RECORD scene={scene} arm={arm} frame={frame} error={error:?}");
                // Invalidate the never-submitted list before terminal teardown.
                drop(list);
                // SAFETY: This failed recording was never submitted, and all
                // previous frames completed their fences. Resources remain live.
                unsafe { (self.destroy)(handle) };
                eprintln!("M9_FAILED_CLEANUP result={:?}", context.destroy());
                std::process::exit(1);
            }
            eprintln!(
                "M9_RECORD scene={scene} arm={arm} frame={frame} code=0x00000000 enabled={} sharpness_bits={:#010x} reset={}",
                strength.is_some(),
                strength.unwrap_or(0.0).to_bits(),
                frame == 0
            );
            let mut metrics = Metrics::default();
            // SAFETY: Finish submits this successful recording exactly once,
            // waits for its fence and copies the resulting output.
            let hr = unsafe { (self.finish)(handle, &mut metrics) };
            if hr < 0 {
                eprintln!("M9_GPU_FAILURE scene={scene} arm={arm} frame={frame} hr={hr:#x}");
                std::process::exit(1); // Preserve possibly in-flight dependencies.
            }
            // SAFETY: The helper is retained, and this frame has completed.
            let fence = unsafe { (self.fence_value)(handle) };
            eprintln!(
                "M9_FRAME scene={scene} arm={arm} frame={frame} finish_hr={hr:#x} fence={fence} width={} height={} finite={} written={} debug_errors={} device_removed={:#x}",
                metrics.width,
                metrics.height,
                metrics.finite_pixels,
                metrics.changed_pixels,
                metrics.debug_errors,
                metrics.device_removed
            );
            assert!(fence >= 1 && fence != u64::MAX);
            assert_eq!((metrics.width, metrics.height), (640, 360));
            assert_eq!(metrics.finite_pixels as usize, PIXELS);
            assert_eq!(metrics.changed_pixels as usize, PIXELS);
            assert_eq!(metrics.debug_errors, 0);
            assert_eq!(metrics.device_removed, 0);
            let mut image = Image {
                bits: vec![0; PIXELS],
                red: vec![0.0; PIXELS],
            };
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
            let path = self
                .output
                .join(format!("scene-{scene}-{arm}-frame-{frame}.red.u16le"));
            let bytes: Vec<_> = image
                .bits
                .iter()
                .flat_map(|bits| bits.to_le_bytes())
                .collect();
            std::fs::write(path, bytes).unwrap();
            images.push(image);
            completed_frames.push((handle, list, resources));
        }
        let destroyed = context.destroy();
        eprintln!("M9_DESTROY scene={scene} arm={arm} result={destroyed:?}");
        destroyed.unwrap();
        for (handle, list, resources) in completed_frames {
            drop(resources);
            drop(list);
            // SAFETY: Every frame is fenced and its context was destroyed.
            unsafe { (self.destroy)(handle) };
        }
        // SAFETY: Inspect the still-live device after all dependent teardown.
        let checked = unsafe { (self.check)(self.device.as_raw()) };
        eprintln!("M9_CLEANUP scene={scene} arm={arm} frames=8 device_check={checked:#x}");
        assert_eq!(checked, 0);
        images
    }
}

fn compare(scene: u32, frame: usize, label: &str, a: &Image, b: &Image) {
    let mut mismatches = 0;
    let mut sum = 0.0_f64;
    for y in 32..328 {
        for x in 32..608 {
            let i = y * 640 + x;
            mismatches += usize::from(a.bits[i] != b.bits[i]);
            sum += (f64::from(a.red[i]) - f64::from(b.red[i])).abs();
        }
    }
    let mean = sum / (296.0 * 576.0);
    eprintln!(
        "M9_COMPARE scene={scene} frame={frame} pair={label} roi_bit_equal={} roi_bit_mismatches={mismatches} roi_mae_f64={mean:.17} diagnostic_only=true",
        mismatches == 0
    );
}

pub(super) fn run() {
    let equipment = Equipment::acquire();
    for scene in 0..2 {
        let a = equipment.sequence(scene, "switch-a", true);
        let b = equipment.sequence(scene, "switch-b", true);
        let disabled = equipment.sequence(scene, "disabled", false);
        for frame in 0..8 {
            compare(scene, frame, "a-b", &a[frame], &b[frame]);
            if frame >= 3 {
                compare(scene, frame, "a-disabled", &a[frame], &disabled[frame]);
            }
        }
    }
    eprintln!("M9 integration: PASSED outputs=48");
}
