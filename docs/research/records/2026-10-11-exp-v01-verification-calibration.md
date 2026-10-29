# Experiment: temporal verification calibration through the public Upscaler

Investigation and execution date: **11 October 2026**. The question was whether
the existing public Rust Upscaler, DX12 equipment and independent Python
analysis can generate valid longer synthetic sequences, retain complete images,
and distinguish motion, jitter, disocclusion and reset behaviour. This is
calibration evidence and a proposed future V01 protocol, **not V01 acceptance**.

**Observed:** the final attempt completed three independent 301-frame GPU
children, retained all 903 RGBA16F outputs and their inputs, and passed structural
checks. All repeats were bit-identical on this deployment; history/reset and
fresh/reset matched exactly. Motion and jitter controls affected the output, but
their rankings depended on the measurement. Several otherwise attractive
disocclusion/leakage comparisons favoured the deliberately wrong inputs.

## Retention update — 11 October 2026

After the original execution, the maintainer selected compact evidence
retention. Full generated image/input payloads, verbose frame analyses, repeated
source snapshots and full log exports were removed from the repository after
their inventory was validated. The new compact pooling exactly matched the
recorded arm and control measurements. Results, pass/fail outcomes and the
limits below are unchanged. The [compact evidence index](2026-10-11-exp-v01-verification-calibration/README.md)
describes the retained files and their identities.

Statements below about full capture and original export describe the historical
execution. Current navigation points to compact measurements/outcomes. Full
offline reanalysis of those original images is no longer available; a new GPU
run uses the retained generator/protocol and can differ with equipment/software.

## Baseline, scope and source state

The starting worktree was clean at
`bd1eeb837ab3450f3d532c50d55f37c8f81fea6e`. The supplied
[scene/reference source record](2026-10-10-src-v01-scene-reference-contract.md)
was treated as mathematical proposals and upstream interpretation, not executed
evidence. The inspected baseline contains stationary single-frame/temporal
smokes, deliberately perturbed semantic comparisons, a public Upscaler dispatch,
and DX12 upload/submission/readback equipment. The existing temporal smoke is
eight stationary frames; existing red-channel comparisons and common depth do
not themselves demonstrate longer geometric disocclusion or RGB reconstruction.
Their lifecycle, single-frame, temporal and semantics prerequisites passed again
in the final attempt. Historical sharpening gates were not re-evaluated.

The changes add an explicitly selected experiment, two analytic scenes,
input readbacks, full output retention and independent analysis. Rust calls the
existing public wrapper; C++ owns DX12 resources; Python owns the case graph,
protocol, input oracle, decoding and diagnostics. There are no SDK/public API
changes or new runtime dependencies. NumPy was already available in the chosen
Python environment; ordinary profiles remain standard-library tools. No package
was installed. `ci`, `release` and `native-upscaler` exclude this experiment.

The final attempt's combined source/SDK input identity is
`3c2739d77d78d502deffcc28669c83579dd248e04c7b8fd438b0daf729e57473`.
[Its input inventory](2026-10-11-exp-v01-verification-calibration/inputs.json)
retains individual SHA256 identities. The original capture included snapshots
of workspace manifests/lockfile, crate sources/tests, helpers and verification
tools for every attempt. The first snapshot filter omitted crate licence texts;
unchanged supplemental licences were added. Those repeated snapshots/licence
copies were removed under the dated retention update. The ordinary source tree
retains its MPL-2.0 and AMD-derived notices and root/crate licence texts. The
[compact evidence index](2026-10-11-exp-v01-verification-calibration/README.md)
states the current reconstruction limits.

After GPU execution, the user requested responsibility-based names under
`CONTRIBUTING.md`. Current entry points are
[temporal_calibration.py](../../../tools/verification/temporal_calibration.py),
[temporal-calibration-protocol.json](../../../tools/verification/temporal-calibration-protocol.json),
[temporal_calibration.rs](../../../crates/fsr-sdk/tests/native/temporal_calibration.rs),
[temporal_calibration.h](../../../crates/fsr-sdk/tests/native/temporal_calibration.h)
and [test_temporal_calibration.py](../../../tools/verification/tests/test_temporal_calibration.py).
The profile is `experiment-temporal-calibration`, with tests
`native_temporal_input_only` and `native_temporal_calibration`. Includes, imports,
export lookups, environment variables, selectors, log parsers and documentation
were renamed together. Captured snapshots and attempt identifiers retain `v01`.
The [rename input check outcomes](2026-10-11-exp-v01-verification-calibration/checks.json)
record the successful check of the rebuilt helper/test exports
with 21 DX12 input-only frames, without AMD upscaling. The renamed analyzer also
revalidated the historical input-only bytes and regenerated a byte-identical
contact sheet using the explicit historical namespace. No new image-quality
claim is attached to this naming check.
Subsequent spelling cleanup uses Oxford `-ize` forms, including
`summarize_calibration.py`, `quantization` and `normalize`. Captured evidence
keeps its original spelling; these editorial changes do not change the scene
or measurements. The summarizer pools recorded frame measurements within this
experiment. No research synthesis or integration has taken place.

