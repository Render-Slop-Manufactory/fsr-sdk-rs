# Independent BGRA/sRGB DX12 views

This bounded experiment checks the view mappings investigated in the
[external source record](../../docs/research/records/2026-10-24-src-bgra-srgb-dx12-upscaling-access.md).
It loads no AMD runtime and accepts no new public wrapper format. The independent
C++ probe uses the pinned public helper only to observe allocation-derived native
metadata; its view table follows the inspected generic backend. It does not link
or execute that backend. Shader execution is direct DX12.

## Execution

From the repository root on Windows x64 with existing MSVC and Windows SDK tools:

```sh
python tools/verification/bgra_srgb.py --native --adapter-index 0 --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
```

Select the actual DXGI adapter index; zero is an example. The probe requires a
hardware FL12.0 device, ordinary debug layer, OPTIONS3 query and explicit native
opt-in. It acquires the existing developer environment through `windows.ps1` and
uses the shared Python runner, subprocess watchdog, source identity and evidence
inventory. It adds no automatic CI/release gate or SDK download. Every attempt
gets an exclusive directory under ignored `target/verification/`.

## Inputs and exact mappings

Use single-mip/slice/sample 8×8 Texture2D resources and default component mapping.
Each source texel alternates by `(x+y)%2` between logical RGBA code tuples
`(48,128,224,160)` and `(192,64,16,127)`. Uploads pack RGBA or BGRA bytes according
to allocation; padding is filled with `0xcc`. CPU evaluation independently checks
every real upload texel and its 256-byte-aligned copy pitch.

| Allocation | Backend-equivalent SRV | Backend-equivalent UAV | Native surface value |
|---|---|---|---|
| R8G8B8A8_UNORM | R8G8B8A8_UNORM | R8G8B8A8_UNORM | 10 |
| B8G8R8A8_UNORM | B8G8R8A8_UNORM | B8G8R8A8_UNORM | 14 |
| R8G8B8A8_UNORM_SRGB | R8G8B8A8_UNORM_SRGB | R8G8B8A8_UNORM | 12 |
| B8G8R8A8_UNORM_SRGB | B8G8R8A8_UNORM_SRGB | B8G8R8A8_UNORM | 15 |
| R8G8B8A8_TYPELESS | R8G8B8A8_UNORM | R8G8B8A8_UNORM | 9 |
| B8G8R8A8_TYPELESS | B8G8R8A8_UNORM | B8G8R8A8_UNORM | 13 |

## T1: shader-visible colour

Read every texel through both `Texture2D<float4>.Load` and point `SampleLevel` at
texel centres. Store 128 float4 results in a structured buffer, initially filled
with `0xa5`. Fence before mapping CPU readback. Alpha uses UNORM conversion,
including for sRGB SRVs. Typeless routes deliberately use the backend's UNORM
view rather than a caller-preferred sRGB view.

The CPU uses independent binary64 piecewise sRGB arithmetic. For byte `b`, let
`x=b/255`; decode `x/12.92` for `x<=0.04045`, otherwise
`((x+0.055)/1.055)^2.4`. UNORM and alpha absolute tolerance is `2e-5`.

**sRGB RGB acceptance uses code-space error, not that absolute float tolerance.**
[Direct3D functional specification, section 3.2.3.7](https://microsoft.github.io/DirectX-Specs/d3d/archive/D3D11_3_FunctionalSpec.htm)
permits 0.5 ULP on the sRGB side. Convert the measured float back with the ideal
inverse transfer **without integer rounding**, multiply by 255, and require its
distance from the uploaded code to be at most 0.5. Endpoints, when tested, must
be exact. The selected RGB codes exercise the nonlinear branch only and do not
establish full-range conversion conformance. The initial external proposal's
`2e-5` float threshold is retained only for UNORM/alpha; its failed initial
sRGB evaluation remains a failed historical attempt.

One extra BGRA input deliberately swaps physical red/blue bytes. Its measured
values must match the independently predicted swapped signal, and evaluation
against the ordinary signal must reject. This checks channel sensitivity.

## T2: storage and subsequent read

For each route query actual UNORM UAV texture/view/store bits and, for a typed
sRGB allocation, fully typed casting support. Attempt the actual allocation
with `ALLOW_UNORDERED_ACCESS`; retain allocation failure separately from a
positive view-support query. Create the exact SRV/UAV pair and write all texels
with an independent `RWTexture2D<float4>` shader storing
`(48/255,128/255,224/255,160/255)`.

Fence and audit raw texture copy readback. Every real texel must be exactly
`30 80 e0 a0` in RGBA storage or `e0 80 30 a0` in BGRA storage (hex). These
chosen values are centred on representable codes, so this is independent of
rounding at half-code boundaries. A missing write, R/B swap or automatic sRGB
encode fails this byte check. Ignore undefined row-padding contents.

Then read the stored texture through its exact SRV using T1's shader and CPU
criteria. This distinguishes unencoded UNORM storage from subsequent sRGB
decoding. Positive results apply to allocation creation, view use and this
specific independent shader, not an FSR provider.

## Outcomes and limits

Unexpected debug errors, device removal, missing/partial/non-finite readback,
wrong metadata, signal mismatch and an undetected swapped control fail.
Unavailable required tools or device setup cannot pass. Output support absence
or rejected allocation is retained as an observation, never a supported route;
the overall run becomes `observed` even if the other measurements pass. The
mandatory RGBA UNORM output must write. Fences have a 15-second limit and the
GPU subprocess a 90-second watchdog. Ordinary debug checks remain enabled; GBV
is not enabled.

This is not a full Direct3D conformance test, provider-support proof, resource
access-bound proof or wrapper verification. T3 direct FSR dispatch and T4
generator checks remain separate. An FSR format decision and both-family
wrapper implementation/verification still require their own reviewed contract.
