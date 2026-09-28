# Source investigation: FidelityFX API runtime sharing and threading

> Intake note (2026-09-22): renamed from `2026-09-22-rename_me_3.md`.
> Findings and source links are preserved as supplied. This source-only intake
> adds no signed-runtime or concurrency verification.

- **Investigation date:** 2026-09-22
- **Topic:** sharing one FidelityFX API runtime across contexts; context/provider lifetime; same-context and cross-context CPU concurrency; global/null-context operations; DX12-relevant shared state.
- **Project:** `fsr-sdk-rs`
- **Authoritative native baseline:** AMD FidelityFX SDK v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- **Evidence cut-off:** pinned v2.3.0 sources are authoritative. Microsoft D3D12 documentation is used only to separate FidelityFX restrictions from underlying D3D12 rules. No newer FidelityFX behavior is used to establish a v2.3.0 guarantee.
- **Target considered:** Windows x64/MSVC, DirectX 12, official signed AMD runtime, modern FidelityFX API.

## Method and exclusions

The investigation inspected the pinned public API headers, API documentation, generic API/provider implementation, FSR3 upscaler provider, DX12 backend, and the v2.3.0 Frame Generation documentation where it contains an explicit threading contract. Statements below distinguish public contract from implementation observation and inference.

The following were deliberately not treated as evidence of thread safety: successful single-threaded samples or tests; Windows DLL reference counting; D3D12's general ability to use multiple CPU threads; absence of an obvious lock; ability to copy or move an opaque context handle; or absence of observed failures.

No Rust ownership type, trait decision, synchronization mechanism, or architecture is proposed here.

Evidence labels used below:

- **Verified** — directly established by the pinned public contract or source.
- **Upstream claim** — explicit AMD statement whose scope matters.
- **Inference** — conclusion supported by source structure but not stated as a public guarantee.
- **Unresolved** — the inspected v2.3.0 evidence does not establish the requested property.

## Exact pinned inputs and source URLs

All AMD sources below are pinned to `60f4ea81909200d8542eca14dccb2628b763a9a3` and were accessed 2026-09-22.

- `Kits/FidelityFX/api/include/ffx_api.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/include/ffx_api_types.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/upscalers/include/ffx_upscale.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/docs/getting-started/ffx-api.md` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/include/ffx_api_loader.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_loader.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/internal/ffx_api.cpp` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/internal/ffx_provider.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/api/internal/ffx_message.cpp` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_message.cpp?utm_source=chatgpt.com)
- `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp?utm_source=chatgpt.com)
- `Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp?utm_source=chatgpt.com)
- `Kits/FidelityFX/backend/dx12/ffx_dx12.h` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.h?utm_source=chatgpt.com)
- `Kits/FidelityFX/backend/dx12/ffx_dx12.cpp` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp?utm_source=chatgpt.com)
- `Kits/FidelityFX/docs/techniques/frame-interpolation-api.md` — [pinned source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md?utm_source=chatgpt.com)

Microsoft supporting evidence, accessed 2026-09-22:

- Direct3D 12, “Design Philosophy of Command Queues and Command Lists” — [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/direct3d12/design-philosophy-of-command-queues-and-command-lists?utm_source=chatgpt.com)
- Direct3D 12, “Creating and recording command lists and bundles” — [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/direct3d12/recording-command-lists-and-bundles?utm_source=chatgpt.com)

## Multiple-context support

### One loaded API runtime serving multiple contexts

**Verified.** The v2.3.0 public architecture is a single five-function API surface exported from the signed DLL. For SDK 2.x, `amd_fidelityfx_loader_dx12.dll` contains no effect implementation itself; AMD describes it as managing loading of the separate effect-type DLLs and providing those effects to the application. Applications are instructed to load the loader DLL rather than individual API front ends.

Nothing in the public `ffxCreateContext` contract defines a one-context-per-loader or one-context-per-export-table limit. The lower-level DX12 backend interface explicitly sizes backend scratch storage by the “maximum number of simultaneous effect contexts that will share the backend,” demonstrating that coexistence of multiple effect contexts is a supported FidelityFX design concept.

