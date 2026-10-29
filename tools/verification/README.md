# Verification

`python tools/verification/run.py` is the current coordinator. It preserves Cargo
tests, private/public controlled fixtures, independent C++ ABI probes and existing
bounded GPU workloads. It needs Python 3.11+ with only the standard library;
Windows preparation also needs PowerShell 7 and x64 MSVC. Ordinary Cargo builds
still require neither Python nor AMD's SDK. No public API or ownership contract changes.

## Architecture and platform boundaries

Python coordinates verification; Cargo remains the Rust test runner. Unit tests,
private/public fixtures, independent C++ ABI probes and native GPU workloads stay
in their existing layers. Each effect owns its workloads and acceptance criteria.

| Layer | Responsibility |
|---|---|
| [run.py](run.py) and [profiles.py](profiles.py) | CLI, named profiles, case inventory and ordered prerequisites |
| [execution.py](execution.py) | Shared subprocess execution, watchdogs and separate execution/evaluation outcomes |
| [consumers.py](consumers.py), [examples.py](examples.py), [release.py](release.py) and [native.py](native.py) | Consumer, example, package, ABI and native case preparation and checks |
| Cargo tests, C++ probes and native helpers | Execute the actual checks and effect-specific workloads |
| [records.py](records.py) and [analysis.py](analysis.py) | Input identity, retained evidence, integrity/export and independent result analysis |
| [windows.ps1](windows.ps1) | Windows developer-environment acquisition and Authenticode inspection |
| [sdk.py](sdk.py) | Explicit maintainer SDK acquisition, pinned extraction and offline integrity checks |
| [proton.py](proton.py) | Experimental Linux cross-build, isolated Proton launch and shared single-frame GPU workload |
| [proton_runtime.py](proton_runtime.py) | Bounded local runtime import, checksum inventory and enforced offline launch |
| [proton_timeout.py](proton_timeout.py) | Readiness-synchronized Windows timeout fixture, prefix ownership and independent cleanup oracles |

The CLI and [CI](../../.github/workflows/ci.yml) currently invoke Python directly.
Optional Bash or PowerShell entry scripts may provide convenient launchers. They
must pass arguments to Python and return its exit status without duplicating
profile selection, test logic, watchdogs or report generation. Direct Python
invocation must remain available.

Platform-specific helpers may perform narrowly scoped environment preparation or
inspection, as `windows.ps1` does today. Their tools must be prerequisites only
for the checks that use them. Portable orchestration must not require PowerShell
or another platform's shell. Additional platforms should extend the affected
cases and helpers while sharing orchestration and evidence handling. Portable
verification does not establish native SDK support on that platform.

Reports must distinguish passed checks from checks that were not executed,
including the reason for missing tools. An unavailable PowerShell parser must
not count as a passed PowerShell syntax check. If that check is required by the
selected profile, its absence must prevent acceptance. Process fixtures,
including negative controls, must clean up their processes even when startup or
assertions fail.

Result JSON is written through a complete pending file and atomic replacement.
On Windows, replacement access/sharing/lock errors (5, 32, 33) are retried up to
eight attempts, with at most 1.27 seconds of backoff per save. Persistent errors
still fail and retain the pending file; other storage errors are not retried.

[archive/check.py](archive/check.py) separates portable archive integrity and
Python/JSON syntax from PowerShell syntax. It parses PowerShell sources when
`pwsh` is available; otherwise that subcheck is explicitly `not_run` /
`not_evaluated`, with the reason retained. Windows `ci` and `release` require
PowerShell parsing through `--require-powershell`; a missing parser fails those
checks. Portable archive acceptance alone makes no PowerShell syntax claim.

## Integrated API walkthroughs

The normal external consumer example checks copy and compile the package's
`examples/upscaler_workflows.rs` on stable and MSRV. On Windows, distinct
`walkthrough.{local,movable}.{simple,provider,abort,utility}` cases also require
all six core entry points and both optional utility compositions to remain
available under their intended features. They use public imports only and
`deny(unused_must_use)`. These are compile checks, not execution of a renderer;
existing controlled/native workloads separately verify recording and cleanup.
The seven targeted consumer cases for discarded builder/selection/status results
and deliberate discard check the compiler diagnostics with no native execution. Lint
rejections use Cargo JSON compiler messages: the exact lint code must have error
severity and a nonzero process exit. Warning-only, wrong-code, successful or
merely echoed lint text cannot satisfy that check; normal E-number controls
retain their existing exact diagnostic requirement.

## Commands and claims

The separate [callback ABI/native spike](callback-spike-protocol.md) uses
`python tools/verification/callback_spike.py --native --sdk-root PATH --adapter-index 0`.
It reuses shared execution/evidence and native equipment for R03/U04's C/Rust
aggregate exchange and callback/allocation observations, without changing
ordinary/release profiles or implementing the public callback API.

