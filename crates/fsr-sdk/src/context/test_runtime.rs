// SPDX-License-Identifier: MPL-2.0

//! Host-test adapter only. Loads the test fixture, never an AMD runtime.
//! Windows/DX12 tests use the production FfxLibrary instead.
#![allow(non_snake_case)]
use fsr_sdk_sys::api::*;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) struct FfxLibrary {
    create: PfnFfxCreateContext,
    destroy: PfnFfxDestroyContext,
    dispatch: PfnFfxDispatch,
    _library: libloading::Library,
    release_probe: Option<Arc<AtomicBool>>,
}

#[path = "../runtime/state.rs"]
mod state;
pub(crate) use state::RuntimeState;

impl Drop for FfxLibrary {
    fn drop(&mut self) {
        if let Some(probe) = &self.release_probe {
            probe.store(true, Ordering::SeqCst);
        }
    }
}

impl FfxLibrary {
    pub(crate) fn observe_release(&mut self, probe: Arc<AtomicBool>) {
        self.release_probe = Some(probe);
    }

    pub(crate) unsafe fn load(path: impl AsRef<Path>) -> Result<Self, libloading::Error> {
        // SAFETY: Test caller supplies its compiled fixture and exact exports.
        unsafe {
            let library = libloading::Library::new(path.as_ref())?;
            let create = *library.get(c"ffxCreateContext")?;
            let destroy = *library.get(c"ffxDestroyContext")?;
            let dispatch = *library.get(c"ffxDispatch")?;
            Ok(Self {
                create,
                destroy,
                dispatch,
                _library: library,
                release_probe: None,
            })
        }
    }
    pub(crate) unsafe fn ffxCreateContext(
        &self,
        context: *mut ffxContext,
        desc: *mut ffxCreateContextDescHeader,
        allocator: *const ffxAllocationCallbacks,
    ) -> u32 {
        // SAFETY: Caller supplies fixture-compatible arguments.
        unsafe { (self.create.unwrap())(context, desc, allocator) }
    }
    pub(crate) unsafe fn ffxDestroyContext(
        &self,
        context: *mut ffxContext,
        allocator: *const ffxAllocationCallbacks,
    ) -> u32 {
        // SAFETY: Caller supplies a live fixture context, exactly once.
        unsafe { (self.destroy.unwrap())(context, allocator) }
    }

    pub(crate) unsafe fn ffxDispatch(
        &self,
        context: *mut ffxContext,
        desc: *const ffxDispatchDescHeader,
    ) -> u32 {
        // SAFETY: Caller supplies the live context and fixture dispatch protocol.
        unsafe { (self.dispatch.unwrap())(context, desc) }
    }
}
