# Experiment: M6a native DX12 dispatch and readback

- Investigation date: 2026-09-24.
- Question: Can the unchanged public M5 `Runtime::load` and `Upscaler::new`
  create a context that records one private upscaler dispatch, submits it on a
  DX12 GPU, and produces a plausible output texture?
- Success criterion: paired dispatch ABI checks; parity with AMD's inline DX12
  resource helper; native `ffxDispatch` OK; command-list close, submission and
  fence completion; pitched output readback with finite, changed, spatially
  varying pixels; no D3D12 debug errors or device removal.
- Exclusions: a public dispatch API, image-quality assessment, a renderer,
  temporal correctness, performance, and hardware/provider support beyond the
  tested configuration.

## Setup and reproducibility

The source baseline is AMD FSR SDK `v2.3.0`, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The signed loader and
upscaler DLLs were staged from the locally installed SDK's
`Kits/FidelityFX/signedbin/` directory. The Rust test used the production
explicit-path runtime and the unchanged public M5 constructor with zero create
flags, not a test-only context constructor. The provider identified itself as
`id=0xf5a5ca1e01001001`, version `4.1.1`.

The machine was Windows 11 Pro build `26200`, Rust `1.98.1` on Windows
x64/MSVC, Visual Studio 2022 Build Tools, AMD Radeon RX 9060 XT
(`vendor=0x1002`, `device=0x7590`, feature level `12_0`) with driver
`32.0.31041.1004`. The D3D12 debug layer was enabled before device creation.
GPU-based validation was disabled for the successful run.

| Input binary | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| Locally built `m6_gpu.dll` | `05366934438B9DCC9FAF89FAAAFC077AA78F4E924622F784C6881D7422194D26` |

These hashes identify the local test inputs, not an independently authenticated
SDK archive. The [runner](../../../crates/fsr-sdk/tests/run-m6.ps1),
[Rust native test](../../../crates/fsr-sdk/src/upscaler/m6.rs), and
[DX12 helper](../../../crates/fsr-sdk/tests/m6_gpu.cpp) contain the executable
setup. In an x64 Visual Studio developer environment with the local SDK at
`external/FidelityFX-SDK/v2.3.0`, the successful invocation was:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6.ps1 -Native'
```

The runner checks formatting, Clippy with warnings denied, ordinary workspace
tests, and the no-default-features build; compiles the paired C++ ABI check
against the pinned headers with MSVC `/W4 /WX`; builds the DX12 helper; and
runs the ignored native Rust test in an isolated child with a 180-second
timeout. It saves fingerprints, machine metadata, stdout, stderr and child
status under `target/m6a/`. Ordinary `cargo test` does not run the GPU case.

## Inputs and recorded work

One DIRECT command queue, allocator and recording graphics list were created
on the context device. Each texture was single-sample, single-mip Texture2D:

| Resource | Size | Format | Data and state before dispatch |
|---|---:|---|---|
| Color | 320×180 | RGBA16F | CPU gradient in red/green and 16-pixel checker in blue; `NON_PIXEL_SHADER_RESOURCE` / FFX compute read |
| Depth | 320×180 | R32F | Constant `0.5`; same read state |
| Motion vectors | 320×180 | RG16F | Zero; same read state |
| Exposure | 1×1 | R32F | `1.0`; same read state |
| Output | 640×360 | RGBA16F | UAV-capable, initialized to half-float `65504` sentinel; `UNORDERED_ACCESS` / FFX UAV |

The uploads used DX12 footprint row pitches and `COPY_DEST` to input-read or
output-UAV barriers. The color was generated with a known subpixel offset
matching dispatch jitter `(0.25, -0.25)`. The descriptor used motion-vector
scale `(320, 180)` for UV vectors, reset `true`, frame time `16.667 ms`, near
`0.1`, far `100`, vertical FOV `π/3`, view units to metres `1`, pre-exposure
`1`, no sharpening, no reactive or transparency mask, and zero flags. After
dispatch, the output was transitioned from UAV to `COPY_SOURCE`, copied to a
readback buffer, submitted, and fenced before CPU inspection. The harness
retained the context, runtime, list, allocator and resources through completion.

The Rust binding's layouts, offsets, selected constants, bool fields and
`PfnFfxDispatch` signature passed Rust checks and an independent MSVC header
check. Before dispatch, all five converted `FfxApiResource` values matched
AMD's inline `ffxApiGetResourceDX12` helper for resource pointer, description,
usage and declared state. Fixture tests also checked descriptor fields, one
native call, invalid input rejection and preservation of an unknown native
error code; those fixtures alone do not prove GPU execution.

## Observations

The successful runner exited `0`; the isolated native case reported
`timeout=False exit=0`. Its native output included:

```text
M6a: DX12 resource parity complete
ffxDispatch: OK
GPU readback: Metrics { width: 640, height: 360, finite_pixels: 230400, changed_pixels: 230400, min_red: 0.101745605, max_red: 0.8486328, left_red: 0.19475134, right_red: 0.7568281, debug_errors: 0, device_removed: 0 }
```

Thus the list closed, submitted and reached the fence, all 230,400 readback
pixels were finite and differed from the sentinel, and red increased from the
left-region mean `0.19475134` to right-region mean `0.7568281`. The D3D12
debug InfoQueue recorded zero errors and the device was not removed. This is
evidence of a meaningful write and a coarse relationship to the input gradient,
not a reconstruction-quality measurement.

Two earlier isolated attempts with GPU-based validation enabled reached the
resource-parity message but timed out after 120 and 180 seconds respectively
inside native `ffxDispatch`, before submission or readback. That mode's result
is inconclusive; the timeout does not identify whether validation overhead,
the provider, a test input or their interaction caused the stall. The runner
now enables that mode only with `-GpuValidation`. A restricted shell also
denied OS/GPU inventory access; the successful run used a normal PowerShell 7
developer shell. These setup and validation observations are separate from the
successful debug-layer-only GPU run.

## Result and limits

M6a's bounded integration question is answered for this Windows/DX12 machine,
driver and signed provider: the private production dispatch path recorded work
through the unchanged M5 constructor and produced a fenced, plausible output.
The proof does not establish public API safety, general resource-state
guarantees, GPU-based validation success, temporal behavior or image quality.
Resource state, backing-memory non-aliasing, residency and GPU completion are
still caller obligations for any future dispatch contract. The next step is to
decide that public contract from this result before exposing a method.
