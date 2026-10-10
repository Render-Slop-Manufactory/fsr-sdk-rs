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
//! With `dx12`, `runtime::Runtime` loads an explicit trusted loader
//! DLL and `upscaler::Upscaler` owns an upscaling context. Its unsafe `dispatch`
//! method records a frame from renderer-owned DX12 objects. Owned error types
//! remain available on other hosts, but the runtime and upscaler modules do not.
//!
//! # Supported workflow
//!
//! 1. Acquire the signed SDK v2.3.0 runtime separately and load it explicitly.
//!    The upscaler provider DLL belongs beside the application executable.
//! 2. Create an upscaler from a DX12 device and checked render/output dimensions.
//! 3. Prepare the five textures, their states, and an open DIRECT command list.
//! 4. Construct the checked jitter, timing and camera values, then record dispatch.
//! 5. Close and submit the list through your renderer, establish queue/fence
//!    ordering, and retain the context and GPU resources until completion.
//!
//! # Safety model
//!
//! The crate retains the runtime, context and its own device reference, builds
//! native descriptors, validates inspectable dimensions/formats/device identity
//! and scalar values, preserves native errors, and rejects further dispatch after
//! a native dispatch failure. It does not submit commands or wait during teardown.
//!
//! The caller supplies correct color/depth/motion/exposure content and establishes
//! actual resource states, non-overlapping backing storage, residency, queue
//! ordering and GPU completion. The caller must also prevent premature resource,
//! command-list or allocator reuse. Rust borrows last only through the CPU call;
//! they do not retain objects for later GPU use. Runtime calls stay on the loading
//! thread and external FidelityFX calls touching shared state must be serialized
//! and non-reentrant. Read `Runtime::load_unchecked` and `Upscaler::dispatch` for
//! the full safety and failure contracts, including native provider trust.
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
//! // SAFETY: The application establishes Runtime::load_unchecked's native trust contract.
//! let runtime = unsafe { Runtime::load_unchecked(loader_path)? };
//! let options = UpscalerOptions::new(
//!     Dimensions::new(320, 180)?,
//!     Dimensions::new(640, 360)?,
//! );
//! let upscaler = Upscaler::new(&runtime, device, options)?;
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
//! use fsr_sdk::upscaler::{CameraParameters, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch};
//!
//! let frame = UpscaleDispatch {
//!     command_list, color, depth, motion_vectors, exposure, output,
//!     jitter: JitterOffsetPixels::new(0.25, -0.25)?,
//!     frame_time: FrameTimeMillis::new(16.667)?,
//!     camera: CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
//!     reset: true,
//! };
//! // SAFETY: The renderer establishes the states, storage and GPU lifetime contract.
//! let recorded = unsafe { upscaler.dispatch(frame) };
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
//! The first profile fixes creation flags to zero and requires distinct,
//! single-sample/mip/slice Texture2D resources: render-size RGBA16F linear non-HDR
//! color, R32F ordinary finite depth, RG16F jitter-free current-to-previous
//! top-down UV motion, 1×1 R32F explicit exposure, and a UAV-capable RGBA16F output
//! at the configured output size. Active sizes equal the context maxima. HDR,
//! dynamic resolution, auto exposure, masks and sharpening are not exposed.
//! Vulkan, Frame Generation, Ray Regeneration, Radiance Caching and engine
//! integrations are not implemented. The published baseline is `0.1.0`.
//!
//! [M6b](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
//! recorded one submitted, fenced public dispatch and plausible readback on one
//! configuration. This does not establish image quality, nonzero-motion temporal
//! behavior, performance or compatibility across GPUs/drivers/providers. The
//! [final release verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-28-exp-final-windows-release-verification.md)
//! subsequently passed a three-frame moving smoke, and post-release source adds
//! [controlled motion/jitter/history-reset comparisons](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-30-exp-dispatch-semantics.md)
//! on that configuration. These are bounded regressions, not general temporal
//! image-quality evidence or revalidation of the published archives. See the
//! [roadmap](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/ROADMAP.md)
//! and accepted [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md).

#![warn(missing_docs)]

/// Owned errors and rejection reasons, available on every host.
pub mod error;

#[cfg(test)]
#[path = "../tests/fixtures/directory.rs"]
mod test_directory;

#[cfg(any(test, all(windows, feature = "dx12")))]
mod context;
#[cfg(all(windows, feature = "dx12"))]
/// DX12 upscaler ownership, checked frame inputs and unsafe command recording.
pub mod upscaler;

#[cfg(all(windows, feature = "dx12"))]
/// Explicit loading and thread-confined ownership of the native runtime.
pub mod runtime;

// Host-only fixture adapter. It does not enable an AMD runtime on other OSes.
#[cfg(all(test, not(all(windows, feature = "dx12"))))]
#[path = "context/test_runtime.rs"]
mod runtime;
