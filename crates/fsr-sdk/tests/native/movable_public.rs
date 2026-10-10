// SPDX-License-Identifier: MPL-2.0

//! Bounded public-path implementation verification using existing fenced equipment.
use super::*;
use crate::upscaler::{NativeSharpness, Sharpening};
use crate::{runtime::MovableRuntime, upscaler::MovableUpscaler};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, mpsc},
    thread,
};

struct Coordinated {
    context: Option<MovableUpscaler>,
    gate: Arc<Mutex<()>>,
}
impl Coordinated {
    fn record(
        &mut self,
        list: &ID3D12GraphicsCommandList,
        resources: &[ID3D12Resource],
        sharpening: Sharpening,
    ) {
        let _guard = self.gate.lock().unwrap_or_else(|p| p.into_inner());
        let mut frame = UpscaleDispatch::new(
            list,
            UpscaleResources::new(
                &resources[0],
                &resources[1],
                &resources[2],
                &resources[3],
                &resources[4],
            ),
            JitterOffsetPixels::new(0.0, 0.0).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0).unwrap(),
            true,
        );
        frame.set_sharpening(sharpening);
        // SAFETY: Existing harness supplies the supported fixed profile and states.
        // Main retains equipment/inputs/context through m6_finish's GPU fence;
        // all potentially shared calls and native cleanup use this common gate.
        if let Err(error) = unsafe { self.context.as_mut().unwrap().dispatch(frame) } {
            // Never submit a failed recording or unwind into unfinished native work.
            abort_dispatch(error, 0);
        }
    }
}
impl Drop for Coordinated {
    fn drop(&mut self) {
        let _guard = self.gate.lock().unwrap_or_else(|p| p.into_inner());
        eprintln!(
            "Public movable Drop: thread={:?} unwinding={} domain_held=true",
            thread::current().id(),
            thread::panicking()
        );
        // The caller fenced every submitted recording before permitting this Drop.
        drop(self.context.take());
    }
}

#[test]
#[ignore = "requires trusted SDK/GPU; native/run.ps1 -Mode movable -Native"]
fn public_siblings_dispatch_and_cleanup_after_thread_handoffs() {
    verify_public_handoffs(Sharpening::Disabled);
}

#[test]
#[ignore = "requires trusted SDK/GPU; native/run.ps1 -Mode movable -Native"]
fn public_native_sharpening_after_thread_handoffs() {
    verify_public_handoffs(Sharpening::Native(NativeSharpness::new(0.35).unwrap()));
}

