// SPDX-License-Identifier: MPL-2.0

//! Private terminal lifecycle ownership. No effect payload interpretation.

use crate::{
    error::{Error, InvariantViolation, Operation},
    runtime::RuntimeState,
};
#[cfg(any(test, all(windows, feature = "dx12")))]
use fsr_sdk_sys::api::ffxDispatchDescHeader;
use fsr_sdk_sys::api::{
    FFX_API_RETURN_OK, ffxAllocationCallbacks, ffxContext, ffxCreateContextDescHeader,
};
use std::{marker::PhantomData, mem, ptr::null_mut, rc::Rc, sync::Arc};

#[cfg(test)]
thread_local! {
    // Measurement equipment can suppress the test-only environment lookup.
    pub(crate) static SUPPRESS_DESTROY_TRACE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

mod sealed {
    use super::*;

    pub trait Retention {}
    impl Retention for Rc<RuntimeState> {}
    impl Retention for Arc<RuntimeState> {}
}

// Closed ownership choices: local Rc and optional movable Arc. The generic
// native owner remains !Send + !Sync; only the audited public movable type is Send.
pub(crate) trait RuntimeRetention: sealed::Retention {
    fn runtime_state(&self) -> &RuntimeState;
}

impl RuntimeRetention for Rc<RuntimeState> {
    fn runtime_state(&self) -> &RuntimeState {
        self
    }
}

impl RuntimeRetention for Arc<RuntimeState> {
    fn runtime_state(&self) -> &RuntimeState {
        self
    }
}

// Field order is cleanup order: retained state, owned device, then runtime.
// D is a private owned dependency, not a backend trait or public extension point.
// Production instantiates it only with an owned ID3D12Device; tests use drop spies.
struct Dependencies<S, D, R> {
    state: S,
    device: D,
    runtime: R,
}

struct LiveContext<S, D, R> {
    handle: ffxContext,
    allocator: *const ffxAllocationCallbacks,
    dependencies: Dependencies<S, D, R>,
}

pub(crate) struct NativeContextOwner<S, D, R: RuntimeRetention = Rc<RuntimeState>> {
    live: Option<LiveContext<S, D, R>>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl<S, D, R: RuntimeRetention> NativeContextOwner<S, D, R> {
    /// # Safety
    /// `root` must point into valid, address-stable storage retained by `state`.
    /// It must describe the matching owned device and runtime. All referenced
    /// dependencies must be owned, remain valid even when forgotten, and have
    /// non-panicking destructors. S/D moves must not invalidate native pointers.
    /// Uphold native threading requirements; no pending work or callback may
    /// require extra shutdown before destroy. Creation supplies no callbacks;
    /// dispatch has separate unsafe GPU-completion obligations. Null/default
    /// allocators must be valid for both create and destroy.
    ///
    /// Returned-error RAII relies on D005's runtime trust assumption. This unsafe
    /// boundary specifies caller obligations; it does not prove that assumption.
    pub(crate) unsafe fn create(
        state: S,
        device: D,
        runtime: R,
        root: *mut ffxCreateContextDescHeader,
    ) -> Result<Self, Error> {
        // SAFETY: Ordinary creation preserves the default allocator/D005 profile.
        unsafe {
            Self::create_with_allocator(state, device, runtime, root, std::ptr::null(), false)
        }
    }

    /// # Safety
    /// The chain must match the retained runtime and owned device. Wrapper-owned
    /// dependencies must remain valid when forgotten and have non-panicking
    /// destructors; moves of S/D/R must not invalidate native addresses. Uphold
    /// native threading and common-domain coordination through every operation.
    /// Retain a stable descriptor chain and optional host table in state. Their
    /// addresses remain valid through successful teardown or exceptional retention.
    /// External callback targets obey the installer's lasting lifetime, allocator,
    /// no-unwind and threading contract, including after errors and destruction.
    /// A successful destroy ends native access to wrapper-owned storage, not
    /// necessarily calls to copied global function pointers. All GPU work is idle.
    pub(crate) unsafe fn create_with_allocator(
        state: S,
        device: D,
        runtime: R,
        root: *mut ffxCreateContextDescHeader,
        allocator: *const ffxAllocationCallbacks,
        retain_create_error: bool,
    ) -> Result<Self, Error> {
        let dependencies = Dependencies {
            state,
            device,
            runtime,
        };
        let mut handle = null_mut();
        // SAFETY: The caller establishes the valid retained chain and native
        // preconditions; dependencies are already owned before native creation.
        let code = unsafe {
            dependencies
                .runtime
                .runtime_state()
                .create_context(&mut handle, root, allocator)
        };
        if code != FFX_API_RETURN_OK {
            // No output-handle operation, including destroy, even if non-null.
            // Ordinary RAII uses D005. New callbacks instead retain the bundle:
            // an error does not prove release of newly registered dependencies.
            if retain_create_error {
                mem::forget(dependencies);
            }
            return Err(Error::native(Operation::Create, code));
        }
        if handle.is_null() {
            // Native success could have retained dependencies without returning
            // a usable handle. Retain state, device AND runtime; never destroy.
            mem::forget(dependencies);
            return Err(Error::invariant(
                crate::error::Operation::Create,
                InvariantViolation::NullContextOnSuccess,
            ));
        }
        // Infallible ownership establishment after native success.
        Ok(Self {
            live: Some(LiveContext {
                handle,
                allocator,
                dependencies,
            }),
            _not_send_sync: PhantomData,
        })
    }

    pub(crate) fn destroy(mut self) -> Result<(), Error> {
        self.teardown()
    }

    fn teardown(&mut self) -> Result<(), Error> {
        // Terminal before entering native code; Drop cannot retry this handle.
        let Some(mut live) = self.live.take() else {
            return Ok(());
        };
        // SAFETY: Only successful non-null creation establishes LiveContext.
        // All dependencies remain owned. The caller of dispatch must
        // finish submitted GPU work before this owner reaches teardown.
        let code = unsafe {
            live.dependencies
                .runtime
                .runtime_state()
                .destroy_context(&mut live.handle, live.allocator)
        };
        #[cfg(test)]
        if !SUPPRESS_DESTROY_TRACE.with(|flag| flag.get())
            && std::env::var_os("FSR_SDK_TEST_CONTEXT_TRACE").is_some()
        {
            use std::io::Write as _;
            let _ = writeln!(
                std::io::stderr().lock(),
                "Context native destroy: thread={:?} code={code:#010x}",
                std::thread::current().id()
            );
        }
        if code != FFX_API_RETURN_OK {
            mem::forget(live.dependencies);
            return Err(Error::native(Operation::Destroy, code));
        }
        // Drop in the documented order, regardless of post-destroy handle bits.
        let Dependencies {
            state,
            device,
            runtime,
        } = live.dependencies;
        drop(state);
        drop(device);
        drop(runtime);
        Ok(())
    }

    #[cfg(all(windows, feature = "dx12"))]
    pub(crate) fn device(&self) -> &D {
        &self.live.as_ref().expect("live owner").dependencies.device
    }

    /// # Safety
    /// The descriptor, list and resources must satisfy the native contract.
    /// The caller must retain the allocator, resources, context and runtime
    /// through GPU completion, with truthful states and ordered submission.
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) unsafe fn dispatch(&mut self, desc: *const ffxDispatchDescHeader) -> u32 {
        let live = self.live.as_mut().expect("live owner");
        // SAFETY: The caller supplies the native descriptor and GPU obligations;
        // this owner retains the context, device and library for the CPU call.
        unsafe {
            live.dependencies
                .runtime
                .runtime_state()
                .dispatch(&mut live.handle, desc)
        }
    }

    /// # Safety
    /// Establish the actual provider's Configure pointer lifetime, GPU/CPU
    /// synchronization, continuation and handle validity. This is raw forwarding.
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) unsafe fn configure(
        &mut self,
        desc: *const fsr_sdk_sys::api::ffxConfigureDescHeader,
    ) -> u32 {
        let live = self.live.as_mut().expect("live owner");
        // SAFETY: Caller supplies operation-specific native obligations; this
        // owner retains the complete dependency bundle and matched live handle.
        unsafe {
            live.dependencies
                .runtime
                .runtime_state()
                .configure(&mut live.handle, desc)
        }
    }

    /// # Safety
    /// Supply a correctly tagged query with initialized outputs under the
    /// acquisition's query lifetime, synchronization and non-retention contract.
    #[cfg(all(windows, feature = "dx12"))]
    pub(crate) unsafe fn query(&mut self, desc: *mut fsr_sdk_sys::api::ffxQueryDescHeader) -> u32 {
        let live = self.live.as_mut().expect("live owner");
        // SAFETY: Caller supplies a valid descriptor; ownership stays live.
        unsafe {
            live.dependencies
                .runtime
                .runtime_state()
                .query(&mut live.handle, desc)
        }
    }
}

impl<S, D, R: RuntimeRetention> Drop for NativeContextOwner<S, D, R> {
    fn drop(&mut self) {
        let _ = self.teardown();
    }
}

#[cfg(test)]
#[path = "context/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "context/movable.rs"]
mod movable;
