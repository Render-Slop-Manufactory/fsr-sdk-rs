# Experiment: current-wrapper CPU performance baseline

- Date: 2026-10-15, Europe/Berlin. Attempt timestamps are UTC on October 14.
- Question: what steady-state CPU work does the current DX12 wrapper add to a
  prepared raw SDK dispatch, and what are its separate context lifecycle costs?
- Scope: P01 baseline for development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`,
  including implemented M11–M14/U03 code. The measured native profile uses fixed
  render/output, explicit exposure and the local ownership family.
- Exclusions: GPU execution time, independent C++ consumer parity (P02),
  variable output, dynamic-size performance, HDR/masks, cross-thread performance,
  other providers/hardware, and the held image-quality studies.
- Success: complete repeated sample matrices, verified ABI/pinned deployment,
  successful native recording/submission/fences, finite/written final output,
  debug-enabled control and terminal cleanup. Failures include missing samples,
  native errors, device removal or invalid controls. No speed threshold is imposed.

## Setup and preserved evidence

The starting worktree was clean. Only experiment/test infrastructure was changed:
the [Rust probe](../../../crates/fsr-sdk/src/upscaler/dx12/tests/performance.rs),
controlled COM counters and optimized fixture support, a small
[GPU helper extension](../../../crates/fsr-sdk/tests/native/performance.h),
optional optimized native build preparation,
[profile orchestration/analysis](../../../tools/verification/performance.py),
sample-integrity tests, profile registration and verification documentation.
Production wrapper code, public APIs, manifests, dependencies and SDK pin were
unchanged. This is the current post-M14-source baseline, not a retrospective
measurement of pre-M14 code or final release archives.

The final attempt `20261014T225132Z-experiment-performance-44cfd9c029bd` completed
all 17 cases, exit 0, with unchanged measured inputs; `verify-run` passed.
The permanent evidence directory contains:

- [Run verdict](2026-10-15-exp-wrapper-cpu-performance/run.json).
- [Exact source/configuration/SDK input hashes](2026-10-15-exp-wrapper-cpu-performance/inputs.json).
- [Equipment, signatures and executable identities](2026-10-15-exp-wrapper-cpu-performance/equipment.json).
- [All six timing matrices, distributions and count observations](2026-10-15-exp-wrapper-cpu-performance/measurements.json).
- [Debug-enabled native control log](2026-10-15-exp-wrapper-cpu-performance/diagnostics.txt).

Full local attempts, exports, executables and AMD binaries remain ignored under
`target/`; no generated images or whole source snapshots are retained here.
Paths and transient pointer values in exports are normalized; provider IDs,
native codes and hashes retain their numeric values. Record/evidence assembly
after measurement does not change the measured Rust/C++/Python probe sources.
The verification README's sentinel-control explanation was expanded afterward;
its retained input hash identifies the documentation present during measurement.

### Equipment

- Windows 11 build 10.0.26300, `x86_64-pc-windows-msvc`.
- AMD Ryzen 5 5600, six cores / twelve logical processors; Windows reports
  maximum clock 3501 MHz. This is metadata, not an observed sustained clock.
- High performance power plan. CPU affinity, boost/frequency, temperature and
  other applications were not controlled; no whole machine inventory was retained.
- AMD Radeon RX 9060 XT, adapter 0, vendor `0x1002`, device `0x7590`,
  feature level 12_0; successful driver query `32.0.31041.1004`.
- SDK v2.3.0 source pin `60f4ea81909200d8542eca14dccb2628b763a9a3`;
  provider `4.1.1`, ID `0xf5a5ca1e01001001`.
- Loader SHA-256 `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`;
  upscaler SHA-256 `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
  Both pinned DLLs pass the existing AMD Authenticode checks. No SDK acquisition
  or new strict online revocation check was performed.
- Rust/Cargo 1.99.0, LLVM 23.1.1; Python 3.14.1; MSVC 14.44.35207,
  Windows SDK 10.0.22621.0. Rust test executable: Cargo release defaults
  (optimized, no LTO override). C++ helper: `/O2 /MT`; controlled DLL: `rustc -O`.

## Protocol and reproduction

From the repository root with the existing cached Cargo inputs, Python 3.11+,
PowerShell 7, x64 MSVC and the pinned local SDK/runtime:

```powershell
python tools/verification/run.py run experiment-performance --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py export --run target/verification/<attempt> --output target/<exclusive-export-directory>
```