## Equipment and identified native inputs

| Item | Actually observed |
|---|---|
| Platform | Windows 11, build `10.0.26300`, x64/MSVC |
| Rust / Cargo | `1.99.0`; Rust commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`; LLVM `23.1.1`; host `x86_64-pc-windows-msvc` |
| Native compiler | MSVC `14.44.35207`; Windows SDK `10.0.22621.0` |
| Analysis | Python `3.12.14`, NumPy `2.3.5` in an existing bundled runtime |
| GPU | AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, hardware adapter 0, feature level `12_0` |
| Driver | Successful driver query, `32.0.31041.1004` |
| SDK | v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; four pinned headers hash-checked; three independent C++ ABI probes compiled |
| Selected provider | Query succeeded, ID `0xf5a5ca1e01001001`, reported name `4.1.1` |

SDK release, provider name and crate versions are distinct. The local SDK root
was `external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX`. Both staged DLL signatures
were `Valid`, signed by Advanced Micro Devices. The actually loaded module
paths were observed while the contexts were live and checked against staged
files by filesystem identity and hash, rather than inferred from configuration.
The loader returned an extended Windows path prefix, which matters to attempt 3
below.

| Loaded file | SHA256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |
| Tested `m6_gpu.dll` helper | `6426afd3e4f7b0d5f15ffff9e50fca9181884bdd9d9c44396dd91dae88d302ca` |
| Staged Rust test executable | `f2833abe7e13d32ec8a8b6da373579855f63fd5dea7590d9b6899d7693b491ce` |

The helper keeps its existing basename. AMD DLLs, test executables, objects and
the complete developer-shell environment are excluded from retained exports.
Reproduction requires the same trusted local SDK deployment; these hashes do
not supply the binaries. Selected equipment, module identities, commands and
native logs remain in the case records.

## Frozen protocol and independent input validation

The [captured protocol](2026-10-11-exp-v01-verification-calibration/protocol.json)
was frozen before the calibration dispatches. Its SHA256 is
`25663486919c47ab92e427a2572ba8eb52b5e5e8b58daa6a82e5ff05ce12c431`.
The scene, controls, dimensions, masks, measurements and frame counts were not
changed after inspecting results. Repairs between attempts affected evidence
plumbing, described below. There was no speed, scene or threshold search.

The render size is 320×180 and output 640×360. Jitter is Halton bases 2 and 3,
indices 1..32, centred by subtracting 0.5 and rounded to binary32. The saved
[JSON table](2026-10-11-exp-v01-verification-calibration/jitter.json)
and [exact decimal input](2026-10-11-exp-v01-verification-calibration/jitter.txt)
are retained. Phase is `frame % 32`; object motion continues across phases 31→0.

The camera is left-handed, looking along +Z, with world Y up and top-down UV
axes (+X right, +Y down). Near is 0.5, far 20, vertical FOV π/3 and metres factor
1. Perspective device depth is
`d(z)=20/19.5 - 10/(19.5*z)`: near→0, far→1, foreground z=2 and background z=8.
The actual binary32 depths are `0.7692307829856873` and `0.9615384340286255`.
Dispatch FOV is the binary32 value `1.0471975803375244`; analytic geometry uses
π/3. This tiny projection rounding difference is below approximately 0.000005
render pixels over this scene, rather than exact equality of real-valued camera
parameters.

An input pixel samples `(x+0.5-jx,y+0.5-jy)`. This inverse sample displacement
corresponds to projected geometry moving by positive image jitter; an independent
projection check verifies that sign. Colour, depth and motion use one half-open
visibility decision. Scene A is an affine colour plane at z=8:
`RGB=(0.125+sx/1024, 0.25+sy/1024, 0.375+(sx+sy)/2048)`, with zero motion.
Scene B is an opaque 64×44 render-pixel rectangle initially at (100,50), moving
by (+0.375,+0.25) render pixels per frame at z=2. Its RGB is
`(0.875,0.125,0.0625)` over background `(0.125,0.25,0.375)` at z=8. It stays in
the viewport throughout both cycles and the cut sequence.

Motion is **unjittered previous UV minus current UV** for the current visible
surface. The foreground uses `(-0.375/320,-0.25/180)` after frame 0; the static
background uses `(0,0)`, including newly uncovered background. Frame 0 motion
is zero. Render-pixel scale is `(320,180)`. Colour/motion scene arithmetic goes
through binary32 and the existing helper's normal-half mantissa truncation
towards zero. This is not round-to-nearest-even. The stored foreground motion
corresponds to approximately `(-0.374755859375,-0.24993896484375)` render pixels.
Input subnormals are excluded. Depth and exposure use binary32.

| Arm | Absolute frames | Change from correct inputs |
|---|---|---|
| A correct | 0..63 | Stationary plane |
| B correct | 0..63 | Moving rectangle |
| B reverse motion | 0..63 | Negate foreground motion only |
| B negated jitter | 0..63 | Negate dispatch jitter; image samples stay unchanged |
| Cut history | 0..40 | Build history; camera cut and reset at frame 37 |
| Cut fresh | 37..40 | New context, identical current inputs and absolute frame/jitter identity; reset at 37 |

For the cut, focal length is `90/tan(π/6)`. At frame 37 the camera translates
in world XY by `(192/focal,96/focal)` and then stays fixed. Foreground projected
shift is (-96,+48), background (-24,+12) render pixels. Cut-frame foreground
motion is `((96-0.375)/320,(-48-0.25)/180)` and background `(24/320,-12/180)`.
Object motion and jitter do not restart. Reset is true at frame 0 and, in both
cut arms, frame 37. No history-without-reset cut arm was run.

Creation/dispatch flags are 0; linear non-HDR `R16G16B16A16_FLOAT` colour and
output, `R32_FLOAT` depth, `R16G16_FLOAT` motion and explicit 1×1 `R32_FLOAT`
exposure 1.0. Pre-exposure is 1.0, frame time binary32 `1000/60` ms, no masks,
sharpening disabled. The input-only child does not load or call AMD's runtime:
it uploads and copies A, B and cut inputs at frames 0,1,31,32,37,40,63 (21 frames).
The independent Python oracle reconstructs perspective geometry, visibility,
quantization and jitter, compares **every byte**, and retains representative
interior/silhouette decoded samples. This succeeded before AMD prerequisites
and was repeated for all 903 full-matrix input payloads. Thus the evidence is
actual DX12 input readback, not just agreement between CPU generators.

## Execution bounds, artefacts and failures

There are 301 output frames per full child (4×64 + 41 + 4), three independent
process repeats. Every output is 1,843,200 tightly packed RGBA16F bytes. Every
input payload is 921,604 bytes in colour/depth/motion/exposure order: 460,800 +
230,400 + 230,400 + 4. All frame identities, dispatch parameters, debug/device
results and finish/fence observations are retained in `frames.jsonl`.

Each frame uses the existing helper's command list, queue, allocator and resource
ownership. Only one frame is submitted at a time and its CPU-observed fence
completes before the next frame. Input copies precede dispatch and restore
declared input states. Every frame's equipment remains retained until context
destruction; resources are then released. Failed/uncertain submission, fence,
device or context destruction takes the isolated fail-fast process path instead
of forcing normal GPU-dependent cleanup. The per-fence bound is 60,000 ms;
input-only child watchdog 180 seconds, matrix child 240 seconds, offline audit
1,200 seconds. The existing process watchdog terminates descendants. Its
infrastructure timeout/failure fixtures passed, but no AMD timeout or uncertain
completion was induced here. Successful observations do not establish safe
recovery from every driver failure.

Raw files are deterministically gzip-compressed at level 6 with mtime 0 and
round-trip checked before removing the uncompressed copy from that fresh
attempt. `payloads.json` retains compressed and decoded sizes/SHA256 identities.
Failed children also package available bytes. Each complete attempt retains
1,827 payload files: 903 outputs, 903 inputs and 21 prerequisite inputs. Its
decoded payload size is 2,515,971,696 bytes; compressed size 284,745,467 bytes.
All failed attempts with available data are retained, not replaced by hashes.

| Attempt (UTC directory prefix and suffix) | Recorded outcome | What actually ran |
|---|---|---|
| `022157Z…b5aece19bb76` / [attempt 1](2026-10-11-exp-v01-verification-calibration/checks.json) | Failed | 21 prerequisite input frames and all three 301-frame children completed; offline audit failed creating its missing parent directory |
| `022600Z…5fda386a4d08` / [attempt 2](2026-10-11-exp-v01-verification-calibration/checks.json) | Failed | Compilation/deployment prerequisites passed; Python closure shadowing caused `UnboundLocalError` before input child execution; no calibration GPU frames; dependent cases blocked |
| `022652Z…e24cf68f8c33` / [attempt 3](2026-10-11-exp-v01-verification-calibration/checks.json) | Failed | Input prerequisite/baselines and all 903 outputs completed; each full child evaluation rejected textual loaded-DLL path comparison (`\\?\` prefix); offline audit blocked |
| `022933Z…4dc0d5597e8e` / [attempt 4](2026-10-11-exp-v01-verification-calibration/checks.json) | Passed structural profile | All 20 required cases completed/passed; source inputs unchanged; full independent audit completed |

All directory dates are `20261011`; full names, identities and times are in
[attempts.json](2026-10-11-exp-v01-verification-calibration/attempts.json).
Repairs were parent creation, a local parser-variable rename and filesystem
identity comparison for the extended path. No scene/metric adjustment followed
failed evaluations. Re-running the profile after these instrumentation failures
caused three complete matrices to be executed in total, rather than only the
intended final triple of children. The extra complete failed-attempt matrices
are disclosed and preserved. Their decoded payload manifest identities match
the final attempt; this comparison supplements the retained raw files.

Attempt 1 was separately reanalysed using the corrected script captured with
attempt 3, taking 301.328 seconds. The
[reanalysis and provenance](2026-10-11-exp-v01-verification-calibration/reanalysis-provenance.json)
are separate; attempt 1 remains failed. The primary attempt ran
02:29:33.857718–02:36:48.979784 UTC (04:29–04:36 Europe/Berlin); its independent
audit took 286.391 seconds. Early preflight compilation caught ownership of
explicit context destruction; two hand-calculated oracle test expectations
(half-motion and affine reprojection) were corrected before GPU calibration.
These were development failures, not GPU quality results.

## Independent measurements

Python's IEEE binary16 decoder checks all four output channels for finiteness
and any remaining `0x7bff` sentinel. RGB measurements use float64 decoded values,
without clipping and without importing the native red-channel image metrics.
Analytic tests cover perspective depth endpoints, continuous motion, jitter
projection direction, half quantization/decoding, exact silhouette coverage,
affine-field reprojection, incomplete/sentinel payload rejection and empty masks.

The unjittered high-resolution point reference samples
`((X+0.5)/2,(Y+0.5)/2)` independently of the low-resolution generator. Scene B's
area reference uses exact rectangle overlap with a display pixel's 0.5×0.5
render-pixel footprint, blending the two constant colours. These are two
measurement targets, not promised FSR outputs. Scene A is affine, so point and
area references coincide. Output quantization and input truncation remain in
the measured error; no native metric or interpolated low-resolution image is
used as ground truth.

All masks exclude the outer eight display pixels. Interiors are farther than
two display pixels from the analytic silhouette, edge pixels at most two.
Permanent background is the intersection of safe background masks over the
whole sequence. Disocclusion age tracks a point-visible FG→BG transition:
age 0, 1, 2..5, split into distances ≤2, (2,8] and >8 display pixels. Age history
restarts at the camera cut. Stationary-background temporal residual is
`e_t-e_(t-1)` over mutually visible safe background. Foreground residual compares
current error with bilinearly reprojected previous error using true flow
(-0.75,-0.5) display pixels; all four previous taps must be foreground interior.
The cut-frame temporal pair is excluded. A also compares frame t with t−32.

For pure background farther than two pixels from the silhouette, directed
foreground leakage is
`g=((U-CB)·(CF-CB))/||CF-CB||²`, with contrast `(0.75,-0.125,-0.3125)`.
This is a dimensionless projection, not proof of a particular provider history
mechanism. Per-frame files retain MAE, RMSE, p95 absolute error, maximum and
signed mean, plus fixed-coordinate traces. Empty regions have count zero and
null metrics. Summary MAE/MSE are pooled with pixel counts, not averaged as
equal-weight frames; counts are pixel-frames, not independent statistical trials.

The tables below use **repeat 1, frames 32..63**, linear RGB units, unless stated.
Repeats 2/3 had identical outputs. Compact measurements are retained under
[primary analysis](2026-10-11-exp-v01-verification-calibration/summary.json).
The original independent frame values were pooled into
[diagnostic-summary.json](2026-10-11-exp-v01-verification-calibration/diagnostic-summary.json).
The current equivalent [pooling implementation](../../../tools/verification/temporal_calibration.py)
was checked against those recorded measurements before bulk removal.

### Spatial and temporal observations

| Scene/region, point reference | Pixel-frames | RGB MAE | RGB RMSE |
|---|---:|---:|---:|
| A interior | 6,868,992 | 0.00109215 | 0.00120022 |
| B correct interior | 6,808,448 | 0.000621367 | 0.00191571 |
| B correct foreground interior | 330,656 | 0.00200124 | 0.00433145 |
| B correct background interior | 6,477,792 | 0.000550932 | 0.00170283 |
| B correct permanent background | 6,210,048 | 0.000481009 | 0.000585708 |
| B correct edge | 60,544 | 0.0593627 | 0.119482 |

A background temporal residual MAE is 0.0000817336 over 6,868,992 pixel-pairs.
A's t versus t−32 pooled MAE is 0.000182341 over 6,868,992 pixel-pairs; frame
32 versus 0 MAE is 0.000864857 (maximum 0.00341797), frame 63 versus 31 MAE
0.0000899428 (maximum 0.00219727). Phase equality therefore does not imply
output equality after different history lengths. Two cycles do not establish
convergence, and the smooth affine plane is a weak textured-scene challenge.

| B measurement, same window/masks | Correct | Reverse motion | Negated dispatch jitter |
|---|---:|---:|---:|
| Interior point MAE, 6,808,448 pixel-frames | 0.000621367 | 0.000530946 | 0.000532336 |
| Edge point MAE, 60,544 pixel-frames | 0.0593627 | 0.0611184 | 0.0975221 |
| Edge area MAE, 60,544 pixel-frames | 0.0489006 | 0.0513651 | 0.0882337 |
| Foreground temporal residual MAE, 324,056 pixel-pairs | 0.000299944 | 0.000640252 | 0.000415732 |
| Background temporal residual MAE, 6,473,520 pixel-pairs | 0.0000597758 | 0.0000466317 | 0.0000574862 |

**Inference:** foreground temporal residual is locally sensitive to reversed
motion (approximately 2.13× correct), and area-reference silhouette error to
negated jitter (approximately 1.80×). Interior error and background temporal
error do not give a universal correct-better-than-wrong ranking. Paired output
differences, per-frame trajectories and p95/maxima are retained to make the
effects inspectable rather than relying on one aggregate scalar.

### Disocclusion and leakage observations

| Background age/region | Pixel-frames | Correct point MAE | Reverse point MAE | Negated-jitter point MAE |
|---|---:|---:|---:|---:|
| Age 0 | 4,152 | 0.0903583 | 0.122029 | 0.149227 |
| Age 1 | 4,152 | 0.0575688 | 0.0261342 | 0.0444603 |
| Age 2..5 | 16,608 | 0.0265459 | 0.00136238 | 0.0207516 |
| Age 2..5, distance (2,8] | 11,192 | 0.0160826 | 0.000666911 | 0.00953215 |
| Age 0 or 1 outside the two-pixel edge band | 0 | null | null | null |
| Age 2..5, distance >8 | 0 | null | null | null |

Age-0 area-reference MAEs are 0.0884156, 0.108128 and 0.143321 respectively;
mixed silhouette coverage explains why this is a distinct measurement.
For pure background at age 2..5 outside the edge band (11,192 pixel-frames),
signed mean leakage is **0.0276800 correct, 0.000756630 reverse motion,
0.0164471 negated jitter**. Absolute leakage means are 0.0276891, 0.000817824
and 0.0164511. These wrong controls improve this leakage measurement, strongly
opposing its use as a generic correctness gate for the present scene. At the
chosen slow speed, young uncovered pixels lie entirely in the excluded mixed
edge band; the pure-background age-0/1 measurement is absent, not perfect.
No faster variant was added to remove this negative calibration outcome.

### Reset and repetition

For frames 37..40, every history/reset versus fresh/reset pair had identical
current input bytes and parameters, and **identical full RGBA output bytes**.
There are four pairs per repeat, twelve overall; all RGB region differences
and RGBA maxima are zero. All 301 outputs per repeat are bit-identical between
repeat 1 and repeats 2/3, including incorrect-input controls.

This is a useful local reset/reproducibility observation, not an SDK guarantee.
Without a history/no-reset cut control it does not prove that reset caused the
match, or identify how this provider discards history. The four post-cut frames
and two-colour scene limit sensitivity. Driver/provider/hardware changes need
new evidence; bit identity across unrelated deployments is not established.

The [contact sheet](2026-10-11-exp-v01-verification-calibration/contact-sheet.png)
shows, top row: A, B correct, reverse motion, negated jitter at frame 63; bottom:
history cut, fresh cut at frame 37, absolute reset difference ×16, B area-reference
absolute error ×4. PNG display values are clipped; numerical analysis is not.

## Runtime, memory and structural evidence

| Final native child | Wall seconds | Peak working set bytes | Maximum sampled private bytes | Peak sampled local GPU usage bytes |
|---|---:|---:|---:|---:|
| Repeat 1 | 13.328 | 599,957,504 | 933,830,656 | 367,353,856 |
| Repeat 2 | 13.265 | 604,270,592 | 935,350,272 | 367,353,856 |
| Repeat 3 | 13.406 | 604,110,848 | 933,769,216 | 367,353,856 |

Each child retains 832,206,004 decoded payload bytes, compressed to 94,443,152
bytes. Compression/byte auditing happen in the coordinator after child exit;
these child times are not whole-profile times or isolated GPU timings. Offline
analysis is more expensive than the child execution on this setup and streams
frame pairs rather than keeping the whole image corpus in RAM.

For each full child the native logs contain 301 begin, close-success,
submission, signal-success, fence-success and mapped-readback observations,
plus six completed context/equipment cleanups. All 903 frame metadata entries
report successful dispatch/finish, fence exactly 1, zero debug/device errors,
230,400 finite and written pixels. Python independently checks every RGBA
channel for finite/written output. There were no timed-out, aborted or
unconfirmed GPU completions in the executed matrices. The
[summarized cross-check](2026-10-11-exp-v01-verification-calibration/diagnostic-summary.json)
asserts these counts directly from the retained logs and metadata.

After every arm's context/frame cleanup, sampled local GPU usage returned to
31,121,408 bytes while the shared runtime/device still lived. Working/private
memory did not return to a zero baseline; caches/runtime remained until process
exit. Per-arm resource retention increases memory with sequence length. These
samples support this bounded harness, not a general performance, residency,
in-flight, threading or multi-queue guarantee. GBV was not enabled.

## Reproduction, integrity and relevant checks

Use an existing Python with NumPy (the captured environment used 3.12.14/2.3.5),
Windows x64/MSVC, the pinned trusted local SDK and matching AMD deployment.
Run from the repository root. `<numpy-python>` below is the interpreter path;
it is a sanitized placeholder, not an executable supplied with the artefacts.

The four captured attempts used the historical entry point and a fresh default
attempt directory each time:

```powershell
& '<numpy-python>' tools/verification/run.py run experiment-v01-calibration --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Current reproduction uses the responsibility-based source entry points and
the retained frozen protocol. The historical source snapshot is no longer
included; source/native hashes remain in `inputs.json`. The observations in this
record stand on the recorded measurements, not on a promise to reproduce exact
historical GPU bytes.

