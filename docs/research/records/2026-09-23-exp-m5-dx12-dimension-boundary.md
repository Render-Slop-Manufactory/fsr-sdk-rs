# Experiment: signed DX12 upscaler creation at the Texture2D width boundary

- Investigation date: 2026-09-23.
- Topic: Behavior of the signed FidelityFX SDK v2.3.0 DX12 upscaler immediately
  below, at, and one unit above the D3D12 Texture2D width limit of 16384.
- Hypothesis: a structurally valid call at each width either creates a context
  that can be destroyed once, returns an ordinary `ffxReturnCode_t`, or fails
  before a normal return. No particular outcome was presumed.
- Observable result: for each fresh child, record entry and return markers,
  exact return code and null/non-null output, child exit/timeout/exception,
  provider identity if a successful live context permits a query, and the
  outcome of exactly one destroy if creation returns OK with non-null output.
- Exclusions: any other dimensions, height-boundary tests, fuzzing, dispatch,
  image correctness, a complete safe size domain, M5 implementation, public API
  changes, and architecture decisions.

## Setup

Repository revision before the experiment: `01753a0064205a61a526d51d7746ca52e11b5dbc`;
the worktree was clean. At execution the only uncommitted changes were the
three B cases in the experiment-only
[`context_dimensions.rs`](../../../crates/fsr-sdk-sys/tests/context_dimensions.rs),
case selection in its [runner](../../../crates/fsr-sdk-sys/tests/context_dimensions/run.ps1),
and a local-memory budget print in the reused
[`device.cpp`](../../../crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp)
helper. No production source or public API changed. This record and its log were
added after execution.

The local SDK checkout was FidelityFX SDK `v2.3.0`, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`, with a clean SDK worktree.
Both runtime DLLs came from its `Kits/FidelityFX/signedbin/` directory:

| DLL | SHA-256 | Authenticode |
|---|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` | Valid; Advanced Micro Devices |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` | Valid; Advanced Micro Devices |

Environment: Windows 11 25H2, build `26200.9457`, x64; Rust `1.98.1`
(`x86_64-pc-windows-msvc`); VS 2022 Build Tools developer prompt `17.14.28`,
MSVC toolset `14.44.35207` (compiler file version `19.44.35224.0`). The reused
M4 helper selected an AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
feature level `12_0`; its driver query returned HRESULT `0x00000000`, version
`32.0.31041.1004`. The helper reported dedicated memory `16974905344` bytes,
local budget `16165400576` bytes, and this child process's pre-create usage
`5279744` bytes for every case. A separate Windows GPU counter showed about
`10.25` billion bytes of dedicated usage on its active adapter before and after
the run. If that counter refers to the selected adapter, the budget had about
`5.9` billion bytes of headroom. The helper's usage figure is process-local, not
total system usage. No clear out-of-memory diagnostic was observed; these
readings do not prove memory availability for every provider allocation.

The probe uses the previously successful M4/M5 descriptor chain
`upscale -> required version (0x01001001) -> DX12 backend -> null`, the same
real live device in the backend node, `flags = 0`, null message callback,
default/null host allocator, null-initialized context output, and no explicit
provider override. Descriptor storage, device and library remain live through
the call. The paired native ABI check is compiled against the pinned SDK
headers. The earlier [M5 construction-input experiment](2026-09-22-exp-m5-upscaler-construction-inputs.md)
observed provider ID `0xf5a5ca1e01001001`, name `4.1.1`, for successful
creation on this machine; that historical identity is not a provider identity
measurement for the three failed creations below.

## Reproduction

From the repository root, with the local SDK at
`external/FidelityFX-SDK/v2.3.0`, run the exact experiment command:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/context_dimensions/run.ps1 -Cases B1,B2,B3'
```

The runner's exact native build commands are:

```powershell
cl /nologo /std:c++17 /W4 /WX /c /Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp /Fotarget/m5-dimensions/native_abi.obj
cl /nologo /std:c++17 /W4 /WX /MT /LD crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fotarget/m5-dimensions/device.obj /link /OUT:target/m5-dimensions/device.dll /IMPLIB:target/m5-dimensions/device.lib d3d12.lib dxgi.lib
```

It builds the Rust executable with:

```powershell
cargo test -p fsr-sdk-sys --features dx12 --test context_dimensions --locked --offline --no-run --message-format=json
```

