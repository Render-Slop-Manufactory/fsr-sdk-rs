// SPDX-License-Identifier: MPL-2.0

//! Isolated M6 GPU proof, now exercising the public M6b route.
//! C++ DX12 equipment lives under tests/.
use super::{
    CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch,
    UpscaleResources, Upscaler, UpscalerOptions, dispatch::resource_for_parity,
};
use crate::{error::Error, runtime::Runtime};
use fsr_sdk_sys::api::{
    FFX_API_RESOURCE_STATE_COMPUTE_READ, FFX_API_RESOURCE_STATE_UNORDERED_ACCESS,
    FFX_API_RETURN_OK, FfxApiResource, ffxApiHeader,
};
use std::{
    ffi::{CStr, c_char, c_void},
    path::PathBuf,
    ptr::null_mut,
};
use windows::{
    Win32::Graphics::{
        Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource},
        Dxgi::Common::{
            DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32_FLOAT,
        },
    },
    core::Interface,
};

mod support;
use support::*;
#[cfg(feature = "movable")]
mod movable_public;
mod semantics;
mod sharpening;
mod single_frame;
mod temporal_smoke;
mod thread_transfer_baseline;
mod thread_transfers;

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tests/run-m6.ps1 -Native"]
fn native_dispatch_readback() {
    single_frame::native_dispatch_readback();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tests/run-m6.ps1 -Native"]
fn native_temporal_smoke() {
    temporal_smoke::native_temporal_smoke();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tests/run-m6.ps1 -Native"]
fn native_semantic_comparisons() {
    semantics::compare();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use native/run.ps1 -Mode sharpening -Native"]
fn native_sharpening_comparisons() {
    sharpening::compare();
}
