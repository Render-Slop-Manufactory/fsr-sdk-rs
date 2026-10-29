// SPDX-License-Identifier: MPL-2.0

use super::{
    Error, GpuMemoryUsage, ID3D12GraphicsCommandList, ProviderInfo, ReactiveMaskResources,
    ResourceRequirements, UpscaleResources, UpscalerCore,
};
use crate::runtime::RuntimeState;
use crate::{
    query::{JitterPhaseCount, QualityMode},
    upscaler::{Dimensions, FrameParameters, JitterOffsetPixels, ReactiveMaskParameters},
};
use std::{cell::Cell, marker::PhantomData, sync::Arc};

/// Optional movable DX12 context: `Send + !Sync`, neither `Copy` nor `Clone`.
///
/// Construction requires the matching [`crate::runtime::MovableRuntime::upscaler_dx12`]; no conversion between
/// local and movable families is provided. Pinned descriptors, an independent
/// device reference and an independent runtime reference survive outer moves.
/// The last context may release the loader on any thread, even after the public
/// runtime has been dropped. Neither this type nor `Arc` serializes native calls.
///
/// All use and cleanup, including automatic `Drop` during unwinding, must remain
/// inside the caller's common coordination domain under
/// [`crate::runtime::MovableRuntime::upscaler_dx12`]. A per-context mutex is insufficient when
/// siblings or other FidelityFX users share native state. Passing this value to
/// safe code does not discharge that lasting unsafe obligation.
///
/// Like [`super::Upscaler`], native dispatch failure poisons dispatch and live queries.
/// Explicit destruction and `Drop` attempt teardown at most once; failure retains
/// the full critical bundle. `Drop` discards the error and neither cleanup path
/// waits for GPU work. See [`Self::record`] for renderer lifetime obligations.
pub struct MovableUpscaler {
    pub(super) core: UpscalerCore<Arc<RuntimeState>>,
    _not_sync: PhantomData<Cell<()>>,
}

// SAFETY: This implementation applies only to this exact field graph, not the
// generic core or NativeContextOwner. The core owns one non-duplicated native
// handle, Pin<Box<UpscaleCreateState>> (root/version/backend, optional callback
// extension/host table/override), an independent ID3D12Device reference and Arc<RuntimeState>.
// Explicit unsafe callback installation requires external targets and allocator
// state to support every destination thread and cleanup, including retained
// failed-create bundles. These targets are not owned by the wrapper.
// Pin preserves all descriptor addresses across outer moves. RuntimeState owns
// only FfxLibrary's private ABI function pointers and libloading::Library; Arc
// makes reference counting transferable, with no Rc-owned dependency here.
// UpscalerOptions contains checked NonZeroU32 dimensions and pointer-free
// depth, motion and exposure enums; resource requirements contain only u64
// masks, and the pointer-free context eligibility enum moves
// with the owner. Cell's phantom marker is Send and explicitly excludes Sync.
// Dispatch borrows lists/resources only during its CPU call and preserves the
// shared poison policy; the caller retains all GPU objects through completion.
// Reactive-mask generation adds only temporary borrows/options/descriptors,
// no owned fields or callbacks. Provider non-retention and state restoration
// trust applies on every use thread; both recording routes precede idle Configure.
// Terminal ownership is removed before native destroy; success releases pinned
// state, device, then runtime, while destroy failure/OK-null creation retain the
// entire bundle without retry. Unsafe loader and movable setup acquisition establish
// arbitrary-thread native compatibility and lasting common coordination for
// every call, COM release, explicit/implicit Drop (including unwind) and final
// library release. Queries add no retained pointers; their output lifetime and
// GPU compatibility are part of that acquisition trust. Configure retains no
// descriptor/value pointers; its provider trust applies on every use thread,
// with GPU-idleness established by prior unsafe dispatch obligations. No incidental native
// pointer or the generic owner's local
// PhantomData<Rc<()>> marker is dereferenced/mutated concurrently by safe Rust.
unsafe impl Send for MovableUpscaler {}

impl MovableUpscaler {
    /// Pure known-usability snapshot; matches [`super::Upscaler::usability`].
    /// No native call, COM inspection, reference-count change or wait occurs.
    /// This neither establishes GPU readiness nor relaxes lasting coordination.
    #[must_use = "usability only returns a snapshot; it does not wait or change the context"]
    pub fn usability(&self) -> crate::upscaler::ContextUsability {
        self.core.eligibility.usability()
    }

