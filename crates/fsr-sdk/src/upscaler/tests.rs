// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::runtime::Runtime;
use std::{
    cell::Cell,
    ffi::{CStr, c_char},
    path::{Path, PathBuf},
    process::Command,
    ptr::null,
    sync::atomic::{AtomicUsize, Ordering},
};
use windows::core::{GUID, HRESULT, IUnknown_Vtbl};

mod dispatch_fixture_tests;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
#[derive(Clone, Copy)]
struct FixtureSnapshot {
    creates: u32,
    destroys: u32,
    root_tag: u64,
    version_tag: u64,
    backend_tag: u64,
    version: u32,
    flags: u32,
    render_width: u32,
    render_height: u32,
    upscale_width: u32,
    upscale_height: u32,
    device: usize,
    chain_valid: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DispatchObservation {
    calls: u32,
    tag: u64,
}

#[repr(C)]
struct FakeDevice {
    vtable: *const IUnknown_Vtbl,
    references: Cell<u32>,
}

unsafe extern "system" fn query_interface(
    _: *mut std::ffi::c_void,
    _: *const GUID,
    out: *mut *mut std::ffi::c_void,
) -> HRESULT {
    // SAFETY: COM callers provide writable output; this test never queries.
    unsafe { *out = null_mut() };
    HRESULT(0x8000_4002u32 as i32)
}

unsafe extern "system" fn add_ref(this: *mut std::ffi::c_void) -> u32 {
    // SAFETY: The test owns a leaked FakeDevice with this vtable.
    let fake = unsafe { &*this.cast::<FakeDevice>() };
    let next = fake.references.get() + 1;
    fake.references.set(next);
    next
}

unsafe extern "system" fn release(this: *mut std::ffi::c_void) -> u32 {
    // SAFETY: The test owns a leaked FakeDevice with this vtable.
    let fake = unsafe { &*this.cast::<FakeDevice>() };
    let next = fake.references.get() - 1;
    fake.references.set(next);
    next
}

static FAKE_VTABLE: IUnknown_Vtbl = IUnknown_Vtbl {
    QueryInterface: query_interface,
    AddRef: add_ref,
    Release: release,
};

fn fake_device() -> (ID3D12Device, &'static FakeDevice) {
    let fake: &'static FakeDevice = Box::leak(Box::new(FakeDevice {
        vtable: &FAKE_VTABLE,
        references: Cell::new(1),
    }));
    // SAFETY: This test-only COM stub provides IUnknown ownership methods.
    // The fixture records the pointer but never calls ID3D12Device methods.
    let device = unsafe { ID3D12Device::from_raw(std::ptr::from_ref(fake).cast_mut().cast()) };
    (device, fake)
}

