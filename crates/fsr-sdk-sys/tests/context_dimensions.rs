// SPDX-License-Identifier: MPL-2.0

//! Opt-in, one-case-per-process signed-runtime dimension experiment.
#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc", feature = "dx12"))]

#[path = "context_lifecycle/abi.rs"]
mod abi;

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

fn dimensions(case: &str) -> ([u32; 2], [u32; 2]) {
    match case {
        "C0" => ([1280, 720], [1920, 1080]),
        "Z1" => ([0, 720], [1920, 1080]),
        "Z2" => ([1280, 0], [1920, 1080]),
        "Z3" => ([1280, 720], [0, 1080]),
        "Z4" => ([1280, 720], [1920, 0]),
        "T1" => ([1, 1], [2, 2]),
        "N1" => ([1279, 719], [1919, 1079]),
        "E1" => ([1280, 720], [1280, 720]),
        "D1" => ([1920, 1080], [1280, 720]),
        "B1" => ([16383, 720], [16383, 720]),
        "B2" => ([16384, 720], [16384, 720]),
        "B3" => ([16385, 720], [16385, 720]),
        _ => panic!("unknown FSR_SDK_DIMENSION_CASE"),
    }
}

#[test]
#[ignore = "requires the explicit signed-runtime runner and one fresh process per case"]
fn signed_runtime_dimensions() {
    let case = std::env::var("FSR_SDK_DIMENSION_CASE").expect("FSR_SDK_DIMENSION_CASE");
    let (render, upscale) = dimensions(&case);
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("device helper path"));
    let loader_path = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("loader path"));

    // SAFETY: The runner builds this helper from our paired C++ source and stages
    // the explicitly selected trusted SDK runtime beside this executable.
    let helper = unsafe { libloading::Library::new(helper_path) }.expect("device helper");
    // SAFETY: Signatures are defined by the paired device.cpp exports.
    let create_device = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .expect("create device export");
    // SAFETY: The helper transfers one COM reference, released once below.
    let release_device =
        unsafe { helper.get::<unsafe extern "C" fn(*mut ID3D12Device)>(c"probe_release_device") }
            .expect("release device export");
    // SAFETY: Explicitly supplied, trusted, ABI-compatible loader and provider.
    let library = unsafe { FfxLibrary::load(loader_path) }.expect("AMD loader");
    let mut device = null_mut();
    // SAFETY: Writable output and paired helper ABI.
    let hr = unsafe { create_device(&mut device) };
    assert!(
        hr >= 0 && !device.is_null(),
        "device creation failed: {hr:#x}"
    );

    let mut backend = ffxCreateBackendDX12Desc {
        header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, null_mut()),
        device,
    };
    let mut version = ffxCreateContextDescUpscaleVersion {
        header: header(
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION,
            &raw mut backend.header,
        ),
        version: FFX_UPSCALER_VERSION,
    };
    let mut root = ffxCreateContextDescUpscale {
        header: header(
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
            &raw mut version.header,
        ),
        flags: 0,
        maxRenderSize: FfxApiDimensions2D {
            width: render[0],
            height: render[1],
        },
        maxUpscaleSize: FfxApiDimensions2D {
            width: upscale[0],
            height: upscale[1],
        },
        fpMessage: None,
    };
    let mut context: ffxContext = null_mut();
    eprintln!(
        "case={case} render={}x{} upscale={}x{} chain=upscale->version->DX12 flags=0 callback=null allocator=null",
        render[0], render[1], upscale[0], upscale[1]
    );
    eprintln!("create_entered=true");
    // SAFETY: All descriptor types, links and storage are valid and live. The
    // real device and library remain live; only numerical dimensions vary.
    let result = unsafe { library.ffxCreateContext(&mut context, &raw mut root.header, null()) };
    eprintln!(
        "create_returned=true return_code={result} handle={}",
        if context.is_null() {
            "null"
        } else {
            "non-null-1"
        }
    );
    if result != FFX_API_RETURN_OK {
        eprintln!("destroy_attempted=false reason=returned-create-error");
        // D005's accepted returned-error assumption applies to this runtime.
        // SAFETY: Release exactly the caller-owned helper reference.
        unsafe { release_device(device) };
        return;
    }
    if context.is_null() {
        eprintln!("invariant_violation=OK-with-null destroy_attempted=false");
        // Match D005's exceptional retention fallback without a speculative
        // destroy or release. Process isolation then terminates the probe.
        std::process::exit(0);
    }

    let mut provider = ffxQueryGetProviderVersion {
        header: header(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION, null_mut()),
        versionId: 0,
        versionName: null(),
    };
    // SAFETY: Paired test-only query ABI and live native context.
    let query = unsafe { library.ffxQuery(&mut context, &raw mut provider.header) };
    if query == FFX_API_RETURN_OK && !provider.versionName.is_null() {
        // SAFETY: Read only while the native context and runtime are live.
        let name = unsafe { CStr::from_ptr(provider.versionName) };
        eprintln!(
            "provider_query={query} provider_id=0x{:016x} provider_name={}",
            provider.versionId,
            name.to_string_lossy()
        );
    } else {
        eprintln!(
            "provider_query={query} provider_id=0x{:016x} provider_name=unavailable",
            provider.versionId
        );
    }
    eprintln!("destroy_entered=true");
    // SAFETY: One successful non-null context, no dispatch or outstanding work;
    // the same default allocator and all dependencies remain live.
    let destroy = unsafe { library.ffxDestroyContext(&mut context, null()) };
    eprintln!("destroy_returned=true return_code={destroy}");
    if destroy != FFX_API_RETURN_OK {
        // D005's terminal failure policy: retain dependencies; never retry.
        std::process::exit(0);
    }
    // SAFETY: The single caller-owned reference outlived successful destroy.
    unsafe { release_device(device) };
}
