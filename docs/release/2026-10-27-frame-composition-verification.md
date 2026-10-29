# Portable frame and named resource composition verification

- Date: 2026-10-27.
- Scope: approved U08 under [D042](../adr/D042.md), wrapper 0.3.0 / sys 0.1.2,
  AMD SDK v2.3.0, Windows x64/MSVC + DX12, Rust 1.88+.
- Result: **U08 implemented and verified within the recorded configuration**.
  Fresh confirmations pass 541 source/MSRV/consumer/native cases, 166 format/
  generator cases, 102 motion cases and 15 callback cases. All four profiles
  have unchanged captured inputs and pass independent artefact-integrity checks.

## Implemented API and preserved boundaries

Portable `FrameParameters::new(jitter, time, camera, HistoryAction)` has private
fields and complete getters/setters. It is Debug + Clone without Default/Copy/
equality commitments. Reuse repeats the current values; update jitter/time/camera
and explicitly change Reset to Continue when continuing history. Optional defaults
remain UV motion, disabled sharpening, pre-exposure 1, maximum active regions,
checked Unspecified colour and debug view off. NativeSharpness/Sharpening are now
portable, preserving finite `[0,1]`, negative-zero bits and enabled-zero semantics.

One `MotionVectorScale::{Uv, Pixels, Native(scale)}` replaces units/override
storage. UV uses the active render/display motion extent, Pixels forwards `(1,1)`,
and Native forwards accepted finite bits literally. Replacing a request leaves no
dormant setting; callers explicitly save/restore temporary intent. One private-field
`ColorEncodingRequest` stores the hint and pairing permission. Replacing it with
`checked` restores ordinary pairing, without changing HDR/exposure or pixels.

`ReactiveMaskParameters` has an explicit region and checked native controls,
complete accessors/updates and no GPU borrow. Its region remains independent of
the context's dynamic-resolution setting. All four generator flags keep their
independent native mapping; read-only generator colours may alias.

DX12 `UpscaleResources::builder()` requires named colour/depth/motion/output and
`ExposureInput::Texture` or `Automatic`. Missing exposure cannot silently select
automatic mode. `ReactiveMaskResources::builder()` requires opaque-only colour,
pre-upscale colour and generated-mask output. Both builders return pure
`Error::MissingBinding { role }`, retain no independent COM references and inspect
nothing. Built resource bundles are Debug + Clone, preserve borrow lifetimes and
support the existing optional supplied masks.

Both concrete owners expose unsafe `record(list, &resources, &parameters)` and
`record_reactive_mask`, plus matching safe `validate_upscale_inputs` /
`validate_reactive_mask_inputs`. Recording always re-inspects current inputs after
optional preflight. One borrowed private view feeds the existing inspection and
descriptor mapper, with no second parameter/default store. Eligibility-first
checks, format/view/device/identity/geometry order, native codes/poison and terminal
cleanup remain. GPU state, residency, storage aliasing, ordered submission,
abandonment, completion and native-provider assumptions remain unsafe caller duties.

Old public combined dispatch types, positional resource constructors and competing
motion/colour setters are removed. No raw ABI, SDK/dependency, backend, tuning,
terminal usability or broad error-taxonomy change is introduced. U09–U12 and final
release archives remain separate. Root README and frozen research/release records
continue to describe their original published/baseline APIs.

## Controlled and consumer coverage

Direct public tests exercise required binding errors without COM/native work,
explicit exposure, portable worker preparation, explicit history/time/jitter/camera
updates, literal scale bits, replacement with Pixels, colour pairing restoration,
both-family recording, complete generator controls and post-preflight metadata
mutation. Earlier controlled/native workloads retain test-only input factories;
these assemble production builders/parameters and forward to the new public
methods, with no alternate mapping/defaults. The test equipment is private under
cfg(test); external controls reject the removed API on release consumers.
Native single-frame and typed generator consumers also call the three-input
public operations directly, without that equipment convenience.

The external inventory verifies portable values/traits/private fields, no implicit
frame defaults, non-exhaustive history/motion choices, complete both-family
upscale/generator workflows, unsafe recording boundaries, backend gates, resource
borrow escape rejection and removed old entry points. Complete development package
README/Rustdoc/examples compile as downstream consumers. Root README remains
published 0.2.0 and is not compiled against this breaking development API.

## Equipment and input identity

Windows 11 10.0.26300, AMD Radeon RX 9060 XT adapter 0 (`1002:7590`), driver
32.0.32015.7029, ordinary DX12 validation with GBV disabled. Rust/Cargo 1.99.0,
exact MSRV 1.88.0, Python 3.14.1, MSVC 14.44.35207 and Windows SDK 10.0.22621.0.
SDK headers are pinned at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
Both staged DLL signatures are valid AMD Authenticode signatures; SHA-256:

| Staged file | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The main query/regression children report default provider `4.1.2 *`. Format/
motion/callback supplements explicitly enumerate/select 4.1.1 and 3.1.5; generator
write/parity cases select 3.1.5. Names are observations, not an audit attributing
all internally loaded code to the staged file or a general capability guarantee.

The four passed profiles share combined repository/SDK input identity
`a798b5650119c7129cff055e165fb37d11ce9bd86a26f8f30dd625ed3ba742bb`.
It includes the repository inventory defined by `records.source_identity`, four
SDK headers and both DLLs. The repository-only identity at those runs is
`cee9a5d98890de6558787067fae870da2aa33d0e698a5dc3f6bc08bd333154e5`.

