// SPDX-License-Identifier: MPL-2.0

//! Checked common inputs and the DX12 upscaler backend.
//!
//! [`common`](crate::upscaler::common) owns configuration and frame scalars
//! without GPU object types. `dx12` owns construction,
//! borrowed GPU inputs, validation and recording.
//! Existing imports from this module remain aliases of those same types.
//!
//! Common inputs are available on every host without backend features. The
//! `dx12` module and its reexports require Windows and the `dx12` feature. Vulkan is
//! not implemented; a future backend needs its own native integration and
//! resource/synchronization contract before exposing a module here.

pub mod common;
#[cfg(all(windows, feature = "dx12"))]
pub mod dx12;

pub use common::{
    CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscalerOptions,
};
#[cfg(all(windows, feature = "dx12", feature = "movable"))]
pub use dx12::MovableUpscaler;
#[cfg(all(windows, feature = "dx12"))]
pub use dx12::{
    ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource, NativeSharpness, Sharpening,
    UpscaleDispatch, UpscaleResources, Upscaler,
};
