# Development

Use [CONTRIBUTING.md](../CONTRIBUTING.md) and [code heuristics](CODE_HEURISTICS.md)
when changing contracts. This guide owns current setup and verification commands;
[verification selection](verification-selection.md) explains targeted development
checks versus final qualification. The [coordinator guide](../tools/verification/README.md)
owns profile inventories, evidence and platform boundaries.

## Release baseline

The published baseline is `fsr-sdk 0.2.0` / `fsr-sdk-sys 0.1.1`.
[Release history](ROADMAP_HISTORY.md#020--published-2026-10-10) retains its package
identities and bounded observations, alongside the older 0.1.0 baseline.
Superseded release preparation and architecture-check exports are retired;
use current commands below rather than their historical runner names.

The checkout prepares `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`. The
[release package table](release/0.3.0-preparation.md#release-packages) identifies
the current archive pair. API/feature
implementation, the shared rc1 Windows/Deck/Machine qualification and final
performance reconciliation are complete within the
[recorded bounds](release/2026-10-29-candidate-verification.md).
[Release preparation](release/0.3.0-preparation.md) records completed publication
simulation and remaining closeout; nothing here authorises publication. The
maintainer approved the updated root README on 2026-10-30; the final pass checks
documentation and links.

Use [migration guidance](MIGRATING_0_3.md) for the canonical public API and
[recovery guidance](DX12_RECOVERY.md) for failure/recording boundaries. Native
performance profiles are separately opt-in; [D043](adr/D043.md) requires a
candidate-bound performance check before every release. The
[rc1 acceptance](release/2026-10-29-rc1-performance-acceptance.md) retains numerical
confidence bounds and limitations. Later runtime/build/SDK changes need the
relevant impact review; prose alone does not justify another native matrix.

## Local setup and checks

The off-by-default `utils` feature adds D036's portable provider-name resolver.
CI and MSRV now cover seven configurations: the existing four plus default+utils,
default+movable+utils and no-default+utils. Consumers verify feature-off imports,
portable normal dependency graphs, public error propagation and complete
feature-enabled examples. Controlled Windows enumeration exercises resolution
through both public ownership families; this is no new GPU compatibility claim.
See the [implementation checks](release/2026-10-22-provider-utils-verification.md).

[D032](adr/D032.md)'s mandatory tuning-domain checks and optional
`validate_upscale_inputs` metadata preflight are implemented;
[DX12 setup/recovery](DX12_RECOVERY.md) describes nonpoisoning preflight,
renderer-owned abort/recreate and initial/batched-idle Configure. Both families
share dispatch inspection; successful preflight never permits skipping it.
The [provider-setup measurement](research/records/2026-10-19-exp-provider-setup-cost.md)
passes three native processes and retains 192 timings. It measures whole setup
paths without selecting a cache or isolating enumeration cost.

The minimum Rust version is **1.88**, declared in both manifests. Use current
stable Rust with rustfmt and Clippy for ordinary development. No SDK, AMD DLL
or Python installation is required for Cargo builds.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --all-targets --no-default-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.88.0 check --workspace --all-targets --locked
cargo +1.88.0 test --workspace --locked
```

Use `RUSTDOCFLAGS="-D warnings"` when validating documentation. Add
`--offline` when dependencies are cached. Tests of new public APIs must
exercise the consumer-visible interface and relevant failures or boundaries.
The [dimensions test](../crates/fsr-sdk/tests/dimensions.rs) illustrates this.

Current source separates `upscaler::common` and `upscaler::dx12`, retaining the
old imports ([D015](adr/D015.md)). Common inputs are available on every host
without backend features; only native APIs retain the Windows/DX12 gate. The
[portable-common verification](ROADMAP_HISTORY.md#portable-inputs-and-loader-checkpoint)
records consumer compatibility, feature boundaries, pinned ABI and selected
native regressions, including the limits of local platform verification.
Native exact filters now use `upscaler::dx12::tests` / `upscaler::dx12::native_dispatch`;
the existing runner commands select them automatically.

The native test helper and its controls now use responsibility-based names.
The [cleanup verification](release/2026-10-15-verification-naming-cleanup.md)
records host checks, preserved historical log parsing and the dispatch-inspection
refactoring. The [Windows batch](release/2026-10-15-windows-feature-batch.md)
confirms rebuilt helpers, independent C++ consumers, controlled inspection,
recording-ahead and supplemental transfer/cleanup regressions.

Repository text uses UTF-8/LF; normalized evidence must use those bytes before
hashing. Scoped historical/export exceptions preserve exact recorded bytes.
See the [contribution rules](../CONTRIBUTING.md#text-files-and-byte-identity) and
[line-ending verification](release/2026-10-15-line-ending-verification.md).

When a test compares paths returned by code that resolves them, resolve its
temporary fixture root before building expected paths. Windows 8.3 aliases and
macOS `/var` symlinks can otherwise make two paths to the same file compare unequal.

Standalone source-compilation fixtures must include the transitive modules used
by imported source, and retain a positive control before testing compiler errors.
The ownership harness now includes the actual query module required by M13's error
field. Check each command's status in shell batches: a later successful command
must not hide an earlier failed test. See the [M13 correction](release/2026-10-14-automatic-exposure-verification.md#ci-failure-and-local-correction).

Fixture directories are created exclusively and skip existing names after
process-ID reuse, without reusing or deleting old fixture artefacts. Other I/O
errors still fail the test. Context and upscaler fixtures share the test-only
[directory helper](../crates/fsr-sdk/tests/fixtures/directory.rs); raw loader tests
use the same collision policy. See the
[2026-10-07 hygiene verification](ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene).

### Current verification system

Choose the scope using [verification selection](verification-selection.md).
The commands below are available entry points, not a mandatory sequence for
each change. Full release qualification is batched after implementation and
documentation integration; intermediate refactors use targeted checks.

[Verification](../tools/verification/README.md) owns current profiles, case IDs,
prerequisites, expected outcomes and evidence commands. Cargo remains the test
runner; existing unit, private/public fixtures, ABI probes and native equipment
remain in their original layers.

```sh
python tools/verification/run.py run fast
python tools/verification/run.py run ci --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run consumers
python tools/verification/run.py plan native-upscaler
```

Use `--require-passed` for CI/release acceptance; see the
[exit-status contract](../tools/verification/README.md#commands-and-claims).

Python 3.11+ is required for orchestration; ordinary Cargo builds do not need it.
Running **every infrastructure test** additionally requires NumPy in that Python
interpreter and PowerShell 7 (`pwsh`) on PATH. Otherwise image-evaluator tests
and real PowerShell-parser tests are explicitly skipped; a partial suite is not
evidence that those checks passed. PowerShell on Linux provides syntax checks,
not Windows SDK execution or Windows Authenticode policy.

The [2026-10-15 Windows batch](release/2026-10-15-windows-m11-m14-verification.md)
passes all 112 CI, 95 MSRV and 27 native-upscaler cases on the recorded
Windows/DX12 configuration. It confirms M11–M14's implemented slices and U03's
SDK helpers, including the corrected M13 ownership fixture. Infrastructure tests
pass 150 of 154 cases; three Proton platform checks and one portable symlink
control are skipped on Windows. These development-source results do not replace
final release-archive and extracted-package verification.

The later [Windows feature/helper batch](release/2026-10-15-windows-feature-batch.md)
passes 115 CI and 98 MSRV cases, plus 180 infrastructure tests with four Windows
skips. Linear HDR native execution and controlled colour forwarding pass.
That batch's supplied-mask case failed before AMD entry under the original
query-based wrapper restriction. A separately declared 79-case selection
confirms independent movable/sharpening/transfer-cleanup regressions without
reclassifying the mask failure. Independent C++ CPU/GPU consumers pass their
45/22-case protocols. Non-linear GPU semantics and final archive checks remain open.

The subsequent [D025 forwarding correction](release/2026-10-15-mask-forwarding-verification.md)
implements the accepted amendment and passes all 115 CI, 98 MSRV and 29 standard
native-upscaler cases. Provider 4.1.1 still reports no optional mask bits, but
accepts all six supplied-mask/exposure combinations and 18 GPU frames with clean
diagnostics and teardown. Both ownership families pass controlled forwarding and
validation fixtures. Reported consumption is not a forwarding allow-list;
successful integration does not establish actual mask consumption.

On an x86_64 Linux host with Nix, the locked
[verification development shell](../tools/verification/nix/README.md) supplies
these dependencies and the host/cross-build tools without replacing the user's
default Python. From the repository root:

```sh
nix develop path:./tools/verification/nix
python3 -m unittest discover -s tools/verification/tests -v
```

The [2026-10-13 complete infrastructure check](release/2026-10-13-infrastructure-dependencies.md)
records its resolved versions and all 125 tests passing without skips.
The experimental Linux-hosted `proton` profile additionally needs a prepared
Windows cross toolchain, host Vulkan driver and CLI Proton launcher; see its
[setup and diagnostic boundaries](../tools/verification/README.md#experimental-proton-verification).
The [Deck/Machine comparison](research/platform-and-backends.md#observed-proton-execution)
records bounded query/single-frame coverage, not the full Windows/temporal suite.
The [selected Deck temporal regressions](research/records/2026-10-13-exp-proton-temporal-regression.md)
extend coverage through four `proton-temporal-*` profiles: reset recovery passes,
quality/history and long-background criteria fail, and a jitter launcher failure
blocks the full jitter image verdict. The [Machine temporal run](research/records/2026-10-14-exp-steam-machine-proton-temporal-regression.md)
also passes reset; quality/long fail foreground motion sensitivity and completed
jitter fails moving alignment. The [integrated comparison and launcher diagnosis](research/upscaling-dispatch.md#selected-proton-temporal-regressions)
preserve these results; exit 247 was not reproduced or explained. Further V07d
quality work is on hold under the [verification priority](../SCOPE.md#verification-priority);
its quality acceptance is deferred. Numerical criteria and ordinary/release
profiles are unchanged; wrapper integration checks continue.
The initial V07 proof is verified under accepted D019. Ordinary Proton launch
requires separate network acknowledgement. The `proton-offline` profile instead
uses checksum-selected local runtime imports and enforced network isolation;
[V08's two Deck confirmations](research/records/2026-10-14-exp-proton-offline-runtime.md)
pass missing-input and shared GPU checks. Remote acquisition/OS closure are outside
that policy. Prepared Proton calls now additionally require bubblewrap PID
containment. [V09's forced-expiry confirmations](research/records/2026-10-14-exp-proton-timeout-cleanup.md)
terminate detached Wine processes and preserve an unrelated prefix; ordinary
query/GPU controls pass afterwards. GPU/kernel hangs remain outside that proof.
Cargo is offline by default; add `--online` for missing Cargo inputs or pinned ABI
headers. [CI](../.github/workflows/ci.yml) runs stable/MSRV profiles on Ubuntu and
Windows, including full no-default+movable execution and MSRV consumer boundaries.
Windows stable also compiles independent common/creation/dispatch/query C++ probes
against four hash-verified headers. CI downloads no AMD binaries and runs no GPU
work. Finalized failed and successful attempts are exported as sanitized artefacts.

The Windows API, including movable, is documented through Rustdoc with warnings
denied. docs.rs targets `x86_64-pc-windows-msvc`; portable builds document common
inputs and errors. Complete development package-README and hidden Rustdoc snippets compile as isolated
downstream examples. The [DX12](../crates/fsr-sdk/examples/dx12_upscale.rs) and
[movable](../crates/fsr-sdk/examples/movable_upscale.rs) examples accept renderer-owned
resources; controlled fixtures establish no AMD threading guarantee.

### ABI and native verification

```sh
python tools/verification/run.py run abi --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run native-upscaler --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run release --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Native execution requires explicit opt-in, PowerShell 7, x64 MSVC, the pinned
trusted local SDK deployment and suitable hardware. The coordinator initializes
the developer environment, checks headers/signatures/DLL hashes, stages exact
test executables and runs isolated watchdog-bounded children. Adapter identity,
driver, provider, commands and payload identities are retained. Explicit adapter
selection never falls back; omit it to use the first suitable hardware adapter.

The native profile covers lifecycle, single-frame/three-frame/semantic regressions,
both public movable sharpening modes and three M9 direct-control integration
repetitions. M9 retains all 144 red half readbacks and independently audits 78
diagnostic comparisons. Image-quality thresholds are absent from this integration
contract; the [original failed effect criteria](research/records/2026-09-30-exp-d014-native-sharpening.md)
remain failed. The selected longer temporal/image baseline is recorded separately
in the [completed V01 checkpoint](ROADMAP_HISTORY.md#v01--temporalimage-baseline--completed-2026-10-12).

Raw allocation, returned-error lifetime, dimensions, provider layouts, shared
runtime, recording-ahead, GBV, original sharpening and private transfer/cleanup
protocols have separately named `experiment-*` profiles. Their controls and
observations are explicit in the plan. An observed abort or timeout cannot become
a passed native acceptance check. Provider termination and broader native lifetime
guarantees remain outside the bounded checks. See the
[native recheck](ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh) for historical limits.

The separate [`experiment-variable-output` spike](../tools/verification/variable-output-protocol.md)
uses raw test-only geometry beyond D024. Its [native record](research/records/2026-10-19-exp-variable-output-resolution.md)
passes 71 integration/evidence cases while distinguishing 4.1.1/3.1.5's variable
write footprints from 2.3.4's maximum output and retaining padding-write observations.
That raw checkpoint implemented no public variable-output API or established
out-of-resource write safety; its allocation/provider/flag choices required the
subsequent decision below.

The subsequently accepted [D024 output amendment](adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
adds public active output within exact maximum allocations. Use
`python tools/verification/run.py run native-variable-output --native --sdk-root PATH --adapter-index 0`
for its separate public-path verification on selected 4.1.1/3.1.5, both ownership
families, four motion profiles, sharpening and rounded output. The independent
input/output audit rejects native errors; padding writes remain observations.
This does not establish out-of-resource access safety or reopen the GBV experiment.
The [implementation report](release/2026-10-19-active-output-verification.md)
records 62/62 public-output and 257/257 consolidated Windows cases passing.
The [renderer example](../crates/fsr-sdk/examples/dx12_active_output.rs) shows
matching selection, maximum allocations and caller-owned recording duties.

### Exact packages and evidence

`release-artifacts` generates or selects the archive pair from current manifest
versions, audits contents/licences, constructs a checksum-verified isolated local
registry, tests extracted bytes and registry-only current/MSRV consumers, then
rechecks that archives and extracted inputs remain unchanged. Examples cover
`utils` on/off, both ownership families and all eight workflow calls on stable
and MSRV. `release` adds source/ABI/native source and artefact checks, including
public callbacks and the packaged raw CB declaration, with shared prerequisites
once. `release-coverage` provides the focused package/examples/callback/native
checkpoint without repeating the full source/consumer contract suite.
Neither publishes. Registry publication observation remains separate.

Supported feature-native profiles and `proton`, `proton-offline`, and
`proton-timeout` accept `--artifacts`. Supply both `--wrapper-archive` and
`--sys-archive` to reuse a verified pair; this also enables archive mode implicitly.
Omitting the pair with `--artifacts` generates fresh packages. Use
`plan <profile> --artifacts` to inspect the package prerequisites and final
integrity check. See the [supported profiles and commands](../tools/verification/README.md#exact-package-native-checks).
`release-dry-run --online` verifies the selected archives and checks Cargo's
publication simulation, requiring both uploads to be aborted and regenerated
archive hashes to match. The [infrastructure verification record](ROADMAP_HISTORY.md#verification-infrastructure--implemented-2026-10-10)
records the completed migration and its bounded execution evidence.

Every invocation creates a fresh attempt under `target/verification`; resume is
disabled. Source/configuration/tool identity, execution and requirement evaluation
are recorded separately. Failed logs/readbacks remain available. `verify-run`
checks completeness and hashes; `analyze` writes separate offline results;
`export --run <attempt> --output <new-directory>` validates integrity, normalizes
paths/pointers and selects evidence without overwriting source or destination.
See [verification commands and boundaries](../tools/verification/README.md).

Temporal calibration now treats full generated frames and verbose analyses as
temporary: the independent audit consumes them before default compaction, on
both successful and failed finalized attempts. Use `--retain-diagnostics` only
for a concrete local diagnosis; exports remain compact. Keep permanent reports,
protocols, results and selected examples small. Run
`python tools/verification/repository_size.py` before committing; CI enforces
the checkout budgets defined in that tool and rejects raw diagnostic
payloads/source snapshots under `docs/`. Full offline reanalysis of a compact
historical record requires a new GPU run and cannot recover old outputs from hashes.

The separate `experiment-temporal-diagnosis` profile examines contrary quality
rankings with a fixed scene/control matrix, exact input checks and independent
point/area references. It requires existing NumPy and Pillow, remains excluded
from ordinary/release profiles, and shares compact default retention with
calibration. See the [protocol and commands](../tools/verification/README.md).

`experiment-temporal-quality` adds analytic known-defect tests for the evaluator,
then three fresh native confirmations under frozen, calibration-derived local
budgets. Its full texture/speed/motion factorial reports interactions without
requiring every wrong-input control to lose every metric. It requires existing
NumPy; ordinary profiles remain unchanged. The verification README describes
the scoped claims, standalone CPU command and compact evidence policy.

`experiment-temporal-long` runs the textured-slow correct/reversed arms for
256 continuous frames each, in three process blocks. The same five budgets
and motion sensitivity predictions apply separately to seven 32-frame windows
after warmup. Per-frame trajectories reveal late changes. The 64-frame profiles
retain their original counts; the shared diagnosis/quality frame plans are
bounded to 64..256 in multiples of 32. Native resources remain retained until
context destruction.
See the verification README for the protocol, command and retention boundary.

`experiment-temporal-reset-explore` gathers one independent cut/reset block;
`experiment-temporal-reset` confirms separately frozen criteria in three new
blocks. Matched post-cut inputs separate history/reset/fresh-context effects.
The verification README owns the selection command, fixed recovery/sensitivity
predictions, analytic windows and compact evidence policy.

`experiment-temporal-jitter-explore` validates a spatial phase estimator against
known translations and independent brightness/contrast changes before one
128-frame-per-arm GPU matrix. `experiment-temporal-jitter` uses three new
processes after alignment and local quality criteria have been frozen. Four
stationary start phases and one moving phase separate valid sequence starts
from X/Y sign, delayed-dispatch and dispatch-only zero-jitter errors.
See the verification README for the selection rule and evidence boundaries.
The [jitter confirmation record](research/records/2026-10-12-exp-temporal-jitter-phase.md)
retains the failed input-only attempt, selection, three fresh confirmations,
mixed individual quality rankings and the bounded completion evidence.


All former milestone, shared native, ABI, consumer and dated collector entry
points are frozen in the [indexed archive](../tools/verification/archive/2026-10-10/README.md).
Historical command text, logs, criteria and findings remain intact. No current
profile imports or executes archived machinery. The read-only archive checker
verifies original bytes, dependency/navigation inventory and source syntax:

```sh
python tools/verification/archive/check.py
python -m unittest discover -s tools/verification/tests -v
```

The archive checker needs Python 3.11+ for integrity and Python/JSON syntax.
It also parses PowerShell when `pwsh` is available; otherwise that subcheck is
reported as `not_run` / `not_evaluated` with its reason. Add
`--require-powershell` to require PowerShell 7 parsing, as Windows `ci` and
`release` do. Installed parser failures remain failures in either mode. The
coordinator retains separate archive subcheck outcomes in its evidence;
`--report <new-json-file>` also makes them available for standalone checks.

## Historical command index

Superseded milestone/release command collections are retired. [Release history](ROADMAP_HISTORY.md)
retains relevant outcomes and limits; the [script archive](../tools/verification/archive/2026-10-10/README.md)
retains original runner source for historical reconstruction. Use the current
commands above for this checkout and the [research index](research/README.md)
for original experimental evidence.

## Feature and platform boundaries

The supported native target is `x86_64-pc-windows-msvc` with the `dx12` feature.
Windows/DX12 APIs are exposed on other Windows targets, but those targets are
currently unverified and unsupported; compilation may succeed.
[SCOPE.md](../SCOPE.md#platform-and-compatibility-boundaries) owns this support policy.
Native verification is limited to the SDK, runtime/provider, GPU, driver and
workload configurations recorded in the verification records, including
[M6b](research/records/2026-09-25-m6b-public-dispatch-verification.md). Target
support does not imply compatibility with every hardware/provider configuration.

`fsr-sdk` enables `dx12` by default and forwards it to `fsr-sdk-sys`. The raw crate
has no default features. Its loader module is gated by Windows and `dx12`.
The reserved `vulkan` feature and deliberate compile error are retained under
the [D031 amendment](adr/D031.md); initial usable Vulkan upscaling is targeted
at wrapper `0.4.0`, subject to official SDK/runtime availability and native evidence.
Vulkan remains a required future support target; Windows/DX12 code, native types
and production dependencies must stay isolated from common and future Vulkan
consumers. `--all-features` deliberately fails while Vulkan remains unimplemented.
Check implemented combinations explicitly (`dx12,movable,utils` for the wrapper,
`dx12` for sys), retain the expected Vulkan-error consumers for both crates and
keep the separate no-default/platform checks to verify those boundaries.

Host builds on macOS do not compile the Windows loader. Run the relevant checks
on Windows with DX12 enabled when changing that boundary. Cross-compilation may
check target-specific Rust code, but cannot establish that a DLL loads or a GPU
operation works. The native SDK baseline is `v2.3.0`
([D002](DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
Explicit maintainer SDK acquisition is available through `run.py sdk install`;
`sdk verify` rehashes the installation offline, and `--strict` adds Windows trust
checks which may need network access. See [SDK acquisition](../tools/verification/README.md#sdk-acquisition).
Ordinary Cargo builds and profile runs do not download AMD binaries.
Five selected public upscaler Queries are implemented under [D018](adr/D018.md).
Its [Windows verification](release/2026-10-12-windows-m10-sdk-verification.md)
records ABI, controlled/consumer/MSRV and native GPU checks on one configuration,
alongside real `sdk.py` strict trust verification. Later D028 selection and D029
`request_tuning` are implemented; their [coherence checks](release/2026-10-18-api-coherence-verification.md)
cover local/packaged behaviour and Windows compilation. The later
[Windows follow-up](release/2026-10-19-windows-upscaler-verification.md) records
MSVC ABI, both-family selected-provider execution and tuning/dispatch cycles;
the [shared rc1 qualification](release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
supplies subsequent exact-package coverage.
The raw loader's forwarding methods do not validate native operation contracts.

The wrapper's private `RuntimeState` acquires the loader, translates load failures
and owns it behind native-call methods. Local and movable runtimes use the same
acquisition method, then retain the state through `Rc` or `Arc`; partial-load
cleanup stays on the acquiring thread. Context code cannot access its library
field. Context ownership, native result handling and terminal cleanup remain in
`NativeContextOwner`. Host fixtures share
the actual [runtime state](../crates/fsr-sdk/src/runtime/state.rs) while replacing
only the platform loader. This boundary changes neither public API nor native,
GPU or threading contracts.

## FFI safety and verification

For native changes, establish and document the applicable contracts:

- Compare layouts, alignment, integer widths, constants, and calling conventions
  against pinned AMD headers. Add meaningful ABI checks as bindings are added.
- Document unsafe public APIs with caller obligations. Explain unsafe blocks
  with `SAFETY` comments identifying the invariants that justify the operation.
- Keep libraries alive while native functions or objects depend on them. Verify
  context destruction, partial initialization cleanup, and descriptor lifetimes.
- Establish resource states, GPU completion, ownership, and threading guarantees;
  Rust borrowing alone cannot guarantee GPU synchronization or justify `Send`
  and `Sync`.
- Exercise relevant failure paths, including missing libraries or symbols and
  native errors. Preserve native error information.

Use focused tests for deterministic validation, conversions, and ownership logic.
Use native integration checks for ABI and runtime behaviour; mocks cover wrapper
logic but cannot validate AMD's runtime. Add a regression test for a fixed bug when
it meaningfully captures the failure.

Report native evidence separately: Windows compilation, runtime loading and symbol
resolution, successful query, context lifecycle, and actual GPU dispatch. For
runtime results, identify the SDK/runtime version, OS, driver, and GPU when
relevant, plus inputs and observed outcomes. Correct output and performance need
their own evidence. If hardware or runtime access is unavailable, state exactly
what remains untested. Bounded investigations use the
[research process](research/PROCESS.md); ordinary checks need only a clear report.

D040's explicit native callbacks and debug controls are covered by
`native-callbacks`, using the explicit native/SDK/adapter arguments above.
The [verification record](release/2026-10-25-callbacks-diagnostics-verification.md) separates controlled failure tests, production Rust/MSVC ABI exchange and
both-family selected-provider GPU execution. Heap SDK invocation remains an
explicit limit; the heap route has ABI and controlled invocation coverage.
