# Experiment: Sequential shared-runtime upscaler lifetime

- **Investigation date:** 2026-09-22.
- **Topic and hypothesis:** On the signed FidelityFX SDK v2.3.0 Windows/DX12
  runtime, two upscaler contexts can coexist through one loaded `FfxLibrary`;
  after A is destroyed, B remains queryable and the staged upscaler DLL remains
  resident.
- **Exclusions:** Concurrent calls, stress, Dispatch, image output, arbitrary
  context counts, other effects/providers/SDK releases, `Send`/`Sync`, public
  runtime ownership design and D008.
- **Observable success:** A and B create with non-null handles; both live-context
  provider queries succeed; A destroys once; B's repeated query succeeds with
  the same provider identity while the same staged provider module is resident;
  B destroys once; the child exits without crash or timeout.
- **Observable failure:** B create fails while A lives; A destroy breaks B's
  query or changes identity; the provider module disappears while B lives;
  a native call fails, crashes or hangs. A failed destroy is terminal under D005.

## Setup

The repository was clean at revision `93199eddf1b655f7b075652e4e7ca1493f026070`
before adding the [test-only probe](../../../crates/fsr-sdk-sys/tests/shared_runtime.rs),
[runner](../../../crates/fsr-sdk-sys/tests/shared_runtime/run.ps1) and this record.
No production source, manifests, dependencies, lockfile, owner or decisions were
changed. The local SDK checkout is exactly
`60f4ea81909200d8542eca14dccb2628b763a9a3` (`v2.3.0`).

Windows 11 build `26200.9457` (`25H2`), Rust `1.98.1` with
`x86_64-pc-windows-msvc`, VS 2022 Build Tools developer shell `17.14.28` and
MSVC `19.44.35224` were used. The existing C++ helper selected an AMD Radeon
RX 9060 XT, vendor `0x1002`, device `0x7590`, hardware feature level `12_0`.
Its DXGI driver query returned HRESULT `0x00000000` and driver
`32.0.31041.1004`.

The runner checked the following local signed files before staging them.
Authenticode returned `Valid`, with `Advanced Micro Devices` as signer, for
both. These hashes identify the exact local inputs; they are not independent
authentication of the upstream release archive.

| SDK `Kits/FidelityFX/signedbin/` input | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

One `FfxLibrary` loaded the staged loader. The provider DLL was placed beside
the staged test executable under `target/shared-runtime/`, as D007 requires for
this baseline. The helper created one DX12 device; both independent, boxed
descriptor chains remained at stable addresses through native destruction.
Each chain was `upscale -> version (0x01001001) -> DX12 -> null`, with render
`1280x720`, upscale `1920x1080`, flags `0`, null message callback and default
null host allocator. The caller's one device reference, helper and library
remained live throughout both context lifetimes. No GPU dispatch occurred.

The probe reused the paired C++/Rust creation and provider-query ABI checks
from the M4 research tests. Its module observation calls `GetModuleHandleW`
and `GetModuleFileNameW`, which do not add a provider-module reference. It
compares the observed canonical module path to the staged file and reports a
sanitized path. HMODULE values were reduced to `first`/`same`/`changed`; no
transient address was retained.

## Reproduction