**Upstream claim.** The Frame Generation integration documentation itself maintains both `FrameGenContext` and `SwapChainContext` and discusses API operations involving the two concurrently existing contexts. This is direct public evidence that the modern API is not a single-context API.

This establishes multiple-context coexistence through the API runtime. It does **not** establish arbitrary concurrent CPU execution on those contexts.

### Multiple contexts of the same upscaler effect

**Implementation evidence only.** No explicit v2.3.0 public statement was found saying that an application may create an arbitrary number of simultaneous upscaler contexts.

The pinned FSR3 upscaler provider nevertheless implements `CreateContext` by allocating a new `InternalFsr3UpscalerUContext`, storing the provider pointer in that context, constructing a backend interface for it, and constructing the underlying FSR3 upscaler context. Its destroy path tears down the resources and backend scratch associated with that particular internal context.

The provider additionally states that its so-called shared resources are not shared between upscaler-provider contexts because providers are intended to remain independent.

**Inference.** The implementation is structurally compatible with more than one FSR3-upscale context existing simultaneously. This is stronger than the existing one-context project experiment, but it is not an explicit public multiplicity guarantee and says nothing about concurrent calls.

### Different effects using the same API runtime

**Verified.** The loader is specifically documented as one front end which loads separate frame-generation, upscaler, denoiser and radiance-cache effect DLLs. The modern API defines distinct effect IDs while retaining the same five entry points.

The documented Frame Generation integration additionally demonstrates simultaneous distinct context types under that API model.

### Provider/version selection

**Verified.** Version override is applied when a context is created; AMD describes the API as supporting an overridden version “of each effect on context creation.” Null-context queries are required to use the same override when querying the corresponding version.

**Implementation observation.** `ffxCreateContext` performs provider selection for that creation call. Each created context starts with an `InternalContextHeader` containing an `ffxProvider*`. Subsequent context-bound `Query`, `Configure`, `Dispatch`, and destruction recover the provider from that context rather than consulting a single “currently selected provider” field.

**Inference.** Provider selection is context-associated in the public implementation, rather than one mutable global provider selection applying to all existing contexts.

**Unresolved.** The public contract does not explicitly promise simultaneous contexts using different versions/providers of the same effect, nor does it state whether operations on such contexts may execute concurrently.

## Runtime/provider lifetime

### Loader/runtime residency

**Verified.** `ffxCreateContext` requires pointers supplied through the creation descriptor to remain live until `ffxDestroyContext`. This is an explicit lifetime requirement independent of DLL lifetime.

**Unresolved.** No inspected v2.3.0 public contract states the complete higher-level residency rule for `amd_fidelityfx_loader_dx12.dll` or effect-type DLLs while contexts exist. In particular, no public statement was found that characterizes the loader as a process-global singleton, a separately instantiable runtime object, or a reference-counted higher-level API object.

The documented model is simply that the application loads the signed loader DLL and invokes its five exports, while that DLL “manages the loading” of effect DLLs. The loader's actual module-load/unload synchronization and residency algorithm was not located in the public pinned source.

**Inference.** Keeping the API code and selected provider code available for every dependent live context is consistent with the public implementation: a context contains an actual provider pointer and later calls dispatch through that provider. There is no public mechanism for detaching a live context from its provider before destruction. This is an implementation-based lifetime inference, not a separately documented loader-lifetime contract.

### Provider ownership visible in source

**Implementation evidence only.** The generic provider base contains a `RefCount` field and every API context begins with a provider pointer. If an external provider is selected during creation, `ffxCreateContext` moves that provider to heap storage; destruction obtains the context's associated provider, delegates destruction to it, and destroys/deallocates the provider object when its reported reference count reaches zero.

This is not enough to conclude that all effect DLLs themselves are reference-counted by context. The DX12 external-provider definition is included from `amdinternal/.../ffx_provider_external.h`, whose implementation is not exposed by the inspected public source, and the loader's effect-module ownership is likewise not established here.

