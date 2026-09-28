// SPDX-License-Identifier: MPL-2.0

// Compile-only check against the local SDK; no linking or runtime calls.
#include <cstddef>
#include <cstdint>
#include <type_traits>
#include "ffx_api.h"

#if !defined(_MSC_VER) || !defined(_M_X64)
#error This layout check targets Windows x64 MSVC.
#endif

static_assert(std::is_same_v<ffxAlloc, void* (__cdecl *)(void*, uint64_t)>);
static_assert(std::is_same_v<ffxDealloc, void (__cdecl *)(void*, void*)>);
static_assert(std::is_same_v<PfnFfxCreateContext, uint32_t (__cdecl *)(void**, ffxApiHeader*, const ffxAllocationCallbacks*)>);
static_assert(std::is_same_v<PfnFfxDestroyContext, uint32_t (__cdecl *)(void**, const ffxAllocationCallbacks*)>);
static_assert(std::is_same_v<PfnFfxConfigure, uint32_t (__cdecl *)(void**, const ffxApiHeader*)>);
static_assert(std::is_same_v<PfnFfxQuery, uint32_t (__cdecl *)(void**, ffxApiHeader*)>);
static_assert(std::is_same_v<PfnFfxDispatch, uint32_t (__cdecl *)(void**, const ffxApiHeader*)>);
static_assert(std::is_same_v<decltype(&ffxCreateContext), PfnFfxCreateContext>);
static_assert(std::is_same_v<decltype(&ffxDestroyContext), PfnFfxDestroyContext>);
static_assert(std::is_same_v<decltype(&ffxConfigure), PfnFfxConfigure>);
static_assert(std::is_same_v<decltype(&ffxQuery), PfnFfxQuery>);
static_assert(std::is_same_v<decltype(&ffxDispatch), PfnFfxDispatch>);
static_assert(std::is_same_v<ffxContext, void*>);
static_assert(std::is_same_v<ffxReturnCode_t, uint32_t>);
static_assert(std::is_same_v<ffxStructType_t, uint64_t>);
static_assert(std::is_same_v<ffxCreateContextDescHeader, ffxApiHeader>);
static_assert(std::is_same_v<ffxConfigureDescHeader, ffxApiHeader>);
static_assert(std::is_same_v<ffxQueryDescHeader, ffxApiHeader>);
static_assert(std::is_same_v<ffxDispatchDescHeader, ffxApiHeader>);
static_assert(std::is_same_v<decltype(ffxApiHeader::type), uint64_t>);
static_assert(std::is_same_v<decltype(ffxApiHeader::pNext), ffxApiHeader*>);
static_assert(std::is_same_v<decltype(ffxAllocationCallbacks::pUserData), void*>);
static_assert(std::is_same_v<decltype(ffxAllocationCallbacks::alloc), ffxAlloc>);
static_assert(std::is_same_v<decltype(ffxAllocationCallbacks::dealloc), ffxDealloc>);
static_assert(sizeof(ffxContext) == 8 && alignof(ffxContext) == 8);
static_assert(sizeof(ffxReturnCode_t) == 4 && alignof(ffxReturnCode_t) == 4);
static_assert(sizeof(ffxStructType_t) == 8 && alignof(ffxStructType_t) == 8);
static_assert(sizeof(ffxApiHeader) == 16 && alignof(ffxApiHeader) == 8);
static_assert(sizeof(ffxAllocationCallbacks) == 24 && alignof(ffxAllocationCallbacks) == 8);
static_assert(sizeof(ffxAlloc) == 8 && alignof(ffxAlloc) == 8);
static_assert(sizeof(ffxDealloc) == 8 && alignof(ffxDealloc) == 8);
static_assert(sizeof(PfnFfxCreateContext) == 8 && alignof(PfnFfxCreateContext) == 8);
static_assert(sizeof(PfnFfxDestroyContext) == 8 && alignof(PfnFfxDestroyContext) == 8);
static_assert(sizeof(PfnFfxConfigure) == 8 && alignof(PfnFfxConfigure) == 8);
static_assert(sizeof(PfnFfxQuery) == 8 && alignof(PfnFfxQuery) == 8);
static_assert(sizeof(PfnFfxDispatch) == 8 && alignof(PfnFfxDispatch) == 8);
static_assert(offsetof(ffxApiHeader, type) == 0);
static_assert(offsetof(ffxApiHeader, pNext) == 8);
static_assert(offsetof(ffxAllocationCallbacks, pUserData) == 0);
static_assert(offsetof(ffxAllocationCallbacks, alloc) == 8);
static_assert(offsetof(ffxAllocationCallbacks, dealloc) == 16);
static_assert(FFX_API_RETURN_OK == 0);
static_assert(FFX_API_RETURN_ERROR == 1);
static_assert(FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE == 2);
static_assert(FFX_API_RETURN_ERROR_RUNTIME_ERROR == 3);
static_assert(FFX_API_RETURN_NO_PROVIDER == 4);
static_assert(FFX_API_RETURN_ERROR_MEMORY == 5);
static_assert(FFX_API_RETURN_ERROR_PARAMETER == 6);
static_assert(FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE == 7);

// Effect tag used to select the count-only version query.
#include "../../upscalers/include/ffx_upscale.h"
static_assert(FFX_API_QUERY_DESC_TYPE_GET_VERSIONS == 4u);
static_assert(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE == 0x00010000u);
static_assert(sizeof(ffxQueryDescGetVersions) == 56);
static_assert(alignof(ffxQueryDescGetVersions) == 8);
static_assert(offsetof(ffxQueryDescGetVersions, header) == 0);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::header), ffxQueryDescHeader>);
static_assert(offsetof(ffxQueryDescGetVersions, createDescType) == 16);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::createDescType), uint64_t>);
static_assert(offsetof(ffxQueryDescGetVersions, device) == 24);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::device), void*>);
static_assert(offsetof(ffxQueryDescGetVersions, outputCount) == 32);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::outputCount), uint64_t*>);
static_assert(offsetof(ffxQueryDescGetVersions, versionIds) == 40);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::versionIds), uint64_t*>);
static_assert(offsetof(ffxQueryDescGetVersions, versionNames) == 48);
static_assert(std::is_same_v<decltype(ffxQueryDescGetVersions::versionNames), const char**>);
