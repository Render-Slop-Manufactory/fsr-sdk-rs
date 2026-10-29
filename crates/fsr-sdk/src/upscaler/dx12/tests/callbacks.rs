// SPDX-License-Identifier: MPL-2.0

//! Controlled foreign callers: payloads are tokens, never real COM allocations.
use super::super::interop::*;
use super::*;
use fsr_sdk_sys::dx12::*;
use std::{
    ffi::c_void,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering::SeqCst},
    },
};

static SERIAL: Mutex<()> = Mutex::new(());
static ERRORS: AtomicUsize = AtomicUsize::new(0);
static HOST_ADDRESS: AtomicUsize = AtomicUsize::new(0);
static CHAIN_ADDRESS: AtomicUsize = AtomicUsize::new(0);
static GROUPS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
fn check(value: bool) {
    if !value {
        ERRORS.fetch_add(1, SeqCst);
    }
}
unsafe extern "C" fn allocate(user: *mut c_void, size: u64) -> *mut c_void {
    check(user as usize == 0x1234 && size == 257);
    CALLS.fetch_or(1, SeqCst);
    user
}
unsafe extern "C" fn deallocate(user: *mut c_void, pointer: *mut c_void) {
    check(user as usize == 0x1234 && (pointer == user || pointer.is_null()));
    CALLS.fetch_or(2, SeqCst);
}
unsafe extern "C" fn resource_allocate(
    effect: u32,
    state: i32,
    heap: *const D3D12_HEAP_PROPERTIES,
    desc: *const D3D12_RESOURCE_DESC,
    ffx: *const FfxApiResourceDescription,
    clear: *const D3D12_CLEAR_VALUE,
    out: *mut *mut fsr_sdk_sys::dx12::ID3D12Resource,
) -> u32 {
    check(
        effect == 0xabcd
            && state == -17
            && heap.is_null()
            && desc.is_null()
            && ffx.is_null()
            && clear.is_null(),
    );
    // SAFETY: Controlled caller provides writable output; token is never dereferenced.
    unsafe {
        *out = 0x4567usize as *mut _;
    }
    CALLS.fetch_or(4, SeqCst);
    0xaaaa
}
unsafe extern "C" fn resource_free(
    effect: u32,
    pointer: *mut fsr_sdk_sys::dx12::ID3D12Resource,
) -> u32 {
    check(effect == 0xabcd && pointer as usize == 0x4567);
    CALLS.fetch_or(8, SeqCst);
    0xbbbb
}
unsafe extern "C" fn heap_allocate(
    effect: u32,
    desc: *const D3D12_HEAP_DESC,
    alias: bool,
    out: *mut *mut ID3D12Heap,
    offset: *mut u64,
) -> u32 {
    check(effect == 0xabcd && desc.is_null() && alias);
    // SAFETY: Controlled caller provides writable outputs, no COM operations.
    unsafe {
        *out = 0x6789usize as *mut _;
        *offset = 0x1234_5678_9012_3456;
    }
    CALLS.fetch_or(16, SeqCst);
    0xcccc
}
unsafe extern "C" fn heap_free(effect: u32, heap: *mut ID3D12Heap, offset: u64, size: u64) -> u32 {
    check(
        effect == 0xabcd
            && heap as usize == 0x6789
            && offset == 0x1234_5678_9012_3456
            && size == 0xfedc_ba98_7654_3210,
    );
    CALLS.fetch_or(32, SeqCst);
    0xdddd
}
unsafe extern "C" fn cb(data: *mut c_void, size: u64) -> FfxApiConstantBufferAllocation {
    check(data as usize == 0x1234 && size == 0xfedc_ba98_7654_3210);
    CALLS.fetch_or(64, SeqCst);
    // SAFETY: This aggregate contains only integers, unions thereof and pointers.
    let mut out: FfxApiConstantBufferAllocation = unsafe { std::mem::zeroed() };
    out.resource.resource = data;
    out.handle = size;
    out
}
unsafe extern "C" fn message(level: u32, text: *const u16) {
    check(level == 0x9876 && text.is_null());
    CALLS.fetch_or(128, SeqCst);
}
fn bundle(groups: usize) -> CreationCallbacks {
    let mut c = CreationCallbacks::default();
    if groups & 1 != 0 {
        c.set_host_allocator(Some(HostAllocationCallbacks::new(
            0x1234usize as *mut _,
            allocate,
            deallocate,
        )));
    }
    let mut d = Dx12AllocationCallbacks::default();
    if groups & 2 != 0 {
        d.set_resources(Some(ResourceAllocationCallbacks::new(
            resource_allocate,
            resource_free,
        )));
    }
    if groups & 4 != 0 {
        d.set_heaps(Some(HeapAllocationCallbacks::new(heap_allocate, heap_free)));
    }
    if groups & 8 != 0 {
        d.set_constant_buffer_allocator(Some(cb));
    }
    c.set_dx12_allocations(d);
    if groups & 16 != 0 {
        c.set_message_callback(Some(message));
    }
    c
}
unsafe extern "C" fn observe(root: *const ffxApiHeader, host: *const c_void, destroy: bool) {
    let groups = GROUPS.load(SeqCst);
    // SAFETY: Fixture forwards the complete pinned production chain/table.
    unsafe {
        let root = &*root.cast::<ffxCreateContextDescUpscale>();
        let version = &*root
            .header
            .pNext
            .cast::<ffxCreateContextDescUpscaleVersion>();
        let backend = &*version.header.pNext.cast::<ffxCreateBackendDX12Desc>();
        check(root.fpMessage.is_some() == (groups & 16 != 0));
        check(host.is_null() == (groups & 1 == 0));
        check(backend.header.pNext.is_null() == (groups & 14 == 0));
        if destroy {
            check(HOST_ADDRESS.load(SeqCst) == host as usize);
            check(CHAIN_ADDRESS.load(SeqCst) == std::ptr::from_ref(root) as usize);
            return;
        }
        HOST_ADDRESS.store(host as usize, SeqCst);
        CHAIN_ADDRESS.store(std::ptr::from_ref(root) as usize, SeqCst);
        if !host.is_null() {
            let h = &*host.cast::<ffxAllocationCallbacks>();
            let p = h.alloc.unwrap()(h.pUserData, 257);
            h.dealloc.unwrap()(h.pUserData, p);
        }
        if !backend.header.pNext.is_null() {
            let d = &*backend
                .header
                .pNext
                .cast::<ffxCreateBackendDX12AllocationCallbacksDesc>();
            check(d.header.r#type == 3 && d.header.pNext.is_null());
            check(d.pfnFfxResourceAllocator.is_some() == (groups & 2 != 0));
            check(d.pfnFfxResourceDeallocator.is_some() == (groups & 2 != 0));
            check(d.pfnFfxHeapAllocator.is_some() == (groups & 4 != 0));
            check(d.pfnFfxHeapDeallocator.is_some() == (groups & 4 != 0));
            check(d.pfnFfxConstantBufferAllocator.is_some() == (groups & 8 != 0));
            if let Some(a) = d.pfnFfxResourceAllocator {
                let mut p = null_mut();
                check(
                    a(
                        0xabcd,
                        -17,
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null(),
                        &mut p,
                    ) == 0xaaaa,
                );
                check(d.pfnFfxResourceDeallocator.unwrap()(0xabcd, p) == 0xbbbb);
            }
            if let Some(a) = d.pfnFfxHeapAllocator {
                let mut p = null_mut();
                let mut offset = 0;
                check(a(0xabcd, std::ptr::null(), true, &mut p, &mut offset) == 0xcccc);
                check(
                    d.pfnFfxHeapDeallocator.unwrap()(0xabcd, p, offset, 0xfedc_ba98_7654_3210)
                        == 0xdddd,
                );
            }
            if let Some(a) = d.pfnFfxConstantBufferAllocator {
                let out = a(0x1234usize as *mut _, 0xfedc_ba98_7654_3210);
                check(
                    out.resource.resource as usize == 0x1234 && out.handle == 0xfedc_ba98_7654_3210,
                );
            }
        }
        if let Some(m) = root.fpMessage {
            m(0x9876, std::ptr::null());
        }
    }
}
fn install(observer: &libloading::Library, groups: usize) {
    ERRORS.store(0, SeqCst);
    CALLS.store(0, SeqCst);
    GROUPS.store(groups, SeqCst);
    // SAFETY: Exact fixture export, static observer outlives every use.
    unsafe {
        observer
            .get::<unsafe extern "C" fn(
                Option<unsafe extern "C" fn(*const ffxApiHeader, *const c_void, bool)>,
            )>(c"fixture_set_create_observer")
            .unwrap()(Some(observe));
    }
}

#[test]
fn complete_and_omitted_groups_roundtrip_after_owner_moves() {
    let _serial = SERIAL.lock().unwrap();
    for groups in [0, 1, 2, 4, 8, 16, 31] {
        let (mut runtime, observer) = fixture_runtime();
        let (device, fake) = fake_device();
        install(&observer, groups);
        // SAFETY: Static controlled targets, serialized fixture with no GPU work.
        let owner = unsafe {
            runtime
                .upscaler_dx12(&device)
                .create_with_callbacks(&sizes(), bundle(groups))
        }
        .unwrap();
        let owner = Box::new(owner);
        drop(runtime);
        if groups % 2 == 0 {
            drop(owner);
        } else {
            owner.destroy().unwrap();
        }
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
        assert_eq!(fake.references.get(), 1);
        assert_eq!(ERRORS.load(SeqCst), 0);
        let expected = (if groups & 1 != 0 { 3 } else { 0 })
            | (if groups & 2 != 0 { 12 } else { 0 })
            | (if groups & 4 != 0 { 48 } else { 0 })
            | (if groups & 8 != 0 { 64 } else { 0 })
            | (if groups & 16 != 0 { 128 } else { 0 });
        assert_eq!(CALLS.load(SeqCst), expected);
    }
}

#[cfg(feature = "movable")]
#[test]
fn movable_callbacks_survive_destination_thread_cleanup() {
    let _serial = SERIAL.lock().unwrap();
    let (mut runtime, observer) = fixture_movable_runtime();
    let (device, fake) = fake_device();
    install(&observer, 31);
    // SAFETY: Static targets and serial fixture are thread-independent; join
    // precedes all later inspection, including the fake device's Cell counter.
    let owner = unsafe {
        runtime
            .upscaler_dx12(&device)
            .create_with_callbacks(&sizes(), bundle(31))
    }
    .unwrap();
    drop(runtime);
    std::thread::spawn(move || owner.destroy().unwrap())
        .join()
        .unwrap();
    assert_eq!(ERRORS.load(SeqCst), 0);
    assert_eq!(fake.references.get(), 1);
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}

#[test]
fn callback_failures_retain_dependencies_without_retry_or_failed_handle_use() {
    let _serial = SERIAL.lock().unwrap();
    macro_rules! family {($fixture:expr,$owner:ty,$downgrade:path)=>{{
    for (create, destroy, null, expected_destroys) in
        [(0xfedc1234, 0, 0, 0), (0, 0, 1, 0), (0, 0xfedc5678, 0, 1)]
    {
        let (mut runtime, observer) = $fixture;
        let weak=$downgrade(&runtime.state);
        let (device, fake) = fake_device();
        install(&observer, 31);
        fixture_set(&observer, create, destroy, null);
        // SAFETY: Targets remain static even when the wrapper retains its bundle.
        let result =
            unsafe { runtime.upscaler_dx12(&device).create_with_callbacks(&sizes(), bundle(31)) };
        if create != 0 {
            assert!(
                matches!(result,Err(Error::Native(crate::error::NativeError { operation: crate::error::Operation::Create, code: code })) if code==create)
            );
        } else if null != 0 {
            assert!(matches!(result, Err(Error::Invariant { operation: _, reason: _ })));
        } else {
            assert!(
                matches!(result.unwrap().destroy(),Err(Error::Native(crate::error::NativeError { operation: crate::error::Operation::Destroy, code: code })) if code==destroy)
            );
        }
        assert_eq!(fake.references.get(), 2);
        assert_eq!(fixture_snapshot(&observer).destroys, expected_destroys);
        drop(runtime);assert!(weak.upgrade().is_some());
        assert_eq!(ERRORS.load(SeqCst), 0);
    }
}};}
    family!(fixture_runtime(), Upscaler, std::rc::Rc::downgrade);
    #[cfg(feature = "movable")]
    family!(
        fixture_movable_runtime(),
        MovableUpscaler,
        std::sync::Arc::downgrade
    );
}

unsafe extern "C" fn query_failure(_: *mut *mut c_void, _: *mut c_void) -> u32 {
    0xf1234567
}
#[test]
fn post_create_query_error_runs_normal_callback_cleanup() {
    let _serial = SERIAL.lock().unwrap();
    macro_rules! family {
        ($fixture:expr,$owner:ty) => {{
            let (mut runtime, observer) = $fixture;
            let (device, fake) = fake_device();
            install(&observer, 31);
            // SAFETY: Matching fixture signature; the query writes no output on error.
            unsafe {
                observer
                    .get::<unsafe extern "C" fn(
                        Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>,
                    )>(c"fixture_set_query_observer")
                    .unwrap()(Some(query_failure));
            }
            // SAFETY: Controlled static callbacks and synchronous idle fixture.
            let result = unsafe {
                runtime
                    .upscaler_dx12(&device)
                    .create_with_callbacks(&sizes(), bundle(31))
            };
            assert!(matches!(
                result,
                Err(Error::Native(crate::error::NativeError {
                    operation: crate::error::Operation::Query,
                    code: 0xf1234567
                }))
            ));
            assert_eq!(fixture_snapshot(&observer).destroys, 1);
            assert_eq!(fake.references.get(), 1);
            assert_eq!(ERRORS.load(SeqCst), 0);
        }};
    }
    family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    family!(fixture_movable_runtime(), MovableUpscaler);
}
