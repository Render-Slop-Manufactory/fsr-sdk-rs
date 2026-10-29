# Experiment: active-render transition CPU/GPU timing

- Date: 2026-10-16, Europe/Berlin. Attempt timestamps use UTC, beginning on October 15 at 22:15 UTC.
- Question: how do one active-render shrink and grow change CPU recording and the
  provider's GPU command interval relative to matching stable-size controls?
- Scope: P05, development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0,
  local-owner Windows x64/MSVC + DX12, fixed maximum allocations and output.
- Outcome: **bounded execution and measurement verified; a consistent transition
  penalty or Rust/C++ speed relationship is inconclusive.**

## Baseline and implementation plan

P04 supplies independent optimized C++/Rust consumers, same-queue GPU timestamps,
separate QPC CPU timing, empty brackets and fenced reuse. M14 supplies checked
active render regions within unchanged maximum input allocations and output.
Their [performance baseline](../performance.md) and [D024](../../adr/D024.md)
establish prerequisites, not measurements of changing render sizes.

The plan deliberately reuses these boundaries rather than introducing a new
benchmark framework. Probe changes extend the shared
[Rust consumer](../../../crates/fsr-sdk/src/upscaler/dx12/tests/performance_comparison.rs),
[independent C++ consumer](../../../crates/fsr-sdk/tests/native/performance_reference.h)
and [GPU equipment](../../../crates/fsr-sdk/tests/native/gpu_timing.h).
The new [profile and independent evaluator](../../../tools/verification/active_timing.py)
are opt-in. Production APIs, bindings, dependencies, SDK pin and resource ownership
are unchanged. The worktree was clean before this work.

The [frozen final protocol](2026-10-16-exp-active-render-timing/protocol.json)
selects the following choices, with reasons:

1. Keep 1920×1080 output. Query SDK Quality for the maximum render size and
   Performance for the smaller size, avoiding an invented provider scale. Both
   consumers check the live maximum query; both query the smaller size on the live
   context and Rust also checks its pre-create result. Observed sizes are
   **1280×720 and 960×540**, with jitter phases **18 and 32**.
2. Enable dynamic resolution for every context, including stable controls. Reuse
   the same maximum input allocations, formats, device and output to isolate
   active-size changes from creation flags or application resource resizing.
3. Use stable maximum, stable smaller and maximum → smaller → maximum workloads.
   Timed contexts run 160 continuous frames: warm-up 0–31, shrink at 64 and grow at
   112. Keeping the same global frame index permits comparisons with the same
   scene position in the stable controls.
4. Generate the same analytic ramp in normalized coordinates. Translation is
   `frame / maximum_render_width`, with UV current-to-previous motion
   `(-1 / maximum_render_width, 0)` after frame 0. Active pixel motion therefore
   changes nominally from −1 to −0.75 without changing the output-space trajectory. Apply
   actual pixel projection jitter to sampling and report that same jitter to FSR.
   Query offsets before the loop using global frame modulo the current size's
   phase. Reset only frame 0, preserving history across size changes.
5. Run separate four-frame debug controls for all six workload/consumer pairs,
   shrinking/growing at 1/3. Use four independent timed blocks; alternate consumer
   order and reverse workload order to avoid always placing one consumer or size
   last. Frame samples within a process remain correlated.
6. Audit checkpoint frames 31, 63, 64, 111, 112 and 159, identically in every timed
   workload. Restore the output sentinel before recording; after fencing, require
   complete finite/written output and unchanged input bytes/guards. The complete
   colour/depth/motion readbacks match uploads. An independent Python oracle checks
   actual GPU samples at two active corners and one inactive point when present.
   It bounds fixture binary16 truncation by one relative binary16 step plus 1e-6
   for binary32 arithmetic; depth, alpha and inactive guards remain exact.
7. Keep generation, copies, sentinel restoration, resolve, synchronization,
   diagnostic resource allocation, readbacks, hashes and logging outside the CPU
   and GPU brackets. Only the effect context and input/output allocations are
   fixed; diagnostic readback resources are created outside the interval. Retain
   first-transition samples separately from later segments without a speed gate.

The native profile fails on incomplete evidence, invalid inputs, bad histories,
non-monotonic timestamps, device/debug failures or mismatched consumers. Variable
performance, including negative differences, is an observation rather than a
failed speed criterion. Image-quality research, variable output/allocation sizes,
HDR/masks, movable ownership and application throughput are excluded.

## Equipment and retained inputs

