# Experiment: camera-cut reset recovery with a retained-history control

- Investigation and execution date: 2026-10-11.
- Question: does a one-off reset discard pre-cut history and reproduce a fresh
  context, while a matched no-reset control exposes the influence of history?
- Result: one exploration and three independent confirmations passed. In every
  block, reset/fresh RGBA output matched exactly on all 64 post-cut frames.
  The no-reset control remained measurably different. All frozen analytic quality
  windows passed; 1,280 matrix GPU frames had complete independent input audits.
- Scope: one textured synthetic cut, one pinned Windows/DX12 deployment. This
  completes the selected cut/reset extension of V01; jitter confirmation remains open.

## Baseline and implementation

The [earlier calibration](2026-10-11-exp-v01-verification-calibration.md) compared
history/reset and fresh/reset on four post-cut flat-scene frames. Their equality
could not establish reset sensitivity without a history/no-reset control.
The [quality evaluator](2026-10-11-exp-temporal-quality-validation.md) and
[continuous-history extension](2026-10-11-exp-temporal-long-sequence.md) provide
independent analytic properties, known-defect tests and longer native histories.

This experiment adds explicit exploration and confirmation profiles, a private
cut-generator export, and optional start/reset-frame fields in the existing
test-only arm table. The shared runner retains its ordinary 64-frame defaults.
Every arm uses one context and retains native resources until fences and context
destruction. The wrapper's public API, resource-lifetime contract, dependencies,
SDK version, support commitments and release gates are unchanged.

The project baseline is fsr-sdk 0.2.0 / fsr-sdk-sys 0.1.1 and AMD SDK v2.3.0.
The starting checkout was `d3d2f720495cebbc6e4e86725f7134237b5c776f`, with the
continuous-history work and this cut/reset implementation uncommitted. The
retained implementation and compact records explain the result without that
revision or unpublished history.

## Fixed scene, controls and predictions

The [exploration protocol](2026-10-11-exp-temporal-reset-recovery/exploration-protocol.json)
uses the existing 320×180 sinusoidal linear-RGB scene, 640×360 output, near
.5/far 20/FOV pi/3, foreground/background depths 2/8, and (.375,.25)
render-pixel foreground velocity. At absolute frame 64 the camera translates
(192/focal,96/focal), where focal=90/tan(pi/6). Foreground shifts (-96,+48),
background (-24,+12) render pixels. World background texture moves accordingly;
foreground texture stays local to the object. The object remains inside the
measured viewport throughout. No masks or sharpening are enabled.

Three arms isolate temporal state:

| Arm | Executed frames | Reset requests |
|---|---|---|
| History/reset | 0–127 in one context | 0 and 64 |
| History/no-reset | 0–127 in one context | 0 |
| Fresh | 64–127 in a new context | 64 |

Every post-cut input byte is identical across arms, including the first fresh
frame's nonzero cut motion. Jitter keeps absolute phase in the original 32-entry
Halton sequence. Cut motion is previousUV-currentUV: foreground
((96-.375)/320,(-48-.25)/180), background (24/320,-12/180). It returns to normal
motion on frame 65. Camera parameters, exposure and frame time remain identical.
The independent input-only DX12 child checks 25 frames before AMD dispatch.

The [analyser](../../../tools/verification/temporal_reset.py) evaluates three
inclusive windows: absolute 64–71, 72–95 and 96–127. Foreground/background use
analytic point references; silhouettes use exact area references; foreground
temporal error residual follows true flow from frame 65 onwards. The cut-crossing
pair is excluded, rather than being treated as ordinary small motion. Existing
2-display-pixel silhouette and 8-display-pixel viewport exclusions remain.

The ghost region is the pre-cut foreground interior intersected with current
background interior. It measures the old screen footprint against its current
analytic background. This is distinct from a physical disocclusion-age model.
Texture error and pixel-visibility-age metrics remain diagnostic. Empty or
nonfinite required window measurements cannot pass.

Two predictions and the quality-limit selection rule were fixed before exploration:

- Reset/fresh RGBA maximum difference must stay at or below .000001 from frame
  64 through 127: immediate recovery, with no later divergence allowed.
- Mean no-reset/fresh interior RGB difference over frames 64–71 must exceed
  reset/fresh by at least .0001. Identical controls cannot prove reset sensitivity.
- Select each analytic quality limit as 1.25 times the largest fresh-context
  window MAE plus .000001, then freeze it before three new confirmations.

