# D013 source record: serialized transfer and final cleanup

- Investigation/access date: **2026-10-07**, Europe/Berlin.
- Project: `fsr-sdk-rs`; proposed D013, optional `movable` feature.
- Status: **Evidence and experiment recommendations only. D013 is not accepted.**
- Requested native baseline: AMD FSR SDK/FidelityFX SDK **v2.3.0**, commit **`60f4ea81909200d8542eca14dccb2628b763a9a3`**; Windows x64/MSVC, DirectX 12, official signed deployment.

## 1. Question, method and evidence boundary

Can a runtime/context family be transferred with exclusive ownership and CPU synchronization, after all outstanding GPU work completes, such that acquisition, creation, successive dispatches, destruction and final dependency release may occur on different threads?

**Unresolved — no affirmative AMD transfer guarantee was established.** Accessible AMD documentation does not settle the selected provider's affinity, shared-state synchronization domain, hidden work, or final termination behaviour. The requested commit could not be inspected successfully in this session. This record therefore cannot claim a fresh, commit-verified implementation audit or an exhaustive absence of guarantees. It supplies a documented platform boundary and bounded experiments, with that limitation made explicit.

### Inputs supplied by the project, not independently verified here

`fsr-sdk-sys` provides faithful ABI declarations and explicit loading; `fsr-sdk` provides ownership, validation, errors and unsafe renderer dispatch. Existing `Runtime`/`Upscaler` are `!Send + !Sync` with `Rc` retention. D013 proposes separate `MovableRuntime`/`MovableUpscaler`, `Send + !Sync`, with `Arc` retention and unsafe acquisition; D014's accepted dispatch redesign is separate.

The first proposed transfer contract excludes outstanding GPU work and overlapping native calls. Coordination must also cover siblings, other runtime families, external FidelityFX users, queries/configuration, acquisition/release that touches shared state, and automatic destruction. Dispatch records commands; the caller controls resource states, residency, backing storage, submission and fences. A dispatch error poisons the context and its failed recording must not be submitted. Destruction is terminal and attempted once; a returned failure retains the full dependency bundle without retry or unloading. These are project policies, not newly discovered AMD promises.

The supplied 2026-09-22 research identified signed effect DLLs, provider-associated contexts and open FSR3/backend shared state, without establishing generic upscaler transfer permission. The controlled host/Windows fixtures passed ownership, pointer stability, sibling retention, create-error/null-success, terminal destroy-failure, unwind and runtime-only transfer checks. These results are accepted here as supplied wrapper evidence.

The supplied 2026-09-30 native run used RX 9060 XT, driver `32.0.31041.1004`, provider version `4.1.1`, ID `0xf5a5ca1e01001001`, zero creation flags, sharpening/masks disabled, and no custom callbacks/allocators. It passed three fully fenced serialized scenarios and same-thread controls: A load/create → B dispatch/fence → C destroy/final owner; three moving frames A/B/C; and two siblings with early external-runtime drop and destruction on B/C. Readbacks were finite, printed control metrics matched, ordinary D3D12 debug checks were clean, and no device removal occurred. No full pixel-equivalence claim was made. The whole family/equipment moved together, rather than separately held public siblings. The wrapper-owned loader disappeared after cleanup on C; the upscaler DLL remained in controls and transfers while the caller device remained retained. The remaining reference was unattributed.

### Source method and access log

Started with the [requested pinned tree][P0]. The tree, commit view, raw-content endpoint, GitHub API tree and attempted pinned file views failed retrieval. A terminal HTTPS source fetch also failed to connect. An alternate browser attempt was explicitly rejected by browser security with a user-denied-access reason; no further workaround was attempted. These failures are access limitations, not evidence that upstream files are absent.

| Requested pinned source | Inspection status and intended relevance |
| --- | --- |
| [`api/include/ffx_api.h`][P1] | Pinned view failed. A linked, unpinned `main` header was accessible; it is not treated as the requested commit. |
| [`api/include/ffx_api_loader.h`][P2] | Pinned raw view failed; no loader-lifetime implementation claim derived. |
| [`api/include/dx12/ffx_api_dx12.h`][P3] | Not inspected at the pin; an attempted `main` view also failed. |
| [`api/internal/ffx_api.cpp`][P4] | Short-commit and `main` attempts failed; no new routing/cache implementation finding. |
| [`api/internal/ffx_provider.h`][P5] | Sought within the inaccessible tree; contents not inspected. |
| [`upscalers/include/ffx_upscale.h`][P6] | Sought within the inaccessible tree; contents not inspected. |
| [`upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp`][P7] | Contents not inspected; the supplied earlier observations remain baseline only. |
| [`backend/dx12/`][P8] | Contents not inspected; no new TLS/device-reference/global-state audit. |
| [`docs/getting-started/ffx-api.md`][P9] | Pinned raw/blob retrieval failed. Published AMD manuals were examined separately. |

Read the accessible AMD manual identifying itself as SDK **2.3.0**, its API introduction, Upscaling **4.1.1**, FSR3 upscaler **3.1.5**, and licence; inspected Microsoft Learn API/architecture pages and official Rust/dependency documentation. Targeted searches included thread/affinity/TLS/unload/cache in the API manual, thread/affinity/asynchronous/lifetime in the 4.1.1 manual, and thread/destroy plus context/history sections in the 3.1.5 manual. No matching thread rule was found in those three AMD pages. They cover integration and API behaviour, not a comprehensive concurrency/lifecycle specification. See [AMD API][A1], [4.1.1 integration][A2] and [3.1.5 integration][A3].

Published manuals and `main` are **not byte-verified against the pin**. No newer release is used to establish v2.3.0 guarantees. Microsoft pages are live Windows contracts; dates are recorded below. Rust standard-library pages exposed version **1.99.0**, commit **`b940084d7`**, dated **2026-09-28**. Although explicit docs.rs version URLs failed, the retrieved `libloading/latest` page identified **0.9.0**, and Microsoft's binding page identified **windows 0.62.2**. No SDK, wrapper or fixture was executed; no API was implemented.

Excluded: image-quality research, new effects, concurrency stress with overlapping native calls, SDK upgrades, forced module unload, reference-count manipulation, invalid resources/lifetimes, allocation fault injection into AMD, and broad licensing research. Only a licensing restriction affecting inspection methods is noted below.

