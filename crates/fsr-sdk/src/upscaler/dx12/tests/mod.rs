// SPDX-License-Identifier: MPL-2.0

mod active_render;
mod automatic_exposure;
mod bgra_srgb;
mod callbacks;
mod color_encoding;
mod com;
mod composition;
mod construction;
mod cpu_comparison;
mod depth_formats;
mod depth_motion;
mod diagnostics;
mod dispatch;
mod gpu_timing;
mod hdr;
pub(super) mod inputs;
mod inspection;
mod lifecycle;
mod masks;
#[cfg(feature = "movable")]
mod movable;
mod native_motion;
mod performance;
mod performance_comparison;
mod pre_exposure;
mod preflight;
mod provider_selection;
mod provider_setup;
mod queries;
mod reactive_mask;
mod scalar_views;
mod sharpening;
mod support;
mod traits;
mod tuning;
mod typed_formats;
mod usability;
mod utility_queries;

use super::*;
use support::*;

// Preserve the documented native runner filters while sharing lifecycle equipment.
#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tools/verification/run.py run native-upscaler --native"]
fn native_lifecycle() {
    lifecycle::native_lifecycle_case(false);
}

#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tools/verification/run.py run native-upscaler --native"]
fn native_lifecycle_drop() {
    lifecycle::native_lifecycle_case(true);
}
