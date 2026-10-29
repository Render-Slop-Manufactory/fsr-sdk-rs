// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Backend inspection failures, available without backend dependencies.
#[derive(Debug)]
#[non_exhaustive]
pub enum BackendError {
    /// DX12/COM inspection failed before SDK recording.
    Dx12Inspection(Dx12InspectionError),
}
/// Owned DX12 failure metadata and the original error source.
///
/// Public signatures expose portable values; no Windows type is required.
#[derive(Debug)]
pub struct Dx12InspectionError {
    object: InspectionObject,
    operation: InspectionOperation,
    hresult: i32,
    source: Box<dyn std::error::Error + Send + Sync>,
}
impl Dx12InspectionError {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn new(
        object: InspectionObject,
        operation: InspectionOperation,
        hresult: i32,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self {
            object,
            operation,
            hresult,
            source,
        }
    }
    /// Object whose inspection failed, including the resource role where known.
    pub fn object(&self) -> InspectionObject {
        self.object
    }
    /// Backend inspection operation, distinct from an SDK operation.
    pub fn operation(&self) -> InspectionOperation {
        self.operation
    }
    /// Exact HRESULT bits stored as the native signed 32-bit value.
    pub fn hresult(&self) -> i32 {
        self.hresult
    }
}
impl fmt::Display for Dx12InspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DX12 {:?} of {:?} failed with HRESULT {:#010x}",
            self.operation, self.object, self.hresult as u32
        )
    }
}
impl std::error::Error for Dx12InspectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}
impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dx12Inspection(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dx12Inspection(error) => Some(error),
        }
    }
}
