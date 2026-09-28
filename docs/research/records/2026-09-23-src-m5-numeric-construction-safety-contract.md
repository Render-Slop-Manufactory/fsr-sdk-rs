# Source investigation record — M5 numerical construction safety contract

> Intake note (2026-09-23): moved from the nested
> `fsr-sdk-rs/docs/research/records/to-be-sorted-numerical.md` staging path.
> The source findings are preserved; no personal paths, credentials or raw
> runtime addresses were present in the supplied record.

**Investigation date:** 2026-09-23  
**Target record:** `docs/research/records/2026-09-23-src-m5-numeric-construction-safety-contract.md`  
**Topic:** Numerical preconditions for safe DX12 upscaler context construction with FidelityFX SDK v2.3.0  
**Result:** **No source-supported bounded domain identified.**

## Bounded research question

For the pinned FidelityFX SDK v2.3.0 DX12 upscaler API, what public AMD contract or auditable provider/backend source establishes preconditions on `maxRenderSize` and `maxUpscaleSize` such that passing those values to `ffxCreateContext` is a valid native operation, and which unsupported or extreme numerical conditions are demonstrably converted into ordinary native errors before any violated numerical/native precondition?

Where the selected signed FSR4/external provider is not auditable, the second question is the narrowest runtime assumption that remains.

This investigation is source-only and supplies evidence rather than an M5 architecture decision, as required by the research brief.

## Project baseline and existing runtime evidence

The target baseline is FidelityFX SDK **v2.3.0**, upstream commit `60f4ea81909200d8542eca14dccb2628b763a9a3`, Windows x64/MSVC, DirectX 12, the modern `ffxCreateContext` API, and the official signed AMD runtime. The proposed public constructor does not yet exist; the private Rust construction path already requires all four dimensions to be `NonZeroU32`.

The [signed-runtime dimension experiment](2026-09-22-exp-m5-upscaler-construction-inputs.md)
is retained exactly as supplied: on one RX 9060 XT system, each structurally
valid descriptor containing one zero creation dimension entered
`ffxCreateContext` but did not return an `ffxReturnCode_t`; the child terminated
with status `0xC0000409`, with Rust reporting that a foreign exception could not
be caught. The experiment did not establish the originating native exception
or mechanism. Conversely, `1×1 -> 2×2`, `1279×719 -> 1919×1079`, equal-size
`1280×720`, and `1920×1080 -> 1280×720` all created and destroyed successfully
and queried provider 4.1.1 on that configuration. These observations are not
generalized beyond the tested provider/device/runtime.

The source question therefore cannot be answered merely by excluding zero: the supplied runtime result already demonstrates successful signed-FSR4 creation at `1×1`, while the open FSR2/FSR3 implementations inspected below have materially different derived-resource behavior at that size.

## Method and exclusions

The investigation inspected the pinned public upscaler ABI, generic API entry points, FSR3 and FSR2 provider adapters and implementations, the DX12 backend, AMD FSR4 documentation, and Microsoft primary D3D12 documentation. Current or later material was not used to project behavior backward onto v2.3.0.

No DLL was executed, no signed binary was reverse engineered, and no behavior of the closed FSR4 provider is inferred from the open FSR2/FSR3 implementations. Rust API shape, ownership, threading, Dispatch resource/synchronization design, frame generation, Vulkan, custom allocators, callbacks, and arbitrary flags remain excluded. These exclusions match the supplied source-only boundary.

## Pinned inputs

| Input | Pinned value |
|---|---|
| FidelityFX SDK | v2.3.0 |
| Upstream commit | `60f4ea81909200d8542eca14dccb2628b763a9a3` |
| Platform/backend | Windows x64/MSVC, DirectX 12 |
| API family | Modern FidelityFX API |
| Creation flags | `0` |
| Device | Valid live supported DX12 device |
| Callback | null |
| Host allocator | default/null |
| Provider override | none |
| Structural descriptor validity | assumed |
| Numerical fields under investigation | both components of `maxRenderSize` and `maxUpscaleSize` |

`FfxApiDimensions2D` stores both coordinates as `uint32_t`. `FfxApiResourceDescription` likewise uses 32-bit unsigned width and height fields.

## Public numerical contract

The pinned upscaler creation descriptor documents `maxRenderSize` as the maximum rendering size and `maxUpscaleSize` as the targeted presentation/upscale size. The generic public ABI does **not** state a numerical minimum, numerical maximum, alignment/granularity requirement, aspect-ratio restriction, or `maxRenderSize <= maxUpscaleSize` requirement.