fn fixture_runtime() -> (Runtime, libloading::Library) {
    let dir = std::env::temp_dir().join(format!(
        "fsr-sdk-upscaler-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join(format!("fixture{}", std::env::consts::DLL_SUFFIX));
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=cdylib",
            "--crate-name=upscaler_fixture",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/upscaler.rs"))
        .arg("-o")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // SAFETY: The locally compiled fixture has exactly the tested signatures.
    let observer = unsafe { libloading::Library::new(&path) }.unwrap();
    let runtime = unsafe { Runtime::load(&path) }.unwrap();
    (runtime, observer)
}

fn fixture_set(observer: &libloading::Library, create: u32, destroy: u32, null_create: u32) {
    // SAFETY: The fixture exports this exact function.
    unsafe {
        let set = observer
            .get::<extern "C" fn(u32, u32, u32)>(c"fixture_set_outcome")
            .unwrap();
        set(create, destroy, null_create);
    }
}

fn fixture_snapshot(observer: &libloading::Library) -> FixtureSnapshot {
    // SAFETY: The fixture exports this exact function and matching C struct.
    unsafe {
        let snapshot = observer
            .get::<extern "C" fn() -> FixtureSnapshot>(c"fixture_snapshot")
            .unwrap();
        snapshot()
    }
}

#[test]
fn private_dispatch_forwards_once_and_preserves_unknown_native_code() {
    let (runtime, observer) = fixture_runtime();
    let (device, _) = fake_device();
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    // SAFETY: These exports are provided by the local fixture, which only
    // snapshots the descriptor header and records no GPU commands.
    unsafe {
        let set = observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap();
        let snapshot = observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap();
        set(u32::MAX);
        let header = ffxApiHeader {
            r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
            pNext: null_mut(),
        };
        assert_eq!(upscaler.owner.dispatch(&header), u32::MAX);
        let result = snapshot();
        assert_eq!(result.calls, 1);
        assert_eq!(result.tag, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
    }
    upscaler.destroy().unwrap();
}

#[test]
fn public_dispatch_core_preserves_preflight_errors_and_poison_policy() {
    use crate::error::{DispatchValidation, InspectionObject, InspectionOperation, Operation};
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    // The local fixture records no GPU commands. A descriptor header is enough
    // to exercise the terminal policy after the separated inspection phase.
    let descriptor = || {
        // SAFETY: The fixture reads only the header; zero is a valid bit pattern
        // for this selected ABI descriptor and its pointers.
        let mut desc: ffxDispatchDescUpscale = unsafe { std::mem::zeroed() };
        desc.header.r#type = FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
        desc
    };
    let observation = || -> DispatchObservation {
        // SAFETY: The locally compiled fixture exports this exact signature.
        unsafe {
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
        }
    };
    let set_code = |code| {
        // SAFETY: The locally compiled fixture exports this exact signature.
        unsafe {
            observer
                .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                .unwrap()(code)
        }
    };
    let invalid = Error::InvalidDispatch {
        reason: DispatchValidation::CommandListType,
    };
    // SAFETY: No native call occurs for the injected preflight error.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Err(invalid)) },
        Err(invalid)
    );
    assert_eq!(observation().calls, 0);
    let com = Error::Dx12Inspection {
        object: InspectionObject::CommandList,
        operation: InspectionOperation::GetDevice,
        hresult: 0x8000_4005u32 as i32,
    };
    // SAFETY: No native call occurs for the injected COM inspection error.
    assert_eq!(unsafe { upscaler.dispatch_core(|_| Err(com)) }, Err(com));
    assert_eq!(observation().calls, 0);
    set_code(0);
    // SAFETY: This fixture records no GPU commands and reads only the header.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Ok(descriptor())) },
        Ok(())
    );
    assert_eq!(observation().calls, 1);
    assert_eq!(observation().tag, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
    set_code(u32::MAX);
    // SAFETY: This fixture records no GPU commands and reads only the header.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Ok(descriptor())) },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX
        })
    );
    assert_eq!(observation().calls, 2);
    // SAFETY: Poison stops inspection and native dispatch before any GPU work.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| panic!("inspected poisoned context")) },
        Err(Error::DispatchPoisoned)
    );
    assert_eq!(observation().calls, 2);
    upscaler.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
    assert_eq!(fake.references.get(), 1);
}

#[test]
fn poisoned_dispatch_still_uses_terminal_destroy_retention() {
    use crate::error::Operation;
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    fixture_set(&observer, 0, u32::MAX, 0);
    // SAFETY: Exact signature from the local fixture.
    unsafe {
        observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap()(u32::MAX)
    };
    // SAFETY: Fixture records no GPU work; descriptor header is sufficient.
    assert!(matches!(
        unsafe {
            upscaler.dispatch_core(|_| {
                let mut desc: ffxDispatchDescUpscale = std::mem::zeroed();
                desc.header.r#type = FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
                Ok(desc)
            })
        },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX
        })
    ));
    assert_eq!(
        upscaler.destroy(),
        Err(Error::Native {
            operation: Operation::Destroy,
            code: u32::MAX
        })
    );
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
    drop(device);
    drop(runtime);
    assert_eq!(fake.references.get(), 1);
    assert!(weak.upgrade().is_some());
}

