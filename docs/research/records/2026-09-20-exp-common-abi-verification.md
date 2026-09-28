# Experiment: Does the first Rust ABI slice match the local SDK header?

- Investigation and recording date: 2026-09-20.
- Method: retrospective record of the header inspection and checks performed
  during the first ABI implementation in this task. Results below come from those
  tool outputs; recording this evidence did not rerun the checks.
- Question: Do the selected Rust declarations and the local AMD declarations
  satisfy the same layout, constant and signature expectations on Windows x64?
- Success: Rust tests and the native compile-only assertions pass. Failure:
  either side rejects an expectation. This does not test a call across the FFI.
- Exclusions: loading, symbol resolution, runtime queries, lifecycle, dispatch,
  GPU behavior, extended descriptors and selection of a full-SDK binding strategy.

## Setup and inputs

Repository HEAD was `1d8e4ea59ad638cae13234cc344f59b5ebc2989e`.
The tested implementation was **uncommitted** on top of that revision; HEAD alone
does not reproduce it. Relevant files:

- [Production declarations](../../../crates/fsr-sdk-sys/src/api.rs).
- [Rust regression checks](../../../crates/fsr-sdk-sys/tests/abi.rs).
- [Native compile-only probe](../../../crates/fsr-sdk-sys/tests/native_abi.cpp).

Environment: Windows, Rust 1.98.1 (`x86_64-pc-windows-msvc`, rustc commit
`48a229ceaefd4985c50990b14116b6d856af0985`), Cargo 1.98.1, VS 2022 Build Tools
developer environment 17.14.28, x64 tools. Exact `cl` compiler version and Windows
build were not captured. GPU and driver are irrelevant to these compile/layout
checks and were not inspected.

Authoritative input: local SDK directory `external/FidelityFX-SDK/v2.3.0/`,
specifically [ffx_api.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h).
The local SDK is ignored by Git and must be supplied separately. D002 identifies
the intended upstream release; this check did not authenticate the local tree
against that upstream commit. SHA-256 of the local header at recording time:
`91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4`.

SHA-256 fingerprints of the uncommitted tested files at recording time (raw
bytes, sensitive to line-ending normalization):

| File | SHA-256 |
|---|---|
| `src/api.rs` | `AE1611EE06BFF33B93EF682327386515BDC1B472EC7B178EC6FA38F7C6FB794F` |
| `tests/abi.rs` | `1E806ABC0D0A5C19BF6F8D20C0CF38D19D7B05D977EA75D00AC187FE10E9B69B` |
| `tests/native_abi.cpp` | `B21CFAE5DCB5F4FC7992AED22FBEBC6E912826C546EE780FA22C2AFA598CC989` |

Paths in this table are relative to `crates/fsr-sdk-sys/`. Other changes included
status/decision documentation, retention of AMD's header notice and formatting
of empty placeholder modules. Manifests and Cargo.lock were unchanged.

## Header findings

**Verified source facts**, from the named declarations in `ffx_api.h`:

- `ffxContext` is `void*`; all five entry points take `ffxContext*`, hence a
  pointer to a pointer, not a context value.
- `ffxReturnCode_t` is `uint32_t`; the eight named return values run from 0 to 7.
  The return type is not the `FfxApiReturnCodes` enum.
- `ffxStructType_t` is `uint64_t`. `ffxApiHeader` contains `type` followed by
  mutable `pNext`; all four descriptor-header typedefs alias that same struct.
- `ffxAlloc` takes `void*` user data and a `uint64_t` size, not `size_t`.
  `ffxDealloc` takes two `void*` arguments. The callback struct contains user data,
  allocation and deallocation pointers, in that order.
- Create and query accept mutable descriptor pointers. Configure and dispatch
  accept const descriptor pointers. Create and destroy accept const allocation
  callback pointers. The header specifies no stdcall modifier; the probe checks
  explicit `__cdecl` types on the tested target.

