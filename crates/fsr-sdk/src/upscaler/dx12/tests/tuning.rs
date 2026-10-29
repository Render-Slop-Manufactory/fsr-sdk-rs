// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};
use crate::error::Operation;
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
};

type Observation = (usize, u64, Option<u32>);
thread_local! {
    static CODE: Cell<u32> = const { Cell::new(0) };
    static SEEN: RefCell<Vec<Observation>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "C" fn observe(context: *mut *mut c_void, ptr: *const c_void) -> u32 {
    // SAFETY: Only registered for complete synchronous tuning descriptors on a
    // retained fixture context. The callback never retains descriptor/value pointers.
    unsafe {
        if context.is_null() || (*context).is_null() || ptr.is_null() {
            return 0xe029;
        }
        let desc = &*ptr.cast::<ffxConfigureDescUpscaleKeyValue>();
        if desc.header.r#type != 0x10007
            || !desc.header.pNext.is_null()
            || desc.key > 4
            || desc.u64 != 0
        {
            return 0xe02a;
        }
        let value = if desc.ptr.is_null() {
            None
        } else {
            Some((*desc.ptr.cast::<f32>()).to_bits())
        };
        SEEN.with_borrow_mut(|seen| seen.push((*context as usize, desc.key, value)));
    }
    // Return OK without applying the value, or the test's exact unknown code.
    CODE.get()
}

unsafe extern "C" fn query_error(_: *mut *mut c_void, _: *mut c_void) -> u32 {
    0xbeef
}

fn register(observer: &libloading::Library) {
    CODE.set(0);
    SEEN.with_borrow_mut(Vec::clear);
    // SAFETY: Exact paired controlled export and callback signature.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32>)>(c"fixture_set_configure_observer").unwrap()(Some(observe));
    }
}

fn calls(observer: &libloading::Library) -> u32 {
    // SAFETY: Exact fixture counter export.
    unsafe {
        observer
            .get::<extern "C" fn() -> u32>(c"fixture_configure_calls")
            .unwrap()()
    }
}

macro_rules! exercise {
    ($runtime:expr, $upscaler:ty) => {{
        let (mut runtime, observer) = $runtime;
        let (device, fake) = fake_device();
        register(&observer);
        let mut context = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()).unwrap();
        assert_eq!(calls(&observer), 0); // Construction never configures implicitly.
        let parameters = [TuningParameter::VelocityFactor, TuningParameter::ReactivenessScale, TuningParameter::ShadingChangeScale, TuningParameter::AccumulationAddedPerFrame, TuningParameter::MinDisocclusionAccumulation];
        for parameter in parameters {
            let maximum = match parameter {
                TuningParameter::ReactivenessScale | TuningParameter::ShadingChangeScale => f32::MAX,
                _ => 1.0,
            };
            context.request_tuning(TuningRequest::new(parameter, TuningValue::Value(-0.0)).unwrap()).unwrap();
            context.request_tuning(TuningRequest::new(parameter, TuningValue::Value(maximum)).unwrap()).unwrap();
            context.request_tuning(TuningRequest::new(parameter, TuningValue::ProviderDefault).unwrap()).unwrap();
        }
        for parameter in parameters {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -2.0] {
                assert_result!(TuningRequest::new(parameter, TuningValue::Value(value)), Err(Error::Input(crate::error::InputError::InvalidTuningValue { parameter })));
            }
        }
        // Reusing a scalar for a narrower parameter must construct a new checked pair.
        let value = TuningValue::Value(2.0);
        context.request_tuning(TuningRequest::new(TuningParameter::ReactivenessScale, value).unwrap()).unwrap();
        let before = calls(&observer);
        assert_result!(TuningRequest::new(TuningParameter::VelocityFactor, value), Err(Error::Input(crate::error::InputError::InvalidTuningValue { parameter: TuningParameter::VelocityFactor })));
        assert_eq!(calls(&observer), before);
        assert_eq!(calls(&observer), 16);
        SEEN.with_borrow(|seen| {
            let handle = seen[0].0;
            for (index, &item) in seen.iter().take(15).enumerate() {
                assert_eq!(item, (handle, (index / 3) as u64, match index % 3 { 0 => Some((-0.0f32).to_bits()), 1 => Some(if matches!(parameters[index / 3], TuningParameter::ReactivenessScale | TuningParameter::ShadingChangeScale) { f32::MAX.to_bits() } else { 1.0f32.to_bits() }), _ => None }));
            }
        });
        let mut context = *Box::new(context);
        // An ordinary native query failure must leave configuration usable.
        // SAFETY: Paired callback returns an ordinary query error without mutation.
        unsafe { observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(Some(query_error)); }
        assert_result!(context.gpu_memory_usage(), Err(Error::native(Operation::Query, 0xbeef)));
        context.request_tuning(TuningRequest::new(parameters[0], TuningValue::ProviderDefault).unwrap()).unwrap();
        SEEN.with_borrow(|seen| assert_eq!(seen.first().unwrap().0, seen.last().unwrap().0));
        CODE.set(0xa1234567);
        assert_result!(context.request_tuning(TuningRequest::new(parameters[3], TuningValue::Value(0.5)).unwrap()), Err(Error::native(Operation::Tuning, 0xa1234567)));
        let count = calls(&observer);
        assert_result!(TuningRequest::new(parameters[0], TuningValue::Value(f32::NAN)), Err(Error::Input(crate::error::InputError::InvalidTuningValue { parameter: parameters[0] })));
        assert_result!(context.request_tuning(TuningRequest::new(parameters[0], TuningValue::ProviderDefault).unwrap()), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.provider(), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.resource_requirements(), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.gpu_memory_usage(), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.upscale_ratio(QualityMode::Quality), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.render_size_for_quality(Dimensions::new(u32::MAX, 1).unwrap(), QualityMode::Quality), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.jitter_phase_count(Dimensions::new(1, 1).unwrap(), Dimensions::new(16384, 1).unwrap()), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_result!(context.jitter_offset(-1, JitterPhaseCount::new(1).unwrap()), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        // SAFETY: Poison rejects before inspection or native dispatch; no GPU work.
        assert_result!(unsafe { context.core.dispatch_core(|_| panic!("poison inspected")) }, Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_eq!(calls(&observer), count);
        context.destroy().unwrap();
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 1);

        // Restore queries for new construction; a sibling/default context remains usable.
        // SAFETY: Exact controlled export; None removes the observer.
        unsafe { observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(None); }
        CODE.set(0);
        let mut context = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()).unwrap();
        context.request_tuning(TuningRequest::new(parameters[0], TuningValue::ProviderDefault).unwrap()).unwrap();
        // SAFETY: Exact fixture setter; dispatch reads only the descriptor header.
        unsafe { observer.get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code").unwrap()(3); }
        // SAFETY: Complete integer/pointer descriptor with valid zero bits; the
        // controlled fixture only reads the header and records no GPU commands.
        let mut desc: ffxDispatchDescUpscale = unsafe { std::mem::zeroed() };
        desc.header.r#type = FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
        // SAFETY: Matched controlled fixture, without GPU work or borrowed inputs.
        assert_result!(unsafe { context.core.dispatch_core(|_| Ok(desc)) }, Err(Error::native(Operation::Upscale, 3)));
        let count = calls(&observer);
        assert_result!(context.request_tuning(TuningRequest::new(parameters[0], TuningValue::ProviderDefault).unwrap()), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Upscale, 3))));
        assert_eq!(calls(&observer), count);
        context.destroy().unwrap();

        let mut context = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()).unwrap();
        CODE.set(u32::MAX);
        assert_result!(context.request_tuning(TuningRequest::new(parameters[4], TuningValue::ProviderDefault).unwrap()), Err(Error::native(Operation::Tuning, u32::MAX)));
        fixture_set(&observer, 0, u32::MAX, 0);
        assert_result!(context.destroy(), Err(Error::native(Operation::Destroy, u32::MAX)));
        assert_eq!(fake.references.get(), 2); // Failed destroy retains the full bundle.
        assert_eq!(fixture_snapshot(&observer).destroys, 3);
    }};
}

