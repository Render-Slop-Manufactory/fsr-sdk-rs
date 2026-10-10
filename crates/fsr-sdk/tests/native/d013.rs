// SPDX-License-Identifier: MPL-2.0

//! Bounded, opt-in D013 AMD experiment. This is not a public movable API.
use super::super::UpscaleCreateState;
use super::*;
use crate::{context::NativeContextOwner, runtime::RuntimeState};
use std::{
    pin::Pin,
    sync::{Arc, Weak},
    thread,
};

type ArcOwner = NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device, Arc<RuntimeState>>;

struct Frame {
    harness: *mut c_void,
    _list: ID3D12GraphicsCommandList,
    _resources: Vec<ID3D12Resource>,
    destroy: Destroy,
}

impl Drop for Frame {
    fn drop(&mut self) {
        // SAFETY: Only successfully fenced frames enter the retained vector.
        // Helper COM refs coexist with Rust refs; helper stays loaded until last.
        unsafe { (self.destroy)(self.harness) };
    }
}

// Transfer the entire family, including test equipment, exclusively. No member
// is handed to an independent worker. Field order also protects unwind cleanup:
// contexts before frames/device and the helper library last.
struct SerialFamily {
    contexts: Vec<Option<ArcOwner>>,
    runtime: Option<Arc<RuntimeState>>,
    frames: Vec<Frame>,
    device: ID3D12Device,
    options: UpscalerOptions,
    weak: Weak<RuntimeState>,
    loader_path: PathBuf,
    provider_path: PathBuf,
    create_harness: CreateHarness,
    set_frame: SetTemporalFrame,
    finish: Finish,
    destroy_harness: Destroy,
    check_device: DeviceCheck,
    _helper: libloading::Library,
}

// SAFETY: Restricted to this opt-in experiment's exact bundle. There is no Rc
// ownership (the core's Rc PhantomData is only a marker), callback, allocator
// hook or borrowed storage. Creation chains remain in their original pinned
// boxes; the only raw harness pointers name stable C++ heap allocations. Every
// list/resource/device owns COM refs (windows 0.62.2 declares these Send+Sync).
// Arc retains the actual FfxLibrary through terminal cleanup, including D005
// exceptional retention. The helper library outlives all function pointers and
// harness cleanup. Nothing is shared with concurrent native users in each
// isolated child. Joined workers establish happens-before; every submitted
// frame is fully fenced before returning the bundle. Dispatch/fence failures
// exit the child without ordinary cleanup of uncertain recordings/work.
// AMD destination-thread compatibility is precisely the hypothesis under test,
// authorized only for the pinned DLL/provider/device configuration. This impl
// supplies no production or generic-owner threading guarantee. Sync remains
// excluded by ArcOwner's local marker; negative compilation assertion below.
unsafe impl Send for SerialFamily {}

const _: fn() = || {
    fn send<T: Send>() {}
    send::<SerialFamily>();
    trait Ambiguous<A> {
        fn check() {}
    }
    impl<T: ?Sized> Ambiguous<()> for T {}
    struct IfSync;
    impl<T: ?Sized + Sync> Ambiguous<IfSync> for T {}
    let _ = <SerialFamily as Ambiguous<_>>::check;
};

fn loaded(path: &std::path::Path) -> bool {
    // The observation acquires and immediately releases its temporary ref.
    libloading::os::windows::Library::open_already_loaded(path).is_ok()
}

fn phase(label: &str) {
    eprintln!(
        "D013 phase={label} thread={:?} name={:?}",
        thread::current().id(),
        thread::current().name()
    );
}

