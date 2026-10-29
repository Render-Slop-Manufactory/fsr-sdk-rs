# Experiment: validate the evaluator and confirm bounded image criteria

- Investigation and execution date: 2026-10-11; online sources accessed the same day.
- Question: can we replace an unjustified universal correct-versus-wrong ranking
  with falsifiable property tests, while systematically exposing interactions?
- Result: CPU evaluator validation and all predeclared textured-slow native
  criteria passed. The new textured-fast cell exposed another ranking reversal;
  it remains diagnostic rather than being retrospectively labelled a quality pass.
- Scope: synthetic linear-RGB reference scenes and one pinned Windows/DX12
  deployment. No public API, dependencies, release gates or support commitments changed.

## Baseline and scientific methods

The preceding [diagnosis](2026-10-11-exp-temporal-quality-metric-diagnosis.md)
established exact input semantics, mixed control rankings, and counterexamples
where a missing object or constant image received a perfect local score.
Those findings motivate testing the evaluator before trusting its conclusions.
The project baseline remains fsr-sdk 0.2.0 / fsr-sdk-sys 0.1.1, partial DX12
upscaling against AMD SDK v2.3.0. Research does not accept an SDK quality policy.

The following are source-supported methods; their application here is our
engineering interpretation, not a claim that the sources prescribe FSR metrics.

