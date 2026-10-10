# Development

Current contributor commands and verification boundaries live here. Dated
release observations are retained in the [verification history](release/2026-09-30-development-history.md).
See [CONTRIBUTING.md](../CONTRIBUTING.md) and [code heuristics](CODE_HEURISTICS.md) before changing contracts.

## Published release

Both crates were published as 0.1.0 on 2026-09-28. The
[publication record](release/2026-09-29-publication-check.md) identifies the registry artefacts.
Current source changes do not modify that published release.

## Local setup and checks

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
[portable-common verification](release/2026-10-09-portable-common-verification.md)
records consumer compatibility, feature boundaries, pinned ABI and selected
native regressions, including the limits of local platform verification.
Native exact filters now use `upscaler::dx12::tests` / `upscaler::dx12::m6`;
the existing runner commands select them automatically.

Fixture directories are created exclusively and skip existing names after
process-ID reuse, without reusing or deleting old fixture artifacts. Other I/O
errors still fail the test. Context and upscaler fixtures share the test-only
[directory helper](../crates/fsr-sdk/tests/fixtures/directory.rs); raw loader tests
use the same collision policy. See the
[2026-10-07 hygiene verification](release/2026-10-07-status-test-hygiene.md).

### Continuous integration

The [workflow](../.github/workflows/ci.yml) checks current stable Rust and exactly
Rust 1.88.0 on Ubuntu and Windows x64/MSVC. Minimum-version jobs check all targets,
run tests with and without default features. Stable jobs also deny Clippy and
Rustdoc warnings. Windows additionally compiles the independent native ABI checks
against four hash-verified headers at the pinned SDK commit.
Feature-enabled tests, Clippy and Rustdoc cover `movable`; minimum-version jobs
also compile it without DX12. Windows and Ubuntu check downstream common/native
API boundaries with [check-movable-api.ps1](../tools/check-movable-api.ps1).
Windows additionally checks the movable feature and family boundaries.
CI does not download AMD binaries or execute GPU work. Ignored native tests need
the opt-in local runner below; successful host builds are not native evidence.

### Consumer API documentation and example

The Windows/DX12 API is documented by Rustdoc with missing documentation warnings
enabled. docs.rs targets `x86_64-pc-windows-msvc`. The
[DX12 example](../crates/fsr-sdk/examples/dx12_upscale.rs) accepts renderer-owned
resources; its executable is guidance rather than a stand-alone renderer.
`Runtime::load_unchecked` requires caller-established DLL trust, while
`Upscaler::dispatch` requires states, submission and GPU lifetimes from the caller.

The optional public family adds [movable cleanup coordination](../crates/fsr-sdk/examples/movable_upscale.rs).
Its example is compiled directly into public consumer fixture tests, covering
independent siblings, early runtime release, failed handoff, unwind and poisoned
lock recovery. These checks use a controlled DLL and an atomic ownership-only
COM stub; they establish no AMD threading guarantee. Run:

```sh
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked -- -D warnings
cargo test --workspace --features fsr-sdk/movable --locked
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked
cargo doc --workspace --features fsr-sdk/movable --no-deps --locked
cargo +1.88.0 test --workspace --features fsr-sdk/movable --locked
```

Run `./tools/check-movable-api.ps1` in PowerShell 7 for positive common/raw-ABI
consumer baselines and diagnostic-specific failures for unavailable native APIs.
On Windows it also checks missing movable features, crossed families and
conversions. It uses cached dependencies offline. Ordinary Cargo builds with
`movable` but without `dx12`, or on non-Windows hosts, omit the movable API.

### Common ABI verification

Run [check-native-abi.ps1](../tools/check-native-abi.ps1) in an x64 Visual Studio
developer shell. It fetches four headers at the pinned commit and verifies their
SHA-256 hashes before compiling common, creation and dispatch probes. Alternatively
pass `-SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX` to use local headers.
This compile-only check links no AMD runtime and is independent of the Cargo build.

### Native verification

