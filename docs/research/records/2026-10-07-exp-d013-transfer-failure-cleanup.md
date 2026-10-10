# Experiment: D013 transferred failure state and terminal cleanup

Investigation date: 2026-10-07 (Europe/Berlin).

Question: Does the existing wrapper core preserve dispatch eligibility, exact
errors, dependency ownership and terminal cleanup across serialized thread
transfers, including failure and Rust unwinding?

Scope: controlled, locally compiled fixtures; no AMD DLL, provider or GPU work.
This experiment does not accept D013 or implement a public movable family.

## Predeclared matrix and oracle

This matrix was written before the first experiment execution. Every new owner
scenario runs in an isolated test child with a 60-second watchdog and an exact
completion marker. Each has a control and transfer variant. A missing marker,
unexpected panic/exit, timeout, wrong event/thread/code/count, pointer change,
retry or premature dependency/library release is a failure. Expected Rust
panics must match the declared payload and end with a successful child exit.
Fixture exports, callbacks and dependency destructors must not unwind.

`create`, `inspect`, `dispatch`, `destroy`, `state-release`, `device-release`
are denoted Cr/I/P/D/S/V below. For transfer cases, loading/creation occur on
named A, rejected/native dispatch on B, and terminal cleanup on C. Joins order
exclusive moves. Controls use the child test thread for these actions.
The runtime's external owner is dropped immediately after creation.

| Cases | Expected ordered events and outcomes |
|---|---|
| known-poison / unknown-poison, control + transfer | Cr(A), I(B), P(B), D(C), S(C), V(C). Native error is Dispatch/code=1 (known ERROR) or 4294967295; later attempt is DispatchPoisoned, without inspection/FFI; one dispatch, one destroy; pointers stable, final runtime/DLL released. |
| preflight, control + transfer | Cr(A), I(B), I(B), I(C), P(C), D(C), S(C), V(C). CommandListType and CommandList/GetDevice HRESULT 0x80004005 preserved exactly, zero native calls on rejections, no poison; recovery dispatch returns 0. |
| destroy-explicit / destroy-drop, control + transfer | Cr(A), I(B), P(B), D(C). Dispatch error=1; rejection after transfer precedes inspection; destroy error=4294967295. One dispatch/destroy, no retry, state/device/runtime/DLL retained. Explicit destroy reports Destroy/code; Drop discards it. |
| unwind-ready / unwind-poisoned, control + transfer | Two Cr(A); poisoned variant adds I(B), P(B) returning 1. Deliberate Rust panic on destination B; D/S/V for first context run on B under family mutex. Surviving sibling waits on a channel, then recovers the poisoned mutex on named sibling-C, performs I/P returning 0 and D/S/V; runtime survives first cleanup, releases finally. Control destination is child test thread; sibling-C remains a coordinated worker. |
| worker missing file | Named worker returns Path error with owned io/NotFound source, no runtime and no acquired module. |
| worker missing each required export | Named worker returns Symbol error with exact ffxCreateContext, ffxDestroyContext, ffxConfigure, ffxQuery or ffxDispatch and owned libloading source. Module absent before load and after failure, no observer retained across cleanup; no runtime. |
| existing D013 and local controls | Repeat eight earlier probes, trait assertions and compiler rejection controls; local ownership/inspection/dispatch regressions pass unchanged. |

There are 14 owner child scenarios and six loader failure observations. Module
observations establish residency after calls, not a DLL termination callback's
executing thread. Retention children terminate without an invented cleanup.

## Setup and tested inputs

The initial worktree status was clean. HEAD observed during verification was
`1b97a92dec11924469e8c666b5ea55e99f2c0bff`, which includes the predeclared matrix.
The investigation used uncommitted changes described below. A separately supplied
source report appeared at the final status check and was preserved read-only.
No commit, push or publication was performed by this experiment.
The complete executed source is included in the repository; the revision is
supplementary rather than necessary evidence. [Input fingerprints](2026-10-07-exp-d013-transfer-failure-cleanup/inputs.json)
contain raw-byte and UTF-8/LF-normalized SHA-256 for both manifests, lockfile,
all wrapper/sys source, both fixture directories and the loader test. Fixture
DLLs were compiled afresh from these sources into exclusively created temporary
directories; transient binaries are not retained or independently fingerprinted.
The [original predeclared matrix](2026-10-07-exp-d013-transfer-failure-cleanup/predeclared-matrix.md)
is preserved with its initial pending-execution footer.

