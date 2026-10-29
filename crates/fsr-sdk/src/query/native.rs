// SPDX-License-Identifier: MPL-2.0

//! Private descriptor construction and result interpretation shared by both families.
use super::{GpuMemoryUsage, ProviderId, ProviderInfo, ResourceFlags, ResourceRequirements};
use crate::{
    context_policy::Eligibility,
    error::{Error, InvariantViolation, Operation},
    upscaler::common::UpscalerOptions,
};
use fsr_sdk_sys::{api::*, upscale::*};
use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{null, null_mut},
};

mod utilities;
#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) use utilities::{
    jitter_offset, jitter_phase_count, render_size_for_quality, upscale_ratio,
};
#[cfg(all(windows, feature = "dx12"))]
pub(crate) use utilities::{
    validate_jitter_offset, validate_jitter_phase_count, validate_render_size,
};

pub(crate) fn provider_override(provider: Option<ProviderId>) -> Option<ffxOverrideVersion> {
    provider.map(|id| ffxOverrideVersion {
        header: header(FFX_API_DESC_TYPE_OVERRIDE_VERSION),
        versionId: id.0,
    })
}

/// # Safety
/// Enumerate on the actual runtime/device under the acquisition's capacity,
/// name-validity and common coordination contract, through immediate copying.
pub(crate) unsafe fn require_provider(
    device: *mut c_void,
    requested: Option<ProviderId>,
    invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<(), Error> {
    let Some(requested) = requested else {
        return Ok(());
    };
    // SAFETY: Caller establishes bounded enumeration and name copying.
    if unsafe { providers(device, invoke) }?
        .iter()
        .any(|info| info.id() == requested)
    {
        Ok(())
    } else {
        Err(Error::Input(
            crate::error::InputError::ProviderNotEnumerated { requested },
        ))
    }
}

fn header(tag: u64) -> ffxQueryDescHeader {
    ffxApiHeader {
        r#type: tag,
        pNext: null_mut(),
    }
}

fn check(code: u32) -> Result<(), Error> {
    if code == FFX_API_RETURN_OK {
        Ok(())
    } else {
        Err(Error::native(Operation::Query, code))
    }
}

fn buffer<T: Clone>(count: u64, initial: T) -> Result<Vec<T>, Error> {
    let size = usize::try_from(count).map_err(|source| {
        Error::QueryAllocation(crate::error::QueryAllocationError::new(
            count,
            crate::error::QueryAllocationReason::CountNotRepresentable,
            source,
        ))
    })?;
    let mut result = Vec::new();
    result.try_reserve_exact(size).map_err(|source| {
        Error::QueryAllocation(crate::error::QueryAllocationError::new(
            count,
            crate::error::QueryAllocationReason::ReservationFailed,
            source,
        ))
    })?;
    result.resize(size, initial);
    Ok(result)
}

unsafe fn copy_provider(id: u64, name: *const c_char) -> Result<ProviderInfo, Error> {
    if name.is_null() {
        return Err(Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::NullProviderNameOnSuccess,
        ));
    }
    // SAFETY: The query acquisition contract guarantees a readable, terminated
    // name through immediate copying, with shared native coordination still held.
    let name = unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned();
    Ok(ProviderInfo {
        id: ProviderId(id),
        name,
    })
}

fn memory(result: FfxApiEffectMemoryUsage) -> GpuMemoryUsage {
    GpuMemoryUsage {
        total_bytes: result.totalUsageInBytes,
        aliasable_bytes: result.aliasableUsageInBytes,
    }
}