From the repository root, with the local SDK checkout and a hardware DX12
device, initialize the x64 VS developer environment and run the probe:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && pwsh -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/shared_runtime/run.ps1'
```

The runner checks the two source DLL hashes and Authenticode signatures, compiles
`tests/context_lifecycle/native_abi.cpp` with `/std:c++17 /W4 /WX`, builds the
existing `device.cpp` helper with `/MT /LD`, and builds the ignored Rust test
with:

```powershell
cargo test -p fsr-sdk-sys --features dx12 --test shared_runtime --locked --offline --no-run --message-format=json
```

It copies the probe, helper and signed loader/provider to
`target/shared-runtime/`, sets absolute child-only paths, and runs exactly one
test in a fresh child process with a 60-second timeout. It rechecks the staged
DLL hashes before the child starts:

```text
shared_runtime.exe --ignored --exact sequential_shared_runtime_lifetime --nocapture --test-threads=1
```

It saves `native.stdout.txt`, `native.stderr.txt` and `native.status.txt` in
that ignored target directory. The first attempt used Windows PowerShell 5.1,
whose session could not resolve `Get-FileHash` after the VS environment setup;
it stopped before compiling or loading AMD code. The successful invocation
above used PowerShell 7 (`pwsh`) and changed no probe inputs.

## Timeline

**Verified signed-runtime observation:** the native C++ ABI check and helper
build succeeded. The Rust test built offline. The child ran one test, reported
`1 passed`, and exited `0` without timeout in `0.38s` on the final run. Its relevant sequential
events were:

| Order | Event | Result |
|---:|---|---|
| 1 | Before loader acquisition | Provider module absent |
| 2 | Load one `FfxLibrary`; create one DX12 device | Loader acquired; device HRESULT `0x00000000` |
| 3 | Create A | Return `0`, non-null handle |
| 4 | Create B while A lives | Return `0`, non-null handle |
| 5 | Query A, then B | Both return `0`, provider `0xf5a5ca1e01001001` / `4.1.1` |
| 6 | Destroy A once | Return `0` |
| 7 | Query B again | Return `0`, same ID and name (`B_identity_equal=true`) |
| 8 | Destroy B once | Return `0` |
| 9 | Release caller device reference; drop `FfxLibrary` | Child then exits cleanly |

No A/B native calls overlapped. The child used no native call on A after its
destruction and did not infer liveness from post-destroy handle bits.

## Module residency observations

| Probe point | Loaded? | Identity against first sighting | Canonical path check |
|---|---|---|---|
| Before loader acquisition | No | — | — |
| After A create | Yes | First | `<stage>/amd_fidelityfx_upscaler_dx12.dll`, matches staged file |
| After B create | Yes | Same | Matches staged file |
| Immediately after A destroy, B live | Yes | Same | Matches staged file |
| After B destroy | Yes | Same | Matches staged file |
| After `FfxLibrary` drop | Yes | Same | Matches staged file |

**Verified:** the staged provider module was resident at every observed point
from A's creation onward, including after the loader owner was dropped. The
lookup itself did not increment its reference count. The probe did not measure
module reference counts or identify what retains the DLL after all contexts
and the API loader owner are gone. `GetModuleHandleW` checks one module by
basename, not the entire module inventory; the exact staged-path check reduces
but does not eliminate questions about other loaded providers or driver code.

## Context observations

| Observation | A | B |
|---|---|---|
| Create return / handle | `0` / non-null | `0` / non-null |
| Query while both live | `0`; ID `0xf5a5ca1e01001001`; name `4.1.1` | `0`; same ID/name |
| Query after A destroy | Not called | `0`; same ID/name |
| Destroy attempts / result | Exactly one / `0` | Exactly one / `0` |

The copied provider name and ID came from a successful live-context query,
not from the SDK version label or DLL filename. This query proves the chosen
context-bound operation still worked on B; it does not test Dispatch or prove
that B's GPU resources would produce a correct image.

## Baseline delta and limits

**Verified delta:** M4 had one successful production-owner lifecycle. This
separate raw, test-only probe establishes that two contexts coexisted
sequentially through one `FfxLibrary` on this exact signed-runtime, OS,
GPU/driver and deployment layout. Destroying A did not prevent a second
successful live-context provider query or destruction of B. The staged
upscaler module remained resident while B lived.

**Inference:** the observation is consistent with a future shared-runtime
owner retaining one loader for multiple contexts. It does not itself establish
the correct Rust ownership API, a normative DLL residency contract, or which
module implemented provider `4.1.1`. The loaded staged DLL and selected
provider identity are separate observations; a driver provider may also have
participated. Residency after the final drop is an observed process state,
not evidence of a required or permanent leak.

**Unresolved:** concurrency and `Send`/`Sync`; more than two contexts; differing
providers, effects, devices and SDK versions; module ownership/reference counts;
later process unload behavior; GPU dispatch and image correctness. No D008
decision or production ownership change follows automatically from this record.
