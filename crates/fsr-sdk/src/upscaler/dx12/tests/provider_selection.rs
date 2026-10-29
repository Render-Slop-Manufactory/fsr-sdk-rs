// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};
use crate::{
    error::{InvariantViolation, Operation, QueryValidation},
    query::QualityMode,
};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
};

thread_local! {
    static DEVICE: Cell<usize> = const { Cell::new(0) };
    static AVAILABLE: Cell<u64> = const { Cell::new(91) };
    static LIVE_OVERRIDE: Cell<u64> = const { Cell::new(0) };
    static FAIL_TAG: Cell<u64> = const { Cell::new(0) };
    static VERSION_SHAPED_NAME: Cell<bool> = const { Cell::new(false) };
    static SEEN: RefCell<Vec<(u64, Option<u64>)>> = const { RefCell::new(Vec::new()) };
}

// These helpers inspect only complete fixture chains, never an AMD runtime.
unsafe fn selection(header: *mut ffxApiHeader) -> Option<u64> {
    if header.is_null() {
        return None;
    }
    // SAFETY: The complete optional node belongs to this synchronous fixture call.
    let node = unsafe { &*header.cast::<ffxOverrideVersion>() };
    assert_eq!(node.header.r#type, FFX_API_DESC_TYPE_OVERRIDE_VERSION);
    assert!(node.header.pNext.is_null());
    Some(node.versionId)
}

unsafe extern "C" fn observe(context: *mut *mut c_void, desc: *mut c_void) -> u32 {
    // SAFETY: Registered only for wrapper-produced complete tagged descriptors;
    // all writes are bounded and names are static through immediate copying.
    unsafe {
        let header = &*desc.cast::<ffxApiHeader>();
        if header.r#type == FFX_API_QUERY_DESC_TYPE_GET_VERSIONS {
            assert!(context.is_null() && header.pNext.is_null());
            let d = &mut *desc.cast::<ffxQueryDescGetVersions>();
            assert_eq!(d.device as usize, DEVICE.get());
            SEEN.with_borrow_mut(|v| v.push((header.r#type, None)));
            if FAIL_TAG.get() == header.r#type {
                return 0xf1234567;
            }
            if *d.outputCount != 0 {
                *d.versionIds = AVAILABLE.get();
                *d.versionNames = if VERSION_SHAPED_NAME.get() {
                    c"4.1.1".as_ptr()
                } else {
                    c"selected-fixture".as_ptr()
                };
            }
            *d.outputCount = 1;
            return 0;
        }
        let requested = if context.is_null() {
            if header.r#type == FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2 {
                let d = &*desc.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                assert_eq!(d.device as usize, DEVICE.get());
                selection(header.pNext)
            } else {
                let b = &*header.pNext.cast::<ffxCreateBackendDX12Desc>();
                assert_eq!(
                    b.header.r#type,
                    FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
                );
                assert_eq!(b.device as usize, DEVICE.get());
                selection(b.header.pNext)
            }
        } else {
            assert!(header.pNext.is_null());
            let root = &*(*context).cast::<ffxCreateContextDescUpscale>();
            let version = &*root
                .header
                .pNext
                .cast::<ffxCreateContextDescUpscaleVersion>();
            assert_eq!(version.version, FFX_UPSCALER_VERSION);
            let b = &*version.header.pNext.cast::<ffxCreateBackendDX12Desc>();
            assert_eq!(b.device as usize, DEVICE.get());
            selection(b.header.pNext)
        };
        SEEN.with_borrow_mut(|v| v.push((header.r#type, requested)));
        if FAIL_TAG.get() == header.r#type {
            return 0xf1234567;
        }
        match header.r#type {
            FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION => {
                let d = &mut *desc.cast::<ffxQueryGetProviderVersion>();
                d.versionId = if LIVE_OVERRIDE.get() != 0 {
                    LIVE_OVERRIDE.get()
                } else {
                    requested.unwrap_or(91)
                };
                d.versionName = c"effective-fixture".as_ptr();
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                d.required_resources = if context.is_null() { (1 << 63) | 3 } else { 3 };
                d.optional_resources = 3;
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2 => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                *d.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage {
                    totalUsageInBytes: 100,
                    aliasableUsageInBytes: 50,
                };
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode>();
                *d.pOutUpscaleRatio = 2.0;
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETRENDERRESOLUTIONFROMQUALITYMODE => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetRenderResolutionFromQualityMode>();
                *d.pOutRenderWidth = d.displayWidth - 1;
                *d.pOutRenderHeight = d.displayHeight - 1;
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTERPHASECOUNT => {
                *(*desc.cast::<ffxQueryDescUpscaleGetJitterPhaseCount>()).pOutPhaseCount = 4;
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GETJITTEROFFSET => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetJitterOffset>();
                *d.pOutX = -0.25;
                *d.pOutY = 0.25;
            }
            _ => return 0xe028,
        }
    }
    0
}

fn register(observer: &libloading::Library, device: &ID3D12Device) {
    DEVICE.set(device.as_raw() as usize);
    AVAILABLE.set(91);
    LIVE_OVERRIDE.set(0);
    FAIL_TAG.set(0);
    VERSION_SHAPED_NAME.set(false);
    SEEN.with_borrow_mut(Vec::clear);
    // SAFETY: Exact controlled export and synchronous matching observer signature.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(Some(observe));
    }
}

fn observed() -> Vec<(u64, Option<u64>)> {
    SEEN.with_borrow_mut(std::mem::take)
}

macro_rules! exercise {
    ($runtime:expr, $upscaler:ty) => {{
        let (mut runtime, observer) = $runtime;
        let (device, fake) = fake_device();
        register(&observer, &device);
        let id = unsafe { runtime.upscaler_dx12(&device) }.providers().unwrap()[0].id();
        let options = sizes();
        observed();
        assert_eq!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).ratio(QualityMode::Quality).unwrap(), 2.0);
        assert_eq!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).render_size_for_quality(options.max_upscale_size(), QualityMode::Quality).unwrap(), Dimensions::new(1919, 1079).unwrap());
        let phase = unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_phase_count(options.max_render_size(), options.max_upscale_size()).unwrap();
        assert_eq!(phase.count(), 4);
        assert_eq!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_offset(1, phase).unwrap().x(), -0.25);
        assert_eq!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).estimate_memory(&options).unwrap().total_bytes(), 100);
        let requirements = unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).resource_requirements().unwrap();
        assert_eq!(requirements.required().unknown_bits(), 1 << 63);
        assert_eq!(requirements.optional().bits(), 3);
        let calls = observed();
        for triple in calls.chunks_exact(3) {
            assert_eq!(triple[0], (FFX_API_QUERY_DESC_TYPE_GET_VERSIONS, None));
            assert_eq!(triple[1], triple[0]);
            assert_eq!(triple[2].1, Some(91));
        }
        assert_eq!(calls.len(), 18);
        // Default helpers neither inherit the options nor enumerate for selection.
        unsafe { runtime.upscaler_dx12(&device) }.ratio(QualityMode::Quality).unwrap();
        unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Default).resource_requirements().unwrap();
        unsafe { runtime.upscaler_dx12(&device) }.estimate_memory(&sizes()).unwrap();
        assert_eq!(observed().iter().map(|v| v.1).collect::<Vec<_>>(), [None, None, None]);
        // Invalid scalar checks precede even the fresh enumeration.
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_offset(4, phase), Err(Error::Input(crate::error::InputError::Query(QueryValidation::JitterIndex)))));
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).render_size_for_quality(Dimensions::new(u32::MAX, 1).unwrap(), QualityMode::Quality), Err(Error::Input(crate::error::InputError::Query(_)))));
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_phase_count(Dimensions::new(2, 1).unwrap(), Dimensions::new(1, 1).unwrap()), Err(Error::Input(crate::error::InputError::Query(_)))));
        assert!(observed().is_empty());
        // Absence and native enumeration errors must not enter selected creation/query.
        AVAILABLE.set(92);
        assert_result!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).ratio(QualityMode::Quality), Err(Error::Input(crate::error::InputError::ProviderNotEnumerated { requested: id })));
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options), Err(Error::Input(crate::error::InputError::ProviderNotEnumerated { .. }))));
        assert_eq!(fixture_snapshot(&observer).creates, 0);
        assert!(observed().iter().all(|v| v.0 == FFX_API_QUERY_DESC_TYPE_GET_VERSIONS));
        AVAILABLE.set(91); FAIL_TAG.set(FFX_API_QUERY_DESC_TYPE_GET_VERSIONS);
        assert_result!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).resource_requirements(), Err(Error::native(Operation::Query, 0xf1234567)));
        FAIL_TAG.set(FFX_API_QUERY_DESC_TYPE_UPSCALE_GETUPSCALERATIOFROMQUALITYMODE);
        assert_result!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).ratio(QualityMode::Quality), Err(Error::native(Operation::Query, 0xf1234567)));
        FAIL_TAG.set(0); observed();
        let context = unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options).unwrap();
        assert_eq!(fixture_snapshot(&observer).chain_valid, 1);
        let sequence = observed();
        assert_eq!(sequence.iter().map(|v| v.0).collect::<Vec<_>>(), [4, 4, 6, FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS]);
        let mut moved = Box::new(context);
        assert_eq!(moved.provider().unwrap().id(), id);
        moved.destroy().unwrap();
        assert_eq!(fake.references.get(), 1);
        // SAFETY: Exact scalar fixture export; destroy reread the retained node.
        assert_eq!(unsafe { observer.get::<extern "C" fn() -> u64>(c"fixture_destroyed_provider").unwrap()() }, 91);
        // A substituted provider and failed identity query both destroy exactly once.
        LIVE_OVERRIDE.set(92);
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options), Err(Error::Invariant { operation: crate::error::Operation::Create, reason: InvariantViolation::ProviderSelectionMismatch { requested, .. } }) if requested == id));
        assert_eq!(fixture_snapshot(&observer).destroys, 2);
        assert_eq!(fake.references.get(), 1);
        LIVE_OVERRIDE.set(0); FAIL_TAG.set(FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION);
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options), Err(Error::Native(crate::error::NativeError { operation: Operation::Query, code: 0xf1234567 }))));
        assert_eq!(fixture_snapshot(&observer).destroys, 3);
        assert_eq!(fake.references.get(), 1);
        FAIL_TAG.set(0); LIVE_OVERRIDE.set(92); fixture_set(&observer, 0, 0xf001, 0);
        assert!(matches!(unsafe { runtime.upscaler_dx12(&device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options), Err(Error::Invariant { operation: crate::error::Operation::Create, reason: InvariantViolation::ProviderSelectionMismatch { .. } })));
        assert_eq!(fixture_snapshot(&observer).destroys, 4);
        assert_eq!(fake.references.get(), 2); // Failed destroy retains its complete bundle.
    }};
}