    /// Locally mark abandonment; matches [`super::Upscaler::mark_recording_abandoned`].
    /// Preserves the first cause and blocks later live use without native calls,
    /// COM inspection, reference-count changes, releases or waits. Mark every
    /// affected context; invalidate recordings and complete prior submitted work
    /// before terminal cleanup in the common native coordination domain.
    pub fn mark_recording_abandoned(&mut self) {
        self.core.eligibility.mark_recording_abandoned();
    }

    pub(super) fn from_core(core: UpscalerCore<Arc<RuntimeState>>) -> Self {
        Self {
            core,
            _not_sync: PhantomData,
        }
    }

    /// Send the deprecated descriptor through this live context. Matches
    /// [`super::Upscaler::configure_global_debug_legacy`], including poison,
    /// exact native error preservation and no fallback.
    ///
    /// # Safety
    /// Uphold that method's complete contract and [`super::interop`] on every
    /// use thread. All affected contexts must be CPU-idle, GPU-complete and have
    /// no executable pending recordings; serialize their common native domain.
    /// Descriptor consumption must finish on success and error; targets survive
    /// all possible later native uses, independently of this owner's lifetime.
    pub unsafe fn configure_global_debug_legacy(
        &mut self,
        level: crate::upscaler::DebugLevel,
        callback: Option<super::interop::MessageCallback>,
    ) -> Result<(), Error> {
        // SAFETY: Caller establishes lasting installation and coordination rules.
        unsafe { self.core.configure_global_debug_legacy(level, callback) }
    }

