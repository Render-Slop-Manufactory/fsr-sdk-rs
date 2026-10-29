// SPDX-License-Identifier: MPL-2.0

//! Standalone Windows fixture for the public constructor, not an AMD provider.
//! Descriptor layouts mirror SDK v2.3.0; AMD notice is in this crate's LICENSE-AMD.
use std::{ffi::c_void, sync::{Mutex, atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering}}};

#[repr(C)]
pub struct Header {
    tag: u64,
    next: *mut Header,
}

#[repr(C)]
struct Dimensions {
    width: u32,
    height: u32,
}

#[repr(C)]
struct Root {
    header: Header,
    flags: u32,
    render: Dimensions,
    upscale: Dimensions,
    message: *const c_void,
}

#[repr(C)]
struct Version {
    header: Header,
    version: u32,
}

#[repr(C)]
struct Backend {
    header: Header,
    device: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Snapshot {
    pub creates: u32,
    pub destroys: u32,
    pub root_tag: u64,
    pub version_tag: u64,
    pub backend_tag: u64,
    pub version: u32,
    pub flags: u32,
    pub render_width: u32,
    pub render_height: u32,
    pub upscale_width: u32,
    pub upscale_height: u32,
    pub device: usize,
    pub chain_valid: u32,
}

#[repr(C)]
struct OverrideVersion { header: Header, id: u64 }
static DESTROYED_PROVIDER: AtomicU64 = AtomicU64::new(0);

// SAFETY: Only this fixture's complete retained backend chain reaches here.
unsafe fn selected_provider(backend: &Backend) -> Option<u64> {
    let mut next = backend.header.next;
    if !next.is_null() && unsafe { (*next).tag } == 3 {
        next = unsafe { (*next).next };
    }
    if next.is_null() { return Some(0); }
    // SAFETY: Caller supplies a complete optional override node.
    let node = unsafe { &*next.cast::<OverrideVersion>() };
    (node.header.tag == 5 && node.header.next.is_null()).then_some(node.id)
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_destroyed_provider() -> u64 { DESTROYED_PROVIDER.load(Ordering::SeqCst) }

static CREATES: AtomicU32 = AtomicU32::new(0);
static DESTROYS: AtomicU32 = AtomicU32::new(0);
static CREATE_CODE: AtomicU32 = AtomicU32::new(0);
static DESTROY_CODE: AtomicU32 = AtomicU32::new(0);
static NULL_CREATE: AtomicU32 = AtomicU32::new(0);
static ROOT_TAG: AtomicUsize = AtomicUsize::new(0);
static VERSION_TAG: AtomicUsize = AtomicUsize::new(0);
static BACKEND_TAG: AtomicUsize = AtomicUsize::new(0);
static VERSION: AtomicU32 = AtomicU32::new(0);
static FLAGS: AtomicU32 = AtomicU32::new(0);
static RENDER_WIDTH: AtomicU32 = AtomicU32::new(0);
static RENDER_HEIGHT: AtomicU32 = AtomicU32::new(0);
static UPSCALE_WIDTH: AtomicU32 = AtomicU32::new(0);
static UPSCALE_HEIGHT: AtomicU32 = AtomicU32::new(0);
static DEVICE: AtomicUsize = AtomicUsize::new(0);
static CHAIN_VALID: AtomicU32 = AtomicU32::new(0);
static DISPATCH_CODE: AtomicU32 = AtomicU32::new(0);
static DISPATCH_CALLS: AtomicU32 = AtomicU32::new(0);
static DISPATCH_TAG: AtomicUsize = AtomicUsize::new(0);
static DISPATCH_OBSERVER: Mutex<Option<unsafe extern "C" fn(*const Header)>> = Mutex::new(None);
static QUERY_CALLS: AtomicU32 = AtomicU32::new(0);
static CONFIGURE_CALLS: AtomicU32 = AtomicU32::new(0);
static CREATE_OBSERVER: Mutex<Option<unsafe extern "C" fn(*const Header,*const c_void,bool)>> = Mutex::new(None);
#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_create_observer(observer: Option<unsafe extern "C" fn(*const Header,*const c_void,bool)>) {
    *CREATE_OBSERVER.lock().unwrap() = observer;
}
static CONFIGURE_OBSERVER: Mutex<Option<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_configure_observer(observer: Option<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32>) {
    *CONFIGURE_OBSERVER.lock().unwrap() = observer;
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_configure_calls() -> u32 { CONFIGURE_CALLS.load(Ordering::SeqCst) }
static QUERY_OBSERVER: Mutex<Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>> =
    Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_query_observer(
    observer: Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>,
) {
    *QUERY_OBSERVER.lock().unwrap() = observer;
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_query_calls() -> u32 {
    QUERY_CALLS.load(Ordering::SeqCst)
}

/// Opt-in typed inspection for calls supplying a complete dispatch descriptor.
#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_dispatch_observer(observer: Option<unsafe extern "C" fn(*const Header)>) {
    *DISPATCH_OBSERVER.lock().unwrap() = observer;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DispatchObservation {
    pub calls: u32,
    pub tag: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_dispatch_code(code: u32) {
    DISPATCH_CODE.store(code, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_dispatch_observation() -> DispatchObservation {
    DispatchObservation {
        calls: DISPATCH_CALLS.load(Ordering::SeqCst),
        tag: DISPATCH_TAG.load(Ordering::SeqCst) as u64,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_set_outcome(create: u32, destroy: u32, null_create: u32) {
    CREATE_CODE.store(create, Ordering::SeqCst);
    DESTROY_CODE.store(destroy, Ordering::SeqCst);
    NULL_CREATE.store(null_create, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_snapshot() -> Snapshot {
    let load = Ordering::SeqCst;
    Snapshot {
        creates: CREATES.load(load),
        destroys: DESTROYS.load(load),
        root_tag: ROOT_TAG.load(load) as u64,
        version_tag: VERSION_TAG.load(load) as u64,
        backend_tag: BACKEND_TAG.load(load) as u64,
        version: VERSION.load(load),
        flags: FLAGS.load(load),
        render_width: RENDER_WIDTH.load(load),
        render_height: RENDER_HEIGHT.load(load),
        upscale_width: UPSCALE_WIDTH.load(load),
        upscale_height: UPSCALE_HEIGHT.load(load),
        device: DEVICE.load(load),
        chain_valid: CHAIN_VALID.load(load),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxCreateContext(
    out: *mut *mut c_void,
    desc: *mut Header,
    allocator: *const c_void,
) -> u32 {
    // SAFETY: The wrapper supplies a complete pinned chain and writable output.
    unsafe {
        CREATES.fetch_add(1, Ordering::SeqCst);
        let root = &*(desc.cast::<Root>());
        let version = &*(root.header.next.cast::<Version>());
        let backend = &*(version.header.next.cast::<Backend>());
        ROOT_TAG.store(root.header.tag as usize, Ordering::SeqCst);
        VERSION_TAG.store(version.header.tag as usize, Ordering::SeqCst);
        BACKEND_TAG.store(backend.header.tag as usize, Ordering::SeqCst);
        VERSION.store(version.version, Ordering::SeqCst);
        FLAGS.store(root.flags, Ordering::SeqCst);
        RENDER_WIDTH.store(root.render.width, Ordering::SeqCst);
        RENDER_HEIGHT.store(root.render.height, Ordering::SeqCst);
        UPSCALE_WIDTH.store(root.upscale.width, Ordering::SeqCst);
        UPSCALE_HEIGHT.store(root.upscale.height, Ordering::SeqCst);
        DEVICE.store(backend.device as usize, Ordering::SeqCst);
        let has_observer=CREATE_OBSERVER.lock().unwrap().is_some();
        CHAIN_VALID.store(
            u32::from((allocator.is_null() || has_observer) && (*out).is_null() &&
                (root.message.is_null() || has_observer) && selected_provider(backend).is_some() &&
                !backend.device.is_null()),
            Ordering::SeqCst,
        );
        let observer = *CREATE_OBSERVER.lock().unwrap();
        if let Some(observer) = observer { observer(desc,allocator,false); }
        *out = if NULL_CREATE.load(Ordering::SeqCst) != 0 {
            std::ptr::null_mut()
        } else {
            desc.cast()
        };
        CREATE_CODE.load(Ordering::SeqCst)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDestroyContext(out: *mut *mut c_void, allocator: *const c_void) -> u32 {
    // SAFETY: Only a successful fixture handle reaches this function.
    unsafe {
        if out.is_null() || (*out).is_null() || (!allocator.is_null() && CREATE_OBSERVER.lock().unwrap().is_none()) {
            return 0xf0000001;
        }
        DESTROYS.fetch_add(1, Ordering::SeqCst);
        // Read the original retained chain again after outer ownership moves.
        let root = &*((*out).cast::<Root>());
        let version = &*(root.header.next.cast::<Version>());
        let backend = &*(version.header.next.cast::<Backend>());
        if root.header.tag != ROOT_TAG.load(Ordering::SeqCst) as u64
            || version.header.tag != VERSION_TAG.load(Ordering::SeqCst) as u64
            || backend.header.tag != BACKEND_TAG.load(Ordering::SeqCst) as u64
            || backend.device.is_null()
            || selected_provider(backend).is_none()
        {
            return 0xf0000003;
        }
        let observer = *CREATE_OBSERVER.lock().unwrap();
        if let Some(observer) = observer { observer((*out).cast(),allocator,true); }
        DESTROYED_PROVIDER.store(selected_provider(backend).unwrap(), Ordering::SeqCst);
        DESTROY_CODE.load(Ordering::SeqCst)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxQuery(context: *mut *mut c_void, desc: *mut c_void) -> u32 {
    QUERY_CALLS.fetch_add(1, Ordering::SeqCst);
    let observer = *QUERY_OBSERVER.lock().unwrap();
    if let Some(observer) = observer {
        // SAFETY: The registering test supplies the matching synchronous callback.
        unsafe { observer(context, desc) }
    } else {
        0
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxConfigure(context: *mut *mut c_void, desc: *const c_void) -> u32 {
    CONFIGURE_CALLS.fetch_add(1, Ordering::SeqCst);
    let observer = *CONFIGURE_OBSERVER.lock().unwrap();
    if let Some(observer) = observer {
        // SAFETY: Registered observer matches the wrapper's synchronous descriptor.
        unsafe { observer(context, desc) }
    } else { 0 }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ffxDispatch(context: *mut *mut c_void, desc: *const Header) -> u32 {
    // SAFETY: The wrapper's private test call supplies a live context and a
    // complete descriptor header. No GPU commands are recorded by this fixture.
    unsafe {
        if context.is_null() || (*context).is_null() || desc.is_null() {
            return 0xf000_0002;
        }
        DISPATCH_CALLS.fetch_add(1, Ordering::SeqCst);
        DISPATCH_TAG.store((*desc).tag as usize, Ordering::SeqCst);
        let observer = *DISPATCH_OBSERVER.lock().unwrap();
        if let Some(observer) = observer {
            // The registering test supplies the matching descriptor and callback.
            observer(desc);
        }
    }
    DISPATCH_CODE.load(Ordering::SeqCst)
}