| Environment | Observed input |
|---|---|
| OS / architecture | Microsoft Windows NT 10.0.26300.0 / X64 |
| Shell | PowerShell 7.6.6 |
| Executed host/target | x86_64-pc-windows-msvc |
| Stable compiler | rustc 1.99.0, b940084d7eb6a299eb4bfeb8e34901bc051e7ac4, 2026-09-28; LLVM 23.1.1 |
| Stable Cargo | 1.99.0, 5f94df478, 2026-08-27 |
| MSRV compiler | rustc 1.88.0, 6b00bc3880198600130e1cf62b8f8a93494488cc, 2025-06-23; LLVM 20.1.5 |
| MSRV Cargo | 1.88.0, 873a06493, 2025-05-10 |
| Locked dependencies | libloading 0.9.0; windows 0.62.2; unchanged |
| SDK profile | v2.3.0, pinned 60f4ea81909200d8542eca14dccb2628b763a9a3; no SDK artefact was executed |

Normal Rust MSVC discovery compiled/linked actual Windows executables and Rust
DLLs. No installation, download or C++ ABI probe was needed. GPU/driver inputs
are irrelevant to this fixture scope and were not collected. The earlier D013
fixture/native records and D005/D008/D011/D013 supplied the execution baseline,
including the unresolved native provider-termination limitation.

The separately commissioned **D013 source record: serialized transfer and final
cleanup** (2026-10-07) became available during final review under the supplied,
untracked path `fsr-sdk-rs/docs/research/records/2026-10-07-src-d013-transfer-cleanup-contract.md`.
It was read after execution, not integrated or moved by this task; its fingerprint
is retained in [inputs.json](2026-10-07-exp-d013-transfer-failure-cleanup/inputs.json).
Intake note (2026-10-07): subsequent synthesis moved that report byte-for-byte
to the [canonical research record](2026-10-07-src-d013-transfer-cleanup-contract.md).
The original intake path and fingerprint above remain historical provenance.
Relevant supplied findings: it establishes no affirmative AMD transfer guarantee,
reports unsuccessful inspection of the requested pinned implementation, separates
module residency from termination/thread attribution, treats missing-export cleanup
as acquisition-local, and recommends controlled returned-error fixtures rather than
invalid native inputs. These reinforce this experiment's existing limits. Its
source references were not independently revalidated here. Its broader suggestions
(prior successful dispatch before poison, additional slot mutations, independently
acquired families/external fixture clients, fixture detach instrumentation, native
four-stage lifecycle and staged natural residency) were not added retrospectively
to the predeclared matrix and remain follow-ups. Its caution about ambiguous native
create-error ownership does not replace accepted D005's bounded returned-error RAII
trust assumption; the existing policy and controls were preserved.

## Changes and safety audit

Production change: [dispatch_policy.rs](../../../crates/fsr-sdk/src/dispatch_policy.rs)
contains the private shared eligibility/preflight/native-error/poison policy.
The existing [Upscaler dispatch core](../../../crates/fsr-sdk/src/upscaler/dispatch.rs)
delegates to it with its existing inspector and real owner invocation. A local
bool copy permits inspection to borrow the complete Upscaler without changing
its layout or private dispatch signature. Inspection can unwind before FFI
without changing readiness; the non-unwinding native return's transition is
written back before returning the error. There is no second copy of the state
machine. Terminal ownership and sealed retention are unchanged.

Test-only changes:

- [context.rs](../../../crates/fsr-sdk/src/context.rs) enables its existing
  private dispatch operation under `cfg(test)` as well as Windows/DX12.
  [test_runtime.rs](../../../crates/fsr-sdk/src/context/test_runtime.rs) resolves
  and forwards the fixture dispatch export when production loading is disabled.
- [d013_movable.rs](../../../crates/fsr-sdk/src/context/d013_movable.rs) adds an
  owned poison bool and optional family-coordination observation to the existing
  concrete bundle. Its sole narrow Send override still applies only to this
  fixture type. RuntimeState/library are Arc-retained; the native handle points
  into an owned stable boxed Input; its callback pointer targets another owned
  Box holding only Arc/Mutex observations and an integer context identifier.
  DeviceSpy owns an Arc and integer, not COM. The coordination observer owns
  an Arc/Mutex with no native dependency. No Rc, Cell-backed COM equipment,
  borrowed native dependency or movable generic owner is hidden in the bundle.
  The existing generic/local markers continue excluding Send/Sync; the concrete
  probe remains Send and not Sync.
