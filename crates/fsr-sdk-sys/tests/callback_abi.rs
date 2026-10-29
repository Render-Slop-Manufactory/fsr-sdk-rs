// SPDX-License-Identifier: MPL-2.0
//! Paired with callbacks_native_abi.cpp: production callback and payload ABI.
#![cfg(all(
    target_os = "windows",
    feature = "dx12",
    target_arch = "x86_64",
    target_env = "msvc"
))]
// Exact identity signatures intentionally spell out each native parameter.
#![allow(clippy::type_complexity)]
use core::{
    ffi::c_void,
    mem::{align_of, offset_of, size_of},
};
use fsr_sdk_sys::{api::*, dx12::*, upscale::*};
#[test]
fn layouts_and_fields() {
    assert_eq!(
        (
            size_of::<FfxApiConstantBufferAllocation>(),
            align_of::<FfxApiConstantBufferAllocation>()
        ),
        (56, 8)
    );
    assert_eq!(offset_of!(FfxApiConstantBufferAllocation, resource), 0);
    let _: fn(FfxApiConstantBufferAllocation) -> FfxApiResource = |v| v.resource;
    assert_eq!(offset_of!(FfxApiConstantBufferAllocation, handle), 48);
    let _: fn(FfxApiConstantBufferAllocation) -> u64 = |v| v.handle;
    assert_eq!(
        (
            size_of::<ffxConfigureDescGlobalDebug1>(),
            align_of::<ffxConfigureDescGlobalDebug1>()
        ),
        (32, 8)
    );
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug1, header), 0);
    let _: fn(ffxConfigureDescGlobalDebug1) -> ffxConfigureDescHeader = |v| v.header;
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug1, fpMessage), 16);
    let _: fn(ffxConfigureDescGlobalDebug1) -> ffxApiMessage = |v| v.fpMessage;
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug1, debugLevel), 24);
    let _: fn(ffxConfigureDescGlobalDebug1) -> u32 = |v| v.debugLevel;
    assert_eq!(
        (
            size_of::<ffxConfigureDescGlobalDebug>(),
            align_of::<ffxConfigureDescGlobalDebug>()
        ),
        (40, 8)
    );
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug, header), 0);
    let _: fn(ffxConfigureDescGlobalDebug) -> ffxConfigureDescHeader = |v| v.header;
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug, effectId), 16);
    let _: fn(ffxConfigureDescGlobalDebug) -> u64 = |v| v.effectId;
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug, fpMessage), 24);
    let _: fn(ffxConfigureDescGlobalDebug) -> ffxApiMessage = |v| v.fpMessage;
    assert_eq!(offset_of!(ffxConfigureDescGlobalDebug, debugLevel), 32);
    let _: fn(ffxConfigureDescGlobalDebug) -> u32 = |v| v.debugLevel;
    assert_eq!(
        (
            size_of::<ffxCreateBackendDX12AllocationCallbacksDesc>(),
            align_of::<ffxCreateBackendDX12AllocationCallbacksDesc>()
        ),
        (56, 8)
    );
    assert_eq!(
        offset_of!(ffxCreateBackendDX12AllocationCallbacksDesc, header),
        0
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> ffxCreateContextDescHeader =
        |v| v.header;
    assert_eq!(
        offset_of!(
            ffxCreateBackendDX12AllocationCallbacksDesc,
            pfnFfxResourceAllocator
        ),
        16
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> PfnFfxResourceAllocatorFunc =
        |v| v.pfnFfxResourceAllocator;
    assert_eq!(
        offset_of!(
            ffxCreateBackendDX12AllocationCallbacksDesc,
            pfnFfxResourceDeallocator
        ),
        24
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> PfnFfxResourceDeallocatorFunc =
        |v| v.pfnFfxResourceDeallocator;
    assert_eq!(
        offset_of!(
            ffxCreateBackendDX12AllocationCallbacksDesc,
            pfnFfxHeapAllocator
        ),
        32
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> PfnFfxHeapAllocatorFunc =
        |v| v.pfnFfxHeapAllocator;
    assert_eq!(
        offset_of!(
            ffxCreateBackendDX12AllocationCallbacksDesc,
            pfnFfxHeapDeallocator
        ),
        40
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> PfnFfxHeapDeallocatorFunc =
        |v| v.pfnFfxHeapDeallocator;
    assert_eq!(
        offset_of!(
            ffxCreateBackendDX12AllocationCallbacksDesc,
            pfnFfxConstantBufferAllocator
        ),
        48
    );
    let _: fn(ffxCreateBackendDX12AllocationCallbacksDesc) -> FfxApiConstantBufferAllocator =
        |v| v.pfnFfxConstantBufferAllocator;
    assert_eq!(
        (
            size_of::<D3D12_HEAP_PROPERTIES>(),
            align_of::<D3D12_HEAP_PROPERTIES>()
        ),
        (20, 4)
    );
    assert_eq!(offset_of!(D3D12_HEAP_PROPERTIES, Type), 0);
    let _: fn(D3D12_HEAP_PROPERTIES) -> i32 = |v| v.Type;
    assert_eq!(offset_of!(D3D12_HEAP_PROPERTIES, CPUPageProperty), 4);
    let _: fn(D3D12_HEAP_PROPERTIES) -> i32 = |v| v.CPUPageProperty;
    assert_eq!(offset_of!(D3D12_HEAP_PROPERTIES, MemoryPoolPreference), 8);
    let _: fn(D3D12_HEAP_PROPERTIES) -> i32 = |v| v.MemoryPoolPreference;
    assert_eq!(offset_of!(D3D12_HEAP_PROPERTIES, CreationNodeMask), 12);
    let _: fn(D3D12_HEAP_PROPERTIES) -> u32 = |v| v.CreationNodeMask;
    assert_eq!(offset_of!(D3D12_HEAP_PROPERTIES, VisibleNodeMask), 16);
    let _: fn(D3D12_HEAP_PROPERTIES) -> u32 = |v| v.VisibleNodeMask;
    assert_eq!(
        (
            size_of::<DXGI_SAMPLE_DESC>(),
            align_of::<DXGI_SAMPLE_DESC>()
        ),
        (8, 4)
    );
    assert_eq!(offset_of!(DXGI_SAMPLE_DESC, Count), 0);
    let _: fn(DXGI_SAMPLE_DESC) -> u32 = |v| v.Count;
    assert_eq!(offset_of!(DXGI_SAMPLE_DESC, Quality), 4);
    let _: fn(DXGI_SAMPLE_DESC) -> u32 = |v| v.Quality;
    assert_eq!(
        (
            size_of::<D3D12_RESOURCE_DESC>(),
            align_of::<D3D12_RESOURCE_DESC>()
        ),
        (56, 8)
    );
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Dimension), 0);
    let _: fn(D3D12_RESOURCE_DESC) -> i32 = |v| v.Dimension;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Alignment), 8);
    let _: fn(D3D12_RESOURCE_DESC) -> u64 = |v| v.Alignment;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Width), 16);
    let _: fn(D3D12_RESOURCE_DESC) -> u64 = |v| v.Width;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Height), 24);
    let _: fn(D3D12_RESOURCE_DESC) -> u32 = |v| v.Height;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, DepthOrArraySize), 28);
    let _: fn(D3D12_RESOURCE_DESC) -> u16 = |v| v.DepthOrArraySize;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, MipLevels), 30);
    let _: fn(D3D12_RESOURCE_DESC) -> u16 = |v| v.MipLevels;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Format), 32);
    let _: fn(D3D12_RESOURCE_DESC) -> i32 = |v| v.Format;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, SampleDesc), 36);
    let _: fn(D3D12_RESOURCE_DESC) -> DXGI_SAMPLE_DESC = |v| v.SampleDesc;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Layout), 44);
    let _: fn(D3D12_RESOURCE_DESC) -> i32 = |v| v.Layout;
    assert_eq!(offset_of!(D3D12_RESOURCE_DESC, Flags), 48);
    let _: fn(D3D12_RESOURCE_DESC) -> i32 = |v| v.Flags;
    assert_eq!(
        (
            size_of::<D3D12_DEPTH_STENCIL_VALUE>(),
            align_of::<D3D12_DEPTH_STENCIL_VALUE>()
        ),
        (8, 4)
    );
    assert_eq!(offset_of!(D3D12_DEPTH_STENCIL_VALUE, Depth), 0);
    let _: fn(D3D12_DEPTH_STENCIL_VALUE) -> f32 = |v| v.Depth;
    assert_eq!(offset_of!(D3D12_DEPTH_STENCIL_VALUE, Stencil), 4);
    let _: fn(D3D12_DEPTH_STENCIL_VALUE) -> u8 = |v| v.Stencil;
    assert_eq!(
        (
            size_of::<D3D12_CLEAR_VALUE>(),
            align_of::<D3D12_CLEAR_VALUE>()
        ),
        (20, 4)
    );
    assert_eq!(offset_of!(D3D12_CLEAR_VALUE, Format), 0);
    let _: fn(D3D12_CLEAR_VALUE) -> i32 = |v| v.Format;
    assert_eq!(
        (size_of::<D3D12_HEAP_DESC>(), align_of::<D3D12_HEAP_DESC>()),
        (48, 8)
    );
    assert_eq!(offset_of!(D3D12_HEAP_DESC, SizeInBytes), 0);
    let _: fn(D3D12_HEAP_DESC) -> u64 = |v| v.SizeInBytes;
    assert_eq!(offset_of!(D3D12_HEAP_DESC, Properties), 8);
    let _: fn(D3D12_HEAP_DESC) -> D3D12_HEAP_PROPERTIES = |v| v.Properties;
    assert_eq!(offset_of!(D3D12_HEAP_DESC, Alignment), 32);
    let _: fn(D3D12_HEAP_DESC) -> u64 = |v| v.Alignment;
    assert_eq!(offset_of!(D3D12_HEAP_DESC, Flags), 40);
    let _: fn(D3D12_HEAP_DESC) -> i32 = |v| v.Flags;
    assert_eq!(
        (
            size_of::<D3D12_CLEAR_VALUE_0>(),
            align_of::<D3D12_CLEAR_VALUE_0>()
        ),
        (16, 4)
    );
    assert_eq!(offset_of!(D3D12_CLEAR_VALUE, Anonymous), 4);
}
#[test]
fn constants() {
    assert_eq!(FFX_API_RESOURCE_TYPE_BUFFER, 0);
    assert_eq!(FFX_API_RESOURCE_STATE_GENERIC_READ, 20);
    assert_eq!(
        (
            FFX_API_MESSAGE_TYPE_ERROR,
            FFX_API_MESSAGE_TYPE_WARNING,
            FFX_API_MESSAGE_TYPE_COUNT
        ),
        (0, 1, 2)
    );
    assert_eq!(FFX_API_EFFECT_ID_UPSCALE, 65536);
    assert_eq!(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1, 1);
    assert_eq!(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG, 7);
    assert_eq!(
        FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS,
        3
    );
    assert_eq!(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_SILENCE, 0);
    assert_eq!(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_ERRORS, 1);
    assert_eq!(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_WARNINGS, 2);
    assert_eq!(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE, 268435455);
    assert_eq!(FFX_UPSCALE_ENABLE_DEBUG_CHECKING, 128);
    assert_eq!(FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION, 512);
    assert_eq!(FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW, 1);
}
const _: fn(ffxAlloc) -> Option<unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void> = |v| v;
const _: fn(ffxDealloc) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> ()> = |v| v;
const _: fn(ffxApiMessage) -> Option<unsafe extern "C" fn(u32, *const u16) -> ()> = |v| v;
const _: fn(
    FfxApiConstantBufferAllocator,
) -> Option<unsafe extern "C" fn(*mut c_void, u64) -> FfxApiConstantBufferAllocation> = |v| v;
const _: fn(
    PfnFfxResourceAllocatorFunc,
) -> Option<
    unsafe extern "C" fn(
        u32,
        D3D12_RESOURCE_STATES,
        *const D3D12_HEAP_PROPERTIES,
        *const D3D12_RESOURCE_DESC,
        *const FfxApiResourceDescription,
        *const D3D12_CLEAR_VALUE,
        *mut *mut ID3D12Resource,
    ) -> u32,
> = |v| v;
const _: fn(
    PfnFfxResourceDeallocatorFunc,
) -> Option<unsafe extern "C" fn(u32, *mut ID3D12Resource) -> u32> = |v| v;
const _: fn(
    PfnFfxHeapAllocatorFunc,
) -> Option<
    unsafe extern "C" fn(u32, *const D3D12_HEAP_DESC, bool, *mut *mut ID3D12Heap, *mut u64) -> u32,
> = |v| v;
const _: fn(
    PfnFfxHeapDeallocatorFunc,
) -> Option<unsafe extern "C" fn(u32, *mut ID3D12Heap, u64, u64) -> u32> = |v| v;
