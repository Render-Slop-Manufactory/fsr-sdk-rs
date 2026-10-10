# DX12 Upscaler API Surface of AMD FidelityFX SDK v2.3.0 and Rust Wrapper Design Implications

> Intake note (2026-09-29): normalized `new-record_1.md` to the dated source-record
> name and removed URL tracking parameters. Substantive findings and the supplied
> v0.1.0 baseline links are preserved. Selected declarations in the pinned
> `ffx_upscale.h` and `ffx_api.h`, plus Query/Configure routing in `ffx_api.cpp`
> and `GetDevice` in `ffx_backends_dx12.cpp`, were checked during integration;
> the full report was not independently reverified. Several implementation
> claims lack direct inline source links; the earlier
> [provenance repair](2026-09-22-src-m5-construction-query-provenance-repair.md)
> supplies relevant pinned implementation references. Proposed Rust APIs and
> milestones below are sketches, not implemented APIs or accepted project plans.

**Date:** 2026-09-29  
**Research type:** source-only investigation; no implementation changes and no new runtime experiments  
**Project baseline:** `fsr-sdk-rs` v0.1.0  
**AMD SDK baseline:** FidelityFX SDK v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`  
**Platform scope:** `x86_64-pc-windows-msvc`, modern FidelityFX API, DX12 upscaling only

## 1. Scope and evidence model

This record investigates the complete public modern upscaler surface relevant to a stable Rust wrapper: context creation, DX12 backend descriptors, provider selection, dispatch, reactive-mask generation, Query, Configure, descriptor extension chains, C++ helpers, and the relationship to the older direct FSR3 API. Other effects, Vulkan, engine integrations, speculative caching, and implementation work are outside scope.

Evidence labels used below are:

- **Verified source fact** — directly established by a pinned public header or inspected pinned source.
- **Upstream claim** — stated by AMD's pinned documentation or sample but not independently established as a universal implementation contract.
- **Inference** — wrapper/design conclusion derived from pinned facts.
- **Unresolved** — not established by the public pinned material, particularly for signed/external providers or runtime image correctness.

The existing repository explicitly distinguishes research from accepted design decisions and records the narrow dispatch verification as bounded evidence rather than general temporal correctness.

## 2. Pinned inputs and method

Existing project research was reviewed before re-investigating upstream material:

- `docs/research/README.md` at v0.1.0. [Pinned project research index](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/research/README.md)
- `docs/research/api-and-safety.md`. [Pinned API and safety synthesis](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/research/api-and-safety.md)
- `docs/research/upscaling-dispatch.md`. [Pinned dispatch synthesis](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/research/upscaling-dispatch.md)
- Query/Configure source record. [Pinned Query/Configure investigation](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/research/records/2026-09-22-src-m5-upscaler-query-configure.md)
- D008, D011 and D012. [D008](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/adr/D008.md) [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/adr/D011.md) [D012](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/v0.1.0/docs/adr/D012.md)

Upstream investigation used the pinned modern headers, generic loader/provider implementation, DX12 helper header, FSR3/FSR4 documentation, DX12 sample, and open FSR3 implementation. The main authoritative ABI sources are `ffx_api.h`, `ffx_api_types.h`, `ffx_api_dx12.h`, and `ffx_upscale.h`. [ffx_api.h at pinned commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h) [ffx_api_types.h at pinned commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h) [DX12 API header at pinned commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h) [Upscaler API header at pinned commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h)

FSR3 and FSR4 technique documentation were treated as upstream behavioral claims, not stronger ABI contracts. In particular, open FSR3 implementation details are not treated as proof of signed FSR4 behavior. [Pinned FSR3 upscaling documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md) [Pinned FSR4 documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md)

## 3. Review-state classification

The distinction below is important because several absent v0.1.0 features were intentionally excluded rather than overlooked.

| Classification | Current status |
|---|---|
| **Already decided** | D008 fixes safe construction around non-zero checked maxima, the required upscaler/version/DX12 descriptor chain, initially zero creation flags, and provider/runtime handling of undocumented numerical limits. D011 fixes the immediate-return `unsafe dispatch(&mut self, ...)`, renderer ownership of submission/states/synchronization/GPU lifetimes, the first five-resource profile, public dispatch fields, explicit exposure, current MV convention, fixed active extents, poison-on-native-dispatch-error, and no builder. D012 fixes private `UpscalerOptions` fields plus constructor/getters and deliberately leaves room for later optional creation settings. |
| **Intentionally deferred** | Quality presets and size helpers, jitter utilities, memory queries, Configure tuning, provider override, callbacks/global debug, additional creation flags, auto exposure, HDR, dynamic resolution, sharpening, masks, broader formats, variable output, broader provider support and temporal/IQ validation were explicitly outside the accepted initial contracts. |
| **Not yet researched / not publicly established** | Signed-provider behavior for every shared flag and combination; general accepted DXGI formats; signed-provider variable-output temporal behavior; complete Configure semantics across providers; callback lifetime/threading for signed providers; COMPUTE-list support; alpha behavior; nontrivial aliasing; general post-state behavior; broad hardware/provider matrices; and actual temporal correctness. The repository already marks several of these as open. |
| **New design decision required** | How to extend the public-field `UpscaleDispatch` once a per-frame feature is actually accepted; which creation flags become supported typed options; what provider-selection abstraction is public; whether Query/Configure become stable high-level APIs; and what broader resource-format support policy the crate promises. These are not reversals of D008/D011/D012; they arise only when deferred capabilities are promoted into supported API. |

No contradiction was found that invalidates the supplied baseline. The significant upstream surface beyond v0.1.0 was already broadly recognized by the repository; this investigation primarily maps that surface onto future API design.

## 4. Modern loader boundary: exports versus descriptors versus helpers

### 4.1 DLL exports

**Verified source fact.** The modern FidelityFX loader exposes five generic C entry points relevant here:

```c
ffxCreateContext(ffxContext*, ffxCreateContextDescHeader*, const ffxAllocationCallbacks*);
ffxDestroyContext(ffxContext*, const ffxAllocationCallbacks*);
ffxConfigure(ffxContext*, const ffxConfigureDescHeader*);
ffxQuery(ffxContext*, ffxQueryDescHeader*);
ffxDispatch(ffxContext*, const ffxDispatchDescHeader*);
```

`ffxCreateContext` also states that pointers passed in its descriptor chain must remain live until destruction, although project research has correctly found more specific evidence that at least `ffxOverrideVersion` is call-scoped and that a universal lifetime rule for every concrete signed-provider node cannot be inferred from this generic wording.

There are no separately exported modern functions named for sharpening, auto exposure, jitter, FSR3 versus FSR4 dispatch, or provider override. Those operations are selected by descriptor tags, fields and `pNext` extensions.

### 4.2 Modern descriptor chain

**Verified source fact.** A DX12 upscaler context is described using an `ffxApiHeader { ffxStructType_t type; ffxApiHeader *pNext; }` chain. The root upscaler descriptor is:

```c
struct ffxCreateContextDescUpscale {
    ffxCreateContextDescHeader header;
    uint32_t flags;
    FfxApiDimensions2D maxRenderSize;
    FfxApiDimensions2D maxUpscaleSize;
    ffxApiMessage fpMessage;
};
```

The upscaler API version at this SDK commit is `4.1.1`, and creation has a dedicated mandatory version node:

```c
struct ffxCreateContextDescUpscaleVersion {
    ffxCreateContextDescHeader header;
    uint32_t version; // must equal FFX_UPSCALER_VERSION
};
```

DX12 supplies:

```c
struct ffxCreateBackendDX12Desc {
    ffxCreateContextDescHeader header;
    ID3D12Device *device;
};
```

The root, version node and DX12 backend are therefore structural creation inputs, not per-frame state.

Optional creation-chain mechanisms include `ffxOverrideVersion { header, uint64_t versionId; }` for explicit provider/version selection and `ffxCreateBackendDX12AllocationCallbacksDesc` for backend resource/heap/constant-buffer allocation callbacks. Host-side FidelityFX object allocation is separate again through `ffxAllocationCallbacks { pUserData, alloc, dealloc }`, passed to create/destroy rather than placed in `pNext`.

**Inference.** Backend allocation customization and host allocation callbacks materially affect ownership architecture, but they are orthogonal to the first useful high-level expansion of upscaling functionality. They should not force the frame API to mirror arbitrary descriptor chains.

### 4.3 Inline C++ helpers

**Verified source fact.** `ffx_api.hpp` supplies inline/template helpers such as `LinkHeaders`, `CreateContext`, `DestroyContext`, `Configure`, `Query`, `Dispatch`, and descriptor initialization helpers. `ffx_upscale.hpp` provides corresponding typed C++ descriptor helper wrappers. They ultimately route to the five C exports; they are not additional DLL ABI entry points.

The DX12 header additionally supplies inline conversion helpers including `ffxApiGetSurfaceFormatDX12` and `ffxApiGetResourceDX12(ID3D12Resource*, uint32_t state = FFX_API_RESOURCE_STATE_COMPUTE_READ, uint32_t additionalUsages = 0)`. These construct `FfxApiResource` descriptions from DX12 resources; their broad DXGI-format conversion table does **not** establish that every upscaler provider accepts every mapped format.

### 4.4 Legacy FSR3 API

**Verified source fact.** The pinned SDK also contains the older direct FSR3 API in `ffx_fsr3upscaler.h`, with effect-specific functions such as `ffxFsr3UpscalerContextCreate`, `ffxFsr3UpscalerContextDispatch`, direct GPU-memory and jitter/quality helpers, and effect-specific context/configuration structures. For example, `ffxFsr3UpscalerContextDispatch` takes `FfxFsr3UpscalerDispatchDescription`, and `ffxFsr3UpscalerGetJitterOffset` is a direct function rather than a modern Query descriptor.

**Inference.** These legacy functions are useful implementation evidence for the open FSR3 provider but should not become the public Rust abstraction for the modern provider-routed API. In particular, a legacy FSR3 behavior cannot establish signed FSR4 behavior.

## 5. Complete modern upscaler descriptor surface

### 5.1 Creation-time surface

The pinned `FfxApiCreateContextUpscaleFlags` contains all of the following creation flags:

| Flag | Native value | Meaning | Rust-relevant classification |
|---|---:|---|---|
| `FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE` | `1 << 0` | Input color is HDR | Deferred capability; typed creation setting required |
| `FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS` | `1 << 1` | MV texture uses presentation resolution | Deferred capability; changes resource validation and scale |
| `FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION` | `1 << 2` | Supplied MVs contain jitter pattern | Deferred capability; changes semantic contract |
| `FFX_UPSCALE_ENABLE_DEPTH_INVERTED` | `1 << 3` | Input depth is reversed/inverted | Deferred capability; typed depth convention |
| `FFX_UPSCALE_ENABLE_DEPTH_INFINITE` | `1 << 4` | Infinite far plane | Deferred capability; typed depth convention |
| `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` | `1 << 5` | Provider computes exposure | Deferred capability; changes dispatch resource requirements |
| `FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION` | `1 << 6` | Application uses dynamic render resolution | Deferred capability |
| `FFX_UPSCALE_ENABLE_DEBUG_CHECKING` | `1 << 7` | Enable runtime API checking/messages | Deferred diagnostics; callback lifetime needs separate contract |
| `FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE` | `1 << 8` | Color input is perceptual/non-linear | FSR4-oriented capability; provider-sensitive |
| `FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION` | `1 << 9` | Permit debug visualization, potentially increasing memory | Provider-sensitive diagnostic capability |

These are exactly the shared-header bits at the pinned commit. Zero is explicitly a valid combination, preserving D008's current contract.

### 5.2 Per-frame dispatch surface

**Verified source fact.** `ffxDispatchDescUpscale` contains the complete shared per-frame payload:

| Field | Exact native type | Timing | Notes |
|---|---|---|---|
| `commandList` | `void *` | frame | DX12 command list |
| `color` | `FfxApiResource` | frame | render-resolution color |
| `depth` | `FfxApiResource` | frame | 32-bit depth |
| `motionVectors` | `FfxApiResource` | frame | render or presentation resolution depending on create flag |
| `exposure` | `FfxApiResource` | frame | optional 1×1 exposure |
| `reactive` | `FfxApiResource` | frame | optional reactive mask |
| `transparencyAndComposition` | `FfxApiResource` | frame | optional T&C mask |
| `output` | `FfxApiResource` | frame | presentation/upscaled output |
| `jitterOffset` | `FfxApiFloatCoords2D` | frame | actual subpixel camera jitter |
| `motionVectorScale` | `FfxApiFloatCoords2D` | frame | converts supplied MV units to expected screen-space range |
| `renderSize` | `FfxApiDimensions2D` | frame | active render extent |
| `upscaleSize` | `FfxApiDimensions2D` | frame | active output extent; optional, otherwise `maxUpscaleSize` |
| `enableSharpening` | `bool` | frame | enables additional sharpening pass |
| `sharpness` | `float` | frame | documented range `[0,1]` |
| `frameTimeDelta` | `float` | frame | milliseconds |
| `preExposure` | `float` | frame | must be `> 0` |
| `reset` | `bool` | frame | temporal-history discontinuity pulse |
| `cameraNear` | `float` | frame | camera near plane |
| `cameraFar` | `float` | frame | camera far plane |
| `cameraFovAngleVertical` | `float` | frame | vertical FOV in radians |
| `viewSpaceToMetersFactor` | `float` | frame | world/view-space scale |
| `flags` | `uint32_t` | frame | shared dispatch flags |



The dispatch flags are:

```c
FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW       = 1 << 0
FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_SRGB = 1 << 1
FFX_UPSCALE_FLAG_NON_LINEAR_COLOR_PQ   = 1 << 2
```

The FSR4 documentation explicitly says the sRGB and PQ flags are mutually exclusive and are used together with the non-linear-colorspace creation capability.

### 5.3 Reactive-mask generation dispatch

The modern API contains a second dispatch descriptor, but not a second exported function:

```c
struct ffxDispatchDescUpscaleGenerateReactiveMask {
    ffxDispatchDescHeader header;
    void *commandList;
    FfxApiResource colorOpaqueOnly;
    FfxApiResource colorPreUpscale;
    FfxApiResource outReactive;
    FfxApiDimensions2D renderSize;
    float scale;
    float cutoffThreshold;
    float binaryValue;
    uint32_t flags;
};
```

Its flags are the four `FfxApiDispatchUpscaleAutoreactiveFlags` bits for tonemap, inverse tonemap, threshold application, and max-component mode. It is submitted through ordinary `ffxDispatch`.

**Inference.** This is logically a separate operation from the upscaling frame itself. A future Rust wrapper need not force it into the main `UpscaleDispatch` value merely because native code uses the same exported `ffxDispatch`.

### 5.4 Query surface

The complete upscaler-relevant modern Query surface consists of two generic descriptors and seven upscaler descriptors, matching the project's earlier Query/Configure investigation.

| Descriptor | Exact inputs/outputs | Context |
|---|---|---|
| `ffxQueryDescGetVersions` | `createDescType: uint64_t`, `device: void*`, `outputCount: uint64_t*`, `versionIds: uint64_t*`, `versionNames: const char**` | null-context enumeration |
| `ffxQueryGetProviderVersion` | result fields `versionId: uint64_t`, `versionName: const char*` | live context |
| `ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode` | `qualityMode: uint32_t`, `pOutUpscaleRatio: float*` | null or live |
| `ffxQueryDescUpscaleGetRenderResolutionFromQualityMode` | display width/height + `qualityMode`, output width/height pointers | null or live |
| `ffxQueryDescUpscaleGetJitterPhaseCount` | render width, display width, `int32_t*` output | null or live |
| `ffxQueryDescUpscaleGetJitterOffset` | `index`, `phaseCount`, two `float*` outputs | null or live |
| `ffxQueryDescUpscaleGetGPUMemoryUsage` | `FfxApiEffectMemoryUsage*` | live context |
| `ffxQueryDescUpscaleGetGPUMemoryUsageV2` | `device`, maxima, create `flags`, memory output | pre-create/null context |
| `ffxQueryDescUpscaleGetResourceRequirements` | integral `required_resources`, `optional_resources` outputs | null or live |

The shared header defines quality modes NativeAA/Quality/Balanced/Performance/Ultra Performance as nominal ratios `1.0/1.5/1.7/2.0/3.0`.

The resource-requirement bits represent Color, Depth, MV, Exposure, Reactive Mask and T&C Mask. The Exposure bit can be satisfied by the explicit exposure resource or the auto-exposure creation mode according to its header description.

**Verified source fact.** For null-context utility queries other than `GetVersions`, the generic implementation routes through a provider using the descriptor type, optional `ffxOverrideVersion`, and a device obtainable from the chained DX12 backend descriptor. `GetVersions` is special because it embeds its own `device`.

**Inference.** A Rust API should hide these routing-chain details rather than expose raw `pNext` construction merely to ask for a ratio, jitter, memory estimate or resource requirements.

### 5.5 Configure surface

There are two generic global-debug Configure descriptors:

```c
ffxConfigureDescGlobalDebug1 {
    header;
    ffxApiMessage fpMessage;
    uint32_t debugLevel;
}

