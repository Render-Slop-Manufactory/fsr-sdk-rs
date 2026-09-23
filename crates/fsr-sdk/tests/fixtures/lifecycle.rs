// SPDX-License-Identifier: MPL-2.0

//! Test protocol only, not AMD ABI descriptors. Built as a standalone cdylib.
use std::ffi::c_void;

#[repr(C)]
pub struct Input {
    pub tag: u64,
    pub next: *mut c_void,
    pub create_code: u32,
    pub destroy_code: u32,
    pub null_create: u32,
    pub clear_destroy: u32,
    pub event: extern "C" fn(*mut c_void, u32),
    pub data: *mut c_void,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxCreateContext(
    out: *mut *mut c_void,
    desc: *mut Input,
    allocator: *const c_void,
) -> u32 {
    // SAFETY: Fixture contract supplies valid input/output through the whole call.
    unsafe {
        if !allocator.is_null() || !(*out).is_null() {
            return 0xf0000001;
        }
        ((*desc).event)((*desc).data, 1);
        *out = if (*desc).null_create != 0 {
            std::ptr::null_mut()
        } else {
            desc.cast()
        };
        (*desc).create_code
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDestroyContext(out: *mut *mut c_void, allocator: *const c_void) -> u32 {
    // SAFETY: Only a successfully returned fixture handle is accepted here.
    unsafe {
        let desc = (*out).cast::<Input>();
        if !allocator.is_null() {
            return 0xf0000002;
        }
        ((*desc).event)((*desc).data, 2);
        if (*desc).clear_destroy != 0 {
            *out = std::ptr::null_mut();
        }
        (*desc).destroy_code
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxQuery(_: *mut *mut c_void, _: *mut c_void) -> u32 {
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxConfigure(_: *mut *mut c_void, _: *const c_void) -> u32 {
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDispatch(_: *mut *mut c_void, _: *const c_void) -> u32 {
    0
}
