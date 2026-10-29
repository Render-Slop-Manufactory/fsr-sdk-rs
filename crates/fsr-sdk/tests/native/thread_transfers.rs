// SPDX-License-Identifier: MPL-2.0

//! Preregistered transfer cleanup protocol; test equipment, not a public API.
use super::super::UpscaleCreateState;
use super::*;
use crate::{context::NativeContextOwner, runtime::RuntimeState};
use std::{
    cell::Cell,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, Weak, mpsc},
    thread::{self, JoinHandle, ThreadId},
};

type Owner = NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device, Arc<RuntimeState>>;
type SetFrame = unsafe extern "C" fn(*mut c_void, u32, f32, f32, u32) -> i32;
type CopyPayload = unsafe extern "C" fn(*mut c_void, *mut u8, u32) -> i32;
type FenceValue = unsafe extern "C" fn(*mut c_void) -> u64;
const PAYLOAD_BYTES: usize = 640 * 360 * 8;
const PANIC: &str = "Transfer cleanup declared Rust unwind";
const JITTER: [(f32, f32); 8] = [
    (0.25, -0.25),
    (-0.25, 0.25),
    (0.125, -0.375),
    (-0.375, 0.125),
    (0.375, -0.125),
    (-0.125, 0.375),
    (0.0625, -0.0625),
    (-0.0625, 0.0625),
];

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

fn loaded(name: &str) -> bool {
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // SAFETY: NUL-terminated name; inspect only whether the instantaneous result
    // is null. This API adds no ref. Never dereference/free/call through its result.
    unsafe { !GetModuleHandleW(wide.as_ptr()).is_null() }
}

fn phase(message: &str) {
    use std::io::Write as _;
    let _ = writeln!(
        std::io::stderr().lock(),
        "Transfer cleanup {message} thread={:?} name={:?}",
        thread::current().id(),
        thread::current().name()
    );
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[derive(Default)]
struct Domain {
    gate: Mutex<()>,
    holder: Mutex<Option<ThreadId>>,
}

struct Gate<'a> {
    domain: &'a Domain,
    owned: Option<MutexGuard<'a, ()>>,
}

impl Domain {
    fn enter(&self) -> Gate<'_> {
        let current = thread::current().id();
        // Release the bookkeeping lock before waiting for the native gate.
        // A destructor nested inside its owning guard must not relock that gate.
        let already_held = *lock(&self.holder) == Some(current);
        let owned = if already_held {
            None
        } else {
            let guard = lock(&self.gate);
            *lock(&self.holder) = Some(current);
            Some(guard)
        };
        Gate {
            domain: self,
            owned,
        }
    }

    fn held_here(&self) -> bool {
        *lock(&self.holder) == Some(thread::current().id())
    }
}

impl Drop for Gate<'_> {
    fn drop(&mut self) {
        if self.owned.is_some() {
            *lock(&self.domain.holder) = None;
        }
        // The MutexGuard field unlocks afterward and records unwind poisoning.
    }
}

struct Frame {
    harness: *mut c_void,
    _list: ID3D12GraphicsCommandList,
    _resources: Vec<ID3D12Resource>,
    destroy: Destroy,
}

// SAFETY: Exact test equipment only. Harness is a stable owned C++ allocation;
// every list/resource is an owned windows 0.62.2 COM ref. Only fenced frames enter
// Equipment.frames; main retains Equipment through all worker joins/cleanup and
// drops frames under the common domain before device/helper. No borrowed pointers,
// callbacks, custom allocators or Rc owners. Mutex access never overlaps AMD calls
// or other equipment operations; no generic/platform Send claim follows.
unsafe impl Send for Frame {}

impl Drop for Frame {
    fn drop(&mut self) {
        // SAFETY: Only fenced frames are retained; their owning Equipment keeps
        // helper code/device alive. Its explicit frame release holds the domain.
        unsafe { (self.destroy)(self.harness) };
    }
}

// Frame fields are released before the original caller device, helper last.
// Shared immutable function pointers/device ownership do not authorize calls:
// all users enter Domain, including resource inspection and automatic cleanup.
struct Equipment {
    frames: Mutex<Vec<Frame>>,
    device: ID3D12Device,
    domain: Arc<Domain>,
    weak: Mutex<Weak<RuntimeState>>,
    options: UpscalerOptions,
    loader: PathBuf,
    output: PathBuf,
    create: CreateHarness,
    set: SetFrame,
    finish: Finish,
    copy: CopyPayload,
    fence: FenceValue,
    destroy: Destroy,
    check: DeviceCheck,
    _helper: libloading::Library,
}

