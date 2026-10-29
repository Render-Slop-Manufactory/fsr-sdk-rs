// SPDX-License-Identifier: MPL-2.0

//! Probe layout only; independent pinned-header C++ assertions are separate.
#[path = "native/reactive_mask_descriptor.rs"]
mod descriptor;

#[test]
#[cfg(target_pointer_width = "64")]
fn reactive_probe_layout_and_flag_inventory() {
    use descriptor::*;
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!(
        (
            size_of::<GenerateReactive>(),
            align_of::<GenerateReactive>()
        ),
        (192, 8)
    );
    assert_eq!(
        [
            offset_of!(GenerateReactive, header),
            offset_of!(GenerateReactive, command_list),
            offset_of!(GenerateReactive, color_opaque_only),
            offset_of!(GenerateReactive, color_pre_upscale),
            offset_of!(GenerateReactive, out_reactive),
            offset_of!(GenerateReactive, render_size),
            offset_of!(GenerateReactive, scale),
            offset_of!(GenerateReactive, cutoff_threshold),
            offset_of!(GenerateReactive, binary_value),
            offset_of!(GenerateReactive, flags)
        ],
        [0, 16, 24, 72, 120, 168, 176, 180, 184, 188]
    );
    assert_eq!(GENERATE_REACTIVE, 0x10006);
    assert_eq!(FLAGS, [1, 2, 4, 8]);
    assert_eq!(FLAGS.into_iter().fold(0, |all, flag| all | flag), 15);
}
