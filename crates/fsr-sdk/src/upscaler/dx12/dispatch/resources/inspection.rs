// SPDX-License-Identifier: MPL-2.0

//! COM identities, device membership and format capability queries.

use super::{
    Converted,
    formats::{depth_views, fixed_view},
    invalid_resource,
};

use crate::error::{
    DispatchValidation, Error, InspectionObject, InspectionOperation, ResourceProperty,
    ResourceRole,
};
use fsr_sdk_sys::api::{
    FFX_API_RESOURCE_USAGE_UAV, FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM,
    FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM, FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM,
    FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT, FFX_API_SURFACE_FORMAT_R16_FLOAT,
    FFX_API_SURFACE_FORMAT_R32G32_FLOAT, FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT,
};
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_FEATURE_D3D12_OPTIONS3, D3D12_FEATURE_DATA_D3D12_OPTIONS3,
    D3D12_FEATURE_DATA_FORMAT_SUPPORT, D3D12_FEATURE_FORMAT_SUPPORT,
    D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL, D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
    D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE, D3D12_FORMAT_SUPPORT1_TEXTURE2D,
    D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW, D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE,
    ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R10G10B10A2_UNORM,
    DXGI_FORMAT_R11G11B10_FLOAT, DXGI_FORMAT_R16_FLOAT, DXGI_FORMAT_R32G32_FLOAT,
    DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT,
};
use windows::core::{IUnknown, Interface};

pub(in super::super) fn inspection(
    object: InspectionObject,
    operation: InspectionOperation,
    error: windows::core::Error,
) -> Error {
    Error::Backend(crate::error::BackendError::Dx12Inspection(
        crate::error::Dx12InspectionError::new(object, operation, error.code().0, Box::new(error)),
    ))
}

pub(in super::super) fn check_identity(
    role: ResourceRole,
    identity: *mut c_void,
    prior: &[*mut c_void],
) -> Result<(), Error> {
    if prior.contains(&identity) {
        Err(invalid_resource(role, ResourceProperty::DistinctIdentity))
    } else {
        Ok(())
    }
}

type ResourceObjects<'a> = [(ResourceRole, Option<&'a ID3D12Resource>); 7];

// The caller retains these canonical references through the remaining inspection.
pub(super) fn inspect_resource_identities(
    objects: &ResourceObjects<'_>,
) -> Result<[Option<IUnknown>; 7], Error> {
    let mut identities: [Option<IUnknown>; 7] = std::array::from_fn(|_| None);
    let mut identity_pointers = [std::ptr::null_mut(); 7];
    for (index, (role, object)) in objects.iter().copied().enumerate() {
        let Some(object) = object else { continue };
        let identity: IUnknown = object.cast().map_err(|error| {
            inspection(
                InspectionObject::Resource(role),
                InspectionOperation::QueryIdentity,
                error,
            )
        })?;
        check_identity(role, identity.as_raw(), &identity_pointers[..index])?;
        identity_pointers[index] = identity.as_raw();
        identities[index] = Some(identity);
    }
    Ok(identities)
}

