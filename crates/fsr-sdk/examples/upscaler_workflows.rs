// SPDX-License-Identifier: MPL-2.0

//! Complete public consumer flows for local and movable ownership.
//! Each family exposes first_frame, selected_provider and record_then_abandon.
//! The verification runner compiles this example as a separate downstream crate
//! on stable and MSRV. main starts no native or GPU work.
#![deny(unused_must_use)]

use fsr_sdk::upscaler::{
    CameraParameters, FrameParameters, FrameTimeMillis, HistoryAction, JitterOffsetPixels,
};

/// Portable preparation: the renderer supplies checked, current values and
/// explicit history intent. Can run on a worker without a runtime or device.
pub fn prepare_frame(
    jitter: JitterOffsetPixels,
    time: FrameTimeMillis,
    camera: CameraParameters,
    history: HistoryAction,
) -> FrameParameters {
    FrameParameters::new(jitter, time, camera, history)
}

/// Reuse storage while updating every required per-frame value explicitly.
/// Continuing an existing history requires Continue; a clone preserves Reset.
pub fn update_frame(
    frame: &mut FrameParameters,
    jitter: JitterOffsetPixels,
    time: FrameTimeMillis,
    camera: CameraParameters,
    history: HistoryAction,
) {
    frame.set_jitter(jitter);
    frame.set_frame_time(time);
    frame.set_camera(camera);
    frame.set_history_action(history);
}

