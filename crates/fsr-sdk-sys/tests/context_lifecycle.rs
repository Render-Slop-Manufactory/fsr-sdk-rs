// SPDX-License-Identifier: MPL-2.0

//! Run only via context_lifecycle/run.ps1 in a dedicated test process.
#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc", feature = "dx12"))]

#[path = "context_lifecycle/abi.rs"]
mod abi;
#[path = "context_lifecycle/allocator.rs"]
mod allocator;
#[path = "context_lifecycle/input_lifetime.rs"]
mod input_lifetime;

use abi::*;
use fsr_sdk_sys::{api::*, loader::FfxLibrary, upscale::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE};
use std::{
    ffi::CStr,
    path::PathBuf,
    ptr::{null, null_mut},
};

fn header(tag: u64, next: *mut ffxApiHeader) -> ffxApiHeader {
    ffxApiHeader {
        r#type: tag,
        pNext: next,
    }
}

fn stop_without_cleanup(message: &str) -> ! {
    eprintln!("{message}; terminating the isolated probe without speculative native cleanup");
    // Failed create/destroy have no established cleanup postcondition. Do not
    // unwind descriptor storage, unload code, release the device or retry destroy.
    std::process::exit(1)
}

#[test]
#[ignore = "isolated allocation-failure experiment; use run.ps1 -FailureInjection"]
fn probes_create_allocation_failure() {
    let fail_at = std::env::var("FSR_SDK_FAIL_AT")
        .expect("set FSR_SDK_FAIL_AT (0 disables injection)")
        .parse::<usize>()
        .expect("allocation index");
    run_lifecycle(Some(fail_at));
}

#[test]
#[ignore = "isolated returned-error experiment; use run.ps1 -InputLifetime"]
fn probes_create_error_input_lifetime() {
    let mode = std::env::var("FSR_SDK_INPUT_LIFETIME")
        .expect("use run.ps1 -InputLifetime to select an isolated lifetime variant");
    assert!(matches!(
        mode.as_str(),
        "retain" | "protect" | "cleanup" | "discover"
    ));
    run_lifecycle(None);
}

