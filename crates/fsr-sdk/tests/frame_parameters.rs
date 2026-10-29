// SPDX-License-Identifier: MPL-2.0

//! Portable preparation and complete per-frame updates without any backend.
use fsr_sdk::upscaler::{
    CameraParameters, ColorEncodingHint, ColorEncodingRequest, Dimensions, FrameParameters,
    FrameTimeMillis, HistoryAction, JitterOffsetPixels, MotionVectorScale, NativeMotionVectorScale,
    NativeSharpness, PreExposure, ReactiveMaskOptions, ReactiveMaskParameters, Sharpening,
};

fn frame() -> FrameParameters {
    FrameParameters::new(
        JitterOffsetPixels::new(-0.0, 0.25).unwrap(),
        FrameTimeMillis::new(16.0).unwrap(),
        CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(),
        HistoryAction::Reset,
    )
}
#[test]
fn reuse_requires_explicit_history_and_required_value_updates() {
    let initial = frame();
    let mut next = initial.clone();
    assert_eq!(next.history_action(), HistoryAction::Reset);
    next.set_history_action(HistoryAction::Continue);
    next.set_jitter(JitterOffsetPixels::new(0.5, -0.5).unwrap());
    next.set_frame_time(FrameTimeMillis::new(20.0).unwrap());
    next.set_camera(CameraParameters::new_infinite(0.25, 1.5, 2.0).unwrap());
    assert_eq!(initial.history_action(), HistoryAction::Reset);
    assert_eq!(initial.jitter().x().to_bits(), (-0.0f32).to_bits());
    assert_eq!(next.history_action(), HistoryAction::Continue);
    assert_eq!(next.jitter().y(), -0.5);
    assert_eq!(next.frame_time().milliseconds(), 20.0);
    assert!(next.camera().has_infinite_far_plane());
    // Portable data can be prepared on a worker with no device or GPU borrow.
    let returned = std::thread::spawn(move || next).join().unwrap();
    assert_eq!(returned.history_action(), HistoryAction::Continue);
}
#[test]
fn motion_and_color_replace_complete_intent_without_dormant_state() {
    let mut p = frame();
    assert_eq!(p.motion_vector_scale(), MotionVectorScale::Uv);
    for scale in [
        NativeMotionVectorScale::new(-0.0, f32::from_bits(1)).unwrap(),
        NativeMotionVectorScale::new(-2.0, 4.0).unwrap(),
    ] {
        p.set_motion_vector_scale(MotionVectorScale::Native(scale));
        let saved = p.motion_vector_scale();
        p.set_motion_vector_scale(MotionVectorScale::Pixels);
        assert_eq!(p.motion_vector_scale(), MotionVectorScale::Pixels);
        p.set_motion_vector_scale(saved);
        let MotionVectorScale::Native(actual) = p.motion_vector_scale() else {
            panic!("literal request lost")
        };
        assert_eq!(
            (actual.x().to_bits(), actual.y().to_bits()),
            (scale.x().to_bits(), scale.y().to_bits())
        );
        p.set_motion_vector_scale(MotionVectorScale::Uv);
        assert_eq!(p.motion_vector_scale(), MotionVectorScale::Uv);
    }
    p.set_color_encoding(ColorEncodingRequest::allow_context_mismatch(
        ColorEncodingHint::Pq,
    ));
    p.set_color_encoding(ColorEncodingRequest::checked(ColorEncodingHint::Srgb));
    assert!(!p.color_encoding().allows_context_mismatch());
    assert_eq!(p.color_encoding().hint(), ColorEncodingHint::Srgb);
}
#[test]
fn optional_defaults_literal_sharpening_and_generator_controls_remain_portable() {
    let mut p = frame();
    assert!(matches!(p.sharpening(), Sharpening::Disabled));
    assert_eq!(p.pre_exposure().value(), 1.0);
    assert_eq!(
        (p.active_render_size(), p.active_output_size()),
        (None, None)
    );
    assert_eq!(
        p.color_encoding(),
        ColorEncodingRequest::checked(ColorEncodingHint::Unspecified)
    );
    assert!(!p.draw_debug_view());
    p.set_sharpening(Sharpening::Native(NativeSharpness::new(-0.0).unwrap()));
    let Sharpening::Native(v) = p.sharpening() else {
        panic!("enabled zero lost")
    };
    assert_eq!(v.value().to_bits(), (-0.0f32).to_bits());
    p.set_pre_exposure(PreExposure::new(f32::MIN_POSITIVE).unwrap());
    p.set_draw_debug_view(true);
    assert!(p.draw_debug_view());
    let size = Dimensions::new(320, 180).unwrap();
    let mut generator =
        ReactiveMaskParameters::new(size, ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap());
    let mut options = ReactiveMaskOptions::new(-0.0, -2.0, 3.0).unwrap();
    options.set_apply_tonemap(true);
    options.set_apply_inverse_tonemap(true);
    options.set_apply_threshold(true);
    options.set_use_components_max(true);
    generator.set_options(options);
    generator.set_render_size(Dimensions::new(160, 90).unwrap());
    assert_eq!(generator.options().scale().to_bits(), (-0.0f32).to_bits());
    assert!(generator.options().apply_tonemap() && generator.options().apply_inverse_tonemap());
    assert!(generator.options().apply_threshold() && generator.options().use_components_max());
    assert_eq!(generator.render_size(), Dimensions::new(160, 90).unwrap());
}
