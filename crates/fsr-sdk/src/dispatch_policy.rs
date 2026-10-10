// SPDX-License-Identifier: MPL-2.0

//! Private dispatch orchestration, independent of descriptor and ownership family.
use crate::error::{Error, Operation};
use fsr_sdk_sys::api::FFX_API_RETURN_OK;

// Callers supply inspection and invocation mechanics. Only this function owns
// eligibility, preflight propagation and the native-error poison transition.
pub(crate) fn dispatch<S, D>(
    poisoned: &mut bool,
    state: &mut S,
    inspect: impl FnOnce(&S) -> Result<D, Error>,
    invoke: impl FnOnce(&mut S, &D) -> u32,
) -> Result<(), Error> {
    if *poisoned {
        return Err(Error::DispatchPoisoned);
    }
    let desc = inspect(state)?;
    let code = invoke(state, &desc);
    if code == FFX_API_RETURN_OK {
        Ok(())
    } else {
        *poisoned = true;
        Err(Error::Native {
            operation: Operation::Dispatch,
            code,
        })
    }
}