This predicts history clearing, not that no-reset must lose every quality metric.
A fresh context is a behavioural comparison, not image ground truth; independent
analytic quality checks prevent equality alone from becoming a quality claim.

## Exploration, criterion selection and confirmation

Exploration ran 20:50:00–20:52:35 UTC and passed all 20 required cases, including
320 matrix GPU frames. Its source/configuration identity was
`677508b8b0dedec95a5e14ef3d6e406c3c7398269b59153b3f0b25991cb8a91d`.
The [input manifest](2026-10-11-exp-temporal-reset-recovery/exploration-inputs.json)
retains exact identities; the coordinator confirmed unchanged inputs.

After exploration completed, an additional conservative selection check was
added before any limits were selected: proposed limits must still reject known
analytic missing-object, constant-image, blur, flicker and 25% old-history
contamination defects. It did not change the scene, measurements, selection
formula or fixed reset/sensitivity predictions. The generated
[criteria](2026-10-11-exp-temporal-reset-recovery/criteria.json) retain those
counterexamples and their numerical margins. Their source exploration summary
SHA256 was verified against the retained exact summary bytes.

| Property | Frozen MAE limit | Known-defect MAE, independently analytic |
|---|---:|---:|
| Foreground | .006163200280 | .235307326337, missing object |
| Background | .003176793749 | .234153612462, constant image |
| Silhouette | .031985412349 | .087736819856, box blur |
| Temporal residual | .001074825386 | .020000000000, alternating bias |
| Old footprint | .003174403099 | .058076412840, 25% contamination |

These allowances are declared local regression policies, not confidence intervals
or perceptual thresholds. Steady-motion limits were not blindly applied to the
cold post-cut transition. The criteria SHA256 was
`96d4a0cedd7b993fb53e7a743afe62d6dedb2d4833e498be588002ac637f901c`.

Confirmation ran 20:54:30–21:01:01 UTC, passed all 23 required cases and executed
960 new matrix GPU frames. Source/configuration identity was
`4fd726f62c8ec108e160a30f3e12de5e11f794394ac73177bb82008e01abf49f`;
the [confirmation manifest](2026-10-11-exp-temporal-reset-recovery/confirmation-inputs.json)
and final run record confirm unchanged inputs, including the criteria.
No threshold was edited after confirmation outcomes. The three independent
process blocks used seeds 419, 827 and 1237, with orders no-reset/reset/fresh,
reset/fresh/no-reset and fresh/no-reset/reset.

## Equipment, commands and checks

Windows 11 build 26300; x86_64-pc-windows-msvc; Rust/Cargo 1.99.0; MSVC
14.44.35207; Windows SDK 10.0.22621.0; existing Python 3.12.14 and NumPy 2.3.5.
Adapter 0 was AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, driver
32.0.31041.1004, with 16,974,905,344 dedicated bytes. Provider 4.1.1 had ID
`0xf5a5ca1e01001001`.

