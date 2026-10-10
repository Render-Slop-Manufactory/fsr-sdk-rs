# Experiment: D013 traits and serialized host lifecycles

- Investigation date: 2026-09-29
- Question: Can the existing private context owner retain either `Rc` or `Arc`
  without changing its terminal cleanup policy, and can a concrete fixture-only
  `Send + !Sync` wrapper preserve ownership across serialized thread transfers?
- Success: the positive/negative trait assertions compile; incompatible retention
  families and unaudited retention implementations fail compilation; ordered
  fixture cleanup and library retention match the same-thread controls.
- Failure: an unexpected trait, accepted incompatible retention, changed pointer,
  premature release, wrong cleanup order, repeated destruction or failed control.
- Exclusions: public movable API, AMD thread compatibility, GPU work, concurrent
  native calls, native dispatch poisoning and partial-loader failure handling.

## Setup and tested source

This is post-publication work on the released `0.1.0` baseline, not a frozen
release candidate. The experiment started from source at `6814225`; during the
run, documentation-only release updates advanced HEAD to
`99dc04a60a87cd8dddd432776d9fbf26abbaaf43`. No production runtime/upscaler/loader
source, dependencies, features or versions changed outside the edits below.
The already proposed D013 and decision-index entry were uncommitted inputs.

The probe uses repository tests, without a separate crate or copied owner:

- [context.rs](../../../crates/fsr-sdk/src/context.rs): the real private owner
  gains a sealed `RuntimeRetention` parameter, defaulting to `Rc<RuntimeState>`;
  only `Rc` and `Arc` implement it. Native operations and terminal cleanup remain
  shared. This is the production-source refactor; public users still use `Rc`.
- [d013_movable.rs](../../../crates/fsr-sdk/src/context/d013_movable.rs): eight
  test-only probes, using that owner and the existing fixture. The sole new
  `unsafe impl Send` applies to a concrete fixture wrapper. The generic owner
  retains its `Rc<()>` marker and remains `!Send + !Sync`.
- [test_runtime.rs](../../../crates/fsr-sdk/src/context/test_runtime.rs): the
  host-only release observer changes from `Rc<Cell<bool>>` to
  `Arc<AtomicBool>`, so the observer itself does not impose a false local trait.
- [tests.rs](../../../crates/fsr-sdk/src/context/tests.rs): reuse of the fixture
  builder, the corresponding observer change, and compilation controls for Arc,
  family mismatch and sealed retention.
- [lifecycle.rs](../../../crates/fsr-sdk/tests/fixtures/lifecycle.rs): unchanged
  Rust `cdylib`, with stable boxed input/callback pointers and injected return
  codes. Its protocol is a test protocol, not an AMD ABI conformance test.

Environment: macOS 27.0.1, ARM64 (`aarch64-apple-darwin`); Rust 1.98.1
(`48a229cea`, 2026-09-01), Cargo 1.98.1 (`797e8a9bc`), LLVM 22.1.8. The installed
`x86_64-pc-windows-msvc` standard library enables cross-checking. Locked inputs
include `libloading` 0.9.0 and `windows` 0.62.2. SDK profile remains v2.3.0,
but no AMD DLL, provider, GPU or driver was used in this experiment.

SHA-256 identifies the actual tested inputs, including local edits:

| Repository path | SHA-256 |
|---|---|
| `Cargo.lock` | `9bfce6f10c4c9cd4f79afb03e4c88c8d11f89aa6fa11dfe3a8475beb9329caec` |
| `crates/fsr-sdk/src/context.rs` | `712bfa9d89af4396477499b5d0d6e891d41665cec7aefc8ba92fdd42bdc09951` |
| `crates/fsr-sdk/src/context/test_runtime.rs` | `f90f5b5130f33f37b7ea3acb9d817a7d4bad62ee72c5db3fd64489be3bf203c4` |
| `crates/fsr-sdk/src/context/tests.rs` | `f55788bfa085828f03901d2c0defb79ee87191f93769ea5a0e7fc1c247b10057` |
| `crates/fsr-sdk/src/context/d013_movable.rs` | `c228605b2f5d2151d3aaed14d46f183ded80901a91424919fec39279376852ac` |
| `crates/fsr-sdk/tests/fixtures/lifecycle.rs` | `41eef79a0c5c91d36ba06e02671136bf349d8c7718aeb4a533d7a26942f2d101` |
| `crates/fsr-sdk/src/runtime.rs` | `e8cd070968b693fdbc363f6319302d0df51e4388e073499eb1829320e994e08d` |
| `crates/fsr-sdk/src/upscaler.rs` | `9c30018c25b27e63cbd06d6916f8c17b835b56353d21f5db01f8a3eba35e7e5c` |
| `crates/fsr-sdk-sys/src/loader.rs` | `6a17b88814e83e16de5fa6b5ae653338f0b584fd59c7ef883e0a9cd3abf7621a` |

