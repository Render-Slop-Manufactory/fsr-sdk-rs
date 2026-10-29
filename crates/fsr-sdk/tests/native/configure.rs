// SPDX-License-Identifier: MPL-2.0

//! Native acknowledgement -> dispatch -> fence -> default -> dispatch -> cleanup.
use super::{followup::Lab, *};
use crate::{
    error::TerminalCause,
    upscaler::{ContextUsability, TuningParameter, TuningRequest, TuningValue},
};

macro_rules! exercise {
    ($lab:ident, $family:literal, $runtime:expr, $owner:ty) => {{
        let mut runtime = $runtime;
        for (parameter, value) in [
            (TuningParameter::VelocityFactor, 0.5),
            (TuningParameter::ReactivenessScale, 0.5),
            (TuningParameter::ShadingChangeScale, 0.5),
            (TuningParameter::AccumulationAddedPerFrame, 0.25),
            (TuningParameter::MinDisocclusionAccumulation, 0.0),
        ] {
            let options = UpscalerOptions::new(Dimensions::new(320, 180).unwrap(), Dimensions::new(640, 360).unwrap());
            let mut context = unsafe { runtime.upscaler_dx12(&$lab.device) }.create(&options).unwrap();
            eprintln!("Configure native provider: family={} {:?}", $family, context.provider().unwrap());
            for (phase, request) in [TuningValue::Value(value), TuningValue::ProviderDefault].into_iter().enumerate() {
                // New context or previous phase's fenced completion establishes idle.
                if let Err(error) = context.request_tuning(TuningRequest::new(parameter, request).unwrap()) {
                    eprintln!("Configure native rejection: family={} parameter={parameter:?} request={request:?} error={error:?}", $family);
                    eprintln!("Configure terminal cleanup: {:?}", context.destroy());
                    std::process::exit(1);
                }
                eprintln!("Configure native acknowledgement: family={} parameter={parameter:?} request={request:?}; application_unproven=true", $family);
                let mut frame = $lab.open();
                let input = frame.input(phase == 0);
                if let Err(error) = context.validate_upscale_inputs(input.command_list, &input.resources, &input.parameters) {
                    eprintln!("Configure preflight failure: {error:?}");
                    $lab.discard_unsubmitted(&mut frame);
                    eprintln!("Configure cleanup: {:?}", context.destroy());
                    $lab.release(frame);
                    std::process::exit(1);
                }
                // SAFETY: Paired helper supplies correct states and retains every
                // object/context through its queue submission and completion below.
                if let Err(error) = unsafe { context.record(input.command_list, &input.resources, &input.parameters) } {
                    eprintln!("Configure dispatch failure: {error:?}");
                    $lab.discard_unsubmitted(&mut frame);
                    eprintln!("Configure cleanup: {:?}", context.destroy());
                    $lab.release(frame);
                    std::process::exit(1);
                }
                $lab.finish(&frame, true);
                $lab.release(frame);
            }
            context.destroy().unwrap();
            $lab.check();
        }
        let options = UpscalerOptions::new(Dimensions::new(320, 180).unwrap(), Dimensions::new(640, 360).unwrap());
        let mut context = unsafe { runtime.upscaler_dx12(&$lab.device) }.create(&options).unwrap();
        eprintln!("Abandonment native provider: family={} {:?}", $family, context.provider().unwrap());
        for phase in 0..2 {
            let mut frame = $lab.open();
            let input = frame.input(phase == 0);
            // SAFETY: Paired helper provides live objects/states; retain all
            // dependencies until fence or explicit recording invalidation below.
            if let Err(error) = unsafe { context.record(input.command_list, &input.resources, &input.parameters) } {
                $lab.discard_unsubmitted(&mut frame);
                eprintln!("Abandonment recording failure: {error:?}");
                eprintln!("Abandonment cleanup: {:?}", context.destroy());
                $lab.release(frame);
                std::process::exit(1);
            }
            if phase == 0 {
                $lab.finish(&frame, true);
                $lab.release(frame);
                continue;
            }
            let before = context.usability();
            context.mark_recording_abandoned();
            context.mark_recording_abandoned();
            let after = context.usability();
            // SAFETY: Terminal guard must reject before inspecting/recording.
            let rejected_record = unsafe { context.record(input.command_list, &input.resources, &input.parameters) };
            let rejected_query = context.provider();
            // Never submit the abandoned list. Invalidate it before any assertion
            // can unwind; retain textures/allocator through native destruction.
            $lab.discard_unsubmitted(&mut frame);
            let destroyed = context.destroy();
            $lab.release(frame);
            assert_eq!(before, ContextUsability::Usable);
            assert_eq!(after, ContextUsability::Unusable(TerminalCause::RecordingAbandoned));
            assert_result!(rejected_record, Err(Error::ContextUnusable(TerminalCause::RecordingAbandoned)));
            assert_result!(rejected_query, Err(Error::ContextUnusable(TerminalCause::RecordingAbandoned)));
            destroyed.unwrap();
            $lab.check();
            eprintln!("Recording abandonment: PASSED family={} completed_frames=1 abandoned_frames=1 invalidated_before_destroy=true", $family);
            break;
        }
    }};
}

