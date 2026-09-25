# fsr-sdk-sys

Raw Rust bindings and an explicit-path Windows/DX12 loader for AMD FSR SDK
v2.3.0. The implemented ABI is a limited subset including upscaler/DX12 creation
and the narrow upscaler dispatch resource and descriptor payloads.
Provider-identification instrumentation remains test-local.
The [ABI coverage inventory](../../docs/ABI_COVERAGE.md) tracks each selected
slice, its binding method, verification and wrapper support status.

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