The published scaling modes include NativeAA at scale factor 1.0, so equality between render and output dimensions is an expressly represented operating mode rather than a generally forbidden relationship.

No pinned public declaration examined establishes the following as an API-wide creation contract:

| Candidate rule | Classification |
|---|---|
| each coordinate must be non-zero | **Unresolved** as public API contract |
| minimum coordinate > 1 | **Unresolved** |
| fixed maximum coordinate | **Unresolved** at upscaler-API level |
| alignment/granularity | **Unresolved** |
| fixed aspect-ratio range | **Unresolved** |
| render dimensions must not exceed upscale dimensions | **Not established** |
| equal dimensions valid | **Verified** as an intended NativeAA configuration |
| arbitrary downscale-shaped maxima valid | **Unresolved** |
| values beyond provider/device capability return an ordinary API error | **Unresolved** |
| arithmetic overflow constraints | **Unresolved** |
| every unsupported numerical pair is safely rejected | **Unresolved** |

The modern generic API defines return codes including `FFX_API_RETURN_ERROR_RUNTIME_ERROR`, `FFX_API_RETURN_ERROR_MEMORY`, and `FFX_API_RETURN_ERROR_PARAMETER`; `ffxCreateContext` returns non-zero on an error when control returns normally. These definitions do not themselves establish which numerical inputs are valid or guarantee that every provider/backend failure reaches such a return.

## FSR3 arithmetic and resource path

The modern FSR3 provider adapter copies `maxRenderSize` and `maxUpscaleSize` into the FSR3 context description and invokes `ffxFsr3UpscalerContextCreate`. Before that call it checks descriptor/context/interface conditions but contains no numerical validation of these coordinates.

The FSR3 core creation routine similarly checks pointers/backend-interface state rather than imposing a numerical extent range before constructing resources. During creation it also assigns the unsigned `maxUpscaleSize.width` and `.height` into `Fsr3UpscalerConstants::maxUpscaleSize`, whose elements are `int32_t`. Thus values greater than `INT32_MAX` undergo a narrowing/sign-changing conversion before any D3D12 resource-dimension rejection can establish an upscaler bound. This is a concrete numerical transition; source inspection alone does **not** demonstrate that the resulting value causes memory unsafety during creation.

The creation resource descriptions expose a more immediate provider-specific lower-bound issue. Several resources use the maximum render dimensions directly, while `shadingChange`, `spdMips`, and related resources derive extents using integer division by two. Consequently, a `maxRenderSize` component of `1` produces a derived texture extent of `0` in this open implementation before D3D12 validates the resulting resource descriptor. `maxUpscaleSize` is also used directly for full-resolution internal textures.

For the relevant maxima-derived default textures, this construction path does not first compute a `width × height × bytes` allocation count in ordinary 32-bit arithmetic; it constructs resource descriptions and delegates allocation sizing/resource creation to the DX12 backend. The principal creation-time numerical transitions found are therefore coordinate division, direct coordinate propagation, and the `uint32_t` to `int32_t` assignment noted above rather than an identified unchecked pixel-count multiplication.

The FSR3 dispatch path later validates render/upscale dimensions against maxima and contains further arithmetic, but those checks occur after context construction and cannot establish that an arbitrary pair of creation maxima is safe to pass into `ffxCreateContext`.

An implementation-derived envelope can nevertheless be stated for the **open FSR3 resource descriptions only**: `maxRenderSize` coordinates of at least `2` avoid the `/2 -> 0` derived-texture case, while direct Texture2D extents must remain within the D3D12 limit discussed below. This is not an SDK-wide or FSR4 contract.

## FSR2 differences

The bundled FSR2 provider follows the same broad pattern: its modern adapter copies `maxRenderSize` and `maxUpscaleSize` into the older FSR2 description (`maxRenderSize` and fixed `displaySize`) without a numerical range check before context creation.

FSR2 also stores creation dimensions in signed `int32_t` shader-constant fields, including `displaySize`, producing the same relevant `uint32_t`→`int32_t` transition for large output coordinates.

Its resource layout differs from FSR3, but it likewise creates render-sized resources directly and derives a `sceneLuminance` resource using `maxRenderSize / 2`. Therefore a render coordinate of `1` again becomes a zero texture coordinate in this open provider. Direct `displaySize` resources require a positive D3D12-valid extent.

