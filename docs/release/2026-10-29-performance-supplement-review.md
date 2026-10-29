# P08 supplemental regression review — 2026-10-29

Status: **review complete; maintainer disposition of shared absolute cost
increases pending**. No additional native measurement was run. CPU temporal
selection remains as documented in the [preceding review](2026-10-29-performance-time-window-review.md).
Final candidate/archive qualification and publication authorization remain
separate. P08 is not marked closed while the disposition below is outstanding.

## Conclusion and proposed disposition

The current matched Rust/C++ observations establish no additional wrapper-specific
GPU or lifecycle slowdown. All 1,920 instrumented warmed recording calls report
zero Rust allocator events/bytes. Setup queries complete successfully and are
observably cheaper than the historical P07 deployment. The independent current
C++ reference and Rust consumer have similar GPU and context-lifecycle costs;
this is not an equivalence proof or a universal speed promise.

A material absolute change must nevertheless be visible: both consumers now take
about 9.8–11.3% longer GPU intervals than historical P06, context creation is
roughly 63–67 ms rather than 50 ms, and unused-context destruction is roughly
13.4–13.6 ms rather than 3.2–3.3 ms. The initial P08 run already observed this
shift under the current driver/provider; no crate source changed between P08
runs. The old provider/driver and some native helper inputs differ, so this does
not isolate the cause or establish a Rust API regression.

**Recommended disposition:** accept these shared absolute costs as an observed
limitation of the recorded current deployment, retain the comparison/limits in
release evidence, and close the bounded P08 review after the maintainer agrees.
No speculative wrapper change or additional long measurement is proposed. This
recommendation has not been recorded as acceptance: [D043](../adr/D043.md)
requires a material regression to be resolved or explicitly accepted by the
maintainer before release closure.

## Inputs and audit boundary

The [compact review data](2026-10-29-performance-supplement-review.json) retains
all setup samples, allocation counters, preparation/raw process means, eight-pair
lifecycle differences and process means, GPU summaries/differences, historical
comparisons, source/package identities and audited artifact hashes.

- Control: 20261028T204807Z-release-performance-9ebf070a378f.
- Native run: 20261028T204914Z-release-performance-reconfirm-75f83c242284.
- Producer source identity:
  9e2409a86f0604fb4f7e01ca22641a72059a43a6bafb33e38d32d6dccd7f682a.
- Full fixed run: 2,660/2,660 cases passed, plus three 11-case setup runs; inputs
  unchanged. Current producer identity still matches that recorded identity.
- SDK v2.3.0, Windows x64/MSVC + DX12, Radeon RX 9060 XT adapter 0;
  driver 32.0.32015.7029, provider 4.1.2 *, ID 0xf5a5ca1e81001002.
- Wrapper candidate SHA-256:
  f55b3a2b46b96406fdbfeb82ef6ff179b3a4469b90c1198b318119b83adb204f.
- Raw candidate SHA-256:
  0bb92f048d27b67d74a3876bc78b5016ac33452640ffa17cffdab77179230bb0.

The review checked 254 relevant artifact entries against their retained SHA-256
and size inventory, reparsed all 77 supplemental process logs (45 preparation/
raw/allocation, 32 lifecycle), and recomputed their summaries exactly. All three
setup telemetry streams reproduce the saved 192 timings, medians and paired
differences. Original CPU/GPU paired primary differences had already been
cross-checked against all 2,560 primary case summaries in the temporal review.
The same retained blocks 35–256 are used for primary GPU observations here.
No new primary exclusions are introduced.

Raw GPU readbacks were not rehashed again in this review. The completed
coordinator previously verified the run; this is an offline metadata/log/arithmetic
review, not a new native execution or exact-package qualification. No samples
from historical or initial P08 runs are pooled. Documentation/derived-data edits
change no measured production code, native consumer, protocol or producer.

## GPU intervals and transitions

Mean provider command intervals below are microseconds. Differences are paired
Rust minus C++ over the same 222 retained blocks. Intervals are nominal two-sided
95% Student t intervals, subject to the previously disclosed post hoc selection.

