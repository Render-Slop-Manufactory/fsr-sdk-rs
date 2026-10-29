// SPDX-License-Identifier: MPL-2.0

//! Opt-in startup-query timing; no cache, speed gate or GPU dispatch.
use super::{super::*, support::*};
use crate::{query::QualityMode, runtime::Runtime};
use std::{hint::black_box, path::PathBuf, time::Instant};

fn setup_queries(
    mut setup: crate::upscaler::dx12::UpscalerSetup<'_, '_>,
    options: &UpscalerOptions,
) -> Result<(), Error> {
    let display = options.max_upscale_size();
    let mode = QualityMode::Quality;
    let ratio = setup.ratio(mode)?;
    let render = setup.render_size_for_quality(display, mode)?;
    let phase = setup.jitter_phase_count(render, display)?;
    let jitter = setup.jitter_offset(0, phase)?;
    let requirements = setup.resource_requirements()?;
    let memory = setup.estimate_memory(options)?;
    black_box((ratio, render, phase, jitter, requirements, memory));
    Ok(())
}

#[test]
#[ignore = "requires staged trusted AMD runtime and DX12 helper; use experiment-provider-setup"]
fn native_provider_setup_cost() {
    let helper_path = std::env::var_os("FSR_SDK_TEST_DEVICE_DLL").expect("native device helper");
    let loader =
        PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("trusted native loader"));
    // SAFETY: The opt-in runner stages the paired helper and trusted signed deployment.
    let helper = unsafe { libloading::Library::new(helper_path) }.unwrap();
    // SAFETY: Paired helper transfers one owned device into valid writable output.
    let create = unsafe {
        helper.get::<unsafe extern "C" fn(*mut *mut ID3D12Device) -> i32>(c"probe_create_device")
    }
    .unwrap();
    let mut raw = null_mut();
    // SAFETY: Writable output, helper/device remain alive through every query.
    assert!(unsafe { create(&mut raw) } >= 0 && !raw.is_null());
    // SAFETY: Adopt the transferred COM reference.
    let device = unsafe { ID3D12Device::from_raw(raw.cast()) };
    // SAFETY: Explicit opt-in establishes actual-provider query/acquisition trust;
    // this child serializes every native operation and never submits GPU work.
    let mut runtime = unsafe { Runtime::load(&loader) }.unwrap();
    let default = sizes();
    let mut context = unsafe { runtime.upscaler_dx12(&device) }
        .create(&default)
        .unwrap();
    let provider = context.provider().unwrap();
    eprintln!("Provider setup identity: {provider:?}");
    let selected = crate::query::ProviderSelection::Explicit(provider.id());
    // Warm both routes, then alternate order across 32 paired setup sequences.
    for _ in 0..4 {
        setup_queries(unsafe { runtime.upscaler_dx12(&device) }, &default).unwrap();
        setup_queries(
            unsafe { runtime.upscaler_dx12(&device) }.with_provider(selected),
            &default,
        )
        .unwrap();
    }
    eprintln!("PROVIDER_SETUP,block,selection,elapsed_ns");
    for block in 0..32 {
        let arms = if block % 2 == 0 {
            [
                ("default", crate::query::ProviderSelection::Default),
                ("selected", selected),
            ]
        } else {
            [
                ("selected", selected),
                ("default", crate::query::ProviderSelection::Default),
            ]
        };
        for (selection, request) in arms {
            let start = Instant::now();
            setup_queries(
                unsafe { runtime.upscaler_dx12(&device) }.with_provider(request),
                &default,
            )
            .unwrap();
            let elapsed = start.elapsed();
            eprintln!("PROVIDER_SETUP,{block},{selection},{}", elapsed.as_nanos());
        }
    }
    assert_eq!(context.provider().unwrap().id(), provider.id());
    context.destroy().unwrap();
    eprintln!("Provider setup timing: PASSED blocks=32 samples=64");
}