#[test]
fn selected_provider_routes_and_failure_cleanup_cover_both_families() {
    exercise!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    exercise!(fixture_movable_runtime(), MovableUpscaler);
}

#[cfg(feature = "utils")]
#[test]
fn utility_selection_uses_public_enumeration_options_and_both_families() {
    use crate::utils::provider::{FsrChoice, SelectionError, select_provider};
    macro_rules! resolve {
        ($runtime:expr, $upscaler:ty) => {{
            let (mut runtime, observer) = $runtime;
            let (device, fake) = fake_device();
            register(&observer, &device);
            let providers = unsafe { runtime.upscaler_dx12(&device) }
                .providers()
                .unwrap();
            assert_eq!(
                select_provider(&providers, FsrChoice::Fsr4),
                Err(SelectionError::NoRecognizedMatch {
                    requested: FsrChoice::Fsr4
                })
            );
            VERSION_SHAPED_NAME.set(true);
            let providers = unsafe { runtime.upscaler_dx12(&device) }
                .providers()
                .unwrap();
            let selected = select_provider(&providers, FsrChoice::Fsr4).unwrap();
            let options = sizes();

            let mut context = unsafe { runtime.upscaler_dx12(&device) }
                .with_provider(selected)
                .create(&options)
                .unwrap();
            assert_eq!(
                crate::query::ProviderSelection::Explicit(context.provider().unwrap().id()),
                selected
            );
            context.destroy().unwrap();
            assert_eq!(fixture_snapshot(&observer).destroys, 1);
            assert_eq!(fake.references.get(), 1);
        }};
    }
    resolve!(fixture_runtime(), Upscaler);
    #[cfg(feature = "movable")]
    resolve!(fixture_movable_runtime(), MovableUpscaler);
}

