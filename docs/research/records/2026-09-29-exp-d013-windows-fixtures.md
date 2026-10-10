# Experiment: D013 Windows x64/MSVC fixture execution

- Investigation date: 2026-09-29 (Europe/Berlin, UTC+02:00).
- Question: Do the eight D013 probes from the
  [compile/host experiment](2026-09-29-exp-d013-movable-compile-host.md)
  actually execute successfully on Windows x64/MSVC, using the production
  Windows loader and its loaded-module observations?
- Success: all eight probes execute (none ignored or filtered from the focused
  run), their ownership/thread/retention assertions pass, and the requested
  workspace test, Clippy and feature-disabled checks exit 0.
- Failure: a missing/skipped probe, unexpected trait, changed retained pointer,
  wrong cleanup/thread order, premature library release, repeated destruction,
  failed error/unwind control, or failed workspace check.
- Scope: locally compiled fixture DLLs and Rust compilation/tests only.
  No AMD DLL, provider, hardware DX12 device, GPU commands, native ABI C++
  helper, native regression or native thread-transfer experiment was run.

## Setup and tested source

The final checks ran against clean `main` at
`cf8dbf051df83163e6b62d1ed091c8bd6a09fb84`. HEAD and worktree cleanliness were
checked again after execution. There were no local source edits during the
checks; this new record is the only resulting repository edit. Production
code, dependencies, manifests, historical records and ADR status were left
unchanged. No commit or push was performed.

The initial checkout was clean at `bc2c5daf8442b8403e512f9b603409a6eea7250e`
and lacked both the supplied baseline record and all D013 files. Reading the
baseline failed with a file-not-found error. A preliminary workspace test run
at that state passed 25 wrapper tests, 12 sys tests and four doctests, with
11 native tests ignored; it did not contain or establish D013 coverage.
The user then supplied the updated checkout. All four requested commands
and the focused D013 execution below were completed on the final source state.
An earlier Clippy invocation also exited 0 after the updated checkout appeared;
the final Clippy result below was verified again on the fixed final state.

The tested implementation is self-contained in these repository inputs:

- [context.rs](../../../crates/fsr-sdk/src/context.rs): actual private terminal
  owner, sealed `RuntimeRetention` with Rc/Arc choices; production defaults to
  `Rc<RuntimeState>` and retains the `PhantomData<Rc<()>>` local trait marker.
- [d013_movable.rs](../../../crates/fsr-sdk/src/context/d013_movable.rs): eight
  test-only probes and a narrowly audited `unsafe impl Send` for the concrete
  fixture owner; no public movable API.
- [tests.rs](../../../crates/fsr-sdk/src/context/tests.rs): fixture builder and
  private-owner compiler controls. The builder invokes host `rustc` on
  [lifecycle.rs](../../../crates/fsr-sdk/tests/fixtures/lifecycle.rs), a Rust
  `cdylib` with five exports and a fixture-specific protocol.
- [runtime.rs](../../../crates/fsr-sdk/src/runtime.rs) and
  [loader.rs](../../../crates/fsr-sdk-sys/src/loader/windows.rs): actual Windows/DX12
  runtime state and `FfxLibrary`, used by these feature-enabled Windows probes.
  The [host adapter](../../../crates/fsr-sdk/src/context/test_runtime.rs) is
  not the runtime path of this focused run.

Environment:

| Input | Observed value |
|---|---|
| OS / process architecture | Microsoft Windows 10.0.26300 / X64 |
| Rust host / active toolchain | `x86_64-pc-windows-msvc` / `stable-x86_64-pc-windows-msvc (default)` |
| rustc | 1.98.1, commit `48a229ceaefd4985c50990b14116b6d856af0985`, 2026-09-01 |
| LLVM | 22.1.8 |
| Cargo | 1.98.1 (`797e8a9bc`, 2026-08-05) |
| Shell | PowerShell 7.6.6 |
| Installed Visual Studio | Build Tools 2022 17.14.28, installation version 17.14.37027.9 |
| Installed default MSVC toolset / x64 linker file version | 14.44.35207 / 14.44.35224.0 |
| Installed Windows SDK library directory | 10.0.22621.0 |
| Locked dependencies relevant to probes | `libloading` 0.9.0, `windows` 0.62.2 |
| Workspace versions / features | both crates 0.1.0; wrapper default `dx12`, sys no default features |

No repository `.cargo/config.toml` was present. `RUSTFLAGS`,
`CARGO_BUILD_TARGET`, `RUSTUP_TOOLCHAIN`, `VCToolsVersion`, `WindowsSDKVersion`
and `VSCMD_ARG_TGT_ARCH` were unset. No developer shell was initialized for
this run; Rust's normal MSVC toolchain discovery linked the executable tests
and the temporary Rust fixture DLLs successfully. `cl` was not on PATH and
no C++ compiler command was required. Installed MSVC/SDK versions are setup
inventory, not a captured verbose linker-command trace. GPU/driver data are
irrelevant to this fixture-only scope and were not collected. The project SDK
profile remains v2.3.0; SDK artifacts were not inputs to this experiment.

