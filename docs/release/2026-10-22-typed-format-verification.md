# Typed colour, output and R16F depth verification

- Date: 2026-10-22, Europe/Berlin. Coordinator attempt names use UTC, beginning
  `20261021` for the checks after local midnight.
- Scope: accepted [D035](../adr/D035.md), development `fsr-sdk 0.3.0` /
  `fsr-sdk-sys 0.2.0`, SDK v2.3.0, Windows x64/MSVC + DX12, both ownership families.
- Status: implementation and bounded verification complete. Stable CI passes
  130 cases, Rust 1.88.0/MSRV 113 and public/C++ native verification 166.
  All three finalized runs retain unchanged verification inputs.

## Implemented contract

The existing borrowed resource APIs now accept RGBA32F, R11G11B10F, RGB10A2 UNORM
and RGBA8 UNORM for main colour, independently selected output and each generator
colour input. Device depth additionally accepts R16F. Exposure remains R32F/1×1,
motion RG16F/RG32F and masks/generator output R8 UNORM. Raw bindings add exactly
five pinned surface constants: 3, 10, 16, 17 and 21; layouts and dependencies stay
unchanged.

Additional typed formats use shared metadata/device inspection in optional
preflight and compulsory recording. Device queries run only after canonical
membership checks, in a fixed role order. Inputs need texture/load/sample support
and a typed UAV view if their allocation permits a UAV. New outputs require a
shader-readable allocation, texture/load/UAV-view support and typed stores.
Read-only UAV-capable inputs do not acquire typed-load/store requirements.
Baseline formats retain their existing query behaviour and RG32F retains its
existing diagnostic representation.

Missing support bits report the resource role and `FormatSupport`. Failed new
queries return `ContextDeviceForResource(role)`, `CheckFormatSupport` and exact
HRESULT bits. Both are pre-native failures and preserve usable eligibility.
Native recording errors retain terminal context/list handling. No conversions,
new public format selectors, resource allocations or provider fallbacks are
introduced in the wrapper.

## Equipment and inputs

| Item | Observed configuration |
|---|---|
| Platform | Windows 11 build 10.0.26300, x86_64-pc-windows-msvc |
| GPU | Adapter 0, AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, feature level 12_0 |
| Driver | `32.0.32015.7029` |
| Stable | Rust/Cargo 1.99.0 |
| MSRV | Rust 1.88.0, separately selected |
| Native compiler | MSVC 14.44.35207, Windows SDK 10.0.22621.0 |
| Coordinator | Python 3.14.1, PowerShell 7.6.7 |
| SDK | v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3` |
| Selected providers | 4.1.1 / `17700776140664213505`; 3.1.5 / `17700776140660019205` |

All four staged headers match [the normalized-text pin hashes](../../tools/native-headers.json).
The loader/provider DLL SHA-256 values are respectively
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Both retain Valid AMD Authenticode signatures. No SDK download or strict online
revocation check is part of these runs.

## Reproduction and result boundary

From an x64 MSVC-capable Windows environment with the pinned local SDK and adapter:

```powershell
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-formats --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Ordinary compiler fixtures need writable temporary directories. The first direct
workspace test attempt hit sandbox access denial; rerunning outside the sandbox
passed after correcting two old tests that still treated RGBA8 as rejected.
Those tests now use unselected BGRA for their format-rejection controls.

The first native attempt, `20261021T231432Z-native-formats-3ef924ad6057`, completed
all 166 cases with passing case evaluations. The coordinator correctly rejected
the overall run because source/test/documentation inputs changed during execution
(`inputs_unchanged: false`). It is not counted as a passing finalized attempt.
The final confirmation runs use frozen inputs. Their observations, status and
compact evidence are recorded below. The earlier CI attempt
`20261021T232204Z-ci-1bdd3566c915` likewise passed all 130 case evaluations but
was rejected for changed inputs; its overall failed status is retained.

## Finalized results

