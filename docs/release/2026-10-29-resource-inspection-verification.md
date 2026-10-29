# Resource inspection module verification — 2026-10-29

**U13 is implemented and verified within the recorded Windows/source bounds.**
Seven existing source/native profiles pass 1,419/1,419 required case checks.
A focused archive inspection adds 7/7 passing checks; Cargo's own package
verification also passes in a fresh unrestricted attempt. These are development
checkpoint results, not final publication-package or Proton qualification.

The [machine-readable record](2026-10-29-resource-inspection-verification.json)
contains every case outcome and command, equipment identities, source hashes,
token-comparison hashes, archive identities and retained-inventory verification.
Repeated command arguments/environments are stored in lossless tables; the JSON
includes the decoding columns and reconstruction rule.
It also preserves the unsuccessful attempts separately. The
[consolidated candidate report](2026-10-29-candidate-verification.md) remains the
release verification hub; its earlier archive-bound results retain their
original byte identities.

## Implemented boundary

`resources.rs` now owns ordered frame inspection and the shared converted
resource representations. Its three private submodules separate fixed DXGI/FFX
surface and view mappings (`formats`), resource metadata validation and native
descriptor conversion (`conversion`), and COM identities, device membership and
format-capability queries (`inspection`). Generation and upscaling retain their
shared helpers through private re-exports. This selects no public API, native
ABI or additional resource support.

| File | Lines |
|---|---:|
| [resources.rs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs) | 338 |
| [conversion.rs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources/conversion.rs) | 230 |
| [formats.rs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources/formats.rs) | 122 |
| [inspection.rs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources/inspection.rs) | 342 |

The former single file contained 959 lines. All 20 function signatures/bodies
match the original token sequences after ignoring whitespace/comments, excluding
visibility/import declarations and one relocated test-local error-type import.
The `Converted` and `ConvertedResources` declarations and `ResourceObjects`
alias also match. The pre-refactor file SHA-256 is
`df267343445e77caa581ba554d6ef0902fab4622a653b9cf883d583cb6c3e315`; the companion record retains
the per-function/declaration token hashes and the four resulting file hashes.
This is a source comparison, not an independent soundness or code-generation proof.

In particular, inspection order, first-error selection, COM identity guards and
their acquisition/release order, native descriptor bits and role/format rules
are preserved. The guard-owning orchestration bodies are unchanged. Existing
controlled tests still check alias/device failures, format/view capabilities,
error sources, retry/non-entry and reference balance. The shared core and both
ownership families remain intact. `common.rs`, the descriptor tuple and the
historical test entry points are unchanged; U14/U15 own those later steps.

## Execution and results

Windows x64/MSVC, Rust 1.99.0 (LLVM 23.1.1), exact MSRV 1.88.0 and Python 3.14.1.
Native execution used adapter 0, AMD Radeon RX 9060 XT (`1002:7590`), driver
`32.0.32015.7029`, pinned SDK v2.3.0 headers at
`60f4ea81909200d8542eca14dccb2628b763a9a3` and the two hash-selected AMD DLLs.
Windows Authenticode checks passed; DLL hashes and compiler/toolset observations
are retained in the companion record. Advanced resource profiles explicitly
select 4.1.1 and 3.1.5, with generator coverage on 3.1.5 under the existing contracts.

| Profile | Passing case checks | Attempt |
|---|---:|---|
| `windows-upscaler-followup` | 625/625 | `20261029T115707Z-windows-upscaler-followup-7fdab7057d50` |
| `native-motion-scale` | 102/102 | `20261029T120910Z-native-motion-scale-48dce9d66447` |
| `native-formats` | 166/166 | `20261029T121356Z-native-formats-0adfa3dd1e95` |
| `native-depth-formats` | 82/82 | `20261029T122402Z-native-depth-formats-bfd948cfcacf` |
| `native-bgra-srgb` | 102/102 | `20261029T122808Z-native-bgra-srgb-d621a1d4508c` |
| `native-resource-formats` | 280/280 | `20261029T123105Z-native-resource-formats-13a907bcfeea` |
| `native-variable-output` | 62/62 | `20261029T123802Z-native-variable-output-6bfe0d62e1d3` |
| `resource-package-inspection` | 7/7 | `20261029T121321Z-resource-package-inspection-402688c1ec7e` |
| **Total passing attempts** | **1,426/1,426** | |

