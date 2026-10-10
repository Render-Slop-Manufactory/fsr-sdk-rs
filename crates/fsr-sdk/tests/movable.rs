// SPDX-License-Identifier: MPL-2.0

#![cfg(all(windows, feature = "dx12", feature = "movable"))]

use fsr_sdk::{
    runtime::{MovableRuntime, Runtime},
    upscaler::{Dimensions, ID3D12Device, MovableUpscaler, Upscaler, UpscalerOptions},
};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
        mpsc,
    },
    thread,
};
use windows::core::{GUID, HRESULT, IUnknown_Vtbl, Interface};

// Compile and exercise exactly the consumer example, using only the public API.
#[path = "fixtures/child.rs"]
mod child;
#[path = "fixtures/directory.rs"]
mod directory;

fn child_case(name: &str) -> bool {
    const VARIABLE: &str = "FSR_SDK_TEST_PUBLIC_MOVABLE_CASE";
    if std::env::var(VARIABLE).as_deref() == Ok(name) {
        true
    } else {
        child::run(
            &directory::directory("fsr-sdk-movable-child"),
            name,
            VARIABLE,
            name,
        );
        false
    }
}
#[allow(dead_code)]
#[path = "../examples/movable_upscale.rs"]
mod example;

macro_rules! not_impl {
    ($ty:ty, $bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct Implemented;
            impl<T: ?Sized + $bound> Ambiguous<Implemented> for T {}
            let _ = <$ty as Ambiguous<_>>::check;
        };
    };
}
not_impl!(Runtime, Send);
not_impl!(Runtime, Sync);
not_impl!(Upscaler, Send);
not_impl!(Upscaler, Sync);
not_impl!(MovableRuntime, Sync);
not_impl!(MovableRuntime, Clone);
not_impl!(MovableUpscaler, Sync);
not_impl!(MovableUpscaler, Clone);

#[test]
fn public_trait_matrix() {
    fn send<T: Send>() {}
    send::<MovableRuntime>();
    send::<MovableUpscaler>();
    send::<example::renderer_integration::CoordinatedContext>();
}

#[test]
fn invalid_acquisition_returns_existing_path_error_without_loading() {
    // SAFETY: The empty path is rejected before loading any native code.
    let error = unsafe { MovableRuntime::load_unchecked("") }
        .err()
        .expect("empty path must fail before acquisition");
    assert!(matches!(error, fsr_sdk::runtime::LoadError::Path { .. }));
}

#[repr(C)]
struct Device {
    vtable: *const IUnknown_Vtbl,
    // Atomic ownership-only stub, unlike the existing thread-confined Cell stub.
    references: AtomicU32,
}
unsafe extern "system" fn query(_: *mut c_void, _: *const GUID, out: *mut *mut c_void) -> HRESULT {
    // SAFETY: COM supplies a writable output; lifecycle fixtures never query it.
    unsafe { *out = std::ptr::null_mut() };
    HRESULT(0x8000_4002u32 as i32)
}
unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: The leaked stub has a live, correctly aligned Device allocation.
    unsafe { &*this.cast::<Device>() }
        .references
        .fetch_add(1, Ordering::SeqCst)
        + 1
}
unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: Every release balances an owned reference; allocation is test-retained.
    unsafe { &*this.cast::<Device>() }
        .references
        .fetch_sub(1, Ordering::SeqCst)
        - 1
}
static VTABLE: IUnknown_Vtbl = IUnknown_Vtbl {
    QueryInterface: query,
    AddRef: add_ref,
    Release: release,
};
fn device() -> (ID3D12Device, &'static AtomicU32) {
    let stub = Box::leak(Box::new(Device {
        vtable: &VTABLE,
        references: AtomicU32::new(1),
    }));
    // SAFETY: Exact ownership-only IUnknown stub; fixture never calls device methods.
    let interface = unsafe { ID3D12Device::from_raw(std::ptr::from_mut(stub).cast()) };
    (interface, &stub.references)
}

fn fixture() -> PathBuf {
    let dir = directory::directory("fsr-sdk-public-movable");
    let name = dir.file_name().unwrap().to_string_lossy();
    let path = dir.join(format!("{name}.dll"));
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=cdylib",
            "--crate-name=movable_fixture",
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
    path
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}
fn loaded(path: &Path) -> bool {
    let name: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: NUL-terminated path; instantaneous observation adds no reference.
    // Never dereference/free/call through the returned handle.
    !unsafe { GetModuleHandleW(name.as_ptr()) }.is_null()
}
fn sizes() -> UpscalerOptions {
    UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    )
}

