# Windows follow-up

Prepared on macOS, 2026-10-18; public generation follow-up added 2026-10-19.
The [2026-10-19 execution report](release/2026-10-19-windows-upscaler-verification.md)
passes the consolidated checks and records mixed generator results: public
3.1.5 generation passes; 4.1.1/2.3.4 mask-write criteria fail despite OK returns.
Three startup-measurement runs also pass. Keep the checkout unchanged during each run; rebuild the helper and Rust executable
together through the coordinator. These checks do not publish anything.
[Preparation checks and limits](research/records/2026-10-18-exp-windows-followup-preparation.md).

## Prerequisites

- Windows x64 with an AMD DX12 GPU, its driver and the D3D12 debug layer.
- x64 MSVC build tools and Windows SDK, PowerShell 7, Python 3.11+, current Rust
  and exact Rust 1.88.0. Both Rust toolchains must support
  `x86_64-pc-windows-msvc`.
- Cached Cargo dependencies and a local pinned SDK v2.3.0 `Kits/FidelityFX`
  directory containing headers and both original `signedbin` DLLs. The runner
  checks header/DLL hashes and AMD signatures. See
  [SDK preparation](../tools/verification/README.md#commands-and-claims) for the
  coordinator's offline/online policy.

Run from the repository root. The examples use the existing Windows SDK cache;
substitute the actual `Kits/FidelityFX` path and desired adapter index.

## 1. Consolidated implementation checks

```powershell
python tools/verification/run.py run windows-upscaler-followup --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

This combines independent MSVC ABI assertions, infrastructure checks, current
Rust CI, exact MSRV, controlled/public consumers and examples, then the existing
native upscaler regressions. Source/consumer/MSRV failure blocks GPU execution.
It covers the pending D028/D029/D031/D032 source and fixture checks and now
compiles the paired production-generator ABI probe, both-family generator
fixtures and composition example under accepted D030. Public generator fixtures
are part of Windows CI; they do not execute the actual AMD provider.
[Local implementation checks](release/2026-10-19-reactive-mask-generation-verification.md).

The new `native.dispatch.configure` case creates a fresh context per tuning
parameter for each owner family. It requests an in-domain value, records and
fences an upscale, restores `ProviderDefault`, then records and fences another
upscale before explicit destruction. Both families give 20 completed frames.
Finite/written output and debug diagnostics check integration. A native
acknowledgement does not prove the numerical effect, clamping or actual default.
The movable control stays on one thread; existing movable regressions retain
their separate handoff coverage. Any native Configure rejection fails this case.

## 2. Isolated D030 raw spike

```powershell
python tools/verification/run.py run experiment-reactive-mask --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The test-only descriptor is checked independently against the pinned header by
MSVC. This spike retains its independent test-only descriptor; the public
generator API is implemented separately and this raw spike does not verify
that public path.
Eighteen fresh processes run an upload/readback control, identical-colour input
control and all 16 flag combinations. Each successful child uses one disposable
context, fixed maximum allocations and active regions
320×180 → 256×128 → 320×180. Inputs are linear RGBA16F; output is a distinct
R8_UNORM UAV, seeded to 227. Scalars are scale `1`, cutoff `0.2`, binary `0.9`.

Successful generation must overwrite the seed, leave pixels outside the active
region untouched and yield zero for identical colours. With flags zero,
distinct inputs must produce a nonzero signal. Other flag combinations record
their actual metrics without assuming a numerical formula. An explicit UAV
barrier and UAV → copy-source → shader-read transitions snapshot the mask before
the public upscaler consumes it. Submission, fence completion, finite/written
upscale output, debug diagnostics and explicit cleanup are required. This tests
the proposed boundary states; it does not silently repair a provider mismatch.
Submission of a supplied mask does not establish which texels the provider uses.

A genuine nonzero generator return is retained as `observed`, with the exact
code, provider, completed frames and cleanup result. The unsubmitted list is
discarded before one terminal destroy attempt; there is no retry or context
reuse. Failed destruction is `failed`. Crashes, timeouts, missing tests,
unchanged masks and failed GPU controls also fail rather than becoming support
observations. One rejected flag does not suppress the other independent children.
The equipment control must pass before generation children run.

`observed` exits zero by default so a fully captured unsupported result can be
inspected. Add `--require-passed` when a fully successful generation run is the
required outcome. Neither result proves other providers safe. D030 is now
accepted with local implementation; this spike still covers only its raw local-owner protocol.
The separate public-path profile now exercises both families with the same
controls, regions, scalars and all 16 flags:

```powershell
python tools/verification/run.py run experiment-public-reactive-mask --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Each family has an independent equipment control and 17 generation children;
failed mask-write criteria remain failures even when native dispatch returned OK.
No provider fallback or production capability policy is introduced. Final
extracted-package verification remains separate.

On the pinned deployment exposing providers `3.1.5` and `2.3.4`, test those exact
providers separately with `experiment-selected-reactive-mask` and the same
native/SDK/adapter arguments. Missing providers fail explicitly. Each
provider/family pair has its own control and all 16 flag combinations; the
effective live ID must match the requested selection.

## 3. Optional startup measurement

```powershell
python tools/verification/run.py run experiment-provider-setup --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

This existing bounded profile measures actual default/selected-provider setup
costs before deciding whether caching is useful. It is independent of GPU
generation; see [the protocol](DX12_RECOVERY.md#measure-provider-setup-before-caching).

## Preserve results

The coordinator prints the fresh attempt under `target/verification`. Retain its
`run.json`, case records and logs, including blocked/failed outcomes. Verify and
export each attempt to a new destination:

```powershell
python tools/verification/run.py verify-run --run target/verification/<attempt>
python tools/verification/run.py export --run target/verification/<attempt> --output target/verification-evidence/<attempt>
```

Integrity verification is separate from acceptance. Final extracted-package and
release checks remain required after the selected 0.3.0 surface is complete.
