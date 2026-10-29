// SPDX-License-Identifier: MPL-2.0

//! Checked upscaler configuration and frame scalars without GPU object dependencies.

#[path = "common/camera.rs"]
mod camera;
pub use camera::CameraParameters;

#[path = "common/diagnostics.rs"]
mod diagnostics;
pub use diagnostics::DebugLevel;

#[path = "common/dimensions.rs"]
mod dimensions;
pub use dimensions::Dimensions;

#[path = "common/frame.rs"]
mod frame;
pub use frame::{
    ColorEncodingRequest, FrameParameters, HistoryAction, MotionVectorScale, ReactiveMaskParameters,
};

#[path = "common/options.rs"]
mod options;
pub use options::UpscalerOptions;

#[path = "common/profile.rs"]
mod profile;
pub use profile::{
    ColorEncodingHint, DepthConvention, ExposureMode, InputColorSpace, MotionVectorJitter,
    MotionVectorResolution,
};

#[path = "common/reactive_mask.rs"]
mod reactive_mask;
pub use reactive_mask::ReactiveMaskOptions;

#[path = "common/scalars.rs"]
mod scalars;
pub use scalars::{FrameTimeMillis, JitterOffsetPixels, NativeMotionVectorScale, PreExposure};

#[path = "common/sharpening.rs"]
mod sharpening;
pub use sharpening::{NativeSharpness, Sharpening};

#[path = "common/tuning.rs"]
mod tuning;
pub use tuning::{TuningParameter, TuningRequest, TuningValue};

#[path = "common/usability.rs"]
mod usability;
pub use usability::ContextUsability;

#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) use options::{check_mask_requirements, check_resource_requirements};

use crate::error::{DispatchScalar, DispatchValidation, Error};

pub(in crate::upscaler) fn invalid_scalar(scalar: DispatchScalar) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Scalar(scalar),
    ))
}

#[cfg(test)]
#[path = "common/tests.rs"]
mod tests;
