# Development

This document covers local checks and native verification. Contribution policy
lives in [CONTRIBUTING.md](../CONTRIBUTING.md); design guidance lives in
[CODE_HEURISTICS.md](CODE_HEURISTICS.md). Manifests, source, and any future CI
configuration establish the checks and features actually implemented.

## Local setup and checks

Use a stable Rust toolchain supporting the workspace's Rust 2024 edition and
resolver 3, with rustfmt and Clippy available. No minimum Rust version is declared
in the manifests yet. Python is not required to build either crate.

Run from the repository root, selecting checks appropriate to the change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --no-default-features --locked
cargo doc --workspace --no-deps --locked
```

Add `--offline` to Cargo build, test, lint, and documentation commands when all
required dependencies are cached. Keep `--locked` for validation; intentional
dependency changes may require updating the lockfile separately.

These commands were checked on macOS on 2026-09-16 with cached dependencies.
Clippy, tests, no-default-features checking, and documentation generation passed.
That historical pass established compilation only. The first common ABI slice
now has three Rust tests plus compile-time type checks; the empty placeholders
have been formatted. The loader adds independent fixture tests and an opt-in
AMD DLL load test. CI runs the ordinary checks on Ubuntu and Windows x64/MSVC.

### Continuous integration

The [CI workflow](../.github/workflows/ci.yml) runs on pull requests, pushes to
`main`, and manual dispatch. Ubuntu checks formatting, Clippy, ordinary tests,
the no-default-features build, and documentation with warnings denied. Windows
x64/MSVC runs Clippy, ordinary tests, and the no-default-features build. The
shared Rust setup installs stable Rust with the required components and restores
a Cargo cache. Validation uses the lockfile.

The ordinary test command runs Rust-side ABI checks and local fixture tests. It
skips the ignored tests that require AMD DLLs or a DX12 device. CI does not
acquire the SDK, compile the paired native C++ header checks, load an AMD
runtime, or execute GPU work. Keep those forms of evidence separate from a
passing CI result. The `vulkan` feature deliberately fails compilation, so CI
does not use `--all-features`. No MSRV is declared yet.

### Common ABI verification

`crates/fsr-sdk-sys/tests/abi.rs` checks return constants, nullable function-pointer
layouts, exact Rust types/signatures, and Windows x64 MSVC struct sizes, alignment
and field offsets. Fixed numeric struct expectations are gated to that target;
tests on other hosts do not establish Windows layout compatibility.

The paired `crates/fsr-sdk-sys/tests/native_abi.cpp` checks the same expectations
against the local SDK v2.3.0 `Kits/FidelityFX/api/include/ffx_api.h`, including
native declarations and explicit `__cdecl` function-pointer signatures. It is
optional, compile-only, and separate from Cargo: no SDK download, linking, build
script or runtime invocation is involved. Keep the paired expectations aligned
when changing the ABI slice.

From the repository root in an **x64 Native Tools Command Prompt for VS 2022**:

```bat
if not exist target\abi-check mkdir target\abi-check
cl /nologo /std:c++17 /W4 /WX /c /Iexternal\FidelityFX-SDK\v2.3.0\Kits\FidelityFX\api\include crates\fsr-sdk-sys\tests\native_abi.cpp /Fotarget\abi-check\native_abi.obj
```

Verified on 2026-09-20 with Rust 1.98.1 (`x86_64-pc-windows-msvc`) and
VS 2022 Build Tools: all three ABI tests and the C++ header check passed.
This establishes the tested common ABI declarations, not runtime or GPU support.
The [verification record](research/records/2026-09-20-exp-common-abi-verification.md)
preserves the tested revision, local-header fingerprint, observations and limits;
the [API/safety synthesis](research/api-and-safety.md) maintains current findings
and remaining questions.

### Loader verification

On Windows with `dx12`, ordinary workspace tests compile small Rust fixture DLLs
using `rustc` from PATH (matching the test executable's toolchain/architecture).
They need no SDK, AMD runtime or GPU. Fixtures are emitted into unique directories
under the OS temporary directory, named `fsr-sdk-loader-<pid>-<counter>`.
Four tests cover a missing file, an invalid DLL, each of the five missing exports
(including partial-load cleanup), and argument/return forwarding plus unloading
after moving and dropping the owner. Two compile-fail doctests check owner
borrowing and private pointer access. No AMD API functions run in these tests.

The separate ignored integration test loads a trusted DLL, resolves its five
exports, and drops it without invoking any API function. From PowerShell at the
repository root, explicitly opt in with:

```powershell
$env:FSR_SDK_TEST_DLL = (Resolve-Path external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll).Path
cargo test -p fsr-sdk-sys --features dx12 --test loader --locked --offline loads_amd_runtime_without_api_calls -- --ignored --exact
```

This executes DLL initialization/termination code. Only supply a trusted,
ABI-compatible SDK DLL and dependencies. The environment variable is solely a
test input, not a library discovery mechanism. The local v2.3.0 DLL passed on
2026-09-20; details and limitations are in the
[loader record](research/records/2026-09-20-exp-explicit-dll-loader.md).

### Native query smoke test

`tests/query.rs` adds one ignored count-only upscaler version query. It passes a
null context and device and checks `FFX_API_RETURN_OK` plus a positive count.
No contexts or GPU resources are created. The tested local setup returned 2.
The absolute path to the original SDK loader alone returned `NO_PROVIDER`;
staging the test executable with the loader and upscaler DLLs succeeded.
Use the exact PowerShell build/staging/run commands in the
[query record](research/records/2026-09-20-exp-version-count-query.md#reproduction).
This is local test deployment, not automatic runtime discovery or installation.
The ordinary Cargo test run skips it. The paired ABI checks now include the
56-byte query descriptor, all field offsets/types and its two required tags.

### M4 context ownership checks

`fsr-sdk` now contains a private terminal lifecycle owner and Windows/DX12
upscaler creation state. Ordinary wrapper tests use a locally compiled fixture
library and no AMD runtime. On macOS a test-only runtime adapter exercises the
same owner; on Windows/DX12 the fixtures go through the production FfxLibrary.
Counters cover cleanup/retention of state, device and runtime ownership; Windows
also checks fixture DLL residency. The Mac adapter's drop signal does not prove
that dyld actually unloads the fixture. Compile checks include a positive control
before rejecting use after consuming destroy and private handle access.

The concrete Windows tests check stable descriptor addresses and owner trait
exclusions. New production ABI declarations are paired with the existing
`tests/context_lifecycle/native_abi.cpp`; provider identification remains
instrumentation-only. The ordinary native lifecycle now uses the production
wrapper owner, not a second raw ownership implementation.

On Windows with Rust and MSVC, run ordinary checks with:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1
```

