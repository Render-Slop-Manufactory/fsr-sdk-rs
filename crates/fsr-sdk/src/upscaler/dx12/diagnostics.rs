// SPDX-License-Identifier: MPL-2.0

//! Exact native diagnostic descriptors; no callback ownership or registry.
use super::interop::MessageCallback;
use crate::{
    error::{Error, Operation},
    runtime::RuntimeState,
    upscaler::DebugLevel,
};
use fsr_sdk_sys::api::*;
use std::ptr::null_mut;

pub(crate) fn with_descriptor(
    level: DebugLevel,
    callback: Option<MessageCallback>,
    legacy: bool,
    invoke: impl FnOnce(*const ffxConfigureDescHeader) -> u32,
) -> u32 {
    let header = |tag| ffxApiHeader {
        r#type: tag,
        pNext: null_mut(),
    };
    if legacy {
        let desc = ffxConfigureDescGlobalDebug1 {
            header: header(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1),
            fpMessage: callback,
            debugLevel: level.native(),
        };
        invoke(&desc.header)
    } else {
        let desc = ffxConfigureDescGlobalDebug {
            header: header(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG),
            effectId: FFX_API_EFFECT_ID_UPSCALE,
            fpMessage: callback,
            debugLevel: level.native(),
        };
        invoke(&desc.header)
    }
}

/// # Safety
/// Establish the interop module's lasting callback contract, idle affected
/// contexts and synchronous descriptor consumption on both success and error.
pub(crate) unsafe fn runtime(
    runtime: &RuntimeState,
    level: DebugLevel,
    callback: Option<MessageCallback>,
    legacy: bool,
) -> Result<(), Error> {
    let code = with_descriptor(level, callback, legacy, |desc| {
        // SAFETY: Caller establishes shared-state coordination, native input
        // consumption and target lifetime; the runtime lives throughout the call.
        unsafe { runtime.configure(null_mut(), desc) }
    });
    if code == FFX_API_RETURN_OK {
        Ok(())
    } else {
        Err(Error::native(
            if legacy {
                Operation::ConfigureGlobalDebugLegacy
            } else {
                Operation::ConfigureDebug
            },
            code,
        ))
    }
}