### Label meanings

| Label | Meaning in this record |
| --- | --- |
| **Verified source fact** | Directly observable content/behaviour in an accessible primary source; source inspection, not runtime verification. |
| **Upstream claim** | An upstream documented assertion; its scope/version remains attached. |
| **Inference** | Reasoned implication or recommended policy, not an upstream guarantee. |
| **Unresolved** | A contractual or attribution question the available evidence does not answer. |
| **Requires experiment** | A question suitable for bounded observation; passing does not create a native contract. |

## 2. AMD transfer and provider boundary

### What the accessible documentation actually establishes

**Upstream claim:** The loader manages effect DLL loading and contains no effect code; the upscaler DLL supplies ML and legacy upscalers. Context creation starts with a null slot; destruction is described as leaving it null. GPU dispatch encodes commands into the supplied list. Non-OK results are errors, possibly unrecoverable. Version overrides must use queried IDs rather than hard-coded values. Compatible allocation callbacks are required between creation and destruction. These statements come from the published API manual, not a newly inspected pinned implementation. [API: DLLs structure, Context creation/destruction, Dispatch, Version selection, Error handling, Memory allocation][A1].

**Upstream claim:** AMD's 2.3 product page says driver releases can update ML implementations used by default and requires prebuilt signed API DLLs. **Inference:** SDK commit, provider version string and provider ID do not alone freeze all executed implementation/dependency code. Record actual deployment files and driver/runtime identity in every native result. [Product page: Automatic updates; General overview][A4].

**Upstream claim:** Upscaling 4.1.1 requires the signed API distribution and refers to FSR3 integration guidance; its temporal reset guidance concerns camera discontinuities. **Inference:** A CPU transfer is not a documented reset condition. The history test should preserve the same context and continuous input history across handoffs. [4.1.1: Integration guidelines; Camera jump cuts][A2]. FSR3 documents persistent intermediate/history surfaces, which explains why reconstructing a fresh context would test a different question. [3.1.5: Memory requirements][A3].

### Serialized calls are not the same proposition as thread affinity

**Unresolved:** No inspected AMD source affirmatively permits A loading, B creating, C dispatching, D destroying, or D releasing the last loader/provider dependencies. No inspected page affirmatively requires all those operations to stay on one thread either. Neither omission is permission. A requirement to avoid overlapping calls can coexist with creation-thread affinity, per-thread initialization, thread-exit cleanup or asynchronous internal work.

**Unresolved:** The selected signed provider's TLS use, creation-thread requirements, callbacks, lazy initialization, CPU workers, internal GPU work, and cache shutdown were not exposed by this investigation. Returning from a command-recording call and completing the application's fences are different observations; neither proves the absence of hidden provider activity. The API manual's statement about usual synchronous CPU dispatches is not a universal shutdown guarantee. [API: Dispatch][A1].

**Inference:** Serialization must be an enduring invariant, including cleanup; an unsafe constructor cannot itself enforce it. `!Sync` stops sharing one object's references through safe Rust, but separately owned siblings/families may still run on separate threads. `Arc` preserves ownership, not provider synchronization. The open FSR3 observations supplied by the project are reasons to investigate a wider coordination domain, not evidence of the selected 4.1.1 implementation's behaviour.

**Unresolved:** Public material establishes the signed deployment architecture and driver/external routing concept, not a complete ownership/affinity contract for provider `0xf5a5ca1e01001001`. Its exact implementation origin and the remaining upscaler module reference are unattributed. Do not identify that provider as open FSR3, or assume an ID is a stable compatibility whitelist.

## 3. Loader, provider and device lifetimes

| Relationship | Evidence and conclusion |
| --- | --- |
| Wrapper → loader | Project baseline: the wrapper owns a load reference. **Verified source fact:** Windows `FreeLibrary` operates on process module references; an ordinary library release need not remove a module with other references. [FreeLibrary: Remarks][M1]. |
| Loader → upscaler | **Upstream claim:** documented loading-management relationship above. **Unresolved:** effect-DLL reference acquisition/release, caching and pinning policies are not established by accessible loader implementation. |
| Context → provider | Project baseline: a selected provider is associated with the context. **Unresolved:** successful destruction's complete provider/dependency shutdown and error-state ownership. |
| Provider/cache → device | **Unresolved:** no inspected AMD contract says whether this provider retains the device, is retained by device-associated state, or caches a module at process/device scope. These directions must not be conflated. |
| Caller/equipment → device and children | **Upstream claim:** D3D12 interface reference counting does not manage all resource/description backing-memory lifetimes. **Inference:** inventory actual ownership rather than assuming child references alone prove the desired device lifetime. [Important changes: Explicit synchronization][M2]. |

### Explanations for continued upscaler residency

**Inference — competing hypotheses, not attribution:** an additional AMD/driver load reference; device-associated provider objects/caches; process-lifetime caching or pinning; another FidelityFX user; a caller/debug/helper-owned reference; an ordinary leaked reference; or a delayed release. A device retained by a provider and a provider retained by device private data are different possible ownership graphs. Persistence in both control and transfer does not support a transfer-specific cause.

**Verified source fact:** `ID3D12Object::SetPrivateDataInterface` increments an associated interface's reference and decrements it when the association is replaced or the owning object is destroyed. This supplies a concrete Windows mechanism for one device/object-associated retention hypothesis; it does **not** show AMD uses it. Do not modify unknown private-data associations to test that hypothesis. [SetPrivateDataInterface: pData][M3].

**Verified source fact:** `D3D12CreateDevice` normally returns an existing process device for the same adapter. **Inference:** two calls called “independent devices” may share identity; use fresh processes to isolate cases, and record adapter LUID/device identity. This is also a reason not to assume separate runtime families isolate provider state. [D3D12CreateDevice: Remarks][M4].

### What disappearance can establish

**Upstream claim:** On ordinary reference-count-zero unloading, Windows invokes a module's entry point, if present, for process detach before unmapping it. The entry-point invocation runs in the triggering thread's context; attach and detach need not occur on the same thread. [FreeLibrary: Remarks][M1]; [Entry-point function: Calling the entry-point function][M5].

