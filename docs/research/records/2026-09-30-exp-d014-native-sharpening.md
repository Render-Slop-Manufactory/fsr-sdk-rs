# Experiment: D014 native sharpening response

- Investigation date: 2026-09-30, Europe/Berlin.
- Question: does the proposed native sharpening mapping execute cleanly, what
  does enabled zero do, and does it satisfy D014's predeclared effect criteria?
- Result: **execution passed; the proposed effect gate failed in all three
  repetitions**, and failed again in an independent readback-audit run.
- SDK: AMD FSR SDK `v2.3.0`, pinned commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`; signed DX12 provider 4.1.1.
- Scope: two stationary synthetic scenes, five settings, eight frames per
  context, three isolated process repetitions. No new public API, production
  dispatch behaviour, dependencies, SDK pin or accepted decision was introduced.

## Conditions fixed before execution

The user authorized the native spike after review of the revised
[D014 proposal](../../adr/D014.md). This runs its existing protocol as an
experiment; it does not accept the API design or release gate. A
[protocol snapshot](2026-09-30-exp-d014-native-sharpening/protocol-before-run.md)
preserves the criteria used before looking at enabled outputs. The complete
D014 file then had SHA-256
`b5c3394f70b286a4ab76c3c5bdee3d95e891788da1afa7b421331c0dee61a643`.
Neither scene, thresholds nor repeat policy was changed after seeing results.

Render is 320x180; output is 640x360. Inputs are RGBA16F colour, R32F depth 0.5,
RG16F motion `(0,0)`, R32F 1x1 exposure 1.0 and RGBA16F UAV output. Native
pre-exposure is 1.0, creation/dispatch flags are zero. Camera values are near
0.1, far 100, vertical FOV pi/3 and unit scale 1; frame time is 16.667 ms.
Alpha is 1. For input pixel centres, `u=x+0.5-jitter_x`, `v=y+0.5-jitter_y`:

| Scene | RGB, identical in all channels |
|---|---|
| Frequency (scene 0) | `0.5 + 0.4*sin(0.8*(u + 0.37*v))` |
| Slanted edge (scene 1) | `0.1` when `u < 128 + 0.37*v`, otherwise `0.9` |

The eight pixel jitter pairs are `(0.25,-0.25)`, `(-0.25,0.25)`,
`(0.125,-0.375)`, `(-0.375,0.125)`, `(0.375,-0.125)`, `(-0.125,0.375)`,
`(0.0625,-0.0625)`, `(-0.0625,0.0625)`. Input half conversion uses the existing
test helper's truncating conversion; no new quantizer or gamma transform was
added. Only frame zero resets history.

For each scene, five fresh contexts process identical eight-frame histories:
disabled A, disabled B, enabled 0, enabled 0.35 and enabled 1. Disabled maps to
`enableSharpening=false, sharpness=0`; enabled maps to `true` and its stated
value. Each frame has fresh textures, uploads, allocator and DIRECT command list.
Its submission is fenced before readback and before the next frame. All frame
dependencies remain live through context destruction; explicit destroy and
helper cleanup precede the final device checkpoint. This experiment uses
per-frame completion rather than testing recording ahead or overlapping queues.

Metrics use final-frame red values inside `x=[32,608), y=[32,328)`:

- `N`: mean absolute difference between disabled A/B; the primary control is
  exact equality of their red half bits, excluding row padding. No fallback
  noise policy was used.
- `D`: enabled-to-disabled-A mean absolute difference.
- `C`: fraction of pixels with absolute difference at least `1/1024`.
- `G`: mean absolute horizontal neighbouring-pixel difference, both pixels in
  the ROI; `G_off` is the disabled-A baseline.
- `B`: absolute difference of ROI mean red.

All sums use f64 over decoded half values. For both nonzero settings, the
frequency scene must meet `D >= 0.001`, `C >= 0.01`,
`G >= 1.01*G_off` with `G_off > 0`, and `B <= 0.005` in each repetition.
Every frame must have finite RGBA, completed readback/fence, clean ordinary
debug/device checks and cleanup. Enabled zero is reported without an effect
requirement. The edge case has execution checks and diagnostics, with no
predeclared width, overshoot or monotonicity acceptance bound.

## Tested source and environment

The source baseline was `bb107d79836158bcb5c3dd2f81b8b7a7b7ea4d87`
(tree `c73e573d2e601eda8bca940c8ce3b37df3eaa971`). D013's test-only shared
inspection seam and native equipment were already present. Existing local
D014, ROADMAP and DECISIONS edits remained present. Experiment changes comprise:

- [sharpening.rs](../../../crates/fsr-sdk/tests/native/sharpening.rs): test-only
  native-field override after the same resource/scalar inspection used by public
  dispatch, through the existing owner and poisoning core;
- [sharpening.h](../../../crates/fsr-sdk/tests/native/sharpening.h): stationary
  colour-upload fixtures and packed raw red-half readback;
- small registrations/includes in
  [dispatch_readback.rs](../../../crates/fsr-sdk/tests/native/dispatch_readback.rs),
  [gpu.cpp](../../../crates/fsr-sdk/tests/native/gpu.cpp) and
  [run.ps1](../../../tools/verification/archive/2026-10-10/files/crates/fsr-sdk/tests/native/run.ps1.txt).

Production files and the public frame API were unchanged. The initial compile
attempt could not call the method-private inspection helper from a sibling test
module; the probe was corrected to use the existing test-only free inspection
function before any GPU execution. After the first complete native run, the only
probe addition was optional final-image serialization, so the audit could decode
actual half bits independently. The full original failure remains preserved.
Source/header/executable fingerprints for both runs are in the supporting
directories' `source-inputs.json` and `native-inputs.json`.

| Component | Observed value |
|---|---|
| Target / OS | x86_64-pc-windows-msvc / Windows 11 Pro 10.0.26300 |
| Selected hardware | AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590` |
| Driver / feature level | 32.0.31041.1004 / 12_0 |
| Provider | name 4.1.1, id `0xf5a5ca1e01001001`; query returned OK |
| Rust | 1.98.1, LLVM 22.1.8 |
| MSVC / compiler | 14.44.35207 / 19.44.35224.0 |
| Windows SDK / PowerShell | 10.0.22621.0 / 7.6.6 |
| Validation | Ordinary DX12 debug layer enabled, GBV disabled |
| Watchdog | 180,000 ms per isolated child; helper fence wait 60,000 ms |

