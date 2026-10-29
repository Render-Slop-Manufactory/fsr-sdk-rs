// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::error::{DispatchValidation, ResourceProperty, ResourceRole};
use crate::upscaler::{MotionVectorResolution, MotionVectorScale};
use std::cell::Cell;

type Observation = ([u32; 10], [f32; 2], bool);
thread_local! {
    static CAPTURED: Cell<Option<Observation>> = const { Cell::new(None) };
}

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Controlled synchronous fixture supplies a complete descriptor.
    let d = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    CAPTURED.set(Some((
        [
            d.renderSize.width,
            d.renderSize.height,
            d.upscaleSize.width,
            d.upscaleSize.height,
            unsafe { d.color.description.width.width },
            unsafe { d.color.description.height.height },
            unsafe { d.output.description.width.width },
            unsafe { d.output.description.height.height },
            unsafe { d.motionVectors.description.width.width },
            unsafe { d.motionVectors.description.height.height },
        ],
        [d.motionVectorScale.x, d.motionVectorScale.y],
        d.reset,
    )));
}

fn check_frames<'a>(
    observer: &libloading::Library,
    scene: &'a Scene,
    dynamic: bool,
    resolution: MotionVectorResolution,
    mut dispatch: impl FnMut(UpscaleTestInputs<'a>) -> Result<(), Error>,
) {
    // SAFETY: Exact fixture ABI; observer copies values only during this call.
    unsafe {
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
    let calls = || unsafe {
        observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()()
        .calls
    };
    let before = calls();
    let counts = scene.reference_counts();
    // Invalid frame bounds must reject before even a failing COM inspection.
    scene
        .resource_object(0)
        .query_failure
        .set(Some(com::QUERY_FAILURE));
    for size in [(1281, 720), (1280, 721), (u32::MAX, u32::MAX)]
        .into_iter()
        .chain((!dynamic).then_some((960, 540)))
    {
        let mut frame = scene.input();
        frame.set_active_render_size(Some(Dimensions::new(size.0, size.1).unwrap()));
        assert_result!(
            dispatch(frame),
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveRenderSize
            )))
        );
    }
    scene.resource_object(0).query_failure.set(None);
    assert_eq!(calls(), before);
    assert_eq!(scene.reference_counts(), counts);

    scene
        .resource_object(0)
        .query_failure
        .set(Some(com::QUERY_FAILURE));
    for size in [(1921, 1080), (1920, 1081), (u32::MAX, u32::MAX)]
        .into_iter()
        .chain((!dynamic).then_some((1537, 863)))
    {
        let mut frame = scene.input();
        frame.set_active_output_size(Some(Dimensions::new(size.0, size.1).unwrap()));
        assert_result!(
            dispatch(frame),
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveOutputSize
            )))
        );
    }
    scene.resource_object(0).query_failure.set(None);
    assert_eq!(calls(), before);
    assert_eq!(scene.reference_counts(), counts);

    // Allocations must still match maxima, even when they cover the active image.
    for (index, role) in [
        (0, ResourceRole::Color),
        (1, ResourceRole::Depth),
        (2, ResourceRole::MotionVectors),
        (4, ResourceRole::Output),
    ] {
        let original = scene.resource_object(index).desc.get();
        let mut wrong = original;
        for delta in [-1i64, 1] {
            wrong.Width = (original.Width as i64 + delta) as u64;
            scene.resource_object(index).desc.set(wrong);
            let mut frame = scene.input();
            if dynamic {
                frame.set_active_render_size(Some(Dimensions::new(960, 540).unwrap()));
                frame.set_active_output_size(Some(Dimensions::new(1537, 863).unwrap()));
            }
            assert_result!(
                dispatch(frame),
                Err(Error::Input(crate::error::InputError::Dispatch(
                    DispatchValidation::Resource {
                        role,
                        property: ResourceProperty::Extent
                    }
                )))
            );
            scene.resource_object(index).desc.set(original);
        }
    }
    assert_eq!(calls(), before);
    assert_eq!(scene.reference_counts(), counts);

    let sizes = if dynamic {
        vec![
            None,
            Some((960, 540)),
            Some((1024, 576)),
            Some((1280, 720)),
            None,
        ]
    } else {
        vec![None, Some((1280, 720)), None]
    };
    for (index, size) in sizes.into_iter().enumerate() {
        let outputs = if dynamic {
            vec![
                None,
                Some((1537, 863)),
                Some((1920, 863)),
                Some((1537, 1080)),
                Some((1920, 1080)),
                None,
            ]
        } else {
            vec![None, Some((1920, 1080))]
        };
        for output in outputs {
            for units in [MotionVectorScale::Uv, MotionVectorScale::Pixels] {
                let base = scene.input();
                let mut frame = UpscaleTestInputs::new(
                    base.command_list(),
                    base.resources().clone(),
                    base.jitter(),
                    base.frame_time(),
                    base.camera(),
                    index == 0,
                );
                assert_eq!(frame.active_render_size(), None);
                assert_eq!(frame.active_output_size(), None);
                let size = size.map(|(w, h)| Dimensions::new(w, h).unwrap());
                frame.set_active_render_size(Some(Dimensions::new(1280, 720).unwrap()));
                frame.set_active_render_size(size);
                assert_eq!(frame.active_render_size(), size);
                let output = output.map(|(w, h)| Dimensions::new(w, h).unwrap());
                frame.set_active_output_size(Some(Dimensions::new(1920, 1080).unwrap()));
                frame.set_active_output_size(output);
                assert_eq!(frame.active_output_size(), output);
                frame.set_motion_vector_scale(units);
                let reset = index == 0;
                assert_eq!(frame.reset(), reset);
                CAPTURED.set(None);
                assert_result!(dispatch(frame), Ok(()));
                let active = size.map_or((1280, 720), |s| (s.width(), s.height()));
                let output = output.map_or((1920, 1080), |s| (s.width(), s.height()));
                let motion = if resolution == MotionVectorResolution::Display {
                    output
                } else {
                    active
                };
                let scale = if units == MotionVectorScale::Pixels {
                    [1.0, 1.0]
                } else {
                    [motion.0 as f32, motion.1 as f32]
                };
                assert_eq!(
                    CAPTURED.take(),
                    Some((
                        [
                            active.0,
                            active.1,
                            output.0,
                            output.1,
                            1280,
                            720,
                            1920,
                            1080,
                            if resolution == MotionVectorResolution::Display {
                                1920
                            } else {
                                1280
                            },
                            if resolution == MotionVectorResolution::Display {
                                1080
                            } else {
                                720
                            }
                        ],
                        scale,
                        reset
                    )),
                    "transition={index}"
                );
                assert_eq!(scene.reference_counts(), counts);
            }
        }
    }
}

