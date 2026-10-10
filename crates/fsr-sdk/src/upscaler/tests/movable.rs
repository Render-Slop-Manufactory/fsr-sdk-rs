// SPDX-License-Identifier: MPL-2.0

use super::{super::*, com::Scene, support::*};
use crate::error::{DispatchValidation, InvariantViolation, Operation};
use std::sync::Arc;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_COMMAND_LIST_TYPE_COMPUTE, D3D12_COMMAND_LIST_TYPE_DIRECT,
};

fn set_dispatch(observer: &libloading::Library, code: u32) {
    // SAFETY: Exact locally compiled fixture signature; no GPU commands.
    unsafe {
        observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap()(code)
    }
}
fn calls(observer: &libloading::Library) -> u32 {
    // SAFETY: Exact locally compiled fixture observation ABI.
    unsafe {
        observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()()
        .calls
    }
}

#[test]
fn movable_public_dispatch_recovers_preflight_then_preserves_unknown_poison() {
    let (runtime, observer) = fixture_movable_runtime();
    let scene = Scene::new(); // Thread-confined COM inspection fixture stays here.
    let mut upscaler = MovableUpscaler::new(&runtime, &scene.device, sizes()).unwrap();
    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
    // SAFETY: Controlled runtime and COM fixture, no GPU work; calls are serial.
    assert_eq!(
        unsafe { upscaler.dispatch(scene.input()) },
        Err(Error::InvalidDispatch {
            reason: DispatchValidation::CommandListType
        })
    );
    assert_eq!(calls(&observer), 0);
    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_DIRECT);
    // SAFETY: The fixture consumes metadata only and records no GPU work.
    assert_eq!(unsafe { upscaler.dispatch(scene.input()) }, Ok(()));
    set_dispatch(&observer, u32::MAX);
    // SAFETY: Fixture returned-error cleanup has no submitted or recorded GPU work.
    assert_eq!(
        unsafe { upscaler.dispatch(scene.input()) },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX
        })
    );
    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
    // SAFETY: Poison rejects before metadata inspection, not merely before FFI.
    assert_eq!(
        unsafe { upscaler.dispatch(scene.input()) },
        Err(Error::DispatchPoisoned)
    );
    assert_eq!(calls(&observer), 2);
    upscaler.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}

#[test]
fn movable_create_errors_release_inputs_and_ok_null_retains_bundle() {
    for code in [1, u32::MAX] {
        let (runtime, observer) = fixture_movable_runtime();
        let (device, fake) = fake_device();
        fixture_set(&observer, code, 0, 0);
        assert!(
            matches!(MovableUpscaler::new(&runtime, &device, sizes()), Err(Error::Native { operation: Operation::Create, code: actual }) if actual == code)
        );
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 0);
    }
    let (runtime, observer) = fixture_movable_runtime();
    let weak = Arc::downgrade(&runtime.state);
    let (device, fake) = fake_device();
    fixture_set(&observer, 0, 0, 1);
    assert!(matches!(
        MovableUpscaler::new(&runtime, &device, sizes()),
        Err(Error::NativeInvariantViolation(
            InvariantViolation::NullContextOnSuccess
        ))
    ));
    drop(runtime);
    drop(device);
    assert!(weak.upgrade().is_some());
    assert_eq!(fake.references.get(), 1);
    assert_eq!(fixture_snapshot(&observer).destroys, 0);
}

#[test]
fn movable_explicit_and_implicit_destroy_failure_retain_every_dependency_without_retry() {
    for implicit in [false, true] {
        let (runtime, observer) = fixture_movable_runtime();
        let weak = Arc::downgrade(&runtime.state);
        let (device, fake) = fake_device();
        let context = MovableUpscaler::new(&runtime, &device, sizes()).unwrap();
        fixture_set(&observer, 0, u32::MAX, 0);
        if implicit {
            drop(context);
        } else {
            assert_eq!(
                context.destroy(),
                Err(Error::Native {
                    operation: Operation::Destroy,
                    code: u32::MAX
                })
            );
        }
        drop(runtime);
        drop(device);
        assert!(weak.upgrade().is_some());
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
    }
}
