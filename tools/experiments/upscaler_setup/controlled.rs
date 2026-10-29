// SPDX-License-Identifier: MPL-2.0

//! New entry points over production ownership/inspection with controlled DLL/COM equipment.
use super::provider_selection::{spike_available, spike_live_id, spike_register, spike_seen};
use super::{com::Scene, *};
use crate::composition_spike::dx12::{BindingRole, ExposureInput, UpscaleResources as Bindings};
use crate::composition_spike::{Config, FrameParameters, HistoryAction, MotionVectorScale};
use crate::error::{InvariantViolation, QueryValidation};
use crate::runtime::setup_spike as api;
use std::cell::Cell;
use std::rc::Rc;
#[cfg(feature = "movable")]
use std::sync::Arc;

fn config() -> Config {
    Config::new(
        Dimensions::new(1280, 720).unwrap(),
        Dimensions::new(1920, 1080).unwrap(),
    )
}
fn frame() -> FrameParameters {
    FrameParameters::new(
        JitterOffsetPixels::new(0.0, 0.0).unwrap(),
        FrameTimeMillis::new(16.0).unwrap(),
        CameraParameters::new(0.1, 1000.0, 1.0, 1.0).unwrap(),
        HistoryAction::Reset,
    )
}
fn bindings(scene: &Scene) -> Bindings<'_> {
    Bindings::builder()
        .color(&scene.resources[0])
        .depth(&scene.resources[1])
        .motion_vectors(&scene.resources[2])
        .exposure(ExposureInput::Texture(&scene.resources[3]))
        .output(&scene.resources[4])
        .build()
        .unwrap()
}
thread_local! { static SCALE: Cell<([u32;2],bool)> = const { Cell::new(([0;2],false)) }; }
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Registered only for a complete synchronous upscale descriptor.
    let desc = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    SCALE.set((
        [
            desc.motionVectorScale.x.to_bits(),
            desc.motionVectorScale.y.to_bits(),
        ],
        desc.reset,
    ));
}

macro_rules! lifecycle {
    ($runtime:ty, $retention:ident) => {{
        let (path, observer) = fixture_library();
        let (device, fake) = fake_device();
        // SAFETY: Exact controlled DLL; no hidden activity, GPU work or affinity.
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let weak = $retention::downgrade(&runtime.state);
        let a;
        let b;
        {
            let before = fixture_snapshot(&observer).creates;
            // SAFETY: Fixture consumes descriptors synchronously under serialized use.
            let mut setup = unsafe { runtime.upscaler_dx12(&device) };
            assert_eq!(setup.selection(), api::ProviderSelection::Default);
            assert_eq!(fixture_snapshot(&observer).creates, before);
            a = setup.create(&config()).unwrap();
            b = setup.create(&config()).unwrap();
        }
        drop(runtime);
        drop(device);
        assert!(weak.upgrade().is_some());
        assert_eq!(fake.references.get(), 2);
        a.destroy().unwrap();
        assert!(weak.upgrade().is_some());
        assert_eq!(fake.references.get(), 1);
        drop(b);
        assert!(weak.upgrade().is_none());
        assert_eq!(fake.references.get(), 0);
        assert_eq!(fixture_snapshot(&observer).destroys, 2);
    }};
}

