// SPDX-License-Identifier: MPL-2.0

use super::*;
#[cfg(not(all(windows, feature = "dx12")))]
use crate::runtime::FfxLibrary;
#[cfg(all(windows, feature = "dx12"))]
use fsr_sdk_sys::loader::FfxLibrary;
#[cfg(not(all(windows, feature = "dx12")))]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{
    cell::RefCell,
    ffi::c_void,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "../../tests/fixtures/lifecycle.rs"]
pub(super) mod fixture;

fn directory() -> PathBuf {
    crate::test_directory::directory("fsr-sdk-context")
}

pub(super) fn library() -> (FfxLibrary, PathBuf) {
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
    assert_eq!(!ledger.released.load(Ordering::SeqCst), expected);
}

struct Ledger {
    events: RefCell<Vec<u32>>,
    #[cfg(all(windows, feature = "dx12"))]
    path: PathBuf,
    #[cfg(not(all(windows, feature = "dx12")))]
    released: Arc<AtomicBool>,
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
        released: Arc::new(AtomicBool::new(false)),
    });
    #[cfg(not(all(windows, feature = "dx12")))]
    let library = {
        let _ = path;
        let mut library = library;
        library.observe_release(ledger.released.clone());
        library
    };
    let runtime = Rc::new(crate::runtime::RuntimeState::new(library));
    (
        create_on(
            &runtime,
            &ledger,
            create_code,
            destroy_code,
            null_create,
            clear_destroy,
        ),
        ledger,
    )
}

fn create_on(
    runtime: &Rc<crate::runtime::RuntimeState>,
    ledger: &Rc<Ledger>,
    create_code: u32,
    destroy_code: u32,
    null_create: bool,
    clear_destroy: bool,
) -> Result<Owner, Error> {
    let mut state = State {
        input: Box::new(fixture::Input {
            tag: 0,
            next: null_mut(),
            create_code,
            destroy_code,
            null_create: null_create.into(),
            clear_destroy: clear_destroy.into(),
            event,
            data: Rc::as_ptr(ledger).cast_mut().cast(),
        }),
        ledger: ledger.clone(),
    };
    let root = (&raw mut *state.input).cast();
    // SAFETY: Fixture owns stable state, does no GPU work, and accesses only its
    // declared input. Device is a drop spy; no fake COM is passed to AMD.
    unsafe { Owner::create(state, Device(ledger.clone()), runtime.clone(), root) }
}

fn shared_runtime() -> (Rc<crate::runtime::RuntimeState>, Rc<Ledger>) {
    let (library, path) = library();
    let ledger = Rc::new(Ledger {
        events: RefCell::new(Vec::new()),
        #[cfg(all(windows, feature = "dx12"))]
        path,
        #[cfg(not(all(windows, feature = "dx12")))]
        released: Arc::new(AtomicBool::new(false)),
    });
    #[cfg(not(all(windows, feature = "dx12")))]
    let library = {
        let _ = path;
        let mut library = library;
        library.observe_release(ledger.released.clone());
        library
    };
    (Rc::new(crate::runtime::RuntimeState::new(library)), ledger)
}

#[test]
fn shared_runtime_survives_public_owner_and_first_context() {
    let (runtime, ledger) = shared_runtime();
    let weak = Rc::downgrade(&runtime);
    let a = create_on(&runtime, &ledger, 0, 0, false, false).unwrap();
    let b = create_on(&runtime, &ledger, 0, 0, false, false).unwrap();
    drop(runtime);
    assert_eq!(*ledger.events.borrow(), [1, 1]);
    a.destroy().unwrap();
    assert_eq!(*ledger.events.borrow(), [1, 1, 2, 3, 4]);
    assert!(weak.upgrade().is_some());
    assert_loaded(&ledger, true);
    drop(b);
    assert_eq!(*ledger.events.borrow(), [1, 1, 2, 3, 4, 2, 3, 4]);
    assert!(weak.upgrade().is_none());
    assert_loaded(&ledger, false);
}

#[test]
fn failed_sibling_create_preserves_live_sibling() {
    let (runtime, ledger) = shared_runtime();
    let a = create_on(&runtime, &ledger, 0, 0, false, false).unwrap();
    let b = create_on(&runtime, &ledger, u32::MAX, 0, false, false);
    assert!(matches!(
        b,
        Err(Error::Native(crate::error::NativeError {
            operation: Operation::Create,
            code: u32::MAX
        }))
    ));
    drop(runtime);
    assert_eq!(*ledger.events.borrow(), [1, 1, 3, 4]);
    assert_loaded(&ledger, true);
    a.destroy().unwrap();
    assert_eq!(*ledger.events.borrow(), [1, 1, 3, 4, 2, 3, 4]);
    assert_loaded(&ledger, false);
}

