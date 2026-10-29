// SPDX-License-Identifier: MPL-2.0

//! Direct public composition, without the workload convenience adapters.
use super::{super::*, com::Scene, support::*};
use crate::{
    error::{DispatchValidation, ResourceProperty, ResourceRole},
    upscaler::{
        FrameParameters, HistoryAction, MotionVectorScale, NativeMotionVectorScale,
        ReactiveMaskOptions, ReactiveMaskParameters,
    },
};
use std::cell::RefCell;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_FLOAT;

fn parameters() -> FrameParameters {
    FrameParameters::new(
        JitterOffsetPixels::new(0.0, 0.0).unwrap(),
        FrameTimeMillis::new(16.0).unwrap(),
        CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(),
        HistoryAction::Reset,
    )
}
fn bindings(scene: &Scene) -> UpscaleResources<'_> {
    UpscaleResources::builder()
        .color(&scene.resources[0])
        .depth(&scene.resources[1])
        .motion_vectors(&scene.resources[2])
        .exposure(ExposureInput::Texture(&scene.resources[3]))
        .output(&scene.resources[4])
        .build()
        .unwrap()
}
#[test]
fn builders_require_all_roles_and_explicit_exposure_without_com_work() {
    let scene = Scene::new();
    let refs = scene.reference_counts();
    let missing = |role| Error::Input(crate::error::InputError::MissingBinding { role });
    let b = UpscaleResources::builder();
    assert!(matches!(
        b.clone().build(),
        Err(Error::Input(crate::error::InputError::MissingBinding {
            role: ResourceRole::Color
        }))
    ));
    let b = b.color(&scene.resources[0]);
    assert_error!(b.clone().build().unwrap_err(), missing(ResourceRole::Depth));
    let b = b.depth(&scene.resources[1]);
    assert_error!(
        b.clone().build().unwrap_err(),
        missing(ResourceRole::MotionVectors)
    );
    let b = b.motion_vectors(&scene.resources[2]);
    assert_error!(
        b.clone().build().unwrap_err(),
        missing(ResourceRole::Exposure)
    );
    let b = b.exposure(ExposureInput::Automatic);
    assert_error!(
        b.clone().build().unwrap_err(),
        missing(ResourceRole::Output)
    );
    let r = b.output(&scene.resources[4]).build().unwrap();
    assert!(matches!(r.exposure(), ExposureInput::Automatic));
    let _ = r.clone();
    let g = ReactiveMaskResources::builder();
    assert_error!(
        g.clone().build().unwrap_err(),
        missing(ResourceRole::OpaqueOnlyColor)
    );
    let g = g.color_opaque_only(&scene.resources[0]);
    assert_error!(g.clone().build().unwrap_err(), missing(ResourceRole::Color));
    let g = g.color_pre_upscale(&scene.resources[0]);
    assert_error!(
        g.clone().build().unwrap_err(),
        missing(ResourceRole::ReactiveMaskOutput)
    );
    let g = g.output(&scene.resources[1]).build().unwrap();
    let _ = g.clone();
    assert_eq!(scene.reference_counts(), refs);
    assert_eq!(scene.device_object().format_query_calls.get(), 0);
}
#[derive(Debug, PartialEq)]
enum Observation {
    Upscale(bool, [u32; 2], u32, [u32; 4], [u32; 2]),
    Generator([u32; 2], [u32; 3], u32),
}
thread_local! {static SEEN: RefCell<Vec<Observation>> = const {RefCell::new(Vec::new())};}
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Controlled synchronous fixture supplies the complete tagged descriptor;
    // retain only copied scalar bits, with no native pointer or resource borrow.
    unsafe {
        if (*header).r#type == FFX_API_DISPATCH_DESC_TYPE_UPSCALE {
            let d = &*header.cast::<ffxDispatchDescUpscale>();
            SEEN.with_borrow_mut(|v| {
                v.push(Observation::Upscale(
                    d.reset,
                    [d.jitterOffset.x.to_bits(), d.jitterOffset.y.to_bits()],
                    d.frameTimeDelta.to_bits(),
                    [
                        d.cameraNear.to_bits(),
                        d.cameraFar.to_bits(),
                        d.cameraFovAngleVertical.to_bits(),
                        d.viewSpaceToMetersFactor.to_bits(),
                    ],
                    [
                        d.motionVectorScale.x.to_bits(),
                        d.motionVectorScale.y.to_bits(),
                    ],
                ))
            });
        } else {
            let d = &*header.cast::<ffxDispatchDescUpscaleGenerateReactiveMask>();
            SEEN.with_borrow_mut(|v| {
                v.push(Observation::Generator(
                    [d.renderSize.width, d.renderSize.height],
                    [
                        d.scale.to_bits(),
                        d.cutoffThreshold.to_bits(),
                        d.binaryValue.to_bits(),
                    ],
                    d.flags,
                ))
            });
        }
    }
}
fn observe(library: &libloading::Library) {
    SEEN.with_borrow_mut(Vec::clear);
    // SAFETY: Matching controlled fixture signature; no worker/GPU/retention.
    unsafe {
        library
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
}
macro_rules! public_workflow {
    ($runtime:expr) => {{
        let (mut runtime, observer) = $runtime;
        let scene = Scene::new();
        observe(&observer);
        // SAFETY: Serialized controlled runtime/COM fixtures, no workers/affinity/GPU work.
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let resources = bindings(&scene);
        let mut p = parameters();
        let refs = scene.reference_counts();
        context
            .validate_upscale_inputs(&scene.list, &resources, &p)
            .unwrap();
        // SAFETY: Controlled recording observes descriptors only; no GPU work.
        unsafe { context.record(&scene.list, &resources, &p) }.unwrap();
        p.set_history_action(HistoryAction::Continue);
        p.set_jitter(JitterOffsetPixels::new(-0.0, 0.25).unwrap());
        p.set_frame_time(FrameTimeMillis::new(20.0).unwrap());
        p.set_camera(CameraParameters::new(0.25, 200.0, 1.5, 2.0).unwrap());
        p.set_motion_vector_scale(MotionVectorScale::Native(
            NativeMotionVectorScale::new(-0.0, f32::from_bits(1)).unwrap(),
        ));
        unsafe { context.record(&scene.list, &resources, &p) }.unwrap();
        p.set_motion_vector_scale(MotionVectorScale::Pixels);
        unsafe { context.record(&scene.list, &resources, &p) }.unwrap();
        SEEN.with_borrow(|v| {
            assert_eq!(
                *v,
                [
                    Observation::Upscale(
                        true,
                        [0, 0],
                        16.0f32.to_bits(),
                        [0.1f32, 100.0, 1.0, 1.0].map(f32::to_bits),
                        [1280.0f32.to_bits(), 720.0f32.to_bits()]
                    ),
                    Observation::Upscale(
                        false,
                        [(-0.0f32).to_bits(), 0.25f32.to_bits()],
                        20.0f32.to_bits(),
                        [0.25f32, 200.0, 1.5, 2.0].map(f32::to_bits),
                        [(-0.0f32).to_bits(), 1]
                    ),
                    Observation::Upscale(
                        false,
                        [(-0.0f32).to_bits(), 0.25f32.to_bits()],
                        20.0f32.to_bits(),
                        [0.25f32, 200.0, 1.5, 2.0].map(f32::to_bits),
                        [1.0f32.to_bits(); 2]
                    )
                ]
            )
        });
        context
            .validate_upscale_inputs(&scene.list, &resources, &p)
            .unwrap();
        let resource = scene.resource_object(0);
        let original = resource.desc.get();
        let mut invalid = original;
        invalid.Format = DXGI_FORMAT_R32_FLOAT;
        resource.desc.set(invalid);
        assert_result!(
            unsafe { context.record(&scene.list, &resources, &p) },
            Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::Resource {
                    role: ResourceRole::Color,
                    property: ResourceProperty::Format
                }
            )))
        );
        SEEN.with_borrow(|v| assert_eq!(v.len(), 3));
        resource.desc.set(original);
        let mask = scene.mask();
        let object = super::com::object_from(&mask);
        let mut desc = object.desc.get();
        desc.Flags |=
            windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
        object.desc.set(desc);
        let g = ReactiveMaskResources::builder()
            .color_opaque_only(&scene.resources[0])
            .color_pre_upscale(&scene.resources[0])
            .output(&mask)
            .build()
            .unwrap();
        let mut gp = ReactiveMaskParameters::new(
            Dimensions::new(320, 180).unwrap(),
            ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap(),
        );
        context
            .validate_reactive_mask_inputs(&scene.list, &g, &gp)
            .unwrap();
        unsafe { context.record_reactive_mask(&scene.list, &g, &gp) }.unwrap();
        let mut controls = ReactiveMaskOptions::new(-0.0, -2.0, 3.0).unwrap();
        controls.set_apply_tonemap(true);
        controls.set_apply_inverse_tonemap(true);
        controls.set_apply_threshold(true);
        controls.set_use_components_max(true);
        gp.set_options(controls);
        gp.set_render_size(Dimensions::new(160, 90).unwrap());
        unsafe { context.record_reactive_mask(&scene.list, &g, &gp) }.unwrap();
        SEEN.with_borrow(|v| {
            assert_eq!(
                v.last(),
                Some(&Observation::Generator(
                    [160, 90],
                    [(-0.0f32).to_bits(), (-2.0f32).to_bits(), 3.0f32.to_bits()],
                    15
                ))
            )
        });
        drop(mask);
        assert_eq!(scene.reference_counts(), refs);
        context.destroy().unwrap();
    }};
}
#[test]
fn public_parameters_and_bindings_drive_both_families_and_reinspect() {
    public_workflow!(fixture_runtime());
    #[cfg(feature = "movable")]
    public_workflow!(fixture_movable_runtime());
}