SHA-256 fingerprints of tested working-tree bytes (Windows CRLF checkout):

| Repository path | SHA-256 |
|---|---|
| `Cargo.toml` | `4bb3c5248c4e21374b6453be653f4d7e15dbd69684d45bd60d51a41f61c322d1` |
| `Cargo.lock` | `cae6468b152bbddf918e5fe6c16d54f697957f3793cffe7da07c995dc8ff5a7c` |
| `crates/fsr-sdk/Cargo.toml` | `8d4fe0142ebbad4cf49e8731c97be91c11135063bcf1b59789cb6ef5a2740ace` |
| `crates/fsr-sdk-sys/Cargo.toml` | `e5e0677fde653d72e4d419ad312d95c69cc41fda7b4d81d8fea8235572ef4569` |
| `crates/fsr-sdk/src/context.rs` | `d97d090be1b9fb4195a262d4ea57d6ca8ce134db91c202a2dc75742f45c90413` |
| `crates/fsr-sdk/src/context/test_runtime.rs` | `d320ecbe3d9c4afe9f9b232c0d864843d6e760c8dae680d2453545634398f969` |
| `crates/fsr-sdk/src/context/tests.rs` | `432078729b756359371039f2cfc8137ab90e5992cd55d330139432e155b06727` |
| `crates/fsr-sdk/src/context/d013_movable.rs` | `28d424234f945e1dd5ffe7f7096bfa1d82ffd593605ecd19c2ff6c107af5677d` |
| `crates/fsr-sdk/tests/fixtures/lifecycle.rs` | `1c6571d930615966e2539928ee7913dcb63430351a02d9662ff8cfba88594b6a` |
| `crates/fsr-sdk/src/runtime.rs` | `431f754c50f84e093f7360d3c57c208714c24538d950c0312f11af684065841e` |
| `crates/fsr-sdk/src/upscaler.rs` | `832555188e0d8225e81995a0b2f87b40e561ca76981e0de4511b582b71013fe9` |
| `crates/fsr-sdk-sys/src/loader.rs` | `d92a04dfa50936ab458300eacc460a1f33f152ee0675dad102156b638f58d5a0` |

Git `core.autocrlf` was `true`. Independently hashing UTF-8 text after replacing
CRLF with LF reproduced all nine hashes in the supplied compile/host record
(lockfile, owner, adapter, test harness, D013 probes, lifecycle fixture, runtime,
upscaler and loader). Thus the raw-byte differences above are line-ending
differences, not substantive changes to those baseline inputs.

## Reproduction and command results

Run from the repository root in PowerShell with Rust/MSVC available and the
locked dependencies cached. The fixture helper runs:

```text
rustc --edition=2024 --crate-type=cdylib --crate-name=lifecycle_fixture <repo>/crates/fsr-sdk/tests/fixtures/lifecycle.rs -o <temp>/fsr-sdk-context-<process>-<sequence>/fixture.dll
```

`<repo>` and `<temp>` replace personal/transient absolute paths; each fixture
has a distinct temporary directory. No new dependency, SDK download,
installation or environment override was needed. The commands below were
executed as written, with default test scheduling and without `--ignored`:

```powershell
cargo test -p fsr-sdk --lib d013_movable --locked -- --nocapture
rustc -Vv
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --workspace --all-targets --no-default-features --locked
```

| Command | Exit | Observed result |
|---|---|---|
| Focused D013 test command | 0 | 8 passed, 0 failed, 0 ignored, 30 filtered out; actual Windows executable execution |
| `rustc -Vv` | 0 | Rust/host/LLVM values recorded above |
| Workspace test command | 0 | 33 wrapper tests, 12 sys tests, four doctests passed; 11 native tests ignored; zero failures |
| Workspace Clippy command | 0 | all targets checked with Rust warnings denied |
| Feature-disabled all-targets check | 0 | both crates checked successfully; compilation only |

The final workspace run executed all eight D013 probes again. Sys counts are
six ABI, two dispatch ABI and four loader-fixture tests. The four doctests
are two wrapper compile examples and two sys loader compile-fail checks.
The ignored tests comprise five wrapper native tests and six sys native tests;
they require explicit AMD/runtime/device or isolated research runners and were
deliberately excluded. Executable tests used native Windows `.exe` binaries;
this is not cross-compilation-only evidence.

## Observations

**Verified by execution:** every focused test below printed `ok`:

