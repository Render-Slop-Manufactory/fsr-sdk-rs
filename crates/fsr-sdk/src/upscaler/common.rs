// SPDX-License-Identifier: MPL-2.0

//! Checked upscaler configuration and frame scalars without GPU object dependencies.
use crate::error::{DispatchScalar, DispatchValidation, Error};
use std::num::NonZeroU32;

/// Nonzero width and height in pixels.
///
/// This type rejects zero only. It does not certify a device/provider-supported
/// size, a render/output ratio or a complete safe numerical domain for the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl Dimensions {
    /// Width in pixels, suitable for renderer resource descriptions.
    pub fn width(self) -> u32 {
        self.width.get()
    }

    /// Height in pixels, suitable for renderer resource descriptions.
    pub fn height(self) -> u32 {
        self.height.get()
    }

    /// Rejects either zero component before any native call.
    ///
    /// Returns [`Error::InvalidDimensions`] with the original values on failure.
    pub fn new(width: u32, height: u32) -> Result<Self, Error> {
        let Some(width_checked) = NonZeroU32::new(width) else {
            return Err(Error::InvalidDimensions { width, height });
        };
        let Some(height_checked) = NonZeroU32::new(height) else {
            return Err(Error::InvalidDimensions { width, height });
        };
        Ok(Self {
            width: width_checked,
            height: height_checked,
        })
    }
}

/// Fixed render and output dimensions for the first public upscaler profile.
///
/// These map to native context maxima. The current dispatch contract requires
/// active texture extents to equal them; smaller or changing active sizes are
/// not supported. Creation flags are fixed to zero, so these options do not
/// enable HDR, dynamic resolution or automatic exposure.
#[derive(Debug, Clone)]
pub struct UpscalerOptions {
    max_render_size: Dimensions,
    max_upscale_size: Dimensions,
}

impl UpscalerOptions {
    /// Sets the checked render and output maxima for the fixed upscaler profile.
    ///
    /// Creation flags remain zero; HDR, dynamic resolution and automatic
    /// exposure are not enabled. Clone these options for explicit reuse.
    pub fn new(max_render_size: Dimensions, max_upscale_size: Dimensions) -> Self {
        Self {
            max_render_size,
            max_upscale_size,
        }
    }

    /// Maximum render width/height in pixels, also the required color, depth and
    /// motion-vector texture extent for every dispatch on this context.
    pub fn max_render_size(&self) -> Dimensions {
        self.max_render_size
    }

    /// Maximum output width/height in pixels, also the required output texture
    /// extent for every dispatch on this context.
    pub fn max_upscale_size(&self) -> Dimensions {
        self.max_upscale_size
    }
}

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

pub(in crate::upscaler) fn invalid_scalar(scalar: DispatchScalar) -> Error {
    Error::InvalidDispatch {
        reason: DispatchValidation::Scalar(scalar),
    }
}