**Inference:** Accurate observations of disappearance while the process stays alive establish that the particular executable module mapping ceased to be present during the observation interval. Correlation with a destination-thread final release strengthens attribution, but a polling absence alone does not identify the triggering thread, prove a specific termination callback existed, or show it completed all logical cleanup. A module can unload while resources remain leaked elsewhere, and separate dependencies can stay resident. Conversely, a resident module may have already destroyed every effect context. Residency and effect cleanup are distinct outcomes.

**Unresolved:** Complete provider termination on C is not established by the supplied loader disappearance. Even future upscaler disappearance would not by itself prove all driver caches, worker activity, device references or dependency cleanup happened on C. Callback/thread attribution requires independent evidence with known observation semantics or an upstream statement.

**Upstream claim:** Process termination is a different detach path; `DllMain` receives a different reserved argument and cleanup may be unsafe after other threads terminate. Forced process/thread termination can omit relevant notifications. **Inference:** module disappearance after child-process exit is excluded as final-cleanup evidence. Observe the child while it remains alive after all ordinary releases. [DllMain: lpvReserved; Remarks][M6].

### Sound natural-residency observation plan

**Requires experiment:** Use an isolated x64 child for the AMD work and an x64 observer process. Establish modules before acquisition, after creation, after fenced work, after final context destruction/runtime release, after equipment release, after final caller device/interface releases, and after ordinary worker-thread exits. Keep the child alive at the last checkpoints without reacquiring AMD or D3D12 objects. The observer holds a process handle and copies module metadata, not an AMD library owner.

**Verified source fact:** `EnumProcessModulesEx` returns snapshot-derived module handles with no resources to free; changing module lists can yield failure/inaccuracy and buffer sizing must be checked. Toolhelp module snapshots have architecture/race limitations and retryable `ERROR_BAD_LENGTH`; close the snapshot handle, not the returned module addresses. [EnumProcessModulesEx: Remarks][M7]; [CreateToolhelp32Snapshot: Remarks][M8].

**Inference:** Prefer external enumeration with repeated, timestamped, bounded observations; log enumeration errors instead of turning them into “module absent.” Record path/base/size to distinguish reloads and same-basename copies. Archive file version, hash and signature metadata from files before cleanup, rather than calling into released code. Polling can miss a transient unload/reload; a bounded continued presence means only “still present at the recorded checkpoints,” not “permanent leak.”

**Verified source fact:** `GetModuleHandleEx` ordinarily adds a reference; its `PIN` option prevents ordinary unload until process exit. `UNCHANGED_REFCOUNT` avoids retention but its handle can become stale/reused immediately. **Inference:** do not obtain retaining/pinned observation handles, free observational handles, dereference stale module pointers or invoke old exports. [GetModuleHandleEx: flags; Remarks][M9].

**Inference:** Audit cloned COM pointers, `GetDevice` results, debug interfaces, error objects, captured closures, thread-local caches, maps and readback/equipment ownership. `GetDevice` explicitly adds a reference to its returned interface. Release owned debug/device interfaces while the device is valid, before the claimed final caller release. Helpers, capture tools, injected layers and extra library owners can change residency; compare the same instrumentation in both arms and include a minimal-observer run. [GetDevice: Remarks][M10]. Do not probe COM reference counts with extra AddRef/Release calls, call `FreeLibrary` on somebody else's reference, clear unknown caches, or force unload.

## 4. Microsoft D3D12, COM and loader requirements

### Interface-specific boundary

The supplied baseline does not enumerate every actual interface/version. The following inventory covers the normal device, resources, queue, command list, allocator and fence apparatus; heaps, descriptors, PSOs and root signatures must be included if retained. This does not introduce swapchain, DXGI presentation, D3D11-on-12 or apartment-bound helper permissions.

| Interface/operation | Source-supported requirement; affinity distinction |
| --- | --- |
| `ID3D12GraphicsCommandList`, `ID3D12CommandAllocator` | **Upstream claim:** the official multithreading sample forbids concurrent calls on the same list or allocator. One allocator has at most one currently recording list. This is exclusion of simultaneous use, not a documented creation-thread pin. [Multithreading sample][M11]; [Recording: ID3D12CommandAllocator][M12]. |
| Allocator `Reset` | **Upstream claim:** GPU execution using its recorded commands must have finished; resetting during execution is undefined behaviour. Do not interpret the recording overview's suggestion of failure as reliable runtime validation: the specific method contract says UB. [Reset: Remarks][M13]. |
| Command-list `Reset` | **Upstream claim:** it may be reset while an earlier execution continues, unlike allocator reset. **Inference:** D013's fully completed-work rule is stricter and remains the experiment rule. [Recording: Recording command lists][M12]. |
| `ID3D12CommandQueue::ExecuteCommandLists` | **Upstream claim:** submission from multiple threads is permitted and serialized by the runtime. **Inference:** caller ordering/fence ownership still matters; this permission does not cover AMD calls. [Executing: Executing command lists][M14]. |
| `ID3D12Fence` | **Upstream claim:** `SetEventOnCompletion` is safe from multiple threads. `GetCompletedValue == UINT64_MAX` indicates device removal. **Inference:** do not count that value as successful fence completion. [SetEventOnCompletion: Remarks][M15]; [GetCompletedValue: Return value][M16]. |
| `ID3D12Device`, resources/heaps/descriptor storage | **Upstream claim:** D3D12 uses explicit CPU/GPU synchronization and caller-controlled memory lifetimes/residency. **Unresolved:** the inspected device/reference pages do not themselves supply a blanket final-release-on-any-thread/apartment assertion for every retained interface. [Important changes][M2]; [ID3D12Device][M17]. |
| Owned COM interface final `Release` | **Upstream claim:** the last reference requires freeing the object; the returned count is for testing, not ownership discovery. **Unresolved:** generic `IUnknown::Release` is not an affinity or marshaling waiver. **Inference:** a final release may execute implementation cleanup; never use the pointer afterward. [IUnknown::Release: Return value; Remarks][M18]. |

**Inference:** Documented D3D12 multithreading supports a serialized recording/queue model; it does not establish an arbitrary-thread lifetime contract for AMD. No creation-thread requirement was found in the cited recording/queue/fence pages, but that bounded finding must not become “all native objects are freely movable.” GPU resources, descriptor backing memory and allocators must survive every submitted use; barriers are not CPU ownership synchronization or completion fences.

