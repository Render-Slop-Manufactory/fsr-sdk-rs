// SPDX-License-Identifier: MPL-2.0

//! Paired with dispatch_native_abi.cpp against pinned SDK v2.3.0 headers.
#[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
use core::{
    ffi::c_void,
    mem::{align_of, offset_of, size_of},
};
use fsr_sdk_sys::{api::*, upscale::*};

const _: fn(
    PfnFfxDispatch,
) -> Option<unsafe extern "C" fn(*mut ffxContext, *const ffxDispatchDescHeader) -> u32> =
    |value| value;

#[test]
fn selected_dispatch_constants() {
    assert_eq!(FFX_UPSCALE_ENABLE_DEPTH_INVERTED, 1 << 3);
    assert_eq!(FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS, 1 << 1);
    assert_eq!(
        FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION,
        1 << 2
    );
    assert_eq!(FFX_UPSCALE_ENABLE_DEPTH_INFINITE, 1 << 4);
    assert_eq!(FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE, 1 << 0);
    assert_eq!(FFX_UPSCALE_ENABLE_AUTO_EXPOSURE, 1 << 5);
    assert_eq!(FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION, 1 << 6);
    assert_eq!(FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE, 1 << 8);
    assert_eq!(FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB, 1 << 1);
    assert_eq!(FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ, 1 << 2);
    assert_eq!(FFX_API_DISPATCH_DESC_TYPE_UPSCALE, 0x0001_0001);
    assert_eq!(FFX_API_SURFACE_FORMAT_UNKNOWN, 0);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32B32A32_TYPELESS, 1);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32B32A32_UINT, 2);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT, 3);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT, 4);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT, 5);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32_FLOAT, 6);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8_UINT, 7);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32_UINT, 8);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8B8A8_TYPELESS, 9);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM, 10);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8B8A8_SNORM, 11);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8B8A8_SRGB, 12);
    assert_eq!(FFX_API_SURFACE_FORMAT_B8G8R8A8_TYPELESS, 13);
    assert_eq!(FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM, 14);
    assert_eq!(FFX_API_SURFACE_FORMAT_B8G8R8A8_SRGB, 15);
    assert_eq!(FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT, 16);
    assert_eq!(FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM, 17);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16_FLOAT, 18);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16_UINT, 19);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16_SINT, 20);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16_FLOAT, 21);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16_UINT, 22);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16_UNORM, 23);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16_SNORM, 24);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8_UNORM, 25);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8_UNORM, 26);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8_UINT, 27);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32_FLOAT, 28);
    assert_eq!(FFX_API_SURFACE_FORMAT_R9G9B9E5_SHAREDEXP, 29);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16B16A16_TYPELESS, 30);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32_TYPELESS, 31);
    assert_eq!(FFX_API_SURFACE_FORMAT_R10G10B10A2_TYPELESS, 32);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16_TYPELESS, 33);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16_TYPELESS, 34);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8_TYPELESS, 35);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8G8_TYPELESS, 36);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32_TYPELESS, 37);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32G32_UINT, 38);
    assert_eq!(FFX_API_SURFACE_FORMAT_R8_SNORM, 39);
    assert_eq!(FFX_API_RESOURCE_TYPE_TEXTURE2D, 2);
    assert_eq!(FFX_API_RESOURCE_FLAGS_NONE, 0);
    assert_eq!(FFX_API_RESOURCE_USAGE_READ_ONLY, 0);
    assert_eq!(FFX_API_RESOURCE_USAGE_UAV, 2);
    assert_eq!(FFX_API_RESOURCE_STATE_COMPUTE_READ, 4);
    assert_eq!(FFX_API_RESOURCE_STATE_UNORDERED_ACCESS, 2);
    assert_eq!(FFX_API_RESOURCE_USAGE_DEPTHTARGET, 4);
    assert_eq!(FFX_API_RESOURCE_USAGE_STENCILTARGET, 32);
}

