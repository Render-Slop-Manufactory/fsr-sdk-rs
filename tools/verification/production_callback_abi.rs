// SPDX-License-Identifier: MPL-2.0

//! Production raw CB return ABI exchange with the independent MSVC driver.

use core::ffi::c_void;

#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[path = "../../crates/fsr-sdk-sys/src/api.rs"]
mod api;
use api::{
    FfxApiResource, FfxApiResourceDepth, FfxApiResourceDescription, FfxApiResourceHeight,
    FfxApiResourceWidth,
};

use api::FfxApiConstantBufferAllocation as ConstantBufferAllocation;

#[unsafe(no_mangle)]
pub extern "C" fn callback_layout(index: u32) -> u64 {
    use core::mem::{align_of, offset_of, size_of};
    let values = [
        size_of::<ConstantBufferAllocation>(),
        align_of::<ConstantBufferAllocation>(),
        offset_of!(ConstantBufferAllocation, handle),
        size_of::<FfxApiResource>(),
        align_of::<FfxApiResource>(),
        offset_of!(FfxApiResource, description),
        offset_of!(FfxApiResource, state),
    ];
    values.get(index as usize).copied().unwrap_or(usize::MAX) as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn callback_return(data: *mut c_void, size: u64) -> ConstantBufferAllocation {
    ConstantBufferAllocation {
        resource: FfxApiResource {
            resource: data,
            description: FfxApiResourceDescription {
                r#type: 0x1000_0001,
                format: 0x2000_0002,
                width: FfxApiResourceWidth { width: 0x3000_0003 },
                height: FfxApiResourceHeight {
                    height: 0x4000_0004,
                },
                depth: FfxApiResourceDepth { depth: 0x5000_0005 },
                mipCount: 0x6000_0006,
                flags: 0x7000_0007,
                usage: 0x8000_0008,
            },
            state: 0x9000_0009,
        },
        handle: 0xf0e1_d2c3_b4a5_9687 ^ size,
    }
}

/// # Safety
/// Supply a valid C callback which returns normally and accepts these values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn callback_invoke(
    callback: unsafe extern "C" fn(*mut c_void, u64) -> ConstantBufferAllocation,
    data: *mut c_void,
    size: u64,
) -> u32 {
    // SAFETY: The independent C++ driver supplies its static callback and valid
    // test payload. Neither callback dereferences data or interprets size as a slice.
    let got = unsafe { callback(data, size) };
    let d = got.resource.description;
    // SAFETY: All union alternatives are u32 with identical representation.
    u32::from(unsafe {
        got.resource.resource == data
            && got.resource.state == 0x9000_0009
            && d.r#type == 0x1000_0001
            && d.format == 0x2000_0002
            && d.width.width == 0x3000_0003
            && d.height.height == 0x4000_0004
            && d.depth.depth == 0x5000_0005
            && d.mipCount == 0x6000_0006
            && d.flags == 0x7000_0007
            && d.usage == 0x8000_0008
            && got.handle == (0xf0e1_d2c3_b4a5_9687 ^ size)
    })
}