### COM initialization and marshaling

**Upstream claim:** Microsoft's DirectX/COM guide uses `D3D12CreateDevice` as its indirect-creation example and says indirect creation methods handle initialization; explicit `CoCreateInstance` requires explicit COM initialization. This does not demonstrate that `D3D12CreateDevice` initializes the caller's apartment or authorizes every auxiliary COM object. [Programming DirectX with COM: Creating a COM object][M19].

**Upstream claim:** General COM apartment rules require marshaling between apartments unless the component's relevant contract provides direct access; threads within the MTA can call MTA objects directly. `CoInitializeEx` is per-thread, each successful call including `S_FALSE` must be balanced on that thread, and an incompatible model returns `RPC_E_CHANGED_MODE`. [Apartments: The Apartment and COM Threading Architecture][M20]; [CoInitializeEx: Remarks][M21].

**Unresolved:** No inspected primary page classifies every retained D3D12 interface here as an apartment-bound COM server or gives a comprehensive all-apartments marshaling exemption. Do not invent a requirement to marshal the AMD opaque context, or a claim that generic COM rules impose STA affinity on the documented D3D12 recording model. Likewise, binding traits are not proof of either classification.

**Inference:** The spike should list all interfaces/helpers and record each worker's COM setup. If the harness uses COM-library services, initialize dedicated workers consistently as MTA, handle initialization errors, and balance explicit initialization only after their owned cleanup. This is a controlled harness choice, not evidence that AMD requires MTA or that initialization resolves provider affinity. It does not cover arbitrary STA destinations or unknown apartment-bound callbacks. Avoid carrying a thread's COM initialization guard to another thread.

### DLL lifecycle and loader lock

**Upstream claim:** `DllMain` runs under the process loader lock with serialized entry-point access. Thread attach affects newly created threads; a dynamically loaded DLL gets no attach notifications for already existing threads. Thread detach normally occurs on the exiting thread; ordinary module unload sends process detach without individual thread-detach calls. These mechanisms allow TLS behaviour but do not prove this AMD provider uses it. [DllMain: fdwReason; Remarks][M6].

**Upstream claim:** Microsoft's DLL guidance forbids loader-lock reentry, unsafe interthread waits/lock inversion and COM initialization from `DllMain`. Restrictions also affect initialization/termination code reached there. **Inference:** execute application acquisition/destruction/release outside DLL entry points and loader-notification callbacks. A wrapper coordination lock must have a reviewed lock order; do not assume holding it across final release is harmless if native detach can acquire another lock. [DLL best practices: General best practices; Deadlocks caused by lock order inversion][M22].

**Unresolved:** Serializing exported AMD calls does not suppress Windows thread notifications or internal workers. The contract needs to define what is coordinated and what upstream must handle itself; application code cannot put all system loader callbacks under its private gate.

## 5. Errors, partial acquisition and Rust unwind

**Verified source fact:** `libloading` 0.9.0's loading safety documentation assigns the caller responsibility for foreign initialization and termination routines. The retrieved Windows implementation calls `FreeLibrary` in `Drop`, ignores its return there, and exposes an error-returning `close` that avoids a second attempt. These concern library ownership, not AMD context-destroy semantics. [Library::new: Safety][R1]; [Windows source: close/Drop, source lines 294–327][R2]. **Inference:** automatic library drop completion does not prove unload succeeded or all dependencies terminated.

**Verified source fact:** `Arc` does not make its contents thread-safe and drops the inner value when the last owner disappears. Rust destructors recursively drop fields after the custom destructor. Mutex poisoning is advisory and cannot be the safety invariant. [Arc: Thread safety][R3]; [Reference: Destructors][R4]; [Mutex: Poisoning][R5]. **Inference:** preserving the bundle on destroy failure must prevent subsequent field destruction too; retaining only the numeric context is insufficient. Coordinated automatic `Drop` must neither skip required cleanup solely on mutex poison nor panic again during unwind.

**Verified source fact:** Rust's FFI rules distinguish unwind-capable boundaries and aborting/non-unwinding behaviour. [Rustonomicon: FFI and unwinding][R6]. **Inference:** use ordinary `panic=unwind`, initiate the planned panic in Rust after known completion and after native calls return, and keep every foreign call non-unwinding. A destructor can call native cleanup while a Rust unwind is in progress without sending that unwind through a native frame. `panic=abort`/forced termination does not exercise this path.

| Path | Additional obligation or gap after transfer |
| --- | --- |
| Creation returns an error | **Inference:** do not publish a live upscaler. Inspect the returned slot for diagnostics, but a nonnull slot on error is not automatically a valid destroyable context. **Unresolved:** partial allocation rollback and ownership for the selected provider. Require an upstream postcondition before calling destroy on an ambiguous error output; quarantine uncertain dependencies under a reviewed policy rather than guessing. Existing fixture success is not native rollback evidence. |
| Dispatch errors after earlier successful fenced work | Project policy: poison; never submit the failed recording or continue history through another dispatch. **Inference:** fence earlier submissions before terminal cleanup, discard the whole failed recording and preserve sibling coordination. **Unresolved:** selected-provider error-state destruction and internal partial-recording effects. |
| Rust panic on destination | Project scope: outside FFI, GPU already complete. **Inference:** cleanup on that thread must enter/preserve the common coordination domain with reviewed guard/drop order; no recursive lock deadlock or second panic. Native compatibility remains unverified. |
| Destroy returns an error | Project policy: terminal attempt exactly once; retain complete dependencies, regardless of whether the native slot changed or became null. **Inference:** do not infer rollback, retry permission or unload permission from a return code/slot. **Unresolved:** provider-internal cleanup postcondition and any internal background work; retained ownership cannot stop work internal to the provider. |
| Load fails before an owner is obtained | **Inference:** report acquisition failure; there is no movable runtime object. Windows may have run initialization/rollback while loading. Do not manufacture a cross-thread partial-load scenario. [DllMain: Return value][M6]. |
| DLL loads but a required export is missing | **Verified source fact:** name-based `GetProcAddress` returns null on failure. **Inference:** preserve the error and release the acquisition-local owned library normally, on the acquisition thread, after invalidating provisional function pointers. This is partial acquisition, not runtime transfer. [GetProcAddress: Return value][M23]; [Windows Library Drop][R2]. |

