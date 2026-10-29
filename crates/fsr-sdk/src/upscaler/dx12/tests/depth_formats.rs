// SPDX-License-Identifier: MPL-2.0

//! Depth allocation/view contracts through both public ownership families.
use super::{
    com::{GET_DEVICE_FAILURE, Scene},
    *,
};
use crate::error::{
    DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};

const DEPTH: [(DXGI_FORMAT, DXGI_FORMAT, DXGI_FORMAT, u32, u32, bool); 7] = [
    (
        DXGI_FORMAT_R32_TYPELESS,
        DXGI_FORMAT_D32_FLOAT,
        DXGI_FORMAT_R32_FLOAT,
        37,
        0,
        false,
    ),
    (
        DXGI_FORMAT_D32_FLOAT,
        DXGI_FORMAT_D32_FLOAT,
        DXGI_FORMAT_R32_FLOAT,
        28,
        4,
        true,
    ),
    (
        DXGI_FORMAT_R32G8X24_TYPELESS,
        DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
        DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS,
        28,
        0,
        false,
    ),
    (
        DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
        DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
        DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS,
        28,
        36,
        true,
    ),
    (
        DXGI_FORMAT_D16_UNORM,
        DXGI_FORMAT_D16_UNORM,
        DXGI_FORMAT_R16_UNORM,
        23,
        4,
        true,
    ),
    (
        DXGI_FORMAT_R24G8_TYPELESS,
        DXGI_FORMAT_D24_UNORM_S8_UINT,
        DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
        8,
        0,
        false,
    ),
    (
        DXGI_FORMAT_D24_UNORM_S8_UINT,
        DXGI_FORMAT_D24_UNORM_S8_UINT,
        DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
        8,
        36,
        true,
    ),
];
fn invalid(property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource {
            role: ResourceRole::Depth,
            property,
        },
    ))
}
macro_rules! exercise {
    ($runtime:expr, $owner:ty) => {{
        let (mut runtime, observer) = $runtime;
        let calls = unsafe {
            *observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()
        };
        let scene = Scene::new();
        let device = scene.device_object();
        let object = scene.resource_object(1);
        let baseline = object.desc.get();
        let refs = scene.reference_counts();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let support = device.format_support.get() | D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL;
        device.format_support.set(support);
        for (allocation, dsv, srv, native, usage, typed) in DEPTH {
            let mut desc = baseline;
            desc.Format = allocation;
            desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
            object.desc.set(desc);
            device.format_queries.borrow_mut().clear();
            let casts = device.casting_query_calls.get();
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Ok(())
            );
            assert_result!(
                unsafe {
                    let recording = scene.input();
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Ok(())
            );
            assert_eq!(&*device.format_queries.borrow(), &[dsv, srv, dsv, srv]);
            assert_eq!(
                device.casting_query_calls.get() - casts,
                if typed { 2 } else { 0 }
            );
            let converted =
                super::super::dispatch::inspect_dispatch(&scene.device, &sizes(), scene.input())
                    .unwrap()
                    .depth;
            assert_eq!(
                (
                    converted.description.format,
                    converted.description.usage,
                    converted.state
                ),
                (native, usage, 4)
            );
            let before = calls().calls;
            for bit in [
                D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL,
                D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
            ] {
                device.format_support.set(support & !bit);
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(ResourceProperty::FormatSupport))
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(ResourceProperty::FormatSupport))
                );
            }
            device.format_support.set(support);
            for view in [dsv, srv] {
                device.format_fail_at.set(Some(view));
                device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
                let expected = Err(crate::error::test_inspection(
                    InspectionObject::ContextDeviceForResource(ResourceRole::Depth),
                    InspectionOperation::CheckFormatSupport,
                    GET_DEVICE_FAILURE.0,
                ));
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
            }
            device.format_query_failure.set(None);
            device.format_fail_at.set(None);
            device.typed_casting.set(false);
            if typed {
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(ResourceProperty::FormatSupport))
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(ResourceProperty::FormatSupport))
                );
                device.casting_query_failure.set(Some(GET_DEVICE_FAILURE));
                let expected = Err(crate::error::test_inspection(
                    InspectionObject::ContextDeviceForResource(ResourceRole::Depth),
                    InspectionOperation::CheckFormatCasting,
                    GET_DEVICE_FAILURE.0,
                ));
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    expected
                );
                device.casting_query_failure.set(None);
            } else {
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Ok(())
                );
            }
            device.typed_casting.set(true);
            assert_eq!(calls().calls, before);
            for (flags, property) in [
                (D3D12_RESOURCE_FLAG_NONE, ResourceProperty::DepthStencil),
                (
                    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL
                        | D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
                    ResourceProperty::ShaderReadable,
                ),
                (
                    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL
                        | D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                    ResourceProperty::UnorderedAccess,
                ),
            ] {
                if allocation == DXGI_FORMAT_R32_TYPELESS && flags == D3D12_RESOURCE_FLAG_NONE {
                    continue;
                }
                let queries = device.format_query_calls.get();
                let casts = device.casting_query_calls.get();
                desc.Flags = flags;
                object.desc.set(desc);
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(property))
                );
                assert_result!(
                    unsafe {
                        let recording = scene.input();
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(property))
                );
                assert_eq!(device.format_query_calls.get(), queries);
                assert_eq!(device.casting_query_calls.get(), casts);
            }
            desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
            object.desc.set(desc);
            object.set_device(scene.foreign_device.as_raw().cast());
            let queries = device.format_query_calls.get();
            let casts = device.casting_query_calls.get();
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(ResourceProperty::Device))
            );
            assert_eq!(device.format_query_calls.get(), queries);
            assert_eq!(device.casting_query_calls.get(), casts);
            object.set_device(scene.device.as_raw().cast());
            // Retry after every metadata/device/query failure remains usable.
            assert_result!(
                unsafe {
                    let recording = scene.input();
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Ok(())
            );
            object.desc.set(baseline);
            for index in [0, 2, 3, 4] {
                if allocation == DXGI_FORMAT_R32_TYPELESS && index == 3 {
                    continue;
                }
                let other = scene.resource_object(index);
                let original = other.desc.get();
                let mut wrong = original;
                wrong.Format = allocation;
                other.desc.set(wrong);
                assert!(matches!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(Error::Input(crate::error::InputError::Dispatch(
                        DispatchValidation::Resource {
                            property: ResourceProperty::Format,
                            ..
                        }
                    )))
                ));
                other.desc.set(original);
            }
        }
        // View-only depth/stencil formats remain outside the allocation contract.
        for format in [
            DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS,
            DXGI_FORMAT_X32_TYPELESS_G8X24_UINT,
            DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
            DXGI_FORMAT_X24_TYPELESS_G8_UINT,
        ] {
            let mut desc = baseline;
            desc.Format = format;
            object.desc.set(desc);
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(ResourceProperty::Format))
            );
        }
        object.desc.set(baseline);
        context.destroy().unwrap();
        assert_eq!(scene.reference_counts(), refs);
    }};
}
#[test]
fn local_depth_allocation_views_and_failures() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_depth_allocation_views_and_failures() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
