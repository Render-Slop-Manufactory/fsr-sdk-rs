# Experiment: Host-allocation failure during DX12 context creation

- Date: 2026-09-20.
- Question: What happens to the output handle and previously allocated host
  memory when a controlled allocation fails during `ffxCreateContext`?
- Observation targets: a returned error with immediate handle state and callback
  balance, or an isolated crash/timeout with the last observable callback events.
- Exclusions: production ABI/API changes, Context/RAII/Drop design, dispatch,
  GPU-resource accounting, device ownership, threading guarantees, null/double
  destroy, use-after-free and architectural decisions.

## Setup and method

Base revision: `450dd962194cadad68b199f30e60c3d58d580472`, initially clean worktree.
Changes are limited to the existing experiment's
[Rust probe](../../../crates/fsr-sdk-sys/tests/context_lifecycle.rs), new
[host allocator](../../../crates/fsr-sdk-sys/tests/context_lifecycle/allocator.rs),
[runner](../../../crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1), and documentation.
No production source, manifest, dependency, public ABI or accepted decision changed.

The prior [successful lifecycle spike](2026-09-20-exp-context-lifecycle.md)
supplies the same root -> version -> DX12 chain, flags 0, 1280x720 render size,
1920x1080 output size, null message callback, and retained device/chain/library.
Only the host allocation callbacks change. Existing Rust ABI assertions and the
paired local-header C++ check cover their signatures and layout; no ABI additions
were necessary. The complete paired native check was compiled again.

Environment: Windows 10.0.26200.0, Rust 1.98.1, x86_64-pc-windows-msvc,
VS 2022 Build Tools 17.14.28. AMD Radeon RX 9060 XT, vendor `0x1002`, device
`0x7590`, driver `32.0.31041.1004`, DX12 device feature level 12_0.
Successful controls identified provider `0xf5a5ca1e01001001`, name `4.1.1`.
The failed processes cannot query a successfully created context; identical
inputs do not independently prove their selected provider identity.

Local SDK v2.3.0 input SHA-256 fingerprints (not upstream authentication):

| Input under `Kits/FidelityFX/` | SHA-256 |
|---|---|
| `api/include/ffx_api.h` | `91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4` |
| `signedbin/amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `signedbin/amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

The allocator numbers attempts starting at 1. `FSR_SDK_FAIL_AT=0` disables
injection; N returns null on exactly attempt N. Other requests use Rust's global
allocator with a stored layout and 16-byte alignment for this Windows x64 target.
The native callback has no alignment argument. Zero-sized requests would reserve
one byte; none occurred. A preallocated bounded ledger records successful pointers,
layouts and individual free state. Reused addresses are matched to a live allocation.
Null frees are counted separately. Unknown/already-freed pointers abort without
performing an invalid deallocation; ledger exhaustion/poison also aborts.
Callback logging ignores I/O errors rather than panicking across FFI.
The mutex protects instrumentation only; it asserts no SDK threading contract.

The runner first requires a clean instrumented create/destroy control. It then
launches a fresh process for each of its observed allocation indices, plus one
unreached-index control. Each child has a 60-second timeout; stdout, stderr and
process status are saved under `target/context-lifecycle/failure-N.*.txt`.
Crashes are recorded and the sweep continues. Runner exit 0 means the sweep
completed, not that each child or native call succeeded.

If create returns an error, the probe reports its handle and ledger and exits
without destroy, manual frees, device release or Rust owner teardown. If create
succeeds with a non-null handle, it identifies the provider and destroys once
with the same compatible callbacks. OS process reclamation is not counted as SDK
cleanup. No error-return branch was reached in this run: all injections crashed.

## Documented contract

The authoritative local `api/include/ffx_api.h`, lines 123-148, permits a null
allocation result to signal failure, requires malloc-compatible alignment,
permits null deallocation arguments, and requires compatible callbacks for
destroy. A nonzero create return indicates an error. It does not specify a
universal failed-create output-handle state or full partial-allocation rollback.