| Workload | C++ mean | Rust mean | Difference [nominal 95% interval] |
|---|---:|---:|---:|
| Maximum render | 1044.228 | 1044.299 | +0.071 [-2.168, 2.309] |
| Small render | 1022.336 | 1022.890 | +0.554 [-1.356, 2.463] |
| Shrink/grow | 1041.975 | 1043.946 | +1.971 [-0.252, 4.195] |
| Supplied masks | 1042.696 | 1043.373 | +0.677 [-1.273, 2.626] |
| Linear HDR | 1042.982 | 1043.894 | +0.912 [-1.130, 2.954] |

All intervals span zero. Mean differences are 0.007–0.189% of the corresponding
C++ interval. There is no selected GPU equivalence margin or numerical GPU gate;
these results do not prove equivalence, FPS or throughput. The historical
[P06 means](../research/records/2026-10-16-exp-native-performance-confirmation.md)
were about 931–941 us in both consumers. Current means are about 1022–1044 us,
a 9.8–11.3% increase in both. Initial P08 already measured about 1031–1060 us.

First shrink/grow versus the same-index stable maximum control also remains
inconclusive. GPU differences in us: C++ shrink -6.737 [-42.353, 28.879], grow
+26.877 [-6.291, 60.045]; Rust shrink +7.920 [-28.700, 44.540], grow -2.499
[-36.425, 31.428]. These exploratory comparisons do not identify a consistent
transition penalty. All first-transition and later excursions remain included.

## Allocations and preparation

All 15 allocation processes (three repetitions of five workloads) retain 128
warmed recording calls each. Every allocation/deallocation/zeroed-allocation/
reallocation/requested-byte counter, including shrink/grow checkpoints, is zero:
**1,920 calls, zero observed events or requested bytes**. This covers only the
calling thread's Rust executable allocator. It makes no native/driver heap,
other-thread or production-wide no-allocation claim.

Separate descriptor/preparation process-mean ranges in us:

| Workload | Current three-process range | Historical P06 range |
|---|---:|---:|
| Maximum render | 4.127–4.205 | 2.186–2.476 |
| Small render | 4.102–4.245 | 2.280–2.811 |
| Shrink/grow | 4.081–4.233 | 2.317–2.484 |
| Supplied masks | 4.442–4.806 | 2.605–2.655 |
| Linear HDR | 4.111–4.267 | 2.345–2.363 |

Preparation is observably higher than that historical configuration; it remains
inside the complete public CPU boundary evaluated in the temporal review, whose
retained nominal upper bounds all remain below the selected 5 us budget. The
historical/current API, provider/driver and native workload environment differ,
so the separate diagnostic does not isolate a causal API-only delta. Prepared
raw Rust means span 45.737–51.109 us (historical 34.127–39.095 us). Raw mode
retains an additional unused wrapper context for queries. Separate diagnostic
processes cannot be subtracted from, or added to, the primary estimate to claim
an isolated cost or speed advantage.

## Default and selected-provider setup

All samples are retained, including the 112.597 ms selected-path sample in the
third process. These separate pre-dispatch setup processes are not subject to
the primary temporal exclusion. Values are medians in ms per complete six-query
sequence, not per query or frame.

| Fresh process | Default median | Selected median |
|---|---:|---:|
| 1 | 18.525 | 50.025 |
| 2 | 20.439 | 56.432 |
| 3 | 19.166 | 51.720 |

The current explicit-selection route is more expensive than the default route;
there is no causal attribution to enumeration alone. Historical
[P07](../research/records/2026-10-19-exp-provider-setup-cost.md) medians were
229.55–230.74 ms default and 696.98–707.10 ms selected; initial P08 medians were
17.41–18.11 ms and 46.14–48.16 ms. Current observations reveal no absolute setup
regression against the historical deployment, but do not prove that the new
Rust API alone caused the lower cost. No cache/session optimization or setup
speed threshold was selected.

## Context lifecycle and first use

Each context type has eight paired fresh processes; each process also measures
eight subsequent unused create/destroy pairs. The inferential unit is the process
mean, not 64 independent contexts. Values below are ms; differences are Rust
minus C++ with two-sided 95% intervals. No primary time filter is applied to these
separately collected lifecycle processes.