Both staged AMD Authenticode signatures were Valid (`Status=0` in the captured
JSON). Trusted SDK binaries were already local; none were downloaded or retained
as research artefacts. DLL SHA-256 values:

| DLL | SHA-256 |
|---|---|
| AMD loader | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| AMD upscaler | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |
| First complete run helper | `6ffae84b8474e59e77c248a448f55b297187ed964e81c3f34447cd3a86fc8df5` |
| First complete run test executable | `59854fed6ac3f5f9dc2ad21b0869a40d2d4568bfbae913fe447cc554f055efe` |
| Audit run helper | `d04a4c004d485d4f6793acfd9d7dd198574d6e65c3b168ee59b3399405364f1e` |
| Audit run test executable | `6cc95524dc7c1a4710ac606fa90347129cc6dfba4897844ca0aacc73d5bdbbf2` |

## Reproduction and run outcomes

Prerequisites: the trusted SDK at
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX`, x64 MSVC developer environment,
cached Cargo dependencies, AMD hardware and the recorded native tooling.
From the repository root in that developer environment:

```powershell
pwsh -NoProfile -File crates/fsr-sdk/tests/native/run.ps1 -Mode sharpening -Native -Stage target/d014-sharpening-20260930-attempt-1
pwsh -NoProfile -File crates/fsr-sdk/tests/native/run.ps1 -Mode sharpening -Native -Stage target/d014-sharpening-20260930-attempt-2
pwsh -NoProfile -File crates/fsr-sdk/tests/native/run.ps1 -Mode sharpening -Native -Stage target/d014-sharpening-20260930-readback-audit
pwsh -NoProfile -File docs/research/records/2026-09-30-exp-d014-native-sharpening/audit-readback.ps1 -Stage target/d014-sharpening-20260930-readback-audit
```

In this session the developer environment was initialized by
`cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\native\run.ps1 -Mode sharpening -Native -Stage <stage-above>'`.
Each GPU child executes the staged test executable with
`--ignored --exact upscaler::m6::native_sharpening_comparisons --nocapture --test-threads=1`.
Compilation occurs outside the per-child watchdog.

