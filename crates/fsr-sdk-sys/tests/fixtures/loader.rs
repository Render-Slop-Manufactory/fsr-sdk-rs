// SPDX-License-Identifier: MPL-2.0

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
// Test-only exports. They inspect pointer values but never dereference them.
// No AMD code or native contexts are involved.
#[path = "../../src/api.rs"]
mod api;
use api::*;

#[cfg(not(missing_ffxCreateContext))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxCreateContext(context: *mut ffxContext, desc: *mut ffxCreateContextDescHeader, memCb: *const ffxAllocationCallbacks) -> ffxReturnCode_t {
    if context as usize == 0x10 && desc as usize == 0x20 && memCb as usize == 0x30 { 101 } else { 0 }
}

#[cfg(not(missing_ffxDestroyContext))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDestroyContext(context: *mut ffxContext, memCb: *const ffxAllocationCallbacks) -> ffxReturnCode_t {
    if context as usize == 0x10 && memCb as usize == 0x30 { 102 } else { 0 }
}

#[cfg(not(missing_ffxConfigure))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxConfigure(context: *mut ffxContext, desc: *const ffxConfigureDescHeader) -> ffxReturnCode_t {
    if context as usize == 0x10 && desc as usize == 0x20 { 103 } else { 0 }
}

#[cfg(not(missing_ffxQuery))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxQuery(context: *mut ffxContext, desc: *mut ffxQueryDescHeader) -> ffxReturnCode_t {
    if context as usize == 0x10 && desc as usize == 0x20 { 104 } else { 0 }
}

#[cfg(not(missing_ffxDispatch))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDispatch(context: *mut ffxContext, desc: *const ffxDispatchDescHeader) -> ffxReturnCode_t {
    if context as usize == 0x10 && desc as usize == 0x20 { 105 } else { 0 }
}
