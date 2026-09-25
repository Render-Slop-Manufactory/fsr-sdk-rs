// SPDX-License-Identifier: MPL-2.0

use crate::{context::NativeContextOwner, error::Error, runtime::Runtime};
use fsr_sdk_sys::{
    api::*,
    dx12::{FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, ffxCreateBackendDX12Desc},
    upscale::*,
};
use std::{marker::PhantomPinned, num::NonZeroU32, pin::Pin, ptr::null_mut};
use windows::core::Interface;

pub use dispatch::{CameraParameters, FrameTimeMillis, JitterOffsetPixels, UpscaleDispatch};
/// The exact `windows` 0.62 DX12 interface types accepted by this API.
pub use windows::Win32::Graphics::Direct3D12::{
    ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource,
};

/// A width and height that have both been checked for zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl Dimensions {
    /// Rejects either zero component before any native call.
    pub fn new(width: u32, height: u32) -> Result<Self, Error> {
        let Some(width_checked) = NonZeroU32::new(width) else {
            return Err(Error::InvalidDimensions { width, height });
        };
        let Some(height_checked) = NonZeroU32::new(height) else {
            return Err(Error::InvalidDimensions { width, height });
        };
        Ok(Self {
            width: width_checked,
            height: height_checked,
        })
    }
}

/// Maximum render and output dimensions for one upscaler context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpscalerOptions {
    pub max_render_size: Dimensions,
    pub max_upscale_size: Dimensions,
}

struct UpscaleCreateState {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
    _pinned: PhantomPinned,
}

type Owner = NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device>;

/// Owned DX12 upscaler context. Neither `Send`, `Sync`, `Copy` nor `Clone`.
///
/// The runtime, an independent COM device reference, and pinned creation
/// descriptors remain owned through teardown. Submitted dispatches must finish
/// on the GPU before this value is destroyed or dropped. A failed recording
/// must never be submitted; see [`Self::dispatch`] for the full contract.
pub struct Upscaler {
    owner: Owner,
    options: UpscalerOptions,
    dispatch_poisoned: bool,
}

impl UpscaleCreateState {
    fn new(
        device: *mut fsr_sdk_sys::dx12::ID3D12Device,
        options: UpscalerOptions,
    ) -> (Pin<Box<Self>>, *mut ffxApiHeader) {
        let header = |tag| ffxApiHeader {
            r#type: tag,
            pNext: null_mut(),
        };
        let dimensions = |size: Dimensions| FfxApiDimensions2D {
            width: size.width.get(),
            height: size.height.get(),
        };
        let mut state = Box::pin(Self {
            root: ffxCreateContextDescUpscale {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE),
                flags: 0,
                maxRenderSize: dimensions(options.max_render_size),
                maxUpscaleSize: dimensions(options.max_upscale_size),
                fpMessage: None,
            },
            version: ffxCreateContextDescUpscaleVersion {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION),
                version: FFX_UPSCALER_VERSION,
            },
            backend: ffxCreateBackendDX12Desc {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12),
                device,
            },
            _pinned: PhantomPinned,
        });
        // SAFETY: Final allocation precedes pointer formation. This private type
        // never exposes mutable references, extracts/replaces pointees or relinks
        // after this block. No Rust reference is kept across native calls.
        let root = unsafe {
            let state = state.as_mut().get_unchecked_mut();
            state.root.header.pNext = &raw mut state.version.header;
            state.version.header.pNext = &raw mut state.backend.header;
            &raw mut state.root.header
        };
        (state, root)
    }
}

impl Upscaler {
    /// Create from an acquired runtime and a borrowed DX12 device.
    ///
    /// The caller may drop its device and runtime values after this returns.
    /// Each attempt retains its own COM reference during the native call and,
    /// on success, until teardown. Dimensions are nonzero; further numerical
    /// limits are delegated to the trusted v2.3.0 runtime and provider. Native
    /// rejection, and for some unsupported inputs a process abort, remain
    /// possible. The runtime's unsafe acquisition contract must remain upheld.
    /// Flags are fixed to zero; no GPU work is submitted here.
    pub fn new(
        runtime: &Runtime,
        device: &ID3D12Device,
        options: UpscalerOptions,
    ) -> Result<Self, Error> {
        // COM Clone calls AddRef: this attempt owns a reference independent of
        // the caller and retains it on exceptional native outcomes.
        let device = device.clone();
        let (state, root) = UpscaleCreateState::new(device.as_raw().cast(), options);
        // SAFETY: The private pinned chain is valid, uses the owned device, and
        // belongs to the runtime family acquired under its unsafe contract.
        // Creation supplies no callbacks or dispatch; native calls are serial
        // on the acquiring thread. D008 bounds further numerical runtime trust.
        let owner =
            unsafe { NativeContextOwner::create(state, device, runtime.state.clone(), root) }?;
        Ok(Self {
            owner,
            options,
            dispatch_poisoned: false,
        })
    }

    /// Attempt native destruction once. On failure, dependencies are retained.
    ///
    /// The caller must first complete all submitted GPU work for this context
    /// and invalidate any abandoned recordings that reference it. Destruction
    /// does not wait for the GPU, including after a failed dispatch.
    pub fn destroy(self) -> Result<(), Error> {
        self.owner.destroy()
    }
}

mod dispatch;

#[cfg(test)]
#[path = "upscaler/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "upscaler/m6.rs"]
mod m6;

#[cfg(test)]
#[path = "upscaler/m6_recording.rs"]
mod m6_recording;
