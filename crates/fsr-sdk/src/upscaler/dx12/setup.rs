// SPDX-License-Identifier: MPL-2.0

//! Effect-specific acquisition, planning and creation, with concrete owner families.
#[cfg(feature = "movable")]
use super::MovableUpscaler;
use super::{ID3D12Device, Upscaler, UpscalerCore, diagnostics, interop};
#[cfg(feature = "movable")]
use crate::runtime::MovableRuntime;
use crate::{
    error::Error,
    query::{
        GpuMemoryUsage, JitterPhaseCount, ProviderInfo, ProviderSelection, QualityMode,
        ResourceRequirements,
    },
    runtime::{Runtime, queries},
    upscaler::{DebugLevel, Dimensions, JitterOffsetPixels, UpscalerOptions},
};

// Generate only concrete forwarders and their full contracts. The owning core
// and its audited local/movable field graphs remain separate from these views.
macro_rules! setup_family {
    ($runtime:ident, $setup:ident, $owner:ident) => {
        /// Borrowed DX12 upscaler integration scope with one provider request.
        ///
        /// Acquiring the scope establishes lasting effect assumptions for its
        /// later safe operations and independently owned contexts. Dropping it
        /// ends CPU borrows only; it does not revoke those assumptions, unregister
        /// callbacks, unload a retained runtime or establish GPU completion.
        /// Local scopes are `!Send + !Sync`; movable scopes are `Send + !Sync`
        /// under the matching family's arbitrary-destination acquisition duties.
        /// An exclusive runtime borrow coordinates neither sibling contexts nor
        /// separate acquisitions, hidden native workers or external native users.
        pub struct $setup<'runtime, 'device> {
            runtime: &'runtime mut $runtime,
            device: &'device ID3D12Device,
            selection: ProviderSelection,
        }

        impl $runtime {
            /// Acquire a DX12 upscaler integration scope without a native call.
            ///
            /// The request initially uses the SDK default. Safe rebinding changes
            /// only this scope; it does not change an existing context or reserve
            /// a provider. Created contexts independently retain their runtime,
            /// device and pinned storage and do not borrow the scope.
            ///
            /// # Safety
            ///
            /// ## Lasting provider processing and ownership
            ///
            /// Establish safe processing or ordinary native rejection for every
            /// provider and checked input domain reachable through this scope's
            /// later safe operations, including default changes and explicit
            /// rebinding. Trust restricted to the initial default or one preferred
            /// ID is insufficient. Preserve compatible process/provider state
            /// through all derived contexts and exceptional retention. Fresh
            /// enumeration/live identity is no compatibility certificate.
            ///
            /// Creation and estimates must safely process nonzero `u32` maxima
            /// and all exposed depth/motion/exposure/dynamic/HDR/non-linear/debug
            /// flags. Retained owned device, pinned descriptor chain and optional
            /// host table must suffice for their native lifetime. On ordinary
            /// callback-free create error, native code must cease accesses to
            /// caller dependencies before return so RAII can release them. This
            /// is conditional native trust, not a universal rollback guarantee.
            /// Successful destroy must end access to wrapper-owned storage;
            /// OK/null create and failed destroy instead retain the whole bundle.
            ///
            /// ## Queries and temporary storage
            ///
            /// Providers must synchronously consume descriptor/backend/override
            /// chains, borrowed devices and initialized outputs without retaining
            /// their CPU addresses. Respect count/fill capacities and return
            /// readable NUL-terminated names valid through immediate copying.
            /// Keep shared coordination through count/fill/copy and each selected
            /// operation. Success must populate documented outputs; explicit
            /// selection must not silently substitute a default.
            ///
            /// Establish numerical safety for the five quality modes, render-size
            /// query display components up to `i32::MAX`, jitter render/display
            /// widths up to `i32::MAX` with display/render in `[1, 16383]`, positive
            /// `i32` phase counts and indices in `0..phase_count`. Live queries
            /// must be safe in every GPU state permitted by recording, including
            /// work in flight; ordinary query errors must preserve liveness.
            ///
            /// ## Configure, recording and shared coordination
            ///
            /// Safe tuning must safely process all five parameter-specific finite
            /// domains and null-default requests or return ordinary errors. Its
            /// temporary descriptor/float storage is synchronously consumed even
            /// on error, with no deferred GPU work or unowned callback lifetime.
            /// Preserve allocations, resource requirements and creation profile
            /// without incompatible sibling effects. A failed context must remain
            /// terminally destroyable. Earlier unsafe recording must establish
            /// GPU completion and no executable pending recording before safe
            /// Configure; neither acquisition nor Configure waits.
            ///
            /// When recording upscaling or reactive-mask generation, also uphold
            /// that unsafe operation's provider-domain, access, resource/state,
            /// boundary-restoration, submission and completion obligations.
            /// Those duties persist through later safe calls and cleanup. Failed
            /// or abandoned recordings require invalidation and prior-work
            /// completion before terminal destruction; no rollback is assumed.
            ///
            /// Uphold loading's lasting common-domain serialization and
            /// non-reentrancy through all calls, error cleanup, Drop/unwinding
            /// and retained dependencies, including siblings, other acquisitions,
            /// hidden workers and external users. A different path/device or
            /// `&mut` borrow does not prove isolation or GPU idleness. Prior
            /// callback installers retain their external targets/allocator and
            /// coordination obligations for all affected users.
            ///
            /// For the movable family, every effect operation, COM/device release,
            /// error path and implicit/explicit cleanup must be compatible with
            /// any subsequent destination thread. One selected handoff or a lock
            /// does not establish absence of native affinity. Loader release
            /// compatibility must already have been established at loading.
            /// Local owners instead stay on their acquiring thread.
            ///
            /// ## Native assumptions and uncertainty
            ///
            /// ABI validity, bounded writes, readable names and non-retention
            /// protect CPU memory/ownership. Numerical/GPU access, hidden activity,
            /// continuation and termination remain conditional actual-provider
            /// assumptions, not universal AMD guarantees or proof that every
            /// violation causes Rust UB. Keep them established after this view
            /// and the public runtime are dropped while derived/retained native
            /// users remain. Do not acquire an incompatible configuration.
            pub unsafe fn upscaler_dx12<'r, 'd>(
                &'r mut self,
                device: &'d ID3D12Device,
            ) -> $setup<'r, 'd> {
                $setup {
                    runtime: self,
                    device,
                    selection: ProviderSelection::Default,
                }
            }
        }

        impl $setup<'_, '_> {
            /// Requested provider, not an effective provider or reservation.
            pub fn selection(&self) -> ProviderSelection {
                self.selection
            }

            /// Replace this scope's request without a native call or live mutation.
            /// Acquisition's trust covers every safely reachable selection.
            #[must_use = "with_provider returns the setup with the new request"]
            pub fn with_provider(mut self, request: ProviderSelection) -> Self {
                self.selection = request;
                self
            }

            /// Enumerate a bounded unselected device/effect snapshot with owned names.
            /// Growth can truncate; this observation reserves no provider.
            pub fn providers(&mut self) -> Result<Vec<ProviderInfo>, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe { queries::providers(&self.runtime.state, self.device) }
            }
            /// Ask the requested provider for a quality-mode ratio.
            /// Every explicit request checks fresh membership; no fallback or cache.
            pub fn ratio(&mut self, mode: QualityMode) -> Result<f32, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::ratio(
                        &self.runtime.state,
                        self.device,
                        self.selection.optional(),
                        mode,
                    )
                }
            }
            /// Query quality-derived dimensions, not a context's active region.
            /// Pure numerical checks precede fresh explicit-ID enumeration.
            pub fn render_size_for_quality(
                &mut self,
                output: Dimensions,
                mode: QualityMode,
            ) -> Result<Dimensions, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::render_size(
                        &self.runtime.state,
                        self.device,
                        self.selection.optional(),
                        output,
                        mode,
                    )
                }
            }
            /// Query a positive sequence length for the caller's actual extents.
            /// Numerical checks precede fresh explicit-ID enumeration.
            pub fn jitter_phase_count(
                &mut self,
                render: Dimensions,
                output: Dimensions,
            ) -> Result<JitterPhaseCount, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::jitter_phase_count(
                        &self.runtime.state,
                        self.device,
                        self.selection.optional(),
                        render,
                        output,
                    )
                }
            }
            /// Query an indexed pixel offset; the wrapper generates no sequence.
            /// Index validation precedes fresh explicit-ID enumeration.
            pub fn jitter_offset(
                &mut self,
                index: i32,
                phases: JitterPhaseCount,
            ) -> Result<JitterOffsetPixels, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::jitter_offset(
                        &self.runtime.state,
                        self.device,
                        self.selection.optional(),
                        index,
                        phases,
                    )
                }
            }
            /// Report required/optional masks without certifying profile/capability.
            /// Unknown bits survive; no creation filter is applied to this query.
            pub fn resource_requirements(&mut self) -> Result<ResourceRequirements, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::resource_requirements(
                        &self.runtime.state,
                        self.device,
                        self.selection.optional(),
                    )
                }
            }
            /// Estimate using this request and the same profile used for creation.
            /// Results reserve no memory and do not validate provider compatibility.
            pub fn estimate_memory(
                &mut self,
                options: &UpscalerOptions,
            ) -> Result<GpuMemoryUsage, Error> {
                // SAFETY: Unsafe setup acquisition establishes this operation's
                // lasting provider/query/non-retention/coordination contract.
                unsafe {
                    queries::estimate_memory(
                        &self.runtime.state,
                        self.device,
                        options,
                        self.selection.optional(),
                    )
                }
            }

            /// Create an independent context from a snapshot of this profile/request.
            ///
            /// The owner retains its runtime, cloned device and pinned chain, so
            /// the scope, public runtime, options and original device can be
            /// dropped. Explicit selection checks fresh membership and effective
            /// live identity. Every context queries resource requirements and
            /// rejects unknown required inputs before being returned. No GPU work
            /// or implicit Configure is performed. Native numerical faults need
            /// not become Rust errors; acquisition's provider assumptions apply.
            ///
            /// # Errors
            /// Native codes retain Create/Query attribution; mismatched identity,
            /// unknown required inputs and successful null handles have distinct
            /// wrapper errors. Post-create rejection uses terminal cleanup.
            pub fn create(&mut self, options: &UpscalerOptions) -> Result<$owner, Error> {
                // SAFETY: Unsafe acquisition of this view establishes all lasting
                // effect, provider, numerical, coordination and cleanup duties.
                let core = unsafe {
                    UpscalerCore::new(
                        self.runtime.state.clone(),
                        self.device,
                        options.clone(),
                        self.selection.optional(),
                    )
                }?;
                Ok($owner::from_core(core))
            }

            /// Create with native callbacks; an empty bundle follows `create`.
            /// Omitted DX12 groups do not reset earlier registrations. Callback-
            /// bearing native create error retains chain/table/device/runtime
            /// without touching the failed handle. Post-create query failure
            /// instead uses ordinary one-attempt teardown.
            ///
            /// # Safety
            /// Uphold the full [`interop`] contract for all possible affected
            /// users, including external target lifetime, allocator compatibility,
            /// no unwind/reentry, global replacement, hidden activity and all
            /// success/error/cleanup paths. External targets are independently
            /// owned and can remain reachable after this context is destroyed.
            /// Native successful destroy ends access to wrapper-owned storage,
            /// not necessarily copied global callback pointers. All obligations
            /// apply on every destination thread for the movable family.
            pub unsafe fn create_with_callbacks(
                &mut self,
                options: &UpscalerOptions,
                callbacks: interop::CreationCallbacks,
            ) -> Result<$owner, Error> {
                // SAFETY: Setup trust covers the effect; this call's installer
                // establishes lasting target/allocator/global-state obligations.
                let core = unsafe {
                    UpscalerCore::new_with_callbacks(
                        self.runtime.state.clone(),
                        self.device,
                        options.clone(),
                        self.selection.optional(),
                        callbacks,
                    )
                }?;
                Ok($owner::from_core(core))
            }

            /// Send modern global upscaler diagnostics, without selection/fallback.
            /// Success acknowledges the request, not message delivery. Errors
            /// preserve Configure/code without poisoning an owned context.
            ///
            /// # Safety
            /// Uphold [`interop`]'s lasting callback contract for every affected
            /// user, including None/error. All affected contexts must be CPU-idle,
            /// GPU-complete and have no executable pending recordings. Serialize
            /// the common non-reentrant native domain, including hidden workers;
            /// establish synchronous descriptor consumption and safe continuation
            /// after error. Targets may remain globally reachable. Movable use
            /// requires compatibility on every possible destination thread.
            pub unsafe fn configure_debug(
                &mut self,
                level: DebugLevel,
                callback: Option<interop::MessageCallback>,
            ) -> Result<(), Error> {
                // SAFETY: Caller supplies global installation/coordination duties.
                unsafe { diagnostics::runtime(&self.runtime.state, level, callback, false) }
            }

            /// Send deprecated global diagnostics with a null native context.
            /// No modern fallback, provider selector or per-context poison is used.
            ///
            /// # Safety
            /// Uphold [`interop`]'s lasting external-target, allocator, no-unwind/
            /// reentry and global-registration contract for all affected users,
            /// even on None/error. All affected contexts must be CPU-idle,
            /// GPU-complete and free of executable pending recordings. Serialize
            /// their common native domain including hidden activity; establish
            /// synchronous descriptor consumption and safe continuation after
            /// error. Targets can remain reachable after teardown. Movable use
            /// covers every destination thread, not only this acquisition thread.
            pub unsafe fn configure_global_debug_legacy(
                &mut self,
                level: DebugLevel,
                callback: Option<interop::MessageCallback>,
            ) -> Result<(), Error> {
                // SAFETY: Caller supplies the distinct legacy/global contract.
                unsafe { diagnostics::runtime(&self.runtime.state, level, callback, true) }
            }
        }
    };
}

setup_family!(Runtime, UpscalerSetup, Upscaler);
#[cfg(feature = "movable")]
setup_family!(MovableRuntime, MovableUpscalerSetup, MovableUpscaler);
