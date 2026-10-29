# Experiment: supplied-mask and linear-HDR Rust/C++ performance comparison

- Date: 2026-10-16, Europe/Berlin; attempt timestamps use UTC on October 15.
- Question: compare wrapper CPU recording and provider GPU command intervals with
  independent C++ for one supplied-mask and one linear-HDR case, retaining fixed
  and active-render controls (P02).
- Scope: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0,
  Windows x64/MSVC + DX12, local ownership on one GPU/driver/provider.
- Outcome: **bounded matched execution and measurement verified; speed parity
  remains inconclusive.** No equivalence margin or speed gate was selected.

## Baseline and selected protocol

[P03](2026-10-15-exp-independent-cpp-cpu-comparison.md) supplies independent C++
CPU comparison, [P04](2026-10-15-exp-1080p-gpu-timing.md) supplies same-queue GPU
timestamps and [P05](2026-10-16-exp-active-render-timing.md) supplies matched
active-render transitions and input audits. Their original records stay unchanged.
[D025](../../adr/D025.md) permits validated supplied masks independently of optional
consumption bits; [D026](../../adr/D026.md) supplies the verified linear-HDR slice.
The broader M16 non-linear colour questions are separate from this selected case.

The [feature protocol](2026-10-16-exp-mask-hdr-performance-comparison/feature-protocol.json)
extends the unchanged [active protocol](2026-10-16-exp-mask-hdr-performance-comparison/active-protocol.json).
Both are retained as exact bytes. Reuse P05's optimized consumers and clocks to
keep recording, GPU work and diagnostic operations within established boundaries.
Probe changes extend test-only Rust/C++ consumers, GPU equipment, validators and
profile composition. The worktree was clean before this task. Production APIs,
dependencies, bindings, SDK pin and native safety/ownership contracts are unchanged.

Five workloads run in each of four balanced blocks: stable maximum, stable
smaller, shrink/grow, both supplied masks, and linear HDR. Odd blocks run those
workloads in order, C++ then Rust; even blocks reverse workload and consumer
order. Each timed process has 160 continuous frames, excluding the first 32 from
measured distributions. Independent debug controls run four frames per workload
and consumer. One shared setup supplies all prerequisites; no duplicated native
setup or parallel timed children are introduced.

Every context enables dynamic resolution. SDK queries select 1280×720 maximum
and 960×540 smaller render regions with fixed 1920×1080 output, jitter phases
18/32 and global frame modulo the current phase. Shrink/grow occurs at 64/112
(debug 1/3). The analytic ramp, current-to-previous UV motion, active-region
sampling and first-frame-only reset preserve P05's output-space trajectory.
Explicit exposure and pre-exposure remain 1. No query runs inside a timed loop.

The two added cases use the maximum active region:

- **Supplied masks:** both R8_UNORM inputs are created/uploaded once before
  timing. Reactive values are 128 for `.25<u<.5`; transparency/composition values
  are 255 for `.5<v<.75`; other values are zero. These are declared static,
  render-aligned synthetic rectangles. Full GPU mask readbacks match uploads at
  each checkpoint. Independent Python integer inequalities generate complete
  expected FNV-1a hashes, avoiding reliance on the shared floating-point generator
  being correct in both consumers. Colour/depth/motion and jitter match the
  stable maximum SDR control exactly.
- **Linear HDR:** set the native HDR creation bit and multiply the ramp's RGB by
  8 while alpha, depth, motion, exposure and pre-exposure remain unchanged.
  Independent GPU input samples use the declared scale. Every checkpoint requires
  output maximum red strictly above 1 and below 64, as well as finite/written
  output checks. This checks bounded HDR execution without a colour-encoding hint,
  PQ normalization, variable exposure or an HDR display/presentation pipeline.

The dynamic-resolution flag, maximum allocations, output, camera, timing and
history controls remain common. The mask case adds two actual R8 resources and
their validation; HDR changes its creation flag and supplied RGB domain. No
cross-feature output-equality requirement is imposed.

