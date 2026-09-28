# Source investigation: M6 upscaler dispatch ABI

> Intake note (2026-09-24): renamed from `rename_me_1.md`. Opaque
> `:chatgpt-content-reference` markers had no supplied index-to-source map and
> were removed. The substantive findings and pinned source list were retained;
> this intake does not independently verify every claim.

Baseline: AMD’s `v2.3.0` tag is commit `60f4ea8`, released June 24, 2026; that release identifies the upscaler as AMD FSR Upscaling 4.1.1. The findings below use that tag as the ABI authority.

## 1. Required M6 ABI surface

| Status | Item | M6 relevance |
|---|---|---|
| **VERIFIED** | `ffxDispatch` | The only new exported FFX entry point intrinsically required for an upscale dispatch. |
| **VERIFIED** | `ffxContext`, `ffxReturnCode_t`, `ffxStructType_t` | Base ABI scalar/handle types. |
| **VERIFIED** | `ffxApiHeader` / `ffxDispatchDescHeader` | Base descriptor header. |
| **VERIFIED** | `ffxDispatchDescUpscale` | Actual upscale dispatch descriptor. |
| **VERIFIED** | `FfxApiResource`, `FfxApiResourceDescription` | Native GPU-resource representation. |
| **VERIFIED** | `FfxApiDimensions2D`, `FfxApiFloatCoords2D` | Inline dimension/vector values. |
| **VERIFIED** | `FFX_API_DISPATCH_DESC_TYPE_UPSCALE = 0x00010001` | Required `header.type`. |
| **VERIFIED** | resource type/usage/state/flags constants | Required to describe imported DX12 resources. |
| **VERIFIED** | `FfxApiSurfaceFormat` constants | Required by `FfxApiResourceDescription.format`. |
| **VERIFIED** | `FfxApiDispatchFsrUpscaleFlags` | Dispatch flags; zero is valid for the ordinary linear-color path. |
| **CONVENIENCE ONLY** | `ffxApiGetResourceDX12`, `ffxApiGetSurfaceFormatDX12` | C++ `static inline` helpers, not exported ABI functions. |
| **SAMPLE ONLY** | `SDKWrapper::ffxGetResourceApi` | Cauldron framework wrapper around the DX12 helper. |
| **NOT REQUIRED FOR M6** | `ffxQuery`, jitter queries, resource-requirement query | Useful utilities/provider introspection, but no dependency exists in the base dispatch ABI. |
| **NOT REQUIRED FOR M6** | `ffxConfigure` | No configuration call is required by `ffxDispatchDescUpscale`. |

The exact entry point is:

```c
ffxReturnCode_t ffxDispatch(
    ffxContext* context,
    const ffxDispatchDescHeader* desc);
```

**VERIFIED.** `ffxContext` itself is `void*`; consequently the first argument is a pointer to the stored opaque handle, not the handle value directly. AMD also declares the matching `PfnFfxDispatch` typedef with exactly the same signature.

**VERIFIED.** The header surrounds the API with `extern "C"` and defines `FFX_API_ENTRY` solely as `__declspec(dllexport)`—there is no `WINAPI`/`__stdcall`. An ABI-faithful Rust declaration is therefore `unsafe extern "C" fn`, particularly important if 32-bit support is ever attempted; `extern "system"` would not be the faithful spelling on x86.

The descriptor ID expands exactly as:

`FFX_API_EFFECT_ID_UPSCALE = 0x00010000`, sub-ID `0x01` → `FFX_API_DISPATCH_DESC_TYPE_UPSCALE = 0x00010001`.

---

## 2. Exact structure definitions

### Base header

**VERIFIED.** The common descriptor is:

| Offset x64 | C field | Exact C type |
|---:|---|---|
| 0 | `type` | `ffxStructType_t` = `uint64_t` |
| 8 | `pNext` | `struct ffxApiHeader*` |

`ffxCreateContextDescHeader`, `ffxConfigureDescHeader`, `ffxQueryDescHeader`, and `ffxDispatchDescHeader` are typedef aliases of this same `ffxApiHeader`; `type` must be set and `pNext` may be null.

### Dimension/vector types