The final run uses AMD Radeon RX 9060 XT, vendor/device `1002:7590`, feature level
12_0, adapter index 0, driver `32.0.31041.1004`, provider **4.1.1**, opaque provider
ID `17700776140664213505`. The pinned SDK commit remains
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Four C++ ABI probes compile and the
staged loader/provider hashes and AMD signatures are checked before execution.
CPU and GPU use separately checked **10 MHz QPC** and **100 MHz direct-queue**
timestamp frequencies. A registry inspection after the native runs identifies
the [CPU](2026-10-16-exp-active-render-timing/cpu-inspection.json) as Ryzen 5 5600.
The recorded host is Windows 11 build 26300, Rust/Cargo 1.99.0, Python 3.14.1,
MSVC 14.44.35207 and Windows SDK 10.0.22621.0.

The [equipment record](2026-10-16-exp-active-render-timing/equipment.json) preserves
the actual OS, compiler/MSVC/Windows SDK versions, pinned native identities and
exact build commands. The [final inputs](2026-10-16-exp-active-render-timing/final/inputs.json)
identify tested source/configuration bytes, including the local probe changes.
The [final verdict](2026-10-16-exp-active-render-timing/final/run.json) records
unchanged inputs over the complete attempt; no Git revision is needed to identify
the measured state. Earlier attempts retain their own input identities.

## Execution and observations

The final profile passes **all 42 required cases**, including six debug controls
and 24 optimized timed processes: **3,864 submitted/fenced frames**. All debug
controls report zero errors. Timed processes disable the debug layer and retain
the existing helper's `UINT64_MAX` unavailable-info-queue marker; it is not
interpreted as zero observed errors. Device-removal checks and terminal teardown
pass. GPU-based validation is not enabled.
Finite output requires all four channels to be finite at every pixel. The
inherited written-pixel criterion requires at least one channel per pixel to
differ from the restored sentinel; it does not prove every channel was written.

Across consumers and repeated processes of the same workload/history length,
canonical initial descriptors, creation flags, device/provider, queries, every
jitter/reset/active-size/motion-scale input, checkpoint output/input FNV-1a
diagnostics and final output SHA-256 agree. FNV-1a is a diagnostic equality check,
not a cryptographic artefact identity. The stable and transition workloads have
identical input hashes, sampled inputs and jitters at matched active sizes.
Before shrink their outputs match too, while histories are identical. After
shrink, output equality between distinct histories is not required. Independent
GPU input sample predictions and all complete read-only input guards pass.

The following values are **GPU microseconds**, transition interval minus the
same-index stable-size control. Each first-transition value is one frame in one
process, retained separately; these are not pooled confidence estimates.

| Block | C++ first shrink | Rust first shrink | C++ first grow | Rust first grow |
|---|---:|---:|---:|---:|
| 1 | +91.36 | +9.40 | +336.84 | −7.28 |
| 2 | +122.20 | +334.92 | +336.56 | +345.24 |
| 3 | +56.12 | −122.32 | −8.40 | +6.08 |
| 4 | +831.72 | −90.12 | +29.52 | −49.56 |

Rust's later smaller-size segment GPU median differences versus stable smaller
controls are −1.36, −3.84, +1.04 and −2.28 µs; the final maximum-size segment gives
+1.52, −1.56, −14.20 and −2.44 µs versus stable maximum. Corresponding CPU recording
differences and Rust-minus-C++ comparisons also vary in magnitude and sign.
The [full stratified summary](2026-10-16-exp-active-render-timing/final/summary.json)
preserves every block, both clocks, all controls and segment distributions.
Distributions use the existing performance helper's upper middle order statistic
for even sample counts and its indexed p95 convention.
The adjacent per-workload process files retain raw CPU/GPU ticks, warm-up, empty
brackets, query/jitter metadata, checkpoint hashes and GPU input samples.

**Inference:** the measurements establish changing-size execution with valid,
matched inputs and observable per-transition timing. Their variability does not
support a single resize-cost estimate or a causal wrapper/language explanation.
Some transition intervals exceed their controls substantially; other signed
differences are negative. No numerical parity or non-inferiority claim follows.

## Attempts and verification

All attempts and their integrity checks are indexed in
[checks.json](2026-10-16-exp-active-render-timing/checks.json).

