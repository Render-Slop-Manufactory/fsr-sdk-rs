// SPDX-License-Identifier: MPL-2.0

use crate::error::{DispatchScalar, DispatchValidation, Error};

/// Direct SDK reactive-mask controls, independent of any GPU context.
///
/// All scalars must be finite; their exact bits are forwarded without clamping.
/// Finiteness does not certify a particular provider's numerical domain. The
/// four flags are independent, including simultaneous forward/inverse tonemaps.
/// No preset, conversion or provider-independent interpretation is implied.
#[derive(Debug, Clone)]
pub struct ReactiveMaskOptions {
    scale: f32,
    cutoff_threshold: f32,
    binary_value: f32,
    apply_tonemap: bool,
    apply_inverse_tonemap: bool,
    apply_threshold: bool,
    use_components_max: bool,
}

impl ReactiveMaskOptions {
    /// Reject non-finite scalars and initialize all four flags to false.
    /// Finite values outside `[0, 1]`, including negative zero, remain unchanged.
    pub fn new(scale: f32, cutoff_threshold: f32, binary_value: f32) -> Result<Self, Error> {
        for (value, scalar) in [
            (scale, DispatchScalar::ReactiveScale),
            (cutoff_threshold, DispatchScalar::ReactiveCutoffThreshold),
            (binary_value, DispatchScalar::ReactiveBinaryValue),
        ] {
            if !value.is_finite() {
                return Err(Error::Input(crate::error::InputError::Dispatch(
                    DispatchValidation::Scalar(scalar),
                )));
            }
        }
        Ok(Self {
            scale,
            cutoff_threshold,
            binary_value,
            apply_tonemap: false,
            apply_inverse_tonemap: false,
            apply_threshold: false,
            use_components_max: false,
        })
    }

    /// Unmodified output scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }
    /// Unmodified binary cutoff threshold.
    pub fn cutoff_threshold(&self) -> f32 {
        self.cutoff_threshold
    }
    /// Unmodified binary output value.
    pub fn binary_value(&self) -> f32 {
        self.binary_value
    }
    /// Whether the SDK forward-tonemap flag is set.
    pub fn apply_tonemap(&self) -> bool {
        self.apply_tonemap
    }
    /// Select the SDK forward-tonemap flag independently.
    pub fn set_apply_tonemap(&mut self, enabled: bool) {
        self.apply_tonemap = enabled;
    }
    /// Whether the SDK inverse-tonemap flag is set.
    pub fn apply_inverse_tonemap(&self) -> bool {
        self.apply_inverse_tonemap
    }
    /// Select the SDK inverse-tonemap flag independently.
    pub fn set_apply_inverse_tonemap(&mut self, enabled: bool) {
        self.apply_inverse_tonemap = enabled;
    }
    /// Whether the SDK binary-threshold flag is set.
    pub fn apply_threshold(&self) -> bool {
        self.apply_threshold
    }
    /// Select the SDK binary-threshold flag independently.
    pub fn set_apply_threshold(&mut self, enabled: bool) {
        self.apply_threshold = enabled;
    }
    /// Whether the SDK maximum-component flag is set.
    pub fn use_components_max(&self) -> bool {
        self.use_components_max
    }
    /// Select the SDK maximum-component flag independently.
    pub fn set_use_components_max(&mut self, enabled: bool) {
        self.use_components_max = enabled;
    }
}
