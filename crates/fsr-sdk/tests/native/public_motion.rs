// SPDX-License-Identifier: MPL-2.0

//! Both public families: exact motion mapping and renderer-owned fenced inputs.
use super::{
    followup::Lab,
    variable_output::{OutputMetrics, Owner},
    *,
};
use crate::upscaler::{
    MotionVectorJitter, MotionVectorResolution, MotionVectorScale, NativeMotionVectorScale,
};

#[repr(C)]
struct Config {
    active_width: u32,
    active_height: u32,
    display_motion: u32,
    format: u32,
    motion_uav: u32,
    frame: u32,
    stored_x: f32,
    stored_y: f32,
    jitter_x: f32,
    jitter_y: f32,
}
type Create = unsafe extern "C" fn(
    *mut c_void,
    *const Config,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> i32;
type Complete = unsafe extern "C" fn(*mut c_void, *mut OutputMetrics) -> i32;
type Copy = unsafe extern "C" fn(*mut c_void, u32, *mut u8, u32) -> i32;
fn setting(name: &str) -> String {
    std::env::var(format!("FSR_SDK_TEST_PUBLIC_MOTION_{name}")).unwrap()
}

pub(super) fn run() {
    assert_eq!(
        (
            std::mem::size_of::<Config>(),
            std::mem::align_of::<Config>()
        ),
        (40, 4)
    );
    assert_eq!(
        (
            std::mem::size_of::<OutputMetrics>(),
            std::mem::align_of::<OutputMetrics>()
        ),
        (40, 8)
    );
    let lab = Lab::acquire();
    // SAFETY: The coordinator builds and pairs these exact C ABI exports.
    let (create, complete, copy, discard, release, compare) = unsafe {
        (
            *lab.library
                .get::<Create>(c"gpu_public_motion_create")
                .unwrap(),
            *lab.library.get::<Complete>(c"gpu_output_finish").unwrap(),
            *lab.library.get::<Copy>(c"gpu_output_copy").unwrap(),
            *lab.library.get::<Destroy>(c"gpu_output_discard").unwrap(),
            *lab.library.get::<Destroy>(c"gpu_output_destroy").unwrap(),
            *lab.library
                .get::<CompareResource>(c"gpu_compare_resource")
                .unwrap(),
        )
    };
    let provider_name = setting("PROVIDER");
    let family = setting("FAMILY");
    let arm = setting("ARM");
    let format: u32 = setting("FORMAT").parse().unwrap();
    let display = setting("DISPLAY") == "1";
    let jittered = setting("JITTER") == "1";
    let dynamic = setting("DYNAMIC") == "1";
    let uav = setting("UAV") == "1";
    let control = setting("CONTROL") == "1";
    let directory = PathBuf::from(setting("DIR"));
    // SAFETY: Exact pinned deployment, isolated serial calls and renderer-owned
    // resources retained until fenced completion. These are integration inputs.
    let mut runtime = unsafe { Runtime::load(&lab.loader) }.unwrap();
    let providers = unsafe { runtime.upscaler_dx12(&lab.device) }
        .providers()
        .unwrap();
    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .unwrap();
    let id = provider.id();
    eprintln!("Public motion provider: {provider:?}");
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );

    options.set_dynamic_resolution(dynamic);
    if display {
        options.set_motion_vector_resolution(MotionVectorResolution::Display);
    }
    if jittered {
        options.set_motion_vector_jitter(MotionVectorJitter::Included);
    }
    let mut context = match family.as_str() {
        "local" => {
            let mut c = unsafe { runtime.upscaler_dx12(&lab.device) }
                .with_provider(crate::query::ProviderSelection::Explicit(id))
                .create(&options)
                .unwrap();
            assert_eq!(c.provider().unwrap().id(), id);
            Owner::Local(c)
        }
        #[cfg(feature = "movable")]
        "movable" => {
            // SAFETY: Same exact pinned/common serialization; no handoff in this batch.
            let mut rt = unsafe { crate::runtime::MovableRuntime::load(&lab.loader) }.unwrap();
            let mut c = unsafe { rt.upscaler_dx12(&lab.device) }
                .with_provider(crate::query::ProviderSelection::Explicit(id))
                .create(&options)
                .unwrap();
            assert_eq!(c.provider().unwrap().id(), id);
            Owner::Movable(c)
        }
        _ => panic!("Unknown public motion ownership family"),
    };
    let jitters = if jittered {
        [(0.25f32, -0.25f32), (-0.25, 0.25), (0.125, -0.375)]
    } else {
        [(0.0, 0.0); 3]
    };
    for phase in 0..3 {
        let active = if dynamic && phase == 1 {
            (512, 288)
        } else {
            (640, 360)
        };
        let motion = if display { (640, 360) } else { (320, 180) };
        let extent = if display { active } else { (320, 180) };
        let applied = match arm.as_str() {
            "reference" | "clear" => (1.0, 1.0),
            "default" => (extent.0 as f32, extent.1 as f32),
            "positive" | "wrong-x" => (2.0, 4.0),
            "mixed" | "jitter-wrong" => (-2.0, 4.0),
            "wrong-y" => (-2.0, -4.0),
            _ => panic!("Unknown public motion encoding"),
        };
        let encoded = if matches!(arm.as_str(), "wrong-x" | "wrong-y") {
            (-2.0, 4.0)
        } else {
            applied
        };
        let pixels = if phase == 0 {
            (0.0, 0.0)
        } else if display {
            (
                -4.0 * (active.0 as f32 / 320.0),
                -2.0 * (active.1 as f32 / 180.0),
            )
        } else {
            (-4.0, -2.0)
        };
        let jitter = jitters[phase];
        let previous = if phase == 0 {
            jitter
        } else {
            jitters[phase - 1]
        };
        let delta = (previous.0 - jitter.0, previous.1 - jitter.1);
        let stored = if arm == "jitter-wrong" {
            (
                pixels.0 / encoded.0 + delta.0,
                pixels.1 / encoded.1 + delta.1,
            )
        } else {
            (
                (pixels.0 + delta.0) / encoded.0,
                (pixels.1 + delta.1) / encoded.1,
            )
        };
        let config = Config {
            active_width: active.0,
            active_height: active.1,
            display_motion: u32::from(display),
            format,
            motion_uav: u32::from(uav),
            frame: phase as u32,
            stored_x: stored.0,
            stored_y: stored.1,
            jitter_x: jitter.0,
            jitter_y: jitter.1,
        };
        let mut handle = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: Writable outputs acquire live independent COM references; the
        // helper retains uploads, queue and allocator through confirmed completion.
        assert_eq!(
            unsafe {
                create(
                    lab.device.as_raw(),
                    &config,
                    &mut handle,
                    &mut raw_list,
                    raw_resources.as_mut_ptr(),
                )
            },
            0
        );
        assert!(
            !handle.is_null() && !raw_list.is_null() && raw_resources.iter().all(|p| !p.is_null())
        );
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<_> = raw_resources
            .into_iter()
            .map(|p| unsafe { ID3D12Resource::from_raw(p) })
            .collect();
        let mut frame = UpscaleTestInputs::new(
            &list,
            UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                .output(&resources[4])
                .build()
                .unwrap(),
            JitterOffsetPixels::new(jitter.0, jitter.1).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
            phase == 0,
        );
        if arm != "default" {
            frame.set_motion_vector_scale(MotionVectorScale::Pixels);
        }
        if !matches!(arm.as_str(), "reference" | "default") {
            frame.set_motion_vector_scale(crate::upscaler::MotionVectorScale::Native(
                NativeMotionVectorScale::new(applied.0, applied.1).unwrap(),
            ));
            if arm == "clear" {
                frame.set_motion_vector_scale(crate::upscaler::MotionVectorScale::Native(
                    NativeMotionVectorScale::new(-2.0, 4.0).unwrap(),
                ));
                // Restore the caller-selected pixel request explicitly, without fallback storage.
                frame.set_motion_vector_scale(MotionVectorScale::Pixels);
            }
        }
        if dynamic {
            frame.set_active_output_size(Some(Dimensions::new(active.0, active.1).unwrap()));
        }
        let desc = context.inspect(&frame).unwrap();
        assert_eq!(
            [
                desc.renderSize.width,
                desc.renderSize.height,
                desc.upscaleSize.width,
                desc.upscaleSize.height
            ],
            [320, 180, active.0, active.1]
        );
        assert_eq!(
            [
                desc.motionVectorScale.x.to_bits(),
                desc.motionVectorScale.y.to_bits()
            ],
            [applied.0.to_bits(), applied.1.to_bits()]
        );
        assert_eq!(
            [desc.jitterOffset.x.to_bits(), desc.jitterOffset.y.to_bits()],
            [jitter.0.to_bits(), jitter.1.to_bits()]
        );
        assert_eq!(
            (desc.reset, desc.enableSharpening, desc.flags),
            (phase == 0, false, 0)
        );
        for (role, native) in [
            &desc.color,
            &desc.depth,
            &desc.motionVectors,
            &desc.exposure,
            &desc.output,
        ]
        .into_iter()
        .enumerate()
        {
            let state = if role == 4 {
                FFX_API_RESOURCE_STATE_UNORDERED_ACCESS
            } else {
                FFX_API_RESOURCE_STATE_COMPUTE_READ
            };
            assert_eq!(
                unsafe { compare(resources[role].as_raw(), native, state) },
                1
            );
        }
        eprintln!(
            "PUBLIC_MOTION_DESCRIPTOR {{\"phase\":{phase},\"provider\":\"{provider_name}\",\"family\":\"{family}\",\"arm\":\"{arm}\",\"format\":{format},\"display\":{display},\"jitter_included\":{jittered},\"dynamic\":{dynamic},\"uav\":{uav},\"control\":{control},\"active\":[{},{}],\"motion_allocation\":[{},{}],\"scale_bits\":[{},{}],\"stored_bits\":[{},{}],\"jitter_bits\":[{},{}],\"override\":{},\"native_format\":{},\"native_usage\":{},\"reset\":{}}}",
            active.0,
            active.1,
            motion.0,
            motion.1,
            desc.motionVectorScale.x.to_bits(),
            desc.motionVectorScale.y.to_bits(),
            stored.0.to_bits(),
            stored.1.to_bits(),
            desc.jitterOffset.x.to_bits(),
            desc.jitterOffset.y.to_bits(),
            matches!(
                frame.motion_vector_scale(),
                crate::upscaler::MotionVectorScale::Native(_)
            ),
            desc.motionVectors.description.format,
            desc.motionVectors.description.usage,
            desc.reset
        );
        let recorded = if control {
            Ok(())
        } else {
            context.public_record(frame)
        };
        if let Err(error) = recorded {
            unsafe { discard(handle) };
            drop(list);
            let cleanup = context.destroy();
            drop(resources);
            unsafe { release(handle) };
            eprintln!(
                "PUBLIC_MOTION_RESULT {{\"outcome\":\"rejected\",\"frames\":{phase},\"cleanup_ok\":{}}}",
                cleanup.is_ok()
            );
            eprintln!("Public motion rejection: {error:?}");
            return;
        }
        let mut metrics = OutputMetrics::default();
        let hr = unsafe { complete(handle, &mut metrics) };
        if hr < 0 || metrics.completed != 1 || metrics.device_removed != 0 {
            eprintln!(
                "PUBLIC MOTION COMPLETION FAILURE hr={hr:#x} completed={} removed={}",
                metrics.completed, metrics.device_removed
            );
            std::process::exit(1);
        }
        for (role, width, height, stride) in [
            (0, 320, 180, 8),
            (1, 320, 180, 4),
            (2, motion.0, motion.1, if format == 0 { 4 } else { 8 }),
            (3, 1, 1, 4),
            (4, 640, 360, 8),
        ] {
            let mut bytes = vec![0u8; (width * height * stride) as usize];
            assert_eq!(
                unsafe { copy(handle, role, bytes.as_mut_ptr(), bytes.len() as u32) },
                0
            );
            std::fs::write(
                directory.join(format!("phase-{phase}-role-{role}.bin")),
                bytes,
            )
            .unwrap();
        }
        eprintln!(
            "PUBLIC_MOTION_FRAME {{\"phase\":{phase},\"finite\":{},\"written\":{},\"outside\":{},\"right\":{},\"bottom\":{},\"input_mismatches\":{},\"debug_errors\":{},\"removed\":{},\"completed\":{}}}",
            metrics.finite_active,
            metrics.written_active,
            metrics.written_outside,
            metrics.rightmost_written,
            metrics.bottommost_written,
            metrics.input_mismatches,
            metrics.debug_errors,
            metrics.device_removed,
            metrics.completed
        );
        drop(list);
        drop(resources);
        unsafe { release(handle) };
        assert_eq!((metrics.debug_errors, metrics.input_mismatches), (0, 0));
    }
    context.destroy().unwrap();
    lab.check();
    eprintln!(
        "PUBLIC_MOTION_RESULT {{\"outcome\":\"completed\",\"frames\":3,\"cleanup_ok\":true}}"
    );
}