| Attempt | Outcome | UTC execution interval |
|---|---|---|
| `20261021T232642Z-ci-0ebbbd90c083` | Passed, 130/130 cases; unchanged inputs | 23:26:42–23:30:22, 2026-10-21 |
| `20261021T232406Z-msrv-15c171b6df2a` | Passed, 113/113 cases; unchanged inputs | 23:24:06–23:26:20, 2026-10-21 |
| `20261021T232341Z-native-formats-a01f7456ea95` | Passed, 166/166 cases; unchanged inputs | 23:23:41–23:33:06, 2026-10-21 |

Stable CI includes independent MSVC ABI assertions, infrastructure/archive checks,
formatting, all four feature configurations' check/test/Clippy/Rustdoc, public
consumers, dependency isolation and complete examples. MSRV repeats the relevant
checks/tests, consumers and examples with Rust 1.88.0. Source suites ran outside
the sandbox to permit their compiler fixtures' temporary files, using separate
ignored Cargo target directories. The native run used the default ignored target.

All three final attempts passed `verify-run` artifact-integrity validation. Their
266 shared repository input files have matching hashes; ABI/native runs
additionally identify their pinned SDK inputs. The self-contained
[compact measurements](2026-10-22-typed-format-results.json) retain each workload,
role/format selection, frame hashes, output metrics, device capabilities, run
identities and earlier rejected outcomes. Its UTF-8/LF bytes have SHA-256
`0c5c0311b3048e66808bd1e166f9fa84452b95000e00596d5aa29bef13c5b86e`.
Raw GPU payloads remain ignored local diagnostics and are not published evidence.

The native inventory contains 152 paired public/C++ workloads: 26 per owner on
4.1.1 and 50 per owner on 3.1.5. Four separate changed-input sensitivity checks
and ten equipment/ABI/build cases bring the total to 166. Across the public/C++
routes, 912 submitted/fenced frame sequences complete, including 24 no-dispatch control
frames. There are 888 main-upscale dispatches and 288 generator dispatches across
the two routes. Every corresponding complete same-format output/mask readback is
byte-identical; diagnostics report no ordinary debug errors or device removal.
All active-write, finite-output, zero/contrast-mask and sensitivity criteria pass.

The first full native pass preceded final test-equipment hardening. Final equipment
explicitly retains the C++ reference's device/library on unownable-success or
destruction failure, discards pending recordings before rejection cleanup and
exits its isolated child without unwinding when completion cannot be proved.
These exceptional outcomes are not claimed as injected native/GPU observations.
The final 166-case confirmation exercises the hardened equipment's success paths.

## Verification coverage and limits

Controlled tests exercise the complete role matrix, each missing capability bit,
HRESULT failures for every new queried role with the same error bits, query order,
actual usage/state/format mapping, baseline no-query routes, metadata/device
precedence, retry, COM reference balance and terminal failure. Both recording
operations and both owners share these checks. Portable downstream consumers
exercise the additional diagnostic variant without backend dependencies.

The [frozen native protocol](../../tools/verification/typed-formats-protocol.md)
compares independent public and pinned-header C++ contexts with the same provider,
formats, inputs and frame sequence. The C++ consumer constructs descriptors
without the Rust mapping. Tight input readbacks, device bits, complete active
writes, finite floating results, no-dispatch controls, changed-RGB sensitivity,
ordinary debug/device checks, fences and cleanup gate exact same-format result
comparisons. Quantized formats are decoded independently; cross-format equality
is not a criterion. Cases include sharpening, UAV-capable inputs, R16F
standard/reversed depth, above-one floating colour and mixed-format active output.

Generator cases independently vary both colour roles on selected 3.1.5, including
mixed formats, equal-black zero-mask controls, contrast masks, UAV-capable inputs,
active-region shrink/grow and subsequent mask consumption. They do not reclassify
the existing 4.1.1/2.3.4 generator no-write failures.

This verifies bounded wrapper access and integration on one GPU/driver deployment.
Device bits and native success certify neither universal provider support nor
bounded shader access. Whole-output writes remain permitted under D024; padding
preservation, reconstruction quality and performance are not acceptance claims.
BGRA/sRGB, depth-buffer/typeless and other applicable candidates remain U05 work.
No final release archives, registry consumers, publication or new target support
are verified by these development-source checks.
