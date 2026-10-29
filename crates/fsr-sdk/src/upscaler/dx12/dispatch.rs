// SPDX-License-Identifier: MPL-2.0

//! Narrow Windows/DX12 dispatch orchestration and terminal error policy.
use super::{Upscaler, UpscalerCore};
use crate::context::RuntimeRetention;
use crate::error::Error;
use fsr_sdk_sys::upscale::ffxDispatchDescUpscale;

mod descriptor;
mod frame;
mod reactive_mask;
mod resources;
use super::ID3D12GraphicsCommandList;
use crate::upscaler::FrameParameters;
#[cfg(test)]
use crate::upscaler::common::{CameraParameters, FrameTimeMillis, JitterOffsetPixels};
pub use crate::upscaler::common::{NativeSharpness, Sharpening};
use frame::UpscaleInputs;
pub use frame::{ExposureInput, UpscaleResources, UpscaleResourcesBuilder};
pub use reactive_mask::{ReactiveMaskResources, ReactiveMaskResourcesBuilder};
#[cfg(test)]
pub(super) use resources::inspect_dispatch;
#[cfg(test)]
pub(super) use resources::resource_for_parity;

impl Upscaler {
    /// Check frame/configuration and DX12 metadata without recording FSR commands.
    ///
    /// Uses the same inspection as [`Self::record`], including device/COM
    /// identity, resource requirements, formats, extents and exposure profile.
    /// It calls DX12 inspection methods but no FidelityFX entry point, changes
    /// no context eligibility and retains no independent COM references.
    /// Existing poison rejects before inspection. A usable context remains usable
    /// after either successful validation or an inspection/validation error.
    ///
    /// Useful during setup, even before opening or transitioning the command
    /// list. It cannot check recording state, pixels, actual GPU states, backing
    /// heap aliasing, residency, provider support, submission or completion.
    /// Success is not a capability or GPU-safety certificate. Dispatch always
    /// inspects again; invoking this method is never a prerequisite for safety.
    /// The runtime's lasting thread/coordination contract still applies.
    pub fn validate_upscale_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        self.core
            .validate_upscale_inputs(command_list, resources, parameters)
    }

    /// Record one upscaler dispatch on an open DIRECT command list.
    ///
    /// All list/resources must belong to this context's DX12 device. The wrapper
    /// checks device identity, texture metadata, distinct COM identities and
    /// checked scalars before entering native code. It cannot inspect actual
    /// states, backing-heap aliasing, residency, pixel content or GPU completion.
    /// The supported formats and semantics are documented on [`UpscaleResources`].
    ///
    /// # Input conventions
    ///
    /// These signal conventions are required for the intended rendering within
    /// the actual provider's supported numerical and resource-access domain.
    /// They do not establish that domain or replace the safety obligations below.
    ///
    /// In the default linear profile, colour must already contain the reported
    /// pre-exposure multiplication. Its dynamic range must match the context's
    /// HDR setting. For non-linear controls, establish the actual provider's
    /// supported creation/hint combination, input encoding and normalization,
    /// exposure ordering, output interpretation and temporal transitions.
    /// Allowing a context/hint mismatch bypasses only the wrapper's pairing check;
    /// all these obligations remain. The wrapper performs no encoding,
    /// tonemapping or HDR display configuration.
    /// Supplied masks must contain matching jitter-aligned `[0, 1]` UNORM signals
    /// in the active render region; they are forwarded without transformation.
    /// Depth/projection and motion content must match the context's conventions
    /// and this frame's motion scaling request. With an override, stored
    /// motion including any jitter contribution must match the actual provider's
    /// scale interpretation. Render images occupy the top-left active
    /// region; UV motion is normalized to its active render/display region,
    /// independently of allocation size. Active output may vary on a dynamic
    /// context; output/display-motion allocations remain at maximum size.
    /// Native success acknowledges recording, not application of requested geometry.
    /// The recorded 2.3.4 path ignores smaller output and cannot support that use.
    ///
    /// The call may change command-list descriptor heaps and compute bindings;
    /// restore application bindings afterward as required by the renderer.
    ///
    /// # Safety
    ///
    /// Temporary Rust borrows and metadata checks cannot establish asynchronous
    /// GPU lifetimes, actual states or the native assumptions below. Uphold the
    /// complete safety contract that follows.
    ///
    /// ## GPU lifetime, states and execution
    ///
    /// The list must be open and recording, with its allocator valid through
    /// submission and completion. Colour, depth, motion, supplied masks and any
    /// supplied exposure must be in
    /// `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`; output must be in
    /// `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. Those states are declared to
    /// FidelityFX as compute-read and unordered-access respectively. Keep all
    /// supplied resources, their backing storage, list/allocator and this upscaler
    /// valid until the submitted GPU work completes, and maintain resource
    /// residency. Backing storage must not overlap during dispatch; the wrapper
    /// checks COM identity, not heap aliasing. The whole output allocation must
    /// be available for writes, with no padding preservation. Supply valid input
    /// texels throughout the provider's sampling footprint; inactive texels
    /// cannot be assumed unread.
    ///
    /// Avoid conflicting access, premature input/output reuse, allocator reset
    /// or list resubmission. Submit each successful recording once and execute
    /// successive dispatches on this context in recording order; use queue/fence
    /// waits for cross-queue dependencies. A shared submission batch alone does
    /// not establish temporal execution order. A CPU return is not GPU completion.
    /// Before any subsequent safe [`Self::request_tuning`], complete all
    /// submitted work from upscaling and reactive-mask generation using this
    /// context with renderer-established queue/fence ordering. Leave no
    /// executable unsubmitted recording referencing the context which could be
    /// submitted after Configure. The normal sequence is
    /// record, submit, wait for completion, configure, then record again.
    /// Abandoning an OK recording still requires terminal cleanup rather than
    /// configuration and resumed use. This lasting obligation belongs to this
    /// unsafe dispatch call; safe Configure does not submit or wait on queues.
    /// Before [`Self::destroy`] or `Drop`, complete all submitted work on this
    /// context and invalidate failed or abandoned recordings referencing it.
    ///
    /// After a native error, no resource post-state is promised: never submit
    /// that list, invalidate its recording, complete earlier submitted work,
    /// then destroy the poisoned context. If an OK recording is abandoned,
    /// call [`Self::mark_recording_abandoned`] on every affected context and
    /// invalidate the recording before destruction. The wrapper cannot observe
    /// abandonment automatically. Runtime calls remain
    /// confined to the acquiring thread and externally serialized as required by
    /// [`Runtime`](crate::runtime::Runtime).
    ///
    /// ## Native assumptions and uncertainty
    ///
    /// The trusted provider must support the selected colour/exposure profile,
    /// creation/hint combination and any explicitly allowed mismatch; resource
    /// requirements do not establish colour capability. It must support the
    /// supplied pre-exposure factor and resulting colour signals, literal
    /// per-axis motion scales, their combination with stored motion/jitter, and
    /// the selected format/view profile. Finite scalars and device format bits
    /// do not prove its numerical domain or safe shader accesses. Metadata
    /// inspection cannot check those contents or native guarantees.
    /// The provider must uphold the SDK contract for supplied mask resources;
    /// absence from resource-query bits does not establish that forwarding is forbidden.
    ///
    /// Dynamic contexts require actual-provider support for the selected flag
    /// and active-size transitions. The provider must honour the requested output
    /// and safely access supplied and internal allocations for the selected
    /// transitions, motion and sharpening. Bounds and successful preflight do
    /// not establish native access safety. Temporary CPU descriptors must be
    /// consumed synchronously without retaining their addresses, including on
    /// the terminal error path. The provider must restore declared boundary
    /// states on success and permit existing terminal destruction after an
    /// error. Restoration by the end of a successful recording is trusted for
    /// the selected v2.3.0 baseline, not guaranteed for arbitrary providers.
    ///
    /// These obligations rely on bounded allocation trust in the signed v2.3.0
    /// provider under
    /// [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md):
    /// the selected trusted runtime manages its transient allocations under
    /// these known preconditions. This is not an established allocator guarantee
    /// for arbitrary providers or a promised queue depth. Requested debug-view
    /// flags require actual-provider compatibility, independently of creation
    /// visualization. Maintain installed callback targets and CB backing/ranges
    /// through native access and GPU completion under the lasting
    /// [`super::interop`] contract.
    ///
    /// Retaining temporary CPU pointers can cause dangling accesses; releasing
    /// backing still used by native/GPU work is a memory-lifetime hazard. The
    /// general Rust/host-memory consequences of GPU-only state, aliasing,
    /// residency or ordering violations are not established. These remain
    /// mandatory conservative native safety obligations, not image-quality advice
    /// or a claim that every DX12 violation necessarily causes Rust UB.
    ///
    /// # Errors
    ///
    /// `Ok(())` means only that native recording returned OK; it proves neither
    /// submission nor GPU completion or output quality. [`Error::Input`]
    /// and [`Error::Backend`] occur before native dispatch, record no FSR
    /// commands in this call and leave a usable context unpoisoned. They do not
    /// undo the renderer's earlier commands. A non-OK native return preserves its
    /// code in [`Error::Native`] and poisons later dispatch, which then returns
    /// [`Error::ContextUnusable`] without FFI. Native faults need not return a
    /// Rust error. Teardown remains available under the lifetime rules above.
    pub unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        // SAFETY: The caller accepted this method's GPU lifetime and state
        // obligations. Inspection runs before the native entry point.
        unsafe { self.core.record(command_list, resources, parameters) }
    }

    /// # Safety
    /// The inspected descriptor must satisfy this module's dispatch contract
    /// and the caller must uphold GPU lifetime, state and ordering obligations.
    #[cfg(test)]
    pub(super) unsafe fn dispatch_core(
        &mut self,
        inspect: impl FnOnce(
            &UpscalerCore<std::rc::Rc<crate::runtime::RuntimeState>>,
        ) -> Result<ffxDispatchDescUpscale, Error>,
    ) -> Result<(), Error> {
        // SAFETY: The fixture/native caller upholds the inspected descriptor contract.
        unsafe { self.core.dispatch_core(inspect) }
    }
}