The important semantic difference is that FSR2's display size is fixed by creation, whereas FSR3 has a maximum upscale size against which a later per-dispatch upscale size can be validated. That difference does not produce a transferable FSR4 construction contract.

The supplied signed-runtime success for `1×1 -> 2×2` therefore provides useful evidence of provider divergence: it cannot be reconciled by treating the open FSR2/FSR3 `/2` resource layout as an invariant of the signed FSR4 provider. It does not establish how FSR4 handles other small or extreme dimensions.

## DX12 backend and delegated limits

The generic DX12 backend does not add an upscaler-specific numerical validation layer. It accepts the effect-generated `FfxResourceDescription`, builds a D3D12 resource description, and invokes D3D12 resource/allocation APIs.

At that conversion boundary:

- FidelityFX width and height originate as `uint32_t`;
- Texture `Width` is assigned to D3D12's 64-bit `Width` member and therefore widens;
- `Height` remains a 32-bit `UINT`;
- depth/array and mip counts are converted to 16-bit D3D12 fields, but the maxima-derived upscaler textures audited here use ordinary depth/mip values rather than deriving those fields from huge creation maxima.

Microsoft's `D3D12_RESOURCE_DESC` contract requires texture dimensions to be at least 1 and no greater than the maximum supported for the resource dimension and feature level. For Texture2D at the relevant Direct3D feature levels, the documented maximum U/V dimension is **16384**. This gives a genuine D3D12 resource-coordinate bound of `1..=16384` for any Texture2D resource description that actually reaches D3D12. It is **not** an AMD upscaler-creation bound, because a provider may transform creation maxima before resource creation or may impose additional constraints.

D3D12 also distinguishes descriptor validity from memory availability: a descriptor may satisfy coordinate limits while resource creation still fails, including with `E_OUTOFMEMORY`.

The pinned FidelityFX DX12 backend's error handling is important to the safety-contract question. Its `TIF` helper throws on a failed HRESULT, and `CreatePlacedResource`/`CreateCommittedResource` are invoked through that helper. The generic modern `ffxCreateContext` source inspected here does not surround `provider->CreateContext` with a catch that would convert such a throw into an `ffxReturnCode_t`. Consequently, the open source path does **not** establish that every invalid or resource-exhausting extent is converted into an ordinary FidelityFX return code.

This is evidence about the pinned open backend path, not proof of the exception behavior of the signed FSR4 provider.

## FSR4 public evidence

AMD's FSR4 documentation for the v2.3.0-era SDK states that FSR4 integration uses the AMD FSR API and requires AMD's signed distributed implementation. It documents NativeAA and the ordinary scaling modes but does not publish the provider implementation needed to audit its internal arithmetic/resource construction.

The documentation gives approximate GPU-local memory figures at several operating conditions, including `7680×4320`, and exposes the V2 GPU-memory-usage query. These figures demonstrate support/measurement at those documented operating conditions; they are not described as minimum or maximum legal resolutions and therefore cannot be promoted into hard API bounds.

No pinned FSR4 public material located in this investigation establishes a stronger creation-time rule for:

- minimum dimensions;
- a maximum dimension;
- alignment;
- permitted aspect ratios;
- render/output ordering;
- arithmetic-overflow avoidance;
- ordinary returned-error behavior for arbitrary unsupported sizes.

The documented debug checker concerns application inputs in the operational/dispatch integration path rather than a complete pre-creation validation of proposed maxima.

The closed signed implementation therefore remains the material evidence gap.

## Public Query evaluation

Provider/version enumeration can establish that a provider exists and identify it, but it does not accept a proposed pair of creation maxima and is not documented as a numerical construction validator.

The V2 GPU-memory query is more relevant. In the open FSR3 and FSR2 implementations it constructs the provider's resource descriptions, sends them through the DX12 resource-sizing path, and sums reported allocation sizes. The DX12 sizing helper treats `GetResourceAllocationInfo` returning `UINT64_MAX` as invalid and returns an error; through the modern-provider adapter this can become `FFX_API_RETURN_ERROR_RUNTIME_ERROR`. Thus the query can expose some invalid D3D12 resource descriptions for those open providers.

However, AMD documents the FSR4 V2 query as reporting required GPU-local memory, not as a complete proof that arbitrary maxima are valid for construction or that every internal provider arithmetic path has been safely evaluated. It therefore classifies as **sizing/partial capability information**, not a complete validator.

