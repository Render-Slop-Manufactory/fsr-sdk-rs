// SPDX-License-Identifier: MPL-2.0

//! One-process, sequential signed-runtime lifetime experiment.
#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc", feature = "dx12"))]

#[path = "context_lifecycle/abi.rs"]
mod abi;

use abi::*;
use fsr_sdk_sys::{api::*, loader::FfxLibrary, upscale::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE};
use std::{
    ffi::{CStr, c_void},
    path::PathBuf,
    ptr::{null, null_mut},
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetModuleFileNameW(module: *mut c_void, path: *mut u16, size: u32) -> u32;
}

struct Modules {
    expected: PathBuf,
    first: Option<*mut c_void>,
}

impl Modules {
    fn observe(&mut self, phase: &str) {
        let name: Vec<u16> = "amd_fidelityfx_upscaler_dx12.dll\0"
            .encode_utf16()
            .collect();
        // SAFETY: The string is terminated. GetModuleHandleW does not acquire a
        // reference; this isolated probe makes no concurrent module changes.
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        if module.is_null() {
            eprintln!("module phase={phase} loaded=false");
            return;
        }
        let mut path = vec![0_u16; 32768];
        // SAFETY: The module is currently resident, no other probe thread can
        // unload it, and the output buffer is writable for the stated length.
        let len = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
        if len == 0 || len as usize >= path.len() {
            eprintln!("module phase={phase} loaded=true path=unavailable");
            return;
        }
        let actual = PathBuf::from(String::from_utf16_lossy(&path[..len as usize]));
        let path_matches = actual.canonicalize().ok().as_deref() == Some(self.expected.as_path());
        let identity = match self.first {
            None => {
                self.first = Some(module);
                "first"
            }
            Some(first) if first == module => "same",
            Some(_) => "changed",
        };
        // Avoid retaining personal absolute paths in the child log. Equality
        // against the canonical staged file still detects an unexpected module.
        let path_label = if path_matches {
            "<stage>/amd_fidelityfx_upscaler_dx12.dll".to_owned()
        } else {
            format!(
                "<unexpected>/{}",
                actual.file_name().unwrap_or_default().to_string_lossy()
            )
        };
        eprintln!(
            "module phase={phase} loaded=true identity={identity} path={path_label} path_matches_stage={path_matches}"
        );
    }
}

fn header(tag: u64, next: *mut ffxApiHeader) -> ffxApiHeader {
    ffxApiHeader {
        r#type: tag,
        pNext: next,
    }
}

struct CreateState {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
}

impl CreateState {
    fn new(device: *mut ID3D12Device) -> Box<Self> {
        let mut state = Box::new(Self {
            root: ffxCreateContextDescUpscale {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE, null_mut()),
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
            },
            version: ffxCreateContextDescUpscaleVersion {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION, null_mut()),
                version: FFX_UPSCALER_VERSION,
            },
            backend: ffxCreateBackendDX12Desc {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, null_mut()),
                device,
            },
        });
        state.root.header.pNext = &raw mut state.version.header;
        state.version.header.pNext = &raw mut state.backend.header;
        state
    }
}

fn query(library: &FfxLibrary, context: &mut ffxContext, phase: &str) -> Option<(u64, String)> {
    let mut provider = ffxQueryGetProviderVersion {
        header: header(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION, null_mut()),
        versionId: 0,
        versionName: null(),
    };
    // SAFETY: This is the paired test-only ABI. The context is still live and
    // query output storage remains valid through the call.
    let result = unsafe { library.ffxQuery(context, &raw mut provider.header) };
    let name = if result == FFX_API_RETURN_OK && !provider.versionName.is_null() {
        // SAFETY: Copy the returned string while the context/provider is live.
        unsafe { CStr::from_ptr(provider.versionName) }
            .to_string_lossy()
            .into_owned()
    } else {
        "unavailable".to_owned()
    };
    eprintln!(
        "query phase={phase} return_code={result} provider_id=0x{:016x} provider_name={name}",
        provider.versionId
    );
    (result == FFX_API_RETURN_OK && provider.versionId != 0 && name != "unavailable")
        .then_some((provider.versionId, name))
}

enum CreateOutcome {
    Created,
    Failed,
    OkNull,
}

fn create(
    library: &FfxLibrary,
    state: &mut CreateState,
    context: &mut ffxContext,
    name: &str,
) -> CreateOutcome {
    // SAFETY: Separate stable chains, valid hardware device, writable null
    // output, and no overlapping native calls. The caller retains all inputs.
    let result = unsafe { library.ffxCreateContext(context, &raw mut state.root.header, null()) };
    eprintln!(
        "create context={name} return_code={result} handle={}",
        if context.is_null() {
            "null"
        } else {
            "non-null"
        }
    );
    if result == FFX_API_RETURN_OK && context.is_null() {
        eprintln!("terminal=OK-with-null context={name}; retaining dependencies");
        return CreateOutcome::OkNull;
    }
    if result == FFX_API_RETURN_OK {
        CreateOutcome::Created
    } else {
        CreateOutcome::Failed
    }
}

