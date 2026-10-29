# Experiment: continuous 256-frame temporal histories

- Investigation and execution date: 2026-10-11.
- Question: do the confirmed textured-slow criteria still hold across a longer
  continuous history, without hiding late failures in a whole-run average?
- Result: all seven predeclared windows passed in all three process blocks,
  including both reversed-motion sensitivity predictions. All 21 profile cases
  passed; the 1,536 GPU frames had complete independent input/output audits.
- Scope: one synthetic steady-motion scene, one Windows/DX12 deployment.
  This completes the selected longer-sequence implementation, not all of V01.

## Baseline and implementation delta

The preceding [quality validation](2026-10-11-exp-temporal-quality-validation.md)
confirmed five local budgets and motion sensitivity on textured-slow frames
32–63. Its known-defect evaluator controls reject misleadingly smooth or
colour-consistent incorrect images. This experiment reuses that evaluator,
generator and budgets without recalibration. The project remains fsr-sdk 0.2.0 /
fsr-sdk-sys 0.1.1 against AMD SDK v2.3.0.

The new explicit `experiment-temporal-long` profile runs correct and reversed
motion for 256 frames per arm. Each arm keeps one context until destruction,
resets only on frame 0, and retains all frame resources until fenced completion
and context destruction. This preserves the existing native lifetime contract.
The private generator's frame bound is now 255. The shared Rust runner and
Python input oracle accept a bounded frame count, 64–256 in multiples of 32;
the existing profiles retain their original 64-frame default and input samples.
No public API, dependency, ownership policy or release gate changes.

The [analyser](../../../tools/verification/temporal_long.py) uses the shared
[property evaluator](../../../tools/verification/temporal_quality.py) to measure
each window separately. Its tests cover viewport/region availability, continuous
motion through repeated jitter phases, illegal counts, missing/duplicate frames,
unexpected reset metadata and a deliberately bad final window whose whole-run
average still passes. They retain the earlier evaluator's missing-object,
constant-image, blur, ghosting, texture-loss and flicker counterexamples.

## Frozen inputs and observable conditions

The [protocol](2026-10-11-exp-temporal-long-sequence/protocol.json), SHA256
`d5892c43ce09b5e2c687b1543f0cd67eeaafc0d447faf28a35f7b02d361c060c`,
was frozen before native execution. The source/configuration identity was
`96c0490e8aa4895adc7456ce6c6fc9b9bb8dc951781dac99770922ab8d569e86`;
the coordinator confirmed unchanged inputs at finalization. The
[manifest](2026-10-11-exp-temporal-long-sequence/inputs.json) records source and
configuration hashes. The starting checkout was
`d3d2f720495cebbc6e4e86725f7134237b5c776f`, with the changes described above
uncommitted. This record and retained source explain the result without that
revision or unpublished history.

The scene uses 320×180 linear RGB inputs and 640×360 outputs, sinusoidal
foreground/background textures, near .5/far 20/FOV pi/3 and depths 2/8. Motion
is (.375,.25) render pixels per frame. The rectangle starts at
[100,164) × [50,94), ends at [195.625,259.625) × [113.75,157.75), and stays
inside the measured render viewport [4,316) × [4,176) throughout. There are no
cuts, masks or sharpening. The original 32-phase Halton sequence repeats eight
times; position and temporal history continue through its boundaries.

After 32 warmup frames, seven disjoint inclusive windows cover every frame
32–255. Each correct-arm window must independently pass all five existing MAE
limits: foreground .006599587519502123, background .003138194689362351,
silhouette .032131707505351895, temporal residual .0008303100348939459,
and uncovered background .014176143044673558. These remain the earlier local
25% regression allowance plus .000001, not perceptual tolerances. Reversed
motion must produce at least 1.1 times correct foreground and temporal MAE
in every window and block. Empty/nonfinite required measurements fail.

Point references measure interiors and true-flow temporal error residuals;
exact area references measure silhouettes. The existing 2-display-pixel
silhouette and 8-display-pixel viewport exclusions remain. Uncovered-background
error uses visibility ages 2–5. Texture NRMSE and per-frame trajectories remain
diagnostic. Acceptance concerns window aggregates; it does not bound every
individual frame. No other window or property can compensate a failed window.

The input-only child samples 17 frames per arm: 0, 1, every phase-31/phase-0
boundary, and 255. All 34 frames pass before AMD dispatch prerequisites.
Every input byte and output is then checked on all 512 frames in each native
child. Three independent process blocks use order seeds 419, 827 and 1237;
arm orders are reverse/correct, correct/reverse and reverse/correct. The
13 known-defect CPU controls pass before native execution.

## Equipment and commands

The recorded equipment was Windows 11 build 26300, Rust/Cargo 1.99.0,
MSVC 14.44.35207, Windows SDK 10.0.22621.0, Python 3.12.14 and NumPy 2.3.5.
Adapter 0 was AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
16,974,905,344 dedicated bytes, driver 32.0.31041.1004. The upscaler provider
was 4.1.1, ID `0xf5a5ca1e01001001`.

