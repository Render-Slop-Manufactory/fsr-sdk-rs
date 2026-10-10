// SPDX-License-Identifier: MPL-2.0

//! Controlled transfer/error spike; shares both production terminal ownership
//! and dispatch orchestration. Never transfers the thread-confined fake COM.
use super::*;
use crate::error::{DispatchValidation, InspectionObject, InspectionOperation};
use fsr_sdk_sys::api::FFX_API_RETURN_ERROR;
use std::sync::mpsc;

#[path = "../../tests/fixtures/child.rs"]
mod child;

impl MovableOwnerProbe {
    fn assert_pointers(&self, expected: usize) {
        let state = &self.0.live.as_ref().unwrap().dependencies.state;
        assert_eq!(self.handle_address(), expected);
        assert_eq!(expected, std::ptr::from_ref(&*state.input) as usize);
        assert_eq!(
            state.input.data,
            std::ptr::from_ref(&*state.callback).cast_mut().cast()
        );
    }

    fn dispatch(
        &mut self,
        inspect: impl FnOnce(&Self) -> Result<fixture::Dispatch, Error>,
    ) -> Result<(), Error> {
        let mut poisoned = self.1;
        let result =
            crate::dispatch_policy::dispatch(&mut poisoned, self, inspect, |this, desc| {
                // SAFETY: This concrete test bundle owns its stable fixture handle;
                // the fixture reads only this stack-local protocol, with no GPU work.
                unsafe { this.0.dispatch(std::ptr::from_ref(desc).cast()) }
            });
        self.1 = poisoned;
        result
    }

    fn attempt(&mut self, code: u32) -> Result<(), Error> {
        self.dispatch(|this| {
            let state = &this.0.live.as_ref().unwrap().dependencies.state;
            // Pointer comparisons happen outside callbacks, before entering FFI.
            assert_eq!(
                this.handle_address(),
                std::ptr::from_ref(&*state.input) as usize
            );
            assert_eq!(
                state.input.data,
                std::ptr::from_ref(&*state.callback).cast_mut().cast()
            );
            state.callback.ledger.record(state.callback.context, 6);
            Ok(fixture::Dispatch {
                tag: 0xd013,
                next: null_mut(),
                code,
            })
        })
    }

    fn reject_poison(&mut self) {
        // Inspection must not run. A failure here is a Rust-only assertion.
        assert_eq!(
            self.dispatch(|_| panic!("poison inspected")),
            Err(Error::DispatchPoisoned)
        );
    }
}

type Setup = (
    MovableOwnerProbe,
    std::sync::Weak<RuntimeState>,
    Arc<Ledger>,
    usize,
    ThreadId,
);

fn create(threaded: bool, destroy_code: u32) -> Setup {
    let action = move || {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let owner = runtime.create(1, 0, destroy_code, false).unwrap();
        let pointer = owner.handle_address();
        drop(runtime);
        (owner, weak, ledger, pointer, thread::current().id())
    };
    if threaded {
        on_thread("d013-A", action)
    } else {
        action()
    }
}

fn act<T: Send + 'static>(
    threaded: bool,
    name: &str,
    action: impl FnOnce() -> T + Send + 'static,
) -> T {
    if threaded {
        on_thread(name, action)
    } else {
        action()
    }
}

fn native_error(operation: Operation, code: u32) -> Result<(), Error> {
    Err(Error::Native { operation, code })
}

fn expect(ledger: &Ledger, expected: &[(u32, u32, ThreadId)]) {
    ledger.expect(&expected.iter().map(|&(c, k, _)| (c, k)).collect::<Vec<_>>());
    let events = ledger.snapshot();
    for (event, &(_, _, actor)) in events.iter().zip(expected) {
        assert_eq!(event.thread, actor);
    }
    // IDs/addresses are asserted, then represented by semantic labels only.
    let mut actors = Vec::new();
    for event in events {
        let actor = match actors.iter().position(|id| *id == event.thread) {
            Some(index) => index,
            None => {
                actors.push(event.thread);
                actors.len() - 1
            }
        };
        let kind = match event.kind {
            1 => "create",
            2 => "destroy",
            3 => "state-release",
            4 => "device-release",
            5 => "dispatch",
            6 => "inspect",
            _ => "invalid",
        };
        println!(
            "event context={} kind={kind} actor={actor} library=live coordination_held={:?}",
            event.context, event.coordination_held
        );
    }
}

