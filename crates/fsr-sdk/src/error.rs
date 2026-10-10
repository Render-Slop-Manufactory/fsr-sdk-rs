// SPDX-License-Identifier: MPL-2.0

use std::fmt;

/// Native operation that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operation {
    /// Native context creation.
    Create,
    /// The context's single terminal native destruction attempt.
    Destroy,
    /// Native upscaler command recording; not queue submission or completion.
    Dispatch,
}

/// A scalar supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchScalar {
    /// Horizontal applied projection jitter in pixels; must be finite.
    JitterX,
    /// Vertical applied projection jitter in pixels; must be finite.
    JitterY,
    /// Positive finite elapsed frame time in milliseconds.
    FrameTimeMillis,
    /// Positive finite near distance in view-space units.
    CameraNear,
    /// Finite far distance greater than the near distance, in the same units.
    CameraFar,
    /// Finite vertical field of view strictly between zero and pi radians.
    VerticalFovRadians,
    /// Positive finite conversion factor from view-space units to metres.
    ViewSpaceToMeters,
    /// Finite native sharpening parameter in inclusive `[0, 1]`.
    NativeSharpness,
}

/// A resource supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceRole {
    /// Linear, non-HDR scene color before tonemapping.
    Color,
    /// Ordinary finite device depth.
    Depth,
    /// Jitter-free current-to-previous displacement.
    MotionVectors,
    /// Explicit exposure multiplier supplied in a 1×1 texture.
    Exposure,
    /// Upscaled color destination.
    Output,
}

/// The inspectable property that rejected a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceProperty {
    /// Two supplied resources have the same canonical COM identity.
    /// Distinct identities still do not prove non-overlapping backing storage.
    DistinctIdentity,
    /// Resource is not a single-sample, single-mip, single-slice Texture2D.
    TextureShape,
    /// Format is outside the role's fixed supported profile.
    Format,
    /// Extent differs from the context maximum, or exposure is not 1×1.
    Extent,
    /// An input denies shader-resource access.
    ShaderReadable,
    /// Output lacks unordered-access capability.
    UnorderedAccess,
    /// Resource does not belong to the context's device.
    Device,
}

/// Reason a dispatch was rejected before entering native code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchValidation {
    /// A checked scalar constructor rejected its input.
    Scalar(DispatchScalar),
    /// Command list is not DIRECT.
    CommandListType,
    /// Command list does not belong to the context's device.
    CommandListDevice,
    /// Inspection rejected a resource's metadata or identity.
    Resource {
        /// The input/output role being inspected.
        role: ResourceRole,
        /// The property that did not satisfy the supported profile.
        property: ResourceProperty,
    },
}

/// COM object whose inspection failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InspectionObject {
    /// Device retained by the upscaler context.
    ContextDevice,
    /// Caller-supplied recording command list.
    CommandList,
    /// Caller-supplied input/output texture.
    Resource(ResourceRole),
}

/// COM operation that failed during dispatch validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InspectionOperation {
    /// Obtaining the object's DX12 device failed.
    GetDevice,
    /// Obtaining canonical `IUnknown` identity failed.
    QueryIdentity,
}

/// A native result that violated the wrapper's ownership assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvariantViolation {
    /// Creation returned success without a usable context handle.
    NullContextOnSuccess,
}

/// Owned construction, dispatch and teardown failures.
///
/// Validation and COM inspection failures occur before native dispatch and do
/// not poison a usable context. A native dispatch failure preserves its code
/// and makes later dispatch terminal; the failed list must not be submitted.
/// Creation/destruction failures have their respective ownership contracts.
/// This error type does not convert arbitrary native crashes or GPU faults into
/// recoverable Rust errors. Runtime loading uses the separate `runtime::LoadError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A dimension constructor received zero width or height.
    InvalidDimensions {
        /// Supplied width in pixels.
        width: u32,
        /// Supplied height in pixels.
        height: u32,
    },
    /// Scalar validation or dispatch preflight rejected an input before FFI.
    InvalidDispatch {
        /// The concrete rejection category.
        reason: DispatchValidation,
    },
    /// A DX12/COM inspection failed before native dispatch.
    Dx12Inspection {
        /// Object whose inspection failed.
        object: InspectionObject,
        /// Inspection operation that failed.
        operation: InspectionOperation,
        /// Exact HRESULT bits, stored as the native signed 32-bit value.
        hresult: i32,
    },
    /// A previous native dispatch failed; this attempt made no native call.
    /// Teardown remains available after the caller completes earlier GPU work
    /// and invalidates failed or abandoned recordings.
    DispatchPoisoned,
    /// A native FidelityFX operation returned a non-OK code.
    Native {
        /// Native operation that failed.
        operation: Operation,
        /// Unmodified native `u32`, including values unknown to this SDK version.
        code: u32,
    },
    /// Native success could not establish a valid owned context.
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
