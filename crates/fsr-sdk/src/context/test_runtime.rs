// SPDX-License-Identifier: MPL-2.0

//! Host-test adapter only. Loads the test fixture, never an AMD runtime.
//! Windows/DX12 tests use the production FfxLibrary instead.
#![allow(non_snake_case)]
use fsr_sdk_sys::api::*;
use std::{cell::Cell, path::Path, rc::Rc};

pub(crate) struct FfxLibrary {
    create: PfnFfxCreateContext,
    destroy: PfnFfxDestroyContext,
    _library: libloading::Library,
    release_probe: Option<Rc<Cell<bool>>>,
}

pub(crate) struct RuntimeState {
    pub(crate) library: FfxLibrary,
}

impl Drop for FfxLibrary {
    fn drop(&mut self) {
        if let Some(probe) = &self.release_probe {
            probe.set(true);
        }
    }
}

impl FfxLibrary {
    pub(crate) fn observe_release(&mut self, probe: Rc<Cell<bool>>) {
        self.release_probe = Some(probe);
    }

    pub(crate) unsafe fn load(path: impl AsRef<Path>) -> Result<Self, libloading::Error> {
        // SAFETY: Test caller supplies its compiled fixture and exact exports.
        unsafe {
            let library = libloading::Library::new(path.as_ref())?;
            let create = *library.get(c"ffxCreateContext")?;
            let destroy = *library.get(c"ffxDestroyContext")?;
            Ok(Self {
                create,
                destroy,
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
}
