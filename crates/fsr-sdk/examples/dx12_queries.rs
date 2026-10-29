// SPDX-License-Identifier: MPL-2.0

//! Acquired setup planning and live queries; main creates no native work.
#[cfg(all(windows, feature = "dx12"))]
pub fn inspect(
    mut setup: fsr_sdk::upscaler::dx12::UpscalerSetup<'_, '_>,
    options: &fsr_sdk::upscaler::UpscalerOptions,
) -> Result<(), fsr_sdk::error::Error> {
    use fsr_sdk::query::QualityMode;
    let output = options.max_upscale_size();
    let ratio = setup.ratio(QualityMode::Balanced)?;
    let suggested = setup.render_size_for_quality(output, QualityMode::Balanced)?;
    let requirements = setup.resource_requirements()?;
    let providers = setup.providers()?;
    let estimate = setup.estimate_memory(options)?;
    for provider in providers {
        println!("available: {:?} {}", provider.id(), provider.name());
    }
    println!(
        "suggested={suggested:?}, required={:#x}, estimate={}",
        requirements.required().bits(),
        estimate.total_bytes()
    );
    let mut context = setup.create(options)?;
    // The context independently retains its runtime/device; it does not borrow setup.
    let provider = context.provider()?;
    let phases = context.jitter_phase_count(options.max_render_size(), output)?;
    let jitter = context.jitter_offset(0, phases)?;
    let requirements = context.resource_requirements()?;
    let usage = context.gpu_memory_usage()?;
    println!(
        "provider={provider:?}, ratio={ratio}, jitter={jitter:?}, requirements={requirements:?}, memory={usage:?}"
    );
    // This function records no GPU work; terminal cleanup needs no earlier-work drain.
    context.destroy()
}
fn main() {}
