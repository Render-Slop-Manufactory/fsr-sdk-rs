// SPDX-License-Identifier: MPL-2.0

//! Translation of inspected frame inputs into the pinned native descriptor.
use super::{
    CameraParameters, FrameTimeMillis, JitterOffsetPixels, Sharpening, resources::Converted,
};
use fsr_sdk_sys::{
    api::*,
    upscale::{FFX_API_DISPATCH_DESC_TYPE_UPSCALE, ffxDispatchDescUpscale},
};
use std::{ffi::c_void, ptr::null_mut};

fn null_resource() -> FfxApiResource {
    FfxApiResource {
        resource: null_mut(),
        description: FfxApiResourceDescription {
            r#type: 0,
            format: 0,
            width: FfxApiResourceWidth { width: 0 },
            height: FfxApiResourceHeight { height: 0 },
            depth: FfxApiResourceDepth { depth: 0 },
            mipCount: 0,
            flags: 0,
            usage: 0,
        },
        state: 0,
    }
}

pub(super) fn build_descriptor(
    command_list: *mut c_void,
    resources: [Converted; 5],
    frame: (JitterOffsetPixels, FrameTimeMillis, CameraParameters, bool),
    sharpening: &Sharpening,
) -> ffxDispatchDescUpscale {
    let [color, depth, motion, exposure, output] = resources;
    let (jitter, frame_time, camera, reset) = frame;
    let (enable_sharpening, sharpness) = match sharpening {
        Sharpening::Disabled => (false, 0.0),
        Sharpening::Native(value) => (true, value.value()),
    };
    ffxDispatchDescUpscale {
        header: ffxApiHeader {
            r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
            pNext: null_mut(),
        },
        commandList: command_list,
        color: color.native,
        depth: depth.native,
        motionVectors: motion.native,
        exposure: exposure.native,
        reactive: null_resource(),
        transparencyAndComposition: null_resource(),
        output: output.native,
        jitterOffset: FfxApiFloatCoords2D {
            x: jitter.x(),
            y: jitter.y(),
        },
        motionVectorScale: FfxApiFloatCoords2D {
            x: color.size.0 as f32,
            y: color.size.1 as f32,
        },
        renderSize: FfxApiDimensions2D {
            width: color.size.0,
            height: color.size.1,
        },
        upscaleSize: FfxApiDimensions2D {
            width: output.size.0,
            height: output.size.1,
        },
        enableSharpening: enable_sharpening,
        sharpness,
        frameTimeDelta: frame_time.milliseconds(),
        preExposure: 1.0,
        reset,
        cameraNear: camera.near(),
        cameraFar: camera.far(),
        cameraFovAngleVertical: camera.vertical_fov_radians(),
        viewSpaceToMetersFactor: camera.view_space_to_meters(),
        flags: 0,
    }
}
