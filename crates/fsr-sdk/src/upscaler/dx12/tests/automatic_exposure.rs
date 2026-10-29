// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::{
    error::{DispatchValidation, Operation, ResourceProperty, ResourceRole},
    upscaler::{ExposureMode, PreExposure},
};
use std::cell::Cell;

thread_local! {
    static REQUIRED: Cell<u64> = const { Cell::new(15) };
    static QUERY_CODE: Cell<u32> = const { Cell::new(0) };
    static CAPTURED: Cell<Option<(usize, bool, u32)>> = const { Cell::new(None) };
}
unsafe extern "C" fn query(
    context: *mut *mut std::ffi::c_void,
    header: *mut std::ffi::c_void,
) -> u32 {
    // SAFETY: Registered for the constructor's matched synchronous live query.
    unsafe {
        if context.is_null() || (*context).is_null() {
            return 0xf001;
        }
        let d = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
        if d.header.r#type != FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS
            || !d.header.pNext.is_null()
        {
            return 0xf002;
        }
        d.required_resources = REQUIRED.get();
        d.optional_resources = u64::MAX;
    }
    QUERY_CODE.get()
}
unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Fixture supplies a complete descriptor; copy values without retention.
    let d = unsafe { &*header.cast::<ffxDispatchDescUpscale>() };
    let e = &d.exposure;
    // SAFETY: The width/height/depth unions use their texture scalar members.
    let empty = unsafe {
        e.resource.is_null()
            && e.description.r#type == 0
            && e.description.format == 0
            && e.description.width.width == 0
            && e.description.height.height == 0
            && e.description.depth.depth == 0
            && e.description.mipCount == 0
            && e.description.flags == 0
            && e.description.usage == 0
            && e.state == 0
    };
    CAPTURED.set(Some((e.resource as usize, empty, d.preExposure.to_bits())));
}
fn register(observer: &libloading::Library) {
    REQUIRED.set(15);
    QUERY_CODE.set(0);
    // SAFETY: Exact controlled fixture ABI; both observers consume synchronously.
    unsafe {
        observer
            .get::<extern "C" fn(
                Option<
                    unsafe extern "C" fn(*mut *mut std::ffi::c_void, *mut std::ffi::c_void) -> u32,
                >,
            )>(c"fixture_set_query_observer")
            .unwrap()(Some(query));
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
}
fn query_calls(observer: &libloading::Library) -> u32 {
    // SAFETY: Exact fixture export returning its current query count.
    unsafe {
        observer
            .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
            .unwrap()()
    }
}
fn frame(scene: &Scene, automatic: bool) -> UpscaleTestInputs<'_> {
    let base = scene.input();
    let resources = if automatic {
        UpscaleResources::builder()
            .color(&scene.resources[0])
            .depth(&scene.resources[1])
            .motion_vectors(&scene.resources[2])
            .exposure(crate::upscaler::dx12::ExposureInput::Automatic)
            .output(&scene.resources[4])
            .build()
            .unwrap()
    } else {
        base.resources().clone()
    };
    UpscaleTestInputs::new(
        base.command_list(),
        resources,
        base.jitter(),
        base.frame_time(),
        base.camera(),
        base.reset(),
    )
}
macro_rules! check_family {
    ($runtime:expr,$context:ty) => {{
        let (mut runtime, observer) = $runtime;
        let scene = Scene::new();
        register(&observer);
        for mode in [ExposureMode::Explicit, ExposureMode::Automatic] {
            let mut options = sizes();
            options.set_exposure_mode(mode);
            let before = query_calls(&observer);
            let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
                .create(&options)
                .unwrap();
            let automatic = mode == ExposureMode::Automatic;
            assert_eq!(
                fixture_snapshot(&observer).flags,
                if automatic { 32 } else { 0 }
            );
            assert_eq!(query_calls(&observer) - before, 1);
            let counts = scene.reference_counts();
            CAPTURED.set(None);
            // SAFETY: Controlled fixtures record no GPU work; mismatched presence
            // is rejected before COM inspection or native dispatch.
            assert_result!(
                unsafe {
                    let recording = frame(&scene, !automatic);
                    context.record(
                        recording.command_list,
                        &recording.resources,
                        &recording.parameters,
                    )
                },
                Err(Error::Input(crate::error::InputError::Dispatch(
                    DispatchValidation::ExposureMode
                )))
            );
            assert!(CAPTURED.get().is_none());
            assert_eq!(scene.reference_counts(), counts);
            if automatic {
                // The omitted resource must never be inspected, even if invalid.
                scene
                    .resource_object(3)
                    .query_failure
                    .set(Some(com::QUERY_FAILURE));
            } else {
                let mut desc = scene.resource_object(3).desc.get();
                desc.Width = 2;
                scene.resource_object(3).desc.set(desc);
                // SAFETY: Fixture metadata rejects before native entry.
                assert_result!(
                    unsafe {
                        let recording = frame(&scene, false);
                        context.record(
                            recording.command_list,
                            &recording.resources,
                            &recording.parameters,
                        )
                    },
                    Err(Error::Input(crate::error::InputError::Dispatch(
                        DispatchValidation::Resource {
                            role: ResourceRole::Exposure,
                            property: ResourceProperty::Extent
                        }
                    )))
                );
                desc.Width = 1;
                scene.resource_object(3).desc.set(desc);
            }
            let mut input = frame(&scene, automatic);
            assert_eq!(input.resources().exposure().texture().is_none(), automatic);
            input.set_pre_exposure(PreExposure::new(0.25).unwrap());
            // SAFETY: Fixture supplies valid matching inputs; no GPU work recorded.
            unsafe { context.record(input.command_list, &input.resources, &input.parameters) }
                .unwrap();
            assert_eq!(
                CAPTURED.take(),
                Some((
                    if automatic {
                        0
                    } else {
                        scene.resources[3].as_raw() as usize
                    },
                    automatic,
                    0.25f32.to_bits()
                ))
            );
            scene.resource_object(3).query_failure.set(None);
            assert_eq!(scene.reference_counts(), counts);
            assert_eq!(context.core.eligibility, Eligibility::Usable);
            context.destroy().unwrap();
        }
        let counts = scene.reference_counts();
        for mode in [ExposureMode::Explicit, ExposureMode::Automatic] {
            let mut options = sizes();
            options.set_exposure_mode(mode);
            for bits in [1 << 6, 1 << 63] {
                REQUIRED.set(bits);
                let before = fixture_snapshot(&observer).destroys;
                let error = unsafe { runtime.upscaler_dx12(&scene.device) }
                    .create(&options)
                    .err()
                    .unwrap();
                match error {
                    Error::Input(crate::error::InputError::UnsupportedResourceRequirements {
                        required,
                    }) => {
                        assert_eq!(required.bits(), bits)
                    }
                    _ => panic!("wrong error: {error:?}"),
                }
                assert_eq!(fixture_snapshot(&observer).destroys, before + 1);
                assert_eq!(scene.reference_counts(), counts);
            }
            REQUIRED.set(15);
            QUERY_CODE.set(0xf1234567);
            let before = fixture_snapshot(&observer).destroys;
            assert!(matches!(
                unsafe { runtime.upscaler_dx12(&scene.device) }.create(&options),
                Err(Error::Native(crate::error::NativeError {
                    operation: Operation::Query,
                    code: 0xf1234567
                }))
            ));
            assert_eq!(fixture_snapshot(&observer).destroys, before + 1);
            assert_eq!(scene.reference_counts(), counts);
            QUERY_CODE.set(0);
            let context = unsafe { runtime.upscaler_dx12(&scene.device) }
                .create(&options)
                .unwrap();
            context.destroy().unwrap();
            assert_eq!(scene.reference_counts(), counts);
        }
    }};
}
#[test]
fn exposure_modes_mapping_preflight_and_failed_construction_cleanup() {
    check_family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    check_family!(fixture_movable_runtime(), MovableUpscaler);
}
