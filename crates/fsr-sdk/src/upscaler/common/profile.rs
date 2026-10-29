// SPDX-License-Identifier: MPL-2.0

//! Portable creation-profile selections and per-frame colour hint bits.

/// Device-depth convention and far-plane projection for a context.
///
/// The caller supplies matching device depth in a supported resource format.
/// Geometric camera distances
/// remain positive with `near < far` for finite projections; do not swap them to
/// select reversed depth. Infinite projections use [`CameraParameters::new_infinite`](super::CameraParameters::new_infinite).
/// Linearized depth is unsupported.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DepthConvention {
    /// Ordinary device depth: near zero, far one. The default.
    #[default]
    StandardFinite,
    /// Reversed device depth: near one, far zero.
    ReversedFinite,
    /// Ordinary device depth with an infinite far plane.
    StandardInfinite,
    /// Reversed device depth with an infinite far plane.
    ReversedInfinite,
}

impl DepthConvention {
    /// Whether the projection has an infinite far plane.
    pub fn is_infinite(self) -> bool {
        matches!(self, Self::StandardInfinite | Self::ReversedInfinite)
    }
}

/// Resolution of the context's motion-vector texture.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MotionVectorResolution {
    /// Texture matches the render allocation; vectors describe its active region.
    #[default]
    Render,
    /// Texture matches maximum output allocation; vectors describe active output.
    Display,
}

/// Whether supplied motion vectors contain projection jitter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MotionVectorJitter {
    /// Renderer has removed projection jitter. The default.
    #[default]
    Excluded,
    /// Vectors contain projection jitter; request native jitter cancellation.
    Included,
}

/// Exposure source selected before context creation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExposureMode {
    /// Caller supplies a positive finite multiplier in a 1×1 R32F texture
    /// or an R32 typeless allocation with its fixed FLOAT view.
    #[default]
    Explicit,
    /// SDK computes exposure internally; omit the exposure texture.
    Automatic,
}

/// Input classification requested when creating a context.
///
/// This controls the native non-linear-colour flag, without converting pixels
/// or detecting provider support. HDR remains an independent setting.
/// See the [colour and exposure contract](crate::upscaler#colour-and-exposure-contract).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputColorSpace {
    /// Non-linear creation flag is off. The default linear profile.
    #[default]
    Linear,
    /// Request the provider's non-linear input profile.
    NonLinear,
}

/// Native hint describing the supplied colour encoding for one frame.
///
/// These are provider controls, not encoding/decoding operations or guarantees
/// of a particular output transfer function. The renderer must establish the
/// actual provider's signal, exposure and output contract.
/// See the [colour and exposure contract](crate::upscaler#colour-and-exposure-contract).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ColorEncodingHint {
    /// No hint bits. This does not assert that the input is linear.
    #[default]
    Unspecified,
    /// Request the native perceptual sRGB hint.
    Srgb,
    /// Request the native perceptual PQ hint; no luminance normalization is applied.
    Pq,
}

impl ColorEncodingHint {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn native_flags(self) -> u32 {
        match self {
            Self::Unspecified => 0,
            Self::Srgb => fsr_sdk_sys::upscale::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB,
            Self::Pq => fsr_sdk_sys::upscale::FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ,
        }
    }
}
