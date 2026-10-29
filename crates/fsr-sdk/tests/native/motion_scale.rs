// SPDX-License-Identifier: MPL-2.0

//! Raw, disposable-context format/scale experiment; no public API widening.
use super::{followup::Lab, *};
use crate::error::Operation;
use fsr_sdk_sys::{api::*, upscale::*};

#[repr(C)]
struct Config {
    format: u32,
    axis: u32,
    frame: u32,
    stored_x: f32,
    stored_y: f32,
}
#[repr(C)]
#[derive(Default)]
struct Metrics {
    finite: u32,
    written: u32,
    outside: u32,
    right: u32,
    bottom: u32,
    input_mismatches: u32,
    debug_errors: u64,
    removed: i32,
    completed: u32,
}
type Create = unsafe extern "C" fn(
    *mut c_void,
    *const Config,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> i32;
type Complete = unsafe extern "C" fn(*mut c_void, *mut Metrics) -> i32;
type Copy = unsafe extern "C" fn(*mut c_void, u32, *mut u8, u32) -> i32;

fn setting(name: &str) -> String {
    std::env::var(format!("FSR_SDK_TEST_MOTION_{name}")).unwrap()
}

pub(super) fn run() {
    assert_eq!(
        (
            std::mem::size_of::<Config>(),
            std::mem::align_of::<Config>()
        ),
        (20, 4)
    );
    assert_eq!(
        (
            std::mem::size_of::<Metrics>(),
            std::mem::align_of::<Metrics>()
        ),
        (40, 8)
    );
    let lab = Lab::acquire();
    // SAFETY: The coordinator compiles and pairs these exact C ABI exports.
    let (create, complete, copy, discard, release, compare) = unsafe {
        (
            *lab.library.get::<Create>(c"gpu_motion_create").unwrap(),
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
    let axis: u32 = setting("AXIS").parse().unwrap();
    let format: u32 = setting("FORMAT").parse().unwrap();
    let stored: Vec<f32> = setting("STORED")
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let scale: Vec<f32> = setting("SCALE")
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!((stored.len(), scale.len()), (2, 2));
    assert!(stored.iter().chain(&scale).all(|v| v.is_finite()));
    let control = setting("CONTROL") == "1";
    let directory = PathBuf::from(setting("DIR"));
    // SAFETY: Pinned signed deployment, isolated serial experiment. The helper
    // retains resources and fences every submission before reuse/destruction.
    let mut runtime = unsafe { Runtime::load(&lab.loader) }.unwrap();
    let providers = unsafe { runtime.upscaler_dx12(&lab.device) }
        .providers()
        .unwrap();
    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .unwrap();
    let id = provider.id();
    eprintln!("Motion probe provider: {provider:?}");
    let options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );

    let mut context = unsafe { runtime.upscaler_dx12(&lab.device) }
        .with_provider(crate::query::ProviderSelection::Explicit(id))
        .create(&options)
        .unwrap();
    assert_eq!(context.provider().unwrap().id(), id);
    for phase in 0..4 {
        let config = Config {
            format,
            axis,
            frame: phase,
            stored_x: stored[0],
            stored_y: stored[1],
        };
        let mut handle = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: Writable outputs receive independent references retained until
        // confirmed completion; no submitted work exists on creation failure.
        let hr = unsafe {
            create(
                lab.device.as_raw(),
                &config,
                &mut handle,
                &mut raw_list,
                raw_resources.as_mut_ptr(),
            )
        };
        assert_eq!(hr, 0);
        assert!(
            !handle.is_null() && !raw_list.is_null() && raw_resources.iter().all(|p| !p.is_null())
        );
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<_> = raw_resources
            .into_iter()
            .map(|p| unsafe { ID3D12Resource::from_raw(p) })
            .collect();
        // Four existing formats use the production converter. The new motion
        // constant is probe-local: value 6 in pinned FfxApiSurfaceFormat, not a
        // production binding or an accepted wrapper role/format pair.
        let motion = if format == 0 {
            resource_for_parity(&resources[2], DXGI_FORMAT_R16G16_FLOAT, false).unwrap()
        } else {
            let actual = unsafe { resources[2].GetDesc() };
            assert_eq!(
                actual.Format,
                windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT
            );
            assert_eq!(
                (
                    actual.Width,
                    actual.Height,
                    actual.DepthOrArraySize,
                    actual.MipLevels,
                    actual.SampleDesc.Count,
                    actual.Flags.0
                ),
                (320, 180, 1, 1, 1, 0)
            );
            FfxApiResource {
                resource: resources[2].as_raw(),
                description: FfxApiResourceDescription {
                    r#type: FFX_API_RESOURCE_TYPE_TEXTURE2D,
                    format: 6,
                    width: FfxApiResourceWidth { width: 320 },
                    height: FfxApiResourceHeight { height: 180 },
                    depth: FfxApiResourceDepth { depth: 1 },
                    mipCount: 1,
                    flags: FFX_API_RESOURCE_FLAGS_NONE,
                    usage: FFX_API_RESOURCE_USAGE_READ_ONLY,
                },
                state: FFX_API_RESOURCE_STATE_COMPUTE_READ,
            }
        };
        // SAFETY: Zeroed POD denotes omitted optional inputs.
        let omitted = || unsafe { std::mem::zeroed::<FfxApiResource>() };
        let desc = ffxDispatchDescUpscale {
            header: ffxApiHeader {
                r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
                pNext: null_mut(),
            },
            commandList: list.as_raw(),
            color: resource_for_parity(&resources[0], DXGI_FORMAT_R16G16B16A16_FLOAT, false)
                .unwrap(),
            depth: resource_for_parity(&resources[1], DXGI_FORMAT_R32_FLOAT, false).unwrap(),
            motionVectors: motion,
            exposure: resource_for_parity(&resources[3], DXGI_FORMAT_R32_FLOAT, false).unwrap(),
            reactive: omitted(),
            transparencyAndComposition: omitted(),
            output: resource_for_parity(&resources[4], DXGI_FORMAT_R16G16B16A16_FLOAT, true)
                .unwrap(),
            jitterOffset: FfxApiFloatCoords2D { x: 0.0, y: 0.0 },
            motionVectorScale: FfxApiFloatCoords2D {
                x: scale[0],
                y: scale[1],
            },
            renderSize: FfxApiDimensions2D {
                width: 320,
                height: 180,
            },
            upscaleSize: FfxApiDimensions2D {
                width: 640,
                height: 360,
            },
            enableSharpening: false,
            sharpness: 0.0,
            frameTimeDelta: 16.667,
            preExposure: 1.0,
            reset: phase == 0,
            cameraNear: 0.1,
            cameraFar: 100.0,
            cameraFovAngleVertical: std::f32::consts::FRAC_PI_3,
            viewSpaceToMetersFactor: 1.0,
            flags: 0,
        };
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
            // SAFETY: Independent pinned C++ conversion of each live resource.
            assert_eq!(
                unsafe { compare(resources[role].as_raw(), native, state) },
                1
            );
        }
        eprintln!(
            "MOTION_DESCRIPTOR {{\"phase\":{phase},\"provider\":\"{provider_name}\",\"axis\":{axis},\"format\":{format},\"scale_bits\":[{},{}],\"stored_bits\":[{},{}],\"reset\":{},\"control\":{control}}}",
            scale[0].to_bits(),
            scale[1].to_bits(),
            stored[0].to_bits(),
            stored[1].to_bits(),
            phase == 0
        );
        if !control {
            context.core.eligibility.check().unwrap();
            // SAFETY: Deliberate test-only format/scale probe with real metadata,
            // live dependencies and serial fenced completion. No public widening.
            let code = unsafe { context.core.owner.dispatch(&desc.header) };
            if let Err(error) = context.core.eligibility.record(Operation::Upscale, code) {
                unsafe { discard(handle) };
                drop(list);
                let cleanup = context.destroy();
                drop(resources);
                unsafe { release(handle) };
                eprintln!(
                    "MOTION_RESULT {{\"outcome\":\"rejected\",\"frames\":{phase},\"code\":{code},\"cleanup_ok\":{}}}",
                    cleanup.is_ok()
                );
                eprintln!("Motion rejection: {error:?}");
                return;
            }
        }
        let mut metrics = Metrics::default();
        let hr = unsafe { complete(handle, &mut metrics) };
        if hr < 0 || metrics.completed != 1 || metrics.removed != 0 {
            eprintln!(
                "MOTION COMPLETION FAILURE hr={hr:#x} completed={} removed={}",
                metrics.completed, metrics.removed
            );
            std::process::exit(1); // Never unwind across uncertain GPU completion.
        }
        for (role, width, height, stride) in [
            (0, 320, 180, 8),
            (1, 320, 180, 4),
            (2, 320, 180, if format == 0 { 4 } else { 8 }),
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
            "MOTION_FRAME {{\"phase\":{phase},\"finite\":{},\"written\":{},\"outside\":{},\"right\":{},\"bottom\":{},\"input_mismatches\":{},\"debug_errors\":{},\"removed\":{},\"completed\":{}}}",
            metrics.finite,
            metrics.written,
            metrics.outside,
            metrics.right,
            metrics.bottom,
            metrics.input_mismatches,
            metrics.debug_errors,
            metrics.removed,
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
        "MOTION_RESULT {{\"outcome\":\"completed\",\"frames\":4,\"code\":0,\"cleanup_ok\":true}}"
    );
}
