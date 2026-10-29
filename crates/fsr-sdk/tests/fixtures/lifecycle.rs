// SPDX-License-Identifier: MPL-2.0

//! Test protocol only, not AMD ABI descriptors. Built as a standalone cdylib.
use std::ffi::c_void;

#[repr(C)]
pub struct Input {
    pub tag: u64,
    pub next: *mut c_void,
    pub create_code: u32,
    pub destroy_code: u32,
    pub null_create: u32,
    pub clear_destroy: u32,
    pub event: extern "C" fn(*mut c_void, u32),
    pub data: *mut c_void,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxCreateContext(
    out: *mut *mut c_void,
    desc: *mut Input,
    allocator: *const c_void,
) -> u32 {
    // SAFETY: Fixture contract supplies valid input/output through the whole call.
    unsafe {
        if !allocator.is_null() || !(*out).is_null() {
            return 0xf0000001;
        }
        ((*desc).event)((*desc).data, 1);
        *out = if (*desc).null_create != 0 {
            std::ptr::null_mut()
        } else {
            desc.cast()
        };
        (*desc).create_code
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDestroyContext(out: *mut *mut c_void, allocator: *const c_void) -> u32 {
    // SAFETY: Only a successfully returned fixture handle is accepted here.
    unsafe {
        let desc = (*out).cast::<Input>();
        if !allocator.is_null() {
            return 0xf0000002;
        }
        ((*desc).event)((*desc).data, 2);
        if (*desc).clear_destroy != 0 {
            *out = std::ptr::null_mut();
        }
        (*desc).destroy_code
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxQuery(_: *mut *mut c_void, _: *mut c_void) -> u32 {
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxConfigure(context: *mut *mut c_void, desc: *const Dispatch) -> u32 {
    // SAFETY: Harness supplies this fixture-specific const descriptor. A live
    // context points at its retained Input; null selects the global path.
    unsafe {
        if (*desc).tag == 0x10007 {
            let tuning = &*desc.cast::<Tuning>();
            if context.is_null()
                || (*context).is_null()
                || !tuning.next.is_null()
                || tuning.key > 4
                || tuning.integer != 0
            {
                return 0xf0000005;
            }
            let input = (*context).cast::<Input>();
            ((*input).event)((*input).data, 20 + tuning.key as u32);
            if tuning.value.is_null() {
                ((*input).event)((*input).data, 30);
                return 0;
            }
            let value = *tuning.value;
            ((*input).event)((*input).data, value.to_bits());
            // Controlled rejection, not an AMD value-domain rule.
            return if value == f32::MAX { 0xa1234567 } else { 0 };
        }
        if (*desc).tag != 0xc028 || !(*desc).next.is_null() {
            return 0xf0000004;
        }
        if !context.is_null() {
            let input = (*context).cast::<Input>();
            ((*input).event)((*input).data, 6);
        }
        (*desc).code
    }
}

#[repr(C)]
struct Tuning {
    tag: u64,
    next: *mut c_void,
    key: u64,
    integer: u64,
    value: *const f32,
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDispatch(out: *mut *mut c_void, desc: *const Dispatch) -> u32 {
    // SAFETY: The retained handle is the original boxed Input. The harness
    // supplies the fixture-specific Dispatch for the duration of this call.
    unsafe {
        let input = (*out).cast::<Input>();
        ((*input).event)((*input).data, 5);
        if (*desc).tag == 0x10006 {
            let mask = &*desc.cast::<GenerateReactive>();
            if !mask.next.is_null() || mask.flags > 15 {
                return 0xf0000006;
            }
            for value in [
                mask.flags,
                mask.scale.to_bits(),
                mask.cutoff.to_bits(),
                mask.binary.to_bits(),
                mask.render[0],
                mask.render[1],
            ] {
                ((*input).event)((*input).data, value);
            }
            return if mask.scale == f32::MAX { 7 } else { 0 };
        }
        if (*desc).tag != 0xd013 || !(*desc).next.is_null() {
            return 0xf0000003;
        }
        (*desc).code
    }
}

// Independent fixture mirror of the v2.3.0 generator payload; no resource
// addresses are dereferenced. AMD-derived ABI: crate LICENSE-AMD.
#[repr(C)]
struct Resource {
    pointer: *mut c_void,
    description: [u32; 8],
    state: u32,
}
#[repr(C)]
struct GenerateReactive {
    tag: u64,
    next: *mut c_void,
    list: *mut c_void,
    resources: [Resource; 3],
    render: [u32; 2],
    scale: f32,
    cutoff: f32,
    binary: f32,
    flags: u32,
}

#[repr(C)]
pub struct Dispatch {
    pub tag: u64,
    pub next: *mut c_void,
    pub code: u32,
}
