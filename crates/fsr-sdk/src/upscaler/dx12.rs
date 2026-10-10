// SPDX-License-Identifier: MPL-2.0

//! DX12 upscaler ownership, resource validation and unsafe command recording.

use crate::{
    context::{NativeContextOwner, RuntimeRetention},
    error::Error,
    runtime::{Runtime, RuntimeState},
};
use fsr_sdk_sys::{
    api::*,
    dx12::{FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12, ffxCreateBackendDX12Desc},
    upscale::*,
};
use std::{marker::PhantomPinned, pin::Pin, ptr::null_mut, rc::Rc};
use windows::core::Interface;

#[cfg(test)]
use super::common::{CameraParameters, FrameTimeMillis, JitterOffsetPixels};
use super::common::{Dimensions, UpscalerOptions};
pub use dispatch::UpscaleDispatch;
/// The exact `windows` 0.62 DX12 interface types accepted by this API.
pub use windows::Win32::Graphics::Direct3D12::{
    ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource,
};

struct UpscaleCreateState {
    root: ffxCreateContextDescUpscale,
    version: ffxCreateContextDescUpscaleVersion,
    backend: ffxCreateBackendDX12Desc,
    _pinned: PhantomPinned,
}

type Owner<R> = NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device, R>;

// Sealed Rc/Arc retention; this core deliberately remains !Send + !Sync.
struct UpscalerCore<R: RuntimeRetention> {
    owner: Owner<R>,
    options: UpscalerOptions,
    dispatch_poisoned: bool,
}

/// Owned DX12 upscaler context. Neither `Send`, `Sync`, `Copy` nor `Clone`.
///
/// The runtime, an independent COM device reference, and pinned creation
/// descriptors remain owned through teardown. Submitted dispatches must finish
/// on the GPU before this value is destroyed or dropped. A failed recording
/// must never be submitted; see [`Self::dispatch`] for the full contract.
///
/// Rust validation and COM inspection failures leave the context usable. A
/// non-OK native dispatch poisons further dispatch, without removing the native
/// owner or disabling terminal destruction. An abandoned successful recording
/// likewise requires the caller to stop using this context and invalidate the
/// recording before teardown; the wrapper cannot detect abandonment.
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
                flags: 0,
                maxRenderSize: dimensions(options.max_render_size()),
                maxUpscaleSize: dimensions(options.max_upscale_size()),
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

impl<R: RuntimeRetention> UpscalerCore<R> {
    fn new(runtime: R, device: &ID3D12Device, options: UpscalerOptions) -> Result<Self, Error> {
        // Independent COM reference retained through terminal or exceptional cleanup.
        let device = device.clone();
        let (state, root) = UpscaleCreateState::new(device.as_raw().cast(), &options);
        // SAFETY: The sealed retention came from the matching acquired family.
        // Its unsafe acquisition establishes native threading/coordination and
        // D008's trust boundary. The pinned chain owns all creation dependencies.
        let owner = unsafe { NativeContextOwner::create(state, device, runtime, root) }?;
        Ok(Self {
            owner,
            options,
            dispatch_poisoned: false,
        })
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
    /// All operations remain on the runtime's loading thread. The first dispatch
    /// profile uses explicit exposure, ordinary finite depth and fixed extents;
    /// see [`UpscaleDispatch`].
    ///
    /// # Errors
    ///
    /// Native creation failures preserve their code in [`Error::Native`]. A
    /// successful native return with a null context becomes
    /// [`Error::NativeInvariantViolation`]. The trusted runtime may abort for
    /// unsupported numerical inputs instead of returning an error.
    pub fn new(
        runtime: &Runtime,
        device: &ID3D12Device,
        options: UpscalerOptions,
    ) -> Result<Self, Error> {
        Ok(Self {
            core: UpscalerCore::new(runtime.state.clone(), device, options)?,
        })
    }

    /// Attempt native destruction once. On failure, dependencies are retained.
    ///
    /// The caller must first complete all submitted GPU work for this context
    /// and invalidate any abandoned recordings that reference it. Destruction
    /// does not wait for the GPU, including after a failed dispatch.
    /// The method consumes the context and is available even after dispatch
    /// poison. On native failure it returns [`Error::Native`] and retains the
    /// native dependencies indefinitely; `Drop` does not retry that attempt.
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
mod m6;

#[cfg(test)]
#[path = "../../tests/native/recording_ahead.rs"]
mod m6_recording;
