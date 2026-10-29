// SPDX-License-Identifier: MPL-2.0

//! DX12 upscaler ownership, resource validation and unsafe command recording.

use crate::{
    context::{NativeContextOwner, RuntimeRetention},
    context_policy::Eligibility,
    error::Error,
    query::{
        GpuMemoryUsage, JitterPhaseCount, ProviderId, ProviderInfo, QualityMode,
        ResourceRequirements,
    },
    runtime::RuntimeState,
};
use fsr_sdk_sys::{
    api::*,
    dx12::{
        FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, ffxCreateBackendDX12AllocationCallbacksDesc,
        ffxCreateBackendDX12Desc,
    },
    upscale::*,
};
use std::{marker::PhantomPinned, pin::Pin, ptr::null_mut, rc::Rc};
use windows::core::Interface;

#[cfg(test)]
use super::common::{CameraParameters, FrameTimeMillis};
use super::common::{DebugLevel, Dimensions, JitterOffsetPixels, TuningRequest, UpscalerOptions};
#[cfg(test)]
use super::common::{TuningParameter, TuningValue};
pub(crate) mod diagnostics;
pub mod interop;
mod setup;
#[cfg(test)]
use crate::runtime::Runtime;
pub use dispatch::{
    ExposureInput, NativeSharpness, ReactiveMaskResources, ReactiveMaskResourcesBuilder,
    Sharpening, UpscaleResources, UpscaleResourcesBuilder,
};
#[cfg(feature = "movable")]
pub use setup::MovableUpscalerSetup;
pub use setup::UpscalerSetup;
/// The exact `windows` 0.62 barrier types and constants for renderer-owned mask transitions.
pub use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
    D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
    D3D12_RESOURCE_BARRIER_TYPE_UAV, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
    D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_TRANSITION_BARRIER,
    D3D12_RESOURCE_UAV_BARRIER,
};
/// The exact `windows` 0.62 DX12 interface types accepted by this API.
pub use windows::Win32::Graphics::Direct3D12::{
    ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource,
};

struct UpscaleCreateState {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
    allocations: Option<ffxCreateBackendDX12AllocationCallbacksDesc>,
    host: Option<ffxAllocationCallbacks>,
    override_version: Option<ffxOverrideVersion>,
    _pinned: PhantomPinned,
}

type Owner<R> = NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device, R>;

// Sealed Rc/Arc retention; this core deliberately remains !Send + !Sync.
struct UpscalerCore<R: RuntimeRetention> {
    owner: Owner<R>,
    options: UpscalerOptions,
    // Internal snapshot for the immutable context profile. Future Configure
    // operations affecting resource consumption must refresh this snapshot.
    resource_requirements: ResourceRequirements,
    eligibility: Eligibility,
}

/// Owned DX12 upscaler context. Neither `Send`, `Sync`, `Copy` nor `Clone`.
///
/// The runtime, an independent COM device reference, and pinned creation
/// descriptors remain owned through teardown. Submitted dispatches must finish
/// on the GPU before this value is destroyed or dropped. A failed recording
/// must never be submitted; see [`Self::record`] for the full contract.
///
/// Rust validation and COM inspection failures leave the context usable. A
/// non-OK native dispatch or Configure poisons dispatch, live queries and Configure, without
/// removing the native owner or disabling terminal destruction. An abandoned successful recording
/// likewise requires the caller to stop using this context and invalidate the
/// recording before teardown. Call [`Self::mark_recording_abandoned`] for each
/// affected context; the wrapper cannot detect abandonment automatically.
///
/// [`Self::destroy`] reports the single native teardown attempt. `Drop` attempts
/// the same operation but discards its error. Neither waits for the GPU. A native
/// destroy failure retains dependencies rather than retrying or unloading them.
pub struct Upscaler {
    core: UpscalerCore<Rc<RuntimeState>>,
}

#[cfg(feature = "movable")]
mod movable;
#[cfg(feature = "movable")]
pub use movable::MovableUpscaler;

