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

// Handwritten Windows SDK pointer-payload ABI used by ffx_api_dx12.h. These
// metadata types own no COM references; native tests compare the system headers.
#[repr(C)]
pub struct ID3D12Resource {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct ID3D12Heap {
    _opaque: [u8; 0],
}
pub type D3D12_RESOURCE_STATES = i32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct D3D12_HEAP_PROPERTIES {
    pub Type: i32,
    pub CPUPageProperty: i32,
    pub MemoryPoolPreference: i32,
    pub CreationNodeMask: u32,
    pub VisibleNodeMask: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DXGI_SAMPLE_DESC {
    pub Count: u32,
    pub Quality: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct D3D12_RESOURCE_DESC {
    pub Dimension: i32,
    pub Alignment: u64,
    pub Width: u64,
    pub Height: u32,
    pub DepthOrArraySize: u16,
    pub MipLevels: u16,
    pub Format: i32,
    pub SampleDesc: DXGI_SAMPLE_DESC,
    pub Layout: i32,
    pub Flags: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct D3D12_DEPTH_STENCIL_VALUE {
    pub Depth: f32,
    pub Stencil: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union D3D12_CLEAR_VALUE_0 {
    pub Color: [f32; 4],
    pub DepthStencil: D3D12_DEPTH_STENCIL_VALUE,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct D3D12_CLEAR_VALUE {
    pub Format: i32,
    pub Anonymous: D3D12_CLEAR_VALUE_0,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct D3D12_HEAP_DESC {
    pub SizeInBytes: u64,
    pub Properties: D3D12_HEAP_PROPERTIES,
    pub Alignment: u64,
    pub Flags: i32,
}

pub type FfxResourceAllocatorFn = unsafe extern "C" fn(
    u32,
    D3D12_RESOURCE_STATES,
    *const D3D12_HEAP_PROPERTIES,
    *const D3D12_RESOURCE_DESC,
    *const crate::api::FfxApiResourceDescription,
    *const D3D12_CLEAR_VALUE,
    *mut *mut ID3D12Resource,
) -> u32;
pub type FfxResourceDeallocatorFn = unsafe extern "C" fn(u32, *mut ID3D12Resource) -> u32;
pub type FfxHeapAllocatorFn =
    unsafe extern "C" fn(u32, *const D3D12_HEAP_DESC, bool, *mut *mut ID3D12Heap, *mut u64) -> u32;
pub type FfxHeapDeallocatorFn = unsafe extern "C" fn(u32, *mut ID3D12Heap, u64, u64) -> u32;
pub type PfnFfxResourceAllocatorFunc = Option<FfxResourceAllocatorFn>;
pub type PfnFfxResourceDeallocatorFunc = Option<FfxResourceDeallocatorFn>;
pub type PfnFfxHeapAllocatorFunc = Option<FfxHeapAllocatorFn>;
pub type PfnFfxHeapDeallocatorFunc = Option<FfxHeapDeallocatorFn>;
pub const FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS: ffxStructType_t = 3;

/// Optional creation extension; omission does not reset prior module callbacks.
#[repr(C)]
pub struct ffxCreateBackendDX12AllocationCallbacksDesc {
    pub header: ffxCreateContextDescHeader,
    pub pfnFfxResourceAllocator: PfnFfxResourceAllocatorFunc,
    pub pfnFfxResourceDeallocator: PfnFfxResourceDeallocatorFunc,
    pub pfnFfxHeapAllocator: PfnFfxHeapAllocatorFunc,
    pub pfnFfxHeapDeallocator: PfnFfxHeapDeallocatorFunc,
    pub pfnFfxConstantBufferAllocator: crate::api::FfxApiConstantBufferAllocator,
}
