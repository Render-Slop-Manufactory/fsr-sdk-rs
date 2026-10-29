// SPDX-License-Identifier: MPL-2.0

//! Probe-only ABI from SDK v2.3.0 `upscalers/include/ffx_upscale.h` at
//! `60f4ea81909200d8542eca14dccb2628b763a9a3`. AMD notice: crate `LICENSE-AMD`.
//! This is not a production binding or acceptance of D030.
use fsr_sdk_sys::api::{FfxApiDimensions2D, FfxApiResource, ffxDispatchDescHeader};
use std::ffi::c_void;

pub const GENERATE_REACTIVE: u64 = 0x10006;
pub const FLAGS: [u32; 4] = [1, 2, 4, 8];

#[repr(C)]
pub struct GenerateReactive {
    pub header: ffxDispatchDescHeader,
    pub command_list: *mut c_void,
    pub color_opaque_only: FfxApiResource,
    pub color_pre_upscale: FfxApiResource,
    pub out_reactive: FfxApiResource,
    pub render_size: FfxApiDimensions2D,
    pub scale: f32,
    pub cutoff_threshold: f32,
    pub binary_value: f32,
    pub flags: u32,
}
