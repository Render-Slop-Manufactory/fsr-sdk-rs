// SPDX-License-Identifier: MPL-2.0

//! Narrow Windows/DX12 dispatch orchestration and terminal error policy.
use super::Upscaler;
use crate::error::Error;
use fsr_sdk_sys::upscale::ffxDispatchDescUpscale;

mod descriptor;
mod frame;
mod resources;
pub use frame::{CameraParameters, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch};
#[cfg(test)]
pub(super) use resources::inspect_dispatch;
#[cfg(test)]
pub(super) use resources::resource_for_parity;

impl Upscaler {
    /// Record one upscaler dispatch on an open DIRECT command list.
    ///
    /// All list/resources must belong to this context's DX12 device. The wrapper
    /// checks device identity, texture metadata, distinct COM identities and
    /// checked scalars before entering native code. It cannot inspect actual
    /// states, backing-heap aliasing, residency, pixel content or GPU completion.
    /// The supported formats and semantics are documented on [`UpscaleDispatch`].
    ///
    /// # Safety
    ///
    /// The list must be open and recording, with its allocator valid through
    /// submission and completion. Color, depth, motion and exposure must be in
    /// `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`; output must be in
    /// `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. Those states are declared to
    /// FidelityFX as compute-read and unordered-access respectively. Keep all
    /// five resources, their backing storage, list/allocator and this upscaler
    /// valid until the submitted GPU work completes, and maintain resource
    /// residency. Backing storage must not overlap during dispatch; the wrapper
    /// checks COM identity, not heap aliasing.
    /// Avoid conflicting access, premature input/output reuse, allocator reset
    /// or list resubmission. Submit each successful recording once and execute
    /// successive dispatches on this context in recording order; use queue/fence
    /// waits for cross-queue dependencies. A shared submission batch alone does
    /// not establish temporal execution order. A CPU return is not GPU completion.
    /// Before [`Self::destroy`] or `Drop`, complete all submitted work on this
    /// context and invalidate failed or abandoned recordings referencing it.
    ///
    /// On success the selected v2.3.0 baseline records restoration to the declared
    /// boundary states by the end of the recorded dispatch. The call may change
    /// command-list descriptor heaps and compute bindings; restore application
    /// bindings afterward.
    /// After a native error, no resource post-state is promised: never submit
    /// that list, invalidate its recording, complete earlier submitted work,
    /// then destroy the poisoned context. If an OK recording is abandoned,
    /// likewise stop dispatching with that context and invalidate it before
    /// destruction. The wrapper cannot observe abandonment. These obligations
    /// rely on bounded allocation trust in the signed v2.3.0 provider under
    /// [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md):
    /// the selected trusted runtime manages its transient allocations under
    /// these known preconditions. This is not an established allocator guarantee
    /// for arbitrary providers or a promised queue depth. Runtime calls remain
    /// confined to the acquiring thread and externally serialized as required by
    /// [`Runtime`](crate::runtime::Runtime).
    ///
    /// # Return value and errors
    ///
    /// `Ok(())` means only that native recording returned OK; it proves neither
    /// submission nor GPU completion or output quality. [`Error::InvalidDispatch`]
    /// and [`Error::Dx12Inspection`] occur before native dispatch, record no FSR
    /// commands in this call and leave a usable context unpoisoned. They do not
    /// undo the renderer's earlier commands. A non-OK native return preserves its
    /// code in [`Error::Native`] and poisons later dispatch, which then returns
    /// [`Error::DispatchPoisoned`] without FFI. Native faults need not return a
    /// Rust error. Teardown remains available under the lifetime rules above.
    pub unsafe fn dispatch(&mut self, dispatch: UpscaleDispatch<'_>) -> Result<(), Error> {
        // SAFETY: The caller accepted this method's GPU lifetime and state
        // obligations. Inspection runs before the native entry point.
        unsafe { self.dispatch_core(|this| this.inspect_dispatch(dispatch)) }
    }

    /// # Safety
    /// The inspected descriptor must satisfy this module's dispatch contract
    /// and the caller must uphold GPU lifetime, state and ordering obligations.
    pub(super) unsafe fn dispatch_core(
        &mut self,
        inspect: impl FnOnce(&Self) -> Result<ffxDispatchDescUpscale, Error>,
    ) -> Result<(), Error> {
        // A local copy permits inspection to borrow the complete Upscaler.
        // Inspection may unwind before FFI without changing eligibility; the
        // native invocation must not unwind, so its transition is written back.
        let mut poisoned = self.dispatch_poisoned;
        let result =
            crate::dispatch_policy::dispatch(&mut poisoned, self, inspect, |this, desc| {
                // SAFETY: The inspected descriptor lives through the call. The
                // caller upholds the native resource/state/GPU lifetime contract.
                unsafe { this.owner.dispatch(&desc.header) }
            });
        self.dispatch_poisoned = poisoned;
        result
    }
}

#[cfg(test)]
mod tests;
