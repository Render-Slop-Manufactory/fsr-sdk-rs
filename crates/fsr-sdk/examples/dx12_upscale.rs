// SPDX-License-Identifier: MPL-2.0

//! Compilable integration reference for a renderer that already owns DX12 objects.
//!
//! `main` only prints guidance; this example does not create a device, acquire
//! AMD binaries or submit GPU work. On Windows with `dx12`, use these functions
//! below from your renderer. Create one context, record the first frame, then
//! close/submit/fence with your own queue. Retain that context for later frames;
//! use renderer-derived timing, camera, jitter and reset values for those frames.

#[cfg(all(windows, feature = "dx12"))]
pub mod renderer_integration {
    //! The supported 320×180 → 640×360 first-frame profile, using only public API.

    use fsr_sdk::{
        error::Error,
        runtime::Runtime,
        upscaler::{
            CameraParameters, Dimensions, ExposureInput, FrameParameters, FrameTimeMillis,
            HistoryAction, ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource,
            JitterOffsetPixels, MotionVectorScale, NativeMotionVectorScale, TuningRequest,
            UpscaleResources, Upscaler, UpscalerOptions,
        },
    };
    use std::path::Path;

    /// Create once on the renderer's FidelityFX thread, before recording frames.
    ///
    /// `loader_path` names the trusted signed SDK v2.3.0 DX12 loader. Deploy
    /// `amd_fidelityfx_upscaler_dx12.dll` beside your executable. The returned
    /// context retains the runtime and its own device reference; no GPU work
    /// is recorded or submitted here.
    ///
    /// # Safety
    ///
    /// Uphold `Runtime::load`'s loader/ABI contract and
    /// `Runtime::upscaler_dx12`'s lasting effect/provider contract, including
    /// teardown contract. Keep wrapper calls on this thread and external FidelityFX use
    /// touching shared state serialized/non-reentrant for the context's lifetime.
    pub unsafe fn create_upscaler(
        loader_path: &Path,
        device: &ID3D12Device,
    ) -> Result<Upscaler, Box<dyn std::error::Error>> {
        // SAFETY: The renderer establishes the runtime acquisition contract.
        let mut runtime = unsafe { Runtime::load(loader_path)? };
        let options = UpscalerOptions::new(Dimensions::new(320, 180)?, Dimensions::new(640, 360)?);
        // SAFETY: The renderer establishes the separate lasting effect contract.
        let mut setup = unsafe { runtime.upscaler_dx12(device) };
        Ok(setup.create(&options)?)
    }

    /// Choose one complete motion encoding; None explicitly selects Pixels.
    /// Native components and jitter contribution must match the actual provider.
    /// Replacing the choice stores no fallback and transforms no input pixels.
    pub fn select_renderer_motion_encoding(
        frame: &mut FrameParameters,
        native_scale: Option<NativeMotionVectorScale>,
    ) {
        frame.set_motion_vector_scale(
            native_scale.map_or(MotionVectorScale::Pixels, MotionVectorScale::Native),
        );
    }

    /// Record the first frame on a context created with the options above.
    ///
    /// The renderer supplies five distinct single-sample/mip/slice Texture2D
    /// resources on the device: 320×180 RGBA16F linear non-HDR scene colour,
    /// ordinary R32F depth, jitter-free current-to-previous top-down UV RG16F
    /// motion, 1×1 R32F exposure containing `1.0`, and 640×360 UAV-capable RGBA16F
    /// output. Colour/depth must have the applied pixel jitter `(0.25, -0.25)` and
    /// match the camera below. This is a reset frame, not a per-frame reset policy.
    /// Additional typed colour/output formats and R16F depth use these same
    /// borrowed-resource APIs when the renderer establishes their device and
    /// actual-provider compatibility; see `UpscaleResources` for the format list.
    ///
    /// # Safety
    ///
    /// Reset/open a DIRECT list and transition the four inputs to
    /// `NON_PIXEL_SHADER_RESOURCE`, output to `UNORDERED_ACCESS`, before calling.
    /// Ensure residency, non-overlapping backing storage and no conflicting GPU
    /// access. Keep the list/allocator, textures/storage and upscaler valid until
    /// submitted work completes. Submit successful recordings once, in temporal
    /// order, and obey `Upscaler::record`'s failure/abandonment rules before
    /// reuse or teardown. Before later safe Configure, finish submitted work and
    /// leave no pending executable recording. This function does not submit,
    /// fence or destroy.
    pub unsafe fn record_first_frame(
        upscaler: &mut Upscaler,
        command_list: &ID3D12GraphicsCommandList,
        color: &ID3D12Resource,
        depth: &ID3D12Resource,
        motion_vectors: &ID3D12Resource,
        exposure: &ID3D12Resource,
        output: &ID3D12Resource,
    ) -> Result<(), Error> {
        let resources = UpscaleResources::builder()
            .color(color)
            .depth(depth)
            .motion_vectors(motion_vectors)
            .exposure(ExposureInput::Texture(exposure))
            .output(output)
            .build()?;
        let frame = FrameParameters::new(
            JitterOffsetPixels::new(0.25, -0.25)?,
            FrameTimeMillis::new(16.667)?,
            CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
            HistoryAction::Reset,
        );
        // SAFETY: The renderer establishes the dispatch state/storage/lifetime contract.
        let recorded = unsafe { upscaler.record(command_list, &resources, &frame) };

        // Renderer-owned next steps:
        // - Ok: restore bindings as needed, close/submit once, then signal/wait a fence.
        // - Native error: never submit this list; invalidate it, finish earlier work,
        //   then tear down the poisoned context. Preflight errors do not poison it.
        // - Do not drop/reuse the context, textures or allocator before completion.
        // Return the result without dropping the caller-owned upscaler on an error.
        recorded
    }

    /// Finish the recorded frame through the renderer, then request one tuning change.
    ///
    /// After `record_first_frame` returns OK, supply a closure that closes and
    /// submits that successful list once, signals its queue fence and waits for
    /// completion. The closure owns the renderer-specific queue/fence details.
    /// After this function returns OK, reset/open the next frame's list and record
    /// with renderer-derived timing/jitter and `HistoryAction::Continue` unless history
    /// actually needs resetting. Keep resources/context alive through that frame's
    /// submission and completion too. No Configure call requests a history reset.
    ///
    /// Prepare a checked `TuningRequest` before recording, using `TuningValue::Value`
    /// or explicit `TuningValue::ProviderDefault` for the provider's own default.
    /// Success acknowledges the request without proving application or support.
    /// A native Configure error makes the context terminal; destroy it normally.
    ///
    /// # Safety
    ///
    /// Uphold the earlier unsafe dispatch and acquisition contracts. The closure
    /// must return OK only after all submitted work on this context has completed,
    /// including cross-queue dependencies, with no executable unsubmitted recording
    /// left for later submission. On a closure error/panic the renderer still owns
    /// completion and failure/abandonment cleanup; this function borrows the context
    /// and never drops it on an error. A CPU return or ownership handoff is not a fence.
    pub unsafe fn finish_frame_and_request_tuning(
        upscaler: &mut Upscaler,
        request: TuningRequest,
        submit_and_wait: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        submit_and_wait()?;
        // Earlier unsafe dispatch and the renderer's completed fence establish idle
        // state. Configure itself remains safe and performs no queue submission/wait.
        upscaler.request_tuning(request)?;
        Ok(())
    }
}

fn main() {
    println!(
        "DX12 integration reference: call create_upscaler and record_first_frame from your renderer."
    );
    println!("Requires Windows with the dx12 feature; no runtime or GPU work is started here.");
}
