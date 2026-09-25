// SPDX-License-Identifier: MPL-2.0

//! Raw upscaler ABI from AMD FidelityFX SDK v2.3.0, `upscalers/include/ffx_upscale.h`.
//!
//! Derived declarations retain AMD's notice in `../LICENSE-AMD`.

use crate::api::ffxStructType_t;

/// Upscaler creation descriptor tag, also used to select the effect for version queries.
/// `ffx_upscale.h`: FFX_API_MAKE_EFFECT_SUB_ID(FFX_API_EFFECT_ID_UPSCALE, 0).
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE: ffxStructType_t = 0x0001_0000;
pub const FFX_API_DISPATCH_DESC_TYPE_UPSCALE: ffxStructType_t = 0x0001_0001;

/// Required upscaler API version descriptor tag.
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION: ffxStructType_t = 0x0001_000b;
pub const FFX_UPSCALER_VERSION: u32 = (4 << 22) | (1 << 12) | 1;

/// Windows layout: the message callback uses Windows wchar_t.
#[cfg(windows)]
#[repr(C)]
pub struct ffxCreateContextDescUpscale {
    pub header: crate::api::ffxCreateContextDescHeader,
    pub flags: u32,
    pub maxRenderSize: crate::api::FfxApiDimensions2D,
    pub maxUpscaleSize: crate::api::FfxApiDimensions2D,
    pub fpMessage: crate::api::ffxApiMessage,
}

#[repr(C)]
pub struct ffxCreateContextDescUpscaleVersion {
    pub header: crate::api::ffxCreateContextDescHeader,
    pub version: u32,
}

/// SDK v2.3.0 `ffx_upscale.h` dispatch descriptor. C++ `bool` occupies one
/// byte on the selected Windows/MSVC ABI; paired tests check its layout.
#[repr(C)]
#[allow(non_snake_case)]
pub struct ffxDispatchDescUpscale {
    pub header: crate::api::ffxDispatchDescHeader,
    pub commandList: *mut core::ffi::c_void,
    pub color: crate::api::FfxApiResource,
    pub depth: crate::api::FfxApiResource,
    pub motionVectors: crate::api::FfxApiResource,
    pub exposure: crate::api::FfxApiResource,
    pub reactive: crate::api::FfxApiResource,
    pub transparencyAndComposition: crate::api::FfxApiResource,
    pub output: crate::api::FfxApiResource,
    pub jitterOffset: crate::api::FfxApiFloatCoords2D,
    pub motionVectorScale: crate::api::FfxApiFloatCoords2D,
    pub renderSize: crate::api::FfxApiDimensions2D,
    pub upscaleSize: crate::api::FfxApiDimensions2D,
    pub enableSharpening: bool,
    pub sharpness: f32,
    pub frameTimeDelta: f32,
    pub preExposure: f32,
    pub reset: bool,
    pub cameraNear: f32,
    pub cameraFar: f32,
    pub cameraFovAngleVertical: f32,
    pub viewSpaceToMetersFactor: f32,
    pub flags: u32,
}