impl UpscaleCreateState {
    fn new(
        device: *mut fsr_sdk_sys::dx12::ID3D12Device,
        options: &UpscalerOptions,
        callbacks: interop::CreationCallbacks,
        provider: Option<ProviderId>,
    ) -> (Pin<Box<Self>>, *mut ffxApiHeader) {
        let header = |tag| ffxApiHeader {
            r#type: tag,
            pNext: null_mut(),
        };
        let dimensions = |size: Dimensions| FfxApiDimensions2D {
            width: size.width(),
            height: size.height(),
        };
        let mut state = Box::pin(Self {
            root: ffxCreateContextDescUpscale {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE),
                flags: options.native_flags(),
                maxRenderSize: dimensions(options.max_render_size()),
                maxUpscaleSize: dimensions(options.max_upscale_size()),
                fpMessage: callbacks.message,
            },
            version: ffxCreateContextDescUpscaleVersion {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION),
                version: FFX_UPSCALER_VERSION,
            },
            backend: ffxCreateBackendDX12Desc {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12),
                device,
            },
            override_version: crate::query::native::provider_override(provider),
            allocations: callbacks.dx12_native(),
            host: callbacks.host_native(),
            _pinned: PhantomPinned,
        });
        // SAFETY: Final allocation precedes pointer formation. This private type
        // never exposes mutable references, extracts/replaces pointees or relinks
        // after this block. No Rust reference is kept across native calls.
        let root = unsafe {
            let state = state.as_mut().get_unchecked_mut();
            state.root.header.pNext = &raw mut state.version.header;
            state.version.header.pNext = &raw mut state.backend.header;
            let tail = state
                .override_version
                .as_mut()
                .map_or(null_mut(), |node| &raw mut node.header);
            state.backend.header.pNext = if let Some(node) = state.allocations.as_mut() {
                node.header.pNext = tail;
                &raw mut node.header
            } else {
                tail
            };
            &raw mut state.root.header
        };
        (state, root)
    }
}

impl<R: RuntimeRetention> UpscalerCore<R> {
    unsafe fn configure_global_debug_legacy(
        &mut self,
        level: DebugLevel,
        callback: Option<interop::MessageCallback>,
    ) -> Result<(), Error> {
        self.eligibility.check()?;
        let code = diagnostics::with_descriptor(level, callback, true, |desc| {
            // SAFETY: The caller establishes the lasting callback/coordination
            // contract and synchronous descriptor consumption, including errors.
            unsafe { self.owner.configure(desc) }
        });
        self.eligibility
            .record(crate::error::Operation::ConfigureContextDebugLegacy, code)
    }

    fn request_tuning(&mut self, request: TuningRequest) -> Result<(), Error> {
        // SAFETY: Unsafe acquisition covers actual-provider parameter-domain/reset processing,
        // synchronous non-retention, coordination and destroyability on error.
        // Prior unsafe dispatch obligations establish idle GPU/recording state.
        unsafe {
            super::tuning::configure(&mut self.eligibility, request, |desc| {
                self.owner.configure(desc)
            })
        }
    }

    fn upscale_ratio(&mut self, mode: QualityMode) -> Result<f32, Error> {
        // SAFETY: Acquisition covers synchronous utility queries and numerical/GPU
        // compatibility. Owner retains dependencies; the helper checks eligibility.
        unsafe {
            crate::query::native::upscale_ratio(self.eligibility, mode, |desc| {
                self.owner.query(desc)
            })
        }
    }

    fn render_size_for_quality(
        &mut self,
        display: Dimensions,
        mode: QualityMode,
    ) -> Result<Dimensions, Error> {
        // SAFETY: Acquisition covers synchronous utility queries and numerical/GPU
        // compatibility. Owner retains dependencies; the helper checks eligibility.
        unsafe {
            crate::query::native::render_size_for_quality(self.eligibility, display, mode, |desc| {
                self.owner.query(desc)
            })
        }
    }

    fn jitter_phase_count(
        &mut self,
        render: Dimensions,
        display: Dimensions,
    ) -> Result<JitterPhaseCount, Error> {
        // SAFETY: Acquisition covers synchronous utility queries and numerical/GPU
        // compatibility. Owner retains dependencies; the helper checks eligibility.
        unsafe {
            crate::query::native::jitter_phase_count(self.eligibility, render, display, |desc| {
                self.owner.query(desc)
            })
        }
    }

    fn jitter_offset(
        &mut self,
        index: i32,
        phase_count: JitterPhaseCount,
    ) -> Result<JitterOffsetPixels, Error> {
        // SAFETY: Acquisition covers synchronous utility queries and numerical/GPU
        // compatibility. Owner retains dependencies; the helper checks eligibility.
        unsafe {
            crate::query::native::jitter_offset(self.eligibility, index, phase_count, |desc| {
                self.owner.query(desc)
            })
        }
    }

    fn provider(&mut self) -> Result<ProviderInfo, Error> {
        // SAFETY: Acquisition covers name validity and synchronization through
        // copying; owner retains context/runtime and the helper rejects poison.
        unsafe { crate::query::native::provider(self.eligibility, |desc| self.owner.query(desc)) }
    }

