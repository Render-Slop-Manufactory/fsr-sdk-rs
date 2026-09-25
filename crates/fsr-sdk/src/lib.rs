// SPDX-License-Identifier: MPL-2.0

//! Windows/DX12 upscaler construction, ownership and bounded unsafe dispatch.
//! Native error values remain available on every host.

pub mod error;

#[cfg(any(test, all(windows, feature = "dx12")))]
mod context;
#[cfg(all(windows, feature = "dx12"))]
pub mod upscaler;

#[cfg(all(windows, feature = "dx12"))]
pub mod runtime;

// Host-only fixture adapter. It does not enable an AMD runtime on other OSes.
#[cfg(all(test, not(all(windows, feature = "dx12"))))]
#[path = "context/test_runtime.rs"]
mod runtime;