- [d013_transfer.rs](../../../crates/fsr-sdk/src/context/d013_transfer.rs)
  exercises real shared dispatch policy and NativeContextOwner, with joined
  exclusive moves, stable handle/Input/callback identity assertions and exact
  event/thread/count/error checks. Descriptor storage for dispatch is stack-local
  and read only during the CPU call. Assertions and deliberate panics are outside
  fixture exports, callbacks and dependency destructors.
- [lifecycle.rs](../../../crates/fsr-sdk/tests/fixtures/lifecycle.rs) now accepts
  a fixture-specific header/code dispatch protocol and records each native call
  through the retained Input. There is no global fixture configuration. Each case
  owns its library, boxes and ledger, so ordinary parallel tests do not mix state.
- [child.rs](../../../crates/fsr-sdk/tests/fixtures/child.rs) launches the exact
  child test, captures output to a fresh directory, enforces exit/completion and
  kills/waits on a 60-second timeout. Its 10-ms polling is solely the watchdog;
  joins, channels and the shared mutex establish all test happens-before edges.
- [loader.rs tests](../../../crates/fsr-sdk-sys/tests/loader.rs) reuse the
  existing missing-file/export machinery on named acquiring workers. Assertions
  now check the source type, NotFound category or exact canonical path/symbol,
  plus absence of a loaded module before/after missing-export failure. The error
  is inspected on the worker; failure creates no RuntimeState or movable runtime.

During unwind, the family mutex guard is declared before the local owned context,
so reverse local destruction holds it through context Drop. Every subsequent
inspection/native callback/dependency-release event checks `try_lock` and records
`coordination_held=Some(true)`. A sibling owns the second context but waits on
zero-capacity channels. The parent joins/catches the exact declared panic, checks
remaining runtime ownership, then permits sibling calls. The sibling recovers
the poisoned mutex with `into_inner`; poison is not cleared or treated as another
panic. No sibling native calls overlap teardown. Successful final cleanup drops
state, device and runtime in D005 order. Failure paths retain those owned fields
and all their pointees; process isolation bounds intentional retention without
introducing a later retry or observer handle.

No dependency, manifest, public API, blanket trait override, D014 implementation
or architecture status changed. New project-maintained Rust files have MPL-2.0
SPDX headers.

## Commands and verification

Commands ran from `<repo>` with cached dependencies and default parallel test
scheduling. No `--ignored` or `--all-features` was used. Fixture compilation is:

```text
rustc --edition=2024 --crate-type=cdylib --crate-name=lifecycle_fixture <repo>/crates/fsr-sdk/tests/fixtures/lifecycle.rs -o <temp>/fsr-sdk-context-<process>-<sequence>/fixture.dll
rustc --edition=2024 --crate-type=cdylib --crate-name=loader_fixture <repo>/crates/fsr-sdk-sys/tests/fixtures/loader.rs -o <temp>/fsr-sdk-loader-<process>-<sequence>/loader_fixture.dll --cfg missing_<symbol>
```

The child invocation uses the current test executable, `--exact
context::d013_movable::transfer::isolated_case --nocapture`, and
`FSR_D013_FAILURE_CASE=<control-or-transfer>-<scenario>`. The test is a no-op
without that selector; its ordinary `ok` alone is not case evidence. The parent
requires all 14 completion markers and successful isolated child exits.

