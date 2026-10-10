// SPDX-License-Identifier: MPL-2.0

//! D013 compile/host experiment on the real private owner. No AMD calls, COM
//! objects, GPU work or public movable API. The only Send override is below.

use super::{NativeContextOwner, RuntimeRetention, tests::fixture};
use crate::{
    error::{Error, InvariantViolation, Operation},
    runtime::RuntimeState,
};
#[cfg(all(windows, feature = "dx12"))]
use std::path::PathBuf;
#[cfg(not(all(windows, feature = "dx12")))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    ptr::null_mut,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

#[derive(Clone, Debug)]
struct Event {
    context: u32,
    kind: u32,
    thread: ThreadId,
    library_live: bool,
    coordination_held: Option<bool>,
}

struct Ledger {
    events: Mutex<Vec<Event>>,
    coordination: Mutex<Option<Arc<Mutex<()>>>>,
    #[cfg(all(windows, feature = "dx12"))]
    path: PathBuf,
    #[cfg(not(all(windows, feature = "dx12")))]
    released: Arc<AtomicBool>,
}

impl Ledger {
    fn library_live(&self) -> bool {
        #[cfg(all(windows, feature = "dx12"))]
        {
            // The handle acquired by this observation is immediately released.
            libloading::os::windows::Library::open_already_loaded(&self.path).is_ok()
        }
        #[cfg(not(all(windows, feature = "dx12")))]
        {
            !self.released.load(Ordering::SeqCst)
        }
    }