impl Equipment {
    fn acquire() -> Arc<Self> {
        assert!(std::env::var_os("FSR_SDK_TEST_GBV").is_none());
        assert!(std::env::var_os("FSR_SDK_TEST_GPU_CONTROL").is_none());
        assert!(!loaded("amd_fidelityfx_loader_dx12.dll"));
        let domain = Arc::new(Domain::default());
        let _guard = domain.enter();
        let helper = std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap();
        // SAFETY: Runner-compiled trusted helper; all symbols match gpu.cpp.
        unsafe {
            let library = libloading::Library::new(helper).unwrap();
            let debug = *library.get::<EnableDebug>(b"gpu_enable_debug\0").unwrap();
            assert_eq!(debug(), 0);
            let device_create = *library
                .get::<CreateDevice>(b"probe_create_device\0")
                .unwrap();
            let mut raw = null_mut();
            assert_eq!(device_create(&mut raw), 0);
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            let amd = *library.get::<DeviceCheck>(b"gpu_require_amd\0").unwrap();
            assert_eq!(amd(device.as_raw()), 0);
            Arc::new(Self {
                frames: Mutex::new(Vec::new()),
                device,
                domain: domain.clone(),
                weak: Mutex::new(Weak::new()),
                options: UpscalerOptions::new(
                    Dimensions::new(320, 180).unwrap(),
                    Dimensions::new(640, 360).unwrap(),
                ),
                loader: std::env::var_os("FSR_SDK_TEST_DLL").unwrap().into(),
                output: std::env::var_os("FSR_SDK_TEST_TRANSFER_OUTPUT")
                    .unwrap()
                    .into(),
                create: *library.get::<CreateHarness>(b"gpu_create\0").unwrap(),
                set: *library.get::<SetFrame>(b"semantic_set_frame\0").unwrap(),
                finish: *library.get::<Finish>(b"gpu_finish\0").unwrap(),
                copy: *library.get::<CopyPayload>(b"gpu_copy_payload\0").unwrap(),
                fence: *library.get::<FenceValue>(b"gpu_fence_value\0").unwrap(),
                destroy: *library.get::<Destroy>(b"gpu_destroy\0").unwrap(),
                check: *library.get::<DeviceCheck>(b"gpu_check_device\0").unwrap(),
                _helper: library,
            })
        }
    }

    fn snapshot(&self, checkpoint: &str) {
        snapshot(checkpoint, Some(lock(&self.weak).strong_count()));
    }

    fn check(&self) {
        let _guard = self.domain.enter();
        // SAFETY: Caller device and helper are live; all submitted work fenced.
        assert_eq!(unsafe { (self.check)(self.device.as_raw()) }, 0);
    }

    fn release_frames(&self) {
        let _guard = self.domain.enter();
        let frames = std::mem::take(&mut *lock(&self.frames));
        phase(&format!("release frames count={}", frames.len()));
        drop(frames);
        self.check();
        self.snapshot("4a-frames-gone-device-live");
    }
}

fn snapshot(checkpoint: &str, strong: Option<usize>) {
    phase(&format!(
        "residency checkpoint={checkpoint} runtime_strong={strong:?} loader={} provider={} helper={}",
        loaded("amd_fidelityfx_loader_dx12.dll"),
        loaded("amd_fidelityfx_upscaler_dx12.dll"),
        loaded("gpu_helper.dll")
    ));
}

struct RuntimeBundle {
    runtime: Option<Arc<RuntimeState>>,
    equipment: Arc<Equipment>,
    _not_sync: PhantomData<Cell<()>>,
}

impl RuntimeBundle {
    fn load(equipment: Arc<Equipment>) -> Self {
        let _guard = equipment.domain.enter();
        phase("load runtime (no context yet)");
        // SAFETY: Trusted signed pinned deployment; the isolated matrix's common
        // gate covers all calls/cleanup. Transfer compatibility is the hypothesis.
        let library = unsafe { fsr_sdk_sys::loader::FfxLibrary::load(&equipment.loader) }.unwrap();
        let runtime = Arc::new(RuntimeState::new(library));
        *lock(&equipment.weak) = Arc::downgrade(&runtime);
        Self {
            runtime: Some(runtime),
            equipment: equipment.clone(),
            _not_sync: PhantomData,
        }
    }

    fn create(&self, id: usize) -> Bundle {
        Bundle::create(
            id,
            self.runtime.as_ref().unwrap().clone(),
            self.equipment.clone(),
        )
    }
}