fn profiles() -> impl Iterator<Item = (bool, MotionVectorResolution)> {
    [false, true].into_iter().flat_map(|dynamic| {
        [
            MotionVectorResolution::Render,
            MotionVectorResolution::Display,
        ]
        .into_iter()
        .map(move |resolution| (dynamic, resolution))
    })
}

fn configure(scene: &Scene, dynamic: bool, resolution: MotionVectorResolution) -> UpscalerOptions {
    let mut options = sizes();
    options.set_dynamic_resolution(dynamic);
    options.set_motion_vector_resolution(resolution);
    if resolution == MotionVectorResolution::Display {
        let mut d = scene.resource_object(2).desc.get();
        d.Width = 1920;
        d.Height = 1080;
        scene.resource_object(2).desc.set(d);
    }
    options
}

#[test]
fn public_active_geometry_mapping_preflight_and_retry() {
    let (mut runtime, observer) = fixture_runtime();
    for (dynamic, resolution) in profiles() {
        let scene = Scene::new();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&configure(&scene, dynamic, resolution))
            .unwrap();
        assert_eq!(
            fixture_snapshot(&observer).flags,
            u32::from(dynamic) * 64 + u32::from(resolution == MotionVectorResolution::Display) * 2
        );
        check_frames(&observer, &scene, dynamic, resolution, |frame| {
            let validated = context.validate_upscale_inputs(
                frame.command_list,
                &frame.resources,
                &frame.parameters,
            );
            // SAFETY: Controlled COM/runtime fixtures record no GPU commands.
            let result =
                unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) };
            assert_result!(validated, result);
            result
        });
        context.destroy().unwrap();
    }
    assert_eq!(fixture_snapshot(&observer).destroys, 4);
}