fn run_lifecycle(fail_at: Option<usize>) {
    let lifetime = std::env::var("FSR_SDK_INPUT_LIFETIME").ok();
    let tracker = allocator::Tracker::new(fail_at.unwrap_or(0));
    let callbacks = tracker.callbacks();
    let mem_cb = if fail_at.is_some() {
        &callbacks
    } else {
        null()
    };
    let loader_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("set FSR_SDK_TEST_DLL"));
    let helper_path = PathBuf::from(
        std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("set FSR_SDK_TEST_DEVICE_DLL"),
    );
    // SAFETY: Opt-in runner supplies our locally compiled helper and the trusted
    // compatible SDK DLLs. Both owners remain live through device/context teardown.
    let helper = unsafe { libloading::Library::new(helper_path) }.expect("device helper");
    // SAFETY: Signatures match the paired C++ helper exports (cdecl, raw COM pointer).
    let create_device = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .expect("create device export");
    // SAFETY: Same helper ABI; release is called exactly once after successful destroy.
    let release_device =
        unsafe { helper.get::<unsafe extern "C" fn(*mut ID3D12Device)>(c"probe_release_device") }
            .expect("release device export");
    // SAFETY: Explicit trusted v2.3.0 runtime, staged with its upscaler provider DLL.
    let library = unsafe { FfxLibrary::load(loader_path) }.expect("AMD loader");
    eprintln!("Loader acquisition: succeeded");
    let mut device = null_mut();
    // SAFETY: Writable null-initialized output; helper uses native Windows headers.
    let device_result = unsafe { create_device(&mut device) };
    eprintln!("D3D12CreateDevice: HRESULT=0x{device_result:08x}; device={device:p}");
    if device_result < 0 || device.is_null() {
        stop_without_cleanup("Device creation failed");
    }

    // Construct tail first. The ordinary control keeps these nodes in place;
    // lifetime variants move them to dedicated pages and rebuild every link
    // before native creation. Both retain the device throughout the create call.
    let mut second_backend = ffxCreateBackendDX12Desc {
        header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, null_mut()),
        device,
    };
    let mut backend = ffxCreateBackendDX12Desc {
        header: header(
            FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12,
            if lifetime.is_some() {
                &raw mut second_backend.header
            } else {
                null_mut()
            },
        ),
        device,
    };
    let mut version = ffxCreateContextDescUpscaleVersion {
        header: header(
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION,
            &raw mut backend.header,
        ),
        version: FFX_UPSCALER_VERSION,
    };
    let mut desc = ffxCreateContextDescUpscale {
        header: header(
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
            &raw mut version.header,
        ),
        flags: 0,
        maxRenderSize: FfxApiDimensions2D {
            width: 1280,
            height: 720,
        },
        maxUpscaleSize: FfxApiDimensions2D {
            width: 1920,
            height: 1080,
        },
        fpMessage: None,
    };
    let mut context: ffxContext = null_mut();
    let tail = if lifetime.is_some() {
        "DX12 -> DX12 -> null"
    } else {
        "DX12 -> null"
    };
    eprintln!(
        "Create chain: upscale -> upscale version (0x{FFX_UPSCALER_VERSION:08x}) -> {tail}; flags=0; render=1280x720; upscale=1920x1080"
    );
    eprintln!("Before create: context={context:p}");
    if lifetime.is_some() {
        eprintln!(
            "Candidate: upscale -> version -> DX12 -> DX12 -> null; same live device in both backends; null callbacks"
        );
    }
    let stack_root = &raw mut desc.header;
    let pages = if lifetime.is_some() {
        Some(input_lifetime::Pages::new(
            &helper,
            desc,
            version,
            backend,
            second_backend,
        ))
    } else {
        None
    };
    // The ordinary control keeps its original stack storage. Lifetime variants
    // use only the dedicated allocation, with links rebuilt at final addresses.
    let root = if let Some(pages) = &pages {
        pages.root()
    } else {
        stack_root
    };
    // SAFETY: Correct tags/layouts, live device and stable complete chain, valid
    // dimensions, writable null output. Optional host callbacks and their state
    // remain live throughout; null allocation results are permitted by ffx_api.h.
    // The experiment's duplicate backend is explicitly rejected by the local
    // v2.3.0 CreateBackend check, not a descriptor tag/payload mismatch.
    let create_result = unsafe { library.ffxCreateContext(&mut context, root, mem_cb) };
    eprintln!("Create: return_code={create_result}; after create: context={context:p}");
    if let Some(mode) = &lifetime
        && create_result != FFX_API_RETURN_OK
    {
        eprintln!(
            "Returned failure: mode={mode}; no destroy; host/message/backend callbacks=null; provider identity unavailable without a live context"
        );
        match mode.as_str() {
            "retain" => eprintln!("Dependencies retained"),
            "protect" => pages.as_ref().unwrap().protect(),
            "cleanup" => {
                drop(pages);
                eprintln!("Releasing caller device reference");
                // SAFETY: Releases exactly the helper-created reference. Native
                // continued use after returned failure is the isolated hypothesis.
                unsafe { release_device(device) };
                eprintln!("Caller device reference released; dropping FfxLibrary");
                drop(library);
                eprintln!("FfxLibrary dropped");
            }
            "discover" => std::process::exit(0),
            _ => stop_without_cleanup("Unknown lifetime mode"),
        }
        let start = std::time::Instant::now();
        eprintln!("Observation begins: requested_ms=2000");
        std::thread::sleep(std::time::Duration::from_millis(2000));
        eprintln!(
            "Observation completed: elapsed_ms={}; normal process exit; no destroy",
            start.elapsed().as_millis()
        );
        std::process::exit(0);
    }
    if fail_at.is_some() {
        tracker.report("after_create");
        if create_result != FFX_API_RETURN_OK {
            // No destroy, manual frees, device release or Rust owner teardown:
            // failed-create cleanup is the subject, not an assumed contract.
            std::process::exit(0);
        }
    }
    if create_result != FFX_API_RETURN_OK || context.is_null() {
        stop_without_cleanup("Context creation failed");
    }

    let mut provider = ffxQueryGetProviderVersion {
        header: header(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION, null_mut()),
        versionId: 0,
        versionName: null(),
    };
    // SAFETY: Successful live context, correctly tagged writable query descriptor.
    let query_result = unsafe { library.ffxQuery(&mut context, &raw mut provider.header) };
    eprintln!(
        "Provider: return_code={query_result}; id=0x{:016x}",
        provider.versionId
    );
    let provider_valid = query_result == FFX_API_RETURN_OK
        && provider.versionId != 0
        && !provider.versionName.is_null();
    if provider_valid {
        // SAFETY: Successful SDK query returns a native display string. Read it
        // immediately while the context/provider and library are still alive.
        let name = unsafe { CStr::from_ptr(provider.versionName) };
        eprintln!("Provider name: {}", name.to_string_lossy());
    }

    eprintln!("Before destroy: context={context:p}");
    // SAFETY: Exactly one destroy of the successfully created context, compatible
    // allocator, live device/chain/library. No dispatch or outstanding GPU work
    // submitted by the probe. Never dereference or reuse the resulting handle.
    let destroy_result = unsafe { library.ffxDestroyContext(&mut context, mem_cb) };
    eprintln!("Destroy: return_code={destroy_result}; after destroy: context={context:p}");
    if fail_at.is_some() {
        tracker.report("after_destroy");
    }
    if destroy_result != FFX_API_RETURN_OK {
        stop_without_cleanup("Context destruction failed");
    }
    // SAFETY: Our device reference is retained through successful context teardown.
    // This releases only the helper-created reference, with no AddRef experiment.
    unsafe { release_device(device) };
    // The raw handle's numeric post-state is evidence, not a live/dead-state test.
    assert!(
        provider_valid,
        "lifecycle completed, but provider identity query failed"
    );
}
