// SPDX-License-Identifier: MPL-2.0

#[macro_use]
#[path = "fixtures/errors.rs"]
mod test_errors;

use fsr_sdk::{
    error::Error,
    upscaler::{TuningParameter as P, TuningRequest, TuningValue as V, common},
};

#[test]
fn every_parameter_checks_its_domain_and_preserves_accepted_bits() {
    let tiny = f32::from_bits(1);
    let above_one = f32::from_bits(1.0f32.to_bits() + 1);
    let below_minus_one = f32::from_bits((-1.0f32).to_bits() + 1);
    for (parameter, accepted, rejected) in [
        (
            P::VelocityFactor,
            vec![-0.0, 0.0, tiny, 1.0],
            vec![-tiny, above_one, f32::MAX],
        ),
        (
            P::AccumulationAddedPerFrame,
            vec![-0.0, 0.0, tiny, 1.0],
            vec![-tiny, above_one, f32::MAX],
        ),
        (
            P::ReactivenessScale,
            vec![-0.0, 0.0, tiny, f32::MAX],
            vec![-tiny, -1.0, f32::MIN],
        ),
        (
            P::ShadingChangeScale,
            vec![-0.0, 0.0, tiny, f32::MAX],
            vec![-tiny, -1.0, f32::MIN],
        ),
        (
            P::MinDisocclusionAccumulation,
            vec![-1.0, -tiny, -0.0, 0.0, tiny, 1.0],
            vec![below_minus_one, above_one, f32::MAX],
        ),
    ] {
        for value in accepted {
            let request = TuningRequest::new(parameter, V::Value(value)).unwrap();
            assert_eq!(request.parameter(), parameter);
            let V::Value(actual) = request.value() else {
                panic!("lost explicit value")
            };
            assert_eq!(actual.to_bits(), value.to_bits());
        }
        for value in rejected.into_iter().chain([
            f32::NAN,
            f32::from_bits(0xffc01234),
            f32::INFINITY,
            f32::NEG_INFINITY,
        ]) {
            assert_result!(
                TuningRequest::new(parameter, V::Value(value)),
                Err(Error::Input(
                    fsr_sdk::error::InputError::InvalidTuningValue { parameter }
                ))
            );
        }
        let reset = TuningRequest::new(parameter, V::ProviderDefault).unwrap();
        assert_eq!(reset.parameter(), parameter);
        assert_eq!(reset.value(), V::ProviderDefault);
    }
}

#[test]
fn portable_requests_can_be_prepared_on_a_worker_and_reused_as_complete_pairs() {
    fn traits<T: std::fmt::Debug + Clone + Copy + PartialEq + Send + Sync>() {}
    traits::<TuningRequest>();
    let requests = std::thread::spawn(|| {
        [
            (P::ReactivenessScale, V::Value(2.0)),
            (P::VelocityFactor, V::ProviderDefault),
        ]
        .into_iter()
        .map(|(p, v)| TuningRequest::new(p, v))
        .collect::<Result<Vec<_>, _>>()
    })
    .join()
    .unwrap()
    .unwrap();
    let first: common::TuningRequest = requests[0];
    assert_eq!(first, requests[0]);
    assert_result!(
        TuningRequest::new(P::VelocityFactor, first.value()),
        Err(Error::Input(
            fsr_sdk::error::InputError::InvalidTuningValue {
                parameter: P::VelocityFactor
            }
        ))
    );
    assert_eq!(requests[1].value(), V::ProviderDefault);
}
