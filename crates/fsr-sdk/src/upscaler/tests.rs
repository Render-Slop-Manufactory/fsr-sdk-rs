// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::{
    ffi::{CStr, c_char},
    path::PathBuf,
    ptr::null,
};

fn sizes() -> ([NonZeroU32; 2], [NonZeroU32; 2]) {
    (
        [
            NonZeroU32::new(1280).unwrap(),
            NonZeroU32::new(720).unwrap(),
        ],
        [
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        ],
    )
}

#[test]
fn descriptor_links_survive_moves_of_outer_storage() {
    let (render, upscale) = sizes();
    // Address sentinel only; no native call and no device dereference.
    let device = std::ptr::dangling_mut::<ID3D12Device>();
    let (state, root) = UpscaleCreateState::new(device, render, upscale);
    let addresses = (
        &raw const state.root,
        &raw const state.version,
        &raw const state.backend,
    );
    let outer = Box::new(Some(state));
    let moved = (*outer).unwrap();
    assert_eq!(
        addresses,
        (
            &raw const moved.root,
            &raw const moved.version,
            &raw const moved.backend
        )
    );
    assert_eq!(root, (&raw const moved.root.header).cast_mut());
    assert_eq!(
        moved.root.header.pNext,
        (&raw const moved.version.header).cast_mut()
    );
    assert_eq!(
        moved.version.header.pNext,
        (&raw const moved.backend.header).cast_mut()
    );
    assert!(moved.backend.header.pNext.is_null());
    assert_eq!(moved.backend.device, device);
    assert_eq!(
        moved.root.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE
    );
    assert_eq!(
        moved.version.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION
    );
    assert_eq!(
        moved.backend.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
    );
    assert_eq!(moved.version.version, FFX_UPSCALER_VERSION);
    assert_eq!(moved.root.flags, 0);
    assert!(moved.root.fpMessage.is_none());
}

// Query declaration solely for test instrumentation. Derived from AMD SDK v2.3.0
// api/include/ffx_api.h. AMD notice is retained in this crate's LICENSE-AMD.
#[repr(C)]
struct ProviderQuery {
    header: ffxQueryDescHeader,
    id: u64,
    name: *const c_char,
}

#[cfg(all(target_arch = "x86_64", target_env = "msvc"))]
const _: () = {
    assert!(std::mem::size_of::<ProviderQuery>() == 32);
    assert!(std::mem::align_of::<ProviderQuery>() == 8);
    assert!(std::mem::offset_of!(ProviderQuery, header) == 0);
    assert!(std::mem::offset_of!(ProviderQuery, id) == 16);
    assert!(std::mem::offset_of!(ProviderQuery, name) == 24);
};

#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tests/run-m4.ps1"]
fn native_lifecycle() {
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
    // from_raw adopts it without AddRef; the owner later releases it once.
    let device = unsafe { OwnedDevice::from_raw(raw.cast()) };
    // SAFETY: Trusted compatible SDK runtime, staged with its provider DLL.
    let library = unsafe { FfxLibrary::load(loader) }.unwrap();
    let (render, upscale) = sizes();
    // SAFETY: Same bounded setup as the recorded successful lifecycle: serial
    // calls, live hardware device, supported sizes, no callbacks or GPU dispatch.
    let context = unsafe { Upscaler::create(library, device, render, upscale) }
        .unwrap_or_else(|error| stop_after_native_error("create", error));
    eprintln!("Production owner: create OK");
    let mut context = Box::new(context); // exercise outer owner movement
    let mut query = ProviderQuery {
        header: ffxApiHeader {
            r#type: 6,
            pNext: null_mut(),
        },
        id: 0,
        name: null(),
    };
    // SAFETY: Correct test-only ABI, live owner and writable query storage.
    let result = unsafe { context.owner.query_for_test(&raw mut query.header) };
    let valid = result == FFX_API_RETURN_OK && query.id != 0 && !query.name.is_null();
    if valid {
        // SAFETY: Read the returned name while native context/runtime remain live.
        let name = unsafe { CStr::from_ptr(query.name) };
        eprintln!(
            "Provider: id={:#018x}; name={}",
            query.id,
            name.to_string_lossy()
        );
    }
    (*context)
        .destroy()
        .unwrap_or_else(|error| stop_after_native_error("destroy", error));
    eprintln!("Production owner: destroy OK");
    assert!(valid, "provider query failed with {result}");
}

macro_rules! assert_not_impl {
    ($bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Ambiguous<IfImplemented> for T {}
            let _ = <Upscaler as Ambiguous<_>>::check;
        };
    };
}
assert_not_impl!(Send);
assert_not_impl!(Sync);
assert_not_impl!(Copy);
assert_not_impl!(Clone);

fn stop_after_native_error(operation: &str, error: Error) -> ! {
    // Production policy has already run (including create-error RAII or
    // exceptional retention). Preserve any remaining test-helper ownership on
    // failure rather than introduce an extra unload into this native smoke test.
    eprintln!("Production owner: {operation} failed: {error}");
    std::process::exit(1)
}