#[test]
fn consumer_siblings_unwind_and_failed_handoff_keep_cleanup_coordinated() {
    let case = "consumer_siblings_unwind_and_failed_handoff_keep_cleanup_coordinated";
    if !child_case(case) {
        return;
    }
    let path = fixture();
    let (device, references) = device();
    let gate = Arc::new(Mutex::new(()));
    assert!(!loaded(&path));
    // SAFETY: Trusted fixture, no hidden work/GPU commands; this single domain
    // covers all ownership/cleanup, and the stub's reference counts are atomic.
    let (first, second) = unsafe {
        example::renderer_integration::create_pair(&path, &device, sizes(), gate.clone())
    }
    .unwrap();
    assert!(loaded(&path)); // Public runtime was already dropped inside create_pair.
    assert_eq!(references.load(Ordering::SeqCst), 3);
    drop(device);

    let (sender, receiver) = mpsc::sync_channel(0);
    drop(receiver);
    let worker = thread::spawn(move || {
        // The failed send returns the owning consumer wrapper. Its automatic
        // Drop must acquire the domain instead of losing coordination here.
        let failed = sender.send(first).err().unwrap();
        drop(failed);
    });
    worker.join().unwrap();
    assert_eq!(references.load(Ordering::SeqCst), 1);
    assert!(loaded(&path)); // The independently held survivor pins the loader.

    let worker = thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(move || {
            let _context = second;
            panic!("declared consumer unwind");
        }));
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"declared consumer unwind")
        );
    });
    worker.join().unwrap();
    assert_eq!(references.load(Ordering::SeqCst), 0);
    assert!(!loaded(&path)); // No observer module reference masks the final release.
    println!("CASE PASS {case}");
}

#[test]
fn runtime_can_move_before_creation_and_survive_independent_sibling_cleanup() {
    let case = "runtime_can_move_before_creation_and_survive_independent_sibling_cleanup";
    if !child_case(case) {
        return;
    }
    let path = fixture();
    let (device, references) = device();
    // SAFETY: Trusted fixture; joined workers serialize all native ownership calls.
    let runtime = unsafe { MovableRuntime::load_unchecked(&path) }.unwrap();
    let (first, second) = thread::spawn(move || {
        let first = MovableUpscaler::new(&runtime, &device, sizes()).unwrap();
        let second = MovableUpscaler::new(&runtime, &device, sizes()).unwrap();
        drop(runtime);
        drop(device);
        (first, second)
    })
    .join()
    .unwrap();
    assert!(loaded(&path));
    thread::spawn(move || first.destroy().unwrap())
        .join()
        .unwrap();
    assert!(loaded(&path));
    thread::spawn(move || drop(second)).join().unwrap();
    assert_eq!(references.load(Ordering::SeqCst), 0);
    assert!(!loaded(&path));
    println!("CASE PASS {case}");
}

#[test]
fn poisoned_consumer_gate_still_serializes_automatic_cleanup() {
    let case = "poisoned_consumer_gate_still_serializes_automatic_cleanup";
    if !child_case(case) {
        return;
    }
    let path = fixture();
    let (device, references) = device();
    let gate = Arc::new(Mutex::new(()));
    // SAFETY: Same controlled fixture and common-domain premises as above.
    let (first, second) = unsafe {
        example::renderer_integration::create_pair(&path, &device, sizes(), gate.clone())
    }
    .unwrap();
    drop(device);
    let poison_gate = gate.clone();
    assert!(
        thread::spawn(move || {
            let _guard = poison_gate.lock().unwrap();
            panic!("application panic between native operations");
        })
        .join()
        .is_err()
    );
    assert!(gate.is_poisoned());
    let a = thread::spawn(move || drop(first));
    let b = thread::spawn(move || second.destroy().unwrap());
    a.join().unwrap();
    b.join().unwrap();
    assert_eq!(references.load(Ordering::SeqCst), 0);
    assert!(!loaded(&path));
    println!("CASE PASS {case}");
}
