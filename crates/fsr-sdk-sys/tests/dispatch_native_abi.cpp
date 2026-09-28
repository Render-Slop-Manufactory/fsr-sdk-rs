// SPDX-License-Identifier: MPL-2.0

// Compile only against the pinned v2.3.0 headers; no SDK binary is linked.
#include <cstddef>
#include <cstdint>
#include <type_traits>
#include "ffx_upscale.h"

#if !defined(_MSC_VER) || !defined(_M_X64)
#error This check targets Windows x64 MSVC.
#endif

static_assert(std::is_same_v<decltype(&ffxDispatch), PfnFfxDispatch>);
static_assert(std::is_same_v<PfnFfxDispatch, uint32_t (__cdecl *)(void**, const ffxApiHeader*)>);
static_assert(sizeof(bool) == 1);
static_assert(FFX_API_DISPATCH_DESC_TYPE_UPSCALE == 0x00010001u);
static_assert(FFX_API_RESOURCE_TYPE_TEXTURE2D == 2);
static_assert(FFX_API_RESOURCE_FLAGS_NONE == 0);
static_assert(FFX_API_RESOURCE_USAGE_READ_ONLY == 0);
static_assert(FFX_API_RESOURCE_USAGE_UAV == 2);
static_assert(FFX_API_RESOURCE_STATE_COMPUTE_READ == 4);
static_assert(FFX_API_RESOURCE_STATE_UNORDERED_ACCESS == 2);
static_assert(FFX_API_SURFACE_FORMAT_UNKNOWN == 0);
static_assert(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT == 4);
static_assert(FFX_API_SURFACE_FORMAT_R16G16_FLOAT == 18);
static_assert(FFX_API_SURFACE_FORMAT_R32_FLOAT == 28);
static_assert(sizeof(FfxApiFloatCoords2D) == 8 && alignof(FfxApiFloatCoords2D) == 4);
static_assert(sizeof(FfxApiResourceDescription) == 32 && alignof(FfxApiResourceDescription) == 4);
static_assert(offsetof(FfxApiResourceDescription, type) == 0);
static_assert(offsetof(FfxApiResourceDescription, format) == 4);
static_assert(offsetof(FfxApiResourceDescription, width) == 8);
static_assert(offsetof(FfxApiResourceDescription, height) == 12);
static_assert(offsetof(FfxApiResourceDescription, depth) == 16);
static_assert(offsetof(FfxApiResourceDescription, mipCount) == 20);
static_assert(offsetof(FfxApiResourceDescription, flags) == 24);
static_assert(offsetof(FfxApiResourceDescription, usage) == 28);
static_assert(sizeof(FfxApiResource) == 48 && alignof(FfxApiResource) == 8);
static_assert(offsetof(FfxApiResource, resource) == 0);
static_assert(offsetof(FfxApiResource, description) == 8);
static_assert(offsetof(FfxApiResource, state) == 40);
static_assert(sizeof(ffxDispatchDescUpscale) == 432 && alignof(ffxDispatchDescUpscale) == 8);
#define CHECK_OFFSET(field, expected) static_assert(offsetof(ffxDispatchDescUpscale, field) == expected)
CHECK_OFFSET(header, 0);
CHECK_OFFSET(commandList, 16);
CHECK_OFFSET(color, 24);
CHECK_OFFSET(depth, 72);
CHECK_OFFSET(motionVectors, 120);
CHECK_OFFSET(exposure, 168);
CHECK_OFFSET(reactive, 216);
CHECK_OFFSET(transparencyAndComposition, 264);
CHECK_OFFSET(output, 312);
CHECK_OFFSET(jitterOffset, 360);
CHECK_OFFSET(motionVectorScale, 368);
CHECK_OFFSET(renderSize, 376);
CHECK_OFFSET(upscaleSize, 384);
CHECK_OFFSET(enableSharpening, 392);
CHECK_OFFSET(sharpness, 396);
CHECK_OFFSET(frameTimeDelta, 400);
CHECK_OFFSET(preExposure, 404);
CHECK_OFFSET(reset, 408);
CHECK_OFFSET(cameraNear, 412);
CHECK_OFFSET(cameraFar, 416);
CHECK_OFFSET(cameraFovAngleVertical, 420);
CHECK_OFFSET(viewSpaceToMetersFactor, 424);
CHECK_OFFSET(flags, 428);
#undef CHECK_OFFSET