| Probe (`context::d013_movable::` prefix) | Verified assertions on Windows |
|---|---|
| `trait_matrix` | RuntimeState/Arc and actual ID3D12Device satisfy Send+Sync; concrete test runtime/owner satisfy Send. Negative Send/Sync assertions compiled on actual Windows types, including local public Runtime/Upscaler. |
| `rc_and_arc_single_thread_controls` | Both retention families preserve create/destroy/state/device order; library stays loaded through dependency cleanup and is no longer observed loaded after final cleanup. |
| `runtime_and_siblings_cross_threads_with_stable_handles` | Joined A/B/C actions create two siblings, explicitly destroy the first and Drop the second; first handle address is unchanged, events occur on expected threads, runtime survives the first cleanup and releases after the second. |
| `returned_create_error_releases_inputs_without_destroy_on_either_thread` | Same-thread and transferred create error `u32::MAX` preserves Create/code, releases state/device without destroy, expires runtime weak reference and releases the DLL. |
| `null_success_retains_dependencies_on_either_thread` | Same-thread and transferred OK/null result reports NullContextOnSuccess and intentionally retains all dependencies and the loaded DLL without destroy. |
| `destroy_error_is_terminal_and_retains_dependencies_on_either_thread` | Same-thread and transferred destroy error `u32::MAX` preserves Destroy/code, records exactly one destroy and retains runtime/dependencies/DLL. |
| `unwinding_cleans_up_on_either_thread` | Intentional panic is caught/joined; both cleanup paths preserve event order, current/destination-thread cleanup, expired weak reference and final DLL release. |
| `runtime_without_context_can_release_on_another_thread` | Both local and transferred runtime-only release expire the weak reference and leave the fixture DLL no longer observed loaded. |

Library assertions use
`libloading::os::windows::Library::open_already_loaded` against the specific
fixture path; its temporary observation handle is immediately released.
Event assertions also require the DLL to remain loaded during callbacks and
state/device cleanup. This adds actual Windows loaded-module evidence to the
baseline's macOS ownership observer, without proving AMD DLL termination or
arbitrary OS memory-unmapping behavior. Null-success and destroy-failure cases
intentionally leak their retained fixture bundles under D005, rather than
inventing a cleanup retry.

**Verified by the workspace compiler harness:** Rc and Arc positive controls
compiled; consumed owner (E0382), private handle access (E0616), incompatible
retention family (E0308) and unaudited custom retention (E0277) were rejected
as expected. These are successful negative controls, not check failures.
The workspace also passed the existing fixture dispatch poison/terminal
retention checks; D013's eight probes themselves add no transferred dispatch
poison scenario.

Errors and diagnostic output:

- The initial missing-baseline error was resolved by the user supplying the
  final source state; preliminary checks are not counted as D013 evidence.
- Rust tool invocations emitted
  `warn: could not canonicalize path <user-home>` before otherwise successful
  output. Personal path normalized; origin not investigated. Clippy still
  exited 0 with `-D warnings` (this launcher diagnostic was not a denied
  compiler lint).
- `--nocapture` exposed two expected panic-hook messages at
  `crates/fsr-sdk/src/context/d013_movable.rs:461:13`, both reading
  `D013 intentional fixture unwind`, on the local test thread and the named
  `d013-unwind` thread. Both were handled by the test, which finished `ok`.
  Transient numeric thread IDs are omitted.
- No unexpected compile, link, DLL-load, test or lint failure occurred on the
  final D013 source. No raw pointer addresses were printed or retained;
  pointer equality was checked inside the probe. This record retains sanitized
  command/result observations rather than raw logs containing personal paths.

## Baseline delta and remaining limits

Windows fixture execution, formerly only compiled in the supplied record, is
now directly verified on this Windows x64/MSVC setup. The shared production
owner, real Windows loader, serialized fixture transfers, loaded-module
retention/release, error handling and unwind assertions pass. The requested
feature-enabled workspace checks and feature-disabled compilation also pass.

This evidence concerns Rust wrapper policy and a controlled fixture protocol,
not AMD ABI conformance or signed-provider transfer compatibility. No real COM
device participates in the D013 transfer probes; actual ID3D12Device types are
trait-checked, and other workspace tests use controlled COM fixtures. No
concurrent fixture native calls, native GPU work, native dispatch/thread hop,
native refactor regression, fence/device/debug-layer result, Miri or
ThreadSanitizer run is established here. A public movable family remains
unimplemented. The native regression and bounded fenced A/B/C transfer harness
described by the baseline remain follow-up work outside this request.
[D013](../../adr/D013.md) remains **proposed**; no architecture or native
transfer contract is accepted by this experiment. Topic synthesis and status
documents were not integrated or rewritten in this fixture-verification task.
