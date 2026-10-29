// SPDX-License-Identifier: MPL-2.0

//! Opt-in CPU baseline. No production code or public contract changes.
use super::super::dispatch::inspect_dispatch;
use super::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    time::Instant,
};

thread_local! {
    static ALLOCATIONS: Cell<Option<[u64; 4]>> = const { Cell::new(None) };
    static COM_CALLS: Cell<Option<[u64; 6]>> = const { Cell::new(None) };
    static ALLOCATION_BYTES: Cell<Option<u64>> = const { Cell::new(None) };
}

struct CountingAllocator;

fn allocation(index: usize) {
    let _ = ALLOCATIONS.try_with(|slot| {
        if let Some(mut counts) = slot.get() {
            counts[index] += 1;
            slot.set(Some(counts));
        }
    });
}

// SAFETY: Every operation forwards unchanged to System. TLS counters allocate
// nothing, and count only explicitly armed operations on the benchmark thread.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocation(0);
        allocation_bytes(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        allocation(1);
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocation(2);
        allocation_bytes(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        allocation(3);
        allocation_bytes(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

// Present only in the Windows/DX12 library test executable, never the crate.
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn allocation_bytes(bytes: usize) {
    let _ = ALLOCATION_BYTES.try_with(|slot| {
        if let Some(total) = slot.get() {
            slot.set(Some(total + bytes as u64));
        }
    });
}

pub(super) fn begin_allocations() {
    ALLOCATIONS.with(|slot| slot.set(Some([0; 4])));
    ALLOCATION_BYTES.with(|slot| slot.set(Some(0)));
}

pub(super) fn end_allocations() -> ([u64; 4], u64) {
    let events = ALLOCATIONS.with(|slot| slot.replace(None).unwrap());
    let bytes = ALLOCATION_BYTES.with(|slot| slot.replace(None).unwrap());
    (events, bytes)
}

pub(super) fn com_call(index: usize) {
    COM_CALLS.with(|slot| {
        if let Some(mut counts) = slot.get() {
            counts[index] += 1;
            slot.set(Some(counts));
        }
    });
}

fn counts<T>(name: &str, action: impl FnOnce() -> T) -> T {
    ALLOCATIONS.with(|slot| slot.set(Some([0; 4])));
    COM_CALLS.with(|slot| slot.set(Some([0; 6])));
    let result = action();
    let allocations = ALLOCATIONS.with(|slot| slot.replace(None).unwrap());
    let com = COM_CALLS.with(|slot| slot.replace(None).unwrap());
    eprintln!("PERF_COUNTS {name} allocations={allocations:?} com={com:?}");
    result
}

fn time<T>(iterations: u32, mut action: impl FnMut() -> T) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(action());
    }
    start.elapsed().as_secs_f64() * 1e9 / f64::from(iterations)
}

pub(super) fn require_optimized() {
    if cfg!(debug_assertions) {
        panic!("performance requires --release");
    }
}

// The direct ABI reference shares creation mapping and a pre-inspected dispatch
// descriptor. It isolates wrapper work; it is not an independent C++ consumer.
pub(super) struct RawContext<'a> {
    library: &'a fsr_sdk_sys::loader::FfxLibrary,
    pub(super) handle: ffxContext,
    _state: Pin<Box<UpscaleCreateState>>,
    _device: ID3D12Device,
}

impl<'a> RawContext<'a> {
    pub(super) fn new(
        library: &'a fsr_sdk_sys::loader::FfxLibrary,
        device: &ID3D12Device,
        options: &UpscalerOptions,
    ) -> Self {
        let device = device.clone();
        let (state, root) =
            UpscaleCreateState::new(device.as_raw().cast(), options, Default::default(), None);
        let mut handle = null_mut();
        // SAFETY: Same pinned descriptors/device and trusted ABI as the wrapper;
        // all dependencies survive explicit terminal destruction below.
        assert_eq!(
            unsafe { library.ffxCreateContext(&mut handle, root, null_mut()) },
            0
        );
        assert!(!handle.is_null());
        Self {
            library,
            handle,
            _state: state,
            _device: device,
        }
    }

