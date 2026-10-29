# Experiment: contrary quality rankings and temporal metric counterexamples

Investigation and execution date: **11 October 2026**. The question was whether
the contrary rankings in the [earlier calibration](2026-10-11-exp-v01-verification-calibration.md)
identify an erroneous metric/reference implementation, a weak scene or region,
or real local image-error trade-offs that invalidate a correctness criterion.

**Observed:** two independent 1,216-frame matrices completed through the public
Rust `Upscaler`; all **2,432 full RGBA16F outputs were bit-identical between
repeats**. Input-only validation checked 95 GPU-uploaded frames before AMD calls,
then every input byte in both matrices. All 19 structural profile cases passed.
The original slow-flat measurements were reproduced exactly. Incorrect motion
still improved local uncovered-background error in a faster scene with nonempty
required regions. A previously suggested jitter/silhouette ranking reversed
in that faster scene. Textured foreground, silhouette and temporal measurements
exposed the incorrect motion/jitter controls, while per-frame reset improved
spatial error but worsened textured temporal stability.

**Conclusion:** the universal rule “correct inputs must outperform incorrect
inputs on each quality metric” is invalid for these measurements. This is not
evidence that the documented motion convention is wrong, nor that MAE arithmetic
is wrong. Individual error measures describe specific image properties; they
are not interchangeable oracles for integration correctness. Choosing only
favourable comparisons from calibration would conceal these counterexamples.
This experiment accepts no replacement quality threshold and does not complete V01.

## Baseline, scope and fixed method

The worktree was initially clean at
`14f006426676b72065f38740c461f13e566e3502` (“Implement temporal calibration with
compact evidence”). Crate versions remain `fsr-sdk 0.2.0` and `fsr-sdk-sys 0.1.1`;
SDK pin remains v2.3.0 / commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The existing calibration was already
integrated in the dispatch topic. Its original full images had been removed
under compact retention; this work executed new GPU frames rather than
pretending to reanalyse those old bytes.

The [protocol](2026-10-11-exp-temporal-quality-metric-diagnosis/protocol.json)
was fixed before the new AMD dispatches, including all scenes, velocities,
controls, the 32..63 measurement window, region definitions and synthetic
counterexamples. Its SHA256 is
`054ccb6e0f1a6995bfdc63fc435d084758f3f08c23633f6c493446f5ea0e20ae`.
The successful source/SDK input identity is
`b068e894bf1bec4d02a4d84393003b74c0c4916a47e2ce7272481d9024c29d47`;
the [inventory](2026-10-11-exp-temporal-quality-metric-diagnosis/inputs.json)
identifies individual files. No scene, velocity, mask, metric or threshold was
searched or adjusted after viewing AMD outputs.

The new profile is `experiment-temporal-diagnosis`, excluded from routine,
release and `native-upscaler` profiles. Probe code comprises
[the Python coordinator/oracles](../../../tools/verification/temporal_diagnosis.py),
[Rust capture](../../../crates/fsr-sdk/tests/native/temporal_diagnosis.rs) and
[C++ input generation](../../../crates/fsr-sdk/tests/native/temporal_diagnosis.h).
It reuses the existing fenced DX12 equipment and input readbacks; the original
calibration generator/matrix is unchanged. Sharing test-only equipment required
parent-module visibility for its existing operations. There are no production
API, SDK, native ownership, dependency, support or licence-policy changes.

### Scenes and controls

Each arm has **64 consecutive frames in its own context**, not 1,216 consecutive
history frames. Each process executes 19 arms:

| Scene | Render-pixel velocity | Controls |
|---|---|---|
| Stationary affine RGB plane | (0,0) | Correct, negated dispatch jitter, zero dispatch jitter, reset every frame |
| Original two-colour rectangle | (0.375,0.25) | Correct, reversed motion, zero motion, negated dispatch jitter, reset every frame |
| Faster two-colour rectangle | (2,1) | The same five controls |
| Textured rectangle and background | (0.375,0.25) | The same five controls |

