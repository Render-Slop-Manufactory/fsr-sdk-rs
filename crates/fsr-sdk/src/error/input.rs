// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Structured rejection of supplied values or their context/resource pairing.
///
/// Checked constructors are pure. Other input errors may follow COM inspection
/// or native provider enumeration, without entering the rejected effect operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputError {
    /// A required named resource binding is absent.
    MissingBinding {
        /// Omitted role.
        role: ResourceRole,
    },
    /// At least one dimension is zero.
    InvalidDimensions {
        /// Supplied width.
        width: u32,
        /// Supplied height.
        height: u32,
    },
    /// An explicit tuning value is non-finite or outside its parameter's domain.
    InvalidTuningValue {
        /// Parameter checked with the value.
        parameter: crate::upscaler::TuningParameter,
    },
    /// A quality/size/jitter query input is invalid.
    Query(QueryValidation),
    /// Recording inputs or their metadata do not satisfy the supported profile.
    Dispatch(DispatchValidation),
    /// The explicit ID is absent from fresh runtime/device enumeration.
    ProviderNotEnumerated {
        /// Requested ID, without implying staleness or capability.
        requested: crate::query::ProviderId,
    },
    /// Required inputs exceed the wrapper's supported dispatch resource roles.
    UnsupportedResourceRequirements {
        /// Complete required mask, including unknown bits.
        required: crate::query::ResourceFlags,
    },
}
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBinding { role } => write!(f, "missing resource binding: {role:?}"),
            Self::InvalidDimensions { width, height } => {
                write!(f, "FSR dimensions must be nonzero (got {width}x{height})")
            }
            Self::InvalidTuningValue { parameter } => write!(
                f,
                "invalid FSR tuning value for {parameter:?}: outside its documented finite domain"
            ),
            Self::Query(reason) => write!(f, "invalid FSR query: {reason:?}"),
            Self::Dispatch(reason) => write!(f, "invalid FSR dispatch: {reason:?}"),
            Self::ProviderNotEnumerated { requested } => write!(
                f,
                "FSR provider {requested:?} was not observed in the current enumeration"
            ),
            Self::UnsupportedResourceRequirements { required } => write!(
                f,
                "FSR provider requires unsupported inputs: {:#x}",
                required.bits()
            ),
        }
    }
}
impl std::error::Error for InputError {}
