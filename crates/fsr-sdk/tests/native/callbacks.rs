// SPDX-License-Identifier: MPL-2.0

//! Public wrapper callback installation with external process-lived C++ targets.
use super::*;
use crate::{
    error::Operation,
    upscaler::{DebugLevel, dx12::interop::*},
};
use fsr_sdk_sys::{
    api::{ffxAllocationCallbacks, ffxApiMessage},
    dx12::ffxCreateBackendDX12AllocationCallbacksDesc,
};

type Tables = unsafe extern "C" fn(
    u32,
    *mut ffxAllocationCallbacks,
    *mut ffxCreateBackendDX12AllocationCallbacksDesc,
    *mut ffxApiMessage,
);
unsafe fn callbacks(tables: Tables, set: u32) -> (CreationCallbacks, MessageCallback) {
    let (mut host, mut dx12, mut message) = (
        std::mem::MaybeUninit::uninit(),
        std::mem::MaybeUninit::uninit(),
        None,
    );
    // SAFETY: Exact paired C++ export fully initialises both production ABI tables.
    let (host, dx12) = unsafe {
        tables(set, host.as_mut_ptr(), dx12.as_mut_ptr(), &mut message);
        (host.assume_init(), dx12.assume_init())
    };
    let mut d = Dx12AllocationCallbacks::default();
    d.set_resources(Some(ResourceAllocationCallbacks::new(
        dx12.pfnFfxResourceAllocator.unwrap(),
        dx12.pfnFfxResourceDeallocator.unwrap(),
    )));
    d.set_heaps(Some(HeapAllocationCallbacks::new(
        dx12.pfnFfxHeapAllocator.unwrap(),
        dx12.pfnFfxHeapDeallocator.unwrap(),
    )));
    d.set_constant_buffer_allocator(dx12.pfnFfxConstantBufferAllocator);
    let mut c = CreationCallbacks::default();
    c.set_host_allocator(Some(HostAllocationCallbacks::new(
        host.pUserData,
        host.alloc.unwrap(),
        host.dealloc.unwrap(),
    )));
    c.set_dx12_allocations(d);
    c.set_message_callback(message);
    (c, message.unwrap())
}

