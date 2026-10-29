# Performance: wrapper costs and native comparison

- Updated: 2026-10-29, rc1 follow-up integrated and final P08 input reconciliation
  accepted by the maintainer;
  P01–P07 measurements and their original source identities remain unchanged.
- Scope: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2` for P01–P06 and
  `fsr-sdk-sys 0.2.0` for P07, SDK v2.3.0
  (`60f4ea81909200d8542eca14dccb2628b763a9a3`), Windows x64/MSVC + DX12.
  Measurements cover a small fixed-size CPU baseline/comparison, one 1080p
  CPU/GPU pilot, active-render shrink/grow timing and selected supplied-mask/
  linear-HDR comparisons, with explicit exposure and local ownership on one
  GPU/driver/provider. Completed P06 adds 112 randomised confirmation blocks:
  all five workloads meet the selected 5 µs upper budget on average additional
  public Rust CPU dispatch cost. GPU equivalence, every-frame latency,
  non-linear colour and broader feature/hardware coverage remain separate.
  P07 separately measures default/selected six-query setup paths without GPU dispatch.
  Current P08 covers the selected 0.3.0/0.1.2 candidate on driver 32.0.32015.7029
  with default provider 4.1.2 *; its accepted temporal subset and the separate
  explicit-4.1.1/default comparison retain their own methods and limits.
- Included records: [current-wrapper CPU experiment](records/2026-10-15-exp-wrapper-cpu-performance.md)
  (P01), [performance-testing source investigation](records/2026-10-15-src-performance-testing-strategy.md),
  [independent C++ CPU comparison](records/2026-10-15-exp-independent-cpp-cpu-comparison.md)
  (P03), [1080p GPU timestamp pilot](records/2026-10-15-exp-1080p-gpu-timing.md)
  (P04), [active-render transition timing](records/2026-10-16-exp-active-render-timing.md)
  (P05) and [supplied-mask/linear-HDR comparison](records/2026-10-16-exp-mask-hdr-performance-comparison.md)
  (P02), and [native characterisation implementation/pilot](records/2026-10-16-exp-native-performance-characterization.md)
  (P06), [completed native performance confirmation](records/2026-10-16-exp-native-performance-confirmation.md)
  and [provider setup-query cost](records/2026-10-19-exp-provider-setup-cost.md) (P07).
  Completed historical records are preserved unchanged. This integration audits
  saved results without further native execution or external source research.
- Current release records: [initial P08 results/plan](../release/2026-10-28-performance-confirmation-plan.md),
  [temporal review](../release/2026-10-29-performance-time-window-review.md),
  [supplemental review](../release/2026-10-29-performance-supplement-review.md),
  [provider probe preparation](records/2026-10-29-exp-provider-performance-preparation.md),
  [completed provider comparison](records/2026-10-29-exp-provider-performance-comparison.md)
  and [maintainer disposition/closure](../release/2026-10-29-performance-review-closure.md).
  Preparation establishes compilation only; the separate completed comparison
  establishes actual provider selection and valid GPU execution.

## Release verification and accepted P08 review — 2026-10-29

[D043](../adr/D043.md) requires a reproducible candidate performance check and
regression review before **every release**. P08 is now closed on its recorded
0.3.0 candidate as a [bounded, maintainer-accepted review](../release/2026-10-29-performance-review-closure.md),
following the explicit acceptance of the remaining provider/deployment differences
as non-blocking. The subsequent
[common rc1 host qualification](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
passes exact-package functional checks on Windows, Deck and Machine. It supplies
no new timings and does not transfer P08's archive identity to rc1. The subsequent
[rc1 timing follow-up](records/2026-10-29-exp-rc1-performance-check.md) supplies
122/122 passing execution checks, ten complete pairs/workload and captured
source hashes matching all ordinary rc1 package members. The maintainer
[accepts the combined source/performance assessment](../release/2026-10-29-rc1-performance-acceptance.md):
final P08 input reconciliation is complete; publication simulation now passes.
No further benchmark is selected. The original full-run verdicts and accepted
limits below remain unchanged.

**Accepted performance claim on the recorded workloads/deployment:** average
added public Rust CPU recording cost below 5 us; P08's pointwise nominal
one-sided 95% upper bounds are **4.046–4.618 us** in its accepted 222-block
subset. Post-selection coverage is unestablished, as disclosed below. These
Student t bounds concern average added cost, not per-frame percentiles or a
two/three-sigma frame limit. The fresh rc1 pilot has mean differences
**3.009–4.841 us**, with no new formal budget verdict. No wrapper-specific GPU
overhead or final-cleanup regression is detected. The rc1 GPU difference
intervals all include zero; the acceptance record retains their numerical
bounds without claiming exact equivalence. All ten rc1 blocks are retained.

The first batch passed 515 execution checks with an inconclusive 26-block CPU
result. The fixed batch passed 2,660 confirmation cases plus three 11-case setup
runs with unchanged inputs; its full 256-block CPU result also remains
inconclusive. The accepted post hoc temporal exclusion retains 222 balanced
blocks/workload with nominal upper CPU bounds of 4.046–4.618 us against the
unchanged 5 us budget. Post-selection 95% coverage is not established. Original
samples/verdicts remain unchanged; closure does not relabel either full run as
an unmodified confirmatory pass or pool historical samples.

The supplemental review retains zero observed Rust allocator events in 1,920
warmed calls, small or uncertain current matched Rust/C++ GPU/lifecycle differences,
and shared absolute historical cost increases. The maintainer accepted these
limits for 0.3.0 after reviewing the provider comparison below. No further
performance experiment is selected as a release blocker. Later changes to
measured runtime/consumer inputs still require impact assessment. The subsequent
[shared rc1 qualification](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
records exact package identities and completed extracted-package/native coverage.

## Provider comparison on the current driver

The [eight-pair C++ experiment](records/2026-10-29-exp-provider-performance-comparison.md)
passed 30/30 cases in 95.477 seconds on driver 32.0.32015.7029, with actual
provider IDs verified and common inputs/clocks/history. The fixed maximum-render
SDR workload uses 1920x1080 output. These are FSR command intervals and context
boundaries, not complete frame times or added Rust overhead.

| Metric | Explicit 4.1.1 | Default 4.1.2 * | Default minus explicit, descriptive 95% interval |
|---|---:|---:|---:|
| Warmed GPU interval | 1.047 ms | 1.059 ms | +11.503 us [-5.392, 28.398] |
| First context creation | 53.555 ms | 58.127 ms | +4.571 ms [3.728, 5.415] |
| Unused context destruction | 3.586 ms | 7.679 ms | +4.093 ms [4.002, 4.185] |

**Verified:** the GPU mean difference is about 1.10%, with its interval spanning
zero; no clear GPU difference is established by this small sample. The selected
context-cost intervals are positive in this path. Enumeration precedes the
pre-create query in both arms, warming startup state differently from original
P08. First-use absolute times therefore do not share an identical P08 startup
boundary. No GPU equivalence, universal provider-cost or image-quality claim follows.

**Inference:** default provider selection alone does not explain the historical
roughly 10% GPU increase: explicit 4.1.1 still measures about 11.58% above the
old P06 maximum-render C++ GPU mean. The driver changed from 32.0.31041.1004 to
32.0.32015.7029; helper inputs and uncontrolled environment also differ. The
maintainer reported the animated wallpaper disabled for the new comparison,
without a controlled on/off test. Driver, wallpaper or other shared-state
causation remains unresolved and is accepted as outside release closure. This
integration adds no new native work or automatic provider-selection policy.

## Current findings

**P08 — complete bounded candidate review with accepted exceptions:** the
[closure](../release/2026-10-29-performance-review-closure.md) records the temporal
subset, all supplementary measurements and the completed provider experiment.
Acceptance is specific to the measured source/deployment; final archives and
registry-only native consumers remain separate. Historical P01–P07 findings below
retain their original versions and are not current-candidate substitutes.

**P01 — verified bounded CPU baseline:** the CPU experiment completed all 17 protocol
cases, including pinned SDK/ABI prerequisites, a separate debug-enabled native
control and three controlled/native process repetitions. Its
[retained measurements](records/2026-10-15-exp-wrapper-cpu-performance/measurements.json)
and [verdict](records/2026-10-15-exp-wrapper-cpu-performance/run.json)
provide a bounded P01 baseline, not a pre-M14 measurement or release-archive test.

| Measurement | Recorded result and boundary |
|---|---|
| Native complete wrapper CPU recording | Per-process medians 35.5–40.2 µs; excludes setup, submission, fences and readback |
| Prepared raw SDK call through Rust | Per-process medians 33.9–38.5 µs; shares creation mapping and skips dispatch inspection |
| Paired wrapper-minus-raw CPU difference | Per-process medians 1.3–1.4 µs; noisy, reference-dependent observations with negative individual differences |
| Native inspection/descriptor construction | Per-process medians 1.7–2.0 µs; independently timed and not algebraically additive to raw-call medians |
| Controlled warmed dispatch | Zero observed Rust-executable allocations; 42 external COM entries, not measured native heap/driver allocation counts |
| Native lifecycle | Creation medians 66–70 ms, destruction 3.1–3.2 ms; includes native SDK work and test-only teardown tracing environment lookup |

P01's results describe a fixed `320×180` render / `640×360` output workload on
Ryzen 5 5600, Radeon RX 9060 XT, driver `32.0.31041.1004`, provider `4.1.1`.
Actual GPU work was submitted and fenced; only CPU intervals were timed. They
establish neither GPU execution time nor FPS, independent C++ parity, production
allocation guarantees or a universal wrapper penalty. Full conditions and
outliers remain in the [experiment record](records/2026-10-15-exp-wrapper-cpu-performance.md#protocol-and-reproduction).

**P03 — verified comparison execution, inconclusive CPU performance:** the
[independent C++ comparison](records/2026-10-15-exp-independent-cpp-cpu-comparison.md)
completed all 45 cases. Its 33 children cover separate debug controls, six
balanced process blocks of C++/Rust-raw/Rust-wrapper and three warm-state control
blocks. They match canonical descriptors/scalar bits, provider, adapter/driver,
QPC frequency and final output within each history-length cohort. The optimized
C++ executable calls the SDK independently; shared equipment supplies resources,
submission, fences, clocks and readback.

For the same small render/output dimensions as P01, Rust-wrapper minus C++
process-median CPU differences range from **−6.4 to +9.4 µs**; Rust-raw minus C++
ranges from **−12.5 to +4.3 µs**. The sign changes prevent a precise overhead or
parity conclusion. The warm-state controls also vary and do not identify a
causal cache-warming effect. The
[retained block analysis](records/2026-10-15-exp-independent-cpp-cpu-comparison/comparison.json)
preserves these outcomes; earlier attempts remain documented in the record.

P03 does not contradict or numerically correct P01's 1.3–1.4 µs paired result.
P01 interleaves two contexts within one process and pre-inspects before each pair;
P03 prepares raw metadata once, compares separate processes and uses a common
QPC helper clock. C++ also omits Rust validation. Those different reference,
timer and sampling conditions preclude pooling the results or treating their
difference as isolated language overhead.

**P04 — verified bounded GPU timing:** the
[1080p pilot](records/2026-10-15-exp-1080p-gpu-timing.md) completed all 22 cases,
including two four-frame debug controls and four balanced Rust/C++ process
blocks. Both consumers independently obtain **1280×720** render dimensions for
**1920×1080** output from SDK Quality queries, confirm live-context agreement,
and use the provider's 18-sample jitter sequence with coherent translating colour
and current-to-previous motion. Provider, device/driver, canonical descriptors,
query/jitter results and final output agree within each history-length cohort.
P04 also records a complete 45-case P03 regression after shared-consumer
extraction; that regression does not replace P03's original measurements.

| P04 measurement | Recorded result and boundary |
|---|---|
| GPU FSR command interval | Process medians **0.93808–0.94564 ms**; same direct queue, including provider-recorded barriers, excluding fixture uploads/transitions, resolve and output readback |
| GPU Rust-wrapper minus C++ | Nearby process-median differences **−0.40 to +1.08 µs**; descriptive, without an equivalence margin or speed verdict |
| CPU Rust-wrapper minus C++ | Process-median differences **−1.4 to +8.2 µs**, still sign-changing; QPC dispatch interval excludes timestamp recording and submission/fence/readback |
| Empty GPU brackets | Process medians **40–80 ns**; not subtracted from FSR intervals |

P04 uses the same recorded CPU/GPU/driver/provider configuration as P01/P03.
GPU queue frequency is checked at **100 MHz**, separately from the **10 MHz**
QPC clock. Each timed child retains 32 warm-up and 128 measured frames, with
fences before query/list/upload reuse. The
[block distributions](records/2026-10-15-exp-1080p-gpu-timing/comparison.json) and
[all process samples](records/2026-10-15-exp-1080p-gpu-timing/processes.json) retain
minima, tails and raw ticks. Timed GPU minima reach 660.72 µs despite medians
near 945 µs. Close medians therefore do not imply constant-time execution or
establish a causal language advantage. Serial fencing and CPU-generated uploads
leave idle gaps; clocks, thermal state and unrelated activity are uncontrolled.
This pilot measures a command interval, not FPS, sustained throughput,
presentation latency, general GPU-work coverage or reconstruction quality.

**P05 — verified bounded transition timing, inconclusive transition penalty:** the
[active-render experiment](records/2026-10-16-exp-active-render-timing.md) passes
all 42 cases: six debug controls and 24 timed Rust/C++ processes in four balanced
blocks, covering **3,864 submitted/fenced frames**. SDK queries select 1280×720
maximum and 960×540 smaller active regions, with 1920×1080 output. All three
workloads enable dynamic resolution and retain identical maximum input/output
allocations: stable maximum, stable smaller and maximum → smaller → maximum.
Timed contexts shrink at frame 64 and grow at 112, reset only frame 0 and preserve
a fixed output-space scene trajectory with matching UV motion and size-specific
queried jitter. The tested GPU/driver/provider configuration matches P04's.

Complete input readbacks match uploads and inactive guards; independent analytic
checks audit sampled GPU colour/depth/motion values. Checkpoint inputs and jitter
match the corresponding stable-size controls. Outputs match before shrink while
histories are identical, and across consumers/repetitions within each workload
and history-length cohort. Separate debug children require zero errors; timed
debug-error counts are marked unavailable, with device-removal checks retained. The
[stratified summary](records/2026-10-16-exp-active-render-timing/final/summary.json)
keeps CPU recording, GPU command intervals, first-transition samples and later
segments separate, alongside all three controls.

First-shrink GPU differences versus the same-index stable smaller control range
from **+56.12 to +831.72 µs for C++** and **−122.32 to +334.92 µs for Rust**.
First-grow differences versus stable maximum range from **−8.40 to +336.84 µs**
and **−49.56 to +345.24 µs**, respectively. Each first-transition value is one
frame per process; these ranges are descriptive observations. CPU differences
and later segment comparisons also vary. Serial fences, uploads and checkpoint
readback/hash gaps leave clock and between-process effects unresolved. The
measurements establish the bounded checkpoint, without a consistent transition
penalty, causal wrapper explanation, speed-parity or general throughput claim.

**P02 — verified bounded feature comparison, inconclusive speed parity:** the
[supplied-mask/linear-HDR experiment](records/2026-10-16-exp-mask-hdr-performance-comparison.md)
passes all **62 cases**, including ten debug and forty timed Rust/C++ children:
**50 native processes and 6,440 submitted/fenced frames**. Four balanced blocks
retain P05's stable maximum, stable smaller and shrink/grow controls, adding both
supplied masks together and a separate linear-HDR case at the maximum active
size. The tested GPU/driver/provider matches P04/P05. Every context enables
dynamic resolution, with fixed maximum allocations and 1920×1080 output;
explicit exposure and pre-exposure remain 1.

The mask case supplies two static render-aligned R8_UNORM rectangles, uploaded
once before timing. Complete GPU mask readbacks match uploads and independent
integer-pattern hash predictions. Linear HDR enables the native creation bit and
scales the input ramp's RGB by 8, preserving alpha and other scene inputs.
Independent sampled input checks and finite/written output checks pass, with
maximum output red above 1 and below 64 at every checkpoint. Provider/device,
descriptors, frame histories, input diagnostics and final outputs match between
consumers/repetitions within each workload/history-length cohort. Debug children
report zero errors; timed debug-error counts are unavailable.

| P02 Rust-wrapper minus C++ process medians | CPU range | GPU range |
|---|---:|---:|
| Supplied masks | −0.10 to +9.10 µs | −2.84 to +2.00 µs |
| Linear HDR | −7.80 to +16.20 µs | −0.44 to +4.24 µs |

These four-block ranges are descriptive. The
[retained summary](records/2026-10-16-exp-mask-hdr-performance-comparison/native/summary.json)
keeps all five workloads, feature-minus-SDR differences and stratified transition
measurements. CPU recording and provider GPU intervals use P04/P05's separate
brackets; uploads, submission/fences and diagnostics remain outside them.
Feature-minus-SDR differences combine different native workloads, validation and
between-process conditions, so they do not isolate wrapper overhead. Uncontrolled
clocks, serial fencing and diagnostic gaps remain interpretation limits.

Masked final output hashes also equal the SDR maximum controls in all eight
timed consumer/block pairs. This observation establishes neither mask consumption
nor a general lack of mask effect. The HDR case covers linear input without
non-linear hints or variable exposure. P02 completes the selected bounded
comparison checkpoint, without establishing an equivalence margin, reconstruction
quality, broader M16 semantics or final release-archive acceptance. Its P03/P04
regressions, source CI and Rust 1.88 MSRV checks pass; actual timed GPU execution
uses Rust 1.99.0. Original P01/P03–P05 measurements remain separate.

**P06 — verified bounded average CPU budget and characterisation:** the
[completed confirmation](records/2026-10-16-exp-native-performance-confirmation.md)
passes all **1,220 cases**, following a separate 122-case fresh pilot. Its
112-block selection and randomised order are frozen before measurement; pilot
and earlier interrupted samples are excluded. The whole batch takes approximately
93 minutes, including 78.75 minutes of primary measurement. Confirmation covers
191,560 submitted/fenced frames, including 143,360 primary post-warm-up calls;
the inferential unit is a paired process block, **112 per workload**, not a frame.

The primary criterion is a pointwise one-sided 95% Student t upper bound strictly
below **5 µs average additional public CPU dispatch cost** on every workload.
Complete public Rust dispatch includes validation/descriptor preparation; the
minimal C++ call does not supply equivalent validation. Uploads, submission,
fences, readback and caller-side input construction are outside the CPU bracket.
The following values are **µs per dispatch**, based on paired process means.
GPU intervals are two-sided 95% and descriptive, with no equivalence margin.

| Workload | Native C++ CPU | Rust CPU | CPU difference | One-sided CPU upper 95% | GPU difference [95% interval] |
|---|---:|---:|---:|---:|---:|
| Stable maximum | 36.826 | 39.211 | +2.385 | 2.770 | +0.847 [−1.303, 2.997] |
| Stable smaller | 36.289 | 38.668 | +2.379 | 2.568 | +1.156 [−2.022, 4.334] |
| Shrink/grow | 36.683 | 39.066 | +2.383 | 2.722 | +0.262 [−2.081, 2.605] |
| Supplied masks | 37.480 | 39.158 | +1.678 | 2.278 | −0.978 [−3.260, 1.303] |
| Linear HDR | 36.682 | 38.861 | +2.179 | 2.454 | +1.707 [−0.605, 4.020] |

All five upper bounds pass; the largest is 2.770 µs. This is a bounded
average-cost result on development `fsr-sdk 0.3.0`, Radeon RX 9060 XT, driver
`32.0.31041.1004`, provider `4.1.1`, with the same five matched 1080p workloads
as P02. It does not establish every-frame/transition overhead, released 0.2.0
performance or universal parity. Bounds are pointwise, not a simultaneous 95%
band; approximate coverage assumes independent, reasonably regular block
differences. Signed outliers are retained. Every GPU difference interval includes
zero; this does not prove zero GPU overhead or GPU equivalence.

Secondary lifecycle measurements use eight paired fresh-process means per
SDR/HDR configuration, with eight warm unused-context pairs within each process.
Values below are **ms per context**, with two-sided 95% difference intervals.

| Context | Boundary | Native C++ | Rust | Difference [95% interval] |
|---|---|---:|---:|---:|
| SDR | First create after provider query | 50.416 | 50.483 | +0.067 [−1.341, 1.475] |
| SDR | Warm unused-context create | 50.624 | 50.618 | −0.006 [−0.508, 0.497] |
| SDR | Used-context destroy after fence | 5.973 | 4.763 | −1.210 [−2.939, 0.518] |
| HDR | First create after provider query | 49.893 | 50.627 | +0.734 [−0.264, 1.732] |
| HDR | Warm unused-context create | 50.634 | 50.472 | −0.162 [−0.859, 0.535] |
| HDR | Used-context destroy after fence | 4.882 | 4.896 | +0.014 [−0.312, 0.340] |

All creation/destruction intervals span zero; no lifecycle speed gate is
selected. First creation follows a separate provider query in a fresh process,
not a cache-cold start. Used destruction excludes the GPU wait. Separate loader,
provider-query, first-dispatch and unused-destruction totals, all confidence
intervals and residual test instrumentation are documented in the record.

Fifteen separate allocation processes observe **zero calling-thread Rust
allocator events across 1,920 warmed dispatches**; native/driver heaps and other
threads are excluded. Separate preparation process means range from 2.186 to
2.811 µs; prepared raw diagnostics retain an extra query context. These
independent measurements do not form an algebraic decomposition of primary
overhead. Descriptive CPU p95, warm-up windows, segment intervals, outliers and
exploratory transition/control comparisons remain in the
[compact evidence](records/2026-10-16-exp-native-performance-confirmation/derived-metrics.json).
The transition/control CPU and GPU intervals span zero, so a consistent causal
transition penalty remains unresolved.

The [original implementation/pilot record](records/2026-10-16-exp-native-performance-characterization.md)
continues to describe the earlier 110-block plan and explicitly stopped run.
The new batch uses a fresh pilot compatible with the subsequent unattended
launcher and Windows result-write retry changes. Neither the earlier pilot nor
partial confirmation is promoted into a confirmed result. No release gate,
mandatory wider matrix or benchmark dependency is introduced. The
[unattended command](../../tools/verification/README.md#native-performance-characterisation)
remains available for separately requested reruns.

**Source-review finding:** the imported investigation distinguishes wrapper
validation, SDK CPU recording, provider GPU intervals and lifecycle costs. Its
[comparison of primary sources](records/2026-10-15-src-performance-testing-strategy.md#what-comparable-projects-actually-do--classified-evidence)
reports CPU/GPU separation in AMD sample/profiling guidance, Microsoft's DX12/PIX
documentation and Dawn's performance suite. Khronos's timestamp sample is an
analogy, not a DX12 or supported-backend proof. Benchmark-tool guidance supports
warm-up, repetitions and attention to variability, without requiring this project
to adopt those tools. These examples do not establish community-wide mandates
for 4K sweeps, particular frame counts or a fixed percentage parity gate.

## Provider setup-query cost

**P07 — verified bounded whole-path measurement:** the
[2026-10-19 experiment](records/2026-10-19-exp-provider-setup-cost.md) runs three
fresh optimized native processes on Windows x64/MSVC, SDK v2.3.0, provider 4.1.1,
RX 9060 XT and driver 32.0.31041.1004. Each process warms default/selected routes
four times and measures 32 alternating paired blocks. Every sample includes
ratio, quality render size, jitter phase/offset, resource requirements and memory
estimate; selected routing includes fresh membership queries/name copying.
All 11 cases pass in each process, with 192 retained timings and no GPU dispatch.

| Process | Default median (ms) | Selected median (ms) | Median paired difference (ms) |
|---|---|---|---|
| First | 230.743 | 707.104 | 474.440 |
| Second | 229.554 | 696.983 | 469.387 |
| Third | 230.207 | 699.049 | 471.453 |

These are complete six-query sequence costs, with context lifetime outside the
timer. They are not per-query, isolated enumeration, frame-dispatch or cache-cold
startup measurements. Selected setup is observably more expensive on this stack;
the data does not identify a causal component or select a cache/session design.
An observed default ID does not freeze default routing. Background activity,
clocks, power and temperature were not controlled, and three processes supply
descriptive evidence without an equivalence margin or speed gate. Do not pool
these costs with P01/P03 CPU recording or P06 lifecycle estimates.

Intake verifies the compact byte inventories, three 11-case verdicts, 192 saved
samples and per-process arithmetic against the canonical experiment and
[Windows batch](../release/2026-10-19-windows-upscaler-verification.md). This
integration repeats no native work and leaves historical measurements unchanged.

## Intake provenance and baseline reconciliation

The source investigation inspected `fsr-sdk-rs-main(37)(2).zip`, reporting SHA-256
`1ccf583fd573a3be0ffa97ba8987d7c83305342ffb18e634020f2c17e39d8d98` and no wrapper
Git identity. That ZIP lacked the P01 record and performance harness, so the
researcher correctly labelled the supplied CPU numbers as reported observations
that could not be audited there. The ZIP bytes and external-source investigation
were not independently revalidated during intake.

**Verified local availability:** the P01 record, compact evidence,
[Python coordinator](../../tools/verification/performance.py),
[Rust probe](../../crates/fsr-sdk/src/upscaler/dx12/tests/performance.rs) and
[GPU helper extension](../../crates/fsr-sdk/tests/native/performance.h) are present
in this checkout. Inspection confirms real COM pre-inspection before each native
pair, alternating arm order by frame, separate contexts, out-of-timer fencing
and final sentinel controls. Arithmetic intake checked all six retained sample
matrices' paired-difference medians against their stored summaries. There are
1,896 retained scalar timing entries, not 1,896 independent process experiments.

The source record's missing-file concern therefore applies to its supplied ZIP,
not the current checkout. Its proposed initial audit is partly satisfied by
available source/evidence. P03 subsequently supplies an independent C++ reference
and warm-state controls; P04 supplies the bounded GPU anchor. Neither finding
establishes a causal explanation for cache effects, timer noise or hardware
variation.

The incoming Markdown was moved from the nested
`fsr-sdk-rs/docs/research/records/` intake directory to the canonical records
directory without changing bytes: SHA-256
`e37a8f39a51d08f244b9c36f6ef10135315d08cd40bd8557e41357ab100e4755`.
Its original observations and snapshot qualifications are preserved. The existing
P01 experiment remains unchanged.

The P03/P04 records and adjacent compact evidence were already in canonical
locations at integration. Intake checks their verdicts, metadata/output cohort
compatibility and retained process-median differences, without rerunning GPU
work or revalidating external sources. Their measured source states remain
identified by their own input manifests. P04's shared-consumer extraction and
different resolution/history protocol do not retroactively change P03's result.

P05's record and compact evidence are integrated on 2026-10-16. Intake verifies
the evidence inventory, final 42-case verdict, all 30 process input audits,
consumer/cohort matching, stable-control comparisons and summary arithmetic.
The initial parser failure and sandbox CI failures remain failed in the retained
[attempt index](records/2026-10-16-exp-active-render-timing/checks.json); later
native, P03/P04 regression, CI and MSRV passes stay separate. Its inputs identify
the measured source/protocol and SDK bytes. No native execution is repeated for
this integration, and historical measurements are not pooled or rewritten.

P02's record and compact evidence are integrated on 2026-10-16. Intake verifies
the evidence inventory, 62-case verdict, all 50 process input audits and cohort
matches, independent complete mask hashes, selected debug/timed logs and summary
arithmetic. Retained protocol bytes match the current producers; input inventories
identify the measured source and SDK bytes. All five recorded native/regression/
CI/MSRV attempts pass their integrity checks. The completed record is already in
its canonical location and contains sanitized paths/pointer labels. No historical
record or evidence bytes are changed, and no native execution is repeated.

P06 confirmation intake verifies both complete local inventories, reparses 1,197
primary/supplemental logs against saved samples, checks native input/output
cohorts and reproduces all 80 primary/segment and 12 lifecycle estimates. The
frozen order and producer inventories agree. Compact evidence preserves all
paired difference vectors, process summaries, allocation counts and identities;
raw ticks/logs/images stay local under ignored `target/`. Exact past image bytes
cannot be recovered from retained hashes. The new record preserves two earlier
incomplete/failed pilot attempts separately. No GPU work is rerun for intake;
historical records and evidence bytes stay unchanged.

## Recommended next experiments

The [source recommendation](records/2026-10-15-src-performance-testing-strategy.md#ordered-programme-and-stopping-conditions)
is a small staged programme. On 2026-10-15 the maintainer requested its conversion
into [performance roadmap entries](../../ROADMAP.md#performance). The roadmap
owns current status, prerequisites and order. **P01 and the P03 → P04 → P05 → P02
sequence have completed their selected bounded checkpoints.** P02 uses verified
supplied-mask and linear-HDR integration; it does not depend on resolving broader
non-linear colour semantics. Completion establishes recorded execution/measurement
coverage, without accepting a numerical speed gate. Those bounded checkpoints
require no further performance experiment. The separately selected P06 now
completes its bounded average CPU budget and characterisation checkpoint;
final release scope and archive checks remain separate. D043 subsequently adds
the distinct P08 candidate check as a release prerequisite. That bounded review
is now closed with accepted limits. Rc1 host qualification and publication
simulation pass; final performance-input reconciliation is accepted. Release
closeout remains.

| Item | Current position | Evidence or remaining checkpoint |
|---|---|---|
| P03 | Verified bounded CPU comparison | Independent C++/Rust consumers and matched execution are established; sign-changing CPU differences and warm-state controls remain inconclusive for precise overhead or parity. |
| P04 | Verified bounded 1080p GPU timing | Same-queue timestamps, separate CPU recording, query/jitter/output matching and empty brackets establish one local GPU anchor, without a universal parity claim. |
| P05 | Verified bounded active-render timing | Fixed-allocation shrink/grow and stable-size controls pass matched execution/input checks across four Rust/C++ blocks; first-transition and later CPU/GPU measurements retain substantial variability. A consistent transition penalty remains inconclusive. |
| P02 | Verified bounded supplied-mask/linear-HDR comparison; parity inconclusive | All 62 cases pass with P05 controls, exact mask/input audits and matched Rust/C++ execution. Four balanced blocks retain sign-changing CPU/GPU differences; any parity assertion needs a separately justified margin and uncertainty treatment. |
| P06 | Verified bounded CPU budget and characterisation | Fresh pilot and 112-block confirmation pass 122/1,220 cases; all five average CPU upper 95% bounds are below 2.78 µs against the selected 5 µs budget. GPU/lifecycle remain descriptive; allocation instrumentation observes zero events across 1,920 warmed calls. |
| P07 | Verified bounded setup-query measurement | Three optimized processes retain 192 whole-path samples: default medians 229.55–230.74 ms, selected 696.98–707.10 ms per six-query sequence. No isolated enumeration cost, GPU dispatch, speed gate or cache/session decision. |
| P08 | Complete bounded review; maintainer-accepted exceptions | [Closure](../release/2026-10-29-performance-review-closure.md): full CPU verdicts remain inconclusive; accepted 222-block temporal subset has all nominal upper bounds below 5 us, with post hoc selection disclosed. Supplements and provider A/B reviewed; the [rc1 follow-up and acceptance](../release/2026-10-29-rc1-performance-acceptance.md) close final input reconciliation, preserving confidence bounds and uncertainty. Publication simulation passes; release closeout remains. |

On 2026-10-16 the maintainer selected P06's bounded characterisation protocol,
initially stopped confirmation, then authorised a fresh unattended batch. That
batch completes the selected checkpoint. P06 itself did not make repeated P01
measurements or further initialization/query/memory work a release requirement.
D043 subsequently requires the distinct candidate check described above; its
coverage must be selected before execution. Sharpening, automatic exposure, motion
variants, movable ownership, larger outputs, another hardware/provider or Proton
are directed follow-ups, not a mandatory Cartesian product. In particular,
1440p/4K sweeps would predominantly characterize provider scaling and have no
selected checkpoint here. Completed P05 uses M14's implemented render slice;
P02 retains its fixed/active-render controls and adds the selected features.
Variable output, non-linear colour, changing pre-exposure and wider ownership/
hardware coverage need their own contracts and directed questions before further
performance measurements.

## Interpretation and unresolved questions

The source investigation recommends strict benchmark-validity checks immediately:
matching runtime/provider, descriptors, flags, allocations and temporal histories;
successful dispatch/submission/completion; independent debug controls; and bounded
output consistency/sentinel checks. Invalid or blocked execution is not a speed
score. These controls support integration validity and do not resume the held
[image-quality research](../../SCOPE.md#verification-priority).

For performance, retain signed absolute differences, complete-call distributions,
process/block order and negative/outlying observations. Different driver/provider
versions need separate baselines. Frame samples from a single process are
correlated; confidence claims must account for independent process/block units.
Small process counts and a small median difference do not establish
equivalence, non-inferiority or a justified speed gate. Timer granularity,
systematic measurement effects, between-block variation and practical caller
budgets must inform any separately selected acceptance margin.

The prepared raw reference does not perform the wrapper's checks. P01's real COM
pre-inspection before each pair warms metadata; its effect on the observed
difference is unknown, and P03's sensitivity controls do not isolate it. Neither
subtracting separate inspection medians nor treating minimal C++ as equivalently
validated fixes that interpretation.
Serial fencing controls reuse/order but suppresses overlap and can affect clocks;
it is not a throughput, presentation-latency or frames-in-flight benchmark.

**Still open:** GPU equivalence, a generalizable estimate or causal explanation
of variable transition costs, tighter lifecycle uncertainty, truly cache-cold
startup and causal attribution/optimization of setup-query cost. P07 measures
the prepared whole setup path only; other initialization/memory profiles remain
separately selected work. P06 establishes
the selected average CPU budget only for its recorded stack and workloads.
P03 has measured warm-state sensitivity,
P04 has validated one GPU timing boundary, P05 adds matched active-size
transitions and P02 adds supplied-mask/linear-HDR comparisons. Actual mask
consumption/benefit, non-linear signal semantics, robustness across clock states
and wider GPU-work coverage remain unresolved. These are limitations or directed
follow-ups, not unfinished selected P02 cases. No profiler capture or benchmark
dependency was added.
SDK shader rankings, game FPS requirements, broad quality matrices and universal
performance promises remain outside the selected wrapper-focused programme.