The staged loader SHA256 was
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`;
the upscaler DLL SHA256 was
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Every GPU child observed those actual loaded identities. The compiled helper
SHA256 was `423368b9f881f42e4f4f916a80b26143d211c6d94c75af3d05e1d983d21935f1`;
the Rust native test executable SHA256 was
`7c38ee639963b087734fd4b9e46eab9da4d54e39f3134d31679d5122d8241050`.

Using the existing bundled Python interpreter with NumPy, the commands were:

```text
python -m unittest discover -s tools/verification/tests -p test_temporal_*.py -v
python -m unittest discover -s tools/verification/tests -v
python -m unittest discover -s tools/verification/tests -p test_temporal_long.py -k motion_does -v
cargo fmt --all -- --check
cargo clippy -p fsr-sdk --all-targets --features dx12,movable --locked --offline -- -D warnings
python tools/verification/run.py run experiment-temporal-long --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --retain-diagnostics
```

The first restricted test attempt had six temporary-file permission errors and
one new fixture error: motion bytes were incorrectly decoded as binary32
instead of binary16. A permitted full rerun passed 61/62 tests and isolated
that fixture error. After correcting the fixture, the focused test passed and
the final permitted full suite passed all 62 tests in 66.180 seconds. Formatting
and Clippy passed; Clippy reported a nonfatal incremental-cache hard-link copy
fallback. There were no failed native attempts or threshold edits. The
[checks](2026-10-11-exp-temporal-long-sequence/checks.json) retain these outcomes.

The native attempt ran from 20:17:43 to 20:26:12 UTC. Each child had a 360-second
native watchdog; observed native command durations were 21.657, 22.141 and
21.594 seconds. These durations exclude Python packing/input verification and
offline analysis and are not performance measurements. ABI/helper compilation,
native lifecycle/dispatch prerequisites and actual image criteria are distinct
passed cases in the [outcomes](2026-10-11-exp-temporal-long-sequence/outcomes.json).

## Observed results

**Verified:** all 21 cases passed with unchanged source inputs. Each GPU child
recorded 512 dispatches, submissions, fence completions and mapped readbacks,
two context cleanups, reset only on frame 0 per arm, zero ordinary debug/device
errors and 230,400 finite, written pixels per output. Each child independently
compared 471,861,248 complete input bytes. See
[native observations](2026-10-11-exp-temporal-long-sequence/native-observations.json).

All five budgets and both sensitivity predictions pass in each of the 21
window/block combinations. Selected output bytes for frames 31–255 are
identical across the three repeats; repetition on this equipment does not
establish a hardware-population uncertainty estimate. The first window's entire
correct-arm metric result exactly matches the previous quality validation.

| Frames | Correct foreground MAE | Correct temporal MAE | Reverse/correct foreground | Reverse/correct temporal |
|---|---:|---:|---:|---:|
| 32–63 | .005278870 | .000663448 | 1.4334 | 2.5260 |
| 64–95 | .005620978 | .000669745 | 1.3763 | 2.5160 |
| 96–127 | .005660815 | .000667106 | 1.3131 | 2.4368 |
| 128–159 | .005918231 | .000662867 | 1.2948 | 2.5693 |
| 160–191 | .005844753 | .000666635 | 1.3153 | 2.5034 |
| 192–223 | .005752386 | .000666549 | 1.2453 | 2.4176 |
| 224–255 | .005901403 | .000663692 | 1.2469 | 2.5268 |

These values are the same in all three blocks. Foreground error varies with
trajectory position: the final window is about 11.8% above the first, while
temporal error remains close to .000667. This is a measured variation, not proof
of monotonic history degradation; scene position and history age change together.
Window scores and all six per-frame metric trajectories are retained in
[measurements](2026-10-11-exp-temporal-long-sequence/measurements.json).

End-of-arm GPU usage was 1,376,083,968 bytes in every arm, against a reported
16,165,400,576-byte budget. Peak host working-set observations ranged from
1,916,338,176 to 1,952,112,640 bytes; private bytes reached 3,446,374,400.
The helper intentionally retains 256 frames' resources until destruction, so
these observations bound this test's retention cost on the recorded equipment.
They do not measure provider-only allocations or establish an engine memory
requirement. The shorter run's end-of-arm GPU usage was 367,353,856 bytes.

## Retention, limits and remaining work

Full payloads were temporarily retained to inspect this first duration
extension. After selecting compact results, verified compaction removed
3,110 files totalling 2,457,514,900 bytes. The
[retention record](2026-10-11-exp-temporal-long-sequence/retention.json) and
[artefact inventory](2026-10-11-exp-temporal-long-sequence/artifacts.json)
disclose that old full-image reanalysis is unavailable. The protocol, hashes,
window/per-frame measurements and sanitized outcomes remain; rerunning requires
the pinned SDK and suitable native equipment and cannot guarantee identical
historical GPU bytes. Paths and transient addresses were normalized to semantic
labels; provider IDs, error codes, hashes and hardware IDs were preserved.

At 60 Hz this sequence covers 4.267 seconds of steady synthetic motion. It does
not cover arbitrary durations, acceleration, different textures, hardware,
drivers, jitter periods or camera cuts. Repeated jitter phases and correlated
frames/windows are not independent random samples. The fixed trajectory changes
scene position with history age; this experiment cannot isolate their causes.
It verifies the selected extension under frozen local criteria and detects
the scoped reversed-motion defect. Jitter-sign confirmation and cut/reset
recovery remain open V01 work; no universal image-quality claim follows.
