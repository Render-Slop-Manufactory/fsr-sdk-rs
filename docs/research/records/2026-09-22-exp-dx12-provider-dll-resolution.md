# Experiment: Signed DX12 provider DLL directory resolution

- Investigation date: 2026-09-22.
- Question: Does loading the signed FidelityFX SDK v2.3.0 loader by absolute
  path allow its upscaler provider to reside beside it in a private directory,
  separate from both executable and working directory?
- Success: loader acquisition succeeds, the existing count-only query returns
  OK and a positive count, and module diagnostics identify the staged provider.
- Failure: distinguish loader acquisition failure, query error, zero count,
  assertion exit, crash and timeout. Baseline failure stops subsequent cases.
- Exclusions: production changes, provider discovery APIs, search-path mutation,
  D007, architecture selection, device/context creation and GPU dispatch.

## Setup

Repository revision: `0d0c642b4245c9bb7bca6d2ebcceed1d33c5c695`.
`git status --short` was empty at experiment start. The only changes for this
run are diagnostics in the existing query test, an experiment helper/runner,
this record and its sanitized text artefacts. No manifests, lockfile, production
loader, decisions, synthesis or unrelated work changed.

Native baseline: the local pinned SDK `v2.3.0`, corresponding to the D002 tag
at `60f4ea81909200d8542eca14dccb2628b763a9a3`. Inputs came from
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/`.
Both inputs reported **Valid** through `Get-AuthenticodeSignature`, with signer
`CN=Advanced Micro Devices, O=Advanced Micro Devices, S=California, C=US`.
These are the actual signed binaries exercised, not rebuilt public-source DLLs.

| DLL | Bytes | SHA-256 |
|---|---:|---|
| `amd_fidelityfx_loader_dx12.dll` | 26376 | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | 28761864 | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

The hashes also match the preceding version-query experiment. Signature status
is a local verification observation, not a new redistribution decision.

Environment: Windows NT `10.0.26200.0`; registry DisplayVersion `25H2`, build
`26200`, UBR `9457` (the legacy ProductName value reports `Windows 10 Pro`).
Rust `1.98.1 (48a229cea 2026-09-01)`, host `x86_64-pc-windows-msvc`, LLVM
`22.1.8`; Cargo `1.98.1 (797e8a9bc 2026-08-05)`. The existing installed MSVC
link environment sufficed; its exact linker version was not captured. The runner
used Windows PowerShell through `powershell -NoProfile`. No GPU/device was
selected, queried or created; GPU/driver identity is not an input to this probe.
This does not prove that arbitrary driver configurations give identical results.

### Probe and isolation

The existing [query test](../../../crates/fsr-sdk-sys/tests/query.rs) still issues
exactly one `ffxQueryDescGetVersions` through `FfxLibrary` and retains its normal
OK/positive-count assertions. No second provider query was implemented.
Inputs are header tag `4`, null pNext, effect selector `0x10000`, null context
and device, writable `u64` count initialized to zero, and null ID/name arrays.
The library remains owned through the synchronous call and diagnostics.

The [module helper](../../../crates/fsr-sdk-sys/tests/provider_resolution/modules.rs)
uses `GetModuleHandleW` and `GetModuleFileNameW` to inspect the provider basename
before loading, after loader acquisition and after the query. It never loads a
provider or changes DLL directories. Diagnostics are enabled only by the
experiment's `FSR_SDK_RESOLUTION_DIAGNOSTICS` input. A separate acquisition
message is printed after `FfxLibrary::load` resolves all five entry points.
Failure at acquisition would instead retain the existing `AMD loader` panic.

The [runner](../../../crates/fsr-sdk-sys/tests/provider_resolution/run.ps1) builds
the query executable using Cargo's reported artifact path, copies original DLLs
into a new GUID-named directory, and executes four independent processes. It
uses redirected stdout/stderr and a 60-second per-child timeout; it waits for
termination before advancing. No case shares a process or previously loaded
module with another case. All cases explicitly select only the one ignored test
with `--test-threads=1`.

Inherited PATH contained 61 entries, all rooted. No upscaler provider was found
in any of those locations or the checked Windows, System32 and System directories.
PATH was inherited unchanged; signedbin was not added. No machine environment or
DLL-directory settings were changed. The runner/probe never calls
`SetDllDirectory`, `AddDllDirectory` or `SetDefaultDllDirectories`.
Child-only environment overrides were:

- `FSR_SDK_TEST_DLL`: the absolute staged loader path below.
- `FSR_SDK_RESOLUTION_DIAGNOSTICS=1`: module inspection.
- `RUST_BACKTRACE=0`: bounded error logs, with query assertions preserved.

The last setting is log sanitization, not search-path sanitization. Full personal
PATH values are not retained. These checks are bounded environmental checks,
not an exhaustive audit of Windows policy, injected software or the runtime's
private internals. Before-load module absence, exact successful module paths and
the failed negative control supply independent contamination checks.

## Reproduction

Prerequisites: Windows x64/MSVC, cached declared Cargo dependencies, installed
Rust/link tools and the trusted local v2.3.0 signed DLLs at the stated paths.
Run from the repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/provider_resolution/run.ps1
```