impl SerialFamily {
    fn create(siblings: bool) -> Self {
        phase("load/create");
        assert!(
            std::env::var_os("FSR_SDK_TEST_GBV").is_none(),
            "GBV excluded"
        );
        assert!(std::env::var_os("FSR_SDK_TEST_M6_CONTROL").is_none());
        let loader_path = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").unwrap());
        let provider_path = loader_path.with_file_name("amd_fidelityfx_upscaler_dx12.dll");
        assert!(!loaded(&loader_path), "loader already loaded in child");
        let helper_path = PathBuf::from(std::env::var_os("FSR_SDK_TEST_M6_DLL").unwrap());
        // SAFETY: Trusted, locally compiled helper; signatures match gpu.cpp.
        let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
        // SAFETY: Exact paired C exports, library retained in this bundle.
        let (
            enable_debug,
            create_device,
            require_amd,
            check_device,
            create_harness,
            set_frame,
            finish,
            destroy_harness,
        ) = unsafe {
            (
                *helper.get::<EnableDebug>(b"m6_enable_debug\0").unwrap(),
                *helper
                    .get::<CreateDevice>(b"probe_create_device\0")
                    .unwrap(),
                *helper.get::<DeviceCheck>(b"m6_require_amd\0").unwrap(),
                *helper.get::<DeviceCheck>(b"m6_check_device\0").unwrap(),
                *helper.get::<CreateHarness>(b"m6_create\0").unwrap(),
                *helper
                    .get::<SetTemporalFrame>(b"m6_set_temporal_frame\0")
                    .unwrap(),
                *helper.get::<Finish>(b"m6_finish\0").unwrap(),
                *helper.get::<Destroy>(b"m6_destroy\0").unwrap(),
            )
        };
        // SAFETY: Enable ordinary debugging before device creation; GBV off.
        assert_eq!(unsafe { enable_debug() }, 0, "debug layer unavailable");
        let mut raw_device = null_mut();
        // SAFETY: Writable output receives one owned live device COM ref.
        assert_eq!(unsafe { create_device(&mut raw_device) }, 0);
        assert!(!raw_device.is_null());
        // SAFETY: Adopt that transferred owned reference.
        let device = unsafe { ID3D12Device::from_raw(raw_device) };
        // SAFETY: Inspect the actual device's adapter.
        assert_eq!(unsafe { require_amd(device.as_raw()) }, 0);
        // SAFETY: Runner stages trusted signed v2.3.0 DLLs. All operations in
        // this child are serialized, with the conditional transfer hypothesis.
        let library = unsafe { fsr_sdk_sys::loader::FfxLibrary::load(&loader_path) }.unwrap();
        let runtime = Arc::new(RuntimeState { library });
        let weak = Arc::downgrade(&runtime);
        let options = UpscalerOptions::new(
            Dimensions::new(320, 180).unwrap(),
            Dimensions::new(640, 360).unwrap(),
        );
        let mut contexts = Vec::new();
        for context in 0..if siblings { 2 } else { 1 } {
            let owned_device = device.clone();
            let (state, root) = UpscaleCreateState::new(owned_device.as_raw().cast(), &options);
            // SAFETY: Reuse the actual pinned creation chain and private owner;
            // retain device and Arc runtime, zero flags, no callbacks, no work.
            let mut owner =
                unsafe { ArcOwner::create(state, owned_device, runtime.clone(), root) }.unwrap();
            eprintln!("D013 create context={context}: native_code={FFX_API_RETURN_OK:#010x}");
            log_provider_owner(&mut owner);
            contexts.push(Some(owner));
        }
        Self {
            contexts,
            runtime: Some(runtime),
            frames: Vec::new(),
            device,
            options,
            weak,
            loader_path,
            provider_path,
            create_harness,
            set_frame,
            finish,
            destroy_harness,
            check_device,
            _helper: helper,
        }
    }

    fn drop_runtime(&mut self) {
        phase("drop runtime before contexts");
        drop(self.runtime.take().unwrap());
        self.retained();
    }

    fn retained(&self) {
        assert!(self.weak.upgrade().is_some());
        assert!(loaded(&self.loader_path), "premature loader unload");
        assert!(
            loaded(&self.provider_path),
            "provider no longer observed loaded"
        );
    }