Upscaling image-quality validation is on hold by maintainer direction
(2026-10-14); see the [verification priority](../../SCOPE.md#verification-priority).
Temporal quality and sharpening-effect profiles remain available as optional
experiments, with recorded failures preserved. Resume that research only when
explicitly selected. Wrapper ABI, mapping, safety, native execution and cleanup
checks continue; held quality criteria are not feature or release gates.

From the repository root:

```sh
python tools/verification/run.py list
python tools/verification/run.py plan ci
python tools/verification/run.py run fast
python tools/verification/run.py run ci --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run consumers
```

Every `plan` enumerates stable case IDs, claims, exact native selectors, scopes and
ordered prerequisites. [profiles.py](profiles.py) owns the current inventory;
[consumer-cases.json](consumer-cases.json) preserves downstream snippets and
[native-cases.json](native-cases.json) preserves regression selectors/markers.
This inventory is separate from [ABI coverage](../../docs/ABI_COVERAGE.md).

Each downstream snippet has its own immutable source file and binary target.
Positive and negative controls cannot reuse a previous snippet's Cargo freshness
result when the filesystem timestamps of rapid rewrites are indistinguishable.

The reserved `vulkan` feature deliberately fails to compile in wrapper `0.3.0` /
sys `0.1.2`. Consumer profiles compile implemented features explicitly for both
crates, verify the reserved feature/forwarding, then require the deliberate Rust
diagnostic for Vulkan alone and `--all-features`. Missing-feature resolution
errors, unrelated compiler failures, timeouts and aborts cannot satisfy these
controls. The same cases run on MSRV and against isolated packaged consumers.

`native-resource-formats` verifies the remaining external format roles under
[the resource-format protocol](resource-formats-protocol.md): fixed typeless
views, SNORM/shared-exponent colour, load-only RGB32F, D16/D24 depth, motion,
exposure and mask access. Both public owners select 3.1.5/4.1.1; generation
selects 3.1.5. It checks metadata, native calls, input integrity, fenced writes
and cleanup, without FSR-output or mask-algorithm comparisons. Run with
`--native --sdk-root <local-sdk-root> --adapter-index <index> --require-passed`.
The separate `format_capabilities.py` inventories helper mappings/device bits
without SDK dispatch; `resource_formats.py --direct` is the independent C++
experiment coordinator. Neither is a hidden production probe.
Successful child raw payloads are discarded after full evaluation and compact
hash/measurement capture, keeping disk usage bounded; failed payloads are retained.

`native-formats` checks D035's additional typed colour/output, R16F device depth
and reactive-generation colour roles through both public owners. Its separate
pinned-header C++ consumer builds its own native contexts/descriptors; independent
input encoders, readbacks, device bits, write/finite checks and ordinary diagnostics
gate byte comparisons. Generator cases select 3.1.5; existing 4.1.1/2.3.4 no-write
results are not reclassified. See the [typed-format protocol](typed-formats-protocol.md).

`native-depth-formats` verifies D037's four D32/D32S8 typed/typeless allocations
on both owners and selected 4.1.1/3.1.5. DSV clears produce the depth; independent
shader readback checks the exact depth view/plane, and post-dispatch copies audit
depth and stencil. Public/C++ and fixed-geometry R32F baseline outputs must match
exactly. Typed depth requires OPTIONS3 casting capability. See the
[depth-format protocol](depth-formats-protocol.md); use the same native arguments
below with `native-depth-formats` as the profile name.

```sh
python tools/verification/run.py run native-formats --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Native case `groups` explicitly select the transfer and transfer-cleanup baselines.
`legacy_name` retains stable case identities for historical comparisons; its
spelling does not determine group membership.

Current test equipment uses `gpu_helper.dll` / `gpu_helper.lib` and `gpu_*`
exports. `FSR_SDK_TEST_GPU_DLL` selects the helper, `FSR_SDK_TEST_GPU_CONTROL`
selects its no-provider control, and transfer/sharpening output controls name
their protocols. New logs use `Upscaler provider:`, `Public queries:`,
`SHARPENING_*`, `Transfer baseline`, `Transfer cleanup` and `Context native destroy:`.
The sharpening/transfer auditors also recognize their frozen historical labels;
historical files and case IDs are unchanged. The
[naming/refactoring verification](../../docs/release/2026-10-15-verification-naming-cleanup.md)
records local checks. The [Windows confirmation](../../docs/release/2026-10-15-windows-feature-batch.md)
covers helper/reference builds, controlled inspection, recording-ahead and
supplemental transfer/cleanup regressions. Rebuild helpers
and test executables together; old helper binaries/environment names are not
compatibility inputs to current runs.

| Profile | Required claims |
|---|---|
| `fast` | Formatting and default workspace tests |
| `ci` | Infrastructure/archive integrity; check/test/Clippy/docs for default, movable, no-default, no-default+movable, default+utils, default+movable+utils and no-default+utils; public consumer diagnostics, Windows/DX12 dependency isolation, implemented-feature package compilation, expected Vulkan compile errors and complete feature-enabled examples; Windows also requires archived PowerShell syntax and compiles independent ABI probes |
| `msrv` | All targets and full tests for the same seven configurations; consumer boundaries and feature-enabled examples on Rust 1.88.0 |
| `abi` | Four pinned header hashes and independent common/creation/dispatch/query/callback C++ assertions; no runtime execution |
| `consumers` | Feature/platform gates, private construction/fields, family and trait boundaries, production dependencies and complete README/Rustdoc examples |
| `native-upscaler` | Signed pinned deployment; metadata/resource/memory queries plus quality/size/jitter utilities for both families before lifecycle/dispatch regression, bounded pre-exposure/depth/motion/automatic-exposure/active-render/supplied-mask/linear-HDR dispatch, both movable modes, three M9 integration repetitions and independent 144-readback/78-comparison audit |
| `windows-upscaler-followup` | Current CI, exact MSRV, consumers/examples and independent MSVC ABI before native regressions, including both-family tuning value/default → dispatch → fence → cleanup. [Windows handoff](../../docs/WINDOWS_FOLLOWUP.md); public generator fixtures compile/run as part of Windows CI, no actual-provider public-generator claim |
| `experiment-reactive-mask` | Test-only raw generator descriptor; independent pinned-header ABI, equipment/identical-colour controls, all 16 flag combinations, active-region shrink/grow, generated-mask readback and upscale integration. Native rejection remains observed, failed cleanup fails; no public generator verification. [Protocol](../../docs/WINDOWS_FOLLOWUP.md#2-isolated-d030-raw-spike) |
| `experiment-public-reactive-mask` | The same equipment and write oracle through both public generator families, with separate controls and all 16 flag combinations per family. A successful native return with an unchanged mask fails. Public-family identity is required; independent children continue after a failed generation case |
| `experiment-selected-reactive-mask` | Explicit selection of pinned providers 3.1.5 and 2.3.4 for both public families using the same controls/all flags. Missing providers fail; exact live provider IDs must match selection. No automatic fallback or generic support policy |
| `experiment-performance` | Optimized current-source P01 CPU baseline: controlled validation/COM/allocation counts, prepared raw-ABI reference, separate context lifecycle, diagnostic control and three native process repetitions. No GPU timing or C++ parity verdict |
| `experiment-provider-performance` | Eight fixed balanced C++ provider 4.1.1/default pairs on the current driver; maximum-render SDR 1080p GPU/CPU/context observations, two debug controls, no speed gate |
| `experiment-provider-performance-build` | Compile/signature/ABI preparation only; no device creation or GPU child launch |
| `experiment-provider-setup` | Optimized default/selected-provider six-query startup sequences, four warm-ups and 32 alternating paired blocks. Records 64 whole-path CPU timings, no GPU dispatch or speed gate. [Protocol and limits](../../docs/DX12_RECOVERY.md#measure-provider-setup-before-caching); actual Windows/native measurement remains required before caching decisions |
| `experiment-variable-output` | [Raw geometry spike](variable-output-protocol.md): three pinned providers, 20 allocation/active-output/motion/sharpening workloads each, independent equipment control, full input/output readback audit and terminal cleanup. Padding writes are observations; native rejection remains observed. No public variable-output implementation or allocation-policy acceptance |
| `experiment-motion-scale` | [Raw format/scale spike](motion-scale-protocol.md): RG16F/RG32F, equivalent positive/signed asymmetric encodings and wrong-sign controls for X/Y/combined motion on 4.1.1/3.1.5. Independent inputs, resource parity, fences and terminal cleanup; exact-equivalence hypotheses separate from integration. No public API or general numerical-domain acceptance |
| `native-motion-scale` | [Public motion verification](public-motion-protocol.md): both owners on 4.1.1/3.1.5; RG32F render/display, finite signed overrides, mixed-sign jitter-inclusive equivalence, UAV-capable motion and active output. Independent packed-input/output audit, device bits, C++ parity, fences and cleanup; no native numerical-domain, jitter-cancellation-consumption or quality claim |
| `native-depth-formats` | [Depth-buffer verification](depth-formats-protocol.md): both owners on 4.1.1/3.1.5, four typed/typeless D32/D32S8 allocations, DSV clears, depth SRV shader readback, depth/stencil copy audit, conditional casting support, standard/reversed depth and active regions, exact C++/R32F comparisons and cleanup |
| `native-variable-output` | Accepted D024 output slice through both public families on selected 4.1.1/3.1.5: maximum allocations, stable/aligned/rounded/independent-axis output, four motion encodings and sharpening off/on. Independent packed-input/output audit, C++ resource parity, separate no-dispatch controls, fences and cleanup. Native rejection fails; padding preservation and out-of-resource safety are not certified |
| `experiment-cpu-comparison` | P03 independent optimized C++ executable versus separate Rust wrapper/prepared-raw processes; six balanced blocks, three warm-state control blocks, canonical descriptor/provider/device/output matching and diagnostic controls. CPU recording only; descriptive comparisons, no speed gate |
| `experiment-gpu-timing` | P04 one 1080p output, provider-queried render size and jitter, independent Rust/C++ consumers, same-queue GPU timestamps and separate CPU recording, four balanced process blocks, empty brackets and debug controls. Descriptive only; no speed gate |
| `experiment-active-timing` | P05 fixed-allocation active-render shrink/grow versus stable maximum/smaller controls; six debug children and four balanced Rust/C++ process blocks, independently audited inputs and transition CPU/GPU intervals. Descriptive only; no speed gate |
| `experiment-feature-comparison` | P02 extends the active-render controls with both supplied masks and one linear-HDR case; ten debug children and forty timed Rust/C++ children, full mask/read-only input audits and separate CPU/GPU intervals. Descriptive comparison; no speed-parity gate |
| `experiment-performance-pilot` | Ten randomised fresh-process blocks across the five existing workloads; separate debug controls, matched execution and block-level CPU noise estimates. No performance verdict |
| `release-performance-reconfirm` | Fixed 256 fresh paired P08 blocks per workload, 5-hour primary cap, prior confirmation for planning only; same CPU criterion and supplements, no pooling or new adaptive pilot |
| `experiment-performance-confirm` | Pilot-derived frozen sample count/order and a 5 µs upper budget on average additional public Rust CPU dispatch cost per workload; descriptive GPU intervals, separate preparation/raw/allocation runs and matched fresh-process/warm lifecycle. Experiment completion and speed outcome are separate; no release gate |
| `experiment-color-signal` | Explicit linear/sRGB/PQ test conventions, independent Decimal input oracle, exact GPU input readbacks and RGBA16F output analysis; neutral signal-domain hypotheses precede pre-exposure packing and exposure/automatic controls. Integration is required; numerical hypotheses and failures are recorded separately. No general colour-conversion, internal-order or quality verdict |
| `proton` | Linux-hosted Windows Rust/C++ builds, explicit pinned-byte trust input, observed AMD module paths, public queries for both families and one shared single-frame GPU workload; Windows diagnostics recorded separately |
| `proton-offline` | Prepared local runtime integrity, missing-input controls and the same cross-build/query/single-frame proof with every umu launch in an isolated network namespace; mandatory Vulkan core/synchronization checks |
| `proton-timeout` | Ready Windows parent/detached child, real watchdog expiry, unrelated-prefix survival, stopped-descendant/heartbeat oracles and subsequent public-query/GPU controls using the prepared offline runtime |
| `proton-temporal-quality`, `proton-temporal-long`, `proton-temporal-reset`, `proton-temporal-jitter` | Selected temporal protocols through the same Proton preparation/launcher, exact input oracle and independent analysers; frozen Windows image criteria, three process blocks and mandatory Vulkan core/synchronization validation |
| `release-artifacts` | Exact Cargo archive pair, byte/content/licence audit, isolated registry, seven feature configurations, current/MSRV tests and consumers, full examples including utils and eight workflow calls per compiler, final unchanged-byte/inventory audit; no GPU claim |
| `release-coverage` | Focused exact-package checkpoint: independent ABI, registry-only stable/MSRV examples/workflows, native regressions and public callbacks, production CB exchange on stable/MSRV; no publication or full source/consumer-contract qualification |
| `release` | Source current/MSRV checks, ABI, archive checks and native source/artefact verification, including packaged public callbacks and production CB exchange; shared prerequisites occur once; no publication |
| `release-dry-run` | Exact archive checks followed by `cargo publish --dry-run`; requires `--online`, checks regenerated package bytes, and always aborts uploads |

For verification profiles, offline is the default. `--online` permits Cargo network access and pinned header
acquisition. `--toolchain` selects the current compiler; MSRV cases always use
`1.88.0`. Cargo caches compiled artefacts under `target/verification-build`;
verification passes are never reused. Resume is disabled.

Use `--require-passed` for CI/release acceptance: only a finalized
`run.json.status` of `passed` returns zero. Without it, both `passed` and
`observed` return zero. The flag changes only the exit policy; recorded outcomes,
evidence, case criteria and intentional unit-test skips are unchanged. CI enables
it for stable and MSRV. `verify-run` checks evidence integrity and completeness,
not acceptance.

CI's Rust cache is restricted to `target/verification-build` alongside Cargo's
registry/git caches. Attempts under `target/verification` and exports under
`target/verification-evidence` are excluded. The cache key prefix changes when
this boundary changes so older caches cannot restore evidence directories and
block exclusive export, even when those restored directories are empty.

The supplied-mask native case runs three continuous frames for each mask alone
and both together, under explicit/automatic exposure with active render sizes
320×180 → 256×144 → 288×162. Distinct R8_UNORM inputs retain maximum allocations;
full mask/input readbacks, output sentinel/finite checks, fences, debug diagnostics
and terminal cleanup check integration. Synthetic masks are forwarded directly;
these checks neither establish image quality nor prove which texels a provider
sampled. The case is selected in both source and extracted-package native checks.
[Windows fixtures/MSRV/C++ ABI pass](../../docs/release/2026-10-15-windows-feature-batch.md).
The original attempt failed in wrapper validation before AMD dispatch because
provider 4.1.1 reported no optional mask bits. The implemented
[D025 amendment](../../docs/adr/D025.md#amendment-rationale-and-implementation-status)
removes that query-based restriction while retaining required-mask and resource
checks. The [correction verification](../../docs/release/2026-10-15-mask-forwarding-verification.md)
passes all 18 native mask frames and all 29 standard native-upscaler cases on the
same provider. The earlier failure is preserved; successful forwarding/execution
does not establish whether the provider consumes masks.

The linear-HDR case uses two fixed-size contexts (explicit and automatic
exposure), each with three continuous frames of constant scene RGB `(8,4,2)` and
matching pre-exposure `0.25 → 1 → 4`. Full fixture input readbacks compare uploaded
bytes after fencing. Finite/written output and red values strictly between 1 and
64 catch clipping to `[0,1]` and stale sentinel values, without selecting an image-
quality threshold. Diagnostics, explicit teardown and retained frame dependencies
use the existing harness. Both source and extracted-package profiles select
`dispatch.linear-hdr`; [Windows native execution passes](../../docs/release/2026-10-15-windows-feature-batch.md)
for both exposure modes and all six frames. Controlled colour-hint forwarding
also passes; actual non-linear GPU signal semantics remain unverified.

## Native performance characterisation

[D043](../../docs/adr/D043.md) now requires a fresh candidate performance check
before every release. The [P08 adaptation](../../docs/design/release-performance.md)
now reuses the P06 equipment below and three P07 setup processes. It explicitly
selects the 5 us CPU criterion for 0.3.0, binds source-matched candidate archive
hashes and leaves release approval to result review. Historical samples are not
reused as current measurements. No timing batch is started by preparing it.

### Recorded P08 invocation

[P08 is closed with maintainer-accepted limits](../../docs/release/2026-10-29-performance-review-closure.md).
Full CPU outcomes remain inconclusive; the accepted 222-block temporal subset,
nominal bounds and inference limits remain explicit. The supplemental and
provider reviews are complete. Final archive qualification is separate; no new
performance batch is selected for 0.3.0.

For reproduction, the completed fixed confirmation was started from the
repository root with:

```powershell
python tools/verification/release_performance.py reconfirm --planning-batch target/release-performance/20261027T213639Z-release-performance-6e3adfcf6698 --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The coordinator packages/checks candidate sources, runs all stages serially and
writes status/logs under target/release-performance. The P06 batch is a linked
sibling folder to keep Windows compiler/readback paths short. The chat/agent need not
remain active. Send the printed status.json path afterwards, including on failure.
Keep the terminal/machine running and the checkout/SDK unchanged. A read-only
preview is `python tools/verification/release_performance.py plan --reconfirmation`.
The original run mode remains available for initial adaptive pilot/confirmation.

Preparation tests mock every native launch; ordinary compilation checks do not
execute the ignored timing tests. The current Rust timed arm calls public record
directly, with input assembly outside the bracket. Candidate matching excludes
Cargo-generated manifest/lock/VCS metadata and compiles the matching checkout;
it is not final extracted-package qualification.

The initial P08 batch completed 515 checks in 33.55 minutes. The
[fixed follow-up](../../docs/release/2026-10-28-performance-confirmation-plan.md)
completed 256 fresh blocks/workload in about 3 hours 18 minutes, without a new
pilot or pooling. Its completed review and accepted subset are linked above.
For an explicitly requested reproduction, reserve at least 50 GB free; the
five-hour cap applies only to primary confirmation, with other stages additional.
A completed batch can still report inconclusive/exceeds-budget and always needs
review; successful execution is not automatic release approval.

### Small provider A/B probe

The [completed provider comparison](../../docs/research/records/2026-10-29-exp-provider-performance-comparison.md)
passes 30/30 cases in 95.477 seconds. It compares explicit 4.1.1 with default
4.1.2 * on the same current driver: warmed GPU means 1.047/1.059 ms, with a
difference interval spanning zero; context costs differ more clearly. The
[preparation record](../../docs/research/records/2026-10-29-exp-provider-performance-preparation.md)
retains separate compile-only evidence. It uses the existing independent C++ consumer, maximum-render
SDR 1920x1080 output, two debug controls and eight fixed balanced pairs. It
records warmed GPU/CPU intervals and separate context lifecycle costs. Provider
availability and actual initial context identity must match the frozen IDs;
there is no silent fallback. Query and creation share the explicit override.
Inputs/clocks/frame history must match; output bits may differ between providers.

For a separately requested reproduction, start from the repository root with
games and other heavy work closed:

```powershell
python tools/verification/run.py run experiment-provider-performance --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/provider-performance
```

The recorded batch took about 95 seconds; future build/runtime duration can
vary. The earlier 3–5 minute planning estimate is not a stopping criterion. The primary phase has a 600-second allowance checked before each child;
build/debug work and bounded child completion are additional. Each child has a
120-second watchdog. Readbacks occupy about 300 MB; reserve 1 GB free. Keep the
checkout unchanged. Send the printed run.json path after completion or failure;
analysis/provider-performance.json contains the descriptive paired estimates.
The chat need not remain active. No driver switch or rollback is performed.

The [protocol](provider-performance-protocol.json) fixes count/order/analysis
before execution. There is no pilot, trimming, early success, optional extension
or numerical release verdict. This provider comparison does not isolate a driver
change or replace P08. A read-only preview is:

```powershell
python tools/verification/run.py plan experiment-provider-performance
```

The separate experiment-provider-performance-build profile stages trusted SDK
files, compiles ABI/helper/probe code, and never executes a GPU child. Preparing
this equipment does not start the A/B measurement.

### Reused P06 equipment

For an unattended complete run, the following command performs a fresh pilot,
freezes a compatible selection and runs the full confirmation sequentially. No
agent or periodic log inspection is required. Runtime depends on the fresh
pilot's frozen selection. The [completed recorded batch](../../docs/research/records/2026-10-16-exp-native-performance-confirmation.md)
took approximately 93 minutes in total: 7.5 minutes for the pilot, 78.75 minutes
of primary confirmation measurement, plus setup and supplemental measurements.
The protocol caps planned primary measurement at two hours. Keep the checkout
unchanged.
Individual profile logs and final result paths are written to a fresh directory
under `target/performance-batches`; its `status.json` records completion or
failure. A failed stage stops the batch. The batch uses the same coordinator,
validity checks and acceptance policy as the individual commands below.

```powershell
python tools/verification/performance_characterization.py run --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The opt-in pilot/confirmation profiles implement the maintainer-selected CPU
budget in [the protocol](performance-characterization-protocol.json). They reuse
the existing five workload generators, independent C++ consumer and full input,
history, mask/HDR and output auditors. Old protocols and evidence are unchanged.

```powershell
python tools/verification/run.py run experiment-performance-pilot --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/performance_characterization.py select --pilot target/verification/<pilot-attempt> --output target/performance-selection.json
python tools/verification/run.py plan experiment-performance-confirm --performance-plan target/performance-selection.json
python tools/verification/run.py run experiment-performance-confirm --performance-plan target/performance-selection.json --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The ten pilot blocks have no speed verdict. Selection uses the largest pilot
standard deviation of paired process-mean CPU differences to target a 1 µs
one-sided margin, with at least 24 and at most 120 confirmation blocks. Its
conservative runtime estimate must fit two hours. If the precision target cannot
fit, the selection says so explicitly; the 5 µs budget is unchanged. Selection
is exclusive, checksum-linked to the pilot and all current producer inputs, and
fixes the sample count, randomised balanced schedule and analysis before new
measurements. A producer/protocol change requires a fresh pilot. The selected
plan is an external input in the confirmation inventory. Pilot samples are
excluded from confirmation estimates. Do not extend a completed confirmation
to obtain a preferred verdict.

The primary estimate is the equal-weight average of Rust-minus-C++ process-mean
CPU dispatch times, over frames 32..159. Student t bounds use process blocks as
replications, with approximate coverage conditional on independent, reasonably
regular block differences. Per-workload bounds are pointwise, not simultaneous
confidence bands. All five upper one-sided 95% bounds must be strictly below
5 µs for the overall within-budget result; a lower one-sided bound strictly
above 5 µs establishes exceeds-budget, otherwise that workload is inconclusive.
Two-sided 95% intervals, consumer mean/median/nearest-rank p95, warm-up windows,
and first-shrink/grow/settling/steady segments remain visible. Primary averages
include transition frames; the claim never bounds every frame or transition.
GPU differences have uncertainty estimates but no numerical speed gate.

Separate supplemental processes measure validation/descriptor preparation,
prepared raw Rust calls and Rust allocation/reallocation requests/bytes. Raw
diagnostics retain an unused wrapper context for provider queries, so they are
not a clean algebraic decomposition of primary overhead. Allocation counters
cover the dispatch calling thread in the Rust executable, not native/driver
heaps or other threads. Timing from instrumented runs is excluded from the
primary estimate. Eight fresh-process lifecycle blocks per SDR/HDR type retain
total costs and matched differences for loader acquisition, the first provider
render-size query, first creation after that query, fenced used-context teardown,
and subsequent unused-context creation/teardown. The first dispatch is reported
separately. Neither fresh processes nor the retained runtime imply cold caches.
C++ warm creation reuses its stable creation descriptors. Rust lifecycle runs
suppress the test-only teardown trace environment lookup; a test-only TLS branch
and allocator hook remain, so sub-microsecond lifecycle differences need caution.

Correct execution/evidence acceptance is separate from performance: the profile
can pass with an inconclusive or exceeds-budget speed result. A missing child,
invalid input/output, equipment mismatch or elapsed-time stop prevents complete
evidence acceptance. A watchdog stops before another primary child starts after
two hours; an already running child retains its 240-second timeout. No early
success stop is allowed. Serial fences, CPU uploads and checkpoint gaps remain
symmetrical but can affect clocks and the workload's representativeness. These
are bounded source-build observations, not release, sustained-throughput, FPS,
image-quality or general hardware guarantees. Full raw diagnostics stay under
ignored `target/`; permanent records retain compact block/process statistics,
selection, producer identities and failed-attempt outcomes.

## Bounded colour-signal experiment

Current code uses `color_signal.py`, `native_color_signal`,
`gpu_set_color_signal`, `FSR_SDK_COLOR_*` and `COLOR_*` machine labels.
The `experiment-colour-signal` / `experiment-colour-signal-controls` commands,
old module/test entry points, native selector and helper export remain compatibility
aliases. Native inputs accept either environment spelling; conflicting
`FSR_SDK_COLOR_*` and `FSR_SDK_COLOUR_*` values fail before native setup.
Metadata readers accept both log-label spellings with the same validation.

Stable `colour.*` case IDs remain evidence identifiers. New analysis writes
`analysis/color-signal.json`; retained historical output is unchanged. The
American-spelled protocol files are byte-identical copies of the frozen originals,
which remain at their established paths for historical links and the controls'
`base_protocol` reference. No numerical budget or signal prediction changes.
The [migration verification](../../docs/release/2026-10-15-color-signal-naming-verification.md)
records native execution and compatibility checks.

```sh
python tools/verification/run.py run experiment-color-signal --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

This separate opt-in profile requires existing NumPy and the standard Windows
native prerequisites. The [frozen protocol](color-signal-protocol.json) defines
eight neutral constant patches, explicit PQ normalization (one relative linear
unit is 100 nits as a test convention), half packing, three process repetitions
and flat-interior numerical discrimination budgets. Python generates bytes;
an independent high-precision inverse audits the declared input signal. The
helper copies bytes without encoding and checks all four input readbacks after
fencing. The Rust public API supplies flags/pre-exposure/exposure independently.
Each fresh context runs three continuous frames, then destroys after fencing.

Five neutral arms (linear, sRGB/PQ with unspecified or matching hints) precede
matched pre-exposure packing `E(P*L)` versus `P*E(L)`, explicit exposure variation
and automatic exposure controls: 16 arms × three processes × three frames.
The analyser retains candidate output-domain matches and failures, rather than
turning a completed integration run into blanket semantic acceptance. Restored
output alone may not identify the internal decode/pre-exposure order. No colour
converter is added to the wrapper, no image-quality study is resumed and no
standard/release acceptance criterion changes. Full input/output payloads remain
local under ignored `target/`; records retain compact patch statistics and hashes.

`experiment-color-signal-controls` uses the same command prerequisites and frozen
numerical budgets. Its [follow-up protocol](color-signal-controls-protocol.json)
holds pre-exposure at 0.5/2 in ten static arms and repeats four non-linear
transition arms with reset on every frame, each in three fresh processes.
It diagnoses the initial transition discrepancies without changing their verdicts.
The [native experiment record](../../docs/research/records/2026-10-15-exp-non-linear-colour-signals.md)
retains 270 completed GPU frames, the 15 initial failed signal predictions and
the successful static/reset controls. Its integration pass is not a universal
colour-semantic acceptance claim.

## CPU performance baseline

```sh
python tools/verification/run.py run experiment-performance --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
```

This opt-in Windows experiment uses pinned signed local SDK inputs and the
ordinary ABI prerequisites. The Rust test executable uses Cargo `--release`;
the helper uses MSVC `/O2` and the controlled DLL uses `rustc -O`.
One separate debug-enabled native child checks dispatch, fences, final finite/
written output and teardown. Timed native children do not enable the debug layer;
GPU-based validation is never enabled by this profile. All native work is
submitted once and fenced before reuse or destruction, outside CPU timing.

Each of three repetitions starts fresh controlled and native processes. Controlled
dispatch has 10,000 warm-up iterations and 31 batches of 1,000 calls for each of
empty loop, prepared raw ABI, inspection/descriptor construction and full wrapper.
Arms rotate within batches. Separate lifecycle batches create 1,000 owners into
preallocated storage, then destroy them, for 31 samples per operation. Stub COM
entry counters and thread-local Rust allocator counts are armed separately from
timing; disabled counter checks remain part of the controlled stub cost.
Native heaps/DLL allocations are excluded from Rust counts. COM counters record
external entries, excluding stub-internal QI/AddRef implementation calls.

Native work uses the current local ownership family, default explicit exposure,
standard finite depth, render-resolution UV motion, sharpening disabled and
fixed `320×180` render / `640×360` output. Pre-exposure is `1`, jitter is zero,
frame time `16.667 ms`, camera `(0.1,100,pi/3,1)`, and only each context's first
frame resets. Raw and wrapper contexts share fenced external resources but have
separate histories. Order alternates each frame; 32 warm-up frames per arm precede
128 timed calls per arm, with separately timed real COM inspection. Final output
is checked for finite/written pixels; this is not an image-quality comparison.
Before dispatch a negative control confirms untouched sentinel output is detected;
the sentinel is restored before each arm's final dispatch so stale output cannot
satisfy its finite/written check. These control operations are outside CPU timing.
Native creation and destruction have eight warm-up pairs and 31 timed pairs.

The raw reference uses `FfxLibrary::ffxDispatch` with a pre-inspected descriptor
and shares the wrapper's pinned creation mapping. It isolates validation and
ownership overhead, not independent C++ integration. Native submission, waits,
resource setup, output readback, descriptor preparation for the raw call and
logging are outside dispatch timing. Test-only teardown tracing still performs
its environment lookup; these results describe this test executable.

Case records retain all samples, min/median/p95/max distributions and paired
wrapper-minus-raw differences in nanoseconds. The p95 is sorted index
`floor((n-1)*0.95)`; no timer/empty-loop subtraction is applied. Missing/duplicate
sample IDs or missing zero-allocation evidence reject protocol completion.
Passing means the measurement protocol completed; no speed or regression
threshold is asserted. Small serial workloads, unpinned CPU scheduling, thermal/
clock changes and other applications limit inference. P02's GPU timestamps,
independent C++ parity and later-feature comparisons remain separate.

## Independent C++ CPU comparison

```sh
python tools/verification/run.py run experiment-cpu-comparison --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
```

The [protocol](cpu-comparison-protocol.json) fixes the small P01 native input
profile, launch order and timing boundaries. The standalone
[C++ consumer](../../crates/fsr-sdk/tests/native/cpu_reference.cpp) uses pinned AMD
headers and dynamically resolves SDK functions; it calls no Rust code. The
[Rust arms](../../crates/fsr-sdk/src/upscaler/dx12/tests/cpu_comparison.rs) use the
public wrapper or prepared raw bridge. Each process owns one context and receives
the same shared-helper static inputs, resource allocation and serial fencing.
The helper performs no AMD calls. C++ uses `/O2 /MT`, Rust Cargo release defaults;
this is the local-owner path with the test binary's movable feature compiled in.

All three arms use the same helper's QueryPerformanceCounter/frequency exports.
Each timer brackets only dispatch; frame construction, setup, submission, fence,
readback and logging are excluded. Return/error handling inside the Rust call and
the indirect-call arrangements can differ; the minimum C++ arm also omits Rust's
validation. No zero-overhead or language-equivalence conclusion follows by
subtracting these timings. Measure 256 empty clock pairs without subtraction.

Primary runs prepare raw metadata once before the frame loop; they do not repeat
P01's per-pair pre-inspection. Six blocks cover every permutation of three arms,
balancing their positions. Each child retains 32 warm-up and 128 measured frame
samples; comparisons use differences of process medians within a block, not
cross-process frame pairs. Three additional blocks compare the two Rust arms
with/without per-frame pre-inspection. These sensitivity controls also experience
scheduling/clock drift and cannot isolate cache effects by themselves.

Before timing, separate debug-enabled children execute four frames each. Every
child checks untouched sentinel output, independently restores it before its
final dispatch, fences completion, checks finite/written output and device
removal, then destroys the context. Timed children keep debug validation off.
Canonical resource metadata and scalar float bits, native provider, actual
adapter/driver and QPC frequency must match. Exact final RGBA16F hashes must match
within the four-frame diagnostic or 160-frame timing cohort; a difference blocks
interpretation until explained, without imposing cross-driver bit identity.
Full payloads stay in ignored run directories; permanent records retain hashes.

Case records preserve all ticks, warm-up/measured values, empty-clock samples and
distributions; `analysis/cpu-comparison.json` records process-block differences
and sensitivity controls. Protocol completion is separate from a performance
verdict: noisy/inconclusive speed comparisons are valid research outcomes.
No GPU timestamps, FPS, HDR/masks, dynamic sizing or numerical speed gate are
part of P03. The complete profile requires 45 cases; ordinary/release profiles
do not automatically run it.

## 1080p GPU timing pilot

```sh
python tools/verification/run.py run experiment-gpu-timing --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
```

The [protocol](gpu-timing-protocol.json) selects one 1920×1080 output case. Both
consumers query SDK Quality render dimensions before allocation, check the live
context reports the same size, then query its jitter phase and offsets. The
shared analytic ramp translates one render pixel right per frame, with matching
projection-jittered samples and current-to-previous UV motion excluding jitter.
Depth is standard finite 0.5; exposure/pre-exposure are 1; masks are absent and
sharpening is off. Reset occurs only on frame zero. This is integration-validity
coverage, without resuming image-quality research.

The existing performance consumers share test-only mechanics in
[Rust](../../crates/fsr-sdk/src/upscaler/dx12/tests/performance_comparison.rs) and
[C++](../../crates/fsr-sdk/tests/native/performance_reference.h). The C++ executable
still calls the SDK directly through pinned headers and never calls Rust. P03's
selectors/profile remain available. A dedicated `/DFSR_GPU_TIMING` build of the
GPU helper allows one process-local dimension configuration before its sole
harness is created; ordinary helper builds retain their original fixed sizes.
The [timing helper](../../crates/fsr-sdk/tests/native/gpu_timing.h) contains no AMD
calls. Production code, public APIs and dependencies are unaffected.

On the same direct queue, two adjacent `EndQuery` timestamps form an empty
bracket, then another timestamp precedes FSR commands and a fourth follows them.
Provider commands/barriers are inside; fixture upload copies and transitions,
final sentinel restoration, query resolve and texture readback are outside the
FSR interval. `ResolveQueryData` writes four UINT64 ticks to a readback buffer;
a completed fence precedes mapping and reuse. Queue frequency must be positive,
unchanged before/after each frame, and match across processes. GPU intervals use
that frequency; independent QPC brackets retain CPU dispatch time and 256 empty
CPU controls. Neither clock's empty bracket is subtracted.

Two separate four-frame debug controls precede four alternating Rust/C++ process
blocks. Each timed process retains 160 frames, with 32 warm-up and 128 measured
samples. Ordinary debug validation is off for timings; GPU-based validation is
off throughout. Untouched/restored-final sentinels, fully written finite output,
device removal, canonical descriptors, provider/adapter/driver, queried render
size/phase and every jitter bit pattern are checked. Final output bytes must
match within the four-frame or 160-frame cohort. A mismatch rejects the process.

The 22-case profile retains raw CPU/GPU ticks, empty brackets and output hashes
in case `samples.json` files and distributions/process-median differences in
`analysis/gpu-timing.json`. Run it without simultaneous builds/tests. All frames
are serially fenced, and CPU-generated uploads create gaps between submissions;
GPU clocks, thermal state and unrelated system activity remain uncontrolled.
These intervals do not measure frame throughput, FPS or presentation latency.
Differences of nearby process medians are descriptive, with no parity margin or
automatic speed verdict. Cold frames stay in the evidence but are excluded from
the measured distributions. Ordinary/release profiles do not run this pilot.

## Active-render transition timing

```sh
python tools/verification/run.py run experiment-active-timing --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The [protocol](active-timing-protocol.json) extends the P04 timing consumers and
equipment without production API changes. Every context enables dynamic
resolution, with identical maximum input allocations and 1920×1080 output.
SDK Quality determines the maximum render size before allocation and agrees with
the live query. Live Performance queries determine one strictly smaller size;
Rust also checks its pre-create result. Each size has its own live jitter phase,
with offsets queried before the loop using global frame modulo current phase.

Three workloads keep the maximum size, keep the smaller size, or shrink at
frame 64 and grow at 112. Each process retains 160 continuous frames, excluding
the first 32 from measured segments. The analytic ramp translates by
`1 / maximum_render_width` UV each frame, with matching current-to-previous
UV motion and jittered active-region sampling. This keeps its output-space
trajectory fixed across sizes. Inactive colour/motion contain guards; depth is
standard finite and exposure/pre-exposure remain 1. Reset occurs only on frame 0.
No allocation resizing, automatic resets, masks or HDR are introduced.

Separate four-frame debug controls exercise all three workloads in both
consumers, with shrink/grow at 1/3. They require zero debug errors. Timed children
disable debugging: the helper's `UINT64_MAX` error-count marker means unavailable
diagnostics, never zero observed errors. Device-removal checks remain mandatory.
Four timed process blocks alternate C++/Rust order and reverse workload order.
The complete profile has 42 required cases and 30 native child processes.

At frames 31, 63, 64, 111, 112 and 159 (all four debug frames), restore the output
sentinel before FSR and audit complete finite/written output after fencing.
Full colour/depth/motion input readbacks must still match uploads, including
inactive guards. Independent Python predictions audit active corners and an
inactive sample from actual GPU readbacks with a declared binary16 quantization
bound. Input/output FNV-1a diagnostics and final output SHA-256 must match across
consumers and repeated processes of the same workload/history length. Matched-size
stable/transition checkpoints require identical inputs/jitter; pre-shrink outputs
also match while histories are identical. Post-transition histories intentionally
differ, so cross-workload output equality is not required there.

P04's CPU/GPU interval boundaries remain: all generation, uploads, sentinel
restoration, queries, resolve, readbacks, hashing and logging are outside them.
Input-readback resources are diagnostic equipment allocated outside the interval;
effect contexts and input/output allocations stay fixed. Every control uses the
same checkpoint schedule, but serial fencing and diagnostic gaps can affect
subsequent clocks. Run without simultaneous builds/tests.

Retain raw CPU/GPU ticks, empty brackets, queried sizes, every jitter/reset/scale,
input samples and checkpoint hashes in case `samples.json`. The summary at
`analysis/active-timing.json` separates pre-transition, first-shrink, small settling,
small steady, first-grow, maximum settling and maximum steady windows, and compares
each with the same-index stable control. A first-transition value is one sample
per process, not a distribution of independent frame repetitions. Report signed
differences and variability without a speed, equivalence, quality or FPS verdict.
Ordinary/release profiles do not select this experiment; raw final images remain
temporary under `target/`, with compact evidence retained in the research record.

## Supplied-mask and linear-HDR comparison

```sh
python tools/verification/run.py run experiment-feature-comparison --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The [selected-feature protocol](feature-comparison-protocol.json) extends the
[active-render protocol](active-timing-protocol.json) with two maximum-active-size
workloads. It shares setup and optimized consumers, retaining the stable maximum,
stable smaller and shrink/grow controls in the same four balanced process blocks.
Every context enables dynamic resolution; exposure/pre-exposure stay 1 and reset
occurs only on frame 0. Jitter, scene trajectory, checkpoint schedule and separate
QPC/GPU timing boundaries follow P05. There are 62 required cases, ten debug and
forty timed native children, and 6,440 submitted/fenced frames in a complete run.

The mask workload supplies both render-aligned R8_UNORM masks together. Static
rectangles are uploaded once before context timing: reactive 128 at `.25<u<.5`,
transparency/composition 255 at `.5<v<.75`, zero elsewhere. Full GPU mask readbacks
must match uploads at every checkpoint. Independent integer inequalities generate
complete expected FNV-1a hashes, catching incorrect mask patterns even if both
consumers share the same generator. The RGB/depth/motion inputs match the stable
SDR maximum control exactly. Mask consumption and image-quality benefit are not
inferred from successful forwarding or timing.

The HDR workload sets the native linear-HDR creation bit and multiplies the
analytic RGBA16F ramp's RGB by 8, preserving alpha 1, motion, depth and history.
Independent GPU input samples use this declared scale. Every output checkpoint
requires maximum red strictly between 1 and 64 in addition to complete finite/
written-pixel checks, fencing and device diagnostics. No encoding hint, PQ
normalization or changing exposure is introduced. M16's wider non-linear questions
are separate from this selected linear-HDR integration/performance case.

Canonical descriptors include both mask resources or the HDR creation flag.
Within each workload/history-length cohort, provider/device, queries, per-frame
metadata, input/output checkpoint diagnostics and final SHA-256 must match
between Rust/C++ and repetitions. Additional mask allocations and HDR colour
content are declared feature differences; output equality across different
features is not required. Setup, uploads, readback, hashing and logging are
outside timed intervals, though their gaps can affect subsequent clocks.

`analysis/feature-comparison.json` retains the full active-control analysis and
separate feature distributions: signed Rust-minus-C++ CPU/GPU medians by workload
and feature-minus-SDR medians per consumer/block. Raw ticks, warm-up, empty brackets,
sampled inputs and full mask hashes remain in each child's `samples.json`.
Debug children require zero errors; timed debug-error counts remain unavailable.
The seven feature-evaluator tests reject wrong flags/mask presence, incomplete
or incorrect mask evidence, SDR/clipped/non-finite HDR and mismatched controls.
Ordinary/release profiles do not select this experiment. No numerical speed gate,
equivalence margin, universal performance, FPS or quality claim is made.

## Experimental Proton verification

The recorded Deck/Machine runs are October 13/14 historical evidence. They
precede the replacement API and do not qualify final 0.3.0 packages. A fresh
bounded current-candidate recheck is explicitly open in the
[release checklist](../../docs/release/0.3.0-preparation.md#archive-coverage-before-final-verification).
By default, profiles build checkout tests with cargo-xwin. `proton`,
`proton-offline` and `proton-timeout` now accept the
[exact-package mode](#exact-package-native-checks): both Rust tests and the C++
GPU helper use the selected archive pair, with isolated Cargo registry inputs
and a final byte/inventory audit. This binding has unit coverage on Windows;
actual Linux cross-build and Proton/GPU execution now pass on Deck and Machine.
The [consolidated candidate verification](../../docs/release/2026-10-29-candidate-verification.md)
reconciles their shared Linux archive hashes, records Deck's additional offline/
timeout proof, and distinguishes the Windows/P08 pair. Final publication-package
reconciliation remains open. Record availability
and results on the chosen Linux/GPU hosts rather than reusing the old pass. Broader temporal
research remains separately scoped; Proton is not native Linux/Vulkan support.

The `proton` profile implements the bounded Linux-hosted proof described by
[D019](../../docs/adr/D019.md), accepted on 2026-10-13. The initial V07 proof and
bounded V07e portability checkpoint are verified. V07d temporal regression is on hold;
bounded V08 local-import/offline execution and V09 timeout cleanup are verified
on Deck, with limits recorded in their sections below and the
[roadmap](../../ROADMAP.md#verification-and-compatibility). The
[Steam Deck experiment](../../docs/research/records/2026-10-13-exp-proton-linux-cross-build.md)
records the tested configuration, failures, provider and GPU output. It does not
establish a supported Linux SDK target, native Vulkan backend or complete native
regression coverage. The earlier Windows-prepared
[handoff spike](proton-spike.md) remains available separately.
The [Vulkan-validation follow-up](../../docs/research/records/2026-10-13-exp-proton-vulkan-validation.md)
records passing core/synchronization checks and the locked development shell,
including retained failed setup and activation-check attempts.
The [Steam Machine experiment](../../docs/research/records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
records FSR 4.1.1 GPU and INT8 shader evidence under Proton 11 and Experimental,
plus the newer runtime's drive mapping and validation differences.
The [integrated comparison](../../docs/research/platform-and-backends.md#observed-proton-execution)
records two bounded GPU/provider configurations. Experimental's upgrade is not
required for this pinned SDK workload on the Machine; older-game upgrades and
per-shader execution tracing were not tested. The later selected temporal
coverage is assessed below; full native regression remains unverified here.

### Selected temporal regression

The four `proton-temporal-*` profiles extend the bounded proof. Use the same
prepared tools and command options as `proton`, with
`--vulkan-validation-manifest` required. `plan <profile>` lists the exact cases.
Each profile cross-builds the same Rust/C++ equipment, repeats public queries
and the single-frame control, then runs an input-only oracle, three independent
matrix processes and the existing offline analyser. Numerical budgets, sensitivity
predictions, repetitions and windows are unchanged from the Windows protocols.
Quality covers the texture/speed/motion factorial; long covers 256-frame histories;
reset uses the frozen cut/recovery criteria. The
[selected jitter protocol](proton-temporal-jitter-protocol.json) covers stationary
and moving phase 0 with all six dispatch controls and 128 frames per arm. Stationary
start phases 8/16/24 are excluded to bound temporary disk use; this is not the full
Windows jitter matrix. Existing group criteria remain unchanged.

The explicit `proton_temporal_diagnosis[_inputs]` selectors share the native
workload. Windows selectors continue to require DirectX diagnostics. Proton
records missing info-queue coverage as `UINT64_MAX`, accepts only zero or that
unavailable sentinel, and requires device-removal checks and separately observed
Vulkan core/synchronization activation without errors for every child. A quality
or sensitivity failure remains a failure; no provider-specific recalibration is
performed. Ordinary, Windows and release profiles are unchanged. Full diagnostics
remain temporary and are compacted after finalized attempts; use
`--retain-diagnostics` only for a concrete diagnosis. Prepared offline execution
and forced timeout cleanup use separate `proton-offline`/`proton-timeout` profiles.

The [Deck temporal record](../../docs/research/records/2026-10-13-exp-proton-temporal-regression.md)
and [Machine temporal record](../../docs/research/records/2026-10-14-exp-steam-machine-proton-temporal-regression.md)
both pass reset. Quality/long criteria fail for different predictions; Machine
jitter fails moving alignment, while a nonzero Deck launcher exit blocks its
full image verdict. The [comparison and exit-247 diagnosis](../../docs/research/upscaling-dispatch.md#selected-proton-temporal-regressions)
retain failed outcomes and the unresolved launcher cause. V07d quality acceptance
is deferred while that work is on hold.

Preparation is explicit. Required tools are Python 3.11+, Rust/rustup with the
Windows MSVC target, `cargo-xwin`, `clang-cl`, `lld-link`, `vulkaninfo`, a working
host Vulkan driver, `umu-run`, bubblewrap with user/PID namespace support and a
concrete installed Proton distribution. Offline execution also needs network
namespace support; `proton-timeout` additionally needs Linux pidfds.
`osslsigncode` is optional for Linux signature diagnostics. Cargo installs
`cargo-xwin`, or the [locked Nix development shell](nix/README.md) supplies it
together with the remaining tools and infrastructure-test dependencies. rustup supplies the target. LLVM and the runtime launcher require
separate installation; Nix is one option, not a profile dependency. GCC is useful
for ordinary host Rust builds but cannot replace Clang/LLD for this MSVC helper.

Tested bootstrap versions (not a general compatibility promise):

```sh
cargo +stable install cargo-xwin --version 0.23.1 --locked
rustup target add --toolchain stable x86_64-pc-windows-msvc
# Optional Nix provisioning of LLVM and the launcher:
nix profile install nixpkgs#llvmPackages.clang-unwrapped nixpkgs#llvmPackages.lld nixpkgs#umu-launcher nixpkgs#osslsigncode
# Explicit Microsoft SDK/CRT acquisition; arguments select component versions:
"$HOME/.cargo/bin/cargo-xwin" cache xwin --xwin-cache-dir target/proton-toolchain --xwin-arch x86_64 --xwin-sdk-version 10.0.22621 --xwin-crt-version 14.44.17.14
python tools/verification/run.py sdk install --root target/sdk-proton/v2.3.0
```

The cross tool itself requires newer Rust than the project's MSRV; install it
with stable. Its downloaded Microsoft headers/libraries retain Microsoft's
licence terms. AMD acquisition retains D017's pins and notices. No fetched
SDK/runtime payload is added to Cargo dependencies or distributed in this repo.

```sh
python tools/verification/run.py plan proton
python tools/verification/run.py run proton --native --allow-launcher-network \
  --toolchain stable --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --cargo-xwin "$HOME/.cargo/bin/cargo-xwin" \
  --umu /path/to/host-compatible/umu-run --proton /path/to/installed/Proton
```

On the tested SteamOS installation, Nix's `umu-launcher` FHS wrapper hid the host
Vulkan driver. Passing the installed `umu-launcher-unwrapped` executable through
`--umu` allowed use of SteamOS's RADV driver. The existing nixGL wrapper configured
OpenGL/EGL, not Vulkan; it was unnecessary for the passing configuration. On
NixOS a different driver setup may be needed; no automatic store-path discovery
or driver substitution is performed.

Each run creates a new prefix under `target/proton-prefixes`. Normal Proton launch
initializes its translator DLLs; `runinprefix` then captures console output with
explicit native D3D12/DXGI selection. Acceptance requires both child exit status
and markers from Proton's redirected log, followed by observed AMD module paths.
No global Cargo runner is configured. Failed attempts and prefixes are retained.
Observed AMD paths are resolved through the isolated prefix's DOS drive mappings
and must still identify the exact application stage; Proton 11 may use `S:` for
the host home directory instead of `Z:`. Available Proton AMD upgrade and
interface DLLs are included in the frozen external inputs. Inherited FSR4
upgrade/watermark and shader compiler controls are removed before ordinary runs.

The GPU selector is `upscaler::dx12::native_dispatch::proton_dispatch_readback`. It shares
Windows Gate A's inputs, dispatch, fence/readback, dimensions, finite/write and
gradient checks and teardown. Windows Gate A retains its required DirectX debug
layer. The separate Proton case reports unavailable debug-layer/info-queue
checks; `UINT64_MAX` means **unavailable**, never zero errors. D3D12 GPU-based
validation remains unavailable. Add `--vulkan-validation-manifest <manifest>`
to require Vulkan core and synchronization validation in both GPU children.
The runner freezes the manifest/library, removes inherited diagnostic controls,
sets explicit unfiltered settings, requires observed activation for every
reported instance and rejects validation errors. Warnings remain observations.
The layer is activated directly rather than through `VKD3D_CONFIG=vk_debug`:
newer VKD3D injects an intentional invalid-call self-test in that mode. No
validation message is excluded from evaluation; core and synchronization
activation remain required.
Full-process output preserves diagnostics across multiple instances; absent
activation fails. Vulkan GPU-assisted validation is not enabled. Without the
option, no Vulkan-validation result is claimed. `--vulkaninfo <executable>`
selects a tool compatible with the host driver independently of the cross tools.
Linux `osslsigncode` observations never
constitute a Windows Authenticode-policy pass; execution explicitly trusts the
D017 pinned bytes.

Cargo remains offline by default with a prepared xwin cache. Ordinary Proton
profiles require `--allow-launcher-network`: umu may contact upstream or acquire
its runtime even though updates are disabled. For prepared launcher execution
with networking disabled, use `proton-offline` below. Remote bootstrap and a
complete OS dependency closure are outside that local policy.
Every prepared Proton invocation additionally requires bubblewrap PID isolation
and `--die-with-parent`; offline calls also isolate networking. The guard verifies
a distinct PID namespace before exec. This contains detached sessions within the
invocation's namespace, whose init terminates on parent death. Failed containment
is never replaced by a global Wine kill. Forced-expiry verification uses the
separate `proton-timeout` profile below. Do not export verbose raw logs
without inspection: launcher debug modes can include inherited environment data.

### Prepared offline runtime

`runtime prepare` explicitly imports an already installed x86_64 Proton/sniper
stack into a new local directory. It downloads nothing, preserves third-party
notices, and never overwrites an existing destination. Preparation is bounded
to 4 GB, 100,000 inventory entries and a 300-second checked import deadline.
Partial failed imports remain local for diagnosis. Supply acquisition provenance;
the resulting checksums identify trusted local inputs, not upstream authenticity
or Windows signature policy. Network acquisition of arbitrary versions is outside
this policy. Tools, Microsoft cache, Rust dependencies and AMD SDK remain separately
prepared inputs. No bundle or third-party binary is published with this repository.

```sh
python3 tools/verification/run.py runtime prepare \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 10.0" \
  --runtime "$HOME/.local/share/umu/steamrt3" --umu "$VERIFICATION_UMU" \
  --launcher-root /path/to/the/installed/umu/package \
  --output target/proton-runtime/local-stack \
  --provenance 'Installed Steam Proton and prepared umu sniper; record installation provenance'
python3 tools/verification/run.py runtime verify \
  --manifest target/proton-runtime/local-stack/runtime.json --sha256 <printed-manifest-sha256>
python3 tools/verification/run.py run proton-offline --native --toolchain stable \
  --runtime-manifest target/proton-runtime/local-stack/runtime.json \
  --runtime-sha256 <printed-manifest-sha256> \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

The manifest records complete regular-file bytes, modes and literal symlink
targets for the copied Proton and steamrt3 distributions and declared umu package,
plus the prepared umu-shim. Verification rejects missing, changed or additional
distribution nodes. Only steamrt3's generated `var/` cache is excluded; host
drivers, system libraries and launcher dependencies outside its package are
environment inputs, not a fully pinned OS closure. The installed launcher remains
at its declared path; imports are not portable self-contained distributions.

`proton-offline` verifies integrity before launcher execution and again after GPU
dispatch. It rejects `--online` and `--allow-launcher-network`. Bubblewrap is a
required Linux tool for all prepared Proton profiles. Offline umu invocations
unshare user, PID and network namespaces, verify distinct PID/network namespaces
with only loopback networking, and require IPv4/IPv6 probes to return ENETUNREACH
before exec. Missing bubblewrap
or unavailable namespace isolation fails rather than falling back to networked
execution. Proxy controls are removed; runtime updates/retries are disabled.
umu can still attempt requests, but those launches have no external IP route.
These checks do not establish detached-service timeout cleanup (V09).
The [two Deck confirmations](../../docs/research/records/2026-10-14-exp-proton-offline-runtime.md)
verify this local-import/offline policy, including missing-input rejection and
unchanged distribution bytes. They do not establish remote acquisition or an
offline fresh-OS tool bootstrap.

### Forced timeout cleanup

Use `run.py run proton-timeout` with the same prepared-runtime arguments as
`proton-offline`. Its C++ Windows fixture creates a parent and detached child,
each with a live heartbeat, cooperative stop file and 120-second hard lifetime.
A second fresh prefix runs an independent sentinel. The fixture synchronizes
readiness before the real Runner's one-second watchdog wait; startup is separately
bounded and included in recorded elapsed time. PIDfds and prefix ownership
inventories must show every observed descendant exited within five seconds,
with stopped target heartbeats and continuing sentinel activity/identity.
It then reruns the existing query and GPU cases on the same verification prefix.
Any timeout accepted here is a required diagnostic stimulus, never acceptance of
an ordinary native timeout. Cooperative/pidfd emergency fixture cleanup runs
independently on failure and cannot satisfy the watchdog verdict. Ownership is
limited to exclusively created verification prefixes; no global wineserver kill
or unrelated-application termination is permitted. Linux pidfds are required.
This tests process termination, not GPU/kernel hangs or provider termination.
The [Deck timeout record](../../docs/research/records/2026-10-14-exp-proton-timeout-cleanup.md)
preserves the failed group-only baseline, two successful PID-contained
confirmations and the normal-profile check. The unrelated sentinel survives,
target descendants exit before cooperative cleanup, and subsequent GPU controls
pass on the same prefix. The original nonzero jitter launch remains unresolved.

## SDK acquisition

Acquisition is a separate explicit action under [D017](../../docs/adr/D017.md).
It adds no verification profile, library runtime discovery or Cargo download.

```sh
python tools/verification/run.py sdk install
python tools/verification/run.py sdk verify
python tools/verification/run.py sdk verify --strict
python tools/verification/run.py sdk install --archive /path/to/FidelityFX-Samples-v2.3.0-prebuilt.zip
# Fully offline installation, including headers and notices:
python tools/verification/run.py sdk install --archive /path/to/FidelityFX-Samples-v2.3.0-prebuilt.zip --source-root /path/to/upstream-tree
```

`--root` defaults to `external/FidelityFX-SDK/v2.3.0/`. Continue passing its
`Kits/FidelityFX` child as `--sdk-root` to native profiles; installation does not
select a runtime or imply that private DLL colocation suffices for application
deployment ([D007](../../docs/adr/D007.md)).

The named ZIP contains sample deployments, **no API headers or licence texts**.
Only the loader and upscaler DLLs from the FSR sample are extracted, after checking
the entire ZIP and both DLLs against fixed hashes. Four headers and the complete
`docs/license.md` / `3rdpartynotice.md` come from commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Header pins remain in
`tools/native-headers.json`, DLL pins in `native.py`, and notice pins in `sdk.py`.
Headers must match raw hashes as well as the existing normalized ABI pins.

Downloads use bounded streaming, restricted GitHub HTTPS redirect origins,
30-second socket timeouts and a 180-second transfer deadline. Limits are 200 MB
for the ZIP, 1 MB per source file, 2,048 ZIP entries, 64 MB per expanded entry,
512 MB total declared expansion and a 200:1 maximum compression ratio. Every ZIP
entry is inspected for unsafe paths, reserved Windows names, links, unsupported
types, encryption, duplicates and case/file-directory collisions. Unneeded safe
entries are ignored; only two exact DLL paths are read.
ZIP member and pinned inventory paths use `PurePosixPath` on every host;
local filesystem access uses `Path`. Non-canonical archive names are rejected
before extraction rather than silently normalized.

Installation checks the complete staged tree before same-filesystem rename.
An exclusive sibling lock serializes this tool. Existing content is never
repaired or replaced. Links, reparse points and hardlinked input/files are
rejected. Parents must be caller-controlled; a process allowed to mutate them
during publication is outside this tool's guarantees. On macOS, use real
`/private/tmp` paths rather than the `/tmp` symlink. Ordinary failures and
interruptions clean up staging and the lock. A hard kill may leave hidden staging
and `.v2.3.0.sdk-lock`: inspect and remove these explicitly only after confirming
that no installation is running. No partially checked final tree is published.

`sdk verify` rehashes **all eight files** against project pins and checks inventory
and provenance without invoking network/trust tools. Provenance is descriptive,
not an attestation; historical archive/signature fields cannot replace fresh
checks or establish current trust. The ZIP is discarded after installation;
the checked installation is the reusable cache. Corruption requires explicit
relocation/replacement by the user, or installation under a new `--root`.

New installations write `license_notices` in `provenance.json`. Verification
also accepts the established `licence_notices` key without rewriting existing
records. If both keys are present, their values must agree and match the pinned
notice hashes. Missing, altered or conflicting inventories are rejected; file
rehashing and trust checks retain their existing requirements.

On Windows, install inspects Authenticode when PowerShell is available. Missing
tooling is `not_run`; an executed failed check blocks installation. `--strict` on
install or verify requires SignTool `verify /pa /all /v /tw`, embedded status
`Valid`, AMD's observed exact publisher name, a timestamp and the Windows
code-signing/TSA chain builders with online revocation and no ignored errors.
Missing checks, unknown revocation and warnings prevent acceptance. Valid cached
revocation data may satisfy OS policy; a new live OCSP response is not promised.
Additional chain checks use the current clock and conservatively reject expired
signer/TSA certificates even when Authenticode accepts a historical timestamp.
Strict mode may need network access and has no non-Windows trust implementation.
Signature mathematics, publisher trust, timestamp/chain validity and revocation
are distinct from file integrity and runtime/GPU functionality.

The four headers and two DLLs are in AMD's MIT exception list; preserve the two
complete notices. No sample dependency is installed. The full SDK has mixed
terms; a third-party inventory is not a licence grant. Keep SDK files and archive
caches out of Git, Cargo packages, public CI caches and release artefacts.
See the [dated verification](../../docs/release/2026-10-12-sdk-acquisition-verification.md)
for measured contents, hashes, commands and unperformed Windows checks.

## Native profiles

The separate opt-in [BGRA/sRGB view experiment](bgra-srgb-protocol.md) runs
`python tools/verification/bgra_srgb.py --native --adapter-index <index> --sdk-root <Kits/FidelityFX>`.
It reuses the shared runner and Windows environment acquisition, but loads no
AMD runtime and adds no CI/release requirement. Independent shader/readback
checks distinguish channel order, sRGB reads and UNORM stores; an unsupported
or rejected output route stays observational. See the
[source audit](../../docs/research/records/2026-10-24-src-bgra-srgb-audit.md) and
[bounded execution record](../../docs/research/records/2026-10-24-exp-bgra-srgb-views.md).

The follow-on [direct SDK experiment](bgra-srgb-fsr-protocol.md) uses
`bgra_srgb_fsr.py` for BGRA/sRGB upscaling and generator calls with the existing
pinned-header C++ reference. Its acceptance covers native mapping, resource
integrity, writes, diagnostics, fences and cleanup; it imposes no image or
algorithm comparison. `bgra_srgb_fsr_check.py --run <attempt>` can review those
recorded integration observations without new GPU calls. The
[execution record](../../docs/research/records/2026-10-24-exp-bgra-srgb-sdk-integration.md)
preserves the initial out-of-scope comparison failure and the explicit scope
correction. This adds no public wrapper format or release requirement.

The [public BGRA/sRGB protocol](bgra-srgb-public-protocol.md) defines opt-in
`native-bgra-srgb` for D038: both public owners, separate main/generator roles,
UAV-capable inputs, device/view checks, resource integrity, fenced SDK recording
and cleanup. It imposes no image or algorithm comparison.

The [2026-10-15 Windows batch](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
passes all 27 source native-upscaler cases, 112 CI cases and 95 MSRV cases on one
recorded GPU/driver/provider configuration. This includes all five feature cases
below. Source and artefact profiles select them, but native execution through
the final release archives remains a separate release-closure requirement.

The `utility-queries` case uses a real device/context to exercise all five SDK
quality modes, queried render sizes, phase counts and three offsets for both
pre-create/live routes and both ownership families. It explicitly destroys the
contexts without GPU submission and records provider values without requiring
provider equality, a fixed formula or image-quality criteria. Its scope is
`native-query`; [Windows native execution](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
passes for both routes and families. Source and artefact native profiles select this case
and the previously listed `pre-exposure`/`depth-motion` cases; an infrastructure
regression checks their inclusion and prerequisites.

The `pre-exposure` native case reuses the single-frame DX12 equipment. Three
fresh contexts receive matching RGB factors `0.25`, `1.0` and `4.0`; required
checks cover successful recording/submission/fencing, finite/written output,
device/debug status and teardown. This is integration verification with no
image-quality comparison. [Windows native execution](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
passes all three factors.

The `automatic-exposure` native case dispatches three moving/jittered frames with
SDK exposure enabled and no exposure resource supplied to FSR. It uses actual
provider requirements/memory queries, fences, finite/written output, device/debug
checks and explicit teardown. The shared test helper still owns an unused exposure
allocation. It tests integration, with no image-quality comparison or allocation
saving claim. [Windows native execution](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
passes the three-frame local-family case. Source/artefact profiles and their registration
regression include the case.

The `active-render` native case uses fixed 320×180 input allocations and 640×360
output, with active render sizes 320×180, 256×144 and 288×162 on one context.
Four profiles exercise render/display motion in UV/pixel units. It verifies
submission/fencing, finite output, complete output sentinel replacement, byte-exact
input preservation including inactive guards, device/debug status and explicit
teardown. Input guards prove read-only preservation, not that inactive texels were
never sampled. No image-quality or performance criterion is imposed. Source and
artefact profiles select the case; [Windows native execution](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
passes all four profiles.

The `depth-motion` native case reuses the three-frame smoke equipment for eight
representative profiles covering standard/reversed finite/infinite depth,
render/display motion, UV/pixel units and excluded/included projection jitter.
Uploads match the selected extents and content conventions. Memory estimation,
native recording, fencing, finite/written output, device/debug checks and explicit
teardown are required; no quality comparison is made. Controlled fixtures cover
all 16 context combinations with both units and preflight failures.
[Windows fixtures and native execution](../../docs/release/2026-10-15-windows-m11-m14-verification.md)
pass, with eight representative GPU profiles. Arbitrary motion scales and coordinate
conversions are deferred.

```sh
python tools/verification/run.py run abi --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run native-upscaler --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run release --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

GPU execution requires `--native` and a local SDK root containing pinned headers
and signed `signedbin` DLLs. Running a profile downloads no AMD binaries. It checks
hashes/signatures and records helper/test executable hashes and adapter, LUID,
GPU/driver/provider observations. An explicit adapter index never falls back;
otherwise the first suitable hardware adapter is selected. CI supplies no GPU evidence.

M9's direct-control image comparisons remain diagnostic. They do not establish
general image quality, longer temporal stability, provider termination or an AMD
threading guarantee. The separate bounded temporal/image baseline is recorded
in [completed V01](../../docs/ROADMAP_HISTORY.md#v01--temporalimage-baseline--completed-2026-10-12).

Each experiment is an explicit `experiment-<suffix>` profile, excluded from ordinary gates:

| Suffix | Retained protocol |
|---|---|
| `allocation` | Successful three-callback control; indices 1–3 observed separately; index 4 unreached control. A different count rejects this bounded sweep |
| `dimensions` | C0 control and eleven raw dimension observations |
| `input-lifetime` | Lifecycle and clean retained returned-error controls before protection/cleanup observations |
| `provider` | Four independent app/runtime/current-directory layouts, positive controls and raw negative observations |
| `shared-runtime` | Sequential raw runtime/context lifetime acceptance |
| `gbv` | Required no-provider DX12 control, then watchdog-bounded dispatch observation |
| `recording` | Same-context 1/2/5/8 and sibling 2/5 recording-ahead children |
| `sharpening` | Three original D014 image-effect gates; independent original metrics even when gates fail |
| `transfer` | Six private same-thread/thread-transfer baselines |
| `transfer-cleanup` | Eleven prerequisites and thirty control/transfer children; 96 RGBA16F payloads, independent fence/destroy/application-residency audit and diagnostic comparisons |
| `temporal-calibration` | Frozen two-scene protocol; 21 input-only DX12 frames validated before AMD calls; three independent 301-frame children with temporary full inputs/RGBA16F and independent RGB/age/reset/repeat diagnostics. Compact outcomes retained; structural completion only, no image-quality gate |
| `temporal-diagnosis` | Fixed affine/flat slow/flat fast/textured scenes; 95 input-only frames, two 1216-frame matrices; reversed/zero motion, wrong jitter and per-frame reset controls; independent point/area references and synthetic metric counterexamples. Diagnostic rankings only, no acceptance threshold |
| `temporal-quality` | Known-defect evaluator tests and severity sweeps; full texture/speed/motion factorial plus a reset control; 45 input-only frames and three independently ordered 576-frame children. Prospective, calibration-derived textured-slow regression and sensitivity checks; other cells remain diagnostic |
| `temporal-long` | Continuous 256-frame textured-slow histories, correct/reversed motion; 34 input-only frames and three independently ordered 512-frame children. Seven separate 32-frame acceptance windows under unchanged quality limits; per-frame trajectories retained |
| `temporal-reset-explore` | Textured camera cut with history/reset, history/no-reset and fresh-context arms; 25 input-only frames and one 320-frame child. Diagnostic selection data only; no native image-quality acceptance |
| `temporal-reset` | Same scene, three independently ordered 320-frame confirmations under separately frozen exploration-derived limits, fixed reset/fresh recovery and retained-history sensitivity checks |
| `temporal-jitter-explore` | Independently validated spatial phase estimator; four stationary start phases and one moving phase, each with correct, X/Y/both-negated, one-frame-delayed and dispatch-only zero jitter. 990 input-only frames and one 3,840-frame selection block |
| `temporal-jitter` | Same 30 arms, three newly ordered 3,840-frame process blocks under frozen alignment/local quality limits and separate per-window sensitivity predictions |

The cleanup matrix executes 516 fenced frames and retains 96 selected payloads;
those are different counts. Its native residency oracle uses the same
`gpu_helper.dll` basename as the build/load paths and must observe it live before
final release.

`experiment-temporal-calibration` additionally requires an existing NumPy-equipped
Python interpreter (no automatic installation). Other profiles retain their
standard-library coordinator requirements. Its [frozen protocol](temporal-calibration-protocol.json)
owns camera, jitter, quantization, regions, controls, runtime bounds and failure
conditions. [temporal_calibration.py](temporal_calibration.py) independently checks every retained input byte and
decodes every RGBA16F output; native image metrics are not its oracle. Run with
the native arguments above. Each frame's raw bytes are packaged in deterministic
gzip files with decoded byte counts and SHA256 identities in `payloads.json`.
The independent audit consumes all full images before retention is reduced.
After a finalized run, bulk calibration files, copied source snapshots and
verbose frame analyses are removed by default, including on failed runs; failure
statuses, logs, frame/payload identities and available compact results remain.
The audit also writes `diagnostic-summary.json` with the predeclared pooled
point/area/temporal/leakage/control measurements. The generator, frozen protocol,
jitter, input identity and selected contact sheet support a new GPU run without
requiring repeated source-tree copies.

For a specific local diagnosis, add `--retain-diagnostics` to the calibration
`run` command. Remove those files when the diagnosis is complete. Calibration
exports always exclude bulk files, even from a full local diagnostic attempt.
`retention.json` / `export.json` disclose that full offline image reanalysis is
unavailable after compaction. GPU outputs can change with driver/provider/hardware;
hashes cannot reconstruct them. For independent reanalysis of an intact local
diagnostic attempt:

```sh
python tools/verification/temporal_calibration.py audit --run <local-diagnostic-attempt> --output target/analysis/<new-name>
```

Attempts captured before the responsibility-based rename retain their original
readback namespace. Add `--namespace experiment.v01-calibration` when analysing
the historical attempts linked from the calibration record.

The profile belongs to calibration only. It is excluded from `ci`, `release`
and `native-upscaler`; its data cannot retrospectively verify image criteria.
The [calibration record](../../docs/research/records/2026-10-11-exp-v01-verification-calibration.md)
retains compact results, failed preparation/audit outcomes, selected examples and
candidate criteria. It records mixed control rankings and empty regions rather
than turning them into image-quality passes.

`experiment-temporal-diagnosis` investigates those contrary rankings with its
[frozen protocol](temporal-diagnosis-protocol.json) and
[independent analyser](temporal_diagnosis.py). It requires an existing NumPy
interpreter and Pillow for a small labelled crop sheet. Its reference integrals,
input controls and deliberately bad synthetic outputs have separate analytic
checks. The new native input export and Rust probe reuse the existing fenced
equipment without changing the original calibration matrix. Run with the native
arguments above; `--retain-diagnostics` keeps temporary local payloads for a
specific diagnosis. Compact retention/export also applies to this profile. It
is excluded from routine, release and `native-upscaler` profiles.

The [metric diagnosis record](../../docs/research/records/2026-10-11-exp-temporal-quality-metric-diagnosis.md)
retains all control rankings, including a counterexample to the proposed
jitter/silhouette rule and the spatial/temporal reset trade-off. Structural
completion does not accept a quality criterion.

`experiment-temporal-quality` validates the evaluator itself with an ideal
analytic reconstruction and twelve deliberately defective outputs: object
deletion, constant foreground, box blur, three levels each of texture loss,
ghost contamination and alternating foreground bias. The responsible property
must reject each defect; error must increase with severity. A conjunction of
foreground, background, silhouette, texture, temporal and uncovered-background
checks prevents a perfect local score from hiding a failed property. Empty or
nonfinite measurements fail rather than becoming zero-error passes. The tests
also detect a disabled evaluator that returns zero for every error.

Its [frozen protocol](temporal-quality-protocol.json) crosses texture, speed and
motion direction in all eight combinations, adds a history-reset ablation, and
executes three independent process blocks with seeded, shuffled arm orders.
Every input byte and output is checked with the existing independent oracle.
The [analyser](temporal_quality.py) confirms five calibration-derived budgets
and three sensitivity predictions on **textured-slow only** in every block.
The 25% regression allowance and 10% minimum sensitivity are declared local
policies, not perceptual thresholds, AMD guarantees or confidence intervals.
Other cells, favourable wrong-input rankings and factorial interaction contrasts
are reported as diagnostics. Frames are correlated; their large count must not
be presented as independent statistical replication. No overall weighted
quality score is produced. The profile is excluded from ordinary/release gates;
these quality confirmations alone do not cover the longer-history, cut/reset
and jitter extensions described below.

Use an existing NumPy-equipped Python interpreter (no additional installation):

```sh
python tools/verification/temporal_quality.py controls --output target/quality-controls.json
python tools/verification/run.py run experiment-temporal-quality --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The first command performs CPU evaluator validation only. The second performs
the 45-frame input-only check, existing native prerequisites and 1,728 new GPU
frames. Failed quality criteria retain their full compact outcome before the
command returns nonzero. Default compaction and `--retain-diagnostics` follow
the same boundaries as the diagnostic profile. The
[follow-up record](../../docs/research/records/2026-10-11-exp-temporal-quality-validation.md)
documents source methods, protocol selection, confirmation outcomes and limits.

`experiment-temporal-long` extends the confirmed textured-slow scene to 256
frames (4.267 seconds at 60 Hz), using one context per arm and reset only at
frame 0. Motion continues through eight repetitions of the 32-phase jitter
sequence; the foreground remains fully inside the measured viewport. After
32 warmup frames, every one of seven disjoint 32-frame windows must pass the
five existing native budgets and both reversed-motion sensitivity checks in
each of three process blocks. Whole-run averages cannot compensate a late
failure. Texture error remains diagnostic. Its
[protocol](temporal-long-protocol.json) and [analyser](temporal_long.py) retain
window results and per-frame trajectories, including failed criteria. Existing
known-defect evaluator controls run before GPU execution. Resources remain live
until context destruction; end-of-arm native memory observations record the
bounded retention cost. This profile requires existing NumPy and shares compact
retention. It adds no ordinary/release gate, cuts or reset/jitter acceptance.

```sh
python tools/verification/run.py run experiment-temporal-long --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The [duration record](../../docs/research/records/2026-10-11-exp-temporal-long-sequence.md)
documents all seven windows, memory observations, complete input/output checks
and the scope limits of that duration extension.

Use the native arguments above. A timeout/abort can be `observed` in a diagnostic
profile; it cannot be `passed` native acceptance. Historical criteria/results and
original scripts remain in the [indexed archive](archive/README.md).

## Cut/reset exploration and confirmation

The reset profiles use a textured camera cut at absolute frame 64, retaining
the 32-phase jitter identity and exact matching post-cut inputs across three
arms. History arms execute 0..127, fresh context 64..127. Only the reset arm
requests reset at the cut; fresh requests reset on its first frame. All native
resources retain the existing fence/context lifetime policy.

The [protocol](temporal-reset-protocol.json) defines separate post-cut windows
0..7, 8..31 and 32..63. The [analyser](temporal_reset.py) checks analytic quality,
old screen-footprint contamination, and per-frame paired differences. Known
synthetic contamination must be detected before GPU execution. Reset/fresh
RGBA maximum must stay within .000001 from the cut onwards; no-reset/fresh
interior MAE over the first eight frames must exceed reset/fresh by .0001.
This tests history clearing; it does not require no-reset to lose every quality
metric or imply universal perceptual benefit. Identical controls are insensitive
and cannot pass the causal prediction.

```sh
python tools/verification/run.py run experiment-temporal-reset-explore --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/temporal_reset.py select --run <exploration-attempt> --output target/reset-candidate-criteria.json
python tools/verification/run.py run experiment-temporal-reset --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Selection requires a verified successful independent exploration and refuses to
overwrite an existing criteria file. It selects five limits from the largest
fresh-context window MAE times 1.25 plus .000001; the fixed recovery/sensitivity
predictions must already be supported. Selected limits must still reject known
missing-object, constant-image, blur, flicker and old-history defects. Three
fresh confirmations then require
every reset/fresh quality window, every recovery frame and the sensitivity
criterion to pass separately. Criteria are frozen as an additional prerequisite
before native dispatch. Failed outcomes are retained, with no post-result edits.
These profiles require existing NumPy, share compact retention and remain
excluded from ordinary/release gates.

The repository contains [confirmed criteria](temporal-reset-criteria.json).
The selection command above writes a new candidate under ignored `target/`;
the confirmation profile uses the repository criteria. Replacing those criteria
is a deliberate new calibration, followed by new confirmation runs.

The [cut/reset record](../../docs/research/records/2026-10-11-exp-temporal-reset-recovery.md)
retains exploration and confirmation separately, including mixed quality
rankings, immediate recovery, retained-history sensitivity and reproduction limits.

### Jitter sign and temporal phase

The [exploration protocol](temporal-jitter-protocol.json) and
[confirmation protocol](temporal-jitter-confirm-protocol.json) use the existing
analytic sinusoidal texture at zero velocity and at (.375,.25) render pixels
per frame. Each arm retains one context for 128 frames, resetting only at zero.
The stationary scene starts the same 32-phase Halton sequence at 0, 8, 16 and
24; the moving cross-check starts at zero. Start phases shift image sampling
and correct dispatch together. X-only, Y-only and both sign reversals,
one-frame-delayed dispatch including wrap, and **dispatch-only** zero jitter
leave all generated image/depth/motion/exposure bytes unchanged.

The [position analyser](temporal_jitter.py) fits independent DC/sine/cosine
coefficients to the known texture in each colour channel. Red/green identify
X/Y displacement; the oblique blue channel reports consistency. It records
brightness bias, amplitude and residual separately and does not realign images
before measuring quality. Spatial sampling uses every fourth output pixel,
with 8-render-pixel viewport/silhouette margins. The bounded phase domain is
one render pixel per axis, allowing .002 pixel numerical error at its edge;
amplitude ratios outside .25..2 are unidentifiable. Known positive/negative
translations, half-precision readbacks, independent brightness/contrast changes,
isolated X/Y and unequal-axis translations, constant images, nonfinite samples
and out-of-range controls validate the estimator before GPU work. A deliberately
coupled-axis estimator that passes anti-diagonal shifts must fail the extended
controls.
Nonfinite/missing regions fail. The existing known-defect quality controls also
run before dispatch.

The first 32 frames are warmup; windows 32..63, 64..95 and 96..127 must each pass
independently. Exploration selects group-specific X/Y/radial alignment RMS and
five local quality limits as 1.25 times the largest correct-arm window plus
.000001. Selection refuses alignment limits at or above the known .125-pixel
translation defect, or quality limits accepting the analytic damaged images.
Each wrong control must exceed its paired correct alignment RMS by 10% plus
.001 pixel in every window (X for X reversal, Y for Y reversal, radial for the
other controls). Other metric rankings remain diagnostic. These allowances
are local regression bounds, not perceptual thresholds or confidence intervals.

```sh
python tools/verification/temporal_jitter.py controls --output target/jitter-position-controls.json
python tools/verification/run.py run experiment-temporal-jitter-explore --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/temporal_jitter.py select --run <completed-exploration> --output target/jitter-candidate-criteria.json
python tools/verification/run.py run experiment-temporal-jitter --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Confirmation reads [the retained criteria](temporal-jitter-criteria.json), frozen from an independent
completed exploration before three fresh, independently ordered process blocks.
The selection command writes an exclusive candidate, never overwriting existing
criteria. Keep known failures if the pilot cannot support the fixed prediction.
Every raw input is still compared byte for byte with the analytic oracle. The
jitter profile uses bounded reuse of identical analytic inputs, and reuses
compressed bytes only after matching raw identity and full decompression
comparison; corruption fails without removing the raw source. Other profiles
keep their existing packaging behaviour. Offline analysis has a 1,800-second
watchdog for three blocks; the native child remains bounded to 360 seconds.
The per-frame trajectories and raw readbacks remain temporary under `target/`;
the compact summary retains window outcomes, paired-input checks and repeat
identities. Four start phases and repeats on one GPU do not establish general
provider/hardware support. Ordinary/release profiles and public APIs are unchanged.

The [jitter confirmation record](../../docs/research/records/2026-10-12-exp-temporal-jitter-phase.md)
retains the initial failed input audit, selection and three passing confirmation
blocks. All 45 declared windows and 225 wrong-dispatch predictions pass; other
quality metrics still have mixed rankings. It completes the selected jitter
extension alongside the earlier quality, continuous-history and cut/reset records.

## Evidence and reanalysis

Every attempt gets an exclusive timestamp/UUID directory under `target/verification`.
`--output-root` changes its parent; it cannot reopen an existing attempt. Records
retain source/configuration/environment identity, actual tools, required cases,
commands, selectors, timing, execution and evaluation separately. Supplied inputs
and sources are rehashed before finalization. Expected compiler failures require
the declared error code and nonzero exit. Missing equipment/cases/markers/readbacks
prevent acceptance. Failures retain their outcomes and logs. Calibration payloads
are temporary by default; other bounded native protocols retain their declared
small readbacks.

New JSON records and exported JSON/TXT/CSV are written as UTF-8/LF on every host,
before their sizes and hashes are calculated. Raw process logs remain verbatim
in the original attempt. Exports carry a hashed `.gitattributes` with `* -text`,
preserving even verbatim payloads across Git check-in/checkout. When selecting
compact evidence, retain that rule or an equivalent scoped exception; follow
the [repository text and byte-identity rules](../../CONTRIBUTING.md#text-files-and-byte-identity).
Historical inventories remain strict byte checks. The five repaired temporal
evidence directories have scoped exceptions and retain their original hashes;
see the [line-ending verification](../../docs/release/2026-10-15-line-ending-verification.md).

`artifacts.json` hashes every retained file. Export checks inventory completeness
and identity first and refuses an existing/overlapping destination. It normalizes
repository/home/run paths and pointer-labelled addresses while retaining native
codes, provider IDs, nulls and hashes. Full environment dumps, Cargo metadata
author/contact fields, AMD binaries and build products stay local. Calibration
exports omit full images, source snapshots and verbose analyses; copied case
results keep payload totals and an inventory digest rather than duplicating
entire manifests. Other native profiles retain their declared small readbacks
unchanged; their hashes alone cannot reconstruct images.

```sh
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py analyze --run target/verification/<attempt> --namespace native --output target/analysis/<new-name>
python tools/verification/run.py analyze --run target/verification/<sharpening-attempt> --original-sharpening --output target/analysis/<new-name>
python tools/verification/run.py export --run target/verification/<attempt> --output target/evidence/<new-name>
```

Omit `--namespace` to characterize available red readbacks, including failures.
Reanalysis never changes original acceptance. Export into ignored `target/`, then
select compact results for a permanent record; do not commit full exports.
`python tools/verification/repository_size.py` checks the 50 MB total and 1 MB
individual-file budgets, including new non-ignored files locally, and rejects
raw payloads/source snapshots under `docs/`. CI exports finalized
failed/passed attempts and retains sanitized evidence for 14 days; abrupt job
termination may prevent finalization/export.

On release profiles, `--wrapper-archive <file> --sys-archive <file>` selects existing
bytes instead of generating packages. Versions come from manifests. Every
non-generated archive file must match the checkout. Extraction rejects traversal,
links, Windows drive paths and duplicates before writing. A fresh Cargo home
contains checksum-verified local registry inputs; consumer source IDs and lockfile
checksums must identify the selected artefacts. Extracted package manifests/bytes
are never edited. Publication simulation and published-registry observation remain
separate operations; these profiles upload nothing.

### Exact-package native checks

`release-coverage` runs the focused package checkpoint on Windows, including
registry-only full examples with `dx12`, `dx12,movable`, `dx12,utils` and
`dx12,movable,utils` on stable and Rust 1.88.0. The eight named workflow calls
also compile separately on each compiler. The public callback fixture builds
from the extracted wrapper, while the generated CB exchange fixture imports
`src/api.rs` from the extracted raw package. Both independent MSVC exchanges
must pass. The final audit rejects changed archives, changed extracted bytes,
added files and symlinks. Existing heap-invocation/provider limits remain.

```sh
python tools/verification/run.py plan release-coverage
python tools/verification/run.py run release-coverage --require-passed --native \
  --sdk-root /path/to/Kits/FidelityFX --adapter-index 0 \
  --wrapper-archive /path/to/fsr-sdk-0.3.0.crate \
  --sys-archive /path/to/fsr-sdk-sys-0.1.2.crate
```

`release` includes the same callback/CB and example coverage in its broader
qualification. The focused profile does not replace its full source, extracted
feature matrix and consumer-contract checks. The
[2026-10-29 verification](../../docs/release/2026-10-29-package-coverage-verification.md)
records the selected development archives; final frozen packages remain separate.

The following standalone profiles also support `--artifacts`:

- `native-upscaler`, `native-callbacks`, `native-motion-scale`, `native-formats`,
  `native-depth-formats`, `native-bgra-srgb`, `native-variable-output`, and
  `native-resource-formats`;
- `proton`, `proton-offline`, and `proton-timeout` on their required Linux hosts.

The public generator probes `experiment-public-reactive-mask` and
`experiment-selected-reactive-mask` also accept the pair. They retain their
original observed/rejected/write criteria: a known provider no-write outcome
must remain visible rather than being converted to a passing release result.
This adds package provenance, without reopening image-quality research.

With `--artifacts`, omitting both archive paths creates fresh packages. Supplying
both paths implies archive mode, even without that flag. A partial pair or an
archive request on an unsupported profile fails rather than falling back to
checkout builds. Ordinary source mode remains the default. Temporal research
profiles do not support this flag.

```sh
python tools/verification/run.py plan native-variable-output --artifacts
python tools/verification/run.py run native-variable-output --artifacts --require-passed --native \
  --sdk-root /path/to/Kits/FidelityFX --adapter-index 0 \
  --wrapper-archive /path/to/fsr-sdk-0.3.0.crate \
  --sys-archive /path/to/fsr-sdk-sys-0.1.2.crate
```

For the selected feature-native matrix, repeat each applicable profile against
the same pair and record provider-specific outcomes. Windows verification
equipment remains separately identified checkout tooling; the archive audit
requires its packaged source inputs to match the checkout. Proton builds its
GPU helper directly from the extracted pair as well as using the isolated
registry for the Rust cross-build. Append `--artifacts` and the pair options to
the documented Proton invocation, retaining its runtime/launcher/diagnostic
options. Compilation alone establishes no Proton or hardware execution claim.

## Maintenance

For changes to Python verification tests or their producers, follow the focused
[test-authoring guide](tests/README.md), including path-contract fixtures and the
complete infrastructure suite. On infrastructure failure, the coordinator prints
the last 16,000 stderr characters after path sanitization; full case output is
retained separately and the failure status is preserved.
The [Windows path-assertion correction](../../docs/release/2026-10-29-ci-infrastructure-verification.md)
records the recurring fixture issue, its reproduction and complete-suite checks.

Keep unit/private/public tests in their current layers and workloads/oracles
effect-specific. Share process management, staging, provenance and storage; there
is no universal effect interface. Do not import executable machinery from dated
records or frozen scripts. [windows.ps1](windows.ps1) only acquires environments
and inspects signatures.

```sh
python -m unittest discover -s tools/verification/tests -v
python tools/verification/archive/check.py
python tools/verification/archive/check.py --require-powershell
```

The last command requires PowerShell 7. The archive checker can write separate
integrity and syntax outcomes with `--report <new-json-file>`; it refuses an
existing report file. The coordinator retains this report and its subcheck
outcomes in the `archive.integrity` case, including failed required parsing and
the reason for unavailable optional parsing. Archived scripts are never executed.

Infrastructure tests cover wrong/missing diagnostics, equipment, watchdogs,
observed timeouts, strict CLI acceptance without relabelling observations,
missing markers/results, source drift, tampering, exclusive
exports, unsafe archive paths and independent half decoding. Descendant tests
establish readiness before the watchdog interval and wait for process exit;
delayed startup, live/parent-only termination controls and assertion-failure
cleanup are covered. Archive tests exercise unavailable/failed parsers and
portable integrity failures using isolated fixtures.

## Public callback verification

`native-callbacks --native --sdk-root <Kits/FidelityFX> --adapter-index 0`
checks production raw callback layouts/signatures against independent MSVC/AMD
and Microsoft declarations, and exchanges the production CB aggregate in both
directions on current Rust and exact 1.88.0, including corrupted-field controls.
The public native case uses both owner families and explicitly selected 3.1.5
and 4.1.1 providers: 12 contexts, 20 submitted/fenced frames, debug-view mapping,
modern global and deprecated null/live configuration, a 0.5 ms warning stimulus
for actual UTF-16 message delivery, allocator replacement and
omission, final allocation balance and DX12 diagnostics. Targets are external,
process-lived C++ equipment; the wrapper owns no target state. Controlled heap
allocation is labelled separately from actual SDK invocation. No output equality
or image-quality oracle is used. The existing CI/MSRV controlled tests cover
paired-group forwarding, stable host-table addresses and exceptional retention.
