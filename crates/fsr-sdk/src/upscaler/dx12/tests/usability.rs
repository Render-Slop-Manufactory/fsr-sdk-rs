// SPDX-License-Identifier: MPL-2.0

use super::{super::*, com::Scene, support::*};
use crate::{
    error::TerminalCause,
    query::QualityMode,
    upscaler::{ContextUsability, ReactiveMaskOptions, ReactiveMaskParameters},
};

fn calls(observer: &libloading::Library) -> [u32; 5] {
    // SAFETY: Exact signatures exported by the controlled synchronous fixture.
    unsafe {
        let count = |name| observer.get::<extern "C" fn() -> u32>(name).unwrap()();
        let snapshot = fixture_snapshot(observer);
        let dispatch = observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()();
        [
            snapshot.creates,
            snapshot.destroys,
            dispatch.calls,
            count(c"fixture_query_calls"),
            count(c"fixture_configure_calls"),
        ]
    }
}

macro_rules! workflow {
    ($runtime:expr) => {{
        let (mut runtime, observer) = $runtime;
        let scene = Scene::new();
        // SAFETY: Serialized controlled runtime/COM fixture, no actual GPU work.
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let mut sibling = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        let input = scene.input();
        let snapshot = context.usability();
        assert_eq!(snapshot, ContextUsability::Usable);
        // Local rejection leaves the context usable and can be corrected.
        scene
            .list_object()
            .get_device_failure
            .set(Some(windows::core::HRESULT(0x80004005u32 as i32)));
        assert!(
            context
                .validate_upscale_inputs(&scene.list, &input.resources, &input.parameters)
                .is_err()
        );
        assert_eq!(context.usability(), ContextUsability::Usable);
        scene.list_object().get_device_failure.set(None);
        // SAFETY: Fixture checks descriptors only, with no workers or GPU commands.
        unsafe { context.record(&scene.list, &input.resources, &input.parameters) }.unwrap();
        unsafe { sibling.record(&scene.list, &input.resources, &input.parameters) }.unwrap();
        let refs = scene.reference_counts();
        let before = calls(&observer);
        let formats = scene.device_object().format_query_calls.get();
        context.mark_recording_abandoned();
        context.mark_recording_abandoned();
        let terminal = ContextUsability::Unusable(TerminalCause::RecordingAbandoned);
        assert_eq!(context.usability(), terminal);
        assert_eq!(snapshot, ContextUsability::Usable);
        // The renderer, rather than an inferred shared-list registry, marks all owners.
        assert_eq!(sibling.usability(), ContextUsability::Usable);
        sibling.mark_recording_abandoned();
        assert_eq!(sibling.usability(), terminal);
        assert_eq!(calls(&observer), before);
        assert_eq!(scene.reference_counts(), refs);
        assert_eq!(scene.device_object().format_query_calls.get(), formats);

        // These would fail inspection if the terminal guard did not run first.
        scene
            .list_object()
            .get_device_failure
            .set(Some(windows::core::HRESULT(0x80004005u32 as i32)));
        let generator = ReactiveMaskResources::builder()
            .color_opaque_only(&scene.resources[0])
            .color_pre_upscale(&scene.resources[0])
            .output(&scene.resources[0])
            .build()
            .unwrap();
        let parameters = ReactiveMaskParameters::new(
            Dimensions::new(1, 1).unwrap(),
            ReactiveMaskOptions::new(1.0, 0.2, 0.0).unwrap(),
        );
        let rejected = || Error::ContextUnusable(TerminalCause::RecordingAbandoned);
        assert_result!(
            context.validate_upscale_inputs(&scene.list, &input.resources, &input.parameters),
            Err(rejected())
        );
        assert_result!(
            context.validate_reactive_mask_inputs(&scene.list, &generator, &parameters),
            Err(rejected())
        );
        // SAFETY: Terminal guard must reject before any inspection/native invocation.
        assert_result!(
            unsafe { context.record(&scene.list, &input.resources, &input.parameters) },
            Err(rejected())
        );
        assert_result!(
            unsafe { context.record_reactive_mask(&scene.list, &generator, &parameters) },
            Err(rejected())
        );
        assert_result!(context.provider(), Err(rejected()));
        assert_result!(context.resource_requirements(), Err(rejected()));
        assert_result!(context.gpu_memory_usage(), Err(rejected()));
        assert_result!(context.upscale_ratio(QualityMode::Quality), Err(rejected()));
        assert_result!(
            context.render_size_for_quality(
                Dimensions::new(u32::MAX, 1).unwrap(),
                QualityMode::Quality
            ),
            Err(rejected())
        );
        assert_result!(
            context.jitter_phase_count(
                Dimensions::new(1, 1).unwrap(),
                Dimensions::new(1, 1).unwrap()
            ),
            Err(rejected())
        );
        assert_result!(
            context.jitter_offset(-1, crate::query::JitterPhaseCount::new(1).unwrap()),
            Err(rejected())
        );
        assert_result!(
            context.request_tuning(
                TuningRequest::new(
                    TuningParameter::VelocityFactor,
                    TuningValue::ProviderDefault
                )
                .unwrap()
            ),
            Err(rejected())
        );
        // SAFETY: Controlled fixture has no callback targets/GPU work; no call may occur.
        assert_result!(
            unsafe { context.configure_global_debug_legacy(DebugLevel::Silent, None) },
            Err(rejected())
        );
        assert_eq!(calls(&observer), before);
        assert_eq!(scene.reference_counts(), refs);
        assert_eq!(scene.device_object().format_query_calls.get(), formats);
        assert_eq!(context.usability(), terminal);
        context.destroy().unwrap();
        drop(sibling);
        assert_eq!(calls(&observer)[1], before[1] + 2);
    }};
}

#[test]
fn abandonment_blocks_all_live_routes_without_native_or_com_work_for_both_families() {
    workflow!(fixture_runtime());
    #[cfg(feature = "movable")]
    workflow!(fixture_movable_runtime());
}

macro_rules! cleanup {
    ($runtime:expr) => {{
        let (mut runtime, observer) = $runtime;
        let (device, fake) = fake_device();
        for destroy_code in [0, 0xf1234567] {
            for explicit in [false, true] {
                fixture_set(&observer, 0, destroy_code, 0);
                let mut context = unsafe { runtime.upscaler_dx12(&device) }
                    .create(&sizes())
                    .unwrap();
                let retained = fake.references.get();
                let before = fixture_snapshot(&observer).destroys;
                context.mark_recording_abandoned();
                assert_eq!(fake.references.get(), retained);
                if explicit {
                    let result = context.destroy();
                    if destroy_code == 0 {
                        result.unwrap();
                    } else {
                        assert_result!(
                            result,
                            Err(Error::native(
                                crate::error::Operation::Destroy,
                                destroy_code
                            ))
                        );
                    }
                } else {
                    drop(context);
                }
                assert_eq!(fixture_snapshot(&observer).destroys, before + 1);
                assert_eq!(
                    fake.references.get(),
                    retained - u32::from(destroy_code == 0)
                );
            }
        }
    }};
}

#[test]
fn abandonment_preserves_single_attempt_cleanup_and_failed_destroy_retention() {
    cleanup!(fixture_runtime());
    #[cfg(feature = "movable")]
    cleanup!(fixture_movable_runtime());
}
