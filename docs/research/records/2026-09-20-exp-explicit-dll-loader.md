# Experiment: Explicit DLL loading with retained ownership

- Date: 2026-09-20.
- Question: Can the bounded loader resolve the five API exports, report failures
  and retain/release its DLL without exposing independent function pointers?
- Success: fixture failure, forwarding and unload checks pass; ownership misuse
  fails compilation; the explicitly selected local AMD DLL loads and resolves all
  five symbols without an API call. Failure: a check rejects those expectations.
- Exclusions: AMD query/context/dispatch calls, safe wrappers, GPU execution,
  automatic discovery, acquisition, installation and distribution decisions.

## Setup and evidence

Base revision: `e01b9185f6666893d84d20db3eaee4608bba15d9`, plus uncommitted
[loader implementation](../../../crates/fsr-sdk-sys/src/loader.rs),
[tests](../../../crates/fsr-sdk-sys/tests/loader.rs) and
[Rust DLL fixture](../../../crates/fsr-sdk-sys/tests/fixtures/loader.rs).
The loader is production code; the fixture and ignored AMD test are probes.
ABI declarations, manifests and lockfile are unchanged. HEAD alone does not
reproduce these additions. Status/decision/research documentation was also updated.

Environment: Windows 10.0.26200.0; Rust 1.98.1, host
`x86_64-pc-windows-msvc`; VS 2022 Build Tools 17.14.28; PE Dumper
14.44.35224.0. No GPU/driver evidence is needed or claimed.

Authoritative local SDK inputs under `external/FidelityFX-SDK/v2.3.0/`:

- `Kits/FidelityFX/api/include/ffx_api.h`: five function signatures.
- `Kits/FidelityFX/api/include/ffx_api_loader.h`: exact symbol names and native
  table. Its helper uses `GetProcAddress` for each of the five functions.
- `Kits/FidelityFX/docs/getting-started/ffx-api.md`, DLL structure section:
  **upstream claim** that the API entry DLL is `amd_fidelityfx_loader_dx12.dll`,
  while effect code resides in separate effect DLLs such as the upscaler DLL.
- `Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll`: observed local test
  artifact. SHA-256:
  `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA`.

The local DLL's signature and correspondence to an upstream release artifact
were not independently authenticated. The hash identifies tested bytes.
The existing cached `libloading 0.9.0` source (`src/safe.rs` and
`src/os/windows/mod.rs`) establishes initialization/termination obligations,
name-only search behavior and rejection of null `GetProcAddress` results.

## Implementation and ownership findings

**Verified by inspection:** `FfxLibrary` owns its `Library` and the five private
existing `PfnFfx*` values. Loading canonicalizes the explicit path before opening
the main DLL. Each symbol name is paired with its matching typedef. On a failed
lookup, local RAII ownership drops the library. On success no pointer, symbol
guard, library handle or pointer table is publicly returned. The five unsafe
native-named methods borrow `self`, forward arguments and return native codes.

On Windows, successful `libloading::get` cannot return a null address. This
establishes the private `Some` invariant used by the forwarding methods, while
ABI compatibility remains the unsafe loader caller's responsibility. Symbol
names alone cannot validate ABI or version. The methods add no descriptor
validation or lifecycle behavior. Native objects and pending work are not tied
to Rust lifetimes by this raw loader.

Errors distinguish path resolution (`io::Error`), library loading and missing
symbols (`libloading::Error`), preserve source errors, and include the path and,
where applicable, export name. Dependencies follow Windows' loader behavior;
there is no process search-path mutation or custom provider discovery.

## Reproduction and observations

From the repository root with cached declared Cargo dependencies:

```text
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --doc --locked --offline
cargo check --workspace --no-default-features --locked --offline
cargo build --workspace --locked --offline
cargo doc --workspace --no-deps --locked --offline
cargo fmt --all -- --check
```

**Observed:** all commands exited 0. The normal tests passed the three existing
ABI tests and four new loader tests; the AMD integration test was ignored by
default. After adding the ownership examples, the separate doctest command
passed both compile-fail tests. The negative runtime tests intentionally rejected
a missing path, a non-PE file and five fixture variants each missing one export.
Every missing-symbol case released the acquired module, including failures after
earlier exports had resolved. The complete fixture verified all argument values
and distinct return codes, remained loaded after moving the owner into a Box,
and was no longer loaded after dropping it. Tests use Windows module lookup to
check loaded state. Fixtures never call AMD code or dereference sentinel pointers.

Tests compile the fixture with `rustc --edition=2024 --crate-type=cdylib
--crate-name=loader_fixture <fixture-path> -o <temporary-dll-path>`, optionally
adding `--cfg missing_<symbol>`. `rustc` on PATH must match the running tests'
target. Temporary fixture directories remain available for inspection.

In the VS 2022 x64 developer environment:

```bat
dumpbin /exports external\FidelityFX-SDK\v2.3.0\Kits\FidelityFX\signedbin\amd_fidelityfx_loader_dx12.dll
```

**Observed:** exit 0; exactly five named exports: `ffxConfigure`,
`ffxCreateContext`, `ffxDestroyContext`, `ffxDispatch`, `ffxQuery`.

The explicit AMD integration run used PowerShell:

```powershell
$env:FSR_SDK_TEST_DLL = (Resolve-Path external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll).Path
cargo test -p fsr-sdk-sys --features dx12 --test loader --locked --offline loads_amd_runtime_without_api_calls -- --ignored --exact
```

**Observed:** exit 0, one passed test. The DLL loaded, all five lookups succeeded
and the owning object was dropped. DLL initialization/termination may execute;
none of the five AMD API entry points was called. No download was required.
Rust emitted a sandbox host-path canonicalization warning without failing checks.

## Limits and next question

This advances evidence from header/compilation checks to actual DLL loading and
symbol resolution. It does not establish effect-provider loading, version
compatibility beyond these observed bytes, AMD callback behavior, query success,
context safety or GPU correctness. Fixture calls establish forwarding behavior,
not FidelityFX runtime behavior. Ownership tests do not make raw native contexts
safe; callers must retain the loader while any native state depends on it.

[D004](../../DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points)
records the separately authorized design. Automatic discovery, deployment and
the broader binding strategy remain open. Next: inspect the descriptor and safety
contract for one native query, then verify it in a separately authorized step.
