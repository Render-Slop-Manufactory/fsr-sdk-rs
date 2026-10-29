// SPDX-License-Identifier: MPL-2.0

use super::{
    descriptor::build_descriptor,
    resources::{check_extents, check_identity, convert_desc, invalid_resource},
    *,
};
use crate::error::{ResourceProperty, ResourceRole};
use fsr_sdk_sys::{api::*, upscale::FFX_API_DISPATCH_DESC_TYPE_UPSCALE};
use std::{ffi::c_void, ptr::null_mut};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

use windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_FLAG_NONE;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;

fn desc(format: DXGI_FORMAT, width: u64, height: u32, output: bool) -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Width: width,
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Flags: if output {
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
        } else {
            D3D12_RESOURCE_FLAG_NONE
        },
        ..Default::default()
    }
}

#[test]
fn narrow_conversion_and_descriptor() {
    let make = |pointer, format, width, height, output| {
        convert_desc(
            pointer as *mut c_void,
            desc(format, width, height, output),
            ResourceRole::Color,
            format,
            output,
        )
        .unwrap()
    };
    let color = make(1, DXGI_FORMAT_R16G16B16A16_FLOAT, 320, 180, false);
    let depth = make(2, DXGI_FORMAT_R32_FLOAT, 320, 180, false);
    let motion = make(3, DXGI_FORMAT_R16G16_FLOAT, 320, 180, false);
    let exposure = make(4, DXGI_FORMAT_R32_FLOAT, 1, 1, false);
    let output = make(5, DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, true);
    let result = build_descriptor(
        6usize as *mut c_void,
        super::resources::ConvertedResources {
            color,
            depth,
            motion,
            output,
            exposure: Some(exposure),
            reactive: None,
            composition: None,
        },
        &FrameParameters::new(
            JitterOffsetPixels::new(0.25, -0.25).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(),
            crate::upscaler::HistoryAction::Reset,
        ),
        super::descriptor::DispatchGeometry {
            render: crate::upscaler::Dimensions::new(320, 180).unwrap(),
            output: crate::upscaler::Dimensions::new(640, 360).unwrap(),
            motion: crate::upscaler::Dimensions::new(320, 180).unwrap(),
        },
    );
    assert_eq!(result.header.r#type, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
    assert!(result.header.pNext.is_null());
    assert_eq!(result.commandList as usize, 6);
    for (actual, expected) in [
        (result.color, 1),
        (result.depth, 2),
        (result.motionVectors, 3),
        (result.exposure, 4),
        (result.output, 5),
    ] {
        assert_eq!(actual.resource as usize, expected);
    }
    assert!(result.reactive.resource.is_null());
    assert!(result.transparencyAndComposition.resource.is_null());
    assert_eq!(
        result.color.description.format,
        FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT
    );
    assert_eq!(
        result.depth.description.format,
        FFX_API_SURFACE_FORMAT_R32_FLOAT
    );
    assert_eq!(
        result.motionVectors.description.format,
        FFX_API_SURFACE_FORMAT_R16G16_FLOAT
    );
    assert_eq!(result.output.description.usage, FFX_API_RESOURCE_USAGE_UAV);
    assert_eq!(result.output.state, FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
    assert_eq!(result.color.state, FFX_API_RESOURCE_STATE_COMPUTE_READ);
    assert_eq!(
        (result.renderSize.width, result.renderSize.height),
        (320, 180)
    );
    assert_eq!(
        (result.upscaleSize.width, result.upscaleSize.height),
        (640, 360)
    );
    assert_eq!(
        (result.motionVectorScale.x, result.motionVectorScale.y),
        (320.0, 180.0)
    );
    assert_eq!(
        (result.jitterOffset.x, result.jitterOffset.y),
        (0.25, -0.25)
    );
    assert_eq!(result.frameTimeDelta, 16.667);
    assert_eq!(result.preExposure, 1.0);
    assert!(result.reset);
    assert!(!result.enableSharpening);
    assert_eq!(result.sharpness, 0.0);
    assert_eq!(result.flags, 0);
}

#[test]
fn rejects_invalid_resources_and_frame_before_native_call() {
    let valid = desc(DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, true);
    let no_uav = desc(DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, false);
    let check = |resource_desc| {
        convert_desc(
            null_mut(),
            resource_desc,
            ResourceRole::Output,
            DXGI_FORMAT_R16G16B16A16_FLOAT,
            true,
        )
        .err()
        .unwrap()
    };
    assert_error!(
        check(no_uav),
        invalid_resource(ResourceRole::Output, ResourceProperty::UnorderedAccess)
    );
    let mut multisampled = valid;
    multisampled.SampleDesc.Count = 2;
    assert_error!(
        check(multisampled),
        invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
    );
    let mut array = valid;
    array.DepthOrArraySize = 2;
    assert_error!(
        check(array),
        invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
    );
    let mut mips = valid;
    mips.MipLevels = 2;
    assert_error!(
        check(mips),
        invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
    );
    let mut wrong_format = valid;
    wrong_format.Format = DXGI_FORMAT_R32_FLOAT;
    assert_error!(
        check(wrong_format),
        invalid_resource(ResourceRole::Output, ResourceProperty::Format)
    );
    let mut oversized = valid;
    oversized.Width = u64::from(u32::MAX) + 1;
    assert_error!(
        check(oversized),
        invalid_resource(ResourceRole::Output, ResourceProperty::Extent)
    );
    let mut zero = valid;
    zero.Height = 0;
    assert_error!(
        check(zero),
        invalid_resource(ResourceRole::Output, ResourceProperty::Extent)
    );
    let mut denied = desc(DXGI_FORMAT_R32_FLOAT, 320, 180, false);
    denied.Flags = D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
    assert_error!(
        convert_desc(
            null_mut(),
            denied,
            ResourceRole::Depth,
            DXGI_FORMAT_R32_FLOAT,
            false
        )
        .err()
        .unwrap(),
        invalid_resource(ResourceRole::Depth, ResourceProperty::ShaderReadable)
    );
}

#[test]
fn exact_extents_and_canonical_identity_reject_before_native_call() {
    let roles = [
        ResourceRole::Color,
        ResourceRole::Depth,
        ResourceRole::MotionVectors,
        ResourceRole::Exposure,
        ResourceRole::Output,
    ];
    let valid = [(320, 180), (320, 180), (320, 180), (1, 1), (640, 360)];
    let inputs = |sizes: [(u32, u32); 5]| -> [(ResourceRole, (u32, u32)); 5] {
        std::array::from_fn(|index| (roles[index], sizes[index]))
    };
    assert_result!(
        check_extents(inputs(valid), (320, 180), (640, 360), (320, 180)),
        Ok(())
    );
    for index in 0..5 {
        let mut sizes = valid;
        sizes[index].0 -= 1;
        assert_result!(
            check_extents(inputs(sizes), (320, 180), (640, 360), (320, 180)),
            Err(invalid_resource(roles[index], ResourceProperty::Extent))
        );
    }
    let mut first = 0u8;
    let mut second = 0u8;
    let pointers = [
        (&raw mut first).cast::<c_void>(),
        (&raw mut second).cast::<c_void>(),
    ];
    assert_result!(
        check_identity(ResourceRole::Depth, pointers[1], &pointers[..1]),
        Ok(())
    );
    assert_result!(
        check_identity(ResourceRole::Output, pointers[0], &pointers),
        Err(invalid_resource(
            ResourceRole::Output,
            ResourceProperty::DistinctIdentity
        ))
    );
}
