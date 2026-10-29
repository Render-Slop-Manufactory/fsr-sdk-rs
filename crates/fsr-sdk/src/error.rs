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
    /// Native upscaling recording, without queue submission or completion.
    Upscale,
    /// Native reactive-mask generation recording.
    GenerateReactiveMask,
    /// Provider metadata, resource/memory or quality/size/jitter query.
    Query,
    /// Context tuning; success acknowledges the request only.
    Tuning,
    /// Setup modern diagnostic configuration with a null context.
    ConfigureDebug,
    /// Setup legacy diagnostic configuration with a null context.
    ConfigureGlobalDebugLegacy,
    /// Live-context legacy diagnostics, potentially affecting global state.
    ConfigureContextDebugLegacy,
}

/// A scalar supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchScalar {
    /// Horizontal applied projection jitter in pixels; must be finite.
    JitterX,
    /// Vertical applied projection jitter in pixels; must be finite.
    JitterY,
    /// Finite native horizontal motion multiplier; forwarded without normalization.
    MotionVectorScaleX,
    /// Finite native vertical motion multiplier; forwarded without normalization.
    MotionVectorScaleY,
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
    /// Positive finite factor already applied to scene colour.
    PreExposure,
    /// Finite native reactive-mask output scale, forwarded unchanged.
    ReactiveScale,
    /// Finite native reactive-mask threshold, forwarded unchanged.
    ReactiveCutoffThreshold,
    /// Finite native reactive-mask binary value, forwarded unchanged.
    ReactiveBinaryValue,
}

/// A resource supplied for an upscaler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceRole {
    /// Scene colour matching the selected HDR, encoding and exposure profile.
    Color,
    /// Opaque-only colour input for reactive-mask generation.
    OpaqueOnlyColor,
    /// Generated reactive-mask destination, shader-readable and UAV-capable.
    ReactiveMaskOutput,
    /// Device depth matching the context's projection convention.
    Depth,
    /// Current-to-previous displacement matching the selected motion profile.
    MotionVectors,
    /// Explicit exposure multiplier supplied in a 1×1 texture.
    Exposure,
    /// Upscaled colour destination.
    Output,
    /// Renderer-supplied reactive mask.
    ReactiveMask,
    /// Renderer-supplied transparency/composition mask.
    TransparencyAndCompositionMask,
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
    /// Format is outside the role's reviewed supported profile.
    Format,
    /// Device format bits do not satisfy the role's typed view/access profile.
    FormatSupport,
    /// Extent differs from the context maximum, or exposure is not 1×1.
    Extent,
    /// A role requiring an SRV denies shader-resource access.
    ShaderReadable,
    /// Output lacks unordered-access capability, or an incompatible format enables it.
    UnorderedAccess,
    /// Depth/stencil capability disagrees with the allocation's fixed view.
    DepthStencil,
    /// Resource does not belong to the context's device.
    Device,
}

/// Reason a dispatch was rejected before entering native code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchValidation {
    /// Active render size exceeds the maximum or differs on a fixed context.
    ActiveRenderSize,
    /// Active output size exceeds the maximum or differs on a fixed context.
    ActiveOutputSize,
    /// Exposure presence does not match explicit/automatic context configuration.
    ExposureMode,
    /// sRGB/PQ hint requires a NonLinear context unless explicitly overridden for this frame.
    ColorEncodingContext,
    /// A mask reported as required by the provider was omitted.
    MissingRequiredResource(ResourceRole),
    /// Camera finite/infinite far-plane mode differs from the context's depth convention.
    CameraDepthConvention,
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
    /// Context device inspected on behalf of this supplied resource role.
    ContextDeviceForResource(ResourceRole),
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
    /// Querying typed format support on the context's DX12 device failed.
    CheckFormatSupport,
    /// Querying support for casting a fully typed allocation failed.
    CheckFormatCasting,
}

/// A successful native result outside the wrapper's owned or checked output contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvariantViolation {
    /// Explicit creation returned a different effective provider.
    ProviderSelectionMismatch {
        /// Requested opaque provider ID.
        requested: crate::query::ProviderId,
        /// Effective live provider ID.
        actual: crate::query::ProviderId,
    },
    /// A successful quality query returned a non-positive or non-finite ratio.
    InvalidUpscaleRatioOnSuccess,
    /// A successful render-size query returned zero or a size larger than display.
    InvalidRenderSizeOnSuccess,
    /// A successful jitter-length query returned a non-positive count.
    InvalidJitterPhaseCountOnSuccess,
    /// A successful jitter-offset query returned a non-finite component.
    InvalidJitterOffsetOnSuccess,
    /// Creation returned success without a usable context handle.
    NullContextOnSuccess,
    /// A successful query returned a null provider name.
    NullProviderNameOnSuccess,
    /// A successful live-provider query returned the explicitly invalid ID zero.
    ZeroProviderIdOnSuccess,
    /// A successful enumeration reported more elements than its supplied capacity.
    QueryCountExceedsCapacity,
}