fn verify_public_handoffs(sharpening: Sharpening) {
    assert!(std::env::var_os("FSR_SDK_TEST_GBV").is_none());
    let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").unwrap());
    let helper_path = PathBuf::from(std::env::var_os("FSR_SDK_TEST_M6_DLL").unwrap());
    let gate = Arc::new(Mutex::new(()));
    // SAFETY: Opt-in runner compiled this helper and authenticated the staged SDK.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Exact existing helper C signatures from gpu.cpp/device.cpp.
    let (device, create, finish, destroy, check) = unsafe {
        let _guard = gate.lock().unwrap();
        let debug = *helper.get::<EnableDebug>(c"m6_enable_debug").unwrap();
        assert_eq!(debug(), 0);
        let acquire = *helper.get::<CreateDevice>(c"probe_create_device").unwrap();
        let mut raw = null_mut();
        assert_eq!(acquire(&mut raw), 0);
        assert!(!raw.is_null());
        let device = ID3D12Device::from_raw(raw);
        let amd = *helper.get::<DeviceCheck>(c"m6_require_amd").unwrap();
        assert_eq!(amd(device.as_raw()), 0);
        (
            device,
            *helper.get::<CreateHarness>(c"m6_create").unwrap(),
            *helper.get::<Finish>(c"m6_finish").unwrap(),
            *helper.get::<Destroy>(c"m6_destroy").unwrap(),
            *helper.get::<DeviceCheck>(c"m6_check_device").unwrap(),
        )
    };
    let load_gate = gate.clone();
    let runtime = thread::spawn(move || {
        let _guard = load_gate.lock().unwrap();
        eprintln!(
            "Public movable acquisition: thread={:?}",
            thread::current().id()
        );
        // SAFETY: Caller assumes arbitrary-thread native compatibility for this
        // bounded signed deployment. One common gate governs every application
        // operation and cleanup; evidence does not prove the native assumption.
        unsafe { MovableRuntime::load_unchecked(loader) }.unwrap()
    })
    .join()
    .unwrap();
    let runtime_weak = Arc::downgrade(&runtime.state); // Observer adds no strong ownership.
    let create_gate = gate.clone();
    let create_device = {
        let _guard = gate.lock().unwrap();
        device.clone()
    };
    let (first, second) = thread::spawn(move || {
        let _guard = create_gate.lock().unwrap();
        eprintln!(
            "Public movable construction: thread={:?}",
            thread::current().id()
        );
        let options = UpscalerOptions::new(
            Dimensions::new(320, 180).unwrap(),
            Dimensions::new(640, 360).unwrap(),
        );
        let mut first = MovableUpscaler::new(&runtime, &create_device, options.clone()).unwrap();
        let mut second = MovableUpscaler::new(&runtime, &create_device, options).unwrap();
        // Test-only provider observation, not an additional production operation.
        log_provider_owner(&mut first.core.owner);
        log_provider_owner(&mut second.core.owner);
        drop(runtime);
        drop(create_device);
        (
            Coordinated {
                context: Some(first),
                gate: create_gate.clone(),
            },
            Coordinated {
                context: Some(second),
                gate: create_gate.clone(),
            },
        )
    })
    .join()
    .unwrap();

    // Hold the second sibling independently on a live worker throughout first use.
    let (inputs_tx, inputs_rx) =
        mpsc::sync_channel::<(ID3D12GraphicsCommandList, Vec<ID3D12Resource>)>(0);
    let (sibling_ready_tx, sibling_ready_rx) = mpsc::sync_channel(0);
    let (sibling_fenced_tx, sibling_fenced_rx) = mpsc::sync_channel(0);
    let sibling_sharpening = sharpening.clone();
    let sibling_worker = thread::spawn(move || {
        let mut second = second;
        let (list, resources) = inputs_rx.recv().unwrap();
        second.record(&list, &resources, sibling_sharpening);
        if sibling_ready_tx.send(()).is_err() || sibling_fenced_rx.recv().is_err() {
            std::process::exit(1); // Preserve owner/equipment if fence coordination fails.
        }
        (second, list, resources)
    });
    let mut equipment = Vec::new();
    let mut retained_inputs = Vec::new();
    for _ in 0..2 {
        let (harness, list, resources) = {
            let _guard = gate.lock().unwrap();
            let mut harness = null_mut();
            let mut list = null_mut();
            let mut resources = [null_mut(); 5];
            // SAFETY: Main owns the helper allocation until fenced cleanup;
            // each returned interface has its own transferred COM reference.
            unsafe {
                assert_eq!(
                    create(
                        device.as_raw(),
                        &mut harness,
                        &mut list,
                        resources.as_mut_ptr()
                    ),
                    0
                );
                assert!(
                    !harness.is_null() && !list.is_null() && resources.iter().all(|p| !p.is_null())
                );
                (
                    harness,
                    ID3D12GraphicsCommandList::from_raw(list),
                    resources
                        .into_iter()
                        .map(|p| ID3D12Resource::from_raw(p))
                        .collect::<Vec<_>>(),
                )
            }
        };
        equipment.push(harness);
        retained_inputs.push((list, resources));
    }
    let second_inputs = retained_inputs.pop().unwrap();
    let (list, resources) = retained_inputs.pop().unwrap();
    let (first_ready_tx, first_ready_rx) = mpsc::sync_channel(0);
    let (first_fenced_tx, first_fenced_rx) = mpsc::sync_channel(0);
    let first_sharpening = sharpening.clone();
    let first_worker = thread::spawn(move || {
        let mut first = first;
        first.record(&list, &resources, first_sharpening);
        if first_ready_tx.send(()).is_err() || first_fenced_rx.recv().is_err() {
            std::process::exit(1);
        }
        (first, list, resources)
    });
    if inputs_tx.send(second_inputs).is_err() {
        std::process::exit(1);
    }
    for (index, &harness) in equipment.iter().enumerate() {
        let ready = if index == 0 {
            &first_ready_rx
        } else {
            &sibling_ready_rx
        };
        if ready.recv().is_err() {
            std::process::exit(1);
        }
        let _guard = gate.lock().unwrap();
        let mut metrics = Metrics::default();
        // SAFETY: Existing finish submits once and waits on its bounded GPU fence.
        if unsafe { finish(harness, &mut metrics) } < 0 {
            std::process::exit(1);
        }
        eprintln!("Public movable fenced readback: {metrics:?}");
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(metrics.finite_pixels, 640 * 360);
        assert!(metrics.changed_pixels > 0);
        assert_eq!(metrics.debug_errors, 0);
        assert_eq!(metrics.device_removed, 0);
        let fenced = if index == 0 {
            &first_fenced_tx
        } else {
            &sibling_fenced_tx
        };
        if fenced.send(()).is_err() {
            std::process::exit(1);
        }
    }
    // Contexts and borrowed interfaces remained on their dispatch workers until
    // the GPU fences completed. No outstanding-work ownership transfer is tested.
    let (first, list, resources) = first_worker.join().unwrap();
    let (second, second_list, second_resources) = sibling_worker.join().unwrap();
    retained_inputs.push((list, resources));
    retained_inputs.push((second_list, second_resources));
    thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(move || {
            let _first = first;
            panic!("declared public movable unwind");
        }));
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"declared public movable unwind")
        );
    })
    .join()
    .unwrap();
    thread::spawn(move || {
        let (tx, rx) = mpsc::sync_channel(0);
        drop(rx);
        // Dropping the returned SendError destroys the final context, and with
        // it the final runtime reference, on this different destination thread.
        drop(tx.send(second).err().unwrap());
    })
    .join()
    .unwrap();
    let _guard = gate.lock().unwrap();
    assert!(
        runtime_weak.upgrade().is_none(),
        "terminal cleanup retained the runtime"
    );
    // SAFETY: The device/helper and every harness are still alive; all GPU work
    // is fenced and both contexts have completed cleanup before equipment release.
    unsafe {
        assert_eq!(check(device.as_raw()), 0);
        for harness in equipment {
            destroy(harness);
        }
        drop(retained_inputs);
        assert_eq!(check(device.as_raw()), 0);
    }
    drop(device);
    drop(helper);
    eprintln!(
        "Public movable: PASSED mode={sharpening:?} (two fenced frames; separate siblings; unwind and failed-handoff Drop)"
    );
}
