// SPDX-License-Identifier: MPL-2.0

//! DX12 creation ABI from AMD FidelityFX SDK v2.3.0,
//! `api/include/dx12/ffx_api_dx12.h`. AMD notice: `../LICENSE-AMD`.
//! Raw pointers only; this module does not own COM references.

use crate::api::{ffxCreateContextDescHeader, ffxStructType_t};

#[repr(C)]
pub struct ID3D12Device {
    _opaque: [u8; 0],
}

pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12: ffxStructType_t = 2;

#[repr(C)]
pub struct ffxCreateBackendDX12Desc {
    pub header: ffxCreateContextDescHeader,
    pub device: *mut ID3D12Device,
}