    fn resource_requirements(&mut self) -> Result<ResourceRequirements, Error> {
        // SAFETY: Acquisition covers query non-retention, GPU compatibility and
        // coordination; owner retains dependencies and the helper rejects poison.
        unsafe {
            crate::query::native::resource_requirements(self.eligibility, |desc| {
                self.owner.query(desc)
            })
        }
    }

    fn gpu_memory_usage(&mut self) -> Result<GpuMemoryUsage, Error> {
        // SAFETY: Acquisition covers query non-retention, GPU compatibility and
        // coordination; initialized output and owner outlive the synchronous call.
        unsafe {
            crate::query::native::gpu_memory_usage(self.eligibility, |desc| self.owner.query(desc))
        }
    }

    /// # Safety
    /// Runtime retention alone establishes only loading trust. Acquire the
    /// matching effect setup and uphold its lasting contracts before creation.
    unsafe fn new(
        runtime: R,
        device: &ID3D12Device,
        options: UpscalerOptions,
        provider: Option<ProviderId>,
    ) -> Result<Self, Error> {
        // SAFETY: An empty bundle installs no new callback. Ordinary acquisition
        // establishes the existing native trust and coordination contract.
        unsafe { Self::new_with_callbacks(runtime, device, options, provider, Default::default()) }
    }

    unsafe fn new_with_callbacks(
        runtime: R,
        device: &ID3D12Device,
        options: UpscalerOptions,
        provider: Option<ProviderId>,
        callbacks: interop::CreationCallbacks,
    ) -> Result<Self, Error> {
        // SAFETY: This constructor requires matching setup acquisition, including
        // fresh enumeration's capacity/name/non-retention and shared-state duties.
        unsafe {
            crate::runtime::queries::require_provider(runtime.runtime_state(), device, provider)
        }?;
        // Independent COM reference retained through terminal or exceptional cleanup.
        let device = device.clone();
        let retain_create_error = !callbacks.is_empty();
        let (state, root) =
            UpscaleCreateState::new(device.as_raw().cast(), &options, callbacks, provider);
        let allocator = state
            .host
            .as_ref()
            .map_or(std::ptr::null(), std::ptr::from_ref);
        // SAFETY: The sealed retention came from the matching acquired family.
        // Its unsafe acquisition establishes native threading/coordination and
        // D008's trust boundary. The pinned chain owns all creation dependencies.
        let mut owner = if retain_create_error {
            // SAFETY: The complete chain/table are pinned before pointers form;
            // caller retains external targets for every possible native use.
            unsafe {
                NativeContextOwner::create_with_allocator(
                    state, device, runtime, root, allocator, true,
                )
            }
        } else {
            // SAFETY: Empty callback bundle is the ordinary default profile.
            unsafe { NativeContextOwner::create(state, device, runtime, root) }
        }?;
        if let Some(requested) = provider {
            // SAFETY: Retained owner and acquisition establish synchronous valid
            // identity/name output under common native coordination.
            let actual = unsafe {
                crate::query::native::provider(Eligibility::Usable, |desc| owner.query(desc))
            }?
            .id();
            if requested != actual {
                return Err(Error::invariant(
                    crate::error::Operation::Create,
                    crate::error::InvariantViolation::ProviderSelectionMismatch {
                        requested,
                        actual,
                    },
                ));
            }
        }

        // SAFETY: The live owner retains all dependencies and acquisition covers
        // synchronous query outputs, native coordination and ordinary errors.
        // On query/profile failure, the owner performs ordinary terminal cleanup.
        let resource_requirements = unsafe {
            crate::query::native::resource_requirements(Eligibility::Usable, |desc| {
                owner.query(desc)
            })
        }?;
        super::common::check_resource_requirements(resource_requirements.required())?;
        Ok(Self {
            owner,
            options,
            resource_requirements,
            eligibility: Eligibility::Usable,
        })
    }
}

impl Upscaler {
    /// Copy the wrapper's known usability without native calls, COM inspection,
    /// reference-count changes or waiting. This is not GPU readiness: a usable
    /// context may have pending recordings or submitted work. The snapshot can
    /// become stale after a later operation.
    #[must_use = "usability only returns a snapshot; it does not wait or change the context"]
    pub fn usability(&self) -> crate::upscaler::ContextUsability {
        self.core.eligibility.usability()
    }