### A valid returned-native-error experiment?

**Unresolved — none established for the selected profile.** Enumerated error codes are not recipes for otherwise-valid failure. The published API manual documents `NO_PROVIDER` for a null-context query lacking required device information; violating that requirement is not an eligible trigger, and a query error would not test context dispatch/destroy recovery anyway. [API: Query][A1]. Public allocator callbacks are described, but this record neither verifies provider-specific failure propagation nor proposes changing the no-custom-allocator profile to inject errors.

**Inference:** Use controlled fixtures for returned create/dispatch/destroy/export-resolution failures. Record the gap as “no selected-provider returned-error cleanup observation,” even if all fixtures pass. A naturally occurring error during an otherwise-valid native run may be recorded with its exact setup and diagnostics; it is not a planned deterministic trigger. Do not create errors by invalid resources, unsupported descriptor combinations, dangling pointers, races, device removal or unsafe allocation hooks.

## 6. Contract matrix

This matrix evaluates the proposed first contract; it is not a permission table. “Same thread” controls still require valid resource/lifetime usage.

| Operation | Proposed cross-thread case | Platform/source boundary | Selected AMD provider result | What would add evidence |
| --- | --- | --- | --- | --- |
| Acquisition | Load on A; later ownership moves | Foreign initialization/termination obligations apply [R1]; no transfer exists before success | **Unresolved:** provider/loader lifecycle compatibility | Runtime-only transfer fixtures already supplied; native creation after transfer N2 is new |
| Creation | Successful runtime A → B; create on B | Device/interface contract and serialized CPU ownership; no blanket provider implication | **Unresolved:** context creation away from loader/device acquisition threads | N2, with A exiting ordinarily as a separate bounded variant |
| Dispatch | Context history A → B → C repeatedly | No concurrent list/allocator calls [M11]; explicit completion/ordering | **Unresolved:** serialized provider compatibility, including temporal state and hidden work | N1 extends prior three frames; failure policy F1 remains fixture-only |
| Explicit destroy | Context created on B, destroyed on D | Earlier work complete; owned dependencies still valid | **Unresolved:** destination destruction and returned-error postconditions | Four-stage lifecycle N2; F2/F3 test wrapper policy |
| Automatic `Drop` | Last context owner falls out of scope or unwinds on D | Rust field/drop order and coordination apply [R4] | **Unresolved:** ordinary native unwind cleanup; previous normal sibling Drop is supplied evidence | N3 separately held siblings; N4 destination panic |
| Final module release | Last runtime/dependency ownership released on D | [FreeLibrary][M1] and [loader-lock requirements][M22] | **Unresolved:** last provider reference, caches and termination thread | N5 after final equipment/device release; F3 retention |
| Failure cleanup | Creation/dispatch/destroy failure after a completed runtime transfer | No invalid-handle inference; fixture-defined ownership postconditions | **Unresolved:** no valid deterministic AMD error trigger | F1–F3, F6; native gap remains |
| Missing-export cleanup | Acquisition fails on B after DLL load | Acquisition-local release, not incomplete-runtime transfer [M23] | **Unresolved:** AMD partial initialization rollback; no transfer proposed | F4 is new acquisition/error coverage |

## 7. Spike 1 — controlled fixtures

**Requires experiment:** Recommendations only; none conducted here. Extend the existing controlled host and Windows DLL fixtures. Run returned-error cases separately from signed AMD tests. Use non-panicking event instrumentation to record operation, logical context/family, native slot, dependency owner, CPU thread ID and coordinator entry/exit. A controlled fixture can specify ownership and slot postconditions that AMD has not specified. Fixture DLLs are substitutes for wrapper-policy tests, not signed provider compatibility tests.

Common pass requirement: event traces have no overlapping foreign operations in the nominated common domain, exactly the required destroy attempts, and dependency destruction only at allowed checkpoints. Timeouts, unexpected retries, uncoordinated native calls, dangling pointers or premature owner release fail the case. Do not depend on a fixture's log DLL staying loaded; copy events into harness-owned storage before ordinary unload.

| Case and question | Control/setup | Observations and success/failure | Added evidence and limits |
| --- | --- | --- | --- |
| **F1: transferred dispatch poison.** Does an error on B stop all further dispatches after earlier completed work on A? | Same scripted fixture and earlier-success sequence on one thread. Transfer only a completed, initialized context; fixture returns a configured error on the next dispatch. | Observe one failing dispatch, durable wrapper poison across a further completed CPU handoff, no later foreign dispatch/query that policy prohibits, no submission of the failed recording, one terminal cleanup attempt under coordination. Any continued dispatch/submission or forgotten poison fails. | New transfer + error-state coverage. A host trace cannot establish real GPU completion or AMD's partial-recording state. |
| **F2: terminal destroy.** Is a returned failure terminal even if the slot changes? | Same-thread explicit-destroy failure; scripted variants leave the slot unchanged, mutate it, or null it. Include a successful destroy control. | Explicit failure consumes live-operation authority; later `Drop`/unwind attempts no second destroy. Failure bundles remain intact; success releases in the declared order. Retry or slot-based accidental release fails. | Adds transferred explicit-failure/Drop interaction beyond prior destroy-failure checks; slot variants are fixture contracts, not claimed AMD behaviour. |
| **F3: full retained bundle.** Can the final owner disappear without unloading dependencies after destroy failure? | Same-thread failure; transfer failure; successful final-release control. Include sibling and early external-runtime-owner release. | Track loader, function table, device owner, context storage and every wrapper-retained backend/callback dependency. No bundle member deallocates on the failed terminal path, even after remaining ordinary owners drop. No retry or ordinary unload follows. | New cross-thread final-owner and field-drop evidence. Intentional retention is a policy outcome, not proof of native safe termination. |
| **F4: missing required export.** Does unsuccessful acquisition release only what it acquired? | Complete-export DLL success control; missing-export DLL fails acquisition on B. Start with a fresh process to avoid another owner's module reference. | Log initialization, export-resolution failure, no published runtime, no context call and acquisition-local ordinary release on B. A successful runtime value, use of provisional exports, wrong-thread cleanup or leaked owned load reference fails. Observe natural fixture detach, not forced unload. | New partial-acquisition coverage. It tests no transfer before `load` returns; simple fixture detach says nothing about AMD initialization rollback. |
| **F5: coordinated unwind.** Does destination-thread panic retain exclusivity through cleanup? | Same-thread unwind, transferred nonpanic Drop, and transferred unwind. No fixture/native frame active at panic. A sibling worker waits at the common gate; it does not enter native code until authorized. | Non-panicking traces must show terminal cleanup under the gate, reviewed guard/drop order, no recursive lock deadlock, no second panic and no sibling entry during cleanup. Check both a clean gate and a previously poisoned gate under a defined invariant-preserving policy. | Adds transfer + lifetime coordination to prior unwind checks. Mutex-poison recovery alone is not a proof that native state is valid. |
| **F6: creation output/error ownership.** Does creation failure after runtime transfer leave the wrapper unpublished and follow known cleanup postconditions? | Same fixture script on one thread; transferred runtime creates on B. Cover error/null, fixture-owned partial allocation, and success/null. | No upscaler published on error or success/null; dependency owners remain until the fixture's specified rollback/quarantine outcome. No destroy on an output the fixture marks invalid. All legitimate fixture rollback occurs exactly once. | Extends existing create-error/null-success coverage to creation after transfer. AMD's ambiguous error output must remain a separate gap; do not import fixture cleanup rules into native calls. |

