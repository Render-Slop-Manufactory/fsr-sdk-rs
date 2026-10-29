// SPDX-License-Identifier: MPL-2.0

use super::*;

fn options() -> UpscalerOptions {
    use crate::upscaler::common::Dimensions;
    UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    )
}

#[test]
fn enumeration_pairs_outputs_copies_names_and_accepts_shrinkage() {
    let device = std::ptr::dangling_mut::<c_void>();
    let mut name = *b"first\0";
    let mut calls = 0;
    // SAFETY: The controlled invocation writes bounded, initialized output and
    // returns a readable local name that outlives immediate copying.
    let result = unsafe {
        providers(device, |header| {
            calls += 1;
            let desc = &mut *header.cast::<ffxQueryDescGetVersions>();
            assert_eq!(desc.header.r#type, FFX_API_QUERY_DESC_TYPE_GET_VERSIONS);
            assert!(desc.header.pNext.is_null());
            assert_eq!(desc.device, device);
            assert_eq!(
                desc.createDescType,
                FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE
            );
            if calls == 1 {
                assert_eq!(*desc.outputCount, 0);
                assert!(desc.versionIds.is_null() && desc.versionNames.is_null());
                *desc.outputCount = 2;
            } else {
                assert_eq!(*desc.outputCount, 2);
                assert_eq!(*desc.versionIds.add(1), 0);
                assert!((*desc.versionNames.add(1)).is_null());
                *desc.versionIds = 42;
                *desc.versionNames = name.as_ptr().cast();
                *desc.outputCount = 1;
            }
            FFX_API_RETURN_OK
        })
    }
    .unwrap();
    name.copy_from_slice(b"later\0");
    assert_eq!(calls, 2);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id(), ProviderId(42));
    assert_eq!(result[0].name(), "first");
}

#[test]
fn enumeration_empty_truncated_and_invalid_capacity_are_distinct() {
    let mut calls = 0;
    // SAFETY: Count-only fixture returns an empty enumeration and retains nothing.
    let empty = unsafe {
        providers(null_mut(), |_| {
            calls += 1;
            0
        })
    }
    .unwrap();
    assert!(empty.is_empty());
    assert_eq!(calls, 1);
    for overflow in [false, true] {
        let mut calls = 0;
        // SAFETY: Only one allocated slot is written; overflow changes the
        // reported count without actually writing beyond its capacity.
        let result = unsafe {
            providers(null_mut(), |header| {
                calls += 1;
                let desc = &mut *header.cast::<ffxQueryDescGetVersions>();
                if calls == 1 {
                    *desc.outputCount = 1;
                } else {
                    *desc.versionIds = 9;
                    *desc.versionNames = c"provider".as_ptr();
                    let capacity = *desc.outputCount;
                    // The provider list grew to two entries. A conforming fill
                    // reports only the one slot it could return; no retry follows.
                    *desc.outputCount = if overflow { 2 } else { capacity.min(2) };
                }
                0
            })
        };
        if overflow {
            assert_result!(
                result,
                Err(Error::invariant(
                    crate::error::Operation::Query,
                    InvariantViolation::QueryCountExceedsCapacity
                ))
            );
        } else {
            assert_eq!(result.unwrap().len(), 1);
        }
        assert_eq!(calls, 2);
    }
}

#[test]
fn enumeration_rejects_impossible_allocation_before_fill() {
    let mut calls = 0;
    // SAFETY: Controlled count-only fixture; allocation fails before any fill.
    let result = unsafe {
        providers(null_mut(), |header| {
            calls += 1;
            let desc = &mut *header.cast::<ffxQueryDescGetVersions>();
            *desc.outputCount = u64::MAX;
            0
        })
    };
    let Error::QueryAllocation(error) = result.unwrap_err() else {
        panic!("wrong category")
    };
    assert_eq!(error.count(), u64::MAX);
    assert_eq!(
        error.reason(),
        if usize::try_from(u64::MAX).is_ok() {
            crate::error::QueryAllocationReason::ReservationFailed
        } else {
            crate::error::QueryAllocationReason::CountNotRepresentable
        }
    );
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(calls, 1);
}