The [verification protocol](../../../tools/verification/README.md#cpu-performance-baseline)
describes the exact cases and measurement boundaries. Each repetition uses fresh
controlled and native processes, preceded by a separate debug-enabled native
control. Timed native children do not enable DirectX debug or GPU-based validation.

Controlled fixture: synthetic `1280×720` inputs and `1920×1080` output, 10,000
warm-up calls per dispatch arm, then 31 batches of 1,000 calls for empty-loop,
raw dispatch, inspection/descriptor building and complete wrapper dispatch.
Order rotates by batch. Lifecycle has 31 batches of 1,000 creations into
preallocated owner storage followed by 1,000 destructions. Counts are armed
separately from timing and scoped to the measuring thread. The disabled TLS
counter checks remain in the stub methods; these times are not real-driver COM
latencies. Allocation counters cover only the Rust test executable, excluding
allocations in fixture/SDK DLLs, drivers and native heaps.

Native profile: fixed `320×180` colour/depth/render-resolution motion,
`640×360` output, explicit `1×1` exposure, standard finite depth, UV motion,
jitter excluded from vectors, zero jitter, pre-exposure `1`, sharpening disabled,
frame time `16.667 ms`, camera `(0.1,100,pi/3,1)`. Existing static input textures
are reused after fences. Each arm has its own context/history and resets only
its first frame. The raw arm shares the pinned creation mapping and uses
`FfxLibrary::ffxDispatch` on a prepared descriptor; it is not an independent
C++ implementation. Order alternates every frame. There are 32 warm-up frames
per arm, followed by 128 individually timed calls per arm per process. Real COM
inspection is timed separately before each pair; this also warms metadata for
the pair. Raw descriptor preparation is outside raw call timing.

The helper submits each successful recording once, fences it, then resets the
allocator/list. All setup, submission, waiting, readback and logging are outside
dispatch timing. Before native dispatch, a no-dispatch control confirms the
readback reports zero changed pixels for the untouched sentinel. The helper
restores that sentinel before each arm's final dispatch, outside CPU timing,
so stale output cannot satisfy its check. Both arms' final outputs have 230,400 finite/written pixels;
device-removal checks pass. The debug control completes four frames per arm,
with zero final debug errors/device removal and successful cleanup. This checks
execution, not output equivalence or reconstruction quality. All dependencies
survive completion; dispatch/fence failures terminate the isolated process.
Unwinding before work or after fences cleans up the raw context explicitly.

Native lifecycle uses eight warm-up create/destroy pairs followed by 31 samples
per operation. The library test executable includes the existing test-only
teardown tracing environment lookup, even without tracing enabled; lifecycle
numbers/counts therefore include that test overhead. DLL loading and device/
resource creation are excluded. No empty-loop/timer cost is subtracted.
Distributions retain min, median, p95 and max; p95 uses sorted index
`floor((n-1)*0.95)`. Paired differences are wrapper minus raw at matching frame/
batch indices; they are not the difference between independently chosen medians.

## Observations

**Verified on the recorded configuration:** all final protocol cases and three
process repetitions completed. The table reports per-process median / p95 in
microseconds; lifecycle rows use milliseconds.

| Measurement | Repetition 1 | Repetition 2 | Repetition 3 |
|---|---:|---:|---:|
| Native raw SDK dispatch, µs | 33.9 / 43.3 | 34.2 / 42.2 | 38.5 / 44.3 |
| Native complete wrapper dispatch, µs | 35.5 / 42.5 | 35.9 / 42.5 | 40.2 / 47.3 |
| Native inspection/descriptor, µs | 1.7 / 2.5 | 1.7 / 2.4 | 2.0 / 2.8 |
| Native paired wrapper-minus-raw, µs | 1.4 / 6.5 | 1.3 / 5.9 | 1.3 / 6.5 |
| Controlled complete wrapper, µs | 0.2550 / 0.2805 | 0.2628 / 0.2962 | 0.2538 / 0.3194 |
| Controlled inspection/descriptor, µs | 0.2401 / 0.2718 | 0.2479 / 0.3268 | 0.2397 / 0.3748 |
| Controlled paired wrapper-minus-raw, µs | 0.2477 / 0.2733 | 0.2553 / 0.2890 | 0.2466 / 0.3102 |
| Native context creation, ms | 66.4727 / 68.8015 | 69.5163 / 73.4707 | 66.2837 / 74.2027 |
| Native context destruction, ms | 3.1035 / 3.4316 | 3.2295 / 3.6604 | 3.0796 / 3.4156 |

Controlled raw-call medians are 7.3 / 7.2 / 7.3 ns; empty-loop median is
0.5 ns per iteration. Controlled lifecycle creation medians are
110.9 / 110.1 / 109.4 ns, destruction 189.6 / 189.3 / 191.2 ns. The retained
matrices preserve every sample rather than treating all within-process frames
as independent hardware experiments.

Native differences are noisy: paired minima are -27.7, -17.1 and -5.8 µs,
and maxima 16.8, 9.4 and 57.5 µs. Those negative observations and outliers are
retained. The roughly 1.3–1.4 µs paired median is a local descriptive baseline,
not an exact additive wrapper cost or a statistically established universal
slowdown. Separate inspection is not expected to add numerically to raw timing
because its placement/cache state and scheduling differ.

All three controlled repetitions give identical separately armed counts:

| Operation | alloc / free / zeroed / realloc | QI / AddRef / Release / GetDevice / GetType / GetDesc |
|---|---|---|
| Warmed complete dispatch | 0 / 0 / 0 / 0 | 12 / 0 / 18 / 6 / 1 / 5 |
| Inspection/descriptor | 0 / 0 / 0 / 0 | 12 / 0 / 18 / 6 / 1 / 5 |
| Prepared raw dispatch | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 / 0 |
| One wrapper create/destroy pair | 2 / 2 / 0 / 0 | 0 / 1 / 1 / 0 / 0 / 0 |

Dispatch therefore performs 42 observed external COM entries in this explicit-
exposure fixture. QI/GetDevice's internal stub reference increments are excluded
from the explicit AddRef-entry count. All fixture reference counts balance.
The nonzero lifecycle allocation observation also confirms the Rust counter is
active; it includes the test-only teardown environment lookup and must not be
reported as a published-crate allocation count.

### Validation and earlier attempts

- `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings`: exit 0.
- `cargo test -p fsr-sdk --lib --features movable --locked --offline`: final
  outside-sandbox run exits 0, 68 passed, 41 ignored. Native/performance selectors
  are intentionally ignored here and execute separately in the protocol.
- `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline`: exit 0. This is MSRV compilation, not a repeated MSRV runtime profile.
- `python -m unittest discover -s tools/verification/tests -p test_performance.py -v`:
  exit 0, three tests, including duplicate/non-finite samples and incomplete or
  changed sample indices.
- `python -m unittest discover -s tools/verification/tests -p test_infrastructure.py -v`:
  final outside-sandbox run exits 0, all 23 tests, including profile prerequisite
  inventory, timeout/failure handling and evidence integrity.
- Rust formatting passes. The final protocol includes all four independent C++
  ABI probes; no final release archive or full release profile was run.

Initial probe compilation found private inspection-method access, a misnamed
provider accessor and moved options; these were corrected in the probe before
execution. Clippy rejected constant assertions; an equivalent optimized-build
guard fixed this. These were implementation failures, not SDK failures.
Two earlier attempts also completed all cases:
`20261014T224556Z-experiment-performance-0a688e244c18` precedes separate real-COM
inspection timing and batched controlled lifecycle timing;
`20261014T224805Z-experiment-performance-a49eb448f664` adds those but precedes
the sentinel negative control and each arm's final output invalidation.
Both remain local preliminary attempts; their measurements are not mixed with
the final tables.

Sandboxed regression attempts failed on temporary files: the Rust positive
ownership compilation could not write `libpositive.rmeta` (OS error 5), and all
23 infrastructure tests failed to create their temporary repository. The CPU
CIM query was also denied inside the sandbox. The unchanged checks/query passed
outside it; no criterion was weakened. The final GPU performance profile itself
completed inside the sandbox.

## Baseline delta and limits

There is now a reproducible current-source CPU baseline with retained samples,
validation cost, allocation/COM observations and separate lifecycle timing.
No wrapper optimization or feature contract was selected by these measurements.
The direct raw call shares mapping code and resource metadata with the wrapper;
it establishes neither independent C++ parity nor zero-overhead abstractions.

This small, serial, fixed-size workload is not an FPS result, GPU timestamp,
representative game benchmark, dynamic-resolution speedup or latency/throughput
promise. It neither restores the deferred pre-M14 baseline nor closes M14's
variable-output work. Native driver/provider allocations, GPU memory allocation
counts and non-local/movable configurations remain unmeasured. CPU/process
scheduling and hardware clock behaviour can dominate small differences.

P02 remains the later independent C++ consumer and GPU-time comparison after
the selected M14–M16 work; its acceptance threshold must be justified against
measurement variability. This record does not accept such a threshold, change
release scope or reopen held quality criteria. Roadmap/topic integration is
separate from preserving these measurements.