impl<R: RuntimeRetention> UpscalerCore<R> {
    #[cfg(test)]
    pub(super) fn descriptor_for_parity(
        &self,
        dispatch: &super::UpscaleTestInputs<'_>,
    ) -> Result<ffxDispatchDescUpscale, Error> {
        self.inspect_dispatch(&UpscaleInputs::new(
            dispatch.command_list,
            &dispatch.resources,
            &dispatch.parameters,
        ))
    }

    pub(super) fn validate_upscale_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        let dispatch = UpscaleInputs::new(command_list, resources, parameters);
        self.eligibility.check()?;
        self.inspect_dispatch(&dispatch).map(|_| ())
    }

    // SAFETY contract: same renderer and native obligations as public dispatch.
    pub(super) unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        let dispatch = UpscaleInputs::new(command_list, resources, parameters);
        // SAFETY: Caller establishes states, storage, ordering and GPU lifetimes.
        unsafe { self.dispatch_core(|this| this.inspect_dispatch(&dispatch)) }
    }

    pub(super) unsafe fn dispatch_core(
        &mut self,
        inspect: impl FnOnce(&Self) -> Result<ffxDispatchDescUpscale, Error>,
    ) -> Result<(), Error> {
        // A local copy permits inspection to borrow the complete Upscaler.
        // Inspection may unwind before FFI without changing eligibility; the
        // native invocation must not unwind, so its transition is written back.
        let mut poisoned = self.eligibility;
        let result = crate::context_policy::dispatch(
            crate::error::Operation::Upscale,
            &mut poisoned,
            self,
            inspect,
            |this, desc| {
                // SAFETY: The inspected descriptor lives through the call. The
                // caller upholds the native resource/state/GPU lifetime contract.
                unsafe { this.owner.dispatch(&desc.header) }
            },
        );
        self.eligibility = poisoned;
        result
    }
}

#[cfg(test)]
mod tests;
