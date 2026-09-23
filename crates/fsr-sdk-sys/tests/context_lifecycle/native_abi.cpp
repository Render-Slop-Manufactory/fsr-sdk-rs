// SPDX-License-Identifier: MPL-2.0

// Paired production creation ABI and test-only provider query check.
// AMD declarations retain ../../LICENSE-AMD.
// Also recheck the common ABI and existing entry-point signatures.
#include "../native_abi.cpp"
#include "dx12/ffx_api_dx12.h"

#define LAYOUT(T, S, A) static_assert(sizeof(T) == S && alignof(T) == A)
#define FIELD(T, F, FT, O) \
    static_assert(std::is_same_v<decltype(T::F), FT>); \
    static_assert(offsetof(T, F) == O)

static_assert(sizeof(wchar_t) == 2 && alignof(wchar_t) == 2);
static_assert(std::is_unsigned_v<wchar_t>);
static_assert(std::is_same_v<ffxApiMessage, void (__cdecl *)(uint32_t, const wchar_t*)>);
LAYOUT(ffxApiMessage, 8, 8);
LAYOUT(FfxApiDimensions2D, 8, 4);
FIELD(FfxApiDimensions2D, width, uint32_t, 0);
FIELD(FfxApiDimensions2D, height, uint32_t, 4);
LAYOUT(ffxCreateContextDescUpscale, 48, 8);
FIELD(ffxCreateContextDescUpscale, header, ffxCreateContextDescHeader, 0);
FIELD(ffxCreateContextDescUpscale, flags, uint32_t, 16);
FIELD(ffxCreateContextDescUpscale, maxRenderSize, FfxApiDimensions2D, 20);
FIELD(ffxCreateContextDescUpscale, maxUpscaleSize, FfxApiDimensions2D, 28);
FIELD(ffxCreateContextDescUpscale, fpMessage, ffxApiMessage, 40);
LAYOUT(ffxCreateContextDescUpscaleVersion, 24, 8);
FIELD(ffxCreateContextDescUpscaleVersion, header, ffxCreateContextDescHeader, 0);
FIELD(ffxCreateContextDescUpscaleVersion, version, uint32_t, 16);
LAYOUT(ffxCreateBackendDX12Desc, 24, 8);
FIELD(ffxCreateBackendDX12Desc, header, ffxCreateContextDescHeader, 0);
FIELD(ffxCreateBackendDX12Desc, device, ID3D12Device*, 16);
LAYOUT(ffxQueryGetProviderVersion, 32, 8);
FIELD(ffxQueryGetProviderVersion, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryGetProviderVersion, versionId, uint64_t, 16);
FIELD(ffxQueryGetProviderVersion, versionName, const char*, 24);
static_assert(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION == 0x0001000bu);
static_assert(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12 == 2u);
static_assert(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION == 6u);
static_assert(FFX_UPSCALER_VERSION == ((4u << 22) | (1u << 12) | 1u));
