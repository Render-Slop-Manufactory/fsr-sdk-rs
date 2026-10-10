// SPDX-License-Identifier: MPL-2.0

use super::{Error, ID3D12Device, UpscaleDispatch, UpscalerCore, UpscalerOptions};
use crate::runtime::{MovableRuntime, RuntimeState};
use std::{cell::Cell, marker::PhantomData, sync::Arc};

/// Optional movable DX12 context: `Send + !Sync`, neither `Copy` nor `Clone`.
///
/// Construction requires the matching [`MovableRuntime`]; no conversion between
/// local and movable families is provided. Pinned descriptors, an independent
/// device reference and an independent runtime reference survive outer moves.
/// The last context may release the loader on any thread, even after the public
/// runtime has been dropped. Neither this type nor `Arc` serializes native calls.
///
/// All use and cleanup, including automatic `Drop` during unwinding, must remain
/// inside the caller's common coordination domain under
/// [`MovableRuntime::load_unchecked`]. A per-context mutex is insufficient when
/// siblings or other FidelityFX users share native state. Passing this value to
/// safe code does not discharge that lasting unsafe obligation.
///
/// Like [`super::Upscaler`], native dispatch failure poisons further dispatch.
/// Explicit destruction and `Drop` attempt teardown at most once; failure retains
/// the full critical bundle. `Drop` discards the error and neither cleanup path
/// waits for GPU work. See [`Self::dispatch`] for renderer lifetime obligations.
pub struct MovableUpscaler {
    pub(super) core: UpscalerCore<Arc<RuntimeState>>,
    _not_sync: PhantomData<Cell<()>>,
}

// SAFETY: This implementation applies only to this exact field graph, not the
// generic core or NativeContextOwner. The core owns one non-duplicated native
// handle, Pin<Box<UpscaleCreateState>> (root/version/backend, no callbacks or
// allocators), an independent ID3D12Device reference and Arc<RuntimeState>.
// Pin preserves all descriptor addresses across outer moves. RuntimeState owns
// only FfxLibrary's private ABI function pointers and libloading::Library; Arc
// makes reference counting transferable, with no Rc-owned dependency here.
// UpscalerOptions contains checked NonZeroU32 dimensions; the poison bool moves
// with the owner. Cell's phantom marker is Send and explicitly excludes Sync.
// Dispatch borrows lists/resources only during its CPU call and preserves the
// shared poison policy; the caller retains all GPU objects through completion.
// Terminal ownership is removed before native destroy; success releases pinned
// state, device, then runtime, while destroy failure/OK-null creation retain the
// entire bundle without retry. MovableRuntime's unsafe acquisition establishes
// arbitrary-thread native compatibility and lasting common coordination for
// every call, COM release, explicit/implicit Drop (including unwind) and final
// library release. No incidental native pointer or the generic owner's local
// PhantomData<Rc<()>> marker is dereferenced/mutated concurrently by safe Rust.
unsafe impl Send for MovableUpscaler {}

impl MovableUpscaler {
    /// Create a context from precisely this movable runtime and a borrowed device.
    ///
    /// Dimensions, zero flags, pinned descriptors and error/retention behavior
    /// match [`super::Upscaler::new`]. Native limits may reject or abort. The
    /// context retains its own device and runtime references; both public inputs
    /// may be dropped afterward, under the runtime's lasting acquisition contract.
    /// Creation and its error cleanup must be serialized with every native user
    /// of potentially shared state. No GPU work is submitted here.
    ///
    /// # Errors
    ///
    /// Native failure preserves its code in [`Error::Native`]. OK/null creation
    /// returns [`Error::NativeInvariantViolation`] and retains the critical bundle.
    ///
    /// The families cannot mix:
    ///
    /// ```compile_fail
    /// use fsr_sdk::{runtime::Runtime, upscaler::{ID3D12Device, MovableUpscaler, UpscalerOptions}};
    /// fn wrong(runtime: &Runtime, device: &ID3D12Device, options: UpscalerOptions) {
    ///     let _ = MovableUpscaler::new(runtime, device, options);
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use fsr_sdk::{runtime::MovableRuntime, upscaler::{ID3D12Device, Upscaler, UpscalerOptions}};
    /// fn wrong(runtime: &MovableRuntime, device: &ID3D12Device, options: UpscalerOptions) {
    ///     let _ = Upscaler::new(runtime, device, options);
    /// }
    /// ```
    pub fn new(
        runtime: &MovableRuntime,
        device: &ID3D12Device,
        options: UpscalerOptions,
    ) -> Result<Self, Error> {
        Ok(Self {
            core: UpscalerCore::new(runtime.state.clone(), device, options)?,
            _not_sync: PhantomData,
        })
    }

    /// Record one fixed-profile dispatch; no submission or GPU wait is performed.
    ///
    /// Validation, metadata/device inspection and poison behavior share the local
    /// [`super::Upscaler::dispatch`] implementation. Pre-native validation/inspection
    /// failures leave the context usable; a returned native error preserves its
    /// code and poisons later dispatch, rejected before inspection or FFI.
    /// `Ok(())` establishes only successful native recording, not GPU completion,
    /// correct output, native thread compatibility or an allowed queue depth.
    ///
    /// # Safety
    ///
    /// Uphold [`MovableRuntime::load_unchecked`]'s lasting common-domain
    /// serialization, non-reentrancy, native compatibility and cleanup contract.
    /// Independently uphold every obligation of [`super::Upscaler::dispatch`]:
    ///
    /// - Supply an open recording DIRECT list on the context's device. Retain the
    ///   list and its allocator through submission/completion as D3D12 requires.
    /// - Color/depth/motion/exposure must be in non-pixel-shader-resource state;
    ///   output must be in unordered-access state. Keep truthful states, residency
    ///   and non-overlapping active backing storage; avoid conflicting access.
    /// - Keep all five textures, backing allocations and this context alive until
    ///   submitted GPU work completes. Rust borrows end after the CPU call.
    /// - Submit each successful recording once, with successive temporal dispatches
    ///   executing in recording order and required cross-queue waits. Avoid
    ///   premature resource reuse, allocator reset or list resubmission. Neither
    ///   CPU handoff nor a shared submission batch establishes GPU ordering.
    /// - Before destruction or `Drop`, including unwind, finish all submitted work
    ///   and invalidate all unsubmitted recordings referencing this context.
    ///   Never submit a list containing failed native recording. After a native
    ///   error, no resource post-state is promised; finish earlier work then tear
    ///   down. Abandoning an OK recording likewise requires stopping dispatch and
    ///   terminal cleanup. Restore application descriptor heaps/bindings as needed.
    ///
    /// D011's bounded signed-runtime transient-allocation/teardown trust remains;
    /// no provider concurrency, universal restoration or outstanding-work transfer
    /// guarantee is added. Native transfer preconditions must hold independently.
    pub unsafe fn dispatch(&mut self, dispatch: UpscaleDispatch<'_>) -> Result<(), Error> {
        // SAFETY: Caller establishes both lasting native and per-dispatch GPU rules.
        unsafe { self.core.dispatch(dispatch) }
    }

    /// Consume this context and attempt native destruction once, on this thread.
    ///
    /// The acquisition's coordination must cover the complete cleanup, including
    /// device/runtime release. Complete submitted GPU work and invalidate failed
    /// or abandoned recordings first; this method does not wait. Destruction is
    /// available after poison. Failure preserves the native code in [`Error::Native`]
    /// and retains the full critical bundle indefinitely, without a `Drop` retry.
    pub fn destroy(self) -> Result<(), Error> {
        self.core.owner.destroy()
    }
}
