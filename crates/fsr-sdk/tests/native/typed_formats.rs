// SPDX-License-Identifier: MPL-2.0

//! Public typed resources and an independent pinned-header C++ reference.
use super::{followup::Lab, variable_output::Owner, *};
use crate::upscaler::{
    DepthConvention, MotionVectorScale, NativeSharpness, ReactiveMaskOptions,
    ReactiveMaskParameters, ReactiveMaskResources, Sharpening,
};
use std::os::windows::ffi::OsStrExt;

type Create = unsafe extern "C" fn(
    *mut c_void,
    *const u32,
    u32,
    u32,
    *const *const u8,
    *const u32,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> i32;
type Finish = unsafe extern "C" fn(*mut c_void, u32, *mut u64, *mut i32) -> i32;
type Copy = unsafe extern "C" fn(*mut c_void, u32, *mut u8, u32) -> i32;
type ReferenceCreate =
    unsafe extern "C" fn(*const u16, *mut c_void, u64, u32, *mut *mut c_void) -> u32;
type ReferenceRecord = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    *const *mut c_void,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
) -> u32;
type ReferenceGenerate =
    unsafe extern "C" fn(*mut c_void, *mut c_void, *const *mut c_void, u32, u32) -> u32;
type ReferenceDestroy = unsafe extern "C" fn(*mut c_void) -> u32;
fn setting(name: &str) -> String {
    std::env::var(format!("FSR_SDK_TEST_FORMAT_{name}")).unwrap()
}
fn number(name: &str) -> u32 {
    setting(name).parse().unwrap()
}

