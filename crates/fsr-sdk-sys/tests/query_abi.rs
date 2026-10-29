// SPDX-License-Identifier: MPL-2.0

//! Paired with query_native_abi.cpp against pinned SDK v2.3.0 headers.
use fsr_sdk_sys::{api::*, upscale::*};

#[test]
fn query_tags_and_resource_bits() {
    assert_eq!(FFX_API_CONFIGURE_DESC_TYPE_UPSCALE_KEYVALUE, 0x10007);
    assert_eq!(
        [
            FFX_API_CONFIGURE_UPSCALE_KEY_FVELOCITYFACTOR,
            FFX_API_CONFIGURE_UPSCALE_KEY_FREACTIVENESSSCALE,
            FFX_API_CONFIGURE_UPSCALE_KEY_FSHADINGCHANGESCALE,
            FFX_API_CONFIGURE_UPSCALE_KEY_FACCUMULATIONADDEDPERFRAME,
            FFX_API_CONFIGURE_UPSCALE_KEY_FMINDISOCCLUSIONACCUMULATION,
        ],
        [0, 1, 2, 3, 4]
    );
    assert_eq!(
        FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE,
        0x10002
    );
    assert_eq!(
        FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE,
        0x10003
    );
    assert_eq!(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT, 0x10004);
    assert_eq!(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET, 0x10005);
    assert_eq!(FFX_UPSCALE_QUALITY_MODE_NATIVEAA, 0);
    assert_eq!(FFX_UPSCALE_QUALITY_MODE_QUALITY, 1);
    assert_eq!(FFX_UPSCALE_QUALITY_MODE_BALANCED, 2);
    assert_eq!(FFX_UPSCALE_QUALITY_MODE_PERFORMANCE, 3);
    assert_eq!(FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE, 4);

    assert_eq!(FFX_API_DESC_TYPE_OVERRIDE_VERSION, 5);
    assert_eq!(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION, 6);
    assert_eq!(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE, 0x10008);
    assert_eq!(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2, 0x10009);
    assert_eq!(
        FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS,
        0x1000a
    );
    assert_eq!(
        [
            FFX_API_QUERY_RESOURCE_INPUT_COLOR,
            FFX_API_QUERY_RESOURCE_INPUT_DEPTH,
            FFX_API_QUERY_RESOURCE_INPUT_MV,
            FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE,
            FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK,
            FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION
        ],
        [1, 2, 4, 8, 16, 32]
    );
}

// All fields are integer/pointer-only; these x64 layouts are also exercised by
// portable host tests. Only the independent MSVC probe establishes the target ABI.
#[cfg(target_pointer_width = "64")]
mod x64 {
    use super::*;
    use std::ffi::{c_char, c_void};
    macro_rules! field {
        ($ty:ty, $name:ident, $field_ty:ty, $offset:expr) => {
            const _: fn($ty) -> $field_ty = |value| value.$name;
            const _: () = assert!(std::mem::offset_of!($ty, $name) == $offset);
        };
    }
    macro_rules! layout {
        ($ty:ty, $size:expr, $align:expr) => {
            const _: () = assert!(std::mem::size_of::<$ty>() == $size);
            const _: () = assert!(std::mem::align_of::<$ty>() == $align);
        };
    }
    layout!(ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode, 32, 8);
    field!(
        ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(
        ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode,
        qualityMode,
        u32,
        16
    );
    field!(
        ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode,
        pOutUpscaleRatio,
        *mut f32,
        24
    );
    layout!(ffxQueryDescUpscaleGetRenderResolutionFromQualityMode, 48, 8);
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        displayWidth,
        u32,
        16
    );
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        displayHeight,
        u32,
        20
    );
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        qualityMode,
        u32,
        24
    );
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        pOutRenderWidth,
        *mut u32,
        32
    );
    field!(
        ffxQueryDescUpscaleGetRenderResolutionFromQualityMode,
        pOutRenderHeight,
        *mut u32,
        40
    );
    layout!(ffxQueryDescUpscaleGetJitterPhaseCount, 32, 8);
    field!(
        ffxQueryDescUpscaleGetJitterPhaseCount,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(ffxQueryDescUpscaleGetJitterPhaseCount, renderWidth, u32, 16);
    field!(
        ffxQueryDescUpscaleGetJitterPhaseCount,
        displayWidth,
        u32,
        20
    );
    field!(
        ffxQueryDescUpscaleGetJitterPhaseCount,
        pOutPhaseCount,
        *mut i32,
        24
    );
    layout!(ffxQueryDescUpscaleGetJitterOffset, 40, 8);
    field!(
        ffxQueryDescUpscaleGetJitterOffset,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(ffxQueryDescUpscaleGetJitterOffset, index, i32, 16);
    field!(ffxQueryDescUpscaleGetJitterOffset, phaseCount, i32, 20);
    field!(ffxQueryDescUpscaleGetJitterOffset, pOutX, *mut f32, 24);
    field!(ffxQueryDescUpscaleGetJitterOffset, pOutY, *mut f32, 32);
    layout!(ffxOverrideVersion, 24, 8);
    layout!(ffxConfigureDescUpscaleKeyValue, 40, 8);
    field!(
        ffxConfigureDescUpscaleKeyValue,
        header,
        ffxConfigureDescHeader,
        0
    );
    field!(ffxConfigureDescUpscaleKeyValue, key, u64, 16);
    field!(ffxConfigureDescUpscaleKeyValue, u64, u64, 24);
    field!(ffxConfigureDescUpscaleKeyValue, ptr, *mut c_void, 32);
    field!(ffxOverrideVersion, header, ffxApiHeader, 0);
    field!(ffxOverrideVersion, versionId, u64, 16);
    layout!(ffxQueryGetProviderVersion, 32, 8);
    field!(ffxQueryGetProviderVersion, header, ffxQueryDescHeader, 0);
    field!(ffxQueryGetProviderVersion, versionId, u64, 16);
    field!(ffxQueryGetProviderVersion, versionName, *const c_char, 24);
    layout!(FfxApiEffectMemoryUsage, 16, 8);
    field!(FfxApiEffectMemoryUsage, totalUsageInBytes, u64, 0);
    field!(FfxApiEffectMemoryUsage, aliasableUsageInBytes, u64, 8);
    layout!(ffxQueryDescUpscaleGetGPUMemoryUsage, 24, 8);
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsage,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsage,
        gpuMemoryUsageUpscaler,
        *mut FfxApiEffectMemoryUsage,
        16
    );
    layout!(ffxQueryDescUpscaleGetGPUMemoryUsageV2, 56, 8);
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsageV2,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsageV2,
        device,
        *mut c_void,
        16
    );
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsageV2,
        maxRenderSize,
        FfxApiDimensions2D,
        24
    );
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsageV2,
        maxUpscaleSize,
        FfxApiDimensions2D,
        32
    );
    field!(ffxQueryDescUpscaleGetGPUMemoryUsageV2, flags, u32, 40);
    field!(
        ffxQueryDescUpscaleGetGPUMemoryUsageV2,
        gpuMemoryUsageUpscaler,
        *mut FfxApiEffectMemoryUsage,
        48
    );
    layout!(ffxQueryDescUpscaleGetResourceRequirements, 32, 8);
    field!(
        ffxQueryDescUpscaleGetResourceRequirements,
        header,
        ffxQueryDescHeader,
        0
    );
    field!(
        ffxQueryDescUpscaleGetResourceRequirements,
        required_resources,
        u64,
        16
    );
    field!(
        ffxQueryDescUpscaleGetResourceRequirements,
        optional_resources,
        u64,
        24
    );
}