macro_rules! selection {
    ($runtime:ty) => {{
        let (path, observer) = fixture_library();
        let (device, _) = fake_device();
        spike_register(&observer, &device);
        // SAFETY: Controlled fixture accepts exactly these descriptor domains.
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let mut setup = unsafe { runtime.upscaler_dx12(&device) };
        let id = setup.providers().unwrap()[0].id();
        let mut setup = setup.with_provider(api::ProviderSelection::Explicit(id));
        spike_seen();
        assert_eq!(
            setup.ratio(crate::query::QualityMode::Quality).unwrap(),
            2.0
        );
        let seen = spike_seen();
        assert_eq!(
            seen.iter()
                .filter(|x| x.0 == FFX_API_QUERY_DESC_TYPE_GET_VERSIONS)
                .count(),
            2
        );
        assert_eq!(seen.last().unwrap().1, Some(91));
        setup.estimate_memory(&config()).unwrap();
        assert_eq!(spike_seen().last().unwrap().1, Some(91));
        let context = setup.create(&config()).unwrap();
        assert!(
            spike_seen()
                .iter()
                .any(|x| x.0 == FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION && x.1 == Some(91))
        );
        context.destroy().unwrap();
        let before = fixture_snapshot(&observer).creates;
        spike_available(92);
        spike_seen();
        let error = setup
            .jitter_offset(-1, crate::query::JitterPhaseCount::new(4).unwrap())
            .unwrap_err();
        assert_eq!(
            error,
            Error::InvalidQuery {
                reason: QueryValidation::JitterIndex
            }
        );
        assert!(spike_seen().is_empty());
        assert!(matches!(
            setup.ratio(crate::query::QualityMode::Quality),
            Err(Error::ProviderNotEnumerated { .. })
        ));
        assert_eq!(fixture_snapshot(&observer).creates, before);
        spike_available(91);
        spike_live_id(92);
        assert!(matches!(
            setup.create(&config()),
            Err(Error::NativeInvariantViolation(
                InvariantViolation::ProviderSelectionMismatch { .. }
            ))
        ));
        assert_eq!(fixture_snapshot(&observer).destroys, 2);
    }};
}

macro_rules! recording {
    ($runtime:ty) => {{
        let (path, observer) = fixture_library();
        let scene = Scene::new();
        // SAFETY: Controlled DLL and COM implementations record no GPU work.
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&config())
            .unwrap();
        drop(runtime);
        let resources = bindings(&scene);
        let mut frame = frame();
        let references = scene.reference_counts();
        assert!(
            context
                .validate_upscale_inputs(&scene.list, &resources, &frame)
                .is_ok()
        );
        let original = scene.resource_object(0).desc.get();
        let mut wrong = original;
        wrong.Width += 1;
        scene.resource_object(0).desc.set(wrong);
        assert!(unsafe { context.record(&scene.list, &resources, &frame) }.is_err());
        assert_eq!(context.usability(), api::ContextUsability::Usable);
        assert_eq!(scene.reference_counts(), references);
        scene.resource_object(0).desc.set(original);
        // SAFETY: Exact paired fixture observer ABI; captures scalar bits only.
        unsafe {
            observer
                .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                    c"fixture_set_dispatch_observer",
                )
                .unwrap()(Some(capture));
        }
        let scale = crate::upscaler::NativeMotionVectorScale::new(-0.0, f32::from_bits(1)).unwrap();
        frame.set_motion_scale(MotionVectorScale::Native(scale));
        unsafe { context.record(&scene.list, &resources, &frame) }.unwrap();
        assert_eq!(SCALE.get(), ([(-0.0f32).to_bits(), 1], true));
        frame.set_motion_scale(MotionVectorScale::Pixels);
        frame.set_history_action(HistoryAction::Continue);
        unsafe { context.record(&scene.list, &resources, &frame) }.unwrap();
        assert_eq!(SCALE.get(), ([1.0f32.to_bits(); 2], false));
        frame.set_motion_scale(MotionVectorScale::Uv);
        unsafe { context.record(&scene.list, &resources, &frame) }.unwrap();
        assert_eq!(SCALE.get().0, [1280.0f32.to_bits(), 720.0f32.to_bits()]);
        context.mark_recording_abandoned();
        assert_eq!(
            context.usability(),
            api::ContextUsability::Unusable(api::TerminalCause::RecordingAbandoned)
        );
        assert!(matches!(
            context.provider(),
            Err(api::SpikeError::Unusable(_))
        ));
        assert!(matches!(
            context.validate_upscale_inputs(&scene.list, &resources, &frame),
            Err(api::SpikeError::Unusable(_))
        ));
        assert!(matches!(
            unsafe { context.record(&scene.list, &resources, &frame) },
            Err(api::SpikeError::Unusable(_))
        ));
        context.destroy().unwrap();
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
    }};
}

