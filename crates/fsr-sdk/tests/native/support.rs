// SPDX-License-Identifier: MPL-2.0

use super::*;

#[repr(C)]
#[derive(Default, Debug)]
pub(super) struct Metrics {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) finite_pixels: u32,
    pub(super) changed_pixels: u32,
    pub(super) min_red: f32,
    pub(super) max_red: f32,
    pub(super) left_red: f32,
    pub(super) right_red: f32,
    pub(super) debug_errors: u64,
    pub(super) device_removed: i32,
}

// Test-only provider-identification query from SDK v2.3.0 ffx_api.h, like M5.
#[repr(C)]
pub(super) struct ProviderQuery {
    pub(super) header: ffxApiHeader,
    pub(super) id: u64,
    pub(super) name: *const c_char,
}

pub(super) type EnableDebug = unsafe extern "C" fn() -> i32;
pub(super) type CreateDevice = unsafe extern "C" fn(*mut *mut c_void) -> i32;
pub(super) type DeviceCheck = unsafe extern "C" fn(*mut c_void) -> i32;
pub(super) type SetTemporalFrame = unsafe extern "C" fn(*mut c_void, u32, f32, f32) -> i32;
pub(super) type CreateHarness =
    unsafe extern "C" fn(*mut c_void, *mut *mut c_void, *mut *mut c_void, *mut *mut c_void) -> i32;
pub(super) type CompareResource =
    unsafe extern "C" fn(*mut c_void, *const FfxApiResource, u32) -> i32;
pub(super) type Finish = unsafe extern "C" fn(*mut c_void, *mut Metrics) -> i32;
pub(super) type Destroy = unsafe extern "C" fn(*mut c_void);

pub(super) fn log_provider(upscaler: &mut Upscaler) {
    log_provider_owner(&mut upscaler.core.owner);
}

pub(super) fn log_provider_owner<S, D, R: crate::context::RuntimeRetention>(
    owner: &mut crate::context::NativeContextOwner<S, D, R>,
) {
    let mut provider = ProviderQuery {
        header: ffxApiHeader {
            r#type: 6,
            pNext: null_mut(),
        },
        id: 0,
        name: std::ptr::null(),
    };
    // SAFETY: This test-only query ABI matches the existing M5 fixture;
    // the context and runtime remain live while reading the provider name.
    let query_code = unsafe { owner.query_for_test(&raw mut provider.header) };
    eprintln!("Provider query: code={query_code:#010x}");
    assert_eq!(query_code, FFX_API_RETURN_OK, "provider query failed");
    assert!(!provider.name.is_null() && provider.id != 0);
    // SAFETY: A successful query returned a live native name pointer.
    let name = unsafe { CStr::from_ptr(provider.name) };
    eprintln!(
        "M6 provider: id={:#018x}; name={}",
        provider.id,
        name.to_string_lossy()
    );
}

pub(super) fn abort_dispatch(error: Error, frame: u32) -> ! {
    match error {
        Error::Native { operation, code } => eprintln!(
            "NATIVE DISPATCH FAILURE: frame={frame} operation={operation:?} code={code:#010x} ({code}); context poisoned"
        ),
        other => eprintln!("HARNESS / INPUT FAILURE: frame={frame} error={other:?}"),
    }
    // Failed native recordings must never be submitted. Terminating this
    // isolated child also preserves the existing fail-fast cleanup strategy.
    std::process::exit(1);
}
