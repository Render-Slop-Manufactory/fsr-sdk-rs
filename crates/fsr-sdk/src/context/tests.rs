// SPDX-License-Identifier: MPL-2.0

use super::*;
#[cfg(not(all(windows, feature = "dx12")))]
use std::cell::Cell;
use std::{
    cell::RefCell,
    ffi::c_void,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "../../tests/fixtures/lifecycle.rs"]
mod fixture;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "fsr-sdk-context-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

fn library() -> (FfxLibrary, PathBuf) {
    let path = directory().join(format!("fixture{}", std::env::consts::DLL_SUFFIX));
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=cdylib",
            "--crate-name=lifecycle_fixture",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lifecycle.rs"))
        .arg("-o")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // SAFETY: Locally compiled fixture with matching exports, not AMD code.
    (unsafe { FfxLibrary::load(&path) }.unwrap(), path)
}

fn assert_loaded(ledger: &Ledger, expected: bool) {
    #[cfg(all(windows, feature = "dx12"))]
    assert_eq!(
        libloading::os::windows::Library::open_already_loaded(&ledger.path).is_ok(),
        expected
    );
    #[cfg(not(all(windows, feature = "dx12")))]
    assert_eq!(!ledger.released.get(), expected);
}

struct Ledger {
    events: RefCell<Vec<u32>>,
    #[cfg(all(windows, feature = "dx12"))]
    path: PathBuf,
    #[cfg(not(all(windows, feature = "dx12")))]
    released: Rc<Cell<bool>>,
}

extern "C" fn event(data: *mut c_void, value: u32) {
    // SAFETY: State owns an Rc to this allocation through every native call.
    let ledger = unsafe { &*data.cast::<Ledger>() };
    ledger.events.borrow_mut().push(value);
}

struct State {
    input: Box<fixture::Input>,
    ledger: Rc<Ledger>,
}
impl Drop for State {
    fn drop(&mut self) {
        assert_loaded(&self.ledger, true);
        self.ledger.events.borrow_mut().push(3);
    }
}
struct Device(Rc<Ledger>);
impl Drop for Device {
    fn drop(&mut self) {
        assert_loaded(&self.0, true);
        self.0.events.borrow_mut().push(4);
    }
}
type Owner = NativeContextOwner<State, Device>;

fn create(
    create_code: u32,
    destroy_code: u32,
    null_create: bool,
    clear_destroy: bool,
) -> (Result<Owner, Error>, Rc<Ledger>) {
    let (library, path) = library();
    let ledger = Rc::new(Ledger {
        events: RefCell::new(Vec::new()),
        #[cfg(all(windows, feature = "dx12"))]
        path,
        #[cfg(not(all(windows, feature = "dx12")))]
        released: Rc::new(Cell::new(false)),
    });
    #[cfg(not(all(windows, feature = "dx12")))]
    let library = {
        let _ = path;
        let mut library = library;
        library.observe_release(ledger.released.clone());
        library
    };
    let mut state = State {
        input: Box::new(fixture::Input {
            tag: 0,
            next: null_mut(),
            create_code,
            destroy_code,
            null_create: null_create.into(),
            clear_destroy: clear_destroy.into(),
            event,
            data: Rc::as_ptr(&ledger).cast_mut().cast(),
        }),
        ledger: ledger.clone(),
    };
    let root = (&raw mut *state.input).cast();
    // SAFETY: Fixture owns stable state, does no GPU work, and accesses only its
    // declared input. Device is a drop spy; no fake COM is passed to AMD.
    let result = unsafe { Owner::create(state, Device(ledger.clone()), library, root) };
    (result, ledger)
}

#[test]
fn successful_teardown_is_once_and_ordered_for_explicit_destroy_and_drop() {
    for clear in [false, true] {
        for explicit in [false, true] {
            let (owner, ledger) = create(0, 0, false, clear);
            let owner = Box::new(owner.unwrap()); // move outer owner
            assert_eq!(*ledger.events.borrow(), [1]);
            assert_loaded(&ledger, true);
            if explicit {
                (*owner).destroy().unwrap();
            } else {
                drop(owner);
            }
            assert_eq!(*ledger.events.borrow(), [1, 2, 3, 4]);
            assert_loaded(&ledger, false);
        }
    }
}

