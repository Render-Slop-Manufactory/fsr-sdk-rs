// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::sizes};
use std::{ffi::CStr, path::PathBuf, ptr::null};

pub(super) fn native_lifecycle_case(drop_b: bool) {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("FSR_SDK_TEST_DLL"));
    let helper_path = PathBuf::from(
        std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("FSR_SDK_TEST_DEVICE_DLL"),
    );
    // SAFETY: Explicit opt-in supplies trusted locally built helper and SDK DLLs.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact helper signature paired with device.cpp, caller owns output.
    let create_device = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .unwrap();
    let mut raw = null_mut();
    // SAFETY: Writable null-initialized output and trusted helper.
    let hr = unsafe { create_device(&mut raw) };
    assert!(hr >= 0 && !raw.is_null(), "device creation: {hr:#x}");
    // SAFETY: Helper transferred exactly one live ID3D12Device COM reference.
    // from_raw adopts it without AddRef; public construction clones it.
    let device = unsafe { ID3D12Device::from_raw(raw.cast()) };
    // SAFETY: Trusted compatible SDK runtime, staged with its provider DLL.
    let mut runtime = unsafe { Runtime::load(&loader) }.unwrap();
    assert!(Dimensions::new(0, 720).is_err());
    assert!(Dimensions::new(1280, 0).is_err());
    eprintln!("Dimensions: render=1280x720 upscale=1920x1080; zero rejected in Rust");
    let context_a = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap_or_else(|error| stop_after_native_error("create A", error));
    let context_b = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap_or_else(|error| stop_after_native_error("create B", error));
    eprintln!("Production owner: create A/B OK");
    let runtime_state = std::rc::Rc::downgrade(&runtime.state);
    drop(runtime);
    drop(device);
    assert!(runtime_state.upgrade().is_some());
    eprintln!(
        "Loader after public owner drop: resident={}",
        libloading::os::windows::Library::open_already_loaded(&loader).is_ok()
    );
    context_a
        .destroy()
        .unwrap_or_else(|error| stop_after_native_error("destroy A", error));
    assert!(runtime_state.upgrade().is_some());
    eprintln!(
        "Loader after A destroy: resident={}",
        libloading::os::windows::Library::open_already_loaded(&loader).is_ok()
    );
    let mut context_b = Box::new(context_b); // exercise outer owner movement
    let mut query = ffxQueryGetProviderVersion {
        header: ffxApiHeader {
            r#type: FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION,
            pNext: null_mut(),
        },
        versionId: 0,
        versionName: null(),
    };
    // SAFETY: Correct canonical ABI, live owner and writable query storage.
    let result = unsafe { context_b.core.owner.query(&raw mut query.header) };
    let valid = result == FFX_API_RETURN_OK && query.versionId != 0 && !query.versionName.is_null();
    if valid {
        // SAFETY: Read the returned name while native context/runtime remain live.
        let name = unsafe { CStr::from_ptr(query.versionName) };
        eprintln!(
            "Provider: id={:#018x}; name={}",
            query.versionId,
            name.to_string_lossy()
        );
    }
    if drop_b {
        drop(context_b);
    } else {
        (*context_b)
            .destroy()
            .unwrap_or_else(|error| stop_after_native_error("destroy B", error));
    }
    assert!(runtime_state.upgrade().is_none());
    eprintln!(
        "Production owner: destroy A/B OK; B via {}",
        if drop_b { "Drop" } else { "explicit destroy" }
    );
    eprintln!(
        "Loader after B cleanup: resident={}",
        libloading::os::windows::Library::open_already_loaded(&loader).is_ok()
    );
    assert!(valid, "provider query failed with {result}");
}

fn stop_after_native_error(operation: &str, error: Error) -> ! {
    // Production policy has already run (including create-error RAII or
    // exceptional retention). Preserve any remaining test-helper ownership on
    // failure rather than introduce an extra unload into this native smoke test.
    eprintln!("Production owner: {operation} failed: {error}");
    std::process::exit(1)
}