```powershell
& '<numpy-python>' tools/verification/run.py plan experiment-temporal-calibration
& '<numpy-python>' tools/verification/run.py run experiment-temporal-calibration --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

All full frames are independently checked before default retention is reduced.
For a concrete local diagnosis, add `--retain-diagnostics`. Independent offline
reanalysis then uses that local attempt, an exclusive output directory and no
GPU/AMD execution:

```powershell
& '<numpy-python>' tools/verification/temporal_calibration.py audit --run <local-diagnostic-attempt> --output target/calibration-reanalysis
```

The original attempt-1 reanalysis remains documented through its compact
provenance and measurements; its original failed status is preserved.

Every original attempt passed `run.py verify-run --run <attempt>` before export.
Exports used `run.py export --run <attempt> --output <exclusive-destination>`
through the same underlying `verify_run`/`export` functions. The root and nested
`artifacts.json` inventories identify every retained file. Repository/home/run
paths and pointer-labelled non-null values are normalized; meaningful hardware
IDs, provider IDs, HRESULTs, versions and hashes are preserved. Full environment
dumps and build/AMD binaries are excluded. A small sanitizer correction preserves
DXGI device IDs beside vendor IDs and normalizes sandbox temporary paths; tests
cover both. No secrets or unrelated machine configuration are required.

Relevant checks on the added/changed areas:

- `cargo fmt --all -- --check`: passed after the rename's formatting update.
- `cargo clippy -p fsr-sdk --all-targets --features dx12,movable --locked --offline -- -D warnings`: passed; Cargo reported a non-fatal incremental hard-link fallback.
- `python -m unittest discover -s tools/verification/tests -v` with the NumPy interpreter: all 42 passed, including the analytic and watchdog fixtures. Temporary-file sandbox denial in an earlier invocation required the permitted host rerun.
- `python tools/verification/run.py run fast`: an initial sandbox attempt failed writing a temporary Rust metadata file; [that failed check](2026-10-11-exp-v01-verification-calibration/checks.json) is retained. The [permitted rerun](2026-10-11-exp-v01-verification-calibration/checks.json) passed default workspace tests and formatting; ignored native tests were not counted as GPU execution.
- Native hash/signature checks, three C++ ABI compilations, `/W4 /WX` helper build, exact Rust test build and all captured final native cases passed. The renamed 21-frame input check passed separately; it does not rerun the full AMD quality matrix.
- `cargo package -p fsr-sdk --list --allow-dirty --offline` and `cargo package -p fsr-sdk --allow-dirty --offline --no-verify --target-dir target/temporal-calibration-package-inspection`: passed. The local 64-file archive contains both byte-identical licences and renamed probes, no AMD DLL. [Inspection result](2026-10-11-exp-v01-verification-calibration/package-inspection.json) retains its hash. This is package-content inspection, not release validation or publication.
- Final diff, responsibility names, documentation links and export inventories were checked. No historical record, research synthesis, research index, roadmap status or design acceptance was changed; no commit, push or publication was performed.

## Proposed later protocol and unresolved questions

**Recommendation, not acceptance:** retain the two scenes as a small structural
and diagnostic calibration baseline. Freeze exact input bytes/quantization,
camera/jitter conventions, decoded image completeness, process/fence status,
mask coverage and required ROI counts as reproducible structural criteria.
Missing payloads, a failed/uncertain dispatch/fence or zero-count required
regions must prevent a future verification claim. Preserve all outcomes.

For this particular moving rectangle, the evidence supports prospective paired
criteria using **foreground reprojected temporal residual for reversed motion**
and **area-reference silhouette error for negated jitter**, with the fixed late
window and masks. A possible later rule is correct < corresponding wrong arm
for each process repeat, with counts and repeat spread reported. The observed
2.13×/1.80× ratios motivate those choices; they are not pre-existing thresholds
and do not verify themselves. Absolute tolerances or a required separation
margin need a separately frozen rationale and fresh runs.

Keep spatial/age/leakage traces as diagnostics. Do not adopt a universal
correct-better-than-wrong interior, background stability or leakage gate from
these results: those comparisons gave contrary rankings. Pure age-0/1 leakage
needs a different predeclared motion/region variant if it is to become a required
measurement; this spike supplies no such variant's results. Smooth A is useful
for repeatability and phase/history residuals, but supplies neither a textured
stability test nor a convergence threshold.

Exact repeat and reset equality are observed local candidates, not cross-driver
requirements. A later reset-sensitivity protocol should predeclare a
history/no-reset control and a stronger history-dependent scene before claiming
causal reset sensitivity. Any new scene, velocity, threshold or reset control
requires its own fixed protocol and new execution, not retrospective acceptance
of these calibration data.

The remaining uncertainty is sensitivity beyond this two-colour rectangle,
two-cycle length and one GPU/provider/driver. This experiment establishes that
the public path and independent evidence workflow can execute and expose
meaningful differences, while also exposing invalid generic quality rankings.
It does not establish V01 completion, general image quality, HDR/mask/exposure
behaviour, provider internals or broader device/threading support.
