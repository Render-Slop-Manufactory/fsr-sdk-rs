// SPDX-License-Identifier: MPL-2.0

//! Portable native diagnostic filtering.

/// Native global message filter; independent of checking/visualization work.
/// Future SDK filters may add variants; downstream matches need a fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DebugLevel {
    /// Silence the configured output route.
    #[default]
    Silent,
    /// Report errors.
    Errors,
    /// Report errors and warnings.
    Warnings,
    /// Request the SDK's verbose output level.
    Verbose,
}
impl DebugLevel {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn native(self) -> u32 {
        match self {
            Self::Silent => 0,
            Self::Errors => 1,
            Self::Warnings => 2,
            Self::Verbose => 0x0fff_ffff,
        }
    }
}
