// SPDX-License-Identifier: MPL-2.0

//! Raw upscaler ABI from AMD FidelityFX SDK v2.3.0, `upscalers/include/ffx_upscale.h`.
//!
//! Derived declarations retain AMD's notice in `../LICENSE-AMD`.

use crate::api::ffxStructType_t;

pub const FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK: ffxStructType_t = 0x0001_0006;
pub const FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_TONEMAP: u32 = 1 << 0;
pub const FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP: u32 = 1 << 1;
pub const FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_THRESHOLD: u32 = 1 << 2;
pub const FFX_UPSCALE_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX: u32 = 1 << 3;

/// Standalone reactive-mask recording descriptor from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxDispatchDescUpscaleGenerateReactiveMask {
    pub header: crate::api::ffxDispatchDescHeader,
    pub commandList: *mut core::ffi::c_void,
    pub colorOpaqueOnly: crate::api::FfxApiResource,
    pub colorPreUpscale: crate::api::FfxApiResource,
    pub outReactive: crate::api::FfxApiResource,
    pub renderSize: crate::api::FfxApiDimensions2D,
    pub scale: f32,
    pub cutoffThreshold: f32,
    pub binaryValue: f32,
    pub flags: u32,
}

pub const FFX_API_CONFIGURE_DESC_TYPE_UPSCALE_KEYVALUE: ffxStructType_t = 0x0001_0007;
pub const FFX_API_CONFIGURE_UPSCALE_KEY_FVELOCITYFACTOR: u32 = 0;
pub const FFX_API_CONFIGURE_UPSCALE_KEY_FREACTIVENESSSCALE: u32 = 1;
pub const FFX_API_CONFIGURE_UPSCALE_KEY_FSHADINGCHANGESCALE: u32 = 2;
pub const FFX_API_CONFIGURE_UPSCALE_KEY_FACCUMULATIONADDEDPERFRAME: u32 = 3;
pub const FFX_API_CONFIGURE_UPSCALE_KEY_FMINDISOCCLUSIONACCUMULATION: u32 = 4;

/// Context-specific float/default request from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxConfigureDescUpscaleKeyValue {
    pub header: crate::api::ffxConfigureDescHeader,
    pub key: u64,
    pub u64: u64,
    pub ptr: *mut core::ffi::c_void,
}

pub const FFX_UPSCALE_QUALITY_MODE_NATIVEAA: u32 = 0;
pub const FFX_UPSCALE_QUALITY_MODE_QUALITY: u32 = 1;
pub const FFX_UPSCALE_QUALITY_MODE_BALANCED: u32 = 2;
pub const FFX_UPSCALE_QUALITY_MODE_PERFORMANCE: u32 = 3;
pub const FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE: u32 = 4;

pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE: ffxStructType_t =
    0x0001_0002;
pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE: ffxStructType_t =
    0x0001_0003;
pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT: ffxStructType_t = 0x0001_0004;
pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET: ffxStructType_t = 0x0001_0005;

/// Provider-routed quality-mode ratio query from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode {
    pub header: crate::api::ffxQueryDescHeader,
    pub qualityMode: u32,
    pub pOutUpscaleRatio: *mut f32,
}

/// Provider-routed render-size query from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetRenderResolutionFromQualityMode {
    pub header: crate::api::ffxQueryDescHeader,
    pub displayWidth: u32,
    pub displayHeight: u32,
    pub qualityMode: u32,
    pub pOutRenderWidth: *mut u32,
    pub pOutRenderHeight: *mut u32,
}

/// Provider-routed jitter-length query from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetJitterPhaseCount {
    pub header: crate::api::ffxQueryDescHeader,
    pub renderWidth: u32,
    pub displayWidth: u32,
    pub pOutPhaseCount: *mut i32,
}

/// Provider-routed jitter-offset query from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetJitterOffset {
    pub header: crate::api::ffxQueryDescHeader,
    pub index: i32,
    pub phaseCount: i32,
    pub pOutX: *mut f32,
    pub pOutY: *mut f32,
}

pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE: ffxStructType_t = 0x0001_0008;
pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2: ffxStructType_t = 0x0001_0009;
pub const FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS: ffxStructType_t = 0x0001_000a;

pub const FFX_API_QUERY_RESOURCE_INPUT_COLOR: u32 = 1 << 0;
pub const FFX_API_QUERY_RESOURCE_INPUT_DEPTH: u32 = 1 << 1;
pub const FFX_API_QUERY_RESOURCE_INPUT_MV: u32 = 1 << 2;
pub const FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE: u32 = 1 << 3;
pub const FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK: u32 = 1 << 4;
pub const FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION: u32 = 1 << 5;

/// Live memory query from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetGPUMemoryUsage {
    pub header: crate::api::ffxQueryDescHeader,
    pub gpuMemoryUsageUpscaler: *mut crate::api::FfxApiEffectMemoryUsage,
}

/// Pre-create memory estimate from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetGPUMemoryUsageV2 {
    pub header: crate::api::ffxQueryDescHeader,
    pub device: *mut core::ffi::c_void,
    pub maxRenderSize: crate::api::FfxApiDimensions2D,
    pub maxUpscaleSize: crate::api::FfxApiDimensions2D,
    pub flags: u32,
    pub gpuMemoryUsageUpscaler: *mut crate::api::FfxApiEffectMemoryUsage,
}

/// Live or pre-create resource masks from SDK v2.3.0 `ffx_upscale.h`.
#[repr(C)]
pub struct ffxQueryDescUpscaleGetResourceRequirements {
    pub header: crate::api::ffxQueryDescHeader,
    pub required_resources: u64,
    pub optional_resources: u64,
}

/// Upscaler creation descriptor tag, also used to select the effect for version queries.
/// `ffx_upscale.h`: FFX_API_MAKE_EFFECT_SUB_ID(FFX_API_EFFECT_ID_UPSCALE, 0).
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE: ffxStructType_t = 0x0001_0000;
pub const FFX_API_DISPATCH_DESC_TYPE_UPSCALE: ffxStructType_t = 0x0001_0001;

/// Required upscaler API version descriptor tag.
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION: ffxStructType_t = 0x0001_000b;
pub const FFX_UPSCALER_VERSION: u32 = (4 << 22) | (1 << 12) | 1;

/// Input colour has high dynamic range (`ffx_upscale.h`).
pub const FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE: u32 = 1 << 0;
/// Input device depth is inverted, from near `1` to far `0` (`ffx_upscale.h`).
pub const FFX_UPSCALE_ENABLE_DEPTH_INVERTED: u32 = 1 << 3;
/// Motion vectors are supplied at display resolution.
pub const FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS: u32 = 1 << 1;
/// Input motion vectors contain projection jitter.
pub const FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION: u32 = 1 << 2;
/// Depth was generated with an infinite far-plane projection.
pub const FFX_UPSCALE_ENABLE_DEPTH_INFINITE: u32 = 1 << 4;
/// SDK computes exposure internally instead of using an exposure resource.
pub const FFX_UPSCALE_ENABLE_AUTO_EXPOSURE: u32 = 1 << 5;
pub const FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION: u32 = 1 << 6;
pub const FFX_UPSCALE_ENABLE_DEBUG_CHECKING: u32 = 1 << 7;
pub const FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION: u32 = 1 << 9;
pub const FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW: u32 = 1 << 0;
/// Input colour contains perceptual (gamma-corrected) values.
pub const FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE: u32 = 1 << 8;
/// Dispatch input contains perceptual sRGB colour.
pub const FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB: u32 = 1 << 1;
/// Dispatch input contains perceptual PQ colour.
pub const FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ: u32 = 1 << 2;

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
