// SPDX-License-Identifier: MPL-2.0

//! Ordered frame inspection before native recording.

mod conversion;
mod formats;
mod inspection;

use super::{
    UpscaleInputs, UpscaleResources, UpscalerCore,
    descriptor::{DispatchGeometry, build_descriptor},
};
use crate::context::RuntimeRetention;

#[cfg(test)]
pub(super) use conversion::convert_desc;
#[cfg(test)]
pub(in crate::upscaler) use conversion::resource_for_parity;
pub(super) use conversion::{check_extents, convert};
use inspection::inspect_resource_identities;
pub(super) use inspection::{
    check_identity, inspect_device_membership, inspect_format_support, inspection,
};

use crate::error::{DispatchValidation, Error, ResourceProperty, ResourceRole};
use crate::upscaler::common::{
    ExposureMode, MotionVectorResolution, MotionVectorScale, UpscalerOptions,
    check_mask_requirements,
};
use fsr_sdk_sys::api::FfxApiResource;
use fsr_sdk_sys::upscale::ffxDispatchDescUpscale;
use windows::Win32::Graphics::Direct3D12::{D3D12_COMMAND_LIST_TYPE_DIRECT, ID3D12Device};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_R8_UNORM, DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT,
    DXGI_FORMAT_R32_FLOAT,
};
use windows::core::Interface;

pub(super) fn invalid_resource(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
    ))
}

pub(super) struct Converted {
    pub(super) native: FfxApiResource,
    pub(super) size: (u32, u32),
    allocation_format: DXGI_FORMAT,
    depth_stencil: bool,
}

pub(super) struct ConvertedResources {
    pub(super) color: Converted,
    pub(super) depth: Converted,
    pub(super) motion: Converted,
    pub(super) exposure: Option<Converted>,
    pub(super) reactive: Option<Converted>,
    pub(super) composition: Option<Converted>,
    pub(super) output: Converted,
}

fn inspect_resource_descriptions(
    resources: &UpscaleResources<'_>,
    options: &UpscalerOptions,
) -> Result<ConvertedResources, Error> {
    let exposure_resource = resources.exposure().texture();
    let color = convert(
        resources.color(),
        ResourceRole::Color,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        false,
    )?;
    let depth = convert(
        resources.depth(),
        ResourceRole::Depth,
        DXGI_FORMAT_R32_FLOAT,
        false,
    )?;
    let motion = convert(
        resources.motion_vectors(),
        ResourceRole::MotionVectors,
        DXGI_FORMAT_R16G16_FLOAT,
        false,
    )?;
    let exposure = exposure_resource
        .map(|resource| {
            convert(
                resource,
                ResourceRole::Exposure,
                DXGI_FORMAT_R32_FLOAT,
                false,
            )
        })
        .transpose()?;
    let reactive = resources
        .reactive_mask()
        .map(|r| convert(r, ResourceRole::ReactiveMask, DXGI_FORMAT_R8_UNORM, false))
        .transpose()?;
    let composition = resources
        .transparency_and_composition_mask()
        .map(|r| {
            convert(
                r,
                ResourceRole::TransparencyAndCompositionMask,
                DXGI_FORMAT_R8_UNORM,
                false,
            )
        })
        .transpose()?;
    let output = convert(
        resources.output(),
        ResourceRole::Output,
        DXGI_FORMAT_R16G16B16A16_FLOAT,
        true,
    )?;
    let render_size = (
        options.max_render_size().width(),
        options.max_render_size().height(),
    );
    let output_size = (
        options.max_upscale_size().width(),
        options.max_upscale_size().height(),
    );
    check_extents(
        [
            Some((ResourceRole::Color, color.size)),
            Some((ResourceRole::Depth, depth.size)),
            Some((ResourceRole::MotionVectors, motion.size)),
            exposure
                .as_ref()
                .map(|resource| (ResourceRole::Exposure, resource.size)),
            Some((ResourceRole::Output, output.size)),
            reactive
                .as_ref()
                .map(|r| (ResourceRole::ReactiveMask, r.size)),
            composition
                .as_ref()
                .map(|r| (ResourceRole::TransparencyAndCompositionMask, r.size)),
        ]
        .into_iter()
        .flatten(),
        render_size,
        output_size,
        match options.motion_vector_resolution() {
            MotionVectorResolution::Render => render_size,
            MotionVectorResolution::Display => output_size,
        },
    )?;

    Ok(ConvertedResources {
        color,
        depth,
        motion,
        exposure,
        reactive,
        composition,
        output,
    })
}

impl<R: RuntimeRetention> UpscalerCore<R> {
    pub(super) fn inspect_dispatch(
        &self,
        dispatch: &UpscaleInputs<'_>,
    ) -> Result<ffxDispatchDescUpscale, Error> {
        check_mask_requirements(
            self.resource_requirements,
            dispatch.resources().reactive_mask().is_some(),
            dispatch
                .resources()
                .transparency_and_composition_mask()
                .is_some(),
        )?;
        inspect_dispatch_borrowed(self.owner.device(), &self.options, dispatch)
    }
}

