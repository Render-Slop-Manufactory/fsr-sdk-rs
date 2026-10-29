# ABI coverage

Updated: 2026-10-29 (0.3.0 public names and common rc1 exact-package qualification reconciled).

D037 adds surface `R32_TYPELESS = 37` and usage `DEPTHTARGET = 4` /
`STENCILTARGET = 32`, each checked by independent Rust and pinned-header C++
dispatch assertions. No layout changes. [The decision](adr/D037.md) separates
native metadata from exact DX12 depth-plane views and device checks.
The [verification record](release/2026-10-22-depth-format-verification.md)
records passing stable/MSRV/ABI and both-family native D32/D32S8 integration.

This is the current inventory of selected AMD FSR SDK v2.3.0 ABI slices in
`fsr-sdk-sys`. It tracks raw bindings, verification, and wrapper coverage
separately, as required by [D006](adr/D006.md). It is not a percentage of the
SDK or a claim that an entire effect is supported. The target for the native
layout checks below is Windows x64/MSVC.

The [2026-10-18 complete-upscaling source inventory](research/upscaling-coverage.md)
originally identified gaps beyond this table: provider override, reactive
generation, Configure/debug controls, formats and DX12 allocation callbacks.
Those applicable routes are now implemented with bounded feature verification,
including [D040's production callback ABI](release/2026-10-25-callbacks-diagnostics-verification.md).
The inventory separates raw declarations, public access, actual SDK execution
and the [exact-package coverage](release/0.3.0-preparation.md#archive-coverage-before-final-verification).
The [post-cleanup functional qualification](release/2026-10-29-candidate-verification.md#post-cleanup-windows-functional-qualification)
passes the current source, MSRV, independent ABI and selected native profiles;
the subsequent [rc1 Windows report](release/2026-10-29-rc1-windows-verification.md)
passes exact-package Stable/MSRV ABI and callback/production CB exchange against
the same frozen pair qualified on Deck and Machine. This extends the evidence
binding, not the ABI slice or native safety guarantees. The subsequent
[publication simulation](release/2026-10-29-candidate-verification.md#rc1-publication-dry-run)
passes against those same frozen bytes.

| Slice | Pinned SDK v2.3.0 header | Raw binding and method | Verification | `fsr-sdk` wrapper use |
|---|---|---|---|---|
| Common API: context and scalar types, descriptor header, return codes, allocation callbacks, five entry-point signatures, dimensions and Windows message callback | `api/include/ffx_api.h`, `api/include/ffx_api_types.h` | [Handwritten `api.rs`](../crates/fsr-sdk-sys/src/api.rs) | [Rust ABI checks](../crates/fsr-sdk-sys/tests/abi.rs) and [paired SDK-header checks](../crates/fsr-sdk-sys/tests/native_abi.cpp); Windows CI runs the Rust checks | Runtime and context ownership use create/destroy; metadata/resource/memory Queries are public under D018 and utilities under D022; Configure is public through direct tuning and explicit unsafe diagnostics; Dispatch is public and unsafe |
| Global version query descriptor and upscaler effect selector | `api/include/ffx_api.h`, `upscalers/include/ffx_upscale.h` | Handwritten in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) and [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | Same paired ABI checks; [ignored count-only native query](../crates/fsr-sdk-sys/tests/query.rs) succeeded on the recorded local setup | `UpscalerSetup::providers` / `MovableUpscalerSetup::providers`, with an explicit device, paired IDs/names and immediate copying |
| Provider override descriptor/tag | `api/include/ffx_api.h` | Handwritten `ffxOverrideVersion` in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) | [Paired Rust/C++ assertions](../crates/fsr-sdk-sys/tests/query_abi.rs) added; [Windows fixtures, MSVC ABI and selected native routes pass](release/2026-10-19-windows-upscaler-verification.md) | D028 default/explicit creation, memory and utility/resource pre-create queries for both families; fresh enumeration and live identity validation |
| Upscaler creation and API-version descriptors and tags | `upscalers/include/ffx_upscale.h` | [Handwritten `upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | [Rust creation checks](../crates/fsr-sdk-sys/tests/context_lifecycle/abi.rs) and [paired SDK-header checks](../crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp); ordinary Windows CI fixtures exercise the public constructor | Both families: setup `create` and explicit unsafe `create_with_callbacks` |
| DX12 backend creation descriptor and device pointer | `api/include/dx12/ffx_api_dx12.h` | [Handwritten `dx12.rs`](../crates/fsr-sdk-sys/src/dx12.rs) | Same paired creation checks and Windows CI constructor fixtures | Both families retain an independent COM device reference; callback targets additionally retain their external dependencies |
| Callback payloads, host/DX12 allocation extension and global diagnostics | `api/include/ffx_api.h`, `api/include/ffx_api_types.h`, `api/include/dx12/ffx_api_dx12.h`, `upscalers/include/ffx_upscale.h` | `api.rs`, `dx12.rs`, `upscale.rs`; by-value CB result, all callback aliases, modern/legacy debug descriptors and flags | [Rust assertions](../crates/fsr-sdk-sys/tests/callback_abi.rs), [MSVC assertions](../crates/fsr-sdk-sys/tests/callbacks_native_abi.cpp), production raw exchange and [public verification](release/2026-10-25-callbacks-diagnostics-verification.md) | D040 callback construction and unsafe diagnostics for both families; heap ABI/controlled calls pass, SDK invocation unobserved |
| Narrow upscaler dispatch payload: resource descriptions, states, formats, usage, float coordinates, descriptor and tag | `api/include/ffx_api_types.h`, `upscalers/include/ffx_upscale.h`, `api/include/dx12/ffx_api_dx12.h` | Handwritten in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) and [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | [Rust ABI checks](../crates/fsr-sdk-sys/tests/dispatch_abi.rs), [paired SDK-header checks](../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp), five-resource comparison with AMD's inline DX12 helper in the [native M6a proof](research/records/2026-09-24-exp-m6a-native-dispatch-proof.md), and [M6b public-route GPU readback](research/records/2026-09-25-m6b-public-dispatch-verification.md) | Both families: unsafe `record(list, &resources, &parameters)` with checked frame inputs and explicit GPU obligations |
| Live-context provider-version query | `api/include/ffx_api.h` | Handwritten public raw declaration in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) | [Rust query assertions](../crates/fsr-sdk-sys/tests/query_abi.rs) and [independent SDK-header query probe](../crates/fsr-sdk-sys/tests/query_native_abi.cpp); [Windows public-query checkpoint](release/2026-10-12-windows-m10-sdk-verification.md) passes for both families | `Upscaler::provider`, owned name and opaque ID |
| Live/pre-create GPU memory descriptors and accounting; resource-requirements descriptor and six known input bits | `api/include/ffx_api.h`, `upscalers/include/ffx_upscale.h` | Handwritten in [`api.rs`](../crates/fsr-sdk-sys/src/api.rs) and [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | Same Rust/query C++ assertions; controlled output/error checks; [Windows MSVC/public-native checkpoint](release/2026-10-12-windows-m10-sdk-verification.md) passes on one configuration | setup `estimate_memory`, `Upscaler::gpu_memory_usage`, `Upscaler::resource_requirements` |
| Quality-mode ratio/render size and jitter phase/offset descriptors, tags and five SDK quality-mode constants | `upscalers/include/ffx_upscale.h` | Handwritten in [`upscale.rs`](../crates/fsr-sdk-sys/src/upscale.rs) | [Paired Rust](../crates/fsr-sdk-sys/tests/query_abi.rs) / [C++](../crates/fsr-sdk-sys/tests/query_native_abi.cpp) assertions, controlled/public utility fixtures and [Windows native pre-create/live queries for both families](release/2026-10-15-windows-m11-m14-verification.md) pass on one configuration | Four setup-bound and live Upscaler helpers, with matching movable methods under [D022](adr/D022.md) |

D030 adds `ffxDispatchDescUpscaleGenerateReactiveMask`, tag `0x10006` and four
`FFX_UPSCALE_AUTOREACTIVEFLAGS_*` constants from pinned `ffx_upscale.h`. The
[Rust assertions](../crates/fsr-sdk-sys/tests/dispatch_abi.rs) and
[paired C++ probe](../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp) check field
types, all constants, size 192/alignment 8 and header/list/three-resource/region/
scalar/flags offsets 0/16/24/72/120/168/176/180/184/188. Portable Rust execution
checks the host's 64-bit layout; Windows compilation does not execute target
assertions. Both wrapper families expose unsafe generation and optional metadata
preflight. [Local implementation checks](release/2026-10-19-reactive-mask-generation-verification.md)
are followed by passing [Windows fixtures/MSVC ABI](release/2026-10-19-windows-upscaler-verification.md)
and [provider probes](research/records/2026-10-19-exp-reactive-mask-providers.md):
3.1.5 writes masks, while 4.1.1/2.3.4 acknowledge without writes. Those failed
write criteria remain explicit; final extracted-package checks remain separate.

D029 adds `ffxConfigureDescUpscaleKeyValue`, tag `0x10007` and five key constants
from the same pinned `ffx_upscale.h`. The existing [Rust](../crates/fsr-sdk-sys/tests/query_abi.rs)
and [independent C++ probe](../crates/fsr-sdk-sys/tests/query_native_abi.cpp) assert
the tag/keys and x64 layout: size 40, alignment 8, header/key/u64/pointer offsets
0/16/24/32 and exact field types. The [implementation checks](release/2026-10-18-upscaler-tuning-verification.md)
record the earlier host assertions and Windows Rust compilation. Subsequent
[Windows verification](release/2026-10-19-windows-upscaler-verification.md) executes
the independent MSVC assertions, fixtures and bounded public Configure cycles. Both wrapper families expose the controls under
[D029](adr/D029.md)'s acknowledgement, ordering and native-error contract.

D028's [implementation follow-up](release/2026-10-18-provider-selection-verification.md)
adds provider override and the pre-create resource-requirements route; the source
inventory above remains the intake baseline. D040 binds the global debug and DX12 allocator descriptors and all callback aliases. Existing function-pointer signatures
remain unchanged; private unsafe forwarding now backs the typed public controls.

No `bindgen` output is currently checked in. The public raw crate binds the
Query/Configure descriptors listed above and the narrow upscaler Dispatch
payload/resources above. The selected Queries follow
[D018](adr/D018.md) in `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`;
[Windows verification](release/2026-10-12-windows-m10-sdk-verification.md) is complete,
while [final release preparation](release/0.3.0-preparation.md) remains open.
The optional `MovableRuntime` / `MovableUpscaler` family reuses these same slices
for construction, dispatch and the selected Queries; [D013's public-path verification](ROADMAP_HISTORY.md#movable-implementation-checkpoint)
records its bounded checks. [D015](adr/D015.md) changes module/dependency gates,
not ABI coverage. [D014](adr/D014.md) sharpening is implemented using the existing
native descriptor fields; [M8 checks](ROADMAP_HISTORY.md#frame-and-sharpening-integration)
cover mapping/controlled FFI. [M9](release/2026-10-10-m9-verification.md) records
public-API GPU integration; the
[combined release check](ROADMAP_HISTORY.md#release-qualification)
covers the selected artefacts including movable ownership.
Development's [D020](adr/D020.md) pre-exposure uses the existing dispatch field;
no raw ABI declarations change. [Windows fixture/MSRV/ABI and native execution](release/2026-10-15-windows-m11-m14-verification.md)
pass for three pre-exposure factors.
[D021](adr/D021.md) adds existing depth-inverted/infinite, display-motion and
jitter-cancellation creation bits and uses the
existing motion-vector scale field. Paired constant assertions and
[Windows fixture/MSRV/ABI and eight native profiles](release/2026-10-15-windows-m11-m14-verification.md)
pass; descriptor layouts are unchanged. The native matrix is representative,
not every combination of the implemented controls.
[D023](adr/D023.md) adds the existing automatic-exposure bit and uses the current
null resource representation. [Windows fixture/MSRV/ABI and native automatic-exposure dispatch](release/2026-10-15-windows-m11-m14-verification.md)
pass; no descriptor layout changes.
The five common function-pointer signatures exist, but they do not by themselves
provide those operations. Windows CI is configured to compile all four paired C++ probes against
hash-verified headers at the pinned commit. It does not run tests requiring AMD
DLLs or a DX12 device; see [development](DEVELOPMENT.md)
for the separate verification paths and [roadmap](../ROADMAP.md) for current
runtime status.

[D024](adr/D024.md) adds the existing dynamic-resolution bit 6 and separates
dispatch active render dimensions from unchanged allocation metadata. Paired
Rust/C++ assertions cover the flag; layouts are unchanged.
[Windows fixture/MSRV/ABI and four native active-region profiles](release/2026-10-15-windows-m11-m14-verification.md)
pass with fixed allocations and output. These development-source checks cover
one Windows GPU/driver/provider configuration; final release archives and native
execution through extracted packages still require verification.

[D025](adr/D025.md) exposes the existing reactive/transparency resource slots
and adds only the R8_UNORM surface-format constant (25). Paired Rust/C++ assertions
cover that value; descriptor layouts are unchanged. Host tests and Windows-target
compilation are recorded in the [implementation checks](release/2026-10-15-supplied-masks-verification.md).
Subsequent [mask-forwarding verification](release/2026-10-15-mask-forwarding-verification.md)
passes Windows fixtures, independent C++ assertions and 18 public native frames.
Final extracted-package confirmation remains separate.

[D026](adr/D026.md) adds only `FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE` (`1 << 0`)
from the pinned `upscalers/include/ffx_upscale.h`. Shared creation/estimate mapping
and paired Rust/C++ assertions preserve existing layouts and exports.
[Implementation checks](release/2026-10-15-linear-hdr-verification.md) distinguish
portable execution and Windows Rust compilation. The later
[Windows feature batch](release/2026-10-15-windows-feature-batch.md) confirms
C++ ABI, Windows fixtures and linear-HDR GPU execution.

[D027](adr/D027.md) adds `FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE` (`1 << 8`),
`FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB` (`1 << 1`) and
`FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ` (`1 << 2`) from the same pinned header.
Shared creation/estimate and dispatch mappings retain layouts and exports;
paired Rust/C++ assertions cover each constant. Checked context/hint pairing
has an explicit per-frame override. [Implementation checks](release/2026-10-15-colour-encoding-verification.md)
separate portable execution and Windows compilation from the later
[Windows fixture/C++ ABI and linear-HDR checks](release/2026-10-15-windows-feature-batch.md).
The [colour-signal experiment](research/records/2026-10-15-exp-non-linear-colour-signals.md)
adds bounded provider execution while retaining failed signal predictions; no
universal non-linear signal semantics are established.

[D034](adr/D034.md) adds `FFX_API_SURFACE_FORMAT_R32G32_FLOAT = 6` from
`api/include/ffx_api_types.h`, with paired Rust and independent MSVC assertions.
Existing resource/coordinate/dispatch layouts are unchanged. The wrapper's
portable `NativeMotionVectorScale` forwards accepted finite component bits and
the shared DX12 path accepts RG32F only for main motion, with device/view checks.
[Stable/MSRV/consumer and both-family public native checks](release/2026-10-21-native-motion-verification.md)
pass within their recorded scope; this does not accept other converter formats
or establish a general native numerical/access domain.

When selecting another slice, add its exact entities and header provenance,
binding method, Rust and independent native checks, platform gate, and safe API
status here. Keep one canonical Rust representation for each native entity on a
supported target. Record unverified parts explicitly rather than treating a
binding or a passing fixture as native runtime support.

[D035](adr/D035.md) adds five `FFX_API_SURFACE_FORMAT_` constants from pinned
`api/include/ffx_api_types.h`: `R32G32B32A32_FLOAT` (3), `R8G8B8A8_UNORM` (10),
`R11G11B10_FLOAT` (16), `R10G10B10A2_UNORM` (17), `R16_FLOAT` (21). Handwritten
common declarations have paired Rust/independent MSVC numeric assertions and
preserve all resource/dispatch layouts. Shared Windows/DX12 wrapper validation
admits these only in selected typed colour/output, depth and generation roles;
the declarations themselves add no provider/role certificate. The `native-formats`
profile compares public recording with a separate pinned-header C++ consumer.
[Stable/MSRV, MSVC ABI and bounded native verification](release/2026-10-22-typed-format-verification.md)
pass on one configuration; D039 subsequently completes applicable U05 formats.
Final release archives remain separate.

[D038](adr/D038.md) adds `R8G8B8A8_SRGB` (12), `B8G8R8A8_UNORM` (14) and
`B8G8R8A8_SRGB` (15) from the same pinned public enum, with paired Rust/MSVC
assertions and no layout changes. Wrapper acceptance is role-specific; typed
sRGB SRV and UNORM UAV/casting requirements are distinct device inspections.

[D039](adr/D039.md) completes the same public surface enum, values 0–39, using
handwritten faithful constants from `api/include/ffx_api_types.h`. Rust and
independent MSVC assertions cover every value; resource/dispatch layouts and
platform gates remain unchanged. Wrapper roles use checked actual allocation
and fixed-view mappings, including the explicit RGB32F public-helper omission.
Raw integer/generic constants do not imply public upscaling-role acceptance.

## Callback and diagnostics slice

[D040](adr/D040.md) adds both global-debug descriptors, all native diagnostic
levels/message values, creation/debug-view flag constants, the by-value
`FfxApiConstantBufferAllocation`, all five DX12 allocation callback signatures
and their creation extension. The [Rust assertions](../crates/fsr-sdk-sys/tests/callback_abi.rs)
and [independent MSVC assertions](../crates/fsr-sdk-sys/tests/callbacks_native_abi.cpp)
check exact types, tags, sizes, alignments and offsets, including the Microsoft
pointer-payload metadata. Both public families share the same mapping. The
`native-callbacks` profile exchanges the production raw CB aggregate in both
directions on current Rust and exact 1.88.0, with corruption controls, and runs
public callbacks and diagnostics on explicitly selected providers. Heap calls
made by equipment are distinct from SDK invocation; absence of SDK calls is
not proof that a route is unavailable or safe to remove.

The [D040 verification](release/2026-10-25-callbacks-diagnostics-verification.md) records passing Windows source/MSRV/consumer checks and all 15 public-native
stages, including the independently compiled callback assertions.

The callback slice also binds `FFX_API_RESOURCE_TYPE_BUFFER = 0` and
`FFX_API_RESOURCE_STATE_GENERIC_READ = 20`. `interop::abi` reexports these,
resource flags/usages/unknown format and message severities so a CB result can
be described through the wrapper dependency. Paired Rust/MSVC assertions and the
isolated public consumer cover the values. These callback payload constants do
not widen ordinary dispatch resource validation to buffers.