    /// Inspect generator metadata without recording; matches
    /// [`super::Upscaler::validate_reactive_mask_inputs`]. Existing poison
    /// rejects before inspection, no FidelityFX entry point is called and no
    /// independent COM references remain. Success certifies neither provider
    /// support nor GPU safety. Common coordination applies on this use thread.
    pub fn validate_reactive_mask_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        self.core
            .validate_reactive_mask_inputs(command_list, resources, parameters)
    }

    /// Record standalone reactive-mask generation; identical mapping and terminal
    /// error policy to [`super::Upscaler::record_reactive_mask`].
    ///
    /// # Input conventions
    ///
    /// Match the local method's colour, frame, projection/jitter, exposure,
    /// encoding and consumer-region conventions. These describe signal meaning;
    /// actual-provider numerical and access support remains required below.
    ///
    /// # Safety
    ///
    /// Uphold every obligation of [`super::Upscaler::record_reactive_mask`],
    /// including provider domain/support, live DIRECT list,
    /// shader-read inputs/UAV output, non-overlapping output storage, residency,
    /// retained GPU dependencies, ordered submission and consumer barriers.
    /// Both recording routes must finish with no executable pending recordings
    /// before safe Configure; complete submitted work and invalidate failed or
    /// abandoned recordings before terminal teardown. Never submit a failed list.
    /// Handoff establishes neither GPU completion nor resource-state transitions.
    /// The movable acquisition's arbitrary-thread native compatibility and common
    /// non-reentrant coordination apply through calls, COM release and cleanup
    /// (including unwinding). No automatic probe, fallback, submission or wait occurs.
    /// Native non-retention/restoration and unresolved GPU-only consequences
    /// retain the local method's safety classification on every destination thread.
    pub unsafe fn record_reactive_mask(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        // SAFETY: Caller accepts the local generator contract plus movable
        // destination-thread compatibility and common coordination obligations.
        unsafe {
            self.core
                .record_reactive_mask(command_list, resources, parameters)
        }
    }

    /// Check frame metadata without FSR recording; matches
    /// [`super::Upscaler::validate_upscale_inputs`].
    ///
    /// No FidelityFX entry point is called and eligibility is unchanged. Existing
    /// poison rejects first. DX12 inspection and COM release stay in the movable
    /// acquisition's common coordination domain on this thread. Success proves
    /// neither provider support nor GPU state/lifetime/completion; dispatch
    /// always repeats inspection. Moving this owner does not validate GPU work.
    pub fn validate_upscale_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        self.core
            .validate_upscale_inputs(command_list, resources, parameters)
    }

    /// Request one tuning change; matches [`super::Upscaler::request_tuning`].
    ///
    /// Success is acknowledgement only. [`super::TuningRequest::new`] checks
    /// parameter domains before native mutation; accepted values are forwarded unchanged.
    /// `ProviderDefault` requests the actual provider default. Acquisition's
    /// synchronous non-retention, parameter-domain/reset safety, coordination and terminal
    /// destroyability trust applies on this use thread. Prior unsafe dispatches
    /// require completed GPU work and no pending executable recording before
    /// Configure; moving this owner does not establish completion. No wait occurs.
    /// Request construction is pure and independent of context eligibility. Native errors preserve
    /// `Operation::Tuning`/code and poison dispatch, live queries and Configure.
    /// Either existing poison rejects before descriptor construction or native entry. Destroy
    /// remains available. No automatic reset, support detection or rollback occurs.
    pub fn request_tuning(&mut self, request: super::TuningRequest) -> Result<(), Error> {
        self.core.request_tuning(request)
    }

    /// Matches [`super::Upscaler::upscale_ratio`] for this context's provider.
    /// The movable acquisition's utility-query trust and common native coordination
    /// apply. Input, native-result and poison errors match the local family.
    pub fn upscale_ratio(&mut self, mode: QualityMode) -> Result<f32, Error> {
        self.core.upscale_ratio(mode)
    }

    /// Matches [`super::Upscaler::render_size_for_quality`] for this context's provider.
    /// The movable acquisition's utility-query trust and common native coordination
    /// apply. Input, native-result and poison errors match the local family.
    pub fn render_size_for_quality(
        &mut self,
        display: Dimensions,
        mode: QualityMode,
    ) -> Result<Dimensions, Error> {
        self.core.render_size_for_quality(display, mode)
    }

    /// Matches [`super::Upscaler::jitter_phase_count`] for this context's provider.
    /// The movable acquisition's utility-query trust and common native coordination
    /// apply. Input, native-result and poison errors match the local family.
    pub fn jitter_phase_count(
        &mut self,
        render: Dimensions,
        display: Dimensions,
    ) -> Result<JitterPhaseCount, Error> {
        self.core.jitter_phase_count(render, display)
    }

    /// Matches [`super::Upscaler::jitter_offset`] for this context's provider.
    /// The movable acquisition's utility-query trust and common native coordination
    /// apply. Input, native-result and poison errors match the local family.
    pub fn jitter_offset(
        &mut self,
        index: i32,
        phase_count: JitterPhaseCount,
    ) -> Result<JitterOffsetPixels, Error> {
        self.core.jitter_offset(index, phase_count)
    }

    /// Owned provider metadata; matches [`super::Upscaler::provider`].
    ///
    /// Keep the common domain held through copying. Query trust applies on this
    /// thread. Poison rejects before FFI; query errors never alter dispatch poison.
    pub fn provider(&mut self) -> Result<ProviderInfo, Error> {
        self.core.provider()
    }

    /// Resource masks; matches [`super::Upscaler::resource_requirements`].
    ///
    /// Unknown bits survive. Query/GPU trust and common-domain coordination
    /// apply. Poison rejects before FFI; query errors never alter dispatch poison.
    pub fn resource_requirements(&mut self) -> Result<ResourceRequirements, Error> {
        self.core.resource_requirements()
    }

    /// Memory snapshot; matches [`super::Upscaler::gpu_memory_usage`].
    ///
    /// The query/GPU contract and common domain apply. This is no GPU wait or
    /// reservation. Poison rejects before FFI; query errors never alter poison.
    pub fn gpu_memory_usage(&mut self) -> Result<GpuMemoryUsage, Error> {
        self.core.gpu_memory_usage()
    }

    /// Record one dispatch with the context's profile and fixed allocation extents.
    /// Dynamic contexts may vary active render/output sizes; no submission or GPU wait is performed.
    ///
    /// Validation, metadata/device inspection and poison behaviour share the local
    /// [`super::Upscaler::record`] implementation. Pre-native validation/inspection
    /// failures leave the context usable; a returned native error preserves its
    /// code and poisons later dispatch, rejected before inspection or FFI.
    /// `Ok(())` establishes only successful native recording, not GPU completion,
    /// correct output, native thread compatibility or an allowed queue depth.
    ///
    /// # Input conventions
    ///
    /// Uphold [`super::Upscaler::record`]'s complete input conventions,
    /// including colour/exposure, mask alignment, depth/projection, motion units
    /// and the encoding of any jitter contribution. Restore application
    /// descriptor heaps/bindings as needed. Moving the owner changes none of
    /// those signal meanings or the actual-provider domain requirements below.
    ///
    /// # Safety
    ///
    /// Uphold [`crate::runtime::MovableRuntime::upscaler_dx12`]'s lasting common-domain
    /// serialization, non-reentrancy, native compatibility and cleanup contract.
    /// Independently uphold the full safety contract of [`super::Upscaler::record`]:
    /// this includes actual-provider format/view and literal motion-scale support,
    /// the encoding of any included jitter contribution, and safe access for active output,
    /// whole-output write permission, valid input sampling footprints and declared
    /// boundary-state restoration. Success does not certify requested geometry.
    ///
    /// - Supply an open recording DIRECT list on the context's device. Retain the
    ///   list and its allocator through submission/completion as D3D12 requires.
    /// - Colour/depth/motion, supplied masks and any supplied exposure must be in non-pixel-shader-resource state;
    ///   output must be in unordered-access state. Keep truthful states, residency
    ///   and non-overlapping active backing storage; avoid conflicting access.
    /// - Keep all supplied textures, backing allocations and this context alive until
    ///   submitted GPU work completes. Rust borrows end after the CPU call.
    /// - Submit each successful recording once, with successive temporal dispatches
    ///   executing in recording order and required cross-queue waits. Avoid
    ///   premature resource reuse, allocator reset or list resubmission. Neither
    ///   CPU handoff nor a shared submission batch establishes GPU ordering.
    /// - Before destruction or `Drop`, including unwind, finish all submitted work
    ///   and invalidate all unsubmitted recordings referencing this context.
    ///   Never submit a list containing failed native recording. After a native
    ///   error, no resource post-state is promised; finish earlier work then tear
    ///   down. Abandoning an OK recording likewise requires stopping dispatch and
    ///   terminal cleanup.
    ///
    /// D011's bounded signed-runtime transient-allocation/teardown trust remains;
    /// no provider concurrency, universal restoration or outstanding-work transfer
    /// guarantee is added. Native transfer preconditions must hold independently.
    /// Before subsequent safe `request_tuning`, complete all submitted upscaling
    /// and reactive-mask generation work on
    /// this context and leave no executable unsubmitted recording for submission
    /// after Configure. Record, submit, wait, configure, then record again. This
    /// is a lasting obligation of unsafe dispatch, including across handoff;
    /// Configure does not wait. Abandoned OK recordings remain terminal and must
    /// be invalidated before destruction, rather than configuring and resuming.
    ///
    /// ## Native assumptions and uncertainty
    ///
    /// The local dispatch's actual-provider pointer-lifetime, numerical/access,
    /// callback and transient-allocation assumptions remain required on this
    /// thread. Its unresolved GPU-only safety consequences are not proof of
    /// harmlessness or of Rust UB for every violation. Arbitrary-thread native
    /// compatibility is likewise a lasting acquisition assumption, not an AMD
    /// guarantee supplied by `Send`, handoff or a successful fenced run.
    pub unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &UpscaleResources<'_>,
        parameters: &FrameParameters,
    ) -> Result<(), Error> {
        // SAFETY: Caller establishes both lasting native and per-dispatch GPU rules.
        unsafe { self.core.record(command_list, resources, parameters) }
    }

    /// Consume this context and attempt native destruction once, on this thread.
    ///
    /// The acquisition's coordination must cover the complete cleanup, including
    /// device/runtime release. Complete submitted GPU work and invalidate failed
    /// or abandoned recordings first; this method does not wait. Destruction is
    /// available after poison. Failure preserves the native code in [`Error::Native`]
    /// and retains the full critical bundle indefinitely, without a `Drop` retry.
    /// These are lasting acquisition/recording obligations; this safe method
    /// adds no independent unchecked safety precondition.
    pub fn destroy(self) -> Result<(), Error> {
        self.core.owner.destroy()
    }
}