/// # Safety
/// Supply a valid borrowed device and a synchronous, non-retaining invocation
/// obeying D018, including capacity bounds and name validity through copying.
/// Keep the runtime and shared native coordination live for this entire operation.
pub(crate) unsafe fn providers(
    device: *mut c_void,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<Vec<ProviderInfo>, Error> {
    let mut count = 0;
    let mut desc = ffxQueryDescGetVersions {
        header: header(FFX_API_QUERY_DESC_TYPE_GET_VERSIONS),
        createDescType: FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
        device,
        outputCount: &mut count,
        versionIds: null_mut(),
        versionNames: null_mut(),
    };
    check(invoke(&mut desc.header))?;
    if count == 0 {
        return Ok(Vec::new());
    }
    let capacity = count;
    let mut ids = buffer(capacity, 0u64)?;
    let mut names = buffer(capacity, null::<c_char>())?;
    let mut result = Vec::new();
    result.try_reserve_exact(ids.len()).map_err(|source| {
        Error::QueryAllocation(crate::error::QueryAllocationError::new(
            capacity,
            crate::error::QueryAllocationReason::ReservationFailed,
            source,
        ))
    })?;
    desc.versionIds = ids.as_mut_ptr();
    desc.versionNames = names.as_mut_ptr();
    check(invoke(&mut desc.header))?;
    if count > capacity {
        return Err(Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::QueryCountExceedsCapacity,
        ));
    }
    for (&id, &name) in ids.iter().zip(&names).take(count as usize) {
        // SAFETY: The caller guarantees valid names and coordination through
        // copying; no subsequent native operation occurs between fill and copy.
        result.push(unsafe { copy_provider(id, name) }?);
    }
    Ok(result)
}

/// # Safety
/// `invoke` must make the matching global query under the D018 acquisition
/// contract, synchronously consuming the valid device and initialized outputs.
pub(crate) unsafe fn estimate_memory(
    device: *mut c_void,
    options: &UpscalerOptions,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<GpuMemoryUsage, Error> {
    let mut output = FfxApiEffectMemoryUsage::default();
    let dimensions = |size: crate::upscaler::common::Dimensions| FfxApiDimensions2D {
        width: size.width(),
        height: size.height(),
    };
    let mut desc = ffxQueryDescUpscaleGetGPUMemoryUsageV2 {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2),
        device,
        maxRenderSize: dimensions(options.max_render_size()),
        maxUpscaleSize: dimensions(options.max_upscale_size()),
        flags: options.native_flags(),
        gpuMemoryUsageUpscaler: &mut output,
    };
    check(invoke(&mut desc.header))?;
    Ok(memory(output))
}

/// # Safety
/// Invoke the matching live-context query under D018. Returned names must be
/// readable and terminated through copying; retain context/runtime and coordination.
pub(crate) unsafe fn provider(
    eligibility: Eligibility,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<ProviderInfo, Error> {
    eligibility.check()?;
    let mut desc = ffxQueryGetProviderVersion {
        header: header(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION),
        versionId: 0,
        versionName: null(),
    };
    check(invoke(&mut desc.header))?;
    if desc.versionId == 0 {
        return Err(Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::ZeroProviderIdOnSuccess,
        ));
    }
    // SAFETY: The caller establishes immediate name validity and synchronization.
    unsafe { copy_provider(desc.versionId, desc.versionName) }
}

/// # Safety
/// Invoke the matching live query or the device-backed pre-create route under
/// acquisition trust and synchronous output/device non-retention. Live calls also
/// require the acquired GPU-state compatibility assumptions.
pub(crate) unsafe fn resource_requirements(
    eligibility: Eligibility,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<ResourceRequirements, Error> {
    eligibility.check()?;
    let mut desc = ffxQueryDescUpscaleGetResourceRequirements {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS),
        required_resources: 0,
        optional_resources: 0,
    };
    check(invoke(&mut desc.header))?;
    Ok(ResourceRequirements {
        required: ResourceFlags(desc.required_resources),
        optional: ResourceFlags(desc.optional_resources),
    })
}

/// # Safety
/// Invoke the matching live-context query synchronously under D018, retaining
/// context/runtime and preserving the acquired GPU-state compatibility assumptions.
pub(crate) unsafe fn gpu_memory_usage(
    eligibility: Eligibility,
    mut invoke: impl FnMut(*mut ffxQueryDescHeader) -> u32,
) -> Result<GpuMemoryUsage, Error> {
    eligibility.check()?;
    let mut output = FfxApiEffectMemoryUsage::default();
    let mut desc = ffxQueryDescUpscaleGetGPUMemoryUsage {
        header: header(FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE),
        gpuMemoryUsageUpscaler: &mut output,
    };
    check(invoke(&mut desc.header))?;
    Ok(memory(output))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