    pub(super) unsafe fn dispatch(&mut self, desc: &ffxDispatchDescUpscale) {
        // SAFETY: Caller supplies truthful states and retains/fences all objects.
        let code = unsafe { self.library.ffxDispatch(&mut self.handle, &desc.header) };
        if code != 0 {
            eprintln!("PERF raw dispatch failed: {code:#x}");
            std::process::exit(1); // Do not submit or unwind in-flight dependencies.
        }
    }

    pub(super) fn destroy(mut self) {
        self.teardown();
    }

    fn teardown(&mut self) {
        if self.handle.is_null() {
            return;
        }
        // SAFETY: Every recorded frame was submitted once and fenced, or this
        // is a controlled no-GPU fixture. Dependencies remain live through FFI.
        let code = unsafe { self.library.ffxDestroyContext(&mut self.handle, null_mut()) };
        if code != 0 {
            eprintln!("PERF raw destruction failed: {code:#x}");
            std::process::exit(1);
        }
        self.handle = null_mut();
    }
}

impl Drop for RawContext<'_> {
    fn drop(&mut self) {
        // All probe errors with possibly in-flight/abandoned recordings exit
        // the process. Remaining unwind paths occur before work or after fences.
        self.teardown();
    }
}

#[test]
#[ignore = "opt-in optimized performance experiment; use experiment-performance"]
fn controlled_cpu_baseline() {
    require_optimized();
    let (path, _observer) = fixture_library_with_optimization(true);
    // SAFETY: Locally compiled controlled fixture; no GPU work or hidden threads.
    let mut runtime = unsafe { Runtime::load(&path) }.unwrap();
    let scene = com::Scene::new();
    let options = sizes();
    let mut wrapper = unsafe { runtime.upscaler_dx12(&scene.device) }
        .create(&options)
        .unwrap();
    // SAFETY: This exact local fixture records no GPU work.
    let raw_library = unsafe { fsr_sdk_sys::loader::FfxLibrary::load(&path) }.unwrap();
    let mut raw = RawContext::new(&raw_library, &scene.device, &options);
    let desc = inspect_dispatch(&scene.device, &options, scene.input()).unwrap();
    let references = scene.reference_counts();
    // SAFETY: Controlled fixture and COM stubs only; no real GPU commands.
    unsafe {
        for _ in 0..10_000 {
            {
                let recording = scene.input();
                wrapper.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            }
            .unwrap();
            raw.dispatch(&desc);
        }
        counts("dispatch", || {
            {
                let recording = scene.input();
                wrapper.record(
                    recording.command_list,
                    &recording.resources,
                    &recording.parameters,
                )
            }
            .unwrap()
        });
        counts("inspect", || {
            inspect_dispatch(&scene.device, &options, scene.input()).unwrap()
        });
        counts("raw", || raw.dispatch(&desc));
        counts("create_destroy", || {
            runtime
                .upscaler_dx12(&scene.device)
                .create(&options)
                .unwrap()
                .destroy()
                .unwrap();
        });
        for sample in 0..31 {
            // Alternate ordering to expose rather than hide drift.
            let mut values = [0.0; 4];
            for step in 0..4 {
                let arm = (step + sample) % 4;
                values[arm] = match arm {
                    0 => time(1000, || black_box(&desc)),
                    1 => time(1000, || raw.dispatch(black_box(&desc))),
                    2 => time(1000, || {
                        inspect_dispatch(&scene.device, &options, black_box(scene.input())).unwrap()
                    }),
                    _ => time(1000, || {
                        {
                            let recording = black_box(scene.input());
                            wrapper.record(
                                recording.command_list,
                                &recording.resources,
                                &recording.parameters,
                            )
                        }
                        .unwrap()
                    }),
                };
            }
            eprintln!(
                "PERF_SAMPLE controlled {sample} empty={} raw={} inspect={} wrapper={}",
                values[0], values[1], values[2], values[3]
            );
        }
    }
    // Separate lifecycle samples; no dispatch/GPU work is included.
    for sample in 0..31 {
        let mut owners = Vec::with_capacity(1000);
        let start = Instant::now();
        for _ in 0..1000 {
            owners.push(
                unsafe { runtime.upscaler_dx12(&scene.device) }
                    .create(&options)
                    .unwrap(),
            );
        }
        let create = start.elapsed().as_secs_f64() * 1e6;
        let start = Instant::now();
        while let Some(owner) = owners.pop() {
            owner.destroy().unwrap();
        }
        let destroy = start.elapsed().as_secs_f64() * 1e6;
        eprintln!("PERF_SAMPLE controlled_lifecycle {sample} create={create} destroy={destroy}");
    }
    assert_eq!(scene.reference_counts(), references);
    raw.destroy();
    wrapper.destroy().unwrap();
    eprintln!("PERF controlled: PASSED");
}