ffxConfigureDescGlobalDebug {
    header;
    uint64_t effectId;
    ffxApiMessage fpMessage;
    uint32_t debugLevel;
}
```

and one upscaler-specific descriptor:

```c
ffxConfigureDescUpscaleKeyValue {
    header;
    uint64_t key;
    uint64_t u64;
    void *ptr;
}
```

The five published upscaler keys are:

| Key | Native key | Published default/domain |
|---|---:|---|
| velocity factor | `0` | default `1.0`, clamped `[0,1]` |
| reactiveness scale | `1` | default `1.0`, clamped `[0,+∞)` |
| shading-change scale | `2` | default `1.0`, clamped `[0,+∞)` |
| accumulation added/frame | `3` | default `0.333`, clamped `[0,1]` |
| minimum disocclusion accumulation | `4` | default `-0.333`, clamped `[-1,1]` |

The current keys consume float values via `ptr`; `u64` remains part of the generic key/value structure.

**Verified source fact.** The generic loader intercepts the two global-debug descriptors before provider dispatch. In the pinned implementation the newer descriptor's `effectId` is present in the ABI but is not consumed by the shown top-level callback-setting path. Context-specific upscaler Configure routes to the selected provider.

**Unresolved.** Open FSR3 shows synchronous float consumption and persistent context tuning behavior, but this does not establish pointer lifetime, reset semantics or support for all five keys in arbitrary signed providers. Stable high-level exposure should therefore not assume more than the selected provider actually documents/tests.

## 6. Provider discovery and selection

**Verified source fact.** Provider IDs are obtained from `ffxQueryDescGetVersions`; `ffxOverrideVersion.versionId` is required to be one of those returned IDs. The public header does not define provider IDs as a stable semantic-version encoding for applications.

The pinned loader scans `pNext` for an override. Without one, provider routing checks internal providers and, on DX12, a driver/external provider; the pinned implementation can choose the external provider when its version is newer than the best supported internal provider. With an override, matching ID plus provider capability/support determines selection.

Enumeration also contains lifetime subtleties: external provider names are copied into static storage because the provider DLL's own pointer may not survive, and ordering is only sorted by version IDs when the `versionIds` output array is requested.

**Inference.** A safe Rust abstraction should use an opaque, runtime-obtained `ProviderId`, copy provider names immediately into owned strings, and avoid promising a stable ordering or deriving semantic meaning from numeric IDs.

**Upstream claim.** The pinned FSR4 documentation says FSR4 requires the modern API plus the signed binary distribution and claims support on AMD 7000-series discrete or AMD 9000-series GPUs or later. This is provider-specific documentation, not a statement that every enumerated modern provider shares those requirements.

## 7. Capability matrix and wrapper implications

| Capability | Native mechanism | Native timing/domain | Rust-validation boundary | Renderer/runtime obligation | Evidence status |
|---|---|---|---|---|---|
| Sharpening | `enableSharpening: bool`, `sharpness: float` | per frame; sharpness `[0,1]` | finite `[0,1]`; represent disabled/enabled without raw contradictory state | evaluate desired IQ; provider executes pass | **Verified source fact** for fields/domain; runtime IQ unresolved |
| Pre-exposure | `preExposure: float` | per frame; header says `>0` | finite positive value type is straightforward | color contents must actually use matching pre-exposure | **Verified source fact**; semantic meaning documented |
| Explicit exposure | `exposure: FfxApiResource` | per frame, normally 1×1 `R32_FLOAT` | dimensions/format/state can be checked; GPU value cannot | supply positive exposure matching later tonemap | **Upstream claim** for intended semantics |
| Auto exposure | create flag `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE`; exposure resource may be absent | creation choice affecting per-frame resources | wrapper can make explicit-vs-auto modes structurally exclusive | provider computes exposure; correctness is runtime/IQ | **Verified source fact** for flag/resource optionality |
| HDR | `FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE` | creation | typed boolean/mode; cannot validate pixel luminance domain | provide content matching advertised HDR mode | **Verified source fact** flag; **upstream claim** integration guidance |
| Non-linear color | create `NON_LINEAR_COLORSPACE`; per-frame sRGB/PQ flags | create capability + per-frame encoding; sRGB/PQ mutually exclusive | encode as enum, prohibit simultaneous flags | resource's actual transfer function must match | **Verified source fact** fields; FSR4-specific behavior is **upstream claim** |
| Depth convention | create `DEPTH_INVERTED`, `DEPTH_INFINITE` | context lifetime | type can represent standard/reversed × finite/infinite | actual projection/depth contents must match | **Verified source fact** |
| MV resolution | create `DISPLAY_RESOLUTION_MOTION_VECTORS` | context lifetime | validate texture active dimensions against selected mode | renderer generates matching buffer | **Verified source fact** |
| MV jitter convention | create `MOTION_VECTORS_JITTER_CANCELLATION` | context lifetime | typed convention possible | pixel values must include/exclude jitter consistently | **Verified source fact** |
| MV scale | `motionVectorScale: FfxApiFloatCoords2D` | per frame | finite/nonzero policy possible; current UV convention permits derivation | renderer owns semantic convention | **Verified source fact** |
| Jitter | `jitterOffset`; jitter queries | per frame + optional utilities | finite values; helper output can be wrapped | camera rendering must use supplied offset | **Verified source fact**; exact temporal correctness runtime-only |
| Reset | `reset: bool` | per frame pulse | trivial structural validation | caller identifies camera/history discontinuities | **Verified source fact** |
| Reactive mask | `reactive: FfxApiResource` | per frame, optional depending provider | format/extent/state where support policy exists | generate semantically correct `[0,1]` values | **Verified source fact** slot; provider requirement queried |
| T&C mask | `transparencyAndComposition` | per frame, optional depending provider | same | generate semantically appropriate values | **Verified source fact** slot |
| Auto-reactive generation | `ffxDispatchDescUpscaleGenerateReactiveMask` | separate dispatch | scalar/resource validation possible | command-list state/sync/lifetime | **Verified source fact** |
| Variable render size | `renderSize`; create `DYNAMIC_RESOLUTION` flag | per frame inside creation max | wrapper can require nonzero and `<= maxRenderSize`; exact flag policy requires decision | allocated resources must cover active region | Shared surface **verified**; signed-provider quality behavior unresolved |
| Variable output size | `upscaleSize` | per frame; omitted/zero form defaults to max | wrapper can expose explicit active extent and avoid native zero sentinel | output allocation must cover active output | Shared field **verified**; broad signed-provider temporal behavior unresolved |
| Debug checker | create `DEBUG_CHECKING`, `fpMessage` | context creation | flags easy; safe callback API is not | callback lifetime/threading/reentrancy | **Unresolved** for robust signed-provider callback contract |
| Debug view | create `DEBUG_VISUALIZATION`; dispatch `DRAW_DEBUG_VIEW` | creation + frame | structural dependency can be enforced | output deliberately becomes diagnostic image | **Upstream claim** for FSR4 behavior |
| Quality presets | Query descriptors | utility, not context mode | enum and checked dimensions | caller chooses policy | **Verified source fact** |
| Memory usage | live and precreate Query descriptors | query | copy scalar result | estimate remains provider/device-specific | **Verified source fact** |
| Resource requirements | Query descriptor | query | map bitset into owned capability value | provider-specific resources still need valid content | **Verified source fact** |
| Provider selection | `GetVersions` + `ffxOverrideVersion` | discovery + create/query chain | opaque IDs and owned names | selected provider may vary by device/driver | **Verified source fact** |
| Tuning constants | `ffxConfigureDescUpscaleKeyValue` | persistent context Configure | ranges can be represented if provider support known | IQ effects/provider support runtime-dependent | shared ABI **verified**, universal behavior **unresolved** |

## 8. Resource formats, extents, states, ownership and lifetimes

### 8.1 Input formats

**Upstream claim.** Pinned FSR3 documentation describes color as application-specified, depth as a one-component floating-point texture, motion vectors as a two-component floating-point texture, masks as render-resolution `R8_UNORM`, and exposure as 1×1 `R32_FLOAT`. Motion vectors use render resolution unless the display-resolution-MV creation flag is set.

The documentation specifies current-to-previous motion in screen space and says `motionVectorScale` adapts application units to the expected `[-width,+width] × [-height,+height]` range. Render-resolution inputs are jittered except motion vectors unless the jitter-cancellation flag says otherwise.

**Inference.** The native API does not justify turning the DX12 helper's complete DXGI conversion table into a safe wrapper whitelist. A widened Rust format contract should be deliberately researched and tested per role/provider rather than inferred from the ability to construct an `FfxApiResource`.

### 8.2 Active extents versus allocation extents

This distinction is important for future API design:

- `maxRenderSize` and `maxUpscaleSize` are context-creation values.
- `renderSize` and `upscaleSize` are frame values.
- the actual DX12 resource descriptions describe allocation extents.
- therefore allocation extent and active image extent are distinct pieces of information.

The shared header explicitly marks `upscaleSize` optional and defaults it to the creation maximum. The open FSR3 implementation interprets both output dimensions being zero as this default, while exactly one zero dimension is diagnosed as suspicious.

**Inference.** A Rust wrapper should not expose the native `{0,0}` sentinel merely to mean “use maximum.” It can retain maximum as the default at the Rust level and supply explicit nonzero active extents when variable output support is enabled.

The open FSR3 implementation also keeps context maxima separately from active render/output sizes and uses active sizes when calculating frame constants. This is useful evidence for the semantic distinction but not proof of signed-provider allocation strategy.

### 8.3 Resource states

`FfxApiResource` contains the native resource pointer, an `FfxApiResourceDescription`, and a declared `state`. The shared state enum includes `FFX_API_RESOURCE_STATE_COMPUTE_READ` and `FFX_API_RESOURCE_STATE_UNORDERED_ACCESS`.

The current project correctly treats state transitions as renderer obligations. Existing source/runtime evidence establishes the narrow current profile using readable inputs and UAV output and records that the backend may disturb descriptor heaps and compute bindings; it does not establish a universal signed-provider post-state guarantee or COMPUTE-list support.

**Inference.** Future resource roles should continue to validate descriptors and the wrapper's declared state assumptions where possible, but the unsafe dispatch boundary must retain responsibility for actual GPU state, aliasing, submission ordering and lifetime.

### 8.4 Ownership and GPU lifetime

Nothing in the extended surface justifies transferring renderer resource ownership to the wrapper. Query output buffers and copied metadata can be Rust-owned; frame textures and command lists remain renderer-owned.

The generic create header has unusually conservative pointer-lifetime wording, while repository research has established that lifetime behavior differs by node and that signed-provider retention of concrete chain/device dependencies is not fully documented. Conservative owned/address-stable descriptor storage therefore remains a defensible internal wrapper policy.

## 9. Individual feature findings

### 9.1 Sharpening

**Verified source fact.** Sharpening is entirely per-frame: `bool enableSharpening` plus `float sharpness`, with documented range `[0,1]`. There is no sharpening creation flag.

**Inference.** A Rust value such as `Option<Sharpness>` maps more cleanly than exposing independent boolean/value fields: `None` means disabled and `Some(v)` means enabled. This would make contradictory states unrepresentable.

**Unresolved.** Source inspection cannot establish useful perceptual ranges, provider equality or temporal interaction. Hardware image comparisons are required.

### 9.2 Pre-exposure and exposure

**Verified source fact.** `preExposure` is per-frame and the header requires it to be greater than zero. The open FSR3 checker treats zero as erroneous.

**Upstream claim.** AMD describes pre-exposure as a factor by which the input is divided to recover the game's original signal, while the exposure texture is subsequently multiplied into that signal and should correspond to later application tonemapping.

Explicit versus automatic exposure is a creation-level choice because `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` alters whether the 1×1 explicit resource is necessary. This is more than an optional frame field: it changes the context's declared operating mode.

**Inference.** A future safe API should represent `ExposureMode::{Explicit, Automatic}` at creation and validate each frame against that mode. It should not silently accept both modes as equivalent.

### 9.3 HDR and color encoding

HDR is a creation flag. Non-linear input is separately represented by another creation flag, while sRGB/PQ are per-frame dispatch hints. No source inspected establishes HDR and PQ as inherently incompatible; the wrapper should not invent such a restriction.

FSR4 documentation says linear color remains preferred and identifies the two non-linear per-frame flags as mutually exclusive.

**Provider-specific limitation.** The open FSR3 modern adapter does not establish support for newer FSR4-oriented bits. Absence from that open path is provider-specific evidence, not proof that the shared ABI fields are invalid. Conversely, their presence in the shared header is not proof that every provider implements them.

### 9.4 Depth

The two independent creation bits represent inverted depth and infinite far plane. Pinned FSR3 documentation strongly recommends reversed/infinite depth but states that alternatives are supported.

**Inference.** A typed `DepthConvention` is preferable to independently exposing raw flag bits because it can enumerate the four actual combinations without allowing unrelated bits.

Rust can validate camera scalars but cannot inspect whether a supplied depth texture actually matches the declared projection. The open FSR3 checker has additional warnings/errors for near/far/FOV, but provider diagnostic thresholds such as the `0.075` warning are not suitable universal safe-wrapper validity limits.

### 9.5 Motion vectors

Creation determines whether motion-vector allocation/active resolution is render or presentation resolution and whether the application supplies vectors with camera jitter included. Per-frame `motionVectorScale` separately describes unit conversion.

The current wrapper's jitter-free current-to-previous UV convention is therefore one valid high-level specialization, not the complete native domain.

**New design decision required.** Expansion could either retain that semantic convention and derive the native scale for both supported resolutions, or expose a more general scale/convention API. Merely exposing raw `motionVectorScale` would move a semantic correctness burden into otherwise safe Rust without proving a need.

### 9.6 Jitter and reset

Jitter Query descriptors are utilities, not mandatory prerequisites. AMD explicitly permits an application-generated sequence as long as the actual applied jitter is passed to `jitterOffset`. Documentation describes the provided sequence as Halton(2,3) and uses unit-pixel offsets.

Existing repository research correctly notes a pinned discrepancy: documentation gives custom sequence length `ceil(8*n²)`, while the inspected open FSR3 implementation truncates positive `8*n²`. This argues for provider-routed Query wrappers rather than reimplementing the formula as a supposedly universal Rust rule.

`reset` is a per-frame history-discontinuity pulse. AMD specifically describes camera cuts as a use case. It is unrelated to GPU synchronization or the Rust poison state.

### 9.7 Masks

The shared API contains both reactive and T&C resource slots, plus the separate auto-reactive generator.

FSR3 documentation gives both masks render-resolution `R8_UNORM` semantics and describes null masks as effectively cleared fallback masks.

FSR4 documentation, however, explicitly says FSR4 no longer requires applications to generate these masks, although they may still be consumed. It introduces the resource-requirements Query specifically so applications can avoid hard-coded mask assumptions.

**Inference.** `ffxQueryDescUpscaleGetResourceRequirements` is the correct provider-neutral source of required/optional-resource shape. A Rust API should not encode “FSR3 needs masks” or “FSR4 does not need masks” in provider-name string checks.

### 9.8 Variable render and output sizes

The native dispatch always contains active `renderSize` and `upscaleSize`; the current Rust profile merely fixes them to context maxima.

`FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION` declares dynamic render-resolution usage at creation. No analogous creation flag exists for changing `upscaleSize`; the latter is already a per-frame dispatch field.

**Inference.** These should be treated as two distinct capabilities in Rust:

- dynamic render size: creation capability plus per-frame active render extent;
- variable output size: per-frame active output extent bounded by the context/output allocation policy.

It would be reasonable for a high-level wrapper to validate nonzero active extents, configured maxima, and backing resource dimensions before native dispatch. Those are wrapper validation proposals, not newly discovered AMD universal limits.

**Unresolved.** Source existence of `upscaleSize` does not establish temporal reconstruction quality, history behavior, all allowed size transitions, or signed-provider behavior when output size changes from frame to frame.

### 9.9 Other API-design-relevant features

Provider discovery/override, resource-requirement discovery, memory queries, debug callbacks/views, tuning constants and custom allocation callbacks all exist now and therefore matter to long-term API shape even if they are not first-priority user features.

The most important distinction is that they occupy different semantic layers:

- provider selection and most creation flags are context configuration;
- sharpening, pre-exposure, active extents, masks and encoding flags are frame configuration;
- quality/jitter/memory/resource requirements are Queries;
- tuning constants are mutable persistent context configuration;
- reactive-mask generation is a separate GPU operation;
- host/backend allocation callbacks are ownership/infrastructure customization.

Flattening all of those into one generic “options” object would obscure native timing and lifetime boundaries.

## 10. What Rust can validate

A future safe layer can validate substantial structural information before crossing the existing unsafe GPU boundary:

| Rust can reasonably validate | Renderer/provider obligations that remain |
|---|---|
| finite scalar inputs and published scalar ranges | actual pixel contents |
| nonzero checked active dimensions and wrapper-supported relations to maxima | undocumented provider/device size limits |
| selected depth/MV/exposure mode consistency | whether resources semantically match those declarations |
| explicit exposure texture 1×1 and supported format | exposure value stored on GPU |
| selected mask formats/extents | mask quality/content |
| motion-vector texture dimensions according to chosen resolution mode | direction, magnitude and jitter semantics of vector contents |
| sRGB/PQ mutual exclusion | actual transfer function of color data |
| output has wrapper-required UAV capability | real GPU state and barriers |
| known resource descriptor shapes | heap aliasing not visible from resource identity alone |
| provider IDs originate from enumeration if wrapper controls both steps | future driver/provider behavior |
| copied provider names | lifetime of native borrowed name pointers |
| active sizes fit supplied texture descriptions | temporal quality across size changes |

D011's unsafe boundary remains appropriate: actual resource states, synchronization, submission order, GPU lifetime and semantic texture correctness cannot be made safe merely by adding more high-level fields.

## 11. Additive, source-compatible API direction

D012 gives creation options room to grow without breaking current construction. No builder is required.

One possible additive shape is:

```rust
let mut options = UpscalerOptions::new(max_render_size, max_upscale_size);

