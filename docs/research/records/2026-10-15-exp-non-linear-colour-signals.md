# Experiment: native sRGB/PQ signal domains and exposure controls

- Investigation and source-access date: 2026-10-15.
- Question: do the implemented non-linear controls execute on the pinned signed
  provider, what output domain do declared test signals produce, and can output
  observations distinguish exposure/pre-exposure ordering?
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0 at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`, accepted [D027](../../adr/D027.md).
  Linear HDR and corrected mask integration were already natively verified.
- Exclusions: reconstruction quality, PQ display integration, gamut conversion,
  performance, public API changes and universal provider support.
- Outcome: **native integration verified** for the selected inputs; output is
  consistent with supplied code values. **Fifteen initial signal-hypothesis
  failures remain failures.** Constant-pre-exposure and reset controls remove
  those discrepancies under the same numerical budgets. Complete internal
  transfer/exposure ordering and provider PQ luminance normalization remain
  **unresolved**. A successful integration profile does not accept every numerical
  hypothesis or close M16's complete semantic contract.

## Source basis and declared conventions

The [preceding source investigation](2026-10-15-src-non-linear-colour-contract.md)
already establishes that creation/dispatch mapping is specified but a complete
PQ/exposure contract is not. This experiment rechecked the pinned guide, header
and sample before choosing inputs:

| Primary source | Relevant finding |
|---|---|
| [AMD 4.1.1 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#exposure) | **Upstream claim:** pre-exposure removal precedes exposure multiplication; internal tonemapping is reversed and output returns to the original input domain. No complete per-encoding arithmetic order is supplied. |
| [Non-linear guide section](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#non-linear-color) and [header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h) | Non-linear creation classification plus optional sRGB/PQ input hints. No PQ white-luminance, gamut or normalization field. |
| [Official sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp) | Sample flag selection alone does not establish correctly encoded input, PQ normalization or a reference transformation. The earlier guide/sample mismatch remains unresolved; no mismatched context/hint diagnostic was selected here. |
| [W3C CSS Color 4 conversion equations](https://www.w3.org/TR/css-color-4/#color-conversion-code) | Source for the standard piecewise sRGB transfer used by the test generator and independent inverse. No W3C sample implementation is copied into the repository. |
| [ITU-R BT.2100-2, Table 4, printed pp. 5–6](https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.2100-2-201807-S!!PDF-E.pdf) | PQ inverse EOTF/EOTF and constants, including the 10000 cd/m² scale. The current BT.2100-3 PDF retrieval timed out; the inspected 2018 edition supplies these equations, not a claim to review every later revision. |

The fixture uses eight achromatic relative linear levels:
`0.0001, 0.001, 0.003, 0.01, 0.03, 0.06, 0.12, 0.18`.
Each fills a 40-pixel-wide stripe in 320×180 RGBA16F input; alpha is one.
Only grayscale is selected, avoiding a gamut/primary-conversion inference.
All tested RGB code values remain within `[0,1]`.

For PQ, **one relative linear unit is declared to represent 100 cd/m²**. Thus
PQ input is the inverse EOTF of `100*L` cd/m², normalized by 10000 inside that
equation. This is a test convention, not an AMD requirement or discovered
provider scale. It covers dim-to-moderate values, not the full PQ range.
HDR is enabled in every arm, including sRGB; a separate non-HDR profile was not
tested. No physical display, swapchain or tonemapper is involved.

## Test equipment, code and checks

The worktree was clean at intake. Changes are probe/test code, optional profile
orchestration, CPU negative controls and documentation. Production Rust behaviour,
resource validation, public APIs, ABI, manifests, dependencies and accepted
decisions are unchanged. Existing NumPy is used; nothing was installed.

- Windows 11 build 10.0.26300, x86_64-pc-windows-msvc.
- Rust/Cargo 1.99.0; separate Rust 1.88.0 MSRV checks.
- Python 3.14.1, NumPy 2.5.4, PowerShell 7.6.7.
- MSVC 14.44.35207; Windows SDK 10.0.22621.0.
- Explicit adapter 0: AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
  feature level 12_0; driver `32.0.31041.1004`.
- Provider `4.1.1`, ID `0xf5a5ca1e01001001`.
- Existing local SDK under `target/sdk-windows-verification/v2.3.0`;
  four pinned header hashes, independent C++ ABI assertions, both DLL hashes and
  Valid AMD Authenticode signatures pass. No SDK download or new strict online
  revocation check was requested.

The [base protocol](../../../tools/verification/colour-signal-protocol.json)
was written before native execution. Python generates exact little-endian half
bytes using its scalar transfer functions. A separately implemented Decimal
inverse at precision 45 independently recovers the declared relative linear
values, allowing 1.2% relative or `1e-7` absolute for input half quantization.
Known standard anchors, wrong encoding/normalization, corrupted patches, wrong
output domains, truncation, sentinels and nonfinite values have CPU controls.
The inverse does not reuse the generator implementation or GPU helper conversion.

The [C++ upload helper](../../../crates/fsr-sdk/tests/native/colour_signal.h)
only copies those RGB bytes and supplies zero motion, constant depth and declared
exposure. It contains no transfer calculation or AMD call. It uses the existing
four-input readback/check path; the historical `HDR input readback/guards` log
label denotes that equipment path even for encoded signals. Full input texture
bytes are compared with uploads after fencing, including depth/motion/exposure.

The [Rust probe](../../../crates/fsr-sdk/tests/native/colour_signal.rs) uses the
ordinary public local owner, matching non-linear classification/hints, and no
override. Fresh command lists/allocators/textures are submitted once and fenced;
all three frames' resources survive explicit context destruction. Unknown
submission/fence completion exits the isolated process without unwinding owners
into potentially in-flight work. Debug-layer errors and device removal are
required to remain zero through teardown. GBV is not enabled.

Three jitters `(0.25,-0.25), (-0.25,0.25), (0.125,-0.125)` accompany the stationary
patches. Their integer stripe boundaries have unchanged pixel-centre membership
under these offsets. Output is 640×360, with zero motion and 16.667 ms frame time.
The independent analyser checks every output channel for finiteness and all RGB
pixels for unchanged sentinel values. It measures RGB patch means/minima/maxima
inside a 16-output-pixel margin, excluding boundaries from signal hypotheses.

The fixed prediction budget is `max(0.003, 0.03*abs(expected_code))` for each
patch/channel mean. It is an exploratory discriminator above half quantization,
not a calibrated reconstruction-quality budget or universal correctness gate.
It was not relaxed after observing failures. Candidate predictions include
supplied codes, decoded relative linear, code/pre-exposure, decoded/pre-exposure,
code×exposure, and for PQ decoded nits or normalized 10000-nit linear values.
These candidates test observable domains; they are not assumed to exhaust every
possible provider transformation.

## Initial experiment: 48 processes and 144 frames

Five neutral arms precede exposure probes: linear, sRGB without/with hint, and
PQ without/with hint. Each uses explicit exposure and pre-exposure one. Remaining
arms use matching hints and test `P=0.5 → 1 → 2` under two declared packings:
`E(P*L)` and `P*E(L)`. Linear needs only one, since they coincide. Separate arms
vary explicit exposure `0.25 → 1 → 4` at fixed input/pre-exposure, or enable
automatic exposure. Each context runs three continuous frames with reset only
on its first frame. All 16 arms run in three fresh process repetitions.

**Verified native integration:** all 60 profile cases pass: equipment, ABI,
signed deployment, independent CPU controls, helper/test builds, 48 native
processes and offline result assembly. All 144 frames complete exact input
readbacks, native dispatch, submission, fencing, finite/written output and clean
diagnostics/explicit teardown. This is deliberately separate from numerical
signal-hypothesis outcomes.

| Arm group | Maximum observed same-code patch-mean error | Frozen same-code prediction |
|---|---:|---|
| Neutral linear | 0.000237744 | Matches in all frames/repetitions |
| Neutral sRGB, matching hint | 0.000359962 | Matches; decoded-linear candidate fails |
| Neutral PQ, matching hint | 0.000247087 | Matches; decoded-relative-linear/nits/10000-nit candidates fail |
| Neutral encoded inputs, unspecified hint | sRGB 0.000192751; PQ 0.000157891 | Matches in all frames/repetitions |
| Linear, changing pre-exposure | 0.000598907 | Matches |
| sRGB `P*E(L)`, changing P | 0.002985158 | Matches |
| sRGB `E(P*L)`, changing P | 0.003003919 | Frame 2 fails in each repetition; marginally exceeds the 0.003 absolute budget at one dark patch |
| PQ `E(P*L)`, changing P | 0.008828101 | Frames 1 and 2 fail in each repetition |
| PQ `P*E(L)`, changing P | 0.009876103 | Frames 1 and 2 fail in each repetition |
| Explicit exposure variation / automatic exposure | ≤0.000573283 | Matches supplied-code domain in all selected frames |

There are **15 failed frame predictions**: five distinct arm/frame combinations,
each reproduced three times. All three repetitions of every arm produce identical
complete output-hash triplets. The sRGB/PQ neutral, explicit-exposure and automatic
arms also have identical output triplets for each encoding; linear does not.
Output invariance does not establish that exposure was ignored: the upstream
contract reverses internal tonemapping, and other internal effects may be
unobservable for these flat signals.

Matching hints versus unspecified hints produce small reproducible differences:
maximum corresponding patch-mean difference `0.000191138` for sRGB and
`0.000133514` for PQ. This is evidence of a bounded output difference under the
changed flag, not proof of a complete standards-correct internal transfer path.

## Follow-up: static pre-exposure and reset controls

The discrepancies motivated a separately declared
[control protocol](../../../tools/verification/colour-signal-controls-protocol.json)
before further GPU execution. Original input levels, encoders, budgets and failed
verdicts remain unchanged. Ten static arms hold P at 0.5 or 2 for linear and both
non-linear packings. Four arms repeat the original non-linear P sequence with
reset on every frame. Three fresh processes per arm give 42 processes/126 frames.

All 54 integration cases pass. **All 126 same-code frame predictions match**, and
each arm's repeated complete output triplets are byte-identical. Maximum
same-code error across static controls is `0.00126763`; across every-frame-reset
controls it is `0.000488281`. The original failed transition predictions are not
reclassified by these different controls.

**Inference:** static/neutral acceptance plus the reset intervention supports
dependence on continuous temporal state during the selected parameter/input
transitions. It weakens an explanation based solely on the selected static input
range or inability to process sRGB/PQ codes. It does not identify the internal
decode/pre-exposure order, prove either packing universally correct, establish
that the discrepancy is a wrapper defect, or justify an automatic reset policy.
The signalled metadata and pixel content are independently captured, but no
standalone raw C++ provider implementation comparison was performed here.

## Reproduction, regression verification and evidence

With existing NumPy, PowerShell 7, x64 MSVC, cached Cargo inputs and the local SDK:

```powershell
python -m unittest discover -s tools/verification/tests -p test_colour_signal.py -v
python tools/verification/run.py run experiment-colour-signal --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-colour-signal-controls --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py verify-run --run target/verification/<fresh-attempt>
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
```

The original run used the initial first-frame-reset-only probe. The follow-up
adds an explicit reset sequence and control profile without changing its
numerical predictions. Current base-profile defaults reproduce the same reset
sequence and matrix. Input inventories identify each actual source state;
reproduction does not promise cross-driver bit identity.

Each native profile's `passed` status requires integration and complete numerical
evaluation records. Numerical predictions are explicitly observations: it is
incorrect to report these run statuses as every signal-semantic test passing.
No standard or release profile acquires these exploratory numerical criteria.

| Finalized check | Required cases | Evidence |
|---|---:|---|
| Initial signal experiment | 60 integration/evaluation cases pass; 15 frame predictions fail | [Run](2026-10-15-exp-non-linear-colour-signals/initial/run.json), [arm outcomes](2026-10-15-exp-non-linear-colour-signals/initial/arm-summary.json) |
| Static/reset controls | 54 integration/evaluation cases pass; no same-code frame prediction fails | [Run](2026-10-15-exp-non-linear-colour-signals/controls/run.json), [arm outcomes](2026-10-15-exp-non-linear-colour-signals/controls/arm-summary.json) |
| Final Windows CI | 115 pass | [Run](2026-10-15-exp-non-linear-colour-signals/ci/run.json) |
| Final Rust 1.88 MSRV | 98 pass | [Run](2026-10-15-exp-non-linear-colour-signals/msrv/run.json) |

Final CI includes all four feature configurations, tests, Clippy, documentation,
public consumer/example/dependency checks and four independent Windows C++ ABI
probes. Its [infrastructure suite](2026-10-15-exp-non-linear-colour-signals/ci/infrastructure.tests.txt)
runs 191 tests: 187 pass, four Linux/Proton or privileged-symlink controls skip.
All seven colour-oracle/negative-control tests and actual PowerShell parser tests
execute. The [Rust movable log](2026-10-15-exp-non-linear-colour-signals/ci/rust.movable.test.txt)
retains Windows controlled colour forwarding and ordinary source regressions.
The new ignored native probe is compiled there; its actual GPU execution is
established by the two separate experiment profiles.

Initial CI/MSRV also passed before adding reset controls. Every initial/final
attempt passes `verify-run` and records unchanged source/configuration/SDK inputs.
[Summary, attempt names and integrity results](2026-10-15-exp-non-linear-colour-signals/summary.json).
CI/MSRV use the previously authorized outside-sandbox environment because prior
ownership fixtures failed to write temporary compiler files in the sandbox.
Native experiments complete inside the normal environment; no failed compiler
or native attempt is hidden in this experiment. After finalization only record
and navigation prose was added; executable probe/protocol/test bytes remain
unchanged.

Permanent evidence keeps each protocol, inputs/commands/results and one complete
patch-statistics measurement per arm; all three repetitions' output hash triplets,
errors and failed predictions are retained separately in arm summaries. This
deduplication is justified by verified exact repetition, not assumed equivalence.
For example, compare the [failed continuous PQ measurement](2026-10-15-exp-non-linear-colour-signals/initial/measurements/pre-pq-encoded-after-encode.json)
with its [reset control](2026-10-15-exp-non-linear-colour-signals/controls/measurements/reset-pq-encoded-after-encode.json).
Selected [neutral PQ](2026-10-15-exp-non-linear-colour-signals/initial/colour.neutral-pq.r1.txt),
[continuous PQ](2026-10-15-exp-non-linear-colour-signals/initial/colour.pre-pq-encoded-after-encode.r1.txt)
and [reset PQ](2026-10-15-exp-non-linear-colour-signals/controls/colour.reset-pq-encoded-after-encode.r1.txt)
logs preserve adapter/driver/provider identity, actual metadata, fenced input
readbacks and clean teardown.

The [inventory](2026-10-15-exp-non-linear-colour-signals/artifacts.json) hashes
normalized UTF-8/LF retained bytes. Private paths and transient pointer-labelled
addresses are normalized; provider IDs, native codes and hashes are preserved.
Full raw input/output images, SDK binaries and build products remain ignored
local diagnostics under `target/`, not repository payloads. The compact record
supports the reported arithmetic and comparisons; complete pixel reanalysis
requires those local attempts or a fresh GPU run. Historical output bytes are
not promised across machines or drivers.

## Baseline delta and remaining questions

There is now native evidence that the selected linear/sRGB/PQ controls execute
with independently audited encoded inputs and preserve their supplied code domain
for neutral and static cases. This adds runtime evidence beyond the earlier
controlled flag-mapping fixtures. The provider is not a requested output colour
converter, and no conversion or normalization was added to production Rust.

The exact internal order remains underdetermined: reversible operations can
produce the same restored output. Preserved code values cannot establish the
provider's absolute PQ luminance scale, reference white or gamut handling. Both
packing families can pass static/reset cases, so the experiment does not choose
one as a public dynamic-pre-exposure contract. Further clarification would need
a provider contract or a selected protocol with a discriminating observable,
not a looser budget or a guessed colour converter. M16's broader non-linear
semantics remain open; this record does not accept a new design or release gate.

Coverage is one GPU/driver/provider, achromatic inputs, HDR-enabled RGBA16F,
three-frame local-owner contexts and the declared moderate code range. It does
not establish coloured-primary behaviour, an HDR display pipeline, long histories,
movable non-linear GPU dispatch, arbitrary scale/brightness/format combinations,
universal native safety or reconstruction quality. Topic/roadmap integration
is separate from preserving this experiment and its failed hypotheses.
