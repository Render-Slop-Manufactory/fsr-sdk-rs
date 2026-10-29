// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{error::Operation, upscaler::DebugLevel};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
};
type Observation = (bool, u64, u64, u32, bool);
thread_local! {
    static CODE: Cell<u32> = const { Cell::new(0) };
    static SEEN: RefCell<Vec<Observation>> = const { RefCell::new(Vec::new()) };
}
unsafe extern "C" fn message(_: u32, _: *const u16) {}
unsafe extern "C" fn observe(context: *mut *mut c_void, desc: *const c_void) -> u32 {
    // SAFETY: Exact descriptors from the public synchronous diagnostic routes.
    unsafe {
        let h = &*desc.cast::<ffxApiHeader>();
        if !h.pNext.is_null() {
            return 0xf0400001;
        }
        let (effect, level, callback) = if h.r#type == 7 {
            let d = &*desc.cast::<ffxConfigureDescGlobalDebug>();
            (d.effectId, d.debugLevel, d.fpMessage)
        } else if h.r#type == 1 {
            let d = &*desc.cast::<ffxConfigureDescGlobalDebug1>();
            (0, d.debugLevel, d.fpMessage)
        } else {
            return 0xf0400002;
        };
        SEEN.with_borrow_mut(|s| {
            s.push((
                !context.is_null(),
                h.r#type,
                effect,
                level,
                callback.is_some(),
            ))
        });
    }
    CODE.get()
}
fn register(observer: &libloading::Library) {
    SEEN.with_borrow_mut(Vec::clear);
    CODE.set(0);
    // SAFETY: Exact paired fixture signature and static callback.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void,*const c_void)->u32>)>(c"fixture_set_configure_observer").unwrap()(Some(observe));
    }
}
macro_rules! exercise {
    ($runtime:expr,$owner:ty) => {{
        let (mut runtime, observer) = $runtime;
        let (device, _) = fake_device();
        register(&observer);
        let mut owner = unsafe { runtime.upscaler_dx12(&device) }
            .create(&sizes())
            .unwrap();
        for (level, native) in [
            (DebugLevel::Silent, 0),
            (DebugLevel::Errors, 1),
            (DebugLevel::Warnings, 2),
            (DebugLevel::Verbose, 0x0fffffff),
        ] {
            // SAFETY: Controlled idle fixture, static targets, no GPU or workers.
            unsafe {
                runtime
                    .upscaler_dx12(&device)
                    .configure_debug(level, Some(message))
                    .unwrap();
                runtime
                    .upscaler_dx12(&device)
                    .configure_global_debug_legacy(level, None)
                    .unwrap();
                owner
                    .configure_global_debug_legacy(level, Some(message))
                    .unwrap();
            }
            SEEN.with_borrow(|s| {
                assert_eq!(
                    &s[s.len() - 3..],
                    &[
                        (false, 7, 0x10000, native, true),
                        (false, 1, 0, native, false),
                        (true, 1, 0, native, true)
                    ]
                )
            });
        }
        CODE.set(0xfedc4321);
        // SAFETY: Fixture returns an ordinary error; continuation is known safe.
        let error = unsafe {
            runtime
                .upscaler_dx12(&device)
                .configure_debug(DebugLevel::Verbose, None)
        };
        assert_result!(
            error,
            Err(Error::native(Operation::ConfigureDebug, 0xfedc4321))
        );
        // SAFETY: The null-context legacy route has the same controlled native
        // failure, but a distinct operation and no owner eligibility transition.
        assert_result!(
            unsafe {
                runtime
                    .upscaler_dx12(&device)
                    .configure_global_debug_legacy(DebugLevel::Verbose, None)
            },
            Err(Error::native(
                Operation::ConfigureGlobalDebugLegacy,
                0xfedc4321
            ))
        );
        CODE.set(0);
        // A runtime-global error does not fabricate poison for the live owner.
        unsafe { owner.configure_global_debug_legacy(DebugLevel::Silent, None) }.unwrap();
        CODE.set(0xfedc1234);
        let error = unsafe { owner.configure_global_debug_legacy(DebugLevel::Warnings, None) };
        assert_result!(
            error,
            Err(Error::native(
                Operation::ConfigureContextDebugLegacy,
                0xfedc1234
            ))
        );
        let before = SEEN.with_borrow(Vec::len);
        assert_result!(
            unsafe { owner.configure_global_debug_legacy(DebugLevel::Silent, None) },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::ConfigureContextDebugLegacy,
                0xfedc1234
            )))
        );
        assert_result!(
            owner.gpu_memory_usage(),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::ConfigureContextDebugLegacy,
                0xfedc1234
            )))
        );
        assert_result!(
            owner.request_tuning(
                TuningRequest::new(
                    TuningParameter::VelocityFactor,
                    TuningValue::ProviderDefault
                )
                .unwrap()
            ),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::ConfigureContextDebugLegacy,
                0xfedc1234
            )))
        );
        assert_result!(
            unsafe { owner.core.dispatch_core(|_| panic!("poison inspected")) },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::ConfigureContextDebugLegacy,
                0xfedc1234
            )))
        );
        assert_eq!(SEEN.with_borrow(Vec::len), before);
        owner.destroy().unwrap();
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
        let mut owner = unsafe { runtime.upscaler_dx12(&device) }
            .create(&sizes())
            .unwrap();
        owner.core.eligibility = crate::context_policy::failed(crate::error::Operation::Upscale, 3);
        assert_result!(
            unsafe { owner.configure_global_debug_legacy(DebugLevel::Silent, None) },
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_eq!(SEEN.with_borrow(Vec::len), before);
        owner.destroy().unwrap();
    }};
}
#[test]
fn local_routes_native_codes_and_poison() {
    exercise!(fixture_runtime(), Upscaler);
}
#[cfg(feature = "movable")]
#[test]
fn movable_routes_native_codes_and_poison() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}