The provider version-enumeration code also contains an important DLL-lifetime observation: an external provider's version-name pointer is copied because it points into a DLL “which may not outlive the caller.” The copy is placed in static storage. This demonstrates that at least some provider-discovery data can originate in transient module-owned storage; it does not establish effect-DLL unloading behavior for live contexts.

### Destruction of one context versus other contexts

**Implementation evidence only.** The FSR3 upscaler provider's destroy path operates on the supplied context's internal resources, its underlying FSR3 context and its backend scratch allocation.

**Unresolved.** No public contract was found guaranteeing that destroying one context cannot alter shared provider/module state used by another context. Nor was public loader source found establishing whether destruction can trigger effect DLL unloading, provider registry changes, or external-provider module release. Therefore the stronger lifetime property required for arbitrary runtime sharing remains unspecified.

## Same-context concurrency

The generic API header defines `Query`, `Configure`, `Dispatch` and destruction but gives no generic “thread-safe,” “not thread-safe,” or external-synchronization rule for these operations. It states only that null-context Query/Configure target global state.

The inspected upscaler public header did not supply an effect-specific CPU-threading contract comparable to Frame Generation.

The FSR3 provider implementation does contain mutable per-context state: an `FfxInterface`, underlying FSR3 context, resources, message callback/debug fields and backend state. `Configure`, context-dependent `Query`, `Dispatch`, and destruction access portions of that state. This observation does not establish whether overlapping accesses are supported or forbidden.

Therefore, for the upscaler/general API:

- **Unresolved — Query vs Query on one context.**
- **Unresolved — Query vs Configure on one context.**
- **Unresolved — Query/Configure vs Dispatch on one context.**
- **Unresolved — Dispatch vs Dispatch on one context.**
- **Unresolved — Destroy overlapping any operation on the same context.**

### Narrow effect-specific exception: Frame Generation

**Upstream claim / Requires external synchronization.** The v2.3.0 Frame Generation documentation explicitly states that the underlying context “is not guaranteed to be thread safe” and that several public operations on `FrameGenContext` and `SwapChainContext` are not thread-safe and must be externally synchronized. AMD specifically identifies Frame Generation creation, destruction, PrepareV2 dispatch, conditional Frame Generation dispatch, swapchain `present`, and swapchain destruction among the guarded operations.

This is significant evidence that FidelityFX does publish effect-specific synchronization requirements when required. It must not be generalized into a rule for FSR upscaling or all five generic API entry points.

## Cross-context concurrency

### Independent contexts

**Unresolved.** No v2.3.0 public statement inspected here guarantees that API calls on two independent contexts may execute simultaneously on separate CPU threads.

The source provides useful but insufficient implementation evidence:

1. Each context carries its own provider association.
2. The FSR3 upscaler provider allocates a separate internal context, backend interface, resources and backend scratch allocation for each modern API upscaler context.
3. The DX12 backend itself has per-backend context storage, including resources, GPU jobs and effect-context arrays.
4. Some backend state is nevertheless process/module-global, including registered DX12 allocation callbacks.
5. API-wide diagnostic callback/debug state is also static global state.

Consequently, independent per-context allocation does not imply independent execution.

The following cases remain separately **Unresolved**:

- same effect, same internal provider;
- same effect, different version/provider;
- different effects;
- contexts using the same `ID3D12Device`;
- contexts using different `ID3D12Device` instances.

A different device does not eliminate loader/provider/global-state interactions, so it does not by itself resolve FidelityFX CPU concurrency.

### Create/Create and other lifecycle overlap

**Unresolved.** Each `ffxCreateContext` invocation performs provider discovery/selection. On DX12 it may construct an external/driver-provider candidate; if selected, that provider is moved to heap storage.

`ffxDestroyContext` recovers the context's provider, invokes its destroy operation, and may destroy the provider object based on its reference count.

No generic synchronization guarantee was found for:

- two simultaneous `ffxCreateContext` calls;
- two simultaneous destroys of different contexts;
- creation while another context is dispatching;
- destruction of one context while another context is queried/configured/dispatched;
- creation or destruction concurrent with provider discovery/version enumeration.

The public loader/module implementation available in the inspected source is insufficient to determine whether effect DLL discovery/loading/unloading introduces additional shared synchronization requirements.

The Frame Generation documentation's guarded create/destroy operations remain a narrower effect-specific exception.

## Global/null-context operations

### Contract

**Verified.** `ffx_api.h` explicitly states:

- null-context `ffxConfigure` “operates on any global state”;
- null-context `ffxQuery` “operates on any global state.”

Only particular query descriptors support null context, as described by the API documentation.

### Global Configure state

**Implementation evidence only.** The generic API recognizes global-debug Configure descriptors without requiring a context and routes them to `ffxSetPrintMessageCallback`. The implementation stores the callback and debug level in module-static variables `s_messageCallback` and `s_debugLevel`, and message output reads those variables.

No synchronization or atomicity contract accompanies these fields in the inspected source.

Therefore:

- concurrent global Configure calls: **Unresolved**;
- global Configure concurrent with operations capable of emitting diagnostic messages: **Unresolved**;
- whether callers must serialize these operations: **Unresolved** at the generic public-contract level.

The existence of unsynchronized-looking statics is an implementation observation, not by itself proof that a particular overlap is contractually forbidden.

### Null-context Query and provider discovery

**Implementation evidence only.** A null-context version query invokes provider-count/version discovery. Other null-context queries perform provider selection and call the selected provider with a null context.

For DX12, provider discovery constructs an external/driver-provider candidate where applicable.

Version enumeration has definite shared storage: the provider implementation uses a static `extProviderName[64]` buffer for an external provider's name, and the public documentation separately warns that some version names reside in global memory and “may be overwritten by later version queries,” recommending that callers copy them.

**Verified.** Returned version-name pointer contents therefore do not have indefinite stability across subsequent version queries.

**Unresolved.** That overwrite warning does not state whether two calls themselves may execute simultaneously. It therefore does not establish either support or prohibition for concurrent null-context Query calls.

**Unresolved.** Null-context Query versus context Create/Destroy is likewise not covered by an explicit synchronization rule even though both paths can involve provider discovery/selection.

## Provider, callback and backend-global state

The following state materially narrows the concurrency question.

| State | Evidence | Status |
|---|---|---|
| Context → selected provider pointer | Every internal context starts with `InternalContextHeader::provider`; context-bound API calls recover it. | **Verified implementation observation** |
| Global API message callback/debug level | Module-static callback and level are written by global Configure and read during diagnostics. | **Verified implementation observation** |
| Version-query name storage | AMD warns some names are global and overwritten by later queries; external-provider enumeration uses static name storage. | **Verified** |
| DX12 allocation callbacks | `CreateBackend` can register resource, heap and constant-buffer callbacks; corresponding callback pointers are module-static in `ffx_dx12.cpp`. | **Verified implementation observation** |
| DX12 backend context | Device, resource tables, effect contexts, job count, descriptor state etc. reside in `BackendContext_DX12`. | **Verified implementation observation** |
| Backend constant-buffer allocator | The fallback allocator has a per-backend `std::mutex` protecting its ring-buffer mutation. | **Verified implementation observation** |
| FSR3 upscaler debug configuration | Upgrader creation and `GLOBALDEBUG1` Configure call `ffxFsr3UpscalerSetGlobalDebugMessage`, while the provider also retains per-context callback/debug fields. | **Verified implementation observation; underlying global-state implementation not established here** |
| Loader/effect module registry | Loader documented as managing effect-DLL loading; its registry/unload synchronization was not established from the public pinned source. | **Unresolved** |
| External/driver-provider internals | Public provider header references an `amdinternal` DX12 external-provider header not exposed in the inspected public tree. | **Unresolved** |

The single mutex found in the DX12 fallback constant-buffer allocator protects that allocator operation only. It cannot be promoted into a general provider/backend/API thread-safety guarantee.

