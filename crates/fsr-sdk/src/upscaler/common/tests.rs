// SPDX-License-Identifier: MPL-2.0

//! Common profile and diagnostic regression tests.

use super::*;
use crate::error::{DispatchValidation, Error};

#[test]
fn encoding_hints_require_non_linear_context_unless_explicitly_overridden() {
    let size = Dimensions::new(320, 180).unwrap();
    let mut options = UpscalerOptions::new(size, size);
    for space in [InputColorSpace::Linear, InputColorSpace::NonLinear] {
        options.set_input_color_space(space);
        for hint in [
            ColorEncodingHint::Unspecified,
            ColorEncodingHint::Srgb,
            ColorEncodingHint::Pq,
        ] {
            for allow in [false, true] {
                let expected = if space == InputColorSpace::Linear
                    && hint != ColorEncodingHint::Unspecified
                    && !allow
                {
                    Err(Error::Input(crate::error::InputError::Dispatch(
                        DispatchValidation::ColorEncodingContext,
                    )))
                } else {
                    Ok(())
                };
                assert_result!(options.check_color_encoding_hint(hint, allow), expected);
            }
        }
    }
    for (hint, bits) in [
        (ColorEncodingHint::Unspecified, 0),
        (ColorEncodingHint::Srgb, 2),
        (ColorEncodingHint::Pq, 4),
    ] {
        assert_eq!(hint.native_flags(), bits);
    }
}

#[test]
fn non_linear_creation_bit_preserves_the_other_profile_flags() {
    let size = Dimensions::new(320, 180).unwrap();
    let mut options = UpscalerOptions::new(size, size);
    options.set_input_color_space(InputColorSpace::NonLinear);
    assert_eq!(options.native_flags(), 256);
    options.set_depth_convention(DepthConvention::ReversedInfinite);
    options.set_motion_vector_resolution(MotionVectorResolution::Display);
    options.set_motion_vector_jitter(MotionVectorJitter::Included);
    options.set_exposure_mode(ExposureMode::Automatic);
    options.set_dynamic_resolution(true);
    options.set_high_dynamic_range(true);
    assert_eq!(options.native_flags(), 383);
    options.set_input_color_space(InputColorSpace::Linear);
    assert_eq!(options.native_flags(), 127);
}

#[test]
fn hdr_flag_is_independent_of_other_creation_flags() {
    let size = Dimensions::new(320, 180).unwrap();
    for combined in [false, true] {
        let mut options = UpscalerOptions::new(size, size);
        if combined {
            options.set_depth_convention(DepthConvention::ReversedInfinite);
            options.set_motion_vector_resolution(MotionVectorResolution::Display);
            options.set_motion_vector_jitter(MotionVectorJitter::Included);
            options.set_exposure_mode(ExposureMode::Automatic);
            options.set_dynamic_resolution(true);
        }
        let other_flags = if combined { 126 } else { 0 };
        for enabled in [false, true, false] {
            options.set_high_dynamic_range(enabled);
            assert_eq!(options.native_flags(), other_flags | u32::from(enabled));
        }
    }
}

#[test]
fn active_render_bounds_and_fixed_default() {
    let maximum = Dimensions::new(320, 180).unwrap();
    let mut options = UpscalerOptions::new(maximum, Dimensions::new(640, 360).unwrap());
    let invalid = Err(Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::ActiveRenderSize,
    )));
    assert_result!(options.check_active_render_size(None), Ok(maximum));
    assert_result!(options.check_active_render_size(Some(maximum)), Ok(maximum));
    assert_result!(
        options.check_active_render_size(Some(Dimensions::new(319, 180).unwrap())),
        invalid
    );
    options.set_dynamic_resolution(true);
    for (width, height) in [(1, 1), (320, 1), (1, 180), (256, 144), (320, 180)] {
        let size = Dimensions::new(width, height).unwrap();
        assert_result!(options.check_active_render_size(Some(size)), Ok(size));
    }
    for (width, height) in [(321, 1), (1, 181), (321, 181), (u32::MAX, u32::MAX)] {
        assert_result!(
            options.check_active_render_size(Some(Dimensions::new(width, height).unwrap())),
            invalid
        );
    }
    assert_result!(options.check_active_render_size(None), Ok(maximum));
}

#[test]
fn active_output_bounds_are_independent_of_render_and_require_dynamic_option() {
    let maximum = Dimensions::new(640, 360).unwrap();
    let mut options = UpscalerOptions::new(Dimensions::new(320, 180).unwrap(), maximum);
    let invalid = Err(Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::ActiveOutputSize,
    )));
    assert_result!(options.check_active_output_size(None), Ok(maximum));
    assert_result!(options.check_active_output_size(Some(maximum)), Ok(maximum));
    assert_result!(
        options.check_active_output_size(Some(Dimensions::new(511, 287).unwrap())),
        invalid
    );
    options.set_dynamic_resolution(true);
    let cloned = options.clone();
    for (width, height) in [(1, 1), (511, 287), (640, 287), (511, 360), (640, 360)] {
        let size = Dimensions::new(width, height).unwrap();
        assert_result!(cloned.check_active_output_size(Some(size)), Ok(size));
    }
    for (width, height) in [(641, 1), (1, 361), (641, 361), (u32::MAX, u32::MAX)] {
        assert_result!(
            cloned.check_active_output_size(Some(Dimensions::new(width, height).unwrap())),
            invalid
        );
    }
    options.set_dynamic_resolution(false);
    assert_result!(options.check_active_output_size(None), Ok(maximum));
    assert_result!(
        options.check_active_render_size(None),
        Ok(options.max_render_size())
    );
}

#[cfg(test)]
#[test]
fn independent_debug_creation_flags() {
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    assert!(!options.debug_checking() && !options.debug_visualization());
    let base = options.native_flags();
    for (checking, visualization) in [(false, false), (true, false), (false, true), (true, true)] {
        options.set_debug_checking(checking);
        options.set_debug_visualization(visualization);
        assert_eq!(
            options.native_flags(),
            base | if checking { 128 } else { 0 } | if visualization { 512 } else { 0 }
        );
    }
}

#[cfg(test)]
#[test]
fn debug_levels_match_sdk_values() {
    for (level, expected) in [
        (DebugLevel::Silent, 0),
        (DebugLevel::Errors, 1),
        (DebugLevel::Warnings, 2),
        (DebugLevel::Verbose, 0x0fffffff),
    ] {
        assert_eq!(level.native(), expected);
    }
}