The resource-requirements query reports resource requirement bitfields and does not validate an arbitrary numerical dimension tuple.

No evidence found contradicts the previous baseline that no unconditional Query/Configure operation is required merely to make the first Dispatch legal.

## Error-path classification

| Numerical/resource condition | Audited scope | Observed source path | Classification |
|---|---|---|---|
| malformed/null generic API parameters | generic modern API | returns `FFX_API_RETURN_ERROR_PARAMETER` where explicitly checked | **Verified** |
| open provider returns an `FfxErrorCode` through `TRY2` | FSR2/FSR3 adapter | translated to `FFX_API_RETURN_ERROR_RUNTIME_ERROR` | **Verified** |
| invalid resource description detected by pre-create memory sizing | open FSR2/FSR3 + DX12 sizing helper | `FFX_ERROR_INVALID_ARGUMENT`, then modern adapter maps failure to runtime error | **Verified** |
| failed `CreateCommittedResource` / `CreatePlacedResource` HRESULT | pinned open DX12 backend | `TIF` throws rather than returning an `FfxErrorCode` from that call site | **Verified** |
| resource exhaustion during ordinary open-provider creation | open DX12 backend | D3D12 can return failure such as `E_OUTOFMEMORY`; backend creation call is on the throwing path | **Verified** |
| `maxRenderSize == 1` in one coordinate | open FSR2/FSR3 | `/2` creates zero derived texture extent before D3D12 validation | **Verified**, provider-specific |
| `maxUpscaleSize > INT32_MAX` | open FSR3 | unsigned value assigned to signed shader constant during creation | **Verified** numerical transition; safety consequence **Unresolved** |
| FSR4 arbitrary extreme/unsupported dimension | signed closed provider | implementation unavailable | **Unresolved** |
| zero dimension on supplied signed provider 4.1.1 configuration | supplied runtime evidence | no native return; child process terminated | **Verified observation**, cause **Unresolved** |

The relevant distinction is that the API possesses ordinary error codes, but public source does not establish a universal theorem that every unsupported creation dimension reaches one before provider/backend arithmetic, descriptor construction, or a foreign exception path.

## Material findings

| Finding | Scope | Evidence status | Consequence for the unanswered question |
|---|---|---|---|
| creation dimensions are unsigned 32-bit values | generic ABI | **Verified** | representation alone gives no legal numerical domain |
| NativeAA represents equal render/output scaling | public upscaler contract | **Verified** | equality cannot be rejected as a general rule |
| no generic min/max/alignment/aspect/order rule was found | public upscaler contract | **Verified** absence in inspected pinned contract | no SDK-wide bounded domain follows |
| FSR3 derives multiple creation resources using `maxRenderSize / 2` | open FSR3 | **Verified** | its resource path implicitly needs render coordinate ≥2 for non-zero half-size textures |
| FSR3 narrows `maxUpscaleSize` to `int32_t` constants | open FSR3 | **Verified** | values above `INT32_MAX` encounter a provider-local numerical transition before D3D12 |
| FSR2 has analogous `/2` resources and signed constants | open FSR2 | **Verified** | reinforces that implementations may rely on unstated numerical assumptions |
| Texture2D resource coordinates are constrained to 1..16384 | D3D12 | **Verified** | gives a backend resource bound, not an upscaler-construction contract |
| failed resource creation can take a C++ throw path | pinned open DX12 backend | **Verified** | open source does not prove universal ordinary-error rejection |
| pre-create memory query catches some invalid D3D12 descriptions | open FSR2/FSR3 | **Verified** | useful partial check, not complete construction validation |
| FSR4 docs contain 8K memory examples but no hard range | public FSR4 docs | **Verified** | 8K cannot be promoted to an API maximum |
| signed FSR4 implementation is unavailable for audit | target provider | **Verified** | its arithmetic, derived resources, and exception/error boundary remain unknown |
| a complete bounded safe domain follows from public v2.3.0 evidence | target question | **Unresolved / not established** | architectural trust decision remains necessary |

## Candidate safe domain

**No source-supported bounded domain identified.**

A useful but strictly provider-specific observation is that the open FSR2/FSR3 resource-description paths fit inside the basic D3D12 Texture2D coordinate contract if approximately:

```text
open FSR2/FSR3 implementation-derived envelope only:

2 <= maxRenderSize.width,height <= 16384
1 <= maxUpscaleSize.width,height <= 16384
```

