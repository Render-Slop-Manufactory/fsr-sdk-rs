// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};
use crate::query::{JitterPhaseCount, QualityMode};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Observation {
    tag: u64,
    live: bool,
    device: usize,
    inputs: [u32; 3],
}
thread_local! {
    static OBSERVATIONS: RefCell<Vec<Observation>> = const { RefCell::new(Vec::new()) };
    static CODE: Cell<u32> = const { Cell::new(0) };
}

unsafe extern "C" fn observe(context: *mut *mut c_void, desc: *mut c_void) -> u32 {
    // SAFETY: The observer is registered only for complete tagged utility queries;
    // copying scalars/pointer identities and writing bounded outputs is synchronous.
    unsafe {
        let header = &*desc.cast::<ffxApiHeader>();
        let device = if context.is_null() {
            if header.pNext.is_null() {
                return 0xf001;
            }
            let backend = &*header.pNext.cast::<ffxCreateBackendDX12Desc>();
            if backend.header.r#type != FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
                || !backend.header.pNext.is_null()
            {
                return 0xf002;
            }
            backend.device as usize
        } else {
            if (*context).is_null() || !header.pNext.is_null() {
                return 0xf003;
            }
            0
        };
        if header.r#type == FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS {
            let d = &mut *desc.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
            d.required_resources = 15;
            d.optional_resources = 48;
            return CODE.get();
        }
        let inputs = match header.r#type {
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode>();
                *d.pOutUpscaleRatio = d.qualityMode as f32 + 1.25;
                [d.qualityMode, 0, 0]
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetRenderResolutionFromQualityMode>();
                *d.pOutRenderWidth = d.displayWidth - 1;
                *d.pOutRenderHeight = d.displayHeight - 1;
                [d.displayWidth, d.displayHeight, d.qualityMode]
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>();
                *d.pOutPhaseCount = 19;
                [d.renderWidth, d.displayWidth, 0]
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetJitterOffset>();
                *d.pOutX = -0.125;
                *d.pOutY = 0.375;
                [d.index as u32, d.phaseCount as u32, 0]
            }
            _ => return 0xf004,
        };
        OBSERVATIONS.with_borrow_mut(|v| {
            v.push(Observation {
                tag: header.r#type,
                live: !context.is_null(),
                device,
                inputs,
            })
        });
    }
    CODE.get()
}

fn register(observer: &libloading::Library) {
    CODE.set(0);
    OBSERVATIONS.with_borrow_mut(Vec::clear);
    // SAFETY: Exact fixture observer ABI; registered for synchronous invocation.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(Some(observe));
    }
}
fn check(live: bool, device: usize, expected: &[(u64, [u32; 3])]) {
    let actual = OBSERVATIONS.with_borrow_mut(std::mem::take);
    assert_eq!(
        actual,
        expected
            .iter()
            .map(|&(tag, inputs)| Observation {
                tag,
                live,
                device,
                inputs
            })
            .collect::<Vec<_>>()
    );
}

// The macro exercises the same ordinary public API for both ownership families.
macro_rules! check_family {
    ($runtime:expr, $upscaler:ty) => {{
        let (mut runtime, observer) = $runtime;
        let scene = super::com::Scene::new();
        let device = &scene.device;
        register(&observer);
        let before = scene.reference_counts();
        assert_eq!(
            unsafe { runtime.upscaler_dx12(device) }
                .ratio(QualityMode::Balanced)
                .unwrap(),
            3.25
        );
        assert_eq!(
            unsafe { runtime.upscaler_dx12(device) }
                .render_size_for_quality(sizes().max_upscale_size(), QualityMode::Performance)
                .unwrap(),
            Dimensions::new(1919, 1079).unwrap()
        );
        let phase = unsafe { runtime.upscaler_dx12(device) }
            .jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size())
            .unwrap();
        let offset = unsafe { runtime.upscaler_dx12(device) }
            .jitter_offset(18, phase)
            .unwrap();
        assert_eq!((offset.x(), offset.y()), (-0.125, 0.375));
        assert_eq!(scene.reference_counts(), before);
        let expected = [
            (0x10002, [2, 0, 0]),
            (0x10003, [1920, 1080, 3]),
            (0x10004, [1280, 1920, 0]),
            (0x10005, [18, 19, 0]),
        ];
        check(false, device.as_raw() as usize, &expected);
        let mut upscaler = unsafe { runtime.upscaler_dx12(device) }
            .create(&sizes())
            .unwrap();
        let before = scene.reference_counts();
        assert_eq!(upscaler.upscale_ratio(QualityMode::Balanced).unwrap(), 3.25);
        assert_eq!(
            upscaler
                .render_size_for_quality(sizes().max_upscale_size(), QualityMode::Performance)
                .unwrap(),
            Dimensions::new(1919, 1079).unwrap()
        );
        let phase = upscaler
            .jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size())
            .unwrap();
        assert_eq!(phase.count(), 19);
        let offset = upscaler.jitter_offset(18, phase).unwrap();
        assert_eq!((offset.x(), offset.y()), (-0.125, 0.375));
        check(true, 0, &expected);
        assert_eq!(scene.reference_counts(), before);
        CODE.set(0xf1234567);
        let error = Error::native(crate::error::Operation::Query, 0xf1234567);
        assert_result!(upscaler.upscale_ratio(QualityMode::Quality), Err(error));
        assert_result!(
            upscaler.render_size_for_quality(sizes().max_upscale_size(), QualityMode::Quality),
            Err(error)
        );
        assert_result!(
            upscaler.jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size()),
            Err(error)
        );
        assert_result!(upscaler.jitter_offset(0, phase), Err(error));
        assert_eq!(upscaler.core.eligibility, Eligibility::Usable);
        assert_result!(
            unsafe { runtime.upscaler_dx12(device) }.ratio(QualityMode::Quality),
            Err(error)
        );
        assert_result!(
            unsafe { runtime.upscaler_dx12(device) }
                .render_size_for_quality(sizes().max_upscale_size(), QualityMode::Quality),
            Err(error)
        );
        assert_result!(
            unsafe { runtime.upscaler_dx12(device) }
                .jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size()),
            Err(error)
        );
        assert_result!(
            unsafe { runtime.upscaler_dx12(device) }.jitter_offset(0, phase),
            Err(error)
        );
        CODE.set(0);
        OBSERVATIONS.with_borrow_mut(Vec::clear);
        assert!(upscaler.upscale_ratio(QualityMode::Quality).is_ok());
        OBSERVATIONS.with_borrow_mut(Vec::clear);
        upscaler.core.eligibility =
            crate::context_policy::failed(crate::error::Operation::Upscale, 3);
        assert_result!(
            upscaler.upscale_ratio(QualityMode::Quality),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            upscaler.render_size_for_quality(sizes().max_upscale_size(), QualityMode::Quality),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            upscaler.jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size()),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            upscaler.jitter_offset(0, phase),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert!(OBSERVATIONS.with_borrow(Vec::is_empty));
        assert!(
            unsafe { runtime.upscaler_dx12(device) }
                .ratio(QualityMode::Quality)
                .is_ok()
        );
        OBSERVATIONS.with_borrow_mut(Vec::clear);
        assert!(matches!(
            unsafe { runtime.upscaler_dx12(device) }.jitter_offset(-1, phase),
            Err(Error::Input(crate::error::InputError::Query(_)))
        ));
        assert!(matches!(
            unsafe { runtime.upscaler_dx12(device) }.render_size_for_quality(
                Dimensions::new(u32::MAX, 1).unwrap(),
                QualityMode::NativeAa
            ),
            Err(Error::Input(crate::error::InputError::Query(_)))
        ));
        assert!(OBSERVATIONS.with_borrow(Vec::is_empty));
        assert_eq!(scene.reference_counts(), before);
        upscaler.destroy().unwrap();
    }};
}

