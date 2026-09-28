# Source investigation: FidelityFX v2.3.0 destroy failure paths

- Investigation date: 2026-09-20
- Topic and scope: Source characterization of `ffxDestroyContext` failure paths for FidelityFX SDK v2.3.0, focused on the Windows/DX12 upscaler API. The investigation covers the public API/common provider layer, open FSR2 and FSR3 upscaler providers, the DX12 backend reached by those providers, and the limits of the available source for the signed loader and FSR4/ML provider.
- Baseline and evidence cut-off: Primary repository `GPUOpen-LibrariesAndSDKs/FidelityFX-SDK`, tag `v2.3.0`, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. The v2.3.0 release is dated 2026-06-24 and advertises AMD FSR Upscaling 4.1.1. Accessed 2026-09-20.
- Method and exclusions: Static inspection of exact-v2.3.0 public source, headers, documentation, source inventory/license material, and release metadata. No implementation work, runtime execution, destructive testing, allocator-failure testing, deliberate context corruption, Vulkan analysis, or Rust API design was performed. Statements below distinguish public-source behavior from behavior of the shipped signed 4.1.1 provider.

## Inputs

Primary version references:

- Exact commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`. [v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3?utm_source=chatgpt.com)
- Release: [FidelityFX SDK v2.3.0 release](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0?utm_source=chatgpt.com)
- Public API header: [Kits/FidelityFX/api/include/ffx_api.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h?utm_source=chatgpt.com)
- C++ wrapper: [Kits/FidelityFX/api/include/ffx_api.hpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.hpp?utm_source=chatgpt.com)
- Client loader helper: [Kits/FidelityFX/api/include/ffx_api_loader.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_loader.h?utm_source=chatgpt.com)
- Common API implementation: [Kits/FidelityFX/api/internal/ffx_api.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp?utm_source=chatgpt.com)
- Provider base/selection: [Kits/FidelityFX/api/internal/ffx_provider.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h?utm_source=chatgpt.com)
- API helpers/allocator/error translation: [Kits/FidelityFX/api/internal/ffx_api_helper.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api_helper.h?utm_source=chatgpt.com)
- Backend-object helpers: [Kits/FidelityFX/api/internal/ffx_object_management.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_object_management.cpp?utm_source=chatgpt.com)
- FSR2 provider/core: [ffx_provider_fsr2.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp?utm_source=chatgpt.com) and [ffx_fsr2.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr2.cpp?utm_source=chatgpt.com)
- FSR3 provider/core: [ffx_provider_fsr3upscale.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp?utm_source=chatgpt.com) and [ffx_fsr3upscaler.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp?utm_source=chatgpt.com)
- DX12 backend: [Kits/FidelityFX/backend/dx12/ffx_dx12.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp?utm_source=chatgpt.com) and [ffx_backends_dx12.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp?utm_source=chatgpt.com)
- Generic API documentation: [Kits/FidelityFX/docs/getting-started/ffx-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md?utm_source=chatgpt.com)
- Public/source-binary inventory: [Kits/FidelityFX/docs/license.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/license.md?utm_source=chatgpt.com)

The release identifies FSR Upscaling 4.1.1, while the license/source inventory distinguishes public source from signed binaries including `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`. The public `ffx_provider.h` also includes an `amdinternal/api/internal/dx12/ffx_provider_external.h` path that is not present in the public v2.3.0 tree inspected. **Verified** for the source inventory; **Unresolved** for the implementation behind that external provider path.

## Findings

### Entry-point and dispatch flow

The public API type is `typedef void* ffxContext`. Relevant API results include `FFX_API_RETURN_OK = 0`, `FFX_API_RETURN_ERROR_RUNTIME_ERROR = 3`, and `FFX_API_RETURN_ERROR_PARAMETER = 6`. The header defines `ffxDestroyContext(ffxContext* context, const ffxAllocationCallbacks* memCb)` and states only that non-zero return values indicate errors; it does not define retry semantics or post-error context validity. **Verified.**

`ffx_api_loader.h` is an application-side symbol resolver. Its `ffxFunctions` table contains `DestroyContext`, and `ffxLoadFunctions` obtains the exported `ffxDestroyContext` with `GetProcAddress`. It does not implement provider destruction itself. **Verified.**

AMD's v2.3.0 documentation says the small signed loader DLL manages loading effect-type DLLs, while the upscaler DLL contains the FSR upscaling implementations. The actual forwarding/validation implementation inside the shipped signed `amd_fidelityfx_loader_dx12.dll` is not in the inspected public source. Consequently the exact binary path from its exported `ffxDestroyContext` to the 4.1.1 provider cannot be reconstructed from public v2.3.0 source. **Upstream claim** for the architectural description; **Unresolved** for the binary's precise destroy implementation.

The corresponding open common implementation in `api/internal/ffx_api.cpp` performs this sequence:

1. `VERIFY(context != nullptr, FFX_API_RETURN_ERROR_PARAMETER)`.
2. Construct `Allocator alloc{memCb}`.
3. Obtain the provider with `GetAssociatedProvider(*context)`.
4. Call `provider->DestroyContext(context, alloc)`.
5. Irrespective of the provider return value, test `provider->GetRefCount() == 0`; if zero, explicitly destroy and deallocate the provider.
6. Return the provider's return code unchanged.

**Verified.**

`GetAssociatedProvider` casts the context value to `InternalContextHeader*` and immediately reads its `provider` field. There is no preceding check that `*context` is non-null, live, points to an `InternalContextHeader`, or has a valid provider. **Verified.**

The open common layer therefore performs no error-code translation around `provider->DestroyContext`; provider API return codes pass through. Inside providers, `TRY2(coreCall)` maps every non-`FFX_OK` core `FfxErrorCode` to the single public value `FFX_API_RETURN_ERROR_RUNTIME_ERROR` (3). `TRY` instead propagates an API return code unchanged. **Verified.**

### Null, stale and invalid contexts

| Input/state | Open common implementation result | Teardown before result | Classification |
|---|---|---|---|
| `context == nullptr`, i.e. null pointer-to-handle argument | Controlled `FFX_API_RETURN_ERROR_PARAMETER` (6) | None. Return occurs before provider lookup. | **Verified** |
| `context != nullptr`, `*context == nullptr` | Common code proceeds to `GetAssociatedProvider(nullptr)` and dereferences through the null context value. No controlled API error is established. | None can be relied upon. | **Verified** source flow; resulting C++ behavior is undefined. |
| Valid live open-provider context | Header yields its associated provider and provider destruction begins. | Provider-specific; detailed below. | **Verified.** |
| Successfully destroyed open-provider context whose caller-visible bits were retained | Backing `Internal*Context` has been freed but caller value was not cleared. A second generic destroy attempts to obtain a provider through freed storage. | Undefined use-after-free; no retry/error contract. | **Verified** source flow / **Inference** about stale-handle classification. |
| Arbitrary non-null/bogus handle | Cast and dereference as an `InternalContextHeader`, then potentially invocation through an arbitrary provider pointer. | Unspecified/undefined; no useful controlled validation. | **Verified** source flow. |

Both the open FSR2 and FSR3 provider `DestroyContext` methods themselves contain checks for both `context` and `*context`, returning `FFX_API_RETURN_ERROR_PARAMETER`. However, the public generic open entry point must already dereference `*context` to discover the provider. Therefore the inner provider check does not make a pointer-to-null-handle a safe public input. **Verified.**

No public-source mechanism recognizes an already-destroyed/stale handle as a distinct state. The caller-visible pointer is not a liveness token. **Verified** for the open implementation; **Unresolved** for any extra validation possibly present in the shipped signed loader/provider.

### Open FSR2 provider path

`InternalFsr2Context` begins with `InternalContextHeader`, contains the backend interface and embedded `FfxFsr2Context`, and is the allocation to which the public `ffxContext` points. Creation stores `header.provider = this`. **Verified.**

`ffxProvider_FSR2::DestroyContext` performs:

1. Provider-level checks for `context` and `*context`.
2. Cast to `InternalFsr2Context`.
3. `TRY2(ffxFsr2ContextDestroy(&internal_context->context))`.
4. Deallocate the backend scratch buffer.
5. Deallocate `internal_context`.
6. Return `FFX_API_RETURN_OK`.

There is no assignment to `*context`. **Verified.**

`ffxFsr2ContextDestroy` has one explicit failure condition: a null `FfxFsr2Context*` parameter produces the core invalid-pointer error. Otherwise it invokes `fsr2Release`. The provider always passes the address of its embedded context object, so this null-pointer condition is not reachable from a structurally valid live `InternalFsr2Context`. **Verified** for the core condition; **Inference** that it has no supported valid-provider trigger.

`fsr2Release` releases pipelines, unregisters external references, releases copy/internal resources, calls `fpDestroyBackendContext`, and then returns `FFX_OK` unconditionally. The generic `ffxSafeReleasePipeline`, `ffxSafeReleaseCopyResource`, and `ffxSafeReleaseResource` helpers invoke backend destruction callbacks without propagating their returned `FfxErrorCode`. **Verified.**

Consequently, for the inspected open FSR2 provider with a valid live provider context, no source-visible ordinary-use path produces a non-success `ffxDestroyContext` result. Backend resource/pipeline destruction errors are not converted into a public destruction error by this core path. **Inference**, directly from the inspected control flow.

If the provider's `TRY2(ffxFsr2ContextDestroy(...))` did return public runtime error 3, provider scratch and `InternalFsr2Context` deallocation would be skipped because `TRY2` returns immediately. The only explicit core error found, however, requires a null pointer that this provider does not supply in valid state. Thus this branch is syntactically present but does not furnish a legitimate valid-context failure mechanism. **Verified** ordering; **Inference** on reachability.

On successful FSR2 teardown, the core/backend teardown occurs first, then host scratch memory and the native context allocation are freed. The caller's handle value is not cleared, so its unchanged non-null bits refer to freed storage. **Verified** for the open provider.

### Open FSR3 provider path

`InternalFsr3UpscalerContext` contains the leading `InternalContextHeader`, backend interface, a `sharedResources[FFX_FSR3_RESOURCE_IDENTIFIER_COUNT]` array, and the embedded core upscaler context. `Allocator::construct` zero-initializes the provider allocation, after which creation populates the provider header and creates its required shared resources. **Verified.**

`ffxProvider_FSR3Upscale::DestroyContext` differs materially from FSR2. After its argument checks, it first iterates through the provider-owned `sharedResources` array and executes:

`TRY2(internal_context->backendInterface.fpDestroyResource(...))`

for each entry. Only after every such operation succeeds does it invoke:

`TRY2(ffxFsr3UpscalerContextDestroy(&internal_context->context))`

and only after that succeeds does it free the backend scratch buffer and `InternalFsr3UpscalerContext`. **Verified.**

This creates a real source-level error-propagation point that FSR2 lacks: any non-`FFX_OK` result from destruction of a provider-owned shared resource becomes public `FFX_API_RETURN_ERROR_RUNTIME_ERROR` (3). **Verified.**

For the stock DX12 backend, `DestroyResourceDX12` checks whether the resource's `internalIndex` lies in the effect context's allocated static-resource range. If it does not, it returns `FFX_ERROR_OUT_OF_RANGE`; otherwise it releases the resource if non-null, clears the backend resource pointer, and returns `FFX_OK`. No other normal error return was found in that function. **Verified.**

The provider-created FSR3 state does not expose a supported route to that out-of-range condition. `CreateBackendContextDX12` initializes the static-resource index range for the effect context; `CreateResourceDX12` assigns indexes from that range and advances `nextStaticResource`. The FSR3 provider allocation begins zeroed, so unused entries contain index zero; with the provider's DX12 backend-context setup, zero is itself accepted by the range check while actually created entries are the indexes handed out by the backend. **Verified** for the individual index-management operations; **Inference** that a structurally valid stock DX12 FSR3 context cannot naturally make this destroy loop return `FFX_ERROR_OUT_OF_RANGE`.

Therefore the FSR3 shared-resource failure branch is source-visible but no legitimate ordinary supported-use trigger was identified. Reaching it with the inspected backend requires inconsistent private resource-index/backend state rather than an ordinary application-controlled destruction condition. Deliberately manufacturing that inconsistency would be corruption, not a useful runtime failure mechanism. **Inference.**

If such a failure nevertheless occurs, its post-state is materially different from an untouched live context: every earlier shared-resource entry in the loop has already been processed, while later entries remain unprocessed; the core FSR3 context has not yet been destroyed; backend scratch and the provider's host context allocation have not been freed. The source therefore establishes **partial teardown** for a failure after at least one successful loop iteration. **Verified** from operation ordering.

`ffxFsr3UpscalerContextDestroy` itself returns the core invalid-pointer error if passed a null pointer and otherwise performs `fsr3UpscalerRelease`. The provider supplies the address of its embedded core context, so the explicit null-pointer failure is not a valid-provider failure mechanism. **Verified** condition; **Inference** on valid-provider reachability.

`fsr3UpscalerRelease`, like FSR2 release, destroys its pipelines/resources and backend context and then returns `FFX_OK`; the safe-release helpers do not propagate backend callback failures. **Verified.**

If a hypothetical core-destroy error were propagated by the provider, all provider-owned shared-resource loop iterations would already have completed, while core teardown and host frees would not have completed. That would likewise be a partially torn-down object. No legitimate valid-state trigger for that core error was identified. **Verified** ordering / **Inference** reachability.

### DX12 backend teardown and device/reference lifetime

The DX12 interface wires `fpDestroyBackendContext` to `DestroyBackendContextDX12`, `fpDestroyResource` to `DestroyResourceDX12`, and `fpDestroyPipeline` to `DestroyPipelineDX12`. **Verified.**

Backend-context creation retains the D3D12 device and increments the backend context/reference accounting. `DestroyBackendContextDX12` sweeps remaining resources, marks the effect context inactive, resets its resource index state, decrements the backend reference count, and, when the last backend context is gone, releases shared DX12 objects including the retained device and DXGI factory. It returns `FFX_OK`. Calls made while sweeping resources do not create a propagated destroy error. **Verified.**

`DestroyPipelineDX12` releases the pipeline-related COM objects and returns `FFX_OK`. **Verified.**

Accordingly, the inspected open FSR2/FSR3 core destruction paths provide no public error result corresponding merely to D3D12 COM release, normal backend-context shutdown, or a backend resource/pipeline callback failure encountered through the core safe-release helpers. **Verified** for the callbacks and return flow; this does not establish behavior of the closed 4.1.1 provider.

### Host allocator behavior during destruction

The API documentation requires destruction to use allocation callbacks compatible with those used for creation. The implementation's deallocation callback has `void` return type; `Allocator::dealloc` therefore has no failure result that can be converted to an `ffxReturnCode_t`. **Verified.**

An incompatible or otherwise contract-violating allocator can cause invalid memory behavior, but it is not a source-supported `ffxDestroyContext` error path. There is no legitimate "host free failed, context remains live" error result in the inspected interface. **Verified** for the absence of a return channel; consequences of violating the callback contract are outside the supported API.

### Complete source-visible non-success classification

For the public/open path inspected at v2.3.0:

| Failure source | Public result | Valid before call? | Ordinary supported use? | State on return |
|---|---:|---|---|---|
| Common `context == nullptr` | `FFX_API_RETURN_ERROR_PARAMETER` (6) | No context supplied | Controlled invalid argument | No teardown; **definitely no context affected**. **Verified.** |
| `context != nullptr`, `*context == nullptr` through common API | No controlled result established | No live context | No; invalid input enters undefined behavior before provider validation | **Unknown/undefined. Verified source flow.** |
| Stale/already-freed handle | No controlled result established | No | No; dereference of freed context header | **Unknown/undefined.** |
| Arbitrary non-null invalid handle | No controlled result established | No | No | **Unknown/undefined.** |
| FSR2 provider's own argument checks, if reached directly | API parameter error (6) | No | Internal/direct-call validation, not a safe generic-null-handle mechanism | No provider teardown before check. **Verified.** |
| FSR2 core destroy non-OK through `TRY2` | API runtime error (3) | Explicit core error is null core pointer | No valid provider trigger identified | Host scratch/context free skipped. Explicit null-pointer core error occurs before core teardown. **Verified/Inference.** |
| FSR3 provider's own argument checks, if reached directly | API parameter error (6) | No | Same limitation as FSR2 | No provider teardown before check. **Verified.** |
| FSR3 `fpDestroyResource` non-OK in shared-resource loop | API runtime error (3) | Could only be called on an apparently provider-associated object; stock DX12 error found is out-of-range private index | No legitimate valid-state trigger identified | Earlier loop items may already be destroyed; core and host allocation remain: **partial teardown**. **Verified/Inference.** |
| FSR3 core destroy non-OK through `TRY2` | API runtime error (3) | Explicit core error is null core pointer | No valid provider trigger identified | Shared-resource phase already completed; core/host teardown not completed: **partial teardown** if this branch were reached. **Verified/Inference.** |
| Host deallocator failure | No return code exists | — | Not representable | No API failure semantics exist. **Verified.** |
| Open FSR2 valid live context | No non-success path identified | Yes | Normal supported use | Successful full teardown; caller value retained as stale bits. **Inference from complete inspected path.** |
| Open FSR3 valid live context on stock DX12 | No legitimate non-success path identified | Yes | Normal supported use | Successful full teardown; caller value retained as stale bits. **Inference from inspected provider/backend invariants.** |
| Signed/ML/external 4.1.1 provider | Cannot enumerate from available source | Potentially | Unknown | **Unresolved.** |

The public error enumeration includes additional values such as generic error, no-provider, memory, and provider-no-support. Their existence does not establish that `ffxDestroyContext` can emit them. No destroy path in the inspected open FSR2/FSR3 implementation was found that generates those values. **Verified** for the enum; **Inference** restricted to the inspected destroy paths.

### Common provider lifetime after an error

The generic `ffxDestroyContext` checks the associated provider's reference count after `provider->DestroyContext` returns, regardless of whether that return value is success or failure. If the count is zero, it explicitly destructs and deallocates the provider before returning the error to the caller. **Verified.**

The open FSR2/FSR3 provider instances inspected are static providers whose base `ffxProvider` reference count starts at one; no corresponding refcount transition to zero was found in their destruction path. **Verified** for construction/source layout.

For an external provider, however, this common code means that "provider destroy returned error" does not itself imply that the provider object is retained. The behavior depends on the external provider's refcount policy, whose implementation is unavailable. **Inference** from common code; **Unresolved** for the v2.3.0 external/4.1.1 provider.

### Retry semantics

No inspected v2.3.0 header or documentation explicitly supports retrying `ffxDestroyContext` after an error. No inspected material states that an error leaves the context intact. The generic documentation instead notes generally that API errors should be handled even where they may be unrecoverable; that is not a retry guarantee. **Verified** for absence in the inspected contract; **Upstream claim** for the general error-handling text.

For successful destruction in the open implementation, retry is clearly not a valid operation: the provider frees the allocation containing the internal context, the common/provider source does not clear the caller's pointer value, and a subsequent generic destroy begins by reading the provider pointer through that stale allocation. **Verified** source ordering; the second call enters undefined use of freed memory.

For a returned error, the source does not provide a general retry rule. In particular, the only substantive propagated cleanup-error path found in the open providers is FSR3's shared-resource loop, where failure can occur after earlier resources have already been destroyed. Thus even in open source, "error means no teardown happened, therefore retry" is false as a general inference. **Verified** ordering.

Whether retrying after that partial FSR3 failure would happen to succeed for any particular inconsistent backend state is not an API guarantee and was not established. **Unresolved.**

For the signed/external/ML 4.1.1 provider, retry behavior after a non-success return is wholly **Unresolved** from available public source. The common open code's willingness to release a zero-refcount provider even when provider destruction returned an error further prevents deriving a generic "error leaves provider/context alive" rule without the provider implementation.

### Handle post-state and documentation conflict

The v2.3.0 generic API documentation states, for context destruction, that "The context will be `NULL` after the call." **Upstream claim.**

The exact-v2.3.0 open implementation contradicts that claim:

- Common `ffxDestroyContext` never assigns null to `*context`.
- Open FSR2 frees `InternalFsr2Context` without clearing the caller's handle.
- Open FSR3 frees `InternalFsr3UpscalerContext` without clearing the caller's handle.
- The C++ `ffx::DestroyContext(Context& context, ...)` helper merely calls `ffxDestroyContext(&context, ...)`; it performs no subsequent null assignment.

This is a direct v2.3.0 documentation/implementation contradiction. **Verified.**

The C header itself is narrower than the generic documentation: it says the function destroys a context, requires compatible allocation callbacks, and treats non-zero as an error, but does not promise that the handle becomes null. It also specifies neither validity after failure nor retry behavior. **Verified.**

For an open-provider success, the strongest source-level postcondition is therefore: native backing allocation destroyed/freed, caller-visible handle bits unchanged, resulting handle stale. This is consistent with the project's observed successful destruction followed by a non-null/unchanged handle. It does not prove that every signed 4.1.1 provider follows the same internal implementation. **Verified** for open source; **Unresolved** for the closed provider.

### Provider differences

**Common/open API layer.** It validates only the outer pointer before reading the provider pointer from the context. It propagates provider return codes and may delete a zero-refcount provider even after provider destruction reports failure. It does not null the caller handle. **Verified.**

**Open FSR2.** No legitimate non-success condition was found for a structurally valid live context. Its core release suppresses backend cleanup return values and returns success; the sole explicit core destroy error is a null core-context pointer not supplied by the valid provider. Successful provider teardown frees scratch and the internal host allocation without nulling the caller handle. **Verified/Inference.**

**Open FSR3.** It introduces a pre-core provider-owned shared-resource cleanup loop whose backend errors are propagated as public runtime error 3. On DX12 the relevant backend error is an out-of-range internal resource index. Source-managed valid state appears to keep those indices in range; therefore no ordinary supported-use trigger was identified. If this failure does occur, teardown may already be partial. **Verified/Inference.**

**FSR4/ML/external provider.** The v2.3.0 release advertises FSR Upscaling 4.1.1, but the provider implementation relevant to the signed binary/external-provider path is not available in the inspected public tree. `ffx_provider.h` references an `amdinternal/.../ffx_provider_external.h` file absent from the public repository, and the source/binary inventory lists the signed upscaler/loader binaries separately. **Verified** absence/inventory; **Unresolved** provider behavior.

Accordingly, none of the open FSR2/FSR3 post-error conclusions may be silently generalized to the provider that reported version 4.1.1 in the project experiment. The release version match identifies relevant packaging, not implementation equivalence. **Inference.**

## Baseline delta

Confirmed:

- The project's observation that successful destruction can leave the native handle non-null and bitwise unchanged is exactly consistent with the v2.3.0 open common/FSR2/FSR3 implementations: none clears `*context`, while successful provider destruction frees the internal context allocation. The remaining value is therefore stale in those implementations.
- A successful open-provider destroy cannot be treated as producing a null liveness marker.
- The C++ convenience wrapper does not repair the discrepancy by nulling the handle.
- Normal open FSR2 destruction exposes no legitimate recoverable failure result for a valid live context.
- Normal open FSR3/DX12 destruction contains an error-propagation point, but the only inspected DX12 trigger requires an inconsistent private resource index rather than an ordinary supported application state.
- Backend/core cleanup is less error-reporting than the public error enum might suggest: several backend callback results are intentionally not propagated by the FSR2/FSR3 core release helpers.

Contradicted:

- The official generic v2.3.0 documentation claim that the context becomes `NULL` after destruction is contradicted by the exact open v2.3.0 common implementation, both open upscaler provider implementations, and the C++ wrapper.

Refined:

- A null pointer-to-context argument is a controlled open-source failure: API parameter error 6, before any teardown.
- A pointer to a null context handle is materially different. In the generic open entry point it reaches `GetAssociatedProvider(*context)` before provider-level null-handle validation and therefore is not a controlled error input.
- An already-destroyed handle is likewise not recognized as stale; successful open destruction frees its backing allocation but leaves its bits unchanged, so re-destruction dereferences freed state.
- A destroy error does not generically imply "nothing happened." The open FSR3 shared-resource path can return after earlier teardown actions have already completed.
- A destroy error also does not generically imply that the provider object itself survives: the common layer performs its zero-refcount provider cleanup even after provider destruction reports an error.
- Compatible destroy-time allocator callbacks are an API obligation, but host deallocation itself has no error-return channel and therefore cannot supply a legitimate `ffxDestroyContext` failure code.

Left unresolved:

- The exact implementation of the exported destroy path inside `amd_fidelityfx_loader_dx12.dll`.
- The `DestroyContext` implementation and failure conditions of the FSR4/ML/external provider corresponding to the shipped 4.1.1 path.
- Whether that 4.1.1 provider can return an error for a valid live context during otherwise supported use.
- If it can, whether such an error means still-live, destroyed, partially destroyed, or some provider-specific state.
- Retry semantics following a 4.1.1 provider failure.
- Whether the shipped signed loader adds validation for null/stale handles beyond the open implementation.
- Source inspection alone does not establish runtime behavior of the signed Windows/DX12 binaries.

## Limits and follow-up

The critical limitation is that the public v2.3.0 repository does not expose enough of the signed/external FSR4/ML 4.1.1 provider implementation to enumerate its destruction errors or post-error state. The open FSR2 and FSR3 implementations are useful evidence about the public API architecture, error translation, handle treatment and possible teardown ordering, but they are not evidence that the observed 4.1.1 provider uses identical destruction semantics.

No legitimate, non-UB, reproducible **valid-live-context destroy failure** was identified from the available v2.3.0 source that is suitable for a lifecycle experiment against the real runtime. Open FSR2 offers none. Open FSR3's propagated DX12 resource-destroy error requires inconsistent/corrupted private backend state under the inspected invariants and therefore should not be manufactured as a test mechanism. The only clean reproducible non-success identified is `ffxDestroyContext(nullptr, ...)` returning parameter error 6 in the open common implementation; because it involves no native context, it can test signed-loader API conformance but cannot answer the unresolved post-failure liveness question.

If AMD later exposes a legitimate 4.1.1 valid-context failure condition, an isolated runtime spike would need to record at minimum the exact selected provider identity/version, exact `ffxDestroyContext` return value, caller-visible handle bits before and after the call, valid allocator deallocation-callback activity during the call, and relevant D3D12 debug/info-queue observations. It should distinguish evidence that teardown occurred before the error from evidence merely that the handle value changed or did not change. A retry or any access through the post-call context should not be used as a probe unless upstream material first establishes that such use is valid.

Without either the unavailable 4.1.1 provider source or a documented legitimate provider failure trigger, the central 4.1.1 question remains **Unresolved**: a non-success destroy result cannot presently be mapped from source to "still live", "already dead", or a specific partial-teardown state. The strongest available v2.3.0 evidence instead shows that the API has no general transactional-destruction guarantee and no documented retry contract, while the open FSR3 flow demonstrates that an error return can in principle occur after teardown has begun.
