// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::error::Error as _;

#[test]
fn portable_error_traits_and_owned_source_categories() {
    fn owned<T: std::error::Error + Send + Sync + 'static>() {}
    owned::<Error>();
    owned::<BackendError>();
    owned::<Dx12InspectionError>();
    owned::<QueryAllocationError>();
    let reason = InputError::InvalidDimensions {
        width: 0,
        height: 7,
    };
    let error = Error::Input(reason);
    assert_eq!(
        error.source().unwrap().downcast_ref::<InputError>(),
        Some(&reason)
    );
    assert!(error.source().unwrap().source().is_none());
    let failure = NativeError::new(Operation::Destroy, u32::MAX);
    let error = Error::Native(failure);
    assert_eq!(
        error.source().unwrap().downcast_ref::<NativeError>(),
        Some(&failure)
    );
    assert_eq!(
        error.to_string(),
        "FSR Destroy failed with native code 4294967295"
    );
    assert!(error.source().unwrap().source().is_none());
    let invariant = Error::Invariant {
        operation: Operation::Create,
        reason: InvariantViolation::ProviderSelectionMismatch {
            requested: crate::query::ProviderId::from_u64(0),
            actual: crate::query::ProviderId::from_u64(u64::MAX),
        },
    };
    assert!(invariant.to_string().contains("Create"));
    assert!(invariant.source().unwrap().is::<InvariantViolation>());
}

#[test]
fn allocation_retains_count_reason_and_original_conversion_or_reservation() {
    let conversion = u8::try_from(256u16).unwrap_err();
    let error = Error::QueryAllocation(QueryAllocationError::new(
        u64::MAX,
        QueryAllocationReason::CountNotRepresentable,
        conversion,
    ));
    let allocation = error
        .source()
        .unwrap()
        .downcast_ref::<QueryAllocationError>()
        .unwrap();
    assert_eq!(allocation.count(), u64::MAX);
    assert_eq!(
        allocation.reason(),
        QueryAllocationReason::CountNotRepresentable
    );
    assert!(
        allocation
            .source()
            .unwrap()
            .is::<std::num::TryFromIntError>()
    );
    // Capacity overflow is deterministic and does not request a huge allocation.
    let reservation = Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err();
    let error = QueryAllocationError::new(
        u64::MAX,
        QueryAllocationReason::ReservationFailed,
        reservation,
    );
    assert_eq!(error.reason(), QueryAllocationReason::ReservationFailed);
    assert_eq!(error.count(), u64::MAX);
    assert!(
        error
            .source()
            .unwrap()
            .is::<std::collections::TryReserveError>()
    );
}

#[test]
fn first_context_failure_survives_every_later_mutation_and_query_attempt() {
    use crate::context_policy::Eligibility;
    let operations = [
        Operation::Upscale,
        Operation::GenerateReactiveMask,
        Operation::Tuning,
        Operation::ConfigureContextDebugLegacy,
    ];
    for operation in operations {
        for code in [3, 0xf1234567, u32::MAX] {
            let mut eligibility = Eligibility::Usable;
            let first = eligibility.record(operation, code).unwrap_err();
            let Error::Native(failure) = first else {
                panic!("lost native category")
            };
            assert_eq!(failure.operation(), operation);
            assert_eq!(failure.code(), code);
            eligibility.mark_recording_abandoned();
            eligibility.mark_recording_abandoned();
            assert_eq!(
                eligibility.usability(),
                crate::upscaler::ContextUsability::Unusable(TerminalCause::Native(failure))
            );
            for later in operations {
                let error = eligibility.record(later, 0).unwrap_err();
                let cause = error
                    .source()
                    .unwrap()
                    .downcast_ref::<TerminalCause>()
                    .unwrap();
                assert_eq!(*cause, TerminalCause::Native(failure));
                assert_eq!(
                    cause.source().unwrap().downcast_ref::<NativeError>(),
                    Some(&failure)
                );
                assert_result!(
                    eligibility.record(later, 0x1234),
                    Err(crate::test_errors::terminal(Error::Native(failure)))
                );
                assert_result!(
                    crate::context_policy::dispatch(
                        later,
                        &mut eligibility,
                        &mut (),
                        |_| -> Result<(), Error> { panic!("terminal inspection") },
                        |_, _| panic!("terminal dispatch")
                    ),
                    Err(crate::test_errors::terminal(Error::Native(failure)))
                );
            }
            // SAFETY: The terminal check must reject before inspecting or invoking.
            let queried = unsafe {
                crate::query::native::gpu_memory_usage(eligibility, |_| panic!("terminal query"))
            };
            assert_result!(
                queried,
                Err(crate::test_errors::terminal(Error::Native(failure)))
            );
        }
    }
}

#[test]
fn abandonment_is_terminal_before_every_mutating_origin_and_has_no_native_source() {
    use crate::{context_policy::Eligibility, upscaler::ContextUsability};
    let mut eligibility = Eligibility::Usable;
    let snapshot = eligibility.usability();
    eligibility.mark_recording_abandoned();
    eligibility.mark_recording_abandoned();
    assert_eq!(snapshot, ContextUsability::Usable);
    assert_eq!(
        eligibility.usability(),
        ContextUsability::Unusable(TerminalCause::RecordingAbandoned)
    );
    for operation in [
        Operation::Upscale,
        Operation::GenerateReactiveMask,
        Operation::Tuning,
        Operation::ConfigureContextDebugLegacy,
    ] {
        for code in [0, 3, u32::MAX] {
            assert_result!(
                eligibility.record(operation, code),
                Err(Error::ContextUnusable(TerminalCause::RecordingAbandoned))
            );
        }
    }
    let error = eligibility.check().unwrap_err();
    let cause = error
        .source()
        .unwrap()
        .downcast_ref::<TerminalCause>()
        .unwrap();
    assert_eq!(*cause, TerminalCause::RecordingAbandoned);
    assert!(cause.source().is_none());
    assert!(error.to_string().contains("abandonment"));
}