## Setup and execution validity

The tested hardware is Ryzen 5 5600 and Radeon RX 9060 XT (`1002:7590`), feature
level 12_0, adapter 0, driver `32.0.31041.1004`. Provider **4.1.1** has opaque ID
`17700776140664213505`. SDK v2.3.0 remains pinned to commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The
[equipment](2026-10-16-exp-mask-hdr-performance-comparison/equipment.json) retains
actual OS/toolchain/MSVC/Windows SDK versions, exact build commands, native
hashes/signatures and CPU inspection. QPC is **10 MHz**, separate from the
**100 MHz** direct-queue timestamp frequency.

The [native verdict](2026-10-16-exp-mask-hdr-performance-comparison/native/run.json)
passes **all 62 required cases**, including ten debug and forty timed children:
**50 native processes and 6,440 submitted/fenced frames**. Four C++ ABI probes
compile, pinned headers/runtime hashes and AMD signatures pass, and both native
consumers execute the selected profiles. Each debug child has zero errors. Timed
children disable the debug layer and retain `UINT64_MAX` for unavailable debug
counts; this is not zero observed errors. GPU-based validation is off.

Initial/restored output sentinels, checkpoint read-only inputs/guards, finite and
written output, device-removal checks and terminal teardown pass. Output finite
checks cover all four channels at each pixel; the inherited written criterion
requires at least one channel per pixel to differ from the sentinel. It does not
prove every channel was written. Independent input samples bound binary16
truncation by one relative binary16 step plus 1e-6 for binary32 arithmetic; depth,
alpha and inactive guards remain exact. Full input comparisons cover all supplied
colour/depth/motion bytes and both complete masks; sampled analytic checks do not
prove every generated colour texel analytically correct.

Canonical descriptor/resource/scalar bits, provider/device, queries, every frame's
active size/reset/jitter/motion scale, checkpoint input/output diagnostics and
final output SHA-256 agree between Rust/C++ and repeated processes within each
workload/history-length cohort. Mask hashes match independent complete predictions.
FNV-1a is a diagnostic equality check, not a cryptographic artefact identity.
The masked cases also produce the same final output SHA-256 as their SDR maximum
controls in all eight timed consumer/block pairs. This bounded observation does
not identify mask consumption or prove that masks cannot affect other workloads.

## Timing observations

The CPU bracket covers dispatch recording only. The GPU bracket covers provider
commands and internal barriers on the same direct queue. Scene generation,
uploads/transitions, sentinel restoration, query resolution, submission/fencing,
readback resource allocation, mapping, hashes and logging are outside both
intervals. Empty clock brackets are retained without subtraction. All raw ticks,
warm-up and per-process metadata remain in the adjacent compact process files.

The following are **Rust-wrapper minus C++ process medians, in microseconds**:

| Block | Mask CPU | Mask GPU | Linear-HDR CPU | Linear-HDR GPU |
|---|---:|---:|---:|---:|
| 1 | +2.50 | +1.60 | −4.10 | +3.04 |
| 2 | +0.30 | +2.00 | −7.80 | +4.24 |
| 3 | +9.10 | −2.84 | +16.20 | −0.44 |
| 4 | −0.10 | −0.28 | +8.80 | +2.68 |

The [summary](2026-10-16-exp-mask-hdr-performance-comparison/native/summary.json)
retains CPU/GPU distributions and signed differences for all five workloads,
feature-minus-SDR differences per consumer/block, and P05's complete stratified
first-transition/settling/steady analysis. Distributions inherit the existing
upper-middle order statistic for even sample counts and indexed p95 convention.
Individual frame samples are correlated; four process blocks do not establish a
confidence interval or a practical equivalence margin.

