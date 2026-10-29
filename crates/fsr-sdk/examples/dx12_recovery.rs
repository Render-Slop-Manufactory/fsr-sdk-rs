// SPDX-License-Identifier: MPL-2.0

//! Renderer-owned abort/recreation and batched idle Configure workflows.
//! This example creates no runtime/device and performs no work from `main`.

#[cfg(all(windows, feature = "dx12"))]
pub mod renderer_integration {
    use fsr_sdk::{
        error::Error,
        runtime::Runtime,
        upscaler::{
            ID3D12Device, TuningParameter, TuningRequest, TuningValue, Upscaler, UpscalerOptions,
        },
    };

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Replace a failed or successfully recorded-but-abandoned context.
    ///
    /// Keep the old context in `Some` until recording invalidation and earlier
    /// submitted-work completion both succeed. A callback error leaves it there
    /// for terminal cleanup, never for resumed recording: marking precedes both
    /// fallible callbacks. Consuming destruction
    /// removes it before the one native attempt; a destroy/create error leaves
    /// `None`. Failed destroy retains dependencies according to the SDK contract.
    /// A fresh context needs `HistoryAction::Reset` on its first ordinary frame.
    ///
    /// # Safety
    ///
    /// The renderer must invalidate every failed/abandoned unsubmitted recording
    /// referencing the old context (including executable copies), then complete
    /// all earlier submitted work across queues. The callbacks must report OK
    /// only once these conditions hold, and must not submit a failed list.
    /// Retain all referenced resources/allocators until their respective use
    /// completes. Other FSR contexts recorded on an abandoned list also need
    /// terminal cleanup. On callback error/panic the renderer retains these duties.
    /// Uphold Runtime::load and Runtime::upscaler_dx12 contracts, including
    /// lasting effect/provider trust and native coordination through callbacks,
    /// destruction and creation. This helper cannot observe submission or fences.
    pub unsafe fn recreate_after_aborted_recording(
        context: &mut Option<Upscaler>,
        runtime: &mut Runtime,
        device: &ID3D12Device,
        options: UpscalerOptions,
        invalidate_unsubmitted: impl FnOnce() -> Result<()>,
        complete_submitted: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        if let Some(old) = context.as_mut() {
            old.mark_recording_abandoned();
        }
        invalidate_unsubmitted()?;
        complete_submitted()?;
        if let Some(old) = context.take() {
            old.destroy()?;
        }
        *context = Some(unsafe { runtime.upscaler_dx12(device) }.create(&options)?);
        Ok(())
    }

    /// Request desired settings during one established idle interval.
    ///
    /// Check the complete desired list before the first native change. A bad
    /// input leaves the context untouched. The same portable preparation can run
    /// on a worker without a runtime or device.
    ///
    /// Call this on a newly created context before its first recording, or in one
    /// renderer-established idle interval. Native calls are independent: a later
    /// native error makes the context terminal without rolling back earlier
    /// acknowledgements. Success does not prove the provider applied any request.
    /// `&mut` does not prove GPU idle: prior unsafe recordings must have completed
    /// all submitted work and left no executable unsubmitted recording.
    pub fn request_desired_at_idle(
        context: &mut Upscaler,
        desired: &[(TuningParameter, TuningValue)],
    ) -> std::result::Result<(), Error> {
        let requests = desired
            .iter()
            .map(|&(parameter, value)| TuningRequest::new(parameter, value))
            .collect::<std::result::Result<Vec<_>, Error>>()?;
        for request in requests {
            context.request_tuning(request)?;
        }
        Ok(())
    }
}

fn main() {
    println!("Renderer-owned DX12 recovery/configuration reference; no GPU work is started.");
}
