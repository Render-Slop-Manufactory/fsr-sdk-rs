// SPDX-License-Identifier: MPL-2.0

//! Explicit-path loading of the five FidelityFX API entry points.
//!
//! No discovery or SDK calls are performed during loading. Windows may execute
//! DLL initialization/termination code. Dependency resolution remains Windows'
//! responsibility; this module does not change the process DLL search path.

use crate::api::*;
use libloading::Library;
use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// Failure to resolve an explicit file, load it, or resolve a required export.
#[derive(Debug)]
pub enum LoadError {
    Path {
        path: PathBuf,
        source: std::io::Error,
    },
    Library {
        path: PathBuf,
        source: libloading::Error,
    },
    Symbol {
        path: PathBuf,
        symbol: &'static str,
        source: libloading::Error,
    },
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

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Path { source, .. } => source,
            Self::Library { source, .. } | Self::Symbol { source, .. } => source,
        })
    }
}

/// Owns the DLL and keeps its entry points private.
///
/// Calls borrow this owner. Native contexts, callbacks and GPU work are not
/// tracked: callers must keep this object alive for all dependent native state.
/// This is not a safe context wrapper and does not serialize native calls.
///
/// An invocation bound to this owner cannot outlive it:
///
/// ```compile_fail
/// use fsr_sdk_sys::loader::FfxLibrary;
/// fn cannot_outlive_owner(library: FfxLibrary) {
///     let query = || unsafe { library.ffxQuery(std::ptr::null_mut(), std::ptr::null_mut()) };
///     drop(library);
///     query();
/// }
/// ```
///
/// The stored function pointer is not public:
///
/// ```compile_fail
/// use fsr_sdk_sys::loader::FfxLibrary;
/// fn cannot_extract_pointer(library: FfxLibrary) {
///     let pointer = library.ffxQuery;
/// }
/// ```
pub struct FfxLibrary {
    ffxCreateContext: PfnFfxCreateContext,
    ffxDestroyContext: PfnFfxDestroyContext,
    ffxConfigure: PfnFfxConfigure,
    ffxQuery: PfnFfxQuery,
    ffxDispatch: PfnFfxDispatch,
    _library: Library,
}

