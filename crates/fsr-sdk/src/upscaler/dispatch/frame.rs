// SPDX-License-Identifier: MPL-2.0

//! Checked frame values and borrowed renderer inputs.
use crate::error::{DispatchScalar, DispatchValidation, Error};
use windows::Win32::Graphics::Direct3D12::{ID3D12GraphicsCommandList, ID3D12Resource};

/// Current-frame projection jitter actually applied to rendering, in pixels.
///
/// X and Y describe horizontal and vertical subpixel offsets, not UV or NDC
/// translations. Report the same offset used for both color and depth projection.
/// The wrapper passes both components unchanged: signs must match the renderer's
/// projection conversion, not an assumed sign for a jitter generator. Do not
/// infer jitter signs from the motion vectors' top-down UV convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JitterOffsetPixels {
    x: f32,
    y: f32,
}

impl JitterOffsetPixels {
    /// Accepts any two finite pixel offsets, including zero and negative values.
    /// No magnitude limit or jitter sequence is imposed. Non-finite components
    /// return [`Error::InvalidDispatch`] with the corresponding scalar reason.
    pub fn new(x: f32, y: f32) -> Result<Self, Error> {
        if !x.is_finite() {
            return Err(invalid_scalar(DispatchScalar::JitterX));
        }
        if !y.is_finite() {
            return Err(invalid_scalar(DispatchScalar::JitterY));
        }
        Ok(Self { x, y })
    }
    /// Horizontal applied jitter in pixels, with the supplied sign unchanged.
    pub fn x(self) -> f32 {
        self.x
    }
    /// Vertical applied jitter in pixels, with the supplied sign unchanged.
    pub fn y(self) -> f32 {
        self.y
    }
}

/// Elapsed render-frame time in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTimeMillis(f32);

impl FrameTimeMillis {
    /// Accepts a positive finite elapsed frame time in milliseconds.
    /// Zero, negative, NaN or infinite values return [`Error::InvalidDispatch`].
    pub fn new(milliseconds: f32) -> Result<Self, Error> {
        if !milliseconds.is_finite() || milliseconds <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::FrameTimeMillis));
        }
        Ok(Self(milliseconds))
    }
    /// Elapsed frame time in milliseconds; no seconds conversion is performed.
    pub fn milliseconds(self) -> f32 {
        self.0
    }
}

/// Finite camera parameters for ordinary device depth, without reversed/infinite depth.
///
/// Near and far share the renderer's view-space distance unit. The conversion
/// factor expresses metres per view-space unit; it does not change depth encoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParameters {
    near: f32,
    far: f32,
    vertical_fov_radians: f32,
    view_space_to_meters: f32,
}

impl CameraParameters {
    /// Checks the camera domain before native dispatch.
    ///
    /// `near` must be positive and finite, `far` finite and greater than `near`,
    /// vertical FOV finite and strictly between zero and pi radians, and the
    /// metres-per-view-space-unit factor positive and finite. Violations return
    /// [`Error::InvalidDispatch`] identifying the scalar; no native call occurs.
    pub fn new(
        near: f32,
        far: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
    ) -> Result<Self, Error> {
        if !near.is_finite() || near <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::CameraNear));
        }
        if !far.is_finite() || far <= near {
            return Err(invalid_scalar(DispatchScalar::CameraFar));
        }
        if !vertical_fov_radians.is_finite()
            || vertical_fov_radians <= 0.0
            || vertical_fov_radians >= std::f32::consts::PI
        {
            return Err(invalid_scalar(DispatchScalar::VerticalFovRadians));
        }
        if !view_space_to_meters.is_finite() || view_space_to_meters <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::ViewSpaceToMeters));
        }
        Ok(Self {
            near,
            far,
            vertical_fov_radians,
            view_space_to_meters,
        })
    }
    /// Near-plane distance in the renderer's view-space units.
    pub fn near(self) -> f32 {
        self.near
    }
    /// Far-plane distance in the same view-space units as the near plane.
    pub fn far(self) -> f32 {
        self.far
    }
    /// Vertical field of view in radians, not degrees or horizontal FOV.
    pub fn vertical_fov_radians(self) -> f32 {
        self.vertical_fov_radians
    }
    /// Metres corresponding to one view-space distance unit.
    pub fn view_space_to_meters(self) -> f32 {
        self.view_space_to_meters
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

pub(super) fn invalid_scalar(scalar: DispatchScalar) -> Error {
    Error::InvalidDispatch {
        reason: DispatchValidation::Scalar(scalar),
    }
}
