// SPDX-License-Identifier: MPL-2.0

//! Rust ownership and checked inputs for AMD's FidelityFX SDK v2.3.0.
//!
//! # What this crate provides
//!
//! The supported native target is `x86_64-pc-windows-msvc` with the default
//! `dx12` feature. Windows/DX12 APIs are exposed on other Windows targets, but
//! those targets are currently unverified and unsupported; compilation may succeed.
//! Native verification is limited to the recorded SDK, runtime/provider, GPU,
//! driver and workload configurations; target support does not imply compatibility
//! with every hardware/provider configuration. The
//! [project scope](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/SCOPE.md#platform-and-compatibility-boundaries)
//! owns this policy.
//!
//! With `dx12`, unsafe `runtime::Runtime::load` acquires a trusted loader.
//! Unsafe `runtime.upscaler_dx12(&device)` separately establishes lasting
//! effect/provider assumptions and returns `upscaler::dx12::UpscalerSetup`.
//! The setup binds one device and `query::ProviderSelection` for enumeration,
//! quality/size/jitter/resource planning, memory estimates and safe `create`.
//! `UpscalerOptions` is a portable creation profile with no provider field.
//! A setup's selection can change; each explicit operation checks fresh
//! membership, without fallback, caching or reservation. Default can resolve
//! differently per call. Acquisition covers every safely reachable selection.
//! Created contexts retain their runtime, device and pinned storage independently
//! and do not borrow the setup. Their unsafe `record` records renderer-owned
//! DX12 objects. Runtime and DX12 APIs require Windows and `dx12`; owned errors,
//! checked inputs, creation options and query results remain portable.
//! In 0.3, setup-bound operations replace the earlier flat runtime queries and
//! public context constructors. See the
//! [migration guide](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/MIGRATING_0_3.md).
//!
//! Portable `upscaler::TuningRequest::new` checks a `TuningParameter` and
//! `TuningValue::Value` or explicit `ProviderDefault` before native mutation.
//! Prepare the complete desired list, then pass each checked request to either
//! family's safe `request_tuning(request)`. Accepted bits are forwarded unchanged.
//! Success acknowledges a request;
//! it does not establish application, support or an effective value. Unsafe
//! acquisition carries actual-provider parameter-domain/reset safety and synchronous
//! non-retention trust. Earlier unsafe dispatches require completed submitted work
//! and no pending executable recording before subsequent safe Configure. The
//! wrapper does not submit/wait, automatically reset history or cache tuning values.
//!
//! Owned `error::Error` has structured input/native/backend/invariant/terminal/
//! allocation categories, preserves original source chains and is Send + Sync.
//! It makes no Clone/Copy/equality commitment. Native failures expose their exact
//! operation/code; later unusable-context errors retain the first terminal cause.
//! Pure `usability()` and local `mark_recording_abandoned()` also represent
//! renderer-reported abandonment without native/COM work, waits or releases.
//! Portable provider metadata can be constructed as ordinary data, without
//! bypassing fresh membership or selected-provider identity checks. Resource-mask
//! composition preserves unknown bits. These APIs establish no GPU readiness.
//!
//! Explicit unsafe native callbacks are exposed through
//! `upscaler::dx12::interop`, setup `create_with_callbacks` and setup/live diagnostic
//! methods. Targets remain externally owned and may remain globally reachable
//! after destruction or errors. The pinned host table is retained through create
//! and destroy; callback-bearing native create failure retains dependencies.
//! Ordinary debug options and the independent frame debug-view flag default off.
//!
//! # Installation and features
//!
//! Requires Rust 1.88 or later. DX12 is enabled by default:
//!
//! ```toml
//! [dependencies]
//! fsr-sdk = "0.3.0"
//! ```
//!
//! Enable `movable` for the separate caller-coordinated thread-transfer family.
//! Enable `utils` for the portable provider-generation resolver; it enables no
//! backend or thread-transfer feature. Use `default-features = false` for portable
//! inputs, errors and query metadata. The reserved `vulkan` feature deliberately
//! fails compilation, including with `--all-features`; Vulkan is not implemented.
//!
//! # Supported workflow
//!
//! 1. Acquire the signed SDK v2.3.0 runtime separately and load it explicitly.
//!    The upscaler provider DLL belongs beside the application executable.
//! 2. Acquire a DX12 setup, optionally bind a provider, then plan/create with checked options.
//! 3. Prepare the required textures, their states, and an open DIRECT command list.
//! 4. Bind named `UpscaleResources` and prepare portable `FrameParameters` from checked jitter,
//!    timing and camera values. Optionally set pre-exposure or native sharpening,
//!    then record.
//! 5. Close and submit the list through your renderer, establish queue/fence
//!    ordering, and retain the context and GPU resources until completion.
//!
//! # Input conventions
//!
//! The caller supplies matching colour, depth, motion and exposure content.
//! Input types and recording methods describe the required signal conventions
//! within the actual provider's supported numerical and access domain. Matching
//! those conventions does not establish the native safety assumptions below.
//!
//! # Safety model
//!
//! The crate retains the runtime, context and its own device reference, builds
//! native descriptors, validates inspectable dimensions/formats/device identity
//! and scalar values, preserves native errors, and rejects dispatch, live queries
//! and Configure after a native dispatch/Configure failure or explicit abandonment.
//! Mark every context affected by an abandoned recording, invalidate executable
//! recordings and complete earlier submitted work before terminal cleanup.
//! Usability is not GPU readiness. The wrapper does not submit
//! commands or wait during configuration or teardown.
//!
//! The caller establishes actual resource states, non-overlapping backing
//! storage, residency, queue ordering and GPU completion. The caller must also prevent premature resource,
//! command-list or allocator reuse. Rust borrows last only through the CPU call;
//! they do not retain objects for later GPU use. Runtime calls stay on the loading
//! thread and external FidelityFX calls touching shared state must be serialized
//! and non-reentrant. Read `Runtime::load` and `Upscaler::record` for
//! the full safety and failure contracts; `Runtime::upscaler_dx12` supplies
//! lasting effect/provider trust separately from loading.
//! CPU pointer/ABI and native memory-lifetime obligations remain distinct from
//! signal interpretation. The general Rust/host-memory consequences of GPU-only
//! state, aliasing or ordering violations remain unresolved; the documented
//! conservative native safety obligations still apply.
//!
//! With the optional `movable` feature, `runtime::MovableRuntime` and
//! `upscaler::MovableUpscaler` instead provide `Send + !Sync` ownership, acquired
//! through unsafe loading and effect setup acquisition. Those contracts require native
//! compatibility on every subsequent destination thread and common coordination
//! through all calls and implicit Drop/unwind. It provides no mutex or concurrency
//! guarantee and leaves dispatch unsafe. The existing local family remains
//! `!Send + !Sync`. The repository's
//! [movable example](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/movable_upscale.rs)
//! demonstrates consumer-owned cleanup coordination, including failed handoffs.
//!
//! With the optional `utils` feature, `utils::provider` resolves an explicit
//! upscaling generation using strict provider-name recognition. It returns the
//! core `ProviderSelection` for application through setup `with_provider`; `Auto` keeps the
//! SDK default. No fallback, compatibility guarantee or live context switching
//! is provided. The resolver is portable and enables no native backend.
//!
//! [Complete local/movable consumer workflows](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/upscaler_workflows.rs)
//! cover first-frame completion, explicit-provider planning/create and portable
//! preparation followed by recording/abandonment, including optional utilities.
//! The [0.3 migration guide](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/MIGRATING_0_3.md)
//! distinguishes published 0.2 source breaks from earlier development API changes.
//!
//! # Minimal integration
//!
//! These excerpts assume an existing renderer and a trusted runtime deployment.
//! Create the context once, before recording frames:
//!
//! ```no_run
//! # #[cfg(all(windows, feature = "dx12"))]
//! # pub unsafe fn create(loader_path: &std::path::Path, device: &fsr_sdk::upscaler::ID3D12Device)
//! #     -> Result<fsr_sdk::upscaler::Upscaler, Box<dyn std::error::Error>> {
//! use fsr_sdk::{runtime::Runtime, upscaler::{Dimensions, Upscaler, UpscalerOptions}};
//!
//! // SAFETY: The application establishes Runtime::load's native trust contract.
//! let mut runtime = unsafe { Runtime::load(loader_path)? };
//! let options = UpscalerOptions::new(
//!     Dimensions::new(320, 180)?,
//!     Dimensions::new(640, 360)?,
//! );
//! // SAFETY: Establish every lasting effect/provider assumption in upscaler_dx12.
//! let mut setup = unsafe { runtime.upscaler_dx12(device) };
//! let upscaler = setup.create(&options)?;
//! // The upscaler retains the runtime even after this public Runtime is dropped.
//! # Ok(upscaler)
//! # }
//! # fn main() {}
//! ```
//!
//! For a first reset frame, with matching textures and renderer-applied jitter:
//!
//! ```no_run
//! # #[cfg(all(windows, feature = "dx12"))]
//! # pub unsafe fn record(upscaler: &mut fsr_sdk::upscaler::Upscaler,
//! #     command_list: &fsr_sdk::upscaler::ID3D12GraphicsCommandList,
//! #     color: &fsr_sdk::upscaler::ID3D12Resource, depth: &fsr_sdk::upscaler::ID3D12Resource,
//! #     motion_vectors: &fsr_sdk::upscaler::ID3D12Resource,
//! #     exposure: &fsr_sdk::upscaler::ID3D12Resource, output: &fsr_sdk::upscaler::ID3D12Resource)
//! #     -> Result<(), fsr_sdk::error::Error> {
//! use fsr_sdk::upscaler::{CameraParameters, FrameTimeMillis, JitterOffsetPixels,
//!     FrameParameters, HistoryAction, ExposureInput, UpscaleResources};
//!
//! let resources = UpscaleResources::builder()
//!     .color(color).depth(depth).motion_vectors(motion_vectors)
//!     .exposure(ExposureInput::Texture(exposure)).output(output).build()?;
//! let frame = FrameParameters::new(
//!     JitterOffsetPixels::new(0.25, -0.25)?,
//!     FrameTimeMillis::new(16.667)?,
//!     CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
//!     HistoryAction::Reset,
//! );
//! // SAFETY: The renderer establishes the states, storage and GPU lifetime contract.
//! let recorded = unsafe { upscaler.record(command_list, &resources, &frame) };
//! // On Ok: submit once in order; keep the upscaler/resources alive through a fence.
//! // On native error: discard this list, finish earlier work, then tear down.
//! # recorded
//! # }
//! # fn main() {}
//! ```
//!
//! The repository's [`dx12_upscale` example](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/dx12_upscale.rs)
//! contains compilable integration functions without a renderer or raw native API.
//!
//! # Limitations
//!
//! DX12 upscaling requires distinct, single-sample/mip/slice Texture2D resources.
//! The Windows/DX12 `upscaler::dx12::UpscaleResources` documentation defines the
//! supported formats and views for each role; accepted metadata does not prove
//! actual-provider compatibility. Motion uses one UV/Pixels/Native scale choice, with stored components matching the provider's
//! current-to-previous interpretation. Exposure uses a 1×1 texture or the
//! context's automatic mode.
//!
//! Colour, depth and mask allocations stay at maximum render size; output stays
//! at maximum upscale size, and motion allocation sizes follow the context's
//! render/display profile. Dynamic contexts may select smaller top-left active
//! render and output regions within those allocations, under actual-provider
//! support and safe-access obligations. The full output allocation may be written;
//! padding is not preserved. Optional reactive and transparency/composition masks
//! contain matching signals in the active render region. Provider requirements
//! are checked at creation and mask presence at dispatch. Linear HDR is opt-in
//! through `UpscalerOptions::set_high_dynamic_range`. Native non-linear creation
//! and per-frame sRGB/PQ hints are exposed with checked pairing by default and
//! an explicit per-frame mismatch override. Actual provider encoding/exposure
//! semantics remain caller-established; no PQ normalization or conversion is
//! provided. See the [colour and exposure contract](upscaler#colour-and-exposure-contract)
//! for caller obligations and the limits of bounded native verification. Depth convention,
//! motion resolution and jitter inclusion are selected before context creation;
//! defaults are standard finite depth and render-resolution jitter-free UV motion.
//! An infinite depth profile requires `CameraParameters::new_infinite`.
//! Native sharpening is disabled by default. `Sharpening::Native(NativeSharpness)`
//! forwards a finite `[0, 1]` parameter unchanged; enabled zero is not disabling.
//! No wrapper filter or image-quality guarantee is provided.
//! Vulkan, Frame Generation, Ray Regeneration, Radiance Caching and engine
//! integrations are not implemented.
//!
//! The [integration coverage](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/upscaling-coverage.md)
//! records provider-specific queries, ABI, resource mapping, fenced GPU execution
//! and cleanup observations. Passing those bounded checks does not establish
//! image quality, arbitrary temporal behaviour, safe numerical domains or
//! compatibility across GPUs/drivers/providers. Native trust and renderer
//! synchronization duties remain part of the documented unsafe contracts.

#![warn(missing_docs)]

#[cfg(test)]
extern crate self as fsr_sdk;
#[cfg(test)]
#[macro_use]
#[path = "../tests/fixtures/errors.rs"]
mod test_errors;

/// Owned errors and rejection reasons, available on every host.
pub mod error;

/// Owned provider metadata, resource masks and memory values, available on every host.
pub mod query;

/// Optional project-defined utilities, available on every host with `utils`.
#[cfg(feature = "utils")]
pub mod utils;

#[cfg(test)]
#[path = "../tests/fixtures/directory.rs"]
mod test_directory;

#[cfg(any(test, all(windows, feature = "dx12")))]
mod context;
#[cfg(any(test, all(windows, feature = "dx12")))]
mod context_policy;
/// Common checked inputs and the DX12 upscaler backend, with legacy reexports.
pub mod upscaler;

#[cfg(all(windows, feature = "dx12"))]
/// Explicit loading with local ownership and optional caller-coordinated movability.
pub mod runtime;

// Host-only fixture adapter. It does not enable an AMD runtime on other OSes.
#[cfg(all(test, not(all(windows, feature = "dx12"))))]
#[path = "context/test_runtime.rs"]
mod runtime;
