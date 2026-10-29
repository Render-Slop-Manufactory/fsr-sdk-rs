<img src="docs/assets/ferris.svg" width="72" align="right" alt="Ferris the Rustacean">

# fsr-sdk-rs

Rust bindings and higher-level abstractions for AMD's official FSR SDK.

> [!WARNING]
> This project is manufactured by **[Render Slop Manufactory](https://github.com/Render-Slop-Manufactory)**.

> [!NOTE]
> **About the name:** FSR originally stood for *FidelityFX Super Resolution*.
> AMD's FSR SDK also covers Frame Generation, Ray Regeneration and Radiance
> Caching. This project aims to cover the entire SDK, including all of these
> technologies.

## Scope

The long-term goal is complete Rust coverage of AMD's FSR SDK: its effects,
runtime controls and supported backends, independent of any game engine or
renderer. Vulkan support is an explicit goal; Windows/DX12 code and dependencies
must stay isolated from common APIs and future backends.
Use `fsr-sdk` for ownership helpers and checked inputs;
`fsr-sdk-sys` provides the low-level native bindings.

## What works today

| SDK feature | In 0.3.0 |
|---|---|
| Upscaling | DX12, including HDR, dynamic resolution, masks, queries and tuning |
| Frame Generation | Planned, including presentation |
| Ray Regeneration | Planned |
| Radiance Caching | Planned |

Native target: **Windows x64/MSVC + DX12**, SDK **v2.3.0**. Proton has
experimental coverage. Common inputs are portable; reactive-mask generation
depends on the provider.

| Cargo feature | Default | Purpose |
|---|---|---|
| `dx12` | On | Windows/DX12 runtime and upscaling |
| `movable` | Off | Runtime and upscaler thread handoff |
| `utils` | Off | Provider selection by FSR generation |
| `vulkan` | Off | Reserved; enabling it fails compilation |

## Performance

Added recording cost versus an independent C++ integration, at 1920×1080 output
on a Radeon RX 9060 XT (Windows/DX12, SDK v2.3.0, provider `4.1.2 *`,
driver `32.0.32015.7029`):

| Workload | CPU difference (µs) | GPU difference (µs) |
|---|---:|---:|
| Maximum render | 3.36 ± 1.21 | −2.28 ± 12.11 |
| Small render | 3.41 ± 3.03 | +0.46 ± 10.84 |
| Shrink/grow | 3.01 ± 1.61 | +1.16 ± 11.26 |
| Supplied masks | 4.84 ± 3.19 | +3.91 ± 18.76 |
| Linear HDR | 4.26 ± 1.69 | −0.50 ± 12.35 |

Values are mean ± sample standard deviation (σ) of 10 paired process-block
differences per workload, Rust minus C++; they are not per-frame latency bounds.
No wrapper-specific GPU overhead was detected. Separate checks observed no Rust
allocator events in 1,920 warmed recording calls on the calling thread.
[More info](docs/research/performance.md).

## Install

Requires Rust **1.88+**. DX12 is enabled by default.

```toml
[dependencies]
fsr-sdk = "0.3.0"
```

See the [changelog](CHANGELOG.md) for API changes.

Acquire the [AMD SDK v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0)
runtime separately. Load its trusted DX12 loader
DLL explicitly and place `amd_fidelityfx_upscaler_dx12.dll` beside your executable.
Cargo builds do not download or bundle AMD binaries.

## Integrate

Create an upscaler once through a device-bound setup, then record frames.
This first-frame sketch assumes your renderer's device, command list and matching
textures are already prepared:

```rust
use fsr_sdk::{
    runtime::Runtime,
    upscaler::{
        CameraParameters, Dimensions, ExposureInput, FrameParameters, FrameTimeMillis,
        HistoryAction, JitterOffsetPixels, UpscalerOptions,
        dx12::UpscaleResources,
    },
};

let mut runtime = unsafe { Runtime::load(loader_path)? };
let mut setup = unsafe { runtime.upscaler_dx12(&device) };
let options = UpscalerOptions::new(
    Dimensions::new(320, 180)?,
    Dimensions::new(640, 360)?,
);
let mut upscaler = setup.create(&options)?;
let resources = UpscaleResources::builder()
    .color(&color)
    .depth(&depth)
    .motion_vectors(&motion_vectors)
    .exposure(ExposureInput::Texture(&exposure))
    .output(&output)
    .build()?;
let frame = FrameParameters::new(
    JitterOffsetPixels::new(0.25, -0.25)?,
    FrameTimeMillis::new(16.667)?,
    CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
    HistoryAction::Reset,
);
let recorded = unsafe { upscaler.record(&command_list, &resources, &frame) };
// On Ok: submit once, then retain context/resources through GPU completion.
// On native error: discard this list, finish earlier work, then tear down.
```

Your renderer manages resource states, submission and GPU completion. Loading,
setup acquisition and recording are unsafe; retain the context, resources,
command list and allocator through their GPU use. Update frame values each frame;
use `HistoryAction::Continue` when continuing history. Mark abandoned recordings
on every affected context with `mark_recording_abandoned()` before cleanup.

See the [API documentation](https://docs.rs/fsr-sdk/0.3.0/fsr_sdk/) for the full
contracts, the [DX12 example](crates/fsr-sdk/examples/dx12_upscale.rs) for integration
functions, and the [migration guide](docs/MIGRATING_0_3.md) for changes from 0.2.

## Features

Enable optional features to suit your integration.

### Optional thread handoff

```toml
fsr-sdk = { version = "0.3.0", features = ["movable"] }
```

`MovableRuntime` and `MovableUpscaler` are `Send` but not `Sync`. Their unsafe
acquisition requires native compatibility on destination threads and coordinated
access to shared native state, including cleanup. The ordinary types stay on
their acquiring thread.

### Provider utilities

```toml
fsr-sdk = { version = "0.3.0", features = ["utils"] }
```

`utils::provider::select_provider` resolves FSR 2, 3 or 4 from provider names
into a selection for your setup. Explicit choices reject missing or ambiguous
matches; names alone establish no hardware compatibility. The utility is portable
and adds no dependencies. See the
[provider example](crates/fsr-sdk/examples/provider_choice.rs).

## Contributing

Interested in helping? The [rolling roadmap](ROADMAP.md) tracks open work.
[CONTRIBUTING.md](CONTRIBUTING.md) covers the workflow and setup instructions.

## Licence

Original code is [MPL-2.0](LICENSE); AMD-derived declarations retain their
[AMD MIT notice](crates/fsr-sdk-sys/LICENSE-AMD). AMD runtime binaries have their
own terms. This project is independent and is not affiliated with or endorsed by AMD.

> [!WARNING]
> **About Render Slop Manufactory**
>
> I have AI tokens to burn, so this is part of an experiment to see how far I can get by orchestrating AI agents through rendering problems I do not necessarily understand in all their horrifying detail myself.
>
> There is a lot of AI-generated code in here. There is also a lot of research, testing, review, and making one model check whether another model made shit up.
>
> Maybe some of it will be useful. At least the AI is convinced there is real verification work underneath it.
