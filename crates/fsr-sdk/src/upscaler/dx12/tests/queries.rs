// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};
use std::{
    ffi::c_void,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
};

static DEVICE: AtomicUsize = AtomicUsize::new(0);
static REQUIRED: AtomicU64 = AtomicU64::new(3);
static QUERY_CODE: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn observe(context: *mut *mut c_void, desc: *mut c_void) -> u32 {
    let code = QUERY_CODE.load(Ordering::SeqCst);
    if code != 0 {
        return code;
    }
    // SAFETY: Registered only for these public methods' complete tagged
    // descriptors. All outputs are bounded, initialized and consumed synchronously.
    unsafe {
        let header = &*desc.cast::<ffxApiHeader>();
        if !header.pNext.is_null() {
            return 0xe001;
        }
        match header.r#type {
            FFX_API_QUERY_DESC_TYPE_GET_VERSIONS => {
                let d = &mut *desc.cast::<ffxQueryDescGetVersions>();
                if !context.is_null()
                    || d.device as usize != DEVICE.load(Ordering::SeqCst)
                    || d.createDescType != FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE
                {
                    return 0xe002;
                }
                if *d.outputCount != 0 {
                    *d.versionIds = 91;
                    *d.versionNames = c"fixture-provider".as_ptr();
                }
                *d.outputCount = 1;
            }
            FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2 => {
                let d = &mut *desc.cast::<ffxQueryDescUpscaleGetGPUMemoryUsageV2>();
                if !context.is_null()
                    || d.device as usize != DEVICE.load(Ordering::SeqCst)
                    || d.flags != 0
                    || d.maxRenderSize.width != 1280
                    || d.maxRenderSize.height != 720
                    || d.maxUpscaleSize.width != 1920
                    || d.maxUpscaleSize.height != 1080
                {
                    return 0xe003;
                }
                *d.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage {
                    totalUsageInBytes: 4096,
                    aliasableUsageInBytes: 2048,
                };
            }
            tag => {
                if context.is_null() || (*context).is_null() {
                    return 0xe004;
                }
                match tag {
                    FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION => {
                        let d = &mut *desc.cast::<ffxQueryGetProviderVersion>();
                        d.versionId = 91;
                        d.versionName = c"fixture-provider".as_ptr();
                    }
                    FFX_API_QUERY_DESC_TYPE_UPSCALE_GET_RESOURCE_REQUIREMENTS => {
                        let d = &mut *desc.cast::<ffxQueryDescUpscaleGetResourceRequirements>();
                        d.required_resources = REQUIRED.load(Ordering::SeqCst);
                        d.optional_resources = 0x30;
                    }
                    FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE => {
                        let d = &mut *desc.cast::<ffxQueryDescUpscaleGetGPUMemoryUsage>();
                        *d.gpuMemoryUsageUpscaler = FfxApiEffectMemoryUsage {
                            totalUsageInBytes: 8192,
                            aliasableUsageInBytes: 1024,
                        };
                    }
                    _ => return 0xe005,
                }
            }
        }
    }
    0
}

fn register(observer: &libloading::Library) {
    REQUIRED.store(3, Ordering::SeqCst);
    // SAFETY: Matched controlled fixture export and synchronous query observer.
    unsafe {
        observer.get::<extern "C" fn(Option<unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32>)>(c"fixture_set_query_observer").unwrap()(Some(observe));
    }
}

fn calls(observer: &libloading::Library) -> u32 {
    // SAFETY: Exact controlled fixture export returning its own counter.
    unsafe {
        observer
            .get::<extern "C" fn() -> u32>(c"fixture_query_calls")
            .unwrap()()
    }
}

#[test]
fn public_query_methods_share_invocation_and_poison_policy_for_both_families() {
    QUERY_CODE.store(0, Ordering::SeqCst);
    let (device, _) = fake_device();
    DEVICE.store(device.as_raw() as usize, Ordering::SeqCst);
    let (mut runtime, observer) = fixture_runtime();
    register(&observer);
    let providers = unsafe { runtime.upscaler_dx12(&device) }
        .providers()
        .unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name(), "fixture-provider");
    assert_eq!(
        unsafe { runtime.upscaler_dx12(&device) }
            .estimate_memory(&sizes())
            .unwrap()
            .total_bytes(),
        4096
    );
    let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    REQUIRED.store((1 << 63) | 3, Ordering::SeqCst);
    let info = upscaler.provider().unwrap();
    assert_eq!(info.id(), providers[0].id());
    assert_eq!(
        upscaler
            .resource_requirements()
            .unwrap()
            .required()
            .unknown_bits(),
        1 << 63
    );
    assert_eq!(upscaler.gpu_memory_usage().unwrap().total_bytes(), 8192);
    QUERY_CODE.store(0xf1234567, Ordering::SeqCst);
    assert!(matches!(
        upscaler.provider(),
        Err(Error::Native(crate::error::NativeError {
            operation: crate::error::Operation::Query,
            code: 0xf1234567
        }))
    ));
    assert_eq!(upscaler.core.eligibility, Eligibility::Usable);
    QUERY_CODE.store(0, Ordering::SeqCst);
    assert!(upscaler.provider().is_ok());
    upscaler.core.eligibility = crate::context_policy::failed(crate::error::Operation::Upscale, 3);
    let before = calls(&observer);
    assert_result!(
        upscaler.provider(),
        Err(crate::test_errors::terminal(Error::native(
            crate::error::Operation::Upscale,
            3
        )))
    );
    assert_result!(
        upscaler.resource_requirements(),
        Err(crate::test_errors::terminal(Error::native(
            crate::error::Operation::Upscale,
            3
        )))
    );
    assert_result!(
        upscaler.gpu_memory_usage(),
        Err(crate::test_errors::terminal(Error::native(
            crate::error::Operation::Upscale,
            3
        )))
    );
    assert_eq!(calls(&observer), before);
    assert!(
        unsafe { runtime.upscaler_dx12(&device) }
            .providers()
            .is_ok()
    );
    upscaler.destroy().unwrap();
    assert_eq!(info.name(), "fixture-provider");

    #[cfg(feature = "movable")]
    {
        let (mut runtime, observer) = fixture_movable_runtime();
        register(&observer);
        assert_eq!(
            unsafe { runtime.upscaler_dx12(&device) }
                .providers()
                .unwrap()[0]
                .id(),
            info.id()
        );
        assert_eq!(
            unsafe { runtime.upscaler_dx12(&device) }
                .estimate_memory(&sizes())
                .unwrap()
                .total_bytes(),
            4096
        );
        let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
            .create(&sizes())
            .unwrap();
        REQUIRED.store((1 << 63) | 3, Ordering::SeqCst);
        assert_eq!(upscaler.provider().unwrap(), info);
        assert_eq!(
            upscaler
                .resource_requirements()
                .unwrap()
                .required()
                .unknown_bits(),
            1 << 63
        );
        assert_eq!(upscaler.gpu_memory_usage().unwrap().total_bytes(), 8192);
        upscaler.core.eligibility =
            crate::context_policy::failed(crate::error::Operation::Upscale, 3);
        let before = calls(&observer);
        assert_result!(
            upscaler.provider(),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            upscaler.resource_requirements(),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_result!(
            upscaler.gpu_memory_usage(),
            Err(crate::test_errors::terminal(Error::native(
                crate::error::Operation::Upscale,
                3
            )))
        );
        assert_eq!(calls(&observer), before);
        upscaler.destroy().unwrap();
    }
}