impl Drop for RuntimeBundle {
    fn drop(&mut self) {
        let _guard = self.equipment.domain.enter();
        phase("external runtime release domain_held=true");
        drop(self.runtime.take());
    }
}

struct Bundle {
    owner: Option<Owner>,
    equipment: Arc<Equipment>,
    id: usize,
    eligibility: crate::context_policy::Eligibility,
    previous: ThreadId,
    last_frame: Option<u32>,
}

// SAFETY: Concrete opt-in bundle, never the generic owner/public local types.
// Arc retains RuntimeState/FfxLibrary; state is the real owned pinned creation
// chain; owner holds its own ID3D12Device ref. Equipment owns every frame's stable
// C++ allocation and COM refs, keeps helper loaded, and is independently retained
// by main until all contexts/runtime are gone and workers are idle. No Rc exists
// beyond the generic owner's PhantomData; no borrowed native dependencies,
// callbacks/allocators or exposed raw handles. Owning channel requests/replies
// establish handoff; every native/equipment call and Drop enters the common gate.
// Every submitted frame is fenced before return/transfer/panic. Dispatch or
// uncertain finish exits the isolated child without unwinding/ordinary cleanup.
// Destroy failure retains the real D005 bundle. Native affinity for these exact
// signed inputs is the experiment's hypothesis, not a general safety guarantee.
// Sync remains excluded by Owner; all cleanup paths retain device/helper code.
unsafe impl Send for Bundle {}

const _: fn() = || {
    fn send<T: Send>() {}
    send::<Bundle>();
    send::<RuntimeBundle>();
    trait Ambiguous<A> {
        fn check() {}
    }
    impl<T: ?Sized> Ambiguous<()> for T {}
    struct IfSync;
    impl<T: ?Sized + Sync> Ambiguous<IfSync> for T {}
    let _ = <Bundle as Ambiguous<_>>::check;
    let _ = <RuntimeBundle as Ambiguous<_>>::check;
};

impl Bundle {
    fn create(id: usize, runtime: Arc<RuntimeState>, equipment: Arc<Equipment>) -> Self {
        let _guard = equipment.domain.enter();
        phase(&format!("create context={id} domain_held=true"));
        let device = equipment.device.clone();
        let (state, root) = UpscaleCreateState::new(
            device.as_raw().cast(),
            &equipment.options,
            Default::default(),
            None,
        );
        // SAFETY: Exact pinned owned descriptors/device/runtime, zero flags and
        // callbacks; the common gate is held and no submitted work exists.
        let result = unsafe { Owner::create(state, device, runtime, root) };
        let mut owner = match result {
            Ok(owner) => owner,
            Err(error) => {
                eprintln!("Transfer cleanup create failure {error:?}");
                std::process::exit(1);
            }
        };
        phase(&format!(
            "create context={id} native_code={FFX_API_RETURN_OK:#010x}"
        ));
        log_provider_owner(&mut owner);
        Self {
            owner: Some(owner),
            equipment: equipment.clone(),
            id,
            eligibility: crate::context_policy::Eligibility::Usable,
            previous: thread::current().id(),
            last_frame: None,
        }
    }

    fn handoff(&mut self) {
        phase(&format!(
            "handoff context={} from={:?} completed_frame={:?} exclusive=true",
            self.id, self.previous, self.last_frame
        ));
        self.previous = thread::current().id();
    }

