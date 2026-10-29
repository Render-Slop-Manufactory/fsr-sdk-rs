# Experiment: native performance characterisation infrastructure and pilot

- Investigation date: 2026-10-16, Europe/Berlin; retained command timestamps use UTC.
- Question: can the existing independent C++ and Rust consumers support a bounded
  upper limit on average additional CPU dispatch cost, with separate GPU,
  lifecycle, preparation and allocation observations?
- Outcome: **implementation and pilot verified; confirmation stopped by explicit
  maintainer request. No confirmatory performance verdict.**
- Baseline: [P01–P05/P02 findings](../performance.md). Historical measurements
  are unchanged and are not pooled with this experiment.

## Implemented protocol

The maintainer selected a **5 µs** upper budget on average additional CPU cost,
with 2 µs aspirational, across the existing maximum, smaller, shrink/grow,
supplied-mask and linear-HDR workloads. GPU differences and lifecycle have no
numerical acceptance gate. This is an opt-in experiment, not a release gate.

The [published protocol](../../../tools/verification/performance-characterization-protocol.json)
and [commands/boundaries](../../../tools/verification/README.md#native-performance-characterisation)
define ten pilot blocks, followed by an exclusive frozen selection of 24..120
new confirmation blocks within a conservative two-hour allowance. Each block
pairs nearby fresh C++/Rust processes. Workload order is randomised; arm launch
positions balance within each two-block group. Pilot samples are excluded from
confirmation estimates. Producer/protocol identities and equipment must match.

The primary statistic is the equal-weight average of paired process-mean CPU
differences over frames 32..159. One-sided 95% Student t bounds across blocks
determine within-budget/exceeds-budget/inconclusive; two-sided 95% intervals,
consumer medians/nearest-rank p95 and transition segments remain visible.
Coverage is model-based and conditional on reasonably regular independent block
differences; consecutive frames are not independent replications. All five
workloads must pass for an overall within-budget claim. No frame-level latency
guarantee or GPU equivalence claim follows.

Separate processes implement validation/descriptor preparation, prepared raw Rust
diagnostics, Rust allocation/reallocation requests and requested bytes, and
SDR/HDR lifecycle measurement. Lifecycle separates loader acquisition, the first
provider/render-size query, first creation after that query, first dispatch,
fenced used-context destruction and subsequent unused-context create/destroy.
Fresh processes are not cache-cold. Rust test-only teardown tracing is suppressed;
a TLS branch and allocator hook remain. C++ warm creation reuses its stable
descriptors. Raw diagnostics retain an unused wrapper query context, so separately
timed quantities are not an algebraic decomposition of wrapper overhead.

The shared full input/history/mask/HDR audits now use the selected block count;
a negative test alters a later block to check that verification does not silently
stop at the old four-block limit. Existing profile counts/contracts stay intact.
No public API, dependency, native ownership or supported-platform contract changed.

## Setup and identified inputs

Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0 commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`, x64 Windows/MSVC + DX12:

| Input | Observed configuration |
|---|---|
| OS | Windows 11 Pro, 10.0.26300, build 26300 |
| CPU | AMD Ryzen 5 5600, active High performance power scheme |
| GPU / driver | Radeon RX 9060 XT / `32.0.31041.1004`, adapter 0 |
| Provider | `4.1.1`, ID `17700776140664213505` |
| Tools | Rust 1.99.0; minimum-version checks Rust 1.88.0; Python 3.14.1; MSVC 14.44.35207; Windows SDK 10.0.22621.0 |
| Clocks | QPC 10 MHz; GPU queue timestamps 100 MHz |
| Workload | 1920×1080 output, queried 1280×720 maximum / 960×540 smaller render regions, 160 frames per timed child, 32 warm-up frames |

Pinned header hashes, signed runtime bytes, actual local source files and
configuration identities are retained in
[source inputs](2026-10-16-exp-native-performance-characterization/source-inputs.json)
and the [check index](2026-10-16-exp-native-performance-characterization/checks.json).
The final producer identity is
`877111d5812eab7e9c8d84646babdb3fe36ce55d83932bdbbbc73f18deb26588`;
the producer protocol SHA-256 is
`cd56823cc8477c8608cab3bf6e5a479f0aba97a7cf447392c79f6dea846ba29a`.
These identify the relevant uncommitted probe changes; no Git revision or external
CI service is required to understand the observations. Only report/evidence and status prose
is added after verification. No commit or publication was performed.

## Execution and observations

| Attempt | Required cases | Outcome |
|---|---:|---|
| First pilot | 122 | Passed; superseded because subsequent test/probe edits changed its producer identity |
| Supplemental native smoke | 19 | Passed: preparation, raw and allocations for all five workloads, plus SDR/HDR C++/Rust lifecycle |
| First source CI | 115 | Failed only the generic profile-inventory test: confirmation requires an external frozen plan |
| Final source CI | 115 | Passed; infrastructure 225 tests, 221 passed and four explicit platform skips |
| Rust 1.88 minimum-version suite | 98 | Passed |
| Existing independent CPU comparison regression | 45 | Passed |
| Existing GPU timing regression | 22 | Passed |
| Final pilot | 122 | Passed; ten complete process blocks and all independent controls/audits |
| Confirmation | 1,200 planned | Stopped by maintainer; 112 passed, one intentionally terminated child, 1,087 dependent cases blocked |

The initial focused Python test invocation encountered the sandbox's temporary
directory access restriction (WinError 5); all ten focused tests passed outside
the sandbox. Full CI/MSRV also ran outside it. The profile-inventory failure was
fixed by explicitly testing missing-selection rejection and the complete valid
selected confirmation graph. No numerical budget was changed to obtain a pass.
All retained coordinator attempts pass evidence-integrity verification, including
the failed and interrupted outcomes; integrity is distinct from acceptance.

The [final pilot analysis](2026-10-16-exp-native-performance-characterization/pilot/analysis.json)
reports CPU block-difference standard deviations of **5.27–9.01 µs** across the
five workloads. The
[frozen selection](2026-10-16-exp-native-performance-characterization/selection.json)
chooses **110 confirmation blocks**, with a planned one-sided margin of
**1.425 µs**. The preferred 1 µs precision target is explicitly infeasible within
the conservative runtime allowance. Estimated unmultiplied measurement time is
about 80 minutes; the conservative estimate is 7,166 seconds, below 7,200.
This is a planning calculation, not a verified overhead bound or a new budget.

Pilot CPU warm-up-window medians across maximum-workload processes, for frames
0..15 / 16..31 / 32..47 / 144..159, are respectively
645.10 / 36.44 / 58.38 / 34.50 µs for C++ and
644.97 / 37.91 / 47.20 / 36.99 µs for Rust. These observations show early cost and
later variability; they do not establish a universally steady state after frame
32 or identify a causal cache/clock effect. Checkpoint readback gaps remain a
fixture limitation. No background builds/tests ran during the selected pilot or
confirmation. One repository-size scan occurred during the superseded pilot.

The [supplemental smoke summaries](2026-10-16-exp-native-performance-characterization/supplement-smoke.json)
retain **zero observed Rust allocation/reallocation requests and zero requested
bytes across 640 warmed dispatches**, one 128-frame allocation process for each
workload, including the shrink/grow frames. This is a calling-thread Rust
executable observation on the first producer state, not native/driver heap
coverage, an allocation guarantee or the planned three-repetition confirmation.
The smoke reused the first pilot's staged binaries; the final source differs in
test coverage, a module comment and explicit null initialization before C++ warm
creation. The final confirmation rebuilt its own consumers but stopped before
its supplemental matrix. Smoke timings remain diagnostic, without confirmatory
lifecycle/dispatch conclusions.

At 2026-10-16 00:11:21 UTC the maintainer-directed stop terminated the active
`characterization.block10.transition.rust-wrapper` child. Nine blocks plus one
additional C++ process had completed. The runner finalized the failed execution,
blocked every dependent case and retained unchanged source inputs; no primary
confirmation analysis ran. The
[interruption record](2026-10-16-exp-native-performance-characterization/confirmation/interruption.json)
explicitly excludes partial samples from performance claims. No partial result
is substituted for the prespecified 110-block experiment.

## Reproduction and retained evidence

Use local pinned SDK/runtime bytes, Python 3.11+, PowerShell 7, x64 MSVC and cached
Cargo dependencies. The exact options, input identities, UTC times, statuses and
failed case records for every coordinator invocation are in the check index.
Commands used for the final source and native checks:

```powershell
python -m unittest discover -s tools/verification/tests -p test_performance_characterization.py -v
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run experiment-cpu-comparison --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-gpu-timing --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-performance-pilot --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/performance_characterization.py select --pilot target/verification/20261015T235524Z-experiment-performance-pilot-2a75582617e2 --output target/performance-characterization-selection-final.json
python tools/verification/run.py run experiment-performance-confirm --performance-plan target/performance-characterization-selection-final.json --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The supplemental smoke executes the first `supplement1` and `lifecycle1` cases
from the published confirmation case builder, sequentially through the shared
Runner, using the first pilot's staged Rust/C++/helper binaries. Its exact
executable hashes, source state, selected case names and configuration are
retained. It supplies path validation, not a second statistical experiment.

The compact [evidence inventory](2026-10-16-exp-native-performance-characterization/artifacts.json)
hashes the final UTF-8/LF files. Retained selection/protocol JSON is normalized;
producer-file hashes identify the original source bytes. Personal paths and
transient pointer-labelled addresses are sanitized. Block differences permit
recomputing every pilot interval using the published `estimate()`; per-process
means/medians/p95, warm-up and empty-bracket summaries are retained separately.
Full per-frame matrices, checkpoint logs, outputs and native build products stay
local under ignored `target/`. This compact record cannot reanalyse raw GPU
images or establish exact output reproducibility on another machine.

## Baseline delta and remaining work

Verified: reusable opt-in infrastructure, frozen sample selection, statistical
and evidence rejection tests, successful matched pilot execution, supplemental
native paths, existing CPU/GPU regressions, source CI and minimum Rust version.
The experiment preserves a strict separation between execution acceptance and
speed outcome. Statistical confidence never converts a failed or partial run
into valid performance evidence.

Unresolved: the selected 5 µs average CPU budget, confirmatory GPU differences,
matched lifecycle confidence intervals and repeated allocation observations.
The long confirmation is deferred at maintainer request. A future confirmation
must run its entire frozen plan from the beginning, or take a new pilot if its
producer/equipment identity changes; partial blocks are not resumed or pooled.
There is no new README speed claim, near-native headline, FPS/throughput claim,
image-quality study, native/driver allocation guarantee or release prerequisite.