After the matrix, only a portable test function's spelling was corrected from
`motion_and_colour_replace_complete_intent_without_dormant_state` to
`motion_and_color_replace_complete_intent_without_dormant_state`, following the
code-name language policy. No test body, library, example, consumer or native
producer changed. All captured repository entries match the final files except
that explicitly verified label-only change in `crates/fsr-sdk/tests/frame_parameters.rs`.
Its SHA-256 changed from `f0931b3c3ebfa3fe9b5b366a0ff529476988f3b8486687237a058ad470e146c7`
to `f160c767fa9a0fdeafe8a385da507757461f5f5771ef519e8b7cba65917c6117`;
reversing that one identifier replacement reproduces the recorded hash exactly.
The final repository identity is
`6a1c98f8c78f11c80f48a0458d2560916279235022d0f5b5bb6857c30064aaff`.
All three portable tests pass again on current Rust and exact MSRV with no default
features after that correction. The matrix identities describe their captured
inputs, while the final identity includes this scoped test-name correction.

## Reproduction and results

Use existing SDK/toolchains from the repository root; no wrapper download is added.

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/frame-composition-verification
python tools/verification/run.py run native-formats --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/frame-composition-verification
python tools/verification/run.py run native-motion-scale --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/frame-composition-verification
python tools/verification/run.py run native-callbacks --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/frame-composition-verification
python tools/verification/repository_size.py
```

| Profile/check | Result | Boundary |
|---|---|---|
| `windows-upscaler-followup` | 541/541 passed | Seven feature combinations, current tests/Clippy/docs, exact MSRV, external consumers/examples, dependency/Vulkan gates, independent C++ ABI and existing GPU regressions |
| `native-formats` | 166/166 passed | Both public owners and an independent C++ consumer: typed roles, complete generator bindings, exact input/output/parity and fenced cleanup |
| `native-motion-scale` | 102/102 passed | Both owners, selected 4.1.1/3.1.5, render/display RG32F motion, signed/asymmetric scales, explicit replacement/restoration, jitter and active output; wrong-sign controls remain distinct |
| `native-callbacks` | 15/15 passed | Production raw aggregate exchange on current/MSRV; 12 public contexts, 20 fenced frames, debug-view flags and compatible callback cleanup |
| Four `run.py verify-run` checks | Passed | Complete required outcomes and retained artefact hashes |
| Final portable test-name correction | 3/3 on each compiler | `cargo test -p fsr-sdk --no-default-features --test frame_parameters --offline --locked` and the same command with `cargo +1.88.0`; production inputs unchanged |
| Formatting / Markdown links / repository size | Passed | Final Rust formatting, changed relative file targets and checkout budget |

Finalised attempts under `target/frame-composition-verification`:

- `20261027T002601Z-windows-upscaler-followup-43a05b1b653c`.
- `20261027T002600Z-native-formats-abe5b3022337`.
- `20261027T003342Z-native-motion-scale-c8b55fed846f`.
- `20261027T003534Z-native-callbacks-84c11815025e`.

The main native suite also passes its three sharpening integration repetitions
and independent 144-readback/78-comparison audit, without image-quality thresholds.
Callback observations retain 24 paired host and 240 paired resource allocations/
frees, 368 CB allocations and 16 messages. External targets/CB backing remain
process-lived; SDK heap-callback invocation remains unobserved. Legacy null-context
code 2 and live success are preserved, without fallback.

Counts are coordinator cases, not unique unit tests, native calls or an added
quality/performance gate. Profiles share prerequisites. Compiler/controlled
execution alone establishes no AMD/GPU behaviour; native returns alone do not
satisfy the output-write and reference-comparison criteria.

## Preparation failures and limits

A first nested compiler control needed explicit module paths for the newly split
portable modules; that test equipment issue is fixed. The following sandboxed run
passed 129 tests but its nested compiler could not write a temporary metadata file
(OS error 5); the permitted repeat passed. A new fixture assertion initially
counted an intentionally live extra mask/device reference; it now releases that
fixture resource before comparing the original reference counts. No production
lifetime or native retention rule changed to satisfy that assertion.

The first downstream preparation run passed 230/231 cases; the remaining positive
control still called the removed automatic-exposure constructor. It now binds
ExposureInput::Automatic through the production builder. Transient Windows
mapped-file errors during formatting were retried successfully. Failed preparation attempts remain under target. The first consolidated
`20261027T001236Z-windows-upscaler-followup-5090f327df4d` and format run
`20261027T001317Z-native-formats-1893f99d5002` later stopped writing results with
ENOSPC. Their run summaries could not be finalised and may still say running;
they are incomplete, not accepted passes. Only generated compiler caches from
the earlier spike were removed; its source copies and recorded evidence remain.
The maintainer then freed additional disk space. The fresh complete confirmations
above supersede those interrupted attempts without changing production code.

Historical generator write failures and non-linear signal discrepancies are not
reclassified. No general provider/hardware support, arbitrary-thread affinity,
callback quiescence, final DLL unload, performance, new image-quality result or
final archive acceptance follows. U08 preserves the conditional native contracts;
new composition types establish neither GPU readiness nor safe sampling footprints.