macro_rules! failure {
    ($runtime:ty) => {{
        for (create, destroy, null) in [(0xf1234567, 0, 0), (0, 0, 1), (0, 0xf1234567, 0)] {
            let (path, observer) = fixture_library();
            let (device, fake) = fake_device();
            // SAFETY: Controlled DLL; failure outcomes preserve the tested storage rules.
            let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
            fixture_set(&observer, create, destroy, null);
            let result = unsafe { runtime.upscaler_dx12(&device) }.create(&config());
            if create != 0 {
                assert!(matches!(
                    result,
                    Err(Error::Native {
                        code: 0xf1234567,
                        ..
                    })
                ));
            } else if null != 0 {
                assert!(matches!(result, Err(Error::NativeInvariantViolation(_))));
            } else {
                assert!(matches!(
                    result.unwrap().destroy(),
                    Err(Error::Native {
                        code: 0xf1234567,
                        ..
                    })
                ));
            }
            drop(runtime);
            drop(device);
            assert_eq!(fake.references.get(), u32::from(create == 0));
            assert_eq!(
                fixture_snapshot(&observer).destroys,
                u32::from(destroy != 0)
            );
        }
    }};
}

macro_rules! poison {
    ($runtime:ty) => {{
        let (path, observer) = fixture_library();
        let scene = Scene::new();
        // SAFETY: Controlled DLL/COM; no GPU work or external callback target.
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&config())
            .unwrap();
        unsafe {
            observer
                .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                .unwrap()(0xf1234567);
        }
        let resources = bindings(&scene);
        assert!(unsafe { context.record(&scene.list, &resources, &frame()) }.is_err());
        let original = context.usability();
        assert_eq!(
            original,
            api::ContextUsability::Unusable(api::TerminalCause::Native {
                operation: api::NativeOperation::Upscale,
                code: 0xf1234567
            })
        );
        context.mark_recording_abandoned();
        assert_eq!(context.usability(), original);
        assert!(matches!(
            context.request_tuning(
                TuningParameter::VelocityFactor,
                TuningValue::Value(f32::NAN)
            ),
            Err(api::SpikeError::Unusable(_))
        ));
        context.destroy().unwrap();
    }};
}