The 625-case integrated profile covers seven supported feature combinations,
Stable/MSRV tests and external consumers/examples, Clippy, warning-denied Rustdoc,
negative API/feature controls, independent MSVC ABI and ordinary native
query/lifecycle/dispatch/sharpening integration. Its infrastructure discovery
discovers 312 tests (308 passed, four expected platform/privilege skips).

The six additional native profiles cover RG32F and explicit motion scales,
typed colour/output/depth and generator inputs, D32/D32S8 views and plane
readback, BGRA/sRGB, the remaining resource roles and active output extents.
Both public families and the profiles' independent C++/readback controls pass.
The resource checks retain the D035–D039 validation and provider boundaries;
successful bounded execution is not universal provider or format support.

All eight passing attempts record unchanged inputs. Every complete retained
artefact inventory was independently re-read and hash-verified after execution.
Full payloads remain under ignored `target/`; this report and its JSON retain
the claims, commands, identities and outcomes needed to understand the result.

## Reproduction

Use the current verification coordinator with cached locked dependencies and a
locally staged, trusted pinned SDK. These commands make no publication attempt:

```powershell
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL = '0'
python tools/verification/run.py run windows-upscaler-followup --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/resource-inspection-verification
```

Run `native-motion-scale`, `native-formats`, `native-depth-formats`,
`native-bgra-srgb`, `native-resource-formats` and `native-variable-output`
sequentially with the same options, substituting each profile name. Verify
each finalized attempt with `run.py verify-run --run <attempt-directory>`.
These commands test the checkout, without `--artifacts`.

The package check runs `cargo package --workspace --list --allow-dirty --locked
--offline` and `cargo package --workspace --allow-dirty --locked --offline
--target-dir <fresh-directory>`. Inspect each tarball against the checkout,
including all four resource modules and both licence files. The focused
seven-case archive attempt uses existing `package_cases`, `registry` and
`archive_integrity_case` producers, then checks both extracted packages with
`cargo check --workspace --all-targets --features fsr-sdk/movable,fsr-sdk/utils`
on Stable and Rust 1.88.0 through the isolated registry. Exact commands and
archive provenance are retained in the JSON.

## Package identities and unsuccessful attempts

The local pair inspected and compiled through the isolated registry is
byte-identical to the pair from the successful built-in Cargo verification:

- `fsr-sdk-0.3.0.crate`: `dc6b69c027c2bd7a25937c095dea481424a310d729988aac2b93d465f88520ba` (163 members).
- `fsr-sdk-sys-0.1.2.crate`: `6321f5f42c9403fd91a918d68ec69306b0a1ba8e4006398c66b5b502d07a7d1f` (33 members).

The first restricted `windows-upscaler-followup` attempt
(`20261029T115408Z-windows-upscaler-followup-2523c693c85e`) remains failed:
587 passed, 15 failed and 23 blocked case checks. Windows denied temporary
fixture cleanup/writes, including Rust compile-control metadata. The identical
source passes the complete fresh attempt with the required Windows access;
no library or test-policy change was made to bypass the failures.

The initial restricted Cargo package verification also remains failed with
the temporary-registry error `no hash listed for fsr-sdk-sys v0.1.2`.
A fresh unrestricted Cargo invocation completes both package verifications,
and produces the same archive hashes. This establishes the successful retry,
not a general diagnosis or fix for every occurrence of that Cargo error.

## Performance and remaining release boundary

The source move adds no allocation, cache, policy, FFI operation or workload
change. It does not establish identical optimized machine code or timing.
No benchmark was rerun for this checkpoint. U15/final candidate reconciliation
must assess compiler/performance impact against D043's closed P08 evidence and
repeat affected measurements if warranted.

Next is [U14 common-type organization, then U15 integration](../../ROADMAP.md#next-actions).
The known active-output Rustdoc correction belongs to that final documentation
pass after the structural changes. Final presentation, one reconciled shipping
archive pair, affected final-package/platform checks and publication simulation
remain separate. No fresh Proton execution, default-generator support claim,
release approval or publication follows from this checkpoint.
