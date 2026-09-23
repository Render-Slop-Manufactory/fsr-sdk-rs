> [!NOTE]
> This project is manufactured by **[Render Slop Manufactory](https://github.com/Render-Slop-Manufactory)**.
>
> I have AI tokens to burn, so this is part of an experiment to see how far I can get by orchestrating AI agents through rendering problems I do not necessarily understand in all their horrifying detail myself.
>
> There is a lot of AI-generated code in here. There is also a lot of research, testing, review, and making one model check whether another model made shit up.
>
> Maybe some of it will be useful. At least the AI is convinced there is real verification work underneath it.

# fsr-sdk-rs

Rust bindings and safe abstractions for AMD's modern FidelityFX Super Resolution SDK.

The project wraps AMD's official FSR SDK and runtime. It does not reimplement FidelityFX Super Resolution.

## Status

Early development. See [ROADMAP.md](ROADMAP.md) for current status and next steps.

The initial target is AMD FSR SDK `v2.3.0` on Windows with DX12, as recorded in
[D002](docs/DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline).

## Crates

- `fsr-sdk-sys` — low-level bindings to the native FidelityFX API
- `fsr-sdk` — safe, idiomatic Rust abstractions built on top of the raw bindings

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

The public API is still under development.

Eventually, most users should only need:

```toml
[dependencies]
fsr-sdk = "..."
```

During development, the crate can be consumed directly from Git for
experimentation and contribution; the high-level wrapper is not yet usable for
actual FSR upscaling:

```toml
[dependencies]
fsr-sdk = { git = "https://github.com/Render-Slop-Manufactory/fsr-sdk-rs" }
```

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