#[repr(C)]
#[derive(Default)]
struct Metrics {
    width: u32,
    height: u32,
    finite: u32,
    changed: u32,
    min: f32,
    max: f32,
    left: f32,
    right: f32,
    debug_errors: u64,
    removed: i32,
}

#[test]
#[ignore = "trusted Windows SDK/GPU performance experiment; use experiment-performance"]
fn native_cpu_baseline() {
    require_optimized();
    let diagnostics = std::env::var("FSR_SDK_PERFORMANCE_DEBUG").unwrap() == "1";
    // SAFETY: Runner compiles this helper, verifies pinned signed AMD binaries
    // and serializes this isolated child. All signatures match their C exports.
    unsafe {
        let helper =
            libloading::Library::new(std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap()).unwrap();
        if diagnostics {
            let debug = helper
                .get::<extern "C" fn() -> i32>(c"gpu_enable_debug")
                .unwrap();
            assert_eq!(debug(), 0);
        }
        let create_device = helper
            .get::<extern "C" fn(*mut *mut std::ffi::c_void) -> i32>(c"probe_create_device")
            .unwrap();
        let mut pointer = null_mut();
        assert_eq!(create_device(&mut pointer), 0);
        let device = ID3D12Device::from_raw(pointer);
        let amd = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(c"gpu_require_amd")
            .unwrap();
        assert_eq!(amd(device.as_raw()), 0);
        type Create = extern "C" fn(
            *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
        ) -> i32;
        let create = helper.get::<Create>(c"gpu_create").unwrap();
        let finish = helper
            .get::<extern "C" fn(*mut std::ffi::c_void, u32, *mut Metrics) -> i32>(
                c"performance_finish",
            )
            .unwrap();
        let destroy = helper
            .get::<extern "C" fn(*mut std::ffi::c_void)>(c"gpu_destroy")
            .unwrap();
        let prepare = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(c"performance_prepare_readback")
            .unwrap();
        let mut handle = null_mut();
        let mut list = null_mut();
        let mut resources = [null_mut(); 5];
        assert_eq!(
            create(
                device.as_raw(),
                &mut handle,
                &mut list,
                resources.as_mut_ptr()
            ),
            0
        );
        let list = ID3D12GraphicsCommandList::from_raw(list);
        let resources = resources.map(|raw| ID3D12Resource::from_raw(raw));
        let mut control = Metrics::default();
        let hr = finish(handle, 1, &mut control);
        if hr != 0 {
            eprintln!("PERF sentinel control fence failed: {hr:#x}");
            std::process::exit(1);
        }
        assert_eq!(
            control.changed, 0,
            "readback must detect untouched sentinel"
        );
        eprintln!("PERF output sentinel control: changed=0");
        let path = std::env::var_os("FSR_SDK_TEST_DLL").unwrap();
        let mut runtime = Runtime::load(&path).unwrap();
        let raw_library = fsr_sdk_sys::loader::FfxLibrary::load(&path).unwrap();
        let options = UpscalerOptions::new(
            Dimensions::new(320, 180).unwrap(),
            Dimensions::new(640, 360).unwrap(),
        );
        let input = |reset| {
            UpscaleTestInputs::new(
                &list,
                UpscaleResources::builder()
                    .color(&resources[0])
                    .depth(&resources[1])
                    .motion_vectors(&resources[2])
                    .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                    .output(&resources[4])
                    .build()
                    .unwrap(),
                JitterOffsetPixels::new(0.0, 0.0).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                reset,
            )
        };
        let mut wrapper = runtime.upscaler_dx12(&device).create(&options).unwrap();
        eprintln!("PERF provider: {:?}", wrapper.provider().unwrap());
        let mut raw = RawContext::new(&raw_library, &device, &options);
        let frames = if diagnostics { 4 } else { 160 };
        for frame in 0..frames {
            let start = Instant::now();
            let desc = black_box(inspect_dispatch(&device, &options, input(frame == 0)).unwrap());
            let inspection = start.elapsed().as_secs_f64() * 1e9;
            // Swap order every frame. Each arm has its own continuous context;
            // shared external resources/list are reused only after a fence.
            for step in 0..2 {
                let arm = (step + frame) % 2;
                if frame == frames - 1 {
                    assert_eq!(prepare(handle), 0);
                }
                let dispatch = input(frame == 0);
                let start = Instant::now();
                if arm == 0 {
                    wrapper
                        .record(
                            dispatch.command_list,
                            &dispatch.resources,
                            &dispatch.parameters,
                        )
                        .unwrap_or_else(|error| {
                            eprintln!("PERF wrapper dispatch failed: {error:?}");
                            std::process::exit(1);
                        });
                } else {
                    raw.dispatch(&desc);
                }
                let elapsed = start.elapsed().as_secs_f64() * 1e9;
                let mut metrics = Metrics::default();
                let readback = u32::from(frame == frames - 1);
                let hr = finish(handle, readback, &mut metrics);
                if hr != 0 {
                    eprintln!("PERF submit/fence failed: {hr:#x}");
                    std::process::exit(1);
                }
                if readback == 1 {
                    assert_eq!(metrics.finite, 640 * 360);
                    assert_eq!(metrics.changed, 640 * 360);
                    assert_eq!(metrics.removed, 0);
                    if diagnostics {
                        assert_eq!(metrics.debug_errors, 0);
                    }
                }
                if frame >= 32 && !diagnostics {
                    let name = if arm == 0 { "wrapper" } else { "raw" };
                    eprintln!("PERF_SAMPLE native {frame} {name}={elapsed}");
                }
            }
            if frame >= 32 && !diagnostics {
                eprintln!("PERF_SAMPLE native {frame} inspect={inspection}");
            }
        }
        raw.destroy();
        wrapper.destroy().unwrap();
        // Native lifecycle is separate from warmed dispatch and resource setup.
        if !diagnostics {
            for sample in 0..39 {
                let start = Instant::now();
                let owner = runtime.upscaler_dx12(&device).create(&options).unwrap();
                let create = start.elapsed().as_secs_f64() * 1e9;
                let start = Instant::now();
                owner.destroy().unwrap();
                let destroy = start.elapsed().as_secs_f64() * 1e9;
                if sample >= 8 {
                    eprintln!(
                        "PERF_SAMPLE native_lifecycle {sample} create={create} destroy={destroy}"
                    );
                }
            }
        }
        drop(resources);
        drop(list);
        destroy(handle);
        let checkpoint = if diagnostics {
            c"gpu_check_device"
        } else {
            c"gpu_check_device_removed"
        };
        let check = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(checkpoint)
            .unwrap();
        assert_eq!(check(device.as_raw()), 0);
        eprintln!("PERF native: PASSED diagnostics={diagnostics}");
    }
}
