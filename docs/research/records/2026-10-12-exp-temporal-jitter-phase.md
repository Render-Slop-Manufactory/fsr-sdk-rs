# Experiment: isolated jitter sign and temporal phase confirmation

- Investigation date: 2026-10-12, Europe/Berlin. Executions below are timestamped
  in UTC and began on 2026-10-11; this is the same local overnight session.
- Question: can an independently validated position measurement distinguish
  correct jitter from wrong X/Y signs, a one-frame dispatch delay and dispatch-only
  zero, while accepting jointly shifted image/dispatch start phases?
- Result: one exploration and three fresh confirmation blocks passed. All 45
  confirmation windows and 225 paired negative-control predictions passed frozen
  criteria. The matrices generated 15,360 GPU output frames in total, including
  selection; confirmation alone generated 11,520. All had independent full input
  byte audits. The first input-only attempt failed and is retained separately.
- Scope: the selected jitter extension of V01 on one Windows/DX12 deployment.
  This is bounded analytic alignment and local quality evidence.

## Baseline and implementation

The [metric diagnosis](2026-10-11-exp-temporal-quality-metric-diagnosis.md) found that
incorrect inputs could improve individual quality measurements. The
[quality validation](2026-10-11-exp-temporal-quality-validation.md),
[continuous-history extension](2026-10-11-exp-temporal-long-sequence.md) and
[cut/reset extension](2026-10-11-exp-temporal-reset-recovery.md) established
separate bounded properties; jitter sign and temporal phase remained open.
Mean-removed texture error does not isolate position from detail loss.

Starting checkout: `62a5513162f9251f120886449818aafc9e8c709e`, clean, fsr-sdk
0.2.0 / fsr-sdk-sys 0.1.1 with AMD SDK v2.3.0. This implementation was uncommitted
during the experiment. It adds two experimental profiles, an offline phase
estimator with independent known-truth controls, protocols and frozen criteria.
The private native arm table gains start phase, separate X/Y factors and dispatch
lag; independent metadata and input checks gain the corresponding fields.
Existing seven/nine-column arms and 64-frame defaults remain supported. There
is no new public API, native export, dependency, SDK pin or quality policy.

The retained code, protocols and measurements establish the result without
requiring that Git revision. Source identities and exact child commands are in
[attempt inputs and cases](2026-10-12-exp-temporal-jitter-phase/evidence.json).

## Fixed experiment and observable predictions

The [exploration protocol](2026-10-12-exp-temporal-jitter-phase/exploration-protocol.json)
and [confirmation protocol](2026-10-12-exp-temporal-jitter-phase/confirmation-protocol.json)
use the existing linear-RGB sinusoidal scene: 320×180 render, 640×360 output,
near .5/far 20/FOV pi/3, foreground/background depth 2/8, unjittered
previousUV-currentUV motion, exposure 1, no masks or sharpening. Foreground
velocity is (0,0) for stationary groups and (.375,.25) render pixels/frame
for the moving cross-check. One context per arm runs frames 0–127, resetting
only at zero; resources remain retained until fencing and context destruction.

The frozen binary32 Halton(2,3) table uses indices 1–32, period 32.
Image sampling is pixel_center-image_jitter. Stationary starts are 0, 8, 16,
24; moving starts at zero. Start phase shifts both image sampling and correct
dispatch together. Each of the five groups has six arms:

| Arm | Dispatch relative to image jitter |
|---|---|
| Correct | Same X/Y and phase |
| Negated X | X negated, Y unchanged |
| Negated Y | Y negated, X unchanged |
| Negated jitter | Both negated |
| Lagged jitter | Previous table entry, including 0→31 wrap |
| Zero jitter | Both dispatch values zero; rendered inputs remain jittered |

