// SPDX-License-Identifier: MPL-2.0

//! Resource metadata validation and native descriptor conversion.

use super::{
    Converted,
    formats::{depth_views, format},
    invalid_resource,
};

use crate::error::{Error, ResourceProperty, ResourceRole};
use fsr_sdk_sys::api::{
    FFX_API_RESOURCE_FLAGS_NONE, FFX_API_RESOURCE_STATE_COMPUTE_READ,
    FFX_API_RESOURCE_STATE_UNORDERED_ACCESS, FFX_API_RESOURCE_TYPE_TEXTURE2D,
    FFX_API_RESOURCE_USAGE_DEPTHTARGET, FFX_API_RESOURCE_USAGE_READ_ONLY,
    FFX_API_RESOURCE_USAGE_STENCILTARGET, FFX_API_RESOURCE_USAGE_UAV, FfxApiResource,
    FfxApiResourceDepth, FfxApiResourceDescription, FfxApiResourceHeight, FfxApiResourceWidth,
};
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
    D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_TYPELESS, DXGI_FORMAT_B8G8R8A8_UNORM,
    DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, DXGI_FORMAT_D16_UNORM, DXGI_FORMAT_D24_UNORM_S8_UINT,
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_D32_FLOAT_S8X24_UINT, DXGI_FORMAT_R8_TYPELESS,
    DXGI_FORMAT_R8G8B8A8_SNORM, DXGI_FORMAT_R8G8B8A8_TYPELESS, DXGI_FORMAT_R8G8B8A8_UNORM,
    DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
    DXGI_FORMAT_R10G10B10A2_TYPELESS, DXGI_FORMAT_R10G10B10A2_UNORM, DXGI_FORMAT_R11G11B10_FLOAT,
    DXGI_FORMAT_R16_FLOAT, DXGI_FORMAT_R16_TYPELESS, DXGI_FORMAT_R16_UNORM,
    DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16_TYPELESS, DXGI_FORMAT_R16G16B16A16_FLOAT,
    DXGI_FORMAT_R16G16B16A16_TYPELESS, DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R32_TYPELESS,
    DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32G32_TYPELESS, DXGI_FORMAT_R32G32B32_FLOAT,
    DXGI_FORMAT_R32G32B32_TYPELESS, DXGI_FORMAT_R32G32B32A32_FLOAT,
    DXGI_FORMAT_R32G32B32A32_TYPELESS,
};
use windows::core::Interface;

pub(in super::super) fn check_extents(
    resources: impl IntoIterator<Item = (ResourceRole, (u32, u32))>,
    render_size: (u32, u32),
    output_size: (u32, u32),
    motion_size: (u32, u32),
) -> Result<(), Error> {
    for (role, actual) in resources {
        let expected = match role {
            ResourceRole::Color
            | ResourceRole::OpaqueOnlyColor
            | ResourceRole::ReactiveMaskOutput
            | ResourceRole::Depth
            | ResourceRole::ReactiveMask
            | ResourceRole::TransparencyAndCompositionMask => render_size,
            ResourceRole::MotionVectors => motion_size,
            ResourceRole::Exposure => (1, 1),
            ResourceRole::Output => output_size,
        };
        if actual != expected {
            return Err(invalid_resource(role, ResourceProperty::Extent));
        }
    }
    Ok(())
}

