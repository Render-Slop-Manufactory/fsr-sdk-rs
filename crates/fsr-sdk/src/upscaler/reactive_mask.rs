// SPDX-License-Identifier: MPL-2.0

//! Platform-independent descriptor mapping after backend inspection.
use super::{Dimensions, ReactiveMaskOptions};
use crate::error::{DispatchValidation, Error};
use fsr_sdk_sys::{api::*, upscale::*};
use std::{ffi::c_void, ptr::null_mut};

pub(crate) fn check_region(render: Dimensions, maximum: Dimensions) -> Result<(), Error> {
    if render.width() > maximum.width() || render.height() > maximum.height() {
        return Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::ActiveRenderSize,
        )));
    }
    Ok(())
}

pub(crate) fn build_descriptor(
    command_list: *mut c_void,
    resources: [FfxApiResource; 3],
    render: Dimensions,
    options: &ReactiveMaskOptions,
) -> ffxDispatchDescUpscaleGenerateReactiveMask {
    let [color_opaque_only, color_pre_upscale, out_reactive] = resources;
    let mut flags = 0;
    for (enabled, flag) in [
        (
            options.apply_tonemap(),
            FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_TONEMAP,
        ),
        (
            options.apply_inverse_tonemap(),
            FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP,
        ),
        (
            options.apply_threshold(),
            FFX_UPSCALE_AUTOREACTIVEFLAGS_APPLY_THRESHOLD,
        ),
        (
            options.use_components_max(),
            FFX_UPSCALE_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX,
        ),
    ] {
        if enabled {
            flags |= flag;
        }
    }
    ffxDispatchDescUpscaleGenerateReactiveMask {
        header: ffxApiHeader {
            r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK,
            pNext: null_mut(),
        },
        commandList: command_list,
        colorOpaqueOnly: color_opaque_only,
        colorPreUpscale: color_pre_upscale,
        outReactive: out_reactive,
        renderSize: FfxApiDimensions2D {
            width: render.width(),
            height: render.height(),
        },
        scale: options.scale(),
        cutoffThreshold: options.cutoff_threshold(),
        binaryValue: options.binary_value(),
        flags,
    }
}

#[cfg(test)]
mod tests;