    fn frame(&mut self, frame: u32, selected: bool) {
        let equipment = &self.equipment;
        let _guard = equipment.domain.enter();
        assert_eq!(self.last_frame.map(|f| f + 1).unwrap_or(0), frame);
        let (jx, jy) = JITTER[frame as usize % JITTER.len()];
        phase(&format!(
            "dispatch context={} frame={frame} translation={} jitter=({jx},{jy}) reset={} domain_held=true",
            self.id,
            frame * 2,
            frame == 0
        ));
        let mut harness = null_mut();
        let mut raw_list = null_mut();
        let mut raw_resources = [null_mut(); 5];
        // SAFETY: Fresh unsubmitted recording; helper transfers owned refs.
        unsafe {
            assert_eq!(
                (equipment.create)(
                    equipment.device.as_raw(),
                    &mut harness,
                    &mut raw_list,
                    raw_resources.as_mut_ptr()
                ),
                0
            );
            assert!(!harness.is_null() && !raw_list.is_null());
            assert_eq!((equipment.set)(harness, frame, jx, jy, 0), 0);
        }
        // SAFETY: Each returned pointer carries one owned COM ref.
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<ID3D12Resource> = raw_resources
            .into_iter()
            .map(|raw| {
                assert!(!raw.is_null());
                // SAFETY: Adopt the helper's owned resource reference.
                unsafe { ID3D12Resource::from_raw(raw) }
            })
            .collect();
        let owner = self.owner.as_mut().unwrap();
        let result = crate::context_policy::dispatch(
            crate::error::Operation::Upscale,
            &mut self.eligibility,
            owner,
            |owner| {
                super::super::dispatch::inspect_dispatch(
                    owner.device(),
                    &equipment.options,
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
                        JitterOffsetPixels::new(jx, jy).unwrap(),
                        FrameTimeMillis::new(16.667).unwrap(),
                        CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0)
                            .unwrap(),
                        frame == 0,
                    ),
                )
            },
            |owner, desc| {
                // SAFETY: Real shared inspection succeeded; all dependencies remain
                // owned through submission/fence; domain excludes sibling calls.
                unsafe { owner.dispatch(&desc.header) }
            },
        );
        if let Err(error) = result {
            abort_dispatch(error, frame);
        }
        phase(&format!(
            "dispatch OK context={} frame={frame} native_code=0",
            self.id
        ));
        let mut metrics = Metrics::default();
        // SAFETY: Close/submit once, wait bounded fence, map output; retained refs.
        let hr = unsafe { (equipment.finish)(harness, &mut metrics) };
        if hr != 0 {
            eprintln!(
                "Transfer cleanup uncertain finish HRESULT={hr:#010x}; containment, no unwind"
            );
            std::process::exit(1);
        }
        // SAFETY: Still-live harness/fence/helper, paired C export.
        let completed = unsafe { (equipment.fence)(harness) };
        if completed == u64::MAX || completed < 1 {
            eprintln!(
                "Transfer cleanup uncertain completed fence={completed}; containment, no unwind"
            );
            std::process::exit(1);
        }
        phase(&format!(
            "fenced context={} frame={frame} completed={completed} metrics={metrics:?}",
            self.id
        ));
        lock(&equipment.frames).push(Frame {
            harness,
            _list: list,
            _resources: resources,
            destroy: equipment.destroy,
        });
        self.last_frame = Some(frame);
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(metrics.finite_pixels, 230_400);
        assert!(metrics.changed_pixels > 115_200);
        assert_eq!(metrics.debug_errors, 0);
        assert_eq!(metrics.device_removed, 0);
        if selected {
            let mut payload = vec![0u8; PAYLOAD_BYTES];
            // SAFETY: GPU completed, retained readback and exact writable byte count.
            assert_eq!(
                unsafe { (equipment.copy)(harness, payload.as_mut_ptr(), PAYLOAD_BYTES as u32) },
                0
            );
            let file = format!("context{}-frame{frame}.rgba16f", self.id);
            std::fs::write(equipment.output.join(&file), payload).unwrap();
            phase(&format!(
                "payload file={file} bytes={PAYLOAD_BYTES} padding=excluded"
            ));
        }
        equipment.check();
    }

    fn destroy(&mut self) {
        let _guard = self.equipment.domain.enter();
        phase(&format!(
            "explicit destroy context={} domain_held=true",
            self.id
        ));
        if let Err(error) = self.owner.take().unwrap().destroy() {
            eprintln!("Transfer cleanup terminal destroy failure {error:?}");
            std::process::exit(1);
        }
    }
}

impl Drop for Bundle {
    fn drop(&mut self) {
        if self.owner.is_some() {
            let _guard = self.equipment.domain.enter();
            phase(&format!(
                "automatic Drop begin context={} domain_held={} unwinding={}",
                self.id,
                self.equipment.domain.held_here(),
                thread::panicking()
            ));
            drop(self.owner.take()); // Real NativeContextOwner::Drop, never copied.
            phase(&format!("automatic Drop end context={}", self.id));
        }
    }
}

#[derive(Default)]
struct Slots {
    runtime: Option<RuntimeBundle>,
    contexts: [Option<Bundle>; 2],
}
type Job = Box<dyn FnOnce(&mut Slots) + Send>;

struct Actor {
    sender: mpsc::SyncSender<Job>,
    join: JoinHandle<()>,
}

