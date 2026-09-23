// SPDX-License-Identifier: MPL-2.0

#![cfg(all(windows, feature = "dx12"))]

use fsr_sdk_sys::{api::*, loader::FfxLibrary, upscale::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE};
use std::{path::PathBuf, ptr::null_mut};

#[path = "provider_resolution/modules.rs"]
mod modules;

#[test]
#[ignore = "requires the trusted local SDK v2.3.0 loader and upscaler DLLs"]
fn queries_upscaler_version_count_without_context() {
    let path = std::env::var_os("FSR_SDK_TEST_DLL")
        .expect("set FSR_SDK_TEST_DLL to the trusted AMD loader DLL");
    modules::report("before_load");
    // SAFETY: Explicit opt-in supplies the trusted SDK loader and its effect
    // dependencies, with compatible ABI and safe initialization/termination.
    let library = unsafe { FfxLibrary::load(PathBuf::from(path)) }.expect("AMD loader");
    eprintln!("LOADER: acquired=true");
    modules::report("after_load");
    let mut count = 0_u64;
    let mut query = ffxQueryDescGetVersions {
        header: ffxApiHeader {
            r#type: FFX_API_QUERY_DESC_TYPE_GET_VERSIONS,
            pNext: null_mut(),
        },
        createDescType: FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
        device: null_mut(),
        outputCount: &mut count,
        versionIds: null_mut(),
        versionNames: null_mut(),
    };
    // SAFETY: This global query accepts a null context. Its tagged repr(C)
    // descriptor and writable count remain live through the synchronous call.
    // Both optional arrays are null, selecting count-only mode. The local SDK
    // supports enumeration with a null device; no device-specific support claim
    // follows. The library remains owned throughout the call.
    let result = unsafe { library.ffxQuery(null_mut(), &mut query.header) };
    eprintln!("GET_VERSIONS: return_code={result}, upscaler_version_count={count}");
    modules::report("after_query");
    assert_eq!(result, FFX_API_RETURN_OK, "global version query failed");
    assert!(
        count > 0,
        "no upscaler providers found; check the SDK effect DLLs"
    );
}