#[test]
#[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
fn windows_x64_dispatch_layout() {
    assert_eq!(
        (
            size_of::<FfxApiFloatCoords2D>(),
            align_of::<FfxApiFloatCoords2D>()
        ),
        (8, 4)
    );
    assert_eq!(
        (
            size_of::<FfxApiResourceDescription>(),
            align_of::<FfxApiResourceDescription>()
        ),
        (32, 4)
    );
    assert_eq!(offset_of!(FfxApiResourceDescription, r#type), 0);
    assert_eq!(offset_of!(FfxApiResourceDescription, format), 4);
    assert_eq!(offset_of!(FfxApiResourceDescription, width), 8);
    assert_eq!(offset_of!(FfxApiResourceDescription, height), 12);
    assert_eq!(offset_of!(FfxApiResourceDescription, depth), 16);
    assert_eq!(offset_of!(FfxApiResourceDescription, mipCount), 20);
    assert_eq!(offset_of!(FfxApiResourceDescription, flags), 24);
    assert_eq!(offset_of!(FfxApiResourceDescription, usage), 28);
    assert_eq!(
        (size_of::<FfxApiResource>(), align_of::<FfxApiResource>()),
        (48, 8)
    );
    assert_eq!(offset_of!(FfxApiResource, resource), 0);
    assert_eq!(offset_of!(FfxApiResource, description), 8);
    assert_eq!(offset_of!(FfxApiResource, state), 40);
    assert_eq!(
        (
            size_of::<ffxDispatchDescUpscale>(),
            align_of::<ffxDispatchDescUpscale>()
        ),
        (432, 8)
    );
    for (actual, expected) in [
        (offset_of!(ffxDispatchDescUpscale, header), 0),
        (offset_of!(ffxDispatchDescUpscale, commandList), 16),
        (offset_of!(ffxDispatchDescUpscale, color), 24),
        (offset_of!(ffxDispatchDescUpscale, depth), 72),
        (offset_of!(ffxDispatchDescUpscale, motionVectors), 120),
        (offset_of!(ffxDispatchDescUpscale, exposure), 168),
        (offset_of!(ffxDispatchDescUpscale, reactive), 216),
        (
            offset_of!(ffxDispatchDescUpscale, transparencyAndComposition),
            264,
        ),
        (offset_of!(ffxDispatchDescUpscale, output), 312),
        (offset_of!(ffxDispatchDescUpscale, jitterOffset), 360),
        (offset_of!(ffxDispatchDescUpscale, motionVectorScale), 368),
        (offset_of!(ffxDispatchDescUpscale, renderSize), 376),
        (offset_of!(ffxDispatchDescUpscale, upscaleSize), 384),
        (offset_of!(ffxDispatchDescUpscale, enableSharpening), 392),
        (offset_of!(ffxDispatchDescUpscale, sharpness), 396),
        (offset_of!(ffxDispatchDescUpscale, frameTimeDelta), 400),
        (offset_of!(ffxDispatchDescUpscale, preExposure), 404),
        (offset_of!(ffxDispatchDescUpscale, reset), 408),
        (offset_of!(ffxDispatchDescUpscale, cameraNear), 412),
        (offset_of!(ffxDispatchDescUpscale, cameraFar), 416),
        (
            offset_of!(ffxDispatchDescUpscale, cameraFovAngleVertical),
            420,
        ),
        (
            offset_of!(ffxDispatchDescUpscale, viewSpaceToMetersFactor),
            424,
        ),
        (offset_of!(ffxDispatchDescUpscale, flags), 428),
    ] {
        assert_eq!(actual, expected);
    }
    assert_eq!(size_of::<*mut c_void>(), 8);
}

// Field types are checked independently of executing the target layout test.
const _: fn(&ffxDispatchDescUpscaleGenerateReactiveMask) = |value| {
    let _: &ffxDispatchDescHeader = &value.header;
    let _: &*mut core::ffi::c_void = &value.commandList;
    let _: &FfxApiResource = &value.colorOpaqueOnly;
    let _: &FfxApiResource = &value.colorPreUpscale;
    let _: &FfxApiResource = &value.outReactive;
    let _: &FfxApiDimensions2D = &value.renderSize;
    let _: &f32 = &value.scale;
    let _: &f32 = &value.cutoffThreshold;
    let _: &f32 = &value.binaryValue;
    let _: &u32 = &value.flags;
};

#[test]
fn reactive_mask_constants() {
    assert_eq!(
        FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK,
        0x10006
    );
    assert_eq!(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_TONEMAP, 1);
    assert_eq!(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP, 2);
    assert_eq!(FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_THRESHOLD, 4);
    assert_eq!(FFX_UPSCALE_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX, 8);
}

// The host result describes its 64-bit C layout, not independent MSVC evidence.
#[cfg(target_pointer_width = "64")]
#[test]
fn reactive_mask_64_bit_layout() {
    use core::mem::{align_of, offset_of, size_of};
    type D = ffxDispatchDescUpscaleGenerateReactiveMask;
    assert_eq!((size_of::<D>(), align_of::<D>()), (192, 8));
    assert_eq!(offset_of!(D, header), 0);
    assert_eq!(offset_of!(D, commandList), 16);
    assert_eq!(offset_of!(D, colorOpaqueOnly), 24);
    assert_eq!(offset_of!(D, colorPreUpscale), 72);
    assert_eq!(offset_of!(D, outReactive), 120);
    assert_eq!(offset_of!(D, renderSize), 168);
    assert_eq!(offset_of!(D, scale), 176);
    assert_eq!(offset_of!(D, cutoffThreshold), 180);
    assert_eq!(offset_of!(D, binaryValue), 184);
    assert_eq!(offset_of!(D, flags), 188);
}
