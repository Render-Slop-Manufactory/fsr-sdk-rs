// SPDX-License-Identifier: MPL-2.0

use super::{LoadError, RuntimeState};
use std::{cell::Cell, marker::PhantomData, path::Path, sync::Arc};

/// Loaded library with Send + !Sync ownership, without safe effect operations.
///
/// Not Clone and not a synchronization service. Unsafe loading establishes
/// arbitrary-destination loader/release compatibility; unsafe effect setup adds
/// its lasting native-operation duties. Derived owners retain the library after
/// this public value is dropped, including exceptional retention paths.
pub struct MovableRuntime {
    pub(crate) state: Arc<RuntimeState>,
    _not_sync: PhantomData<Cell<()>>,
}
impl MovableRuntime {
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
    /// Before this Send runtime is made available to safe code, establish
    /// loader/dependency use and final release compatibility on every possible
    /// destination thread, including Drop/unwinding/error paths. Compatibility
    /// with only one handoff is insufficient; serialization does not remove
    /// native affinity. A later effect setup cannot establish this retroactively.
    /// The movable runtime is Send but never Sync or Clone.
    ///
    /// Effect-specific query, construction, Configure and recording trust is
    /// established by unsafe setup acquisition, not this load. These common
    /// duties persist after the public runtime drops while dependent or retained
    /// objects remain; no wrapper lock or GPU wait establishes them for you.
    /// ABI/lifetime/threading assumptions are caller-established native trust,
    /// not a universal AMD initialization, worker-shutdown or unload guarantee.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        // SAFETY: Caller establishes initialization/failure/final-release trust
        // and arbitrary-destination compatibility before this Send value exists.
        let state = unsafe { RuntimeState::load_unchecked(path) }?;
        Ok(Self {
            state: Arc::new(state),
            _not_sync: PhantomData,
        })
    }
}