The shared [native runner](../crates/fsr-sdk/tests/native/run.ps1) accepts
`-Mode lifecycle` or `-Mode dispatch`. Ordinary checks run by default;
`-Native` opts into locally acquired, trusted signed SDK binaries and hardware.
Run it in an x64 VS developer shell. It stages helpers and DLLs under `target`,
records fingerprints, checks signatures, and executes each GPU case in an isolated
process with a timeout. Existing `run-m4.ps1`, `run-m5.ps1` and `run-m6.ps1`
commands remain compatibility entry points. `-GpuValidation` is an opt-in diagnostic.
The [D013 transfer-failure fixture spike](research/records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
runs in ordinary tests: fourteen watchdog-bounded control/transfer children and
acquiring-worker loader failures exercise the shared private dispatch/terminal
owner. It uses locally compiled fixtures only and establishes no AMD transfer
or public movable-API support.
Dispatch mode includes the [controlled semantic comparisons](research/records/2026-09-30-exp-dispatch-semantics.md)
for motion direction, jitter direction and history reset, alongside the earlier
single-frame and three-frame regressions.
The experiment-only `-Mode d013 -Native` runs six isolated same-thread/transfer
cases using the real Arc-retained private owner; GBV is excluded. See its
[native transfer record](research/records/2026-09-30-exp-d013-native-thread-transfer.md)
for commands, configuration and the provider-unload limitation.
The implementation-verification `-Mode movable -Native` enables the actual public
family, reruns local lifecycle/dispatch/semantic regressions, and checks two fenced
public movable frames with independent siblings and coordinated destination-thread
Drop during unwind/failed handoff. GBV is excluded. See the
[2026-10-09 public-path record](release/2026-10-09-d013-public-api-verification.md).
This is a bounded implementation check, not a new native research matrix.
The follow-up `-Mode d013-cleanup -Native` first runs existing native regressions,
then three isolated repetitions of each control/transfer case for creation after
runtime transfer, continuous history, independently held siblings and unwind
cleanup. Tests under `tests/native/thread_transfers.rs` have functional names.
Its [2026-10-08 record](research/records/2026-10-08-exp-d013-native-transfer-cleanup.md)
separates exact selected readback comparisons from natural provider residency.
The experiment-only `-Mode sharpening -Native` runs D014's three isolated
stationary-scene comparisons and writes final red half images under the target
stage for independent auditing; GBV is excluded. Its
[research record](research/records/2026-09-30-exp-d014-native-sharpening.md)
preserves the successful execution and failed proposed effect criteria.
Renderer objects, submission, fences and readback belong to test equipment under
`crates/fsr-sdk/tests/native`; production modules contain no GPU harness.

## Historical command index

Earlier release records link to the anchors below. Their exact commands and
observations remain in the dated history; they are not instructions to publish.

### Crate packaging

See [recorded commands and observations](release/2026-09-30-development-history.md#crate-packaging).

### Release simulation

See [recorded commands and observations](release/2026-09-30-development-history.md#release-simulation).

### Runtime acquisition API rename

See [recorded commands and observations](release/2026-09-30-development-history.md#runtime-acquisition-api-rename).

### Options construction API and native target policy

See [recorded commands and observations](release/2026-09-30-development-history.md#options-construction-api-and-native-target-policy).

### Loader verification

See [recorded commands and observations](release/2026-09-30-development-history.md#loader-verification).

### Native query smoke test

See [recorded commands and observations](release/2026-09-30-development-history.md#native-query-smoke-test).

### M4 context ownership checks

See [recorded commands and observations](release/2026-09-30-development-history.md#m4-context-ownership-checks).

### M5 public construction checks

See [recorded commands and observations](release/2026-09-30-development-history.md#m5-public-construction-checks).

### M6a private native dispatch proof

See [recorded commands and observations](release/2026-09-30-development-history.md#m6a-private-native-dispatch-proof).

### M6b public dispatch verification

See [recorded commands and observations](release/2026-09-30-development-history.md#m6b-public-dispatch-verification).

### Final Windows release verification

See [recorded commands and observations](release/2026-09-30-development-history.md#final-windows-release-verification).

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
The `vulkan` feature intentionally raises a compile error; do not use
`--all-features` as the workspace's success check.

Host builds on macOS do not compile the Windows loader. Run the relevant checks
on Windows with DX12 enabled when changing that boundary. Cross-compilation may
check target-specific Rust code, but cannot establish that a DLL loads or a GPU
operation works. The native SDK baseline is `v2.3.0`
([D002](DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
No automated SDK acquisition or public Query/Configure operations are implemented.
The raw loader's forwarding methods do not validate native operation contracts.

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
Use native integration checks for ABI and runtime behavior; mocks cover wrapper
logic but cannot validate AMD's runtime. Add a regression test for a fixed bug when
it meaningfully captures the failure.

Report native evidence separately: Windows compilation, runtime loading and symbol
resolution, successful query, context lifecycle, and actual GPU dispatch. For
runtime results, identify the SDK/runtime version, OS, driver, and GPU when
relevant, plus inputs and observed outcomes. Correct output and performance need
their own evidence. If hardware or runtime access is unavailable, state exactly
what remains untested. Bounded investigations use the
[research process](research/PROCESS.md); ordinary checks need only a clear report.
