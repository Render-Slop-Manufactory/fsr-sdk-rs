# Experiment: A context-free version-count query

- Date: 2026-09-20.
- Question: Can the Rust ABI and existing loader execute one real AMD query
  without a context, device, resource or GPU dispatch?
- Success: `ffxQuery` returns `FFX_API_RETURN_OK` and writes a positive upscaler
  version count. No exact count is required by the test.
- Exclusions: enumeration of IDs/names, contexts, safe wrappers, generic query
  abstractions, device creation, resource allocation and dispatch.

## Setup and source findings

Base revision: `b1fd6d6677c12c101bad3c353feae48eb4326dbf`, plus uncommitted
additions to [api.rs](../../../crates/fsr-sdk-sys/src/api.rs), the paired
[Rust ABI tests](../../../crates/fsr-sdk-sys/tests/abi.rs) and
[C++ checks](../../../crates/fsr-sdk-sys/tests/native_abi.cpp), and the new
[opt-in query test](../../../crates/fsr-sdk-sys/tests/query.rs).
The test is the probe; the descriptor and two tags are production declarations.
Loader code, manifests and dependencies are unchanged.

Environment: Windows 10.0.26200.0, Rust 1.98.1 on x86_64-pc-windows-msvc,
VS 2022 Build Tools 17.14.28. No device or GPU was selected by the test.

Local SDK v2.3.0 sources, under `external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/`:

- `api/include/ffx_api.h`, `ffxQueryDescGetVersions`: tag 4, six fields,
  count-only mode when both result arrays are null; zero initial count requests
  the available count. `ffxQuery` permits global queries with a null context.
- `upscalers/include/ffx_upscale.h`: creation descriptor tag expands to
  `0x00010000`, used only as the query's effect selector.
- `api/internal/ffx_api.cpp`, null-context `ffxQuery` branch: recognizes this
  descriptor and writes the count using `GetProviderCount`.
- `api/internal/ffx_provider.h`: count delegates to provider enumeration without
  output arrays; the default `IsSupported` ignores the device. The source FSR2
  and FSR3 upscaler providers inherit that default. In
  `upscalers/fsr3/internal/ffx_provider_fsr2.cpp`, `CanProvide` matches the upscaler
  effect bits.
- `docs/getting-started/ffx-api.md`, version-query example and DLL structure:
  version enumeration is performed before context creation; effects live in
  separate DLLs. The C example omits `header.type` initialization. The test sets
  it explicitly according to the header and implementation instead of copying
  that omission. Device-free behavior of the signed runtime is established by
  the observed run below, not by assuming all source matches its binary.
- `docs/techniques/denoising.md` illustrates placing loader and effect DLLs in
  the executable directory. This motivates the local test deployment; it is not
  proof of all upscaler DLL search rules.

Chosen query: count-only `ffxQueryDescGetVersions` for the upscaler effect.
This uses one descriptor, no returned string lifetimes and no array allocation.
A provider-version query concerns an existing context; effect-specific queries
would introduce other descriptors without improving this first boundary check.
The broader binding strategy remains open; handwriting this small extension is
within the explicitly requested smoke-test scope.

## ABI evidence

`ffxQueryDescGetVersions` is `repr(C)`, size 56 and alignment 8 on Windows x64.
Offsets: header 0, createDescType 16, device 24, outputCount 32, versionIds 40,
versionNames 48. Rust and C++ checks cover all fields/types and both tag values.
The name array is `*mut *const c_char`, corresponding to native `const char**`.
Rust tag constants use `ffxStructType_t` for descriptor fields; native macros are
unsigned integer expressions with the checked values.

The standard MSVC command documented in [Development](../../DEVELOPMENT.md)
compiled the extended probe successfully, including the actual upscaler header.
An initial Clippy type-complexity warning in the test's six-field tuple was fixed
by checking each field type separately; the final Clippy run passed.

## Runtime observations

1. Direct Cargo run with `FSR_SDK_TEST_DLL` pointing to the original SDK loader
   DLL: query returned **4 (`FFX_API_RETURN_NO_PROVIDER`)**, count **0**. The test
   failed as intended. Merely loading that absolute path had not established
   effect-provider availability for this deployment.
2. Copied the test executable, original loader DLL and original upscaler DLL into
   `target/query-smoke/`, then ran that executable with the copied loader path:
   query returned **0 (`FFX_API_RETURN_OK`)**, count **2**; one test passed.

The descriptor used tag 4, effect selector 0x10000, null pNext, null context,
null device, writable count initialized to zero and both output arrays null.
The library and descriptor remained alive through the synchronous call.
There was exactly one AMD API call per test invocation and no context creation.
No loader code or process search-path setting changed between runs.

**Inference:** deployment affected provider discovery. These runs do not isolate
every internal loader search rule or prove which provider versions were counted.
Two available versions do not establish FSR4 support or device compatibility.

SHA-256 of original DLLs copied for the successful run:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

These identify local inputs, not authenticated upstream artifacts or a
redistribution policy. Copies and executable are ignored build artifacts.

## Reproduction

From PowerShell at repository root, with the trusted local SDK and cached Cargo
dependencies, build and stage only the probe and two DLLs:

```powershell
$messages = cargo test -p fsr-sdk-sys --features dx12 --test query --locked --offline --no-run --message-format=json
if ($LASTEXITCODE -ne 0) { throw 'Query test build failed' }
$artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'query' -and $_.executable } | Select-Object -Last 1
New-Item -ItemType Directory -Force target/query-smoke | Out-Null
Copy-Item -LiteralPath $artifact.executable -Destination target/query-smoke/query.exe
Copy-Item external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll,external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll -Destination target/query-smoke
$env:FSR_SDK_TEST_DLL = (Resolve-Path target/query-smoke/amd_fidelityfx_loader_dx12.dll).Path
& ./target/query-smoke/query.exe --ignored --nocapture
```

The failed baseline was `cargo test -p fsr-sdk-sys --features dx12 --test query
--locked --offline -- --ignored --nocapture`, with the environment variable set
to the original SDK loader path instead. The normal workspace test run skips
the query; no installed AMD runtime is required.

Final validation: format check, workspace build, tests, Clippy with `-D warnings`,
no-default-features check and documentation build passed using the commands in
Development with `--locked --offline`. Five ABI tests, four loader tests and two
compile-fail doctests passed; both AMD integration tests are ignored by default.
The MSVC compile-only ABI check passed; the staged query passed separately.

## Limits and next question

This is evidence of one real ABI/loader/query interaction with the identified
local runtime. It does not verify ID/name arrays, driver-provided versions,
GPU support, thread safety, native context lifetimes, resource use or dispatch.
No new public query abstraction or safe wrapper was introduced. Future context
work must establish its own ownership and safety contracts before implementation.
