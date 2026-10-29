// SPDX-License-Identifier: MPL-2.0

//! Checked direct native tuning requests.

use crate::error::Error;

/// One direct native context tuning parameter, without a support guarantee.
///
/// Tuning requests validate against the shared SDK v2.3.0 parameter domains.
/// These are provider-independent wrapper input rules, not capability guarantees.
/// These controls do not change creation options or request a history reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TuningParameter {
    /// Native velocity factor; accepts finite values in `[0, 1]`.
    VelocityFactor,
    /// Native reactiveness scale; accepts finite non-negative values.
    ReactivenessScale,
    /// Native shading-change scale; accepts finite non-negative values.
    ShadingChangeScale,
    /// Native accumulation added per frame; accepts finite values in `[0, 1]`.
    AccumulationAddedPerFrame,
    /// Native minimum disocclusion accumulation; accepts finite values in `[-1, 1]`.
    MinDisocclusionAccumulation,
}

/// An explicit value or a request to restore the actual provider's default.
///
/// This value is unchecked until paired with a parameter by [`TuningRequest::new`].
/// Accepted bits, including negative zero, are forwarded unchanged.
/// Neither variant implies support or an observed effective value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum TuningValue {
    /// Request this parameter's provider default using the native null-pointer route.
    /// Rounded header defaults are not substituted by the wrapper.
    ProviderDefault,
    /// Request a value, checked against the parameter's domain during request construction.
    Value(f32),
}

/// One checked parameter/value pair, ready for an idle context's tuning operation.
///
/// Construct all desired requests before applying any of them to avoid partial
/// native changes caused by a later invalid input. Native calls remain ordered
/// and nontransactional: a later native error does not undo earlier acknowledgements.
/// A checked request proves neither provider support/application nor GPU idleness.
/// It performs no native operation, wait, history reset or context mutation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningRequest {
    parameter: TuningParameter,
    value: TuningValue,
}

impl TuningRequest {
    /// Check the shared SDK v2.3.0 parameter domain, independently of any provider.
    ///
    /// Returns [`Error::Input`] for non-finite or out-of-domain values.
    /// All accepted bits are retained, including signed zero and subnormals;
    /// nothing is clamped. `ProviderDefault` preserves the native null reset.
    /// Reusing a [`TuningValue`] with another parameter requires a new check.
    pub fn new(parameter: TuningParameter, value: TuningValue) -> Result<Self, Error> {
        if let TuningValue::Value(value) = value {
            let in_range = match parameter {
                TuningParameter::VelocityFactor | TuningParameter::AccumulationAddedPerFrame => {
                    (0.0..=1.0).contains(&value)
                }
                TuningParameter::ReactivenessScale | TuningParameter::ShadingChangeScale => {
                    value >= 0.0
                }
                TuningParameter::MinDisocclusionAccumulation => (-1.0..=1.0).contains(&value),
            };
            if !value.is_finite() || !in_range {
                return Err(Error::Input(crate::error::InputError::InvalidTuningValue {
                    parameter,
                }));
            }
        }
        Ok(Self { parameter, value })
    }

    /// The parameter checked together with this request's value.
    pub const fn parameter(self) -> TuningParameter {
        self.parameter
    }

    /// The original checked value or explicit provider-default request.
    pub const fn value(self) -> TuningValue {
        self.value
    }
}
