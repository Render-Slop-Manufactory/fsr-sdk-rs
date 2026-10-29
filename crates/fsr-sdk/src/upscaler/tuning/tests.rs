// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    context_policy,
    query::{self, JitterPhaseCount, QualityMode},
    upscaler::Dimensions,
};

const PARAMETERS: [TuningParameter; 5] = [
    TuningParameter::VelocityFactor,
    TuningParameter::ReactivenessScale,
    TuningParameter::ShadingChangeScale,
    TuningParameter::AccumulationAddedPerFrame,
    TuningParameter::MinDisocclusionAccumulation,
];

#[test]
fn checked_requests_map_all_keys_null_resets_and_exact_bits() {
    let mut state = Eligibility::Usable;
    for (expected_key, parameter) in PARAMETERS.into_iter().enumerate() {
        let mut values = vec![TuningValue::ProviderDefault];
        for value in [-0.0, 0.0, f32::from_bits(1), 1.0] {
            values.push(TuningValue::Value(value));
        }
        if matches!(
            parameter,
            TuningParameter::ReactivenessScale | TuningParameter::ShadingChangeScale
        ) {
            values.push(TuningValue::Value(f32::MAX));
        }
        if parameter == TuningParameter::MinDisocclusionAccumulation {
            values.extend([
                TuningValue::Value(-1.0),
                TuningValue::Value(-f32::from_bits(1)),
            ]);
        }
        for value in values {
            let request = TuningRequest::new(parameter, value).unwrap();
            // SAFETY: Synchronous observer consumes complete descriptors without
            // retention or GPU work. OK deliberately acknowledges without application.
            unsafe {
                configure(&mut state, request, |ptr| {
                    let desc = &*ptr.cast::<ffxConfigureDescUpscaleKeyValue>();
                    assert_eq!(desc.header.r#type, 0x10007);
                    assert!(desc.header.pNext.is_null());
                    assert_eq!(desc.key, expected_key as u64);
                    assert_eq!(desc.u64, 0);
                    match value {
                        TuningValue::ProviderDefault => assert!(desc.ptr.is_null()),
                        TuningValue::Value(value) => {
                            assert!(!desc.ptr.is_null());
                            assert_eq!((*desc.ptr.cast::<f32>()).to_bits(), value.to_bits());
                        }
                    }
                    0
                })
                .unwrap();
            }
            assert_eq!(state, Eligibility::Usable);
        }
    }
}

#[test]
fn native_errors_keep_codes_and_first_poison_origin_blocks_every_route() {
    let size = Dimensions::new(1, 1).unwrap();
    let phase = JitterPhaseCount::new(1).unwrap();
    for operation in [Operation::Tuning, Operation::Upscale] {
        for code in [3, 0xa1234567, u32::MAX] {
            let mut state = Eligibility::Usable;
            // SAFETY: Synchronous no-op observer; no pointers retained or GPU work.
            let result = if operation == Operation::Tuning {
                unsafe {
                    configure(
                        &mut state,
                        TuningRequest::new(PARAMETERS[0], TuningValue::Value(1.0)).unwrap(),
                        |_| code,
                    )
                }
            } else {
                context_policy::dispatch(
                    crate::error::Operation::Upscale,
                    &mut state,
                    &mut (),
                    |_| Ok(()),
                    |_, _| code,
                )
            };
            assert_result!(result, Err(Error::native(operation, code)));
            let expected = if operation == Operation::Tuning {
                crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, code))
            } else {
                crate::test_errors::terminal(Error::native(crate::error::Operation::Upscale, code))
            };
            // Entry eligibility still rejects every checked request before native entry.
            for value in [TuningValue::ProviderDefault, TuningValue::Value(-0.0)] {
                // SAFETY: Poisoned invocation must be rejected before native entry.
                assert_result!(
                    unsafe {
                        configure(
                            &mut state,
                            TuningRequest::new(PARAMETERS[1], value).unwrap(),
                            |_| panic!("poison configured"),
                        )
                    },
                    Err(expected)
                );
            }
            assert_result!(
                context_policy::dispatch(
                    crate::error::Operation::Upscale,
                    &mut state,
                    &mut (),
                    |_| -> Result<(), Error> { panic!("poison inspected") },
                    |_, _| panic!("poison dispatched")
                ),
                Err(expected)
            );
            // SAFETY: All seven live query routes must reject before invoking.
            unsafe {
                let mut invoke = |_| panic!("poison queried");
                assert_result!(query::native::provider(state, &mut invoke), Err(expected));
                assert_result!(
                    query::native::resource_requirements(state, &mut invoke),
                    Err(expected)
                );
                assert_result!(
                    query::native::gpu_memory_usage(state, &mut invoke),
                    Err(expected)
                );
                assert_result!(
                    query::native::upscale_ratio(state, QualityMode::Quality, &mut invoke),
                    Err(expected)
                );
                assert_result!(
                    query::native::render_size_for_quality(
                        state,
                        size,
                        QualityMode::Quality,
                        &mut invoke
                    ),
                    Err(expected)
                );
                assert_result!(
                    query::native::jitter_phase_count(state, size, size, &mut invoke),
                    Err(expected)
                );
                assert_result!(
                    query::native::jitter_offset(state, -1, phase, &mut invoke),
                    Err(expected)
                );
            }
        }
    }
}

#[test]
fn ordinary_query_errors_leave_configuration_and_dispatch_available() {
    let mut state = Eligibility::Usable;
    // SAFETY: Observer returns an ordinary error without retaining query storage.
    assert_result!(
        unsafe { query::native::gpu_memory_usage(state, |_| 0xbeef) },
        Err(Error::native(Operation::Query, 0xbeef))
    );
    // SAFETY: Synchronous acknowledgement only, without GPU work or retention.
    unsafe {
        configure(
            &mut state,
            TuningRequest::new(PARAMETERS[4], TuningValue::ProviderDefault).unwrap(),
            |_| 0,
        )
    }
    .unwrap();
    context_policy::dispatch(
        crate::error::Operation::Upscale,
        &mut state,
        &mut (),
        |_| Ok(()),
        |_, _| 0,
    )
    .unwrap();
    assert_eq!(state, Eligibility::Usable);
}