    /// Record that the renderer has abandoned a recording using this context.
    ///
    /// This safe, local, idempotent operation preserves the first terminal cause,
    /// including an earlier native failure. Subsequent recording, context-dependent
    /// preflight, live queries, tuning and context diagnostics reject before
    /// inspection or FFI. Status and terminal destruction remain available.
    /// It makes no native call, inspects no COM objects, changes no reference counts,
    /// releases nothing and does not wait, invalidate recordings or recover history.
    ///
    /// Mark every context affected by the abandoned recording, including successful
    /// generator recordings. Still invalidate executable recordings and complete
    /// earlier submitted work before destruction, retaining resources and other
    /// dependencies until those obligations finish. Neither marking nor querying
    /// status proves GPU completion or callback quiescence.
    pub fn mark_recording_abandoned(&mut self) {
        self.core.eligibility.mark_recording_abandoned();
    }

    fn from_core(core: UpscalerCore<Rc<RuntimeState>>) -> Self {
        Self { core }
    }

    /// Send the deprecated global-debug descriptor through this live context.
    /// No fallback is attempted. Native error preserves its Configure code and
    /// poisons later dispatch, live queries and Configure; prior poison rejects
    /// before FFI. `Ok(())` is acknowledgement, not proof of message delivery.
    ///
    /// # Safety
    /// Uphold [`interop`]'s lasting callback contract, including for `None` and
    /// errors. All affected contexts must be CPU-idle and GPU-complete with no
    /// executable pending recordings. Serialize their shared native domain and
    /// establish synchronous descriptor consumption on success and failure.
    pub unsafe fn configure_global_debug_legacy(
        &mut self,
        level: DebugLevel,
        callback: Option<interop::MessageCallback>,
    ) -> Result<(), Error> {
        // SAFETY: Caller establishes installation, idleness and native trust.
        unsafe { self.core.configure_global_debug_legacy(level, callback) }
    }

    /// Request one direct native tuning change on this context's actual provider.
    ///
    /// `Ok(())` acknowledges the request, without proving support, application or
    /// an effective value; some providers silently ignore it. [`TuningRequest::new`]
    /// checks each parameter/value pair independently of the selected provider.
    /// Prepare the complete desired list before calling this method. Accepted
    /// bits are forwarded without clamping.
    /// `ProviderDefault` uses the native null reset, not a rounded numeric default.
    /// No history reset, automatic reapplication, cached effective value or batch
    /// rollback is provided. Construction makes no implicit Configure calls.
    ///
    /// Unsafe acquisition establishes provider compatibility and common native
    /// coordination. Prior unsafe dispatch calls require all submitted work to
    /// complete before Configure, with no unsubmitted executable recording left
    /// for later submission. New contexts are already idle. This method neither
    /// submits nor waits; `&mut self` does not establish GPU completion.
    ///
    /// Existing dispatch/Configure poison rejects before descriptor construction or FFI.
    /// Request construction is pure and independent of context eligibility.
    /// Calls remain ordered and nontransactional: a later native error does not
    /// roll back earlier acknowledgements, even when the complete list was checked.
    /// A native error preserves its code with `Operation::Tuning` and rejects
    /// later dispatch, live queries and Configure with `Error::ContextUnusable`.
    /// Stop using that context and destroy it normally; resetting a parameter is
    /// not recovery. Configure receives no command list and records no GPU work.
    pub fn request_tuning(&mut self, request: TuningRequest) -> Result<(), Error> {
        self.core.request_tuning(request)
    }

    /// Query the selected provider's per-dimension ratio for an SDK quality mode.
    /// The positive finite result is forwarded unchanged; no ratio table is imposed.
    /// No commands are submitted or GPU waits introduced; acquisition's native
    /// coordination and utility-query trust remain required.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Invalid inputs return `Error::Input`;
    /// non-OK native codes retain `Operation::Query`. Inconsistent successful outputs
    /// return `Error::Invariant`. Query errors never alter poison.
    pub fn upscale_ratio(&mut self, mode: QualityMode) -> Result<f32, Error> {
        self.core.upscale_ratio(mode)
    }

    /// Query render dimensions for a display size and SDK quality mode.
    /// This reports a quality-mode result, not the current active render size.
    /// Display components must be at most `i32::MAX`; results must be nonzero and
    /// no larger than display. Rounding comes from the provider. This does not
    /// resize the context or change its options.
    /// No commands are submitted or GPU waits introduced; acquisition's native
    /// coordination and utility-query trust remain required.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Invalid inputs return `Error::Input`;
    /// non-OK native codes retain `Operation::Query`. Inconsistent successful outputs
    /// return `Error::Invariant`. Query errors never alter poison.
    pub fn render_size_for_quality(
        &mut self,
        display: Dimensions,
        mode: QualityMode,
    ) -> Result<Dimensions, Error> {
        self.core.render_size_for_quality(display, mode)
    }