**Inference:** F2/F3 must verify retaining the bundle through normal return and unwind, not just omitting an explicit `FreeLibrary`. F5 must make the synchronization domain outlive every value whose destructor needs it. Repeat its waiting-worker schedule with independently acquired fixture runtime families and a separately owned fixture client participating in the same gate; this checks the harness's common-domain policy, not compliance by unknown real external users. Record fixture DLL detach with a minimal, loader-safe event mechanism rather than reentering Rust logging or the application gate from `DllMain`. A failure during cleanup must not let field destructors silently release retained dependencies. These checks concretize the proposed policy without choosing a public API.

## 8. Spike 2 — native transfer and natural cleanup

**Requires experiment:** Preserve the supplied native profile: official signed v2.3.0 deployment, RX 9060 XT/recorded driver where available, no custom allocators/callbacks, zero creation flags, sharpening/masks disabled. Record deviations; do not label a different provider/profile a replication. Use only ordinary valid operation sequences and no overlapping native calls. Keep all relevant GPU work completed before **every** handoff, destruction or panic. A CPU channel/gate handoff must also establish ownership/visibility; a GPU fence alone is not that CPU handoff.

Common observations: runtime/provider query results while valid, adapter LUID, exact signed file identity, D3D12 runtime/OS/driver versions, interfaces retained, thread start/exit and operation IDs, context identity, fence targets/completed values, submission order, HRESULT/FFX returns, debug messages, device-removed reason before device release, deterministic readback metrics, dependency-release checkpoints and module snapshots. Obtain metadata before final release. No queries/callbacks into released contexts/modules afterward.

Common failures: native error/crash/hang, device removal, ownership/coordinator violation, incomplete fence, unexpected provider change, or unexplained divergence from predefined control tolerances. An ordinary debug check passing or finite pixels is necessary evidence for this harness, not complete validation. Predeclare a small fixed workload, repetitions and timeouts; for example 60 consecutive frames per context over three dedicated workers, with three isolated repetitions per arm. This is bounded repetition, not broad concurrency stress.

| Case and question | Matched control | Required observations and outcome | Added evidence and limits |
| --- | --- | --- | --- |
| **N1: repeated history-preserving handoffs.** Does one context continue normally across more than three fenced moving frames? | Same deterministic frame inputs/order, same context lifetime, same repetition count on one thread. Reset only for the same initialization/camera discontinuities in both arms. | Rotate exclusive ownership A/B/C after each completed frame; log context identity, frames, gate and fences, readback metrics/debug results. Pass means valid normal operation and predefined control agreement; drift/errors/hangs fail. | Extends previous three-frame evidence to bounded history/repetition. Do not claim full pixel equivalence unless actually compared; no arbitrary-thread or `Send` proof. |
| **N2: create after runtime transfer; four-stage lifecycle.** Is acquisition distinct from context-creation thread? | Load/create/dispatch/destroy/release on A with identical device acquisition and inputs. Transfer arm loads on A, creates on B, dispatches/fences on C, explicitly destroys and releases final runtime on D. | Trace the actual thread for every stage and ensure no context exists before B. Keep device/equipment valid, and follow N5 release checkpoints. A small separate arm lets A exit normally before B's creation; compare with A kept alive. | Newly tests load A/create B, the requested four threads, and source-thread exit beyond prior runtime-only wrapper checks. Thread-exit success is bounded evidence; no TLS absence claim. |
| **N3: separately held siblings under one domain.** Does independent ownership preserve common coordination and retention? | Two independently held siblings with the same sequential operation/destruction schedule on one thread. Transfer arm places one on B and the other on C, without moving a whole family container. | A shared test coordinator authorizes one native operation at a time, including Drop and final runtime release; no guard is transferred between threads. Drop external runtime early, destroy sibling 1 on B, then dispatch/fence sibling 2 and finally Drop it on C. Pass includes valid surviving sibling and traced final-owner ordering; interleaving native calls or premature unload fails. | Directly fills the supplied experiment's independently held sibling gap. Coordinator validity is a premise, not established by `!Sync` or this success. |
| **N4: destination-thread panic cleanup.** Does ordinary unwind reach native destruction after transfer? | Same-thread panic after identical completed work; transferred ordinary nonpanic Drop. Catch the Rust unwind outside the scope owning the context so that it really drops during unwind. | Panic on destination after all fences and FFI returns; context/runtime owners must be inside the unwind scope, equipment/device outside and alive. Log destroy attempt/result, gate, thread and later natural releases. No foreign unwind, retry, second panic or hang. A returned destroy failure enters retention policy and is reported as failure to establish normal cleanup. | New native unwind-path evidence. It does not test dispatch/create errors or panic while GPU work is pending. |
| **N5: staged natural residency after all caller releases.** Does upscaler persistence depend on retained equipment/device or thread exit? | Fresh same-thread child with identical release order and observer. Also establish a no-FSR D3D12 child to characterize platform/helper modules. | After final contexts/runtime, release equipment/auxiliary COM refs, then final caller device refs, then ordinary workers; keep child alive for immediate and bounded later snapshots, e.g. 100 ms, 1 s, 5 s. Record loader/upscaler/other relevant modules at every checkpoint and any natural unload/reload observations. | Goes beyond retained-device baseline. Disappearance is a residency observation; continued residency is unresolved attribution, not automatically a leak or transfer failure. Process exit is not a success criterion. |

