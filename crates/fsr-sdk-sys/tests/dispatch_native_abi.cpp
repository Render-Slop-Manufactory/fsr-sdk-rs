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
static_assert(FFX_UPSCALE_ENABLE_DEPTH_INVERTED == (1 << 3));
static_assert(FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS == (1 << 1));
static_assert(FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION == (1 << 2));
static_assert(FFX_UPSCALE_ENABLE_DEPTH_INFINITE == (1 << 4));
static_assert(FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE == (1 << 0));
static_assert(FFX_UPSCALE_ENABLE_AUTO_EXPOSURE == (1 << 5));
static_assert(FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION == (1 << 6));
static_assert(FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE == (1 << 8));
static_assert(FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB == (1 << 1));
static_assert(FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ == (1 << 2));
static_assert(FFX_API_SURFACE_FORMAT_UNKNOWN == 0);
static_assert(FFX_API_SURFACE_FORMAT_R32G32B32A32_TYPELESS == 1);
static_assert(FFX_API_SURFACE_FORMAT_R32G32B32A32_UINT == 2);
static_assert(FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT == 3);
static_assert(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT == 4);
static_assert(FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT == 5);
static_assert(FFX_API_SURFACE_FORMAT_R32G32_FLOAT == 6);
static_assert(FFX_API_SURFACE_FORMAT_R8_UINT == 7);
static_assert(FFX_API_SURFACE_FORMAT_R32_UINT == 8);
static_assert(FFX_API_SURFACE_FORMAT_R8G8B8A8_TYPELESS == 9);
static_assert(FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM == 10);
static_assert(FFX_API_SURFACE_FORMAT_R8G8B8A8_SNORM == 11);
static_assert(FFX_API_SURFACE_FORMAT_R8G8B8A8_SRGB == 12);
static_assert(FFX_API_SURFACE_FORMAT_B8G8R8A8_TYPELESS == 13);
static_assert(FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM == 14);
static_assert(FFX_API_SURFACE_FORMAT_B8G8R8A8_SRGB == 15);
static_assert(FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT == 16);
static_assert(FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM == 17);
static_assert(FFX_API_SURFACE_FORMAT_R16G16_FLOAT == 18);
static_assert(FFX_API_SURFACE_FORMAT_R16G16_UINT == 19);
static_assert(FFX_API_SURFACE_FORMAT_R16G16_SINT == 20);
static_assert(FFX_API_SURFACE_FORMAT_R16_FLOAT == 21);
static_assert(FFX_API_SURFACE_FORMAT_R16_UINT == 22);
static_assert(FFX_API_SURFACE_FORMAT_R16_UNORM == 23);
static_assert(FFX_API_SURFACE_FORMAT_R16_SNORM == 24);
static_assert(FFX_API_SURFACE_FORMAT_R8_UNORM == 25);
static_assert(FFX_API_SURFACE_FORMAT_R8G8_UNORM == 26);
static_assert(FFX_API_SURFACE_FORMAT_R8G8_UINT == 27);
static_assert(FFX_API_SURFACE_FORMAT_R32_FLOAT == 28);
static_assert(FFX_API_SURFACE_FORMAT_R9G9B9E5_SHAREDEXP == 29);
static_assert(FFX_API_SURFACE_FORMAT_R16G16B16A16_TYPELESS == 30);
static_assert(FFX_API_SURFACE_FORMAT_R32G32_TYPELESS == 31);
static_assert(FFX_API_SURFACE_FORMAT_R10G10B10A2_TYPELESS == 32);
static_assert(FFX_API_SURFACE_FORMAT_R16G16_TYPELESS == 33);
static_assert(FFX_API_SURFACE_FORMAT_R16_TYPELESS == 34);
static_assert(FFX_API_SURFACE_FORMAT_R8_TYPELESS == 35);
static_assert(FFX_API_SURFACE_FORMAT_R8G8_TYPELESS == 36);
static_assert(FFX_API_SURFACE_FORMAT_R32_TYPELESS == 37);
static_assert(FFX_API_SURFACE_FORMAT_R32G32_UINT == 38);
static_assert(FFX_API_SURFACE_FORMAT_R8_SNORM == 39);
static_assert(FFX_API_RESOURCE_TYPE_TEXTURE2D == 2);
static_assert(FFX_API_RESOURCE_FLAGS_NONE == 0);
static_assert(FFX_API_RESOURCE_USAGE_READ_ONLY == 0);
static_assert(FFX_API_RESOURCE_USAGE_UAV == 2);
static_assert(FFX_API_RESOURCE_STATE_COMPUTE_READ == 4);
static_assert(FFX_API_RESOURCE_STATE_UNORDERED_ACCESS == 2);
static_assert(FFX_API_RESOURCE_USAGE_DEPTHTARGET == 4);
static_assert(FFX_API_RESOURCE_USAGE_STENCILTARGET == 32);
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

static_assert(FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK == 0x10006);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_TONEMAP == 1);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP == 2);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_THRESHOLD == 4);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX == 8);
using Generator = ffxDispatchDescUpscaleGenerateReactiveMask;
static_assert(sizeof(Generator) == 192 && alignof(Generator) == 8);
static_assert(offsetof(Generator, header) == 0);
static_assert(std::is_same_v<decltype(Generator::header), ffxDispatchDescHeader>);
static_assert(offsetof(Generator, commandList) == 16);
static_assert(std::is_same_v<decltype(Generator::commandList), void*>);
static_assert(offsetof(Generator, colorOpaqueOnly) == 24);
static_assert(std::is_same_v<decltype(Generator::colorOpaqueOnly), FfxApiResource>);
static_assert(offsetof(Generator, colorPreUpscale) == 72);
static_assert(std::is_same_v<decltype(Generator::colorPreUpscale), FfxApiResource>);
static_assert(offsetof(Generator, outReactive) == 120);
static_assert(std::is_same_v<decltype(Generator::outReactive), FfxApiResource>);
static_assert(offsetof(Generator, renderSize) == 168);
static_assert(std::is_same_v<decltype(Generator::renderSize), FfxApiDimensions2D>);
static_assert(offsetof(Generator, scale) == 176);
static_assert(std::is_same_v<decltype(Generator::scale), float>);
static_assert(offsetof(Generator, cutoffThreshold) == 180);
static_assert(std::is_same_v<decltype(Generator::cutoffThreshold), float>);
static_assert(offsetof(Generator, binaryValue) == 184);
static_assert(std::is_same_v<decltype(Generator::binaryValue), float>);
static_assert(offsetof(Generator, flags) == 188);
static_assert(std::is_same_v<decltype(Generator::flags), uint32_t>);