**VERIFIED.**

| Type | Fields | C types |
|---|---|---|
| `FfxApiDimensions2D` | `width`, `height` | `uint32_t`, `uint32_t` |
| `FfxApiFloatCoords2D` | `x`, `y` | `float`, `float` |

Both are inline values, not pointers.

### `FfxApiResourceDescription`

**VERIFIED.** Exact field order is:

| Offset | Field/union slot | Exact C representation |
|---:|---|---|
| 0 | `type` | `uint32_t` |
| 4 | `format` | `uint32_t` |
| 8 | `width` / `size` | anonymous union of two `uint32_t` |
| 12 | `height` / `stride` | anonymous union of two `uint32_t` |
| 16 | `depth` / `alignment` | anonymous union of two `uint32_t` |
| 20 | `mipCount` | `uint32_t` |
| 24 | `flags` | `uint32_t` |
| 28 | `usage` | `uint32_t` |

The important ABI detail is that the fields holding format/type/state/etc. are explicitly `uint32_t`; they are not C enum-typed fields. This avoids dependence on the compiler's enum storage choice.

### `FfxApiResource`

**VERIFIED.**

| Field | Exact type |
|---|---|
| `resource` | `void*` |
| `description` | inline `struct FfxApiResourceDescription` |
| `state` | `uint32_t` |

There is no resource-name pointer or ownership field. `FFX_RESOURCE_NAME_SIZE` happens to precede the declaration but is not a member of this structure.

For DX12, AMD's helper directly assigns an `ID3D12Resource*` to the `void* resource` member, proving that no intermediate FFX resource object is required.

### `ffxDispatchDescUpscale`

**VERIFIED.** Exact field order and types from `ffx_upscale.h`:

| # | Field | Exact C type | Pointer? | Semantics / requirement |
|---:|---|---|---|---|
| 0 | `header` | `ffxDispatchDescHeader` | no | Mandatory; type `0x00010001`; base `pNext` normally null. |
| 1 | `commandList` | `void*` | native pointer | Mandatory for GPU dispatch. |
| 2 | `color` | `struct FfxApiResource` | resource contains pointer | Core required input, render resolution. |
| 3 | `depth` | `struct FfxApiResource` | same | Core required input. |
| 4 | `motionVectors` | `struct FfxApiResource` | same | Core required input. |
| 5 | `exposure` | `struct FfxApiResource` | same | Optional/conditional. |
| 6 | `reactive` | `struct FfxApiResource` | same | Optional for FSR4. |
| 7 | `transparencyAndComposition` | `struct FfxApiResource` | same | Optional for FSR4. |
| 8 | `output` | `struct FfxApiResource` | same | Required presentation-resolution output. |
| 9 | `jitterOffset` | `struct FfxApiFloatCoords2D` | no | Jitter actually applied while rendering. |
| 10 | `motionVectorScale` | `struct FfxApiFloatCoords2D` | no | Converts application MV convention to FSR's expected screen-space scale. |
| 11 | `renderSize` | `struct FfxApiDimensions2D` | no | Actual input render resolution. |
| 12 | `upscaleSize` | `struct FfxApiDimensions2D` | no | Header calls it optional; otherwise `maxUpscaleSize` is assumed. |
| 13 | `enableSharpening` | `bool` | no | Enables sharpening pass. |
| 14 | `sharpness` | `float` | no | `[0,1]`. |
| 15 | `frameTimeDelta` | `float` | no | Required; milliseconds. |
| 16 | `preExposure` | `float` | no | Must be `> 0.0f`. |
| 17 | `reset` | `bool` | no | Set on discontinuous camera transformation. |
| 18 | `cameraNear` | `float` | no | Near-plane distance. |
| 19 | `cameraFar` | `float` | no | Far-plane distance. |
| 20 | `cameraFovAngleVertical` | `float` | no | Vertical FOV, radians. |
| 21 | `viewSpaceToMetersFactor` | `float` | no | View-space-unit → metres factor. |
| 22 | `flags` | `uint32_t` | no | Bitset of dispatch flags. |

