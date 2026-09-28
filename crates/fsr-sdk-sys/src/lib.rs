// SPDX-License-Identifier: MPL-2.0

//! Selected raw FidelityFX ABI declarations pinned to AMD SDK v2.3.0.
//!
//! This is a reviewed ABI slice, not complete SDK bindings. It includes common
//! entry-point/descriptor types and the selected upscaler creation and dispatch
//! payloads. The explicit DLL loader and DX12 declarations require Windows and
//! the `dx12` feature. Normal Windows/DX12 consumers should prefer `fsr-sdk` for
//! context ownership, checked inputs, errors and the documented dispatch boundary.
//!
//! The supported native target is `x86_64-pc-windows-msvc` with the `dx12` feature,
//! which this raw crate does not enable by default. Windows/DX12 APIs are exposed
//! on other Windows targets, but those targets are currently unverified and
//! unsupported; compilation may succeed. Native verification is limited to the
//! recorded SDK, runtime/provider, GPU, driver and workload configurations; target
//! support does not imply compatibility with every hardware/provider configuration.
//! The [project scope](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/SCOPE.md#platform-and-compatibility-boundaries)
//! owns this policy; the [M6b record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
//! preserves one tested public-dispatch configuration.
//!
//! Raw callers must establish descriptor/pointer validity, provider compatibility,
//! native threading rules and dependency lifetimes themselves. GPU resource
//! states, submission, ordering and completion are additional caller obligations;
//! ABI fidelity does not make native calls safe.
//!
//! The [ABI coverage inventory](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/ABI_COVERAGE.md)
//! records selected slices and their checks. The
//! [package README](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk-sys/README.md)
//! records pinned header provenance and retained AMD notices. No SDK headers or
//! runtime binaries are bundled.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod api;
pub mod upscale;

#[cfg(all(windows, feature = "dx12"))]
pub mod loader;

#[cfg(all(windows, feature = "dx12"))]
pub mod dx12;

#[cfg(feature = "vulkan")]
compile_error!(
    "the `vulkan` feature is intentionally unsupported: \
     the modern AMD FSR SDK 2.3.0 path used by fsr-sdk-rs \
     does not provide the Vulkan backend required by this wrapper"
);
