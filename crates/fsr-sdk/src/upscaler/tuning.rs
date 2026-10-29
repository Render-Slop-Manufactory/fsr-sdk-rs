// SPDX-License-Identifier: MPL-2.0

//! Context-specific tuning mapping and native-error policy shared by both families.
use super::{TuningParameter, TuningRequest, TuningValue};
use crate::{
    context_policy::Eligibility,
    error::{Error, Operation},
};
use fsr_sdk_sys::{
    api::{ffxApiHeader, ffxConfigureDescHeader},
    upscale::*,
};
use std::ptr::null_mut;

fn key(parameter: TuningParameter) -> u64 {
    u64::from(match parameter {
        TuningParameter::VelocityFactor => FFX_API_CONFIGURE_UPSCALE_KEY_FVELOCITYFACTOR,
        TuningParameter::ReactivenessScale => FFX_API_CONFIGURE_UPSCALE_KEY_FREACTIVENESSSCALE,
        TuningParameter::ShadingChangeScale => FFX_API_CONFIGURE_UPSCALE_KEY_FSHADINGCHANGESCALE,
        TuningParameter::AccumulationAddedPerFrame => {
            FFX_API_CONFIGURE_UPSCALE_KEY_FACCUMULATIONADDEDPERFRAME
        }
        TuningParameter::MinDisocclusionAccumulation => {
            FFX_API_CONFIGURE_UPSCALE_KEY_FMINDISOCCLUSIONACCUMULATION
        }
    })
}

/// # Safety
/// Invocation must satisfy the actual-provider acquisition trust for the exposed
/// parameter domains/null resets, synchronous non-retention even on error and terminal
/// destroyability. Prior unsafe dispatches must have established an idle context;
/// retain the owner/runtime and common native coordination through invocation.
pub(crate) unsafe fn configure(
    eligibility: &mut Eligibility,
    request: TuningRequest,
    invoke: impl FnOnce(*const ffxConfigureDescHeader) -> u32,
) -> Result<(), Error> {
    eligibility.check()?;
    let mut scalar = match request.value() {
        TuningValue::Value(value) => Some(value),
        TuningValue::ProviderDefault => None,
    };
    let desc = ffxConfigureDescUpscaleKeyValue {
        header: ffxApiHeader {
            r#type: FFX_API_CONFIGURE_DESC_TYPE_UPSCALE_KEYVALUE,
            pNext: null_mut(),
        },
        key: key(request.parameter()),
        u64: 0,
        ptr: scalar
            .as_mut()
            .map_or(null_mut(), |value| std::ptr::from_mut(value).cast()),
    };
    // Descriptor and optional float remain at final addresses until this returns.
    let code = invoke(&desc.header);
    eligibility.record(Operation::Tuning, code)
}

#[cfg(test)]
mod tests;