There are therefore **no pointer-valued optional image fields**: an absent exposure/reactive/composition mask is represented by an inline `FfxApiResource` whose `resource` pointer is null. AMD's sample constructs exactly such null resources.

### Mandatory/optional semantics

**CORROBORATED.** FSR4 documentation identifies color, depth and motion vectors as its core input-resource set. Color is render resolution and normally linear; depth is a single floating-point component; motion vectors are two-component floating point and may be render or presentation resolution depending on the context-creation flag.

**VERIFIED.** Exposure is optional when `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` was enabled during context creation; AMD recommends that mode.

**VERIFIED.** In FSR4 specifically, reactive and transparency/composition masks are no longer required, although they may still be supplied. The new resource-requirement Query exists mainly to support applications that also expose earlier FSR providers.

**VERIFIED.** Jitter is not optional semantically: AMD says the application must set `jitterOffset` whether it uses AMD's query helper or its own sequence generator. A custom generator is explicitly permitted, and AMD warns against a sequence producing `(0,0)`. Thus jitter Query is convenience, not dispatch ABI.

**VERIFIED.** `frameTimeDelta` is required in milliseconds. `reset=true` is prescribed for the first frame following a discontinuous camera transformation.

**CORROBORATED.** The v2.3.0 DX12 sample supplies jitter, MV scale, reset, sharpening, milliseconds, pre-exposure, both sizes and near/far/FOV; notably, the zero-initialized sample does **not** assign `viewSpaceToMetersFactor`.

### Expected MSVC ABI layout

The declarations contain no special packing attached to these structures. The following values are **INFERRED, high confidence** from the exact v2.3.0 declarations using default MSVC ABI rules; they should become VERIFIED by the native fixture described below rather than being trusted solely from arithmetic.

| Type | Win64 size / align | Win32 size / align |
|---|---:|---:|
| `ffxApiHeader` | 16 / 8 | 16 / 8 |
| `FfxApiDimensions2D` | 8 / 4 | 8 / 4 |
| `FfxApiFloatCoords2D` | 8 / 4 | 8 / 4 |
| `FfxApiResourceDescription` | 32 / 4 | 32 / 4 |
| `FfxApiResource` | 48 / 8 | 40 / 4 |
| `ffxDispatchDescUpscale` | **432 / 8** | **376 / 8** |

Expected top-level offsets:

| Field | Win64 | Win32 |
|---|---:|---:|
| `header` | 0 | 0 |
| `commandList` | 16 | 16 |
| `color` | 24 | 20 |
| `depth` | 72 | 60 |
| `motionVectors` | 120 | 100 |
| `exposure` | 168 | 140 |
| `reactive` | 216 | 180 |
| `transparencyAndComposition` | 264 | 220 |
| `output` | 312 | 260 |
| `jitterOffset` | 360 | 300 |
| `motionVectorScale` | 368 | 308 |
| `renderSize` | 376 | 316 |
| `upscaleSize` | 384 | 324 |
| `enableSharpening` | 392 | 332 |
| `sharpness` | 396 | 336 |
| `frameTimeDelta` | 400 | 340 |
| `preExposure` | 404 | 344 |
| `reset` | 408 | 348 |
| `cameraNear` | 412 | 352 |
| `cameraFar` | 416 | 356 |
| `cameraFovAngleVertical` | 420 | 360 |
| `viewSpaceToMetersFactor` | 424 | 364 |
| `flags` | 428 | 368 |

On Win64, `FfxApiResource.resource/description/state` are expected at `0/8/40`; on Win32, `0/4/36`. Both `bool` members occupy one byte under the expected MSVC ABI, followed by three bytes of padding before their subsequent `float`.

### Relevant constants

**VERIFIED. Resource types:** `BUFFER=0`, `TEXTURE1D=1`, `TEXTURE2D=2`, `TEXTURE_CUBE=3`, `TEXTURE3D=4`.

**VERIFIED. Resource usage:** `READ_ONLY=0`, `RENDERTARGET=1`, `UAV=2`, `DEPTHTARGET=4`, `INDIRECT=8`, `ARRAYVIEW=16`, `STENCILTARGET=32`, `DCC_RENDERTARGET=32768`.