macro_rules! selected_native_queries {
    ($runtime:ident, $device:ident, $upscaler:ty, $provider:expr) => {{
        let id = $provider;
        let options = sizes();

        let ratio = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).ratio(crate::query::QualityMode::Quality).unwrap();
        let render = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).render_size_for_quality(options.max_upscale_size(), crate::query::QualityMode::Quality).unwrap();
        let phase = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_phase_count(render, options.max_upscale_size()).unwrap();
        let jitter = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).jitter_offset(0, phase).unwrap();
        let estimate = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).estimate_memory(&options).unwrap();
        let requirements = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).resource_requirements().unwrap();
        let mut selected = unsafe { $runtime.upscaler_dx12(&$device) }.with_provider(crate::query::ProviderSelection::Explicit(id)).create(&options).unwrap();
        assert_eq!(selected.provider().unwrap().id(), id);
        eprintln!("Selected provider queries: provider={id:?}; ratio={ratio}; render={render:?}; phase={phase:?}; jitter={jitter:?}; estimate={estimate:?}; requirements={requirements:?}");
        selected.destroy().unwrap();
    }};
}

#[test]
#[ignore = "requires the staged trusted AMD v2.3.0 runtime and DX12 helper; use the native-upscaler profile"]
fn native_public_queries() {
    use std::path::PathBuf;
    let helper_path = std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("native device helper");
    let loader =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("trusted native loader"));
    // SAFETY: Runner stages the trusted helper with this paired device signature.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Paired helper transfers one device reference into writable output.
    let create = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .unwrap();
    let mut raw = null_mut();
    // SAFETY: Valid writable output for the trusted helper.
    assert!(unsafe { create(&mut raw) } >= 0 && !raw.is_null());
    // SAFETY: Adopt the helper's independently owned live COM device reference.
    let device = unsafe { ID3D12Device::from_raw(raw.cast()) };
    // SAFETY: Opt-in selects trusted compatible providers under D018's query
    // assumptions; this isolated child serializes every query and cleanup.
    let mut runtime = unsafe { Runtime::load(&loader) }.unwrap();
    let providers = unsafe { runtime.upscaler_dx12(&device) }
        .providers()
        .unwrap();
    assert!(!providers.is_empty());
    let estimate = unsafe { runtime.upscaler_dx12(&device) }
        .estimate_memory(&sizes())
        .unwrap();
    let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    let provider = upscaler.provider().unwrap();
    let requirements = upscaler.resource_requirements().unwrap();
    let usage = upscaler.gpu_memory_usage().unwrap();
    eprintln!(
        "Local queries: provider={provider:?}; estimate={estimate:?}; requirements={requirements:?}; usage={usage:?}"
    );
    selected_native_queries!(runtime, device, Upscaler, provider.id());
    upscaler.destroy().unwrap();
    drop(runtime);
    #[cfg(feature = "movable")]
    {
        // SAFETY: Same isolated serialized trusted configuration; movable
        // compatibility is the existing lasting acquisition assumption.
        let mut runtime = unsafe { crate::runtime::MovableRuntime::load(&loader) }.unwrap();
        assert!(
            !unsafe { runtime.upscaler_dx12(&device) }
                .providers()
                .unwrap()
                .is_empty()
        );
        let estimate = unsafe { runtime.upscaler_dx12(&device) }
            .estimate_memory(&sizes())
            .unwrap();
        let mut upscaler = unsafe { runtime.upscaler_dx12(&device) }
            .create(&sizes())
            .unwrap();
        let provider = upscaler.provider().unwrap();
        let requirements = upscaler.resource_requirements().unwrap();
        let usage = upscaler.gpu_memory_usage().unwrap();
        eprintln!(
            "Movable queries: provider={provider:?}; estimate={estimate:?}; requirements={requirements:?}; usage={usage:?}"
        );
        selected_native_queries!(runtime, device, MovableUpscaler, provider.id());
        upscaler.destroy().unwrap();
    }
    eprintln!("Public queries: PASSED");
}
