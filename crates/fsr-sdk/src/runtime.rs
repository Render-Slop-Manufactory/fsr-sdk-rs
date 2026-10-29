// SPDX-License-Identifier: MPL-2.0

//! Explicit trusted library loading; effect operations require unsafe setup acquisition.
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
pub(crate) mod queries;

/// Loaded library with local ownership, without safe effect operations.
///
/// Neither Send, Sync nor Clone. Effect setup acquisition establishes its own
/// lasting contract. Derived contexts retain the library independently; dropping
/// this public value does not unload it while retained owners remain.
pub struct Runtime {
    pub(crate) state: Rc<RuntimeState>,
}

#[cfg(feature = "movable")]
mod movable;
#[cfg(feature = "movable")]
pub use movable::MovableRuntime;

impl Runtime {
    /// Load the explicit trusted Windows/DX12 runtime and resolve five exports.
    ///
    /// The path is canonicalized; no download, automatic discovery, device
    /// binding or upscaler query/create is performed. Native initialization may
    /// run while loading. Acquire `upscaler_dx12` before using effect operations.
    ///
    /// # Errors
    /// Returns an owned LoadError for path, library or symbol failure, preserving
    /// its source. Native initialization faults need not become Rust errors.
    ///
    /// # Safety
    /// The DLL and all resolved dependencies must be trusted code with the pinned
    /// v2.3.0 ABI/calling conventions and valid initialization, required exports,
    /// partial-load cleanup and termination. Symbol lookup does not authenticate
    /// code or verify native behaviour. Keep compatible process/provider state
    /// and native dependencies alive for this runtime and every derived/retained
    /// owner, including exceptional failure retention after public owners drop.
    ///
    /// Serialize the entire common native domain without reentry: loading,
    /// native calls, error cleanup, explicit/implicit Drop, unwinding and final
    /// library release. Coordinate siblings, independent acquisitions, local and
    /// movable users, external integrations and hidden native activity. Different
    /// paths/devices do not prove isolation. Prior callback installers maintain
    /// their independent lasting target/allocator/coordination duties.
    ///
    /// All wrapper native use and release remain on this acquiring thread.
    /// The local runtime and its contexts are neither Send nor Sync.
    ///
    /// Effect-specific query, construction, Configure and recording trust is
    /// established by unsafe setup acquisition, not this load. These common
    /// duties persist after the public runtime drops while dependent or retained
    /// objects remain; no wrapper lock or GPU wait establishes them for you.
    /// ABI/lifetime/threading assumptions are caller-established native trust,
    /// not a universal AMD initialization, worker-shutdown or unload guarantee.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        // SAFETY: Caller establishes trusted initialization/failure/release and
        // this family's lasting native lifetime and coordination obligations.
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
    fn public_load_maps_path_error_and_retains_io_source() {
        // SAFETY: An empty path fails canonicalization before any DLL is loaded.
        let error = unsafe { Runtime::load("") }
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
