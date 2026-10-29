# Experiment: matched 1080p FSR GPU timestamp pilot

- Date: 2026-10-15, Europe/Berlin; run timestamps below use UTC on October 14.
- Question: can independent Rust and C++ consumers record a matched, valid 1080p
  workload and measure its GPU command interval separately from CPU dispatch?
- Scope: P04, development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, one Windows x64
  MSVC/DX12 configuration and one SDK Quality render-size query.
- Baseline: [P03 CPU comparison](2026-10-15-exp-independent-cpp-cpu-comparison.md)
  established matched independent consumers but no GPU timestamps. Its noisy CPU
  findings remain historical evidence, not a blocker or correction factor here.
- Exclusions: FPS, presentation/latency, frames-in-flight throughput, quality
  assessment, masks/HDR, active-render transitions, broad resolution/hardware
  matrices, production optimization, release acceptance and full P02 parity.

## Result

**Verified bounded GPU timing:** all 22 final protocol cases pass, including
independent optimized builds, four pinned-header ABI probes, two separate debug
controls and eight timed processes. All ten children match provider, adapter,
driver, canonical descriptors, render-size/jitter queries and jitter bit patterns
within each history-length cohort. Final output hashes agree within each cohort.
All timestamps resolve after valid recordings, frequency checks and fences;
output is finite and fully written, with no device removal or debug errors in
the diagnostic children.

Measured GPU process medians range from **0.93808 to 0.94564 ms**. Nearby Rust
wrapper minus C++ median differences range from **−0.40 to +1.08 µs**. Those are
descriptive observations on this workload, not an equivalence test. CPU median
differences still change sign. No speed threshold or uncertainty-based parity
margin was selected; a precise wrapper/language overhead remains inconclusive.

## Implementation and pinned setup

The worktree was clean at intake. All changes are opt-in experiment code,
verification orchestration, documentation and evidence. Production behavior,
public APIs, ownership contracts, manifests, dependencies and SDK pin are unchanged.

- [Protocol](../../../tools/verification/gpu-timing-protocol.json) and
  [orchestration/parser](../../../tools/verification/gpu_timing.py): case ordering,
  isolated subprocesses, canonical metadata, timestamp validity and analysis.
- [Rust entry](../../../crates/fsr-sdk/src/upscaler/dx12/tests/gpu_timing.rs) and
  [shared consumer mechanics](../../../crates/fsr-sdk/src/upscaler/dx12/tests/performance_comparison.rs):
  the public local-owner wrapper path, with setup, queries and I/O outside timing.
- [C++ entry](../../../crates/fsr-sdk/tests/native/gpu_reference.cpp) and
  [shared native consumer](../../../crates/fsr-sdk/tests/native/performance_reference.h):
  an independent executable using pinned AMD headers and dynamically resolved SDK
  symbols. It calls no Rust code and omits Rust wrapper validation intentionally.
- [GPU equipment](../../../crates/fsr-sdk/tests/native/gpu_timing.h): resource
  upload, same-queue timestamps, resolve and fenced readback, with no AMD calls.
  A dedicated `/DFSR_GPU_TIMING` build of the existing helper permits exactly the
  selected process-local configuration before creating its sole harness. This is
  a protocol obligation, not a general concurrent configurable-harness API.
  Ordinary builds retain their original fixed dimensions.

P03's Rust/C++ mechanics were extracted into those shared implementation files;
its original selectors/profile remain available. Its default helper configuration
and measured descriptor values remain unchanged. A complete P03 regression run
passed after extraction, rather than inferring compatibility from compilation.

The SDK is **v2.3.0**, pinned source
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Native preparation verifies four
header identities and the two local runtime DLL hashes/signatures. SDK version,
Rust crate versions and actual selected provider identity are separate:
**4.1.1**, ID **17700776140664213505**. Both pre-create and live-context Quality
queries return **1280×720** for **1920×1080** output; live jitter phase count is
**18**. Each consumer obtains these values independently.

Equipment: AMD Ryzen 5 5600, AMD Radeon RX 9060 XT, adapter index 0,
vendor/device `1002:7590`, feature level 12_0, driver **32.0.31041.1004**;
Windows 11 build 26300. Rust/Cargo **1.99.0**, LLVM **23.1.1**, Python **3.14.1**,
MSVC **14.44.35207**, Windows SDK **10.0.22621.0**. Rust uses Cargo release
defaults, C++ and helper `/O2 /MT`; C++ keeps `/W4 /WX`. The Rust test binary
includes `dx12,movable`, but the measured owner is the local family.

After the run, `Get-ItemPropertyValue` of the Windows CPU registry entry confirms
the CPU name; `powercfg /GETACTIVESCHEME` reports High performance. No power,
driver, clock or thermal setting was changed. Stable GPU power state and clock
locking were not enabled. Toolchain identities, build commands, SDK trust inputs
and executable/helper hashes are retained in [equipment.json](2026-10-15-exp-1080p-gpu-timing/equipment.json).

## Workload and measurement boundaries

