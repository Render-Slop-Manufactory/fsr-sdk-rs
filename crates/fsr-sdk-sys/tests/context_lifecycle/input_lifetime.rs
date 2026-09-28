// SPDX-License-Identifier: MPL-2.0

//! Experiment-only descriptor storage. No claim of a general failed-create contract.
use super::{abi::*, header};
use fsr_sdk_sys::api::ffxApiHeader;
use std::{ffi::c_void, mem::size_of};

struct State {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
    second: ffxCreateBackendDX12Desc,
}

pub struct Pages<'a> {
    state: *mut State,
    protect: libloading::Symbol<'a, unsafe extern "C" fn(*mut c_void, usize) -> i32>,
    free: libloading::Symbol<'a, unsafe extern "C" fn(*mut c_void) -> i32>,
}

impl<'a> Pages<'a> {
    pub fn new(
        helper: &'a libloading::Library,
        root: ffxCreateContextDescUpscale,
        version: ffxCreateContextDescUpscaleVersion,
        backend: ffxCreateBackendDX12Desc,
        second: ffxCreateBackendDX12Desc,
    ) -> Self {
        // SAFETY: Exact cdecl exports paired with device.cpp; helper stays borrowed.
        let (allocate, protect, free) = unsafe {
            (
                helper
                    .get::<unsafe extern "C" fn(usize) -> *mut c_void>(c"probe_allocate_pages")
                    .unwrap(),
                helper.get(c"probe_protect_pages").unwrap(),
                helper.get(c"probe_free_pages").unwrap(),
            )
        };
        // SAFETY: VirtualAlloc returns a dedicated, suitably aligned writable region.
        let state = unsafe { allocate(size_of::<State>()) }.cast::<State>();
        assert!(!state.is_null(), "VirtualAlloc failed");
        // SAFETY: Initialize once, then link at final addresses. Only raw pointers
        // escape; there are no Rust references to the subsequently protected pages.
        unsafe {
            state.write(State {
                root,
                version,
                backend,
                second,
            });
            (*state).root.header.pNext = &raw mut (*state).version.header;
            (*state).version.header.pNext = &raw mut (*state).backend.header;
            (*state).backend.header.pNext = &raw mut (*state).second.header;
            (*state).second.header = header(
                FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12,
                std::ptr::null_mut(),
            );
        }
        eprintln!(
            "Dedicated descriptor storage: base={state:p}; bytes={}",
            size_of::<State>()
        );
        Self {
            state,
            protect,
            free,
        }
    }

    pub fn root(&self) -> *mut ffxApiHeader {
        // SAFETY: Address formation only; state remains allocated at a stable address.
        unsafe { &raw mut (*self.state).root.header }
    }

    pub fn protect(&self) {
        // SAFETY: Sole dedicated allocation; no Rust references/accesses survive.
        // Deliberately tests the unknown native post-return lifetime in a child.
        assert_eq!(
            unsafe { (self.protect)(self.state.cast(), size_of::<State>()) },
            1,
            "PAGE_NOACCESS or VirtualQuery verification failed"
        );
        eprintln!("Descriptor pages: PAGE_NOACCESS verified by VirtualQuery");
    }
}

impl Drop for Pages<'_> {
    fn drop(&mut self) {
        // SAFETY: VirtualFree needs only the allocation address, never reads the
        // protected contents. Fields have no destructors; release exactly once.
        assert_eq!(
            unsafe { (self.free)(self.state.cast()) },
            1,
            "VirtualFree failed"
        );
        eprintln!("Descriptor storage released: VirtualFree succeeded");
    }
}
