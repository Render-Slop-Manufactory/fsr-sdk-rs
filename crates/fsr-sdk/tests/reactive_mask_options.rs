// SPDX-License-Identifier: MPL-2.0

#[macro_use]
#[path = "fixtures/errors.rs"]
mod test_errors;

use fsr_sdk::{
    error::{DispatchScalar, DispatchValidation, Error},
    upscaler::{ReactiveMaskOptions, common},
};

#[test]
fn finite_scalars_are_unchanged_and_flags_are_explicit() {
    fn traits<T: Send + Sync + Clone + std::fmt::Debug>() {}
    traits::<ReactiveMaskOptions>();
    for value in [
        -0.0,
        0.0,
        -f32::from_bits(1),
        f32::from_bits(1),
        -2.0,
        2.0,
        f32::MIN,
        f32::MAX,
    ] {
        let mut options: common::ReactiveMaskOptions =
            ReactiveMaskOptions::new(value, value, value).unwrap();
        assert_eq!(
            [
                options.scale().to_bits(),
                options.cutoff_threshold().to_bits(),
                options.binary_value().to_bits()
            ],
            [value.to_bits(); 3]
        );
        assert!(
            !options.apply_tonemap()
                && !options.apply_inverse_tonemap()
                && !options.apply_threshold()
                && !options.use_components_max()
        );
        options.set_apply_tonemap(true);
        options.set_apply_inverse_tonemap(true);
        options.set_apply_threshold(true);
        options.set_use_components_max(true);
        let copy = options.clone();
        assert!(
            copy.apply_tonemap()
                && copy.apply_inverse_tonemap()
                && copy.apply_threshold()
                && copy.use_components_max()
        );
        options.set_apply_tonemap(false);
        options.set_apply_inverse_tonemap(false);
        options.set_apply_threshold(false);
        options.set_use_components_max(false);
        assert!(
            !options.apply_tonemap()
                && !options.apply_inverse_tonemap()
                && !options.apply_threshold()
                && !options.use_components_max()
        );
    }
}

#[test]
fn every_nonfinite_scalar_has_its_own_error_even_with_all_flags_disabled() {
    for value in [
        f32::NAN,
        f32::from_bits(0xffc01234),
        f32::INFINITY,
        f32::NEG_INFINITY,
    ] {
        for (index, scalar) in [
            DispatchScalar::ReactiveScale,
            DispatchScalar::ReactiveCutoffThreshold,
            DispatchScalar::ReactiveBinaryValue,
        ]
        .into_iter()
        .enumerate()
        {
            let mut values = [0.0; 3];
            values[index] = value;
            assert_error!(
                ReactiveMaskOptions::new(values[0], values[1], values[2]).unwrap_err(),
                Error::Input(fsr_sdk::error::InputError::Dispatch(
                    DispatchValidation::Scalar(scalar)
                ))
            );
        }
    }
}
