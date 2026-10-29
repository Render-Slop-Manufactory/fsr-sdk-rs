// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::error::{DispatchValidation, ResourceProperty, ResourceRole};
use crate::upscaler::{
    DepthConvention, MotionVectorJitter, MotionVectorResolution, MotionVectorScale,
};
use std::cell::Cell;

type Observation = ([f32; 6], [u32; 4]);

thread_local! {
    static CAPTURED: Cell<Option<Observation>> = const { Cell::new(None) };
}

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Controlled public dispatch supplies a complete live descriptor.
    // Copy scalars only; do not retain any resource pointers.
    let desc = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    CAPTURED.set(Some((
        [
            desc.motionVectorScale.x,
            desc.motionVectorScale.y,
            desc.jitterOffset.x,
            desc.jitterOffset.y,
            desc.cameraNear,
            desc.cameraFar,
        ],
        [
            desc.renderSize.width,
            desc.renderSize.height,
            desc.upscaleSize.width,
            desc.upscaleSize.height,
        ],
    )));
}

fn check_frame<'a>(
    observer: &libloading::Library,
    scene: &'a Scene,
    options: &UpscalerOptions,
    mut dispatch: impl FnMut(UpscaleTestInputs<'a>) -> Result<(), Error>,
) {
    // SAFETY: Exact fixture ABI, synchronously observing this test's descriptor.
    unsafe {
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
    let motion = if options.motion_vector_resolution() == MotionVectorResolution::Display {
        options.max_upscale_size()
    } else {
        options.max_render_size()
    };
    let mut desc = scene.resource_object(2).desc.get();
    desc.Width = u64::from(motion.width());
    desc.Height = motion.height();
    scene.resource_object(2).desc.set(desc);
    let infinite = options.depth_convention().is_infinite();
    let camera = |infinite| {
        if infinite {
            CameraParameters::new_infinite(0.1, 1.0, 1.0).unwrap()
        } else {
            CameraParameters::new(0.1, 1000.0, 1.0, 1.0).unwrap()
        }
    };
    let input = |infinite| {
        let base = scene.input();
        UpscaleTestInputs::new(
            base.command_list(),
            base.resources().clone(),
            JitterOffsetPixels::new(0.25, -0.25).unwrap(),
            base.frame_time(),
            camera(infinite),
            false,
        )
    };
    let calls = || {
        // SAFETY: Exact synchronous fixture ABI; no pointers retained.
        unsafe {
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
            .calls
        }
    };
    let before = calls();
    let counts = scene.reference_counts();
    // An incompatible camera is rejected before COM inspection and native entry.
    scene
        .resource_object(0)
        .query_failure
        .set(Some(com::QUERY_FAILURE));
    assert_result!(
        dispatch(input(!infinite)),
        Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::CameraDepthConvention
        )))
    );
    scene.resource_object(0).query_failure.set(None);
    assert_eq!(calls(), before);
    assert_eq!(scene.reference_counts(), counts);
    // Motion must match the selected extent; rejection leaves the context usable.
    let mut wrong = desc;
    let wrong_size = if options.motion_vector_resolution() == MotionVectorResolution::Display {
        options.max_render_size()
    } else {
        options.max_upscale_size()
    };
    wrong.Width = u64::from(wrong_size.width());
    wrong.Height = wrong_size.height();
    scene.resource_object(2).desc.set(wrong);
    assert_result!(
        dispatch(input(infinite)),
        Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::Resource {
                role: ResourceRole::MotionVectors,
                property: ResourceProperty::Extent
            }
        )))
    );
    scene.resource_object(2).desc.set(desc);
    assert_eq!(calls(), before);
    assert_eq!(scene.reference_counts(), counts);
    for (units, scale) in [
        (None, (motion.width() as f32, motion.height() as f32)),
        (Some(MotionVectorScale::Pixels), (1.0, 1.0)),
        (
            Some(MotionVectorScale::Uv),
            (motion.width() as f32, motion.height() as f32),
        ),
    ] {
        let mut frame = input(infinite);
        assert_eq!(frame.motion_vector_scale(), MotionVectorScale::Uv);
        if let Some(units) = units {
            frame.set_motion_vector_scale(MotionVectorScale::Pixels);
            frame.set_motion_vector_scale(units);
        }
        assert_eq!(frame.motion_vector_scale(), units.unwrap_or_default());
        assert!(!frame.reset());
        assert_eq!(frame.pre_exposure().value(), 1.0);
        assert!(matches!(frame.sharpening(), Sharpening::Disabled));
        let counts = scene.reference_counts();
        CAPTURED.set(None);
        assert_result!(dispatch(frame), Ok(()));
        assert_eq!(scene.reference_counts(), counts);
        assert_eq!(
            CAPTURED.take(),
            Some((
                [scale.0, scale.1, 0.25, -0.25, 0.1, camera(infinite).far()],
                [1280, 720, 1920, 1080]
            ))
        );
    }
    assert_eq!(calls(), before + 3);
}

fn profiles() -> impl Iterator<Item = (UpscalerOptions, u32)> {
    [
        (DepthConvention::StandardFinite, 0),
        (DepthConvention::ReversedFinite, 8),
        (DepthConvention::StandardInfinite, 16),
        (DepthConvention::ReversedInfinite, 24),
    ]
    .into_iter()
    .flat_map(|(depth, depth_flags)| {
        [
            (MotionVectorResolution::Render, 0),
            (MotionVectorResolution::Display, 2),
        ]
        .into_iter()
        .flat_map(move |(resolution, resolution_flags)| {
            [
                (MotionVectorJitter::Excluded, 0),
                (MotionVectorJitter::Included, 4),
            ]
            .into_iter()
            .map(move |(jitter, jitter_flags)| {
                let mut options = sizes();
                options.set_depth_convention(depth);
                options.set_motion_vector_resolution(resolution);
                options.set_motion_vector_jitter(jitter);
                (options, depth_flags | resolution_flags | jitter_flags)
            })
        })
    })
}

#[test]
fn public_depth_and_motion_profile_mapping_and_preflight() {
    let (mut runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    for (options, flags) in profiles() {
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&options)
            .unwrap();
        assert_eq!(fixture_snapshot(&observer).flags, flags);
        check_frame(&observer, &scene, &options, |frame| {
            // SAFETY: Controlled COM/runtime fixtures; no GPU commands recorded.
            unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) }
        });
        context.destroy().unwrap();
    }
    assert_eq!(fixture_snapshot(&observer).destroys, 16);
}

#[cfg(feature = "movable")]
#[test]
fn movable_depth_and_motion_profile_mapping_and_preflight() {
    let (mut runtime, observer) = fixture_movable_runtime();
    let scene = Scene::new();
    for (options, flags) in profiles() {
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&options)
            .unwrap();
        assert_eq!(fixture_snapshot(&observer).flags, flags);
        check_frame(&observer, &scene, &options, |frame| {
            // SAFETY: Controlled COM/runtime fixtures, serialized on this thread.
            unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) }
        });
        context.destroy().unwrap();
    }
    assert_eq!(fixture_snapshot(&observer).destroys, 16);
}
