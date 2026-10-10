// SPDX-License-Identifier: MPL-2.0

//! Locally compiled runtime fixture and minimal ownership-only COM device.
use super::super::*;
use std::{cell::Cell, path::Path, process::Command};
use windows::core::{GUID, HRESULT, IUnknown_Vtbl};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct FixtureSnapshot {
    pub(super) creates: u32,
    pub(super) destroys: u32,
    pub(super) root_tag: u64,
    pub(super) version_tag: u64,
    pub(super) backend_tag: u64,
    pub(super) version: u32,
    pub(super) flags: u32,
    pub(super) render_width: u32,
    pub(super) render_height: u32,
    pub(super) upscale_width: u32,
    pub(super) upscale_height: u32,
    pub(super) device: usize,
    pub(super) chain_valid: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DispatchObservation {
    pub(super) calls: u32,
    pub(super) tag: u64,
}

#[repr(C)]
pub(super) struct FakeDevice {
    pub(super) vtable: *const IUnknown_Vtbl,
    pub(super) references: Cell<u32>,
}

unsafe extern "system" fn query_interface(
    _: *mut std::ffi::c_void,
    _: *const GUID,
    out: *mut *mut std::ffi::c_void,
) -> HRESULT {
    // SAFETY: COM callers provide writable output; this test never queries.
    unsafe { *out = null_mut() };
    HRESULT(0x8000_4002u32 as i32)
}

unsafe extern "system" fn add_ref(this: *mut std::ffi::c_void) -> u32 {
    // SAFETY: The test owns a leaked FakeDevice with this vtable.
    let fake = unsafe { &*this.cast::<FakeDevice>() };
    let next = fake.references.get() + 1;
    fake.references.set(next);
    next
}

unsafe extern "system" fn release(this: *mut std::ffi::c_void) -> u32 {
    // SAFETY: The test owns a leaked FakeDevice with this vtable.
    let fake = unsafe { &*this.cast::<FakeDevice>() };
    let next = fake.references.get() - 1;
    fake.references.set(next);
    next
}

static FAKE_VTABLE: IUnknown_Vtbl = IUnknown_Vtbl {
    QueryInterface: query_interface,
    AddRef: add_ref,
    Release: release,
};

pub(super) fn fake_device() -> (ID3D12Device, &'static FakeDevice) {
    let fake: &'static FakeDevice = Box::leak(Box::new(FakeDevice {
        vtable: &FAKE_VTABLE,
        references: Cell::new(1),
    }));
    // SAFETY: This test-only COM stub provides IUnknown ownership methods.
    // The fixture records the pointer but never calls ID3D12Device methods.
    let device = unsafe { ID3D12Device::from_raw(std::ptr::from_ref(fake).cast_mut().cast()) };
    (device, fake)
}

pub(super) fn fixture_runtime() -> (Runtime, libloading::Library) {
    let (path, observer) = fixture_library();
    // SAFETY: The locally compiled fixture has exactly the tested signatures.
    let runtime = unsafe { Runtime::load_unchecked(&path) }.unwrap();
    (runtime, observer)
}

#[cfg(feature = "movable")]
pub(super) fn fixture_movable_runtime() -> (crate::runtime::MovableRuntime, libloading::Library) {
    let (path, observer) = fixture_library();
    // SAFETY: Controlled fixture; these tests serialize every native operation
    // and cleanup. It has no hidden workers or thread affinity and records no GPU work.
    let runtime = unsafe { crate::runtime::MovableRuntime::load_unchecked(&path) }.unwrap();
    (runtime, observer)
}

fn fixture_library() -> (std::path::PathBuf, libloading::Library) {
    let dir = crate::test_directory::directory("fsr-sdk-upscaler");
    let path = dir.join(format!("fixture{}", std::env::consts::DLL_SUFFIX));
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=cdylib",
            "--crate-name=upscaler_fixture",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/upscaler.rs"))
        .arg("-o")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // SAFETY: The locally compiled fixture has exactly the tested signatures.
    let observer = unsafe { libloading::Library::new(&path) }.unwrap();
    (path, observer)
}

pub(super) fn fixture_set(
    observer: &libloading::Library,
    create: u32,
    destroy: u32,
    null_create: u32,
) {
    // SAFETY: The fixture exports this exact function.
    unsafe {
        let set = observer
            .get::<extern "C" fn(u32, u32, u32)>(c"fixture_set_outcome")
            .unwrap();
        set(create, destroy, null_create);
    }
}

pub(super) fn fixture_snapshot(observer: &libloading::Library) -> FixtureSnapshot {
    // SAFETY: The fixture exports this exact function and matching C struct.
    unsafe {
        let snapshot = observer
            .get::<extern "C" fn() -> FixtureSnapshot>(c"fixture_snapshot")
            .unwrap();
        snapshot()
    }
}

pub(super) fn sizes() -> UpscalerOptions {
    UpscalerOptions::new(
        Dimensions::new(1280, 720).unwrap(),
        Dimensions::new(1920, 1080).unwrap(),
    )
}
