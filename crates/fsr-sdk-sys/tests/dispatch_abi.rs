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
    assert_eq!(FFX_API_DISPATCH_DESC_TYPE_UPSCALE, 0x0001_0001);
    assert_eq!(FFX_API_RESOURCE_TYPE_TEXTURE2D, 2);
    assert_eq!(FFX_API_RESOURCE_FLAGS_NONE, 0);
    assert_eq!(FFX_API_RESOURCE_USAGE_READ_ONLY, 0);
    assert_eq!(FFX_API_RESOURCE_USAGE_UAV, 2);
    assert_eq!(FFX_API_RESOURCE_STATE_COMPUTE_READ, 4);
    assert_eq!(FFX_API_RESOURCE_STATE_UNORDERED_ACCESS, 2);
    assert_eq!(FFX_API_SURFACE_FORMAT_UNKNOWN, 0);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT, 4);
    assert_eq!(FFX_API_SURFACE_FORMAT_R16G16_FLOAT, 18);
    assert_eq!(FFX_API_SURFACE_FORMAT_R32_FLOAT, 28);
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
