# M5 source provenance repair: safe construction and Query/Configure

> Intake note (2026-09-23): moved from `to-be-sorted-repair.md` to the dated
> source-record path. Removed 107 opaque `:chatgpt-content-reference` markers;
> their index mapping was not supplied. The pinned URLs in the source table and
> the substantive findings are retained. Source IDs below identify the cited
> primary materials, but the removed markers do not establish a claim-by-claim
> citation map. This is a provenance limit, not a new source verification.

**Investigation date:** 2026-09-22  
**Topic:** `fsr-sdk-rs` M5 construction and Query/Configure provenance repair  
**Scope:** AMD FidelityFX/FSR SDK modern API only; Windows x64/MSVC; DirectX 12; official signed AMD runtime as the project deployment target.  
**Pinned baseline:** FSR SDK v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. GitHub records tag `v2.3.0` at that commit.  
**Evidence cut-off:** The pinned commit is normative. The v2.3.0 release record is used only to establish release metadata and explicitly documented v2.3.0 changes. No behavior from later `main` is imported.

## Method and exclusions

This record reconstructs the relevant claims from pinned public headers, pinned documentation, pinned samples, and pinned public implementation source. Source-built FSR2/FSR3 behavior is kept distinct from the signed/external FSR4 provider. The public provider layer references `amdinternal/api/internal/dx12/ffx_provider_external.h`; that implementation is not present in the public source inspected here, so source inspection cannot establish its hidden validation or lifetime rules.

No native runtime experiment, binary reverse engineering, production ABI modification, Rust API design, Builder/D008 decision, Runtime ownership decision, `Send`/`Sync` decision, Dispatch synchronization investigation, or Bevy/wgpu work was performed.

Evidence labels used below:

| Label | Meaning |
|---|---|
| **Verified** | Directly supported by identified pinned primary evidence. |
| **Upstream claim** | AMD explicitly states the behavior, but it was not independently reproduced here. |
| **Inference** | Follows from identified evidence but is not itself a normative upstream statement. |
| **Unresolved** | The pinned public evidence is insufficient. |

## Inputs

Unless otherwise stated, every source below is repository **`GPUOpen-LibrariesAndSDKs/FidelityFX-SDK`**, revision **`60f4ea81909200d8542eca14dccb2628b763a9a3` / tag `v2.3.0`**, accessed **2026-09-22**.