Thus each block has 30×128=3,840 GPU frames. Controls within a group share all
image/depth/motion/exposure bytes. A separate input-only child checks 33 frames
per arm (990 total), covering zero/one and wrap boundaries for every start phase.
Every frame of each dispatched matrix is then compared with an independent
analytic full-byte oracle and checked for completed fences, finite fully written
output, no debug errors, no device removal and the intended loaded modules.

Frames 0–31 are warmup. Inclusive windows 32–63, 64–95 and 96–127 are evaluated
separately; a good window cannot compensate for a failing later one. Before
selection, the protocol fixes the following rules:

- Correct X/Y/radial position RMS and five local quality limits are
  1.25×the largest correct window in each stationary/moving group + .000001.
- Selection refuses any alignment limit ≥.125 render pixel, or any quality
  limit accepting its predeclared analytic damaged-image control.
- Each wrong arm must exceed paired correct alignment RMS by at least 10%
  plus .001 pixel in every window: X for X reversal, Y for Y reversal,
  radial for both reversal, lag and zero.
- Freeze criteria before three new processes with arm-order seeds 419, 827,
  1237. Exploration uses seed 419. Wrong inputs need not lose every other metric.

These are local regression allowances and sensitivity predictions, rather than
perceptual thresholds or confidence intervals.

## Independently validated measurement

The [position analyser](../../../tools/verification/temporal_jitter.py) fits
DC+sine+cosine independently per known RGB sinusoid. Red determines X phase,
green determines Y; the oblique blue channel reports consistency. DC bias,
amplitude ratio and residual are recorded separately. A reported shift δ means
the output at (x,y) fits the reference at (x+δx,y+δy); it is a reference-coordinate
shift, not an assumed visual object-motion sign. Images are never realigned
before computing the other quality metrics.

The estimator samples every fourth output pixel, excludes an 8-render-pixel
viewport margin and silhouette neighbourhood, requires at least 100 samples
and design Gram condition ≤100. The valid phase domain is ±1 render pixel
per axis, with .002 numerical tolerance at the boundary; positive amplitude
ratios must be .25–2 in every channel. Empty, nonfinite, constant or out-of-domain
measurements fail. Within this bounded domain the periodic texture has a unique
phase solution; its shortest relevant period is about 15.7 render pixels.
This is not a registration algorithm for arbitrary images or global translations.

Known images are generated directly from analytic translated sinusoids, independently
of the fitted coefficients, and quantized through IEEE binary16 readbacks.
Controls cover ideal images, independent brightness +.03, contrast .5,
±.125/.25/.5/1 translations, translations with contrast 1.5/bias -.02,
isolated X/Y and unequal-axis translations, constant images, displacement 2,
and NaN samples. Final confirmation passes 44 controls across foreground and
background. A zero-output estimator fails; a coupled estimator mapping
(x,y)→(-y,-x), which passes anti-diagonal-only shifts, fails the isolated-axis
controls. These mutant checks exercise actual blind spots.

The five quality properties remain foreground/background point MAE, silhouette
exact-area MAE, true-flow foreground temporal error-residual MAE, and mean-removed
texture NRMSE. Known missing-object, constant-foreground, blur, .01 flicker and
25% texture-loss controls guard their respective budgets. Disocclusion-age
metrics are not acceptance criteria in this no-cut jitter experiment; the earlier
V01 quality validation supplies that separate evidence.

## Equipment, commands and source boundaries

Windows 11 build 26300, x64 MSVC 14.44.35207, Windows SDK 10.0.22621.0,
Rust/Cargo 1.99.0, Python 3.12.14 and NumPy 2.3.5. Adapter 0 was AMD Radeon
RX 9060 XT, vendor/device 1002/7590, 16,974,905,344 dedicated bytes,
driver 32.0.31041.1004. Provider 4.1.1, ID `0xf5a5ca1e01001001`.
Explicit adapter selection did not fall back. Native children use PowerShell 7
and the existing trusted local SDK deployment.

Loaded DLL SHA-256 identities:

