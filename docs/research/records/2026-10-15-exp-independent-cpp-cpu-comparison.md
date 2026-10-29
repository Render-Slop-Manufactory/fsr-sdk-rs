# Experiment: independent C++ and Rust CPU recording comparison

- Date: 2026-10-15, Europe/Berlin; retained attempt timestamps use UTC on October 14.
- Question: how does the current Rust public dispatch compare with an independent
  minimal C++ SDK consumer, using the Rust prepared-raw call as a bridge?
- Scope: P03, development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0,
  one local-owner fixed-size Windows/DX12 profile. This extends the
  [P01 CPU baseline](2026-10-15-exp-wrapper-cpu-performance.md) under the
  [source-reviewed strategy](2026-10-15-src-performance-testing-strategy.md).
- Exclusions: GPU timestamps, FPS, dynamic render performance, HDR/masks,
  reconstruction-quality studies, broad hardware coverage, speed acceptance
  thresholds and full P02 parity.

## Result

**Verified bounded execution:** all 45 final protocol cases passed, including
independent optimized C++ compilation, separate debug controls for three arms,
six balanced comparison blocks and three warm-state control blocks. All 33 GPU
children use matching descriptor metadata, scalar float bits, provider, adapter,
driver and clock frequency; final output bytes agree within each history-length
cohort. Native dispatch, submission, fences, sentinel/output checks and cleanup
pass on the recorded configuration.

**Performance conclusion: inconclusive.** In the final run, differences between
nearby Rust-wrapper and C++ process medians range from **-6.4 to +9.4 µs**.
Rust prepared-raw versus C++ also changes sign and ranges from **-12.5 to
+4.3 µs**. The observed process variation prevents assigning a precise language
or wrapper overhead, declaring equivalence or non-inferiority, or selecting a
performance gate. Protocol completion is not a speed-parity verdict.

## Implementation and setup

The initial worktree contained the preceding performance-roadmap/documentation
edits. They were preserved; the source identities below capture actual measured
inputs without relying on a wrapper Git revision. Production wrapper behaviour,
public API, manifests, dependencies, SDK pin and release selection are unchanged.

Probe changes:

- [Standalone C++ consumer](../../../crates/fsr-sdk/tests/native/cpu_reference.cpp),
  using AMD's pinned headers and independently built SDK context/dispatch
  descriptors. It dynamically resolves SDK functions and calls no Rust code.
- [Rust comparison arms](../../../crates/fsr-sdk/src/upscaler/dx12/tests/cpu_comparison.rs),
  exercising public `Upscaler::dispatch` or the prepared-raw bridge. P01's
  existing test-only raw owner/optimized-build guard are reused through private
  test-module visibility; P01's measurement bodies are unchanged.
- A small [equipment ABI header](../../../crates/fsr-sdk/tests/native/cpu_equipment.h)
  and shared-helper QPC/frequency exports in
  [performance equipment](../../../crates/fsr-sdk/tests/native/performance.h).
  Shared equipment supplies device/resources, input pixels, clock, submission,
  fencing and readback; it makes no AMD SDK calls. Native API integration and
  descriptor construction are independent in the C++ consumer.
- [Coordinator and analysis](../../../tools/verification/cpu_comparison.py),
  [frozen protocol](../../../tools/verification/cpu-comparison-protocol.json),
  evidence-rejection tests, profile registration and verification instructions.
  No new dependency, SDK download or provider instrumentation was introduced.

Equipment:

- Windows 11 build `10.0.26300`, x86_64 MSVC; AMD Ryzen 5 5600.
  CPU model was read from Windows processor metadata. The active power plan is
  High performance. Affinity, clocks, thermals and unrelated applications were
  not controlled; no profiler or stable-power override was enabled.
- Radeon RX 9060 XT, hardware adapter 0, vendor `0x1002`, device `0x7590`,
  feature level 12_0; driver `32.0.31041.1004`.
- SDK v2.3.0, pinned source `60f4ea81909200d8542eca14dccb2628b763a9a3`,
  provider `4.1.1`, diagnostic ID `0xf5a5ca1e01001001`.
