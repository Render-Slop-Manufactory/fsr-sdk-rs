// SPDX-License-Identifier: MPL-2.0

//! Nonzero pixel dimensions without device or provider assumptions.

use crate::error::Error;
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
    /// Returns [`Error::Input`] with the original values on failure.
    pub fn new(width: u32, height: u32) -> Result<Self, Error> {
        let Some(width_checked) = NonZeroU32::new(width) else {
            return Err(Error::Input(crate::error::InputError::InvalidDimensions {
                width,
                height,
            }));
        };
        let Some(height_checked) = NonZeroU32::new(height) else {
            return Err(Error::Input(crate::error::InputError::InvalidDimensions {
                width,
                height,
            }));
        };
        Ok(Self {
            width: width_checked,
            height: height_checked,
        })
    }
}