The lower render bound follows from provider-specific `/2` resource derivations; the upper bound follows from D3D12 Texture2D descriptors; these values also remain within positive `int32_t`. This envelope is **not** a safe-wrapper domain for the target signed FSR4 provider.

It cannot be promoted for four independent reasons:

1. the signed FSR4 provider may derive different resources or perform different arithmetic;
2. D3D12's `16384` bound constrains actual Texture2D descriptors, not arbitrary upstream maxima before provider transformations;
3. even the open backend does not establish an ordinary-return guarantee for every resource-creation failure;
4. AMD does not document the FSR4 memory query or another public Query as a complete proposed-dimension validator.

The supplied success of signed FSR4 at `1×1 -> 2×2`, despite the open providers' half-resolution resource issue, is direct evidence that an open-provider-derived lower limit must not be projected onto FSR4.

Accordingly, neither `1..=16384`, `2..=16384`, `1..=INT32_MAX`, nor any convenient display-oriented limit such as 8K/16K is justified as the target provider's complete safe construction domain by the available public evidence.

## Remaining signed-provider assumption

**Evidence status: Unresolved. This investigation does not accept the assumption.**

The narrow remaining assumption for later architectural review can be stated as:

> For structurally valid FidelityFX SDK v2.3.0 upscaler creation descriptors, the required API-version descriptor, `flags = 0`, null callback/default host allocator, a valid supported DX12 device, and a creation-dimension tuple belonging to whatever numerical domain is separately approved by the project, the trusted signed AMD FSR4/external provider evaluates every creation-time use of those dimensions—including conversions, arithmetic, derived resource descriptions, capability checks and allocations—such that context creation either succeeds or rejects the unsupported condition without violating caller memory/FFI safety and without an unhandled foreign exception crossing the API boundary.

This formulation does not assume that every non-zero `u32` belongs to that domain, nor that D3D12's Texture2D maximum is sufficient, nor that the provider will return a particular error code.

The missing closed-provider fact is precise: **public evidence does not reveal or contract the complete mapping from the four creation coordinates to FSR4's internal arithmetic/resources together with a guarantee that every unsupported numerical/resource condition is rejected through an ABI-safe failure path.**

## Experiment follow-up, if justified

One concrete source-derived boundary is worth a targeted signed-runtime diagnostic: the D3D12 Texture2D maximum coordinate of **16384**. This is not proposed as a proof of a safe domain.

Keeping the known structural inputs fixed and using equal render/upscale maxima to avoid introducing a render-vs-output ordering question, a narrow width-boundary probe would be:

```text
16383x720 -> 16383x720
16384x720 -> 16384x720
16385x720 -> 16385x720
```

The observable distinction should be recorded as:

- successful creation and destruction;
- an ordinary returned `ffxReturnCode_t`;
- or failure to return normally, including a foreign exception/process termination.

The specific hypothesis is whether the signed provider handles the first dimension just beyond the delegated D3D12 Texture2D limit through an ordinary ABI-safe rejection path and whether `16384` itself is usable on that configuration. Even a clean result would establish only behavior around this boundary on that tested signed provider/device/runtime; it would not prove all lower numerical values safe.

No further generic dimension sweep, random-resolution test, or broad “large values” spike is justified by the source. The `1`/`2` transition in open FSR2/FSR3 does not warrant another generic probe because the supplied signed-runtime evidence has already shown successful FSR4 creation at `1×1`, demonstrating implementation divergence.

## Baseline delta

| Supplied baseline point | Delta classification | Result |
|---|---|---|
| dimensions are unsigned 32-bit values with documented max-render/max-upscale meanings | **confirmed** | pinned headers agree |
| generic public ABI states no single min/max/alignment/aspect/render≤upscale rule | **confirmed** | no stronger generic rule found |
| NativeAA makes equal render/output dimensions intended | **confirmed** | public scaling-mode documentation agrees |
| FSR3 uses creation maxima for internal resources and later dispatch bounds | **confirmed** | source trace reproduces this |
| FSR2/FSR3 cannot prove signed FSR4 behavior | **confirmed and strengthened** | signed `1×1` success conflicts with an open-provider-derived `>=2` resource envelope |
| signed/external FSR4 implementation is unavailable | **confirmed** | public docs require signed implementation but do not expose its internals |
| provider enumeration is not a complete arbitrary-input validator | **confirmed** | enumeration does not consume proposed maxima |
| pre-create GPU-memory query has no relevance to validation | **refined** | it can detect some invalid resource descriptions in open providers, but is not documented as a complete construction validator |
| open providers have no relevant numerical transitions before D3D12 | **refined** | `/2` resource sizing and unsigned-to-signed constant assignments occur before/while constructing resources |
| ordinary resource failure is necessarily returned as a FidelityFX error | **not supported; refined** | pinned open DX12 creation uses a throwing HRESULT helper |
| any supplied baseline claim | **contradicted** | none |
| complete signed-FSR4 numerical construction domain | **still unresolved** | closed implementation and missing public contract prevent closure |