    /// Query the provider's jitter sequence length for render/display widths.
    /// Heights are ignored. Widths must be at most `i32::MAX`, with display/render
    /// in inclusive `[1, 16383]` to avoid signed SDK arithmetic overflow. The
    /// positive returned count is preserved; no phase-count formula is substituted.
    /// No commands are submitted or GPU waits introduced; acquisition's native
    /// coordination and utility-query trust remain required.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Invalid inputs return `Error::Input`;
    /// non-OK native codes retain `Operation::Query`. Inconsistent successful outputs
    /// return `Error::Invariant`. Query errors never alter poison.
    pub fn jitter_phase_count(
        &mut self,
        render: Dimensions,
        display: Dimensions,
    ) -> Result<JitterPhaseCount, Error> {
        self.core.jitter_phase_count(render, display)
    }

    /// Query a finite pixel jitter offset within the supplied sequence length.
    /// `index` must be in `0..phase_count.count()`; wrap frame indices explicitly.
    /// Components/signs are forwarded unchanged. The caller applies projection
    /// jitter and reports the actual offset to dispatch; this call changes no frame.
    /// No commands are submitted or GPU waits introduced; acquisition's native
    /// coordination and utility-query trust remain required.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Invalid inputs return `Error::Input`;
    /// non-OK native codes retain `Operation::Query`. Inconsistent successful outputs
    /// return `Error::Invariant`. Query errors never alter poison.
    pub fn jitter_offset(
        &mut self,
        index: i32,
        phase_count: JitterPhaseCount,
    ) -> Result<JitterOffsetPixels, Error> {
        self.core.jitter_offset(index, phase_count)
    }

    /// Query this context's opaque provider ID and immediately copied display name.
    ///
    /// Metadata establishes no general capability or semantic version. Acquisition's
    /// query trust and coordination apply through copying. This call does not wait
    /// for GPU completion.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Native codes are preserved with
    /// `Operation::Query`; successful zero IDs or null names are invariant failures.
    /// Query errors never alter either poison state.
    pub fn provider(&mut self) -> Result<ProviderInfo, Error> {
        self.core.provider()
    }

    /// Query provider-specific required and optional input-resource masks.
    ///
    /// Unknown bits survive. Requirements do not expand the supported dispatch
    /// profile. Acquisition establishes native coordination and compatibility with
    /// permitted GPU states. No commands are submitted and no GPU wait is introduced.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Non-OK native codes are preserved;
    /// error-path masks are not returned. Query errors do not change eligibility.
    pub fn resource_requirements(&mut self) -> Result<ResourceRequirements, Error> {
        self.core.resource_requirements()
    }

    /// Query this context's provider-specific GPU memory accounting, in bytes.
    ///
    /// This snapshot is no budget, reservation or completion check. Acquisition's
    /// query trust/coordination covers permitted work in flight. No commands are
    /// submitted and no GPU wait is introduced.
    ///
    /// # Errors
    /// Dispatch or Configure poison rejects before FFI. Non-OK native codes are preserved.
    /// Query errors do not change eligibility.
    pub fn gpu_memory_usage(&mut self) -> Result<GpuMemoryUsage, Error> {
        self.core.gpu_memory_usage()
    }

    /// Attempt native destruction once. On failure, dependencies are retained.
    ///
    /// The caller must first complete all submitted GPU work for this context
    /// and invalidate any abandoned recordings that reference it. Destruction
    /// does not wait for the GPU, including after a failed dispatch.
    /// The method consumes the context and is available even after dispatch
    /// poison. On native failure it returns [`Error::Native`] and retains the
    /// native dependencies indefinitely; `Drop` does not retry that attempt.
    /// GPU completion and recording invalidation are lasting obligations of
    /// prior unsafe recording calls; this safe method does not establish them.
    pub fn destroy(self) -> Result<(), Error> {
        self.core.owner.destroy()
    }
}

mod dispatch;

#[cfg(test)]
#[path = "dx12/tests/mod.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/native/dispatch_readback.rs"]
mod native_dispatch;

#[cfg(test)]
#[path = "../../tests/native/recording_ahead.rs"]
mod recording_ahead;

#[cfg(test)]
pub(crate) use tests::inputs::{ReactiveMaskTestInputs, UpscaleTestInputs};