#[test]
fn local_public_tuning_mapping_errors_and_lifecycle() {
    exercise!(fixture_runtime(), Upscaler);
}

#[cfg(feature = "movable")]
#[test]
fn movable_public_tuning_mapping_errors_and_lifecycle() {
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}

macro_rules! staged_requests {
    ($runtime:expr, $owner:ty) => {{
        let (mut runtime, observer) = $runtime;
        let (device, fake) = fake_device();
        register(&observer);
        // SAFETY: Matched synchronous fixture; no GPU work or retained callbacks.
        let mut context = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()).unwrap();
        let prepare = |desired: &[(TuningParameter, TuningValue)]| {
            desired.iter().map(|&(p, v)| TuningRequest::new(p, v)).collect::<Result<Vec<_>, Error>>()
        };
        let mut desired = [
            (TuningParameter::VelocityFactor, TuningValue::Value(0.5)),
            (TuningParameter::ShadingChangeScale, TuningValue::Value(f32::MAX)),
            (TuningParameter::VelocityFactor, TuningValue::ProviderDefault),
            (TuningParameter::AccumulationAddedPerFrame, TuningValue::Value(2.0)),
        ];
        let apply = |context: &mut $owner, desired: &[(TuningParameter, TuningValue)]| -> Result<(), Error> {
            let requests = prepare(desired)?;
            for request in requests {
                context.request_tuning(request)?;
            }
            Ok(())
        };
        // A later invalid pair must prevent every earlier native mutation.
        assert_result!(apply(&mut context, &desired), Err(Error::Input(crate::error::InputError::InvalidTuningValue { parameter: TuningParameter::AccumulationAddedPerFrame })));
        assert_eq!(calls(&observer), 0);
        assert_eq!(context.core.eligibility, Eligibility::Usable);
        desired[3].1 = TuningValue::Value(0.25);
        let requests = prepare(&desired).unwrap();
        // Order and duplicate parameters are caller intent, not a set to coalesce.
        for request in requests.iter().copied() { context.request_tuning(request).unwrap(); }
        SEEN.with_borrow(|seen| assert_eq!(seen.iter().map(|x| x.1).collect::<Vec<_>>(), [0, 2, 0, 3]));
        let error = requests.into_iter().enumerate().try_for_each(|(index, request)| {
            CODE.set(if index == 1 { 0xa1234567 } else { 0 });
            context.request_tuning(request)
        });
        assert_result!(error, Err(Error::native(Operation::Tuning, 0xa1234567)));
        assert_eq!(calls(&observer), 6); // Four prior calls, one acknowledgement, then terminal failure.
        let reset = TuningRequest::new(TuningParameter::VelocityFactor, TuningValue::ProviderDefault).unwrap();
        assert_result!(context.request_tuning(reset), Err(crate::test_errors::terminal(Error::native(crate::error::Operation::Tuning, 0xa1234567))));
        assert_eq!(calls(&observer), 6);
        context.destroy().unwrap();
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, 1);
    }};
}

#[test]
fn local_full_list_staging_and_ordered_native_partial_failure() {
    staged_requests!(fixture_runtime(), Upscaler);
}

#[cfg(feature = "movable")]
#[test]
fn movable_full_list_staging_and_ordered_native_partial_failure() {
    staged_requests!(fixture_movable_runtime(), MovableUpscaler);
}
