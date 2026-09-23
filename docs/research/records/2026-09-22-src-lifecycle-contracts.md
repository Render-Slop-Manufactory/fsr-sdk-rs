# FidelityFX v2.3.0 lifecycle contract investigation

- **Date:** 2026-09-22
- **Scope:** AMD FidelityFX SDK v2.3.0, Windows x64/MSVC, AMD FSR API and DX12 backend; creation-descriptor storage, upscaler message callback, DX12 device COM ownership, and CPU-side context/threading contracts.
- **SDK/tag/commit:** `v2.3.0`, release/tag commit `60f4ea8`. GitHub identifies that commit for the v2.3.0 release; the release contains AMD FSR Upscaling 4.1.1.
- **Method:** Inspected the exact tagged public headers, tagged documentation, common API/provider routing, open FSR2/FSR3 providers, common DX12 backend, and message implementation. Source-control flow is used only to establish open implementation facts. No runtime experiments were performed. Current `main`/live documentation was not substituted for tagged material.

## Question

What lifecycle guarantees does AMD FidelityFX SDK v2.3.0 actually establish for:

1. storage making up an `ffxApiHeader::pNext` creation chain;
2. `ffxCreateContextDescUpscale::fpMessage`;
3. the `ID3D12Device*` in `ffxCreateBackendDX12Desc`;
4. concurrent use, creation, destruction, provider selection, and callbacks?

The relevant concrete creation chain is:

```text
ffxCreateContextDescUpscale
  -> ffxCreateContextDescUpscaleVersion
  -> ffxCreateBackendDX12Desc
  -> null
```

The distinction throughout this record is between the public API contract, behavior visible in the open v2.3.0 implementation, and behavior of a signed/external provider selected by the shipped runtime.

## Baseline supplied

The supplied baseline identified four unresolved points: apparently broad creation-pointer lifetime language versus a shorter version-override lifetime; persistent storage of the upscaler message function in open providers without a stated execution model; observed DX12 backend COM retention without an equivalent public guarantee; and absence of a general upscaler thread-safety statement.

Those points are substantially confirmed, but two require material refinement: descriptor-node lifetime is demonstrably descriptor-specific rather than uniformly “until destroy”, and the open FSR2/FSR3 message callback has module-global state whose lifetime is not bounded by destruction of the context that installed it.

## Sources inspected

All repository paths below are from tag `v2.3.0` / commit `60f4ea8`; access date 2026-09-22.