## Reproduction and results

Run from the repository root with the locked dependencies cached and the Windows
target already installed. The fixture helper invokes the host `rustc` to build
temporary shared libraries; Rust must be on PATH. No SDK acquisition is needed.

```sh
cargo fmt --all -- --check
cargo test -p fsr-sdk --lib d013_movable --locked --offline
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --no-default-features --locked --offline
cargo check --workspace --all-targets --no-default-features --target x86_64-pc-windows-msvc --locked --offline
```

All final commands exited 0. The isolated D013 run passed eight tests. The final
workspace run passed 18 wrapper tests (the previous ten plus eight), five sys
tests and two doctests. The host doctests omit their Windows-only bodies.
Clippy passed on host and Windows with warnings denied. Windows checks compiled
the real Windows/DX12 types and test assertions; they did not execute Windows
tests or link/run a Windows binary. Feature-disabled checks also passed.

### Compile observations

**Verified by compilation**, with actual production runtime/device types on the
Windows target and the host fixture adapter on macOS:

| Type | Send | Sync |
|---|---|---|
| `RuntimeState`, `Arc<RuntimeState>` | yes | yes |
| `Rc<RuntimeState>` | no | no |
| `*mut c_void`, pointer-bearing fixture state | no | no |
| Arc-retained private owner, with original local marker | no | no |
| Test runtime with `PhantomData<Cell<()>>` | yes | no |
| Concrete test owner with the audited manual implementation | yes | no |
| Actual `ID3D12Device` (Windows) | yes | yes |
| Actual public `Runtime`, `Upscaler` (Windows) | no | no |

Negative traits use ambiguity assertions on the actual types. The existing
compiler harness includes the real `context.rs` with stub library methods and
ABI types: Rc and Arc controls compile, mixing owner families fails with E0308,
and implementing retention for a custom wrapper fails with E0277. Existing
consumption/private-handle checks still produce E0382/E0616. This harness checks
type contracts, not native ABI or public movable families, which do not exist.

An initial probe compilation failed with E0277 (exit 101): direct `owner.0`
access inside a closure made Rust capture the inner, non-Send owner instead of
the Send wrapper. Methods on the concrete wrapper preserve capture of the
audited type. No broad Send implementation was added to bypass the failure.

### Executed host observations

**Verified on the fixture**, using joined, named threads to order each handoff:

- Rc and Arc same-thread controls preserve create/destroy/state-drop/device-drop
  order. Dropping the external runtime first does not release its library while
  either context survives.
- The runtime moves to A, which creates two siblings. B consumes the first
  context after dropping the external runtime; C drops the second. The native
  handle still points to the same retained allocation after transfer. Recorded
  callbacks and dependency drops occur on the expected thread; the runtime's
  weak reference expires only after final cleanup on C.
- Returned create error with a non-null output releases inputs without calling
  destroy. Successful null creation retains the whole bundle. Destroy failure
  retains the bundle and calls destroy once. Both ordinary and transferred
  executions preserve these D005 outcomes.
- Intentional unwinding cleans up successfully on the current or destination
  thread. A runtime without contexts also releases after a thread transfer.

The event oracle checks all callbacks and dependency drops while the library is
retained. On macOS, the atomic observer marks entry into library-wrapper Drop;
after a joined action, the weak reference and observer establish completed Rust
ownership release. It does not independently measure OS unmapping. On Windows,
the same tests use the existing loaded-module observation, but were only
compiled here. Exceptional retention intentionally leaks fixture allocations
and library ownership, as D005 requires; there is no cleanup retry.

## Baseline delta, limits and next question

The earlier source-derived trait claims now have compile evidence, and serialized
Arc fixture lifecycles work through the real private cleanup core. A narrow Send
implementation is sufficient for this concrete fixture shape. That does not
audit or prove movability of real upscaler descriptors, COM dependencies or AMD
provider state. The public API and local-family traits remain unchanged.

Miri and ThreadSanitizer were not run. The installed nightly lacks Miri, and
these fixtures dynamically load separately built libraries. A Miri-compatible
pure-Rust fixture and a consistently instrumented sanitizer build would be
separate work; ordinary passing tests do not establish absence of data races.
No DLL partial-load failure, AMD thread hop, concurrent sibling call, dispatch
poison path, native regression or GPU fence was exercised by this probe.

The remaining native question needs a bounded Windows/DX12 harness on the
recorded AMD deployment: serialized load/create on A, fenced dispatch on B,
destroy/final release on C, plus same-thread controls, siblings, multi-frame
work, debug-layer/device checks and watchdog isolation. That harness is not
implemented by this record. New native evidence must cover the shared-core
refactor before claiming native regression validation. Even a successful run
would support only its identified configuration. [D013](../../adr/D013.md)
remains proposed; this experiment accepts no native transfer contract.