**VERIFIED. Resource flags:** `NONE=0`, `ALIASABLE=1`, `UNDEFINED=2`.

**VERIFIED. ABI resource states:** `COMMON=1`, `UNORDERED_ACCESS=2`, `COMPUTE_READ=4`, `PIXEL_READ=8`, `PIXEL_COMPUTE_READ=12`, `COPY_SRC=16`, `COPY_DEST=32`, `GENERIC_READ=20`, `INDIRECT_ARGUMENT=64`, `PRESENT=128`, `RENDER_TARGET=256`, `DEPTH_ATTACHMENT=512`. These are FidelityFX state bits, not `D3D12_RESOURCE_STATES` values.

**VERIFIED. Dispatch flags:** `DRAW_DEBUG_VIEW=1`, `NON_LINEAR_COLOR_SRGB=2`, `NON_LINEAR_COLOR_PQ=4`; the two nonlinear-color flags are documented as mutually exclusive.

**VERIFIED. Surface-format discriminants**, in declaration order: `UNKNOWN=0`, `R32G32B32A32_TYPELESS=1`, `R32G32B32A32_UINT=2`, `R32G32B32A32_FLOAT=3`, `R16G16B16A16_FLOAT=4`, `R32G32B32_FLOAT=5`, `R32G32_FLOAT=6`, `R8_UINT=7`, `R32_UINT=8`, `R8G8B8A8_TYPELESS=9`, `R8G8B8A8_UNORM=10`, `R8G8B8A8_SNORM=11`, `R8G8B8A8_SRGB=12`, `B8G8R8A8_TYPELESS=13`, `B8G8R8A8_UNORM=14`, `B8G8R8A8_SRGB=15`, `R11G11B10_FLOAT=16`, `R10G10B10A2_UNORM=17`, `R16G16_FLOAT=18`, `R16G16_UINT=19`, `R16G16_SINT=20`, `R16_FLOAT=21`, `R16_UINT=22`, `R16_UNORM=23`, `R16_SNORM=24`, `R8_UNORM=25`, `R8G8_UNORM=26`, `R8G8_UINT=27`, `R32_FLOAT=28`, `R9G9B9E5_SHAREDEXP=29`, `R16G16B16A16_TYPELESS=30`, `R32G32_TYPELESS=31`, `R10G10B10A2_TYPELESS=32`, `R16G16_TYPELESS=33`, `R16_TYPELESS=34`, `R8_TYPELESS=35`, `R8G8_TYPELESS=36`, `R32_TYPELESS=37`, `R32G32_UINT=38`, `R8_SNORM=39`.

Return codes are `uint32_t`; `OK=0`, followed by generic error `1`, unknown descriptor `2`, runtime error `3`, no provider `4`, memory `5`, parameter `6`, provider-does-not-support-new-descriptor `7`.

### Provider/version-specific data

**VERIFIED.** There is **no provider/version field in `ffxDispatchDescUpscale`**. Version compatibility is a creation-time extension: v2.3.0 defines `ffxCreateContextDescUpscaleVersion { header; uint32_t version; }`, with `version = FFX_UPSCALER_VERSION`; the latter is `4.1.1`, packed as `0x01001001`. Official documentation says this creation extension has been necessary since SDK 2.1.

---

## 3. Resource conversion path

### `ID3D12Resource*` → `FfxApiResource`

**VERIFIED.** `ffxApiGetResourceDX12` is declared after `#if defined(__cplusplus)` and as `static inline`; it is therefore neither C ABI nor a DLL export. A Rust wrapper should replicate its small conversion logic rather than attempt to resolve it dynamically.

Its behavior is:

1. Zero-initialize `FfxApiResource`; assign `resource = pRes` and the caller-supplied FidelityFX `state`; null resource returns immediately.
2. Call `ID3D12Resource::GetDesc()`.
3. Buffers get `type=BUFFER`, `flags=NONE`, `usage=UAV`, `size=(uint32_t)Width`, and `stride=(uint32_t)Height`. AMD's Cauldron wrapper subsequently replaces the buffer stride with its own buffer metadata, which confirms that `SDKWrapper::ffxGetResourceApi` is framework adaptation rather than ABI.
4. Textures get `flags=NONE`; D16/D32 depth formats imply `DEPTHTARGET`, D24S8/D32S8 imply `DEPTHTARGET|STENCILTARGET`, otherwise initial usage is `READ_ONLY`; `ALLOW_UNORDERED_ACCESS` adds `UAV`.
5. `width=Width`, `height=Height`, `depth=DepthOrArraySize`, `mipCount=MipLevels`. Thus for a 2D array the ABI field named `depth` actually contains `DepthOrArraySize`.
6. D3D12 1D/2D/3D map to corresponding FFX resource types; a 2D resource with exactly six array slices is classified as `TEXTURE_CUBE`.
7. `format = ffxApiGetSurfaceFormatDX12(desc.Format)` and caller-provided `additionalUsages` is ORed into `usage`.

The DX12 format helper explicitly maps supported DXGI formats and returns `FFX_API_SURFACE_FORMAT_UNKNOWN` for anything else. Important mappings include D32 and D32S8 families → `R32_FLOAT`, D24S8 family → `R32_UINT`, stencil views → `R8_UINT`, D16 → `R16_UNORM`, plus the expected RGBA32/RGBA16/RG32/RGBA8/BGRA8/RG16/R32/RG8/R16/R8/R9G9B9E5 mappings. Notably, existence of an FFX format enumerant does **not** guarantee the DX12 helper has a case for it; for example the source leaves several DXGI cases commented out.

For M6 I would reproduce that helper's switch exactly and fixture-test it rather than create an independent "reasonable" DXGI mapping.

### Resource states

**VERIFIED.** AMD's FSR4 integration guide says DX12 **input** resources should be transitioned to `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE` before the upscale call. The corresponding FidelityFX state supplied to the descriptor is `FFX_API_RESOURCE_STATE_COMPUTE_READ`.

**CORROBORATED.** The official Cauldron sample instead happens to keep its resources in combined pixel/non-pixel shader-readable state and therefore describes them as `FFX_API_RESOURCE_STATE_PIXEL_COMPUTE_READ`; this shows the `state` field should describe the actual state at the boundary rather than always using the helper's default.

**UNKNOWN.** The inspected v2.3.0 public material does not give a simple general rule for the output texture's required pre-dispatch D3D12 state or promise a particular post-dispatch state. The sample supplies the output with its actual combined-read state and lets the FidelityFX backend perform the work. Do not invent an external UAV-state requirement from older FSR APIs.

**INFERRED, strong.** The output native texture should be created UAV-capable because the DX12 helper marks a texture as `FFX_API_RESOURCE_USAGE_UAV` precisely when `D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS` is set and an upscaler must write the output. I did not find a v2.3.0 FSR4 sentence explicitly stating "output must have ALLOW_UNORDERED_ACCESS", so this should be validated by the real-runtime test/debug layer rather than promoted to a sourced ABI rule.

### Command list

**VERIFIED.** ABI representation is simply `void* commandList`.

**CORROBORATED.** The v2.3.0 DX12 sample assigns `pCmdList->GetImpl()->DX12CmdList()`, whose concrete Cauldron accessor returns `ID3D12GraphicsCommandList2*`. The ABI does not encode that COM interface type; it remains erased to `void*`.

There is no resource-conversion object for the command list and no `ffxApiGetCommandListDX12` equivalent required by this API.

### GPU synchronization

**VERIFIED.** `ffxDispatch` encodes GPU work into the command list supplied in the descriptor; no queue, fence, completion callback, or synchronization token is part of this dispatch ABI.

**INFERRED.** Therefore a safe Rust CPU borrow ending when `ffxDispatch` returns cannot establish GPU completion. With no GPU synchronization abstraction in the project, the first high-level dispatch boundary should either remain `unsafe` or explicitly place the burden of command-list/resource/context lifetime and D3D12 state correctness on the caller. Retaining COM objects only for the duration of the Rust call is insufficient as a general GPU-lifetime guarantee.

---

## 4. Likely `fsr-sdk-sys` binding delta

Based only on the repository state you supplied, the additions beyond the existing creation/destruction surface are:

1. **VERIFIED:** dynamically load/export a `PfnFfxDispatch` equivalent:
   ```rust
   unsafe extern "C" fn(
       context: *mut ffxContext,
       desc: *const ffxDispatchDescHeader,
   ) -> ffxReturnCode_t
   ```
   with `ffxContext` itself representing the opaque `void*`. Do not change this to a single-level opaque pointer.

2. **VERIFIED:** add `ffxDispatchDescUpscale` and `FFX_API_DISPATCH_DESC_TYPE_UPSCALE`.

3. **VERIFIED:** if not already present from M1–M5, add `FfxApiResource`, `FfxApiResourceDescription`, their anonymous-union slots, `FfxApiDimensions2D`, `FfxApiFloatCoords2D`, resource types/usages/flags/states and surface-format constants.

4. **VERIFIED:** add the three `FfxApiDispatchFsrUpscaleFlags` constants.

5. **CONVENIENCE, NOT ABI:** implement a private Rust equivalent of `ffxApiGetSurfaceFormatDX12`/`ffxApiGetResourceDX12`. There is no symbol to bind.

6. **DO NOT BIND FOR M6:** `SDKWrapper::ffxGetResourceApi`, `ffx::Dispatch`, `ffx::DispatchDescUpscale`, and `InitHelper`. The first is Cauldron code; the latter three are C++ wrappers around the `.h` ABI. AMD explicitly describes `ffx_upscale.hpp` as helper types while the API definition resides in `.h`.

7. **DO NOT ADD YET solely for dispatch:** Query/configure descriptor types or the autoreactive-mask dispatch. FSR4 permits the two masks to be absent, and caller-generated jitter is supported.

At `fsr-sdk` level, the minimum defensible abstraction is correspondingly small: a DX12 dispatch-parameter object containing native resources plus explicit current FFX resource states and scalar camera/timing/jitter information, with the actual dispatch operation retaining an unsafe GPU-lifetime/state contract rather than inventing a resource/fence framework.

---

## 5. ABI verification plan

### Rust compile-time checks

For `windows + x86_64-msvc`, assert `size_of`, `align_of`, and `offset_of!` for:

- `ffxApiHeader`: `16/8`, `type=0`, `pNext=8`.
- `FfxApiResourceDescription`: `32/4`, offsets `0,4,8,12,16,20,24,28`.
- `FfxApiResource`: expected `48/8`, offsets `0,8,40`.
- `FfxApiDimensions2D` and `FfxApiFloatCoords2D`: `8/4`.
- `ffxDispatchDescUpscale`: expected `432/8` and every offset in the table above.
- the descriptor ID `0x00010001`, all resource state/usage/type constants, dispatch flags and whichever format constants the DX12 converter supports.

These are **INFERRED layout expectations from the upstream declarations**, not substitutes for a native reference.

### Native C/C++ fixture

This should be the authoritative layout oracle, built against the **exact v2.3.0 headers with MSVC**:

```text
static_assert(sizeof(...))
static_assert(alignof(...))
static_assert(offsetof(...))
static_assert(sizeof(bool) == 1)
static_assert(FFX_API_DISPATCH_DESC_TYPE_UPSCALE == 0x00010001)
static_assert(std::is_same_v<decltype(&ffxDispatch), PfnFfxDispatch>)
```

Export fixture functions returning the same `sizeof`/`alignof`/`offsetof`/macro values and compare them from Rust as well; that catches accidental cfg-dependent Rust layouts instead of merely proving that the C++ fixture compiled.

A second native helper-parity fixture should call **AMD's actual inline** `ffxApiGetSurfaceFormatDX12` and `ffxApiGetResourceDX12` on representative D3D12 textures, then compare every returned Rust field. This is especially valuable for depth/stencil mappings, cube detection, UAV usage and null resources.

### Real-runtime M6 fixture

The acceptance test should go past `FFX_API_RETURN_OK`:

- create separate known color/depth/MV/output textures;
- enable the D3D12 debug layer;
- transition the required inputs correctly;
- build the descriptor through the Rust conversion path;
- call the real `ffxDispatch`;
- close and execute the command list;
- signal/wait a fence before destroying/readback;
- copy the output to a readback resource;
- verify a deterministic nontrivial image property rather than merely successful dispatch.

