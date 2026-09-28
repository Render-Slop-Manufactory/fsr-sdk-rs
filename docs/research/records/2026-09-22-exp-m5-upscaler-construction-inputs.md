# Experiment: signed v2.3.0 upscaler construction dimensions

- Investigation date: 2026-09-22.
- Topic: Windows/DX12 `ffxCreateContext` behavior for a bounded set of
  structurally valid, numerically unusual upscaler dimensions.
- Hypothesis: each case returns a native code or creates a destroyable context.
- Observable success: the call returns; record its exact code and handle state;
  destroy successful non-null contexts once. Observable failure: process abort,
  exception, timeout, or another failure before the call returns.
- Exclusions: malformed descriptors, invalid pointers, large allocation stress,
  fuzzing, dispatch, image correctness, public constructor implementation and
  architecture decisions.

## Setup

Repository revision at experiment start: `a4566a5105e2a0a1647d085a34227fc884e3dd66`,
clean worktree before the probe was added. At execution, only the new
[test-local probe](../../../crates/fsr-sdk-sys/tests/context_dimensions.rs) and
[runner](../../../crates/fsr-sdk-sys/tests/context_dimensions/run.ps1) were
uncommitted experiment changes. An unrelated, untracked source repair record
(now [dated here](2026-09-22-src-m5-construction-query-provenance-repair.md))
appeared after execution; it was not used or modified for this experiment. No
production API, M5 implementation, accepted decision or synthesis was changed.

Local SDK checkout: FidelityFX SDK `v2.3.0`, Git HEAD
`60f4ea81909200d8542eca14dccb2628b763a9a3`, clean. The paired native ABI
check compiled against this checkout's public headers. AMD's
[v2.3.0 release listing](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0)
identifies the same tag and abbreviated commit. These fingerprints and
signatures identify the tested files; they do not independently attest an entire
upstream release archive.

| Signed runtime input | SHA-256 | Authenticode |
|---|---|---|
| `signedbin/amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` | Valid; Advanced Micro Devices |
| `signedbin/amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` | Valid; Advanced Micro Devices |

Environment: Windows 11 25H2 build `26200.9457`, x64; Rust `1.98.1`
(`x86_64-pc-windows-msvc`); VS 2022 Build Tools developer prompt `17.14.28`,
MSVC toolset `14.44.35207` (compiler file version `19.44.35224.0`). The reused
M4 helper selected AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
feature level `12_0`; DXGI driver query returned `0x00000000`, version
`32.0.31041.1004`. No device-scoped provider enumeration was performed. The
probe queried provider identity only after successful creation.

The probe uses the M4 descriptor chain `upscale -> version (0x01001001) -> DX12
backend -> null`, a real live device from the same helper, `flags = 0`, null
message callback, default/null host allocator, null-initialized output handle,
and no provider override. Its raw ABI path is necessary because the private
production constructor already requires `NonZeroU32`; this experiment does not
exercise that constructor for zero inputs. Unlike M4's production-owner control,
this control directly calls the same pinned native entry points. The raw chain
and device helper match M4's successful native inputs.

## Reproduction

