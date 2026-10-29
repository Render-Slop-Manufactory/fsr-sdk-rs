// SPDX-License-Identifier: MPL-2.0

#[macro_use]
#[path = "fixtures/errors.rs"]
mod test_errors;

use fsr_sdk::{
    error::{Error, QueryValidation},
    query::{JitterPhaseCount, QualityMode},
};

#[test]
fn checked_query_inputs_are_portable_and_preserve_positive_counts() {
    fn traits<T: Copy + Eq + Send + Sync + std::fmt::Debug>() {}
    traits::<QualityMode>();
    traits::<JitterPhaseCount>();
    for count in [1, 20, i32::MAX] {
        assert_eq!(JitterPhaseCount::new(count).unwrap().count(), count);
    }
    for count in [0, -1, i32::MIN] {
        assert_result!(
            JitterPhaseCount::new(count),
            Err(Error::Input(fsr_sdk::error::InputError::Query(
                QueryValidation::JitterPhaseCount
            )))
        );
    }
}