- Loader SHA-256 `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`;
  upscaler SHA-256 `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
  Existing local DLLs pass pinned-byte and AMD Authenticode checks. No new strict
  online revocation check or acquisition was performed.
- Rust/Cargo 1.99.0, LLVM 23.1.1; Python 3.14.1; MSVC 14.44.35207,
  Windows SDK 10.0.22621.0. Cargo release defaults, `dx12,movable` compiled in;
  only the local ownership family is exercised. C++ `/O2 /MT /EHsc /W4 /WX`;
  helper `/O2 /MT`. Consumers are Windows executables, not final crate archives.

## Protocol and reproduction

The [verification documentation](../../../tools/verification/README.md#independent-c-cpu-comparison)
and [protocol JSON](../../../tools/verification/cpu-comparison-protocol.json)
define the runnable experiment. From the repository root, with cached Cargo
inputs, Python 3.11+, PowerShell 7, x64 MSVC and the existing pinned local SDK:

```powershell
python tools/verification/run.py run experiment-cpu-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py export --run target/verification/<attempt> --output target/<exclusive-export-directory>
```

Each child owns one context. Both consumers use the shared helper's static pixel
generator: `320×180` colour/depth/motion, explicit `1×1` exposure, `640×360`
output; `RGBA16F/R32F/RG16F/R32F/RGBA16F` formats. Context flags are zero: finite
standard depth, render-resolution jitter-excluded UV motion, fixed sizing and
explicit exposure. Pre-exposure is 1; sharpening and masks are absent. Jitter and
motion are zero, native motion scale `(320,180)`, frame time `16.667 ms`, camera
`(0.1,100,pi/3,1)`. Reset is true only on frame zero. Allocation dimensions,
states, resource presence, descriptor tags/scalars and creation profile are
audited outside timing. Rust's canonical dispatch comes from the same inspection
function used by public dispatch; C++ uses its independently assembled descriptor
and AMD's inline resource conversion. Rust creation-profile reporting is tied
to its inspected default options/mapping, not a native interceptor trace.

All successful recordings are submitted once and fenced before reuse. An initial
no-dispatch readback confirms untouched sentinel output. The sentinel is restored
before each final recording; final output must have 230,400 finite and modified
pixels. The helper restores boundary states, then resets list/allocator only
after the fence. SDK/equipment failures terminate isolated children rather than
submitting failed lists or unwinding possibly in-flight dependencies. Context
destruction follows completion; device-removal checks pass. Separate four-frame
children enable ordinary debug validation and assert zero errors; timed children
leave debug validation off. GPU-based validation is not enabled.

The Rust public arm includes metadata/identity/device validation and descriptor
building. The Rust raw bridge uses the existing native ownership/loader path and
a descriptor prepared once. The minimal C++ arm omits equivalent validation and
constructs native descriptors directly from pinned headers. Its retained creation
chain stays address-stable through destruction. These unequal host policies are
intentional, and prevent interpreting the full difference as language overhead.

Both consumers call the same helper's `QueryPerformanceCounter` and frequency
exports. QPC frequency is **10,000,000 Hz** (100 ns per tick) in all children.
Record 256 empty clock brackets: every child's median is 0 ns and p95 is 100 ns;
that quantized observation does not mean timer calls are free. Nothing is
subtracted. One bracket encloses dispatch only. Frame construction, SDK/context
setup, helper upload/clear commands, list submission, fence waiting, readback,
file output and logging are outside it. Rust return/error handling and C++/Rust
indirect/import call arrangements can contribute different fixed host costs.

Each timed child retains all 160 frame intervals: first 32 are warm-up, last 128
form its measured distribution. Primary P03 does **not** repeat P01's per-pair
pre-inspection: raw metadata is prepared once before the frame loop. Six process
blocks cover every permutation of the three arms, twice per launch position.
Compare nearby process medians within each block, not individual frames across
executables. The empirical median is sorted index `n//2`; p95 is
`floor((n-1)*0.95)`. These match the existing baseline analyser.

Three further control blocks run Rust wrapper/raw with and without per-frame
pre-inspection, using retained varied launch orders. Those controls measure
sensitivity under separate processes; scheduling, clocks and idle/fence effects
remain confounders. They do not isolate a causal cache-warming effect. There is
no hard speed criterion, automatic outlier removal or pooled independent-frame
claim. A bounded inconclusive result is an allowed completion outcome.

## Final observations

Final attempt: `20261014T232259Z-experiment-cpu-comparison-8eb0ce345c36`.
All 45 cases pass, exit 0, with unchanged measured inputs; `verify-run` passes.
This final run followed completion of regression/compilation checks. No assistant-
launched compilation or test process ran alongside its timed GPU children.
Unrelated system activity and clock/scheduler state remain uncontrolled.

Per-block medians and signed differences below are in **microseconds**:

| Block | Launch order | C++ minimal | Rust raw | Rust wrapper | Wrapper − C++ | Raw − C++ |
|---|---|---:|---:|---:|---:|---:|
| 1 | C++ → raw → wrapper | 34.2 | 37.9 | 40.5 | +6.3 | +3.7 |
| 2 | wrapper → raw → C++ | 38.2 | 32.0 | 33.7 | -4.5 | -6.2 |
| 3 | raw → C++ → wrapper | 32.3 | 36.6 | 41.7 | +9.4 | +4.3 |
| 4 | wrapper → C++ → raw | 49.2 | 36.7 | 42.8 | -6.4 | -12.5 |
| 5 | raw → wrapper → C++ | 32.1 | 32.7 | 33.7 | +1.6 | +0.6 |
| 6 | C++ → wrapper → raw | 32.0 | 32.3 | 33.8 | +1.8 | +0.3 |

Warm-state control differences, Rust-wrapper process median minus Rust-raw
process median, in microseconds:

| Control block | Prepared once | Per-frame pre-inspection |
|---|---:|---:|
| 1 | +3.4 | +5.3 |
| 2 | -0.6 | +6.3 |
| 3 | -0.1 | +0.2 |

