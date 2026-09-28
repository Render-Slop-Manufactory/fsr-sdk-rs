# Experiment: M5 public construction on Windows/DX12

- Investigation date: 2026-09-24.
- Question: Does the implemented public `Runtime` and `Upscaler` path complete
  the two-context native lifecycle on this Windows x64/MSVC machine?
- Success: ordinary checks and paired native ABI compilation pass; each isolated
  native case creates A and B, identifies a provider through B after A is
  destroyed, cleans up B, and exits without a timeout.
- Failure: a failed check, native error, crash, missing provider identity, or
  timeout. The two cases cover explicit destruction and `Drop` for B.
- Exclusions: Dispatch, reconstructed images, performance, concurrent use,
  general numerical input safety, and support for other machines or providers.

## Setup

The working tree was clean before execution. No production code, test code,
manifest, or lockfile changed for the native run. The public path under test
used `Runtime::load` and `Upscaler::new`, with shared loader retention and an
independent COM reference to the borrowed device. The existing
[M5 runner](../../../crates/fsr-sdk/tests/run-m5.ps1),
[native lifecycle tests](../../../crates/fsr-sdk/src/upscaler/tests.rs), and
[paired C++ ABI check](../../../crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp)
were the executable evidence. Documentation was updated after the run. This
record does not archive the tested source tree; a later checkout requires a new
run to establish its behavior.

Windows 11 Pro build `26200`, Rust `1.98.1` on `x86_64-pc-windows-msvc`, and
Visual Studio 2022 Build Tools developer prompt `17.14.28` were used. The DX12
helper selected an AMD Radeon RX 9060 XT (`vendor=0x1002`, `device=0x7590`,
feature level `12_0`); the reported driver was `32.0.31041.1004`. Inputs were
the locally staged SDK `v2.3.0` signed loader and upscaler DLLs under
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/`. Both source DLLs
reported valid Authenticode signatures by Advanced Micro Devices. The staged
DLL fingerprints were:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| Locally built `device.dll` | `DB93261E73C829CBCDCACCF5A8E49A689F84B3A689F7CC4167B8F737B9B70CEE` |

These hashes identify the local inputs, rather than independently proving the
contents of an upstream release archive. The runner used render size `1280x720`,
upscale size `1920x1080`, flags zero, and a hardware DX12 device. It placed the
provider beside the test executable. No GPU work was submitted.

## Reproduction

From the repository root on this machine, with the local SDK at the path above:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && pwsh -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk/tests/run-m5.ps1 -Native'
```

The successful invocation required normal machine access for `Get-CimInstance`
OS/GPU inventory; the restricted session returned access denied before either
native case started. The process-local execution-policy override did not change
machine policy. The runner saved `native.environment.txt` and each case's
stdout, stderr, and status under `target/m5-lifecycle/`. The environment file's
absolute hash paths were sanitized to DLL basenames after execution; the runner
was then changed to emit basenames in future runs. This metadata change did not
alter either native test. Each native child had a 60-second timeout and ran in
a separate process. The runner also executed format,
Clippy, ordinary workspace tests, no-default-features checking, paired C++ ABI
compilation, and the DX12 helper build.

## Observations

The first attempt used Windows PowerShell 5.1 after `vcvars64.bat`. Format,
Clippy, ordinary tests, the no-default-features check, paired C++ ABI compilation,
and helper/test builds passed. It then exited 1 before native execution because
`Get-FileHash` was unavailable in that developer-shell PowerShell process. A
direct command lookup reproduced that absence. The second attempt used
PowerShell 7.6.6 (`pwsh`) and passed the same phases but exited 1 at
`Get-CimInstance Win32_OperatingSystem` with access denied in the restricted
session. Neither failed attempt reached a native lifecycle case.

**Verified:** the final invocation exited 0. Ordinary tests passed (15 wrapper
tests, six ABI tests, four loader tests, and two compile-fail doctests; opt-in
tests remained ignored in that phase). Format, Clippy, no-default-features, the
paired C++ ABI check, and the native helper build passed. Both isolated native
cases reported one passing test, `timeout=False exit=0`:

| Case | Native observation |
|---|---|
| B explicit destruction | A and B creation succeeded; provider `id=0xf5a5ca1e01001001`, name `4.1.1`; A and B destruction succeeded. |
| B `Drop` | A and B creation succeeded; the same provider was identified; A explicit destruction and B `Drop` completed. |

In both cases the loader was resident after dropping the public runtime and
caller device, remained resident after destroying A, and was no longer resident
after B cleanup. This residency observation concerns the API loader DLL. The
earlier [raw shared-runtime experiment](2026-09-22-exp-m5-shared-runtime-provider-lifetime.md)
observed the upscaler provider DLL, which remained resident after its owner was
dropped; these are different modules and different probes. The native test's
Rust ownership assertions passed. It also rejected zero dimensions in Rust
before native construction. Provider identity was queried while B remained live.
The test did not identify every participating provider module or prove universal
DLL unloading.

## Baseline delta and limits

M5 now has successful public-constructor native lifecycle evidence on this
Windows x64/MSVC, GPU, driver, and signed-runtime configuration, for both B
cleanup paths. This closes the opt-in M5 verification item in the
[roadmap](../../../ROADMAP.md) for its bounded construction scope. The two
setup failures are recorded above; neither was a native test failure.

One pass per case does not establish broader hardware compatibility, native
concurrency, all valid dimension ranges, correct image output, or dispatch
safety. The next implementation milestone remains DX12 dispatch, whose resource
states, synchronization, and output need separate evidence.