pub(in super::super) fn convert_desc(
    pointer: *mut c_void,
    desc: D3D12_RESOURCE_DESC,
    role: ResourceRole,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<Converted, Error> {
    if desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
        || desc.DepthOrArraySize != 1
        || desc.MipLevels != 1
        || desc.SampleDesc.Count != 1
    {
        return Err(invalid_resource(role, ResourceProperty::TextureShape));
    }
    // The operation supplies its baseline format; additions remain role-specific,
    // independently of the private converter's representation coverage.
    let additional = match role {
        ResourceRole::Color | ResourceRole::OpaqueOnlyColor | ResourceRole::Output
            if expected_format == DXGI_FORMAT_R16G16B16A16_FLOAT =>
        {
            matches!(
                desc.Format,
                DXGI_FORMAT_R32G32B32A32_FLOAT
                    | DXGI_FORMAT_R11G11B10_FLOAT
                    | DXGI_FORMAT_R10G10B10A2_UNORM
                    | DXGI_FORMAT_R8G8B8A8_UNORM
                    | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
                    | DXGI_FORMAT_B8G8R8A8_UNORM
                    | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB
                    | DXGI_FORMAT_R32G32B32A32_TYPELESS
                    | DXGI_FORMAT_R16G16B16A16_TYPELESS
                    | DXGI_FORMAT_R10G10B10A2_TYPELESS
                    | DXGI_FORMAT_R8G8B8A8_TYPELESS
                    | DXGI_FORMAT_B8G8R8A8_TYPELESS
                    | DXGI_FORMAT_R8G8B8A8_SNORM
                    | DXGI_FORMAT_R9G9B9E5_SHAREDEXP
            ) || !output
                && matches!(
                    desc.Format,
                    DXGI_FORMAT_R32G32B32_TYPELESS | DXGI_FORMAT_R32G32B32_FLOAT
                )
        }
        ResourceRole::Depth if expected_format == DXGI_FORMAT_R32_FLOAT => {
            matches!(
                desc.Format,
                DXGI_FORMAT_R16_FLOAT | DXGI_FORMAT_R16_TYPELESS | DXGI_FORMAT_R16_UNORM
            ) || depth_views(desc.Format).is_some()
        }
        ResourceRole::MotionVectors if expected_format == DXGI_FORMAT_R16G16_FLOAT => {
            matches!(
                desc.Format,
                DXGI_FORMAT_R32G32_FLOAT
                    | DXGI_FORMAT_R32G32_TYPELESS
                    | DXGI_FORMAT_R16G16_TYPELESS
            )
        }
        ResourceRole::Exposure => desc.Format == DXGI_FORMAT_R32_TYPELESS,
        ResourceRole::ReactiveMask
        | ResourceRole::TransparencyAndCompositionMask
        | ResourceRole::ReactiveMaskOutput => desc.Format == DXGI_FORMAT_R8_TYPELESS,
        _ => false,
    };
    if desc.Format != expected_format && !additional {
        return Err(invalid_resource(role, ResourceProperty::Format));
    }
    let width =
        u32::try_from(desc.Width).map_err(|_| invalid_resource(role, ResourceProperty::Extent))?;
    if width == 0 || desc.Height == 0 {
        return Err(invalid_resource(role, ResourceProperty::Extent));
    }
    if (!output || role == ResourceRole::ReactiveMaskOutput || additional)
        && (desc.Flags.0 & D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE.0) != 0
    {
        return Err(invalid_resource(role, ResourceProperty::ShaderReadable));
    }
    let has_uav = (desc.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0) != 0;
    let depth_stencil = role == ResourceRole::Depth
        && depth_views(desc.Format).is_some()
        && (desc.Format != DXGI_FORMAT_R32_TYPELESS
            || desc.Flags.contains(D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL));
    if !depth_stencil && additional && desc.Flags.contains(D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL)
    {
        return Err(invalid_resource(role, ResourceProperty::DepthStencil));
    }
    if has_uav
        && matches!(
            desc.Format,
            DXGI_FORMAT_R32G32B32_TYPELESS | DXGI_FORMAT_R32G32B32_FLOAT
        )
    {
        return Err(invalid_resource(role, ResourceProperty::UnorderedAccess));
    }
    if depth_stencil {
        if !desc.Flags.contains(D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL) {
            return Err(invalid_resource(role, ResourceProperty::DepthStencil));
        }
        if has_uav {
            return Err(invalid_resource(role, ResourceProperty::UnorderedAccess));
        }
    }
    if output && !has_uav {
        return Err(invalid_resource(role, ResourceProperty::UnorderedAccess));
    }
    let native = FfxApiResource {
        resource: pointer,
        description: FfxApiResourceDescription {
            r#type: FFX_API_RESOURCE_TYPE_TEXTURE2D,
            format: format(desc.Format)
                .ok_or_else(|| invalid_resource(role, ResourceProperty::Format))?,
            width: FfxApiResourceWidth { width },
            height: FfxApiResourceHeight {
                height: desc.Height,
            },
            depth: FfxApiResourceDepth {
                depth: u32::from(desc.DepthOrArraySize),
            },
            mipCount: u32::from(desc.MipLevels),
            flags: FFX_API_RESOURCE_FLAGS_NONE,
            // Match the pinned public helper: only typed depth allocations set
            // these bits; typeless allocations retain READ_ONLY (zero).
            usage: match desc.Format {
                DXGI_FORMAT_D32_FLOAT | DXGI_FORMAT_D16_UNORM => FFX_API_RESOURCE_USAGE_DEPTHTARGET,
                DXGI_FORMAT_D32_FLOAT_S8X24_UINT | DXGI_FORMAT_D24_UNORM_S8_UINT => {
                    FFX_API_RESOURCE_USAGE_DEPTHTARGET | FFX_API_RESOURCE_USAGE_STENCILTARGET
                }
                _ => FFX_API_RESOURCE_USAGE_READ_ONLY,
            } | if has_uav {
                FFX_API_RESOURCE_USAGE_UAV
            } else {
                0
            },
        },
        state: if output {
            FFX_API_RESOURCE_STATE_UNORDERED_ACCESS
        } else {
            FFX_API_RESOURCE_STATE_COMPUTE_READ
        },
    };
    Ok(Converted {
        native,
        size: (width, desc.Height),
        allocation_format: desc.Format,
        depth_stencil,
    })
}

pub(in super::super) fn convert(
    resource: &ID3D12Resource,
    role: ResourceRole,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<Converted, Error> {
    // SAFETY: A valid COM resource returns its immutable resource description.
    let desc = unsafe { resource.GetDesc() };
    convert_desc(resource.as_raw(), desc, role, expected_format, output)
}

#[cfg(test)]
pub(in crate::upscaler) fn resource_for_parity(
    resource: &ID3D12Resource,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<FfxApiResource, Error> {
    Ok(convert(resource, ResourceRole::Color, expected_format, output)?.native)
}