#[test]
fn memory_estimate_forwards_device_maxima_zero_flags_and_exact_bytes() {
    let device = std::ptr::dangling_mut::<c_void>();
    // SAFETY: Controlled synchronous fixture; output is initialized and retained
    // only for this call, and the dummy device is recorded without dereferencing.
    let result = unsafe {
        estimate_memory(device, &options(), |header| {
            let desc = &mut *header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
            assert_eq!(
                desc.header.r#type,
                FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2
            );
            assert!(desc.header.pNext.is_null());
            assert_eq!(desc.device, device);
            assert_eq!(
                (desc.maxRenderSize.width, desc.maxRenderSize.height),
                (320, 180)
            );
            assert_eq!(
                (desc.maxUpscaleSize.width, desc.maxUpscaleSize.height),
                (640, 360)
            );
            assert_eq!(desc.flags, 0);
            assert_eq!((*desc.gpuMemoryUsageUpscaler).totalUsageInBytes, 0);
            *desc.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage {
                totalUsageInBytes: 1 << 40,
                aliasableUsageInBytes: 1 << 41,
            };
            0
        })
    }
    .unwrap();
    assert_eq!(result.total_bytes(), 1 << 40);
    assert_eq!(result.aliasable_bytes(), 1 << 41);
}

#[test]
fn memory_estimate_uses_the_selected_profile_creation_flags() {
    use crate::upscaler::{DepthConvention, MotionVectorJitter, MotionVectorResolution};
    for (depth, depth_flags) in [
        (DepthConvention::StandardFinite, 0),
        (DepthConvention::ReversedFinite, 8),
        (DepthConvention::StandardInfinite, 16),
        (DepthConvention::ReversedInfinite, 24),
    ] {
        for (resolution, resolution_flags) in [
            (MotionVectorResolution::Render, 0),
            (MotionVectorResolution::Display, 2),
        ] {
            for (jitter, jitter_flags) in [
                (MotionVectorJitter::Excluded, 0),
                (MotionVectorJitter::Included, 4),
            ] {
                let mut selected = options();
                selected.set_depth_convention(depth);
                selected.set_motion_vector_resolution(resolution);
                selected.set_motion_vector_jitter(jitter);
                // SAFETY: Fixture reads only this live descriptor and synchronously
                // writes its initialized output; no device is used.
                let result = unsafe {
                    estimate_memory(null_mut(), &selected, |header| {
                        let desc = &mut *header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                        assert_eq!(desc.flags, depth_flags | resolution_flags | jitter_flags);
                        0
                    })
                };
                assert_eq!(result.unwrap().total_bytes(), 0);
            }
        }
    }
}

#[test]
fn live_results_copy_lossy_names_and_preserve_unknown_overlapping_bits_and_zero_memory() {
    let mut name = [b'A', 0xff, 0];
    // SAFETY: The fixture returns local readable, terminated name bytes through
    // copying and writes the matched live-provider descriptor synchronously.
    let info = unsafe {
        provider(Eligibility::Usable, |header| {
            let desc = &mut *header.cast::<ffxQueryGetProviderVersion>();
            assert_eq!(
                desc.header.r#type,
                FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION
            );
            assert!(desc.header.pNext.is_null());
            desc.versionId = 99;
            desc.versionName = name.as_ptr().cast();
            0
        })
    }
    .unwrap();
    name[0] = b'B';
    assert_eq!(name[0], b'B');
    assert_eq!(info.name(), "A\u{fffd}");
    assert_eq!(info.id(), ProviderId(99));
    // SAFETY: The matched fixture writes only descriptor fields, retaining nothing.
    let masks = unsafe {
        resource_requirements(Eligibility::Usable, |header| {
            let desc = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
            assert_eq!(
                desc.header.r#type,
                FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS
            );
            assert!(desc.header.pNext.is_null());
            desc.required_resources = (1 << 63) | 3;
            desc.optional_resources = u64::MAX;
            0
        })
    }
    .unwrap();
    assert!(masks.required().contains(ResourceFlags::COLOR));
    assert!(masks.required().contains(ResourceFlags::DEPTH));
    assert_eq!(masks.required().unknown_bits(), 1 << 63);
    assert_eq!(masks.optional().bits(), u64::MAX);
    // SAFETY: Matched fixture writes the initialized output structure synchronously.
    let usage = unsafe {
        gpu_memory_usage(Eligibility::Usable, |header| {
            let desc = &mut *header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsage>();
            assert_eq!(
                desc.header.r#type,
                FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE
            );
            assert!(desc.header.pNext.is_null());
            *desc.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage::default();
            0
        })
    }
    .unwrap();
    assert_eq!((usage.total_bytes(), usage.aliasable_bytes()), (0, 0));
}