fn destroy(library: &FfxLibrary, context: &mut ffxContext, name: &str) {
    // SAFETY: Called once for a successful non-null create. No dispatch was
    // submitted and all dependencies remain live. Handle bits after return are
    // ignored; a failed destroy terminates without releasing dependencies.
    let result = unsafe { library.ffxDestroyContext(context, null()) };
    eprintln!("destroy context={name} return_code={result}");
    if result != FFX_API_RETURN_OK {
        eprintln!("terminal=destroy-failed context={name}; retaining dependencies");
        std::process::exit(2);
    }
}

#[test]
#[ignore = "requires trusted signed DLLs, hardware DX12 and the isolated runner"]
fn sequential_shared_runtime_lifetime() {
    let helper_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("device helper path"));
    let loader_path = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("loader path"));
    let provider_path =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_PROVIDER_DLL").expect("provider path"));
    let mut modules = Modules {
        expected: provider_path
            .canonicalize()
            .expect("canonical provider path"),
        first: None,
    };
    modules.observe("before_load");

    // SAFETY: The runner builds this helper and stages the verified SDK DLLs.
    let helper = unsafe { libloading::Library::new(&helper_path) }.expect("device helper");
    // SAFETY: Signatures match the paired C++ helper exports.
    let create_device = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .expect("create device export");
    // SAFETY: This releases exactly the reference transferred by the helper.
    let release_device =
        unsafe { helper.get::<unsafe extern "C" fn(*mut ID3D12Device)>(c"probe_release_device") }
            .expect("release device export");
    // SAFETY: Explicit trusted v2.3.0 loader with executable-adjacent provider.
    let library = unsafe { FfxLibrary::load(&loader_path) }.expect("AMD loader");
    eprintln!("loader acquired=true");
    let mut device = null_mut();
    // SAFETY: Writable output and the paired helper ABI.
    let hr = unsafe { create_device(&mut device) };
    eprintln!(
        "device HRESULT=0x{hr:08x} handle={}",
        if device.is_null() { "null" } else { "non-null" }
    );
    if hr < 0 || device.is_null() {
        eprintln!("terminal=device-failed");
        std::process::exit(2);
    }

    let mut a_state = CreateState::new(device);
    let mut b_state = CreateState::new(device);
    let mut a: ffxContext = null_mut();
    let mut b: ffxContext = null_mut();
    eprintln!(
        "inputs A=B render=1280x720 upscale=1920x1080 flags=0 callback=null allocator=null chain=upscale->version->DX12->null"
    );
    match create(&library, &mut a_state, &mut a, "A") {
        CreateOutcome::Created => {}
        CreateOutcome::Failed => {
            modules.observe("after_A_failed_create");
            // D005's returned-error trust assumption permits caller cleanup.
            unsafe { release_device(device) };
            drop(library);
            modules.observe("after_loader_drop");
            return;
        }
        CreateOutcome::OkNull => std::process::exit(2),
    }
    modules.observe("after_A_create");
    match create(&library, &mut b_state, &mut b, "B") {
        CreateOutcome::Created => {}
        CreateOutcome::Failed => {
            modules.observe("after_B_failed_create");
            destroy(&library, &mut a, "A");
            modules.observe("after_A_destroy");
            unsafe { release_device(device) };
            drop(library);
            modules.observe("after_loader_drop");
            return;
        }
        CreateOutcome::OkNull => {
            // A has a valid owner and still receives its single teardown. B's
            // unknown native state keeps the other dependencies/library live.
            destroy(&library, &mut a, "A");
            modules.observe("after_A_destroy_B_OK_null");
            std::process::exit(2);
        }
    }
    modules.observe("after_B_create");
    let _a_before = query(&library, &mut a, "A_both_live");
    let b_before = query(&library, &mut b, "B_both_live");
    destroy(&library, &mut a, "A");
    modules.observe("after_A_destroy_B_live");
    let b_after = query(&library, &mut b, "B_after_A_destroy");
    eprintln!(
        "B_identity_equal={}",
        b_before.is_some() && b_before == b_after
    );
    destroy(&library, &mut b, "B");
    modules.observe("after_B_destroy");
    // SAFETY: The caller's one device reference is released only after both
    // contexts are destroyed successfully.
    unsafe { release_device(device) };
    drop(library);
    modules.observe("after_loader_drop");
}