#[cfg(test)]
pub(in crate::upscaler) fn inspect_dispatch(
    device: &ID3D12Device,
    options: &UpscalerOptions,
    dispatch: crate::upscaler::dx12::UpscaleTestInputs<'_>,
) -> Result<ffxDispatchDescUpscale, Error> {
    inspect_dispatch_borrowed(
        device,
        options,
        &UpscaleInputs::new(
            dispatch.command_list,
            &dispatch.resources,
            &dispatch.parameters,
        ),
    )
}

fn inspect_dispatch_borrowed(
    device: &ID3D12Device,
    options: &UpscalerOptions,
    dispatch: &UpscaleInputs<'_>,
) -> Result<ffxDispatchDescUpscale, Error> {
    if let MotionVectorScale::Native(scale) = dispatch.motion_vector_scale() {
        scale.validate()?;
    }
    let active_render = options.check_active_render_size(dispatch.active_render_size())?;
    let active_output = options.check_active_output_size(dispatch.active_output_size())?;
    options.check_color_encoding_hint(
        dispatch.color_encoding().hint(),
        dispatch.color_encoding().allows_context_mismatch(),
    )?;
    if dispatch.camera().has_infinite_far_plane() != options.depth_convention().is_infinite() {
        return Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::CameraDepthConvention,
        )));
    }
    let list = dispatch.command_list();
    let resources = dispatch.resources();
    let exposure_resource = resources.exposure().texture();
    if exposure_resource.is_some() != (options.exposure_mode() == ExposureMode::Explicit) {
        return Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::ExposureMode,
        )));
    }
    // SAFETY: GetType inspects the live command-list COM object.
    if unsafe { list.GetType() } != D3D12_COMMAND_LIST_TYPE_DIRECT {
        return Err(Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::CommandListType,
        )));
    }
    let objects = [
        (ResourceRole::Color, Some(resources.color())),
        (ResourceRole::Depth, Some(resources.depth())),
        (
            ResourceRole::MotionVectors,
            Some(resources.motion_vectors()),
        ),
        (ResourceRole::Exposure, exposure_resource),
        (ResourceRole::Output, Some(resources.output())),
        (ResourceRole::ReactiveMask, resources.reactive_mask()),
        (
            ResourceRole::TransparencyAndCompositionMask,
            resources.transparency_and_composition_mask(),
        ),
    ];
    // Keep both groups of COM guards alive until descriptor construction ends,
    // with the same acquisition and release order as the individual checks.
    let _resource_identities = inspect_resource_identities(&objects)?;
    let converted = inspect_resource_descriptions(resources, options)?;
    let _device_identities = inspect_device_membership(device, list, &objects)?;
    // Fixed diagnostic order, shared by preflight and recording. Baseline pairs
    // make no new query; RG32F retains its existing error representation.
    for (role, resource) in [
        (ResourceRole::Color, &converted.color),
        (ResourceRole::Depth, &converted.depth),
        (ResourceRole::MotionVectors, &converted.motion),
        (ResourceRole::Output, &converted.output),
    ]
    .into_iter()
    .chain(
        [
            (ResourceRole::Exposure, converted.exposure.as_ref()),
            (ResourceRole::ReactiveMask, converted.reactive.as_ref()),
            (
                ResourceRole::TransparencyAndCompositionMask,
                converted.composition.as_ref(),
            ),
        ]
        .into_iter()
        .filter_map(|(role, resource)| resource.map(|r| (role, r))),
    ) {
        inspect_format_support(device, role, resource)?;
    }

    Ok(build_descriptor(
        list.as_raw(),
        converted,
        dispatch.parameters,
        DispatchGeometry {
            render: active_render,
            output: active_output,
            motion: match options.motion_vector_resolution() {
                MotionVectorResolution::Render => active_render,
                MotionVectorResolution::Display => active_output,
            },
        },
    ))
}

#[cfg(test)]
#[test]
fn inspection_retains_original_windows_error_through_portable_sources() {
    use crate::error::{InspectionObject, InspectionOperation};
    use std::error::Error as _;
    let code = 0x80004005u32 as i32;
    let original = windows::core::Error::from_hresult(windows::core::HRESULT(code));
    let error = inspection(
        InspectionObject::Resource(ResourceRole::Output),
        InspectionOperation::GetDevice,
        original,
    );
    let backend = error
        .source()
        .unwrap()
        .downcast_ref::<crate::error::BackendError>()
        .unwrap();
    let inspected = backend
        .source()
        .unwrap()
        .downcast_ref::<crate::error::Dx12InspectionError>()
        .unwrap();
    assert_eq!(inspected.hresult(), code);
    assert_eq!(
        inspected.object(),
        InspectionObject::Resource(ResourceRole::Output)
    );
    assert_eq!(inspected.operation(), InspectionOperation::GetDevice);
    let source = inspected
        .source()
        .unwrap()
        .downcast_ref::<windows::core::Error>()
        .unwrap();
    assert_eq!(source.code().0, code);
    assert_eq!(
        error.to_string(),
        "DX12 GetDevice of Resource(Output) failed with HRESULT 0x80004005"
    );
}