/// An input rejected before a quality/size/jitter query enters native code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryValidation {
    /// A utility dimension exceeds `i32::MAX`, the wrapper's signed SDK domain.
    DimensionsOutOfRange,
    /// Jitter width ratio is outside inclusive `[1, 16383]`.
    JitterScale,
    /// Jitter sequence length must be positive and representable as `i32`.
    JitterPhaseCount,
    /// Sequence index must be non-negative and less than the phase count.
    JitterIndex,
}

#[path = "error/allocation.rs"]
mod allocation;
#[path = "error/backend.rs"]
mod backend;
#[path = "error/input.rs"]
mod input;
#[path = "error/native.rs"]
mod native;
pub use allocation::{QueryAllocationError, QueryAllocationReason};
pub use backend::{BackendError, Dx12InspectionError};
pub use input::InputError;
pub use native::{NativeError, TerminalCause};

/// Owned input, native, backend, invariant and terminal failures.
///
/// Input/inspection failures do not poison a usable context. A mutating native
/// context failure retains its first operation/code; explicit recording abandonment
/// instead records a local cause unless an earlier terminal cause exists. Later live operations reject
/// before inspection or FFI. Ordinary query errors do not change eligibility.
/// Terminal destruction and exceptional dependency retention keep their existing
/// contracts. This error neither catches native crashes nor establishes GPU idle.
/// Loading uses the separate `runtime::LoadError` boundary.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Supplied inputs were rejected; inspection/enumeration may have occurred.
    Input(InputError),
    /// A native SDK call returned its exact nonzero failure code.
    Native(NativeError),
    /// Backend inspection failed, retaining its owned original cause.
    Backend(BackendError),
    /// Native success returned inconsistent output.
    Invariant {
        /// Operation whose result violated the wrapper contract.
        operation: Operation,
        /// Observed inconsistency.
        reason: InvariantViolation,
    },
    /// A previous mutating native failure or explicit abandonment prevents live use.
    ContextUnusable(TerminalCause),
    /// A fallible query-buffer conversion or reservation failed.
    QueryAllocation(QueryAllocationError),
}

#[allow(dead_code)] // Also included by standalone ownership compile controls.
impl Error {
    pub(crate) fn native(operation: Operation, code: u32) -> Self {
        Self::Native(NativeError::new(operation, code))
    }
    pub(crate) fn invariant(operation: Operation, reason: InvariantViolation) -> Self {
        Self::Invariant { operation, reason }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => error.fmt(f),
            Self::Native(error) => error.fmt(f),
            Self::Backend(error) => error.fmt(f),
            Self::Invariant { operation, reason } => write!(
                f,
                "FSR {operation:?} returned inconsistent success output: {reason}"
            ),
            Self::ContextUnusable(cause) => write!(f, "FSR context is unusable after {cause}"),
            Self::QueryAllocation(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Input(error) => error,
            Self::Native(error) => error,
            Self::Backend(error) => error,
            Self::Invariant { reason, .. } => reason,
            Self::ContextUnusable(cause) => cause,
            Self::QueryAllocation(error) => error,
        })
    }
}
impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderSelectionMismatch { requested, actual } => {
                write!(f, "requested provider {requested:?}, selected {actual:?}")
            }
            reason => write!(f, "{reason:?}"),
        }
    }
}
impl std::error::Error for InvariantViolation {}

#[cfg(test)]
pub(crate) fn test_inspection(
    object: InspectionObject,
    operation: InspectionOperation,
    hresult: i32,
) -> Error {
    // Fixture expectations compare metadata; original-source preservation has
    // separate tests through real controlled DX12 inspection calls.
    Error::Backend(BackendError::Dx12Inspection(Dx12InspectionError::new(
        object,
        operation,
        hresult,
        Box::new(std::io::Error::from_raw_os_error(hresult)),
    )))
}

#[cfg(test)]
#[path = "error/tests.rs"]
mod tests;