One context per isolated process renders 160 frames, resetting only frame zero.
The first 32 are retained warm-up; the last 128 form the measured distributions.
Separate debug children execute four frames. The fixed warm-up count is not proof
of complete clock/cache stabilization. All live-provider jitter queries happen
before the frame loop, using frame index modulo the queried phase count.

Shared deterministic input generation samples an analytic ramp translating one
render pixel right per frame. Color sampling includes the actual projection
jitter: `sx=x+0.5-jx`, `sy=y+0.5-jy`, red
`0.1+0.75*(sx-frame)/render_width`, green `0.1+0.75*sy/render_height`, blue 0.2,
alpha 1. Motion is current-to-previous UV displacement `−1/render_width`, zero
on frame zero, with jitter excluded. Depth is standard finite 0.5. Formats are
RGBA16F color/output, RG16F motion and R32F depth/exposure. Explicit 1×1 exposure
and pre-exposure are 1; sharpening and optional masks are off; creation flags 0.
Frame delta is 16.667 ms, camera near/far 0.1/100, vertical FOV the same f32 pi/3,
view-space-to-meters 1. Canonical resource metadata/scalar bits and the complete
actual jitter sequence are retained. The initial canonical descriptor uses zero
jitter for comparison; frame dispatches use the separately retained offsets.
This is an integration workload, without a reconstruction-quality criterion.

Each frame is submitted on its harness's **direct queue**. That same queue owns
the queried frequency. Four timestamp queries record adjacent empty start/end,
then FSR start/end. The FSR bracket includes the provider's recorded commands and
barriers; fixture upload copies/transitions and final sentinel restoration occur
before it. `ResolveQueryData` and final output-copy commands follow it. The
completed monotonic fence precedes query-buffer mapping, allocator/list reset,
upload mutation and query reuse. Failure terminates the isolated child without
submitting a failed recording or unwinding potentially unfinished dependencies.
Context teardown follows the last successful fence and output check.

This follows Microsoft's [DX12 timing documentation](https://learn.microsoft.com/en-us/windows/win32/direct3d12/timing)
(accessed 2026-10-15): timestamp results are UINT64 ticks converted using the
submitting queue's frequency. The implementation uses `EndQuery` and explicit
GPU-timeline resolve. Queue frequency is positive and unchanged at initialization
and after every frame: **100,000,000 Hz**, nominally 10 ns/tick. This is not a
measurement of shader core clocks. The parser rejects missing/duplicate samples,
non-monotonic timestamps within/between frames, empty FSR intervals, invalid or
changing frequencies and malformed/non-periodic jitter bits.

An independent QPC bracket measures CPU dispatch only, using the shared helper
clock at **10,000,000 Hz**. Frame construction, queries, upload preparation,
timestamp recording, submission, waiting, readback and logging are outside that
CPU bracket. Its 256 empty pairs have median 0 ns and p95 100 ns in all children;
this quantization does not establish free timer calls. Empty GPU brackets have
per-process medians **40–80 ns**, with retained zero-valued samples and p95
**40–80 ns** across diagnostic/timed children. Nothing is subtracted.

Four process blocks alternate C++→Rust and Rust→C++, balancing launch position.
All per-frame ticks are retained; differences compare nearby **process medians**,
not paired frames across processes. Median is sorted index `n//2`, p95 is
`floor((n−1)*0.95)`, matching existing analysis. No samples are removed or pooled
as independent repetitions. Both ordinary debug validation and GPU-based
validation are off for timings; diagnostics enable only ordinary debug validation.

Every child first checks an untouched sentinel, then restores the sentinel before
the final dispatch and requires all **2,073,600** output pixels finite and written.
The final RGBA16F payload is **16,588,800 bytes**, excluding row padding. The
eight timed outputs share SHA-256
`bd9fd40fe7456bd2be5f6ae6f57f08053c5cdf5196e4e4fcb56b8576aabe3904`;
the two debug outputs share
`524cd583ea532e9d46bcd105773391f588031dcaeef02d55fd0d840506247c97`.
The cohorts have different history lengths and need not match each other.
Cross-driver/GPU bit identity and image quality are not implied.

## Final observations

Final attempt: `20261014T233902Z-experiment-gpu-timing-75c5d92cd3c6`, from
23:39:03 to 23:39:39 UTC. All **22 cases pass**, exit 0, `inputs_unchanged=true`;
complete artefact/case verification passes. Input identity:
`9a194ce4fc0d74661e1c9996fff8a3fc7952452a497fca70b5026c02ec50d0b6`.
No assistant-launched compilation or tests ran alongside the timed children.
Unrelated system activity remains uncontrolled.

GPU measurements below are **microseconds**, including all 128 measured samples
per process in their distributions:

| Block | C++ median | C++ p95 | Rust median | Rust p95 | Rust − C++ median |
|---|---:|---:|---:|---:|---:|
| 1, C++→Rust | 942.20 | 944.56 | 941.80 | 945.40 | −0.40 |
| 2, Rust→C++ | 944.56 | 947.52 | 945.64 | 948.24 | +1.08 |
| 3, C++→Rust | 938.08 | 942.96 | 938.64 | 945.16 | +0.56 |
| 4, Rust→C++ | 942.00 | 945.52 | 942.44 | 945.12 | +0.44 |