#[test]
fn invalid_successful_provider_outputs_are_errors() {
    for (id, name, reason) in [
        (
            0,
            c"name".as_ptr(),
            InvariantViolation::ZeroProviderIdOnSuccess,
        ),
        (1, null(), InvariantViolation::NullProviderNameOnSuccess),
    ] {
        // SAFETY: The matched fixture returns either null or a static valid name.
        let result = unsafe {
            provider(Eligibility::Usable, |header| {
                let desc = &mut *header.cast::<ffxQueryGetProviderVersion>();
                desc.versionId = id;
                desc.versionName = name;
                0
            })
        };
        assert_result!(
            result,
            Err(Error::invariant(crate::error::Operation::Query, reason))
        );
    }
    let mut calls = 0;
    // SAFETY: Initialized pointer arrays remain null, exercising success validation.
    let result = unsafe {
        providers(null_mut(), |header| {
            calls += 1;
            let desc = &mut *header.cast::<ffxQueryDescGetVersions>();
            *desc.outputCount = 1;
            0
        })
    };
    assert_eq!(calls, 2);
    assert_result!(
        result,
        Err(Error::invariant(
            crate::error::Operation::Query,
            InvariantViolation::NullProviderNameOnSuccess
        ))
    );
}

#[test]
fn all_non_ok_codes_are_preserved_without_interpreting_error_outputs() {
    for code in [1, 2, 3, 4, 5, 6, 7, 0xf123_4567] {
        let expected = Error::native(Operation::Query, code);
        // SAFETY: Error fixtures return synchronously without retaining pointers;
        // null names/default outputs must never be interpreted after a native error.
        unsafe {
            assert_result!(providers(null_mut(), |_| code), Err(expected));
            assert_result!(
                estimate_memory(null_mut(), &options(), |_| code),
                Err(expected)
            );
            assert_result!(provider(Eligibility::Usable, |_| code), Err(expected));
            assert_result!(
                gpu_memory_usage(Eligibility::Usable, |_| code),
                Err(expected)
            );
            assert_result!(
                resource_requirements(Eligibility::Usable, |header| {
                    let desc = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                    desc.required_resources = u64::MAX;
                    desc.optional_resources = u64::MAX;
                    code
                }),
                Err(expected)
            );
        }
    }
    let mut calls = 0;
    // SAFETY: Count succeeds; fill returns an error without writing outputs.
    let result = unsafe {
        providers(null_mut(), |header| {
            calls += 1;
            if calls == 1 {
                *(*header.cast::<ffxQueryDescGetVersions>()).outputCount = 1;
                0
            } else {
                0xf123_4567
            }
        })
    };
    assert_result!(result, Err(Error::native(Operation::Query, 0xf123_4567)));
}