The local `api/internal/ffx_api.cpp` initializes the output slot and delegates
creation, with conditional provider-object cleanup on a returned error. This is
source evidence, not proof that the signed runtime follows that path or that
every provider cleans up all partial state. No destroy-after-failed-create
contract was established.

## Observed runtime behavior

Every case began with a null context handle. The three callback sizes in the
successful controls were 1,099,864, 1,634,664 and 16 bytes, in that order.
Each successful create retained all three allocations until destroy. Destroy
freed them in order 2, 1, 3, each exactly once, and returned 0; the numeric handle
remained non-null afterward, as in the previous experiment.

| Failure index | Injection reached | Attempts | Successful allocations | Observed frees | Result | Handle after create |
|---|---|---:|---:|---:|---|---|
| 0 (disabled) | No | 3 | 3 | 0 after create; 3 after destroy | Create 0; destroy 0; process 0 | Non-null |
| 1 | Yes | 1 | 0 | 0 | Process `0xC0000005` | Unavailable: create never returned |
| 2 | Yes | 2 | 1 | 0 | Process `0xC0000005` | Unavailable: create never returned |
| 3 | Yes | 3 | 2 | 0 | Process `0xC0000005` | Unavailable: create never returned |
| 4 (beyond control count) | No | 3 | 3 | 0 after create; 3 after destroy | Create 0; destroy 0; process 0 | Non-null |

`0xC0000005` is Windows access violation (signed process exit `-1073741819`),
**not an FFX return code**. No injected case produced `ERROR_MEMORY` or any other
observable API return. No free/null-free event occurred before any crash. Thus
index 2 left one and index 3 left two successful callback allocations without an
observed matching free before termination. Index 1 had no successful callback
allocation to balance; that vacuous balance is not successful cleanup.
No post-create handle observation is available for a crashing call.

The sweep ran twice: first to obtain results, then to verify runner status-file
persistence and the stricter successful-control exit check. Both sweeps produced
the same counts, sizes and exit outcomes. No timeout or invalid-free guard fired.
The final sweep's [event log](2026-09-20-exp-create-allocation-failure/events.txt)
preserves pointer identities, order, summaries and process statuses. Numeric
addresses are process-specific observations, not reproduction inputs.

An initial runner launch failed before any probe process started because
Windows PowerShell `Start-Process` rejected duplicate inherited `Path`/`PATH`
entries. The final runner uses `ProcessStartInfo` with inherited environment,
redirected output and no window; no permanent environment change was made.

## Reproduction and validation

From the repository root in an x64 VS 2022 developer shell, with trusted local
SDK files and cached dependencies:

```powershell
powershell -NoProfile -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -FailureInjection
```

The actual invocation from an ordinary PowerShell session was:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -FailureInjection'
```

Runner: native ABI compile and device-helper build succeeded; the completed
sweeps returned 0, with the per-child failures recorded above. There are no
downloads, installation, distribution or permanent DLL-search changes.

The following checks all passed:

```powershell
cargo fmt --all -- --check
cargo build --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
cargo doc --workspace --no-deps --locked --offline
```

Ordinary tests passed five ABI tests, four loader fixture tests and two
compile-fail doctests. Four AMD runtime tests were ignored, including the new
failure probe. The existing toolchain home-path canonicalization warning remains;
no compilation, lint or ordinary test failure occurred.

## Inference and unresolved questions

**Inference:** this tested execution path does not turn the three permitted
host-allocation failures into recoverable FFX errors. It offers no observed
partial-state cleanup before the crash at indices 2 and 3. These observations
do not support a wrapper policy of attempting destroy after failed creation.

**Unresolved:** exact fault location and cause (no debugger/stack captured),
whether loader, bundled provider or driver implementation faults, behavior of
other providers/builds, and cleanup/handle postconditions when create actually
returns an error. The successful control identifies a provider version, not
which binary supplied its implementation. Callback accounting excludes GPU
resources, internal allocators and allocations outside these callbacks; it
cannot establish general leak freedom or total resource cleanup.

This adds isolated failure evidence beyond the successful lifecycle spike.
It establishes neither a general API contract nor an accepted ownership design.
