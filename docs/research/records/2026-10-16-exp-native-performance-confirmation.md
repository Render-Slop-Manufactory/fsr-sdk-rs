# Experiment: native performance confirmation on five matched DX12 workloads

- Experiment and offline analysis date: 2026-10-16.
- Question: does average additional public Rust CPU dispatch cost stay below
  the prespecified **5 µs per dispatch** budget on each selected workload?
- Outcome: **within budget on all five workloads**, with 112 new paired process
  blocks per workload. All 1,220 confirmation cases pass independently of the
  speed verdict. GPU and lifecycle measurements remain descriptive.
- Baseline: the [implementation/pilot record](2026-10-16-exp-native-performance-characterization.md)
  preserves the earlier pilot, 110-block selection and intentionally interrupted
  confirmation. This record concerns a separate, complete unattended batch with
  a fresh pilot and a newly frozen 112-block selection. Historical samples are
  neither pooled into this estimate nor overwritten.

## Setup and identified inputs

Measured code is development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, not the
published `fsr-sdk 0.2.0` archive. The producer input identity is
`ec05938be270cf0944c33b2fd152eaa059a66b1a426d9328d17fbf92fb54bc7d`.
The [pilot input inventory](2026-10-16-exp-native-performance-confirmation/pilot-inputs.json)
and [confirmation input inventory](2026-10-16-exp-native-performance-confirmation/confirmation-inputs.json)
identify source and verification inputs by path and SHA-256. Both runs report
unchanged inputs. Documentation integration follows measurement; no library or
benchmark implementation changes are made during this analysis.

| Component | Recorded configuration |
|---|---|
| Platform/build | Windows x64/MSVC, DX12, optimized independent C++ and Rust consumers |
| Toolchain | Rust 1.99.0; MSVC 14.44.35207; Windows SDK 10.0.22621.0 |
| AMD SDK | v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3` |
| GPU/driver | Radeon RX 9060 XT, adapter 0, driver `32.0.31041.1004` |
| Provider | 4.1.1, ID `17700776140664213505` |
| Clocks | QPC 10 MHz; GPU direct-queue timestamp frequency 100 MHz |
| Images | 1920×1080 output; queried 1280×720 maximum and 960×540 smaller render regions |
| Context/profile | Local ownership, dynamic resolution enabled, explicit exposure and pre-exposure 1 |

The prior machine record identifies Ryzen 5 5600 and Windows 11 Pro build 26300;
CPU/OS details and power-plan state were not re-queried for this batch. CPU/GPU
clock states, temperatures, background activity and power were not controlled
or logged. No claim of thermal or frequency stability follows from duration.

The [retained checks](2026-10-16-exp-native-performance-confirmation/checks.json)
include signed-runtime/header hashes, executable hashes, toolchain observations,
source identities, case counts, timestamps and original run/inventory hashes.
The C++ executables have different build hashes between pilot and confirmation;
both were rebuilt from the same inventoried producer inputs. No binary identity
across those builds is claimed.

## Protocol, execution and reproduction

The [frozen protocol](2026-10-16-exp-native-performance-confirmation/protocol.json)
and [selection](2026-10-16-exp-native-performance-confirmation/selection.json)
retain the actual methods and complete randomized process order. A ten-block
pilot estimates noise; its measurements are discarded from confirmation.
The worst pilot CPU difference standard deviation is 12.154 µs. Runtime and
sample caps select 112 blocks with a planned one-sided margin of 1.905 µs;
the preferred 1 µs precision target is explicitly infeasible under that plan.
The 5 µs budget remains fixed. Selection is frozen before confirmation, with
seed `6101602`; there is no success-based early stop or optional extension.

Each block contains both consumers for all five workloads, in randomized order.
Each timed child submits and fences 160 frames, discarding frames 0–31 from
the primary estimate. It preserves context history, resetting only frame 0.
The transition workload shrinks at frame 64 and grows at 112; those frames
remain in its primary average. Other workloads are stable maximum, stable
smaller, both supplied masks, and separate linear HDR at maximum size.

CPU timing brackets the complete public Rust dispatch versus a prepared minimal
C++ native call. Rust validation and descriptor preparation are included;
caller-side argument construction, upload, query, submission, fences, readback
and logging are excluded. GPU timing brackets the provider's command interval
on the same direct queue. Serial fences and diagnostic gaps remain symmetric
but limit throughput and clock-state inference. Empty brackets are retained and
not subtracted. Independent debug controls require zero errors; timed children
do not enable those diagnostics and cannot establish zero timed debug errors.

The estimator is the equal-weight mean of **112 paired process-mean differences**
per workload, Rust minus C++. Its one-sided 95% Student t upper bound must be
strictly below 5 µs. Intervals are approximate, conditional on independent,
reasonably regular block differences. Frames within a process are not independent
inferential units. Bounds are pointwise; they are not a simultaneous 95% band.
The overall criterion requires every workload to pass. Two-sided 95% intervals
are supplementary. No sample trimming or post-hoc exclusions are applied.

From the repository root, the completed batch used:

```powershell
python tools/verification/performance_characterization.py run --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

