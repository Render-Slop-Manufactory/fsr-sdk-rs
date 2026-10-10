// SPDX-License-Identifier: MPL-2.0

//! DX12 metadata, identity and device inspection before native recording.
use super::{UpscaleDispatch, UpscalerCore, descriptor::build_descriptor};
use crate::context::RuntimeRetention;
use crate::error::{
    DispatchValidation, Error, InspectionObject, InspectionOperation, ResourceProperty,
    ResourceRole,
};
use crate::upscaler::UpscalerOptions;
use fsr_sdk_sys::{
    api::{
        FFX_API_RESOURCE_FLAGS_NONE, FFX_API_RESOURCE_STATE_COMPUTE_READ,
        FFX_API_RESOURCE_STATE_UNORDERED_ACCESS, FFX_API_RESOURCE_TYPE_TEXTURE2D,
        FFX_API_RESOURCE_USAGE_READ_ONLY, FFX_API_RESOURCE_USAGE_UAV,
        FFX_API_SURFACE_FORMAT_R16G16_FLOAT, FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT,
        FFX_API_SURFACE_FORMAT_R32_FLOAT, FfxApiResource, FfxApiResourceDepth,
        FfxApiResourceDescription, FfxApiResourceHeight, FfxApiResourceWidth,
    },
    upscale::ffxDispatchDescUpscale,
};
use std::ffi::c_void;
use windows::{
    Win32::Graphics::{
        Direct3D12::{
            D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_RESOURCE_DESC,
            D3D12_RESOURCE_DIMENSION_TEXTURE2D, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, ID3D12Device, ID3D12Resource,
        },
        Dxgi::Common::{
            DXGI_FORMAT, DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R32_FLOAT,
        },
    },
    core::{IUnknown, Interface},
};

pub(super) fn invalid_resource(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::InvalidDispatch {
        reason: DispatchValidation::Resource { role, property },
    }
}

fn inspection(
    object: InspectionObject,
    operation: InspectionOperation,
    error: windows::core::Error,
) -> Error {
    Error::Dx12Inspection {
        object,
        operation,
        hresult: error.code().0,
    }
}

pub(super) fn check_identity(
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

pub(super) fn check_extents(
    resources: [(ResourceRole, (u32, u32)); 5],
    render_size: (u32, u32),
    output_size: (u32, u32),
) -> Result<(), Error> {
    for ((role, actual), expected) in
        resources
            .into_iter()
            .zip([render_size, render_size, render_size, (1, 1), output_size])
    {
        if actual != expected {
            return Err(invalid_resource(role, ResourceProperty::Extent));
        }
    }
    Ok(())
}

pub(super) struct Converted {
    pub(super) native: FfxApiResource,
    pub(super) size: (u32, u32),
}

fn format(format: DXGI_FORMAT) -> Option<u32> {
    match format {
        DXGI_FORMAT_R16G16B16A16_FLOAT => Some(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT),
        DXGI_FORMAT_R16G16_FLOAT => Some(FFX_API_SURFACE_FORMAT_R16G16_FLOAT),
        DXGI_FORMAT_R32_FLOAT => Some(FFX_API_SURFACE_FORMAT_R32_FLOAT),
        _ => None,
    }
}

pub(super) fn convert_desc(
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
    if desc.Format != expected_format {
        return Err(invalid_resource(role, ResourceProperty::Format));
    }
    let width =
        u32::try_from(desc.Width).map_err(|_| invalid_resource(role, ResourceProperty::Extent))?;
    if width == 0 || desc.Height == 0 {
        return Err(invalid_resource(role, ResourceProperty::Extent));
    }
    if !output && (desc.Flags.0 & D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE.0) != 0 {
        return Err(invalid_resource(role, ResourceProperty::ShaderReadable));
    }
    let has_uav = (desc.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0) != 0;
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
            usage: FFX_API_RESOURCE_USAGE_READ_ONLY
                | if has_uav {
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
    })
}

fn convert(
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

impl<R: RuntimeRetention> UpscalerCore<R> {
    pub(super) fn inspect_dispatch(
        &self,
        dispatch: UpscaleDispatch<'_>,
    ) -> Result<ffxDispatchDescUpscale, Error> {
        inspect_dispatch(self.owner.device(), &self.options, dispatch)
    }
}

pub(in crate::upscaler) fn inspect_dispatch(
    device: &ID3D12Device,
    options: &UpscalerOptions,
    dispatch: UpscaleDispatch<'_>,
) -> Result<ffxDispatchDescUpscale, Error> {
    let list = dispatch.command_list;
    // SAFETY: GetType inspects the live command-list COM object.
    if unsafe { list.GetType() } != D3D12_COMMAND_LIST_TYPE_DIRECT {
        return Err(Error::InvalidDispatch {
            reason: DispatchValidation::CommandListType,
        });
    }
    let objects = [
        (ResourceRole::Color, dispatch.color),
        (ResourceRole::Depth, dispatch.depth),
        (ResourceRole::MotionVectors, dispatch.motion_vectors),
        (ResourceRole::Exposure, dispatch.exposure),
        (ResourceRole::Output, dispatch.output),
    ];
    let mut identities: [Option<IUnknown>; 5] = std::array::from_fn(|_| None);
    let mut identity_pointers = [std::ptr::null_mut(); 5];
    for (index, (role, object)) in objects.into_iter().enumerate() {
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
    let color = convert(
        dispatch.color,
        ResourceRole::Color,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        false,
    )?;
    let depth = convert(
        dispatch.depth,
        ResourceRole::Depth,
        DXGI_FORMAT_R32_FLOAT,
        false,
    )?;
    let motion = convert(
        dispatch.motion_vectors,
        ResourceRole::MotionVectors,
        DXGI_FORMAT_R16G16_FLOAT,
        false,
    )?;
    let exposure = convert(
        dispatch.exposure,
        ResourceRole::Exposure,
        DXGI_FORMAT_R32_FLOAT,
        false,
    )?;
    let output = convert(
        dispatch.output,
        ResourceRole::Output,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        true,
    )?;
    let render_size = (
        options.max_render_size.width.get(),
        options.max_render_size.height.get(),
    );
    let output_size = (
        options.max_upscale_size.width.get(),
        options.max_upscale_size.height.get(),
    );
    check_extents(
        [
            (ResourceRole::Color, color.size),
            (ResourceRole::Depth, depth.size),
            (ResourceRole::MotionVectors, motion.size),
            (ResourceRole::Exposure, exposure.size),
            (ResourceRole::Output, output.size),
        ],
        render_size,
        output_size,
    )?;

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
        .ok_or(Error::InvalidDispatch {
            reason: DispatchValidation::CommandListDevice,
        })?
        .cast()
        .map_err(|error| {
            inspection(
                InspectionObject::CommandList,
                InspectionOperation::QueryIdentity,
                error,
            )
        })?;
    if list_identity.as_raw() != context_device.as_raw() {
        return Err(Error::InvalidDispatch {
            reason: DispatchValidation::CommandListDevice,
        });
    }
    for (role, object) in objects {
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

    Ok(build_descriptor(
        list.as_raw(),
        [color, depth, motion, exposure, output],
        (
            dispatch.jitter,
            dispatch.frame_time,
            dispatch.camera,
            dispatch.reset,
        ),
    ))
}