Render/output sizes are 320×180 / 640×360. The rectangle is
`[100+vx*t,164+vx*t) × [50+vy*t,94+vy*t)` and stays in the viewport.
The camera, normal finite depth, formats, explicit exposure/pre-exposure 1,
disabled sharpening and absent masks match the earlier calibration. Foreground
z=2, background z=8, near=0.5, far=20, FOV=π/3. Colour/depth/motion select the
same opaque surface at sample `(x+0.5-jx,y+0.5-jy)`. Jitter is the same binary32
32-phase Halton table, repeating without restarting object motion.

Correct foreground motion is unjittered previous UV minus current UV,
`(-vx/320,-vy/180)` after frame 0; static background motion is zero. This matches
the current-to-previous direction in the pinned
[AMD guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#providing-motion-vectors).
The wrapper scales UV motion by render dimensions. The guide's separate NDC
example discrepancy remains as documented in the existing source research;
these experiments do not adopt that NDC example or resolve its origin.

Reversed/zero motion change foreground motion only. Jitter controls change
dispatch jitter without changing the rendered inputs. Reset-all requests reset
on every frame in the same context; correct arms reset only frame 0. There are
no camera cuts in this matrix. These are deliberately incorrect temporal
semantics within the existing valid resource/lifetime boundary.

Textured background RGB is
`(.25+.1 sin(.35x), .30+.1 sin(.27y), .35+.1 sin(.21x+.17y))`.
Foreground RGB in object-local coordinates `u=x-left,v=y-top` is
`(.65+.15 sin(.4u), .20+.10 sin(.3v), .15+.08 sin(.23u+.19v))`.
Both surfaces change relative to the flat scene; this is not a factorial test
isolating background texture alone. Arithmetic/half truncation is retained from
the original helper. The reference is evaluated independently at unjittered
display points, and exactly integrated over each display-pixel box, including
partial silhouette coverage and sinusoidal texture. The sine rectangle integral
uses midpoint sine times the two sinc factors and footprint area. Dense
independent quadrature checked it before GPU execution. Neither reference is
a promised provider reconstruction filter.

## Execution and evidence

| Equipment | Observed |
|---|---|
| OS / target | Windows 11 `10.0.26300`, `x86_64-pc-windows-msvc` |
| Rust / Cargo | 1.99.0; Rust commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM 23.1.1 |
| Native tools | MSVC 14.44.35207, Windows SDK 10.0.22621.0 |
| Analysis | Existing Python 3.12.14, NumPy 2.3.5, Pillow 12.3.0; nothing installed |
| GPU / driver | AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, adapter 0, driver `32.0.31041.1004` |
| Provider | Successful query, ID `0xf5a5ca1e01001001`, name `4.1.1` |

The staged loader/provider signatures were valid AMD signatures. Actual loaded
paths were compared by filesystem identity and SHA256 with the staged files.

| Native file | SHA256 |
|---|---|
| Loader | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| Upscaler provider | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |
| Compiled helper | `11bb5545786f86c8bd39f61900c40374ddfd43e0b2b395cde8d651c84090c2a9` |
| Staged Rust test executable | `776df18639fc8150d52a4f0520cb920196196ecf9c680431374bdd17f01f84c2` |

The 95-frame input-only child covers frames 0,1,31,32,63 of every arm without
loading/calling AMD. Every full matrix input was also checked byte-for-byte
against the independent Python oracle. For each scene/frame, colour, depth and
exposure are identical across controls; jitter/reset arms also retain identical
motion bytes. There are 1,216 such paired-input checks per repeat.

Each matrix independently completed 1,216 successful close/submission/signal/
fence/mapped-readback observations and 19 context/equipment cleanups. Every
frame reports dispatch success, fence exactly 1, all 230,400 pixels finite and
written, zero ordinary DX12 debug errors and zero device removal. Python checked
every RGBA channel for finiteness and sentinel replacement. The matrices took
54.203 and 57.188 seconds in their native children; the full independent audit
took 829.797 seconds. Packing/input validation is additional time. A preliminary
seven-arm read-only analysis ran alongside the audit; these durations are not
benchmarks or isolated GPU timings.

Only one frame is submitted at a time. Per-arm equipment survives all fences
and context destruction, then is released. Existing uncertain-completion and
destroy-error paths terminate the isolated process. The child watchdog is 360s,
input child 180s, offline audit 900s, per-fence bound 60,000ms. No uncertain GPU
completion, provider failure, GBV, multithreaded transfer or recording-ahead case
was induced here.

### Failed attempts and repairs

All four attempt outcomes are retained in
[attempts.json](2026-10-11-exp-temporal-quality-metric-diagnosis/attempts.json).
All directory names begin with `20261011`:

| UTC prefix / suffix | Outcome | Execution and repair |
|---|---|---|
| `125730Z…5ef387746cb3` | Failed | Preparation/build passed; a module-parser variable shadowed the outer child name, causing `UnboundLocalError` before the input child. No new GPU child or AMD dispatch ran. Rename the local parser variable. |
| `125818Z…ce269c959e9e` | Failed, incomplete case inventory | Sandbox denied replacement of `abi.headers/result.json.pending`; four case outcomes recorded. No input child or AMD dispatch ran. The permitted host retry used a fresh attempt. |
| `125839Z…238de74070ca` | Failed input oracle | All 95 input-only GPU frames captured; exact comparison rejected signed-zero motion bytes in `flat-fast-zero-motion`, frame 1. C++ produced −0 while the integer-valued Python expression produced +0. Canonicalize zero-motion to +0 in both generators. AMD prerequisites/matrices were blocked. |
| `125928Z…075141f7cb06` | Passed structural profile | All 19 cases passed, including input validation, existing lifecycle/single/temporal/semantic prerequisites, both new matrices and the independent audit. Source/SDK inputs remained unchanged during execution. |

Zero sign has no geometric displacement, but the strict byte check correctly
refused the undocumented representation difference. These repairs precede all
new AMD outputs; they did not adjust scene/quality measurements to obtain a
preferred ranking. Synthetic temporal counterexamples were also changed from
a stated zero to an explicitly computed two-frame reprojection residual before
AMD execution; their zero result was preserved.

## What the measurements establish

All native values below pool **frames 32..63**, using repeat 1. Repeat 2 has
identical full outputs and identical pooled measurements. RGB units are linear,
unclipped; counts are pixel-frames, not independent statistical trials. Regions
exclude the outer eight display pixels; foreground/background interiors are
farther than two display pixels from the silhouette. Temporal foreground error
residual uses bilinear reprojection by the true object motion, requiring all
previous taps to be foreground interior. Full compact values for every arm and
both point/area targets are in
[measurements.json](2026-10-11-exp-temporal-quality-metric-diagnosis/measurements.json).

### 1. A metric can be correct and unsuitable as a stand-alone criterion

The independent CPU negative controls are explicitly **synthetic images**, not
AMD outputs. Replacing the entire scene with constant background gives
background-leakage MAE **0**, while foreground RGB MAE is **0.395833**: the object
has disappeared. It also has a computed foreground temporal residual of zero
on the flat scene. A constant foreground image similarly has zero foreground
temporal residual while background RGB MAE is 0.395833. A 3×3-blurred ideal image
has zero interior/leakage error on this flat scene while edge MAE is 0.0659212.

Thus leakage measures colour contamination in a specific region; temporal
residual measures variation of error; interior error excludes the silhouette.
None alone proves that the object, detail or temporal reconstruction is correct.
The synthetic controls expose a criterion problem without alleging an arithmetic
bug in those measures.

### 2. The original contrary ranking is reproducible

| Slow-flat control | FG point MAE | Area-edge MAE | FG temporal residual MAE | Pure age-2..5 leakage MAE |
|---|---:|---:|---:|---:|
| Correct | 0.00200124 | 0.0489006 | 0.000299944 | 0.0276891 |
| Reverse motion | 0.00161204 | 0.0513651 | 0.000640252 | 0.000817824 |
| Zero motion | 0.00159748 | 0.0630695 | 0.000336055 | 0.00640694 |
| Negated jitter | 0.00122911 | 0.0882337 | 0.000415732 | 0.0164511 |
| Reset every frame | 0.000193195 | 0.0417835 | 0.0000000692648 | 0.0000703087 |

The original correct/reverse/jitter measurements match the earlier compact
record exactly. Reset-all improves **all four** listed measures. A constant
two-colour interior gives a weak challenge to history/detail preservation.
The pooled interior also weights approximately 95% background and 5%
foreground, excluding the silhouette; a good aggregate can conceal important
object/edge changes.

**Inference:** altered reprojection/history handling can reduce local flat-scene
contamination even with invalid motion. Reset-all supports history handling as
a relevant variable. It does not prove that reversing motion internally performs
a reset or identify signed-provider history weights, rejection or dilation.

### 3. Fixing the empty region does not restore the expected ranking

The original slow scene has no pure-background age-0/1 pixels beyond the edge
exclusion. The faster scene has **5,664 pure age-0 pixel-frames** and **19,200
pure age-1 pixel-frames**, with the same counts across all controls.

| Fast-flat control | Pure age-0 point MAE | Pure age-2..5 leakage MAE | Area-edge MAE |
|---|---:|---:|---:|
| Correct | 0.00558500 | 0.00260737 | 0.0522869 |
| Reverse motion | 0.00130849 | 0.000341615 | 0.0620470 |
| Zero motion | 0.00291568 | 0.000621034 | 0.0703381 |
| Negated jitter | 0.00397384 | 0.00134413 | 0.0495655 |
| Reset every frame | 0.000140781 | 0.0000677384 | 0.0505914 |

Incorrect motion still improves the measured uncovered background. This refutes
the explanation that empty young regions alone caused the contradiction.
Moreover, **negated jitter improves area-edge MAE at this velocity**, reversing
the earlier proposed correct-better-than-negated-jitter silhouette criterion.
That criterion does not transfer even to this predeclared flat velocity variant.
The wrong motion arms simultaneously worsen foreground reconstruction and its
temporal residual; no single local ranking describes the whole image.

### 4. Texture exposes the spatial/temporal trade-off

| Textured-slow control | FG point MAE | Area-edge MAE | FG temporal residual MAE |
|---|---:|---:|---:|
| Correct | 0.00527887 | 0.0257046 | 0.000663448 |
| Reverse motion | 0.00756687 | 0.0333491 | 0.00167585 |
| Zero motion | 0.00865649 | 0.0362833 | 0.00161240 |
| Negated jitter | 0.00971701 | 0.0565716 | 0.00193197 |
| Reset every frame | 0.00244259 | 0.0253951 | 0.00235988 |

Reversed motion worsens FG error by **1.43×**, area-edge error by **1.30×** and
FG temporal residual by **2.53×**. Zero motion and negated jitter also worsen
all three. This supplies a useful counterexample to the flat scene's misleading
foreground ranking and is consistent with the established motion/jitter inputs.

Reset-all, however, reduces FG point error to **0.463×** correct while increasing
FG temporal residual to **3.56×** correct. Its actual current frames are closer
to this point reference, yet their reprojected error fluctuates more. Calling
either scalar “the quality” would hide the other property. Temporal residual is
not a perceptual flicker score, and bilinear reprojection contributes its own
filtering; these are bounded image measurements, not a subjective-quality study.

### 5. The affine plane is also an ambiguous alignment diagnostic

Correct plane point MAE is 0.00109215 and signed mean is **−0.00109215**; error
is dominated by negative RGB bias. Negated jitter gives MAE 0.00120350 and
background temporal residual 0.000273999 versus correct 0.0000817336. Zero
dispatch jitter improves spatial MAE to 0.000990866 but worsens temporal residual
to 0.000130661. Reset-all gives MAE 0.000269506 and residual 0.000135957.

The predeclared ±0.5-render-pixel shifted point targets reduce correct-plane MAE
for left/up shifts (0.000852107 / 0.000848011) and worsen it for right/down
(0.00133629). On an affine colour field, shifting the target also changes its
brightness. **Inference:** this comparison confounds spatial registration with
RGB bias; it cannot establish a jitter-sign correction. Input truncation is
included in error, and the source of the additional provider/history bias is
unresolved. A future alignment test needs identifiable spatial structure and
separate bias/phase checks, rather than fitting this smooth ramp.

## Reproduction, retention and checks

Use an existing Python with NumPy/Pillow, Windows x64/MSVC and the trusted local
SDK deployment. `<numpy-python>` below is an interpreter placeholder. From the
repository root:

```powershell
& '<numpy-python>' -m unittest discover -s tools/verification/tests -p test_temporal_diagnosis.py -v
& '<numpy-python>' tools/verification/run.py plan experiment-temporal-diagnosis
& '<numpy-python>' tools/verification/run.py run experiment-temporal-diagnosis --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --retain-diagnostics
& '<numpy-python>' tools/verification/run.py verify-run --run <new-attempt>
& '<numpy-python>' tools/verification/temporal_diagnosis.py audit --run <new-attempt> --output target/analysis/<new-name>
```

`--retain-diagnostics` was used for this specific diagnosis. Each matrix captured
3,362,001,664 decoded bytes / 831,977,605 compressed bytes. Full generated
inputs/outputs and verbose analyses were kept under ignored `target/`, consumed
by independent validation, then compacted after the diagnosis. The permanent
[evidence index](2026-10-11-exp-temporal-quality-metric-diagnosis/README.md)
contains compact measurements, protocol/jitter, source/native identities,
commands/outcomes and a fixed frame-40 crop sheet. No full historical GPU images,
AMD binaries, repeated source snapshots or complete environment dumps are
included. Old full-image offline reanalysis is no longer available after
compaction; new execution may differ with equipment/software. Run paths and
personal paths are normalized in retained case records.

Relevant checks:

- All 19 structural/native/offline cases passed; run inventory verified before
  evidence selection and again after compaction. Native counts are cross-checked
  from logs and frame metadata in the retained native observations.
- Five independent analytic tests passed before execution and after repairs:
  dense quadrature, exact equality with the original flat reference, input
  isolation, synthetic metric counterexamples and profile/retention isolation.
- The full infrastructure suite passed **52 tests** on the permitted host.
  Its first sandbox invocation failed on temporary-file permissions; that was
  not an analytic or native-quality result.
- `cargo fmt --all -- --check` passed. `cargo clippy -p fsr-sdk --all-targets
  --features dx12,movable --locked --offline -- -D warnings` passed, with the
  existing nonfatal incremental hard-link fallback.
- Four pinned headers, three independent C++ ABI probes, `/W4 /WX` helper
  compilation and exact Rust test compilation passed in the native profile.
- `cargo package -p fsr-sdk --list --allow-dirty --offline` and local packaging
  with `--no-verify --target-dir target/temporal-diagnosis-package-inspection`
  passed. The inspected 66-file archive includes both new probes, byte-identical
  MPL/AMD licence texts and no AMD DLL. This is content inspection, not release
  verification or publication.
- Documentation references, final diff and repository size were checked.
  No commit, push, publication, synthesis update or design acceptance occurred.

## Baseline delta and remaining questions

**Verified:** arithmetic/reference checks and complete byte-audited execution
give no evidence of a broken MAE calculation or a motion/jitter generator sign
error for these cases. They do expose empty regions, degenerate scene/metric
combinations and actual conflicting spatial/temporal error rankings. A previously
suggested jitter/silhouette criterion has a direct native counterexample.

**Recommendation, not acceptance:** replace universal per-metric rankings with
separate, purpose-specific checks. Validate input geometry/conventions directly;
evaluate reconstruction with spatially identifiable foreground/background/edge
regions; evaluate temporal change alongside preserved detail; require nonempty
regions; and calibrate any candidate using erased/constant/blurred/reset controls
and independently chosen scene variants. Preserve contrary comparisons instead
of discarding them. These findings reject automatic promotion of the earlier
candidate rules; they do not yet select a new V01 gate or numerical tolerance.

**Unresolved:** the signed provider's history/rejection/dilation mechanics,
source of ramp bias, filter/reference choice appropriate to real rendering,
perceptual weighting of detail versus temporal stability, textured fast
disocclusion, high-frequency detail recovery, longer-than-two-cycle behaviour,
camera cuts/reset causality and other hardware/provider/driver configurations.
No result here establishes that intentionally incorrect motion is a valid
integration strategy or that per-frame reset is a recommended quality policy.
