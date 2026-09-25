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

`Upscaling` · `Frame Generation` · `Ray Regeneration` · `Radiance Caching`

The project wraps AMD's official FSR SDK and runtime. It does not reimplement FidelityFX Super Resolution.

>[!NOTE]
>**Naming note:** AMD currently brands the FidelityFX SDK as the **FSR SDK**, hence the repository name `fsr-sdk-rs`. Historically, FSR meant *FidelityFX Super Resolution*, but the current SDK also includes technologies such as Frame Generation, Ray Regeneration, and Radiance Caching. This project aims to cover those parts of the SDK as well, rather than only super-resolution functionality.

## Status

Early development. See [ROADMAP.md](ROADMAP.md) for current status and next steps.

The initial target is AMD FSR SDK `v2.3.0` on Windows with DX12, as recorded in
[D002](docs/DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline).

## Crates

- `fsr-sdk-sys` — low-level bindings to the native FidelityFX API
- `fsr-sdk` — checked Rust abstractions built on the raw bindings, with an
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

The Windows/DX12 API can load an explicit trusted v2.3.0 runtime and construct
upscaler contexts. The bounded `unsafe Upscaler::dispatch` method records a
Windows/DX12 frame from borrowed command-list and resource inputs. The caller
owns GPU states, submission, synchronization and post-return lifetimes under
[D011](docs/adr/D011.md). A public-route GPU readback passed on one tested
configuration ([M6b verification](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)).
The `windows` 0.62 device, command-list and resource types are part of this API.

Eventually, most users should only need:

```toml
[dependencies]
fsr-sdk = "..."
```

During development, the crate can be consumed directly from Git for
experimentation and contribution:

```toml
[dependencies]
fsr-sdk = { git = "https://github.com/Render-Slop-Manufactory/fsr-sdk-rs" }
```

Use `runtime::Runtime::load(path)` inside an `unsafe` block after establishing
the trusted runtime and external/native FidelityFX integration obligations in
[D008](docs/adr/D008.md). Pass that runtime, a borrowed
`upscaler::ID3D12Device`, and checked `upscaler::Dimensions` in
`upscaler::UpscalerOptions` to `upscaler::Upscaler::new`. A context retains the
runtime and its own COM device reference until teardown. Zero dimensions are
rejected in Rust; further size limits are entrusted to the supported runtime
and may fail or abort. The supported signed v2.3.0 DX12 deployment places the
upscaler provider DLL beside the executable; see [D007](docs/adr/D007.md).

## Platform support

Current focus:

| Backend | Status |
|---|---|
| DX12 | In development |
| Vulkan | Not implemented; enabling the feature fails compilation |

Platform support follows the capabilities of the wrapped AMD SDK.

## Documentation

See [SCOPE.md](SCOPE.md) for goals, boundaries, crate responsibilities, and
durable engineering principles. [ROADMAP.md](ROADMAP.md) tracks implementation
status and next steps.

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