fn frame(
    helper: &libloading::Library,
    device: &ID3D12Device,
    draw: bool,
    mut dispatch: impl FnMut(UpscaleTestInputs<'_>) -> Result<(), Error>,
) {
    // SAFETY: All exports are compiled by the opt-in runner from local equipment.
    unsafe {
        let create = *helper.get::<CreateHarness>(c"gpu_create").unwrap();
        let finish = *helper.get::<Finish>(c"gpu_finish").unwrap();
        let destroy = *helper.get::<Destroy>(c"gpu_destroy").unwrap();
        let (mut harness, mut list, mut raw) = (null_mut(), null_mut(), [null_mut(); 5]);
        assert_eq!(
            create(device.as_raw(), &mut harness, &mut list, raw.as_mut_ptr()),
            0
        );
        let list = ID3D12GraphicsCommandList::from_raw(list);
        let resources: Vec<_> = raw
            .into_iter()
            .map(|p| ID3D12Resource::from_raw(p))
            .collect();
        let mut d = UpscaleTestInputs::new(
            &list,
            UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                .output(&resources[4])
                .build()
                .unwrap(),
            JitterOffsetPixels::new(0.25, -0.25).unwrap(),
            FrameTimeMillis::new(0.5).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
            true,
        );
        d.set_draw_debug_view(draw);
        if let Err(e) = dispatch(d) {
            abort_dispatch(e, 0);
        }
        let mut metrics = Metrics::default();
        if finish(harness, &mut metrics) < 0 {
            std::process::exit(1);
        }
        assert_eq!(
            (
                metrics.width,
                metrics.height,
                metrics.finite_pixels,
                metrics.changed_pixels
            ),
            (640, 360, 230400, 230400)
        );
        assert_eq!((metrics.debug_errors, metrics.device_removed), (0, 0));
        eprintln!(
            "PUBLIC_FRAME draw={draw} finite={} written={} errors={} removed={}",
            metrics.finite_pixels,
            metrics.changed_pixels,
            metrics.debug_errors,
            metrics.device_removed
        );
        destroy(harness);
    }
}
fn legacy_result(result: Result<(), Error>, family: &str, provider: &str, live: bool) {
    let code = match result {
        Ok(()) => 0,
        Err(Error::Native(failure))
            if failure.operation()
                == if live {
                    Operation::ConfigureContextDebugLegacy
                } else {
                    Operation::ConfigureGlobalDebugLegacy
                } =>
        {
            failure.code()
        }
        other => panic!("Unexpected diagnostic outcome: {other:?}"),
    };
    eprintln!("PUBLIC_LEGACY family={family} provider={provider} live={live} code={code}");
    assert!(
        code == 0 || code == 2,
        "Unexpected pinned-runtime diagnostic outcome"
    );
}

#[cfg(feature = "movable")]
#[test]
#[ignore = "public callback integration; use native-callbacks"]
fn public_native_callbacks() {
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").unwrap());
    // SAFETY: This process is the runner's isolated child. The helper ABI is
    // pinned by compilation; all SDK calls/cleanup are serial and fenced.
    unsafe {
        let helper =
            libloading::Library::new(std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap()).unwrap();
        let equipment =
            libloading::Library::new(std::env::var_os("FSR_SDK_TEST_CALLBACK_DLL").unwrap())
                .unwrap();
        let enable = *helper.get::<EnableDebug>(c"gpu_enable_debug").unwrap();
        assert!(enable() >= 0);
        let create = *helper.get::<CreateDevice>(c"probe_create_device").unwrap();
        let require = *helper.get::<DeviceCheck>(c"gpu_require_amd").unwrap();
        let check = *helper.get::<DeviceCheck>(c"gpu_check_device").unwrap();
        let mut raw = null_mut();
        assert!(create(&mut raw) >= 0);
        let device = ID3D12Device::from_raw(raw);
        assert_eq!(require(device.as_raw()), 0);
        equipment
            .get::<unsafe extern "C" fn(*mut c_void)>(c"callbacks_device")
            .unwrap()(device.as_raw());
        equipment
            .get::<unsafe extern "C" fn()>(c"callbacks_heap_control")
            .unwrap()();
        let tables = *equipment.get::<Tables>(c"callbacks_tables").unwrap();
        macro_rules! family {
            ($runtime:ty,$owner:ty,$name:literal) => {{
                let mut runtime = <$runtime>::load(&loader).unwrap();
                let providers = runtime.upscaler_dx12(&device).providers().unwrap();
                for version in ["3.1.5", "4.1.1"] {
                    let selected = providers
                        .iter()
                        .find(|p| p.name() == version)
                        .expect("selected pinned provider")
                        .id();
                    let mut options = UpscalerOptions::new(
                        Dimensions::new(320, 180).unwrap(),
                        Dimensions::new(640, 360).unwrap(),
                    );

                    options.set_debug_checking(true);
                    options.set_debug_visualization(true);
                    runtime
                        .upscaler_dx12(&device)
                        .with_provider(crate::query::ProviderSelection::Explicit(selected))
                        .estimate_memory(&options)
                        .unwrap();
                    let (c, m) = callbacks(tables, 1);
                    let mut a = runtime
                        .upscaler_dx12(&device)
                        .with_provider(crate::query::ProviderSelection::Explicit(selected))
                        .create_with_callbacks(&options, c)
                        .unwrap();
                    assert_eq!(a.provider().unwrap().id(), selected);
                    runtime
                        .upscaler_dx12(&device)
                        .configure_debug(DebugLevel::Verbose, Some(m))
                        .unwrap();
                    frame(&helper, &device, false, |d| {
                        a.record(d.command_list, &d.resources, &d.parameters)
                    });
                    let (c, m) = callbacks(tables, 2);
                    let mut b = runtime
                        .upscaler_dx12(&device)
                        .with_provider(crate::query::ProviderSelection::Explicit(selected))
                        .create_with_callbacks(&options, c)
                        .unwrap();
                    assert_eq!(b.provider().unwrap().id(), selected);
                    runtime
                        .upscaler_dx12(&device)
                        .configure_debug(DebugLevel::Warnings, Some(m))
                        .unwrap();
                    frame(&helper, &device, true, |d| {
                        a.record(d.command_list, &d.resources, &d.parameters)
                    });
                    frame(&helper, &device, false, |d| {
                        b.record(d.command_list, &d.resources, &d.parameters)
                    });
                    let mut omitted = runtime
                        .upscaler_dx12(&device)
                        .with_provider(crate::query::ProviderSelection::Explicit(selected))
                        .create(&options)
                        .unwrap();
                    assert_eq!(omitted.provider().unwrap().id(), selected);
                    frame(&helper, &device, true, |d| {
                        omitted.record(d.command_list, &d.resources, &d.parameters)
                    });
                    legacy_result(
                        runtime
                            .upscaler_dx12(&device)
                            .configure_global_debug_legacy(DebugLevel::Warnings, Some(m)),
                        $name,
                        version,
                        false,
                    );
                    // Bounded pinned-runtime continuation after the null-context
                    // unsupported route: no affected recording/work is pending.
                    frame(&helper, &device, false, |d| {
                        a.record(d.command_list, &d.resources, &d.parameters)
                    });
                    legacy_result(
                        b.configure_global_debug_legacy(DebugLevel::Warnings, Some(m)),
                        $name,
                        version,
                        true,
                    );
                    b.destroy().unwrap();
                    omitted.destroy().unwrap();
                    a.destroy().unwrap();
                    runtime
                        .upscaler_dx12(&device)
                        .configure_debug(DebugLevel::Silent, None)
                        .unwrap();
                    eprintln!(
                        "PUBLIC_CONTEXTS family={} provider={version} created=3 destroyed=3",
                        $name
                    );
                }
            }};
        }
        family!(Runtime, Upscaler, "local");
        family!(
            crate::runtime::MovableRuntime,
            crate::upscaler::MovableUpscaler,
            "movable"
        );
        assert_eq!(check(device.as_raw()), 0);
        equipment
            .get::<unsafe extern "C" fn()>(c"callbacks_balance")
            .unwrap()();
        // All callback targets, device references, CB allocations and code remain
        // process-live even after ordinary context destruction/global nulling.
        std::mem::forget(equipment);
        std::mem::forget(helper);
        eprintln!("Public callbacks: PASSED contexts=12 frames=20");
    }
}
