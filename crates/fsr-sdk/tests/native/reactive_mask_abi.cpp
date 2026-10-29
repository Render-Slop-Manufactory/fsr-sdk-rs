// SPDX-License-Identifier: MPL-2.0

// Independent assertions against the pinned SDK header; no copied declarations.
#include <cstddef>
#include <type_traits>
#include "ffx_upscale.h"
using Probe = ffxDispatchDescUpscaleGenerateReactiveMask;
static_assert(FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK == 0x10006);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_TONEMAP == 1);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP == 2);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_THRESHOLD == 4);
static_assert(FFX_UPSCALE_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX == 8);
static_assert(sizeof(Probe) == 192 && alignof(Probe) == 8);
#define FIELD(F, T, O) static_assert(std::is_same_v<decltype(Probe::F), T>); static_assert(offsetof(Probe, F) == O)
FIELD(header, ffxDispatchDescHeader, 0);
FIELD(commandList, void*, 16);
FIELD(colorOpaqueOnly, FfxApiResource, 24);
FIELD(colorPreUpscale, FfxApiResource, 72);
FIELD(outReactive, FfxApiResource, 120);
FIELD(renderSize, FfxApiDimensions2D, 168);
FIELD(scale, float, 176);
FIELD(cutoffThreshold, float, 180);
FIELD(binaryValue, float, 184);
FIELD(flags, uint32_t, 188);
