// SPDX-License-Identifier: MPL-2.0

//! Paired production creation ABI checks and test-only provider query ABI.
//! Derived from local AMD SDK v2.3.0; see ../../LICENSE-AMD.
#![allow(non_camel_case_types, non_snake_case)]

pub use fsr_sdk_sys::{
    api::{FfxApiDimensions2D, ffxApiHeader, ffxApiMessage, ffxQueryDescHeader},
    dx12::*,
    upscale::{
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION, FFX_UPSCALER_VERSION,
        ffxCreateContextDescUpscale, ffxCreateContextDescUpscaleVersion,
    },
};
use std::ffi::c_char;

#[repr(C)]
pub struct ffxQueryGetProviderVersion {
    pub header: ffxQueryDescHeader,
    pub versionId: u64,
    pub versionName: *const c_char,
}

pub const FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION: u64 = 6;

// Paired field types and numeric layouts are checked against native_abi.cpp.
macro_rules! field {
    ($ty:ty, $name:ident, $field_ty:ty, $offset:expr) => {
        const _: fn($ty) -> $field_ty = |value| value.$name;
        const _: () = assert!(std::mem::offset_of!($ty, $name) == $offset);
    };
}
macro_rules! layout {
    ($ty:ty, $size:expr, $align:expr) => {
        const _: () = assert!(std::mem::size_of::<$ty>() == $size);
        const _: () = assert!(std::mem::align_of::<$ty>() == $align);
    };
}
layout!(FfxApiDimensions2D, 8, 4);
field!(FfxApiDimensions2D, width, u32, 0);
field!(FfxApiDimensions2D, height, u32, 4);
layout!(ffxApiMessage, 8, 8);
layout!(ffxCreateContextDescUpscale, 48, 8);
field!(ffxCreateContextDescUpscale, header, ffxApiHeader, 0);
field!(ffxCreateContextDescUpscale, flags, u32, 16);
field!(
    ffxCreateContextDescUpscale,
    maxRenderSize,
    FfxApiDimensions2D,
    20
);
field!(
    ffxCreateContextDescUpscale,
    maxUpscaleSize,
    FfxApiDimensions2D,
    28
);
field!(ffxCreateContextDescUpscale, fpMessage, ffxApiMessage, 40);
layout!(ffxCreateContextDescUpscaleVersion, 24, 8);
field!(ffxCreateContextDescUpscaleVersion, header, ffxApiHeader, 0);
field!(ffxCreateContextDescUpscaleVersion, version, u32, 16);
layout!(ffxCreateBackendDX12Desc, 24, 8);
field!(ffxCreateBackendDX12Desc, header, ffxApiHeader, 0);
field!(ffxCreateBackendDX12Desc, device, *mut ID3D12Device, 16);
layout!(ffxQueryGetProviderVersion, 32, 8);
field!(ffxQueryGetProviderVersion, header, ffxApiHeader, 0);
field!(ffxQueryGetProviderVersion, versionId, u64, 16);
field!(ffxQueryGetProviderVersion, versionName, *const c_char, 24);

const _: () = assert!(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION == 0x0001_000b);
const _: () = assert!(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12 == 2);
const _: () = assert!(FFX_UPSCALER_VERSION == 0x0100_1001);
const _: fn(ffxApiMessage) -> Option<unsafe extern "C" fn(u32, *const u16)> = |value| value;