This command starts new native work when run again. It needs the pinned trusted
SDK runtime, Windows/MSVC, the selected DX12 GPU, an otherwise unchanged checkout
and the [documented prerequisites](../../../tools/verification/README.md#native-performance-characterisation).
The retained protocol identifies the measured version; future source changes
require a fresh pilot and selection.

The completed local batch is
`target/performance-batches/20261016T122758Z-native-performance-5d9ac4cbd59e`.
Its [completion metadata](2026-10-16-exp-native-performance-confirmation/batch.json)
records exit code 0 for both stages:

| Stage | UTC on 2026-10-16 | Outcome |
|---|---|---|
| Pilot | 12:27:59–12:35:32 | 122/122 cases passed |
| Confirmation | 12:35:38–14:00:27 | 1,220/1,220 cases passed; 112 blocks complete |
| Whole batch | 12:27:58–14:01:01 | Completed in approximately 93 minutes, 14:27–16:01 Berlin time |

Primary measurement took 4,724.695 seconds, approximately 78.75 minutes; setup,
pilot and supplements explain the longer batch. Confirmation covers 1,120 primary
children, 77 supplemental children and ten four-frame debug children: **191,560
submitted/fenced frames**, including 143,360 primary post-warm-up dispatches.
The fresh pilot adds 16,040 frames; the whole batch totals 207,600.

## Observed CPU and GPU dispatch costs

All values below are **µs per dispatch**. Absolute totals average process means;
differences and intervals use paired blocks. The
[confirmation analysis](2026-10-16-exp-native-performance-confirmation/confirmation.json)
retains every primary/segment difference vector and verdict. The
[derived metrics](2026-10-16-exp-native-performance-confirmation/derived-metrics.json)
add absolute totals and descriptive sensitivity checks.

| Workload | C++ CPU | Rust CPU | CPU difference [two-sided 95% interval] | One-sided CPU upper 95% | Verdict |
|---|---:|---:|---:|---:|---|
| Stable maximum | 36.826 | 39.211 | +2.385 [1.925, 2.845] | 2.770 | Within 5 µs |
| Stable smaller | 36.289 | 38.668 | +2.379 [2.153, 2.604] | 2.568 | Within 5 µs |
| Shrink/grow | 36.683 | 39.066 | +2.383 [1.977, 2.788] | 2.722 | Within 5 µs |
| Supplied masks | 37.480 | 39.158 | +1.678 [0.962, 2.395] | 2.278 | Within 5 µs |
| Linear HDR | 36.682 | 38.861 | +2.179 [1.851, 2.507] | 2.454 | Within 5 µs |

| Workload | C++ GPU | Rust GPU | GPU difference [two-sided 95% interval] |
|---|---:|---:|---:|
| Stable maximum | 938.530 | 939.377 | +0.847 [−1.303, 2.997] |
| Stable smaller | 930.725 | 931.881 | +1.156 [−2.022, 4.334] |
| Shrink/grow | 939.242 | 939.504 | +0.262 [−2.081, 2.605] |
| Supplied masks | 940.398 | 939.420 | −0.978 [−3.260, 1.303] |
| Linear HDR | 939.340 | 941.048 | +1.707 [−0.605, 4.020] |

Every GPU interval includes zero. No GPU equivalence margin was selected;
these results establish neither zero GPU overhead nor a GPU equivalence verdict.
CPU results support the bounded average-cost statement, not a universal language
penalty or an every-frame/transition latency bound.

Pooled post-warm-up CPU medians/p95 below are descriptive, with 14,336 frames per
consumer/workload. P95 uses nearest rank; differences of these percentiles are
not confidence bounds on paired overhead.

| Workload | C++ median / p95, µs | Rust median / p95, µs |
|---|---:|---:|
| Stable maximum | 32.4 / 58.7 | 34.5 / 63.2 |
| Stable smaller | 32.6 / 53.0 | 34.8 / 56.4 |
| Shrink/grow | 32.4 / 58.3 | 34.6 / 61.9 |
| Supplied masks | 32.5 / 62.6 | 34.6 / 63.8 |
| Linear HDR | 32.5 / 56.7 | 34.5 / 60.5 |

Signed outliers are retained. Supplied-mask block differences range from −30.965
to +6.720 µs; its median block difference is +2.160 µs rather than its +1.678 µs
mean. Across workloads, first-half mean differences range from 1.420 to 2.581 µs
and last-half means from 1.937 to 2.827 µs. These exploratory splits do not alter
the frozen estimate, prove stationarity or replace the uncertainty model.

## Context lifecycle and first use

Lifecycle uses eight fresh-process pairs per SDR/HDR configuration. Each process
also measures eight subsequent unused-context create/destroy pairs. Inference
uses **eight paired process means**, not 64 independent contexts. Times below
are **ms per context**, with Rust-minus-C++ two-sided 95% intervals.

| Context | Boundary | C++ | Rust | Difference [95% interval] |
|---|---|---:|---:|---:|
| SDR | First create after provider query | 50.416 | 50.483 | +0.067 [−1.341, 1.475] |
| SDR | Warm unused-context create | 50.624 | 50.618 | −0.006 [−0.508, 0.497] |
| SDR | Used-context destroy after fence | 5.973 | 4.763 | −1.210 [−2.939, 0.518] |
| SDR | Unused-context destroy | 3.245 | 3.272 | +0.027 [−0.077, 0.131] |
| HDR | First create after provider query | 49.893 | 50.627 | +0.734 [−0.264, 1.732] |
| HDR | Warm unused-context create | 50.634 | 50.472 | −0.162 [−0.859, 0.535] |
| HDR | Used-context destroy after fence | 4.882 | 4.896 | +0.014 [−0.312, 0.340] |
| HDR | Unused-context destroy | 3.228 | 3.242 | +0.014 [−0.146, 0.174] |

Creation/destruction intervals all span zero; there is no lifecycle speed gate.
Fresh-process first creation follows a provider/render query and is **not a
cache-cold start**. Loader acquisition/symbol resolution and the pre-create
query are separate boundaries; neither is included in context creation. Across
SDR/HDR, C++ load means are 0.630–0.640 ms and Rust 0.515–0.527 ms; provider-query
means are 65.428–66.745 ms and 64.912–65.652 ms. Different integration boundaries
prevent treating the shorter Rust loader interval as a causal language advantage.
Used destruction excludes the already-completed GPU wait. C++ warm creation
reuses descriptors; Rust suppresses trace environment lookup but retains a
test-only TLS branch and allocation hooks. Full differences/intervals remain in
the [supplemental analysis](2026-10-16-exp-native-performance-confirmation/supplements.json).

First-dispatch CPU recording is also much larger than warmed recording: SDR C++
9.367 ms versus Rust 9.494 ms, difference +0.127 [−0.177, 0.431] ms; HDR 9.431
versus 9.485 ms, +0.054 [−0.136, 0.245] ms. These secondary eight-pair results
exclude subsequent submission and GPU completion.

## Preparation, allocation and transition diagnostics

Three separate preparation processes per workload yield process-mean ranges
2.186–2.476 µs (maximum), 2.280–2.811 (smaller), 2.317–2.484 (transition),
2.605–2.655 (masks), and 2.345–2.363 (HDR). Prepared raw Rust process means range
from 34.127 to 39.095 µs across workloads. These diagnostics run separately;
the raw path retains an extra unused wrapper query context. They cannot be
subtracted or added to reconstruct the primary overhead algebraically.

The 15 separate allocation processes cover three repetitions of all five
workloads, each with 128 warmed dispatches: **zero observed allocation,
deallocation, zeroed-allocation, reallocation or requested-byte events across
1,920 calls**. Instrumentation covers the dispatch calling thread's Rust
executable allocator, excluding other threads and native/driver heaps. It is
not a production-wide no-allocation guarantee.

First-transition versus same-index stable-control comparisons are exploratory,
separate from the Rust-minus-C++ budget. GPU shrink means are +16.70 µs for C++
[−8.25, 41.64] and +17.01 µs for Rust [−14.09, 48.11]; grow means are +18.73
[−15.95, 53.42] and −10.20 [−39.53, 19.12], respectively. All these intervals
and the corresponding CPU intervals span zero. The larger sample therefore
does not establish a consistent causal transition penalty. Retained primary
first-transition, settling and steady segment intervals remain secondary.

## Offline audit and retained evidence

No new native execution occurs during analysis/integration. Both complete local
run inventories are SHA-256 verified. Offline parsing rechecks **1,197 primary
and supplemental native logs** against saved sample matrices, canonical
descriptors, provider/device/cohort compatibility and mask/HDR checkpoints.
Recomputation matches all 80 saved primary/segment CPU/GPU estimates, 12 saved
lifecycle estimates and the exact frozen primary process order. Original native
evaluations also check stable-control histories and separate debug controls.
The [audit summary](2026-10-16-exp-native-performance-confirmation/audit.json)
states the boundary of this verification.

The adjacent compact artefacts retain protocol/selection, identities, outcomes,
all paired difference vectors, process mean/median/p95/first-dispatch/warm-up
summaries and final output hashes. Full native logs, raw frame ticks and raw
GPU images remain local under ignored `target/`; they are not published in this
record. Hashes cannot reconstruct past image bytes, and future reruns need not
produce identical timings or images. Paths are sanitized to `<repo>`/`<run>`;
no transient pointer addresses are retained. The compact
[inventory](2026-10-16-exp-native-performance-confirmation/artifacts.json)
identifies retained bytes. Recheck retained arithmetic and inventory without a
GPU or benchmark run:

```powershell
python docs/research/records/2026-10-16-exp-native-performance-confirmation/reanalyze.py
```

Before this successful batch, local pilot attempts at 00:20:57 and 00:21:23 UTC
did not complete. One retains an unfinished `running` status; the other fails
while replacing `result.json.pending` with `result.json` with Windows access
denied (error 5), before primary measurement. The producer now retries transient
result-file replacement failures. The underlying cause is not established.
Their compact [attempt index](2026-10-16-exp-native-performance-confirmation/attempts.json)
preserves those outcomes; neither is a speed result. Earlier failures and the
explicitly stopped confirmation remain in the unchanged baseline record.

## Baseline delta and remaining limits

The earlier pilot supplied descriptive measurements and no confirmed speed
verdict. This independent completed confirmation now supports **less than 5 µs
average additional CPU dispatch cost on each of these five matched workloads
on the recorded Windows/GPU/provider stack**. Achieved one-sided margins are
0.189–0.600 µs despite the noisier planning pilot; no plan or budget is changed
after observing confirmation. Pilot and confirmation noise differ; neither
precision nor overhead estimates should be assumed portable to another run.

This completes the selected bounded characterization checkpoint. It does not
establish FPS, every-frame latency, GPU equivalence, truly cold-cache startup,
image quality, actual mask consumption, other adapters/providers/backends,
movable ownership performance or release-archive acceptance. Broader hardware,
non-linear colour and other features require separately directed questions.
The full source CI/MSRV checks reported by the baseline concern its identified
earlier producer state; this record does not represent a new full-CI or release
verification on the completed batch's source state.