    fn frame(&mut self, context: usize, frame: u32) {
        phase("dispatch/submit/fence/readback");
        self.retained();
        let (jx, jy) = [(0.25, -0.25), (-0.25, 0.25), (0.125, -0.125)][frame as usize];
        eprintln!(
            "D013 context={context} frame={frame} translation={frame}px jitter=({jx},{jy}) reset={}",
            frame == 0
        );
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: Fresh recording equipment; outputs transfer extra COM refs.
        assert_eq!(
            unsafe {
                (self.create_harness)(
                    self.device.as_raw(),
                    &mut harness,
                    &mut raw_list,
                    raw_resources.as_mut_ptr(),
                )
            },
            0
        );
        assert!(!harness.is_null() && !raw_list.is_null());
        // SAFETY: Rewrite only fresh, unsubmitted upload storage.
        assert_eq!(unsafe { (self.set_frame)(harness, frame, jx, jy) }, 0);
        // SAFETY: Adopt the helper's transferred list reference.
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<ID3D12Resource> = raw_resources
            .into_iter()
            .map(|p| {
                assert!(!p.is_null());
                // SAFETY: Each pointer carries one owned reference from the helper.
                unsafe { ID3D12Resource::from_raw(p) }
            })
            .collect();
        let owner = self.contexts[context].as_mut().unwrap();
        let desc = super::super::dispatch::inspect_dispatch(
            owner.device(),
            &self.options,
            UpscaleDispatch {
                command_list: &list,
                color: &resources[0],
                depth: &resources[1],
                motion_vectors: &resources[2],
                exposure: &resources[3],
                output: &resources[4],
                jitter: JitterOffsetPixels::new(jx, jy).unwrap(),
                frame_time: FrameTimeMillis::new(16.667).unwrap(),
                camera: CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0)
                    .unwrap(),
                reset: frame == 0,
            },
        )
        .unwrap();
        // SAFETY: Shared inspection succeeded; helper recorded boundary states.
        // All pointers, COM refs, context/library and allocator survive fence.
        let code = unsafe { owner.dispatch(&desc.header) };
        if code != FFX_API_RETURN_OK {
            eprintln!("D013 dispatch failed: code={code:#010x}; no submission/cleanup");
            std::process::exit(1);
        }
        eprintln!("D013 dispatch context={context} frame={frame}: native_code={code:#010x}");
        let mut metrics = Metrics::default();
        // SAFETY: Finish closes/submits once, fences fully (60s bound), maps output.
        let hr = unsafe { (self.finish)(harness, &mut metrics) };
        if hr != 0 {
            eprintln!("D013 finish failed: HRESULT={hr:#010x}; completion uncertain; no cleanup");
            std::process::exit(1);
        }
        // Establish RAII only once GPU completion is certain; keep every frame
        // through context cleanup, including its native heap allocation.
        self.frames.push(Frame {
            harness,
            _list: list,
            _resources: resources,
            destroy: self.destroy_harness,
        });
        eprintln!("D013 fenced readback context={context} frame={frame}: {metrics:?}");
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(metrics.finite_pixels, 230_400);
        assert!(metrics.changed_pixels > 115_200);
        assert_eq!(metrics.debug_errors, 0);
        assert_eq!(metrics.device_removed, 0);
        self.check();
    }

    fn check(&self) {
        // SAFETY: Retained caller device and helper; GPU already fenced.
        assert_eq!(unsafe { (self.check_device)(self.device.as_raw()) }, 0);
    }

    fn destroy_first(&mut self) {
        phase("explicit destroy first sibling");
        self.contexts[0].take().unwrap().destroy().unwrap();
        eprintln!("D013 explicit destroy context=0: native_code={FFX_API_RETURN_OK:#010x}");
        self.retained();
        self.check();
    }

    fn cleanup(mut self, implicit: bool) {
        phase("final context cleanup/runtime release");
        if implicit {
            drop(self.contexts[1].take().unwrap());
            eprintln!("D013 final sibling Drop completed (native result not exposed)");
        } else {
            self.contexts[0].take().unwrap().destroy().unwrap();
            eprintln!("D013 explicit destroy context=0: native_code={FFX_API_RETURN_OK:#010x}");
            drop(self.runtime.take().unwrap());
        }
        assert!(self.weak.upgrade().is_none(), "final Arc still retained");
        let loader_live = loaded(&self.loader_path);
        let provider_live = loaded(&self.provider_path);
        eprintln!(
            "D013 after final release: weak_live=false loader_loaded={loader_live} provider_loaded={provider_live}"
        );
        assert!(
            !loader_live,
            "loader still observed loaded after final release"
        );
        self.check();
        for frame in self.frames.drain(..) {
            // All retained references survived final context cleanup/release.
            drop(frame);
        }
        phase("after frame resources cleanup");
        self.check();
    }
}

fn worker<T: Send + 'static>(name: &str, action: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new()
        .name(name.into())
        .spawn(action)
        .unwrap()
        .join()
        .unwrap()
}

fn run(scenario: u32, transfer: bool) {
    eprintln!("D013 scenario={scenario} transfer={transfer}; ordinary debug enabled, GBV off");
    if !transfer {
        worker("d013-control", move || {
            let mut family = SerialFamily::create(scenario == 3);
            if scenario == 3 {
                family.drop_runtime();
            }
            family.frame(0, 0);
            if scenario == 2 {
                family.frame(0, 1);
                family.frame(0, 2);
            }
            if scenario == 3 {
                family.destroy_first();
                family.frame(1, 0);
            }
            family.cleanup(scenario == 3);
        });
    } else {
        let family = worker("d013-A", move || {
            let mut family = SerialFamily::create(scenario == 3);
            if scenario == 3 {
                family.drop_runtime();
            }
            if scenario == 2 {
                family.frame(0, 0);
            }
            family
        });
        let family = worker("d013-B", move || {
            let mut family = family;
            family.frame(0, if scenario == 2 { 1 } else { 0 });
            if scenario == 3 {
                family.destroy_first();
            }
            family
        });
        worker("d013-C", move || {
            let mut family = family;
            if scenario == 2 {
                family.frame(0, 2);
            }
            if scenario == 3 {
                family.frame(1, 0);
            }
            family.cleanup(scenario == 3);
        });
    }
    eprintln!("D013 scenario={scenario} transfer={transfer}: PASSED");
}

macro_rules! scenario {
    ($name:ident, $number:literal, $transfer:literal) => {
        #[test]
        #[ignore = "requires trusted SDK DLLs and hardware DX12; use native/run.ps1 -Mode d013 -Native"]
        fn $name() { run($number, $transfer); }
    };
}
scenario!(control_abc, 1, false);
scenario!(transfer_abc, 1, true);
scenario!(control_three_frames, 2, false);
scenario!(transfer_three_frames, 2, true);
scenario!(control_siblings, 3, false);
scenario!(transfer_siblings, 3, true);
