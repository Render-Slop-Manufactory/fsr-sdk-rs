// SPDX-License-Identifier: MPL-2.0

//! Matched native consumers; raw diagnostics retain a separate query context.
use super::super::dispatch::inspect_dispatch;
use super::performance::{RawContext, begin_allocations, end_allocations, require_optimized};
use super::*;
use std::{ffi::CStr, hint::black_box};

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
    errors: u64,
    removed: i32,
}

fn spec(
    desc: &ffxDispatchDescUpscale,
    render: Dimensions,
    output: Dimensions,
    dynamic: bool,
    hdr: bool,
) {
    let resources: Vec<_> = [
        &desc.color,
        &desc.depth,
        &desc.motionVectors,
        &desc.exposure,
        &desc.reactive,
        &desc.transparencyAndComposition,
        &desc.output,
    ]
    .into_iter()
    .map(|r| {
        // SAFETY: The selected texture2D descriptor uses the dimension union arms.
        unsafe {
            vec![
                u32::from(!r.resource.is_null()),
                r.description.r#type,
                r.description.format,
                r.description.width.width,
                r.description.height.height,
                r.description.depth.depth,
                r.description.mipCount,
                r.description.flags,
                r.description.usage,
                r.state,
            ]
        }
    })
    .collect();
    let shape = [
        desc.header.r#type,
        u64::from(!desc.header.pNext.is_null()),
        u64::from(!desc.commandList.is_null()),
        u64::from(desc.renderSize.width),
        u64::from(desc.renderSize.height),
        u64::from(desc.upscaleSize.width),
        u64::from(desc.upscaleSize.height),
        u64::from(desc.enableSharpening),
        u64::from(desc.reset),
        u64::from(desc.flags),
    ];
    let floats = [
        desc.jitterOffset.x,
        desc.jitterOffset.y,
        desc.motionVectorScale.x,
        desc.motionVectorScale.y,
        desc.sharpness,
        desc.frameTimeDelta,
        desc.preExposure,
        desc.cameraNear,
        desc.cameraFar,
        desc.cameraFovAngleVertical,
        desc.viewSpaceToMetersFactor,
    ]
    .map(f32::to_bits);
    let creation = [
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION,
        u64::from(FFX_UPSCALER_VERSION),
        FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12,
        (if dynamic { 64 } else { 0 }) | u64::from(hdr),
        u64::from(render.width()),
        u64::from(render.height()),
        u64::from(output.width()),
        u64::from(output.height()),
        0,
    ];
    eprintln!(
        "CPU_SPEC {{\"creation\":{creation:?},\"shape\":{shape:?},\"float_bits\":{floats:?},\"resources\":{resources:?}}}"
    );
}

fn checked(code: i32) {
    if code != 0 {
        eprintln!("CPU equipment failed: {code:#x}");
        // Never unwind through possibly in-flight/abandoned native recordings.
        std::process::exit(1);
    }
}

struct TraceGuard(bool);

impl Drop for TraceGuard {
    fn drop(&mut self) {
        crate::context::SUPPRESS_DESTROY_TRACE.with(|flag| flag.set(self.0));
    }
}

