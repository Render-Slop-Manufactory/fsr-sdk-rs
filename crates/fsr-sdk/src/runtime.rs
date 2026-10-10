// SPDX-License-Identifier: MPL-2.0

//! Explicit local ownership and optional movable ownership of a FidelityFX loader.
//!
//! Runtime ownership retains the library and common entry points, with no GPU
//! resources. The current loader and native acquisition contracts require
//! Windows/DX12, so this module retains that platform/feature gate.

use fsr_sdk_sys::loader::LoadError as SysLoadError;
use std::{
    error::Error as StdError,
    fmt,
    path::{Path, PathBuf},
    rc::Rc,
};

/// Failure to resolve a loader path, load its DLL, or find a required export.
///
/// The underlying I/O or dynamic-loader error is retained as the source without
/// making the raw crate's error types part of this API.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadError {
    /// The explicit DLL path could not be resolved before loading.
    Path {
        /// Path supplied by the caller.
        path: PathBuf,
        /// Owned path-resolution cause, also returned by `Error::source`.
        source: Box<dyn StdError + Send + Sync>,
    },
    /// Loading the resolved DLL failed, potentially during native initialization.
    Library {
        /// Resolved DLL path that was attempted.
        path: PathBuf,
        /// Owned loading cause, also returned by `Error::source`.
        source: Box<dyn StdError + Send + Sync>,
    },
    /// The DLL loaded but a required FidelityFX entry point was absent.
    Symbol {
        /// Resolved DLL path.
        path: PathBuf,
        /// Required export that could not be resolved.
        symbol: &'static str,
        /// Owned symbol-resolution cause, also returned by `Error::source`.
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl LoadError {
    fn from_sys(error: SysLoadError) -> Self {
        match error {
            SysLoadError::Path { path, source } => Self::Path {
                path,
                source: Box::new(source),
            },
            SysLoadError::Library { path, source } => Self::Library {
                path,
                source: Box::new(source),
            },
            SysLoadError::Symbol {
                path,
                symbol,
                source,
            } => Self::Symbol {
                path,
                symbol,
                source: Box::new(source),
            },
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { path, source } => write!(
                f,
                "could not resolve FidelityFX DLL path '{}': {source}",
                path.display()
            ),
            Self::Library { path, source } => write!(
                f,
                "could not load FidelityFX DLL '{}': {source}",
                path.display()
            ),
            Self::Symbol {
                path,
                symbol,
                source,
            } => write!(
                f,
                "FidelityFX DLL '{}' is missing required symbol '{symbol}': {source}",
                path.display()
            ),
        }
    }
}

impl StdError for LoadError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Path { source, .. }
            | Self::Library { source, .. }
            | Self::Symbol { source, .. } => source.as_ref(),
        })
    }
}

mod state;
pub(crate) use state::RuntimeState;

/// A loaded Windows/DX12 FidelityFX runtime.
///
/// Contexts created from this runtime each retain it independently. Dropping
/// this value does not unload the library while a context is alive. This type
/// is neither `Send` nor `Sync`; all wrapper operations stay on its acquiring
/// thread. The wrapper neither downloads the SDK nor exposes loader discovery or
/// provider-selection APIs; native dependency/provider resolution still applies.
/// It does not expose native functions or context handles.
pub struct Runtime {
    pub(crate) state: Rc<RuntimeState>,
}

#[cfg(feature = "movable")]
mod movable;
#[cfg(feature = "movable")]
pub use movable::MovableRuntime;

impl Runtime {
    /// Load an explicit FidelityFX loader DLL path.
    ///
    /// For the supported signed v2.3.0 DX12 upscaler deployment, place
    /// `amd_fidelityfx_upscaler_dx12.dll` beside the executable. Dependencies
    /// and providers may also be resolved through Windows or the driver.
    /// The wrapper canonicalizes the path and resolves its five required exports.
    /// It neither creates a context nor verifies general GPU/provider support.
    ///
    /// `_unchecked` refers to the caller-established runtime trust and native
    /// integration obligations below. Path canonicalization and required export
    /// resolution still occur. The name reserves `load` for a possible future
    /// safe acquisition API; such an API is not provided.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] for path resolution, loading or a missing export.
    /// Native initialization faults need not become a Rust error.
    ///
    /// # Safety
    ///
    /// The loader and every resolved dependency/provider must be trusted,
    /// compatible with the v2.3.0 ABI, and safe to initialize and terminate,
    /// including after partial load failure. Native initialization, teardown
    /// and threading preconditions must hold. For this runtime and all its
    /// dependent contexts, keep external/native FidelityFX use touching shared
    /// state serialized and non-reentrant, preserve compatible process/provider
    /// configuration, and do not prematurely release native dependencies. A
    /// different path or device does not prove isolation. These obligations
    /// persist after this value is dropped while contexts remain, and an
    /// exceptional native failure can retain dependencies indefinitely.
    pub unsafe fn load_unchecked(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        // SAFETY: The caller establishes the native acquisition obligations.
        let state = unsafe { RuntimeState::load_unchecked(path) }?;
        Ok(Self {
            state: Rc::new(state),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_load_unchecked_maps_path_error_and_retains_io_source() {
        // SAFETY: An empty path fails canonicalization before any DLL is loaded.
        let error = unsafe { Runtime::load_unchecked("") }
            .err()
            .expect("empty path must fail");
        assert!(matches!(&error, LoadError::Path { path, .. } if path.as_os_str().is_empty()));
        assert!(
            StdError::source(&error)
                .and_then(|source| source.downcast_ref::<std::io::Error>())
                .is_some()
        );
    }

    #[test]
    fn library_and_symbol_errors_keep_their_causes() {
        let make_source = || {
            // SAFETY: An empty path cannot name a DLL to initialize.
            unsafe { libloading::Library::new("") }.expect_err("empty path must fail")
        };
        let path = PathBuf::from("loader.dll");
        let library = LoadError::from_sys(SysLoadError::Library {
            path: path.clone(),
            source: make_source(),
        });
        assert!(matches!(&library, LoadError::Library { path: p, .. } if p == &path));
        assert!(
            StdError::source(&library)
                .and_then(|source| source.downcast_ref::<libloading::Error>())
                .is_some()
        );

        let symbol = LoadError::from_sys(SysLoadError::Symbol {
            path: path.clone(),
            symbol: "ffxDispatch",
            source: make_source(),
        });
        assert!(
            matches!(&symbol, LoadError::Symbol { path: p, symbol: "ffxDispatch", .. } if p == &path)
        );
        assert!(
            StdError::source(&symbol)
                .and_then(|source| source.downcast_ref::<libloading::Error>())
                .is_some()
        );
    }
}
