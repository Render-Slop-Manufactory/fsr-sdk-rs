# D013 public API implementation verification — 2026-10-09

The accepted [D013](../adr/D013.md) now has an optional public implementation in
current source. This record verifies that implementation and selected existing
native scenarios; it is not another architecture decision or a new exploratory
thread-compatibility study. Nothing is published, no release version is selected,
and D014 / `0.2.0` scope remain unchanged.

## Implemented boundary and field audit

- `movable` is opt-in and does not enable `dx12`. Windows + `dx12` + `movable`
  exposes `runtime::MovableRuntime` and `upscaler::MovableUpscaler`.
- Unsafe runtime acquisition establishes the complete lasting contract in
  [public Rustdoc source](../../crates/fsr-sdk/src/runtime/movable.rs).
  Construction takes only the matching runtime. No public conversions, generic
  owners, native handles, Clone or internal synchronization are introduced.
- Both movable types are `Send + !Sync`; both existing local types remain
  `!Send + !Sync`. The runtime derives Send through `Arc<RuntimeState>` and uses
  `PhantomData<Cell<()>>` to exclude Sync. `RuntimeState` holds only the existing
  private `FfxLibrary` function pointers and `libloading::Library`; local ownership
  retains Rc. No generic owner or dependency bundle gains an unsafe trait impl.
- The sole production manual Send implementation is on the exact
  [MovableUpscaler](../../crates/fsr-sdk/src/upscaler/movable.rs) field graph. Its
  private sealed `UpscalerCore<Arc<RuntimeState>>` contains one native owner, pinned
  creation descriptors, an independent owned `ID3D12Device` reference, checked
  dimension options and a poison bool. The pinned allocation preserves the
  root/version/backend pointers across moves. No real Rc dependency, callbacks,
  custom allocator or borrowed dispatch input is retained in this bundle. The
  generic owner's phantom Rc marker contains no actual Rc ownership; it remains
  confined unless enclosed by this specifically audited public type.
- Construction, inspection, dispatch, poison and terminal cleanup share the
  private core and existing owner. Successful cleanup releases pinned state,
  device and runtime in order. Destruction is attempted once; destroy error and
  OK/null creation retain the full critical bundle. Returned create errors retain
  D005's bounded RAII trust policy. Dispatch remains `unsafe` with `&mut self`,
  including independent D011 GPU, recording and abandonment obligations.

The [consumer example](../../crates/fsr-sdk/examples/movable_upscale.rs) wraps each
independently transferable context with the same application mutex domain. Its
Drop acquires that domain and removes the raw context before releasing the guard,
including during unwinding and destruction of a failed channel send's returned
owner. Creation declares the guard before native owners and drops the public
runtime early. Poison recovery is permitted only under the stated preserved
invariant. The consumer must still manage GPU completion, other native users and
non-reentrancy; dropping a wrapper while already holding its gate would deadlock.
The library adds no scheduler or built-in mutex.

## Inputs and reproducibility

