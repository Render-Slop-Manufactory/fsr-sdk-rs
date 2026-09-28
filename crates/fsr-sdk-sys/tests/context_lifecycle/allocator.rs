// SPDX-License-Identifier: MPL-2.0

//! Experiment-local host allocator. Never manually cleans up SDK allocations.
use fsr_sdk_sys::api::ffxAllocationCallbacks;
use std::{
    alloc::{Layout, alloc, dealloc},
    ffi::c_void,
    io::Write,
    sync::{Mutex, MutexGuard},
};

struct Allocation {
    pointer: usize,
    layout: Layout,
    freed: bool,
}

struct State {
    attempts: usize,
    frees: usize,
    null_frees: usize,
    allocations: Vec<Allocation>,
}

pub struct Tracker {
    fail_at: usize,
    state: Mutex<State>,
}

impl Tracker {
    pub fn new(fail_at: usize) -> Self {
        Self {
            fail_at,
            state: Mutex::new(State {
                attempts: 0,
                frees: 0,
                null_frees: 0,
                allocations: Vec::with_capacity(4096),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|_| std::process::abort())
    }

    pub fn callbacks(&self) -> ffxAllocationCallbacks {
        ffxAllocationCallbacks {
            pUserData: std::ptr::from_ref(self).cast_mut().cast(),
            alloc: Some(allocate),
            dealloc: Some(free),
        }
    }

    pub fn report(&self, phase: &str) {
        let state = self.lock();
        let live = state.allocations.iter().filter(|a| !a.freed).count();
        let _ = writeln!(
            std::io::stderr(),
            "SUMMARY phase={phase} fail_at={} reached={} attempts={} successful={} frees={} null_frees={} live={} all_freed_once={}",
            self.fail_at,
            self.fail_at != 0 && state.attempts >= self.fail_at,
            state.attempts,
            state.allocations.len(),
            state.frees,
            state.null_frees,
            live,
            live == 0
        );
    }
}

unsafe extern "C" fn allocate(user: *mut c_void, size: u64) -> *mut c_void {
    // SAFETY: The probe retains a stable Tracker until process exit or teardown.
    let tracker = unsafe { &*user.cast::<Tracker>() };
    let mut state = tracker.lock();
    state.attempts += 1;
    let index = state.attempts;
    // Windows x64 malloc-equivalent alignment. The SDK supplies no alignment
    // argument. Size zero gets a distinct freeable allocation of one byte.
    let layout = usize::try_from(size)
        .ok()
        .and_then(|size| Layout::from_size_align(size.max(1), 16).ok());
    let injected = index == tracker.fail_at;
    let pointer = if injected {
        std::ptr::null_mut()
    } else if let Some(layout) = layout {
        if state.allocations.len() == state.allocations.capacity() {
            std::process::abort();
        }
        // SAFETY: Valid nonzero layout; paired free uses this exact layout.
        let pointer = unsafe { alloc(layout) };
        if !pointer.is_null() {
            state.allocations.push(Allocation {
                pointer: pointer as usize,
                layout,
                freed: false,
            });
        }
        pointer.cast()
    } else {
        std::ptr::null_mut()
    };
    let _ = writeln!(
        std::io::stderr(),
        "ALLOC index={index} size={size} alignment=16 injected={injected} pointer={pointer:p}"
    );
    pointer
}

unsafe extern "C" fn free(user: *mut c_void, pointer: *mut c_void) {
    // SAFETY: Same retained Tracker as allocation; all mutation is serialized.
    let tracker = unsafe { &*user.cast::<Tracker>() };
    let mut state = tracker.lock();
    if pointer.is_null() {
        state.null_frees += 1;
        let _ = writeln!(std::io::stderr(), "FREE null");
        return;
    }
    let Some(index) = state
        .allocations
        .iter()
        .position(|a| a.pointer == pointer as usize && !a.freed)
    else {
        let _ = writeln!(
            std::io::stderr(),
            "INVALID_FREE pointer={pointer:p}; aborting without deallocation"
        );
        std::process::abort();
    };
    let allocation = &mut state.allocations[index];
    allocation.freed = true;
    // SAFETY: Matching live allocation and original layout, never freed twice.
    unsafe { dealloc(pointer.cast(), allocation.layout) };
    state.frees += 1;
    let _ = writeln!(
        std::io::stderr(),
        "FREE allocation_record={} pointer={pointer:p}",
        index + 1
    );
}
