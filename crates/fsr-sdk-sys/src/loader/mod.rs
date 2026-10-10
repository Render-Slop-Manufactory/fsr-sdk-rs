// SPDX-License-Identifier: MPL-2.0

//! Explicit-path loading of the five FidelityFX API entry points.
//!
//! This interface currently requires Windows and the `dx12` feature. The private
//! platform implementation owns loading, symbol resolution and DLL retention.
//! Platform organization does not establish another native runtime or backend.

mod windows;

pub use windows::{FfxLibrary, LoadError};