Neither comparison supplies a stable numerical correction to P01. P01 interleaved
two contexts within one process and used a different host timer; P03 uses separate
executables/contexts, QPC helper brackets and process-level differences. These
are different reference and sampling conditions. Negative and high observations
are retained. The bounded data do not identify the cause of the variation.

All 30 timed children produce the same final RGBA16F output SHA-256:
`031e1c2fdcd80f6a535a847708cad5d1e2c38690cc9caacc0f2b473e721b968e`.
All three four-frame debug children agree at:
`5097338675a805521419720c249d21ac5f5cc637351851382b93f36a16a63987`.
The cohorts differ in history length and are not required to match each other.
Each payload is 1,843,200 bytes, without row padding. Exact agreement is bounded
execution-consistency evidence on this provider; it is no reconstruction-quality
verdict or promise of bit identity on another driver/GPU.

### Preserved evidence

- [Run verdict](2026-10-15-exp-independent-cpp-cpu-comparison/run.json).
- [Measured input hashes](2026-10-15-exp-independent-cpp-cpu-comparison/inputs.json).
- [Toolchain, SDK, helper and consumer build identities/commands](2026-10-15-exp-independent-cpp-cpu-comparison/equipment.json).
- [All 33 process matrices, metadata, output hashes and timing commands](2026-10-15-exp-independent-cpp-cpu-comparison/processes.json).
- [Block analysis and exact protocol/order](2026-10-15-exp-independent-cpp-cpu-comparison/comparison.json).

Permanent evidence excludes executables, AMD distributions and raw output
payloads. Full attempts and sanitized exports remain ignored under `target/`.
Repository/home/run paths and pointer-labelled transient values are normalized;
provider IDs, hardware IDs, QPC values, hashes and native codes are preserved.
Assembly checked tick conversion, per-process distributions, cross-consumer
metadata/output compatibility and retained sample counts against case records.

### Checks and earlier attempts

- Rust formatting and denied-warning Clippy for workspace/all targets with
  `fsr-sdk/movable`: exit 0. The compiler emitted an incremental-cache hard-link
  fallback warning; it copied cache files and completed successfully.
- `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline`:
  exit 0; MSRV compilation only, not a full repeated MSRV runtime profile.
- `cargo test -p fsr-sdk --lib --features movable --locked --offline` outside
  the sandbox: exit 0, 68 passed, 42 ignored. The new native selector is ignored
  in ordinary tests and executed explicitly in the comparison profile.
- `python -m unittest discover -s tools/verification/tests -p test_cpu_comparison.py -v`:
  exit 0, five tests. Rejects missing/duplicate samples, invalid clocks,
  mismatched canonical descriptors, missing validity/arm markers, changed
  provider/adapter/driver/output; checks balanced launch positions.
- `python -m unittest discover -s tools/verification/tests -p test_infrastructure.py -v`:
  outside-sandbox exit 0, all 23 tests, including profile prerequisites, failure/
  timeout handling and evidence integrity. The initial sandbox attempt failed
  all 23 during temporary-directory setup (access denied); unchanged tests passed
  outside. No criterion was relaxed. Native comparison runs completed inside
  the sandbox. Regression/metadata checks outside it were explicitly permitted.
- The native profile independently compiles all four pinned C++ ABI probes.
  No final release archive, full release profile or GPU timestamp test was run.

Preserved earlier local attempts:

| Attempt suffix after `target/verification/` | Outcome and interpretation |
|---|---|
| `20261014T231850Z-experiment-cpu-comparison-1ef7b833aa5b` | C++ build failed because `/WX` rejected deprecated `getenv`/`_wgetenv` calls (C4996/C2220). No comparison child ran. Replaced environment reads with checked Windows wide-character API calls; warnings remained errors. |
| `20261014T231936Z-experiment-cpu-comparison-b2ecfa1bdf61` | All 45 cases passed before strict adapter/driver metadata matching was added; some verification work overlapped execution. Exploratory timings are not mixed with the final table. |
| `20261014T232131Z-experiment-cpu-comparison-57afd1088ed2` | All 45 cases passed with the final measured code and strict metadata matching, but overlapped Rust regression work. A further unchanged-code run after those checks became the final result above. |

The last two runs used identical measured source/configuration identities. The
final run still has sign-changing differences; removing known assistant workload
overlap did not establish a precise speed conclusion. Earlier numeric samples
remain local diagnostics, rather than being silently pooled with the final run.

## Baseline delta and limits

P03 now has a reproducible independent C++ native consumer, matched Rust wrapper/
raw controls, canonical descriptor and exact output checks, balanced process
blocks, a common timer and explicit warm-state sensitivity samples. It addresses
P01's lack of an independent C++ reference, while preserving P01's historical
conditions and observations. It establishes valid integration/execution for
the selected consumers, not a universal timing relation between them.

The next planned performance step is P04's separate 1080p GPU timestamp pilot.
P03 does not close GPU timing, active-render transition performance, masks/HDR
coverage or full P02. A future precise CPU-overhead claim needs a more stable
process/clock design and uncertainty treatment; it should not be obtained by
rerunning until a preferred sign appears. The existing six-block result is
retained as inconclusive. No production optimization, release requirement,
quality research or numerical parity threshold was selected by this experiment.