The first attempted invocation without `-ExecutionPolicy Bypass` was rejected
by Windows PowerShell's script execution policy before the runner executed.
The successful invocation above used an explicitly approved process-only policy
override; no persistent execution policy was changed. This launch failure is
not a FidelityFX load/query failure and no layout ran in that attempt.

The runner's exact build command is:

```powershell
cargo test -p fsr-sdk-sys --features dx12 --test query --locked --offline --no-run --message-format=json
```

Every staged executable receives exactly:

```text
--ignored --exact queries_upscaler_version_count_without_context --nocapture --test-threads=1
```

For the observed run let `S` be the absolute path
`<repo>\target\provider-resolution\f6b86b9f0889403f92c6700786886285`.
`<repo>` consistently replaces the actual repository root in retained logs;
it is not a literal path supplied to Windows. The runner constructs actual
absolute paths. A new reproduction chooses a new GUID to prevent stale files.

| Case | Executable | Absolute loader input | Child working directory |
|---|---|---|---|
| A | `S\case-a\app\query.exe` | `S\case-a\app\amd_fidelityfx_loader_dx12.dll` | `S\case-a\cwd` |
| B | `S\case-b\app\query.exe` | `S\case-b\runtime\amd_fidelityfx_loader_dx12.dll` | `S\case-b\cwd` |
| C | `S\case-c\app\query.exe` | `S\case-c\runtime\amd_fidelityfx_loader_dx12.dll` | `S\case-c\cwd` |
| D | `S\case-d\app\query.exe` | `S\case-d\runtime\amd_fidelityfx_loader_dx12.dll` | `S\case-d\cwd` |

Exact binary layouts (all `cwd/` directories were empty; the runner also creates
an empty `runtime/` in A):

```text
case-a/
  app/query.exe
  app/amd_fidelityfx_loader_dx12.dll
  app/amd_fidelityfx_upscaler_dx12.dll
  runtime/ (empty)
  cwd/ (empty)
case-b/
  app/query.exe
  runtime/amd_fidelityfx_loader_dx12.dll
  runtime/amd_fidelityfx_upscaler_dx12.dll
  cwd/ (empty)
case-c/
  app/query.exe
  app/amd_fidelityfx_upscaler_dx12.dll
  runtime/amd_fidelityfx_loader_dx12.dll
  cwd/ (empty)
case-d/
  app/query.exe
  runtime/amd_fidelityfx_loader_dx12.dll
  cwd/ (empty)
```

No additional DLLs were staged. Text logs are written at each case root after
execution. Executables and AMD binaries stay in ignored `target/`; only sanitized
text evidence is retained beside this record.

## Observations

**Verified runtime behavior:** all four processes acquired the loader. In every
case the upscaler provider was absent both before and immediately after that
acquisition. No timeout or native crash occurred.

| Case | Exit | Loader acquired | `ffxQuery` return | Count | Successful enumeration | Provider after query |
|---|---:|---|---:|---:|---|---|
| A: colocated beside executable | 0 | yes | 0 (OK) | 2 | yes | `S\case-a\app\amd_fidelityfx_upscaler_dx12.dll` |
| B: loader/provider private | 101 | yes | 4 (NO_PROVIDER) | 0 | no | absent |
| C: private loader, executable-adjacent provider | 0 | yes | 0 (OK) | 2 | yes | `S\case-c\app\amd_fidelityfx_upscaler_dx12.dll` |
| D: no staged provider | 101 | yes | 4 (NO_PROVIDER) | 0 | no | absent |

The numeric return value 4 was printed by the actual query in B and D; its name
matches `FFX_API_RETURN_NO_PROVIDER` in the pinned ABI. Exit 101 is the Rust test
assertion failure after observing that return, not loader acquisition failure
or an inferred native error. The unchanged assertion prints left 4, right 0.
The outer runner completed with exit 0 because it captured every case; this
does not mean the two negative child queries passed.

Retained evidence (status includes exact loader/cwd and staged file inventory):