Actual loaded staged AMD DLL identities were checked in every child:
loader `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`,
upscaler `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Confirmation helper SHA256 was
`d2f8329d52a766ccd1d889914f84b110722939702a07785839ecb729b8994ba6`;
Rust native executable SHA256 was
`38e45560d18e716d369a17c037b8223b0b6312589c464b8fab495be922bdb326`.
Per-phase helper/executable identities, compilation commands, ABI checks and
native prerequisites are retained in the outcomes.

Using the existing bundled Python interpreter, the executed commands were:

```text
python -m unittest discover -s tools/verification/tests -p test_temporal_reset.py -v
cargo fmt --all
cargo fmt --all -- --check
cargo clippy -p fsr-sdk --all-targets --features dx12,movable --locked --offline -- -D warnings
python -m unittest discover -s tools/verification/tests -v
python tools/verification/run.py run experiment-temporal-reset-explore --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --retain-diagnostics
python tools/verification/temporal_reset.py select --run target/verification/20261011T205000Z-experiment-temporal-reset-explore-ede038cd46bb --output tools/verification/temporal-reset-criteria.json
python tools/verification/run.py run experiment-temporal-reset --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --retain-diagnostics
```

The initial six new tests passed. Restricted full-suite execution failed on
temporary-file permissions; the permitted rerun passed 68 tests. After adding
the selection counterexample guard, seven focused tests and the final full suite
of 69 tests passed (89.669 seconds). Formatting and Clippy passed, with a nonfatal
incremental-cache hard-link copy fallback. A local evidence-collection script
initially had a syntax error before creating files; it was corrected and rerun.
There were no failed native attempts. See
[checks](2026-10-11-exp-temporal-reset-recovery/checks.json).

The selection destination above did not exist at selection time. The repository
now retains those confirmed criteria; selection refuses to overwrite an existing
file. For new reproduction, write a candidate to ignored `target/` and keep the
confirmed criteria distinct. Native evidence requires the pinned local SDK and
suitable GPU; no installation or download was introduced.

## Observations and interpretation

**Verified:** each matrix child independently compared all 294,913,280 input
bytes across 320 frames. Post-cut inputs additionally matched across all three
arms. Each child recorded 320 dispatches, submissions, fence completions and
mapped readbacks, three context cleanups, fence value 1, zero ordinary debug/device
errors, and 230,400 finite written pixels per output. Reset requests were exactly
0/64 for reset, 0 for no-reset, and 64 for fresh. The two history arms' pre-cut
output bytes were identical in all four blocks. See the
[confirmation native observations](2026-10-11-exp-temporal-reset-recovery/confirmation-native-observations.json)
and corresponding exploration observations.

In all three confirmations, all 64 reset/fresh pairs had maximum RGBA difference
0. Recovery was immediate at frame 64 and remained stable through 127. Mean
no-reset/fresh interior difference over the first eight frames was
.001449914753665552, versus reset/fresh 0, exceeding the fixed .0001 sensitivity
criterion. The no-reset difference fell from .002972787973 at frame 64 to
.000856416856 at 71, and was still .000675461239 at 127. This establishes the
influence of pre-cut history on this scene, not an explanation of the provider's
internal history rejection algorithm.

All five properties passed separately on both reset and fresh arms in each of
the three windows and three confirmation blocks. Whole-run averages could not
compensate a failed transition or late window. All 320 output images were
bit-identical across confirmation processes, despite their different arm orders.
The [confirmation measurements](2026-10-11-exp-temporal-reset-recovery/confirmation-measurements.json)
retain every window, per-frame trajectory, paired difference and verdict; the
[exploration measurements](2026-10-11-exp-temporal-reset-recovery/exploration-measurements.json)
remain separately identified as selection data.

Mixed metric rankings remain visible. In frames 64–71, reset/fresh foreground
MAE was .002511330951 versus no-reset .005468483913; temporal MAE was
.000859060309 versus no-reset .000670855633. In frames 96–127, no-reset background
and old-footprint MAE were slightly smaller than fresh/reset. The no-reset arm
even fits the selected analytic quality limits. It is the retained-state
comparison that discriminates the reset effect. Calling every no-reset output
inferior would contradict these measurements.

End-of-arm GPU usage reached 703,160,320 bytes for 128-frame history arms.
Fresh 64-frame arms reported 367,353,856–400,908,288 bytes, against a
16,165,400,576-byte budget. These include intentionally retained test resources,
not provider-only allocations. Confirmation native-command durations were
14.766, 14.969 and 17.453 seconds, excluding packing/input validation and
offline analysis; they are not a performance benchmark.

## Retention and remaining limits

Full diagnostics were temporarily retained to verify pre-cut equality and
cross-process output identities, then removed by verified compaction. Exploration
removed 667 files / 515,390,133 bytes; confirmation removed 1,949 files /
1,534,008,789 bytes. Retention records explicitly mark full-image reanalysis
unavailable. Protocols, source/native identities, selection provenance, compact
measurements and sanitized outcomes remain, with an
[artefact inventory](2026-10-11-exp-temporal-reset-recovery/artifacts.json).
Paths and transient native addresses were normalized to semantic labels; hashes,
provider/hardware IDs and error codes were preserved. Historical GPU bytes cannot
be reconstructed from hashes or guaranteed by a new run.

The cut occurs at one jitter phase and one camera displacement. The post-cut
sequence is 64 frames, about 1.067 seconds at 60 Hz. Frames and windows are
correlated; three processes on one GPU/driver are not population replications.
The local reset/fresh tolerance is not an AMD guarantee for other providers,
drivers, render scales, content or jitter schedules. The old-footprint metric is
a synthetic diagnostic, not a perceptual ghosting standard. The result verifies
the stated one-off cut/reset behaviour and frozen local quality limits; broader
cuts, arbitrary reset timings and universal quality remain outside its scope.
Jitter-sign/phase confirmation is the remaining selected V01 task.
