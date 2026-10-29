// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::error::{DispatchValidation, Operation, ResourceProperty, ResourceRole};
use windows::Win32::Graphics::{
    Direct3D12::D3D12_COMMAND_LIST_TYPE_COMPUTE, Dxgi::Common::DXGI_FORMAT_R32_FLOAT,
};

macro_rules! exercise {
    ($runtime:expr, $upscaler:ty) => {{
        let (mut runtime, observer) = $runtime;
        let scene = Scene::new();
        let mut context = unsafe { runtime.upscaler_dx12(&scene.device) }
            .create(&sizes())
            .unwrap();
        // SAFETY: These paired fixture exports only read counters or set a return code.
        let (query_calls, dispatch_calls, set_dispatch_code) = unsafe {
            (
                observer
                    .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
                    .unwrap(),
                observer
                    .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                    .unwrap(),
                observer
                    .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                    .unwrap(),
            )
        };
        let queries = query_calls();
        let frame = scene.input();
        let references = scene.reference_counts();
        assert_result!(
            context.validate_upscale_inputs(
                frame.command_list,
                &frame.resources,
                &frame.parameters
            ),
            Ok(())
        );
        assert_eq!(scene.reference_counts(), references);
        assert_eq!(dispatch_calls().calls, 0);
        assert_eq!(query_calls(), queries);

        // A prior successful check must not let dispatch skip current metadata.
        let original = scene.resource_object(0).desc.get();
        let mut wrong = original;
        wrong.Format = DXGI_FORMAT_R32_FLOAT;
        scene.resource_object(0).desc.set(wrong);
        let expected = Error::Input(crate::error::InputError::Dispatch(
            DispatchValidation::Resource {
                role: ResourceRole::Color,
                property: ResourceProperty::Format,
            },
        ));
        assert_result!(
            context.validate_upscale_inputs(
                frame.command_list,
                &frame.resources,
                &frame.parameters
            ),
            Err(expected)
        );
        // SAFETY: The controlled fixture records no GPU work; rejection precedes FFI.
        assert_result!(
            unsafe { context.record(frame.command_list, &frame.resources, &frame.parameters) },
            Err(expected)
        );
        assert_eq!(dispatch_calls().calls, 0);
        assert_eq!(scene.reference_counts(), references);
        scene.resource_object(0).desc.set(original);
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

        set_dispatch_code(7); // Unsupported is not assumed side-effect-free.
        // SAFETY: Matched fixture only observes descriptors and returns the set code.
        assert_result!(
            unsafe {
                let recording = scene.input();
                context.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            },
            Err(Error::native(Operation::Upscale, 7))
        );
        assert_eq!(dispatch_calls().calls, 1);
        scene
            .list_object()
            .list_type
            .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
        for (state, error) in [
            (
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                crate::test_errors::terminal(Error::native(crate::error::Operation::Upscale, 3)),
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
        }
        assert_eq!(dispatch_calls().calls, 1);
        assert_eq!(query_calls(), queries);
        assert_eq!(scene.reference_counts(), references);
        context.destroy().unwrap();
    }};
}

#[test]
fn local_preflight_is_nonmutating_and_dispatch_rechecks() {
    exercise!(fixture_runtime(), Upscaler);
}

#[cfg(feature = "movable")]
#[test]
fn movable_preflight_is_nonmutating_and_dispatch_rechecks() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
