# Source investigation: audit of the BGRA/sRGB external findings

- Investigation/access date: 2026-10-24.
- Question: which incoming BGRA/sRGB findings survive comparison with the exact
  pinned SDK, and what needs correction before native format decisions?
- Method: inspect current wrapper/manifests, compare six local SDK files with
  fresh immutable upstream content, trace helper/backend/shader symbols, check
  Microsoft specifications and narrowly corroborate integration examples.
- This record contains source evidence. The separately linked
  [DX12 experiment](2026-10-24-exp-bgra-srgb-views.md) records actual execution.
  No public format contract or implementation is accepted here.

## Inputs and baseline

The incoming [external record](2026-10-24-src-bgra-srgb-dx12-upscaling-access.md)
is preserved unchanged: 50,249 bytes, SHA-256
`9230d235205f2f23b1f820e4077daf4e12473d1808471859efbdabea1d351ca4`.
Its supplied project ZIP has no independently validated identity here.

Current source at audit start: `7dbde52`, with that incoming record untracked.
The manifests now select wrapper `0.3.0` / sys `0.1.2` under amended D031,
rather than the external brief's earlier sys `0.2.0`. This is a baseline drift,
not a format-research finding. The SDK remains v2.3.0 at
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The shared converter in
`crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs` still rejects the new
BGRA/sRGB formats. Existing D034/D035/D037 access remains unchanged.

All six files below were fetched from `raw.githubusercontent.com` at that exact
commit and compared to `external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/`.
Comparison normalizes CRLF to LF before hashing. Every comparison matched.
The first restricted-network attempt failed at DNS; the approved retry passed.
No local SDK file was replaced.

| Path under `Kits/FidelityFX/` | SHA-256 of UTF-8/LF source |
|---|---|
| `api/include/ffx_api_types.h` | `cc0adc0ffc804dd7cd03c39c15581b575e416383177ffe95060201d52cc0fc49` |
| `api/include/dx12/ffx_api_dx12.h` | `c9afbc3c4c673723e9e5369a666f0ff54560543b5a93537f8ff453bf1c3bf530` |
| `backend/dx12/ffx_dx12.cpp` | `3f6ffa9079caa62a10abb4e108fc833c34b555bc1affcec0007dbc623536be77` |
| `upscalers/include/ffx_upscale.h` | `65be36d5653eb71c10379e0204f7bb7cd6a8aeec96e6f8cffc5f59ffeb1f89bf` |
| `upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h` | `61bf134d78b72ab81a7e39ce9481d73027f91941137bb6d69e4354830b476901` |
| `docs/techniques/super-resolution-ml.md` | `9c761f10d1a0a6aba02cde98adef10229f4800500e36b58f0bcb5424778f2d02` |

## Citation corrections

**Verified source facts:** several ranges described as checked in the external
report miss their claimed symbols. The immutable source itself is available;
there is no reason to attribute this mismatch to a different SDK version.

| Incoming reference | Claimed location | Actual relevant location in pinned file |
|---|---|---|
| S1 formats | types 24–44 | enum 29 onward; BGRA values extend through 46 |
| S2 resource helper | DX12 header 183–243 | helper starts 204; final format and additional usage assignments are 267–268 |
| S3a native-to-DXGI mapping | backend 585–628 | `ffxGetDX12FormatFromSurfaceFormat` starts 637 |
| S3b UAV conversion | backend 455–503 | `convertFormatUav` starts 502 |
| S3c SRV conversion | backend 505–552 | `convertFormatSrv` starts 554 |
| S3d format patch | backend 1904–1936 | `patchDxgiFormatWithFfxUsage` starts 2097 |
| S3e resource registration | backend 1959–2163 | definition starts 2156; view formats assigned 2194–2199; texture SRV/UAV creation 2343–2380 |
| S4b generation descriptor | header 138–151 | `ffxDispatchDescUpscaleGenerateReactiveMask` starts 153 |
| S5a input callbacks | shader 389–401 | `LoadInputColor`/`SampleInputColor` 421–429 |
| S5b output callback | shader 477–484 | `StoreUpscaledOutput` 517–520 |
| S5c opaque colour | shader 768–775 | `LoadOpaqueOnly` starts 838 |
| S6 linear guidance | guide 42–47 | linear-input recommendation at 52 |

