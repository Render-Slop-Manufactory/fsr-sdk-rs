// SPDX-License-Identifier: MPL-2.0

//! Shared context eligibility and dispatch orchestration, independent of ownership family.
use crate::error::{Error, NativeError, Operation, TerminalCause};
use fsr_sdk_sys::api::FFX_API_RETURN_OK;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Eligibility {
    #[default]
    Usable,
    Unusable(TerminalCause),
}

impl Eligibility {
    pub(crate) fn usability(self) -> crate::upscaler::ContextUsability {
        match self {
            Self::Usable => crate::upscaler::ContextUsability::Usable,
            Self::Unusable(cause) => crate::upscaler::ContextUsability::Unusable(cause),
        }
    }

    pub(crate) fn mark_recording_abandoned(&mut self) {
        if matches!(self, Self::Usable) {
            *self = Self::Unusable(TerminalCause::RecordingAbandoned);
        }
    }

    pub(crate) fn check(self) -> Result<(), Error> {
        match self {
            Self::Usable => Ok(()),
            Self::Unusable(cause) => Err(Error::ContextUnusable(cause)),
        }
    }

    // Only invoked after check; query errors never enter this transition.
    pub(crate) fn record(&mut self, operation: Operation, code: u32) -> Result<(), Error> {
        self.check()?;
        if code == FFX_API_RETURN_OK {
            return Ok(());
        }
        assert!(
            matches!(
                operation,
                Operation::Upscale
                    | Operation::GenerateReactiveMask
                    | Operation::Tuning
                    | Operation::ConfigureContextDebugLegacy
            ),
            "only mutating context operations establish terminality"
        );
        let failure = NativeError::new(operation, code);
        *self = Self::Unusable(TerminalCause::Native(failure));
        Err(Error::Native(failure))
    }
}

// Callers supply inspection and invocation mechanics. Only this function owns
// eligibility, preflight propagation and the native-error poison transition.
pub(crate) fn dispatch<S, D>(
    operation: Operation,
    eligibility: &mut Eligibility,
    state: &mut S,
    inspect: impl FnOnce(&S) -> Result<D, Error>,
    invoke: impl FnOnce(&mut S, &D) -> u32,
) -> Result<(), Error> {
    eligibility.check()?;
    let desc = inspect(state)?;
    let code = invoke(state, &desc);
    eligibility.record(operation, code)
}

#[cfg(test)]
pub(crate) fn failed(operation: Operation, code: u32) -> Eligibility {
    Eligibility::Unusable(TerminalCause::Native(NativeError::new(operation, code)))
}
