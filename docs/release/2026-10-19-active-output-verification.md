# Active output: implementation and Windows verification

Date: 2026-10-19. Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`.
Outcome: **62/62 public-output cases and 257/257 consolidated Windows cases pass**,
both with unchanged inputs and successful evidence-integrity checks.

## Accepted implementation

The maintainer explicitly accepted the [D024 output amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
and authorized implementation. `UpscaleDispatch::{active_output_size,set_active_output_size}`
stores an optional checked output request; `None` uses the receiving context's
maximum. Smaller output requires the existing dynamic option. Independent-axis
bounds reject with `DispatchValidation::ActiveOutputSize` before COM/native entry,
without poisoning a usable context. Existing poison and required-mask precedence
are preserved. Preflight and dispatch share inspection; dispatch always rechecks.

Output/display-motion allocations still equal the context maximum. Native
resource descriptions retain actual allocation dimensions while `upscaleSize`
uses active output. Display UV motion uses that active size as its native scale;
render UV motion uses active render, and pixel motion retains `(1, 1)`. Both
owners share this mapping and existing terminal native-error policy. Reset is
forwarded unchanged. No raw ABI, dependencies, runtime binary distribution,
provider-name gate, fallback or capability query is added.

The two unsafe acquisition contracts retain actual-provider context/query trust;
both dispatch contracts explicitly require supported transitions and safe access,
valid input sampling footprints, whole-output write permission, synchronous CPU
descriptor consumption and boundary-state restoration on success. These are
conditional native obligations, not properties established by metadata or OK.
The [renderer example](../../crates/fsr-sdk/examples/dx12_active_output.rs) preserves
caller-owned resources, content, reset, submission/fences and exceptional cleanup.

## Tested inputs and commands

Windows 11 build 10.0.26300, x86_64-pc-windows-msvc + DX12 debug layer, adapter 0
AMD Radeon RX 9060 XT (`0x1002:0x7590`, feature level 12_0), driver
`32.0.31041.1004`. Rust/Cargo 1.99.0 and exact Rust 1.88.0; MSVC 14.44.35207,
Windows SDK 10.0.22621.0, Python 3.14.1 and PowerShell 7.6.7.

SDK v2.3.0 remains pinned at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
Four header hashes and Valid AMD signatures are checked. Loader/provider SHA-256:
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` /
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
No download or strict online revocation verification ran. Selected live IDs are
`17700776140664213505` (4.1.1) and `17700776140660019205` (3.1.5), checked against
the requested enumerated IDs after context creation.

From the repository root, with cached dependencies:

```powershell
$env:CARGO_INCREMENTAL='0'
python tools/verification/run.py run native-variable-output --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed
```

| Check | Result and retained evidence |
|---|---|
| Public output | 62/62 pass, attempt `20261019T211156Z-native-variable-output-59cfac16a3bf`: ten preparation/build cases and 52 children. [Commands, inputs, native identities and verdicts](2026-10-19-active-output-verification/public-path.json). |
| Consolidated Windows | 257/257 pass, attempt `20261019T211500Z-windows-upscaler-followup-ef5c08660198`: infrastructure/archive checks, independent MSVC ABI, stable tests/check/Clippy/docs in four configurations, exact MSRV tests/checks, public consumers, dependency isolation, examples and existing native regressions. [Commands, inputs and verdicts](2026-10-19-active-output-verification/regression.json). |
| Controlled geometry/error tests | Both families cover defaults/setter clearing, independent bounds, dynamic gating, render/output changes separately/together, render/display UV/pixel mapping, real allocation metadata, exact smaller/oversized extent rejection, reference retention, nonpoisoning retries and unchanged reset. Successful preflight cannot authorize a later changed request. Unknown native error bits survive, and poison wins over invalid output/COM inspection. |
| Auditor controls | Eight tests pass: original raw controls plus public family/route identity, public rejection failure and independently checked render-pixel input units. The complete profile dependency/uniqueness inventory passes within the Windows suite. |
| Packages | `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` and workspace packaging with `--no-verify` pass. Opened archives include the exact new example/API source, byte-identical MPL and retained AMD licence texts, and no DLL/EXE/raw payload. [File inventories and hashes](2026-10-19-active-output-verification/package-checks.json). This is packaging inspection, not final extracted-package verification. |

