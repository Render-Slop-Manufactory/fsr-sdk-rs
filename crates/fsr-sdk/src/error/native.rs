// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Exact failure of one wrapper operation at a native SDK entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeError {
    pub(crate) operation: Operation,
    pub(crate) code: u32,
}
impl NativeError {
    #[allow(dead_code)] // Also used by standalone ownership compile controls.
    pub(crate) const fn new(operation: Operation, code: u32) -> Self {
        Self { operation, code }
    }
    /// The operation which failed, including the recording/Configure route.
    pub const fn operation(self) -> Operation {
        self.operation
    }
    /// Unmodified SDK failure code, including unknown values.
    pub const fn code(self) -> u32 {
        self.code
    }
}
impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FSR {:?} failed with native code {}",
            self.operation, self.code
        )
    }
}
impl std::error::Error for NativeError {}

/// First known terminal cause, without implying GPU completion or recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalCause {
    /// A mutating native context operation failed; its exact origin is retained.
    Native(NativeError),
    /// The renderer marked a recording using this context as abandoned.
    /// This is a local observation, without invalidation, submission or GPU completion.
    RecordingAbandoned,
}
impl fmt::Display for TerminalCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(error) => error.fmt(f),
            Self::RecordingAbandoned => f.write_str("renderer-marked recording abandonment"),
        }
    }
}
impl std::error::Error for TerminalCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native(error) => Some(error),
            Self::RecordingAbandoned => None,
        }
    }
}