pub(super) fn run() {
    let lab = Lab::acquire();
    // SAFETY: The coordinator builds this exact helper from the paired sources.
    let (
        create,
        finish,
        copy,
        consume,
        discard,
        release,
        compare,
        ref_create,
        ref_record,
        ref_generate,
        ref_destroy,
    ) = unsafe {
        (
            *lab.library.get::<Create>(c"gpu_formats_create").unwrap(),
            *lab.library.get::<Finish>(c"gpu_formats_finish").unwrap(),
            *lab.library.get::<Copy>(c"gpu_formats_copy").unwrap(),
            *lab.library
                .get::<Destroy>(c"gpu_formats_consume_mask")
                .unwrap(),
            *lab.library.get::<Destroy>(c"gpu_formats_discard").unwrap(),
            *lab.library.get::<Destroy>(c"gpu_formats_destroy").unwrap(),
            *lab.library
                .get::<CompareResource>(c"gpu_compare_resource")
                .unwrap(),
            *lab.library
                .get::<ReferenceCreate>(c"formats_reference_create")
                .unwrap(),
            *lab.library
                .get::<ReferenceRecord>(c"formats_reference_record")
                .unwrap(),
            *lab.library
                .get::<ReferenceGenerate>(c"formats_reference_generate")
                .unwrap(),
            *lab.library
                .get::<ReferenceDestroy>(c"formats_reference_destroy")
                .unwrap(),
        )
    };
    let name = setting("PROVIDER");
    let family = setting("FAMILY");
    let mode = setting("MODE");
    let directory = PathBuf::from(setting("DIR"));
    let formats: Vec<u32> = setting("FORMATS")
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(formats.len(), 7);
    let generate = number("GENERATE") != 0;
    let dynamic = number("DYNAMIC") != 0;
    let reversed = number("REVERSED") != 0;
    let hdr = number("HDR") != 0;
    let sharpen = number("SHARPEN") != 0;
    let control = number("CONTROL") != 0;
    let supplied_mask: u32 = std::env::var("FSR_SDK_TEST_FORMAT_MASK")
        .ok()
        .map(|v| v.parse().unwrap())
        .unwrap_or(0);
    let depth_buffer: u32 = std::env::var("FSR_SDK_TEST_FORMAT_DEPTH_BUFFER")
        .ok()
        .map(|v| v.parse().unwrap())
        .unwrap_or(1);
    // SAFETY: Pinned deployment and explicit provider selection; each child's
    // native calls and cleanup are serial, and every submitted frame is fenced.
    let mut runtime = unsafe { Runtime::load(&lab.loader) }.unwrap();
    let providers = unsafe { runtime.upscaler_dx12(&lab.device) }
        .providers()
        .unwrap();
    let provider = providers.iter().find(|p| p.name() == name).unwrap();
    let id = provider.id();
    eprintln!("Format provider: {provider:?}");
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );

    options.set_dynamic_resolution(dynamic || generate);
    options.set_high_dynamic_range(hdr);
    if reversed {
        options.set_depth_convention(DepthConvention::ReversedFinite);
    }
    let flags = u32::from(hdr) | (u32::from(reversed) << 3) | (u32::from(dynamic || generate) << 6);
    let mut reference = null_mut();
    let mut context = if mode == "reference" {
        let wide: Vec<u16> = lab
            .loader
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let raw_id = if name == "4.1.1" {
            17700776140664213505u64
        } else {
            assert_eq!(name, "3.1.5");
            17700776140660019205u64
        };
        // SAFETY: Pinned C++ consumer retains its own loader and context, validates
        // live provider identity and constructs descriptors independently of Rust.
        assert_eq!(
            unsafe {
                ref_create(
                    wide.as_ptr(),
                    lab.device.as_raw(),
                    raw_id,
                    flags,
                    &mut reference,
                )
            },
            0
        );
        None
    } else {
        let owner = match family.as_str() {
            "local" => {
                let mut c = unsafe { runtime.upscaler_dx12(&lab.device) }
                    .with_provider(crate::query::ProviderSelection::Explicit(id))
                    .create(&options)
                    .unwrap();
                assert_eq!(c.provider().unwrap().id(), id);
                Owner::Local(c)
            }
            #[cfg(feature = "movable")]
            "movable" => {
                // SAFETY: Same pinned serial lifetime contract; this batch makes no handoff.
                let mut r = unsafe { crate::runtime::MovableRuntime::load(&lab.loader) }.unwrap();
                let mut c = unsafe { r.upscaler_dx12(&lab.device) }
                    .with_provider(crate::query::ProviderSelection::Explicit(id))
                    .create(&options)
                    .unwrap();
                assert_eq!(c.provider().unwrap().id(), id);
                Owner::Movable(c)
            }
            _ => panic!("unknown format owner"),
        };
        Some(owner)
    };
    for phase in 0..3u32 {
        let active = if dynamic && phase == 1 {
            (512, 288)
        } else {
            (640, 360)
        };
        let render = if (generate
            || dynamic && matches!(formats[1], 19 | 20 | 39 | 40 | 44 | 45 | 55))
            && phase == 1
        {
            (160, 90)
        } else {
            (320, 180)
        };
        let input: Vec<Vec<u8>> = (0..7)
            .map(|r| std::fs::read(directory.join(format!("input-{phase}-{r}.bin"))).unwrap())
            .collect();
        let pointers: Vec<_> = input.iter().map(|v| v.as_ptr()).collect();
        let counts: Vec<_> = input.iter().map(|v| v.len() as u32).collect();
        let mut handle = null_mut();
        let mut raw_list = null_mut();
        let mut raw = [null_mut(); 7];
        // SAFETY: Initialized input arrays remain live during this copying call;
        // helper transfers COM refs and owns uploads, queue, allocator and fence.
        assert_eq!(
            unsafe {
                create(
                    lab.device.as_raw(),
                    formats.as_ptr(),
                    number("UAV"),
                    depth_buffer,
                    pointers.as_ptr(),
                    counts.as_ptr(),
                    &mut handle,
                    &mut raw_list,
                    raw.as_mut_ptr(),
                )
            },
            0
        );
        let list = unsafe { ID3D12GraphicsCommandList::from_raw(raw_list) };
        let resources: Vec<_> = raw
            .iter()
            .map(|p| unsafe { ID3D12Resource::from_raw(*p) })
            .collect();
        let mut recorded = Ok(());
        if !control && generate {
            let bindings = ReactiveMaskResources::builder()
                .color_opaque_only(&resources[5])
                .color_pre_upscale(&resources[0])
                .output(&resources[6])
                .build()
                .unwrap();
            let parameters = ReactiveMaskParameters::new(
                Dimensions::new(render.0, render.1).unwrap(),
                ReactiveMaskOptions::new(1.0, 0.2, 0.9).unwrap(),
            );
            recorded = if let Some(owner) = context.as_mut() {
                // SAFETY: Prepared maximum allocations and matched signals; all
                // preceding submissions are fenced, this list is never resubmitted.
                match owner {
                    Owner::Local(c) => c
                        .validate_reactive_mask_inputs(&list, &bindings, &parameters)
                        .and_then(|()| unsafe {
                            c.record_reactive_mask(&list, &bindings, &parameters)
                        }),
                    #[cfg(feature = "movable")]
                    Owner::Movable(c) => c
                        .validate_reactive_mask_inputs(&list, &bindings, &parameters)
                        .and_then(|()| unsafe {
                            c.record_reactive_mask(&list, &bindings, &parameters)
                        }),
                }
            } else {
                // SAFETY: Same resources/states; C++ builds its own native descriptor.
                let code = unsafe {
                    ref_generate(reference, list.as_raw(), raw.as_ptr(), render.0, render.1)
                };
                if code == 0 {
                    Ok(())
                } else {
                    Err(Error::Native(crate::error::NativeError {
                        operation: crate::error::Operation::GenerateReactiveMask,
                        code,
                    }))
                }
            };
            if recorded.is_ok() {
                unsafe { consume(handle) };
            }
        }
        if recorded.is_ok() && !control {
            if supplied_mask != 0 {
                unsafe { consume(handle) };
            }
            let mut bundle = UpscaleResources::builder()
                .color(&resources[0])
                .depth(&resources[1])
                .motion_vectors(&resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(&resources[3]))
                .output(&resources[4])
                .build()
                .unwrap();
            if generate || supplied_mask == 1 {
                bundle.set_reactive_mask(Some(&resources[6]));
            } else if supplied_mask == 2 {
                bundle.set_transparency_and_composition_mask(Some(&resources[6]));
            }
            let mut frame = UpscaleTestInputs::new(
                &list,
                bundle,
                JitterOffsetPixels::new(0.0, 0.0).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
                phase == 0,
            );
            frame.set_motion_vector_scale(MotionVectorScale::Pixels);
            if sharpen {
                frame.set_sharpening(Sharpening::Native(NativeSharpness::new(0.5).unwrap()));
            }
            if dynamic {
                frame.set_active_output_size(Some(Dimensions::new(active.0, active.1).unwrap()));
            }
            if generate || dynamic && matches!(formats[1], 19 | 20 | 39 | 40 | 44 | 45 | 55) {
                frame.set_active_render_size(Some(Dimensions::new(render.0, render.1).unwrap()));
            }
            recorded = if let Some(owner) = context.as_mut() {
                match owner.inspect(&frame) {
                    Ok(desc) => {
                        for (role, native) in [
                            &desc.color,
                            &desc.depth,
                            &desc.motionVectors,
                            &desc.exposure,
                            &desc.output,
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            // SAFETY: The C++ helper compares actual descriptions to the
                            // wrapper's temporary descriptor; it retains neither pointer.
                            if unsafe { compare(raw[role], native, if role == 4 { 2 } else { 4 }) }
                                != 1
                            {
                                // Avoid unwinding into native cleanup with a pending
                                // generator recording when descriptor parity fails.
                                unsafe { discard(handle) };
                                eprintln!(
                                    "typed resource descriptor parity failed for role {role}"
                                );
                                std::process::exit(1);
                            }
                        }
                        owner.public_record(frame)
                    }
                    Err(error) => Err(error),
                }
            } else {
                // SAFETY: Independent C++ descriptors, same fenced workload inputs.
                let code = unsafe {
                    ref_record(
                        reference,
                        list.as_raw(),
                        raw.as_ptr(),
                        render.0,
                        render.1,
                        active.0,
                        active.1,
                        phase,
                        u32::from(sharpen),
                        if supplied_mask == 2 {
                            2
                        } else {
                            u32::from(generate || supplied_mask == 1)
                        },
                    )
                };
                if code == 0 {
                    Ok(())
                } else {
                    Err(Error::Native(crate::error::NativeError {
                        operation: crate::error::Operation::Upscale,
                        code,
                    }))
                }
            };
        }
        if let Err(error) = recorded {
            // SAFETY: Discard the never-submitted list before either context is
            // destroyed; earlier frames have already completed their fences.
            unsafe { discard(handle) };
            drop(list);
            if let Some(owner) = context.take() {
                owner.destroy().unwrap();
            }
            if !reference.is_null() {
                assert_eq!(unsafe { ref_destroy(reference) }, 0);
            }
            drop(resources);
            unsafe { release(handle) };
            panic!("typed format recording failed: {error}");
        }
        let mut errors = 0;
        let mut removed = 0;
        // SAFETY: Submit exactly once, copy all resources and wait for completion.
        let completion = unsafe {
            finish(
                handle,
                u32::from((generate || supplied_mask != 0) && !control),
                &mut errors,
                &mut removed,
            )
        };
        if completion != 0 {
            // Completion is unproven: isolated process exit avoids native
            // destruction of a context or resources with possible in-flight work.
            eprintln!("typed format completion failed: {completion:#x}");
            std::process::exit(1);
        }
        assert_eq!((errors, removed), (0, 0));
        for (role, count) in counts.iter().enumerate() {
            let mut bytes = vec![0u8; *count as usize];
            assert_eq!(
                unsafe { copy(handle, role as u32, bytes.as_mut_ptr(), *count) },
                0
            );
            if role != 4 && !(role == 6 && generate && !control) {
                assert_eq!(bytes, input[role], "input mutation in role {role}");
            }
            std::fs::write(directory.join(format!("{mode}-{phase}-{role}.bin")), bytes).unwrap();
        }
        if depth_buffer != 0 && matches!(formats[1], 19 | 20 | 39 | 40 | 44 | 45 | 55) {
            let depth_copy =
                unsafe { *lab.library.get::<Copy>(c"gpu_formats_depth_copy").unwrap() };
            let mut sampled = vec![0u8; 320 * 180 * 4];
            // SAFETY: The completed helper owns this shader-readback buffer.
            assert_eq!(
                unsafe { depth_copy(handle, 0, sampled.as_mut_ptr(), sampled.len() as u32) },
                0
            );
            assert_eq!(
                sampled, input[1],
                "depth SRV sampled the wrong values/plane"
            );
            std::fs::write(
                directory.join(format!("{mode}-{phase}-depth-sampled.bin")),
                sampled,
            )
            .unwrap();
            if matches!(formats[1], 19 | 20 | 44 | 45) {
                let mut stencil = vec![0u8; 320 * 180];
                assert_eq!(
                    unsafe { depth_copy(handle, 1, stencil.as_mut_ptr(), stencil.len() as u32) },
                    0
                );
                assert!(stencil.iter().all(|v| *v == 0x5a), "stencil plane changed");
                std::fs::write(
                    directory.join(format!("{mode}-{phase}-stencil.bin")),
                    stencil,
                )
                .unwrap();
            }
        }
        eprintln!(
            "FORMAT_FRAME {{\"phase\":{phase},\"debug_errors\":{errors},\"removed\":{removed}}}"
        );
        drop(list);
        drop(resources);
        unsafe { release(handle) };
    }
    if let Some(owner) = context {
        owner.destroy().unwrap();
    }
    if !reference.is_null() {
        assert_eq!(unsafe { ref_destroy(reference) }, 0);
    }
    eprintln!("FORMAT_RESULT {{\"frames\":3,\"cleanup_ok\":true}}");
}