| Path / source | Relevant material |
|---|---|
| GitHub release `v2.3.0` — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0?utm_source=chatgpt.com) | tag/commit, FSR Upscaling 4.1.1 |
| `Kits/FidelityFX/api/include/ffx_api.h` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/ffx_api.h](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/ffx_api.h?utm_source=chatgpt.com) | `ffxApiHeader`, callback typedef, `ffxCreateContext` lifetime text |
| `Kits/FidelityFX/api/include/ffx_api.hpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/ffx_api.hpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/ffx_api.hpp?utm_source=chatgpt.com) | C++ descriptor-chain helper |
| `Kits/FidelityFX/api/internal/ffx_api.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_api.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_api.cpp?utm_source=chatgpt.com) | version selection, provider selection, global debug configuration |
| `Kits/FidelityFX/api/internal/ffx_provider.h` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_provider.h](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_provider.h?utm_source=chatgpt.com) | internal/external-provider selection, enumeration shared state |
| `Kits/FidelityFX/docs/getting-started/ffx-api.md` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md?utm_source=chatgpt.com) | descriptor model, context examples, override lifetime |
| `Kits/FidelityFX/upscalers/include/ffx_upscale.h` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com) | `fpMessage`, `ffxCreateContextDescUpscaleVersion` |
| `Kits/FidelityFX/docs/techniques/super-resolution-ml.md` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md?utm_source=chatgpt.com) | FSR4 compatibility descriptor and debug checker |
| `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h?utm_source=chatgpt.com) | public DX12 backend descriptor |
| `Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp?utm_source=chatgpt.com) | synchronous backend-extension traversal |
| `Kits/FidelityFX/backend/dx12/ffx_dx12.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp?utm_source=chatgpt.com) | DX12 `AddRef`/`Release`, backend reference count |
| `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp?utm_source=chatgpt.com) | open FSR2 provider |
| `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp?utm_source=chatgpt.com) | open FSR3 provider |
| `Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr2.cpp` / `ffx_fsr3upscaler.cpp` | legacy upscaler dispatch/debug implementation |
| `Kits/FidelityFX/api/internal/ffx_message.cpp` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_message.cpp](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_message.cpp?utm_source=chatgpt.com) | process/module-global callback state |
| `Kits/FidelityFX/docs/techniques/frame-interpolation-api.md` — [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md?utm_source=chatgpt.com) | explicit Frame Generation thread-safety wording for comparison only |

## Findings

### Descriptor-chain / `pNext` lifetime

**Generic public wording — Verified.** `ffxCreateContext` states: “Pointers passed in desc must remain live until ffxDestroyContext is called on the context.” Separately, `ffxApiHeader::pNext` is defined as a pointer to another descriptor used for extensions.

That wording does **not** unambiguously define whether “pointers passed in desc” means every pointer value including `pNext` itself, only pointer-valued resources referenced by descriptor fields, or both. A purely literal interpretation that every `pNext` target object must remain live until context destruction is contradicted by AMD's own version-override example.

**`ffxOverrideVersion` lifetime — Verified.** The exact v2.3.0 API guide constructs `CreateContextDescOverrideVersion` and explicitly comments that its lifetime only “must last until after CreateContext call.” This is direct evidence that at least one creation-chain node may cease to exist immediately after context creation.

The common implementation is consistent with that statement: `GetVersionOverride` synchronously traverses the chain before selecting a provider. However, source behavior alone would not establish the contractual lifetime because the selected provider still receives the original creation-chain head; the documentation's explicit lifetime statement is the stronger evidence.

**Reconciliation — Unresolved at the generic level.** The evidence supports either of two narrower readings: descriptor-specific exceptions exist to the broad header sentence, or that sentence primarily concerns pointees/dependencies whose liveness extends into context operation rather than necessarily the storage containing every descriptor node. AMD does not state either reconciliation explicitly. Consequently, “all creation descriptor objects must remain allocated until destruction” is not a defensible universal v2.3.0 contract.

The tagged API guide's ordinary context example also uses local `createUpscale` and `createBackend` objects, and the C++ helper constructs `pNext` links immediately before `ffxCreateContext`. Neither source explicitly says those particular objects may die when the call returns, so they do not provide the same lifetime evidence as the override comment.

**Open FSR2/FSR3 providers — Verified implementation facts.** Their persistent provider-context structures contain copied state, backend interface/context objects and the callback pointer, but no pointer to the original creation descriptor or chain. Creation copies the relevant upscale fields and synchronously calls `MustCreateBackend`. The common DX12 backend walks `desc->pNext` synchronously, reads the backend descriptor, translates its device into an `FfxInterface`, and does not store the descriptor-node address. Thus no retained `pNext` descriptor address was found in the inspected open FSR2/FSR3+DX12 path.

That is **not** a universal contract: the common provider layer can select an external provider on DX12, including in preference to an internal provider based on version.

**`ffxCreateContextDescUpscaleVersion` — Verified requirement, Unresolved lifetime.** FSR4 documentation requires this descriptor to be linked through `header.pNext` and says it is used for API compatibility; the public structure itself merely requires `version = FFX_UPSCALER_VERSION`. Neither source assigns it a post-create lifetime.

Therefore, for the concrete chain

```text
Upscale -> UpscaleVersion -> BackendDX12 -> null
```

there is no source-supported rule reducing all three descriptor-node lifetimes to the `ffxOverrideVersion` rule. Conversely, the generic header wording cannot prove that all three node objects must survive until destroy. **The node-storage lifetime of `UpscaleVersion` and `BackendDX12`, as a public signed-provider contract, remains unresolved.**

No public v2.3.0 taxonomy was found distinguishing “selection-only extension descriptors” from “persistent-dependency descriptors”. Such a distinction is visible in implementation behavior, but AMD does not formalize it as a generic API rule.

### Message callback lifetime and threading

**Public surface — Verified.** `ffxCreateContextDescUpscale::fpMessage` is documented only as a function that may receive runtime messages and “may be null”. Its type is `void (*)(uint32_t, const wchar_t*)`; there is no userdata pointer.

The FSR4 v2.3.0 documentation says the debug checker validates application inputs “at dispatch upscale” and that an application can assign `fpMessage` to receive those messages. This is an **Upstream claim** describing shipped-runtime behavior, not an independently reproduced result here. It does not specify callback lifetime, thread identity, serialization, reentrancy, or asynchronous execution.

**Open FSR2/FSR3 storage — Verified implementation fact.** Both providers copy `desc->fpMessage` into persistent provider-context state. They also pass it into their legacy implementation's creation description and, after creation, install it using `ffxFsr2SetGlobalDebugMessage` / `ffxFsr3UpscalerSetGlobalDebugMessage`.

The important refinement is that those setters feed a **module-global** callback variable. `ffx_message.cpp` has static `s_messageCallback` and `s_debugLevel`; the setter simply assigns them, and `ffxPrintMessage` directly invokes `s_messageCallback` when non-null. No mutex or atomic synchronization is present around those variables.

**Invocation path — Verified for the open providers.** FSR2 and FSR3 debug-check dispatch code synchronously executes validation and directly reaches `FFX_PRINT_MESSAGE`; the common message function then directly calls the callback. For that path, callback execution occurs on the CPU thread executing that call stack; there is no worker-thread handoff in the inspected message path. This is an implementation/control-flow fact, not a public calling-thread guarantee.

The inspected open provider routines establish the following operation-level behavior:

- **Create:** stores/passes/installs the callback. No direct invocation by the provider wrapper itself was found.
- **Configure:** `GLOBALDEBUG1` can replace the global callback and the per-context copy. The common API also supports global debug configuration independently of an effect context.
- **Dispatch:** active debug-check paths invoke messages synchronously.
- **Query:** provider-side validator code that might have used the stored callback is commented out in the inspected FSR3 provider; no active direct callback was found there.
- **Destroy:** neither open FSR2 nor FSR3 provider clears the global callback before freeing its context.

The last point prevents treating context destruction as a demonstrated callback-lifetime boundary for these implementations. **Inference:** after a non-null callback has been installed, its function pointer can remain in module-global state after the installing context is destroyed, until another operation replaces or clears it. Therefore even the open implementation does not establish “the callback only needs to remain valid until this context's destroy”.

Because `s_messageCallback` is global, any source path in that module reaching `ffxPrintMessage` while that pointer is installed can potentially invoke it. Thus the implementation structure itself is broader than a purely per-context callback model.

**Asynchrony/threading — Unresolved publicly.** No v2.3.0 upscaler/common API documentation inspected provides a calling-thread guarantee or states that callback calls are serialized. Keyword inspection found no `thread` wording in `ffx_api.h`, `ffx-api.md`, `ffx_upscale.h`, or the FSR4 Super Resolution document. The open FSR2/FSR3 debug path is synchronous, but that cannot establish behavior of an external/signed provider.

**Concurrent callback execution — Unresolved as a contract.** The open global callback machinery supplies no internal serialization. If independently permissible calling paths reached `ffxPrintMessage` concurrently, nothing in that code would serialize callback entry; concurrent mutation of the global callback/debug-level variables is likewise not protected. Whether those higher-level calls themselves are permitted concurrently is not documented, so this cannot be promoted to a public concurrency rule.

**Null callback — Verified.** Null is explicitly permitted. In the open message implementation, null selects debugger output instead of an application callback. Therefore a creation descriptor that supplies null introduces no application callback function-pointer lifetime dependency of its own. This does not say anything about other separately configured global debug callbacks.

**Provider variability — Unresolved/publicly unconstrained beyond the ABI.** External providers are a real selection path and FSR4 4.1.1 is supplied through the v2.3.0 upscaler runtime. The sparse public callback contract does not require an external provider to reproduce the legacy provider's internal global-state implementation.

### DX12 device COM ownership

**Public declaration — Verified.** `ffxCreateBackendDX12Desc::device` is documented as “Device on which the backend will run.” The public descriptor declaration contains no `AddRef`, ownership-transfer, retained-reference, or caller-release statement.

No inspected public v2.3.0 source explicitly says that FidelityFX assumes ownership of a COM reference, nor that the application may release its own reference immediately after successful context creation.

The generic `ffxCreateContext` sentence nevertheless says pointers supplied through the creation descriptor must remain live until destruction. The strongest application-facing textual reading is therefore: **the application must ensure that the device referred to by the creation data remains usable/live for the period required by that contract.** Translating “live” into a particular COM-reference ownership transfer is not stated by AMD. Classification: **Inference from verified public wording**.

**Open DX12 implementation — Verified.** `CreateBackendContextDX12` obtains the backend scratch `BackendContext_DX12`; on its first active effect context (`refCount == 0`) it calls `ID3D12Device::AddRef()` and stores the device. Destruction decrements that backend context's reference count and, when it reaches zero, releases the stored device and other shared backend objects.

This establishes an implementation fact, not a public ownership guarantee.

The precise lifetime unit is also narrower than “one AddRef per FidelityFX context”: the retained COM reference belongs to a `BackendContext_DX12` scratch/backend-interface instance and is shared across the effect backend contexts represented by that backend's internal `refCount`. The open FSR2 provider asks for a backend capable of `FFX_FSR2_CONTEXT_COUNT` effect contexts, whereas the FSR3 upscaler provider asks for one.

Both open legacy upscaler providers therefore obtain the retention behavior by passing through the common open DX12 backend. This does **not** prove that a signed/external upscaler provider uses that backend implementation internally.

The provider framework explicitly constructs/checks an external DX12 provider and can select it instead of an internal provider. FSR v2.3.0 documentation further describes the upscaler DLL as containing FSR4 plus legacy FSR3/FSR2 providers. Consequently, open-backend `AddRef` behavior cannot establish COM behavior of the signed 4.1.1 provider path.

The resulting distinctions are:

- **“Open implementation calls `AddRef`”: Verified.**
- **“Public FidelityFX API guarantees a retained COM reference”: Unresolved / no such guarantee found.**
- **“Public API explicitly permits immediate release of the application's reference”: Unresolved / no such permission found.**
- **“The device must remain live/usable under the generic creation-pointer wording”: Inference from verified contract text.**
- **“Holding an independent application COM reference is sufficient to provide such object liveness”: Inference from COM semantics, not an AMD-required ownership model.**

### Context concurrency/thread safety

No general or upscaler-specific v2.3.0 statement was found guaranteeing or prohibiting concurrent operations on an upscaler context. In particular, the inspected common API/header, API guide, upscaler header, FSR4 documentation and common DX12 chain adapter do not contain a usable thread-safety contract.

Accordingly, the following are all **Unresolved as public contract**:

- simultaneous operations on one upscaler context;
- simultaneous operations on separate upscaler contexts;
- concurrent context creation or destruction;
- destruction racing dispatch, configure, or query;
- concurrent provider enumeration or provider selection;
- callback serialization or execution on worker threads;
- any required application-side mutex.

Open source provides reasons not to infer a positive guarantee. FSR2 and FSR3 dispatch mutate persistent per-context state such as `firstExecution`, `resourceFrameIndex`, and resource tables without visible per-context serialization in that path. This is a **Verified implementation fact** supporting only the narrower conclusion that the source does not supply obvious internal same-context dispatch serialization; it is not itself an AMD statement that concurrent dispatch is forbidden.

There is also shared mutable common state. The message callback/debug level are unsynchronized statics. Provider enumeration uses a static buffer for the external provider name, and the API guide independently warns that some version names reside in global memory and may be overwritten by later version queries. These are implementation/documentation facts about shared state, not a complete concurrency contract.

By contrast, v2.3.0 Frame Generation documentation contains an explicit **Thread safety** section saying its underlying context is not guaranteed thread-safe, naming operations that require external synchronization including create and destroy. This demonstrates that AMD does document effect-specific synchronization requirements when intended. It does not transfer those rules to Upscaling.

Therefore absence of equivalent Upscaling wording must remain exactly that: **absence of a documented guarantee or restriction**, not proof of either safety or unsafety.

## Contract vs implementation matrix

| Question | Public contract | Open v2.3.0 implementation | Closed/signed-provider applicability | Classification |
|---|---|---|---|---|
| Generic creation pointers | “Pointers passed in desc” remain live until context destroy. | Providers synchronously consume/copy many values. | Contract text applies, but its precise meaning for node storage is ambiguous. | **Verified text / Unresolved interpretation** |
| Every `pNext` node survives to destroy? | Not established; version override is explicit counterexample to a universal rule. | No retained chain addresses found in inspected FSR2/3+DX12 path. | Cannot infer from legacy providers. | **Unresolved generally** |
| `ffxOverrideVersion` node | Explicitly needs to survive only through `CreateContext`. | Read synchronously during provider selection. | Public tagged documentation applies. | **Verified** |
| `UpscaleVersion` node | Required in chain for FSR4 compatibility; no lifetime stated. | Legacy open provider path does not establish FSR4 handling. | Signed provider is the material consumer. | **Unresolved** |
| `BackendDX12` node storage | No node-storage-specific lifetime rule found. | Traversed synchronously; address not retained by common open DX12 wrapper. | External provider behavior not established. | **Unresolved contract / Verified open behavior** |
| `fpMessage` nullability | May be null. | Null uses debugger-output branch in common legacy message path. | Signed provider must accept null under public API. | **Verified** |
| `fpMessage` persistence | No stated lifetime bound. | Copied per-context **and** installed module-globally; destroy does not clear it. | Cannot assume same representation externally. | **Unresolved contract / Verified implementation** |
| Message callback dispatch execution | FSR4 docs say debug checker operates at upscale dispatch. | Direct dispatch → debug check → message → callback call. | Documentation describes shipped runtime behavior but not threading. | **Upstream claim + Verified legacy control flow** |
| Message callback thread/concurrency | No guarantee found. | Direct synchronous legacy path; global callback state unsynchronized. | Opaque provider may differ. | **Unresolved contract** |
| DX12 COM retention | No AddRef/ownership promise or immediate-release permission found. | First backend effect context `AddRef`s device; last releases it. | Cannot be projected onto external provider. | **Verified implementation / Unresolved public guarantee** |
| Device lifetime | Generic pointer-liveness wording is relevant. | Backend holds a COM reference while its backend refcount is nonzero. | External implementation unknown. | **Inference for application-facing liveness** |
| Same-context concurrency | No guarantee/restriction found. | Mutable state without visible dispatch serialization. | Unknown. | **Unresolved** |
| Separate-context concurrency | No guarantee/restriction found. | Some global common state exists. | Unknown. | **Unresolved** |
| Create/destroy concurrency | No upscaler rule found. | No general serialization contract visible. | Unknown. | **Unresolved** |
| Provider enumeration | No thread-safety promise found. | External version name uses shared static storage; docs warn later queries may overwrite global names. | External internals otherwise opaque. | **Verified shared-state fact / Unresolved concurrency** |

## Differences from supplied baseline

### Confirmed

- The generic `ffxCreateContext` lifetime sentence exists exactly as described.
- AMD's version-override example explicitly gives `ffxOverrideVersion` only creation-call lifetime.
- Open FSR2 and FSR3 providers copy `fpMessage` into persistent provider state.
- The open DX12 backend retains the D3D12 device with `AddRef` and releases it when its final backend effect context disappears.
- No general upscaler CPU concurrency guarantee was identified.
- The external-provider boundary is source-visible and materially prevents treating legacy open-provider details as universal behavior.

### Refined

- The conflict over descriptor lifetime is stronger than a mere ambiguous example: AMD explicitly annotates one creation-chain descriptor as call-scoped. A blanket “every `pNext` node until destroy” interpretation is therefore not sustainable.
- The open upscaler callback is not simply a per-context retained field. FSR2/FSR3 also install it into unsynchronized module-global message state.
- Destruction of an FSR2/FSR3 provider context does not visibly clear that global callback, so context destruction is not demonstrated to be the end of the open implementation's function-pointer dependency.
- DX12 device retention is one retained reference associated with a shared `BackendContext_DX12` lifetime, not necessarily one COM reference per abstract FidelityFX owner.
- The open FSR2/FSR3 dispatch callback path is synchronous/direct, but that remains an implementation fact rather than a public callback-thread contract.

### Contradicted

- Any interpretation that **all** creation-chain descriptor-node storage is contractually required to remain live through destroy is contradicted by AMD's explicit `ffxOverrideVersion` lifetime statement.
- Any interpretation that the open FSR2/FSR3 `fpMessage` dependency is necessarily confined to the lifetime of the context that supplied it is contradicted by its installation into persistent module-global state and the absence of a destroy-time clear.

No supplied conservative Rust ownership choice is contradicted here because those choices were correctly presented as wrapper policy rather than AMD contract.

### Unchanged / unresolved

- Lifetime of the concrete `ffxCreateContextDescUpscaleVersion` node after successful creation.
- Lifetime of the concrete `ffxCreateBackendDX12Desc` **node storage**, distinct from the device it references.
- COM-retention behavior of the signed FSR 4.1.1 provider.
- Calling thread, asynchronous behavior, or callback concurrency of the signed provider.
- Same-context and cross-context CPU concurrency guarantees for upscaling.
- Thread safety of provider enumeration/selection as a public guarantee.

## Unresolved questions

1. Whether AMD intends the generic creation-pointer lifetime statement to govern descriptor-node addresses except where individual descriptors override it, or instead primarily pointer-valued native dependencies.
2. Whether the signed FSR4 provider retains or rereads `ffxCreateContextDescUpscaleVersion` or backend descriptor storage after `ffxCreateContext`.
3. Whether the signed FSR4 provider independently retains the D3D12 device by COM reference.
4. Whether the signed provider ever calls `fpMessage` asynchronously, from another thread, or concurrently.
5. Whether any CPU operations on separate or identical upscaler contexts are guaranteed safe concurrently.
6. Whether the loader/external-provider implementation has synchronization not represented in the open common code.

These are unresolved because the tagged public contract does not answer them and the relevant external-provider implementation is not established by the inspected open code.

## Experiment-only follow-up

- **Descriptor retention probe:** create the signed 4.1.1 context with separately allocated descriptor nodes, then make selected node storage inaccessible after successful creation while keeping referenced resources such as the D3D12 device valid; exercise query/configure/dispatch/destroy. A resulting access would demonstrate post-create retention for that exact provider/path. No access would **not** establish a contractual call-only lifetime or prove all paths safe.
- **Device COM-retention observation:** create with the signed provider, relinquish the application's reference under a controlled COM/debug-layer instrumentation setup, then exercise and destroy the context while observing object lifetime. This could establish observed retention/liveness for that binary and environment; it could not turn that behavior into an AMD public guarantee.
- **Callback execution tracing:** provide a callback recording thread IDs, nesting/reentrancy and the surrounding API phase while exercising create/configure/query/dispatch/destroy from controlled threads. This could characterize the tested binary; it could not establish a portable contract.
- **Concurrency stress/instrumentation:** deliberately overlap same-context and separate-context operations while using sanitizing/debug instrumentation where available. A reproducible race/failure could falsify safety for the tested binary/usage; successful tests cannot prove thread safety or create a contractual guarantee.

No experiment can resolve the documentary question “what does AMD guarantee” unless accompanied by a corresponding AMD contract statement.

## Research implications

The evidence permits a later wrapper-design review to distinguish sharply between **documented liveness obligations**, **legacy open-source retention behavior**, and **opaque-provider behavior**.

Descriptor lifetime cannot be reduced to one universal rule from v2.3.0 evidence: `ffxOverrideVersion` is explicitly call-scoped, while no equivalent lifetime statement exists for `UpscaleVersion` or `BackendDX12` node storage. Retaining those nodes longer may be a wrapper policy, but cannot be described as a demonstrated AMD requirement for every node.

For `fpMessage`, null is the only configuration here with an unambiguous absence of an application function-pointer dependency. Any future non-null callback abstraction would have to account for a public contract that does not specify lifetime/threading and legacy implementations whose callback state is global rather than strictly context-owned.

For the DX12 device, the open backend's `AddRef` is useful implementation evidence but does not authorize relying on COM retention by every provider. The public evidence supports requiring continued device liveness more strongly than it supports any particular reference-ownership transfer mechanism.

For concurrency, v2.3.0 provides neither a positive upscaler thread-safety guarantee nor an explicit upscaler prohibition comparable to Frame Generation's documented mutex requirements. A later Rust `Send`/`Sync` decision can therefore be conservative wrapper policy, but the inspected sources do not justify presenting either `Send`/`Sync` capability or their absence as an AMD upscaler contract.

The strongest defensible v2.3.0 record is therefore: **descriptor-node lifetime is descriptor-specific and incompletely specified; non-null message-callback lifetime/threading is incompletely specified and legacy implementations use global state; public DX12 COM ownership is not promised despite open-backend retention; and upscaler CPU concurrency remains undocumented.**