impl Actor {
    fn spawn(name: &str) -> Self {
        let (sender, receiver) = mpsc::sync_channel::<Job>(0);
        let join = thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                phase("persistent worker start");
                let mut slots = Slots::default();
                for job in receiver {
                    job(&mut slots);
                }
                assert!(slots.runtime.is_none() && slots.contexts.iter().all(Option::is_none));
                phase("persistent worker normal exit (no owners)");
            })
            .unwrap();
        Self { sender, join }
    }

    fn run<T: Send + 'static>(&self, action: impl FnOnce(&mut Slots) -> T + Send + 'static) -> T {
        let (sender, receiver) = mpsc::sync_channel(0);
        self.sender
            .send(Box::new(move |slots| {
                sender.send(action(slots)).unwrap();
            }))
            .unwrap();
        receiver.recv().unwrap()
    }

    fn frame(&self, id: usize, frame: u32, selected: bool) {
        self.run(move |slots| slots.contexts[id].as_mut().unwrap().frame(frame, selected));
    }

    fn destroy(&self, id: usize, explicit: bool) {
        self.run(move |slots| {
            let mut owned = slots.contexts[id].take().unwrap();
            if explicit {
                owned.destroy();
            }
            drop(owned);
        });
    }

    fn unwind(&self, id: usize) {
        self.run(move |slots| {
            let owned = slots.contexts[id].take().unwrap();
            assert!(owned.last_frame.is_some());
            let equipment = owned.equipment.clone();
            let domain = equipment.domain.clone();
            let result = catch_unwind(AssertUnwindSafe(move || {
                // Reverse local destruction: owned context first, guard second.
                // Both locals are inside the caught scope; equipment survives it.
                let _guard = domain.enter();
                let _owned = owned;
                phase("intentional panic outside FFI, all GPU work completed");
                std::panic::panic_any(PANIC);
            }));
            let error = result.expect_err("declared panic must unwind");
            assert_eq!(error.downcast_ref::<&str>(), Some(&PANIC));
            assert!(equipment.domain.gate.is_poisoned());
            phase("expected panic caught, owner unwound, domain poisoned and recoverable");
            equipment.check();
        });
    }
}

fn transfer(actors: &[Actor], from: usize, to: usize, id: usize) {
    let bundle = actors[from].run(move |slots| slots.contexts[id].take().unwrap());
    actors[to].run(move |slots| {
        assert!(slots.contexts[id].is_none());
        let mut bundle = bundle;
        bundle.handoff();
        slots.contexts[id] = Some(bundle);
    });
}

fn initialize(actors: &[Actor], equipment: &Arc<Equipment>, count: usize, early_drop: bool) {
    let equipment = equipment.clone();
    actors[0].run(move |slots| {
        slots.runtime = Some(RuntimeBundle::load(equipment.clone()));
        for id in 0..count {
            slots.contexts[id] = Some(slots.runtime.as_ref().unwrap().create(id));
        }
        equipment.snapshot("1-contexts-live");
        if early_drop {
            drop(slots.runtime.take());
            assert_eq!(lock(&equipment.weak).strong_count(), count);
            equipment.snapshot("2-external-runtime-gone");
        }
    });
}

