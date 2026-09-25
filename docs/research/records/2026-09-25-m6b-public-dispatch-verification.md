# M6b public DX12 dispatch verification — 2026-09-25

## Scope and tested implementation

This record covers the first public `unsafe Upscaler::dispatch` implementation
under [D011](../../adr/D011.md). The Windows/DX12 method borrows one DIRECT
command list and five textures, validates the inspectable fixed profile, builds
the pinned SDK v2.3.0 descriptor, and records via `ffxDispatch`. A non-OK native
return poisons later dispatch but leaves terminal teardown available. The test
uses the public M5 runtime/context constructor and the public dispatch method.
It checks one synthetic reset frame, not temporal reconstruction or image quality.

Affected production paths are `crates/fsr-sdk/src/upscaler.rs`,
`crates/fsr-sdk/src/upscaler/dispatch.rs` and `crates/fsr-sdk/src/error.rs`.
The native harness is `crates/fsr-sdk/src/upscaler/m6.rs` plus
`crates/fsr-sdk/tests/m6_gpu.cpp`; fixture policy tests are in
`crates/fsr-sdk/src/upscaler/tests.rs`. Source and tests are in the repository;
the `target/m6a` outputs are local run artifacts only.

## Inputs and commands

- Host: Windows 11 Pro 10.0.26200, x86-64 MSVC; Rust 1.98.1.
- GPU: AMD Radeon RX 9060 XT, feature level 12_0, driver 32.0.31041.1004.
- SDK headers and signed runtime: local pinned FidelityFX SDK v2.3.0.
  Loader DLL SHA-256 `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA`;
  upscaler DLL SHA-256 `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46`.
  Provider query returned ID `0xf5a5ca1e01001001`, name `4.1.1`.
- Profile: 320×180 linear RGBA16F color, ordinary R32F depth, zero RG16F
  motion vectors, 1×1 R32F exposure at 1.0, and a distinct 640×360 UAV-capable
  RGBA16F output. Jitter `(0.25, -0.25)` pixels, 16.667 ms frame time,
  near/far `0.1/100`, vertical FOV `π/3`, metres factor `1`, reset true.
  Inputs were transitioned to `NON_PIXEL_SHADER_RESOURCE`, output to
  `UNORDERED_ACCESS`. The harness submitted, fenced and read back with row pitch.

From the repository root, in a PowerShell shell with the x64 VS 2022 Build Tools:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6.ps1 -Native'
```

The runner performed `cargo fmt --all -- --check`, workspace Clippy with
`-D warnings`, ordinary workspace tests, and the no-default-features check,
all locked/offline. It compiled the paired `dispatch_native_abi.cpp` against
v2.3.0 headers and the DX12 helper with MSVC, staged both signed DLLs beside
the isolated Rust test executable, then ran the ignored public-route test in
one process with a 180-second timeout. GPU-based validation was disabled;
the D3D12 debug layer was enabled. The runner saved environment, status and
output in `target/m6a/`. A first sandboxed attempt passed ordinary checks and
native compilation but stopped at the runner's Windows OS metadata query with
`Access denied`; it did not execute the GPU child. The repeat with permission
to query that metadata completed the native run.

## Observations

- Ordinary Rust suite: 24 wrapper unit tests passed, four hardware tests
  ignored; sys ABI/loader tests and two compile-fail doctests passed. The
  dispatch fixtures include descriptor construction, scalar/resource rejection,
  injected preflight errors with no native call or poison, unknown native-code
  preservation, poisoning, and terminal destroy retention after poison.
  Injected COM failure tests the wrapper policy, not a failing real DX12 call.
- Paired native dispatch ABI compilation and five-resource conversion comparison
  with AMD's inline DX12 helper passed.
- Public `Upscaler::dispatch` returned OK. Child status was `timeout=False
  exit=0`; list submission and fence completed. Readback reported 230,400
  finite pixels and 230,400 changed from the sentinel. Red channel minimum and
  maximum were `0.101745605` and `0.8486328`; left/right samples were
  `0.19475134` and `0.7568281`. Debug errors `0`; device removal `0`.

## Limits

This demonstrates one public-route dispatch, GPU completion and plausible
spatially varying output on the named configuration. It does not prove image
quality, nonzero-motion temporal behavior, varying exposure, other formats,
devices, drivers, SDK/provider releases or arbitrary in-flight capacity.
The [earlier GBV diagnostic](2026-09-24-exp-m6-gbv-dispatch-stall.md) timed out
inside native `ffxDispatch`; this run did not resolve that issue. Actual resource
states, heap aliasing, residency, GPU lifetime and submission order remain
unsafe caller obligations. The `GetDesc`, `GetDevice` and canonical `IUnknown`
checks add CPU-side work per dispatch; no performance measurement was made.
