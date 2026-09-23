# Source investigation: FidelityFX v2.3.0 context lifecycle and ownership

- **Investigation date:** 2026-09-20
- **Topic and scope:** Native `ffxContext` creation, ownership, destruction, allocation, provider selection, and concurrency contract for the FidelityFX SDK v2.3.0 Windows/DX12 upscaler API. Dispatch/resource synchronization is excluded except where context creation or destruction directly depends on it.
- **Baseline and evidence cut-off:** AMD FidelityFX SDK tag `v2.3.0`, release commit `60f4ea8`, corresponding to FSR SDK v2.3.0 and AMD FSR Upscaling 4.1.1. No current-`main` behavior is used as v2.3.0 evidence.
- **Method and exclusions:** Public v2.3.0 headers were treated as the ABI/contract baseline; tagged implementation was used to determine what AMD's open providers actually do; tagged documentation and release notes were used where behavior is not encoded in headers. No Windows/DX12 runtime experiment was performed as part of this investigation. Therefore source-level implementation observations below do not establish that the distributed signed DLLs behave identically. “Verified” means directly established by tagged primary source; “Upstream claim” means AMD documents behavior not independently reproduced here; “Inference” is derived from source but not an explicit contract; “Unresolved” indicates insufficient or conflicting evidence.

## Inputs

Project baseline supplied for this investigation:

- raw C ABI/runtime-loading layer already exists;
- `amd_fidelityfx_loader_dx12.dll` is explicitly loaded and kept live while calls remain possible;
- `ffxCreateContext`, `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, and `ffxDispatch` are resolved;
- a real context-free `ffxQueryDescGetVersions` call succeeded with the v2.3.0 loader and `amd_fidelityfx_upscaler_dx12.dll`;
- no context has yet been created or destroyed.

Primary v2.3.0 sources inspected, all accessed 2026-09-20:

1. **Release/tag metadata**, tag `v2.3.0`, commit `60f4ea8`.  
   URL:

2. **Public common C API header:** `Kits/FidelityFX/api/include/ffx_api.h`; relevant symbols include `ffxContext`, `ffxApiHeader`, `ffxAllocationCallbacks`, `ffxCreateContext`, `ffxDestroyContext`, `ffxOverrideVersion`, `ffxReturnCode_t`.  
   URL:

3. **Public C++ API helper:** `Kits/FidelityFX/api/include/ffx_api.hpp`; relevant helpers include `LinkHeaders`, `CreateContext`, and `DestroyContext`.  
   URL: [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api.hpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api.hpp)

4. **Public DX12 API header:** `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h`; relevant types include `ffxCreateBackendDX12Desc` and `ffxCreateBackendDX12AllocationCallbacksDesc`.  
   URL:

5. **Public upscaler API header:** `Kits/FidelityFX/upscalers/include/ffx_upscale.h`; relevant types include `ffxCreateContextDescUpscale` and `ffxCreateContextDescUpscaleVersion`.  
   URL:

6. **Generic FidelityFX API documentation:** `Kits/FidelityFX/docs/getting-started/ffx-api.md`.  
   URL:

7. **FSR Upscaling documentation:** `Kits/FidelityFX/docs/techniques/super-resolution-ml.md`.  
   URL: [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md)

8. **SDK 2.1 API-change note**, retained in the v2.3.0 tree: `Kits/FidelityFX/docs/whats-new/version_2_1_0.md`.  
   URL:

9. **Core API implementation:** `Kits/FidelityFX/api/internal/ffx_api.cpp`; relevant implementations are `ffxCreateContext` and `ffxDestroyContext`.  
   URL:

10. **Provider infrastructure:** `Kits/FidelityFX/api/internal/ffx_provider.h`; relevant elements are `InternalContextHeader`, `GetAssociatedProvider`, and provider-selection logic.  
    URL:

11. **Backend infrastructure:** `Kits/FidelityFX/api/internal/ffx_backends.h`; relevant functions include `CreateBackend`, `MustCreateBackend`, and `GetDevice`.  
    URL:

12. **Open FSR3 Upscale provider:** `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp`.  
    URL:

13. **Open FSR2 provider:** `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp`.  
    URL:

14. **Adjacent Frame Generation API documentation**, inspected only for comparison of explicit thread-safety wording.  
    URL:

## Findings

### 1. Minimal valid DX12 upscaler create chain

**Verified — descriptor identities and tags.** The v2.3.0 upscaler root descriptor is:

- `ffxCreateContextDescUpscale`
- `header.type = FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE`
- macro expansion: `FFX_API_MAKE_EFFECT_SUB_ID(FFX_API_EFFECT_ID_UPSCALE, 0x00)`
- numeric tag: `0x00010000`
- payload: `flags`, `maxRenderSize`, `maxUpscaleSize`, and optional `ffxApiMessage fpMessage`.

**Verified — version descriptor identity.** The effect-specific version node is:

- `ffxCreateContextDescUpscaleVersion`
- `header.type = FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION`
- macro expansion sub-ID `0x0b`
- numeric tag: `0x0001000B`
- `version` must be `FFX_UPSCALER_VERSION`; in v2.3.0 that means version 4.1.1, encoded by the public macro as `0x01001001`.

**Upstream claim — the version descriptor is mandatory.** The v2.3.0 FSR Upscaling documentation explicitly says that since SDK 2.1 an `ffxCreateContextDescUpscaleVersion` must be created, filled with `FFX_UPSCALER_VERSION`, and linked into the creation chain. The v2.1 change note independently says it “must” be linked when creating an upscaler context.

**Verified — DX12 backend descriptor identity.** The DX12 backend node is:

- `ffxCreateBackendDX12Desc`
- `header.type = FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12`
- macro expansion sub-ID `0x02` under backend ID DX12 `0`
- numeric tag: `0x00000002`
- payload: `ID3D12Device* device`.

**Verified for the open FSR2/FSR3 providers; upstream claim for the generic API — a backend descriptor is required for normal DX12 creation.** AMD's generic create example includes `ffxCreateBackendDX12Desc`, while the open FSR2/FSR3 provider implementations call `MustCreateBackend`, which fails creation if no backend descriptor was found.

The therefore source-supported minimum node set for an ordinary v2.3.0 DX12 upscaler creation is:

```text
ffxCreateContextDescUpscale
    -> ffxCreateContextDescUpscaleVersion
    -> ffxCreateBackendDX12Desc
    -> NULL