| Executed command | Exit / observation |
|---|---|
| `cargo test -p fsr-sdk --lib transfer_failure_cleanup_matrix --locked --offline -- --nocapture` | 0; first targeted attempt: all 14 children passed. |
| `cargo test -p fsr-sdk --lib d013_movable --locked --offline -- --nocapture` | 0; final source: eight existing probes, matrix parent and selector entry pass; all 14 actual children pass. [Full events](2026-10-07-exp-d013-transfer-failure-cleanup/d013-final.txt). |
| `cargo test -p fsr-sdk-sys --features dx12 --test loader --locked --offline -- --nocapture` | 0; five tests pass, one AMD test ignored; six new worker failure observations pass. [Loader observations](2026-10-07-exp-d013-transfer-failure-cleanup/loader-final.txt). |
| `cargo test -p fsr-sdk --lib d013_movable --no-default-features --locked --offline -- --nocapture` | 0; same eight controls and 14 children pass through the host adapter on Windows. [Adapter events](2026-10-07-exp-d013-transfer-failure-cleanup/adapter-tests.txt). This is not a Unix execution. |
| `cargo test --workspace --locked --offline` | First sandbox attempt: 101, compiler control cannot write metadata, 35 wrapper tests pass and one fails; unchanged-source rerun outside sandbox: 0. [Failure](2026-10-07-exp-d013-transfer-failure-cleanup/stable-tests.txt), [successful regression](2026-10-07-exp-d013-transfer-failure-cleanup/stable-tests-rerun.txt). |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/clippy-final.txt). |
| `cargo fmt --all -- --check` | 0, empty diagnostic output. |
| `cargo check --workspace --all-targets --no-default-features --locked --offline` | 0. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/feature-disabled.txt). |
| `$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --workspace --no-deps --locked --offline` | 0. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/rustdoc.txt). |
| `cargo +1.88.0 check --workspace --all-targets --locked --offline` | 0. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/msrv-check.txt). |
| `cargo +1.88.0 test --workspace --locked --offline` | First sandbox attempt: same metadata-write failure, exit 101; unchanged-source rerun outside sandbox: 0. [Failure](2026-10-07-exp-d013-transfer-failure-cleanup/msrv-tests-attempt-1.txt), [successful regression](2026-10-07-exp-d013-transfer-failure-cleanup/msrv-tests-rerun.txt). |
| `cargo +1.88.0 check --workspace --all-targets --no-default-features --locked --offline` | 0. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/msrv-feature-disabled.txt). |
| `cargo package --workspace --list --allow-dirty --locked --offline` | 0; new shared policy, transfer probe and child helper included. [List](2026-10-07-exp-d013-transfer-failure-cleanup/package-list.txt). |
| `cargo package --workspace --no-verify --allow-dirty --locked --offline` | 0; local archives only, no package-build verification or publication. [Log](2026-10-07-exp-d013-transfer-failure-cleanup/package-archive.txt). |
| `tar -tf target/package/fsr-sdk-0.1.0.crate`; corresponding sys archive | New files and loader fixture/tests, both LICENSE and LICENSE-AMD observed in their respective archives. |
| `git diff --check` | 0. |

Stable and MSRV successful workspace runs each execute 36 wrapper unit tests,
one wrapper dimensions integration test, 13 sys tests (six ABI, two dispatch ABI,
five loader) and four doctests, with zero failures. Nineteen AMD/hardware/research
tests remain deliberately ignored (13 wrapper, six sys). Each workspace matrix
parent also executes all 14 child scenarios. Existing local-family inspection,
unknown dispatch code, preflight/poison, poison/destroy-retention, construction,
ownership and directory-hygiene checks pass.

The owner compiler harness passes Rc/Arc positive controls and rejects consuming
reuse (E0382), private handle access (E0616), mixing Rc/Arc ownership (E0308) and
arbitrary retention implementation (E0277). The actual Windows Runtime/Upscaler
negative Send/Sync assertions and concrete probe positive Send/negative Sync
assertions compile on both stable and MSRV.

## Observed case results

All events match the predeclared matrix, including the asserted executing
ThreadIds. Logs normalize them to `actor=0,1,2` in order of first appearance:
transfer cases map to A/B/C; single-thread poison/preflight controls use actor 0;
unwind controls use actor 0 for creation/panic/first cleanup and actor 1 for the
surviving sibling. Thread IDs and personal paths are normalized; pointer addresses
are not emitted, and handle/Input/callback equality is asserted directly.