The public GPU run precedes two Rustdoc-only corrections to active-render and
movable-construction descriptions. [Exact source delta](2026-10-19-active-output-verification/source-delta.json)
identifies those replacements and before/after hashes; reversing them reproduces
the public run's source hashes. All other inventoried source bytes match between
runs. The consolidated run and package inspection use the corrected descriptions;
no executable logic changed between the two coordinator runs.

Initial direct Rust/auditor checks encountered sandbox-denied writes in temporary
directories. They passed after running outside that sandbox; neither blocked
attempt is counted as a GPU failure or a passing check. Incremental compilation
was disabled to avoid the environment's hard-link warnings.

## Public GPU observations

Each provider/family has twelve three-frame workloads plus one no-dispatch
control: **144 dispatched frames and 12 control frames**, 52 successful context
destructions, zero debug errors/device removals/input mismatches. Maxima are
render 320×180/output 640×360; output/display-motion textures remain maximum-sized.
The workload covers stable/default output, aligned and rounded shrink/grow,
width-only and height-only changes, and rounded output with render/display UV
and pixel motion, sharpening off/on. The scene is the raw spike's translating
analytic plane; selected-provider jitter queries use current output dimensions.
Reset is true only on the first frame; each prior frame is CPU-fenced before
replacement resources and the next per-frame queue are used.

The public route runs real preflight and unsafe dispatch. Production resource
descriptions are compared with independent pinned C++ inline conversion; descriptor
geometry/scalars are checked before recording. The C++ helper checks every GPU
input byte against uploads. Python independently reconstructs packed inputs and
audits every output readback. All active pixels are finite and changed from the
seed; a written pixel means any channel changed, with finiteness checked on all
four channels. [Measurements](2026-10-19-active-output-verification/measurements.json)
retain per-child selections, descriptors, metrics and input/output hashes.

Both providers' rounded sequence 637×357 → 511×287 → 637×357 writes through
640×360 → 512×288 → 640×360 in these maximum allocations, with sharpening off/on.
Outside-active writes are 2,991 → 799 → 2,991 pixels. No padding preservation
criterion is imposed. All 24 matched local/movable comparisons have identical
inputs/outputs; all 16 matched UV/pixel encoding comparisons have identical
outputs across their three phases ([comparisons](2026-10-19-active-output-verification/comparisons.json)).
This is bounded descriptor/encoding consistency, not proof of general motion
consumption, sampling coverage or temporal quality.

## Limits and completion boundary

The accepted maximum-allocation output slice is implemented and verified for
bounded integration on this stack. Native rejection/error cleanup was not
triggered in the GPU runs; controlled errors do not prove AMD failure atomicity.
No generic safe numerical domain, out-of-resource access proof, GBV result,
overlapping execution or new thread-handoff permission follows. The recorded
2.3.4 path ignores smaller output and was not exercised through this public
contract. Smaller/oversized allocations and render-allocation relaxation remain
separate work; the raw spike and its [footprint correction](../research/upscaling-dispatch.md#variable-output-native-follow-up-2026-10-19)
are preserved unchanged. Renderer-owned reset and GPU duties remain in force.

Full logs, raw payloads and development archives remain under ignored `target/`.
Compact evidence is normalized UTF-8/LF, with personal paths/transient pointers
sanitized and [byte inventory](2026-10-19-active-output-verification/inventory.json)
protected by scoped Git preservation. Historical GPU hashes are measurements,
not promises of reproduction elsewhere. Complete upscaling coverage and final
release-archive verification/publication remain open.