**Inference:** successful matched execution extends coverage to the supplied-mask
and linear-HDR paths. Signed CPU/GPU differences vary between blocks and do not
establish a consistent language advantage or speed parity. Feature-minus-SDR
differences also mix different native workloads, validation work and between-
process conditions; they are not isolated wrapper overhead. No numerical
threshold was loosened to obtain acceptance.

## Verification and reproduction

The [check index](2026-10-16-exp-mask-hdr-performance-comparison/checks.json)
identifies every finalized attempt, commands/statuses and integrity outcomes.
The [native input inventory](2026-10-16-exp-mask-hdr-performance-comparison/native/inputs.json)
identifies actual source/protocol/SDK bytes, including the local probe changes.
These identities and retained commands/results make the result understandable
without a Git revision or external CI service. Source and archive acceptance are
separate; this is a development-source experiment.

| Finalized check | Required cases | Outcome |
|---|---:|---|
| P02 combined native comparison | 62 | Passed, including 50 native processes/6,440 frames |
| P03 shared CPU-consumer regression | 45 | Passed |
| P04 shared GPU-timing regression | 22 | Passed |
| Source CI | 115 | Passed; infrastructure 215 tests, 211 passed/four platform-specific skips |
| Rust 1.88 MSRV | 98 | Passed; all-target checks, four ordinary-test configurations and consumers |

All five attempts pass `verify-run` and retain unchanged source/protocol inputs.
The five coordinator attempts all pass on their first invocation.
CI and MSRV use the previously authorised outside-sandbox environment because
earlier experiments established temporary-file/compiler fixture access failures
in the sandbox; native runs complete in the normal environment. CI includes
formatting, Clippy, documentation, consumers/examples, tooling and Windows ABI.
MSRV checks compile the native probes but do not execute their GPU workloads;
actual timed native execution uses Rust 1.99.0. Only report/evidence and roadmap
prose is added after these checks.

Use existing Python 3.11+, PowerShell 7, x64 MSVC, cached Cargo inputs and a trusted
local SDK/runtime. Choose the actual hardware adapter:

```powershell
python -m unittest discover -s tools/verification/tests -p test_feature_comparison.py -v
python tools/verification/run.py run experiment-feature-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py run experiment-cpu-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-gpu-timing --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
```

The seven new evaluator tests reject wrong flags or missing mask descriptors,
incomplete/wrong mask hashes/readbacks, SDR/clipped/non-finite HDR, and mismatched
SDR controls. Known FNV and small integer-pattern controls check the mask oracle.
Existing active-size and CPU/GPU evaluator tests retain their original profiles.
The combined profile does not enter ordinary or release profiles automatically.

## Limits and remaining questions

No assistant-launched builds/tests ran alongside timed children. GPU clocks,
thermal state, scheduling and unrelated system activity remain uncontrolled.
Serial fences, CPU-generated uploads and diagnostic readback/hash gaps can affect
later clocks; mask readbacks add different untimed work. A 32-frame warm-up does
not prove steady clocks. These conditions prevent a causal explanation of small
timing differences and do not measure FPS, overlap, sustained throughput or
presentation latency.

Coverage is one local-owner configuration, two active sizes, one analytic scene,
one static mask pattern and one linear-HDR scaling convention. Actual mask
consumption/benefit, non-linear colour, changing-pre-exposure semantics, broader
M16 acceptance, reconstruction quality, movable GPU dispatch and other hardware
remain outside this proof. A parity assertion still requires a justified
practical margin and uncertainty analysis; this result selects neither.

Permanent evidence keeps compact JSON with full numeric arrays, selected sanitized
logs, protocol bytes, commands and identities. Private paths/transient pointer
labels are normalized; meaningful IDs/codes/hashes remain. Raw final GPU images
and build products remain ignored under `target/`; original GPU input buffers
are not permanently retained. Full historical image reanalysis needs surviving
local images or a fresh GPU run, whose bytes/timings need not match this stack.
Topic synthesis and release-scope selection remain separate work.