Run from the repository root on Windows x64/MSVC with cached Cargo dependencies
and the separately acquired trusted SDK staged at
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX`. There is no SDK download in Cargo.
The runner invokes the existing pinned-header ABI checks and builds the existing
`gpu.cpp` / `device.cpp` equipment. Native cases each get a fresh process, a
180-second parent watchdog and 60-second GPU fence timeout. GBV is off; the
ordinary D3D12 debug layer is on. All six selected native cases exited zero with
their required success marker and without a watchdog timeout.

| Input | Recorded value |
|---|---|
| OS / target | Windows 11 Pro 10.0.26300, `x86_64-pc-windows-msvc` |
| Rust | 1.99.0 `b940084d7` (2026-09-28); minimum-version checks use 1.88.0 |
| Native compiler / SDK | MSVC 19.44.35224.0, toolset 14.44.35207, Windows SDK 10.0.22621.0 |
| Shell | PowerShell 7.6.6 |
| Native SDK / provider | SDK v2.3.0; provider `4.1.1`, ID `0xf5a5ca1e01001001` on both public movable contexts |
| Hardware / driver | AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, feature level 12_0; driver 32.0.31041.1004 |
| Signed loader SHA-256 | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| Signed upscaler SHA-256 | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

Authenticode status was Valid for both AMD DLLs. The
[source manifest](2026-10-09-d013-public-api-verification/source-inputs.json)
captures all crate files, helpers, manifests and lockfile at the final native
run. [Native inputs](2026-10-09-d013-public-api-verification/native-inputs.json)
capture header and executable/helper fingerprints. The
[environment](2026-10-09-d013-public-api-verification/native.environment.txt)
records the exact toolchain, DLL hashes and deployment variables. These relative
file manifests and the checked-in sources make the result independent of Git
history or an external CI page.

After that run only the package README changed among captured crate files;
[old/new hashes](2026-10-09-d013-public-api-verification/post-native-documentation.json)
record this documentation delta. Rust and native runner source did not change.
[Supplemental fingerprints](2026-10-09-d013-public-api-verification/supplemental-source-inputs.json)
identify the consumer-control script, ABI script and CI configuration.
Evidence excludes binaries and personal paths; the declared unwind's OS thread
ID is redacted, while Rust thread IDs and native codes are preserved.

## Commands and results

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass, Windows feature-enabled targets/examples |
| `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings` | Pass |
| `cargo test --workspace --features fsr-sdk/movable --locked --offline` | Pass: wrapper 39 unit tests, dimensions 1, public movable integration 6, raw ABI/loader 13 and 6 doctests; hardware tests remain ignored |
| `cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline` | Pass: wrapper 21, raw ABI 8, wrapper doctests 2; public Windows/DX12 API absent |
| `cargo check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo +1.88.0 check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo +1.88.0 test --workspace --features fsr-sdk/movable --locked --offline` | Pass; then the final acquisition test was added and the final six integration tests rerun with the next command |
| `cargo +1.88.0 test -p fsr-sdk --test movable --features movable --locked --offline` | Pass, all 6 public integration tests |
| `cargo doc --workspace --features fsr-sdk/movable --no-deps --locked --offline` with `RUSTDOCFLAGS=-D warnings` | Pass |
| `./tools/check-movable-api.ps1` | Pass, 2 positive and 6 expected-failure downstream consumers |
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode movable -Native -Stage target/d013-public-api/attempt-2` | Pass, default checks, three pinned ABI probes, helper build and all 6 native children |
| `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` and `cargo package -p fsr-sdk --allow-dirty --no-verify --locked --offline` | Pass; local archive inspected, license notices and new public/example/test sources match; no package-build verification or publication |

The native runner's [command statuses](2026-10-09-d013-public-api-verification/checks.status.txt)
also preserve default-feature fmt/Clippy/tests and no-default compilation.
[Package audit](2026-10-09-d013-public-api-verification/package-audit.json)
identifies the local archive and inspected entries. Packaging uses the existing
0.1.0 manifest number; this is no new 0.1.0 release or release-candidate claim.
CI adds feature-enabled checks and Windows downstream controls; remote CI and a
new non-Windows execution were not run here. Compiler subprocess fixtures use
the installed default `rustc` (1.99.0), even when Cargo compiles the crates with
1.88.0, as in the existing fixture design.

One initial sandboxed workspace test failed because the compiler-control test
could not write a `.rmeta` file in Temp (AccessDenied). The authorized run outside
the sandbox passed. An initial Clippy check caught an unused native-test loop
index; it was removed before the successful final checks. Incremental cache
hard-link warnings fell back to copies. The first native run passed before the
final weak-runtime assertion and destroy tracing were added; the complete
selected native run above was repeated successfully on that final source.
No failed native gate or timeout is hidden by this report.

## Public API and failure coverage

[Public integration tests](../../crates/fsr-sdk/tests/movable.rs) import exactly
the consumer example and compile it against public types. Trait controls assert
local !Send/!Sync, movable Send/!Sync and absence of Clone. A negative trait's
unexpected implementation causes ambiguous trait resolution at compile time.
The empty acquisition path is rejected before DLL initialization with the
existing path error. Three fresh watchdog-bounded DLL-fixture children cover:

1. Early public-runtime/device release, failed ownership send followed by Drop
   of the returned owner, survivor retention and final Drop during caught unwind.
2. Runtime transfer before context creation, independent siblings and ordered
   explicit/implicit cleanup on further threads.
3. An application panic between native operations poisoning the shared gate,
   then coordinated sibling cleanup on two workers with recovery.

The ownership-only COM stub uses atomic reference counts. After successful final
cleanup, all test-owned references reach zero and the loader fixture is absent
according to `GetModuleHandleW`, which adds no module reference. Fixture destroy
re-reads the retained root/version/backend chain, testing pinned-address validity
after actual owner movement. These are controlled fixture observations, not AMD
failure/driver evidence.

[Public movable failure fixtures](../../crates/fsr-sdk/src/upscaler/tests/movable.rs)
verify rejection before native dispatch and recovery, successful dispatch,
preservation of an unknown native error, poison rejection before inspection,
returned-create failure, OK/null exceptional retention and explicit/implicit
destroy error without retry. Existing Rc/Arc private-owner and local public
failure/ownership tests also pass through the shared-core refactor.

