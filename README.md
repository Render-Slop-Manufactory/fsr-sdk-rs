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

Rust bindings and higher-level abstractions for the AMD FSR SDK.

**Released:** `0.1.0` on [crates.io](https://crates.io/crates/fsr-sdk), with
[API documentation](https://docs.rs/fsr-sdk/0.1.0/fsr_sdk/) and a
[GitHub release](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.1.0)
published on 2026-09-28.

**Today:** Windows/DX12 upscaling. **Future scope:** Frame Generation, Ray
Regeneration, Radiance Caching, and other backends.

**Next release:** [D014](docs/adr/D014.md) selects a small `0.2.0` with extensible
dispatch inputs and explicit native sharpening, disabled by default. The design
was accepted on 2026-10-07; implementation and release verification are pending.
The [roadmap](ROADMAP.md#next-actions) tracks that work. The examples below use
the implemented `0.1.0` API.

**Current source, unreleased:** [D013](docs/adr/D013.md) adds the optional
`movable` feature with `MovableRuntime` / `MovableUpscaler` (`Send + !Sync`).
Unsafe acquisition requires lasting native compatibility and common-domain
coordination, including implicit cleanup. The
[consumer example](crates/fsr-sdk/examples/movable_upscale.rs) and
[bounded public-path verification](docs/release/2026-10-09-d013-public-api-verification.md)
cover independently transferred contexts. Existing local types remain
`!Send + !Sync`; this independent track is outside `0.2.0`, with no release selected.

The project wraps AMD's official FSR SDK and runtime. It does not reimplement FidelityFX Super Resolution.

Current source separates checked sizes, options and frame scalars in
`upscaler::common` from GPU construction and dispatch in `upscaler::dx12`
([D015](docs/adr/D015.md)). Existing `upscaler::{Upscaler, UpscaleDispatch, ...}`
imports remain valid. Common inputs are available on every host without backend
features; `upscaler::dx12`, its reexports and `runtime` require Windows plus
`dx12`. Vulkan remains unsupported. These source changes are unreleased.

>[!NOTE]
>**Naming note:** AMD currently brands the FidelityFX SDK as the **FSR SDK**, hence the repository name `fsr-sdk-rs`. Historically, FSR meant *FidelityFX Super Resolution*, but the current SDK also includes technologies such as Frame Generation, Ray Regeneration, and Radiance Caching. This project aims to cover those parts of the SDK as well, rather than only super-resolution functionality.

## What works today

The public `fsr-sdk` API provides a narrow DX12 upscaling path. The supported
native target is `x86_64-pc-windows-msvc` with `dx12`, enabled by default.
Windows/DX12 APIs are exposed on other Windows targets, but those targets are
currently unverified and unsupported; compilation may succeed.
[SCOPE.md](SCOPE.md#platform-and-compatibility-boundaries) owns this support policy.

- `runtime::Runtime::load_unchecked` explicitly loads a trusted FidelityFX SDK
  `v2.3.0` loader DLL. Loading is `unsafe` and does not acquire or install the SDK.
- `upscaler::Upscaler::new` constructs a context from checked dimensions and a
  borrowed DX12 device. The context retains the runtime and its own device
  reference.
- `unsafe Upscaler::dispatch` records one fixed-profile frame from a borrowed
  DX12 command list and five resources. The caller owns resource states, command
  submission, GPU synchronization, and lifetimes through completion.
- Public failures use the crate-owned `runtime::LoadError` and `error::Error`
  types, including native error codes where supplied.

One public dispatch was submitted, fenced, and read back on the
[recorded Windows/DX12 configuration](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md).
The [final Windows verification](docs/research/records/2026-09-28-exp-final-windows-release-verification.md)
repeated that regression and passed a three-frame moving smoke through one
upscaler. These results do not establish temporal image quality or support across GPUs,
drivers, or providers. Native verification is limited to the recorded SDK,
runtime/provider, GPU, driver and workload configurations; target support does
not imply compatibility with every hardware/provider configuration.
The current source also includes [controlled motion, jitter and history-reset
comparisons](docs/research/records/2026-09-30-exp-dispatch-semantics.md), verified
on that configuration; these add bounded semantic checks rather than a general
image-quality benchmark.
Vulkan and the other named SDK effects are not implemented.
See [ROADMAP.md](ROADMAP.md) for current priorities and [D011](docs/adr/D011.md)
for the dispatch contract.

## Crates

The current source workspace requires Rust 1.88 or newer. CI tests exactly 1.88.0
and current stable Rust on Ubuntu and Windows; see [development](docs/DEVELOPMENT.md).

- [`fsr-sdk-sys`](https://crates.io/crates/fsr-sdk-sys) — low-level bindings to the native FidelityFX API
- [`fsr-sdk`](https://crates.io/crates/fsr-sdk) — checked Rust abstractions built on the raw bindings, with an
  explicit unsafe DX12 dispatch boundary

The native API currently centres around:

```text
ffxCreateContext
ffxDestroyContext
ffxDispatch
ffxQuery
ffxConfigure
```

## Goals

- faithful bindings to AMD's public FSR SDK API
- safe ownership and lifetime management where possible
- dynamic native runtime loading with explicit, application-controlled acquisition
- clear error handling
- minimal downstream setup
- no dependency on a particular game engine or renderer

## Usage

Add the published crate to your application:

```toml
[dependencies]
fsr-sdk = "0.1.0"
```

The sketch below assumes a Windows/DX12 caller already has a device, an open
DIRECT command list, and five textures in the required formats and states. It
also assumes a surrounding function whose error type accepts both public error
types. The API uses the reexported `windows` 0.62 DX12 interface types.

```rust
use fsr_sdk::{
    runtime::Runtime,
    upscaler::{CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels,
               UpscaleDispatch, Upscaler, UpscalerOptions},
};

let runtime = unsafe { Runtime::load_unchecked(loader_path)? };
let options = UpscalerOptions::new(
    Dimensions::new(320, 180)?,
    Dimensions::new(640, 360)?,
);
let mut upscaler = Upscaler::new(&runtime, &device, options)?;
let frame = UpscaleDispatch {
    command_list: &command_list,
    color: &color, depth: &depth, motion_vectors: &motion_vectors,
    exposure: &exposure, output: &output,
    jitter: JitterOffsetPixels::new(0.25, -0.25)?,
    frame_time: FrameTimeMillis::new(16.667)?,
    camera: CameraParameters::new(0.1, 100.0, std::f32::consts::PI / 3.0, 1.0)?,
    reset: true,
};
let recorded = unsafe { upscaler.dispatch(frame) };
// On Ok: submit once, then retain context/resources through GPU completion.
// On native error: discard this list, finish earlier work, then tear down.
```

Read the [`dispatch` safety contract](crates/fsr-sdk/src/upscaler/dx12/dispatch.rs)
before calling it. A successful return only records work; it does not submit or
wait. Loading and construction also have native obligations under
[D008](docs/adr/D008.md). Zero dimensions are rejected in Rust; further size
limits are entrusted to the supported runtime and may fail or abort. The signed
v2.3.0 DX12 deployment places the upscaler provider DLL beside the executable;
see [D007](docs/adr/D007.md).

The [DX12 integration example](crates/fsr-sdk/examples/dx12_upscale.rs) provides
compilable creation and first-frame recording functions for renderer-owned
objects, without a standalone renderer or raw FidelityFX calls.

## Documentation

See [SCOPE.md](SCOPE.md) for goals, boundaries, crate responsibilities, and
durable engineering principles. [ROADMAP.md](ROADMAP.md) tracks implementation
status and next steps.

The detailed public API contract is published on docs.rs for
[`fsr-sdk 0.1.0`](https://docs.rs/fsr-sdk/0.1.0/fsr_sdk/) and
[`fsr-sdk-sys 0.1.0`](https://docs.rs/fsr-sdk-sys/0.1.0/fsr_sdk_sys/).
Generate it locally with the
[`Windows-target documentation command`](docs/DEVELOPMENT.md#consumer-api-documentation-and-example).
The [crate README](crates/fsr-sdk/README.md) is the package-page overview. The
[publication check](docs/release/2026-09-29-publication-check.md) records the public
registry, documentation and release status.

See the [research index](docs/research/README.md) for investigation topics,
evidence conventions, and research templates.
The [Milestone 6 plan](docs/MILESTONE_6.md) scopes a minimal native DX12
dispatch and GPU readback check through the public M5 construction path;
[M6a verification](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
records the first completed private dispatch, and
[M6b verification](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
records the public dispatch path on one tested configuration.

- [Contributing](CONTRIBUTING.md) — workflow, dependencies, and provenance
- [Development](docs/DEVELOPMENT.md) — local checks and native verification
- [Decisions](docs/DECISIONS.md) — consequential choices and rationale

## Licensing

Original code in this project is licensed under [MPL-2.0](LICENSE).
Portions derived from AMD SDK headers retain AMD's copyright and MIT license
notice; see [LICENSE-AMD](crates/fsr-sdk-sys/LICENSE-AMD) and the
[header provenance](crates/fsr-sdk-sys/README.md).

AMD SDK components and runtime binaries remain subject to their respective AMD licence terms. See the relevant AMD SDK documentation before redistributing upstream binaries.

## Disclaimer

This project is independent and is not affiliated with or endorsed by AMD.
