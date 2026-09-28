// SPDX-License-Identifier: MPL-2.0

// Experiment diagnostics only: inspect module presence without loading it or
// changing search policy. The single-test process performs no concurrent unload.
use std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetModuleFileNameW(module: *mut c_void, path: *mut u16, size: u32) -> u32;
}

pub fn report(phase: &str) {
    if std::env::var_os("FSR_SDK_RESOLUTION_DIAGNOSTICS").is_none() {
        return;
    }
    let name: Vec<u16> = "amd_fidelityfx_upscaler_dx12.dll\0"
        .encode_utf16()
        .collect();
    // SAFETY: name is terminated and readable. This borrows an existing module
    // handle; no reference is acquired and it must not be freed by this probe.
    let module = unsafe { GetModuleHandleW(name.as_ptr()) };
    if module.is_null() {
        eprintln!("PROVIDER: phase={phase}, loaded=false");
        return;
    }
    let mut path = vec![0_u16; 32768];
    // SAFETY: module is present in this isolated process; path is writable for
    // its declared capacity and no code here unloads modules during this call.
    let len = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
    assert!(
        len > 0 && (len as usize) < path.len(),
        "module path unavailable"
    );
    eprintln!(
        "PROVIDER: phase={phase}, loaded=true, path={}",
        String::from_utf16_lossy(&path[..len as usize])
    );
}