[Downstream controls](../../tools/check-movable-api.ps1) require a successful
local baseline and successful movable Send baseline before accepting six exact
diagnostic categories: missing `movable` or `dx12` gives E0432, crossed-family
constructors E0308, and absent runtime conversions E0277. Each retained
`consumer-*.txt` file contains the result. Thus compile-fail success is not
inferred from an unrelated compiler or dependency failure.

## Selected native results

| Child | Observation / retained evidence |
|---|---|
| Local explicit lifecycle | Two siblings and early public-runtime release; final loader absence. [stderr](2026-10-09-d013-public-api-verification/native-explicit.stderr.txt) |
| Local Drop lifecycle | Same lifecycle with implicit final destruction. [stderr](2026-10-09-d013-public-api-verification/native-drop.stderr.txt) |
| Local single-frame dispatch | Public dispatch, submission, fence/readback and explicit destroy. [stderr](2026-10-09-d013-public-api-verification/gate-a.stderr.txt) |
| Local temporal smoke | Three fenced moving frames through one local context. [stderr](2026-10-09-d013-public-api-verification/gate-b.stderr.txt) |
| Local semantic comparisons | Existing motion/jitter/reset checks remain successful. [stderr](2026-10-09-d013-public-api-verification/gate-c.stderr.txt) |
| Public movable family | Two fenced frames, separately held sibling owners, early runtime release, unwind and failed-handoff destination Drop. [stderr](2026-10-09-d013-public-api-verification/public-movable.stderr.txt), [stdout](2026-10-09-d013-public-api-verification/public-movable.stdout.txt), [status](2026-10-09-d013-public-api-verification/public-movable.status.txt) |

The [actual public-family test](../../crates/fsr-sdk/tests/native/movable_public.rs)
loads on worker ThreadId(3), creates both contexts on ThreadId(4), and dispatches
on two independent workers under one caller-owned mutex domain. The renderer
thread retains helper equipment, submits and fences each frame before permitting
context/list/resource ownership to leave the dispatch workers. A separate worker
destroys the first context during a declared caught Rust panic outside FFI:
ThreadId(7), `unwinding=true`, native destroy code zero. Drop of the failed
handoff's returned owner destroys the final sibling on ThreadId(8), native code
zero. The weak runtime observer then cannot upgrade, proving release of the last
strong runtime reference without adding an observer owner. Equipment and borrowed
COM inputs are released only afterward, under the same domain.

Both public movable readbacks are 640×360, with 230400 finite and changed pixels,
red range 0.1005249–0.84765625, debug errors zero and device removal zero. Device
checks before and after equipment release also report zero errors/removal.
Provider identity queries use existing private test instrumentation; context
construction, dispatch, movement and cleanup use the actual public family.
The consumer integration tests separately prove coordination using only public
API access. No private test-only Send owner substitutes for MovableUpscaler.

## Limits and status

No implementation soundness blocker was found. D013 stays **accepted**; its
optional implementation and bounded public verification milestone are complete.
The full caller safety contract remains authoritative in Rustdoc / the ADR.
Native arbitrary-thread compatibility, internal worker/termination behavior,
returned-error trust and effective coordination of all potentially shared users
remain caller-established assumptions, even for these tested binaries.

The checks do not establish universal AMD guarantees, provider/dependency unload
or its executing thread, native partial-load/returned-error recovery, concurrent
use, transfers with outstanding GPU work, other hardware/provider configurations,
image quality or performance. There is no GBV, Miri or ThreadSanitizer result.
No new native experiment or general research phase was conducted. A selected
future release still needs independent artifact/release verification.

## Subsequent CI exit-status correction — 2026-10-09

CI reported eight passing consumer controls but process exit 1: the script left
the last expected compiler failure in PowerShell's exit status. The original
local verification checked each control but did not check the CI shell wrapper.
The script now explicitly exits zero only after all controls have been validated;
earlier throws still fail. The retained supplemental fingerprint describes the
original script before this correction; Rust/native inputs are unchanged.

The following local invocation now passes all eight real controls and exits zero:

```powershell
pwsh -NoProfile -Command '$ErrorActionPreference = "Stop"; ./tools/check-movable-api.ps1; if (Test-Path -LiteralPath variable:\LASTEXITCODE) { exit $LASTEXITCODE }'
```

Three separate subprocess checks substitute a PowerShell `cargo` function to
simulate a failing positive control, unexpected negative-control success, and
exit 101 with a wrong `error[E0000]` diagnostic after two successful baselines.
All three still throw at the corresponding validation and exit 1. No GitHub CI
rerun is claimed here.