#[cfg(feature = "movable")]
#[test]
fn movable_active_geometry_mapping_preflight_and_retry() {
    let (mut runtime, observer) = fixture_movable_runtime();
    for (dynamic, resolution) in profiles() {
        let scene = Scene::new();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&configure(&scene, dynamic, resolution))
            .unwrap();
        assert_eq!(
            fixture_snapshot(&observer).flags,
            u32::from(dynamic) * 64 + u32::from(resolution == MotionVectorResolution::Display) * 2
        );
        check_frames(&observer, &scene, dynamic, resolution, |frame| {
            let validated = context.validate_upscale_inputs(
                frame.command_list,
                &frame.resources,
                &frame.parameters,
            );
            // SAFETY: Controlled COM/runtime fixtures are serialized on this thread.
            let result =
                unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) };
            assert_result!(validated, result);
            result
        });
        context.destroy().unwrap();
    }
    assert_eq!(fixture_snapshot(&observer).destroys, 4);
}

macro_rules! output_terminal_case {
    ($name:ident, $runtime:ident, $context:ident) => {
        #[test]
        fn $name() {
            use crate::error::Operation;
            let (mut runtime, observer) = $runtime();
            let scene = Scene::new();
            let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                .create(&configure(&scene, true, MotionVectorResolution::Display))
                .unwrap();
            let mut frame = scene.input();
            frame.set_active_output_size(Some(Dimensions::new(1537, 863).unwrap()));
            assert_result!(
                context.validate_upscale_inputs(
                    frame.command_list,
                    &frame.resources,
                    &frame.parameters
                ),
                Ok(())
            );
            // Successful preflight cannot authorize later changed frame metadata.
            frame.set_active_output_size(Some(Dimensions::new(1921, 1080).unwrap()));
            // SAFETY: Controlled fixture records no GPU work; invalid geometry rejects.
            assert_result!(
                unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) },
                Err(Error::Input(crate::error::InputError::Dispatch(
                    DispatchValidation::ActiveOutputSize
                )))
            );
            // SAFETY: The local fixture supplies this exact synchronous export.
            unsafe {
                observer
                    .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                    .unwrap()(u32::MAX);
            }
            let mut frame = scene.input();
            frame.set_active_output_size(Some(Dimensions::new(1537, 863).unwrap()));
            // SAFETY: The controlled native error records no GPU commands.
            assert_result!(
                unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) },
                Err(Error::native(Operation::Upscale, u32::MAX))
            );
            scene
                .resource_object(0)
                .query_failure
                .set(Some(com::QUERY_FAILURE));
            let mut frame = scene.input();
            frame.set_active_output_size(Some(Dimensions::new(1921, 1080).unwrap()));
            assert_result!(
                context.validate_upscale_inputs(
                    frame.command_list,
                    &frame.resources,
                    &frame.parameters
                ),
                Err(crate::test_errors::terminal(Error::native(
                    crate::error::Operation::Upscale,
                    u32::MAX
                )))
            );
            // SAFETY: Poison rejects before inspection/FFI.
            assert_result!(
                unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) },
                Err(crate::test_errors::terminal(Error::native(
                    crate::error::Operation::Upscale,
                    u32::MAX
                )))
            );
            let calls = unsafe {
                observer
                    .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                    .unwrap()()
                .calls
            };
            assert_eq!(calls, 1);
            context.destroy().unwrap();
            assert_eq!(fixture_snapshot(&observer).destroys, 1);
        }
    };
}
output_terminal_case!(
    public_active_output_rechecks_and_preserves_native_poison,
    fixture_runtime,
    Upscaler
);
#[cfg(feature = "movable")]
output_terminal_case!(
    movable_active_output_rechecks_and_preserves_native_poison,
    fixture_movable_runtime,
    MovableUpscaler
);