**Upstream contracts, not runtime-verified behavior:** the comments require
pointers passed in a creation descriptor to remain live until destruction;
allocation must provide the requested bytes with alignment for any type and may
return null; deallocation may receive null. A null allocation-callback struct
pointer selects the system allocator at creation. Destruction requires compatible
callbacks. Query/configure accept a null context pointer for global operations.
This does not imply that null individual allocation function pointers are valid.

## Reproduction

Prerequisites: the local SDK above, the tested uncommitted files (or a subsequent
revision containing them), Rust with rustfmt/Clippy and VS 2022 x64 C++ tools.
The already-declared Cargo dependencies must be cached for offline commands.

The successful Rust validation commands, from the repository root, were:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo check --workspace --no-default-features --locked --offline
cargo build --workspace --locked --offline
cargo doc --workspace --no-deps --locked --offline
git diff --check
```

For the native check, the installed VS 2022 Build Tools `vcvars64.bat` was invoked
to select x64 tools and `target/abi-check` was created. In that developer command
environment the exact compile command was:

```bat
cl /nologo /std:c++17 /W4 /WX /c /Iexternal\FidelityFX-SDK\v2.3.0\Kits\FidelityFX\api\include crates\fsr-sdk-sys\tests\native_abi.cpp /Fotarget\abi-check\native_abi.obj
```

The object is a disposable build artifact in `target/`, not a linked library or
an executable. The probe is deliberately outside the Cargo build.

## Observations

All final commands above succeeded. Three Rust tests passed: `return_codes`,
`nullable_function_pointer_layouts`, and `windows_x64_layouts`. Rust compile-time
checks covered aliases and function/field types. The C++ compile exited 0 with
all static assertions accepted, including comparison of the actual five function
declarations with their `PfnFfx*` typedefs.

| Tested Windows x64 layout | Size | Alignment | Field offsets |
|---|---:|---:|---|
| `ffxContext` | 8 | 8 | — |
| `ffxReturnCode_t` | 4 | 4 | — |
| `ffxStructType_t` | 8 | 8 | — |
| `ffxApiHeader` | 16 | 8 | `type`: 0, `pNext`: 8 |
| `ffxAllocationCallbacks` | 24 | 8 | `pUserData`: 0, `alloc`: 8, `dealloc`: 16 |
| Allocation and five entry-point function pointers | 8 | 8 | — |

Negative setup results: the initial offline Cargo checks failed because
`libloading 0.9.0` was not cached. An ordinary online retry could not connect to
the registry download host. After approved network escalation, Cargo downloaded
that already-declared dependency and tests passed. Subsequent offline checks
passed. These were dependency-access failures, not ABI failures. Rust tooling
also emitted a host-path canonicalization warning on sandboxed runs, without
preventing the final checks.

## Baseline delta and limits

Before this work the API module was empty. We now have checked expectations for
the small common boundary and reproducible probes against the supplied header.
The production code is the Rust declaration module; the C++ file is verification
code, not an FSR implementation or a runtime dependency.

**Inference:** agreement of the two separately checked sets of expectations
supports this ABI mapping on Windows x64/MSVC. The probe does not directly compare
Rust and C++ types, exercise callback invocation or call AMD code. Both sides
could share an omitted check. x64 also does not distinguish cdecl/stdcall as x86
does; this run is not evidence for a 32-bit target. Other platforms, SDK versions,
binary/header consistency, lifecycle safety and GPU correctness remain untested.

The AMD notice was retained as a precaution during implementation. This check
does not establish whether these limited declarations legally require that file,
or settle its placement or distribution policy; no legal conclusion was tested.

The hand-written scope was separately authorized in D003, not inferred from test
success. Full-SDK generation strategy remains open. Next investigations should
address runtime artifacts/discovery and the contracts needed for loading and a
first query, with separate execution evidence when those steps are authorized.