pub(in super::super) fn inspect_device_membership(
    device: &ID3D12Device,
    list: &ID3D12GraphicsCommandList,
    objects: &[(ResourceRole, Option<&ID3D12Resource>)],
) -> Result<(IUnknown, IUnknown), Error> {
    let context_device: IUnknown = device.cast().map_err(|error| {
        inspection(
            InspectionObject::ContextDevice,
            InspectionOperation::QueryIdentity,
            error,
        )
    })?;
    let mut list_device: Option<ID3D12Device> = None;
    // SAFETY: GetDevice writes a fresh COM reference into this output.
    unsafe { list.GetDevice(&mut list_device) }.map_err(|error| {
        inspection(
            InspectionObject::CommandList,
            InspectionOperation::GetDevice,
            error,
        )
    })?;
    let list_identity: IUnknown = list_device
        .ok_or(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::CommandListDevice,
        )))?
        .cast()
        .map_err(|error| {
            inspection(
                InspectionObject::CommandList,
                InspectionOperation::QueryIdentity,
                error,
            )
        })?;
    if list_identity.as_raw() != context_device.as_raw() {
        return Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::CommandListDevice,
        )));
    }
    for (role, object) in objects.iter().copied() {
        let Some(object) = object else { continue };
        let mut device: Option<ID3D12Device> = None;
        // SAFETY: GetDevice writes a fresh COM reference into this output.
        unsafe { object.GetDevice(&mut device) }.map_err(|error| {
            inspection(
                InspectionObject::Resource(role),
                InspectionOperation::GetDevice,
                error,
            )
        })?;
        let identity: IUnknown = device
            .ok_or_else(|| invalid_resource(role, ResourceProperty::Device))?
            .cast()
            .map_err(|error| {
                inspection(
                    InspectionObject::Resource(role),
                    InspectionOperation::QueryIdentity,
                    error,
                )
            })?;
        if identity.as_raw() != context_device.as_raw() {
            return Err(invalid_resource(role, ResourceProperty::Device));
        }
    }

    // Preserve the original release order: list identity, then context device.
    Ok((list_identity, context_device))
}

pub(in super::super) fn inspect_format_support(
    device: &ID3D12Device,
    role: ResourceRole,
    converted: &Converted,
) -> Result<(), Error> {
    let resource = &converted.native;
    if let Some(uav) = match converted.allocation_format {
        DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => Some(DXGI_FORMAT_R8G8B8A8_UNORM),
        DXGI_FORMAT_B8G8R8A8_UNORM_SRGB => Some(DXGI_FORMAT_B8G8R8A8_UNORM),
        _ => None,
    } {
        // The backend retains the sRGB SRV but creates a same-family UNORM UAV.
        // Inspect the actual views separately, including on UAV-capable inputs.
        let object = InspectionObject::ContextDeviceForResource(role);
        let has_uav = resource.description.usage & FFX_API_RESOURCE_USAGE_UAV != 0;
        if has_uav {
            inspect_typed_casting(device, role)?;
        }
        let output = role == ResourceRole::Output;
        let support = query_format_support(device, object, converted.allocation_format)?;
        let mut required = D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_SHADER_LOAD;
        if !output {
            required |= D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE;
        }
        if !support.Support1.contains(required) {
            return Err(invalid_resource(role, ResourceProperty::FormatSupport));
        }
        if has_uav {
            let support = query_format_support(device, object, uav)?;
            if !support.Support1.contains(
                D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
            ) || output
                && !support
                    .Support2
                    .contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE)
            {
                return Err(invalid_resource(role, ResourceProperty::FormatSupport));
            }
        }
        return Ok(());
    }
    if let Some((dsv, srv, typed)) =
        depth_views(converted.allocation_format).filter(|_| converted.depth_stencil)
    {
        let object = InspectionObject::ContextDeviceForResource(role);
        if typed {
            inspect_typed_casting(device, role)?;
        }
        for (view, required) in [
            (
                dsv,
                D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL,
            ),
            (
                srv,
                D3D12_FORMAT_SUPPORT1_TEXTURE2D
                    | D3D12_FORMAT_SUPPORT1_SHADER_LOAD
                    | D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
            ),
        ] {
            let support = query_format_support(device, object, view)?;
            if !support.Support1.contains(required) {
                return Err(invalid_resource(role, ResourceProperty::FormatSupport));
            }
        }
        return Ok(());
    }
    if let Some(view) = fixed_view(converted.allocation_format) {
        let support = query_format_support(
            device,
            InspectionObject::ContextDeviceForResource(role),
            view,
        )?;
        let output = matches!(
            role,
            ResourceRole::Output | ResourceRole::ReactiveMaskOutput
        );
        let mut required = D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_SHADER_LOAD;
        // RGB32F supports loads but not filtering; the caller's selected provider
        // must accept that operation-specific input, as with every format.
        if !output && view != DXGI_FORMAT_R32G32B32_FLOAT {
            required |= D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE;
        }
        if resource.description.usage & FFX_API_RESOURCE_USAGE_UAV != 0 {
            required |= D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW;
        }
        if !support.Support1.contains(required)
            || output
                && !support
                    .Support2
                    .contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE)
        {
            return Err(invalid_resource(role, ResourceProperty::FormatSupport));
        }
        return Ok(());
    }
    let (format, object) = match resource.description.format {
        FFX_API_SURFACE_FORMAT_R32G32_FLOAT if role == ResourceRole::MotionVectors => {
            (DXGI_FORMAT_R32G32_FLOAT, InspectionObject::ContextDevice)
        }
        FFX_API_SURFACE_FORMAT_R32G32B32A32_FLOAT => (
            DXGI_FORMAT_R32G32B32A32_FLOAT,
            InspectionObject::ContextDeviceForResource(role),
        ),
        FFX_API_SURFACE_FORMAT_R11G11B10_FLOAT => (
            DXGI_FORMAT_R11G11B10_FLOAT,
            InspectionObject::ContextDeviceForResource(role),
        ),
        FFX_API_SURFACE_FORMAT_R10G10B10A2_UNORM => (
            DXGI_FORMAT_R10G10B10A2_UNORM,
            InspectionObject::ContextDeviceForResource(role),
        ),
        FFX_API_SURFACE_FORMAT_R8G8B8A8_UNORM => (
            DXGI_FORMAT_R8G8B8A8_UNORM,
            InspectionObject::ContextDeviceForResource(role),
        ),
        FFX_API_SURFACE_FORMAT_B8G8R8A8_UNORM => (
            DXGI_FORMAT_B8G8R8A8_UNORM,
            InspectionObject::ContextDeviceForResource(role),
        ),
        FFX_API_SURFACE_FORMAT_R16_FLOAT => (
            DXGI_FORMAT_R16_FLOAT,
            InspectionObject::ContextDeviceForResource(role),
        ),
        _ => return Ok(()),
    };
    let support = query_format_support(device, object, format)?;
    let output = role == ResourceRole::Output;
    let mut required = D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_SHADER_LOAD;
    if !output {
        required |= D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE;
    }
    if resource.description.usage & FFX_API_RESOURCE_USAGE_UAV != 0 {
        // Registration also creates this view for UAV-capable read-only inputs.
        // Those need no typed UAV access bits; output stores are checked below.
        required |= D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW;
    }
    if !support.Support1.contains(required)
        || (output
            && !support
                .Support2
                .contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE))
    {
        return Err(invalid_resource(role, ResourceProperty::FormatSupport));
    }
    Ok(())
}

