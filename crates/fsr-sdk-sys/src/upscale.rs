// SPDX-License-Identifier: MPL-2.0

//! Raw upscaler ABI from AMD FidelityFX SDK v2.3.0, `upscalers/include/ffx_upscale.h`.
//!
//! Derived declarations retain AMD's notice in `../LICENSE-AMD`.

use crate::api::ffxStructType_t;

/// Upscaler creation descriptor tag, also used to select the effect for version queries.
/// `ffx_upscale.h`: FFX_API_MAKE_EFFECT_SUB_ID(FFX_API_EFFECT_ID_UPSCALE, 0).
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE: ffxStructType_t = 0x0001_0000;

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
