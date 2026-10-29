# fsr-sdk-sys

Curated raw Rust bindings and an explicit-path Windows/DX12 loader for AMD FSR
SDK **v2.3.0**. Version **0.1.2** supplies the native ABI used by `fsr-sdk 0.3.0`.
Most applications should use [`fsr-sdk`](https://crates.io/crates/fsr-sdk/0.3.0)
for context ownership, checked inputs and structured errors.

## Installation and API documentation

Requires Rust **1.88+**. Enable DX12 explicitly for native use:

```toml
[dependencies]
fsr-sdk-sys = { version = "0.1.2", features = ["dx12"] }
```

See the [0.1.2 API documentation](https://docs.rs/fsr-sdk-sys/0.1.2/fsr_sdk_sys/).
Without backend features, the common ABI declarations remain available on other
hosts; this does not provide a native runtime there.

## Included ABI

This is a reviewed subset of AMD's SDK, not complete generated bindings:

- `api`: common context/descriptor types, return codes, five entry points,
  provider enumeration and override, callbacks and shared resource metadata.
- `upscale`: upscaler creation, recording and reactive-mask generation,
  provider/resource/memory and quality/size/jitter queries, direct tuning and
  diagnostics. Creation/dispatch flags include exposure, depth/motion, HDR,
  non-linear colour and sRGB/PQ hints.
- `dx12`: backend creation, allocation callbacks and the corresponding opaque
  pointer payloads, available only on Windows with `dx12`.
- `loader`: explicit library acquisition and forwarding, under the same gate.

All 40 pinned `FFX_API_SURFACE_FORMAT_*` values are declared with their native
values 0–39. Constants describe native representation; supported resource roles,
views and device capabilities remain separate contracts. The
[ABI coverage inventory](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/ABI_COVERAGE.md)
records each selected slice, its verification and wrapper use.

## Platform, loading and safety

The supported native target is **Windows x64/MSVC + DX12**. Other Windows targets
are unverified and unsupported even if compilation succeeds. There is no native
Linux loader or Vulkan implementation. The reserved `vulkan` feature deliberately
fails compilation, including with `--all-features`; initial wrapper Vulkan
upscaling is planned for 0.4.0, subject to separate native verification.

`loader::{FfxLibrary, LoadError}` resolves an explicit path, loads all five
required exports and retains the library. Acquire AMD's trusted runtime
separately and deploy `amd_fidelityfx_upscaler_dx12.dll` beside the executable.
Cargo builds do not discover, download or bundle AMD binaries. `libloading` is
optional, Windows-specific and enabled by `dx12`; common-only production
consumers need no dynamic-loader dependency.

Raw callers establish exact descriptor layouts, valid CPU pointers, provider
compatibility, callback ownership, native coordination and dependency lifetimes.
GPU resource states, command recording, submission, ordering and completion are
additional caller obligations. Opaque handles and pointer payloads own no COM
references; successful symbol lookup or an SDK return code proves neither native
safety nor GPU completion. Read each unsafe function's complete contract.

Native observations are bounded by the tested SDK/provider, GPU, driver and
workload. Existing DX12 and translated Proton results establish no native
Linux/Vulkan support or universal hardware/provider compatibility.

## Licensing and provenance

Original code is licensed under [MPL-2.0](LICENSE). Portions derived from AMD
SDK headers retain AMD's copyright and MIT licence notice in
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

`src/dx12.rs` also handwrites the pointer-payload ABI for heap/resource/clear
metadata named by the AMD callback signatures. Microsoft enum representations
and structure fields are ABI facts checked against installed Windows SDK
`d3d12.h` / `dxgicommon.h` / `dxgiformat.h`; no Microsoft header text is bundled.
The raw metadata and opaque COM pointers own no references. Callback function
aliases use the exact C calling convention; the CB result is returned by value.

All four source headers carry the retained AMD 2026 MIT notice. Native ABI
checks include separately installed SDK headers; no SDK headers or runtime
binaries are bundled. Those components retain their applicable terms.

Project documentation: <https://github.com/Render-Slop-Manufactory/fsr-sdk-rs>.
