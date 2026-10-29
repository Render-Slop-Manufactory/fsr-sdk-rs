// SPDX-License-Identifier: MPL-2.0

//! Public-path scale and new-format inspection, with controlled COM/FFI failures.
use super::{
    com::{GET_DEVICE_FAILURE, Scene},
    *,
};
use crate::{
    error::{
        DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
    },
    upscaler::{
        MotionVectorResolution, MotionVectorScale, NativeMotionVectorScale, ReactiveMaskOptions,
    },
};
use fsr_sdk_sys::api::{FFX_API_RESOURCE_USAGE_UAV, FFX_API_SURFACE_FORMAT_R32G32_FLOAT};
use std::cell::Cell;
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT};

type Observation = ([u32; 2], u32, u32, [u32; 4]);
thread_local! { static CAPTURED: Cell<Option<Observation>> = const { Cell::new(None) }; }

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: The controlled runtime supplies a complete live descriptor. Copy
    // bits only; no borrowed resource pointer or native handle is retained.
    let desc = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    CAPTURED.set(Some((
        [
            desc.motionVectorScale.x.to_bits(),
            desc.motionVectorScale.y.to_bits(),
        ],
        desc.motionVectors.description.format,
        desc.motionVectors.description.usage,
        [
            desc.renderSize.width,
            desc.renderSize.height,
            desc.upscaleSize.width,
            desc.upscaleSize.height,
        ],
    )));
}

fn invalid(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
    ))
}