| Primary source | Supported method | Application in this experiment |
|---|---|---|
| [NIST, full factorial designs](https://www.itl.nist.gov/div898/handbook/pri/section3/pri333.htm) | Cross every setting of each selected factor with every setting of the others | All eight texture × speed × motion-direction cells; preserve interaction contrasts |
| [NIST, randomized blocks](https://www.itl.nist.gov/div898/handbook/pri/section3/pri332.htm) | Hold controlled nuisance factors within blocks and randomize remaining treatment order | Three independent processes on one fixed deployment; seeded shuffled arm order |
| [NIST, confirmation runs](https://www.itl.nist.gov/div898/handbook/pri/section4/pri46.htm) | Verify predictions on fresh runs; plan at least three confirmations | Previous measurements select criteria; three new GPU runs confirm them |
| [NASA/NPARC, verification assessment](https://www.grc.nasa.gov/www/wind/valid/tutorial/verassess.html), updated 2021-02-10 | Use accurate analytic verification cases; errors can interact and cancel | Analytic references, exact independent input checks, separate properties without compensating averages |
| [Terry et al., Validation in Fusion Research, v1, 2008](https://arxiv.org/html/0801.2787v1), sections III.B–F | Qualify the domain, examine sensitivity and fortuitous agreement, and validate synthetic diagnostics themselves | Known-defect evaluator controls, severity sweeps, scoped claims and explicit systematic limitations |

The fusion paper also distinguishes calibration of tolerance from assessment
of a model's validity. We use previously observed errors to choose a local
regression allowance and freeze it before confirmation. This is not a physical
validation tolerance or a perceptual quality standard. The full factorial has
only three selected factors: avoiding aliasing their interactions is affordable.
It does not cover the entire FSR parameter space. The texture factor changes
foreground and background together; their individual contributions are not isolated.

## Frozen protocol and observable conditions

The [protocol](2026-10-11-exp-temporal-quality-validation/protocol.json), SHA256
`04ed563f01814063d952bb2c6197c90e7399dbbbfc4c6bf9af342fbcdc669554`, was copied
before execution. Source/configuration identity was
`2f3f973ba101e1c989b93c00f2d4c246e02aab71b79b4b6f25d42c1789e83074`;
the coordinator confirmed those input bytes remained unchanged at finalization.
The [input manifest](2026-10-11-exp-temporal-quality-validation/inputs.json)
retains file/hash identities, including the test equipment and pinned SDK inputs.
The starting checkout was `14f006426676b72065f38740c461f13e566e3502`, with the
preceding diagnosis uncommitted. This record and the retained implementation are
sufficient to understand the experiment without that revision.

The new [evaluator](../../../tools/verification/temporal_quality.py) measures:

- foreground and background RGB error against the analytic point reference;
- silhouette error against the exact area reference;
- foreground texture error after removing channel-wise mean error, normalized
  by the reference texture energy;
- temporal error residual after true-flow reprojection;
- uncovered-background error at visibility ages 2–5 outside the silhouette exclusion.

Properties remain separate. Empty or nonfinite required measurements fail.
Texture with zero reference energy is unevaluated. Texture energies are pooled
before taking their ratio. Temporal reprojection follows the known physical flow,
independently of the perturbed dispatch motion. The slow native scene uses
bilinear reprojection, which contributes an analysis-filter limitation; the CPU
controls use integer display-pixel flow to avoid that ambiguity.

CPU validation uses analytic textured-fast frames 39/40. The ideal area image
must pass all six synthetic budgets. Twelve defective candidates must fail their
named property and the conjunction: erased object, constant foreground, 9×9 box
blur, texture loss at 25/50/100%, ghost contamination at 25/50/100%, and alternating
foreground bias at amplitudes .01/.02/.04. The three severity families must have
strictly increasing targeted error. These are known synthetic defects, not AMD
outputs or human perceptual judgements.

The native matrix crosses flat/textured scenes, slow (.375,.25) / fast (2,1)
render-pixel velocity, and correct/reversed foreground motion. One additional
textured-slow arm resets history every frame. Nine arms × 64 frames × three
independent children yield 1,728 dispatched frames. A 45-frame input-only DX12
child runs before AMD dispatch prerequisites. Camera, depth, resolution, jitter,
linear colour, exposure, half truncation and textures match the previous diagnosis:
320×180 to 640×360; near .5/far 20/FOV pi/3; depths 2/8; 32 Halton entries;
no masks, sharpening or cuts. The generator and analytic reference are retained
in [temporal_diagnosis.py](../../../tools/verification/temporal_diagnosis.py) and
the paired test-only C++/Rust equipment.

Native acceptance applies **only to textured-slow-correct**, pooled over frames
32–63. Each budget is 1.25 times the earlier calibration MAE plus .000001.
This 25% allowance is an explicit local regression policy, not an estimated
confidence interval. Three further predictions require at least 10% separation:
reversed motion worsens foreground and temporal error; per-frame reset worsens
temporal error. Every budget and prediction must pass in every block. Other
factorial cells and their raw difference contrasts remain diagnostic.

The generic matrix coordinator was extended to accept a frozen arm inventory
and three repetitions; Rust now derives frame counts from a bounded 1–32-arm
table and checks safe arm names and finite velocities. Existing default diagnosis
still uses 19 arms and two repetitions. Native resource ownership, one-frame
submission/fencing, retained resources through context destruction, and uncertain
completion process-exit handling are unchanged. Ordinary profiles do not select
the new experiment. No provider API or native ownership contract changed.

## Execution and reproduction

The completed attempt ran 2026-10-11 14:11:24–14:18:22 UTC, exit 0, with all
21 required cases passed. It used Windows build 26300, Rust/Cargo 1.99.0,
MSVC 14.44.35207, Python 3.12.14 and NumPy 2.3.5. Adapter 0 was AMD Radeon
RX 9060 XT, vendor 1002/device 7590, driver 32.0.31041.1004, provider 4.1.1
with ID `f5a5ca1e01001001`. SDK headers and signed staged DLLs were checked
against the repository's pinned hashes. Actual loaded module identities are
retained in [native observations](2026-10-11-exp-temporal-quality-validation/native-observations.json).
Helper SHA256 was `b2b26ae0d907e51bff7f7c2a89ad38daea45ef9e531de8d0166498c423e27d50`;
test executable SHA256 was `414f65010c4753c397bd075d006faa4ca65a8454d2b17e17699c4aff59993e26`.

From the repository root, use an existing NumPy-equipped Python interpreter and
the already staged SDK; neither command downloads dependencies:

```sh
python tools/verification/temporal_quality.py controls --output target/quality-controls.json
python tools/verification/run.py run experiment-temporal-quality --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python -m unittest discover -s tools/verification/tests -v
cargo clippy -p fsr-sdk --all-targets --features dx12,movable --locked --offline -- -D warnings
cargo fmt --all -- --check
```

Execution here substituted the existing bundled Python 3.12.14 executable for
`python`. Native children had 360-second watchdogs, the input-only child 180,
offline analysis 900, and the existing GPU fence limit 60,000 ms. The
[outcomes](2026-10-11-exp-temporal-quality-validation/outcomes.json) preserve
actual arguments, subprocess statuses/times, prerequisites, shuffle seeds and
orders, input audits and deployment observations. Personal checkout/attempt
paths and transient addresses use the existing semantic sanitizer; meaningful
provider, hardware and status values remain intact.

Every native frame had successful dispatch, close/submission/signal/fence and
readback; finite fully written pixels; zero ordinary debug/device errors. Every
retained input byte was checked independently against the oracle in each child.
Offline quality uses decoded RGB, never the C++ helper's quality metrics. All
33 analysis frames per arm (31 for history plus 32–63 for measurement) were
identical across the three blocks despite shuffled orders. The remaining frames
were structurally checked; whole-sequence equality was not asserted.

## Results

The ideal CPU image passed; all twelve defective candidates failed the correct
property; all severity sweeps increased monotonically. A second, deliberately
disabled evaluator returning zero for every error was detected by the test suite.
Two particularly useful controls show why the conjunction matters:

| Synthetic candidate | Misleading local result | Property that rejects it |
|---|---:|---:|
| Erased object | Background and uncovered-background error 0 | Foreground MAE .235307 |
| Constant foreground | Temporal residual 0 | Background MAE .234154; texture NRMSE 1 |
| 25% ghost contamination | Pooled background MAE .000744 passes | Uncovered-background MAE .057838 fails |

The small local ghost is detected even when foreground, background, edges,
texture and temporal checks individually pass. All detailed candidate scores
are in [measurements](2026-10-11-exp-temporal-quality-validation/measurements.json).

Native textured-slow results were identical in all three blocks:

| Property | Fresh correct MAE | Frozen maximum | Measured pixel-frames |
|---|---:|---:|---:|
| Foreground | .005278870 | .006599588 | 330,656 |
| Background | .002509756 | .003138195 | 6,477,792 |
| Area-reference silhouette | .025704566 | .032131708 | 60,544 |
| Foreground temporal residual | .000663448 | .000830310 | 324,056 |
| Uncovered background, ages 2–5 | .011340114 | .014176143 | 11,192 |

Reversed/correct foreground error was 1.4334, reversed/correct temporal error
2.5260, and reset/correct temporal error 3.5570. All exceed the predeclared 1.1
minimum. Reset still improves instantaneous foreground error; the temporal check
prevents interpreting that isolated improvement as overall superiority.

The previously unmeasured textured-fast cell is an instructive additional result:

| Metric | Correct motion | Reversed motion | Reversed/correct |
|---|---:|---:|---:|
| Foreground RGB MAE | .003179491 | .002464907 | .7753 |
| Centred foreground texture NRMSE | .030099587 | .042341461 | 1.4067 |
| Foreground temporal residual MAE | .000837059 | .002534750 | 3.0282 |

The improved colour MAE accompanies worse texture and temporal scores. Signed
mean error changes from −.003059690 to −.000810637, so mean-colour bias is a
plausible contributor to the MAE improvement. This is an observation and an
interpretation of measured errors, not evidence about FSR4's internal algorithm.
It also demonstrates why the earlier textured-slow ranking must not be promoted
to a universal texture/motion rule. All eight cells and unnormalised factorial
difference contrasts are retained, including every favourable wrong-input result.

Four focused evaluator tests and the full 56-test verification-infrastructure
suite passed. The first full-suite attempt failed with sandbox-denied temporary
file access; a permitted host rerun passed, exit 0. Clippy passed with a nonfatal
incremental-cache hard-link/copy fallback. Rust formatting was corrected after
the initial formatting check identified wrapping, then checked again. There
were no failed native attempts or post-result threshold changes in this experiment.

## Retention, baseline delta and limits

Default validated compaction removed 3,505 bulk files / 1,621,687,908 bytes,
including raw inputs/outputs and copied source snapshots. It retained outcomes,
frame/payload identities and compact metrics. The
[retention record](2026-10-11-exp-temporal-quality-validation/retention.json)
discloses that complete historical image reanalysis is unavailable. Hashes cannot
reconstruct outputs, and a new GPU run is not guaranteed to reproduce the same
bytes. The supporting directory's [inventory](2026-10-11-exp-temporal-quality-validation/artifacts.json)
identifies the selected evidence without committing the bulk dataset.

The baseline now includes a falsifiable evaluator with known-defect and severity
checks, a complete selected-factor matrix, and prospective confirmation of
bounded calibration-derived criteria. It does not claim universal perceptual
quality, all camera/motion/jitter conventions, spatial/temporal convergence,
HDR, masks, provider portability or cross-driver stability. Correlated frames
and identical blocks do not justify population confidence intervals or p-values.
The point/area references and reprojection filter remain explicit analysis
choices; the provider promises neither reference reconstruction filter.

V01 remains open, including longer/cut sequences and broader jitter/reset
coverage. Follow-up experiments can separately vary foreground/background texture,
motion magnitude and jitter convention, and use these results to freeze new
criteria before fresh confirmations. Newly observed textured-fast texture/error
behaviour is calibration evidence for that follow-up, not retrospective acceptance.
