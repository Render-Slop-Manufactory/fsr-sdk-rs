// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    context_policy::{self, Eligibility},
    error::Operation,
};

fn resource(pointer: usize, output: bool) -> FfxApiResource {
    FfxApiResource {
        resource: pointer as *mut c_void,
        description: FfxApiResourceDescription {
            r#type: FFX_API_RESOURCE_TYPE_TEXTURE2D,
            format: if output {
                FFX_API_SURFACE_FORMAT_R8_UNORM
            } else {
                FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT
            },
            width: FfxApiResourceWidth { width: 1280 },
            height: FfxApiResourceHeight { height: 720 },
            depth: FfxApiResourceDepth { depth: 1 },
            mipCount: 1,
            flags: 0,
            usage: if output {
                FFX_API_RESOURCE_USAGE_UAV
            } else {
                FFX_API_RESOURCE_USAGE_READ_ONLY
            },
        },
        state: if output {
            FFX_API_RESOURCE_STATE_UNORDERED_ACCESS
        } else {
            FFX_API_RESOURCE_STATE_COMPUTE_READ
        },
    }
}

#[test]
fn all_flags_exact_scalar_bits_and_active_region_preserve_allocation_metadata() {
    for flags in 0..16 {
        let mut options = ReactiveMaskOptions::new(-0.0, -2.0, f32::MAX).unwrap();
        options.set_apply_tonemap(flags & 1 != 0);
        options.set_apply_inverse_tonemap(flags & 2 != 0);
        options.set_apply_threshold(flags & 4 != 0);
        options.set_use_components_max(flags & 8 != 0);
        for (width, height) in [(1280, 720), (1, 1), (960, 540), (1280, 720)] {
            let desc = build_descriptor(
                4usize as *mut c_void,
                [resource(1, false), resource(1, false), resource(3, true)],
                Dimensions::new(width, height).unwrap(),
                &options,
            );
            assert_eq!(desc.header.r#type, 0x10006);
            assert!(desc.header.pNext.is_null());
            assert_eq!(desc.commandList as usize, 4);
            assert_eq!(desc.flags, flags);
            assert_eq!(
                [
                    desc.scale.to_bits(),
                    desc.cutoffThreshold.to_bits(),
                    desc.binaryValue.to_bits()
                ],
                [(-0.0f32).to_bits(), (-2.0f32).to_bits(), f32::MAX.to_bits()]
            );
            assert_eq!(
                (desc.renderSize.width, desc.renderSize.height),
                (width, height)
            );
            for (actual, expected) in [
                (desc.colorOpaqueOnly, resource(1, false)),
                (desc.colorPreUpscale, resource(1, false)),
                (desc.outReactive, resource(3, true)),
            ] {
                assert_eq!(actual.resource, expected.resource);
                assert_eq!(actual.description.r#type, expected.description.r#type);
                assert_eq!(actual.description.format, expected.description.format);
                assert_eq!(actual.description.mipCount, 1);
                assert_eq!(actual.description.flags, 0);
                assert_eq!(actual.description.usage, expected.description.usage);
                assert_eq!(actual.state, expected.state);
                // SAFETY: Production mapping retains texture dimension union members.
                unsafe {
                    assert_eq!(
                        (
                            actual.description.width.width,
                            actual.description.height.height,
                            actual.description.depth.depth
                        ),
                        (1280, 720, 1)
                    );
                }
            }
        }
    }
}

#[test]
fn bounded_regions_reject_per_axis_and_shared_policy_rejects_before_inspection() {
    let maximum = Dimensions::new(1280, 720).unwrap();
    for size in [(1280, 720), (1, 1), (960, 540)] {
        assert_result!(
            check_region(Dimensions::new(size.0, size.1).unwrap(), maximum),
            Ok(())
        );
    }
    let invalid = Dimensions::new(1281, 720).unwrap();
    let error = Error::Input(crate::error::InputError::Dispatch(
        DispatchValidation::ActiveRenderSize,
    ));
    for size in [invalid, Dimensions::new(1280, 721).unwrap()] {
        assert_result!(check_region(size, maximum), Err(error));
    }
    let mut eligibility = Eligibility::Usable;
    assert_result!(
        context_policy::dispatch(
            crate::error::Operation::GenerateReactiveMask,
            &mut eligibility,
            &mut (),
            |_| check_region(invalid, maximum),
            |_, _| panic!("validation reached FFI")
        ),
        Err(error)
    );
    assert_eq!(eligibility, Eligibility::Usable);
    for code in [7, u32::MAX, 0x8000_1234] {
        eligibility = Eligibility::Usable;
        assert_result!(
            context_policy::dispatch(
                crate::error::Operation::GenerateReactiveMask,
                &mut eligibility,
                &mut (),
                |_| Ok(()),
                |_, _| code
            ),
            Err(Error::native(Operation::GenerateReactiveMask, code))
        );
        assert_eq!(
            eligibility,
            crate::context_policy::failed(crate::error::Operation::GenerateReactiveMask, code)
        );
    }
    for (mut eligibility, error) in [
        (
            crate::context_policy::failed(crate::error::Operation::GenerateReactiveMask, 3),
            crate::test_errors::terminal(Error::native(
                crate::error::Operation::GenerateReactiveMask,
                3,
            )),
        ),
        (
            crate::context_policy::failed(crate::error::Operation::Tuning, 3),
            crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 3)),
        ),
    ] {
        assert_result!(
            context_policy::dispatch(
                crate::error::Operation::GenerateReactiveMask,
                &mut eligibility,
                &mut (),
                |_| -> Result<(), Error> { panic!("poison reached inspection") },
                |_, _| panic!("poison reached FFI")
            ),
            Err(error)
        );
    }
}