options.set_depth_convention(DepthConvention::ReversedInfinite);
options.set_exposure_mode(ExposureMode::Automatic);
options.set_motion_vector_resolution(MotionVectorResolution::Output);
options.set_motion_vector_jitter(MotionVectorJitter::Included);
options.set_color_input(ColorInput::HdrLinear);
options.set_dynamic_resolution(true);

let mut upscaler = runtime.create_upscaler_dx12(&device, options)?;
```

The current minimal caller remains unchanged:

```rust
let options = UpscalerOptions::new(max_render_size, max_upscale_size);
let mut upscaler = runtime.create_upscaler_dx12(&device, options)?;
```

This is source-compatible because D012 deliberately made creation fields private and the constructor can continue supplying today's defaults.

Read-only/provider utilities can also be additive without disturbing dispatch:

```rust
let providers = runtime.upscaler_providers_dx12(&device)?;

let ratio = runtime.upscale_ratio_dx12(
    &device,
    QualityMode::Quality,
    provider.as_ref(),
)?;

let requirements = runtime.upscaler_resource_requirements_dx12(
    &device,
    provider.as_ref(),
)?;
```

The exact ownership placement (`Runtime`, an `UpscalerApi` facade, free functions, etc.) is a separate ergonomics choice; the evidence only argues that provider routing and descriptor-chain construction should be hidden.

### The dispatch limitation

The existing public-field value cannot acquire even an optional field source-compatibly for callers using struct literals:

```rust
pub struct UpscaleDispatch<'a> {
    pub command_list: ...,
    ...
}
```

D011 explicitly accepted that consequence.

Therefore adding `pre_exposure`, `sharpening`, `reactive_mask`, `active_render_size`, `active_output_size`, or dispatch color flags directly to this public struct is a source-breaking change.

A compatibility-preserving expansion consequently needs some second channel for genuinely new per-frame inputs. One example is an advanced frame type while retaining the existing method unchanged:

```rust
pub unsafe fn dispatch(
    &mut self,
    dispatch: UpscaleDispatch<'_>,
) -> Result<(), Error>;