From the repository root, with the local SDK at
`external/FidelityFX-SDK/v2.3.0`, run:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/context_dimensions/run.ps1'
```

The runner compiles the existing C++ ABI check with `/std:c++17 /W4 /WX`,
compiles the existing DX12 device helper with `/std:c++17 /W4 /WX /MT /LD`,
and builds the Rust probe using:

```powershell
cargo test -p fsr-sdk-sys --features dx12 --test context_dimensions --locked --offline --no-run --message-format=json
```

It stages the helper and both signed DLLs beside
`target/m5-dimensions/context_dimensions.exe`. For each case it starts a fresh
child with working directory set to that stage, a PATH limited to Windows
`System32`, and absolute `FSR_SDK_TEST_DLL` and `FSR_SDK_TEST_DEVICE_DLL` paths.
Each child receives its case ID in `FSR_SDK_DIMENSION_CASE` and runs:

```text
context_dimensions.exe --ignored --exact signed_runtime_dimensions --nocapture --test-threads=1
```

The runner waits at most 60 seconds per child and retains its signed/hex exit
status, stdout and stderr. It does not mutate process-wide DLL-search APIs.
Restricting PATH does not prove isolation from all Windows or driver module
resolution. Two initial runner invocations failed during setup before any child
case ran because of a PowerShell environment-variable assignment error. The
runner was corrected, and the complete nine-case run below exited 0 as an
orchestrator; individual child failures remain visible.

## Case matrix

`OK` means exact `ffxReturnCode_t = 0`. Each successful creation returned a
non-null handle, identified provider `id=0xf5a5ca1e01001001`, name `4.1.1`
(`ffxQuery` code `0`), and made exactly one destroy call returning `0`. Those
handles are labeled `non-null-1` within each process; no raw addresses are
retained. The four aborted cases had no returned code or output-handle
observation and no destroy attempt.

| Case | Max render | Max upscale | Child exit | Create returned | Result / destroy |
|---|---:|---:|---|---|---|
| C0 control | 1280×720 | 1920×1080 | `0x00000000` | yes | OK, non-null; destroy OK |
| Z1 | 0×720 | 1920×1080 | `0xC0000409` | no | Rust fatal foreign-exception report; no destroy |
| Z2 | 1280×0 | 1920×1080 | `0xC0000409` | no | same; no destroy |
| Z3 | 1280×720 | 0×1080 | `0xC0000409` | no | same; no destroy |
| Z4 | 1280×720 | 1920×0 | `0xC0000409` | no | same; no destroy |
| T1 | 1×1 | 2×2 | `0x00000000` | yes | OK, non-null; destroy OK |
| N1 | 1279×719 | 1919×1079 | `0x00000000` | yes | OK, non-null; destroy OK |
| E1 | 1280×720 | 1280×720 | `0x00000000` | yes | OK, non-null; destroy OK |
| D1 | 1920×1080 | 1280×720 | `0x00000000` | yes | OK, non-null; destroy OK |

## Observations

**Verified:** each child reached the `create_entered=true` marker. All four
zero-component children then printed
`fatal runtime error: Rust cannot catch foreign exceptions, aborting` and
terminated without `create_returned=true`. Their timeout flags were false.
`0xC0000409` is the observed process exit status; the stderr identifies a
foreign exception crossing the Rust FFI boundary as the reported abort. The
experiment did not identify the original native exception type or throw site,
and this is not evidence of a specific access violation. The failing calls'
selected provider identities are unavailable because no live context existed to
query. The successful cases selected provider 4.1.1; that does not establish
which provider was selected or entered in a failing case.

**Verified:** C0, T1, N1, E1 and D1 returned native code `0`, a non-null handle,
provider query code `0`, and destroy code `0` in separate child processes. No
AMD debug callback was installed; no separate AMD validation message was
observed. Exact per-case status, stdout and stderr are preserved in the
[sanitized event log](2026-09-22-exp-m5-upscaler-construction-inputs/events.txt).
Transient pointers were never printed; `non-null-1` is a semantic label scoped
to each child. Numeric provider IDs, return codes, exit statuses and hardware
IDs were retained unchanged.

## Baseline delta and limits

The C0 raw-ABI control agrees with the earlier
[M4 production-owner lifecycle](2026-09-22-exp-m4-windows-verification.md):
same successful dimensions, device/helper path and observed 4.1.1 provider.
The important new observation is that zero in *any* of the four creation
dimension components did **not** become an ordinary returned native error on
this signed runtime/configuration. A future safe constructor cannot delegate
zero-dimension validation to a returned `ffxReturnCode_t` on this tested path.
The existing private `NonZeroU32` boundary excludes these inputs; this record
does not change that API or decide further validation rules.

Success for 1×1/2×2, non-multiples of eight, equal maxima, and render maxima
larger than upscale maxima characterizes construction and destruction only. It
does not establish that dispatch works, that downscaling is supported, that
nonzero dimensions are generally sufficient, or that other dimensions return
normally. One GPU, driver, OS and signed DLL pair were tested once per case.
External/driver provider provenance and the behavior of other providers, GPUs,
drivers or SDK releases remain unresolved. The local M5 source-intake record
itself notes unverified citations for some implementation claims; those claims
were not used to infer the observed signed-runtime behavior.