## DX12/COM interaction relevant to this question

The FidelityFX DX12 creation descriptor carries an `ID3D12Device*`, and the DX12 backend stores the corresponding device in backend-local state. Modern FSR3 upscaler creation requests its own backend interface with capacity `1`; this differs from the lower-level backend API's ability to allocate one backend for several simultaneous effect contexts.

Microsoft's D3D12 contract permits important forms of CPU parallelism: multiple command lists can be recorded concurrently, and any thread may submit command lists to a command queue, while a single command list itself is not free-threaded.

**Inference.** D3D12 therefore does not supply a blanket “one CPU thread per device” restriction that would by itself prohibit two FidelityFX contexts from referencing the same device.

**Unresolved.** Conversely, D3D12's multithreading model does not establish FidelityFX context thread safety. No generic AMD v2.3.0 statement was found granting concurrent FidelityFX calls merely because the contexts use the same or different `ID3D12Device` objects.

## Concurrency evidence matrix

The matrix concerns the generic modern API/upscaler case unless a narrower effect-specific exception is stated.

| Operation relationship | Contract status | Primary evidence |
|---|---|---|
| Query vs Query, same context | **Unresolved** | Generic `ffxQuery` delegates to the context-associated provider; no generic synchronization statement is supplied. Some FSR3 queries access internal context state. |
| Query vs Configure, same context | **Unresolved** | Both delegate to the same context-associated provider; the FSR3 provider has mutable context fields, but no public overlap rule is stated. |
| Query/Configure vs Dispatch, same context | **Unresolved** | `Dispatch` and context-bound Query/Configure all use the associated provider/context. No generic thread-safety statement exists. |
| Dispatch vs Dispatch, same context | **Unresolved** | Generic API states only that dispatch operates on the supplied valid context. No generic overlap rule was found. Frame Generation is a narrower exception with explicit synchronization requirements. |
| Destroy vs any operation, same context | **Unresolved** | Destroy delegates through the context's provider and tears down context state; no generic concurrent-destroy rule is stated. Frame Generation destruction is explicitly synchronized in its integration contract. |
| Create vs Create, independent contexts | **Unresolved** | Each call independently performs provider discovery/selection, but no parallel-create guarantee or prohibition is published. |
| Operations on independent contexts | **Unresolved** | Contexts have independent associations/state, but global API/backend state also exists and there is no generic cross-context concurrency contract. |
| Global/null-context Query vs another global Query | **Unresolved** | Null version queries perform provider discovery; version-name storage may be overwritten by later queries and external-provider names use a static buffer. No simultaneous-call rule is stated. |
| Global/null-context Query vs Create/Destroy | **Unresolved** | Null Query and Create both enter provider discovery/selection paths; loader/provider synchronization is unspecified. |
| Contexts sharing one DX12 device | **Unresolved** | D3D12 supports multithreaded command-list patterns, and FidelityFX accepts an `ID3D12Device*`, but AMD supplies no generic cross-context CPU concurrency guarantee. |
| Frame Generation documented guarded operations versus conflicting FrameGen/SwapChain operations | **Requires external synchronization** | AMD explicitly says the underlying contexts are not guaranteed thread-safe and enumerates guarded Create/Destroy/Dispatch/Present operations. |
| Coexistence of multiple effect contexts | **Explicitly supported** | DX12 backend documentation explicitly describes “simultaneous effect contexts”; Frame Generation documentation maintains two context types concurrently. |
| Arbitrary multiple simultaneous FSR3-upscale contexts | **Implementation evidence only** | Provider `CreateContext` allocates a fresh internal context/backend/resources for each call, but no explicit multiplicity guarantee was found. |

No required matrix row can be classified as **Explicitly unsupported** for the generic upscaler API from the inspected evidence. Silence was not converted into a prohibition.

## Create/destroy and provider loading conclusions

**Verified.** Multiple contexts can coexist in the FidelityFX architecture; the API is not contractually limited to one live context.

