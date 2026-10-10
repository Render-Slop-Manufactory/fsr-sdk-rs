// SPDX-License-Identifier: MPL-2.0

//! Private native acquisition and invocation boundary.
//! Host fixtures share invocation and ownership through their loader adapter.
//! Context lifecycle and operation result handling remain in the context owner.

#[cfg(not(all(windows, feature = "dx12")))]
use super::FfxLibrary;
#[cfg(all(windows, feature = "dx12"))]
use super::LoadError;
#[cfg(all(test, windows, feature = "dx12"))]
use fsr_sdk_sys::api::ffxQueryDescHeader;
use fsr_sdk_sys::api::{
    ffxAllocationCallbacks, ffxContext, ffxCreateContextDescHeader, ffxDispatchDescHeader,
    ffxReturnCode_t,
};
#[cfg(all(windows, feature = "dx12"))]
use fsr_sdk_sys::loader::FfxLibrary;
#[cfg(all(windows, feature = "dx12"))]
use std::path::Path;

pub(crate) struct RuntimeState {
    library: FfxLibrary,
}

impl RuntimeState {
    /// Load the explicit native library and translate its loading failures.
    ///
    /// # Safety
    /// The loader and all resolved dependencies/providers must be trusted and
    /// compatible with the v2.3.0 ABI. Uphold native initialization, termination
    /// and threading preconditions, including cleanup after partial load failure.
    /// Establish the acquiring public runtime family's lasting lifetime and
    /// coordination obligations; movable owners also require arbitrary-thread
    /// use and release.
    #[cfg(all(windows, feature = "dx12"))]
    pub(super) unsafe fn load_unchecked(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        // SAFETY: The caller establishes native acquisition and lasting family
        // obligations. Failed acquisition finishes cleanup on this calling thread.
        let library = unsafe { FfxLibrary::load(path) }.map_err(LoadError::from_sys)?;
        Ok(Self { library })
    }

    // Fixtures may supply their controlled loader and release instrumentation.
    #[cfg(test)]
    pub(crate) fn new(library: FfxLibrary) -> Self {
        Self { library }
    }

    /// # Safety
    /// Supply a writable, null-initialized output, valid address-stable creation
    /// descriptors and compatible optional allocation callbacks. Retain this
    /// runtime and all referenced dependencies for their native lifetimes.
    /// Uphold native threading requirements; callbacks must not unwind over FFI.
    pub(crate) unsafe fn create_context(
        &self,
        context: *mut ffxContext,
        desc: *mut ffxCreateContextDescHeader,
        allocator: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t {
        // SAFETY: The caller establishes the native creation contract; this
        // state retains the library during invocation.
        unsafe { self.library.ffxCreateContext(context, desc, allocator) }
    }

    /// # Safety
    /// Supply a live context from this runtime and allocation callbacks compatible
    /// with creation. Finish dependent GPU work before destruction, uphold native
    /// threading requirements and never reuse a destroyed context. Callbacks must
    /// not unwind over FFI.
    pub(crate) unsafe fn destroy_context(
        &self,
        context: *mut ffxContext,
        allocator: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t {
        // SAFETY: The caller establishes the native destruction contract; this
        // state retains the library during invocation.
        unsafe { self.library.ffxDestroyContext(context, allocator) }
    }

    /// # Safety
    /// Supply a live context from this runtime and valid dispatch descriptors and
    /// resources. Uphold native threading, device, resource-state, submission and
    /// synchronization requirements. Retain this runtime, the context and all
    /// required GPU objects through completion. Callbacks must not unwind over FFI.
    pub(crate) unsafe fn dispatch(
        &self,
        context: *mut ffxContext,
        desc: *const ffxDispatchDescHeader,
    ) -> ffxReturnCode_t {
        // SAFETY: The caller establishes the native dispatch contract; this
        // state retains the library during the CPU call, not subsequent GPU work.
        unsafe { self.library.ffxDispatch(context, desc) }
    }

    /// # Safety
    /// Supply a live context from this runtime (or null for a supported global
    /// query) and a valid writable descriptor chain and output storage. Uphold
    /// query-specific lifetimes and native synchronization/threading requirements.
    /// Callbacks must not unwind over FFI.
    #[cfg(all(test, windows, feature = "dx12"))]
    pub(crate) unsafe fn query_for_test(
        &self,
        context: *mut ffxContext,
        desc: *mut ffxQueryDescHeader,
    ) -> ffxReturnCode_t {
        // SAFETY: The test caller establishes the native query contract; this
        // state retains the library during invocation.
        unsafe { self.library.ffxQuery(context, desc) }
    }
}
