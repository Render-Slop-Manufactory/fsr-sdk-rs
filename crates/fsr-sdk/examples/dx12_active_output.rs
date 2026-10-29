// SPDX-License-Identifier: MPL-2.0

//! Renderer-owned active output with maximum-sized external textures.
//! This example creates no device/runtime and submits no GPU work from `main`.

#[cfg(all(windows, feature = "dx12"))]
pub mod renderer_integration {
    use fsr_sdk::{
        error::Error,
        upscaler::{
            Dimensions, FrameParameters, ID3D12GraphicsCommandList, MotionVectorResolution,
            UpscaleResources, Upscaler, UpscalerOptions,
        },
    };

    /// Build the profile for a provider selected through the caller's setup.
    /// Allocate colour/depth at `render_max` and output/display motion at `output_max`.
    /// The acquired setup must establish support for this integration's output transitions.
    pub fn options(render_max: Dimensions, output_max: Dimensions) -> UpscalerOptions {
        let mut options = UpscalerOptions::new(render_max, output_max);
        options.set_dynamic_resolution(true);
        options.set_motion_vector_resolution(MotionVectorResolution::Display);
        options
    }

    /// Record a caller-prepared frame at its intended output size.
    ///
    /// The renderer prepares jitter using current active render/output dimensions
    /// through this selected context's queries. The frame carries matching colour,
    /// motion units/content and caller-selected first-frame/cut reset. With UV display
    /// motion, dispatch maps active output dimensions to native motion scale.
    ///
    /// # Safety
    /// Uphold `Upscaler::record`, including actual-provider support and safe access
    /// for this size/motion/sharpening profile. 2.3.4's recorded path ignores smaller
    /// output. Maximum textures and preflight do not prove bounded native access.
    /// The whole output allocation may be written. The renderer owns transitions,
    /// submission/fences, resource retention and any replacement after prior use.
    /// After a native error or abandoned successful recording, invalidate the list,
    /// complete earlier work and perform terminal cleanup before recreation.
    pub unsafe fn record(
        context: &mut Upscaler,
        list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        frame: &mut FrameParameters,
        output: Dimensions,
    ) -> Result<(), Error> {
        frame.set_active_output_size(Some(output));
        // Optional setup diagnosis does not authorize skipping dispatch inspection.
        context.validate_upscale_inputs(list, resources, frame)?;
        // SAFETY: The renderer establishes the documented native and GPU obligations.
        unsafe { context.record(list, resources, frame) }
    }
}

fn main() {}