That last point matters: the current creation/destruction tests establish CPU-side ABI/lifetime correctness, but they cannot detect a malformed resource descriptor that happens not to crash until GPU execution.

---

## 6. Minimal first-dispatch boundary

The smallest v2.3.0 FSR4/DX12 path I would target is:

1. Use the already-created upscale context, with its creation flags and v2.3.0 `ffxCreateContextDescUpscaleVersion` chain already correct. For the simplest M6 fixture, use `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE`.
2. Supply four distinct native textures: color, depth, motion vectors and output. Distinct color/output matches AMD's sample and avoids relying on unproven aliasing support.
3. Transition color/depth/MV to `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`, and describe them as `FFX_API_RESOURCE_STATE_COMPUTE_READ`.
4. Convert each `ID3D12Resource*` by faithfully reproducing `ffxApiGetResourceDX12`; describe the output's actual current state, not a fabricated one.
5. Use null `FfxApiResource`s for exposure, reactive and transparency/composition. Exposure omission is valid under auto-exposure; both masks are optional for FSR4.
6. Set `renderSize` and explicitly set `upscaleSize` rather than relying on its optional fallback for the first implementation.
7. Render with a valid nonzero subpixel jitter and place that same applied offset in `jitterOffset`; supply an appropriate `motionVectorScale`. For NDC-space motion vectors AMD's example uses render width/height as that scale.
8. Supply positive `preExposure`, milliseconds in `frameTimeDelta`, appropriate near/far/FOV, sharpening disabled initially, `flags=0`, and `reset` according to camera-history continuity.
9. Set `header.type=0x00010001`, `header.pNext=null`, `commandList=<native DX12 graphics command-list pointer>` and call `ffxDispatch(&context, &desc.header)`.
10. Execute and fence the command list before readback or releasing GPU-referenced objects.

**VERIFIED:** no Query or Configure call appears in that dependency chain. AMD's generic API documentation defines dispatch independently, while jitter can be generated by the application and FSR4's mask requirement query exists for provider-dependent integrations.

---

## 7. Uncertainties

**UNKNOWN — exact `viewSpaceToMetersFactor` requirements for FSR4.** The ABI comment defines its meaning, but the v2.3.0 FSR4 integration material inspected does not provide validation bounds/default semantics. The official sample zero-initializes the descriptor and never overwrites this field, which shows what the sample does but is not enough to generalize a semantic rule.

**UNKNOWN — input/output aliasing.** The ABI does not state whether `color.resource == output.resource` is supported. AMD's DX12 sample explicitly copies its color target to a temporary input before upscaling back into the target, so M6 should use separate resources rather than assume aliasing works.

**UNKNOWN — general output pre/post state guarantee.** The sources establish input transitions and show the sample's output state, but I found no v2.3.0 public contract saying exactly what state every output must enter with or what state it is guaranteed to have after dispatch. Treat the supplied state as truthful current-state metadata and test this with the DX12 debug layer.

**INFERRED — UAV-capable output.** This is strongly indicated by the DX12 import helper and the nature of the output write, but I would not encode it as a claimed header-level contract until the native runtime fixture confirms it.

**INFERRED — sizes/offsets above.** They follow the declarations under normal MSVC packing and match the expected Windows ABI, but AMD does not publish static size/offset assertions in the inspected public headers. The native MSVC fixture should be the project's source of truth before freezing Rust compile-time assertions.

**VERIFIED — no hidden provider field in the dispatch descriptor.** Provider/version selection occurs through context creation/version machinery, not through an extra FSR4 dispatch member.

Tag-pinned primary files: [ffx_api.h @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api.h?utm_source=chatgpt.com) · [ffx_api_types.h @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api_types.h?utm_source=chatgpt.com) · [ffx_api_dx12.h @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h?utm_source=chatgpt.com) · [ffx_upscale.h @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com) · [FSR 4.1.1 integration guide @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md?utm_source=chatgpt.com) · [official DX12 FSR sample @ v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp?utm_source=chatgpt.com).