| Run | Outcome |
|---|---|
| Sandbox attempt 1 | Runner exit 1: `Get-CimInstance Win32_OperatingSystem` returned access denied after ordinary checks, ABI and helper/executable compilation. No native child started. Retained [check statuses](2026-09-30-exp-d014-native-sharpening/attempt-1/checks.status.txt). |
| Attempt 2, with permitted environment access | 3 children, 80 frames each; all execution checks passed. Each child exited 101 at the final effect assertion; no timeout. Runner exited 1. |
| Readback audit, with permitted environment access | Same 3x80-frame protocol, same execution success and effect failure. Children exited 101, no timeout; runner exited 1. Independent audit exited 0 and matched all displayed Rust metrics to ten decimal places. |

Attempt 2's children took 4.29, 4.08 and 4.11 seconds of test time;
audit children took 4.39, 4.31 and 4.48 seconds. Exact start/end status and
original child stdout/stderr are retained under
[attempt-2](2026-09-30-exp-d014-native-sharpening/attempt-2/sharpening-repeat-1.stderr.txt)
and [readback-audit](2026-09-30-exp-d014-native-sharpening/readback-audit/sharpening-repeat-1.stderr.txt)
for all three repeats. Transient thread IDs were replaced with `<thread-id>`;
no runtime pointer addresses appeared in these retained logs. Provider IDs,
HRESULTs, versions, hardware IDs and hashes are preserved.

Both complete runs passed `cargo fmt --all -- --check`, workspace/all-target
Clippy with warnings denied, workspace tests and no-default-features check
(all locked/offline), paired native ABI compilation, C++ helper compilation
with `/W4 /WX`, and the dx12 test-executable build. These compilation/fixture
results are separate from actual GPU evidence. Existing ignored lifecycle,
motion/jitter/reset and thread-transfer native gates were not rerun by this mode.

## Observations

**Verified execution:** each complete run submitted, fenced and read back 240
outputs on the selected AMD hardware. Across both runs, all 480 outputs had
230,400 finite RGBA pixels and 230,400 pixels changed from the fresh sentinel.
Native dispatch, list close, queue signal, fence and mapped readback succeeded.
Every per-frame and post-context/resource-cleanup checkpoint reported zero
debug errors/corruption and `S_OK` device status. No watchdog or fence timed out.
All 60 contexts were explicitly destroyed after completed work.

**Verified repeat control:** disabled A/B had zero ROI red-bit mismatches and
`N=0` for both scenes in every repetition. Displayed metrics/profile values were
identical across the six repetitions. In the audit run, each scene/setting's
full packed red-image SHA-256 was identical across all three processes;
disabled A/B also had identical full-image fingerprints.

### Frequency scene: failed effect gate

The following values occurred in every repetition of both complete runs.
`G_off=0.0976884703`, so the minimum accepted `G` is `0.0986653550`.

| Enabled setting | D | C | G / G_off | B | Outcome |
|---|---:|---:|---:|---:|---|
| 0 | 0.0022058795 | 0.8322365334 | 1.0068531333 | 0.0002401376 | Diagnostic only |
| 0.35 | 0.0034433764 | 0.8488762200 | 1.0058436764 | 0.0013968057 | **Fail:** gradient increase below 1% |
| 1 | 0.0305979964 | 0.9914191535 | 1.0653898496 | 0.0195217544 | **Fail:** mean shift above 0.005 |

Both nonzero settings meet the difference floor and changed-area requirement.
0.35 meets mean preservation but has only a 0.584% horizontal-gradient increase.
1 meets the gradient criterion but has a 0.01952 absolute mean shift. These
are failures of the specific proposed policy, not native errors. Thresholds
were not relaxed and no passing replacement policy was declared.

### Edge scene: execution and diagnostics

All edge executions passed. These metrics do not define a second effect gate.

| Setting | D | C | G | B |
|---|---:|---:|---:|---:|
| Enabled 0 | 0.0010861201 | 0.2518064940 | 0.0042398606 | 0.0007913059 |
| Enabled 0.35 | 0.0029274483 | 0.4985806119 | 0.0022112184 | 0.0025952632 |
| Enabled 1 | 0.0042345393 | 0.4987272429 | 0.0100947167 | 0.0010167820 |

The diagnostic profile averages 64 bilinear samples along parallel edge normals,
with centres at output coordinates `Y=80..280`, `X=256+0.37*Y`. Each profile
samples signed normal distance -8..8 output pixels in 0.25-pixel steps. The
normal is `(1,-0.37)/sqrt(1+0.37^2)`. Width is the distance between the first
rising interpolated crossings of nominal values 0.18 and 0.82. Overshoot and
undershoot use the full ROI extrema against nominal 0.9 and 0.1, not profile
extrema. Half quantization and digital edge sampling affect these diagnostics.

