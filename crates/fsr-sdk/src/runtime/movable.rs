// SPDX-License-Identifier: MPL-2.0

use super::{LoadError, RuntimeState};
use std::{cell::Cell, marker::PhantomData, path::Path, sync::Arc};

/// Optional movable Windows/DX12 runtime: `Send`, but never `Sync` or `Clone`.
///
/// Each derived context independently retains this runtime. Dropping the public
/// runtime does not end its unsafe acquisition obligations or unload the loader
/// while contexts retain it. The final release may run on any destination thread.
/// There is no internal lock or concurrency guarantee: callers must coordinate
/// all potentially shared native state, including implicit cleanup and unwinding.
/// See [`Self::load_unchecked`] for the complete lasting contract.
pub struct MovableRuntime {
    pub(crate) state: Arc<RuntimeState>,
    _not_sync: PhantomData<Cell<()>>,
}

impl MovableRuntime {
    /// Load an explicit trusted v2.3.0 FidelityFX loader for movable ownership.
    ///
    /// Path canonicalization, five-export resolution, deployment and error
    /// behavior match [`super::Runtime::load_unchecked`]. The `movable` feature
    /// does not authenticate native inputs or establish AMD thread compatibility.
    /// Safe construction and destruction rely on the lasting requirements below.
    /// Native initialization faults need not become a Rust error.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] for path resolution, loading or a missing export.
    /// Partial-load cleanup finishes inside this call, on its calling thread;
    /// no partially constructed movable runtime is published or transferred.
    ///
    /// # Safety
    ///
    /// The caller must guarantee all of the following:
    ///
    /// 1. The loader and every resolved dependency/provider, including driver
    ///    providers, are trusted, ABI-compatible with the selected v2.3.0 DX12
    ///    profile, and safe to initialize, operate and release, including partial
    ///    acquisition, returned-error and terminal cleanup. Preserve compatible
    ///    process/provider configuration and never prematurely release native
    ///    dependencies needed by the runtime or any derived context. The bounded
    ///    construction trust of [`super::Runtime::load_unchecked`] still applies.
    /// 2. The actual loader, providers, contexts, driver and native dependencies
    ///    permit movement and every subsequent operation on **any thread** where
    ///    these general `Send` owners may be used or dropped. This includes context
    ///    creation, dispatch, affected queries/configuration, explicit destruction,
    ///    implicit `Drop` during normal execution or Rust unwinding, error cleanup,
    ///    COM/device-reference release and final wrapper-owned DLL release/unload.
    ///    Destruction must not require the creating thread; library release must
    ///    not require the acquiring thread. Compatibility restricted to a selected
    ///    handoff or subset of threads is insufficient. Serialization is not proof
    ///    of absence of native thread affinity.
    /// 3. All caller-controlled native operations potentially touching shared
    ///    FidelityFX runtime/provider/global state are serialized and non-reentrant:
    ///    acquisition, creation, dispatch, queries, configuration, explicit/implicit
    ///    destruction, error cleanup and final library release. Coordinate sibling
    ///    contexts, separately acquired runtimes, local and movable families and
    ///    external native users. Different paths/devices or per-context locks do
    ///    not prove isolation; separate coordination domains require established
    ///    isolation. This contract adds no public Query/Configure API.
    /// 4. Synchronization establishes happens-before from prior CPU operations to
    ///    the next thread's use or cleanup. Transfer exclusive ownership only after
    ///    prior CPU operations finish. Correct channels, joins or locks can establish
    ///    ordering; elapsed time, `Send`, `!Sync`, `&mut self` and `Arc` alone cannot
    ///    order operations on independently held contexts or external users.
    /// 5. Ownership and coordination preserve these invariants for every safe call
    ///    and every explicit or automatic destruction, including during unwinding
    ///    and unsuccessful ownership handoffs. Where a guard provides coordination,
    ///    cleanup must finish before it is released; poisoned-lock recovery must
    ///    preserve that invariant. The requirements persist for every derived
    ///    context after this public runtime is dropped. Exceptional native outcomes
    ///    may retain the entire critical dependency bundle indefinitely: retained
    ///    dependencies or disappearing public owners do not establish native
    ///    quiescence or authorize other users to disregard shared-state restrictions.
    ///    Passing an owner to other safe code does not discharge this obligation.
    /// 6. The native implementation safely handles its own hidden workers,
    ///    callbacks and termination under serialized application operations. An
    ///    application lock does not govern internal activity. Any native termination
    ///    is safe on its actual executing thread. Final wrapper-owned loader release
    ///    is distinct from provider/dependency termination; provider unload and its
    ///    time or thread are not promised.
    /// 7. Independently uphold the full D3D12 contract of
    ///    [`MovableUpscaler::dispatch`](crate::upscaler::MovableUpscaler::dispatch):
    ///    valid recording list/allocator, truthful resource states, non-overlapping
    ///    active backing storage, residency, no conflicting access, object lifetimes
    ///    through GPU completion, ordered submission/temporal execution and no
    ///    premature reuse. Complete all submitted work and invalidate unsubmitted
    ///    recordings before context destruction, including during unwinding. CPU
    ///    synchronization does not imply GPU completion. Satisfy any additional
    ///    native preconditions for transfer with outstanding GPU work; historical
    ///    experiments establish only fenced transfers.
    ///
    /// These are caller-established assumptions, **not verified AMD guarantees**,
    /// including for previously tested binaries. If the selected native configuration
    /// and every subsequent use cannot satisfy them, do not call this API.
    /// The accepted [D013](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D013.md)
    /// explains the design and evidence limits; acquisition does not make dispatch safe.
    pub unsafe fn load_unchecked(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        // SAFETY: The caller establishes initialization, failure cleanup and all
        // lasting arbitrary-thread compatibility/coordination requirements above.
        let state = unsafe { RuntimeState::load_unchecked(path) }?;
        Ok(Self {
            state: Arc::new(state),
            _not_sync: PhantomData,
        })
    }
}
