// SPDX-License-Identifier: MPL-2.0

//! Independent raw and public generator routes sharing only GPU equipment/oracles.
use super::{followup::Lab, *};
use crate::error::Operation;
use fsr_sdk_sys::api::FfxApiDimensions2D;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8_UNORM;
#[path = "reactive_mask_descriptor.rs"]
mod descriptor;
use descriptor::{FLAGS, GENERATE_REACTIVE, GenerateReactive};

type Prepare =
    unsafe extern "C" fn(*mut c_void, u32, u32, *mut *mut c_void, *mut *mut c_void) -> i32;
type MaskMetrics = unsafe extern "C" fn(*mut c_void, *mut ReactiveMetrics) -> i32;
#[repr(C)]
#[derive(Default, Debug)]
struct ReactiveMetrics {
    width: u32,
    height: u32,
    active_pixels: u32,
    changed_pixels: u32,
    zero_pixels: u32,
    outside_changed: u32,
    minimum: u32,
    maximum: u32,
}

macro_rules! exercise {
    ($runtime:ty, $owner:ty, $raw:literal) => {{
        let flags: u32 = std::env::var("FSR_SDK_TEST_REACTIVE_FLAGS")
            .unwrap()
            .parse()
            .unwrap();
        assert!(flags <= FLAGS.into_iter().fold(0, |a, b| a | b));
        let alias = std::env::var("FSR_SDK_TEST_REACTIVE_ALIAS").unwrap() == "1";
        let control = std::env::var("FSR_SDK_TEST_REACTIVE_CONTROL").unwrap() == "1";
        let lab = Lab::acquire();
        // SAFETY: Exact paired opt-in helper exports, retained throughout all calls.
        let (prepare, after, read) = unsafe {
            (
                *lab.library.get::<Prepare>(c"gpu_probe_prepare").unwrap(),
                *lab.library
                    .get::<Destroy>(c"gpu_probe_after_generation")
                    .unwrap(),
                *lab.library
                    .get::<MaskMetrics>(c"gpu_probe_metrics")
                    .unwrap(),
            )
        };
        // SAFETY: Explicit isolated probe under trusted pinned ABI/provider and cleanup
        // assumptions. No unsupported-error atomicity or provider-wide support is assumed.
        let mut runtime = unsafe { <$runtime>::load(&lab.loader) }.unwrap();
        if !$raw {
            eprintln!(
                "Reactive available providers: {:?}",
                unsafe { runtime.upscaler_dx12(&lab.device) }.providers().unwrap()
            );
        }
        let mut options = UpscalerOptions::new(
            Dimensions::new(320, 180).unwrap(),
            Dimensions::new(640, 360).unwrap(),
        );
        options.set_dynamic_resolution(true);
        let mut requested = crate::query::ProviderSelection::Default;
        if let Ok(name) = std::env::var("FSR_SDK_TEST_REACTIVE_PROVIDER") {
            let providers = unsafe { runtime.upscaler_dx12(&lab.device) }.providers().unwrap();
            let selected = providers.iter().find(|provider| provider.name() == name)
                .expect("Requested pinned provider is unavailable");

            requested = crate::query::ProviderSelection::Explicit(selected.id());
            eprintln!("Reactive requested provider: name={name}");
        }
        let mut context = unsafe { runtime.upscaler_dx12(&lab.device) }.with_provider(requested).create(&options).unwrap();
        if let crate::query::ProviderSelection::Explicit(id) = requested {
            assert_eq!(context.provider().unwrap().id(), id);
        }
        eprintln!("Reactive probe provider: {:?}", context.provider().unwrap());
        for (phase, (width, height)) in [(320, 180), (256, 128), (320, 180)].into_iter().enumerate() {
            let mut frame = lab.open();
            let mut opaque = null_mut();
            let mut mask = null_mut();
            // SAFETY: Paired helper creates distinct live resources, seeds mask=227
            // and records their initial upload/barriers on this never-submitted list.
            assert_eq!(
                unsafe { prepare(frame.handle, width, height, &mut opaque, &mut mask) },
                0
            );
            assert!(!opaque.is_null() && !mask.is_null());
            // SAFETY: Each returned pointer transfers one independent COM reference.
            let opaque = unsafe { ID3D12Resource::from_raw(opaque) };
            let mask = unsafe { ID3D12Resource::from_raw(mask) };
            let mut inputs = UpscaleResources::builder().color(&frame.resources[0]).depth(&frame.resources[1]).motion_vectors(&frame.resources[2]).exposure(crate::upscaler::dx12::ExposureInput::Texture(&frame.resources[3])).output(&frame.resources[4]).build().unwrap();
            inputs.set_reactive_mask(Some(&mask));
            let mut upscale = UpscaleTestInputs::new(
                frame.list.as_ref().unwrap(),
                inputs,
                JitterOffsetPixels::new(0.25, -0.25).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                phase == 0,
            );
            upscale.set_active_render_size(Some(Dimensions::new(width, height).unwrap()));
            // Validate the future consumer before any generator native entry.
            context.validate_upscale_inputs(upscale.command_list, &upscale.resources, &upscale.parameters).unwrap();
            if !control {
                let source = if alias { &frame.resources[0] } else { &opaque };
                let result = if $raw {
                    let desc = GenerateReactive {
                        header: ffxApiHeader {
                            r#type: GENERATE_REACTIVE,
                            pNext: null_mut(),
                        },
                        command_list: frame.list.as_ref().unwrap().as_raw(),
                        color_opaque_only: resource_for_parity(
                            source,
                            DXGI_FORMAT_R16G16B16A16_FLOAT,
                            false,
                        )
                        .unwrap(),
                        color_pre_upscale: resource_for_parity(
                            &frame.resources[0],
                            DXGI_FORMAT_R16G16B16A16_FLOAT,
                            false,
                        )
                        .unwrap(),
                        out_reactive: resource_for_parity(&mask, DXGI_FORMAT_R8_UNORM, true).unwrap(),
                        render_size: FfxApiDimensions2D { width, height },
                        scale: 1.0,
                        cutoff_threshold: 0.2,
                        binary_value: 0.9,
                        flags,
                    };
                    context.core.eligibility.check().unwrap();
                    // SAFETY: Probe-local pinned ABI, matching resources/list and declared
                    // states. Context, descriptors and resources stay live through recording,
                    // queue completion, or terminal rejection handling; calls are serialized.
                    let code = unsafe { context.core.owner.dispatch(&desc.header) };
                    context.core.eligibility.record(Operation::Upscale, code)
                } else {
                    use crate::upscaler::{ReactiveMaskTestInputs, ReactiveMaskOptions};
                    let mut options = ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap();
                    options.set_apply_tonemap(flags & 1 != 0);
                    options.set_apply_inverse_tonemap(flags & 2 != 0);
                    options.set_apply_threshold(flags & 4 != 0);
                    options.set_use_components_max(flags & 8 != 0);
                    let request = ReactiveMaskTestInputs::new(
                        frame.list.as_ref().unwrap(),
                        source,
                        &frame.resources[0],
                        &mask,
                        Dimensions::new(width, height).unwrap(),
                        options,
                    );
                    context.validate_reactive_mask_inputs(request.command_list, &request.resources, &request.parameters).unwrap();
                    // SAFETY: Same disposable-context, live-resource, state and
                    // serialized native-domain contract as the independent raw arm.
                    unsafe { context.record_reactive_mask(request.command_list, &request.resources, &request.parameters) }
                };
                if let Err(error) = result {
                    let code = match error {
                        Error::Native(crate::error::NativeError { operation: Operation::GenerateReactiveMask, code }) => code,
                        _ => {
                            lab.discard_unsubmitted(&mut frame);
                            eprintln!("Reactive public validation failure: {error:?}");
                            eprintln!("Reactive terminal cleanup: {:?}", context.destroy());
                            drop(mask);
                            drop(opaque);
                            lab.release(frame);
                            std::process::exit(1);
                        }
                    };
                    eprintln!("Reactive native rejection: phase={phase} flags={flags} error={error:?}");
                    lab.discard_unsubmitted(&mut frame);
                    let cleanup = context.destroy();
                    eprintln!("Reactive rejection terminal cleanup: {cleanup:?}");
                    drop(mask);
                    drop(opaque);
                    lab.release(frame);
                    eprintln!(
                        "REACTIVE_PROBE {{\"outcome\":\"rejected\",\"flags\":{flags},\"alias\":{alias},\"control\":false,\"frames\":{phase},\"native_code\":{code},\"cleanup_ok\":{}}}",
                        cleanup.is_ok()
                    );
                    return; // No reuse, fallback, or support claim after a rejection.
                }
            }
            // SAFETY: All buffers were allocated before native entry. Record a UAV
            // barrier, snapshot the mask, and transition it to the consumer's SRV state.
            unsafe { after(frame.handle) };
            if !control {
                // SAFETY: Generation and upscale use one ordered list, matching active
                // region and resources retained through the immediate fenced finish.
                if let Err(error) = unsafe { context.record(upscale.command_list, &upscale.resources, &upscale.parameters) } {
                    eprintln!("Reactive consumer failure: {error:?}");
                    lab.discard_unsubmitted(&mut frame);
                    eprintln!(
                        "Reactive consumer terminal cleanup: {:?}",
                        context.destroy()
                    );
                    drop(mask);
                    drop(opaque);
                    lab.release(frame);
                    std::process::exit(1);
                }
            }
            lab.finish(&frame, !control);
            let mut metrics = ReactiveMetrics::default();
            // SAFETY: Helper's fence completed, and output points to writable matching ABI.
            assert_eq!(unsafe { read(frame.handle, &mut metrics) }, 0);
            assert_eq!(
                (metrics.width, metrics.height, metrics.active_pixels),
                (width, height, width * height)
            );
            eprintln!(
                "REACTIVE_FRAME {{\"phase\":{phase},\"width\":{width},\"height\":{height},\"changed\":{},\"zeros\":{},\"outside_changed\":{},\"minimum\":{},\"maximum\":{}}}",
                metrics.changed_pixels,
                metrics.zero_pixels,
                metrics.outside_changed,
                metrics.minimum,
                metrics.maximum
            );
            drop(mask);
            drop(opaque);
            lab.release(frame);
        }
        context.destroy().unwrap();
        lab.check();
        let outcome = if control { "control" } else { "generated" };
        eprintln!(
            "REACTIVE_PROBE {{\"outcome\":\"{outcome}\",\"flags\":{flags},\"alias\":{alias},\"control\":{control},\"frames\":3,\"native_code\":0,\"cleanup_ok\":true}}"
        );
    }};
}

pub(super) fn run() {
    exercise!(Runtime, Upscaler, true);
}

pub(super) fn run_public() {
    let family = std::env::var("FSR_SDK_TEST_REACTIVE_FAMILY").unwrap();
    eprintln!("Reactive public route: family={family}");
    match family.as_str() {
        "local" => exercise!(Runtime, Upscaler, false),
        #[cfg(feature = "movable")]
        "movable" => exercise!(
            crate::runtime::MovableRuntime,
            crate::upscaler::MovableUpscaler,
            false
        ),
        _ => panic!("Unknown public reactive family: {family}"),
    }
}