The runner stages `context_dimensions.exe`, `device.dll`, and both signed AMD
DLLs together in `target/m5-dimensions/`, with the provider DLL beside the
executable as required by D007. Each case starts a fresh child with that stage
as working directory, absolute `FSR_SDK_TEST_DLL` and
`FSR_SDK_TEST_DEVICE_DLL` paths, a `PATH` limited to Windows `System32`, and
its own `FSR_SDK_DIMENSION_CASE`. The exact child invocation is:

```text
context_dimensions.exe --ignored --exact signed_runtime_dimensions --nocapture --test-threads=1
```

The runner waits up to 60 seconds per child and saves signed/hex status,
stdout and stderr. It does not change process-wide DLL search settings. An
initial runner invocation passed `B1,B2,B3` as one case string; that child
panicked in Rust case selection with exit `101` before device acquisition or
`ffxCreateContext`. The runner's selector was corrected to split the list;
the following complete run executed B1, B2 and B3 once each. The failed setup
invocation is not a native boundary result.

## Case matrix

Only width changes. Equal render/upscale maxima were previously successful at
`1280x720`, avoiding a separate render/output ordering question. Height stays
`720`; `16384x720` is `11796480` pixels.

| Case | `maxRenderSize` | `maxUpscaleSize` | Child exit | Timeout | Create entered / returned | Return code | Context | Provider query | Destroy |
|---|---|---|---|---|---|---:|---|---|---|
| B1 | `16383x720` | `16383x720` | signed `0`, `0x00000000` | false | yes / yes | `1` (`FFX_API_RETURN_ERROR`) | null | unavailable; no live context | not attempted |
| B2 | `16384x720` | `16384x720` | signed `0`, `0x00000000` | false | yes / yes | `1` (`FFX_API_RETURN_ERROR`) | null | unavailable; no live context | not attempted |
| B3 | `16385x720` | `16385x720` | signed `0`, `0x00000000` | false | yes / yes | `1` (`FFX_API_RETURN_ERROR`) | null | unavailable; no live context | not attempted |

## Observations

**Verified:** every child reached `create_entered=true` and
`create_returned=true`. All three calls returned exact `ffxReturnCode_t = 1`
with a null output. Each child exited normally with status `0x00000000` and
`timeout=False`; there was no observed foreign exception, access violation,
abort or process termination. A returned non-OK result caused no destroy
attempt, following D005. No context was available to query for provider ID or
name. Stdout reported one passing harness test per child; stderr contained
the adapter/driver/budget readings, dimensions, entry/return markers and
`destroy_attempted=false`. Exact per-case output is retained in the
[sanitized event log](2026-09-23-exp-m5-dx12-dimension-boundary/events.txt).
No transient pointer address was printed: the context output was null in every
case. Numeric dimensions, status, return code, device IDs and runtime hashes
were preserved.

**Inference:** B3 demonstrates that this particular structurally valid
`16385x720 -> 16385x720` call was rejected through the ordinary native ABI on
the tested configuration. B1 and B2 returned the same generic error, so this
matrix does not isolate the D3D12 width limit as the cause of rejection or
demonstrate that width `16384` is accepted. The code does not identify the
rejection site or whether the selected provider reached D3D12 resource creation.
The generic code `1` does not identify out-of-memory, a provider size/aspect
rule, or another cause.

## Baseline delta and limits

The [source investigation](2026-09-23-src-m5-numeric-construction-safety-contract.md)
identified D3D12's Texture2D width/height limit of `16384`, but no complete
signed-provider numerical contract. This experiment adds one normal-return
observation on each side of that width boundary. It does not reproduce the
earlier zero-component process abort; the inputs and observed outcomes differ.
The three generic returned errors leave the acceptance boundary unresolved.

**Unresolved:** the tested provider identity during these failed calls, the
exact rejection cause, any resource-pressure contribution, and whether a
different device/driver/provider/Windows configuration accepts or rejects any
case differently. Earlier successful equal-size construction at `1280x720`
does not imply successful equal-size construction at these widths. No result
establishes that all widths in `1..=16384` are safe or supported, nor that all
unsupported dimensions return normally. There was no GPU dispatch, no image
output, and no basis for a Rust validation rule or runtime-trust decision.
Proposed D008 and the accepted architecture remain unchanged.