fn poison(threaded: bool, code: u32, destroy_failure: bool, explicit: bool) {
    let destroy_code = if destroy_failure { u32::MAX } else { 0 };
    let (mut owner, weak, ledger, pointer, a) = create(threaded, destroy_code);
    let (owner, b) = act(threaded, "d013-B", move || {
        owner.assert_pointers(pointer);
        assert_eq!(owner.attempt(code), native_error(Operation::Dispatch, code));
        assert!(owner.1);
        (owner, thread::current().id())
    });
    let c = act(threaded, "d013-C", move || {
        let mut owner = owner;
        owner.assert_pointers(pointer);
        owner.reject_poison();
        if explicit {
            assert_eq!(
                owner.destroy(),
                if destroy_failure {
                    native_error(Operation::Destroy, u32::MAX)
                } else {
                    Ok(())
                }
            );
        } else {
            drop(owner);
        }
        thread::current().id()
    });
    let mut expected = vec![(1, 1, a), (1, 6, b), (1, 5, b), (1, 2, c)];
    if !destroy_failure {
        expected.extend([(1, 3, c), (1, 4, c)]);
    }
    expect(&ledger, &expected);
    assert_eq!(weak.upgrade().is_some(), destroy_failure);
    assert_eq!(ledger.library_live(), destroy_failure);
    if destroy_failure {
        // Retained state owns CallbackData -> ledger; DeviceSpy owns another Arc.
        assert_eq!(Arc::strong_count(&ledger), 3);
        assert_eq!(weak.strong_count(), 1);
    }
    println!(
        "dispatch_code={code} dispatch_calls=1 destroy_code={destroy_code} destroy_calls=1 inspect_calls=1 pointers=stable retained={destroy_failure}"
    );
}

fn preflight(threaded: bool) {
    let (mut owner, weak, ledger, pointer, a) = create(threaded, 0);
    let b_ledger = ledger.clone();
    let (owner, b) = act(threaded, "d013-B", move || {
        for error in [
            Error::InvalidDispatch {
                reason: DispatchValidation::CommandListType,
            },
            Error::Dx12Inspection {
                object: InspectionObject::CommandList,
                operation: InspectionOperation::GetDevice,
                hresult: 0x8000_4005u32 as i32,
            },
        ] {
            assert_eq!(
                owner.dispatch(|_| {
                    b_ledger.record(1, 6);
                    Err(error)
                }),
                Err(error)
            );
            assert!(!owner.1);
            assert!(!b_ledger.snapshot().iter().any(|e| e.kind == 5));
            println!("preflight={error:?} dispatch_calls=0 poisoned=false");
        }
        (owner, thread::current().id())
    });
    let c = act(threaded, "d013-C", move || {
        let mut owner = owner;
        owner.assert_pointers(pointer);
        assert_eq!(owner.attempt(0), Ok(()));
        assert!(!owner.1);
        owner.destroy().unwrap();
        thread::current().id()
    });
    expect(
        &ledger,
        &[
            (1, 1, a),
            (1, 6, b),
            (1, 6, b),
            (1, 6, c),
            (1, 5, c),
            (1, 2, c),
            (1, 3, c),
            (1, 4, c),
        ],
    );
    assert!(weak.upgrade().is_none());
    assert!(!ledger.library_live());
    println!("recovery_code=0 dispatch_calls=1 destroy_calls=1 pointers=stable retained=false");
}