fn run(case: u32, moving: bool) {
    phase(&format!(
        "case={case} transfer={moving} preregistered protocol 2026-10-08"
    ));
    let equipment = Equipment::acquire();
    let actors: Vec<_> = [
        "transfer-worker-1",
        "transfer-worker-2",
        "transfer-worker-3",
        "transfer-worker-4",
    ]
    .into_iter()
    .map(Actor::spawn)
    .collect();
    let on = |index| if moving { index } else { 0 };
    match case {
        1 => {
            let eq = equipment.clone();
            actors[0].run(move |slots| slots.runtime = Some(RuntimeBundle::load(eq)));
            let runtime = actors[0].run(|slots| slots.runtime.take().unwrap());
            actors[on(1)].run(move |slots| {
                slots.contexts[0] = Some(runtime.create(0));
                runtime
                    .equipment
                    .snapshot("1-context-live-after-runtime-transfer");
                slots.runtime = Some(runtime);
            });
            transfer(&actors, on(1), on(2), 0);
            let runtime = actors[on(1)].run(|slots| slots.runtime.take().unwrap());
            actors[on(2)].run(move |slots| slots.runtime = Some(runtime));
            actors[on(2)].frame(0, 0, true);
            transfer(&actors, on(2), on(3), 0);
            let runtime = actors[on(2)].run(|slots| slots.runtime.take().unwrap());
            actors[on(3)].run(move |slots| slots.runtime = Some(runtime));
            actors[on(3)].destroy(0, true);
            actors[on(3)].run(|slots| drop(slots.runtime.take()));
        }
        2 => {
            initialize(&actors, &equipment, 1, true);
            for frame in 0..64 {
                let here = on(frame as usize % 4);
                actors[here].frame(0, frame, [0, 1, 7, 15, 31, 63].contains(&frame));
                if frame < 63 {
                    transfer(&actors, here, on((frame as usize + 1) % 4), 0);
                }
            }
            actors[on(3)].destroy(0, true);
        }
        3 => {
            initialize(&actors, &equipment, 2, true);
            transfer(&actors, 0, on(1), 0);
            transfer(&actors, 0, on(2), 1);
            for frame in 0..8 {
                actors[on(if frame <= 3 { 1 } else { 3 })].frame(
                    0,
                    frame,
                    frame == 0 || frame == 7,
                );
                actors[on(2)].frame(1, frame, frame == 0 || frame == 7);
                if frame == 3 {
                    transfer(&actors, on(1), on(3), 0);
                }
            }
            actors[on(3)].destroy(0, true);
            assert_eq!(lock(&equipment.weak).strong_count(), 1);
            equipment.snapshot("3-first-sibling-destroyed");
            actors[on(2)].frame(1, 8, true);
            transfer(&actors, on(2), on(1), 1);
            actors[on(1)].destroy(1, false);
        }
        4 | 5 => {
            let siblings = case == 5;
            initialize(&actors, &equipment, if siblings { 2 } else { 1 }, true);
            transfer(&actors, 0, on(1), 0);
            actors[on(1)].frame(0, 0, true);
            if siblings {
                transfer(&actors, 0, on(2), 1);
                actors[on(2)].frame(1, 0, true);
            }
            transfer(&actors, on(1), on(3), 0);
            actors[on(3)].unwind(0);
            if siblings {
                assert_eq!(lock(&equipment.weak).strong_count(), 1);
                equipment.snapshot("3-first-sibling-unwound");
                actors[on(2)].frame(1, 1, true);
                transfer(&actors, on(2), on(1), 1);
                actors[on(1)].destroy(1, false);
            }
        }
        _ => unreachable!(),
    }
    assert_eq!(
        lock(&equipment.weak).strong_count(),
        0,
        "retained native failure bundle"
    );
    assert!(!loaded("amd_fidelityfx_loader_dx12.dll"));
    equipment.snapshot("4-final-context-runtime-gone");
    equipment.check();
    equipment.release_frames();
    for actor in &actors {
        actor.run(|slots| {
            assert!(slots.runtime.is_none() && slots.contexts.iter().all(Option::is_none))
        });
    }
    assert_eq!(
        Arc::strong_count(&equipment),
        1,
        "unreleased equipment reference"
    );
    let domain = equipment.domain.clone();
    {
        let _guard = domain.enter();
        drop(equipment); // frames empty, last caller device then helper.
    }
    snapshot("5-all-application-equipment-device-helper-gone", Some(0));
    assert!(!loaded("gpu_helper.dll"));
    for Actor { sender, join } in actors {
        drop(sender);
        join.join().unwrap();
    }
    snapshot("6-before-normal-child-exit", Some(0));
    phase(&format!("case={case} transfer={moving}: PASSED"));
}

macro_rules! scenario {
    ($name:ident, $case:literal, $moving:literal) => {
        #[test]
        #[ignore = "requires trusted SDK/GPU; tools/verification/run.py run experiment-transfer-cleanup --native"]
        fn $name() {
            run($case, $moving);
        }
    };
}
scenario!(
    same_thread_runtime_transfer_before_context_creation,
    1,
    false
);
scenario!(
    runtime_transfer_before_context_creation_on_another_thread,
    1,
    true
);
scenario!(
    same_thread_continuous_history_across_ownership_handoffs,
    2,
    false
);
scenario!(continuous_history_across_worker_thread_transfers, 2, true);
scenario!(
    same_thread_independent_siblings_share_cleanup_coordination,
    3,
    false
);
scenario!(
    independent_siblings_share_cleanup_coordination_across_threads,
    3,
    true
);
scenario!(
    same_thread_automatic_context_cleanup_during_unwind,
    4,
    false
);
scenario!(transferred_context_automatic_cleanup_during_unwind, 4, true);
scenario!(
    same_thread_sibling_cleanup_during_unwind_preserves_survivor,
    5,
    false
);
scenario!(
    transferred_sibling_cleanup_during_unwind_preserves_survivor,
    5,
    true
);
