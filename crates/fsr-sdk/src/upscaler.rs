// SPDX-License-Identifier: MPL-2.0

//! Checked common inputs and the DX12 upscaler backend.
//!
//! [`common`](crate::upscaler::common) owns configuration and frame scalars
//! without GPU object types. `dx12` owns construction,
//! borrowed GPU inputs, validation and recording.
//! Existing imports from this module remain aliases of those same types.
//!
//! Common inputs are available on every host without backend features. The
//! `dx12` module and its reexports require Windows and the `dx12` feature. Vulkan is
//! a planned support target, not yet implemented. Its future module needs its own
//! native integration and resource/synchronization contract. Windows/DX12 types,
//! code and dependencies must stay behind their platform/backend gates so that
//! backend-independent inputs and future Vulkan consumers do not require them.
//!
//! # Colour and exposure contract
//!
//! Colour controls describe the renderer's supplied signal. The wrapper forwards
//! them to the actual provider; it does not encode/decode pixels, normalize PQ,
//! convert gamuts, tonemap or configure a display. DX12 colour and output remain
//! typed or fixed-view typeless colour textures for both ownership families;
//! see the DX12 `UpscaleResources` documentation for its accepted formats.
//!
//! | Control | Meaning |
//! | --- | --- |
//! | [`set_high_dynamic_range`](crate::upscaler::UpscalerOptions::set_high_dynamic_range) | Describe HDR input before creation; independent of input encoding and exposure mode. Defaults to false. |
//! | [`set_input_color_space`](crate::upscaler::UpscalerOptions::set_input_color_space) | Select the native linear/non-linear creation classification. Defaults to `Linear`. Creation and memory estimation use the same flags. |
//! | [`ColorEncodingHint`](crate::upscaler::ColorEncodingHint) | Describe one frame's encoding using mutually exclusive native sRGB/PQ hints. `Unspecified` sets no hint and does not assert linear input. |
//! | [`PreExposure`](crate::upscaler::PreExposure) | Report a positive finite factor unchanged; no multiplication, reciprocal conversion or clamping occurs. Defaults to `1.0`. |
//! | [`ExposureMode`](crate::upscaler::ExposureMode) | Select explicit exposure (the default) or provider automatic exposure, independently of pre-exposure. |
//!
//! In the default linear profile, supply pre-tonemap RGB already multiplied by
//! the reported pre-exposure factor. Enable HDR before creating the context for
//! HDR input. Output remains in the input signal domain. Explicit exposure uses
//! the separate positive finite multiplier in a 1×1 R32F texture (typed or
//! fixed-view typeless); automatic exposure omits that texture.
//!
//! For non-linear input, the caller must establish the actual provider's supported
//! creation/hint combination, input encoding and normalization, exposure ordering,
//! output interpretation and temporal transitions. The wrapper supplies no
//! universal PQ reference luminance or rule choosing between multiplying linear
//! RGB before encoding and multiplying encoded code values. Native success and
//! resource requirements do not establish those signal semantics.
//!
//! By default, per-frame sRGB/PQ hints require a `NonLinear` context. A mismatch
//! rejects before COM/native inspection without poisoning a usable context.
//! `ColorEncodingRequest::allow_context_mismatch` bypasses only this conservative
//! pairing check; all provider, resource and unsafe dispatch obligations remain.
//! Replacing it with `ColorEncodingRequest::checked` restores checking.
//! Changing standalone options never reconfigures an existing context, and
//! these controls never reset history.
//!
//! Bounded native checks have exercised linear HDR and non-linear signals on one
//! GPU/driver/provider. Changing non-linear pre-exposure produced signal-prediction
//! discrepancies; static/reset controls passed, without establishing a universal
//! packing rule, a wrapper defect or an automatic reset policy. This API exposes
//! the SDK controls; it does not certify an end-to-end non-linear rendering pipeline.

pub mod common;
#[cfg(all(windows, feature = "dx12"))]
pub mod dx12;
#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) mod reactive_mask;
#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) mod tuning;

pub use common::{
    CameraParameters, ColorEncodingHint, ColorEncodingRequest, ContextUsability, DebugLevel,
    DepthConvention, Dimensions, ExposureMode, FrameParameters, FrameTimeMillis, HistoryAction,
    InputColorSpace, JitterOffsetPixels, MotionVectorJitter, MotionVectorResolution,
    MotionVectorScale, NativeMotionVectorScale, NativeSharpness, PreExposure, ReactiveMaskOptions,
    ReactiveMaskParameters, Sharpening, TuningParameter, TuningRequest, TuningValue,
    UpscalerOptions,
};
#[cfg(all(windows, feature = "dx12", feature = "movable"))]
pub use dx12::MovableUpscaler;
#[cfg(all(windows, feature = "dx12"))]
pub use dx12::{
    ExposureInput, ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource, ReactiveMaskResources,
    ReactiveMaskResourcesBuilder, UpscaleResources, UpscaleResourcesBuilder, Upscaler,
};

#[cfg(all(test, windows, feature = "dx12"))]
pub(crate) use dx12::ReactiveMaskTestInputs;
