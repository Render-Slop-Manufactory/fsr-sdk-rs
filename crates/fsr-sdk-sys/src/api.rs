// SPDX-License-Identifier: MPL-2.0

//! Minimal common ABI from AMD FidelityFX SDK v2.3.0, `api/include/ffx_api.h`.
//!
//! These are raw declarations, not a loader or a safe calling interface. Native
//! pointer validity, descriptor lifetimes and allocator compatibility remain the
//! caller's responsibility. The header uses C calling conventions (no stdcall).
//! Nullable C function pointers are represented by `Option<unsafe extern "C" fn>`;
//! representability of null does not imply that an SDK operation accepts it.
//!
//! Derived declarations retain AMD's notice in `../LICENSE-AMD`.

use core::ffi::{c_char, c_void};

pub type ffxContext = *mut c_void;
pub type ffxReturnCode_t = u32;
pub type ffxStructType_t = u64;

// The native functions return uint32_t, not the FfxApiReturnCodes enum.
pub const FFX_API_RETURN_OK: ffxReturnCode_t = 0;
pub const FFX_API_RETURN_ERROR: ffxReturnCode_t = 1;
pub const FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE: ffxReturnCode_t = 2;
pub const FFX_API_RETURN_ERROR_RUNTIME_ERROR: ffxReturnCode_t = 3;
pub const FFX_API_RETURN_NO_PROVIDER: ffxReturnCode_t = 4;
pub const FFX_API_RETURN_ERROR_MEMORY: ffxReturnCode_t = 5;
pub const FFX_API_RETURN_ERROR_PARAMETER: ffxReturnCode_t = 6;
pub const FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE: ffxReturnCode_t = 7;

#[repr(C)]
pub struct ffxApiHeader {
    /// Native `type` field; must identify the enclosing descriptor.
    pub r#type: ffxStructType_t,
    /// Optional descriptor chain; null terminates the chain.
    pub pNext: *mut ffxApiHeader,
}

pub type ffxCreateContextDescHeader = ffxApiHeader;
pub type ffxConfigureDescHeader = ffxApiHeader;
pub type ffxQueryDescHeader = ffxApiHeader;
pub type ffxDispatchDescHeader = ffxApiHeader;

/// `api/include/ffx_api.h`: global version enumeration descriptor tag.
pub const FFX_API_QUERY_DESC_TYPE_GET_VERSIONS: ffxStructType_t = 4;

/// Raw global version query from SDK v2.3.0 `api/include/ffx_api.h`.
#[repr(C)]
pub struct ffxQueryDescGetVersions {
    pub header: ffxQueryDescHeader,
    pub createDescType: u64,
    pub device: *mut c_void,
    /// Input array capacity; output count. Zero requests the available count.
    pub outputCount: *mut u64,
    /// Optional writable ID array. Both arrays null requests only the count.
    pub versionIds: *mut u64,
    /// Optional writable array of pointers to native, read-only names.
    pub versionNames: *mut *const c_char,
}

/// Select an ID returned by provider enumeration (`ffx_api.h`).
pub const FFX_API_DESC_TYPE_OVERRIDE_VERSION: ffxStructType_t = 5;

#[repr(C)]
pub struct ffxOverrideVersion {
    pub header: ffxApiHeader,
    pub versionId: u64,
}

pub const FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION: ffxStructType_t = 6;

/// Live provider metadata from SDK v2.3.0 `api/include/ffx_api.h`.
#[repr(C)]
pub struct ffxQueryGetProviderVersion {
    pub header: ffxQueryDescHeader,
    pub versionId: u64,
    pub versionName: *const c_char,
}

/// Memory accounting from SDK v2.3.0 `api/include/ffx_api_types.h`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FfxApiEffectMemoryUsage {
    pub totalUsageInBytes: u64,
    pub aliasableUsageInBytes: u64,
}

/// Allocation must hold at least `size` bytes aligned for any type; null means failure.
/// Callbacks must not unwind across the native boundary.
pub type ffxAllocFn = unsafe extern "C" fn(pUserData: *mut c_void, size: u64) -> *mut c_void;
pub type ffxAlloc = Option<ffxAllocFn>;
/// Deallocation must accept a null `pMem` and must not unwind.
pub type ffxDeallocFn = unsafe extern "C" fn(pUserData: *mut c_void, pMem: *mut c_void);
pub type ffxDealloc = Option<ffxDeallocFn>;

#[repr(C)]
pub struct ffxAllocationCallbacks {
    pub pUserData: *mut c_void,
    pub alloc: ffxAlloc,
    pub dealloc: ffxDealloc,
}

