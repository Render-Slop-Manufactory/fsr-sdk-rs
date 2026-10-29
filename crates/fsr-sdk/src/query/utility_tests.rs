// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::error::Operation;
use fsr_sdk_sys::api::FFX_API_RETURN_OK;

fn size(width: u32, height: u32) -> Dimensions {
    Dimensions::new(width, height).unwrap()
}
fn phase() -> JitterPhaseCount {
    JitterPhaseCount::new(20).unwrap()
}
fn invariant(reason: InvariantViolation) -> Error {
    Error::invariant(crate::error::Operation::Query, reason)
}

#[test]
fn utility_descriptors_forward_inputs_and_provider_outputs_exactly() {
    for (mode, ordinal) in [
        (QualityMode::NativeAa, 0),
        (QualityMode::Quality, 1),
        (QualityMode::Balanced, 2),
        (QualityMode::Performance, 3),
        (QualityMode::UltraPerformance, 4),
    ] {
        // SAFETY: Controlled callbacks synchronously inspect matched live descriptors
        // and write only supplied outputs. No pointers are retained.
        unsafe {
            let ratio = upscale_ratio(Eligibility::Usable, mode, |header| {
                let d = &mut *header.cast::<ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode>();
                assert_eq!(d.header.r#type, 0x10002);
                assert!(d.header.pNext.is_null());
                assert_eq!(d.qualityMode, ordinal);
                assert!((*d.pOutUpscaleRatio).is_nan());
                *d.pOutUpscaleRatio = 1.75;
                FFX_API_RETURN_OK
            })
            .unwrap();
            assert_eq!(ratio.to_bits(), 1.75f32.to_bits());
            let render =
                render_size_for_quality(Eligibility::Usable, size(1921, 1081), mode, |header| {
                    let d = &mut *header
                        .cast::<ffxQueryDescUpscaleGetRenderResolutionFromQualityMode>();
                    assert_eq!(d.header.r#type, 0x10003);
                    assert!(d.header.pNext.is_null());
                    assert_eq!(
                        (d.displayWidth, d.displayHeight, d.qualityMode),
                        (1921, 1081, ordinal)
                    );
                    assert_eq!((*d.pOutRenderWidth, *d.pOutRenderHeight), (0, 0));
                    *d.pOutRenderWidth = 1101;
                    *d.pOutRenderHeight = 619;
                    0
                })
                .unwrap();
            assert_eq!(render, size(1101, 619));
        }
    }
    // SAFETY: As above; a deliberately non-formula result proves provider forwarding.
    unsafe {
        let count = jitter_phase_count(
            Eligibility::Usable,
            size(1280, 9),
            size(1920, 7),
            |header| {
                let d = &mut *header.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>();
                assert_eq!(d.header.r#type, 0x10004);
                assert!(d.header.pNext.is_null());
                assert_eq!((d.renderWidth, d.displayWidth), (1280, 1920));
                assert_eq!(*d.pOutPhaseCount, 0);
                *d.pOutPhaseCount = 20;
                0
            },
        )
        .unwrap();
        assert_eq!(count, phase());
        let offset = jitter_offset(Eligibility::Usable, 19, count, |header| {
            let d = &mut *header.cast::<ffxQueryDescUpscaleGetJitterOffset>();
            assert_eq!(d.header.r#type, 0x10005);
            assert!(d.header.pNext.is_null());
            assert_eq!((d.index, d.phaseCount), (19, 20));
            assert!((*d.pOutX).is_nan() && (*d.pOutY).is_nan());
            *d.pOutX = -0.0;
            *d.pOutY = -0.375;
            0
        })
        .unwrap();
        assert_eq!(offset.x().to_bits(), (-0.0f32).to_bits());
        assert_eq!(offset.y(), -0.375);
    }
}

#[test]
fn query_guards_reject_invalid_inputs_and_poison_without_native_entry() {
    let mut calls = 0;
    let mut invoke = |_| {
        calls += 1;
        0
    };
    // SAFETY: Rejections must happen before invoking the callback.
    unsafe {
        for display in [size(u32::MAX, 1), size(1, u32::MAX)] {
            assert_result!(
                render_size_for_quality(
                    Eligibility::Usable,
                    display,
                    QualityMode::NativeAa,
                    &mut invoke
                ),
                Err(invalid(QueryValidation::DimensionsOutOfRange))
            );
        }
        for (render, display, reason) in [
            (
                size(u32::MAX, 1),
                size(u32::MAX, 1),
                QueryValidation::DimensionsOutOfRange,
            ),
            (
                size(1, 1),
                size(u32::MAX, 1),
                QueryValidation::DimensionsOutOfRange,
            ),
            (size(640, 1), size(320, 1), QueryValidation::JitterScale),
            (size(1, 1), size(16384, 1), QueryValidation::JitterScale),
        ] {
            assert_result!(
                jitter_phase_count(Eligibility::Usable, render, display, &mut invoke),
                Err(invalid(reason))
            );
        }
        for index in [-1, i32::MIN, 20, i32::MAX] {
            assert_result!(
                jitter_offset(Eligibility::Usable, index, phase(), &mut invoke),
                Err(invalid(QueryValidation::JitterIndex))
            );
        }
        assert_result!(
            upscale_ratio(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                QualityMode::Quality,
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            render_size_for_quality(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                size(u32::MAX, 1),
                QualityMode::Quality,
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            jitter_phase_count(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                size(1, 1),
                size(16384, 1),
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            jitter_offset(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                -1,
                phase(),
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
    }
    assert_eq!(calls, 0);
}

#[test]
fn valid_numerical_boundaries_are_forwarded() {
    for width in [1, i32::MAX as u32] {
        // SAFETY: Matched fixture writes valid dimensions/count synchronously.
        unsafe {
            assert_eq!(
                render_size_for_quality(
                    Eligibility::Usable,
                    size(width, width),
                    QualityMode::NativeAa,
                    |header| {
                        let d = &mut *header
                            .cast::<ffxQueryDescUpscaleGetRenderResolutionFromQualityMode>();
                        *d.pOutRenderWidth = d.displayWidth;
                        *d.pOutRenderHeight = d.displayHeight;
                        0
                    }
                )
                .unwrap(),
                size(width, width)
            );
            assert_eq!(
                jitter_phase_count(
                    Eligibility::Usable,
                    size(width, u32::MAX),
                    size(width, u32::MAX),
                    |header| {
                        *(*header.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>())
                            .pOutPhaseCount = 1;
                        0
                    }
                )
                .unwrap()
                .count(),
                1
            );
        }
    }
    // SAFETY: The largest selected scale and index remain representable.
    unsafe {
        assert!(
            jitter_phase_count(Eligibility::Usable, size(1, 1), size(16383, 1), |header| {
                *(*header.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>()).pOutPhaseCount =
                    i32::MAX;
                0
            })
            .is_ok()
        );
        assert!(
            jitter_offset(
                Eligibility::Usable,
                i32::MAX - 1,
                JitterPhaseCount::new(i32::MAX).unwrap(),
                |header| {
                    let d = &mut *header.cast::<ffxQueryDescUpscaleGetJitterOffset>();
                    *d.pOutX = 0.0;
                    *d.pOutY = 0.0;
                    0
                }
            )
            .is_ok()
        );
    }
}

#[test]
fn invalid_or_missing_success_outputs_are_invariant_failures() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -0.0, -1.0] {
        // SAFETY: Controlled descriptor/result storage; no pointers retained.
        unsafe {
            assert_result!(
                upscale_ratio(Eligibility::Usable, QualityMode::Quality, |header| {
                    *(*header.cast::<ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode>())
                        .pOutUpscaleRatio = value;
                    0
                }),
                Err(invariant(InvariantViolation::InvalidUpscaleRatioOnSuccess))
            );
        }
    }
    for (width, height) in [(0, 0), (0, 100), (100, 0), (641, 360), (640, 361)] {
        // SAFETY: Fixture writes only the matched scalar outputs.
        unsafe {
            assert_result!(
                render_size_for_quality(
                    Eligibility::Usable,
                    size(640, 360),
                    QualityMode::Quality,
                    |header| {
                        let d = &mut *header
                            .cast::<ffxQueryDescUpscaleGetRenderResolutionFromQualityMode>();
                        *d.pOutRenderWidth = width;
                        *d.pOutRenderHeight = height;
                        0
                    }
                ),
                Err(invariant(InvariantViolation::InvalidRenderSizeOnSuccess))
            );
        }
    }
    for count in [0, -1, i32::MIN] {
        // SAFETY: Controlled scalar output; invalid value is never exposed as checked.
        unsafe {
            assert_result!(
                jitter_phase_count(
                    Eligibility::Usable,
                    size(320, 180),
                    size(640, 360),
                    |header| {
                        *(*header.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>())
                            .pOutPhaseCount = count;
                        0
                    }
                ),
                Err(invariant(
                    InvariantViolation::InvalidJitterPhaseCountOnSuccess
                ))
            );
        }
    }
    for (x, y) in [
        (f32::NAN, 0.0),
        (0.0, f32::NAN),
        (f32::INFINITY, 0.0),
        (0.0, f32::NEG_INFINITY),
    ] {
        // SAFETY: Controlled matched outputs; validation checks both components.
        unsafe {
            assert_result!(
                jitter_offset(Eligibility::Usable, 0, phase(), |header| {
                    let d = &mut *header.cast::<ffxQueryDescUpscaleGetJitterOffset>();
                    *d.pOutX = x;
                    *d.pOutY = y;
                    0
                }),
                Err(invariant(InvariantViolation::InvalidJitterOffsetOnSuccess))
            );
        }
    }
    // SAFETY: Success without any output writes must also fail.
    unsafe {
        assert_result!(
            upscale_ratio(Eligibility::Usable, QualityMode::Quality, |_| 0),
            Err(invariant(InvariantViolation::InvalidUpscaleRatioOnSuccess))
        );
        assert_result!(
            render_size_for_quality(
                Eligibility::Usable,
                size(640, 360),
                QualityMode::Quality,
                |_| 0
            ),
            Err(invariant(InvariantViolation::InvalidRenderSizeOnSuccess))
        );
        assert_result!(
            jitter_phase_count(Eligibility::Usable, size(320, 180), size(640, 360), |_| 0),
            Err(invariant(
                InvariantViolation::InvalidJitterPhaseCountOnSuccess
            ))
        );
        assert_result!(
            jitter_offset(Eligibility::Usable, 0, phase(), |_| 0),
            Err(invariant(InvariantViolation::InvalidJitterOffsetOnSuccess))
        );
    }
}

#[test]
fn utility_errors_preserve_all_native_codes_before_output_interpretation() {
    for code in [1, 2, 3, 4, 5, 6, 7, u32::MAX] {
        let expected = Error::native(Operation::Query, code);
        // SAFETY: Error callbacks do not retain/write outputs; sentinels must not
        // override the native error with an invariant failure.
        unsafe {
            assert_result!(
                upscale_ratio(Eligibility::Usable, QualityMode::Quality, |_| code),
                Err(expected)
            );
            assert_result!(
                render_size_for_quality(
                    Eligibility::Usable,
                    size(640, 360),
                    QualityMode::Quality,
                    |_| code
                ),
                Err(expected)
            );
            assert_result!(
                jitter_phase_count(Eligibility::Usable, size(320, 180), size(640, 360), |_| {
                    code
                }),
                Err(expected)
            );
            assert_result!(
                jitter_offset(Eligibility::Usable, 0, phase(), |_| code),
                Err(expected)
            );
        }
    }
}
