// SPDX-License-Identifier: MPL-2.0

//! Borrowed DX12 command-list and resource inputs.
use crate::upscaler::common::{CameraParameters, FrameTimeMillis, JitterOffsetPixels};
use windows::Win32::Graphics::Direct3D12::{ID3D12GraphicsCommandList, ID3D12Resource};

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
    /// Open, recording DIRECT list on the context device. Dispatch records work;
    /// it does not close, submit or wait. Keep its allocator valid under the
    /// shared lifetime requirements above.
    pub command_list: &'a ID3D12GraphicsCommandList,
    /// Render-size `R16G16B16A16_FLOAT` linear, non-HDR color before tonemapping,
    /// without pre-exposure. Extent equals `max_render_size`; it shares the depth
    /// texture's jittered projection and the input state/lifetime rules above.
    pub color: &'a ID3D12Resource,
    /// Render-size shader-readable `R32_FLOAT` ordinary finite device depth
    /// (near toward zero, far toward one), not linearized, reversed or infinite
    /// depth. Uses the color projection and the shared input state/lifetime rules.
    pub depth: &'a ID3D12Resource,
    /// Render-size `R16G16_FLOAT` current-to-previous normalized UV displacement,
    /// with projection jitter removed. Axes are +X right and +Y down. The wrapper
    /// multiplies UV displacement by render width/height for native pixel scale;
    /// do not supply raw NDC or pixel displacement. Shared input state/lifetime
    /// rules apply; the wrapper cannot inspect vector content.
    pub motion_vectors: &'a ID3D12Resource,
    /// Shader-readable 1×1 `R32_FLOAT` positive finite exposure multiplier, also
    /// used by subsequent tonemapping. Explicit exposure is required; auto
    /// exposure is not enabled. The recorded GPU proof used `1.0`. Shared input
    /// state/lifetime rules apply; pixel validity is the caller's responsibility.
    pub exposure: &'a ID3D12Resource,
    /// Distinct `R16G16B16A16_FLOAT` output at `max_upscale_size`, with UAV
    /// capability and initial `UNORDERED_ACCESS` state. Shared storage, residency
    /// and lifetime rules apply; a distinct COM object does not prove non-aliasing.
    pub output: &'a ID3D12Resource,
    /// Pixel offset actually applied to this frame's color and depth projection.
    /// Positive X/Y move the projected image right/down. For an analytic image,
    /// sample the unjittered scene at `(pixel_center - jitter)`; this is distinct
    /// from the projection matrix's NDC translation and the sequence helper sign.
    pub jitter: JitterOffsetPixels,
    /// Positive finite elapsed render-frame time in milliseconds.
    pub frame_time: FrameTimeMillis,
    /// Finite camera values matching the renderer's ordinary-depth projection.
    pub camera: CameraParameters,
    /// Reset temporal history for the first frame and after camera cuts/history
    /// discontinuities. This is not a fence or recovery from dispatch poison.
    pub reset: bool,
}