- Initial attempt `20261015T221521Z-experiment-active-timing-b168436882ea`:
  all six debug controls pass. The first timed C++ process exits zero with valid
  native output, but the evaluator rejects its unavailable debug-error count as
  an error; later dependent cases are blocked. The initial parser incorrectly
  required zero for timed children. Its [failed verdict](2026-10-16-exp-active-render-timing/failed-parser/run.json)
  and [native log](2026-10-16-exp-active-render-timing/failed-parser/timed-max-cpp.txt)
  remain evidence. No GPU prediction or timing budget was changed to repair this.
- Corrected complete attempt
  `20261015T221643Z-experiment-active-timing-4517497fe448`: all 42 pass with the
  unchanged native producers. Its summary and source identity are retained in
  `first-complete/`. The final evaluator subsequently adds explicit stable-control
  input/pre-shrink-output comparisons; it does not change the measured sequence.
- Final attempt `20261015T221942Z-experiment-active-timing-2bc25ce2edbc`:
  all 42 pass, including those additional cross-workload comparisons.
- Shared-consumer P04 GPU timing regression: all 22 cases pass.
- Shared-consumer P03 CPU comparison: all 45 cases pass.
- Initial source CI `20261015T222408Z-ci-a8e4b3699445`: temporary-directory and
  compiler fixtures encounter sandbox `WinError 5` access denied. Infrastructure
  and all four Rust test configurations fail, while checks/lints/docs/consumers
  pass. The failed attempt is preserved; it is not a passed source check.
- Authorized outside-sandbox CI `20261015T222620Z-ci-5a31cc3bbeda`: all **115 cases
  pass**, including four feature configurations, Clippy, documentation, consumers,
  examples and the four C++ ABI probes. Infrastructure runs **208 tests: 204 pass,
  four platform-specific skips**. Existing image-evaluator unit tests are checks
  of tooling; no held native quality study is resumed.
- Authorized outside-sandbox Rust 1.88 verification
  `20261015T222951Z-msrv-44a7aa5f4a53`: all **98 cases pass**, including all-target
  compilation, ordinary tests for four feature configurations and consumers.
  Actual timed GPU execution above uses Rust 1.99.0; MSRV source tests do not
  establish MSRV GPU execution.

All eight finalized attempts pass `verify-run` and record unchanged source inputs.
The initial failed evaluator and sandbox failures stay failed. The final native,
P03/P04 regressions, source CI and MSRV checks use unchanged executable probe and
protocol bytes. Only report/evidence and roadmap prose is added afterwards.

The eight active-timing evaluator tests cover missing/duplicate evidence, wrong
size sequences/scales/resets/flags, phase-dependent jitter, invalid GPU input
samples including wrong motion and guards, unmatched consumers and stable inputs,
and single-transition stratification with signed differences. Existing GPU and
CPU evaluator suites also pass (five tests each).

## Reproduction and limits

Use Python 3.11+, PowerShell 7, x64 MSVC, cached Cargo inputs and the trusted local
pinned SDK/runtime on suitable AMD DX12 hardware. Select the actual adapter:

```powershell
python -m unittest discover -s tools/verification/tests -p test_active_timing.py -v
python tools/verification/run.py run experiment-active-timing --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py run experiment-gpu-timing --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-cpu-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
```

No assistant-launched builds or tests ran alongside timed children. Unrelated
system activity, GPU clocks, scheduling and thermal state remain uncontrolled.
Serial fences, CPU-generated uploads and checkpoint readback/hash gaps can affect
subsequent clocks; the stable controls use the same schedule but do not eliminate
between-process drift. Thirty-two warm-up frames do not prove steady GPU clocks.
The named settling/steady windows are preselected reporting bins.

This is one GPU/driver/provider, two active sizes, one analytic linear-SDR scene,
one shrink/grow cycle per process and local-owner contexts. It establishes neither
universal dynamic-resolution safety/performance nor FPS, presentation latency,
sustained throughput, coloured/HDR behaviour or reconstruction quality. Sampled
analytic input checks do not prove every texel analytically correct; complete
read-only input comparisons do cover all supplied colour/depth/motion bytes.

Permanent evidence is compact JSON and selected sanitized logs, normalized to
UTF-8/LF before hashing. Private paths and transient pointer-labelled addresses
are normalized; provider/hardware IDs, native codes and meaningful hashes remain.
Final raw output images and build products stay ignored under `target/`; full
historical GPU image reanalysis requires those local attempts or a new run.
Original input readback buffers are not permanently retained. Rerunning need not
reproduce historical output bytes or timing on another stack. Final package or
release acceptance is separate from this development-source checkpoint.