#[test]
fn utility_queries_route_devices_and_live_contexts_for_both_families() {
    check_family!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    check_family!(fixture_movable_runtime(), MovableUpscaler);
}

#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the DX12 device helper; use native-upscaler"]
fn native_utility_queries() {
    use std::path::PathBuf;
    let helper_path = std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("native device helper");
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("trusted loader"));
    // SAFETY: Opt-in runner stages the trusted paired helper and pinned runtime.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact device-helper export/signature, transferring one COM reference.
    let create = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .unwrap();
    let mut raw = null_mut();
    // SAFETY: Valid writable output; the helper creates a live DX12 device.
    assert!(unsafe { create(&mut raw) } >= 0 && !raw.is_null());
    // SAFETY: Adopt the transferred independent device reference.
    let device = unsafe { ID3D12Device::from_raw(raw.cast()) };
    macro_rules! run_family {
        ($runtime:ty, $upscaler:ty) => {{
            // SAFETY: Opt-in trusted utility-query configuration; isolated child
            // serializes query/construction/destruction and has no GPU work in flight.
            let mut runtime = unsafe { <$runtime>::load(&loader) }.unwrap();
            let mut context = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()).unwrap();
            eprintln!("Provider: {:?}", context.provider().unwrap());
            for mode in [QualityMode::NativeAa, QualityMode::Quality, QualityMode::Balanced, QualityMode::Performance, QualityMode::UltraPerformance] {
                let pre_ratio = unsafe { runtime.upscaler_dx12(&device) }.ratio(mode).unwrap();
                let pre_render = unsafe { runtime.upscaler_dx12(&device) }.render_size_for_quality(sizes().max_upscale_size(), mode).unwrap();
                let live_ratio = context.upscale_ratio(mode).unwrap();
                let live_render = context.render_size_for_quality(sizes().max_upscale_size(),mode).unwrap();
                eprintln!("Utility quality={mode:?}: pre_ratio={pre_ratio:?} pre_render={pre_render:?} live_ratio={live_ratio:?} live_render={live_render:?}");
            }
            let pre_phase: JitterPhaseCount = unsafe { runtime.upscaler_dx12(&device) }.jitter_phase_count(sizes().max_render_size(), sizes().max_upscale_size()).unwrap();
            let live_phase = context.jitter_phase_count(sizes().max_render_size(),sizes().max_upscale_size()).unwrap();
            for index in [0, pre_phase.count()/2, pre_phase.count()-1] {
                let offset = unsafe { runtime.upscaler_dx12(&device) }.jitter_offset(index, pre_phase).unwrap();
                eprintln!("Utility pre jitter: phase={pre_phase:?} index={index} offset={offset:?}");
            }
            for index in [0, live_phase.count()/2, live_phase.count()-1] {
                let offset = context.jitter_offset(index,live_phase).unwrap();
                eprintln!("Utility live jitter: phase={live_phase:?} index={index} offset={offset:?}");
            }
            context.destroy().unwrap();
        }};
    }
    run_family!(Runtime, Upscaler);
    #[cfg(feature = "movable")]
    run_family!(crate::runtime::MovableRuntime, MovableUpscaler);
    eprintln!("Upscaler utility queries: PASSED");
}
