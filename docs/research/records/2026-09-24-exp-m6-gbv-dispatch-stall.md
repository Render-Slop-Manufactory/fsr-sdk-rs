# Experiment: isolate the M6a GPU-based-validation stall

- Investigation date: 2026-09-24.
- Question: Does the earlier GPU-based-validation (GBV) timeout occur in the
  Rust pre-call checks, the shared DX12 upload/submission/readback setup, or
  after entering the signed FSR4 provider's native `ffxDispatch`?
- Observable distinction: with the same device, context, textures, recording
  list, uploads and readback, run a GBV control that omits only FSR dispatch;
  then enter native dispatch with a marker directly around its call. Record
  completion, debug errors, timeout and process CPU time.
- Exclusions: determining the exact native instruction, assigning fault to AMD
  or Microsoft, testing other driver/GPU versions, and image quality.

## Setup and probe delta

The worktree was clean at the start, at revision
`2379baf34e1821ad3acf3d1eb64bd060f5319995`. The baseline was the
[M6a proof](2026-09-24-exp-m6a-native-dispatch-proof.md): pinned AMD FSR SDK
`v2.3.0` (commit `60f4ea81909200d8542eca14dccb2628b763a9a3`), signed
loader and upscaler DLLs, unchanged public M5 `Runtime::load` and
`Upscaler::new`, provider `4.1.1` (`id=0xf5a5ca1e01001001`), and the same
320×180 → 640×360 five-texture synthetic input.

The probe adds only test instrumentation: two test-build markers around the
private native call in [`dispatch.rs`](../../../crates/fsr-sdk/src/upscaler/dispatch.rs),
an optional control branch in [`m6.rs`](../../../crates/fsr-sdk/src/upscaler/m6.rs)
selected by `FSR_SDK_TEST_M6_CONTROL`, and a bounded
[runner](../../../crates/fsr-sdk/tests/run-m6-gbv-spike.ps1). The control
still creates the signed provider context and checks all five DX12-to-FFX
resource conversions. It omits `ffxDispatch`, closes/submits the same list,
waits on its fence and reads the sentinel output. The dispatch case uses the
original private production path. Neither case changes production dispatch
inputs or the public constructor.

The host was Windows 11 Pro build `26200`, Rust `1.98.1` on Windows x64/MSVC,
Visual Studio 2022 Build Tools, AMD Radeon RX 9060 XT (`vendor=0x1002`,
`device=0x7590`, feature level `12_0`), driver `32.0.31041.1004`. The D3D12
debug layer and GBV were enabled before device creation. Inputs retained the
same signed DLL hashes as the M6a record:

| Input binary | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| Locally built diagnostic `m6_gpu.dll` | `3ED7A46E0218A509A4D0CB97B656E461FE7BAC2531AF041E7FEF701D21799629` |
| Locally built diagnostic `m6.exe` | `1C9750CB654429D1517FA83C6BE18CE7DC500F50706C4D71EB2DA7468E74F4D7` |

## Reproduction

With the local pinned SDK under `external/FidelityFX-SDK/v2.3.0`, run from the
repository root in an x64 Visual Studio developer environment:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6-gbv-spike.ps1'
```

The runner first performs ordinary workspace checks and an isolated native M6a
run with the D3D12 debug layer but without GBV; this also builds/stages the
diagnostic binaries. It then runs two separate children with GBV: control
(45-second limit) and dispatch (90-second limit). It writes stdout, stderr,
status and ten-second CPU-time samples to `target/m6-gbv-spike/`. The parent
terminates a timed-out child. A runner exit code of zero means it collected
both cases; each case's status determines its outcome. The successful run used
normal machine access for OS/GPU inventory, which the restricted shell denies.

## Observations

The ordinary checks and debug-layer-only native baseline passed. The baseline
entered and returned from native `ffxDispatch` with code `0`, submitted and
fenced work, and read back 230,400 finite changed pixels with zero debug
errors and no device removal.

| GBV case | Result | Last relevant observation |
|---|---|---|
| Same DX12 setup, no FSR dispatch | `timeout=False exit=0` | Fence/readback completed; 230,400 finite sentinel pixels, zero changed pixels, zero debug errors, device not removed. |
| Same setup, FSR dispatch | `timeout=True exit=-1` after 90 seconds | Resource parity completed; `M6a: entering native ffxDispatch` printed; the matching return marker never printed. No submission or readback occurred. |

In the timed-out child, cumulative process CPU time rose from `9.70 s` at
10 seconds elapsed to `87.63 s` at 90 seconds. This is roughly one busy CPU
core over the measured interval. The process did not merely wait asleep on a
fence. Earlier M6a attempts under GBV had timed out after 120 and 180 seconds
at the same coarse stage; this probe locates the boundary more precisely.

## Baseline delta and limits

**Verified:** GBV works for this machine's tested DX12 resource creation,
uploads, barriers, copy submission, fence and readback, even with a live FSR4
context. Rust conversion and pre-call validation also complete. The observed
stall begins after entering native `ffxDispatch` and before it returns or the
application submits the command list. The same dispatch succeeds with the
ordinary debug layer and GBV disabled.

**Inference:** the trigger is the interaction between GBV and this signed
provider dispatch on the tested driver/setup. Process CPU consumption suggests
active native work or a busy loop, but does not identify which. The control
does not execute a compute shader, so it cannot exclude a broader GBV compute
instrumentation/driver problem. Without a native call stack, provider internals,
or a second driver/GPU, the experiment cannot assign responsibility among the
provider, the driver, GBV instrumentation, or a provider-sensitive input.
No D3D12 validation message or device-removal reason was obtained from the
timed-out dispatch case because it never reached submission/readback.

The bounded spike therefore rules out a general failure of the shared DX12
setup under GBV and rules out Rust pre-call checks as the stall point. It does
not establish a successful GBV-validated FSR dispatch. A later investigation
would need a native stack/trace or a controlled driver/provider comparison if
that distinction becomes necessary; this experiment does not broaden M6a or
accept a public dispatch contract.