fn inspect_typed_casting(device: &ID3D12Device, role: ResourceRole) -> Result<(), Error> {
    let mut options = D3D12_FEATURE_DATA_D3D12_OPTIONS3::default();
    // SAFETY: Live canonical device and the exact writable feature struct.
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_D3D12_OPTIONS3,
            std::ptr::from_mut(&mut options).cast(),
            std::mem::size_of_val(&options) as u32,
        )
    }
    .map_err(|error| {
        inspection(
            InspectionObject::ContextDeviceForResource(role),
            InspectionOperation::CheckFormatCasting,
            error,
        )
    })?;
    if !options.CastingFullyTypedFormatSupported.as_bool() {
        return Err(invalid_resource(role, ResourceProperty::FormatSupport));
    }
    Ok(())
}

fn query_format_support(
    device: &ID3D12Device,
    object: InspectionObject,
    format: DXGI_FORMAT,
) -> Result<D3D12_FEATURE_DATA_FORMAT_SUPPORT, Error> {
    let mut support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: format,
        ..Default::default()
    };
    // SAFETY: Live canonical device and the exact writable feature struct.
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_FORMAT_SUPPORT,
            std::ptr::from_mut(&mut support).cast(),
            std::mem::size_of_val(&support) as u32,
        )
    }
    .map_err(|error| inspection(object, InspectionOperation::CheckFormatSupport, error))?;
    Ok(support)
}
