// SPDX-License-Identifier: MPL-2.0

//! Native GPU dispatch checks through the public upscaler route.
//! C++ DX12 equipment lives under tests/.
use super::{
    CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscaleResources,
    UpscaleTestInputs, Upscaler, UpscalerOptions, dispatch::resource_for_parity,
};
use crate::{error::Error, runtime::Runtime};
use fsr_sdk_sys::api::{
    FFX_API_RESOURCE_STATE_COMPUTE_READ, FFX_API_RESOURCE_STATE_UNORDERED_ACCESS,
    FFX_API_RETURN_OK, FfxApiResource, ffxApiHeader,
};
use std::{
    ffi::{CStr, c_void},
    path::PathBuf,
    ptr::null_mut,
};
use windows::{
    Win32::Graphics::{
        Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource},
        Dxgi::Common::{
            DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32_FLOAT,
        },
    },
    core::Interface,
};

#[cfg(feature = "movable")]
mod callbacks;
mod support;
use support::*;
mod color_signal;
mod colour_signal;
mod configure;
mod followup;
mod motion_scale;
#[cfg(feature = "movable")]
mod movable_public;
mod public_motion;
mod reactive_mask;
mod semantics;
mod sharpening;
mod sharpening_integration;
mod single_frame;
mod temporal_calibration;
mod temporal_diagnosis;
mod temporal_smoke;
mod thread_transfer_baseline;
mod thread_transfers;
mod typed_formats;
mod variable_output;

#[test]
#[ignore = "test-only raw motion format/scale probe; use experiment-motion-scale"]
fn motion_scale_probe() {
    motion_scale::run();
}

#[test]
#[ignore = "both public native motion families; use native-motion-scale"]
fn public_native_motion() {
    public_motion::run();
}

#[test]
#[ignore = "typed format public/C++ execution; use native-formats"]
fn public_typed_formats() {
    typed_formats::run();
}

#[test]
#[ignore = "test-only raw output geometry; use experiment-variable-output"]
fn variable_output_probe() {
    variable_output::run();
}

#[test]
#[ignore = "requires pinned runtime and DX12; use native-variable-output"]
fn public_variable_output() {
    variable_output::run_public();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_dispatch_readback() {
    single_frame::native_dispatch_readback();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_pre_exposure_dispatch() {
    single_frame::native_pre_exposure_dispatch();
}

#[test]
#[ignore = "requires pinned trusted runtime and hardware DX12; use native-upscaler"]
fn native_configure_dispatch() {
    configure::run();
}

#[test]
#[ignore = "requires pinned runtime and DX12; use native-upscaler"]
fn native_selected_provider_dispatch() {
    configure::selected_provider_dispatch();
}

#[test]
#[ignore = "probe-only raw descriptor; use experiment-reactive-mask, not a public generator API"]
fn reactive_mask_probe() {
    reactive_mask::run();
}

#[test]
#[ignore = "requires pinned runtime and DX12; use experiment-public-reactive-mask"]
fn public_reactive_mask_probe() {
    reactive_mask::run_public();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_depth_motion_dispatch() {
    temporal_smoke::native_depth_motion_dispatch();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use native-upscaler"]
fn native_automatic_exposure_dispatch() {
    temporal_smoke::native_automatic_exposure_dispatch();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use native-upscaler"]
fn native_active_render_dispatch() {
    temporal_smoke::native_active_render_dispatch();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use native-upscaler"]
fn native_supplied_masks_dispatch() {
    temporal_smoke::native_supplied_masks_dispatch();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use native-upscaler"]
fn native_hdr_dispatch() {
    temporal_smoke::native_hdr_dispatch();
}

#[test]
#[ignore = "requires trusted SDK/DX12 and declared signal inputs; use experiment-color-signal"]
fn native_color_signal() {
    color_signal::run();
}

#[test]
#[ignore = "legacy selector; use native_color_signal or experiment-color-signal"]
fn native_colour_signal() {
    colour_signal::run();
}

#[test]
#[ignore = "requires explicitly staged Proton equipment; use tools/verification/run.py run proton --native"]
fn proton_dispatch_readback() {
    single_frame::proton_dispatch_readback();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_temporal_smoke() {
    temporal_smoke::native_temporal_smoke();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_semantic_comparisons() {
    semantics::compare();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run experiment-sharpening --native"]
fn native_sharpening_comparisons() {
    sharpening::compare();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use tools/verification/run.py run native-upscaler --native"]
fn native_sharpening_integration() {
    sharpening_integration::run();
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use experiment-temporal-calibration"]
fn native_temporal_input_only() {
    temporal_calibration::run(true);
}

#[test]
#[ignore = "requires trusted SDK DLLs and hardware DX12; use experiment-temporal-calibration"]
fn native_temporal_calibration() {
    temporal_calibration::run(false);
}

#[test]
#[ignore = "requires explicitly staged native temporal diagnosis equipment"]
fn native_temporal_diagnosis_inputs() {
    temporal_diagnosis::run(true);
}

#[test]
#[ignore = "requires explicitly staged native temporal diagnosis equipment"]
fn native_temporal_diagnosis() {
    temporal_diagnosis::run(false);
}

#[test]
#[ignore = "requires explicitly staged Proton temporal regression equipment"]
fn proton_temporal_diagnosis_inputs() {
    temporal_diagnosis::run_with_diagnostics(true, true);
}

#[test]
#[ignore = "requires explicitly staged Proton temporal regression equipment"]
fn proton_temporal_diagnosis() {
    temporal_diagnosis::run_with_diagnostics(false, true);
}