**Implementation evidence only.** Modern FSR3 upscaler contexts possess separate internal context/backend/resource state, while generic API contexts retain their selected provider association.

**Unresolved.** Whether two creations can execute concurrently, whether two independent destroys can execute concurrently, or whether create/destroy can overlap calls on other contexts remains unspecified.

**Unresolved.** The public loader documentation does not expose enough of the loader's module registry/load/unload machinery to determine whether destroying one of several contexts can unload an effect/provider DLL or otherwise affect module-global lifetime. Public provider reference-count observations do not establish the loader's DLL reference policy.

**Unresolved.** No general provider-global thread-safety guarantee was located for same-provider or different-provider contexts.

## Baseline delta

Relative to the supplied baseline, which conservatively treats general native threading guarantees as unresolved and gives each M4 context exclusive runtime ownership, the pinned v2.3.0 evidence adds the following facts without making a Rust design decision:

1. **Multiple live contexts are an upstream-supported concept.** The lower-level DX12 API explicitly provisions for “simultaneous effect contexts,” and the modern Frame Generation integration uses two live modern API context types. One context per loaded API runtime is therefore not an upstream invariant.

2. **The loader is intended as a common front end for multiple effect DLLs.** `amd_fidelityfx_loader_dx12.dll` contains no effect code and manages loading separate effect-type DLLs behind the same five API exports.

3. **Provider selection is context-associated in the public implementation.** Creation selects a provider and the resulting context stores that provider pointer; subsequent context-bound calls recover it. A single globally mutable “current provider” was not observed in this path.

4. **FSR3 upscaler API contexts have separate provider-side state.** The provider creates a fresh internal upscaler context, backend interface, resources and scratch allocation for each creation call. This gives positive implementation evidence for independent lifetime state, but not concurrent execution safety.

5. **Concrete shared/global state now has been identified.** It includes the API-wide message callback/debug level, global/version-query name storage, and module-static DX12 allocation callback pointers.

6. **Not all native threading rules are wholly undocumented.** Frame Generation has an explicit v2.3.0 upstream statement that its underlying contexts are not guaranteed thread-safe and that specified operations require external synchronization. That evidence is effect-specific and does not resolve FSR upscaler behavior.

7. **The central M5 question remains unresolved:** no pinned public contract was found permitting simultaneous native API execution on two independent upscaler contexts, whether they use the same provider, different providers, the same DX12 device, or different devices.

8. **Runtime/provider module lifetime remains only partly observable.** Contexts retain provider associations and external-provider objects have visible ownership behavior, but effect-DLL residency/unloading and loader synchronization are not sufficiently exposed to establish a stronger sharing contract.

These findings neither confirm nor invalidate the project's existing M4 `!Send + !Sync` and exclusive-runtime choices; those remain project-side constraints rather than conclusions of this source investigation.

## Limits and follow-up

The main limit is source visibility. The documented signed loader's module-management implementation and the DX12 external/driver-provider internals are not sufficiently present in the inspected public source to answer effect-module unloading, provider-global synchronization, or driver-provider concurrency questions. The generic API and upscaler public contracts also contain no affirmative cross-context thread-safety statement.

Two bounded native experiments could add implementation evidence without being mistaken for contractual proof:

- **Provider/effect-module lifetime observation:** create two contexts through one loaded API runtime, observe the relevant effect/provider modules, destroy only one context, and determine whether the current signed v2.3.0 runtime unloads or otherwise changes the provider module while the second remains alive. This would distinguish current-binary residency behavior; it would not establish a portable lifetime guarantee.
- **Concurrent independent-context rejection observation:** perform a narrowly selected pair of otherwise-valid calls on two independently created contexts and record whether the signed runtime produces an explicit API error, validation message, or other deterministic rejection attributable to overlap. An explicit rejection would be useful evidence of a current implementation constraint. Successful execution or absence of a crash would **not** establish thread safety.

No generic parallel stress test can promote the unresolved rows in the matrix to a contractual concurrency guarantee.
