# Source investigation: v2.3.0 upscaler Query/Configure surface

> Intake note (2026-09-22): renamed from `2026-09-22-rename_me_2.md`.
> The pinned source bibliography is usable, but the supplied inline
> `:chatgpt-content-reference` markers had no accompanying index-to-source map.
> On 2026-09-23, 78 opaque markers were removed as a clerical cleanup; the
> substantive findings and direct source links were retained. They were not
> independently reverified at intake. See the later
> [source provenance repair](2026-09-22-src-m5-construction-query-provenance-repair.md)
> and [current synthesis](../api-and-safety.md#m5-source-intake-and-provenance).

**Investigation date:** 2026-09-22  
**Topic:** Public `ffxQuery` / `ffxConfigure` surface relevant to the modern FidelityFX upscaler  
**Authoritative baseline:** AMD FidelityFX SDK v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`  
**Platform/backend scope:** Windows x64/MSVC, official DX12 runtime  
**Evidence cut-off:** The pinned commit is normative for this record. Newer `main` material is not used to establish v2.3.0 behavior.

Evidence labels used below:

- **Verified** — directly established by the pinned header, source, or pinned documentation.
- **Upstream claim** — stated by AMD documentation but not independently established by the inspected implementation.
- **Inference** — conclusion derived from multiple pinned facts; not stated verbatim upstream.
- **Unresolved** — the public v2.3.0 contract does not establish the point.

## Scope and method

This investigation enumerates the generic and upscaler-specific public Query/Configure descriptors that can meaningfully participate in upscaler creation, version/provider selection, resolution/jitter utilities, resource planning, diagnostics, or tuning. Dispatch resource structures, DX12 resource-state handling, frame generation, denoisers except for contrast with generic API behavior, custom allocation, Rust API design, and engine integration are excluded.

Headers were used to establish ABI surface and documented fields. Pinned API/technique documentation was used for intended calling modes. Generic API and in-tree FSR3 provider source were inspected where necessary to establish null-context routing, provider selection, output ownership, and implementation-specific behavior. Sample code is treated only as evidence of a supported integration pattern, not as a universal contract.

The release page identifies v2.3.0 with commit `60f4ea8`; all source references below are pinned to the full commit.

## Exact pinned inputs

All sources accessed 2026-09-22.

- `Kits/FidelityFX/api/include/ffx_api.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h)
- `Kits/FidelityFX/api/include/ffx_api_types.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h)
- `Kits/FidelityFX/upscalers/include/ffx_upscale.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h)
- `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h)
- `Kits/FidelityFX/docs/getting-started/ffx-api.md` — [pinned documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md)
- `Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md` — [pinned FSR3 documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md)
- `Kits/FidelityFX/docs/techniques/super-resolution-ml.md` — [pinned ML/FSR4 documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md)
- `Kits/FidelityFX/api/internal/ffx_api.cpp` — [pinned generic implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp)
- `Kits/FidelityFX/api/internal/ffx_provider.h` — [pinned provider routing source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h)
- `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` — [pinned FSR3 provider source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp)
- `Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp` — [pinned FSR3 host implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp)
- `Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp` — [pinned DX12 sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp)

## Findings

### Public surface boundary

**Verified.** The relevant v2.3.0 surface consists of two generic Query descriptors, seven upscaler Query descriptors, two generic global-debug Configure descriptors, and one upscaler key/value Configure descriptor containing five published keys. No dedicated public upscaler “device capabilities” Query descriptor exists in the pinned upscaler header. Hardware/provider support instead participates in provider discovery/routing; `GetProviderVersions` filters providers using the supplied device and provider `IsSupported` behavior.

Every descriptor's `header.type` must match its actual structure. AMD explicitly states that a mismatch is undefined behavior and is likely to crash. This is a **Safety requirement**, not merely an ergonomic convention.

The generic return-code domain is `OK`, generic error, unknown descriptor type, underlying runtime/effect error, no provider, allocation failure, invalid parameter, and provider-does-not-support-new-descriptor. Individual descriptors generally do not publish a narrower exhaustive error set.

## Query operations

| Descriptor / tag | Context and provider routing | Inputs and output storage | Ownership / lifetime | Failure and version notes | Construction / dispatch relevance |
|---|---|---|---|---|---|
| `ffxQueryDescGetVersions` / `FFX_API_QUERY_DESC_TYPE_GET_VERSIONS` | **Verified:** documented null-context operation. `createDescType` selects the effect family; embedded `device` is used for DX12 provider/support enumeration, so no backend descriptor in `pNext` is required merely to carry the device. Core source special-cases this descriptor rather than routing it as an ordinary effect query. | `createDescType`, `device`, `outputCount`; optional caller arrays `versionIds` and `versionNames`. `*outputCount == 0` gives the available count. On an enumeration pass, initial `*outputCount` is array capacity and is replaced with returned count. Either array may be null; both null gives count only. | Caller owns the arrays. `versionNames[]` contains borrowed pointers. AMD warns that some names reside in global memory and may be overwritten by later version queries; provider source also copies an external provider DLL's name into a static buffer because the original DLL pointer may not survive. Retained names therefore need copying. | Version IDs used for overrides must come from this query for the appropriate create descriptor and must not be hard-coded. Results can vary with query parameters/device. A null DX12 device may produce a useful enumeration, as in the supplied project evidence, but does not establish device-specific external-provider availability. | Not universally required. **Conditionally required as an API-validity prerequisite if the application chooses an explicit `ffxOverrideVersion`.** |
| `ffxQueryGetProviderVersion` / `FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION` | **Verified:** requires a live context; it is not among the public null-context queries. It reports the provider associated with that context. | No caller output buffer: result fields are `versionId` and `const char *versionName` in the descriptor itself. Invalid query is represented by ID `0` / null name according to the header. | `versionName` is a borrowed `const char *`; no ownership transfer or retention lifetime is documented. **Unresolved:** exact validity after later provider/runtime operations. | Generic FidelityFX functionality; value is necessarily provider/version-specific. | Diagnostics/introspection only; not required for creation or dispatch. |
| `ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode` / `...GETUPSCALERATIOFROMQUALITYMODE` | **Verified:** null context is supported; a live upscaler context also reaches the associated provider. For null-context use, an `ffxCreateBackendDX12Desc` carrying the DX12 device must be chained when the driver/external provider is to be reachable; an optional version override selects a particular provider version. | Input `qualityMode`; output through caller `float *pOutUpscaleRatio`. | In-tree FSR3 writes synchronously and does not retain the pointer. The public contract does not explicitly state pointer-retention semantics for arbitrary external providers. | In-tree FSR3 maps the five valid modes to `1.0`, `1.5`, `1.7`, `2.0`, `3.0`; an invalid enum yields `0.0f` in that provider rather than an explicit query failure. | Optional general utility. |
| `ffxQueryDescUpscaleGetRenderResolutionFromQualityMode` / `...GETRENDERRESOLUTIONFROMQUALITYMODE` | Same null/live routing as ratio query. | Inputs `displayWidth`, `displayHeight`, `qualityMode`; writable `pOutRenderWidth`, `pOutRenderHeight`. | In-tree FSR3 writes requested scalar results during the call and retains no supplied output pointer in the inspected path. | FSR3 validates the quality enum, divides each display dimension by the selected ratio and casts the positive result to `uint32_t`: fractional pixels are truncated; no additional alignment is performed in this implementation. Invalid quality propagates through the provider's `TRY2` path as `FFX_API_RETURN_ERROR_RUNTIME_ERROR`. | Optional general utility. |
| `ffxQueryDescUpscaleGetJitterPhaseCount` / `...GETJITTERPHASECOUNT` | Null and live context supported; null external-provider routing requires DX12 backend/device in `pNext`. | `renderWidth`, `displayWidth`; caller `int32_t *pOutPhaseCount`. | Scalar output; no retention in inspected FSR3 implementation. | **Unresolved normative rounding:** pinned technique documentation specifies custom phase length `ceil(8*n²)`, but pinned FSR3 source computes `int32_t(8*n²)`, i.e. truncation for positive non-integral values. Preset examples still give Quality 18, Balanced 23, Performance 32, Ultra Performance 72. | Optional general utility. The temporal algorithm needs a suitable jitter sequence, but this Query is not mandatory. |
| `ffxQueryDescUpscaleGetJitterOffset` / `...GETJITTEROFFSET` | Null and live context supported; same external-provider routing rule. | Inputs `index`, `phaseCount`; caller `float *pOutX`, `float *pOutY`. | Scalar results. In-tree provider does not retain output pointers. | FSR3 uses base-2/base-3 Halton components at `(index % phaseCount)+1`, shifted by `-0.5`; `phaseCount <= 0` is rejected by the host and becomes a runtime-error return through `TRY2`. | Optional general utility. AMD explicitly permits an application's own jitter sequence generator; the actual applied jitter must still be supplied to dispatch. |
| `ffxQueryDescUpscaleGetGPUMemoryUsage` / `FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE` | **Verified:** live context required; not on the public null-context list. | Caller supplies writable `FfxApiEffectMemoryUsage *gpuMemoryUsageUpscaler`; output contains `totalUsageInBytes` and `aliasableUsageInBytes`. | Caller owns output structure. | In-tree FSR3 explicitly rejects missing context, null native context, or null output with `ERROR_PARAMETER`, zeroes output and computes usage for the created context. | Diagnostics/introspection; useful after creation, never a creation prerequisite. |
| `ffxQueryDescUpscaleGetGPUMemoryUsageV2` / `...GPU_MEMORY_USAGE_V2` | **Verified:** intended null-context/pre-create query. It embeds its own device, so no DX12 backend chain is needed solely to provide the device. Optional `ffxOverrideVersion` selects a non-default version. | Application fills `device`, `maxRenderSize`, `maxUpscaleSize`, `flags`, and output pointer. | Caller owns `FfxApiEffectMemoryUsage`. | The in-tree FSR3 provider checks the output pointer and invokes its pre-create memory estimator. Notably, its call into `ffxFsr3UpscalerGetGpuMemoryUsage` does **not** pass the descriptor's `flags`; therefore the header's `flags` field must not be assumed to affect every provider/version identically. | Diagnostics/capacity planning; optional. |
| `ffxQueryDescUpscaleGetResourceRequirements` / `...GET_RESOURCE_REQUIREMENTS` | **Verified:** either live context, to describe that context's selected version, or null context before creation. For a null query, provider/version is selected through DX12 backend/device plus, where needed, `ffxOverrideVersion`. | No external output buffers: `required_resources` and `optional_resources` are written into the descriptor. Bits represent Color, Depth, Motion Vectors, Exposure, Reactive Mask, and Transparency/Composition Mask. | Results are copied integral bitmasks, with no borrowed result lifetime. | Explicitly provider/version-sensitive. Pinned FSR3 implementation reports Color, Depth, MV and Exposure as required, Reactive and T&C masks optional; the Exposure identifier may instead be satisfied through the auto-exposure creation flag. FSR4 documentation explains this Query was added so applications can distinguish the newer optional-mask requirements. | Optional general utility/introspection. It can remove hard-coded provider-specific assumptions, but is not itself declared a dispatch prerequisite. |

### Device capability/support information

**Verified.** There is no `ffxQueryDescUpscale...Capabilities` or equivalent public upscaler descriptor in the pinned header. `GetVersions` is the nearest device-sensitive discovery operation: its `device` participates in provider enumeration and `IsSupported(device)` filtering. That can establish which provider versions are offered for that supplied device, but it does not return a hardware-capability structure and is not equivalent to a general device-capability query.

### Null-context Query behavior

**Verified.** AMD explicitly lists the following upscaler-relevant null-context operations: `GetVersions`, quality-to-ratio, quality-to-render-resolution, jitter phase count, jitter offset, GPU-memory V2, and resource requirements. All other Query descriptor types require a valid context.

For null-context queries other than `GetVersions`, the generic implementation selects a provider from the query type, optional `ffxOverrideVersion`, and a device extracted from the descriptor chain. If no provider can be found, it returns `FFX_API_RETURN_NO_PROVIDER`; when no device is present, the pinned implementation explicitly diagnoses that a DX12 backend descriptor must be chained to reach driver/external providers.

`GetVersions` differs: the generic core handles it directly using `createDescType`, its embedded `device`, capacity and output arrays. Provider enumeration constructs/checks an external-provider slot where applicable and filters providers by their support for the descriptor/device. It is therefore not justified to model version enumeration as a mathematically pure lookup.

**Unresolved:** the public contract does not specify whether any particular null Query causes a DLL/provider to be dynamically loaded at that moment versus consulting providers/runtime components already staged by the loader. The source establishes provider selection/enumeration, not a stable loading-timing guarantee.

## Configure operations

| Descriptor / operation | Context / timing | State affected and values | Provider specificity / failure | Ordinary-operation requirement |
|---|---|---|---|---|
| `ffxConfigureDescGlobalDebug1` / `FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1` | **Verified:** may be called with null context. Generic source intercepts it before any context/provider lookup. It can therefore be used before creation. | `fpMessage` plus `debugLevel`: `SILENCE`, `ERRORS`, `WARNINGS`, or `VERBOSE` constants. It changes the global runtime message callback/level. | Generic FidelityFX debug facility. Null header produces `ERROR_PARAMETER`; the intercepted valid descriptor returns OK after setting global state. Thread/concurrency rules for changing the callback are not documented. | Tuning/debug; not required. |
| `ffxConfigureDescGlobalDebug` / `FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG` | Same null-context/global interception; also works independently of a live effect context. | Fields are `effectId`, `fpMessage`, `debugLevel`. | **Unresolved:** although the public struct contains `effectId`, the pinned top-level implementation shown here calls the same global callback setter and does not consume `effectId`. Therefore effect-scoped semantics must not be inferred from the field alone. | Tuning/debug; not required. |
| `ffxConfigureDescUpscaleKeyValue` / `FFX_API_CONFIGURE_DESC_TYPE_UPSCALE_KEYVALUE` | **Verified:** requires a valid upscaler context. Generic Configure documentation requires a created context unless descriptor-specific documentation says otherwise, and the FSR3 provider explicitly validates `context` and `*context`. | Persistent context tuning constants; not a creation descriptor and not per-frame dispatch state. The public struct has `key`, `u64`, `ptr`; all five published upscaler keys use a float obtained via `ptr`. | Provider-sensitive. Pinned Windows FSR3 supports the descriptor and ignores `u64` for these keys; Xbox rejects this descriptor. Unknown key reaches `FFX_ERROR_INVALID_ENUM` in FSR3 and `TRY2` maps non-`FFX_OK` host failures to `FFX_API_RETURN_ERROR_RUNTIME_ERROR`. | Tuning/debug/provider-specific; not required. |

### Upscaler key/value keys

| Key | Pinned documented value | State/effect |
|---|---|---|
| `FFX_API_CONFIGURE_UPSCALE_KEY_FVELOCITYFACTOR` (`0`) | Float via `ptr`; default `1.0`; clamped `[0,1]`. | Velocity-related internal constant; `0` is documented as potentially improving temporal stability of bright pixels. |
| `FFX_API_CONFIGURE_UPSCALE_KEY_FREACTIVENESSSCALE` (`1`) | Float; default `1.0`; clamped `[0,+∞)`. | Explicitly described as intended for development/testing of ghosting/reactive-mask behavior. |
| `FFX_API_CONFIGURE_UPSCALE_KEY_FSHADINGCHANGESCALE` (`2`) | Float; default `1.0`; clamped `[0,+∞)`. | Scales the FSR3.1-computed shading-change value. |
| `FFX_API_CONFIGURE_UPSCALE_KEY_FACCUMULATIONADDEDPERFRAME` (`3`) | Float; default approximately `0.333`; clamped `[0,1]`. | Accumulation/ghosting/flicker tuning. |
| `FFX_API_CONFIGURE_UPSCALE_KEY_FMINDISOCCLUSIONACCUMULATION` (`4`) | Float; default approximately `-0.333`; clamped `[-1,1]`. | Disocclusion accumulation / ghosting/flicker tuning. |

**Verified, FSR3 implementation only.** Passing `ptr == nullptr` to one of these keys resets that FSR3 internal constant to its coded default. The Configure call mutates CPU-side persistent context constants immediately and does not recreate the context in the inspected implementation. This null-pointer reset behavior and exact visibility semantics are not documented as a provider-independent API contract.

**Unresolved:** v2.3.0 public material does not define whether `ffxConfigure` may race with a dispatch on the same context, whether GPU completion is required before changing a key, or the exact frame at which a concurrent/in-flight change becomes observable. No such synchronization guarantee should be inferred from the absence of GPU work in the inspected setter.

## Quality-mode helpers

The pinned public quality enumeration is:

| Constant | Value | Nominal per-dimension ratio |
|---|---:|---:|
| `FFX_UPSCALE_QUALITY_MODE_NATIVEAA` | 0 | 1.0x |
| `FFX_UPSCALE_QUALITY_MODE_QUALITY` | 1 | 1.5x |
| `FFX_UPSCALE_QUALITY_MODE_BALANCED` | 2 | 1.7x |
| `FFX_UPSCALE_QUALITY_MODE_PERFORMANCE` | 3 | 2.0x |
| `FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE` | 4 | 3.0x |

These values and ratios are present directly in the pinned public header.

**Verified.** Quality mode is not a field of `ffxCreateContextDescUpscale`. Creation takes flags plus independent `maxRenderSize` and `maxUpscaleSize`; the quality enumeration is consumed by utility Query descriptors instead.

**Inference.** A quality mode therefore does not define a context's maxima. It can be used to select a convenient runtime render size, while the created context's limits remain the explicit maxima supplied at creation. The pinned FSR3 host independently validates dispatched render/upscale dimensions against those maxima.

For the in-tree FSR3 provider, render-resolution conversion is:

`render_dimension = uint32_t(float(display_dimension) / ratio)`

so positive fractional results are truncated and no separate alignment step occurs.

The quality helpers are not strictly provider-independent functions despite their common public descriptor types. Null-context execution is routed to a selected provider/device/version, and a live query is routed to the context's associated provider. Thus the named presets form a common API vocabulary, while exact implementation/error/rounding behavior may remain provider/version-sensitive.

The jitter helper has a pinned source/documentation discrepancy for arbitrary custom scale factors: documentation says `ceil(8*n²)`, while FSR3 source performs an integer conversion of `8*n²`. **Unresolved:** which rule is the normative cross-provider v2.3.0 contract.

## Minimal required sequence

For an ordinary upscaler context using the selected/default provider:

```text
load runtime
    -> create context
    -> first dispatch
