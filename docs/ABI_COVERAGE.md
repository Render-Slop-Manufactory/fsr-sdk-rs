# ABI coverage

This is the current inventory of selected AMD FSR SDK v2.3.0 ABI slices in
`fsr-sdk-sys`. It tracks raw bindings, verification, and wrapper coverage
separately, as required by [D006](adr/D006.md). It is not a percentage of the
SDK or a claim that an entire effect is supported. The target for the native
layout checks below is Windows x64/MSVC.

| Slice | Pinned SDK v2.3.0 header | Raw binding and method | Verification | `fsr-sdk` wrapper use |
|---|---|---|---|---|
| Common API: context and scalar types, descriptor header, return codes, allocation callbacks, five entry-point signatures, dimensions and Windows message callback | `api/include/ffx_api.h`, `api/include/ffx_api_types.h` | [Handwritten `api.rs`](../crates/fsr-sdk-sys/src/api.rs) | [Rust ABI checks](../crates/fsr-sdk-sys/tests/abi.rs) and [paired SDK-header checks](../crates/fsr-sdk-sys/tests/native_abi.cpp); Windows CI runs the Rust checks | Runtime and context ownership use create/destroy; Query and Configure have no public methods; Dispatch is public and unsafe |
| Global version query descriptor and upscaler effect selector | `api/include/ffx_api.h`, `upscalers/include/ffx_upscale.h` | Handwritten in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) and [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | Same paired ABI checks; [ignored count-only native query](../crates/fsr-sdk-sys/tests/query.rs) succeeded on the recorded local setup | None |
| Upscaler creation and API-version descriptors and tags | `upscalers/include/ffx_upscale.h` | [Handwritten `upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | [Rust creation checks](../crates/fsr-sdk-sys/tests/context_lifecycle/abi.rs) and [paired SDK-header checks](../crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp); ordinary Windows CI fixtures exercise the public constructor | Checked DX12 construction in `Upscaler::new` |
| DX12 backend creation descriptor and device pointer | `api/include/dx12/ffx_api_dx12.h` | [Handwritten `dx12.rs`](../crates/fsr-sdk-sys/src/dx12.rs) | Same paired creation checks and Windows CI constructor fixtures | `Upscaler::new` borrows the device and retains its own COM reference |
| Narrow upscaler dispatch payload: resource descriptions, states, formats, usage, float coordinates, descriptor and tag | `api/include/ffx_api_types.h`, `upscalers/include/ffx_upscale.h`, `api/include/dx12/ffx_api_dx12.h` | Handwritten in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) and [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | [Rust ABI checks](../crates/fsr-sdk-sys/tests/dispatch_abi.rs), [paired SDK-header checks](../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp), five-resource comparison with AMD's inline DX12 helper in the [native M6a proof](research/records/2026-09-24-exp-m6a-native-dispatch-proof.md), and [M6b public-route GPU readback](research/records/2026-09-25-m6b-public-dispatch-verification.md) | `unsafe Upscaler::dispatch` with checked fixed-profile inputs and explicit GPU obligations |
| Live-context provider-version query | `api/include/ffx_api.h` | [Handwritten test-only declaration](../crates/fsr-sdk-sys/tests/context_lifecycle/abi.rs); absent from the public raw crate | Paired creation checks cover layout and tag; native lifecycle probes have queried the provider | None; test instrumentation only |

No `bindgen` output is currently checked in. The public raw crate binds no other
Query or Configure descriptor payloads and only the narrow upscaler Dispatch
payload and resources above.
The five common function-pointer signatures exist, but they do not by themselves
provide those operations. Windows CI does not compile the paired C++ checks or
run tests requiring AMD DLLs or a DX12 device; see [development](DEVELOPMENT.md)
for the separate verification paths and [roadmap](../ROADMAP.md) for current
runtime status.

When selecting another slice, add its exact entities and header provenance,
binding method, Rust and independent native checks, platform gate, and safe API
status here. Keep one canonical Rust representation for each native entity on a
supported target. Record unverified parts explicitly rather than treating a
binding or a passing fixture as native runtime support.