#[test]
fn setup_rebinds_future_calls_and_contexts_outlive_all_borrows() {
    use crate::query::ProviderSelection;
    macro_rules! exercise_setup {
        ($make:expr) => {{
            let (mut runtime, observer) = $make;
            let (device, fake) = fake_device();
            register(&observer, &device);
            assert_eq!(fixture_snapshot(&observer).creates, 0);
            assert_eq!(fixture_snapshot(&observer).destroys, 0);
            assert!(observed().is_empty());
            // SAFETY: Synchronous controlled fixture, no workers/GPU/affinity;
            // every safely reachable selection has the same bounded behaviour.
            let mut setup = unsafe { runtime.upscaler_dx12(&device) };
            assert_eq!(setup.selection(), ProviderSelection::Default);
            setup = setup.with_provider(ProviderSelection::Default);
            assert!(observed().is_empty()); // Acquisition/rebinding call no FFX export.
            assert_eq!(fake.references.get(), 1); // Setup borrows the device.
            let id = setup.providers().unwrap()[0].id();
            setup = setup.with_provider(ProviderSelection::Explicit(id));
            let options = sizes();
            let mut first = setup.create(&options).unwrap();
            let mut second = setup.create(&options).unwrap();
            setup = setup.with_provider(ProviderSelection::Default);
            setup.ratio(QualityMode::Quality).unwrap();
            assert_eq!(observed().last().unwrap().1, None);
            AVAILABLE.set(92);
            setup = setup.with_provider(ProviderSelection::Explicit(id));
            assert!(matches!(
                setup.create(&options),
                Err(Error::Input(
                    crate::error::InputError::ProviderNotEnumerated { .. }
                ))
            ));
            assert_eq!(fixture_snapshot(&observer).creates, 2);
            // Rust ends the setup borrow before dropping the public runtime/device.
            // Both created owners retain their own native dependencies and chains.
            drop(runtime);
            drop(device);
            assert_eq!(fake.references.get(), 2);
            assert_eq!(first.provider().unwrap().id(), id);
            assert_eq!(second.provider().unwrap().id(), id);
            first.destroy().unwrap();
            second.destroy().unwrap();
            assert_eq!(fixture_snapshot(&observer).destroys, 2);
            assert_eq!(fake.references.get(), 0);
        }};
    }
    exercise_setup!(fixture_runtime());
    #[cfg(feature = "movable")]
    exercise_setup!(fixture_movable_runtime());
}