These are substantive citation-quality faults. They do not, by themselves,
falsify the traced mappings. Correct direct references are:
[public helper](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h#L204-L269),
[UAV/SRV conversions](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L502-L605),
[patch and registration](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L2097-L2380),
and [colour callbacks](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h#L417-L520).

## Confirmed source findings

**Verified:** allocation-derived native values are RGBA8 typeless 9, UNORM 10,
sRGB 12; BGRA8 typeless 13, UNORM 14, sRGB 15. The public helper derives usage
from actual allocation flags and passes no caller-owned view.

For ordinary helper metadata, typed sRGB allocations retain an sRGB SRV and
receive a same-family UNORM UAV. Typeless RGBA8/BGRA8 allocations receive UNORM
SRVs and UAVs. Typed BGRA UNORM keeps BGRA UNORM views. The backend constructs
the views, creates an SRV for texture registration and adds UAV descriptors when
the real resource has the UAV flag. This also matters for UAV-capable inputs;
calling a role read-only does not suppress those descriptor requirements.

The patch function honours native format metadata for fully typed RGBA8
UNORM/sRGB, but not BGRA or the two typeless families. A public override cannot
generically select an sRGB SRV for those routes. No new view API follows from
finding that backend limitation.

The open FSR3 callbacks consume RGB and store alpha 1 in the identified output
callback. This establishes source behaviour for that callback, not every closed
provider or sharpening path. Typed sRGB view decoding and native non-linear
creation/dispatch hints remain separate operations. Their combination must be
chosen for the signal the provider actually receives; no automatic hint change
is justified by a texture's name alone.

## DX12 rules and a protocol correction

Microsoft's [relaxed casting specification](https://microsoft.github.io/DirectX-Specs/d3d/RelaxedCasting.html)
supports same-family typed reinterpretation subject to OPTIONS3 and its
exceptions, and describes optional BGRA UAV support queried per device.
The [FL12.0 format tables](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/hardware-support-for-direct3d-12-0-formats)
distinguish UNORM typed stores from unsupported sRGB UAV formats.
The [resource flags documentation](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ne-d3d12-d3d12_resource_flags)
also constrains resource creation. Therefore positive UNORM view bits must not
be reported as proof that a typed-sRGB allocation with UAV permission was
created. The independent T2 probe records allocation and actual view execution
separately; its bounded success is in the experiment record.

**Correction to the proposed experiment:** an sRGB SRV is not required to return
the ideal decoded binary64 result within `2e-5` absolute float error. The
[Direct3D functional specification, section 3.2.3.7](https://microsoft.github.io/DirectX-Specs/d3d/archive/D3D11_3_FunctionalSpec.htm)
defines tolerance on the sRGB side: invert the measured result without integer
rounding and measure against the input code, with a 0.5-code budget for 8-bit
input. Endpoint requirements are separate. This source-defined criterion,
documented before confirmation, replaces the external report's guessed
float-precision threshold for sRGB RGB only. UNORM/alpha keep `2e-5`.

## Integration practice and decisions

The [wgpu v28.0.1 texture descriptor](https://github.com/gfx-rs/wgpu/blob/v28.0.1/wgpu-types/src/texture.rs#L483-L492)
does distinguish allocation format and compatible view interpretation. Its
public view-format option does not establish that this SDK backend can consume
arbitrary views.

A concrete [pinned AMD/Cauldron renderer integration](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp)
uses renderer-owned temporary colour and output targets and sets creation and
per-dispatch colour options separately (`UpdateFSRContext` and `Execute`). This
supports preserving resource/signal distinctions; it is not an independent
cross-project consensus or a verified recipe for a new format. No additional
community-wide convention is needed to begin the narrow DX12 probes.

**Recommendation, not acceptance:** retain logical shader RGB, actual allocation
metadata, independent explicit colour controls and renderer-owned conversions.
Complete T3/T4 per-role provider probes before proposing the new admitted-format
matrix. Treat typeless default UNORM access and a desired sRGB reinterpretation
as distinct requirements. The external staged recommendation remains a proposal.

## Baseline delta and remaining work

The central backend mappings survive inspection. Citation ranges, the stale sys
version and the sRGB test precision need correction. The incoming record is
retained rather than rewritten. Source inspection adds neither native-provider
compatibility nor wrapper support; the first two direct DX12 probes now have a
separate execution record.

Still open: actual FSR main-input/output acceptance and signal behaviour (T3),
both generator colour-input roles on the writing provider (T4), and only if exact
CPU mask expectations are claimed, tracing the selected generator arithmetic.
These are narrower than a new broad online survey. Full PQ/exposure semantics,
quality studies, other format families and public API design are not selected
by this audit. No downloaded AMD material is redistributed; original probe code
uses the existing local headers and their retained terms.
