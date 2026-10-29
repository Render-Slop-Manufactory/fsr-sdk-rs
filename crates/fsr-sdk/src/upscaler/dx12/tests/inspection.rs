// SPDX-License-Identifier: MPL-2.0

use super::com::{GET_DEVICE_FAILURE, QUERY_FAILURE, Scene};
use super::*;
use crate::error::{
    DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
};
use windows::{
    Win32::Graphics::Direct3D12::{
        D3D12_COMMAND_LIST_TYPE_COMPUTE, D3D12_COMMAND_LIST_TYPE_DIRECT,
        D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, D3D12_RESOURCE_FLAG_NONE,
    },
    core::Interface,
};

fn observation(observer: &libloading::Library) -> DispatchObservation {
    // SAFETY: The locally compiled runtime fixture exports this exact signature.
    unsafe {
        observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()()
    }
}

fn invalid(reason: DispatchValidation) -> Error {
    Error::Input(crate::error::InputError::Dispatch(reason))
}
fn resource_error(role: ResourceRole, property: ResourceProperty) -> Error {
    invalid(DispatchValidation::Resource { role, property })
}

#[test]
fn public_dispatch_inspects_com_and_recovers_after_each_preflight_error() {
    let (mut runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    let mut upscaler = unsafe { runtime.upscaler_dx12(&scene.device) }
        .create(&sizes())
        .unwrap();
    let mut calls = 0;
    let mut check = |expected: Error, reset: &dyn Fn()| {
        let before = scene.reference_counts();
        assert_result!(
            {
                let recording = &scene.input();
                upscaler.validate_upscale_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(expected)
        );
        assert_eq!(observation(&observer).calls, calls);
        assert_eq!(scene.reference_counts(), before);
        // SAFETY: The fixture runtime records only the descriptor; no GPU work.
        assert_result!(
            unsafe {
                let recording = scene.input();
                upscaler.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(expected)
        );
        assert_eq!(observation(&observer).calls, calls);
        assert_eq!(scene.reference_counts(), before);
        reset();
        let before = scene.reference_counts();
        assert_result!(
            {
                let recording = &scene.input();
                upscaler.validate_upscale_inputs(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        assert_eq!(observation(&observer).calls, calls);
        // SAFETY: The fixture runtime records no GPU commands.
        assert_result!(
            unsafe {
                let recording = scene.input();
                upscaler.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Ok(())
        );
        calls += 1;
        assert_eq!(observation(&observer).calls, calls);
        assert_eq!(scene.reference_counts(), before);
    };

    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
    check(invalid(DispatchValidation::CommandListType), &|| {
        scene
            .list_object()
            .list_type
            .set(D3D12_COMMAND_LIST_TYPE_DIRECT)
    });

    scene
        .list_object()
        .set_device(scene.foreign_device.as_raw().cast());
    check(invalid(DispatchValidation::CommandListDevice), &|| {
        scene.list_object().set_device(scene.device.as_raw().cast())
    });

    scene
        .resource_object(2)
        .set_device(scene.foreign_device.as_raw().cast());
    check(
        resource_error(ResourceRole::MotionVectors, ResourceProperty::Device),
        &|| {
            scene
                .resource_object(2)
                .set_device(scene.device.as_raw().cast())
        },
    );

    // Different ID3D12Resource pointers report the same canonical IUnknown.
    assert_ne!(scene.resources[0].as_raw(), scene.resources[1].as_raw());
    scene
        .resource_object(1)
        .set_canonical(scene.resources[0].as_raw().cast());
    check(
        resource_error(ResourceRole::Depth, ResourceProperty::DistinctIdentity),
        &|| {
            scene
                .resource_object(1)
                .set_canonical(scene.resources[1].as_raw().cast())
        },
    );

    scene
        .list_object()
        .get_device_failure
        .set(Some(GET_DEVICE_FAILURE));
    check(
        crate::error::test_inspection(
            InspectionObject::CommandList,
            InspectionOperation::GetDevice,
            GET_DEVICE_FAILURE.0,
        ),
        &|| scene.list_object().get_device_failure.set(None),
    );

    scene
        .resource_object(4)
        .get_device_failure
        .set(Some(GET_DEVICE_FAILURE));
    check(
        crate::error::test_inspection(
            InspectionObject::Resource(ResourceRole::Output),
            InspectionOperation::GetDevice,
            GET_DEVICE_FAILURE.0,
        ),
        &|| scene.resource_object(4).get_device_failure.set(None),
    );

    scene
        .resource_object(2)
        .query_failure
        .set(Some(QUERY_FAILURE));
    check(
        crate::error::test_inspection(
            InspectionObject::Resource(ResourceRole::MotionVectors),
            InspectionOperation::QueryIdentity,
            QUERY_FAILURE.0,
        ),
        &|| scene.resource_object(2).query_failure.set(None),
    );

    scene.device_object().query_failure.set(Some(QUERY_FAILURE));
    check(
        crate::error::test_inspection(
            InspectionObject::ContextDevice,
            InspectionOperation::QueryIdentity,
            QUERY_FAILURE.0,
        ),
        &|| scene.device_object().query_failure.set(None),
    );

    let original = scene.resource_object(0).desc.get();
    let mut wrong = original;
    wrong.Width = 1279;
    scene.resource_object(0).desc.set(wrong);
    check(
        resource_error(ResourceRole::Color, ResourceProperty::Extent),
        &|| scene.resource_object(0).desc.set(original),
    );

    let original = scene.resource_object(1).desc.get();
    let mut denied = original;
    denied.Flags = D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
    scene.resource_object(1).desc.set(denied);
    check(
        resource_error(ResourceRole::Depth, ResourceProperty::ShaderReadable),
        &|| scene.resource_object(1).desc.set(original),
    );

    let original = scene.resource_object(4).desc.get();
    let mut no_uav = original;
    no_uav.Flags = D3D12_RESOURCE_FLAG_NONE;
    scene.resource_object(4).desc.set(no_uav);
    check(
        resource_error(ResourceRole::Output, ResourceProperty::UnorderedAccess),
        &|| scene.resource_object(4).desc.set(original),
    );

    assert_eq!(
        observation(&observer).tag,
        FFX_API_DISPATCH_DESC_TYPE_UPSCALE
    );
    upscaler.destroy().unwrap();
}

#[test]
fn inspection_preserves_error_order_and_releases_references_on_each_path() {
    let (mut runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    let mut upscaler = unsafe { runtime.upscaler_dx12(&scene.device) }
        .create(&sizes())
        .unwrap();
    let original = scene.resource_object(0).desc.get();
    let mut wrong = original;
    wrong.Format = windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_FLOAT;
    scene.resource_object(0).desc.set(wrong);
    scene
        .resource_object(1)
        .set_canonical(scene.resources[0].as_raw().cast());
    scene
        .list_object()
        .set_device(scene.foreign_device.as_raw().cast());
    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);

    let mut reject = |expected| {
        let references = scene.reference_counts();
        // SAFETY: The controlled runtime consumes descriptors and records no GPU work.
        assert_result!(
            unsafe {
                let recording = scene.input();
                upscaler.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(expected)
        );
        assert_eq!(observation(&observer).calls, 0);
        assert_eq!(scene.reference_counts(), references);
    };
    reject(invalid(DispatchValidation::CommandListType));
    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_DIRECT);
    reject(resource_error(
        ResourceRole::Depth,
        ResourceProperty::DistinctIdentity,
    ));
    scene
        .resource_object(1)
        .set_canonical(scene.resources[1].as_raw().cast());
    reject(resource_error(
        ResourceRole::Color,
        ResourceProperty::Format,
    ));
    wrong = original;
    wrong.Width -= 1;
    scene.resource_object(0).desc.set(wrong);
    reject(resource_error(
        ResourceRole::Color,
        ResourceProperty::Extent,
    ));
    scene.resource_object(0).desc.set(original);
    reject(invalid(DispatchValidation::CommandListDevice));

    scene.list_object().set_device(scene.device.as_raw().cast());
    let references = scene.reference_counts();
    // SAFETY: All fixture metadata is valid; the fixture records no GPU commands.
    assert_result!(
        unsafe {
            let recording = scene.input();
            upscaler.record(
                recording.command_list,
                &recording.resources,
                &recording.parameters,
            )
        },
        Ok(())
    );
    assert_eq!(observation(&observer).calls, 1);
    assert_eq!(scene.reference_counts(), references);
    upscaler.destroy().unwrap();
}