**Inference:** N5 has separate outcomes: successful valid lifecycle, loader disappearance, upscaler disappearance, continued residency, and independently evidenced termination-thread attribution. Do not merge them into one “cleanup passed.” Keeping C/D alive versus letting them exit ordinarily is a bounded thread-exit/residency comparison; no `TerminateThread`, forced unload or fresh device reacquisition at the last checkpoint.

**Unresolved:** N3 covers one family. Other runtime families and external users remain a contractual coordination obligation, even if absent from the isolated child. A single shared test gate may be used across separately acquired families for a bounded follow-up, but success cannot show unknown external users obey it or establish that no hidden state crosses domains. A per-family lock is insufficient evidence of process-wide coordination.

**Unresolved:** There is no documented complete-provider-termination criterion in the inspected sources. If the upscaler remains after all controlled caller references disappear, retain the timeline and report attribution unresolved. If it disappears, report that narrower result and the known release thread; do not invent a provider termination callback or its thread. The proposed spike contains no deterministic selected-provider error injection.

**Upstream claim:** AMD's published SDK licence restricts reverse engineering, decompilation and disassembly for the default binary distribution while listing source exceptions. **Inference:** these spikes use public API calls and external module observations, not binary reverse engineering; instrument controlled fixture code instead of modifying signed AMD binaries. The published licence is not independently matched to the pinned release package here. [Licence: opening distribution terms][A5].

## 9. Baseline delta and decisions still required

| Classification | Delta from supplied baseline |
| --- | --- |
| Confirmed in accessible published sources | Signed loader/effect architecture; recording vs submission distinction; relevant explicit D3D12 synchronization requirements. Exact pinned implementation remains unverified. |
| Newly established for this record | Device-associated interface retention is a documented possible mechanism, not AMD attribution; normally shared per-adapter D3D12 device identity; observation APIs can retain/pin modules; loader attach/detach threads may differ; partial export-resolution failure is acquisition-local; dependency loading safety includes termination routines. |
| Contrary to stronger interpretations | A retained device does not identify the upscaler's remaining reference. Loader disappearance does not establish selected-provider termination on C. `Send + !Sync` on individually owned siblings and `Arc` retention do not enforce family/process coordination. Allocator reset during GPU execution is UB, not a dependable returned-error experiment. |
| Not contradicted by accessible evidence | Supplied three native successes and fixture results; the bounded first-contract exclusion of outstanding GPU work. No new positive native movability guarantee and no explicit AMD same-thread rule were established. |
| Still unresolved | Selected provider's serialized lifecycle contract, hidden work/TLS/thread-exit behaviour, provider caches/device references, native error/unwind cleanup, final dependency termination and its thread, coordination with external users, and requested pinned implementation inspection. |

**Inference — maintainer decisions required before any D013 acceptance:**

1. **Evidence threshold and scope.** Decide whether a supported AMD statement is required, and what explicitly documented unsafe preconditions would make a narrower experimental profile supportable. Successful repetitions cannot establish universal movability. Treat driver/provider auto-updates as a compatibility-scope question, not as evidence that the examined binary is stable forever.
2. **Coordination that survives safe operations.** Specify the common domain, how siblings/local families/external users participate, and how automatic Drop/final release remain coordinated during normal return and unwind. Acquisition-time promises must remain true for the entire lifetime. Do not expose safe independently transferable owners whose implicit cleanup can violate the promised exclusion.
3. **Cleanup boundary.** Decide whether the contract promises only wrapper-owned references/calls or provider/dependency termination, and what evidence supports the latter. No termination-on-destination claim is supportable from the supplied residency result. Define ordinary thread-exit and hidden native callbacks as explicit unresolved boundaries.
4. **Failure ownership.** Approve terminal no-retry retention and its complete bundle, how creation-error ambiguity is handled, and non-panicking cleanup/poison/lock-order policy. An ambiguous native error slot must not silently become destroy authority.
5. **Platform interface inventory.** Identify actual D3D12 versions and every helper/callback/COM object retained. Resolve any required apartment access or marshaling separately; the published Rust binding traits do not establish AMD compatibility. [windows 0.62.2 ID3D12Device traits][R7].

**Inference — concrete reasons to revise the proposed contract:** replace any “provider cleanup occurs on the destination thread” wording with the narrower verified ownership/call boundary until attribution exists; require lasting common coordination covering implicit cleanup rather than relying on `Arc`, `!Sync` or an unsafe constructor; and avoid claiming a source-verified v2.3.0 transfer guarantee from this access-limited investigation. If the required coordination or cleanup cannot be made enforceable within the chosen unsafe preconditions, retain the local types and leave D013 pending. This record makes no API selection and does not accept the design.

## 10. Source register

All sources were accessed or attempted on **2026-10-07**. AMD `fsr_sdk` pages identify SDK **2.3.0**; their exact Git commit is not exposed/verified here. The 4.1.1 manual's version history dates it **2026-06-24**. The older `fidelityfx_sdk2` API manual identifies SDK **2.0.0** and was used only as a historical lead. No historical or unpinned page substitutes for pinned inspection.

