# Verification naming and dispatch inspection cleanup — 2026-10-15

## Change and boundaries

Current helper builds, Rust dynamic loads, C++ imports and residency checks now
agree on `gpu_helper.dll` / `gpu_helper.lib`. Seventeen `m6_*` / `d013_*` helper
exports now use `gpu_*`; the three existing `gpu_timing_*` exports are unchanged.
Signatures and equipment behaviour are unchanged. This is an internal test-helper
interface change, not a change to AMD's ABI or the public Rust API. Rebuild the
helper and its consumers together; old helper binaries are not supported by the
new test executables.

| Current control | Responsibility |
|---|---|
| `FSR_SDK_TEST_GPU_DLL` | Native GPU equipment library |
| `FSR_SDK_TEST_GPU_CONTROL` | No-provider GPU control |
| `FSR_SDK_TEST_CONTEXT_TRACE` | Native context destruction observations |
| `FSR_SDK_TEST_TRANSFER_OUTPUT` | Transfer cleanup readbacks |
| `FSR_SDK_TEST_SHARPENING_INTEGRATION_OUTPUT` | Public sharpening integration readbacks |
| `FSR_TRANSFER_FAILURE_CASE` | Isolated ownership/failure fixture scenario |

Worker/temp names and new provider/query/sharpening/transfer log labels likewise
describe their responsibilities. `audit_sharpening_integration` replaces the
milestone-named Python function. Auditors recognise frozen historical sharpening
and transfer labels without rewriting logs. Historical case IDs remain metadata;
explicit groups still select transfer baselines. The existing test-only numeric
descriptor sentinel `0xd013` is unchanged; it is a fixture value, not a selected
case name or an upstream API identifier.

The source comparison covers all 37 profile plans: case IDs, exact selectors,
order, prerequisites, watchdogs, payload counts and evaluation criteria match the
starting plans after the declared marker/control renames. The transfer profiles
still contain 27 and 58 cases. No acceptance threshold or workload was changed.
All 904 pre-existing files under research records, release records and the frozen
script archive were compared by SHA-256 and remain byte-identical.

`inspect_dispatch` now orchestrates named resource-identity,
resource-description/extent and device-membership checks. Scalar and command-list
checks still run first; descriptor construction still runs last. Canonical
resource references remain owned by the caller through the rest of inspection.
Device guards return in list-identity/context-device order to preserve their
previous release order, followed by the resource guards. No new COM queries,
allocations, public contracts or native calls were introduced by the extraction;
this is source reasoning, not a performance measurement.

## Local verification

Commands executed on macOS with cached dependencies:

```sh
python3.14 -B -m unittest discover -s tools/verification/tests -v
cargo test --workspace --features movable --locked --offline
cargo test --workspace --no-default-features --features movable --locked --offline
cargo clippy --workspace --all-targets --features movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings
cargo fmt --all -- --check
python3.14 -B tools/verification/archive/check.py
python3.14 -B tools/verification/repository_size.py
git diff --check
```

The infrastructure suite ran 184 tests: 179 passed; two real PowerShell parser
tests and three Linux/Proton tests were skipped for unavailable prerequisites.
Process-tree tests ran with permission for local sockets. New tests compare all
20 helper export names with Rust loads and C++ imports; that is source agreement,
not a compiled C++ ABI or DLL-loading result. Parser tests accept current and
historical 144-frame/78-comparison sharpening logs and the 30-case/516-frame
transfer matrix, and reject duplicate/changed/failed observations. Parser-only
sharpening fixtures mock image decoding; existing independent decoder tests still
run. No synthetic fixture is native execution evidence.

Host workspace tests, Clippy and formatting pass. Windows Rust cross-compilation
passes, including a new controlled inspection test with simultaneous failures:
command-list type, canonical identity, resource format/extent and device mismatch
must retain their existing precedence, restore COM reference counts and avoid a
native dispatch until corrected. That Windows test was compiled, not executed
on macOS. Archive integrity/Python/JSON syntax pass; real PowerShell parsing is
not run. Repository size and whitespace checks pass.

## Pending Windows/DX12 confirmation

The maintainer explicitly deferred native verification. Still required on Windows:
build/link the renamed helper and independent C++ references; run controlled COM
inspection tests; confirm dynamic symbol loading, debug/device checks and helper
residency/release; execute the existing public, transfer-cleanup and recording-ahead
regressions against rebuilt equipment. A Windows cross-check does not establish
any of those runtime results. No GPU, image-quality or performance experiment,
release publication or support expansion was performed by this cleanup.
