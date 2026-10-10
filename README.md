> [!WARNING]
> This project is manufactured by **[Render Slop Manufactory](https://github.com/Render-Slop-Manufactory)**.
>
> I have AI tokens to burn, so this is part of an experiment to see how far I can get by orchestrating AI agents through rendering problems I do not necessarily understand in all their horrifying detail myself.
>
> There is a lot of AI-generated code in here. There is also a lot of research, testing, review, and making one model check whether another model made shit up.
>
> Maybe some of it will be useful. At least the AI is convinced there is real verification work underneath it.

<img src="docs/assets/ferris.svg" width="72" align="right" alt="Ferris the Rustacean">

# fsr-sdk-rs

Rust bindings and higher-level abstractions for AMD's official FSR SDK.

> [!NOTE]
> **About the name:** FSR originally stood for *FidelityFX Super Resolution*.
> AMD's FSR SDK also covers Frame Generation, Ray Regeneration and Radiance
> Caching. This project aims to cover the entire SDK, including all of these
> technologies.

## Scope

The long-term goal is complete Rust coverage of AMD's FSR SDK: its effects,
runtime controls and supported backends, independent of any game engine or
renderer. Use `fsr-sdk` for ownership helpers and checked inputs;
`fsr-sdk-sys` provides the low-level native bindings.

## What works today

| SDK feature | In 0.2.0 | Coverage |
|---|---|---|
| Upscaling | Available (partial) | DX12, fixed non-HDR profile, optional native sharpening |
| Frame Generation | Not implemented | Planned, including presentation and frame pacing |
| Ray Regeneration | Not implemented | Planned ray-tracing denoising integration |
| Radiance Caching | Not implemented | Planned radiance-cache integration |

Native support is Windows x64/MSVC with SDK v2.3.0; Vulkan is not implemented.
Upscaling uses explicit exposure. The optional `movable` feature allows runtime
and upscaler thread handoffs. Common input types are portable.
Native testing covers one GPU/driver/provider configuration; broader compatibility
and image quality remain unverified.

## Install

Requires Rust **1.88+**. DX12 is enabled by default.

```toml
[dependencies]
fsr-sdk = "0.2.0"
```

Upgrading from 0.1.0? See the [changelog](CHANGELOG.md) for API changes.

Acquire the [AMD SDK v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0)
runtime separately. Load its trusted DX12 loader
DLL explicitly and place `amd_fidelityfx_upscaler_dx12.dll` beside your executable.
Cargo builds do not download or bundle AMD binaries.

## Integrate

Create an upscaler once, then record frames using your renderer's device,
command list and textures. This sketch assumes those objects are already prepared:

```rust
use fsr_sdk::{
    runtime::Runtime,
    upscaler::{CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels,
               UpscaleDispatch, UpscaleResources, Upscaler, UpscalerOptions},
};

let runtime = unsafe { Runtime::load_unchecked(loader_path)? };
let options = UpscalerOptions::new(
    Dimensions::new(320, 180)?,
    Dimensions::new(640, 360)?,
);
let mut upscaler = Upscaler::new(&runtime, &device, options)?;
let frame = UpscaleDispatch::new(
    &command_list,
    UpscaleResources::new(&color, &depth, &motion_vectors, &exposure, &output),
    JitterOffsetPixels::new(0.25, -0.25)?,
    FrameTimeMillis::new(16.667)?,
    CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
    true,
);
let recorded = unsafe { upscaler.dispatch(frame) };
// On Ok: submit once, then retain context/resources through GPU completion.
// On native error: discard this list, finish earlier work, then tear down.
```

Your renderer manages resource states, submission and GPU synchronization.
Loading and dispatch are unsafe; keep the context and GPU resources alive until
work completes. See the [API documentation](https://docs.rs/fsr-sdk/0.2.0/fsr_sdk/)
for the full contracts and the [DX12 example](crates/fsr-sdk/examples/dx12_upscale.rs)
for integration functions.

## Optional thread handoff

```toml
fsr-sdk = { version = "0.2.0", features = ["movable"] }
```

`MovableRuntime` and `MovableUpscaler` are `Send` but not `Sync`. Their unsafe
acquisition requires native compatibility on destination threads and coordinated
access to shared native state, including cleanup. The ordinary types stay on
their acquiring thread.

## Contributing

Interested in helping? The [rolling roadmap](ROADMAP.md) tracks open work.
[CONTRIBUTING.md](CONTRIBUTING.md) covers the workflow and setup instructions.

## License

Original code is [MPL-2.0](LICENSE); AMD-derived declarations retain their
[AMD MIT notice](crates/fsr-sdk-sys/LICENSE-AMD). AMD runtime binaries have their
own terms. This project is independent and is not affiliated with or endorsed by AMD.
