// SPDX-License-Identifier: MPL-2.0

//! Borrowed DX12 command-list and resource inputs.
use super::Sharpening;
use crate::upscaler::common::{CameraParameters, FrameTimeMillis, JitterOffsetPixels};
use windows::Win32::Graphics::Direct3D12::{ID3D12GraphicsCommandList, ID3D12Resource};

/// Borrowed textures for one DX12 upscaler recording.
///
/// Construction and cloning only assemble borrows: they perform no COM inspection,
/// native calls or independent COM retention. Resource compatibility is checked
/// during dispatch; contents, states, storage and GPU lifetimes remain caller
/// obligations described on [`UpscaleDispatch`].
#[derive(Debug, Clone)]
pub struct UpscaleResources<'a> {
    color: &'a ID3D12Resource,
    depth: &'a ID3D12Resource,
    motion_vectors: &'a ID3D12Resource,
    exposure: &'a ID3D12Resource,
    output: &'a ID3D12Resource,
}

impl<'a> UpscaleResources<'a> {
    /// Assemble all five texture roles, requiring explicit exposure.
    ///
    /// No metadata or pixel inspection occurs here. Clone the bundle to reuse
    /// its borrows when constructing later frames; cloning does not extend GPU
    /// lifetimes. Automatic exposure and omitted exposure are not supported.
    pub fn new(
        color: &'a ID3D12Resource,
        depth: &'a ID3D12Resource,
        motion_vectors: &'a ID3D12Resource,
        exposure: &'a ID3D12Resource,
        output: &'a ID3D12Resource,
    ) -> Self {
        Self {
            color,
            depth,
            motion_vectors,
            exposure,
            output,
        }
    }

    /// Render-size linear, non-HDR, non-pre-exposed RGBA16F scene color.
    /// Must share the depth projection and enter shader-readable as documented
    /// on [`UpscaleDispatch`].
    pub fn color(&self) -> &'a ID3D12Resource {
        self.color
    }

    /// Render-size R32F ordinary finite device depth, near zero and far one.
    /// Linearized, reversed or infinite depth is outside the supported profile.
    pub fn depth(&self) -> &'a ID3D12Resource {
        self.depth
    }

    /// Render-size RG16F jitter-free current-to-previous UV displacement.
    /// Axes are +X right and +Y down. The wrapper multiplies UV values by render
    /// width/height for native pixel scale; do not supply NDC or pixel values.
    pub fn motion_vectors(&self) -> &'a ID3D12Resource {
        self.motion_vectors
    }

    /// Shader-readable 1×1 R32F positive finite exposure multiplier, also used
    /// by subsequent tonemapping. The recorded GPU profile used `1.0`.
    ///
    /// Returns `Some` for every currently constructible bundle. The optional
    /// return leaves room for a later exposure mode; this API requires exposure.
    pub fn exposure(&self) -> Option<&'a ID3D12Resource> {
        Some(self.exposure)
    }

    /// Distinct RGBA16F destination at the context's maximum output size,
    /// with UAV capability and initial `UNORDERED_ACCESS` state.
    /// Distinct COM identity does not prove non-overlapping backing storage.
    pub fn output(&self) -> &'a ID3D12Resource {
        self.output
    }
}

/// Borrowed inputs for one DX12 upscaler recording.
///
/// Textures must be distinct single-sample, single-mip, single-slice Texture2D
/// resources on the context device. Color is linear, non-HDR, pre-tonemap,
/// non-pre-exposed RGBA16F; depth is ordinary finite device depth (near 0,
/// far 1) in R32F; jitter-free motion vectors are current-to-previous,
/// top-down UV displacement in RG16F at render size. Exposure is a 1×1 R32F
/// texture with a positive finite multiplier also used by subsequent tonemapping.
/// Output is a distinct UAV-capable RGBA16F texture. Inputs match the context's
/// maximum render size; output matches its maximum upscale size. The caller
/// applies `jitter` to color and depth projection and reports that same pixel
/// offset here. Pulse `reset` for the first frame and after cuts or history
/// discontinuities; it does not recover a poisoned context. Use the camera's
/// near/far distances in the same units and a vertical FOV in radians. The
/// caller supplies content consistent with these semantics; the wrapper can inspect
/// descriptions but not pixel values or actual resource states.
///
/// # Shared resource and lifetime requirements
///
/// Every input must enter in `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`;
/// output must enter in `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. The borrow only
/// covers the CPU call. All five textures, their backing storage, the command
/// list/allocator and the upscaler must remain valid for recorded GPU use through
/// completion, with resources resident and backing storage non-overlapping during
/// dispatch. See [`Upscaler::dispatch`](crate::upscaler::Upscaler::dispatch) for submission, failure and teardown rules.
pub struct UpscaleDispatch<'a> {
    command_list: &'a ID3D12GraphicsCommandList,
    resources: UpscaleResources<'a>,
    jitter: JitterOffsetPixels,
    frame_time: FrameTimeMillis,
    camera: CameraParameters,
    reset: bool,
    sharpening: Sharpening,
}

impl<'a> UpscaleDispatch<'a> {
    /// Assemble a frame with native sharpening disabled by default.
    ///
    /// Supply renderer-selected jitter, timing, camera and reset explicitly.
    /// This constructor performs no COM inspection or native call and retains
    /// no independent COM references. Change required inputs by constructing a
    /// new frame, reusing a cloned resource bundle where appropriate.
    pub fn new(
        command_list: &'a ID3D12GraphicsCommandList,
        resources: UpscaleResources<'a>,
        jitter: JitterOffsetPixels,
        frame_time: FrameTimeMillis,
        camera: CameraParameters,
        reset: bool,
    ) -> Self {
        Self {
            command_list,
            resources,
            jitter,
            frame_time,
            camera,
            reset,
            sharpening: Sharpening::Disabled,
        }
    }

    /// Open recording DIRECT list on the context device. Dispatch neither
    /// closes, submits nor waits; keep its allocator valid through completion.
    pub fn command_list(&self) -> &'a ID3D12GraphicsCommandList {
        self.command_list
    }

    /// Borrow the texture bundle without cloning its borrows.
    pub fn resources(&self) -> &UpscaleResources<'a> {
        &self.resources
    }

    /// Pixel offset actually applied to color and depth projection.
    /// Positive X/Y move the image right/down. For analytic images, sample at
    /// `pixel_center - jitter`; this is distinct from NDC translation/helper sign.
    pub fn jitter(&self) -> JitterOffsetPixels {
        self.jitter
    }

    /// Positive finite elapsed render-frame time in milliseconds.
    pub fn frame_time(&self) -> FrameTimeMillis {
        self.frame_time
    }

    /// Finite camera values matching the ordinary-depth projection.
    pub fn camera(&self) -> CameraParameters {
        self.camera
    }

    /// Reset on the first frame and after cuts/history discontinuities.
    /// This is neither a fence nor poison recovery. Mode switching alone does
    /// not require a reset; temporal history continues under the caller's control.
    pub fn reset(&self) -> bool {
        self.reset
    }

    /// Borrow this frame's sharpening selection without cloning it.
    pub fn sharpening(&self) -> &Sharpening {
        &self.sharpening
    }

    /// Replace this frame's selection without a native call or context mutation.
    /// Use [`Sharpening::Disabled`] to disable the pass; enabled zero stays enabled.
    pub fn set_sharpening(&mut self, value: Sharpening) {
        self.sharpening = value;
    }
}
