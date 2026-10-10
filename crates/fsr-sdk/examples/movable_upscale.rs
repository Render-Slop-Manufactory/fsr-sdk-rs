// SPDX-License-Identifier: MPL-2.0

//! Consumer-owned cleanup coordination for independently transferable contexts.
//! Compile with --features movable. main is guidance, not a standalone renderer.

#[cfg(all(windows, feature = "dx12", feature = "movable"))]
pub mod renderer_integration {
    //! One application domain, including automatic cleanup and poisoned-lock recovery.
    //! All other native users of shared state must join this domain or be isolated.
    use fsr_sdk::{
        error::Error,
        runtime::MovableRuntime,
        upscaler::{ID3D12Device, MovableUpscaler, UpscaleDispatch, UpscalerOptions},
    };
    use std::{
        path::Path,
        sync::{Arc, Mutex, MutexGuard},
    };

    fn enter(gate: &Mutex<()>) -> MutexGuard<'_, ()> {
        // This example never exposes the gate while native operations are in
        // progress. A Rust panic between operations may poison it; no invariant
        // is left half-mutated, and cleanup still must acquire exclusive access.
        gate.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    /// Consumer wrapper retaining its coordination domain through implicit Drop.
    ///
    /// It exposes no raw movable owner. A failed channel send returns this wrapper;
    /// dropping that returned value is coordinated just like ordinary/unwind Drop.
    /// The renderer still owns GPU completion, including before unwinding cleanup.
    pub struct CoordinatedContext {
        gate: Arc<Mutex<()>>,
        context: Option<MovableUpscaler>,
    }

    impl CoordinatedContext {
        /// Record inside the domain; the caller manages submission and completion.
        ///
        /// # Safety
        /// Uphold every requirement of MovableUpscaler::dispatch, including GPU
        /// completion before this wrapper is dropped normally or during unwinding.
        pub unsafe fn dispatch(&mut self, frame: UpscaleDispatch<'_>) -> Result<(), Error> {
            let _guard = enter(&self.gate);
            // SAFETY: Caller establishes renderer/native preconditions; this
            // wrapper provides shared application-call coordination only.
            unsafe { self.context.as_mut().unwrap().dispatch(frame) }
        }

        /// Explicitly destroy inside the domain after finishing all GPU work.
        pub fn destroy(mut self) -> Result<(), Error> {
            let _guard = enter(&self.gate);
            self.context.take().unwrap().destroy()
        }
    }

    impl Drop for CoordinatedContext {
        fn drop(&mut self) {
            if self.context.is_some() {
                let _guard = enter(&self.gate);
                // The native owner is destroyed before the guard, including
                // unwind and SendError cleanup. Never hold this gate while
                // explicitly dropping one of these wrappers (non-recursive mutex).
                drop(self.context.take());
            }
        }
    }

    /// Create independent siblings and release the public runtime before returning.
    ///
    /// # Safety
    /// Establish the complete MovableRuntime::load_unchecked contract. The supplied
    /// gate must be the common domain for all potentially shared native users,
    /// including other acquisitions/local families and external integrations.
    /// Native arbitrary-thread compatibility remains a caller assumption.
    /// No GPU work is recorded here, so construction/unwind cleanup needs no fence.
    pub unsafe fn create_pair(
        path: &Path,
        device: &ID3D12Device,
        options: UpscalerOptions,
        gate: Arc<Mutex<()>>,
    ) -> Result<(CoordinatedContext, CoordinatedContext), Box<dyn std::error::Error>> {
        let (first, second) = {
            // Declared before native owners: errors/unwinding drop them first.
            let _guard = enter(&gate);
            // SAFETY: Caller establishes native trust, compatibility and all
            // external coordination; this example preserves its own cleanup order.
            let runtime = unsafe { MovableRuntime::load_unchecked(path)? };
            let first = MovableUpscaler::new(&runtime, device, options.clone())?;
            let second = MovableUpscaler::new(&runtime, device, options)?;
            drop(runtime); // Contexts retain the actual loader independently.
            (first, second)
        };
        Ok((
            CoordinatedContext {
                gate: gate.clone(),
                context: Some(first),
            },
            CoordinatedContext {
                gate,
                context: Some(second),
            },
        ))
    }
}

fn main() {
    println!("Use renderer_integration::create_pair with one common native coordination domain.");
}