macro_rules! exercise {
    ($runtime:expr, $context:ty) => {{
        let (mut runtime, observer) = $runtime;
        // SAFETY: Exact paired fixture exports; callbacks observe synchronously
        // and the fixture records no GPU work.
        let (observe, calls) = unsafe {
            (
                *observer
                    .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                        c"fixture_set_dispatch_observer",
                    )
                    .unwrap(),
                *observer
                    .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                    .unwrap(),
            )
        };
        observe(Some(capture));
        for resolution in [
            MotionVectorResolution::Render,
            MotionVectorResolution::Display,
        ] {
            let scene = Scene::new();
            let device = scene.device_object();
            // RG32F is read through an SRV even when its allocation permits a
            // UAV descriptor; no typed UAV shader load/store bit is required.
            device.format_support2.set(Default::default());
            let mut options = sizes();
            options.set_dynamic_resolution(true);
            options.set_motion_vector_resolution(resolution);
            if resolution == MotionVectorResolution::Display {
                let mut desc = scene.resource_object(2).desc.get();
                desc.Width = 1920;
                desc.Height = 1080;
                scene.resource_object(2).desc.set(desc);
            }
            let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                .create(&options)
                .unwrap();
            // A broken format query cannot affect the original RG16F route.
            device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
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
            assert_eq!(device.format_query_calls.get(), 0);
            device.format_query_failure.set(None);
            let motion = scene.resource_object(2);
            let mut desc = motion.desc.get();
            desc.Format = DXGI_FORMAT_R32G32_FLOAT;
            motion.desc.set(desc);
            let references = scene.reference_counts();
            for (x, y) in [
                (0.0, -0.0),
                (f32::from_bits(1), f32::from_bits(0x8000_0001)),
                (f32::MAX, -f32::MAX),
                (-2.0, 4.0),
            ] {
                let mut frame = scene.input();
                assert_eq!(frame.motion_vector_scale(), MotionVectorScale::Uv);
                let scale = NativeMotionVectorScale::new(x, y).unwrap();
                frame.set_motion_vector_scale(crate::upscaler::MotionVectorScale::Native(scale));
                let MotionVectorScale::Native(stored) = frame.motion_vector_scale() else {
                    panic!("expected literal request")
                };
                assert_eq!(
                    (stored.x().to_bits(), stored.y().to_bits()),
                    (x.to_bits(), y.to_bits())
                );
                frame.set_active_render_size(Some(Dimensions::new(1024, 576).unwrap()));
                frame.set_active_output_size(Some(Dimensions::new(1600, 900).unwrap()));
                let before = device.format_query_calls.get();
                assert_result!(
                    context.validate_upscale_inputs(
                        frame.command_list,
                        &frame.resources,
                        &frame.parameters
                    ),
                    Ok(())
                );
                // SAFETY: These COM/runtime fixtures inspect only and record no GPU work.
                assert_result!(
                    unsafe {
                        context.record(frame.command_list, &frame.resources, &frame.parameters)
                    },
                    Ok(())
                );
                assert_eq!(device.format_query_calls.get(), before + 2);
                assert_eq!(
                    CAPTURED.take(),
                    Some((
                        [x.to_bits(), y.to_bits()],
                        FFX_API_SURFACE_FORMAT_R32G32_FLOAT,
                        0,
                        [1024, 576, 1600, 900]
                    ))
                );
                assert_eq!(scene.reference_counts(), references);
            }
            for units in [MotionVectorScale::Uv, MotionVectorScale::Pixels] {
                let mut frame = scene.input();
                frame.set_motion_vector_scale(units);
                frame.set_motion_vector_scale(crate::upscaler::MotionVectorScale::Native(
                    NativeMotionVectorScale::new(-2.0, 4.0).unwrap(),
                ));
                frame.set_active_render_size(Some(Dimensions::new(1024, 576).unwrap()));
                frame.set_active_output_size(Some(Dimensions::new(1600, 900).unwrap()));
                frame.set_motion_vector_scale(units);
                assert_result!(
                    unsafe {
                        context.record(frame.command_list, &frame.resources, &frame.parameters)
                    },
                    Ok(())
                );
                let (x, y) = if units == MotionVectorScale::Pixels {
                    (1.0f32, 1.0f32)
                } else if resolution == MotionVectorResolution::Render {
                    (1024.0, 576.0)
                } else {
                    (1600.0, 900.0)
                };
                assert_eq!(CAPTURED.take().unwrap().0, [x.to_bits(), y.to_bits()]);
            }
            // Missing bits and HRESULTs are inspectable failures, never SDK rejection.
            let full = device.format_support.get();
            let before_calls = calls().calls;
            let mut uav = motion.desc.get();
            uav.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
            motion.desc.set(uav);
            for bit in [
                D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
                D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
            ] {
                device.format_support.set(full & !bit);
                let expected = Err(invalid(
                    ResourceRole::MotionVectors,
                    ResourceProperty::FormatSupport,
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
                assert_eq!(calls().calls, before_calls);
            }
            device.format_support.set(full);
            device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
            let expected = Err(crate::error::test_inspection(
                InspectionObject::ContextDevice,
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
            assert_eq!(calls().calls, before_calls);
            device.format_query_failure.set(None);
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
            assert_eq!(CAPTURED.take().unwrap().2, FFX_API_RESOURCE_USAGE_UAV);
            // Neither typed UAV load nor store bits were supplied by the fixture.
            uav.Flags = D3D12_RESOURCE_FLAG_NONE;
            motion.desc.set(uav);
            device
                .format_support
                .set(full & !D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW);
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
            device.format_support.set(full);
            for (index, role) in [
                (0, ResourceRole::Color),
                (1, ResourceRole::Depth),
                (3, ResourceRole::Exposure),
                (4, ResourceRole::Output),
            ] {
                let object = scene.resource_object(index);
                let original = object.desc.get();
                let mut wrong = original;
                wrong.Format = DXGI_FORMAT_R32G32_FLOAT;
                object.desc.set(wrong);
                let before = device.format_query_calls.get();
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(invalid(role, ResourceProperty::Format))
                );
                assert_eq!(device.format_query_calls.get(), before);
                object.desc.set(original);
            }
            let mut wrong = motion.desc.get();
            wrong.Width += 1;
            motion.desc.set(wrong);
            let before = device.format_query_calls.get();
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(
                    ResourceRole::MotionVectors,
                    ResourceProperty::Extent
                ))
            );
            assert_eq!(device.format_query_calls.get(), before);
            wrong.Width -= 1;
            motion.desc.set(wrong);
            motion.set_device(scene.foreign_device.as_raw().cast());
            assert_result!(
                {
                    let recording = &scene.input();
                    context.validate_upscale_inputs(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(invalid(
                    ResourceRole::MotionVectors,
                    ResourceProperty::Device
                ))
            );
            assert_eq!(device.format_query_calls.get(), before);
            motion.set_device(scene.device.as_raw().cast());
            // Generation still rejects RG32F colour; its role table is unchanged.
            let mask = scene.mask();
            let object = com::object_from(&mask);
            let mut d = object.desc.get();
            d.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
            object.desc.set(d);
            let input = ReactiveMaskTestInputs::new(
                &scene.list,
                &scene.resources[2],
                &scene.resources[0],
                &mask,
                Dimensions::new(1280, 720).unwrap(),
                ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap(),
            );
            assert_result!(
                context.validate_reactive_mask_inputs(
                    input.command_list,
                    &input.resources,
                    &input.parameters
                ),
                Err(invalid(
                    ResourceRole::OpaqueOnlyColor,
                    ResourceProperty::Format
                ))
            );
            drop(mask);
            for (state, error) in [
                (
                    crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                    crate::test_errors::terminal(Error::native(
                        crate::error::Operation::Upscale,
                        3,
                    )),
                ),
                (
                    crate::context_policy::failed(crate::error::Operation::Tuning, 3),
                    crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 3)),
                ),
            ] {
                context.core.eligibility = state;
                assert_result!(
                    {
                        let recording = &scene.input();
                        context.validate_upscale_inputs(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(error)
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
                    Err(error)
                );
                assert_eq!(device.format_query_calls.get(), before);
            }
            assert_eq!(scene.reference_counts(), references);
            context.destroy().unwrap();
        }
    }};
}

#[test]
fn local_native_motion_mapping_role_and_device_failures() {
    exercise!(fixture_runtime(), Upscaler);
}

#[cfg(feature = "movable")]
#[test]
fn movable_native_motion_mapping_role_and_device_failures() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
