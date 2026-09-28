// SPDX-License-Identifier: MPL-2.0

//! Private terminal lifecycle ownership. No effect payload interpretation.

use crate::{
    error::{Error, InvariantViolation, Operation},
    runtime::RuntimeState,
};
#[cfg(all(windows, feature = "dx12"))]
use fsr_sdk_sys::api::ffxDispatchDescHeader;
use fsr_sdk_sys::api::{FFX_API_RETURN_OK, ffxContext, ffxCreateContextDescHeader};
use std::{marker::PhantomData, mem, ptr::null_mut, rc::Rc};

// Field order is cleanup order: retained state, owned device, then runtime.
// D is a private owned dependency, not a backend trait or public extension point.
// Production instantiates it only with an owned ID3D12Device; tests use drop spies.
struct Dependencies<S, D> {
    state: S,
    device: D,
    runtime: Rc<RuntimeState>,
}

struct LiveContext<S, D> {
    handle: ffxContext,
    dependencies: Dependencies<S, D>,
}

pub(crate) struct NativeContextOwner<S, D> {
    live: Option<LiveContext<S, D>>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl<S, D> NativeContextOwner<S, D> {
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
        runtime: Rc<RuntimeState>,
        root: *mut ffxCreateContextDescHeader,
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
                .library
                .ffxCreateContext(&mut handle, root, std::ptr::null())
        };
        if code != FFX_API_RETURN_OK {
            // No output-handle operation, including destroy, even if non-null.
            // Ordered RAII uses D005; it asserts no provider-internal rollback.
            return Err(Error::Native {
                operation: Operation::Create,
                code,
            });
        }
        if handle.is_null() {
            // Native success could have retained dependencies without returning
            // a usable handle. Retain state, device AND runtime; never destroy.
            mem::forget(dependencies);
            return Err(Error::NativeInvariantViolation(
                InvariantViolation::NullContextOnSuccess,
            ));
        }
        // Infallible ownership establishment after native success.
        Ok(Self {
            live: Some(LiveContext {
                handle,
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
                .library
                .ffxDestroyContext(&mut live.handle, std::ptr::null())
        };
        if code != FFX_API_RETURN_OK {
            mem::forget(live.dependencies);
            return Err(Error::Native {
                operation: Operation::Destroy,
                code,
            });
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
    #[cfg(all(windows, feature = "dx12"))]
    pub(crate) unsafe fn dispatch(&mut self, desc: *const ffxDispatchDescHeader) -> u32 {
        let live = self.live.as_mut().expect("live owner");
        // SAFETY: The caller supplies the native descriptor and GPU obligations;
        // this owner retains the context, device and library for the CPU call.
        unsafe {
            live.dependencies
                .runtime
                .library
                .ffxDispatch(&mut live.handle, desc)
        }
    }

    // Native provider instrumentation is test-only; no production handle accessor.
    #[cfg(all(test, windows, feature = "dx12"))]
    pub(crate) unsafe fn query_for_test(
        &mut self,
        desc: *mut fsr_sdk_sys::api::ffxQueryDescHeader,
    ) -> u32 {
        let live = self.live.as_mut().expect("live owner");
        // SAFETY: Test caller supplies a valid descriptor; ownership stays live.
        unsafe {
            live.dependencies
                .runtime
                .library
                .ffxQuery(&mut live.handle, desc)
        }
    }
}

impl<S, D> Drop for NativeContextOwner<S, D> {
    fn drop(&mut self) {
        let _ = self.teardown();
    }
}

#[cfg(test)]
#[path = "context/tests.rs"]
mod tests;
