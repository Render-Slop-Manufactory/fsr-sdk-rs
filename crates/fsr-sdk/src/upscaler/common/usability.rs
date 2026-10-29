// SPDX-License-Identifier: MPL-2.0

use crate::error::TerminalCause;

/// Snapshot of the wrapper's known context eligibility, without GPU readiness.
///
/// A usable context may still have pending recordings or outstanding GPU work.
/// This value tracks neither submission nor completion and establishes no native
/// capability, recovery or permission to free resources. A copied snapshot does
/// not update when the context subsequently becomes unusable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextUsability {
    /// The wrapper has not recorded a terminal cause.
    Usable,
    /// Live operations are blocked by the first known terminal cause.
    Unusable(TerminalCause),
}
