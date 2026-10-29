// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::upscaler::PreExposure;
use std::cell::Cell;

thread_local! {
    static CAPTURED: Cell<Option<u32>> = const { Cell::new(None) };
}

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: The controlled public dispatch supplies a complete live descriptor.
    // Only the scalar's bits are copied; no borrowed resource pointer is retained.
    let descriptor = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    CAPTURED.set(Some(descriptor.preExposure.to_bits()));
}

fn check_mapping<'a>(
    observer: &libloading::Library,
    scene: &'a Scene,
    mut dispatch: impl FnMut(UpscaleTestInputs<'a>) -> Result<(), Error>,
) {
    // SAFETY: Exact controlled-fixture ABI; its callback observes complete
    // descriptors synchronously and records no GPU work.
    unsafe {
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
    for factor in [None, Some(0.25), Some(4.0), Some(1.0)] {
        let mut frame = scene.input();
        assert_eq!(frame.pre_exposure(), PreExposure::default());
        if let Some(value) = factor {
            frame.set_pre_exposure(PreExposure::new(2.0).unwrap());
            frame.set_pre_exposure(PreExposure::new(value).unwrap());
        }
        assert_eq!(frame.pre_exposure().value(), factor.unwrap_or(1.0));
        assert!(frame.reset());
        assert!(matches!(frame.sharpening(), Sharpening::Disabled));
        let counts = scene.reference_counts();
        CAPTURED.set(None);
        assert_result!(dispatch(frame), Ok(()));
        assert_eq!(scene.reference_counts(), counts);
        assert_eq!(CAPTURED.take(), Some(factor.unwrap_or(1.0).to_bits()));
    }
}

#[test]
fn public_dispatch_forwards_default_and_selected_pre_exposure() {
    let (mut runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
        .create(&sizes())
        .unwrap();
    check_mapping(&observer, &scene, |frame| {
        // SAFETY: Controlled COM/runtime fixtures record no GPU commands.
        unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) }
    });
    context.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}

#[cfg(feature = "movable")]
#[test]
fn movable_dispatch_forwards_default_and_selected_pre_exposure() {
    let (mut runtime, observer) = fixture_movable_runtime();
    let scene = Scene::new();
    let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
        .create(&sizes())
        .unwrap();
    check_mapping(&observer, &scene, |frame| {
        // SAFETY: Controlled COM/runtime fixtures record no GPU commands;
        // every native operation is serialized on this test's thread.
        unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) }
    });
    context.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}