pub unsafe fn dispatch_frame(
    &mut self,
    frame: UpscaleFrame<'_>,
) -> Result<(), Error>;
```

with a private-field extensible `UpscaleFrame`:

```rust
let mut frame = UpscaleFrame::new(
    command_list,
    color,
    depth,
    motion_vectors,
    output,
    jitter,
    frame_time,
    camera,
);

frame.set_exposure(ExposureInput::Explicit(exposure));
frame.set_pre_exposure(PreExposure::new(1.0)?);
frame.set_sharpening(Some(Sharpness::new(0.35)?));
frame.set_active_render_size(render_size);
frame.set_active_output_size(output_size);
frame.set_reactive_mask(Some(reactive));
frame.set_reset(false);

unsafe {
    upscaler.dispatch_frame(frame)?;
}
```

This is only an API-design example; a second dispatch method is **not** required by upstream. It is the consequence of choosing source compatibility with D011 while extending per-frame data.

An alternative additive design would use mutable per-context frame defaults/setters before the existing `dispatch`. That avoids a second method but is less attractive for resources, active extents and reset-like values because per-frame state becomes hidden context state and validation/poison semantics become more complicated.

## 12. Deliberate breaking redesign

A deliberate breaking release can instead repair the specific extensibility property D011 knowingly traded away by making the frame value's fields private.

For example:

```rust
pub struct UpscaleDispatch<'a> {
    // private, extensible
}

