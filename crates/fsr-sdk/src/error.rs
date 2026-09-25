// SPDX-License-Identifier: MPL-2.0

use std::fmt;

/// Native operation that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operation {
    Create,
    Destroy,
    Dispatch,
}

/// A scalar supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchScalar {
    JitterX,
    JitterY,
    FrameTimeMillis,
    CameraNear,
    CameraFar,
    VerticalFovRadians,
    ViewSpaceToMeters,
}

/// A resource supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceRole {
    Color,
    Depth,
    MotionVectors,
    Exposure,
    Output,
}

/// The inspectable property that rejected a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceProperty {
    DistinctIdentity,
    TextureShape,
    Format,
    Extent,
    ShaderReadable,
    UnorderedAccess,
    Device,
}

/// Reason a dispatch was rejected before entering native code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchValidation {
    Scalar(DispatchScalar),
    CommandListType,
    CommandListDevice,
    Resource {
        role: ResourceRole,
        property: ResourceProperty,
    },
}

/// COM object whose inspection failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InspectionObject {
    ContextDevice,
    CommandList,
    Resource(ResourceRole),
}

/// COM operation that failed during dispatch validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InspectionOperation {
    GetDevice,
    QueryIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvariantViolation {
    NullContextOnSuccess,
}

/// Native codes are open u32 values, including codes unknown to this SDK version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    InvalidDimensions {
        width: u32,
        height: u32,
    },
    InvalidDispatch {
        reason: DispatchValidation,
    },
    Dx12Inspection {
        object: InspectionObject,
        operation: InspectionOperation,
        hresult: i32,
    },
    DispatchPoisoned,
    Native {
        operation: Operation,
        code: u32,
    },
    NativeInvariantViolation(InvariantViolation),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(f, "FSR dimensions must be nonzero (got {width}x{height})")
            }
            Self::InvalidDispatch { reason } => write!(f, "invalid FSR dispatch: {reason:?}"),
            Self::Dx12Inspection {
                object,
                operation,
                hresult,
            } => write!(
                f,
                "DX12 {operation:?} of {object:?} failed with HRESULT {:#010x}",
                *hresult as u32
            ),
            Self::DispatchPoisoned => {
                f.write_str("FSR dispatch is unavailable after a native dispatch failure")
            }
            Self::Native { operation, code } => {
                write!(f, "FSR {operation:?} failed with native code {code}")
            }
            Self::NativeInvariantViolation(InvariantViolation::NullContextOnSuccess) => {
                f.write_str("FSR creation returned success with a null context")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unknown_codes_and_operation() {
        let error = Error::Native {
            operation: Operation::Destroy,
            code: u32::MAX,
        };
        assert_eq!(
            error.to_string(),
            "FSR Destroy failed with native code 4294967295"
        );
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn preserves_hresult_bits_separately_from_native_codes() {
        let error = Error::Dx12Inspection {
            object: InspectionObject::Resource(ResourceRole::Output),
            operation: InspectionOperation::GetDevice,
            hresult: 0x8000_4005u32 as i32,
        };
        assert_eq!(
            error.to_string(),
            "DX12 GetDevice of Resource(Output) failed with HRESULT 0x80004005"
        );
    }
}
