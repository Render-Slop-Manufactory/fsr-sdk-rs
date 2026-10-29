// SPDX-License-Identifier: MPL-2.0

//! Translation of inspected frame inputs into the pinned native descriptor.
use super::{Sharpening, resources::ConvertedResources};
use crate::upscaler::{Dimensions, FrameParameters, HistoryAction, MotionVectorScale};
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

/// Validated image regions, distinct from native resource allocation metadata.
pub(super) struct DispatchGeometry {
    pub(super) render: Dimensions,
    pub(super) output: Dimensions,
    pub(super) motion: Dimensions,
}

pub(super) fn build_descriptor(
    command_list: *mut c_void,
    resources: ConvertedResources,
    frame: &FrameParameters,
    geometry: DispatchGeometry,
) -> ffxDispatchDescUpscale {
    let ConvertedResources {
        color,
        depth,
        motion,
        exposure,
        reactive,
        composition,
        output,
    } = resources;
    let jitter = frame.jitter();
    let camera = frame.camera();
    let (motion_x, motion_y) = match frame.motion_vector_scale() {
        MotionVectorScale::Native(scale) => (scale.x(), scale.y()),
        MotionVectorScale::Uv => (
            geometry.motion.width() as f32,
            geometry.motion.height() as f32,
        ),
        MotionVectorScale::Pixels => (1.0, 1.0),
    };
    let (enable_sharpening, sharpness) = match frame.sharpening() {
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
        exposure: exposure.map_or_else(null_resource, |resource| resource.native),
        reactive: reactive.map_or_else(null_resource, |r| r.native),
        transparencyAndComposition: composition.map_or_else(null_resource, |r| r.native),
        output: output.native,
        jitterOffset: FfxApiFloatCoords2D {
            x: jitter.x(),
            y: jitter.y(),
        },
        motionVectorScale: FfxApiFloatCoords2D {
            x: motion_x,
            y: motion_y,
        },
        renderSize: FfxApiDimensions2D {
            width: geometry.render.width(),
            height: geometry.render.height(),
        },
        upscaleSize: FfxApiDimensions2D {
            width: geometry.output.width(),
            height: geometry.output.height(),
        },
        enableSharpening: enable_sharpening,
        sharpness,
        frameTimeDelta: frame.frame_time().milliseconds(),
        preExposure: frame.pre_exposure().value(),
        reset: match frame.history_action() {
            HistoryAction::Reset => true,
            HistoryAction::Continue => false,
        },
        cameraNear: camera.near(),
        cameraFar: camera.far(),
        cameraFovAngleVertical: camera.vertical_fov_radians(),
        viewSpaceToMetersFactor: camera.view_space_to_meters(),
        flags: frame.color_encoding().hint().native_flags()
            | if frame.draw_debug_view() {
                fsr_sdk_sys::upscale::FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW
            } else {
                0
            },
    }
}