- Loader: `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler: `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
- Confirmation helper: `701cc853e2d15660240a8a3afbda6e3aefc12a4116de429d47d31fc93dca506b`.
- Confirmation fsr-sdk test executable: `cda94ce5e49a2c2f966b6846553fbe1e9c4b2403f213879ce5f4728bdd71f2b7`.

Commands from the repository root, using the existing Python runtime:

```sh
python tools/verification/run.py run experiment-temporal-jitter-explore --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/temporal_jitter.py select --run target/verification/20261011T224100Z-experiment-temporal-jitter-explore-526c51434326 --output tools/verification/temporal-jitter-criteria.json
python tools/verification/run.py run experiment-temporal-jitter --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python -m unittest discover -s tools/verification/tests
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

The first command was issued twice: the initial failed input-only attempt, then
the corrected exploration. Selection exited 0 and exclusively created the
criteria before confirmation. Reproduction should select into a fresh candidate
file; overwriting the retained criteria would destroy this selection boundary.
The first full unit-test invocation could not create temporary directories in
the sandbox; the authorised rerun passed all 79 tests in 123.198 seconds.
Formatting and Clippy checks passed. Initial development controls exposed
half-precision perturbation just outside ±1 and an outdated input-frame plan;
both were corrected before the successful native exploration.

## Attempts and measured outcomes

| Attempt | UTC interval | Outcome |
|---|---|---|
| Initial failure, `20261011T223651Z-…-721ded9f43e5` | 22:36:51–22:39:28 | Input-only byte mismatch; main matrix and dependent checks blocked |
| Exploration, `20261011T224100Z-…-526c51434326` | 22:41:00–23:04:32 | 20/20 cases passed, 3,840 matrix frames |
| Confirmation, `20261011T231001Z-…-d16869396763` | 23:10:01–23:54:35 | 23/23 cases passed, 11,520 matrix frames |

All intervals are on 2026-10-11 UTC. Each final source/input identity check
reported unchanged inputs. Exploration input identity is
`019c51050bdd24ff3bff126ba5e5f45db25d02964eb0b83d343ef0cbc7957bf6`;
confirmation is `4d4f5def1195b9fa8d4e8eb0411921b094737e1d7c46fb61cf05b3d95ec44ff0`.
The [criteria](2026-10-12-exp-temporal-jitter-phase/criteria.json) bind the
exploration summary SHA-256
`7a7102836d5e3e2ef83bd4208547bf6a9b09304e7269fc04fae571e27e152808`.

The first input-only native child completed 990 frames, but the independent
oracle rejected stationary-p00-correct frame 1, first differing byte 755601
in the half-motion-vector payload. This exposed inconsistent signed-zero
encoding for stationary foreground motion. Native generator and Python oracle
now explicitly encode positive zero; physical zero motion is unchanged.
The original raw data had already been compacted, so no exact old byte pair or
compiler-specific cause is claimed. The failed attempt is preserved in
[run](2026-10-12-exp-temporal-jitter-phase/initial-failure-run.json) and
[case outcomes](2026-10-12-exp-temporal-jitter-phase/initial-failure-cases.json).
No AMD matrix ran in that attempt; it is not evidence of a provider failure.

After exploration and before confirmation, the implementation added isolated
X/Y/unequal-axis and nonfinite CPU controls, bounded analytic input reuse,
byte-verified compression reuse, reuse of stationary analytic references, and
an 1,800-second offline watchdog (the native matrix remains bounded to 360s).
The native generator, position fit, quality measurements, selection formula
and frozen limits did not change. Exploration has 26 position controls;
confirmation has 44. The offline audits took 370.672 and 977.531 seconds.
Every newly generated payload was fully round-trip compared before raw removal.
Cache corruption is tested to fail while preserving the raw source.

Full window measurements are in
[exploration](2026-10-12-exp-temporal-jitter-phase/exploration-measurements.json)
and [confirmation](2026-10-12-exp-temporal-jitter-phase/confirmation-measurements.json).
The following maxima and weakest ratios are identical across the three fresh
confirmation blocks:

| Group / correct RMS | X | Y | Radial |
|---|---:|---:|---:|
| Stationary measured maximum | .029564534 | .035958809 | .046550836 |
| Stationary frozen limit | .036956667 | .044949512 | .058189545 |
| Moving measured maximum | .016880675 | .028397787 | .033036215 |
| Moving frozen limit | .021101844 | .035498234 | .041296269 |

All position values are in render pixels. Correct-arm five-property quality
limits also passed every window. Four stationary starts all pass the same
group limits. The wrong/correct alignment RMS minimum over all applicable
windows is:

| Wrong dispatch | Stationary | Moving |
|---|---:|---:|
| Negated X, X RMS | 3.0824 | 7.4525 |
| Negated Y, Y RMS | 4.4483 | 4.6471 |
| Both negated, radial RMS | 3.9236 | 4.4029 |
| One-frame lag, radial RMS | 3.2405 | 4.1088 |
| Zero, radial RMS | 7.6287 | 2.1769 |

The smallest excess above the entire required 1.1×correct+.001 prediction
is .034578 render pixel (moving zero dispatch). Thus the result is not only
three equal runs barely satisfying a comparison.

Mixed quality rankings remain. Across 180 stationary negative-control/window
comparisons in confirmation, wrong jitter improves edge MAE in 36 and temporal
MAE in 123. Across 45 moving comparisons, foreground point MAE improves in 6.
These are correlated diagnostic counts, not independent statistical trials.
They are not converted into failed alignment predictions or hidden by a
combined score. [Derived results](2026-10-12-exp-temporal-jitter-phase/results.json)
retain these counts alongside the measured position separation.

[Output identity comparison](2026-10-12-exp-temporal-jitter-phase/output-identities.json)
compares all 3,840 decoded output SHA-256 entries, including warmup, from the
exploration with each freshly generated confirmation block: all match.
All confirmation window dictionaries also match the exploration exactly.
Repetition is
observed on this configuration; compression caches do not substitute native
generation or independent byte audits.

## Retention, baseline delta and limits

The adjacent [evidence inventory](2026-10-12-exp-temporal-jitter-phase/evidence.json)
retains sanitized attempt/input identities, exact commands and statuses,
protocols, criteria, compact window measurements and retention manifests.
Identical confirmation window dictionaries are shared only after exact equality
checking, retaining all repeat verdicts and the original summary hash. Raw
inputs/readbacks and verbose per-frame trajectories were removed by verified
default compaction; old full-image reanalysis is unavailable. Personal paths
and labelled transient addresses are normalized, while meaningful numeric
IDs, hashes, versions and error codes remain. New execution requires the pinned
SDK, existing NumPy runtime and suitable native equipment; historical GPU byte
identity is not guaranteed by re-executability.

**Verified:** the selected local sign/phase errors are distinguishable from correct
jitter in every declared window, including sequence wrap; jointly changed start
phases are accepted; known translations are measured independently of brightness
and contrast. **Inference:** a property-specific position measurement resolves
the earlier ambiguity better than demanding a total ordering of all quality
scores. The mixed scores remain true observations, not universal judgements
about correct or incorrect inputs.

The evidence covers one GPU/provider/driver, one periodic texture family, fixed
render/output sizes, positive bounded contrast, four stationary starts and one
moving trajectory. It does not establish arbitrary-image registration, jitter
policy, arbitrary phase lag, perceptual quality, other hardware or unlimited
history. Frames and windows are correlated; there is no population uncertainty
estimate. Other textures and equipment are follow-ups for broader claims,
rather than missing inputs for this selected confirmation. Ordinary/release
profiles remain unchanged. This closes the selected jitter question; roadmap
completion is tracked separately with the other V01 records.