## Limits and follow-up

This source investigation can close the research question without closing the architectural decision: public v2.3.0 evidence does **not** support a complete conservative numerical domain for safe FSR4 construction.

The open FSR2/FSR3 implementations are useful because they demonstrate concrete kinds of hidden numerical assumptions—half-resolution derivations, signed conversions, and provider-specific resource layouts—but their exact bounds cannot be transferred to FSR4. D3D12 supplies a hard resource-coordinate bound, but not a proof about the transformations a closed provider performs before reaching D3D12.

The missing evidence could be resolved only by a stronger AMD contract, auditable FSR4 provider source, or a later explicit architectural decision to trust a precisely scoped behavior of the signed provider. Targeted runtime evidence can falsify particular assumptions or characterize a specific boundary, but cannot exhaustively derive the closed provider's safe numerical domain.

This record therefore neither accepts the remaining signed-provider assumption
nor accepts [proposed D008](../../../docs/adr/D008.md), authorizes M5
implementation, or makes a safe constructor public. That separation is required
by the research brief.

## Primary source ledger

All AMD repository sources below are pinned to commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date for all sources is **2026-09-23**.

| Source | Relevant symbol/section | Direct source |
|---|---|---|
| AMD `Kits/FidelityFX/api/include/ffx_api_types.h` | `FfxApiDimensions2D`, resource descriptions | [Pinned ffx_api_types.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h) |
| AMD `Kits/FidelityFX/api/include/ffx_api.h` | return codes, `ffxCreateContext`, version/provider queries | [Pinned ffx_api.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h) |
| AMD `Kits/FidelityFX/upscalers/include/ffx_upscale.h` | creation dimensions, quality modes, memory/resource queries | [Pinned ffx_upscale.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h) |
| AMD `Kits/FidelityFX/api/internal/ffx_api.cpp` | generic `ffxCreateContext` provider call | [Pinned ffx_api.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp) |
| AMD API helper | `VERIFY`, `TRY`, `TRY2` error propagation | [Pinned API helper source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api_helper.h) |
| AMD `ffx_provider_fsr3upscale.cpp` | modern FSR3 adapter and Queries | [Pinned FSR3 provider adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp) |
| AMD `ffx_fsr3upscaler.cpp` | FSR3 context creation, resources, memory query, Dispatch validation | [Pinned FSR3 implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp) |
| AMD `ffx_provider_fsr2.cpp` | modern FSR2 provider adapter | [Pinned FSR2 provider adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp) |
| AMD `ffx_fsr2.cpp` | FSR2 creation/resources/memory query | [Pinned FSR2 implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr2.cpp) |
| AMD `ffx_backends_dx12.cpp` | modern backend setup/resource-size delegation | [Pinned DX12 backend adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp) |
| AMD `ffx_dx12.cpp` | resource description conversion, allocation sizing, resource creation, HRESULT helper | [Pinned DX12 implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp) |
| AMD `super-resolution-ml.md` | FSR4 scaling modes, signed implementation, memory guidance/query | [Pinned FSR4 documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md) |
| Microsoft `D3D12_RESOURCE_DESC` | texture dimension validity and feature-level limit delegation | [Microsoft D3D12_RESOURCE_DESC documentation](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ns-d3d12-d3d12_resource_desc) |
| Microsoft D3D12 hardware feature levels | maximum Texture2D dimension | [Microsoft D3D12 hardware feature levels](https://learn.microsoft.com/en-us/windows/win32/direct3d12/hardware-feature-levels) |
| Microsoft `ID3D12Device::CreateCommittedResource` | HRESULT/resource allocation failure including `E_OUTOFMEMORY` | [Microsoft CreateCommittedResource documentation](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device-createcommittedresource) |