```

**Verified:** no public v2.3.0 Query or Configure operation is stated as a universal prerequisite between successful context creation and first dispatch. `ffxDispatch` requires a valid created context; the Query and Configure documentation defines available operations but does not insert either into the validity preconditions for dispatch.

There is a separate creation requirement that is not Query/Configure: v2.3.0's upscaler interface includes `ffxCreateContextDescUpscaleVersion`, whose `version` must be `FFX_UPSCALER_VERSION`; AMD's pinned ML-upscaler documentation says this descriptor has been necessary since SDK 2.1 for API compatibility.

Particular cases alter what is needed:

- **Version override:** `GetVersions` becomes an API-validity prerequisite for obtaining the version ID; IDs must not be hard-coded. Null-context utility queries intended to describe that same selected version must use the same override.
- **Jitter:** temporal upscaling requires the application to render using an appropriate jitter and report the applied offset, but AMD explicitly permits either the supplied jitter Query or an application's own sequence generator. Therefore `GetJitterPhaseCount`/`GetJitterOffset` themselves are optional utilities.
- **Provider-dependent input masks:** `GetResourceRequirements` provides a runtime way to discover differences such as FSR4's optional reactive/T&C masks, but the documentation does not state that invoking the Query is itself mandatory before dispatch.
- **GPU-memory queries:** both forms are measurement/planning facilities; AMD documents the live form after creation and V2 before creation. Neither is a validity prerequisite.
- **Configure:** global debug and constant overrides are debug/tuning facilities; none is required for normal operation.

The pinned DX12 sample does not demonstrate a minimal query-free integration: it enumerates versions, queries resource requirements, performs pre-create GPU-memory and resolution queries, uses live-context jitter queries, and exposes tuning controls in its UI. Those are sample integration choices and instrumentation/utilities, not evidence that the calls are universally mandatory.

## Live-context semantics

For a non-null context, generic `ffxQuery` finds the provider associated with that context and calls that provider's Query implementation. The generic API does not declare Query to be pure or immutable; the provider receives the context and a mutable query descriptor. The inspected FSR3 handlers write requested outputs and, where relevant, read context state; no general no-mutation contract exists.

The following remain **Unresolved** from the public v2.3.0 contract:

- whether Query is permitted concurrently with dispatch on the same context;
- whether a live Query requires prior GPU completion;
- whether an external provider may internally mutate state while servicing a Query;
- whether Configure is safe concurrently with dispatch;
- exact dispatch visibility timing of a Configure change.

The FSR3 constant setter's direct CPU-side assignment establishes implementation behavior, but not a cross-provider synchronization contract.

## M5 evidence classification

| Operation | Descriptive classification | Required for construction? | Required before first dispatch? | Qualification |
|---|---|---:|---:|---|
| `ffxQueryDescGetVersions` | Optional general utility | No* | No | *Conditionally an API-validity prerequisite if an explicit version override is chosen. |
| `ffxQueryGetProviderVersion` | Diagnostics/introspection | No | No | Reports provider actually attached to a live context. |
| Quality → upscale ratio | Optional general utility | No | No | Provider-routed helper. |
| Quality → render resolution | Optional general utility | No | No | Provider-routed helper. |
| Jitter phase count | Optional general utility | No | No | Jitter is required for correct temporal integration; this particular helper is not. |
| Jitter offset | Optional general utility | No | No | Application-owned jitter generator is explicitly permitted. |
| GPU memory usage, live | Diagnostics/introspection | No | No | Post-create measurement. |
| GPU memory usage V2 | Diagnostics/introspection / planning | No | No | Designed for pre-create estimation. |
| Resource requirements | Optional general utility / introspection | No | No | Especially useful where selected providers have different required/optional inputs. |
| `GlobalDebug1` | Tuning/debug/provider-independent global facility | No | No | Mutates global runtime debug state. |
| `GlobalDebug` | Tuning/debug; exact effect scoping unresolved | No | No | Top-level pinned implementation does not use its `effectId`. |
| Upscale key/value Configure | Tuning/debug/provider-specific | No | No | Five published float overrides; normal rendering does not require them. |

No enumerated operation falls into the unconditional **Required for construction** or **Required before first dispatch** classes. The only conditional dependency found is version enumeration before using an explicit version override.

## Caller obligations

| Obligation | Classification | Evidence |
|---|---|---|
| `header.type` must exactly match the descriptor structure. | **Safety requirement** | AMD explicitly calls mismatch undefined behavior / likely crash. |
| A descriptor documented as live-context-only must receive a valid, still-live context. | **Safety requirement / API validity rule** | Configure default rule and Query null-context whitelist. |
| Writable Query output pointers/arrays must point to sufficient valid storage while the call writes them. | **Safety requirement** | Query outputs are written through caller pointers; `GetVersions` uses caller-provided capacity. |
| `GetVersions.outputCount` must carry the intended array capacity when arrays are supplied. | **API validity rule** | Explicit header semantics. |
| Version IDs used in `ffxOverrideVersion` must come from an appropriate `GetVersions` query and must not be hard-coded. | **API validity rule** | Explicit version-selection documentation. |
| Null-context query intended to describe the version later/also selected at creation must carry the same version override. | **API validity rule** | Explicit documentation. |
| A null-context query lacking an embedded device needs a chained DX12 backend/device descriptor when it must reach a driver/external provider. | **API validity rule** | Without it, generic source returns `NO_PROVIDER` when no provider route exists. |
| Retained `GetVersions.versionNames` should be copied rather than retaining returned pointers across later version queries. | **Ergonomic/recommended rule with lifetime implications** | AMD warns global name memory may be overwritten. |
| For the five FSR3 key/value controls, `ptr` must denote a readable `float` during the call when non-null. | **Safety requirement for the inspected FSR3 provider** | Provider passes `ptr` to a setter which immediately dereferences it as `float*`; `u64` is unused. |
| Concurrent Query/Configure/Dispatch and GPU-completion ordering. | **Unresolved** | No public synchronization contract was found. |

## Source-versus-runtime boundary

No runtime behavior is claimed here beyond the supplied project baseline. In particular, the in-tree FSR3 implementation is source evidence for the provider shipped in the SDK source; it does not establish undocumented behavior of every signed/external provider that the loader may select.

The public Query interface is provider-routed. Consequently, helper operations that look arithmetical—quality conversion and jitter generation included—must not automatically be classified as provider-independent pure functions solely from their descriptor shape.

## Baseline delta

### Confirms

**Verified against source:** the project's existing count-only `ffxQueryDescGetVersions` experiment exercises a genuine first-class public v2.3.0 pattern. The generic core explicitly treats `outputCount != nullptr` with either initial count zero or both result arrays null as a provider-count query, and returns `FFX_API_RETURN_OK` after writing the count.

The supplied observed `OK / count 2` therefore establishes that this raw generic Query path and the staged runtime's provider enumeration worked in that specific process/runtime state. It does not by itself establish device-specific support, live-context Query behavior, external-provider routing with a DX12 device, or any Configure behavior.

### Adds

This investigation establishes the broader pinned surface beyond that single raw query:

- two relevant generic Query descriptors: version enumeration and live provider metadata;
- seven public upscaler Query descriptors: quality ratio, render resolution, jitter phase count, jitter offset, live GPU memory, pre-create GPU memory V2, and resource requirements;
- two generic global-debug Configure descriptors;
- the upscaler key/value Configure descriptor and its five published keys;
- the documented null-context whitelist and the DX12 backend/device routing rule;
- provider-owned/global version-name lifetime concerns;
- the distinction between pre-create memory estimation and live-context memory measurement;
- provider/version-sensitive resource requirements;
- quality modes as utility inputs rather than creation parameters;
- the absence of a dedicated upscaler device-capability Query;
- the absence of any unconditional Query/Configure prerequisite between successful context creation and first dispatch.

### Contradicts

Nothing in the supplied M4 evidence is contradicted.

The source does narrow one possible overinterpretation of that evidence: a successful null-context, null-device `GetVersions` count is evidence of provider enumeration in that runtime state, not evidence that all null-context queries are device-free. Most upscaler null-context queries require a DX12 backend/device chain to reach a driver/external provider.

### Leaves unresolved

1. **Jitter phase rounding across providers.** Pinned documentation says custom sequence length is `ceil(8*n²)` while pinned FSR3 code truncates the computed value to `int32_t`.
2. **Signed/external-provider helper details.** The interface and routing are public, but the closed provider's exact rounding, invalid-input and key/value behavior cannot be derived from the in-tree FSR3 implementation.
3. **Query/Configure concurrency and GPU synchronization.** No public v2.3.0 guarantee was found.
4. **`ffxQueryGetProviderVersion.versionName` retention lifetime.** It is a borrowed pointer, but an exact invalidation point is not documented.
5. **`ffxConfigureDescGlobalDebug.effectId` semantics.** The field is public, but the pinned generic implementation installs the callback globally without consuming that field.
6. **Cross-provider pointer retention for Query/Configure pointer fields.** The inspected FSR3 provider reads/writes them synchronously and retains none, but no explicit general provider contract was found.

No claim in this record depends on current `main`; no newer-main descriptor delta is asserted.

## Limits and follow-up

Where the remaining issue concerns the behavior of the actual signed provider rather than a missing normative synchronization guarantee, a bounded signed-runtime experiment can characterize the concrete v2.3.0 runtime:

- For quality/render/jitter helpers, invoke the null-context form with an actual DX12 device and an explicit version ID obtained from `GetVersions`, then compare with the same queries on a context created with that override. Include non-integral custom scale ratios so the experiment distinguishes truncation from `ceil` behavior.
- For resource requirements and GPU-memory V2, issue the query with the same device/version/create parameters used for the subsequent context and compare the returned requirements with the corresponding live-context query where an equivalent live form exists.
- For upscaler key/value Configure on a signed provider, invoke each documented key on an idle live context using valid in-range float storage and record whether the provider returns `OK`, `UNKNOWN_DESCTYPE`, `PROVIDER_NO_SUPPORT_NEW_DESCTYPE`, or another error. This distinguishes support/acceptance; determining visual or dispatch-time effects belongs outside this source investigation.
- Pointer-retention experiments can characterize a current signed provider but cannot establish a normative lifetime guarantee. Likewise, absence of a crash under concurrent Query/Configure/Dispatch cannot establish thread safety. Those two questions require an upstream contract if they become safety-critical rather than being inferred from runtime observations.

The bounded evidence therefore supports the factual conclusion that v2.3.0 exposes a substantial Query utility/introspection surface and a small debug/tuning Configure surface, while ordinary successful upscaler construction and entry into first dispatch do not, by the public API contract examined here, require a Query or Configure call.
