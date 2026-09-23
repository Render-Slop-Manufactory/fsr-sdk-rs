// SPDX-License-Identifier: MPL-2.0

//! Paired with native_abi.cpp; numeric layouts cover the initial Windows x64 target.
use core::ffi::c_void;
#[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
use core::mem::offset_of;
use core::mem::{align_of, size_of};
use fsr_sdk_sys::api::*;
use fsr_sdk_sys::upscale::FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE;

const _: fn(ffxQueryDescGetVersions) -> ffxQueryDescHeader = |q| q.header;
const _: fn(ffxQueryDescGetVersions) -> u64 = |q| q.createDescType;
const _: fn(ffxQueryDescGetVersions) -> *mut c_void = |q| q.device;
const _: fn(ffxQueryDescGetVersions) -> *mut u64 = |q| q.outputCount;
const _: fn(ffxQueryDescGetVersions) -> *mut u64 = |q| q.versionIds;
const _: fn(ffxQueryDescGetVersions) -> *mut *const core::ffi::c_char = |q| q.versionNames;

#[test]
fn version_query_tags() {
    assert_eq!(FFX_API_QUERY_DESC_TYPE_GET_VERSIONS, 4);
    assert_eq!(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE, 0x0001_0000);
}

#[test]
#[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
fn version_query_windows_x64_layout() {
    assert_eq!(size_of::<ffxQueryDescGetVersions>(), 56);
    assert_eq!(align_of::<ffxQueryDescGetVersions>(), 8);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, header), 0);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, createDescType), 16);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, device), 24);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, outputCount), 32);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, versionIds), 40);
    assert_eq!(offset_of!(ffxQueryDescGetVersions, versionNames), 48);
}

// Identity functions enforce exact type equality, including pointer mutability,
// integer signedness, nullability, unsafety and the calling convention.
const _: fn(ffxAlloc) -> Option<unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void> =
    |value| value;
const _: fn(ffxDealloc) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> ()> =
    |value| value;
const _: fn(
    PfnFfxCreateContext,
) -> Option<
    unsafe extern "C" fn(
        *mut ffxContext,
        *mut ffxCreateContextDescHeader,
        *const ffxAllocationCallbacks,
    ) -> u32,
> = |value| value;
const _: fn(
    PfnFfxDestroyContext,
)
    -> Option<unsafe extern "C" fn(*mut ffxContext, *const ffxAllocationCallbacks) -> u32> =
    |value| value;
const _: fn(
    PfnFfxConfigure,
)
    -> Option<unsafe extern "C" fn(*mut ffxContext, *const ffxConfigureDescHeader) -> u32> =
    |value| value;
const _: fn(
    PfnFfxQuery,
) -> Option<unsafe extern "C" fn(*mut ffxContext, *mut ffxQueryDescHeader) -> u32> = |value| value;
const _: fn(
    PfnFfxDispatch,
) -> Option<unsafe extern "C" fn(*mut ffxContext, *const ffxDispatchDescHeader) -> u32> =
    |value| value;
const _: fn(ffxContext) -> *mut c_void = |value| value;
const _: fn(ffxReturnCode_t) -> u32 = |value| value;
const _: fn(ffxStructType_t) -> u64 = |value| value;
const _: fn(ffxCreateContextDescHeader) -> ffxApiHeader = |value| value;
const _: fn(ffxConfigureDescHeader) -> ffxApiHeader = |value| value;
const _: fn(ffxQueryDescHeader) -> ffxApiHeader = |value| value;
const _: fn(ffxDispatchDescHeader) -> ffxApiHeader = |value| value;

const _: fn(ffxApiHeader) -> (u64, *mut ffxApiHeader) = |h| (h.r#type, h.pNext);
const _: fn(ffxAllocationCallbacks) -> (*mut c_void, ffxAlloc, ffxDealloc) =
    |c| (c.pUserData, c.alloc, c.dealloc);

#[test]
fn return_codes() {
    assert_eq!(FFX_API_RETURN_OK, 0);
    assert_eq!(FFX_API_RETURN_ERROR, 1);
    assert_eq!(FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE, 2);
    assert_eq!(FFX_API_RETURN_ERROR_RUNTIME_ERROR, 3);
    assert_eq!(FFX_API_RETURN_NO_PROVIDER, 4);
    assert_eq!(FFX_API_RETURN_ERROR_MEMORY, 5);
    assert_eq!(FFX_API_RETURN_ERROR_PARAMETER, 6);
    assert_eq!(FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE, 7);
}

#[test]
fn nullable_function_pointer_layouts() {
    assert_eq!(size_of::<ffxAlloc>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<ffxAlloc>(), align_of::<*mut c_void>());
    assert_eq!(size_of::<ffxDealloc>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<ffxDealloc>(), align_of::<*mut c_void>());
    assert_eq!(size_of::<PfnFfxCreateContext>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<PfnFfxCreateContext>(), align_of::<*mut c_void>());
    assert_eq!(size_of::<PfnFfxDestroyContext>(), size_of::<*mut c_void>());
    assert_eq!(
        align_of::<PfnFfxDestroyContext>(),
        align_of::<*mut c_void>()
    );
    assert_eq!(size_of::<PfnFfxConfigure>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<PfnFfxConfigure>(), align_of::<*mut c_void>());
    assert_eq!(size_of::<PfnFfxQuery>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<PfnFfxQuery>(), align_of::<*mut c_void>());
    assert_eq!(size_of::<PfnFfxDispatch>(), size_of::<*mut c_void>());
    assert_eq!(align_of::<PfnFfxDispatch>(), align_of::<*mut c_void>());
}

#[test]
#[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
fn windows_x64_layouts() {
    assert_eq!((size_of::<ffxContext>(), align_of::<ffxContext>()), (8, 8));
    assert_eq!(
        (size_of::<ffxReturnCode_t>(), align_of::<ffxReturnCode_t>()),
        (4, 4)
    );
    assert_eq!(
        (size_of::<ffxStructType_t>(), align_of::<ffxStructType_t>()),
        (8, 8)
    );
    assert_eq!(
        (size_of::<ffxApiHeader>(), align_of::<ffxApiHeader>()),
        (16, 8)
    );
    assert_eq!(offset_of!(ffxApiHeader, r#type), 0);
    assert_eq!(offset_of!(ffxApiHeader, pNext), 8);
    assert_eq!(
        (
            size_of::<ffxAllocationCallbacks>(),
            align_of::<ffxAllocationCallbacks>()
        ),
        (24, 8)
    );
    assert_eq!(offset_of!(ffxAllocationCallbacks, pUserData), 0);
    assert_eq!(offset_of!(ffxAllocationCallbacks, alloc), 8);
    assert_eq!(offset_of!(ffxAllocationCallbacks, dealloc), 16);
}

#[test]
fn creation_constants_and_dimension_field_types() {
    use fsr_sdk_sys::upscale::{
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION, FFX_UPSCALER_VERSION,
    };
    assert_eq!(
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION,
        0x0001_000b
    );
    assert_eq!(FFX_UPSCALER_VERSION, 0x0100_1001);
    let _: fn(FfxApiDimensions2D) -> (u32, u32) = |d| (d.width, d.height);
}
