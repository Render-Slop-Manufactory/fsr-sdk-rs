# fsr-sdk-sys

Raw Rust bindings and an explicit-path Windows/DX12 loader for AMD FSR SDK
v2.3.0. The implemented ABI is a limited subset including upscaler/DX12 creation
and the narrow upscaler dispatch resource and descriptor payloads.
Provider-identification instrumentation remains test-local.
The [ABI coverage inventory](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/ABI_COVERAGE.md) tracks each selected
slice, its binding method, verification and wrapper support status.

The supported native target is `x86_64-pc-windows-msvc` with the `dx12` feature,
which this raw crate does not enable by default. Windows/DX12 APIs are exposed
on other Windows targets, but those targets are currently unverified and
unsupported; compilation may succeed. Native verification is limited to the
recorded SDK, runtime/provider, GPU, driver and workload configurations; target
support does not imply compatibility with every hardware/provider configuration.
The [project scope](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/SCOPE.md#platform-and-compatibility-boundaries)
owns this policy; the [M6b record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
preserves one tested public-dispatch configuration.

## Installation and API documentation

The [`0.1.0` crate](https://crates.io/crates/fsr-sdk-sys) and its
[API documentation](https://docs.rs/fsr-sdk-sys/0.1.0/fsr_sdk_sys/) are published.
For direct Windows/DX12 use, enable the feature explicitly:

```toml
[dependencies]
fsr-sdk-sys = { version = "0.1.0", features = ["dx12"] }
```

Most applications should use [`fsr-sdk`](https://crates.io/crates/fsr-sdk) for
context ownership and checked inputs.

## Licensing and provenance

Original code is licensed under [MPL-2.0](LICENSE). Portions derived from AMD
SDK headers retain AMD's copyright and MIT license notice in
[LICENSE-AMD](LICENSE-AMD). The MPL markers identify this project's licensing;
they do not remove AMD's retained notice. This package is not offered under a
package-wide MIT alternative.

The baseline is AMD SDK v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Source paths below are relative to
the SDK's `Kits/FidelityFX/` directory:

| Derived declarations | Source headers |
|---|---|
| `src/api.rs` | `api/include/ffx_api.h`, `api/include/ffx_api_types.h` |
| `src/upscale.rs` | `upscalers/include/ffx_upscale.h` and common API tag definitions |
| `src/dx12.rs` | `api/include/dx12/ffx_api_dx12.h` |
| `tests/context_lifecycle/abi.rs` (provider query only) | `api/include/ffx_api.h` |

All four source headers carry the retained AMD 2026 MIT notice. Native ABI
checks include separately installed SDK headers; no SDK headers or runtime
binaries are bundled. Those components retain their applicable terms.

Project documentation: <https://github.com/Render-Slop-Manufactory/fsr-sdk-rs>.
