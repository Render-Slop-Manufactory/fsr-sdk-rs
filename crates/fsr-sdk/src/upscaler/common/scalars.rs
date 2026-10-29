// SPDX-License-Identifier: MPL-2.0

//! Checked frame scalars and literal motion-vector multipliers.

use super::invalid_scalar;
use crate::error::{DispatchScalar, Error};

/// Two literal native motion-vector scale components.
///
/// This is the native dispatch parameter, not a final UV multiplier or an NDC
/// conversion. Reported open FSR3 arithmetic multiplies stored motion by this
/// scale divided by the active motion extent; other providers may differ.
/// The renderer must match its stored encoding and jitter convention to the
/// actual provider. Finiteness establishes no provider-safe numerical range or
/// finite downstream products. Unsafe dispatch retains those obligations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeMotionVectorScale {
    x: f32,
    y: f32,
}

impl NativeMotionVectorScale {
    /// Accept any finite components, preserving their original bits.
    ///
    /// Zero, negative zero, subnormal and extreme finite values are literal
    /// requests, not defaults. NaN/infinities reject X first, then Y, with the
    /// corresponding [`DispatchScalar`] reason. No clamping or native call occurs.
    pub fn new(x: f32, y: f32) -> Result<Self, Error> {
        let value = Self { x, y };
        value.validate()?;
        Ok(value)
    }

    /// Native horizontal multiplier, with accepted bits unchanged.
    pub fn x(self) -> f32 {
        self.x
    }

    /// Native vertical multiplier, with accepted bits unchanged.
    pub fn y(self) -> f32 {
        self.y
    }

    pub(crate) fn validate(self) -> Result<(), Error> {
        if !self.x.is_finite() {
            return Err(invalid_scalar(DispatchScalar::MotionVectorScaleX));
        }
        if !self.y.is_finite() {
            return Err(invalid_scalar(DispatchScalar::MotionVectorScaleY));
        }
        Ok(())
    }
}

/// Current-frame projection jitter actually applied to rendering, in pixels.
///
/// X and Y describe horizontal and vertical subpixel offsets, not UV or NDC
/// translations. Report the same offset used for both colour and depth projection.
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
    /// return [`Error::Input`] with the corresponding scalar reason.
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
    /// Zero, negative, NaN or infinite values return [`Error::Input`].
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

/// Positive finite factor already applied to the current frame's scene colour.
///
/// In the linear profile, the SDK divides input colour by this factor to recover
/// the original scene signal. Non-linear encoding, normalization and exposure
/// ordering require a caller-established contract with the actual provider;
/// this does not prescribe multiplication of encoded sRGB/PQ code values.
/// This is separate from the explicit exposure texture. The caller must
/// supply matching colour content; this type validates the scalar only, not
/// pixels or a complete provider-supported numerical domain.
/// See the [colour and exposure contract](crate::upscaler#colour-and-exposure-contract).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreExposure(f32);

impl PreExposure {
    /// Accept a positive finite factor, preserving its original `f32` bits.
    ///
    /// Zero (including negative zero), negative values, NaN and infinities return
    /// [`Error::Input`] with [`DispatchScalar::PreExposure`]. No
    /// clamping, reciprocal conversion or native call occurs.
    pub fn new(value: f32) -> Result<Self, Error> {
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::PreExposure));
        }
        Ok(Self(value))
    }

    /// The factor already applied to input colour, passed unchanged to the SDK.
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for PreExposure {
    /// Unmodified scene colour: a factor of `1.0`.
    fn default() -> Self {
        Self(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_motion_scales_preserve_finite_bits_and_reject_each_non_finite_axis() {
        let finite = [
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x7f7f_ffff,
            0xff7f_ffff,
            (-2.0f32).to_bits(),
            4.0f32.to_bits(),
        ];
        for x in finite {
            for y in finite {
                let value =
                    NativeMotionVectorScale::new(f32::from_bits(x), f32::from_bits(y)).unwrap();
                let copy = value;
                assert_eq!((copy.x().to_bits(), copy.y().to_bits()), (x, y));
                assert_result!(copy.validate(), Ok(()));
            }
        }
        for invalid in [
            f32::NAN,
            f32::from_bits(0xffc0_1234),
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            for (x, y, axis) in [
                (invalid, 1.0, DispatchScalar::MotionVectorScaleX),
                (1.0, invalid, DispatchScalar::MotionVectorScaleY),
                (invalid, invalid, DispatchScalar::MotionVectorScaleX),
            ] {
                assert_result!(
                    NativeMotionVectorScale::new(x, y),
                    Err(invalid_scalar(axis))
                );
                // The compulsory boundary guard uses the same check even if a
                // future internal producer bypasses the public constructor.
                assert_result!(
                    NativeMotionVectorScale { x, y }.validate(),
                    Err(invalid_scalar(axis))
                );
            }
        }
    }
}