| Setting | 10–90% width, output pixels | ROI min | ROI max | Dark undershoot | Bright overshoot |
|---|---:|---:|---:|---:|---:|
| Disabled A/B | 1.448454 | 0.0996704102 | 0.8994140625 | 0.0003295898 | 0 |
| Enabled 0 | 1.299406 | 0.0916748047 | 0.9531250000 | 0.0083251953 | 0.0531250000 |
| Enabled 0.35 | 1.221960 | 0.0852661133 | 1.0126953125 | 0.0147338867 | 0.1126953125 |
| Enabled 1 | 0.760511 | 0.0514526367 | 1.7343750000 | 0.0485473633 | 0.8343750000 |

The provider returned finite values above 1 for the enabled edge cases; the
test did not clamp them. These observations must not be described as universal
quality, brightness preservation or strength-monotonicity guarantees.

### Enabled zero and independent check

**Verified:** enabled zero differs from disabled on this provider/configuration
for both scenes, including an observable edge-profile change. The parameter's
zero endpoint therefore cannot be documented as disabling the pass or ensuring
identity output. The pinned native header separately controls pass enablement
with a bool and describes 0..1 as the sharpness range
([ffx_upscale.h at the pinned commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h),
lines 102–103). Its wording does not establish equality between different paths.
This experiment does not identify RCAS internals of the closed signed provider.

To check whether the effect failures came from measurement code, the second
complete run retained tightly packed final-frame red half bits under its target
stage. The independent [.NET audit](../../../tools/verification/archive/2026-10-10/files/docs/research/records/2026-09-30-exp-d014-native-sharpening/audit-readback.ps1.txt)
uses `BitConverter.UInt16BitsToHalf`, rather than the C++ decoder, and separate
f64 loops for all metrics. Its retained
[CSV](2026-09-30-exp-d014-native-sharpening/readback-audit.csv) matches every Rust
D/C/G/B value to ten decimal places, reproduces zero repeat noise/bit mismatches,
and records all 30 image fingerprints. CSV `criteria` mechanically evaluates
the frequency thresholds for each row; it is not an acceptance assertion for
enabled zero, disabled controls or the edge scene. Raw image binaries remain
reproducible local `target` artefacts rather than checked-in research data.

**Inference:** the proposed 1% gradient and 0.005 mean bounds do not jointly
describe this fixture's observed provider response. Shared descriptor inspection,
unchanged scalar mapping, repeat controls and independent half decoding provide
no evidence of an input/mapping or metric bug in this spike. They cannot explain
the provider's internal brightness response or prove the thresholds appropriate
for wider workloads.

## Baseline delta, limits and next question

The source inventory previously established available sharpening fields, but
provided no executed evidence for this signed provider. This spike adds bounded
successful enabled-pass execution, deterministic measurable changes, enabled-zero
observations and a **reproducible failure of the proposed effect gate**.

It establishes neither the proposed public constructors/getters/scalar validation
nor new trait guarantees. It does not test HDR, auto exposure, masks, variable
extents, other cameras, moving scenes, arbitrary providers/hardware, GBV,
cross-thread use, recording ahead, transient allocation capacity or provider
unload. Clean execution is bounded trust evidence, not a general allocator or
GPU safety proof. No general image-quality acceptance follows from the edge
diagnostics or difference metrics.

D014 remains proposed. Before treating its current effect gate as a release
requirement, review the recorded failures and decide what response the gate is
intended to establish. Any new scene or threshold policy requires an explicit
amendment before a new complete validation run; this original gate stays failed.
The native zero observation can support precise parameter documentation without
claiming an identity filter. No additional source-research claim about the
closed provider is needed to preserve these measured facts.

> Archive navigation added 2026-10-10: retired verification scripts are preserved in the
> [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Script links now point to exact archived source bytes;
> recorded commands, observations and fingerprints are unchanged. Historical commands require
> their original layout and compatible inputs; archived sources are not current entry points.

> Orchestration migration (2026-10-10): the shared native, ABI and consumer entry points are now frozen in the [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Current commands live in [verification](../../../tools/verification/README.md); command text and observations below remain historical.

> Documentation compaction (2026-10-29): the original pre-run protocol remains byte-identical, including its historical M6 plan reference. That superseded plan is now in Git history; the [retained milestone summary](../../ROADMAP_HISTORY.md#010--published-2026-09-28) links the original native evidence. Experimental criteria, measurements and failed outcomes are unchanged.