#[test]
fn setup_spike_local_contexts_outlive_setup_and_runtime() {
    lifecycle!(api::Runtime, Rc);
}
#[test]
fn setup_spike_local_selection_is_fresh_and_identity_checked() {
    selection!(api::Runtime);
}
#[test]
fn setup_spike_local_recording_and_abandonment() {
    recording!(api::Runtime);
}
#[test]
fn setup_spike_local_failure_retention() {
    failure!(api::Runtime);
}
#[test]
fn setup_spike_local_first_terminal_cause() {
    poison!(api::Runtime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_contexts_outlive_setup_and_runtime() {
    lifecycle!(api::MovableRuntime, Arc);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_selection_is_fresh_and_identity_checked() {
    selection!(api::MovableRuntime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_recording_and_abandonment() {
    recording!(api::MovableRuntime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_failure_retention() {
    failure!(api::MovableRuntime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_first_terminal_cause() {
    poison!(api::MovableRuntime);
}

#[test]
fn setup_spike_builder_requires_explicit_roles_and_exposure() {
    let scene = Scene::new();
    let resources = &scene.resources;
    assert_eq!(Bindings::builder().build().unwrap_err(), BindingRole::Color);
    assert_eq!(
        Bindings::builder()
            .color(&resources[0])
            .depth(&resources[1])
            .motion_vectors(&resources[2])
            .output(&resources[4])
            .build()
            .unwrap_err(),
        BindingRole::Exposure
    );
    assert!(
        Bindings::builder()
            .color(&resources[0])
            .depth(&resources[1])
            .motion_vectors(&resources[2])
            .exposure(ExposureInput::Automatic)
            .output(&resources[4])
            .build()
            .is_ok()
    );
}

unsafe extern "C" fn message(_: u32, _: *const u16) {}
unsafe extern "C" fn fail_configure(
    _: *mut *mut std::ffi::c_void,
    _: *const std::ffi::c_void,
) -> u32 {
    0xf1234567
}
macro_rules! callbacks_and_configure {
    ($runtime:ty) => {{
        let (path, observer) = fixture_library();
        let (device, fake) = fake_device();
        // SAFETY: Controlled DLL; callback target is a static no-op and all
        // native operations/cleanup are serialized with no hidden activity.
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let mut callbacks = crate::upscaler::dx12::interop::CreationCallbacks::default();
        callbacks.set_message_callback(Some(message));
        fixture_set(&observer, 0xf1234567, 0, 0);
        assert!(
            unsafe {
                runtime
                    .upscaler_dx12(&device)
                    .create_with_callbacks(&config(), callbacks)
            }
            .is_err()
        );
        drop(runtime);
        drop(device);
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 0);

        let (path, observer) = fixture_library();
        let (device, _) = fake_device();
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let mut context = unsafe { runtime.upscaler_dx12(&device) }
            .create(&config())
            .unwrap();
        // SAFETY: Exact controlled observer signature; it returns an ordinary
        // failure without retaining descriptor addresses or invoking GPU work.
        unsafe {
            observer
                .get::<extern "C" fn(
                    Option<
                        unsafe extern "C" fn(
                            *mut *mut std::ffi::c_void,
                            *const std::ffi::c_void,
                        ) -> u32,
                    >,
                )>(c"fixture_set_configure_observer")
                .unwrap()(Some(fail_configure));
        }
        assert!(
            context
                .request_tuning(TuningParameter::VelocityFactor, TuningValue::Value(0.5))
                .is_err()
        );
        assert_eq!(
            context.usability(),
            api::ContextUsability::Unusable(api::TerminalCause::Native {
                operation: api::NativeOperation::Tuning,
                code: 0xf1234567
            })
        );
        context.mark_recording_abandoned();
        assert!(matches!(
            context.usability(),
            api::ContextUsability::Unusable(api::TerminalCause::Native { .. })
        ));
        context.destroy().unwrap();
    }};
}
#[test]
fn setup_spike_local_callbacks_and_configure_failure() {
    callbacks_and_configure!(api::Runtime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_callbacks_and_configure_failure() {
    callbacks_and_configure!(api::MovableRuntime);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_owner_destroys_on_destination_thread() {
    let (path, observer) = fixture_library();
    let (device, fake) = fake_device();
    // SAFETY: Controlled DLL has no affinity/workers/GPU work; all access to
    // its COM reference counter is serialized before and after thread::join.
    let mut runtime = unsafe { api::MovableRuntime::load(&path) }.unwrap();
    let context = unsafe { runtime.upscaler_dx12(&device) }
        .create(&config())
        .unwrap();
    drop(runtime);
    std::thread::spawn(move || context.destroy())
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(fake.references.get(), 1);
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}

macro_rules! loader_and_unwind {
    ($runtime:ty, $retention:ident) => {{
        let (path, observer) = fixture_library();
        // SAFETY: Controlled DLL, no native worker or thread affinity.
        let runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let weak = $retention::downgrade(&runtime.state);
        drop(runtime);
        assert!(weak.upgrade().is_none());
        assert_eq!(fixture_snapshot(&observer).creates, 0);
        assert_eq!(fixture_snapshot(&observer).destroys, 0);
        // SAFETY: Exact fixture export, only reads a counter.
        assert_eq!(
            unsafe {
                observer
                    .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
                    .unwrap()()
            },
            0
        );

        let (device, fake) = fake_device();
        let mut runtime = unsafe { <$runtime>::load(&path) }.unwrap();
        let weak = $retention::downgrade(&runtime.state);
        let context = unsafe { runtime.upscaler_dx12(&device) }
            .create(&config())
            .unwrap();
        drop(runtime);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _held = context;
            panic!("intentional controlled unwind");
        }));
        assert!(result.is_err());
        assert!(weak.upgrade().is_none());
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
    }};
}
#[test]
fn setup_spike_local_loader_only_and_unwind_cleanup() {
    loader_and_unwind!(api::Runtime, Rc);
}
#[cfg(feature = "movable")]
#[test]
fn setup_spike_movable_loader_only_and_unwind_cleanup() {
    loader_and_unwind!(api::MovableRuntime, Arc);
}
