// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Stage at which fallible query-buffer preparation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryAllocationReason {
    /// Native count cannot be represented as a host usize.
    CountNotRepresentable,
    /// A vector reservation failed, including capacity overflow.
    ReservationFailed,
}
/// Query-buffer count, failure reason and original conversion/reservation source.
#[derive(Debug)]
pub struct QueryAllocationError {
    count: u64,
    reason: QueryAllocationReason,
    source: Box<dyn std::error::Error + Send + Sync>,
}
impl QueryAllocationError {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn new(
        count: u64,
        reason: QueryAllocationReason,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            count,
            reason,
            source: Box::new(source),
        }
    }
    /// Number of provider slots requested by the native count query.
    pub fn count(&self) -> u64 {
        self.count
    }
    /// Conversion or reservation stage that failed.
    pub fn reason(&self) -> QueryAllocationReason {
        self.reason
    }
}
impl fmt::Display for QueryAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "could not allocate FSR query storage for {} providers: {:?}",
            self.count, self.reason
        )
    }
}
impl std::error::Error for QueryAllocationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}
