// SPDX-License-Identifier: MPL-2.0

use crate::{
    error::{DispatchScalar, Error},
    upscaler::common::invalid_scalar,
};

/// Checked native parameter for an enabled FSR sharpening pass, in `[0, 1]`.
///
/// Zero is the lower parameter endpoint; it does not disable the pass or promise
/// an identity output. The selected provider receives the value unchanged,
/// including the sign of negative zero. This is no guarantee of sharper images,
/// brightness preservation, monotonic response or a particular image quality.
/// See [`Sharpening`] for enablement and the bounded native observations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeSharpness(f32);

impl NativeSharpness {
    /// Accepts finite values in inclusive `[0, 1]`, including negative zero.
    ///
    /// NaN, infinities and out-of-range values return [`Error::Input`]
    /// with [`DispatchScalar::NativeSharpness`]. No clamping or native call occurs.
    pub fn new(value: f32) -> Result<Self, Error> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(invalid_scalar(DispatchScalar::NativeSharpness));
        }
        Ok(Self(value))
    }

    /// The checked native parameter, with its original `f32` bits preserved.
    pub fn value(self) -> f32 {
        self.0
    }
}

/// Per-frame control of the selected FSR provider's native sharpening pass.
///
/// New [`FrameParameters`](super::FrameParameters) starts with [`Self::Disabled`].
/// Native control adds no wrapper filter, quality profile, output clamping or
/// perceptual remapping. It promises no sharpening gain, brightness preservation,
/// monotonic response or particular image quality. Native recording may fail
/// under the existing dispatch error contract.
///
/// [Recorded native observations](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-30-exp-d014-native-sharpening.md)
/// include enabled-zero differences, brightness changes and edge overshoot and
/// undershoot on one scene/provider/GPU/driver configuration. They are bounded
/// observations, not universal filter semantics or public-API verification.
/// [Public-API integration observations](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/release/2026-10-10-m9-verification.md)
/// separately cover mode changes with continuing history. Enabled zero differed
/// from Disabled on those two scenes; this is not a universal output guarantee.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Sharpening {
    /// Disable the native pass, mapping to `false` and positive `0.0`.
    Disabled,
    /// Enable the native pass and forward the checked parameter unchanged.
    /// `Native(0)` remains enabled; use [`Self::Disabled`] to disable the pass.
    Native(NativeSharpness),
}
