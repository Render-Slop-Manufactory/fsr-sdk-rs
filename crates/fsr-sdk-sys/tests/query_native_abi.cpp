// SPDX-License-Identifier: MPL-2.0

// Independent pinned-header assertions; no copied native descriptor definitions.
#include <cstddef>
#include <type_traits>
#include "ffx_api.h"
#include "ffx_upscale.h"

#define LAYOUT(T, S, A) static_assert(sizeof(T) == S && alignof(T) == A)
#define FIELD(T, F, FT, O) \
    static_assert(std::is_same_v<decltype(T::F), FT>); \
    static_assert(offsetof(T, F) == O)

static_assert(FFX_API_DESC_TYPE_OVERRIDE_VERSION == 5);
static_assert(FFX_API_CONFIGURE_DESC_TYPE_UPSCALE_KEYVALUE == 0x10007);
static_assert(FFX_API_CONFIGURE_UPSCALE_KEY_FVELOCITYFACTOR == 0);
static_assert(FFX_API_CONFIGURE_UPSCALE_KEY_FREACTIVENESSSCALE == 1);
static_assert(FFX_API_CONFIGURE_UPSCALE_KEY_FSHADINGCHANGESCALE == 2);
static_assert(FFX_API_CONFIGURE_UPSCALE_KEY_FACCUMULATIONADDEDPERFRAME == 3);
static_assert(FFX_API_CONFIGURE_UPSCALE_KEY_FMINDISOCCLUSIONACCUMULATION == 4);
LAYOUT(ffxConfigureDescUpscaleKeyValue, 40, 8);
FIELD(ffxConfigureDescUpscaleKeyValue, header, ffxConfigureDescHeader, 0);
FIELD(ffxConfigureDescUpscaleKeyValue, key, uint64_t, 16);
FIELD(ffxConfigureDescUpscaleKeyValue, u64, uint64_t, 24);
FIELD(ffxConfigureDescUpscaleKeyValue, ptr, void*, 32);
LAYOUT(ffxOverrideVersion, 24, 8);
FIELD(ffxOverrideVersion, header, ffxApiHeader, 0);
FIELD(ffxOverrideVersion, versionId, uint64_t, 16);
LAYOUT(ffxQueryGetProviderVersion, 32, 8);
FIELD(ffxQueryGetProviderVersion, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryGetProviderVersion, versionId, uint64_t, 16);
FIELD(ffxQueryGetProviderVersion, versionName, const char*, 24);
LAYOUT(FfxApiEffectMemoryUsage, 16, 8);
FIELD(FfxApiEffectMemoryUsage, totalUsageInBytes, uint64_t, 0);
FIELD(FfxApiEffectMemoryUsage, aliasableUsageInBytes, uint64_t, 8);
LAYOUT(ffxQueryDescUpscaleGetGPUMemoryUsage, 24, 8);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsage, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsage, gpuMemoryUsageUpscaler, FfxApiEffectMemoryUsage*, 16);
LAYOUT(ffxQueryDescUpscaleGetGPUMemoryUsageV2, 56, 8);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, device, void*, 16);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, maxRenderSize, FfxApiDimensions2D, 24);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, maxUpscaleSize, FfxApiDimensions2D, 32);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, flags, uint32_t, 40);
FIELD(ffxQueryDescUpscaleGetGPUMemoryUsageV2, gpuMemoryUsageUpscaler, FfxApiEffectMemoryUsage*, 48);
LAYOUT(ffxQueryDescUpscaleGetResourceRequirements, 32, 8);
FIELD(ffxQueryDescUpscaleGetResourceRequirements, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetResourceRequirements, required_resources, uint64_t, 16);
FIELD(ffxQueryDescUpscaleGetResourceRequirements, optional_resources, uint64_t, 24);
static_assert(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION == 6);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE == 0x10008);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2 == 0x10009);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS == 0x1000a);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_COLOR == 1);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_DEPTH == 2);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_MV == 4);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE == 8);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK == 16);
static_assert(FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION == 32);

static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE == 0x10002);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE == 0x10003);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT == 0x10004);
static_assert(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET == 0x10005);
static_assert(FFX_UPSCALE_QUALITY_MODE_NATIVEAA == 0);
static_assert(FFX_UPSCALE_QUALITY_MODE_QUALITY == 1);
static_assert(FFX_UPSCALE_QUALITY_MODE_BALANCED == 2);
static_assert(FFX_UPSCALE_QUALITY_MODE_PERFORMANCE == 3);
static_assert(FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE == 4);
LAYOUT(ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode, 32, 8);
FIELD(ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode, qualityMode, uint32_t, 16);
FIELD(ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode, pOutUpscaleRatio, float*, 24);
LAYOUT(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, 48, 8);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, displayWidth, uint32_t, 16);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, displayHeight, uint32_t, 20);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, qualityMode, uint32_t, 24);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, pOutRenderWidth, uint32_t*, 32);
FIELD(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, pOutRenderHeight, uint32_t*, 40);
LAYOUT(ffxQueryDescUpscaleGetJitterPhaseCount, 32, 8);
FIELD(ffxQueryDescUpscaleGetJitterPhaseCount, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetJitterPhaseCount, renderWidth, uint32_t, 16);
FIELD(ffxQueryDescUpscaleGetJitterPhaseCount, displayWidth, uint32_t, 20);
FIELD(ffxQueryDescUpscaleGetJitterPhaseCount, pOutPhaseCount, int32_t*, 24);
LAYOUT(ffxQueryDescUpscaleGetJitterOffset, 40, 8);
FIELD(ffxQueryDescUpscaleGetJitterOffset, header, ffxQueryDescHeader, 0);
FIELD(ffxQueryDescUpscaleGetJitterOffset, index, int32_t, 16);
FIELD(ffxQueryDescUpscaleGetJitterOffset, phaseCount, int32_t, 20);
FIELD(ffxQueryDescUpscaleGetJitterOffset, pOutX, float*, 24);
FIELD(ffxQueryDescUpscaleGetJitterOffset, pOutY, float*, 32);