    fn record(&self, context: u32, kind: u32) {
        // No assert or panic crosses the fixture callback or dependency Drop.
        let coordination_held = self
            .coordination
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(|gate| matches!(gate.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
        self.events
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Event {
                context,
                kind,
                thread: thread::current().id(),
                library_live: self.library_live(),
                coordination_held,
            });
    }

    fn snapshot(&self) -> Vec<Event> {
        self.events
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    fn expect(&self, expected: &[(u32, u32)]) {
        let events = self.snapshot();
        let actual: Vec<_> = events.iter().map(|e| (e.context, e.kind)).collect();
        assert_eq!(actual, expected);
        assert!(events.iter().all(|e| e.library_live));
    }
}

struct CallbackData {
    ledger: Arc<Ledger>,
    context: u32,
}

extern "C" fn event(data: *mut c_void, kind: u32) {
    // SAFETY: FixtureState owns this stable allocation throughout every call.
    // Calls are serialized by the harness; the ledger is independently locked.
    let callback = unsafe { &*data.cast::<CallbackData>() };
    callback.ledger.record(callback.context, kind);
}

struct FixtureState {
    input: Box<fixture::Input>,
    callback: Box<CallbackData>,
}

impl Drop for FixtureState {
    fn drop(&mut self) {
        self.callback.ledger.record(self.callback.context, 3);
    }
}

struct DeviceSpy(Arc<Ledger>, u32);

impl Drop for DeviceSpy {
    fn drop(&mut self) {
        self.0.record(self.1, 4);
    }
}

type Owner<R> = NativeContextOwner<FixtureState, DeviceSpy, R>;
type ArcOwner = Owner<Arc<RuntimeState>>;

// Public-like probe shapes, private and test-only. The runtime marker excludes
// Sync explicitly; the real core retains its original !Send/!Sync marker.
struct MovableRuntimeProbe {
    state: Arc<RuntimeState>,
    ledger: Arc<Ledger>,
    _not_sync: PhantomData<Cell<()>>,
}

struct MovableOwnerProbe(ArcOwner, bool);

impl MovableOwnerProbe {
    fn destroy(self) -> Result<(), Error> {
        self.0.destroy()
    }

    fn handle_address(&self) -> usize {
        self.0.live.as_ref().unwrap().handle as usize
    }
}

// SAFETY: This concrete owner holds only an Arc-retained test library, boxed
// fixture input/callback storage, and Arc/Mutex or atomic observers. Its handle
// points into its own retained Box, whose address is stable across moves. The
// reused fixture has no thread-local state, COM or GPU obligations. Joins order
// every call and cleanup. This does NOT assert that an AMD context is movable.
// The additional bool is dispatch eligibility, owned and transferred with the
// context. Dispatch uses only stack-local fixture descriptors, not COM objects.
unsafe impl Send for MovableOwnerProbe {}

fn fixture_state() -> (RuntimeState, Arc<Ledger>) {
    let (library, path) = super::tests::library();
    let ledger = Arc::new(Ledger {
        events: Mutex::new(Vec::new()),
        coordination: Mutex::new(None),
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
    (RuntimeState::new(library), ledger)
}

impl MovableRuntimeProbe {
    fn new() -> Self {
        let (state, ledger) = fixture_state();
        Self {
            state: Arc::new(state),
            ledger,
            _not_sync: PhantomData,
        }
    }

    fn create(
        &self,
        context: u32,
        create_code: u32,
        destroy_code: u32,
        null: bool,
    ) -> Result<MovableOwnerProbe, Error> {
        create_on(
            &self.state,
            &self.ledger,
            context,
            create_code,
            destroy_code,
            null,
        )
        .map(|owner| MovableOwnerProbe(owner, false))
    }
}

fn create_on<R: RuntimeRetention + Clone>(
    runtime: &R,
    ledger: &Arc<Ledger>,
    context: u32,
    create_code: u32,
    destroy_code: u32,
    null: bool,
) -> Result<Owner<R>, Error> {
    let mut callback = Box::new(CallbackData {
        ledger: ledger.clone(),
        context,
    });
    let mut state = FixtureState {
        input: Box::new(fixture::Input {
            tag: 0,
            next: null_mut(),
            create_code,
            destroy_code,
            null_create: null.into(),
            clear_destroy: 0,
            event,
            data: (&raw mut *callback).cast(),
        }),
        callback,
    };
    let root = (&raw mut *state.input).cast();
    // SAFETY: The owned boxes/Arcs retain all fixture pointers through cleanup
    // or exceptional retention. No fake native resource is passed to AMD.
    unsafe {
        Owner::create(
            state,
            DeviceSpy(ledger.clone(), context),
            runtime.clone(),
            root,
        )
    }
}

fn on_thread<T: Send + 'static>(name: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new()
        .name(name.into())
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

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

assert_not_impl!(Rc<RuntimeState>, Send);
assert_not_impl!(Rc<RuntimeState>, Sync);
assert_not_impl!(*mut c_void, Send);
assert_not_impl!(*mut c_void, Sync);
assert_not_impl!(FixtureState, Send);
assert_not_impl!(FixtureState, Sync);
assert_not_impl!(ArcOwner, Send);
assert_not_impl!(ArcOwner, Sync);
assert_not_impl!(MovableRuntimeProbe, Sync);
assert_not_impl!(MovableOwnerProbe, Sync);

#[cfg(all(windows, feature = "dx12"))]
assert_not_impl!(crate::runtime::Runtime, Send);
#[cfg(all(windows, feature = "dx12"))]
assert_not_impl!(crate::runtime::Runtime, Sync);
#[cfg(all(windows, feature = "dx12"))]
assert_not_impl!(crate::upscaler::Upscaler, Send);
#[cfg(all(windows, feature = "dx12"))]
assert_not_impl!(crate::upscaler::Upscaler, Sync);

#[test]
fn trait_matrix() {
    fn send<T: Send>() {}
    fn send_sync<T: Send + Sync>() {}
    send_sync::<RuntimeState>();
    send_sync::<Arc<RuntimeState>>();
    send::<MovableRuntimeProbe>();
    send::<MovableOwnerProbe>();
    #[cfg(all(windows, feature = "dx12"))]
    send_sync::<crate::upscaler::ID3D12Device>();
}

fn cleanup_control<R: RuntimeRetention + Clone>(make: impl FnOnce(RuntimeState) -> R) {
    let (state, ledger) = fixture_state();
    let runtime = make(state);
    let first = create_on(&runtime, &ledger, 1, 0, 0, false).unwrap();
    let second = create_on(&runtime, &ledger, 2, 0, 0, false).unwrap();
    drop(runtime);
    first.destroy().unwrap();
    assert!(ledger.library_live());
    drop(second);
    ledger.expect(&[
        (1, 1),
        (2, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 2),
        (2, 3),
        (2, 4),
    ]);
    assert!(!ledger.library_live());
    assert!(
        ledger
            .snapshot()
            .iter()
            .all(|e| e.thread == thread::current().id())
    );
}

#[test]
fn rc_and_arc_single_thread_controls() {
    cleanup_control(Rc::new);
    cleanup_control(Arc::new);
}

#[test]
fn runtime_and_siblings_cross_threads_with_stable_handles() {
    let runtime = MovableRuntimeProbe::new();
    let weak = Arc::downgrade(&runtime.state);
    let ledger = runtime.ledger.clone();
    let (runtime, first, second, create_thread, address) = on_thread("d013-create", move || {
        let first = runtime.create(1, 0, 0, false).unwrap();
        let second = runtime.create(2, 0, 0, false).unwrap();
        let address = first.handle_address();
        (runtime, first, second, thread::current().id(), address)
    });
    let (second, destroy_thread) = on_thread("d013-explicit-destroy", move || {
        assert_eq!(first.handle_address(), address);
        drop(runtime);
        first.destroy().unwrap();
        (second, thread::current().id())
    });
    assert!(weak.upgrade().is_some());
    let drop_thread = on_thread("d013-final-drop", move || {
        drop(second);
        thread::current().id()
    });
    assert!(weak.upgrade().is_none());
    assert!(!ledger.library_live());
    ledger.expect(&[
        (1, 1),
        (2, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 2),
        (2, 3),
        (2, 4),
    ]);
    for event in ledger.snapshot() {
        let expected = if event.kind == 1 {
            create_thread
        } else if event.context == 1 {
            destroy_thread
        } else {
            drop_thread
        };
        assert_eq!(event.thread, expected);
    }
}

#[test]
fn returned_create_error_releases_inputs_without_destroy_on_either_thread() {
    for threaded in [false, true] {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let action = move || {
            let result = runtime.create(1, u32::MAX, 0, false);
            assert!(matches!(
                result,
                Err(Error::Native {
                    operation: Operation::Create,
                    code: u32::MAX
                })
            ));
            drop(runtime);
            thread::current().id()
        };
        let actor = if threaded {
            on_thread("d013-create-error", action)
        } else {
            action()
        };
        ledger.expect(&[(1, 1), (1, 3), (1, 4)]);
        assert!(ledger.snapshot().iter().all(|e| e.thread == actor));
        assert!(weak.upgrade().is_none());
        assert!(!ledger.library_live());
    }
}

#[test]
fn null_success_retains_dependencies_on_either_thread() {
    for threaded in [false, true] {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let action = move || {
            assert!(matches!(
                runtime.create(1, 0, 0, true),
                Err(Error::NativeInvariantViolation(
                    InvariantViolation::NullContextOnSuccess
                ))
            ));
            drop(runtime);
        };
        if threaded {
            on_thread("d013-null-success", action)
        } else {
            action()
        }
        ledger.expect(&[(1, 1)]);
        assert!(weak.upgrade().is_some());
        assert!(ledger.library_live());
        // D005 intentionally leaks the complete bundle; no cleanup is invented.
    }
}

#[test]
fn destroy_error_is_terminal_and_retains_dependencies_on_either_thread() {
    for threaded in [false, true] {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let owner = runtime.create(1, 0, u32::MAX, false).unwrap();
        drop(runtime);
        let action = move || {
            assert!(matches!(
                owner.destroy(),
                Err(Error::Native {
                    operation: Operation::Destroy,
                    code: u32::MAX
                })
            ));
            thread::current().id()
        };
        let actor = if threaded {
            on_thread("d013-destroy-error", action)
        } else {
            action()
        };
        ledger.expect(&[(1, 1), (1, 2)]);
        assert_eq!(ledger.snapshot()[1].thread, actor);
        assert!(weak.upgrade().is_some());
        assert!(ledger.library_live());
    }
}

#[test]
fn unwinding_cleans_up_on_either_thread() {
    for threaded in [false, true] {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let owner = runtime.create(1, 0, 0, false).unwrap();
        drop(runtime);
        let action = move || {
            let _owner = owner;
            panic!("D013 intentional fixture unwind");
        };
        let actor = if threaded {
            let worker = thread::Builder::new()
                .name("d013-unwind".into())
                .spawn(action)
                .unwrap();
            let actor = worker.thread().id();
            assert!(worker.join().is_err());
            actor
        } else {
            assert!(std::panic::catch_unwind(action).is_err());
            thread::current().id()
        };
        ledger.expect(&[(1, 1), (1, 2), (1, 3), (1, 4)]);
        assert!(ledger.snapshot()[1..].iter().all(|e| e.thread == actor));
        assert!(weak.upgrade().is_none());
        assert!(!ledger.library_live());
    }
}

#[test]
fn runtime_without_context_can_release_on_another_thread() {
    for threaded in [false, true] {
        let runtime = MovableRuntimeProbe::new();
        let ledger = runtime.ledger.clone();
        let weak = Arc::downgrade(&runtime.state);
        if threaded {
            on_thread("d013-runtime-drop", move || drop(runtime));
        } else {
            drop(runtime);
        }
        assert!(weak.upgrade().is_none());
        assert!(!ledger.library_live());
        ledger.expect(&[]);
    }
}

#[path = "d013_transfer.rs"]
mod transfer;