pub(super) fn run() {
    let lab = Lab::acquire();
    // SAFETY: Opt-in signed pinned deployment, actual-provider Configure/query
    // trust, and serialized same-thread calls/cleanup throughout this child.
    exercise!(
        lab,
        "local",
        unsafe { Runtime::load(&lab.loader) }.unwrap(),
        Upscaler
    );
    #[cfg(feature = "movable")]
    {
        // SAFETY: Same native trust plus the accepted movable acquisition contract;
        // this bounded control stays serialized on one thread, without handoff.
        exercise!(
            lab,
            "movable",
            unsafe { crate::runtime::MovableRuntime::load(&lab.loader) }.unwrap(),
            crate::upscaler::MovableUpscaler
        );
    }
    eprintln!(
        "Configure native integration: PASSED parameters=5 phases=2 families={}",
        if cfg!(feature = "movable") { 2 } else { 1 }
    );
}

pub(super) fn selected_provider_dispatch() {
    let lab = Lab::acquire();
    macro_rules! selected_family {
        ($runtime:ty, $owner:ty, $family:literal) => {{
            // SAFETY: Pinned, signed deployment and serialized actual-provider
            // acquisition/query trust; the helper retains the live DX12 device.
            let mut runtime = unsafe { <$runtime>::load(&lab.loader) }.unwrap();
            let options = UpscalerOptions::new(
                Dimensions::new(320, 180).unwrap(),
                Dimensions::new(640, 360).unwrap(),
            );
            let mut default = unsafe { runtime.upscaler_dx12(&lab.device) }
                .create(&options)
                .unwrap();
            let provider = default.provider().unwrap();
            default.destroy().unwrap();

            let mut selected = unsafe { runtime.upscaler_dx12(&lab.device) }
                .with_provider(crate::query::ProviderSelection::Explicit(provider.id()))
                .create(&options)
                .unwrap();
            assert_eq!(selected.provider().unwrap().id(), provider.id());
            eprintln!(
                "Selected dispatch provider: family={} {provider:?}",
                $family
            );
            for phase in 0..2 {
                let mut frame = lab.open();
                let input = frame.input(phase == 0);
                selected
                    .validate_upscale_inputs(
                        input.command_list,
                        &input.resources,
                        &input.parameters,
                    )
                    .unwrap();
                // SAFETY: Same-device/list/resources and declared states from
                // paired equipment, retained through immediate submission/fence.
                if let Err(error) = unsafe {
                    selected.record(input.command_list, &input.resources, &input.parameters)
                } {
                    lab.discard_unsubmitted(&mut frame);
                    eprintln!("Selected dispatch failure: {error:?}");
                    eprintln!("Selected terminal cleanup: {:?}", selected.destroy());
                    lab.release(frame);
                    std::process::exit(1);
                }
                lab.finish(&frame, true);
                lab.release(frame);
            }
            assert_eq!(selected.provider().unwrap().id(), provider.id());
            selected.destroy().unwrap();
            lab.check();
        }};
    }
    selected_family!(Runtime, Upscaler, "local");
    #[cfg(feature = "movable")]
    selected_family!(
        crate::runtime::MovableRuntime,
        crate::upscaler::MovableUpscaler,
        "movable"
    );
    eprintln!(
        "Selected provider dispatch: PASSED frames_per_family=2 families={}",
        if cfg!(feature = "movable") { 2 } else { 1 }
    );
}