#[test]
fn returned_create_errors_never_destroy_even_nonnull_outputs() {
    for null in [false, true] {
        let (owner, ledger) = create(u32::MAX, 0, null, false);
        assert!(matches!(
            owner,
            Err(Error::Native {
                operation: Operation::Create,
                code: u32::MAX
            })
        ));
        assert_eq!(*ledger.events.borrow(), [1, 3, 4]);
        assert_loaded(&ledger, false);
    }
}

#[test]
fn ok_null_retains_every_dependency_without_destroy() {
    let (owner, ledger) = create(0, 0, true, false);
    assert!(matches!(
        owner,
        Err(Error::NativeInvariantViolation(
            InvariantViolation::NullContextOnSuccess
        ))
    ));
    assert_eq!(*ledger.events.borrow(), [1]);
    assert_eq!(Rc::strong_count(&ledger), 3); // external + state + device
    assert_loaded(&ledger, true);
}

#[test]
fn destroy_errors_retain_every_dependency_and_never_retry() {
    for clear in [false, true] {
        for explicit in [false, true] {
            let (owner, ledger) = create(0, u32::MAX, false, clear);
            let owner = owner.unwrap();
            if explicit {
                assert_eq!(
                    owner.destroy(),
                    Err(Error::Native {
                        operation: Operation::Destroy,
                        code: u32::MAX
                    })
                );
            } else {
                drop(owner);
            }
            assert_eq!(*ledger.events.borrow(), [1, 2]);
            assert_eq!(Rc::strong_count(&ledger), 3);
            assert_loaded(&ledger, true);
        }
    }
}

// Ambiguity assertions check the real private type, not import privacy. Each
// expression ceases to compile if the trait is implemented, without a dependency.
macro_rules! assert_not_impl {
    ($ty:ty, $bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Ambiguous<IfImplemented> for T {}
            let _ = <$ty as Ambiguous<_>>::check;
        };
    };
}
assert_not_impl!(Owner, Send);
assert_not_impl!(Owner, Sync);
assert_not_impl!(Owner, Copy);
assert_not_impl!(Owner, Clone);

#[test]
fn consumed_owner_and_private_handle_are_compile_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let dir = directory();
    // Compile the actual owner source inside a small harness. A positive control
    // rules out failed imports/dependencies as the reason negative cases fail.
    let prefix = format!(
        r#"
#![allow(dead_code)]
extern crate self as fsr_sdk_sys;
// Only the types consumed by context.rs; this harness checks ownership, not ABI.
#[allow(non_camel_case_types)]
mod api {{
    pub type ffxContext = *mut std::ffi::c_void;
    pub type ffxCreateContextDescHeader = std::ffi::c_void;
    pub type ffxAllocationCallbacks = std::ffi::c_void;
    pub const FFX_API_RETURN_OK: u32 = 0;
}}
#[path = {:?}] mod error;
#[path = {:?}] mod context;
mod runtime {{
    use crate::api::*;
    pub struct FfxLibrary;
    impl FfxLibrary {{
        pub unsafe fn ffxCreateContext(&self, _: *mut ffxContext, _: *mut ffxCreateContextDescHeader, _: *const ffxAllocationCallbacks) -> u32 {{ 0 }}
        pub unsafe fn ffxDestroyContext(&self, _: *mut ffxContext, _: *const ffxAllocationCallbacks) -> u32 {{ 0 }}
    }}
}}
use context::NativeContextOwner;
"#,
        root.join("error.rs"),
        root.join("context.rs")
    );
    for (name, body, diagnostic) in [
        (
            "positive",
            "fn use_owner(owner: NativeContextOwner<(), ()>) { let _ = owner.destroy(); }",
            None,
        ),
        (
            "consumed",
            "fn use_owner(owner: NativeContextOwner<(), ()>) { let _ = owner.destroy(); drop(owner); }",
            Some("E0382"),
        ),
        (
            "private",
            "fn use_owner(owner: NativeContextOwner<(), ()>) { let _ = owner.live; }",
            Some("E0616"),
        ),
    ] {
        let source = dir.join(format!("{name}.rs"));
        std::fs::write(&source, format!("{prefix}\n{body}")).unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&dir)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        match diagnostic {
            None => assert!(output.status.success(), "{stderr}"),
            Some(code) => {
                assert!(!output.status.success());
                assert!(stderr.contains(code), "{stderr}");
            }
        }
    }
}
