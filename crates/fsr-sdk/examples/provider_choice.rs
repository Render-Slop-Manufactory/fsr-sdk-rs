// SPDX-License-Identifier: MPL-2.0

//! Name-based provider resolution through one acquired setup; no GPU work from main.
#[cfg(all(windows, feature = "dx12", feature = "utils"))]
pub mod renderer_integration {
    use fsr_sdk::{
        query::{ProviderSelection, QualityMode},
        upscaler::{
            UpscalerOptions,
            dx12::{Upscaler, UpscalerSetup},
        },
        utils::provider::{FsrChoice, select_provider},
    };

    /// Plan and create through one request on an already acquired effect scope.
    /// The acquiring caller maintains its lasting native/coordination contract.
    /// Results guide renderer allocations but certify no GPU compatibility.
    pub fn create(
        mut setup: UpscalerSetup<'_, '_>,
        options: &UpscalerOptions,
        choice: FsrChoice,
    ) -> Result<Upscaler, Box<dyn std::error::Error>> {
        let selected = if choice == FsrChoice::Auto {
            ProviderSelection::Default
        } else {
            let providers = setup.providers()?;
            for provider in &providers {
                eprintln!("provider {:?}: {}", provider.id(), provider.name());
            }
            select_provider(&providers, choice)?
        };
        let mut setup = setup.with_provider(selected);
        let output = options.max_upscale_size();
        let ratio = setup.ratio(QualityMode::Quality)?;
        let render = setup.render_size_for_quality(output, QualityMode::Quality)?;
        let phases = setup.jitter_phase_count(options.max_render_size(), output)?;
        let jitter = setup.jitter_offset(0, phases)?;
        let requirements = setup.resource_requirements()?;
        let memory = setup.estimate_memory(options)?;
        eprintln!(
            "ratio={ratio}, quality render={render:?}, phases={phases:?}, jitter={jitter:?}, requirements={requirements:?}, memory={memory:?}"
        );
        Ok(setup.create(options)?)
    }
}
fn main() {}