#[cfg(all(windows, feature = "dx12"))]
mod dx12 {
    use super::*;
    use fsr_sdk::{
        error::Error,
        query::QualityMode,
        upscaler::{
            ContextUsability, Dimensions, ExposureInput, UpscalerOptions,
            dx12::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource, UpscaleResources},
        },
    };
    use std::path::Path;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Named roles borrow the renderer's objects, with no COM inspection or
    /// independent retention. These borrows do not extend GPU lifetime.
    pub fn bind<'a>(
        color: &'a ID3D12Resource,
        depth: &'a ID3D12Resource,
        motion: &'a ID3D12Resource,
        exposure: &'a ID3D12Resource,
        output: &'a ID3D12Resource,
    ) -> std::result::Result<UpscaleResources<'a>, Error> {
        UpscaleResources::builder()
            .color(color)
            .depth(depth)
            .motion_vectors(motion)
            .exposure(ExposureInput::Texture(exposure))
            .output(output)
            .build()
    }

    // Mechanical duplication stays in this example, not a public ownership trait.
    macro_rules! family {
        ($module:ident, $runtime:ident, $setup:ident, $owner:ident) => {
            pub mod $module {
                use super::*;
                use fsr_sdk::{runtime::$runtime, upscaler::dx12::{$setup, $owner}};

                /// Initialise once and complete the first 320x180 -> 640x360 frame.
                /// The caller's slot must be empty. It receives the owner before
                /// recording, so any later error/panic leaves it available for
                /// renderer-managed cleanup. Success retains it for later frames;
                /// runtime/setup borrows have ended. Do not call this every frame.
                ///
                /// # Safety
                /// Uphold the concrete runtime's load/acquisition contracts and
                /// owner's record contract, including actual-provider compatibility,
                /// common native coordination and all allowed destination threads
                /// for movable ownership. Supply matching explicit-exposure linear
                /// resources, correct states and a live DIRECT list, with Reset
                /// on the initial frame. The closure must submit successful work
                /// once, in order, and return OK only after genuine GPU completion,
                /// leaving no executable pending recording. On error/panic, retain
                /// slot, resources, list and allocator until their obligations
                /// finish; mark every context on any abandoned list. A callback
                /// error does not reveal whether submission occurred. No callback
                /// is called on record failure; never submit a failed recording.
                pub unsafe fn first_frame(
                    loader: &Path,
                    device: &ID3D12Device,
                    slot: &mut Option<$owner>,
                    list: &ID3D12GraphicsCommandList,
                    resources: &UpscaleResources<'_>,
                    parameters: &FrameParameters,
                    submit_and_complete: impl FnOnce() -> Result<()>,
                ) -> Result<()> {
                    if slot.is_some() {
                        return Err("first_frame needs an empty context slot".into());
                    }
                    // SAFETY: Caller establishes loader and lasting effect trust.
                    let mut runtime = unsafe { $runtime::load(loader)? };
                    let options = UpscalerOptions::new(
                        Dimensions::new(320, 180)?, Dimensions::new(640, 360)?);
                    *slot = Some(unsafe { runtime.upscaler_dx12(device) }.create(&options)?);
                    // SAFETY: Renderer supplies and retains all recording dependencies.
                    unsafe { slot.as_mut().unwrap().record(list, resources, parameters)? };
                    submit_and_complete()?;
                    // On subsequent frames use update_frame with current values
                    // and Continue when appropriate, then record on this same owner.
                    Ok(())
                }

                /// Use an explicit provider for planning and creation, with fresh
                /// native membership checks and no fallback. The returned owner
                /// does not borrow setup/runtime/device. The acquiring caller
                /// retains the lasting native trust/coordination obligations.
                /// Results guide allocations, not capability or GPU readiness.
                pub fn selected_provider(
                    mut setup: $setup<'_, '_>,
                    options: &UpscalerOptions,
                    provider: fsr_sdk::query::ProviderId,
                ) -> Result<$owner> {
                    setup = setup.with_provider(fsr_sdk::query::ProviderSelection::Explicit(provider));
                    let render = setup.render_size_for_quality(options.max_upscale_size(), QualityMode::Quality)?;
                    let requirements = setup.resource_requirements()?;
                    let memory = setup.estimate_memory(options)?;
                    eprintln!("quality render={render:?}, requirements={requirements:?}, memory={memory:?}");
                    Ok(setup.create(options)?)
                }

                /// Compose the selected optional utility through the same setup.
                /// Auto skips enumeration; explicit choices reject absent or
                /// ambiguous names without certifying native compatibility.
                #[cfg(feature = "utils")]
                pub fn generation_choice(
                    mut setup: $setup<'_, '_>,
                    options: &UpscalerOptions,
                    choice: fsr_sdk::utils::provider::FsrChoice,
                ) -> Result<$owner> {
                    use fsr_sdk::{query::ProviderSelection, utils::provider::{FsrChoice, select_provider}};
                    let selection = if choice == FsrChoice::Auto {
                        ProviderSelection::Default
                    } else {
                        select_provider(&setup.providers()?, choice)?
                    };
                    Ok(setup.with_provider(selection).create(options)?)
                }

                /// Attempt recording, deliberately abandon the whole list and
                /// destroy only after invalidation and earlier GPU completion.
                /// The slot must contain the affected owner. A native error is
                /// returned after cleanup; marking preserves its first cause.
                /// Either callback error/panic leaves a terminal owner in Some.
                /// A destroy error consumes the slot and retains dependencies
                /// under the production teardown policy, without retry.
                ///
                /// # Safety
                /// Uphold this family's complete acquisition/recording contract
                /// and common coordination through callbacks and destruction.
                /// The renderer intentionally discards this list, even if the
                /// current call rejects metadata; account for all earlier commands.
                /// Never submit it. Invalidate every executable copy before the
                /// first closure returns OK; the second returns OK only after
                /// all earlier submitted work across queues genuinely completes.
                /// Retain resources/list/allocator dependencies as needed on every
                /// path, including error/panic. Independently mark every other
                /// context affected by this list; this helper knows only one.
                /// No status or CPU return proves completion/callback quiescence.
                pub unsafe fn record_then_abandon(
                    slot: &mut Option<$owner>,
                    list: &ID3D12GraphicsCommandList,
                    resources: &UpscaleResources<'_>,
                    parameters: &FrameParameters,
                    invalidate_unsubmitted: impl FnOnce() -> Result<()>,
                    complete_submitted: impl FnOnce() -> Result<()>,
                ) -> Result<ContextUsability> {
                    let context = slot.as_mut().ok_or("no context to abandon")?;
                    // SAFETY: Caller establishes recording trust and retains inputs.
                    let recorded = unsafe { context.record(list, resources, parameters) };
                    context.mark_recording_abandoned();
                    let cause = context.usability();
                    invalidate_unsubmitted()?;
                    complete_submitted()?;
                    slot.take().unwrap().destroy()?;
                    recorded?;
                    Ok(cause)
                }
            }
        };
    }
    family!(local, Runtime, UpscalerSetup, Upscaler);
    #[cfg(feature = "movable")]
    family!(
        movable,
        MovableRuntime,
        MovableUpscalerSetup,
        MovableUpscaler
    );
}

#[cfg(all(windows, feature = "dx12"))]
pub use dx12::*;

fn main() {
    println!("Consumer integration references only; no runtime or GPU work is started.");
}