| Scenario (both control and transfer passed) | Native dispatch / destroy counts | Exact result and lifetime observation |
|---|---|---|
| known-poison | 1 / 1 | Dispatch/1, then DispatchPoisoned without a second inspection or dispatch; successful destroy/0; S then V release, runtime weak expires, DLL absent. |
| unknown-poison | 1 / 1 | Dispatch/4294967295 lossless, then DispatchPoisoned without inspection/FFI; destroy/0, complete release. |
| preflight | 0 for two rejects, then 1 / 1 | InvalidDispatch/CommandListType and Dx12Inspection/CommandList/GetDevice/-2147467259 (bits 0x80004005) preserved exactly; ready remains ready; recovery/0, destroy/0, complete release. |
| destroy-explicit | 1 / 1 | Dispatch/1; poison rejection; consuming destroy returns Destroy/4294967295, later Rust Drop makes no second native call; no S/V releases; runtime strong count 1, ledger strong count 3, DLL live. |
| destroy-drop | 1 / 1 | Same dispatch poison and retained-bundle observations; automatic Drop discards destroy code 4294967295; no retry or S/V releases. |
| unwind-ready | 0 first + 1 sibling / 2 | Exact Rust panic caught/joined; first D/S/V under mutex on destination; sibling recovery dispatch/0 and D/S/V; library live through each event, weak survives first cleanup and expires after sibling cleanup. |
| unwind-poisoned | 1 first + 1 sibling / 2 | First Dispatch/1; exact panic and coordinated poisoned cleanup as above; sibling remains usable, dispatch/0; final complete release. |

Each poison attempt checks the exact first error and flag on B, then stable
pointers and DispatchPoisoned on C. The complete event sequence excludes new I/P
events for rejection, and excludes extra D events after consuming destruction or
automatic Drop. All native callbacks and dependency-release observations find the
library alive. Exceptional-retention strong counts establish the owned descriptor
state/callback, device surrogate and runtime references remain; no observer pins
the library. Fresh process isolation prevents those retained bundles contaminating
another scenario's unload observation.

The worker loader observations match Path/io/NotFound and all five exact Symbol
errors with a retained libloading source. Each worker's name is asserted. Each
missing-export module is absent before acquisition and after its failure. The
temporary `open_already_loaded` observation is immediately released and no such
handle spans cleanup. There is no transfer of partially initialized loading state.
The fixture returns sentinel forwarding codes 101–105 in its separate existing
all-arguments control, which also passed.

## Failed attempts, qualifications and remaining questions

The stable and MSRV sandbox workspace attempts failed the existing positive
compiler control at `context/tests.rs:403`: writing
`<temp>/fsr-sdk-context-<process>-<case-dir>/libpositive.rmeta` returned access
denied (`os error 5`). Both attempts are preserved as failures, not counted as
passing regression evidence. Repeating the identical commands with approved
execution outside the sandbox passed that control and the complete workspace.
No experiment oracle, production code or compiler control was weakened to pass.
The first successful matrix was subsequently strengthened with observed mutex
coverage and callback-pointer checks, and rerun successfully on the final source.
The initial loader run used an extra child wrapper; final tests simply reuse the
existing acquisition machinery directly on workers, avoiding cross-crate helper
dependencies. Its final worker run and both full regressions passed.

Sandbox builds also emitted incremental-cache hard-link warnings and copied cache
files instead. Clippy with denied warnings still exited 0; these infrastructure
diagnostics are preserved in the logs and were not source lint failures. The
expected panic hooks appear twice in the older D013 unwind controls and once in
each of the four new unwind children. Their declared oracles handled them; no
unexplained crash, child timeout, missing completion or unexpected FFI unwind was
observed.

**Verified fixture result:** the real shared dispatch policy and terminal owner
preserve the requested transferred state/errors/cleanup under the tested
serialized Rust ownership arrangements. No wrapper contract defect was found.
The delta from the earlier eight D013 probes is transferred known/unknown native
dispatch poison, transferred exact preflight failures and recovery, poison combined
with explicit/automatic destroy failure, coordinated ready/poisoned unwind with a
surviving sibling, and acquiring-worker loader failures. Existing create-failure,
OK/null, terminal-retention, sibling and trait controls were reused and passed.

The fixture dispatch protocol is not an AMD descriptor and the injected COM error
is an Error value from the inspection seam, not a COM method running on transferred
objects. Actual local COM inspection regressions passed separately. This spike
does not audit a real movable DX12 resource/dependency bundle, verify AMD affinity,
GPU safety, provider rollback/quiescence, provider/device termination on destination
threads, concurrent calls, unfenced work, Miri or ThreadSanitizer. The no-default
feature run exercises the host adapter on Windows; no Linux/macOS run is claimed.
Windows residency observations do not establish a DLL termination callback's
executing thread. Native error/unwind behavior and the earlier provider-termination
limitation remain questions for a separately authorized native spike and D013
contract review. No public Send support or D013 acceptance follows from these
controlled results. D014 remains a separate task, and topic synthesis/ADR status
was not changed by this experiment.
