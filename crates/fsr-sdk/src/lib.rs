// SPDX-License-Identifier: MPL-2.0

//! Private Windows/DX12 lifecycle foundation. Public construction and dispatch
//! are not exposed yet. Native error values remain available on every host.

pub mod error;

// Until M5 exposes construction, these production internals have no public caller.
#[cfg(any(test, all(windows, feature = "dx12")))]
#[allow(dead_code)]
mod context;
#[cfg(all(windows, feature = "dx12"))]
#[allow(dead_code)]
mod upscaler;

#[cfg(all(windows, feature = "dx12"))]
mod runtime {
    pub(crate) use fsr_sdk_sys::loader::FfxLibrary;
}

// Host-only fixture adapter. It does not enable an AMD runtime on other OSes.
#[cfg(all(test, not(all(windows, feature = "dx12"))))]
#[path = "context/test_runtime.rs"]
mod runtime;