fn unwind(threaded: bool, poisoned: bool) {
    let action = || {
        let runtime = MovableRuntimeProbe::new();
        let weak = Arc::downgrade(&runtime.state);
        let ledger = runtime.ledger.clone();
        let first = runtime.create(1, 0, 0, false).unwrap();
        let sibling = runtime.create(2, 0, 0, false).unwrap();
        drop(runtime);
        (first, sibling, weak, ledger, thread::current().id())
    };
    let (first, mut sibling, weak, ledger, a) = act(threaded, "d013-A", action);
    let first_pointer = first.handle_address();
    let sibling_pointer = sibling.handle_address();
    // The guard is declared before the context local: reverse local destruction
    // keeps family coordination held throughout context Drop during unwinding.
    let gate = Arc::new(Mutex::new(()));
    *ledger.coordination.lock().unwrap() = Some(gate.clone());
    let destination_gate = gate.clone();
    let (start_tx, start_rx) = mpsc::sync_channel(0);
    let (resume_tx, resume_rx) = mpsc::sync_channel(0);
    let survivor_gate = gate.clone();
    let survivor_ledger = ledger.clone();
    let survivor = thread::Builder::new()
        .name("d013-sibling-C".into())
        .spawn(move || {
            start_tx.send(()).unwrap();
            resume_rx.recv().unwrap();
            let guard = survivor_gate.lock().unwrap_or_else(|p| p.into_inner());
            assert!(survivor_gate.is_poisoned());
            assert!(survivor_ledger.library_live());
            sibling.assert_pointers(sibling_pointer);
            assert_eq!(sibling.attempt(0), Ok(()));
            sibling.destroy().unwrap();
            drop(guard);
            thread::current().id()
        })
        .unwrap();
    start_rx.recv().unwrap();
    let destination = move || {
        let _guard = destination_gate.lock().unwrap_or_else(|p| p.into_inner());
        let mut owner = first;
        owner.assert_pointers(first_pointer);
        if poisoned {
            assert_eq!(
                owner.attempt(FFX_API_RETURN_ERROR),
                native_error(Operation::Dispatch, FFX_API_RETURN_ERROR)
            );
        }
        panic!("D013 declared destination unwind");
    };
    let b = if threaded {
        let worker = thread::Builder::new()
            .name("d013-unwind-B".into())
            .spawn(destination)
            .unwrap();
        let actor = worker.thread().id();
        let panic = worker.join().expect_err("declared panic must occur");
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"D013 declared destination unwind")
        );
        actor
    } else {
        let panic = std::panic::catch_unwind(destination).expect_err("declared panic must occur");
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"D013 declared destination unwind")
        );
        thread::current().id()
    };
    // Join/catch completes cleanup before allowing the surviving sibling calls.
    // A same-thread caught panic also poisons its mutex; recovery is explicit.
    assert!(gate.is_poisoned());
    assert!(weak.upgrade().is_some());
    assert!(ledger.library_live());
    resume_tx.send(()).unwrap();
    let c = survivor.join().unwrap();
    assert!(
        ledger.snapshot()[2..]
            .iter()
            .all(|event| event.coordination_held == Some(true))
    );
    let mut expected = vec![(1, 1, a), (2, 1, a)];
    if poisoned {
        expected.extend([(1, 6, b), (1, 5, b)]);
    }
    expected.extend([
        (1, 2, b),
        (1, 3, b),
        (1, 4, b),
        (2, 6, c),
        (2, 5, c),
        (2, 2, c),
        (2, 3, c),
        (2, 4, c),
    ]);
    expect(&ledger, &expected);
    assert!(weak.upgrade().is_none());
    assert!(!ledger.library_live());
    println!("unwind=declared mutex_poison=recovered sibling_dispatch=0 destroys=2 retained=false");
}

#[test]
fn isolated_case() {
    let Ok(case) = std::env::var("FSR_D013_FAILURE_CASE") else {
        return;
    };
    let threaded = case.starts_with("transfer-");
    match case
        .strip_prefix("transfer-")
        .or_else(|| case.strip_prefix("control-"))
    {
        Some("known-poison") => poison(threaded, FFX_API_RETURN_ERROR, false, true),
        Some("unknown-poison") => poison(threaded, u32::MAX, false, true),
        Some("preflight") => preflight(threaded),
        Some("destroy-explicit") => poison(threaded, FFX_API_RETURN_ERROR, true, true),
        Some("destroy-drop") => poison(threaded, FFX_API_RETURN_ERROR, true, false),
        Some("unwind-ready") => unwind(threaded, false),
        Some("unwind-poisoned") => unwind(threaded, true),
        _ => panic!("undeclared case {case}"),
    }
    println!("CASE PASS {case}");
}

#[test]
fn transfer_failure_cleanup_matrix() {
    for family in ["control", "transfer"] {
        for scenario in [
            "known-poison",
            "unknown-poison",
            "preflight",
            "destroy-explicit",
            "destroy-drop",
            "unwind-ready",
            "unwind-poisoned",
        ] {
            child::run(
                &crate::test_directory::directory("fsr-sdk-d013-child"),
                "context::d013_movable::transfer::isolated_case",
                "FSR_D013_FAILURE_CASE",
                &format!("{family}-{scenario}"),
            );
        }
    }
}