#[test]
fn public_constructor_forwards_checked_chain_and_owns_device() {
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let samples = [
        sizes(),
        UpscalerOptions {
            max_render_size: Dimensions::new(1, 2).unwrap(),
            max_upscale_size: Dimensions::new(3, 4).unwrap(),
        },
        UpscalerOptions {
            max_render_size: Dimensions::new(2048, 1024).unwrap(),
            max_upscale_size: Dimensions::new(1024, 512).unwrap(),
        },
    ];
    for (index, options) in samples.into_iter().enumerate() {
        let context = Upscaler::new(&runtime, &device, options).unwrap();
        assert_eq!(fake.references.get(), 2);
        let snapshot = fixture_snapshot(&observer);
        assert_eq!(snapshot.creates, (index + 1) as u32);
        assert_eq!(snapshot.root_tag, FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE);
        assert_eq!(
            snapshot.version_tag,
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION
        );
        assert_eq!(
            snapshot.backend_tag,
            FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
        );
        assert_eq!(snapshot.version, FFX_UPSCALER_VERSION);
        assert_eq!(snapshot.flags, 0);
        assert_eq!(snapshot.chain_valid, 1);
        assert_eq!(snapshot.device, device.as_raw() as usize);
        assert_eq!(snapshot.render_width, options.max_render_size.width.get());
        assert_eq!(snapshot.render_height, options.max_render_size.height.get());
        assert_eq!(snapshot.upscale_width, options.max_upscale_size.width.get());
        assert_eq!(
            snapshot.upscale_height,
            options.max_upscale_size.height.get()
        );
        context.destroy().unwrap();
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, (index + 1) as u32);
    }
    let a = Upscaler::new(&runtime, &device, sizes()).unwrap();
    let b = Upscaler::new(&runtime, &device, sizes()).unwrap();
    drop(runtime);
    drop(device);
    assert_eq!(fake.references.get(), 2);
    assert!(weak.upgrade().is_some());
    a.destroy().unwrap();
    assert_eq!(fake.references.get(), 1);
    assert!(weak.upgrade().is_some());
    drop(b);
    assert_eq!(fake.references.get(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn zero_is_rejected_without_entering_native_create() {
    let (_runtime, observer) = fixture_runtime();
    for pair in [(0, 0), (0, 1), (1, 0)] {
        assert!(Dimensions::new(pair.0, pair.1).is_err());
    }
    assert_eq!(fixture_snapshot(&observer).creates, 0);
}

#[test]
fn public_failure_paths_preserve_com_and_runtime_policy() {
    for (create_code, destroy_code, null_create) in [(u32::MAX, 0, 0), (0, 0, 1), (0, u32::MAX, 0)]
    {
        let (runtime, observer) = fixture_runtime();
        let (device, fake) = fake_device();
        let weak = std::rc::Rc::downgrade(&runtime.state);
        fixture_set(&observer, create_code, destroy_code, null_create);
        let result = Upscaler::new(&runtime, &device, sizes());
        if create_code != 0 {
            assert!(matches!(
                result,
                Err(Error::Native {
                    operation: crate::error::Operation::Create,
                    code: u32::MAX
                })
            ));
            assert_eq!(fake.references.get(), 1);
            assert_eq!(fixture_snapshot(&observer).destroys, 0);
        } else if null_create != 0 {
            assert!(matches!(result, Err(Error::NativeInvariantViolation(_))));
            assert_eq!(fake.references.get(), 2);
            assert_eq!(fixture_snapshot(&observer).destroys, 0);
        } else {
            assert_eq!(
                result.unwrap().destroy(),
                Err(Error::Native {
                    operation: crate::error::Operation::Destroy,
                    code: u32::MAX
                })
            );
            assert_eq!(fake.references.get(), 2);
            assert_eq!(fixture_snapshot(&observer).destroys, 1);
        }
        drop(device);
        drop(runtime);
        assert_eq!(weak.upgrade().is_some(), create_code == 0);
        assert_eq!(fake.references.get(), u32::from(create_code == 0));
    }
}

#[test]
fn public_siblings_survive_failed_create_and_destroy() {
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let a = Upscaler::new(&runtime, &device, sizes()).unwrap();
    fixture_set(&observer, u32::MAX, 0, 0);
    assert!(matches!(
        Upscaler::new(&runtime, &device, sizes()),
        Err(Error::Native {
            operation: crate::error::Operation::Create,
            code: u32::MAX
        })
    ));
    assert_eq!(fake.references.get(), 2); // caller + A; failed sibling released
    fixture_set(&observer, 0, u32::MAX, 0);
    let b = Upscaler::new(&runtime, &device, sizes()).unwrap();
    drop(runtime);
    drop(device);
    assert_eq!(fake.references.get(), 2); // A + B
    assert_eq!(
        a.destroy(),
        Err(Error::Native {
            operation: crate::error::Operation::Destroy,
            code: u32::MAX
        })
    );
    fixture_set(&observer, 0, 0, 0);
    b.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 2);
    assert_eq!(fake.references.get(), 1); // A retained after destroy failure
    assert!(weak.upgrade().is_some());
}

fn sizes() -> UpscalerOptions {
    UpscalerOptions {
        max_render_size: Dimensions::new(1280, 720).unwrap(),
        max_upscale_size: Dimensions::new(1920, 1080).unwrap(),
    }
}

#[test]
fn descriptor_links_survive_moves_of_outer_storage() {
    // Address sentinel only; no native call and no device dereference.
    let device = std::ptr::dangling_mut::<fsr_sdk_sys::dx12::ID3D12Device>();
    let (state, root) = UpscaleCreateState::new(device, sizes());
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
    assert_eq!(moved.root.maxRenderSize.width, 1280);
    assert_eq!(moved.root.maxRenderSize.height, 720);
    assert_eq!(moved.root.maxUpscaleSize.width, 1920);
    assert_eq!(moved.root.maxUpscaleSize.height, 1080);
}

#[test]
fn dimensions_reject_every_zero_component() {
    for (width, height) in [(0, 0), (0, 720), (1280, 0)] {
        assert_eq!(
            Dimensions::new(width, height),
            Err(Error::InvalidDimensions { width, height })
        );
    }
    // Both option fields use the same checked type.
    assert!(Dimensions::new(u32::MAX, 1).is_ok());
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
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tests/run-m5.ps1"]
fn native_lifecycle() {
    native_lifecycle_case(false);
}

#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tests/run-m5.ps1"]
fn native_lifecycle_drop() {
    native_lifecycle_case(true);
}

fn native_lifecycle_case(drop_b: bool) {
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
    let runtime = unsafe { Runtime::load(&loader) }.unwrap();
    assert!(Dimensions::new(0, 720).is_err());
    assert!(Dimensions::new(1280, 0).is_err());
    eprintln!("Dimensions: render=1280x720 upscale=1920x1080; zero rejected in Rust");
    let context_a = Upscaler::new(&runtime, &device, sizes())
        .unwrap_or_else(|error| stop_after_native_error("create A", error));
    let context_b = Upscaler::new(&runtime, &device, sizes())
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
    let mut query = ProviderQuery {
        header: ffxApiHeader {
            r#type: 6,
            pNext: null_mut(),
        },
        id: 0,
        name: null(),
    };
    // SAFETY: Correct test-only ABI, live owner and writable query storage.
    let result = unsafe { context_b.owner.query_for_test(&raw mut query.header) };
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

macro_rules! assert_runtime_not_impl {
    ($bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Ambiguous<IfImplemented> for T {}
            let _ = <Runtime as Ambiguous<_>>::check;
        };
    };
}
assert_runtime_not_impl!(Send);
assert_runtime_not_impl!(Sync);
assert_runtime_not_impl!(Copy);
assert_runtime_not_impl!(Clone);

fn stop_after_native_error(operation: &str, error: Error) -> ! {
    // Production policy has already run (including create-error RAII or
    // exceptional retention). Preserve any remaining test-helper ownership on
    // failure rather than introduce an extra unload into this native smoke test.
    eprintln!("Production owner: {operation} failed: {error}");
    std::process::exit(1)
}