| Case | Status/layout | stdout | stderr, query and module observations |
|---|---|---|---|
| A | [status](2026-09-22-exp-dx12-provider-dll-resolution/case-a.status.txt) | [stdout](2026-09-22-exp-dx12-provider-dll-resolution/case-a.stdout.txt) | [stderr](2026-09-22-exp-dx12-provider-dll-resolution/case-a.stderr.txt) |
| B | [status](2026-09-22-exp-dx12-provider-dll-resolution/case-b.status.txt) | [stdout](2026-09-22-exp-dx12-provider-dll-resolution/case-b.stdout.txt) | [stderr](2026-09-22-exp-dx12-provider-dll-resolution/case-b.stderr.txt) |
| C | [status](2026-09-22-exp-dx12-provider-dll-resolution/case-c.status.txt) | [stdout](2026-09-22-exp-dx12-provider-dll-resolution/case-c.stdout.txt) | [stderr](2026-09-22-exp-dx12-provider-dll-resolution/case-c.stderr.txt) |
| D | [status](2026-09-22-exp-dx12-provider-dll-resolution/case-d.status.txt) | [stdout](2026-09-22-exp-dx12-provider-dll-resolution/case-d.stdout.txt) | [stderr](2026-09-22-exp-dx12-provider-dll-resolution/case-d.stderr.txt) |

[Environment/signature evidence](2026-09-22-exp-dx12-provider-dll-resolution/environment.txt)
preserves the sanitized search-location audit and binary fingerprints.

Validation: `cargo fmt --all -- --check` and
`cargo clippy -p fsr-sdk-sys --features dx12 --test query --locked --offline -- -D warnings`
passed. `cargo test -p fsr-sdk-sys --features dx12 --test query --locked --offline`
passed with the native test ignored as intended. The actual native coverage is
the four explicitly invoked cases above; compilation/skipping adds no runtime
evidence. No changed ABI requires a new C++ layout check.

## Baseline delta and limits

Relative to the [source investigation](2026-09-21-src-dx12-provider-dll-resolution.md),
this closes the specific private-directory sufficiency question for the tested
signed binaries and environment. It confirms and isolates the earlier query
experiment's deployment observation, without contradicting the source record's
careful unresolved status or repeating its broad source investigation.

**Inference:** these results are consistent with executable-directory provider
resolution. They do not reveal AMD's exact internal LoadLibrary argument, API,
flags or general search algorithm. The module was not resident at the after-load
sample and was resident after successful enumeration; no call trace was taken,
so transient loads/unloads or the complete eager/lazy implementation remain
unresolved. GetModuleHandleW observes one basename, not a full module inventory.

Only one run per layout was performed, with one pinned loader/provider pair on
one Windows configuration. The tested private directory is named `runtime/`;
no separate literal `runtimes/fsr/` directory-name variant was executed. No GPU
dispatch, provider identities, device compatibility, alternate providers,
collision handling, transitive dependency closure, future releases or arbitrary
Windows policy configurations are established. System-installed dependencies
were not inventoried; this is not a clean-machine deployment certification.

1. **Does the private runtime/fsr layout work?** **Verified:** the structurally
   equivalent private `runtime/` layout B did not: loader acquisition succeeded,
   but enumeration returned 4 and count 0. Colocation alone was insufficient in
   this tested process environment.
2. **Does putting the provider beside the executable change the result?**
   **Verified:** yes. C succeeded with return 0/count 2 while the loader remained
   private; the loaded module path was exactly the staged executable-adjacent DLL.
3. **Does the negative control confirm dependence on the staged provider?**
   **Verified:** D, with the same private-loader structure and no staged provider,
   returned 4/count 0 with no provider module present. Together with C's exact
   module path, this confirms that success in this experiment depends on the
   staged provider, rather than an accidentally discoverable external copy.
4. **What is now verified runtime behavior?** For these signed v2.3.0 DLLs on
   this Windows x64/MSVC configuration, A and C enumerate two versions using the
   staged provider beside the executable; B and D do not enumerate a provider.
   All absolute-path loader acquisitions succeed without probe/runner search-path
   manipulation. This establishes enumeration, not actual upscaling support.
5. **What remains unresolved?** AMD's exact provider-loading call/path/flags,
   full timing/search algorithm, undocumented controls, same-basename coexistence,
   dependency closure, other systems/releases and GPU use remain unresolved.
6. **Is more runtime research necessary before a later D007 decision?** No
   additional run is needed to answer this bounded colocation-sufficiency question
   for these inputs. Whether a later decision needs more experiments depends on
   its requirements: alternate loading mechanisms, multiple runtimes, other
   systems or clean-machine dependency guarantees would need separate evidence.
   This experiment does not choose or recommend an acquisition/deployment
   architecture, write D007, or settle licensing/packaging prerequisites.
