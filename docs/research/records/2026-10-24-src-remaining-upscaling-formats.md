# Source investigation: remaining external upscaling formats

- Investigation date and evidence cut-off: 2026-10-24.
- Baseline: checkout `a6ca2b7` with D034/D035/D037/D038 implemented; SDK
  v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Question: account for every remaining D035 format family, distinguishing
  missing wrapper access, exact backend views and formats outside the external
  upscaling contract. This is not an inventory of all possible DXGI textures.
- Method: inspect the pinned public helper, full 40-entry surface enum, generic
  DX12 backend, external-input documentation and open shader resource types.
  No image-quality, reconstruction, mask-algorithm or performance investigation.

## Inputs

AMD files below are from the exact commit above, inspected in the local SDK
deployment. Links identify that immutable source; accessed 2026-10-24.

1. [Surface enum and resource metadata](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h),
   `FfxApiSurfaceFormat`, all values 0–39.
2. [Public DX12 helper](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h),
   `ffxApiGetSurfaceFormatDX12` and `ffxApiGetResourceDX12`.
3. [Generic DX12 backend](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp),
   `convertFormatSrv`, `convertFormatUav`, `ffxGetDX12FormatFromSurfaceFormat`,
   `patchDxgiFormatWithFfxUsage` and `RegisterResourceDX12`.
