// SPDX-License-Identifier: MPL-2.0

//! Provider-routed utilities; formulas and sequences remain in the AMD runtime.
use super::{check, header};
use crate::context_policy::Eligibility;
use crate::{
    error::{Error, InvariantViolation, QueryValidation},
    query::{JitterPhaseCount, QualityMode},
    upscaler::common::{Dimensions, JitterOffsetPixels},
};
use fsr_sdk_sys::{api::ffxQueryDescHeader, upscale::*};

fn invalid(reason: QueryValidation) -> Error {
    Error::Input(crate::error::InputError::Query(reason))
}

fn signed_dimension(value: u32) -> Result<(), Error> {
    if value > i32::MAX as u32 {
        return Err(invalid(QueryValidation::DimensionsOutOfRange));
    }
    Ok(())
}

pub(crate) fn validate_render_size(display: Dimensions) -> Result<(), Error> {
    signed_dimension(display.width())?;
    signed_dimension(display.height())
}

pub(crate) fn validate_jitter_phase_count(
    render: Dimensions,
    display: Dimensions,
) -> Result<(), Error> {
    signed_dimension(render.width())?;
    signed_dimension(display.width())?;
    // Preserve the existing conservative signed phase-count arithmetic domain.
    let scale = display.width() as f32 / render.width() as f32;
    if display.width() < render.width() || scale > 16383.0 {
        return Err(invalid(QueryValidation::JitterScale));
    }
    Ok(())
}

pub(crate) fn validate_jitter_offset(
    index: i32,
    phase_count: JitterPhaseCount,
) -> Result<(), Error> {
    if index < 0 || index >= phase_count.count() {
        return Err(invalid(QueryValidation::JitterIndex));
    }
    Ok(())
}

/// # Safety
/// Invoke the matched global/live query synchronously under the acquisition's
/// utility-query trust, preserving outputs, native coordination and dependencies.
pub(crate) unsafe fn upscale_ratio(
    eligibility: Eligibility,
    mode: QualityMode,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<f32, Error> {
    eligibility.check()?;
    let mut ratio = f32::NAN;
    let mut desc = ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE),
        qualityMode: mode.native_value(),
        pOutUpscaleRatio: &mut ratio,
    };
    check(invoke(&mut desc.header))?;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::InvalidUpscaleRatioOnSuccess,
        ));
    }
    Ok(ratio)
}

/// # Safety
/// Invoke the matched global/live utility query synchronously under acquisition
/// trust, including numerical safety within the checked signed-dimension domain.
pub(crate) unsafe fn render_size_for_quality(
    eligibility: Eligibility,
    display: Dimensions,
    mode: QualityMode,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<Dimensions, Error> {
    eligibility.check()?;
    validate_render_size(display)?;
    let (mut width, mut height) = (0, 0);
    let mut desc = ffxQueryDescUpscaleGetRenderResolutionFromQualityMode {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE),
        displayWidth: display.width(),
        displayHeight: display.height(),
        qualityMode: mode.native_value(),
        pOutRenderWidth: &mut width,
        pOutRenderHeight: &mut height,
    };
    check(invoke(&mut desc.header))?;
    let invalid_result = Error::invariant(
        crate::error::Operation::Query,
        InvariantViolation::InvalidRenderSizeOnSuccess,
    );
    if width > display.width() || height > display.height() {
        return Err(invalid_result);
    }
    Dimensions::new(width, height).map_err(|_| invalid_result)
}

/// # Safety
/// Invoke the matched global/live utility query synchronously under acquisition
/// trust. The wrapper rejects signed-width and known phase-count overflow domains.
pub(crate) unsafe fn jitter_phase_count(
    eligibility: Eligibility,
    render: Dimensions,
    display: Dimensions,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<JitterPhaseCount, Error> {
    eligibility.check()?;
    validate_jitter_phase_count(render, display)?;
    let mut count = 0;
    let mut desc = ffxQueryDescUpscaleGetJitterPhaseCount {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT),
        renderWidth: render.width(),
        displayWidth: display.width(),
        pOutPhaseCount: &mut count,
    };
    check(invoke(&mut desc.header))?;
    JitterPhaseCount::new(count).map_err(|_| {
        Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::InvalidJitterPhaseCountOnSuccess,
        )
    })
}

/// # Safety
/// Invoke the matched global/live utility query synchronously under acquisition
/// trust, preserving output validity, coordination and numerical/provider safety.
pub(crate) unsafe fn jitter_offset(
    eligibility: Eligibility,
    index: i32,
    phase_count: JitterPhaseCount,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<JitterOffsetPixels, Error> {
    eligibility.check()?;
    validate_jitter_offset(index, phase_count)?;
    let (mut x, mut y) = (f32::NAN, f32::NAN);
    let mut desc = ffxQueryDescUpscaleGetJitterOffset {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET),
        index,
        phaseCount: phase_count.count(),
        pOutX: &mut x,
        pOutY: &mut y,
    };
    check(invoke(&mut desc.header))?;
    JitterOffsetPixels::new(x, y).map_err(|_| {
        Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::InvalidJitterOffsetOnSuccess,
        )
    })
}

#[cfg(test)]
#[path = "../utility_tests.rs"]
mod tests;