| ID | Primary source and locator | Version/date boundary |
| --- | --- | --- |
| A1 | [Introduction to the AMD FSR API][A1], named sections above; retrieved lines 218–241, 262–325, 338–368, 487–513, 575–621 | Published SDK 2.3.0 manual; live, not commit-verified |
| A2 | [Upscaling 4.1.1][A2], Integration guidelines, Camera jump cuts, Limitations, Version history | Effect 4.1.1; version history 2026-06-24 |
| A3 | [FSR Super Resolution 3.1.5][A3], Memory requirements; Camera jump cuts | Effect 3.1.5; not selected-provider implementation |
| A4 | [AMD FSR SDK product page][A4], Automatic updates; General overview; Version history | Identifies SDK 2.3, June 2026; live |
| A5 | [SDK licence][A5], initial binary-distribution terms and source exceptions | Live manual, not release-package verification |
| A6 | [Older FidelityFX API introduction][A6], DLLs structure | SDK 2.0.0 historical lead only |
| M1 | [FreeLibrary][M1], Remarks | Page updated 2021-10-13 |
| M2 | [Important changes from D3D11 to D3D12][M2], Explicit synchronization; Residency | Live Microsoft D3D12 documentation |
| M3 | [SetPrivateDataInterface][M3], pData | Updated 2025-05-21 |
| M4 | [D3D12CreateDevice][M4], Remarks | Live Microsoft API contract |
| M5 | [DLL entry-point function][M5], Calling the entry-point function | Updated 2021-01-07 |
| M6 | [DllMain][M6], fdwReason, lpvReserved, Return value, Remarks | Updated 2022-09-02 |
| M7 | [EnumProcessModulesEx][M7], Remarks | Live Microsoft API contract |
| M8 | [CreateToolhelp32Snapshot][M8], flags and Remarks | Updated 2021-10-13 |
| M9 | [GetModuleHandleEx][M9], flags and Remarks | Updated 2023-02-09; A spelling, flags apply to W equivalent |
| M10 | [ID3D12DeviceChild::GetDevice][M10], Remarks | Updated 2024-02-22 |
| M11 | [D3D12 multithreading sample][M11], introductory requirements | Page dated 2026-07-08; documented sample, not newly executed |
| M12 | [Creating/recording lists and bundles][M12], allocator and recording sections | Live Microsoft D3D12 documentation |
| M13 | [Allocator Reset][M13], Remarks | Updated 2024-06-26 |
| M14 | [Executing/synchronizing lists][M14], execution and fences | Updated 2021-12-30 |
| M15 | [Fence SetEventOnCompletion][M15], Remarks | Updated 2024-02-22 |
| M16 | [Fence GetCompletedValue][M16], Return value | Live Microsoft API contract |
| M17 | [ID3D12Device][M17], inheritance and Remarks | Updated 2024-06-12 |
| M18 | [IUnknown::Release][M18], Return value; Remarks | Updated 2024-02-22 |
| M19 | [Programming DirectX with COM][M19], Creating a COM object | Live Microsoft guide; indirect-creation wording, not a per-interface affinity specification |
| M20 | [Processes, threads and apartments][M20], COM threading architecture | Updated 2026-07-17 |
| M21 | [CoInitializeEx][M21], Remarks | Updated 2024-11-12 |
| M22 | [DLL best practices][M22], general practices; lock order | Page updated 2023-05-12; article body identifies May 17, 2006 |
| M23 | [GetProcAddress][M23], Return value; named exports | Updated 2024-02-06 |
| R1 | [libloading Library][R1], new Safety; traits | Retrieved latest page explicitly identifies 0.9.0 |
| R2 | [libloading Windows source][R2], close/Drop source lines 294–327; traits 26–40 | Linked latest source; associated documentation identifies 0.9.0, no independent release hash |
| R3 | [Arc][R3], Thread safety | std 1.99.0, `b940084d7`, 2026-09-28 |
| R4 | [Rust Reference destructors][R4], destructor operation/drop scopes | Live reference; compiler version not pinned by project |
| R5 | [Mutex][R5], Poisoning | std 1.99.0, `b940084d7`, 2026-09-28 |
| R6 | [Rustonomicon FFI][R6], FFI and unwinding | Live official Rust documentation |
| R7 | [Microsoft windows-rs ID3D12Device][R7], traits | Retrieved page explicitly identifies windows 0.62.2 |

[P0]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3
[P1]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h
[P2]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_loader.h
[P3]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h
[P4]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp
[P5]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h
[P6]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h
[P7]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp
[P8]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12
[P9]: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md
[A1]: https://gpuopen.com/manuals/fsr_sdk/getting-started/ffx-api/
[A2]: https://gpuopen.com/manuals/fsr_sdk/techniques/super-resolution-ml/
[A3]: https://gpuopen.com/manuals/fsr_sdk/techniques/super-resolution-upscaler/
[A4]: https://gpuopen.com/amd-fsr-sdk/
[A5]: https://gpuopen.com/manuals/fsr_sdk/license/
[A6]: https://gpuopen.com/manuals/fidelityfx_sdk2/getting-started/ffx-api/
[M1]: https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary
[M2]: https://learn.microsoft.com/en-us/windows/win32/direct3d12/important-changes-from-directx-11-to-directx-12
[M3]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12object-setprivatedatainterface
[M4]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-d3d12createdevice
[M5]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-entry-point-function
[M6]: https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain
[M7]: https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-enumprocessmodulesex
[M8]: https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
[M9]: https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandleexa
[M10]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12devicechild-getdevice
[M11]: https://learn.microsoft.com/en-us/samples/microsoft/directx-graphics-samples/d3d12-multithreading-sample-win32/
[M12]: https://learn.microsoft.com/en-us/windows/win32/direct3d12/recording-command-lists-and-bundles
[M13]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12commandallocator-reset
[M14]: https://learn.microsoft.com/en-us/windows/win32/direct3d12/executing-and-synchronizing-command-lists
[M15]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12fence-seteventoncompletion
[M16]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12fence-getcompletedvalue
[M17]: https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nn-d3d12-id3d12device
[M18]: https://learn.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-release
[M19]: https://learn.microsoft.com/en-us/windows/win32/prog-dx-with-com
[M20]: https://learn.microsoft.com/en-us/windows/win32/com/processes--threads--and-apartments
[M21]: https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex
[M22]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices
[M23]: https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress
[R1]: https://docs.rs/libloading/latest/libloading/struct.Library.html
[R2]: https://docs.rs/libloading/latest/src/libloading/os/windows/mod.rs.html
[R3]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[R4]: https://doc.rust-lang.org/reference/destructors.html
[R5]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
[R6]: https://doc.rust-lang.org/nomicon/ffi.html
[R7]: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Graphics/Direct3D12/struct.ID3D12Device.html
