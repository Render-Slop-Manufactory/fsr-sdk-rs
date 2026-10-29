// SPDX-License-Identifier: MPL-2.0

//! Checked finite and infinite camera projection parameters.

use super::invalid_scalar;
use crate::error::{DispatchScalar, Error};

/// Camera parameters for standard or reversed device depth.
///
/// Near and far share the renderer's view-space distance unit. The conversion
/// factor expresses metres per view-space unit; it does not change depth encoding.
/// Near/far are geometric distances with the same ordering in either depth
/// convention. Finite/infinite projection must match the context's depth convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParameters {
    near: f32,
    far: f32,
    vertical_fov_radians: f32,
    view_space_to_meters: f32,
    infinite_far: bool,
}

impl CameraParameters {
    /// Checks the camera domain before native dispatch.
    ///
    /// `near` must be positive and finite, `far` finite and greater than `near`,
    /// vertical FOV finite and strictly between zero and pi radians, and the
    /// metres-per-view-space-unit factor positive and finite. Violations return
    /// [`Error::Input`] identifying the scalar; no native call occurs.
    pub fn new(
        near: f32,
        far: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
    ) -> Result<Self, Error> {
        Self::checked(near, far, vertical_fov_radians, view_space_to_meters, false)
    }

    /// Camera with an infinite far plane; near, FOV and unit checks match [`Self::new`].
    ///
    /// Select an infinite [`DepthConvention`](super::DepthConvention) on the context. The native far
    /// field uses `f32::MAX`, matching the pinned SDK sample's finite placeholder;
    /// IEEE infinity is never passed. The renderer supplies infinite device depth.
    pub fn new_infinite(
        near: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
    ) -> Result<Self, Error> {
        Self::checked(
            near,
            f32::MAX,
            vertical_fov_radians,
            view_space_to_meters,
            true,
        )
    }

    fn checked(
        near: f32,
        far: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
        infinite_far: bool,
    ) -> Result<Self, Error> {
        if !near.is_finite() || near <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::CameraNear));
        }
        if !infinite_far && (!far.is_finite() || far <= near) {
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
            infinite_far,
        })
    }
    /// Near-plane distance in the renderer's view-space units.
    pub fn near(self) -> f32 {
        self.near
    }
    /// Finite far distance, or `f32::MAX` as the native infinite-plane placeholder.
    pub fn far(self) -> f32 {
        self.far
    }
    /// Whether this camera was constructed with an infinite far plane.
    pub fn has_infinite_far_plane(self) -> bool {
        self.infinite_far
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
