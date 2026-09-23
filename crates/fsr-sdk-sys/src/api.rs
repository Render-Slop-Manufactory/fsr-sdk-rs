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

/// Allocation must hold at least `size` bytes aligned for any type; null means failure.
/// Callbacks must not unwind across the native boundary.
pub type ffxAlloc = Option<unsafe extern "C" fn(pUserData: *mut c_void, size: u64) -> *mut c_void>;
/// Deallocation must accept a null `pMem` and must not unwind.
pub type ffxDealloc = Option<unsafe extern "C" fn(pUserData: *mut c_void, pMem: *mut c_void)>;

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

/// Windows message callback from `api/include/ffx_api.h` (16-bit wchar_t).
#[cfg(windows)]
pub type ffxApiMessage = Option<unsafe extern "C" fn(u32, *const u16)>;