#[test]
fn all_live_queries_reject_poison_before_invocation() {
    let mut calls = 0;
    let mut invoke = |_| {
        calls += 1;
        0
    };
    // SAFETY: Poison must prevent these callbacks from running at all.
    unsafe {
        assert_result!(
            provider(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            resource_requirements(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
                &mut invoke
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            gpu_memory_usage(
                crate::context_policy::failed(crate::error::Operation::Upscale, 3),
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
fn automatic_exposure_flags_and_provider_requirements_are_checked() {
    use crate::upscaler::{
        DepthConvention, ExposureMode, MotionVectorJitter, MotionVectorResolution,
        common::check_resource_requirements,
    };
    let mut selected = options();
    selected.set_exposure_mode(ExposureMode::Automatic);
    for (combined, flags) in [(false, 32), (true, 62)] {
        if combined {
            selected.set_depth_convention(DepthConvention::ReversedInfinite);
            selected.set_motion_vector_resolution(MotionVectorResolution::Display);
            selected.set_motion_vector_jitter(MotionVectorJitter::Included);
        }
        // SAFETY: Synchronous descriptor fixture with initialized output storage.
        unsafe {
            estimate_memory(null_mut(), &selected, |header| {
                assert_eq!(
                    (*header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>()).flags,
                    flags
                );
                0
            })
        }
        .unwrap();
    }
    for bits in [0, 7, 15, 16, 32, 1 << 63, u64::MAX] {
        // SAFETY: Fixture writes only this descriptor's matched scalar outputs.
        let result = unsafe {
            resource_requirements(Eligibility::Usable, |header| {
                let d = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                d.required_resources = bits;
                d.optional_resources = u64::MAX;
                0
            })
        }
        .unwrap();
        assert_result!(
            check_resource_requirements(result.required()),
            if bits & !63 == 0 {
                Ok(())
            } else {
                Err(Error::Input(
                    crate::error::InputError::UnsupportedResourceRequirements {
                        required: result.required(),
                    },
                ))
            }
        );
    }
}

#[test]
fn mask_requirements_require_presence_without_treating_consumption_as_permission() {
    use crate::{
        error::{DispatchValidation, ResourceRole},
        upscaler::common::check_mask_requirements,
    };
    for required in [0, 16, 32, 48] {
        for optional in [0, 16, 32, 48, u64::MAX] {
            // SAFETY: Synchronous controlled output, with complete matched storage.
            let requirements = unsafe {
                resource_requirements(Eligibility::Usable, |header| {
                    let d = &mut *header.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                    d.required_resources = required;
                    d.optional_resources = optional;
                    0
                })
            }
            .unwrap();
            for present in [0, 16, 32, 48] {
                let expected = [
                    (16, ResourceRole::ReactiveMask),
                    (32, ResourceRole::TransparencyAndCompositionMask),
                ]
                .into_iter()
                .find_map(|(bit, role)| {
                    if required & bit != 0 && present & bit == 0 {
                        Some(DispatchValidation::MissingRequiredResource(role))
                    } else {
                        None
                    }
                });
                assert_result!(
                    check_mask_requirements(requirements, present & 16 != 0, present & 32 != 0),
                    expected.map_or(Ok(()), |reason| Err(Error::Input(
                        crate::error::InputError::Dispatch(reason)
                    )))
                );
            }
        }
    }
}

#[test]
fn color_memory_estimates_forward_classification_independently_of_hdr() {
    use crate::upscaler::InputColorSpace;
    let mut selected = options();
    for (space, hdr, flags) in [
        (InputColorSpace::Linear, false, 0),
        (InputColorSpace::NonLinear, false, 256),
        (InputColorSpace::NonLinear, true, 257),
        (InputColorSpace::Linear, true, 1),
    ] {
        selected.set_input_color_space(space);
        selected.set_high_dynamic_range(hdr);
        // SAFETY: This synchronous fixture observes a complete descriptor and
        // its initialized output, without retaining or dereferencing a device.
        unsafe {
            estimate_memory(null_mut(), &selected, |header| {
                let d = &*header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                assert_eq!(d.flags, flags);
                0
            })
        }
        .unwrap();
    }
}

#[test]
fn hdr_memory_estimates_preserve_the_creation_flag_and_exposure_mode() {
    use crate::upscaler::ExposureMode;
    for (automatic, exposure_flags) in [(false, 0), (true, 32)] {
        let mut selected = options();
        if automatic {
            selected.set_exposure_mode(ExposureMode::Automatic);
        }
        for hdr in [false, true, false] {
            selected.set_high_dynamic_range(hdr);
            // SAFETY: The synchronous fixture only inspects the live descriptor;
            // its zero-initialized output is valid and no device is dereferenced.
            unsafe {
                estimate_memory(null_mut(), &selected, |header| {
                    let d = &*header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                    assert_eq!(d.flags, exposure_flags | u32::from(hdr));
                    0
                })
            }
            .unwrap();
        }
    }
}

#[test]
fn dynamic_resolution_memory_estimates_use_creation_flags_and_maxima() {
    use crate::upscaler::{
        DepthConvention, ExposureMode, MotionVectorJitter, MotionVectorResolution,
    };
    let mut selected = options();
    for (dynamic, combined, flags) in [
        (false, false, 0),
        (true, false, 64),
        (true, true, 126),
        (false, true, 62),
    ] {
        selected.set_dynamic_resolution(dynamic);
        if combined {
            selected.set_depth_convention(DepthConvention::ReversedInfinite);
            selected.set_exposure_mode(ExposureMode::Automatic);
            selected.set_motion_vector_resolution(MotionVectorResolution::Display);
            selected.set_motion_vector_jitter(MotionVectorJitter::Included);
        }
        // SAFETY: Synchronous fixture reads a complete live descriptor and retains nothing.
        unsafe {
            estimate_memory(null_mut(), &selected, |header| {
                let d = &*header.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                assert_eq!(d.flags, flags);
                assert_eq!((d.maxRenderSize.width, d.maxRenderSize.height), (320, 180));
                assert_eq!(
                    (d.maxUpscaleSize.width, d.maxUpscaleSize.height),
                    (640, 360)
                );
                0
            })
        }
        .unwrap();
    }
}

#[test]
fn explicit_selection_is_a_fresh_observation_without_default_fallback() {
    let device = std::ptr::dangling_mut::<c_void>();
    // SAFETY: Default selection must make no native call.
    unsafe { require_provider(device, None, |_| panic!("default must not enumerate")) }.unwrap();
    for observed in [42, 43] {
        let mut calls = 0;
        // SAFETY: Complete count/fill fixture, one readable static name and one
        // ID slot, synchronous without retention or real device access.
        let result = unsafe {
            require_provider(device, Some(ProviderId(42)), |header| {
                calls += 1;
                let d = &mut *header.cast::<ffxQueryDescGetVersions>();
                assert_eq!(d.device, device);
                assert!(d.header.pNext.is_null());
                if *d.outputCount != 0 {
                    *d.versionIds = observed;
                    *d.versionNames = c"fixture".as_ptr();
                }
                *d.outputCount = 1;
                0
            })
        };
        assert_eq!(calls, 2);
        assert_result!(
            result,
            if observed == 42 {
                Ok(())
            } else {
                Err(Error::Input(
                    crate::error::InputError::ProviderNotEnumerated {
                        requested: ProviderId(42),
                    },
                ))
            }
        );
    }
    // SAFETY: An ordinary error writes no outputs; no fallback/retry is valid.
    let failure = unsafe { require_provider(device, Some(ProviderId(42)), |_| 0xf1234567) };
    assert_result!(failure, Err(Error::native(Operation::Query, 0xf1234567)));
    assert!(provider_override(None).is_none());
    let node = provider_override(Some(ProviderId(42))).unwrap();
    assert_eq!(node.header.r#type, FFX_API_DESC_TYPE_OVERRIDE_VERSION);
    assert!(node.header.pNext.is_null());
    assert_eq!(node.versionId, 42);
    assert_eq!(crate::query::ProviderSelection::default().optional(), None);
    assert_eq!(
        crate::query::ProviderSelection::Explicit(ProviderId(42)).optional(),
        Some(ProviderId(42))
    );
}

#[test]
fn numeric_zero_and_maximum_ids_are_explicit_and_still_require_membership() {
    for value in [0, u64::MAX] {
        let id = ProviderId::from_u64(value);
        let request = crate::query::ProviderSelection::Explicit(id).optional();
        assert_eq!(request, Some(id));
        let node = provider_override(request).unwrap();
        assert_eq!(node.versionId, value);
        let mut calls = 0;
        // SAFETY: Complete synchronous enumeration output with one static name;
        // no device dereference, native work or retained descriptor pointer.
        let result = unsafe {
            require_provider(null_mut(), request, |header| {
                calls += 1;
                let desc = &mut *header.cast::<ffxQueryDescGetVersions>();
                if !desc.versionIds.is_null() {
                    *desc.versionIds = 42;
                    *desc.versionNames = c"fixture".as_ptr();
                }
                *desc.outputCount = 1;
                0
            })
        };
        assert_eq!(calls, 2);
        assert_result!(
            result,
            Err(Error::Input(
                crate::error::InputError::ProviderNotEnumerated { requested: id }
            ))
        );
    }
}
