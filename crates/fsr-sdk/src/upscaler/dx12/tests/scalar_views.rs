// SPDX-License-Identifier: MPL-2.0

//! Fixed scalar/vector views, with real allocation metadata retained.
use super::{
    com::{GET_DEVICE_FAILURE, Scene},
    *,
};
use crate::error::{
    DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
};
use crate::upscaler::ReactiveMaskOptions;
use std::cell::RefCell;
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};
thread_local! { static CAPTURED: RefCell<Vec<fsr_sdk_sys::api::FfxApiResource>> = const { RefCell::new(Vec::new()) }; }
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    let resources = unsafe {
        if (*header).r#type == FFX_API_DISPATCH_DESC_TYPE_UPSCALE {
            let d = &*header.cast::<ffxDispatchDescUpscale>();
            vec![
                d.color,
                d.depth,
                d.motionVectors,
                d.exposure,
                d.output,
                d.reactive,
                d.transparencyAndComposition,
            ]
        } else {
            let d = &*header.cast::<ffxDispatchDescUpscaleGenerateReactiveMask>();
            vec![d.outReactive]
        }
    };
    CAPTURED.with_borrow_mut(|v| *v = resources);
}

fn invalid(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::Resource { role, property },
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
        let observe = unsafe {
            *observer
                .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                    c"fixture_set_dispatch_observer",
                )
                .unwrap()
        };
        observe(Some(capture));
        let scene = Scene::new();
        let device = scene.device_object();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let refs = scene.reference_counts();
        let full = scene.mask();
        let full_object = com::object_from(&full);
        full_object.desc.set(scene.resource_object(0).desc.get());
        let mask = scene.mask();
        let mask_object = com::object_from(&mask);
        let baseline_mask = mask_object.desc.get();
        for (index, role, allocation, view, native) in [
            (
                1,
                ResourceRole::Depth,
                DXGI_FORMAT_R16_TYPELESS,
                DXGI_FORMAT_R16_FLOAT,
                34,
            ),
            (
                1,
                ResourceRole::Depth,
                DXGI_FORMAT_R32_TYPELESS,
                DXGI_FORMAT_R32_FLOAT,
                37,
            ),
            (
                1,
                ResourceRole::Depth,
                DXGI_FORMAT_R16_UNORM,
                DXGI_FORMAT_R16_UNORM,
                23,
            ),
            (
                2,
                ResourceRole::MotionVectors,
                DXGI_FORMAT_R32G32_TYPELESS,
                DXGI_FORMAT_R32G32_FLOAT,
                31,
            ),
            (
                2,
                ResourceRole::MotionVectors,
                DXGI_FORMAT_R16G16_TYPELESS,
                DXGI_FORMAT_R16G16_FLOAT,
                33,
            ),
            (
                3,
                ResourceRole::Exposure,
                DXGI_FORMAT_R32_TYPELESS,
                DXGI_FORMAT_R32_FLOAT,
                37,
            ),
            (
                5,
                ResourceRole::ReactiveMask,
                DXGI_FORMAT_R8_TYPELESS,
                DXGI_FORMAT_R8_UNORM,
                35,
            ),
            (
                5,
                ResourceRole::TransparencyAndCompositionMask,
                DXGI_FORMAT_R8_TYPELESS,
                DXGI_FORMAT_R8_UNORM,
                35,
            ),
            (
                5,
                ResourceRole::ReactiveMaskOutput,
                DXGI_FORMAT_R8_TYPELESS,
                DXGI_FORMAT_R8_UNORM,
                35,
            ),
        ] {
            let object = if index == 5 {
                mask_object
            } else {
                scene.resource_object(index)
            };
            let original = object.desc.get();
            let output = role == ResourceRole::ReactiveMaskOutput;
            let input = || {
                let input = scene.input();
                let mut bundle = input.resources().clone();
                match role {
                    ResourceRole::ReactiveMask => bundle.set_reactive_mask(Some(&mask)),
                    ResourceRole::TransparencyAndCompositionMask => {
                        bundle.set_transparency_and_composition_mask(Some(&mask))
                    }
                    _ => (),
                }
                UpscaleTestInputs::new(
                    &scene.list,
                    bundle,
                    input.jitter(),
                    input.frame_time(),
                    input.camera(),
                    input.reset(),
                )
            };
            let generator = || {
                ReactiveMaskTestInputs::new(
                    &scene.list,
                    &scene.resources[0],
                    &full,
                    &mask,
                    Dimensions::new(1280, 720).unwrap(),
                    ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap(),
                )
            };
            macro_rules! validate {
                () => {
                    if output {
                        {
                            let recording = &generator();
                            context.validate_reactive_mask_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    } else {
                        {
                            let recording = &input();
                            context.validate_upscale_inputs(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    }
                };
            }
            macro_rules! record {
                () => {
                    if output {
                        unsafe {
                            let recording = generator();
                            context.record_reactive_mask(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    } else {
                        unsafe {
                            let recording = input();
                            context.record(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                    }
                };
            }
            let support1 = device.format_support.get();
            let support2 = device.format_support2.get();
            for uav in [false, true] {
                if output && !uav {
                    continue;
                }
                let mut desc = original;
                desc.Format = allocation;
                desc.Flags = if uav {
                    D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
                } else {
                    D3D12_RESOURCE_FLAG_NONE
                };
                object.desc.set(desc);
                let profile = D3D12_FORMAT_SUPPORT1_TEXTURE2D
                    | D3D12_FORMAT_SUPPORT1_SHADER_LOAD
                    | if output {
                        D3D12_FORMAT_SUPPORT1::default()
                    } else {
                        D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE
                    }
                    | if uav {
                        D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW
                    } else {
                        D3D12_FORMAT_SUPPORT1::default()
                    };
                device.format_support.set(profile);
                device.format_support2.set(if output {
                    D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE
                } else {
                    D3D12_FORMAT_SUPPORT2::default()
                });
                device.format_queries.borrow_mut().clear();
                let casts = device.casting_query_calls.get();
                assert_result!(validate!(), Ok(()));
                assert_result!(record!(), Ok(()));
                assert_eq!(&*device.format_queries.borrow(), &[view, view]);
                assert_eq!(device.casting_query_calls.get(), casts);
                CAPTURED.with_borrow(|v| {
                    let slot = if output {
                        0
                    } else if role == ResourceRole::TransparencyAndCompositionMask {
                        6
                    } else {
                        index
                    };
                    let r = &v[slot];
                    assert_eq!(
                        (r.description.format, r.description.usage, r.state),
                        (native, if uav { 2 } else { 0 }, if output { 2 } else { 4 })
                    );
                });
                let before = calls().calls;
                device.format_query_failure.set(Some(GET_DEVICE_FAILURE));
                let expected = Err(crate::error::test_inspection(
                    InspectionObject::ContextDeviceForResource(role),
                    InspectionOperation::CheckFormatSupport,
                    GET_DEVICE_FAILURE.0,
                ));
                assert_result!(validate!(), expected);
                assert_result!(record!(), expected);
                device.format_query_failure.set(None);
                for bit in [
                    D3D12_FORMAT_SUPPORT1_TEXTURE2D,
                    D3D12_FORMAT_SUPPORT1_SHADER_LOAD,
                    D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
                    D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
                ] {
                    if !profile.contains(bit) {
                        continue;
                    }
                    device.format_support.set(profile & !bit);
                    let expected = Err(invalid(role, ResourceProperty::FormatSupport));
                    assert_result!(validate!(), expected);
                    assert_result!(record!(), expected);
                }
                device.format_support.set(profile);
                if output {
                    device.format_support2.set(D3D12_FORMAT_SUPPORT2::default());
                    assert_result!(
                        record!(),
                        Err(invalid(role, ResourceProperty::FormatSupport))
                    );
                    device
                        .format_support2
                        .set(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE);
                }
                assert_eq!(calls().calls, before);
                object.set_device(scene.foreign_device.as_raw().cast());
                assert_result!(validate!(), Err(invalid(role, ResourceProperty::Device)));
                object.set_device(scene.device.as_raw().cast());
                desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
                object.desc.set(desc);
                // R32 typeless depth deliberately switches to the established D32 route.
                if !(role == ResourceRole::Depth && allocation == DXGI_FORMAT_R32_TYPELESS) {
                    assert_result!(
                        validate!(),
                        Err(invalid(role, ResourceProperty::DepthStencil))
                    );
                }
                desc.Flags &= !D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
                object.desc.set(desc);
                assert_result!(record!(), Ok(()));
            }
            device.format_support.set(support1);
            device.format_support2.set(support2);
            object.desc.set(original);
        }
        mask_object.desc.set(baseline_mask);
        drop(full);
        drop(mask);
        assert_eq!(scene.reference_counts(), refs);
        context.destroy().unwrap();
    }};
}

#[test]
fn local_fixed_scalar_views_and_failures() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_fixed_scalar_views_and_failures() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
