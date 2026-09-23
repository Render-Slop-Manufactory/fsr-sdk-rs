// SPDX-License-Identifier: MPL-2.0

use crate::{context::NativeContextOwner, error::Error, runtime::FfxLibrary};
use fsr_sdk_sys::{api::*, dx12::*, upscale::*};
use std::{marker::PhantomPinned, num::NonZeroU32, pin::Pin, ptr::null_mut};
use windows::{Win32::Graphics::Direct3D12::ID3D12Device as OwnedDevice, core::Interface};

struct UpscaleCreateState {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
    _pinned: PhantomPinned,
}

type Owner = NativeContextOwner<Pin<Box<UpscaleCreateState>>, OwnedDevice>;

/// Internal lifecycle only; no dispatch, external handle access or public builder.
pub(crate) struct Upscaler {
    owner: Owner,
}

impl UpscaleCreateState {
    fn new(
        device: *mut ID3D12Device,
        render: [NonZeroU32; 2],
        upscale: [NonZeroU32; 2],
    ) -> (Pin<Box<Self>>, *mut ffxApiHeader) {
        let header = |tag| ffxApiHeader {
            r#type: tag,
            pNext: null_mut(),
        };
        let dimensions = |size: [NonZeroU32; 2]| FfxApiDimensions2D {
            width: size[0].get(),
            height: size[1].get(),
        };
        let mut state = Box::pin(Self {
            root: ffxCreateContextDescUpscale {
                header: header(FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE),
                flags: 0,
                maxRenderSize: dimensions(render),
                maxUpscaleSize: dimensions(upscale),
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
    /// # Safety
    /// Supply a compatible trusted v2.3.0 DX12 runtime and live device, with
    /// dimensions supported by that device/provider (nonzero is not sufficient
    /// validation of all native limits). Uphold native creation/destruction and
    /// global threading preconditions. No concurrent native operation may race
    /// this lifecycle. Device ownership is transferred, never silently cloned.
    pub(crate) unsafe fn create(
        library: FfxLibrary,
        device: OwnedDevice,
        render: [NonZeroU32; 2],
        upscale: [NonZeroU32; 2],
    ) -> Result<Self, Error> {
        let (state, root) = UpscaleCreateState::new(device.as_raw().cast(), render, upscale);
        // SAFETY: Private chain is valid and stable; independent COM ownership
        // and runtime move into the owner. No callbacks/dispatch are exposed.
        let owner = unsafe { NativeContextOwner::create(state, device, library, root) }?;
        Ok(Self { owner })
    }

    pub(crate) fn destroy(self) -> Result<(), Error> {
        self.owner.destroy()
    }
}

#[cfg(test)]
#[path = "upscaler/tests.rs"]
mod tests;
