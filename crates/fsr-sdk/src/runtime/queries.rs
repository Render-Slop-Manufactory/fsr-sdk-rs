// SPDX-License-Identifier: MPL-2.0

//! Shared DX12 pre-create routing for both ownership families.
use super::RuntimeState;
use crate::context_policy::Eligibility;
use crate::{
    error::Error,
    query::{
        GpuMemoryUsage, JitterPhaseCount, ProviderId, ProviderInfo, QualityMode,
        ResourceRequirements, native,
    },
    upscaler::{
        common::{Dimensions, JitterOffsetPixels, UpscalerOptions},
        dx12::ID3D12Device,
    },
};
use fsr_sdk_sys::{api::*, dx12::*};
use windows::core::Interface;

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn providers(
    state: &RuntimeState,
    device: &ID3D12Device,
) -> Result<Vec<ProviderInfo>, Error> {
    // SAFETY: Local or movable setup acquisition establishes D018's query trust and
    // common coordination on this use thread. Runtime and device remain valid
    // through both enumeration calls and immediate name copying.
    unsafe {
        native::providers(device.as_raw(), |desc| {
            state.query(std::ptr::null_mut(), desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn require_provider(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
) -> Result<(), Error> {
    // SAFETY: Acquired state establishes capacity/name validity and coordination
    // through this complete synchronous count/fill/copy sequence.
    unsafe {
        native::require_provider(device.as_raw(), provider, |desc| {
            state.query(std::ptr::null_mut(), desc)
        })
    }
}

/// # Safety
/// Supply a complete writable query without extensions. Acquisition covers the
/// actual provider, synchronous pointer non-retention and shared coordination.
/// An explicit ID must already have passed fresh runtime/device enumeration.
unsafe fn invoke(
    state: &RuntimeState,
    device: Option<&ID3D12Device>,
    provider: Option<ProviderId>,
    desc: *mut ffxQueryDescHeader,
) -> u32 {
    let mut selection = native::provider_override(provider);
    let selection_ptr = selection
        .as_mut()
        .map_or(std::ptr::null_mut(), |node| &raw mut node.header);
    let mut backend = device.map(|device| ffxCreateBackendDX12Desc {
        header: ffxApiHeader {
            r#type: FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12,
            pNext: selection_ptr,
        },
        device: device.as_raw().cast(),
    });
    // SAFETY: Final-address stack nodes and initialized outputs remain live for
    // the synchronous call; the borrowed root link is restored before returning.
    unsafe {
        debug_assert!((*desc).pNext.is_null());
        (*desc).pNext = backend
            .as_mut()
            .map_or(selection_ptr, |node| &raw mut node.header);
        let code = state.query(std::ptr::null_mut(), desc);
        (*desc).pNext = std::ptr::null_mut();
        code
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn ratio(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
    mode: QualityMode,
) -> Result<f32, Error> {
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Fresh selection and acquisition's numerical/non-retention trust
    // cover this known utility descriptor and its output/device lifetime.
    unsafe {
        native::upscale_ratio(Eligibility::Usable, mode, |desc| {
            invoke(state, Some(device), provider, desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn render_size(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
    display: Dimensions,
    mode: QualityMode,
) -> Result<Dimensions, Error> {
    native::validate_render_size(display)?;
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Pure numerical checks and fresh selection passed; acquisition
    // covers the actual provider and synchronous temporary storage consumption.
    unsafe {
        native::render_size_for_quality(Eligibility::Usable, display, mode, |desc| {
            invoke(state, Some(device), provider, desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn jitter_phase_count(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
    render: Dimensions,
    display: Dimensions,
) -> Result<JitterPhaseCount, Error> {
    native::validate_jitter_phase_count(render, display)?;
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Checked arithmetic and fresh selection under acquisition trust;
    // no temporary descriptor/device/output address is retained.
    unsafe {
        native::jitter_phase_count(Eligibility::Usable, render, display, |desc| {
            invoke(state, Some(device), provider, desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn jitter_offset(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
    index: i32,
    phase_count: JitterPhaseCount,
) -> Result<JitterOffsetPixels, Error> {
    native::validate_jitter_offset(index, phase_count)?;
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Checked index/count and fresh selection; acquisition covers the
    // selected provider's numerical and synchronous pointer-consumption contract.
    unsafe {
        native::jitter_offset(Eligibility::Usable, index, phase_count, |desc| {
            invoke(state, Some(device), provider, desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn resource_requirements(
    state: &RuntimeState,
    device: &ID3D12Device,
    provider: Option<ProviderId>,
) -> Result<ResourceRequirements, Error> {
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Acquisition covers this device-backed null-context route and its
    // default/explicit provider. The returned masks are informational only.
    unsafe {
        native::resource_requirements(Eligibility::Usable, |desc| {
            invoke(state, Some(device), provider, desc)
        })
    }
}

/// # Safety
/// Establish the matching setup's lasting effect/query contract, including
/// safely reachable providers, numerical domains, non-retention and common
/// native coordination. A loaded RuntimeState alone establishes none of these.
pub(crate) unsafe fn estimate_memory(
    state: &RuntimeState,
    device: &ID3D12Device,
    options: &UpscalerOptions,
    provider: Option<ProviderId>,
) -> Result<GpuMemoryUsage, Error> {
    // SAFETY: The caller establishes the setup's enumeration/selection trust.
    unsafe { require_provider(state, device, provider) }?;
    // SAFETY: Checked options and fresh selection under acquisition trust. V2
    // carries its device directly, so no backend extension is added.
    unsafe {
        native::estimate_memory(device.as_raw(), options, |desc| {
            invoke(state, None, provider, desc)
        })
    }
}