pub(super) fn run(gpu: bool) {
    require_optimized();
    let arm = std::env::var("FSR_SDK_CPU_ARM").unwrap();
    assert!(arm == "rust-wrapper" || arm == "rust-raw");
    let characterization = std::env::var("FSR_SDK_CHARACTERIZATION").ok();
    assert!(characterization.as_deref().is_none_or(|mode| matches!(
        mode,
        "timing" | "preparation" | "allocations" | "lifecycle" | "raw"
    )));
    assert!(!gpu || arm == "rust-wrapper" || characterization.as_deref() == Some("raw"));
    let _trace_guard = TraceGuard(
        crate::context::SUPPRESS_DESTROY_TRACE
            .with(|flag| flag.replace(characterization.is_some())),
    );
    let debug = std::env::var("FSR_SDK_PERFORMANCE_DEBUG").unwrap() == "1";
    let preinspect = std::env::var("FSR_SDK_CPU_PREINSPECT").unwrap() == "1";
    let active_mode = std::env::var("FSR_SDK_ACTIVE_TIMING").ok();
    assert!(
        active_mode
            .as_deref()
            .is_none_or(|mode| matches!(mode, "max" | "small" | "transition"))
    );
    assert!(active_mode.is_none() || gpu);
    let feature = std::env::var("FSR_SDK_FEATURE_TIMING").ok();
    assert!(
        feature
            .as_deref()
            .is_none_or(|value| matches!(value, "masks" | "hdr"))
    );
    assert!(feature.is_none() || active_mode.as_deref() == Some("max"));
    // SAFETY: Opt-in runner supplies trusted compiled equipment and pinned signed
    // SDK. All exports below match C ABI signatures; every frame is fenced.
    unsafe {
        let helper =
            libloading::Library::new(std::env::var_os("FSR_SDK_TEST_GPU_DLL").unwrap()).unwrap();
        let enable = helper
            .get::<extern "C" fn() -> i32>(c"gpu_enable_debug")
            .unwrap();
        if debug {
            checked(enable());
        }
        let create_device = helper
            .get::<extern "C" fn(*mut *mut std::ffi::c_void) -> i32>(c"probe_create_device")
            .unwrap();
        let mut pointer = null_mut();
        checked(create_device(&mut pointer));
        let device = ID3D12Device::from_raw(pointer);
        let amd = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(c"gpu_require_amd")
            .unwrap();
        checked(amd(device.as_raw()));
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
        let prepare = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(c"performance_prepare_readback")
            .unwrap();
        let copy = helper
            .get::<extern "C" fn(*mut std::ffi::c_void, *mut u8, u32) -> i32>(c"gpu_copy_payload")
            .unwrap();
        let configure = gpu.then(|| {
            *helper
                .get::<extern "C" fn(u32, u32) -> i32>(c"gpu_timing_configure")
                .unwrap()
        });
        let begin = gpu.then(|| {
            *helper
                .get::<extern "C" fn(*mut std::ffi::c_void, u32, f32, f32, u32) -> i32>(
                    c"gpu_timing_begin",
                )
                .unwrap()
        });
        let gpu_finish = gpu.then(|| {
            *helper
                .get::<extern "C" fn(*mut std::ffi::c_void, u32, *mut Metrics, *mut u64) -> i32>(
                    c"gpu_timing_finish",
                )
                .unwrap()
        });
        let active_begin = active_mode.as_ref().map(|_| {
            *helper
                .get::<extern "C" fn(*mut std::ffi::c_void, u32, u32, u32, f32, f32, u32) -> i32>(
                    c"active_timing_begin",
                )
                .unwrap()
        });
        let active_hashes = active_mode.as_ref().map(|_| {
            *helper
                .get::<extern "C" fn(*mut std::ffi::c_void, *mut u64) -> i32>(
                    c"active_timing_hashes",
                )
                .unwrap()
        });
        let destroy = helper
            .get::<extern "C" fn(*mut std::ffi::c_void)>(c"gpu_destroy")
            .unwrap();
        let clock = *helper
            .get::<extern "C" fn() -> i64>(c"performance_clock")
            .unwrap();
        let frequency = helper
            .get::<extern "C" fn() -> i64>(c"performance_frequency")
            .unwrap()();
        assert!(frequency > 0);
        let loader = std::env::var_os("FSR_SDK_TEST_DLL").unwrap();
        let load_start = clock();
        let mut runtime = Runtime::load(&loader).unwrap();
        let load_ticks = clock() - load_start;
        let output =
            Dimensions::new(if gpu { 1920 } else { 640 }, if gpu { 1080 } else { 360 }).unwrap();
        let query_start = clock();
        let render = if gpu {
            let size = runtime
                .upscaler_dx12(&device)
                .render_size_for_quality(output, crate::query::QualityMode::Quality)
                .unwrap();
            checked(configure.unwrap()(size.width(), size.height()));
            size
        } else {
            Dimensions::new(320, 180).unwrap()
        };
        let query_ticks = clock() - query_start;
        let mut handle = null_mut();
        let mut list = null_mut();
        let mut resources = [null_mut(); 5];
        checked(create(
            device.as_raw(),
            &mut handle,
            &mut list,
            resources.as_mut_ptr(),
        ));
        let list = ID3D12GraphicsCommandList::from_raw(list);
        let resources = resources.map(|raw| ID3D12Resource::from_raw(raw));
        if let Some(value) = feature.as_deref() {
            let configure = helper
                .get::<extern "C" fn(*mut std::ffi::c_void, u32) -> i32>(
                    c"feature_timing_configure",
                )
                .unwrap();
            checked(configure(handle, if value == "masks" { 1 } else { 2 }));
        }
        let masks: Vec<ID3D12Resource> = if feature.as_deref() == Some("masks") {
            let create = helper
                .get::<extern "C" fn(
                    *mut std::ffi::c_void,
                    u32,
                    u32,
                    u32,
                    f32,
                    f32,
                    *mut *mut std::ffi::c_void,
                ) -> i32>(c"gpu_create_masks")
                .unwrap();
            let mut raw = [null_mut(); 2];
            checked(create(
                handle,
                render.width(),
                render.height(),
                0,
                0.,
                0.,
                raw.as_mut_ptr(),
            ));
            raw.into_iter()
                .map(|value| ID3D12Resource::from_raw(value))
                .collect()
        } else {
            Vec::new()
        };
        let mut metrics = Metrics::default();
        checked(finish(handle, 1, &mut metrics));
        assert_eq!(metrics.changed, 0, "untouched sentinel control");
        let raw_library = (arm == "rust-raw" || characterization.is_none())
            .then(|| fsr_sdk_sys::loader::FfxLibrary::load(&loader).unwrap());
        let mut options = UpscalerOptions::new(render, output);
        options.set_dynamic_resolution(active_mode.is_some());
        options.set_high_dynamic_range(feature.as_deref() == Some("hdr"));
        let create_start = clock();
        let mut wrapper = (arm == "rust-wrapper" || gpu)
            .then(|| runtime.upscaler_dx12(&device).create(&options).unwrap());
        let create_ticks = clock() - create_start;
        let mut raw = (arm == "rust-raw")
            .then(|| RawContext::new(raw_library.as_ref().unwrap(), &device, &options));
        let mut provider = ffxQueryGetProviderVersion {
            header: ffxApiHeader {
                r#type: FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION,
                pNext: null_mut(),
            },
            versionId: 0,
            versionName: std::ptr::null(),
        };
        let code = if let Some(owner) = wrapper.as_mut() {
            owner.core.owner.query(&mut provider.header)
        } else {
            raw_library
                .as_ref()
                .unwrap()
                .ffxQuery(&mut raw.as_mut().unwrap().handle, &mut provider.header)
        };
        assert_eq!(code, 0);
        assert!(provider.versionId != 0 && !provider.versionName.is_null());
        eprintln!(
            "CPU_PROVIDER {} {}",
            provider.versionId,
            CStr::from_ptr(provider.versionName).to_str().unwrap()
        );
        let phase = gpu.then(|| {
            let owner = wrapper.as_mut().unwrap();
            assert_eq!(
                owner
                    .render_size_for_quality(output, crate::query::QualityMode::Quality)
                    .unwrap(),
                render
            );
            owner.jitter_phase_count(render, output).unwrap()
        });
        let small = active_mode.as_ref().map(|_| {
            let size = runtime
                .upscaler_dx12(&device)
                .render_size_for_quality(output, crate::query::QualityMode::Performance)
                .unwrap();
            assert!(size.width() < render.width() && size.height() < render.height());
            let owner = wrapper.as_mut().unwrap();
            assert_eq!(
                owner
                    .render_size_for_quality(output, crate::query::QualityMode::Performance)
                    .unwrap(),
                size
            );
            (size, owner.jitter_phase_count(size, output).unwrap())
        });
        let frames = if debug { 4 } else { 160 };
        let active_sizes: Vec<_> = (0..frames)
            .map(|frame| {
                let use_small = active_mode.as_deref() == Some("small")
                    || (active_mode.as_deref() == Some("transition")
                        && if debug {
                            (1..3).contains(&frame)
                        } else {
                            (64..112).contains(&frame)
                        });
                if use_small { small.unwrap().0 } else { render }
            })
            .collect();
        let phases: Vec<_> = active_sizes
            .iter()
            .map(|&size| {
                if size != render {
                    Some(small.unwrap().1)
                } else {
                    phase
                }
            })
            .collect();
        let jitters: Vec<_> = (0..frames)
            .map(|frame| {
                phases[frame].map_or(JitterOffsetPixels::new(0., 0.).unwrap(), |count| {
                    wrapper
                        .as_mut()
                        .unwrap()
                        .jitter_offset(frame as i32 % count.count(), count)
                        .unwrap()
                })
            })
            .collect();
        if let Some(count) = phase {
            eprintln!(
                "GPU_QUERY {} {} {}",
                render.width(),
                render.height(),
                count.count()
            );
        }
        let input = |reset, jitter| {
            let mut inputs = UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                .output(&resources[4])
                .build()
                .unwrap();
            if !masks.is_empty() {
                inputs.set_reactive_mask(Some(&masks[0]));
                inputs.set_transparency_and_composition_mask(Some(&masks[1]));
            }
            UpscaleTestInputs::new(
                &list,
                inputs,
                jitter,
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100., std::f32::consts::FRAC_PI_3, 1.).unwrap(),
                reset,
            )
        };
        // Prepare once: unlike P01, primary P03 does not inspect before every pair.
        let mut desc = inspect_dispatch(
            &device,
            &options,
            input(true, JitterOffsetPixels::new(0., 0.).unwrap()),
        )
        .unwrap();
        spec(
            &desc,
            render,
            output,
            active_mode.is_some(),
            options.high_dynamic_range(),
        );
        let mut empty = Vec::with_capacity(256);
        for _ in 0..256 {
            let start = clock();
            let end = clock();
            assert!(start >= 0 && end >= start);
            empty.push(end - start);
        }
        let mut ticks = Vec::with_capacity(frames);
        let mut gpu_ticks = Vec::with_capacity(frames);
        let mut checkpoints = Vec::new();
        let mut preparation_ticks = Vec::with_capacity(frames);
        let mut allocation_counts = Vec::with_capacity(frames);
        for (frame, &jitter) in jitters.iter().enumerate() {
            let readback = frame == frames - 1
                || (active_mode.is_some() && (debug || [31, 63, 64, 111, 112].contains(&frame)));
            desc.reset = frame == 0;
            if preinspect {
                black_box(inspect_dispatch(&device, &options, input(frame == 0, jitter)).unwrap());
            }
            if let Some(begin) = active_begin {
                checked(begin(
                    handle,
                    frame as u32,
                    active_sizes[frame].width(),
                    active_sizes[frame].height(),
                    jitter.x(),
                    jitter.y(),
                    u32::from(readback),
                ));
            } else if let Some(begin) = begin {
                checked(begin(
                    handle,
                    frame as u32,
                    jitter.x(),
                    jitter.y(),
                    u32::from(frame == frames - 1),
                ));
            } else if frame == frames - 1 {
                checked(prepare(handle));
            }
            let mut dispatch = input(frame == 0, jitter);
            if active_mode.is_some() {
                dispatch.set_active_render_size(Some(active_sizes[frame]));
            }
            if arm == "rust-raw" {
                // Prepared native reference: update changing scalar values only;
                // resource inspection is deliberately outside this boundary.
                desc.jitterOffset.x = jitter.x();
                desc.jitterOffset.y = jitter.y();
                desc.renderSize.width = active_sizes[frame].width();
                desc.renderSize.height = active_sizes[frame].height();
                desc.motionVectorScale.x = active_sizes[frame].width() as f32;
                desc.motionVectorScale.y = active_sizes[frame].height() as f32;
            }
            if characterization.as_deref() == Some("preparation") {
                let start = clock();
                black_box(inspect_dispatch(&device, &options, dispatch).unwrap());
                preparation_ticks.push(clock() - start);
                dispatch = input(frame == 0, jitter);
                dispatch.set_active_render_size(Some(active_sizes[frame]));
            }
            let start = clock();
            if characterization.as_deref() == Some("allocations") {
                begin_allocations();
            }
            if arm == "rust-wrapper" {
                let owner = wrapper.as_mut().unwrap();
                // Measure the public 0.3 recording boundary directly; input assembly
                // above remains outside the timed interval, as in the P06 protocol.
                if let Err(error) = owner.record(
                    dispatch.command_list,
                    &dispatch.resources,
                    &dispatch.parameters,
                ) {
                    eprintln!("CPU wrapper failure: {error:?}");
                    std::process::exit(1);
                }
            } else {
                raw.as_mut().unwrap().dispatch(&desc);
            }
            if characterization.as_deref() == Some("allocations") {
                allocation_counts.push(end_allocations());
            }
            let end = clock();
            // Do not assert/unwind before the successful recording is submitted.
            if let Some(finish) = gpu_finish {
                let mut values = [0u64; 5];
                checked(finish(
                    handle,
                    u32::from(readback),
                    &mut metrics,
                    values.as_mut_ptr(),
                ));
                gpu_ticks.push(values);
            } else {
                checked(finish(handle, u32::from(frame == frames - 1), &mut metrics));
            }
            if let Some(hashes) = active_hashes.filter(|_| readback) {
                assert_eq!(
                    (metrics.finite, metrics.changed, metrics.removed),
                    (
                        output.width() * output.height(),
                        output.width() * output.height(),
                        0
                    )
                );
                if debug {
                    assert_eq!(metrics.errors, 0);
                }
                let mut values = [0u64; 2];
                checked(hashes(handle, values.as_mut_ptr()));
                checkpoints.push((
                    frame,
                    metrics.finite,
                    metrics.changed,
                    metrics.errors,
                    values,
                ));
            }
            assert!(start >= 0 && end >= start);
            ticks.push(end - start);
        }
        assert_eq!(
            (
                metrics.width,
                metrics.height,
                metrics.finite,
                metrics.changed,
                metrics.removed
            ),
            (
                output.width(),
                output.height(),
                output.width() * output.height(),
                output.width() * output.height(),
                0
            )
        );
        if debug {
            assert_eq!(metrics.errors, 0);
        }
        let mut payload = vec![0u8; (output.width() * output.height() * 8) as usize];
        checked(copy(handle, payload.as_mut_ptr(), payload.len() as u32));
        std::fs::write(std::env::var_os("FSR_SDK_CPU_OUTPUT").unwrap(), payload).unwrap();
        if let Some(owner) = raw {
            owner.destroy();
        }
        let destroy_start = clock();
        if let Some(owner) = wrapper {
            owner.destroy().unwrap();
        }
        let destroy_ticks = clock() - destroy_start;
        if characterization.as_deref() == Some("lifecycle") {
            eprintln!("CHAR_LIFECYCLE load 0 {load_ticks}");
            eprintln!("CHAR_LIFECYCLE provider_query 0 {query_ticks}");
            eprintln!("CHAR_LIFECYCLE first_create 0 {create_ticks}");
            eprintln!("CHAR_LIFECYCLE used_destroy 0 {destroy_ticks}");
            for sample in 0..8 {
                let start = clock();
                let owner = runtime.upscaler_dx12(&device).create(&options).unwrap();
                let created = clock();
                owner.destroy().unwrap();
                let destroyed = clock();
                eprintln!("CHAR_LIFECYCLE warm_create {sample} {}", created - start);
                eprintln!(
                    "CHAR_LIFECYCLE unused_destroy {sample} {}",
                    destroyed - created
                );
            }
        }
        for (frame, value) in preparation_ticks.iter().enumerate() {
            eprintln!("CHAR_PREPARATION {frame} {value}");
        }
        for (frame, (events, bytes)) in allocation_counts.iter().enumerate() {
            eprintln!(
                "CHAR_ALLOCATIONS {frame} {} {} {} {} {bytes}",
                events[0], events[1], events[2], events[3]
            );
        }
        if let Some(mode) = characterization {
            eprintln!("CHAR_MODE {mode}");
        }
        drop(resources);
        drop(masks);
        drop(list);
        destroy(handle);
        let check = helper
            .get::<extern "C" fn(*mut std::ffi::c_void) -> i32>(if debug {
                c"gpu_check_device"
            } else {
                c"gpu_check_device_removed"
            })
            .unwrap();
        checked(check(device.as_raw()));
        eprintln!("CPU_CLOCK {frequency}");
        for (index, value) in empty.iter().enumerate() {
            eprintln!("CPU_EMPTY {index} {value}");
        }
        for (index, value) in ticks.iter().enumerate() {
            eprintln!("CPU_FRAME {index} {value}");
        }
        for (index, values) in gpu_ticks.iter().enumerate() {
            eprintln!(
                "GPU_FRAME {index} {} {} {} {} {}",
                values[0], values[1], values[2], values[3], values[4]
            );
            eprintln!(
                "GPU_JITTER {index} {} {}",
                jitters[index].x().to_bits(),
                jitters[index].y().to_bits()
            );
        }
        if let Some(mode) = active_mode {
            let (small_size, small_phase) = small.unwrap();
            eprintln!(
                "ACTIVE_QUERY {mode} {} {} {}",
                small_size.width(),
                small_size.height(),
                small_phase.count()
            );
            for (frame, size) in active_sizes.iter().enumerate() {
                eprintln!(
                    "ACTIVE_FRAME {frame} {} {} {} {} {} {}",
                    size.width(),
                    size.height(),
                    phases[frame].unwrap().count(),
                    u32::from(frame == 0),
                    (size.width() as f32).to_bits(),
                    (size.height() as f32).to_bits()
                );
            }
            for (frame, finite, written, errors, hashes) in checkpoints {
                eprintln!(
                    "ACTIVE_VALID {frame} {finite} {written} {errors} {} {}",
                    hashes[0], hashes[1]
                );
            }
        }
        eprintln!(
            "CPU_VALID sentinel=0 finite={} written={} removed=0 debug={debug} preinspect={preinspect}",
            metrics.finite, metrics.changed
        );
        eprintln!("CPU_COMPARISON PASSED {arm}");
    }
}