impl<'a> UpscaleDispatch<'a> {
    pub fn new(
        command_list: &'a ID3D12GraphicsCommandList,
        color: &'a ID3D12Resource,
        depth: &'a ID3D12Resource,
        motion_vectors: &'a ID3D12Resource,
        output: &'a ID3D12Resource,
        jitter: JitterOffsetPixels,
        frame_time: FrameTimeMillis,
        camera: CameraParameters,
    ) -> Self;

    pub fn set_exposure(&mut self, exposure: ExposureInput<'a>);
    pub fn set_pre_exposure(&mut self, value: PreExposure);
    pub fn set_sharpening(&mut self, value: Option<Sharpness>);
    pub fn set_active_render_size(&mut self, value: Dimensions);
    pub fn set_active_output_size(&mut self, value: Dimensions);
    pub fn set_reactive_mask(&mut self, value: Option<&'a ID3D12Resource>);
    pub fn set_transparency_and_composition_mask(
        &mut self,
        value: Option<&'a ID3D12Resource>,
    );
    pub fn set_reset(&mut self, reset: bool);
}
```

Current explicit-exposure usage becomes:

```rust
let mut frame = UpscaleDispatch::new(
    command_list,
    color,
    depth,
    motion_vectors,
    output,
    jitter,
    frame_time,
    camera,
);

frame.set_exposure(ExposureInput::Explicit(exposure));

unsafe {
    upscaler.dispatch(frame)?;
}
```

Auto-exposure contexts could use:

```rust
frame.set_exposure(ExposureInput::Automatic);
```

or have exposure inferred from the context's creation mode, depending on the desired validation contract.

Private fields—not builders specifically—are the important property. Setters, constructors, nested option objects or another representation can all preserve future source compatibility.

### Comparison

| Property | Additive path | Breaking private-frame redesign |
|---|---|---|
| v0.1.0 source compatibility | preserved | intentionally broken |
| existing minimal call | unchanged | requires migration |
| creation extensions | clean via D012 in either design | clean |
| first new per-frame feature | needs another input channel | fits existing dispatch value |
| later per-frame additions | clean if new advanced type is private | clean |
| number of dispatch concepts | at least legacy + extended route unless stateful side channel used | one coherent route |
| risk of hidden mutable frame state | avoidable with advanced frame value | avoidable |
| change to `unsafe` GPU boundary | none required | none required |
| change to `&mut self` | none justified | none justified |

## 13. Recommendation

**Proposal, not accepted project decision.**

Do **not** break v0.1.0 merely to mirror fields that exist upstream. Creation-side evolution can proceed additively under D012, and read-only Query/provider APIs can also be added independently.

Before accepting the **first per-frame extension**, however, make a dedicated dispatch-extensibility ADR. At that point there are two coherent choices:

1. preserve v0.1.0 source compatibility and introduce one private/extensible advanced frame input while keeping the D011 method indefinitely; or
2. deliberately make one early breaking release that replaces public `UpscaleDispatch` fields with an extensible private representation and retains a single dispatch path.

If the intended supported profile is expected to include several of sharpening, variable active extents, automatic exposure, masks, pre-exposure and non-linear color, the second option produces the cleaner long-term API because those are all genuine per-frame dimensions of the same native operation. That is an API-maintenance argument, not evidence that D011 was wrong when it deliberately selected a narrow first-release profile.

Conversely, if source compatibility with v0.1.0 callers is the dominant constraint, the additive advanced-frame route is technically sound and does not require reopening the existing method.

Nothing found supports changing either of these D011 properties:

```rust
pub unsafe fn dispatch(&mut self, ...);
```

The operation still records GPU work into caller-owned resources and lists, and mutable context state remains evident throughout the upstream API.

## 14. Proposed implementation milestones

These are proposed sequencing boundaries, not authorization to implement them.

### M1 — sys-surface completeness

Add/verify ABI declarations and layout tests for the complete shared upscaler descriptor/flag/query/configuration surface required by planned work. Keep modern C exports distinct from inline C++ helpers and legacy FSR3 functions.

No high-level behavior needs to change at this stage.

### M2 — provider and read-only Query surface

Expose owned provider metadata using opaque `ProviderId`, plus selected low-risk Query utilities:

- provider enumeration/current provider;
- resource requirements;
- precreate/live memory usage;
- quality ratio/render-size helpers;
- jitter queries.

Copy native provider names immediately. Hide raw descriptor chains.

This milestone can remain entirely additive and does not require changing dispatch.

### M3 — creation-capability ADR

Choose typed contracts and defaults for the first supported creation options. Depth convention, exposure mode and MV convention are good candidates because they materially determine frame-resource interpretation.

Do not expose a raw `u32 flags`.

Provider override can be considered independently because D012 already permits owned future configuration.

### M4 — dispatch-extensibility ADR

Before exposing any new frame field, choose explicitly between the additive advanced-frame route and an intentional breaking private-frame redesign.

This is the main **new design decision required** by the post-v0.1.0 API surface.

### M5 — low-coupling frame scalars

Once the frame representation is settled, sharpening and pre-exposure are relatively contained additions because they introduce no new resources.

### M6 — active dimensions

Add variable render and output active extents with explicit allocation-vs-active validation. Test changing sizes temporally rather than merely checking native return codes.

### M7 — exposure and masks

Add auto-exposure context mode and optional mask resources only with resource-requirement-aware validation. Treat automatic reactive-mask generation as a separate GPU operation.

### M8 — HDR/non-linear/debug/provider-specific features

Expand HDR and non-linear color after comparative provider testing. Debug callbacks and visualization should remain separate from ordinary production configuration because their lifetime and provider behavior are less well established.

### M9 — Configure tuning

Treat the five tuning constants as a separate API decision, potentially experimental/provider-sensitive initially. Their presence in the shared header is insufficient evidence for a provider-neutral stable semantic contract.

Custom host/backend allocation callbacks can remain below the high-level upscaler API until an actual ownership use case requires them.

## 15. Hardware experiments needed for correctness rather than successful dispatch

The existing one-GPU smoke test establishes real writes and bounded multi-frame execution, not correct temporal reconstruction.

The following experiments would discriminate semantic correctness:

| Feature | Experiment | Evidence of correctness beyond “dispatch returned OK” |
|---|---|---|
| Sharpening | static slanted edges/frequency patterns at `0`, intermediate and `1` | measured edge profile/overshoot and monotonic sharpening effect |
| Pre-exposure | render equivalent scenes with pre-exposure factors such as `0.25`, `1`, `4` | normalized outputs remain equivalent within expected numerical tolerance |
| Explicit exposure | controlled luminance ramp with known frame-varying exposure | output responds consistently with the documented exposure domain |
| Auto exposure | matched explicit exposure versus auto mode over controlled luminance transitions | convergence and temporal response, not merely finite output |
| Depth modes | synthetic moving foreground/background occlusion with standard, reversed, finite and infinite projections | correct disocclusion/history rejection; deliberately wrong flag should visibly differ |
| MV direction/scale | analytically known subpixel translation against high-resolution reference | low temporal residual only for correct direction and scale |
| MV resolution | same motion encoded at render and output resolution with corresponding create mode | equivalent reconstructed motion within expected error |
| MV jitter | paired jitter-included and jitter-free vectors with matching/mismatching creation flag | correct pair minimizes stationary-edge instability and ghosting |
| Jitter sequence | stationary diagonal/high-frequency geometry across a full phase | temporal stability and convergence versus wrong-sign/no-jitter controls |
| Reset | accumulate stable history, perform hard camera cut, compare reset false/true | first post-cut frame discards stale history and subsequent frames reconverge |
| Reactive/T&C masks | translucent particles/reflections without reliable MV | measurable reduction in trails/lock artifacts with semantically appropriate mask |
| Auto-reactive generation | same scene with hand-authored and generated masks | generated mask content/effect correlates with intended translucent regions |
| Variable render size | alternate several active extents within one allocation; put sentinels outside active regions | correct active-region interpretation and stable temporal transitions |
| Variable output size | alternate active output extents below max | correct write extent, no unintended region writes, appropriate history transition |
| HDR | equivalent normalized SDR/HDR scenes | transfer-normalized reconstruction without unexpected clipping or exposure drift |
| sRGB/PQ input | equivalent scene encoded linear/sRGB/PQ; pair correct and deliberately wrong flags | correct-flag outputs agree after transfer normalization; wrong flags serve as control |
| Resource requirements | enumerate/select every available provider and query before/live context | returned requirements agree with actual accepted resource omission |
| Configure constants | ghosting/disocclusion fixtures at default and controlled extremes | visible/measurable effect, persistence across frames and documented reset behavior |
| Provider override | enumerate each provider ID and explicitly create it | actual provider query reports selected ID/name and feature behavior is tested separately |

Useful measurements include high-resolution-reference PSNR/SSIM or simple linear error, temporal residuals on stationary regions, trail energy behind moving objects, edge overshoot, frame-to-frame variance, and sentinel checks outside active regions. The point is not to standardize one metric as “FSR quality” but to ensure that each capability has an observable consequence consistent with its intended semantics.

Tests should retain D3D12 debug-layer checks, device-removal checks and fences as execution prerequisites, but those are not substitutes for image-semantic validation.

## 16. Source-answerable versus runtime-only questions

### Source-answerable at this pinned release

The following are established well enough from source to design ABI bindings and preliminary typed APIs:

- exact descriptor structures, tags and field types;
- five modern DLL exports;
- creation and dispatch flag bit definitions;
- creation-time versus per-frame placement;
- Query and Configure descriptor inventory;
- quality-mode nominal ratios;
- published scalar ranges/defaults;
- explicit provider enumeration/override mechanism;
- resource-requirement bit definitions;
- published mask/exposure dimensions and formats;
- MV resolution and jitter-convention flags;
- active `renderSize`/`upscaleSize` versus creation maxima;
- existence of host and DX12 backend allocation callbacks;
- distinction between exported ABI and inline C++ helpers;
- distinction between modern provider-routed API and legacy direct FSR3 API.

### Runtime-only or provider-dependent

Source inspection cannot establish:

- temporal reconstruction quality;
- whether a signed provider implements every shared flag identically;
- useful or accepted DXGI format sets beyond documented examples;
- correctness of arbitrary render/output size transitions;
- auto-exposure response and stability;
- actual MV/jitter/reset/mask correctness;
- interaction quality among HDR, exposure and non-linear inputs;
- exact signed-provider Configure support/persistence/reset behavior;
- general signed-provider resource-state restoration behavior;
- callback threading/reentrancy/lifetime rules;
- COMPUTE-list support;
- broad multi-context/concurrency limits;
- signed-provider internal allocation/sharing behavior;
- behavior on hardware/provider combinations not actually tested.

A fixed SDK source release does not remove these runtime/provider uncertainties because provider selection can involve driver/external implementations not represented by the open FSR3 source.

## 17. Differences from the supplied baseline

The supplied baseline is accurate. The investigation adds the following information rather than overturning it:

1. The complete shared creation flag domain is ten bits, covering HDR, MV resolution/jitter convention, depth inversion/infinite plane, auto exposure, dynamic resolution, debug checking, non-linear color and debug visualization. The current zero-flag profile is one intentionally narrow point in this space.
2. The dispatch descriptor already contains active render and output extents, sharpening, pre-exposure, optional masks and dispatch flags in addition to fields exposed by v0.1.0.
3. Variable output size is structurally separate from dynamic render resolution: `upscaleSize` is inherently per-frame and has no corresponding creation bit.
4. Resource requirements provide a provider-neutral mechanism for deciding whether masks are required/optional; this matters particularly because pinned FSR4 documentation differs from FSR3 guidance.
5. Provider selection is an explicit public mechanism based on `GetVersions` plus an opaque runtime-generated `ffxOverrideVersion`, while default selection may involve a driver/external provider.
6. Query and Configure are materially larger than quality/jitter helpers: they also expose provider metadata, memory usage, resource requirements, global debugging and five persistent tuning keys.
7. A separate modern reactive-mask-generation dispatch exists and should not be confused with simply supplying optional masks to the main upscaler dispatch.
8. DX12 backend allocation callbacks and generic host allocation callbacks exist, but they form an infrastructure/ownership layer rather than an argument for exposing raw extension chains in the high-level wrapper.
9. D012 already solves the creation-options semver problem. The unsolved extensibility problem is specifically D011's public per-frame struct once a deferred per-frame capability is accepted.
10. No inspected evidence justifies weakening the unsafe GPU boundary, removing mutable access to the context, or transferring renderer synchronization/resource lifetime into the wrapper.

## 18. Unresolved questions before broader stable support

1. Which of the ten shared creation flags are intended to be stable cross-provider guarantees versus provider-version capabilities discoverable only through successful enumeration/testing?
2. What exact color/depth/MV/output DXGI format set should `fsr-sdk` promise safely, as opposed to formats merely convertible by `ffxApiGetResourceDX12`?
3. Does every targeted signed provider support temporally correct frame-to-frame changes of `upscaleSize`, and are there undocumented transition constraints?
4. What exact policy should relate `FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION` to changing `renderSize`; specifically, should the Rust safe API reject varying render extents unless that creation mode was selected?
5. Should the wrapper continue to own one fixed MV semantic convention and derive `motionVectorScale`, or deliberately expose multiple semantic conventions?
6. What lifecycle and thread contract is safe for non-null context/global message callbacks with signed providers?
7. Which Configure keys are accepted by each target provider, are values consumed synchronously, how are defaults restored, and should these controls be considered stable production API at all?
8. What provider-selection guarantees should a public `ProviderId` make across driver updates or repeated enumeration? The evidence supports opacity, not persistence.
9. Does signed FSR4 restore all registered external resources to declared states on every successful/error path, or is the current observed restoration behavior narrower?
10. What exact semantics should a future safe frame type use for explicit versus automatic exposure when resource-requirement queries and provider behavior disagree or evolve?
11. If the first new per-frame capability is accepted, should compatibility with v0.1.0's public struct literals take priority over a single coherent long-term dispatch representation? This is the principal new API decision; source research cannot answer it.

## 19. Overall finding

The pinned modern DX12 upscaler API is descriptor-driven and substantially broader than the v0.1.0 wrapper, but its breadth does not require immediate API expansion. D008/D011 intentionally selected a narrow, evidence-backed profile, and D012 deliberately made creation options extensible.

The clean architectural boundary exposed by the upstream API is:

```text
runtime/provider discovery
        ↓
context creation policy
        ↓
persistent Configure state
        ↓
per-frame dispatch inputs
        ↓
unsafe renderer/GPU boundary
```

Queries sit alongside these layers as discovery and utility operations rather than prerequisites.

Creation-side growth can remain additive. The only unavoidable post-release structural issue identified by this investigation is expansion of D011's public-field frame value. That should become a new explicit design decision only when a deferred per-frame capability is actually promoted. Until then, the existing decision should remain intact rather than being reopened merely because the native descriptor contains additional fields.