```

**Inference — relative extension ordering.** The root must be the upscaler descriptor because provider selection begins from `desc->type`. AMD specifically describes the version descriptor as linked through the upscaler descriptor's `header.pNext`; generic chain machinery then walks linked headers. No examined source states a general contractual ordering between the version and backend extension nodes once the upscaler descriptor is first. The sequence above follows the effect-specific documentation literally rather than asserting that all permutations are supported.

**Verified — `ffxOverrideVersion` is different and optional.** `ffxOverrideVersion`, tag `FFX_API_DESC_TYPE_OVERRIDE_VERSION == 5`, selects a provider/version ID obtained from `ffxQueryDescGetVersions`; it is not a substitute for `ffxCreateContextDescUpscaleVersion`. The latter states the upscaler API compatibility version, whereas the former optionally selects a provider implementation.

**Verified — backend allocation callbacks are optional extensions.** `ffxCreateBackendDX12AllocationCallbacksDesc`, tag `0x00000003`, supplies optional resource, heap, and constant-buffer allocation callback function pointers. Individual callbacks may be `NULL`; it is not part of the minimum chain.

**Conflict in AMD's v2.3.0 documentation.** The generic `ffx-api.md` create example shows only an upscaler descriptor plus DX12 backend descriptor and omits `ffxCreateContextDescUpscaleVersion`. That example is inconsistent with the effect-specific v2.3.0 documentation and the SDK 2.1 API-change note. For an upscaler context, the effect-specific mandatory-version statement is the more specific evidence.

### 2. Ownership and lifetime of pointers reachable from creation

**Verified — public lifetime rule.** `ffxCreateContext`'s public C declaration states that pointers passed in `desc` must remain live until `ffxDestroyContext` is called on the resulting context. This is the strongest explicit v2.3.0 lifetime statement found.

This has several concrete consequences, but the scope of that sentence has one ambiguity discussed below.

| Pointer/data | v2.3.0 evidence | Classification |
|---|---|---|
| `ffxCreateBackendDX12Desc::device` | Pointer-valued field inside the create chain; covered by the public lifetime sentence. No public statement was found that the SDK takes COM ownership or performs `AddRef`. | **Verified lifetime contract; unresolved COM ownership mechanism.** |
| `ffxCreateContextDescUpscale::fpMessage` | May be `NULL`. The open FSR2 and FSR3 providers copy the function pointer into their internal context for later use. | **Verified for open providers.** |
| DX12 backend allocation callback function pointers | Stored in an optional create-chain descriptor and potentially needed to manage persistent backend allocations. | **Verified descriptor contract; exact FSR4 retention unresolved.** |
| `pNext` values and the storage containing chained descriptors | `pNext` is itself a pointer field, so a strict reading of the C-header lifetime sentence includes its target; however AMD's version-override example says that override descriptor storage need only survive through `CreateContext`. | **Unresolved due to ambiguous/conflicting primary evidence.** |
| Scalar values in the upscaler descriptor | Open FSR2/FSR3 providers copy required scalar dimensions/flags into provider initialization data. | **Verified for open providers; not a global provider guarantee.** |
| `ffxOverrideVersion::versionId` | Core code scans the chain synchronously during provider selection before provider `CreateContext`. | **Verified source behavior.** |
| Host `ffxAllocationCallbacks* memCb` | Separate argument, not a member of the descriptor chain. AMD explicitly documents that its `pUserData` is passed through, not dereferenced or stored by the API. | **Verified exception to descriptor-lifetime discussion.** |

The open FSR3 provider's internal context contains, among other state, an `FfxInterface`, effect context/resources, and `ffxApiMessage fpMessage`; creation copies dimensions/flags and stores `fpMessage`. The FSR2 provider follows the same relevant pattern. Neither implementation retains the original root descriptor address.

**Inference — “copied during create” cannot be generalized.** The open providers demonstrate that several values are copied, but the public API does not promise that all providers copy all descriptor-referenced state. In particular, v2.3.0 may select an opaque external/driver provider, and the current FSR4 provider is distributed through signed binaries rather than being fully characterized by the inspected open legacy provider source. Therefore a call-lifetime-only assumption for all create-chain pointees is not established.

**Unresolved — precise meaning of the `pNext` lifetime.** The public C comment says pointers passed in `desc` remain live until destroy, and `ffxApiHeader::pNext` is literally such a pointer. Conversely, AMD's provider-version example says the `ffxOverrideVersion` object's lifetime must last only until after `CreateContext`, and core implementation consumes that node synchronously.
The sources therefore do not establish a clean general rule that all extension-node storage may be released immediately after creation, nor do they convincingly establish that every `pNext` node must remain allocated until destruction. This ambiguity is material to any safe ownership layer.

**Verified — message callback value is retained by known providers.** For both open FSR2 and FSR3 upscaling, `desc->fpMessage` is assigned into persistent internal-context state. If non-null, the pointed-to code must consequently remain callable for as long as the provider may invoke it. There is no message-callback userdata pointer in this descriptor.

**Unresolved — device COM reference ownership.** No inspected public v2.3.0 contract says that `ffxCreateContext` or the DX12 backend calls `ID3D12Device::AddRef`, transfers ownership, or permits the application to release its owning device reference immediately after creation. The public pointer-lifetime rule is sufficient to require a live device, but not sufficient to characterize COM reference-count ownership.

### 3. Precise `ffxContext` lifecycle

`ffxContext` is publicly `typedef void* ffxContext`.

**Upstream claim — value before create.** AMD's generic documentation says the context variable should be initialized to `NULL` before calling `ffxCreateContext`.

**Verified source behavior — valid non-null arguments are overwritten.** `ffxCreateContext` first rejects `desc == nullptr`, then rejects `context == nullptr`; after both checks it executes `*context = nullptr` before provider selection. Thus, for a valid descriptor pointer and output-pointer address, the prior handle value is not consulted. If `desc` itself is null, however, the function returns before touching the output value.

**Verified — successful creation.** The open FSR2/FSR3 providers assign their allocated internal context to `*context` only at the end of successful creation and then return `FFX_API_RETURN_OK`. Their internal context begins with `InternalContextHeader`, allowing the top-level API to associate the opaque handle with its provider.

**Verified — some failed-create states are deterministic.**

- `desc == NULL`: `FFX_API_RETURN_ERROR_PARAMETER`; output storage is not touched.
- `context == NULL`: `FFX_API_RETURN_ERROR_PARAMETER`.
- valid pointers followed by provider-selection failure: output was already set to `NULL`, and the top-level returns `FFX_API_RETURN_NO_PROVIDER`.

**Unresolved — provider-level creation failure does not have a universal postcondition.** Once a provider is selected, the top-level function forwards `context` to `provider->CreateContext`. If that returns an error, the top-level code performs provider bookkeeping but does not write `*context = nullptr` again. The open FSR2/FSR3 implementations happen to assign the handle only on final success, so their ordinary earlier error paths leave the top-level-initialized null value intact. That is implementation evidence for those providers, not a documented provider-independent guarantee.

**Upstream claim contradicted by tagged source — state after successful destroy.** The generic v2.3.0 documentation says the context will be `NULL` after `ffxDestroyContext`.
The tagged top-level implementation does not clear `*context`. The open FSR3 upscaler provider destroys/deallocates its internal context and returns success without clearing the caller's handle; FSR2 does the same. The C++ `DestroyContext` helper merely forwards to the C API and also performs no nulling. Therefore, for these tagged open providers, a successful destroy leaves the caller's raw `ffxContext` value containing a dangling pointer.

That contradiction is source-level evidence; whether the shipped v2.3.0 `amd_fidelityfx_upscaler_dx12.dll` path selected on a particular machine exhibits the same behavior still requires runtime verification.

**Verified source behavior — null-handle destruction is not an idempotent no-op.** The top-level `ffxDestroyContext` checks whether the pointer-to-handle argument is null, but does not check whether `*context` is null. It immediately passes `*context` to `GetAssociatedProvider`; that helper casts the handle to `InternalContextHeader*` and dereferences it. Consequently:

- `ffxDestroyContext(NULL, ...)` has a defined top-level `FFX_API_RETURN_ERROR_PARAMETER` path.
- `ffxContext c = NULL; ffxDestroyContext(&c, ...)` reaches a null dereference in the tagged implementation rather than a documented harmless no-op.
- a second destroy after the open FSR2/FSR3 successful destroy uses a dangling handle and therefore reaches freed memory.

No v2.3.0 source inspected specifies double-destroy as valid.

**Unresolved — destroy after partially failed create.** No inspected contract instructs the caller to invoke `ffxDestroyContext` after a nonzero creation result. Since provider-level failures have no universal handle postcondition, and destroy assumes a valid provider-associated internal handle, blindly destroying after every failed create is not source-supported. Cleanup of allocations made before a provider's `CreateContext` failure is therefore primarily a provider-side obligation; whether every FSR4 failure path satisfies it is not established by the available open source.

**Unresolved — state after failed destroy.** `ffxDestroyContext` returns the provider's destruction result and does not normalize the handle afterward. The public API defines no transactional guarantee that an error means “nothing was destroyed” or that retrying is valid. A failed destroy therefore has no source-established general post-state.

### 4. Allocator contract across create and destroy

**Verified — host allocator callbacks are optional.** `ffxCreateContext` accepts an optional `ffxAllocationCallbacks*`. With `NULL`, AMD specifies standard `malloc`/`free` behavior. Custom callbacks consist of `alloc`, `dealloc`, and `pUserData`.

**Verified — destruction requires compatible callbacks, not the same struct object.** `ffxDestroyContext` documents that its allocator must be compatible with the callbacks used for creation. The generic API documentation further defines compatibility operationally: every pointer allocated using the creation callbacks/userdata must be deallocatable using the callbacks/userdata supplied at destruction.

**Verified — `pUserData` is not retained by this generic allocation API.** AMD explicitly states that `pUserData` is passed unchanged to the callbacks and that the API does not dereference or store it. Therefore the original `ffxAllocationCallbacks` object itself is not documented as context-owned state. However, whatever state is needed to make destruction's allocator compatible must still exist or be reproducible when destruction occurs.

**Verified — callback edge cases.** The allocation callback may return null. The deallocation callback is documented as potentially being invoked with a null allocation.

**Important distinction.** `ffxAllocationCallbacks` is the host allocation mechanism passed separately to create/destroy. `ffxCreateBackendDX12AllocationCallbacksDesc` is a different, optional descriptor containing DX12 resource/heap/constant-buffer allocation functions and no userdata field. These two callback families must not be conflated.

**Upstream claim — persistent GPU allocations originate during context creation.** The v2.3.0 upscaler documentation states that local GPU memory for persistent intermediates is allocated when the upscaler context is created, through backend-interface callbacks. This makes lifetime of any supplied backend allocation/deallocation functions materially relevant to the entire context lifetime.

### 5. Loader, upscaler DLL, and provider/version requirements

**Upstream claim — DLL roles.** AMD documents `amd_fidelityfx_loader_dx12.dll` as the small application-facing loader containing no effect implementation; it manages loading effect-specific DLLs. `amd_fidelityfx_upscaler_dx12.dll` supplies upscaling providers, including the current FSR Upscaling implementation and legacy FSR3/FSR2 providers. Applications are directed to load the loader rather than the older monolithic library arrangement.

**Verified source behavior — provider selection is not necessarily fixed to one bundled implementation.** The tagged provider infrastructure can consider both internal providers and an external/driver-side provider associated with the DX12 device. Without an explicit override, it can compare provider version identifiers and select the newer supported provider; with an override it seeks the specified provider/version.

This materially limits what can be inferred from the open FSR2/FSR3 provider sources: a machine may execute a different provider path.

**Verified — optional provider override IDs must originate from enumeration.** `ffxOverrideVersion::versionId` must be a value returned from the version-query API; AMD warns against hard-coding those provider IDs. The override should also be applied consistently to context-free queries and subsequent creation if the caller intends both to address the same provider.

**Verified — mandatory upscaler API version and optional provider version are independent mechanisms.** `ffxCreateContextDescUpscaleVersion.version = FFX_UPSCALER_VERSION` declares the expected upscaler API generation. `ffxOverrideVersion.versionId` chooses one of the enumerated provider implementations. Both can therefore legitimately appear in a chain for different purposes.

**Inference — module lifetime must encompass live contexts.** AMD's API documentation recommends runtime DLL loading but does not give a precise unload contract for a live context. Tagged contexts contain provider associations and execute provider code during destruction, so unloading the corresponding implementation while a context remains live would invalidate code/data required by that context. Retaining loader/provider modules until all associated contexts are destroyed is therefore a source-derived safety constraint, not an explicitly worded v2.3.0 ownership promise.

**Unresolved — operation without the bundled upscaler provider DLL.** Provider source permits external-provider participation, but the inspected sources do not establish a supported application contract that context creation can intentionally omit `amd_fidelityfx_upscaler_dx12.dll` merely because an external provider might exist. This requires runtime/provider-loader characterization and should not be inferred from `GetProvider` alone.

### 6. Thread safety and concurrency

**Unresolved — no explicit upscaler create/destroy concurrency guarantee was located.** Neither the inspected generic API material nor the v2.3.0 upscaler documentation establishes that:

- separate `ffxCreateContext`/`ffxDestroyContext` calls are safe concurrently;
- two operations on the same `ffxContext` may run concurrently;
- destruction may race another operation;
- provider enumeration/selection is globally thread-safe.

Absence of such wording is not evidence that the calls are unsafe; it means no usable guarantee was found.

The distinction is meaningful because another v2.3.0 effect documents synchronization explicitly: Frame Generation states that its underlying context is not guaranteed to be thread-safe and describes mutex protection around relevant context operations. That statement is **not** evidence about the upscaler, but it demonstrates that AMD specifies effect-level synchronization where required rather than providing an obvious SDK-wide blanket rule.

A runtime stress test could expose a race but cannot establish a contractual thread-safety guarantee that the v2.3.0 sources do not state.

### 7. `ffxCreateContext` / `ffxDestroyContext` failure codes and cleanup

The public ABI defines:

| Value | Name |
|---:|---|
| 0 | `FFX_API_RETURN_OK` |
| 1 | `FFX_API_RETURN_ERROR` |
| 2 | `FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE` |
| 3 | `FFX_API_RETURN_ERROR_RUNTIME_ERROR` |
| 4 | `FFX_API_RETURN_NO_PROVIDER` |
| 5 | `FFX_API_RETURN_ERROR_MEMORY` |
| 6 | `FFX_API_RETURN_ERROR_PARAMETER` |
| 7 | `FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE` |

The API documents zero as success and nonzero as error, while also allowing future return-code additions.

For **creation**, source-confirmed paths include:

- **`ERROR_PARAMETER` (6):** null `desc` or null output `context` pointer in the top-level API.
- **`NO_PROVIDER` (4):** provider selection yields no provider.
- **`ERROR_UNKNOWN_DESCTYPE` (2):** open FSR2/FSR3 providers reject an inappropriate root creation descriptor.
- **`ERROR_MEMORY` (5):** open provider creation contains explicit allocation-failure paths.
- **`ERROR` (1):** the open backend path can use the generic error when the required backend is not found through `MustCreateBackend`.

`ERROR_RUNTIME_ERROR` and `PROVIDER_NO_SUPPORT_NEW_DESCTYPE` exist in the ABI, but the inspected top-level creation path does not establish them as an exhaustive or mandatory outcome for a particular malformed upscaler create request. Provider/backend errors can be propagated, so the complete set remains provider-dependent.

For **destruction**, the only generic top-level error explicitly introduced before provider dispatch is `ERROR_PARAMETER` when the pointer-to-handle itself is null. Other destruction failures are provider/backend dependent. Because a null handle value is dereferenced during provider lookup before the provider's own validation, `&null_context` does not have a clean public error path in the tagged top-level implementation.

**Unresolved — no per-error caller cleanup matrix exists in the inspected public contract.** No source was found saying, for example, “destroy after `ERROR_MEMORY`”, “do not destroy after `NO_PROVIDER`”, or “retry destroy after `RUNTIME_ERROR`”. The only explicit cross-call cleanup requirement is allocator compatibility for a legitimately created context. Since failed creation has no universal non-null-handle contract, caller-side destruction after arbitrary create failure cannot be derived safely from the sources.

**Verified internal cleanup, but not a caller guarantee.** The top-level create implementation performs some provider-object cleanup when provider creation fails and provider reference accounting permits it. This concerns SDK-internal provider ownership; it does not establish complete cleanup of every partially created backend/effect object in opaque providers.

### 8. Other constraints material to an ownership-safe wrapper

**Verified — descriptor type correctness is a safety precondition.** AMD's generic API documentation states that the `type` field must correctly identify the actual descriptor and warns that violating this is undefined behavior and likely to crash. A higher-level ownership layer therefore cannot treat arbitrary tag/payload mismatches as merely recoverable native errors.

**Verified — successful native destroy cannot be assumed to poison/null the raw handle.** The tagged open upscaler providers contradict the generic documentation on this point. Any ownership reasoning that uses “native handle became null” as proof of destruction would be unsound for those source implementations.

**Verified — null and double destroy are not specified safe operations and are source-level hazardous.** `GetAssociatedProvider` assumes a live internal-context header. This constrains any ownership scheme to enforce exactly-once destruction independently of the native value left behind.

**Unresolved — create-failure ownership is provider-sensitive.** The top-level initializes a valid output slot to null but does not enforce null on provider failure. The open legacy providers behave favorably by publishing the handle only on final success; opaque/provider-specific implementations are not contractually required by the inspected material to do the same.

**Unresolved — descriptor-node lifetime needs clarification before treating a temporary `pNext` chain as universally safe.** Known implementation paths consume descriptor nodes during creation, but the public lifetime sentence can be read more strictly, and AMD's own version-override sample supplies conflicting call-lifetime guidance for one extension node.

**Unresolved — opaque/provider-selected behavior matters.** Source inspection of FSR2/FSR3 establishes useful lower-level behavior but cannot prove FSR4 or external-driver provider behavior. Provider selection itself can vary with the device and available provider versions.

**Verified — destruction is a fallible native operation.** `ffxDestroyContext` returns `ffxReturnCode_t`; neither its signature nor its implementation makes destruction infallible. Moreover, no generic postcondition is given for an error. This is a native-contract constraint independently of how a future language binding chooses to expose it.

## Baseline delta

**Confirmed.**

- The project's use of `amd_fidelityfx_loader_dx12.dll` as the application-facing DX12 API module matches AMD's v2.3.0 documentation.
- `amd_fidelityfx_upscaler_dx12.dll` is the relevant effect-provider library for upscaling.
- Retaining loaded code while native calls remain possible is consistent with the source-level provider/context relationship, although AMD does not state an explicit DLL-unload ownership rule.
- The already successful context-free version query proves that the tested loader/provider installation can enumerate providers; it does not establish any context-lifecycle behavior.

**Refined.**

- The minimum v2.3.0 DX12 upscaler create chain is not merely `ffxCreateContextDescUpscale + ffxCreateBackendDX12Desc`; effect-specific v2.3.0 evidence requires `ffxCreateContextDescUpscaleVersion` with `version = FFX_UPSCALER_VERSION`.
- `ffxCreateContextDescUpscaleVersion` and `ffxOverrideVersion` serve different purposes; the former is mandatory API compatibility metadata, the latter is optional provider selection.
- The public C contract gives descriptor-contained pointers lifetime extending to context destruction, but exact treatment of `pNext` descriptor storage itself is ambiguous.
- Host allocation callbacks need to be compatible across create/destroy; the identical callback-struct object need not be retained.
- Open FSR2/FSR3 implementations retain the upscaler message callback value.
- Generic documentation's claim that destroy nulls the handle is contradicted by tagged open implementation.
- A null handle and a double destroy are not source-supported harmless operations.
- Failed creation does not have a provider-independent, explicitly documented handle postcondition once execution enters the provider.
- Provider selection may involve an external/driver-side provider, so behavior cannot be modeled solely from the bundled open legacy-provider implementation.

**Contradicted.**

- No stated project-baseline fact is contradicted.
- AMD's own generic v2.3.0 create example is contradicted/refined by its effect-specific requirement because the example omits the mandatory upscaler-version descriptor.
- AMD's generic statement that the context becomes `NULL` after destruction is contradicted by the tagged top-level, FSR2, and FSR3 source paths.

**Left unresolved.**

- Whether the exact signed v2.3.0 FSR4 provider copies or retains each descriptor-referenced item.
- Whether chained descriptor-node storage itself, as opposed to pointer-valued payloads, is contractually required until destroy.
- Whether the DX12 backend takes an independent COM reference on `ID3D12Device`.
- Exact output-handle state after every FSR4/external-provider creation failure.
- Exact state after every destruction failure.
- Explicit upscaler create/destroy thread-safety guarantees.
- Supported loader/provider-DLL unload sequencing.
- Whether an external provider makes an intentionally absent bundled upscaler DLL a supported configuration.

## Limits and follow-up

The following questions require native Windows/DX12/runtime experiments; source inspection alone cannot settle them:

1. Create an actual v2.3.0 DX12 upscaler context with the three-node root/version/backend chain and record the selected provider/version using the applicable provider query.
2. Record raw handle values immediately before and after successful destruction against the distributed signed DLLs; this determines whether the observed runtime reproduces the tagged open providers' non-nulling behavior.
3. Exercise controlled create failures—missing version descriptor, missing backend, null/invalid device where safely testable, injected host-allocation failure—and record both return code and output-handle value. Undefined-behavior cases such as deliberately mismatched descriptor tags should only be characterized in an isolated process, if at all.
4. Instrument custom `ffxAllocationCallbacks` to determine allocation/deallocation pairing over successful creation, destruction, and injected failures, including whether a merely compatible replacement callback set behaves as documented.
5. Instrument `ffxCreateBackendDX12AllocationCallbacksDesc` to identify which callbacks are retained or invoked during destruction by the actual FSR4 provider.
6. Characterize device ownership separately if necessary; source evidence does not establish `ID3D12Device::AddRef` behavior. Any experiment should distinguish “SDK retained a COM reference” from the weaker public requirement that the device remain alive.
7. Force provider-level creation failures after partial allocation, where practical, to determine actual cleanup and output-handle behavior. An allocation-failure injector is preferable to intentionally invalid pointer inputs.
8. Compare context creation with and without `ffxOverrideVersion` and record whether an external/driver provider is selected on test systems. Provider identity must accompany lifecycle observations because the implementation can change with provider selection.
9. Treat thread safety as unspecified unless AMD supplies a stronger v2.3.0 statement. Concurrency stress testing may reveal defects but cannot turn absence of a source contract into a positive thread-safety guarantee.
10. Do not use deliberate use-after-free of descriptor/device/callback state as evidence that shorter lifetimes are supported: surviving such a test would only show that one provider happened not to dereference the object in that run, while the public lifetime contract remains stricter.

The remaining ambiguities—especially `pNext` storage lifetime, provider-failure handle state, device COM ownership, and opaque FSR4/external-provider behavior—are relevant to ownership soundness. This record establishes evidence and native constraints only; it does not constitute acceptance of a Rust ownership architecture.