/// Descriptor pointers must remain live until context destruction. A null `memCb`
/// selects the native system allocator.
pub type PfnFfxCreateContext = Option<
    unsafe extern "C" fn(
        context: *mut ffxContext,
        desc: *mut ffxCreateContextDescHeader,
        memCb: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t,
>;
/// Allocation callbacks must be compatible with those used during creation.
pub type PfnFfxDestroyContext = Option<
    unsafe extern "C" fn(
        context: *mut ffxContext,
        memCb: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t,
>;
/// A null context pointer selects global configuration.
pub type PfnFfxConfigure = Option<
    unsafe extern "C" fn(
        context: *mut ffxContext,
        desc: *const ffxConfigureDescHeader,
    ) -> ffxReturnCode_t,
>;
/// A null context pointer selects a global query.
pub type PfnFfxQuery = Option<
    unsafe extern "C" fn(
        context: *mut ffxContext,
        desc: *mut ffxQueryDescHeader,
    ) -> ffxReturnCode_t,
>;
pub type PfnFfxDispatch = Option<
    unsafe extern "C" fn(
        context: *mut ffxContext,
        desc: *const ffxDispatchDescHeader,
    ) -> ffxReturnCode_t,
>;

/// Dimensions from SDK v2.3.0 `api/include/ffx_api_types.h`.
#[repr(C)]
pub struct FfxApiDimensions2D {
    pub width: u32,
    pub height: u32,
}

/// Selected resource ABI from SDK v2.3.0 `api/include/ffx_api_types.h`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfxApiFloatCoords2D {
    pub x: f32,
    pub y: f32,
}

pub const FFX_API_RESOURCE_TYPE_BUFFER: u32 = 0;
pub const FFX_API_RESOURCE_TYPE_TEXTURE2D: u32 = 2;
pub const FFX_API_RESOURCE_FLAGS_NONE: u32 = 0;
pub const FFX_API_RESOURCE_USAGE_READ_ONLY: u32 = 0;
pub const FFX_API_RESOURCE_USAGE_UAV: u32 = 1 << 1;
pub const FFX_API_RESOURCE_STATE_COMPUTE_READ: u32 = 1 << 2;
pub const FFX_API_RESOURCE_STATE_GENERIC_READ: u32 = (1 << 4) | (1 << 2);
pub const FFX_API_RESOURCE_STATE_UNORDERED_ACCESS: u32 = 1 << 1;
pub const FFX_API_SURFACE_FORMAT_UNKNOWN: u32 = 0;
pub const FFX_API_SURFACE_FORMAT_R32G32B32A32_TYPELESS: u32 = 1;
pub const FFX_API_SURFACE_FORMAT_R32G32B32A32_UINT: u32 = 2;
pub const FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT: u32 = 3;
pub const FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT: u32 = 4;
pub const FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT: u32 = 5;
pub const FFX_API_SURFACE_FORMAT_R32G32_FLOAT: u32 = 6;
pub const FFX_API_SURFACE_FORMAT_R8_UINT: u32 = 7;
pub const FFX_API_SURFACE_FORMAT_R32_UINT: u32 = 8;
pub const FFX_API_SURFACE_FORMAT_R8G8B8A8_TYPELESS: u32 = 9;
pub const FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM: u32 = 10;
pub const FFX_API_SURFACE_FORMAT_R8G8B8A8_SNORM: u32 = 11;
pub const FFX_API_SURFACE_FORMAT_R8G8B8A8_SRGB: u32 = 12;
pub const FFX_API_SURFACE_FORMAT_B8G8R8A8_TYPELESS: u32 = 13;
pub const FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM: u32 = 14;
pub const FFX_API_SURFACE_FORMAT_B8G8R8A8_SRGB: u32 = 15;
pub const FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT: u32 = 16;
pub const FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM: u32 = 17;
pub const FFX_API_SURFACE_FORMAT_R16G16_FLOAT: u32 = 18;
pub const FFX_API_SURFACE_FORMAT_R16G16_UINT: u32 = 19;
pub const FFX_API_SURFACE_FORMAT_R16G16_SINT: u32 = 20;
pub const FFX_API_SURFACE_FORMAT_R16_FLOAT: u32 = 21;
pub const FFX_API_SURFACE_FORMAT_R16_UINT: u32 = 22;
pub const FFX_API_SURFACE_FORMAT_R16_UNORM: u32 = 23;
pub const FFX_API_SURFACE_FORMAT_R16_SNORM: u32 = 24;
pub const FFX_API_SURFACE_FORMAT_R8_UNORM: u32 = 25;
pub const FFX_API_SURFACE_FORMAT_R8G8_UNORM: u32 = 26;
pub const FFX_API_SURFACE_FORMAT_R8G8_UINT: u32 = 27;
pub const FFX_API_SURFACE_FORMAT_R32_FLOAT: u32 = 28;
pub const FFX_API_SURFACE_FORMAT_R9G9B9E5_SHAREDEXP: u32 = 29;
pub const FFX_API_SURFACE_FORMAT_R16G16B16A16_TYPELESS: u32 = 30;
pub const FFX_API_SURFACE_FORMAT_R32G32_TYPELESS: u32 = 31;
pub const FFX_API_SURFACE_FORMAT_R10G10B10A2_TYPELESS: u32 = 32;
pub const FFX_API_SURFACE_FORMAT_R16G16_TYPELESS: u32 = 33;
pub const FFX_API_SURFACE_FORMAT_R16_TYPELESS: u32 = 34;
pub const FFX_API_SURFACE_FORMAT_R8_TYPELESS: u32 = 35;
pub const FFX_API_SURFACE_FORMAT_R8G8_TYPELESS: u32 = 36;
pub const FFX_API_SURFACE_FORMAT_R32_TYPELESS: u32 = 37;
pub const FFX_API_SURFACE_FORMAT_R32G32_UINT: u32 = 38;
pub const FFX_API_SURFACE_FORMAT_R8_SNORM: u32 = 39;
pub const FFX_API_RESOURCE_USAGE_DEPTHTARGET: u32 = 1 << 2;
pub const FFX_API_RESOURCE_USAGE_STENCILTARGET: u32 = 1 << 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub union FfxApiResourceWidth {
    pub width: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FfxApiResourceHeight {
    pub height: u32,
    pub stride: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FfxApiResourceDepth {
    pub depth: u32,
    pub alignment: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct FfxApiResourceDescription {
    pub r#type: u32,
    pub format: u32,
    pub width: FfxApiResourceWidth,
    pub height: FfxApiResourceHeight,
    pub depth: FfxApiResourceDepth,
    pub mipCount: u32,
    pub flags: u32,
    pub usage: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfxApiResource {
    pub resource: *mut c_void,
    pub description: FfxApiResourceDescription,
    pub state: u32,
}

/// Windows message callback from `api/include/ffx_api.h` (16-bit wchar_t).
#[cfg(windows)]
pub type ffxApiMessageFn = unsafe extern "C" fn(u32, *const u16);
#[cfg(windows)]
pub type ffxApiMessage = Option<ffxApiMessageFn>;

pub const FFX_API_EFFECT_ID_UPSCALE: u64 = 0x0001_0000;
pub const FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1: ffxStructType_t = 1;
pub const FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG: ffxStructType_t = 7;
pub const FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_SILENCE: u32 = 0;
pub const FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_ERRORS: u32 = 1;
pub const FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_WARNINGS: u32 = 2;
pub const FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE: u32 = 0x0fff_ffff;
pub const FFX_API_MESSAGE_TYPE_ERROR: u32 = 0;
pub const FFX_API_MESSAGE_TYPE_WARNING: u32 = 1;
pub const FFX_API_MESSAGE_TYPE_COUNT: u32 = 2;

/// Deprecated global diagnostic descriptor; callback uses Windows wchar_t.
#[cfg(windows)]
#[repr(C)]
pub struct ffxConfigureDescGlobalDebug1 {
    pub header: ffxConfigureDescHeader,
    pub fpMessage: ffxApiMessage,
    pub debugLevel: u32,
}

/// Effect-scoped diagnostic request; actual routing depends on the runtime.
#[cfg(windows)]
#[repr(C)]
pub struct ffxConfigureDescGlobalDebug {
    pub header: ffxConfigureDescHeader,
    pub effectId: u64,
    pub fpMessage: ffxApiMessage,
    pub debugLevel: u32,
}

/// Callback result from `api/include/ffx_api_types.h`; contains no Rust owner.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfxApiConstantBufferAllocation {
    pub resource: FfxApiResource,
    pub handle: u64,
}

/// `data` contains payload bytes, not callback userdata. Returns by value.
pub type FfxApiConstantBufferAllocatorFn =
    unsafe extern "C" fn(data: *mut c_void, dataSize: u64) -> FfxApiConstantBufferAllocation;
pub type FfxApiConstantBufferAllocator = Option<FfxApiConstantBufferAllocatorFn>;
