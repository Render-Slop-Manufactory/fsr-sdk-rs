// SPDX-License-Identifier: MPL-2.0

//! Shared readback equipment for raw geometry and public active-output recording.
use super::{followup::Lab, *};
use crate::{error::Operation, upscaler::MotionVectorResolution};
use fsr_sdk_sys::{api::*, upscale::*};

pub(super) enum Owner {
    Local(Upscaler),
    #[cfg(feature = "movable")]
    Movable(crate::upscaler::MovableUpscaler),
}

impl Owner {
    pub(super) fn inspect(
        &self,
        frame: &UpscaleTestInputs<'_>,
    ) -> Result<ffxDispatchDescUpscale, Error> {
        match self {
            Self::Local(c) => c.core.descriptor_for_parity(frame),
            #[cfg(feature = "movable")]
            Self::Movable(c) => c.core.descriptor_for_parity(frame),
        }
    }

    pub(super) fn public_record(&mut self, frame: UpscaleTestInputs<'_>) -> Result<(), Error> {
        match self {
            Self::Local(c) => {
                c.validate_upscale_inputs(frame.command_list, &frame.resources, &frame.parameters)?;
                // SAFETY: Pinned supported profile; helper owns states, allocations,
                // serialized recording, fenced completion and terminal invalidation.
                unsafe { c.record(frame.command_list, &frame.resources, &frame.parameters) }
            }
            #[cfg(feature = "movable")]
            Self::Movable(c) => {
                c.validate_upscale_inputs(frame.command_list, &frame.resources, &frame.parameters)?;
                // SAFETY: Same bounded native/GPU obligations, all calls on one thread.
                unsafe { c.record(frame.command_list, &frame.resources, &frame.parameters) }
            }
        }
    }

    fn raw_record(&mut self, desc: &ffxDispatchDescUpscale) -> Result<(), Error> {
        let c = match self {
            Self::Local(c) => c,
            #[cfg(feature = "movable")]
            Self::Movable(_) => unreachable!("raw probe is local-only"),
        };
        c.core.eligibility.check()?;
        // SAFETY: Deliberate isolated raw probe beyond public geometry policy;
        // actual resource metadata and fenced/terminal helper ownership are retained.
        let code = unsafe { c.core.owner.dispatch(&desc.header) };
        c.core.eligibility.record(Operation::Upscale, code)
    }

    pub(super) fn destroy(self) -> Result<(), Error> {
        match self {
            Self::Local(c) => c.destroy(),
            #[cfg(feature = "movable")]
            Self::Movable(c) => c.destroy(),
        }
    }
}

#[repr(C)]
struct Config {
    active_width: u32,
    active_height: u32,
    output_width: u32,
    output_height: u32,
    motion_width: u32,
    motion_height: u32,
    motion_mode: u32,
    frame: u32,
    jitter_x: f32,
    jitter_y: f32,
}
#[repr(C)]
#[derive(Default)]
pub(super) struct OutputMetrics {
    pub(super) finite_active: u32,
    pub(super) written_active: u32,
    pub(super) written_outside: u32,
    pub(super) rightmost_written: u32,
    pub(super) bottommost_written: u32,
    pub(super) input_mismatches: u32,
    pub(super) debug_errors: u64,
    pub(super) device_removed: i32,
    pub(super) completed: u32,
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
    std::env::var(format!("FSR_SDK_TEST_OUTPUT_{name}")).unwrap()
}
fn allocation(policy: &str, active: (u32, u32)) -> (u32, u32) {
    match policy {
        "active" => active,
        "maximum" => (640, 360),
        "large" => (672, 384),
        _ => panic!("Unknown output allocation policy"),
    }
}

pub(super) fn run() {
    run_route(false);
}

pub(super) fn run_public() {
    run_route(true);
}