4. [External-input contract](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md#input-resources)
   and [open FSR3 callbacks](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h),
   external float colour, scalar depth, two-component motion, exposure and masks.
5. Microsoft [format requirements](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/hardware-support-for-direct3d-12-1-formats),
   [planar depth/stencil](https://microsoft.github.io/DirectX-Specs/d3d/PlanarDepthStencilDDISpec.html),
   [fully typed casting](https://microsoft.github.io/DirectX-Specs/d3d/RelaxedCasting.html)
   and [revised shared-exponent support](https://microsoft.github.io/DirectX-Specs/d3d/D3D12R9G9B9E5Format.html),
   accessed 2026-10-24. These are current online specifications, not SDK pin inputs.

Local AMD source identities, SHA-256 after LF normalization:

| Input | SHA-256 |
|---|---|
| `api/include/ffx_api_types.h` | `cc0adc0ffc804dd7cd03c39c15581b575e416383177ffe95060201d52cc0fc49` |
| `api/include/dx12/ffx_api_dx12.h` | `c9afbc3c4c673723e9e5369a666f0ff54560543b5a93537f8ff453bf1c3bf530` |
| `backend/dx12/ffx_dx12.cpp` | `3f6ffa9079caa62a10abb4e108fc833c34b555bc1affcec0007dbc623536be77` |
| `docs/techniques/super-resolution-upscaler.md` | `402a62efb47c96f4cbd47b639fa126051584f4981e9b1d446865d87508c73f2c` |
| `upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h` | `61bf134d78b72ab81a7e39ce9481d73027f91941137bb6d69e4354830b476901` |

## Findings

The public helper is not a role whitelist. Registration reads the real resource
description and creates its fixed typed views. Surface metadata does not permit
an arbitrary caller-selected view. Device requirements must refer to those
views rather than the typeless allocation's support bits.

### Remaining applicable routes

| Allocation | Fixed shader interpretation | Native surface metadata | Applicable external roles |
|---|---|---|---|
| `R32G32B32A32_TYPELESS` | RGBA32F | 1 | Main/generation colour, output |
| `R16G16B16A16_TYPELESS` | RGBA16F | 30 | Same |
| `R10G10B10A2_TYPELESS` | RGB10A2 UNORM | 32 | Same |
| `R8G8B8A8_TYPELESS` | RGBA8 UNORM | 9 | Same; never selects an sRGB SRV |
| `B8G8R8A8_TYPELESS` | BGRA8 UNORM | 13 | Same; never selects an sRGB SRV |
| `R8G8B8A8_SNORM` | RGBA8 SNORM | 11 | Same, with compatible signals |
| `R9G9B9E5_SHAREDEXP` | Shared-exponent RGB | 29 | Same; output/UAV capability is optional |
| `R32G32B32_FLOAT`, `R32G32B32_TYPELESS` | RGB32F | Explicit 5 | Main/generation colour, load-only, no UAV/output |
| `R32G32_TYPELESS`, `R16G16_TYPELESS` | RG32F, RG16F | 31, 33 | Motion |
| `R16_TYPELESS` without depth/stencil capability | R16F | 34 | Scalar floating device depth |
| `R32_TYPELESS` without depth/stencil capability | R32F | 37 | Device depth or explicit exposure |
| `R8_TYPELESS` | R8 UNORM | 35 | Supplied masks or generated-mask output |
| `D16_UNORM` | R16 UNORM SRV | 23; depth usage 4 | Device depth, with fully typed casting |
| `D24_UNORM_S8_UINT`, `R24G8_TYPELESS` | R24 UNORM depth-plane SRV | 8; usage 36 or 0 | Device depth; casting for typed allocation |
| `R16_UNORM` | R16 UNORM | 23 | Scalar depth in the same normalized representation |

These are source-derived applicability recommendations. Execution and design
acceptance are separate. In particular:

- **D24 metadata is not an integer depth view.** The helper collapses it to
  native `R32_UINT`, but the backend patches from the real D24 allocation to
  `R24_UNORM_X8_TYPELESS`. Shader depth is a normalized floating value. Preserve
  the actual allocation for plane/support checks. D16 similarly reads UNORM
  depth as float; the documented scalar float signal need not have float storage.
- **R16 typeless is not D16 access.** The backend always selects R16F for that
  allocation. It does not inspect the depth flag to select UNORM. Reject a
  depth-capable R16 typeless allocation rather than silently interpreting its
  D16 production as half-float bits. Typed D16 is the available direct route.
- **RGB32F is a helper omission.** Its public enum and reverse conversion exist;
  the helper's RGB32 cases are commented out and return UNKNOWN. Registration
  preserves the real typed allocation and converts RGB32 typeless to FLOAT.
  Use explicit enum 5 for these two checked allocations. This does not authorize
  overriding unrelated formats. RGB32F supports texture loads but neither
  filtering nor typed UAV access in the referenced format requirements. A
  provider requiring filtered access cannot use this route. Generator and main
  operation support must be checked independently in bounded native execution.
- **Shared exponent is not input-only in all environments.** Microsoft's revised
  specification adds optional UAV stores. Require actual typed-view/store bits
  for output and view support for UAV-capable inputs. Do not prohibit all outputs
  based on an older format table or assume the current adapter's bits transfer.
- Typeless defaults provide existing float/UNORM shader representations. They
  add allocation access, not new exposure/mask encodings or normalized motion.

### Complete surface-enum accounting

This partitions all 40 native values; names below omit `FFX_API_SURFACE_FORMAT_`.
The enum inventory alone does not select all values for all roles.

| Values | Names / family | Disposition |
|---|---|---|
| 0 | UNKNOWN | No concrete checked texture representation |
| 1, 3, 4, 5 | RGBA32 typeless/float, RGBA16F, RGB32F | Colour routes above or already implemented; RGB32F input only |
| 9–17 | RGBA8 typeless/UNORM/SNORM/sRGB, BGRA8 typeless/UNORM/sRGB, R11G11B10F, RGB10A2 UNORM | Colour routes above or already implemented |
| 29, 30, 32 | Shared exponent, RGBA16 typeless, RGB10A2 typeless | Remaining colour routes above |
| 6, 18, 31, 33 | RG32F, RG16F and their typeless allocations | Two floating motion components |
| 21, 23, 28, 34, 37 | R16F, R16 UNORM, R32F, R16/R32 typeless | Depth routes; exposure only the documented R32F interpretation |
| 25, 35 | R8 UNORM, R8 typeless | Documented mask representation |
| 8 | R32 UINT | No integer depth allocation; D24's collapsed metadata is the explicit allocation-dependent exception |
| 2, 7, 19, 20, 22, 27, 38 | RGBA32 UINT, R8 UINT, RG16 UINT/SINT, R16 UINT, RG8 UINT, RG32 UINT | Generic integer shader resources; no matching documented external float-colour/depth/motion or explicit exposure/mask representation |
| 24, 26, 36, 39 | R16 SNORM, RG8 UNORM/typeless, R8 SNORM | Generic normalized resources; no documented alternative exposure/mask encoding, no two floating-storage motion representation, no fixed depth-buffer route |

The last two rows are **outside the selected documented external role contract**,
not evidence that no specially constrained provider could ever consume them.
U05 concerns applicable pinned upscaling access; it does not promise arbitrary
format reinterpretation through accidental backend behaviour.

Additional helper aliases are accounted for: `A8_UNORM` is alpha-only and does
not expose mask data in the shader's red component; stencil-only views expose
the wrong plane/type. Depth SRV-only formats are views of depth allocations,
not additional ordinary resource allocation routes. Existing D037 covers D32
and D32S8. Other DXGI formats whose helper returns UNKNOWN and which have no
corresponding native format are not silently assigned a different enum; RGB32F's
explicit native representation is the identified exception.

## Baseline delta and limits

D035's residual rows can be resolved by the explicit routes and exclusions
above. Neither UNKNOWN from the helper nor membership in the converter is a
sufficient blanket conclusion. This source record establishes no closed-provider
execution, shader access bounds, image semantics or cross-device support.
The [bounded experiment](2026-10-24-exp-remaining-upscaling-formats.md) supplies
separate observations. AMD source/binaries retain their existing terms; no
dependency, redistribution, SDK pin or acquisition change is proposed.