impl FfxLibrary {
    /// Load exactly the supplied file and resolve all five exports.
    ///
    /// Relative paths are resolved against the current directory and canonicalized
    /// before loading, so a bare name never triggers a PATH search for the main DLL.
    /// No version or authenticity check is performed. Dependencies still follow
    /// Windows loading rules. On failure, any acquired library handle is dropped.
    ///
    /// # Safety
    ///
    /// The file and its dependencies must be trusted and safe to initialize and
    /// unload, including unloading after partial symbol-resolution failure. Any
    /// matching exports must have the corresponding SDK v2.3.0 ABI. Native
    /// initialization, termination and threading preconditions must be satisfied.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        let path = path.canonicalize().map_err(|source| LoadError::Path {
            path: path.to_owned(),
            source,
        })?;
        // SAFETY: The caller guarantees initialization and termination safety.
        let library = unsafe { Library::new(&path) }.map_err(|source| LoadError::Library {
            path: path.clone(),
            source,
        })?;
        // SAFETY: The caller guarantees the ABI for this exact named export.
        // Windows libloading rejects null GetProcAddress results; success is Some.
        let ffxCreateContext = unsafe { library.get::<PfnFfxCreateContext>(c"ffxCreateContext") }
            .map(|symbol| *symbol)
            .map_err(|source| LoadError::Symbol {
                path: path.clone(),
                symbol: "ffxCreateContext",
                source,
            })?;
        // SAFETY: The caller guarantees the ABI for this exact named export.
        // Windows libloading rejects null GetProcAddress results; success is Some.
        let ffxDestroyContext =
            unsafe { library.get::<PfnFfxDestroyContext>(c"ffxDestroyContext") }
                .map(|symbol| *symbol)
                .map_err(|source| LoadError::Symbol {
                    path: path.clone(),
                    symbol: "ffxDestroyContext",
                    source,
                })?;
        // SAFETY: The caller guarantees the ABI for this exact named export.
        // Windows libloading rejects null GetProcAddress results; success is Some.
        let ffxConfigure = unsafe { library.get::<PfnFfxConfigure>(c"ffxConfigure") }
            .map(|symbol| *symbol)
            .map_err(|source| LoadError::Symbol {
                path: path.clone(),
                symbol: "ffxConfigure",
                source,
            })?;
        // SAFETY: The caller guarantees the ABI for this exact named export.
        // Windows libloading rejects null GetProcAddress results; success is Some.
        let ffxQuery = unsafe { library.get::<PfnFfxQuery>(c"ffxQuery") }
            .map(|symbol| *symbol)
            .map_err(|source| LoadError::Symbol {
                path: path.clone(),
                symbol: "ffxQuery",
                source,
            })?;
        // SAFETY: The caller guarantees the ABI for this exact named export.
        // Windows libloading rejects null GetProcAddress results; success is Some.
        let ffxDispatch = unsafe { library.get::<PfnFfxDispatch>(c"ffxDispatch") }
            .map(|symbol| *symbol)
            .map_err(|source| LoadError::Symbol {
                path: path.clone(),
                symbol: "ffxDispatch",
                source,
            })?;
        Ok(Self {
            ffxCreateContext,
            ffxDestroyContext,
            ffxConfigure,
            ffxQuery,
            ffxDispatch,
            _library: library,
        })
    }

    /// Forward directly to the native `ffxCreateContext` export.
    ///
    /// # Safety
    ///
    /// Provide a writable context output initialized to null, valid creation descriptors and
    /// valid optional allocation callbacks. All descriptor-owned pointers and callbacks must
    /// remain valid for their native lifetimes. Keep this library alive until every created
    /// context and any pending native work have been released.
    /// Callbacks must not unwind across the FFI boundary.
    pub unsafe fn ffxCreateContext(
        &self,
        context: *mut ffxContext,
        desc: *mut ffxCreateContextDescHeader,
        memCb: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t {
        // SAFETY: load established the signature and non-null invariant; self
        // retains the DLL. The caller upholds the native operation's contract.
        unsafe { (self.ffxCreateContext.expect("resolved Windows export"))(context, desc, memCb) }
    }

    /// Forward directly to the native `ffxDestroyContext` export.
    ///
    /// # Safety
    ///
    /// Provide a valid context from this library and compatible allocation callbacks. Satisfy
    /// the SDK synchronization requirements before destruction; do not reuse a destroyed
    /// context.
    /// Callbacks must not unwind across the FFI boundary.
    pub unsafe fn ffxDestroyContext(
        &self,
        context: *mut ffxContext,
        memCb: *const ffxAllocationCallbacks,
    ) -> ffxReturnCode_t {
        // SAFETY: load established the signature and non-null invariant; self
        // retains the DLL. The caller upholds the native operation's contract.
        unsafe { (self.ffxDestroyContext.expect("resolved Windows export"))(context, memCb) }
    }

    /// Forward directly to the native `ffxConfigure` export.
    ///
    /// # Safety
    ///
    /// Provide a valid context from this library (or null for a supported global operation)
    /// and a valid configuration descriptor chain. Uphold all descriptor-specific pointer,
    /// callback lifetime and synchronization requirements.
    /// Callbacks must not unwind across the FFI boundary.
    pub unsafe fn ffxConfigure(
        &self,
        context: *mut ffxContext,
        desc: *const ffxConfigureDescHeader,
    ) -> ffxReturnCode_t {
        // SAFETY: load established the signature and non-null invariant; self
        // retains the DLL. The caller upholds the native operation's contract.
        unsafe { (self.ffxConfigure.expect("resolved Windows export"))(context, desc) }
    }

    /// Forward directly to the native `ffxQuery` export.
    ///
    /// # Safety
    ///
    /// Provide a valid context from this library (or null for a supported global query) and a
    /// valid writable query descriptor chain with valid output storage. Uphold the
    /// query-specific lifetime and synchronization requirements.
    /// Callbacks must not unwind across the FFI boundary.
    pub unsafe fn ffxQuery(
        &self,
        context: *mut ffxContext,
        desc: *mut ffxQueryDescHeader,
    ) -> ffxReturnCode_t {
        // SAFETY: load established the signature and non-null invariant; self
        // retains the DLL. The caller upholds the native operation's contract.
        unsafe { (self.ffxQuery.expect("resolved Windows export"))(context, desc) }
    }

    /// Forward directly to the native `ffxDispatch` export.
    ///
    /// # Safety
    ///
    /// Provide a valid context from this library and valid dispatch descriptors and
    /// resources. Uphold native resource state, lifetime, device, queue and synchronization
    /// requirements, including those extending beyond this call.
    /// Callbacks must not unwind across the FFI boundary.
    pub unsafe fn ffxDispatch(
        &self,
        context: *mut ffxContext,
        desc: *const ffxDispatchDescHeader,
    ) -> ffxReturnCode_t {
        // SAFETY: load established the signature and non-null invariant; self
        // retains the DLL. The caller upholds the native operation's contract.
        unsafe { (self.ffxDispatch.expect("resolved Windows export"))(context, desc) }
    }
}