CPU recording process medians and differences, also **microseconds**:

| Block | C++ | Rust wrapper | Rust − C++ |
|---|---:|---:|---:|
| 1 | 37.2 | 36.9 | −0.3 |
| 2 | 39.6 | 47.8 | +8.2 |
| 3 | 33.5 | 40.0 | +6.5 |
| 4 | 41.2 | 39.8 | −1.4 |

The GPU medians are close on this bounded configuration. However, measured GPU
frame minima reach 660.72 µs, while some process medians are near 945 µs; the
retained distributions are not constant-time traces. CPU tails also vary.
The experiment does not identify the causes. Serial fences and CPU-generated
uploads leave idle gaps, so this is not a sustained GPU-saturation benchmark.
Uncontrolled clocks, scheduling, cache and thermal state limit precision.
No inference of a causal Rust/C++ GPU advantage or universal parity follows.

### Preserved evidence

- [Run verdict](2026-10-15-exp-1080p-gpu-timing/run.json).
- [Source/configuration/SDK identities](2026-10-15-exp-1080p-gpu-timing/inputs.json).
- [Equipment and exact build commands](2026-10-15-exp-1080p-gpu-timing/equipment.json).
- [All ten process matrices, commands, metadata and output hashes](2026-10-15-exp-1080p-gpu-timing/processes.json).
- [Protocol, distributions and block differences](2026-10-15-exp-1080p-gpu-timing/comparison.json).

Assembly rechecked tick conversion, distributions, cohort compatibility and
actual local payload sizes/hashes. Paths and pointer-labelled transient addresses
are normalized; provider IDs, hardware IDs, clocks, native codes and hashes are
preserved. Full diagnostics, binaries and raw readbacks remain ignored under
`target/`; permanent evidence contains no raw images or copied source snapshots.

### Verification and earlier attempts

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings`:
  passed. The first attempt flagged an indexed jitter loop; it was replaced by
  iteration with `enumerate`, without suppressing the lint. The compiler's
  incremental-cache hard-link fallback warning also appeared; compilation copied
  files successfully.
- `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline`:
  passed after the final Rust edit; compilation evidence, not MSRV runtime coverage.
- `cargo test -p fsr-sdk --lib --features movable --locked --offline`: outside-
  sandbox pass, **68 passed, 43 ignored**. The first sandbox attempt passed 67 but
  its ownership compiler fixture failed writing a temporary `.rmeta` with access
  denied. The unchanged test passed outside, with explicit execution permission.
- `python -m unittest discover -s tools/verification/tests -p test_gpu_timing.py -v`:
  **five passed**: separate clocks/warm-up, missing/duplicate samples, invalid
  dimensions/clocks/order/jitter, cross-consumer mismatch, balanced profile/DAG.
- The same command selecting `test_cpu_comparison.py`: **five passed**.
- The same command selecting `test_infrastructure.py`: outside-sandbox **23
  passed**. All 23 initially failed during sandbox temporary-directory setup with
  access denied; unchanged tests passed with explicit outside execution permission.
- P03 regression command, `python tools/verification/run.py run experiment-cpu-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX`:
  all **45 passed**, `inputs_unchanged=true`, attempt
  `20261014T233817Z-experiment-cpu-comparison-77b2ef146b22`; `verify-run` passed.
  This checks the shared-consumer extraction/default-size helper. It does not
  replace or reinterpret P03's earlier retained performance findings.
- Initial GPU attempt `20261014T233445Z-experiment-gpu-timing-e67d7436e797`:
  all **22 passed**, before stricter cross-frame/phase parsing, the iterator
  correction and C++ compile-time branch cleanup. Its medians are not pooled
  into the final result. Neither native GPU attempt required sandbox escalation.
- No complete release/archive profile, native Proton timing, PIX capture,
  expanded resolution matrix or image-quality experiment was performed.
- Final source/SDK identities and retained distribution calculations match;
  133 local file/anchor references resolve. `git diff --check` passes and the
  repository size check reports 17,646,924 bytes (17.65 MB), within its budgets.

## Reproduction and baseline delta

With cached Rust dependencies, x64 MSVC/PowerShell 7 and the local pinned SDK:

```sh
python tools/verification/run.py run experiment-gpu-timing --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py verify-run --run target/verification/<fresh-attempt>
```

The runner builds both consumers before the debug/timed children, uses watchdogs,
requires all 22 cases and preserves every outcome. Run without concurrent builds
or tests. Exact historical output bytes are observations, not cross-machine
reproduction promises. Ordinary Cargo builds remain independent of Python/SDK,
and ordinary/release profiles do not acquire this performance requirement.

P04 adds independent same-queue GPU timing to P03's CPU-only foundation, with
provider-queried dimensions/jitter and a coherent moving-input history. P05 can
now reuse this bounded method to study M14's active-render shrink/grow slice.
Allocation resizing/variable output, masks/HDR and full P02 remain separate.