| ID | Exact path / material | Relevant context | Direct pinned source |
|---|---|---|---|
| U0 | GitHub release `v2.3.0` | tag→commit mapping; FSR Upscaling 4.1.1 RX 7000 addition; signed prebuilt distribution; `ffxConfigureDescGlobalDebug` release description | [v2.3.0 release](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0) |
| U1 | `Kits/FidelityFX/upscalers/include/ffx_upscale.h` | `FFX_UPSCALER_VERSION`; quality modes; create flags/root descriptor; Query/Configure ABI; version descriptor | [ffx_upscale.h@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L29-L206) |
| U2 | `Kits/FidelityFX/api/include/ffx_api.h` | return codes; base header; global debug; version enumeration/provider metadata; create pointer-lifetime comment; API calls | [ffx_api.h@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h#L29-L145) |
| U3 | `Kits/FidelityFX/api/include/ffx_api.hpp` | C++ chain order; `CreateContext`; descriptor `DynamicCast` | [ffx_api.hpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.hpp#L59-L173) |
| U4 | `Kits/FidelityFX/api/internal/ffx_api.cpp` | provider/version selection; whole-chain override scan; global-debug interception; null-context Query routing | [ffx_api.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp#L28-L151) |
| U5 | `Kits/FidelityFX/docs/getting-started/ffx-api.md` | descriptor UB; Query output/null-context rules; Configure; version selection/name lifetime; error handling | [ffx-api.md@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md#L29-L254) |
| U6 | `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` | `ffxCreateBackendDX12Desc` and `ID3D12Device*` | [ffx_api_dx12.h@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h#L42-L47) |
| U7 | `Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp` | DX12 backend whole-chain scan; null-device rejection; `GetDevice` whole-chain scan | [ffx_backends_dx12.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp#L37-L142) |
| U8 | `Kits/FidelityFX/api/internal/ffx_backends.h` | `MustCreateBackend` | [ffx_backends.h@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_backends.h#L26-L35) |
| U9 | `Kits/FidelityFX/api/internal/ffx_provider.h` | `IsSupported`; external-provider boundary; provider enumeration/filtering; version-name storage | [ffx_provider.h@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h#L53-L271) |
| U10 | `Kits/FidelityFX/api/internal/ffx_message.cpp` | module-static message callback/debug level | [ffx_message.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_message.cpp#L37-L65) |
| U11 | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` | FSR3 create adapter; backend requirement; Query inventory; key/value Configure | [ffx_provider_fsr3upscale.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L49-L304) |
| U12 | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp` | maxima-backed resources; dispatch bounds; quality/render helpers; immediate key/value reads | [ffx_fsr3upscaler.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L469-L520); [dispatch/helpers](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1201-L1294); [configure constants](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1376-L1437) |
| U13 | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp` | FSR2 create adapter; backend; Query/Configure handling | [ffx_provider_fsr2.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp#L47-L239) |
| U14 | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr2.cpp` | FSR2 dispatch bound and quality helpers | [ffx_fsr2.cpp@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr2.cpp#L1084-L1168) |
| U15 | `Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md` | FSR3 scaling modes and integration | [FSR3.1.5 documentation@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md#L54-L90) |
| U16 | `Kits/FidelityFX/docs/techniques/super-resolution-ml.md` | signed FSR4 requirement; NativeAA; mandatory API-version descriptor; resource/jitter Queries | [FSR4.1.1 documentation@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#L42-L84); [FSR4 resource/jitter sections](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#L133-L210) |
| U17 | `Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp` | official construction order; pre-create queries; provider metadata query | [FSR DX12 sample@60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp#L857-L952) |

## Construction provenance repair

### Root and required descriptors

**Verified:** `ffxCreateContextDescUpscale` is the upscaler root creation descriptor. Its header type is `FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE`; it carries `flags`, `maxRenderSize`, `maxUpscaleSize`, and an optional message callback. `ffxCreateContext` selects a provider from the root descriptor's type, which makes this root semantically distinct from chained extensions.

**Verified public contract:** v2.3.0 requires `ffxCreateContextDescUpscaleVersion` for the modern upscaler API. Its `version` denotes the API version the application was built against and must be `FFX_UPSCALER_VERSION`; the pinned FSR4 documentation says this descriptor has been necessary since SDK 2.1 and must be linked through `pNext`.

**Verified:** this is not the same namespace or semantic value as explicit provider selection. `ffxOverrideVersion.versionId` must be an ID returned by `ffxQueryDescGetVersions`, whereas `ffxCreateContextDescUpscaleVersion.version` is the API compatibility value.

For the DX12 baseline, `ffxCreateBackendDX12Desc` carries the `ID3D12Device*`. The public FSR2 and FSR3 providers call `MustCreateBackend`, and the DX12 backend scanner returns `FFX_API_RETURN_ERROR_PARAMETER` when that descriptor's device is null.

### Descriptor-chain ordering

The official DX12 sample constructs the chain through the C++ helper in this order:

`ffxCreateContextDescUpscale` → `ffxCreateBackendDX12Desc` → `ffxCreateContextDescUpscaleVersion` → optional `ffxOverrideVersion`.

Without an override it uses root → backend → API-version. The helper preserves the argument order when linking `pNext`.

The public loader independently scans the whole chain for `ffxOverrideVersion`; the DX12 backend code independently scans chained descriptors for the backend descriptor, and `GetDevice` scans the whole chain for a usable device. Therefore backend and override discovery do not require adjacency.

**Refinement:** this does **not** prove that `ffxCreateContextDescUpscaleVersion` and `ffxCreateBackendDX12Desc` may occupy either relative order for the signed FSR4 provider. The source-built FSR2/FSR3 adapters do not consume the API-version descriptor; the external signed provider that must interpret it is not public. The prior broad “source scans the chain, therefore those two nodes have no fixed relative order” conclusion is not reconstructible for the signed provider. The official sample order is directly established; other signed-provider orders remain **Unresolved**.

### Flags

**Verified:** `ffxCreateContextDescUpscale.flags` explicitly accepts zero or a combination of `FfxApiCreateContextUpscaleFlags`. Thus `flags = 0` is a defined ABI value, not an invalid “missing option” state.

No pinned generic contract assigns a deterministic error to every unsupported or nonsensical flag combination. The source-built FSR2/FSR3 adapters translate the flags they understand, but that does not establish how a closed provider treats every bit pattern.

### Dimensions

At the generic upscaler ABI level, `maxRenderSize` is described as the maximum rendering size and `maxUpscaleSize` as the targeted presentation/upscale size. The pinned header does not state one universal nonzero minimum, maximum numerical dimension, or alignment multiple for these creation fields.

For public **FSR3**, the creation adapter copies both values into the FSR3 context; internal resources are sized from those maxima, and dispatch explicitly rejects render or upscale dimensions greater than the stored maxima with `FFX_ERROR_OUT_OF_RANGE`. This verifies that the FSR3 source treats the creation dimensions as capacity maxima for later dispatch.

For public **FSR2**, `maxRenderSize` is copied as a maximum render size and `maxUpscaleSize` becomes FSR2's fixed `displaySize`; dispatch checks only that the current render dimensions do not exceed `maxRenderSize`. This is not the same per-dispatch upscale-size model as FSR3.

For **FSR4**, the pinned documentation requires the signed binary distribution, so its exact creation-time numerical rejection rules are outside the public source.

NativeAA is defined generically at 1.0× and is documented by both FSR3 and FSR4 as a 1.0× mode, so equal render/presentation dimensions are clearly intended for providers supporting that mode. However, the bundled FSR2 helper is an important exception: it handles only Quality through Ultra Performance; NativeAA returns no valid ratio and is rejected by its render-resolution helper's enum range check.

Accordingly, the evidence supports **no SDK-wide numerical size rule beyond provider contracts and the ABI's maximum/target semantics**. Exact FSR4 minimums, maximums, alignment requirements, and rejection behavior remain **Unresolved**.

### Device/provider compatibility

The generic provider layer has an `IsSupported(device)` gate. Version enumeration includes providers only when they can provide the requested descriptor type and pass that support check; explicit override selection likewise checks the selected provider.

That makes enumeration with the intended device useful evidence that a provider exposes itself as supported for that device. It is not documented as proof that a later context creation with arbitrary dimensions, flags, resources, memory conditions, or runtime state must succeed. That stronger interpretation would be an **Inference** unsupported as a complete compatibility contract.

The public FSR2 and FSR3 provider classes contain no provider-specific `IsSupported` override, so they inherit the base implementation that returns true. This says only that their **provider-selection** gate imposes no extra device filter; backend/context creation may still encounter device or runtime failures.

FSR4 is different in kind: its implementation is external/signed. The v2.3.0 release explicitly added FSR Upscaling 4.1 support for Radeon RX 7000-series discrete GPUs, while the pinned FSR4 documentation includes reference numbers for both RX 9000- and RX 7000-series cards. Neither source constitutes a complete compatibility matrix or exact construction predicate.

### Error behavior and malformed descriptors

The generic ABI exposes `OK`, generic `ERROR`, `ERROR_UNKNOWN_DESCTYPE`, `ERROR_RUNTIME_ERROR`, `NO_PROVIDER`, `ERROR_MEMORY`, `ERROR_PARAMETER`, and `PROVIDER_NO_SUPPORT_NEW_DESCTYPE`.

Concrete public cases include null `ffxCreateContext` arguments producing `ERROR_PARAMETER`, absence of a selectable provider producing `NO_PROVIDER`, a null DX12 backend device producing `ERROR_PARAMETER`, and the source-built providers' missing backend ultimately producing generic `ERROR`.

The pinned API documentation explicitly states that a descriptor's `header.type` must correspond to its actual descriptor structure and that failing to do so is undefined behavior, likely resulting in crashes. This is therefore outside the ordinary “invalid parameter returns an error code” contract.

No pinned public contract specifies one deterministic return code for every malformed or unsupported creation dimension/flag combination, particularly for the signed FSR4 provider. Exact signed-provider rejection remains **Unresolved**.

### Creation-pointer lifetime

One generic header comment states that pointers passed in the creation descriptor must remain live until `ffxDestroyContext`. The exact scope of that statement is not further specified: notably, the official sample comments that its `ffxCreateContextDescUpscaleVersion` object itself only needs to survive through `CreateContext`.

Therefore it is **Verified** that the API contains a retained-pointer warning, but **Unresolved** whether it normatively applies to every descriptor node, only pointer-valued descriptor payloads, or a narrower provider-specific subset. This record does not convert that ambiguity into a Rust ownership decision.

## Query and Configure provenance repair

The pinned ABI directly reconstructs the previously broken inventory. No generated citation markers are needed.

| Descriptor / facility | Public evidence and context | Classification for basic Create → Dispatch |
|---|---|---|
| `ffxQueryDescGetVersions` | Enumerates provider/version IDs and names for a creation descriptor type; has a DX12 device field and caller-supplied output capacity/arrays. | **Construction prerequisite only when explicit provider override is requested**; otherwise optional selection/introspection. |
| `ffxQueryGetProviderVersion` | Returns ID/name for an already-created context. | **Diagnostics/introspection**; unrelated to first basic dispatch. |
| `ffxQueryDescUpscaleGetUpscaleRatioFromQualityMode` | Quality-mode input, float output pointer. | **Optional utility**. |
| `ffxQueryDescUpscaleGetRenderResolutionFromQualityMode` | Display dimensions + quality input; caller receives calculated render dimensions. | **Optional utility**. |
| `ffxQueryDescUpscaleGetJitterPhaseCount` | Computes phase count from render/display widths. | **Optional utility**. |
| `ffxQueryDescUpscaleGetJitterOffset` | Computes sequence offset. FSR4 docs explicitly allow using the provided Query or the application's own sequence generator. | **Optional utility**, not an API-call prerequisite. The dispatch still needs the actual applied jitter value. |
| `ffxQueryDescUpscaleGetGPUMemoryUsage` | Writes current context's GPU-memory usage. Public FSR3 implementation requires a context. | **Diagnostics/introspection**, post-create. |
| `ffxQueryDescUpscaleGetGPUMemoryUsageV2` | Takes device, creation maxima and flags; returns pre-create memory estimate. | **Optional utility / diagnostics**, pre-create. |
| `ffxQueryDescUpscaleGetResourceRequirements` | Returns required/optional resource bitmaps. FSR4 documentation explicitly uses it to distinguish FSR4 from older upscalers. | **Optional utility** that can inform dispatch preparation; the Query call itself is not a pre-dispatch prerequisite. |
| `ffxConfigureDescGlobalDebug1` | Global-debug callback/level descriptor. | **Tuning/debug**, unrelated to basic path. |
| `ffxConfigureDescGlobalDebug` | New v2.3 descriptor adding `effectId`, callback and level. | **Tuning/debug**, unrelated to basic path. |
| `ffxConfigureDescUpscaleKeyValue` | Generic upscaler key/value descriptor with five published float tuning keys. | **Tuning/debug**, not an ordinary first-dispatch prerequisite. |

The five ABI-published tuning keys are `FVELOCITYFACTOR`, `FREACTIVENESSSCALE`, `FSHADINGCHANGESCALE`, `FACCUMULATIONADDEDPERFRAME`, and `FMINDISOCCLUSIONACCUMULATION`; the header gives defaults and value constraints. The public FSR3 provider forwards this descriptor to `ffxFsr3UpscalerSetConstant`, whose implementation immediately reads and copies the supplied float into context state. This proves immediate consumption for that source-built provider only; it is not a cross-provider pointer-retention contract.

The public FSR2 adapter does not implement the upscaler key/value case; its Configure switch handles `GLOBALDEBUG1` only. Consequently the existence of the generic five-key ABI must not be interpreted as proof of equivalent tuning support in every provider. Signed FSR4 behavior is **Unresolved** from public source.

### Null-context Queries

The pinned API documentation provides the authoritative public null-context list. Relevant upscaler members are version enumeration, quality ratio, render resolution, jitter phase count, jitter offset, memory V2, and resource requirements. All other Query descriptor types require a valid created context; this excludes live GPU-memory usage and live provider metadata.

For null-context Queries that do not embed a device, AMD documents chaining `ffxCreateBackendDX12Desc` when routing to driver/external providers; without it, the call can return `FFX_API_RETURN_NO_PROVIDER`. `ffxQueryDescGetVersions` and memory V2 already embed the device and do not require that extra chained device descriptor.

### Quality modes are utilities, not creation fields

`FfxApiUpscaleQualityMode` is consumed by the quality/render-resolution Query descriptors; `ffxCreateContextDescUpscale` itself contains flags and maximum dimensions but no quality-mode field. Thus selection of “Quality”, “Balanced”, etc. is not an independent required creation parameter in the generic ABI.

The exact arithmetic used by the public FSR3 helper is float division followed by conversion to `uint32_t`; that is provider implementation evidence, not a generic rounding contract and not evidence for the closed signed FSR4 provider. Closed-provider helper rounding remains **Unresolved**.

### Version enumeration and provider metadata storage

Explicit provider override is optional, but when used AMD requires IDs obtained from `ffxQueryDescGetVersions` for the appropriate creation type and warns not to hard-code them. The same override must be used for relevant null-context Queries.

Version enumeration uses caller-owned storage: `outputCount` is both input capacity and output count, while `versionIds` and `versionNames` point to arrays supplied by the caller. AMD's example performs a count query, allocates vectors, then performs a second query.

AMD explicitly warns that some version names are backed by global memory and may be overwritten by later version queries, recommending that applications copy them. The public loader corroborates one concrete case: its external-provider enumeration uses a static 64-byte name buffer after copying the name out of the external DLL.

This does **not** establish an exact invalidation point for every provider name, nor the lifetime of `ffxQueryGetProviderVersion.versionName` across every provider. Those stronger lifetime rules remain **Unresolved**.

### Memory and resource Queries

Live `ffxQueryDescUpscaleGetGPUMemoryUsage` is a context query; memory V2 exists specifically for estimating the default or explicitly overridden provider before context creation. The pinned FSR4 documentation states both uses directly.

Resource requirements are provider/version dependent by design: with a live context they describe that context's version; before creation, FSR4 documentation directs the caller to a null-context Query with the desired version override. Public FSR2 and FSR3 adapters both expose required/optional bitmaps, while the signed FSR4 provider implementation itself is unavailable.

### Global debug

The ABI exposes both global-debug descriptors, and v2.3.0 release notes describe the newer `ffxConfigureDescGlobalDebug.effectId` as enabling per-effect debug routing.

The pinned public loader implementation, however, intercepts both debug descriptor types before context/provider routing and calls the same `ffxSetPrintMessageCallback` path; it does not consult `effectId`. That callback and debug level are stored in module-static variables in `ffx_message.cpp`.

Therefore **Verified source behavior** is broader than a context-local setting within that public loader module. Calling it “process-global” is too strong: cross-DLL/process scoping and the packaged signed runtime's exact handling are **Unresolved**. There is also a primary-source discrepancy between the v2.3.0 release description of per-effect routing and this pinned loader implementation; source inspection cannot reconcile it.

### Is Query or Configure mandatory before first Dispatch?

The generic Dispatch documentation requires a valid context but does not specify an intervening Query or Configure call. The public FSR3 provider's dispatch path likewise consumes the dispatch descriptor directly; the memory, helper, resource-requirement and tuning operations are separate Query/Configure cases.

**Inference, supported by the public API contract:** no unconditional application-side `ffxQuery` or `ffxConfigure` call is required between an otherwise successful ordinary context creation and the first valid upscaler `ffxDispatch`. Individual dispatch inputs still have their own requirements; using a helper Query is not the only way to obtain them.

### Query/Configure lifetime and concurrency

Query output is documented as being written through pointers supplied in the Query descriptor, and version enumeration expressly uses caller-provided capacity and arrays. No generic statement was found granting the runtime ownership of those output buffers after return.

Conversely, no generic pinned statement was found that guarantees every Query or Configure implementation consumes every supplied pointer synchronously and never retains it. Public FSR3's five tuning values are copied immediately, but that implementation cannot establish signed-provider behavior. General Query/Configure pointer retention is therefore **Unresolved**.

No pinned public contract establishing thread safety, permitted concurrent Query/Configure/Dispatch combinations, or cross-context synchronization semantics was found. General concurrency remains **Unresolved**.

## Claim/status/source table

| Supplied M5 conclusion | Status | Evidence result |
|---|---|---|
| `ffxCreateContextDescUpscale` is the root descriptor. | **Confirmed** | **Verified**, U1/U4. |
| `ffxCreateContextDescUpscaleVersion` is mandatory and contains `FFX_UPSCALER_VERSION`. | **Confirmed** | **Verified public contract**, U1/U16; runtime enforcement by closed FSR4 not inspected. |
| API version is distinct from provider/version ID. | **Confirmed** | **Verified**, U1/U2/U5. |
| DX12 construction uses `ffxCreateBackendDX12Desc`. | **Confirmed** | **Verified**, U6/U8/U11/U13. |
| Inspected DX12 backend rejects null device. | **Confirmed** | **Verified**, returns `ERROR_PARAMETER`. |
| Official sample ordering differs from project M4; source proves backend/version nodes have no fixed relative order. | **Refined** | Sample order is **Verified**. Whole-chain backend/override scans are **Verified**, but signed-provider handling of the mandatory API-version node is unavailable; arbitrary backend↔API-version ordering is **Unresolved**. |
| `flags = 0` is valid. | **Confirmed** | **Verified** by ABI comment. |
| Exact generic numerical domain of creation sizes is incompletely documented. | **Confirmed** | **Verified absence of a stated generic numeric domain** in the relevant pinned ABI/docs; actual closed-provider limits remain **Unresolved**. |
| No established single SDK-wide min/max/alignment rule. | **Confirmed** | No such normative rule was identified; this is an evidence boundary, not proof that every provider accepts arbitrary sizes. |
| Equal render/output sizes are intended for NativeAA-capable providers. | **Confirmed / refined** | Generic ABI, FSR3 and FSR4 specify NativeAA=1.0×; bundled FSR2 helper does not support that mode. |
| FSR3 creation sizes are maxima for later dispatch. | **Confirmed** | **Verified** by allocation and dispatch bounds. |
| Provider/device requirements differ between FSR2/FSR3/FSR4. | **Refined** | FSR2/3 public selection uses inherited `IsSupported=true`; FSR4 is external/signed and v2.3.0 explicitly adds RX7000 support. Complete provider/device predicates are **Unresolved**. |
| Enumeration with a device is useful availability evidence, not complete compatibility proof. | **Confirmed** | Provider filtering by `IsSupported(device)` is **Verified**; “not a complete proof” is a conservative **Inference** because creation has additional failure paths. |
| Generic API has parameter/runtime/memory/no-provider errors but not deterministic codes for every malformed size/flag combination. | **Confirmed** | **Verified** enum; complete per-input mapping is **Unresolved**. |
| Type/layout mismatch may be UB. | **Confirmed**, stronger | Pinned documentation explicitly calls incorrect descriptor `type` undefined behavior. |
| Generic version enumeration exists. | **Confirmed** | **Verified**, U2/U5. |
| Live provider metadata Query exists. | **Confirmed** | **Verified**, U2. |
| Quality-ratio/render-resolution Queries exist. | **Confirmed** | **Verified**, U1. |
| Jitter-phase/jitter-offset Queries exist. | **Confirmed** | **Verified**, U1/U16. |
| Live GPU-memory and pre-create V2 Queries exist. | **Confirmed** | **Verified**, U1/U16. |
| Resource-requirements Query exists. | **Confirmed** | **Verified**, U1/U16. |
| Global debug Configure descriptors exist. | **Confirmed / refined** | ABI is **Verified**; exact scoping is narrower/less certain than “process-global.” |
| Upscaler key/value Configure has five published float tuning keys. | **Confirmed / refined** | ABI is **Verified**; public FSR3 supports them, FSR2 does not implement the key/value case, signed FSR4 support remains **Unresolved**. |
| No unconditional Query/Configure is required between ordinary successful creation and first dispatch. | **Confirmed** | **Inference** from the public call contract and provider path; no such prerequisite is specified. |
| Provider override requires obtaining an ID through version enumeration. | **Confirmed** | **Verified**; returned IDs only, no hard-coding. |
| Quality modes are Query inputs, not creation fields. | **Confirmed** | **Verified**, U1. |
| Only certain Queries permit null context. | **Confirmed** | **Verified**, explicit public list. |
| Some null-context helpers need DX12 backend/device information to reach external providers. | **Confirmed** | **Verified**; GetVersions and memory V2 embed device. |
| Some version names are global/provider-owned and may be overwritten by later queries. | **Confirmed / refined** | AMD warning and external-name static buffer are **Verified**; exact lifetime for every returned name remains **Unresolved**. |
| Query arrays/buffers require caller-owned writable storage with sufficient capacity. | **Confirmed** | **Verified**, especially `GetVersions`. |
| Live memory requires context; V2 permits pre-create estimation. | **Confirmed** | **Verified**. |
| Resource requirements are provider-dependent. | **Confirmed** | **Verified** by context/version-specific documentation and differing provider implementations. |
| Global debug has broader module/process implications. | **Refined** | Public loader is **Verified module-static** and ignores `effectId`; process-wide/cross-module scope is **Unresolved**. |
| No general Query/Configure concurrency or pointer-retention contract was established. | **Confirmed as unresolved** | FSR3 immediate-copy behavior is provider-specific; no generic concurrency/lifetime guarantee was found. |

## Substantive corrections

1. **Descriptor ordering must be stated more narrowly.** Public code proves whole-chain scanning for the DX12 backend, device discovery, and provider override. It does not expose signed FSR4's handling of `ffxCreateContextDescUpscaleVersion`; therefore it cannot prove arbitrary relative ordering of the backend and API-version descriptors. The only directly auditable signed-provider-oriented construction order is the official sample's root → backend → API-version → optional override.

2. **NativeAA is not uniformly supported by all bundled historical providers.** The generic ABI, FSR3 and FSR4 define 1.0× NativeAA, but the pinned FSR2 quality/render helpers exclude it. Any prior wording implying FSR2 shares generic NativeAA behavior requires correction.

3. **`ffxConfigureDescGlobalDebug.effectId` has an unresolved implementation/documentation discrepancy.** The v2.3.0 release describes per-effect routing, but the pinned public loader ignores `effectId` and writes one module-static callback/debug-level pair. A safe contract should not promise per-effect or process-global semantics from this source record alone.

4. **Provider enumeration is support evidence, not construction certification.** The implementation filters using `CanProvide`/`IsSupported`, but successful enumeration does not normatively guarantee successful creation for every creation parameter or runtime condition.

5. **Creation has a public retained-pointer warning that previous construction synthesis should not omit.** `ffx_api.h` says pointers passed through the creation description must remain live until context destruction, but the exact intended set of pointers is ambiguous and should not be expanded into an ownership rule without further evidence.

6. **The five key/value tuning keys are ABI inventory, not uniform provider capability.** Public FSR3 implements them; public FSR2 does not implement that Configure descriptor. Signed FSR4 handling cannot be reconstructed from source.

## Baseline delta

**Confirmed:** root upscaler creation descriptor; mandatory API-version descriptor and `FFX_UPSCALER_VERSION`; separation of API version from provider/version ID; DX12 backend descriptor and null-device rejection; validity of zero creation flags; lack of a documented generic numerical dimension domain; FSR3 maximum-size behavior; generic error-code inventory; explicit UB for descriptor type/layout mismatch; the complete listed Query inventory; the two memory Query forms; resource-requirements Query; version enumeration/provider metadata; null-context Query restrictions; external-provider device routing; caller-owned Query outputs; version-name overwrite warning; and absence of an unconditional Query/Configure call in the documented basic Create → Dispatch application path.

**Refined:** descriptor ordering, because public chain scans do not expose signed FSR4's API-version-descriptor handling; NativeAA, because pinned FSR2 helpers do not support it; provider/device differences, because complete FSR4 compatibility is closed and FSR2/FSR3 inherit the same permissive provider-selection `IsSupported`; global debug scope, because source proves module-static state rather than process-global state; provider enumeration, because it is support/availability evidence rather than complete creation compatibility; key/value Configure, because ABI presence is not uniform provider support; and provider-name lifetime, because only some overwrite behavior is specified.

**Contradicted:** none of the supplied high-level conclusions is wholly contradicted after applying the provider-specific qualifications above. The earlier unqualified implications about descriptor ordering, NativeAA uniformity, and global-debug scope are too broad and must not be carried forward unchanged.

**Still unresolved:** exact signed-FSR4 creation dimension rejection; complete FSR4/device compatibility predicate; signed-provider helper rounding; arbitrary backend/API-version descriptor ordering in the signed provider; exact invalidation/lifetime of every borrowed provider name; exact scope of the generic create-pointer lifetime statement; signed-provider key/value pointer retention; global-debug cross-module/process scope and the `effectId` discrepancy; general Query/Configure pointer retention; and general Query/Configure/context concurrency.

## Limits and follow-up

Public source cannot establish the behavior hidden inside `amd_fidelityfx_upscaler_dx12.dll` or the external/driver provider. FSR3 source evidence must therefore not be promoted into an FSR4 runtime guarantee.

The bounded signed-runtime questions that remain observable, if M5 later requires them, are: whether the v2.3.0 signed upscaler accepts/rejects specific boundary creation dimensions and with which return code; whether backend/API-version descriptor order is accepted in both observed permutations; whether provider enumeration followed by identical-device creation can still fail for provider-specific compatibility reasons; what rounding the selected signed provider returns from render-resolution Query; whether returned provider-name pointers change after subsequent version queries; whether `ffxConfigureDescGlobalDebug.effectId` actually isolates messages in the packaged signed loader/provider combination; and whether any signed-provider Configure input pointer is retained after the call. These are discrete observable questions, not evidence supplied by this source-only investigation.

Research evidence here repairs provenance only. It does not select a Rust construction API, establish Runtime ownership, authorize implementation, or decide thread-safety traits.