| Context | Boundary | C++ mean | Rust mean | Difference [95% interval] |
|---|---|---:|---:|---:|
| SDR | Loader acquisition | 0.709 | 0.590 | -0.119 [-0.203, -0.034] |
| SDR | First provider/render query | 74.206 | 79.027 | +4.821 [-1.796, 11.437] |
| SDR | First create after query | 63.591 | 66.303 | +2.712 [-0.433, 5.857] |
| SDR | Warm unused create | 65.653 | 66.986 | +1.333 [-1.696, 4.361] |
| SDR | Used destroy after fence | 15.441 | 15.166 | -0.275 [-1.194, 0.644] |
| SDR | Unused destroy | 13.574 | 13.579 | +0.005 [-0.481, 0.492] |
| SDR | First dispatch recording | 11.037 | 11.872 | +0.835 [-0.996, 2.667] |
| HDR | Loader acquisition | 0.733 | 0.580 | -0.153 [-0.236, -0.070] |
| HDR | First provider/render query | 79.410 | 76.354 | -3.056 [-11.085, 4.973] |
| HDR | First create after query | 67.077 | 65.831 | -1.247 [-5.400, 2.906] |
| HDR | Warm unused create | 64.789 | 65.261 | +0.472 [-2.786, 3.730] |
| HDR | Used destroy after fence | 15.002 | 14.939 | -0.063 [-0.522, 0.395] |
| HDR | Unused destroy | 13.422 | 13.412 | -0.010 [-0.610, 0.590] |
| HDR | First dispatch recording | 11.270 | 10.856 | -0.414 [-0.987, 0.158] |

All query/create/destroy/first-dispatch intervals span zero. Loader boundaries
are different integration paths and do not establish a causal language advantage.
First create follows the measured provider query and is not cache-cold. Used
destruction excludes already-completed GPU waits. Warm Rust creation retains the
existing test-only trace/TLS/allocator instrumentation; the C++ path reuses its
descriptors. Broad intervals do not establish lifecycle equivalence.

The material historical shift, shown as ranges of SDR/HDR means rather than
uncertainty intervals, is:

| Boundary | Historical C++ / Rust (ms) | Current C++ / Rust (ms) |
|---|---|---|
| First create | 49.893–50.416 / 50.483–50.627 | 63.591–67.077 / 65.831–66.303 |
| Warm unused create | 50.624–50.634 / 50.472–50.618 | 64.789–65.653 / 65.261–66.986 |
| Used destroy | 4.882–5.973 / 4.763–4.896 | 15.002–15.441 / 14.939–15.166 |
| Unused destroy | 3.228–3.245 / 3.242–3.272 | 13.422–13.574 / 13.412–13.579 |

Initial P08 already recorded first creation around 63–64 ms, warm creation around
63 ms and unused destruction around 13.3 ms in both consumers. There were no
crate-source changes between the two P08 measurements; the same provider/driver
was observed. This is not solely an artifact of the long run's noisy beginning.
It remains an observed absolute change requiring disposition, even though no
additional Rust-specific penalty is established by the current matched pairs.

## Comparability, checks and remaining decision

Historical P06 used driver 32.0.31041.1004/provider 4.1.1; current P08 uses
32.0.32015.7029/4.1.2 *. The signed loader/upscaler DLL hashes and the independent
C++ consumer's gpu_reference.cpp and performance_reference.h hashes match those
historical inputs. The shared native gpu.cpp helper and Rust/API inputs changed;
clock/thermal/background state was not controlled or logged. The combination
prevents attributing absolute changes solely to the driver, provider, helper or
wrapper. Timing data does not prove that any retained period was idle.

Offline checks reparse complete telemetry, verify relevant inventory entries,
reproduce process/lifecycle/setup estimates, retain all diagnostic outliers and
use the original common CPU/GPU temporal selection. Recalculation, local links,
JSON, whitespace and the repository-size budget are checked for this documentation
and data change; no Rust build or GPU rerun is needed.

The remaining maintainer choice is whether to accept the observed shared current
GPU/context costs as the deployment baseline for bounded 0.3.0 verification.
Acceptance would close the performance review with its stated limits, followed
by final candidate/archive qualification. It would not prove causation, upgrade
the post hoc subset into an independent pass, or authorize publication.
