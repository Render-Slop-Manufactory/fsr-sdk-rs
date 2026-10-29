// SPDX-License-Identifier: MPL-2.0

//! Assertions compare specified diagnostic fields, without a public Error equality contract.
#![allow(dead_code, unused_macros)]
use fsr_sdk::error::{BackendError, Error};

pub fn assert_error_details(actual: &Error, expected: &Error) {
    match (actual, expected) {
        (Error::Input(a), Error::Input(b)) => assert_eq!(a, b),
        (Error::Native(a), Error::Native(b)) => {
            assert_eq!(a.operation(), b.operation());
            assert_eq!(a.code(), b.code());
        }
        (
            Error::Invariant {
                operation: a,
                reason: ar,
            },
            Error::Invariant {
                operation: b,
                reason: br,
            },
        ) => {
            assert_eq!(a, b);
            assert_eq!(ar, br);
        }
        (Error::ContextUnusable(a), Error::ContextUnusable(b)) => assert_eq!(a, b),
        (
            Error::Backend(BackendError::Dx12Inspection(a)),
            Error::Backend(BackendError::Dx12Inspection(b)),
        ) => {
            assert_eq!(a.object(), b.object());
            assert_eq!(a.operation(), b.operation());
            assert_eq!(a.hresult(), b.hresult());
        }
        (Error::QueryAllocation(a), Error::QueryAllocation(b)) => {
            assert_eq!(a.count(), b.count());
            assert_eq!(a.reason(), b.reason());
        }
        _ => panic!("different error categories: actual={actual:?}, expected={expected:?}"),
    }
}
pub fn terminal(error: Error) -> Error {
    let Error::Native(failure) = error else {
        panic!("expected a native failure")
    };
    Error::ContextUnusable(fsr_sdk::error::TerminalCause::Native(failure))
}
macro_rules! assert_error {
    ($actual:expr, $expected:expr $(,)?) => {{
        $crate::test_errors::assert_error_details(&$actual, &$expected);
    }};
}
pub fn assert_results<T: std::fmt::Debug + PartialEq>(
    actual: &Result<T, Error>,
    expected: &Result<T, Error>,
) {
    match (actual, expected) {
        (Ok(actual), Ok(expected)) => assert_eq!(actual, expected),
        (Err(actual), Err(expected)) => assert_error_details(actual, expected),
        (actual, expected) => {
            panic!("different outcomes: actual={actual:?}, expected={expected:?}")
        }
    }
}
macro_rules! assert_result {
    ($actual:expr, Err($expected:expr) $(,)?) => {{
        match &$actual {
            Err(actual) => $crate::test_errors::assert_error_details(actual, &$expected),
            Ok(actual) => panic!("expected rejection, got success: {actual:?}"),
        }
    }};
    ($actual:expr, $expected:expr $(,)?) => {{
        $crate::test_errors::assert_results(&$actual, &$expected);
    }};
}
