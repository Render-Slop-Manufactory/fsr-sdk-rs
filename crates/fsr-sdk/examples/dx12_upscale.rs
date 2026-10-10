// SPDX-License-Identifier: MPL-2.0

//! Compilable integration reference for a renderer that already owns DX12 objects.
//!
//! `main` only prints guidance; this example does not create a device, acquire
//! AMD binaries or submit GPU work. On Windows with `dx12`, use the two functions
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
            CameraParameters, Dimensions, FrameTimeMillis, ID3D12Device, ID3D12GraphicsCommandList,
            ID3D12Resource, JitterOffsetPixels, UpscaleDispatch, UpscaleResources, Upscaler,
            UpscalerOptions,
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
    /// Uphold `Runtime::load_unchecked`'s trusted ABI/provider, initialization and
    /// teardown contract. Keep wrapper calls on this thread and external FidelityFX use
    /// touching shared state serialized/non-reentrant for the context's lifetime.
    pub unsafe fn create_upscaler(
        loader_path: &Path,
        device: &ID3D12Device,
    ) -> Result<Upscaler, Box<dyn std::error::Error>> {
        // SAFETY: The renderer establishes the runtime acquisition contract.
        let runtime = unsafe { Runtime::load_unchecked(loader_path)? };
        let options = UpscalerOptions::new(Dimensions::new(320, 180)?, Dimensions::new(640, 360)?);
        Ok(Upscaler::new(&runtime, device, options)?)
    }

    /// Record the first frame on a context created with the options above.
    ///
    /// The renderer supplies five distinct single-sample/mip/slice Texture2D
    /// resources on the device: 320×180 RGBA16F linear non-HDR scene color,
    /// ordinary R32F depth, jitter-free current-to-previous top-down UV RG16F
    /// motion, 1×1 R32F exposure containing `1.0`, and 640×360 UAV-capable RGBA16F
    /// output. Color/depth must have the applied pixel jitter `(0.25, -0.25)` and
    /// match the camera below. This is a reset frame, not a per-frame reset policy.
    ///
    /// # Safety
    ///
    /// Reset/open a DIRECT list and transition the four inputs to
    /// `NON_PIXEL_SHADER_RESOURCE`, output to `UNORDERED_ACCESS`, before calling.
    /// Ensure residency, non-overlapping backing storage and no conflicting GPU
    /// access. Keep the list/allocator, textures/storage and upscaler valid until
    /// submitted work completes. Submit successful recordings once, in temporal
    /// order, and obey `Upscaler::dispatch`'s failure/abandonment rules before
    /// reuse or teardown. This function does not submit, fence or destroy.
    pub unsafe fn record_first_frame(
        upscaler: &mut Upscaler,
        command_list: &ID3D12GraphicsCommandList,
        color: &ID3D12Resource,
        depth: &ID3D12Resource,
        motion_vectors: &ID3D12Resource,
        exposure: &ID3D12Resource,
        output: &ID3D12Resource,
    ) -> Result<(), Error> {
        let frame = UpscaleDispatch::new(
            command_list,
            UpscaleResources::new(color, depth, motion_vectors, exposure, output),
            JitterOffsetPixels::new(0.25, -0.25)?,
            FrameTimeMillis::new(16.667)?,
            CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
            true,
        );
        // SAFETY: The renderer establishes the dispatch state/storage/lifetime contract.
        let recorded = unsafe { upscaler.dispatch(frame) };

        // Renderer-owned next steps:
        // - Ok: restore bindings as needed, close/submit once, then signal/wait a fence.
        // - Native error: never submit this list; invalidate it, finish earlier work,
        //   then tear down the poisoned context. Preflight errors do not poison it.
        // - Do not drop/reuse the context, textures or allocator before completion.
        // Return the result without dropping the caller-owned upscaler on an error.
        recorded
    }
}

fn main() {
    println!(
        "DX12 integration reference: call create_upscaler and record_first_frame from your renderer."
    );
    println!("Requires Windows with the dx12 feature; no runtime or GPU work is started here.");
}