#[test]
fn sibling_destroy_failure_retains_runtime_after_other_cleanup() {
    let (runtime, ledger) = shared_runtime();
    let weak = Rc::downgrade(&runtime);
    let a = create_on(&runtime, &ledger, 0, u32::MAX, false, false).unwrap();
    let b = create_on(&runtime, &ledger, 0, 0, false, false).unwrap();
    drop(runtime);
    assert_result!(
        a.destroy(),
        Err(Error::native(Operation::Destroy, u32::MAX))
    );
    b.destroy().unwrap();
    assert_eq!(*ledger.events.borrow(), [1, 1, 2, 2, 3, 4]);
    assert!(weak.upgrade().is_some());
    assert_loaded(&ledger, true);
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
            Err(Error::Native(crate::error::NativeError {
                operation: Operation::Create,
                code: u32::MAX
            }))
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
        Err(Error::Invariant {
            operation: crate::error::Operation::Create,
            reason: InvariantViolation::NullContextOnSuccess
        })
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
                assert_result!(
                    owner.destroy(),
                    Err(Error::native(Operation::Destroy, u32::MAX))
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
    pub type ffxDispatchDescHeader = std::ffi::c_void;
    pub type ffxAllocationCallbacks = std::ffi::c_void;
    pub type ffxReturnCode_t = u32;
    pub const FFX_API_RETURN_OK: u32 = 0;
}}
// Populate query dependencies from the compiled raw crate, including when this
// test runs from an extracted package without a sibling source checkout.
mod upscale {{
    pub const FFX_API_QUERY_RESOURCE_INPUT_COLOR: u32 = {color};
    pub const FFX_API_QUERY_RESOURCE_INPUT_DEPTH: u32 = {depth};
    pub const FFX_API_QUERY_RESOURCE_INPUT_MV: u32 = {motion};
    pub const FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE: u32 = {exposure};
    pub const FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK: u32 = {reactive};
    pub const FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION: u32 = {transparency};
}}
#[path = {:?}] mod error;
#[path = {:?}] mod query;
mod upscaler {{ #[path = {:?}] pub mod common; pub use common::TuningParameter; }}
#[path = {:?}] mod context;
mod runtime {{
    use crate::api::*;
    pub struct FfxLibrary;
    #[path = {:?}] mod state;
    pub(crate) use state::RuntimeState;
    impl FfxLibrary {{
        pub unsafe fn ffxCreateContext(&self, _: *mut ffxContext, _: *mut ffxCreateContextDescHeader, _: *const ffxAllocationCallbacks) -> u32 {{ 0 }}
        pub unsafe fn ffxDestroyContext(&self, _: *mut ffxContext, _: *const ffxAllocationCallbacks) -> u32 {{ 0 }}
        pub unsafe fn ffxDispatch(&self, _: *mut ffxContext, _: *const ffxDispatchDescHeader) -> u32 {{ 0 }}
    }}
}}
use context::NativeContextOwner;
"#,
        root.join("error.rs"),
        root.join("query.rs"),
        root.join("upscaler/common.rs"),
        root.join("context.rs"),
        root.join("runtime/state.rs"),
        color = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_COLOR,
        depth = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_DEPTH,
        motion = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_MV,
        exposure = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE,
        reactive = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK,
        transparency = fsr_sdk_sys::upscale::FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION,
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
            "arc_positive",
            "fn use_owner(owner: NativeContextOwner<(), (), std::sync::Arc<runtime::RuntimeState>>) { let _ = owner.destroy(); }",
            None,
        ),
        (
            "family_mismatch",
            "fn requires_local(_: NativeContextOwner<(), ()>) {} fn mix(owner: NativeContextOwner<(), (), std::sync::Arc<runtime::RuntimeState>>) { requires_local(owner); }",
            Some("E0308"),
        ),
        (
            "retention_sealed",
            "struct CustomRuntime(runtime::RuntimeState); impl context::RuntimeRetention for CustomRuntime { fn runtime_state(&self) -> &runtime::RuntimeState { &self.0 } }",
            Some("E0277"),
        ),
        (
            "private",
            "fn use_owner(owner: NativeContextOwner<(), ()>) { let _ = owner.live; }",
            Some("E0616"),
        ),
        (
            "runtime_library_private",
            "fn extract_library(state: runtime::RuntimeState) { let _ = state.library; }",
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

#[test]
fn configure_forwards_const_descriptor_and_exact_global_or_live_code() {
    let (runtime, ledger) = shared_runtime();
    let mut owner = create_on(&runtime, &ledger, 0, 0, false, false).unwrap();
    let desc = fixture::Dispatch {
        tag: 0xc028,
        next: null_mut(),
        code: 0xa1234567,
    };
    let ptr = (&raw const desc).cast::<fsr_sdk_sys::api::ffxConfigureDescHeader>();
    // SAFETY: Matched fixture protocol with retained context; no GPU or callbacks
    // beyond the owned test ledger, and no pointer retention or handle mutation.
    unsafe {
        assert_eq!(runtime.configure(null_mut(), ptr), desc.code);
        assert_eq!(owner.configure(ptr), desc.code);
    }
    assert_eq!(*ledger.events.borrow(), [1, 6]);
    owner.destroy().unwrap();
    assert_eq!(*ledger.events.borrow(), [1, 6, 2, 3, 4]);
}

#[test]
fn tuning_through_loaded_owner_preserves_storage_poison_and_terminal_retention() {
    use crate::{
        context_policy::Eligibility,
        upscaler::{TuningParameter as P, TuningRequest, TuningValue as V},
    };
    for destroy_code in [0, u32::MAX] {
        for explicit in [false, true] {
            let (owner, ledger) = create(0, destroy_code, false, false);
            let mut owner = owner.unwrap();
            let mut eligibility = Eligibility::Usable;
            // SAFETY: This locally built fixture consumes descriptors synchronously,
            // does no GPU work and leaves the owner destroyable on rejection.
            unsafe {
                crate::upscaler::tuning::configure(
                    &mut eligibility,
                    TuningRequest::new(P::VelocityFactor, V::Value(-0.0)).unwrap(),
                    |ptr| owner.configure(ptr),
                )
                .unwrap();
                crate::upscaler::tuning::configure(
                    &mut eligibility,
                    TuningRequest::new(P::AccumulationAddedPerFrame, V::ProviderDefault).unwrap(),
                    |ptr| owner.configure(ptr),
                )
                .unwrap();
            }
            assert_eq!(
                *ledger.events.borrow(),
                [1, 20, (-0.0f32).to_bits(), 23, 30]
            );
            let storage = Box::new(owner);
            let mut owner = *storage;
            // SAFETY: Same synchronous, idle fixture after moving the owner.
            let result = unsafe {
                crate::upscaler::tuning::configure(
                    &mut eligibility,
                    TuningRequest::new(P::ReactivenessScale, V::Value(f32::MAX)).unwrap(),
                    |ptr| owner.configure(ptr),
                )
            };
            assert_result!(result, Err(Error::native(Operation::Tuning, 0xa1234567)));
            assert_eq!(
                eligibility,
                crate::context_policy::failed(crate::error::Operation::Tuning, 0xa1234567)
            );
            let before = ledger.events.borrow().clone();
            // SAFETY: Poison rejects before touching the fixture or its owner.
            assert_result!(
                unsafe {
                    crate::upscaler::tuning::configure(
                        &mut eligibility,
                        TuningRequest::new(P::ReactivenessScale, V::ProviderDefault).unwrap(),
                        |_| panic!("poison called native"),
                    )
                },
                Err(crate::test_errors::terminal(Error::native(
                    crate::error::Operation::Tuning,
                    0xa1234567
                )))
            );
            assert_eq!(*ledger.events.borrow(), before);
            if explicit {
                let expected = if destroy_code == 0 {
                    Ok(())
                } else {
                    Err(Error::native(Operation::Destroy, destroy_code))
                };
                assert_result!(owner.destroy(), expected);
            } else {
                drop(owner);
            }
            let events = ledger.events.borrow();
            assert_eq!(events.iter().filter(|&&e| e == 2).count(), 1);
            if destroy_code == 0 {
                assert!(events.ends_with(&[2, 3, 4]));
            } else {
                assert!(events.ends_with(&[2]));
                assert!(!events.contains(&3) && !events.contains(&4));
                assert_eq!(Rc::strong_count(&ledger), 3);
                assert_loaded(&ledger, true);
            }
        }
    }
}

#[test]
fn generated_mask_through_loaded_owner_keeps_mapping_poison_and_terminal_retention() {
    use crate::{
        context_policy::{self, Eligibility},
        upscaler::{Dimensions, ReactiveMaskOptions, reactive_mask::build_descriptor},
    };
    for destroy_code in [0, u32::MAX] {
        for explicit in [false, true] {
            let (owner, ledger) = create(0, destroy_code, false, false);
            let mut owner = owner.unwrap();
            let mut eligibility = Eligibility::Usable;
            for flags in 0..16 {
                let mut options = ReactiveMaskOptions::new(-0.0, -2.0, 2.0).unwrap();
                options.set_apply_tonemap(flags & 1 != 0);
                options.set_apply_inverse_tonemap(flags & 2 != 0);
                options.set_apply_threshold(flags & 4 != 0);
                options.set_use_components_max(flags & 8 != 0);
                let before = ledger.events.borrow().len();
                context_policy::dispatch(
                    crate::error::Operation::GenerateReactiveMask,
                    &mut eligibility,
                    &mut owner,
                    |_| {
                        // SAFETY: These zero-valued raw resources contain no Rust
                        // references; the controlled fixture never dereferences them.
                        let resources = unsafe { std::mem::zeroed() };
                        Ok(build_descriptor(
                            null_mut(),
                            resources,
                            Dimensions::new(960, 540).unwrap(),
                            &options,
                        ))
                    },
                    |owner, desc| {
                        // SAFETY: Complete synchronous fixture descriptor, retained
                        // owner/runtime; no GPU work or resource pointer dereferences.
                        unsafe { owner.dispatch(&desc.header) }
                    },
                )
                .unwrap();
                assert_eq!(
                    &ledger.events.borrow()[before..],
                    &[
                        5,
                        flags,
                        (-0.0f32).to_bits(),
                        (-2.0f32).to_bits(),
                        2.0f32.to_bits(),
                        960,
                        540
                    ]
                );
            }
            let mut owner = *Box::new(owner);
            let options = ReactiveMaskOptions::new(f32::MAX, 0.0, 0.0).unwrap();
            assert_result!(
                context_policy::dispatch(
                    crate::error::Operation::GenerateReactiveMask,
                    &mut eligibility,
                    &mut owner,
                    |_| {
                        // SAFETY: POD resources, never dereferenced by this fixture.
                        Ok(build_descriptor(
                            null_mut(),
                            unsafe { std::mem::zeroed() },
                            Dimensions::new(1, 1).unwrap(),
                            &options,
                        ))
                    },
                    |owner, desc| {
                        // SAFETY: Matching loaded fixture; arbitrary rejection remains
                        // terminal and never represents an AMD capability observation.
                        unsafe { owner.dispatch(&desc.header) }
                    },
                ),
                Err(Error::native(Operation::GenerateReactiveMask, 7))
            );
            assert_eq!(
                eligibility,
                crate::context_policy::failed(crate::error::Operation::GenerateReactiveMask, 7)
            );
            let before = ledger.events.borrow().clone();
            assert_result!(
                context_policy::dispatch(
                    crate::error::Operation::GenerateReactiveMask,
                    &mut eligibility,
                    &mut owner,
                    |_| -> Result<(), Error> { panic!("poison inspected") },
                    |_, _| panic!("poison reached native")
                ),
                Err(crate::test_errors::terminal(Error::native(
                    crate::error::Operation::GenerateReactiveMask,
                    7
                )))
            );
            assert_eq!(*ledger.events.borrow(), before);
            if explicit {
                assert_result!(
                    owner.destroy(),
                    if destroy_code == 0 {
                        Ok(())
                    } else {
                        Err(Error::native(Operation::Destroy, destroy_code))
                    }
                );
            } else {
                drop(owner);
            }
            let events = ledger.events.borrow();
            // Earlier events may equal 2; the exact terminal suffix detects cleanup.
            if destroy_code == 0 {
                assert!(events.ends_with(&[2, 3, 4]));
            } else {
                assert!(events.ends_with(&[2]));
                assert_eq!(Rc::strong_count(&ledger), 3);
                assert_loaded(&ledger, true);
            }
        }
    }
}