From an **x64 VS developer shell**, with the trusted local SDK v2.3.0 and a
hardware DX12 device, explicitly opt into native verification:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1 -Native
```

The latter also compiles paired C++ ABI checks, builds the helper, stages SDK
DLLs and runs `upscaler::tests::native_lifecycle` in a separate process with a
60-second timeout. Stdout/stderr/status are saved under `target/m4-lifecycle/`.
It uses production create/destroy policy, identifies the provider while the
context is live and submits no GPU work. These Windows execution checks passed
on 2026-09-22: 17 ordinary executable tests, two compile-fail doctests, paired C++
ABI compilation and the isolated production-owner lifecycle (provider 4.1.1,
RX 9060 XT, driver 32.0.31041.1004). The
[M4 verification record](research/records/2026-09-22-exp-m4-windows-verification.md)
preserves inputs, results and the process-local PowerShell execution-policy
override needed on this machine. Historical raw-runtime evidence is
preserved in the [lifecycle record](research/records/2026-09-20-exp-context-lifecycle.md).

The old `crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1` invocation without
switches delegates to this production-owner runner. Its research switches remain
separate from M4 verification.

Pass `-FailureInjection` to the sys research runner for the separate host-allocation
failure experiment. It measures an instrumented successful control, then runs
each observed allocation index and one unreached control in separate processes
with 60-second timeouts. Per-process status and output are retained under
`target/context-lifecycle/failure-N.*.txt`. Child crashes are observations, so
runner success does not imply that all native calls succeeded. Ordinary tests
skip this probe. The tested runtime crashed at all three injected indices;
see the [failure record](research/records/2026-09-20-exp-create-allocation-failure.md)
for callback balances, reproduction and limits.
`-InputLifetime` likewise retains its separate raw probe, now with a production-
owner successful control. Historical records describe their original runner.

On a non-Windows host with the Windows standard library installed, this checks
Rust code and tests without linking or executing Windows binaries:

```sh
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked -- -D warnings
```

No Rust cross-check substitutes for the native C++/DLL/device checks above.

## Feature and platform boundaries

`fsr-sdk` enables `dx12` by default and forwards it to `fsr-sdk-sys`. The raw crate
has no default features. Its loader module is gated by Windows and `dx12`.
The `vulkan` feature intentionally raises a compile error; do not use
`--all-features` as the workspace's success check.

Host builds on macOS do not compile the Windows loader. Run the relevant checks
on Windows with DX12 enabled when changing that boundary. Cross-compilation may
check target-specific Rust code, but cannot establish that a DLL loads or a GPU
operation works. The native SDK baseline is `v2.3.0`
([D002](DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
No acquisition mechanism or high-level runtime operations are implemented.
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