fn run_route(public: bool) {
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
    // SAFETY: The coordinator compiles/stages these exact paired C ABI exports.
    let (create, complete, copy, discard, release, compare) = unsafe {
        (
            *lab.library.get::<Create>(c"gpu_output_create").unwrap(),
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
    let sequence = setting("SEQUENCE");
    let output_policy = setting("ALLOCATION");
    let motion_policy = setting("MOTION_ALLOCATION");
    let motion_mode: u32 = setting("MOTION").parse().unwrap();
    let dynamic = setting("DYNAMIC") == "1";
    let sharpen = setting("SHARPEN") == "1";
    let control = setting("CONTROL") == "1";
    let family = if public {
        setting("FAMILY")
    } else {
        "local".into()
    };
    eprintln!("OUTPUT_ROUTE {{\"public\":{public},\"family\":\"{family}\"}}");
    if public {
        assert!(matches!(provider_name.as_str(), "4.1.1" | "3.1.5"));
        assert_eq!(
            (&*output_policy, &*motion_policy, dynamic),
            ("maximum", "maximum", true)
        );
    }
    let output_dir = PathBuf::from(setting("DIR"));
    // SAFETY: Pinned signed deployment; disposable-context native experiment,
    // isolated process, serialized calls and explicit fenced/terminal cleanup.
    let mut runtime = unsafe { Runtime::load(&lab.loader) }.unwrap();
    let providers = unsafe { runtime.upscaler_dx12(&lab.device) }
        .providers()
        .unwrap();
    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .unwrap();
    eprintln!("Output probe provider: {provider:?}");
    let id = provider.id();
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );

    options.set_dynamic_resolution(dynamic);
    if matches!(motion_mode, 1 | 2) {
        options.set_motion_vector_resolution(MotionVectorResolution::Display);
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
            // SAFETY: Same exact pinned deployment/common serialization; no handoff.
            let mut rt = unsafe { crate::runtime::MovableRuntime::load(&lab.loader) }.unwrap();
            let mut c = unsafe { rt.upscaler_dx12(&lab.device) }
                .with_provider(crate::query::ProviderSelection::Explicit(id))
                .create(&options)
                .unwrap();
            assert_eq!(c.provider().unwrap().id(), id);
            Owner::Movable(c)
        }
        _ => panic!("Unknown output ownership family"),
    };
    let regions = match sequence.as_str() {
        "stable" => [(640, 360); 3],
        "small" => [(511, 287); 3],
        "aligned" => [(640, 360), (512, 288), (640, 360)],
        "rounded" => [(637, 357), (511, 287), (637, 357)],
        "width" => [(637, 357), (511, 357), (637, 357)],
        "height" => [(637, 357), (637, 287), (637, 357)],
        _ => panic!("Unknown output sequence"),
    };
    for (phase, active) in regions.into_iter().enumerate() {
        let (ow, oh) = allocation(&output_policy, active);
        let (mw, mh) = if matches!(motion_mode, 0 | 3) {
            (320, 180)
        } else {
            allocation(&motion_policy, active)
        };
        let render = Dimensions::new(320, 180).unwrap();
        let phases = unsafe { runtime.upscaler_dx12(&lab.device) }
            .with_provider(crate::query::ProviderSelection::Explicit(id))
            .jitter_phase_count(render, Dimensions::new(active.0, active.1).unwrap())
            .unwrap();
        let jitter = unsafe { runtime.upscaler_dx12(&lab.device) }
            .with_provider(crate::query::ProviderSelection::Explicit(id))
            .jitter_offset(phase as i32, phases)
            .unwrap();
        let config = Config {
            active_width: active.0,
            active_height: active.1,
            output_width: ow,
            output_height: oh,
            motion_width: mw,
            motion_height: mh,
            motion_mode,
            frame: phase as u32,
            jitter_x: jitter.x(),
            jitter_y: jitter.y(),
        };
        let mut handle = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: Writable outputs adopt independent COM references to live
        // list/textures; helper retains uploads/allocator/queue until release.
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
        assert!(!handle.is_null() && !raw_list.is_null());
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        // SAFETY: All five outputs independently own a live transferred reference.
        let resources: Vec<_> = raw_resources
            .into_iter()
            .map(|p| unsafe { ID3D12Resource::from_raw(p) })
            .collect();
        let scale = if matches!(motion_mode, 2 | 3) {
            (1.0, 1.0)
        } else if motion_mode == 1 {
            (active.0 as f32, active.1 as f32)
        } else {
            (320.0, 180.0)
        };
        // SAFETY: POD C resource representation; zero denotes an omitted optional input.
        let null_resource = || unsafe { std::mem::zeroed::<FfxApiResource>() };
        let mut desc = ffxDispatchDescUpscale {
            header: ffxApiHeader {
                r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
                pNext: null_mut(),
            },
            commandList: list.as_raw(),
            color: resource_for_parity(&resources[0], DXGI_FORMAT_R16G16B16A16_FLOAT, false)
                .unwrap(),
            depth: resource_for_parity(&resources[1], DXGI_FORMAT_R32_FLOAT, false).unwrap(),
            motionVectors: resource_for_parity(&resources[2], DXGI_FORMAT_R16G16_FLOAT, false)
                .unwrap(),
            exposure: resource_for_parity(&resources[3], DXGI_FORMAT_R32_FLOAT, false).unwrap(),
            reactive: null_resource(),
            transparencyAndComposition: null_resource(),
            output: resource_for_parity(&resources[4], DXGI_FORMAT_R16G16B16A16_FLOAT, true)
                .unwrap(),
            jitterOffset: FfxApiFloatCoords2D {
                x: jitter.x(),
                y: jitter.y(),
            },
            motionVectorScale: FfxApiFloatCoords2D {
                x: scale.0,
                y: scale.1,
            },
            renderSize: FfxApiDimensions2D {
                width: 320,
                height: 180,
            },
            upscaleSize: FfxApiDimensions2D {
                width: active.0,
                height: active.1,
            },
            enableSharpening: sharpen,
            sharpness: if sharpen { 0.5 } else { 0.0 },
            frameTimeDelta: 16.667,
            preExposure: 1.0,
            reset: phase == 0,
            cameraNear: 0.1,
            cameraFar: 100.0,
            cameraFovAngleVertical: std::f32::consts::FRAC_PI_3,
            viewSpaceToMetersFactor: 1.0,
            flags: 0,
        };
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
            jitter,
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
            phase == 0,
        );
        if public {
            // Stable maximum tests constructor defaults; every other arm uses the setter.
            if sequence != "stable" {
                frame.set_active_output_size(Some(Dimensions::new(active.0, active.1).unwrap()));
            }
            if matches!(motion_mode, 2 | 3) {
                frame.set_motion_vector_scale(crate::upscaler::MotionVectorScale::Pixels);
            }
            if sharpen {
                frame.set_sharpening(crate::upscaler::Sharpening::Native(
                    crate::upscaler::NativeSharpness::new(0.5).unwrap(),
                ));
            }
            let actual = context.inspect(&frame).unwrap();
            assert_eq!(
                (actual.renderSize.width, actual.renderSize.height),
                (320, 180)
            );
            assert_eq!(
                (actual.upscaleSize.width, actual.upscaleSize.height),
                active
            );
            assert_eq!(
                (actual.motionVectorScale.x, actual.motionVectorScale.y),
                scale
            );
            assert_eq!(
                (actual.reset, actual.enableSharpening, actual.sharpness),
                (phase == 0, sharpen, if sharpen { 0.5 } else { 0.0 })
            );
            assert_eq!(
                (
                    actual.jitterOffset.x.to_bits(),
                    actual.jitterOffset.y.to_bits()
                ),
                (jitter.x().to_bits(), jitter.y().to_bits())
            );
            assert_eq!(
                (
                    actual.frameTimeDelta,
                    actual.preExposure,
                    actual.cameraNear,
                    actual.cameraFar,
                    actual.cameraFovAngleVertical,
                    actual.viewSpaceToMetersFactor,
                    actual.flags
                ),
                (
                    desc.frameTimeDelta,
                    desc.preExposure,
                    desc.cameraNear,
                    desc.cameraFar,
                    desc.cameraFovAngleVertical,
                    desc.viewSpaceToMetersFactor,
                    desc.flags
                )
            );
            assert!(
                actual.reactive.resource.is_null()
                    && actual.transparencyAndComposition.resource.is_null()
            );
            desc = actual; // Independent C++ parity below checks the production description.
        }
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
            // SAFETY: Independent pinned C++ inline conversion, same live COM
            // resource and complete descriptor; no FidelityFX runtime calls.
            assert_eq!(
                unsafe { compare(resources[role].as_raw(), native, state) },
                1
            );
        }
        eprintln!(
            "OUTPUT_DESCRIPTOR {{\"phase\":{phase},\"active\":[{},{}],\"allocation\":[{ow},{oh}],\"motion_allocation\":[{mw},{mh}],\"motion_scale\":[{},{}],\"jitter\":[{},{}],\"jitter_bits\":[{},{}],\"reset\":{},\"dynamic\":{dynamic},\"sharpen\":{sharpen},\"motion_mode\":{motion_mode},\"control\":{control}}}",
            active.0,
            active.1,
            scale.0,
            scale.1,
            jitter.x(),
            jitter.y(),
            jitter.x().to_bits(),
            jitter.y().to_bits(),
            phase == 0
        );
        if !control {
            let result = if public {
                context.public_record(frame)
            } else {
                context.raw_record(&desc)
            };
            if let Err(error) = result {
                let code = match error {
                    Error::Native(failure) => failure.code(),
                    _ => panic!("Output preflight failure: {error:?}"),
                };
                drop(list);
                unsafe { discard(handle) };
                eprintln!("Output native rejection: {error:?}");
                let cleanup = context.destroy();
                drop(resources);
                unsafe { release(handle) };
                eprintln!(
                    "OUTPUT_RESULT {{\"outcome\":\"rejected\",\"frames\":{phase},\"code\":{code},\"cleanup_ok\":{}}}",
                    cleanup.is_ok()
                );
                return;
            }
        }
        let mut metrics = OutputMetrics::default();
        // SAFETY: Context and all frame dependencies remain live; helper closes,
        // submits once and confirms its own queue fence and device status.
        let hr = unsafe { complete(handle, &mut metrics) };
        if hr < 0 || metrics.completed != 1 || metrics.device_removed != 0 {
            eprintln!(
                "OUTPUT COMPLETION FAILURE hr={hr:#x} completed={} removed={}",
                metrics.completed, metrics.device_removed
            );
            std::process::exit(1); // Never unwind while GPU completion is uncertain.
        }
        for (role, width, height, stride) in [
            (0, 320, 180, 8),
            (1, 320, 180, 4),
            (2, mw, mh, 4),
            (3, 1, 1, 4),
            (4, ow, oh, 8),
        ] {
            let mut bytes = vec![0u8; (width * height * stride) as usize];
            // SAFETY: Fenced readback into a writable buffer with exact packed size.
            assert_eq!(
                unsafe { copy(handle, role, bytes.as_mut_ptr(), bytes.len() as u32) },
                0
            );
            std::fs::write(
                output_dir.join(format!("phase-{phase}-role-{role}.bin")),
                bytes,
            )
            .unwrap();
        }
        eprintln!(
            "OUTPUT_FRAME {{\"phase\":{phase},\"finite\":{},\"written\":{},\"outside\":{},\"right\":{},\"bottom\":{},\"input_mismatches\":{},\"debug_errors\":{},\"removed\":{},\"completed\":{}}}",
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
        if metrics.debug_errors != 0 || metrics.input_mismatches != 0 {
            let cleanup = context.destroy();
            eprintln!(
                "OUTPUT_RESULT {{\"outcome\":\"diagnostic_failure\",\"frames\":{},\"code\":0,\"cleanup_ok\":{}}}",
                phase + 1,
                cleanup.is_ok()
            );
            return;
        }
    }
    context.destroy().unwrap();
    lab.check();
    eprintln!(
        "OUTPUT_RESULT {{\"outcome\":\"completed\",\"frames\":3,\"code\":0,\"cleanup_ok\":true}}"
    );
}
