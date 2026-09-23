// SPDX-License-Identifier: MPL-2.0

use std::fmt;

/// Native lifecycle operation that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Create,
    Destroy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantViolation {
    NullContextOnSuccess,
}

/// Native codes are open u32 values, including codes unknown to this SDK version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Native { operation: Operation, code: u32 },
    NativeInvariantViolation(InvariantViolation),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native { operation, code } => {
                write!(f, "FSR {operation:?} failed with native code {code}")
            }
            Self::NativeInvariantViolation(InvariantViolation::NullContextOnSuccess) => {
                f.write_str("FSR creation returned success with a null context")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unknown_codes_and_operation() {
        let error = Error::Native {
            operation: Operation::Destroy,
            code: u32::MAX,
        };
        assert_eq!(
            error.to_string(),
            "FSR Destroy failed with native code 4294967295"
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}
