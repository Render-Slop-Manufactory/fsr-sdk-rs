# Source investigation: M6 DX12 state and synchronization

> Intake note (2026-09-24): renamed from `rename_me_3.md`. Opaque
> `:chatgpt-content-reference` markers had no supplied index-to-source map and
> were removed. The substantive findings and pinned source list were retained;
> this intake does not independently verify every claim.

I re-checked this against the exact `v2.3.0` tag (`60f4ea8`, released 2026-06-24) and corrected the sourcing/state wording.

## 1. Command-list contract

**VERIFIED — `ffxDispatch` records into a caller-supplied command list.** `ffxDispatchDescUpscale.commandList` is documented as the command list “to record upscaling rendering commands into,” and the generic FSR API documentation states that GPU dispatches encode their commands into the command list/buffer supplied in the descriptor.

**VERIFIED — DX12 ultimately requires an `ID3D12GraphicsCommandList`-compatible object.** `ffxGetCommandListDX12` accepts `ID3D12CommandList*` and performs only an opaque cast; `ExecuteGpuJobsDX12` casts it to `ID3D12GraphicsCommandList*` and directly records clear, copy, compute, barrier, and discard jobs. No `AddRef` or other ownership transfer occurs.

**VERIFIED — the list must already be in recording state.** FidelityFX immediately calls recording methods on the list. It never calls `Reset`. D3D12 states that command-list methods add commands only while a list is in recording state; `Reset` puts a closed list back into that state.

**VERIFIED — FidelityFX does not reset, close, execute, fence, change the allocator, or retain the supplied list.** `ExecuteGpuJobsDX12` records its queued jobs, resets its internal `gpuJobCount`, and returns. There is no command-queue operation in that path.

### DIRECT versus COMPUTE

**CORROBORATED — AMD's v2.3.0 sample uses a DIRECT list.** Every render-module callback gets a `CommandQueue::Graphics` command list; in the DX12 Cauldron backend, `Graphics` maps to `D3D12_COMMAND_LIST_TYPE_DIRECT`. The FSR module passes that exact list to `ffxDispatch`.

**UNKNOWN — AMD does not explicitly guarantee that the ML upscaler supports COMPUTE command lists.** The generic DX12 job vocabulary is largely compatible with COMPUTE lists: D3D12 allows compute dispatches, copies, UAV clears, barriers, descriptor binding, and `DiscardResource` on compute lists under the appropriate restrictions. The FidelityFX backend does not inspect `GetType()`. That is implementation compatibility, not an FSR4 API guarantee.

For M6, **DIRECT-only is therefore the smallest source-backed contract**. COPY and BUNDLE cannot support the backend's general job stream.

---

## 2. Queue/submission contract

**VERIFIED — upscale dispatch itself has no command-queue input.** `ffxDispatchDescUpscale` contains the command list and resources, but no `ID3D12CommandQueue`, fence, or submission object.

**VERIFIED — submission remains entirely with the application.** The backend only records commands. Thus the caller subsequently owns `Close`, `ExecuteCommandLists`, queue signaling, fence tracking, and any CPU wait.

**CORROBORATED — AMD's sample submits FSR on its Graphics/DIRECT queue.** Cauldron creates a Graphics command list for each render module, closes it afterwards, batches the lists, and submits that batch to `CommandQueue::Graphics`.

**INFERRED — recording now and submitting later is legal.** That is ordinary D3D12 deferred-command-list behavior; FidelityFX does not retain or independently submit the list.

For temporal ordering, one D3D12 detail matters: two successive `ExecuteCommandLists` calls are strictly ordered, whereas multiple lists passed in a *single* `ExecuteCommandLists` call may be merged so that later-list work begins before all earlier-list work finishes. This makes arbitrary batching of multiple dispatches for the **same temporal upscaler** something that should not be claimed safe without testing.

---

## 3. Resource-state table

AMD explicitly says that, for DX12, **all input resources should be transitioned to `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE` before `ffxDispatchDescUpscale`**.

The official sample actually enters with the combined read state `NON_PIXEL_SHADER_RESOURCE | PIXEL_SHADER_RESOURCE` and declares it as `FFX_API_RESOURCE_STATE_PIXEL_COMPUTE_READ`. The backend maps that FFX state to exactly that D3D12 combination. Thus the documentation's `NON_PIXEL_SHADER_RESOURCE` is the normative recommendation, while the combined read state is officially demonstrated as acceptable sample behavior.

| Resource | Before dispatch | During dispatch | Encoded state afterward | Status |
|---|---|---|---|---|
| Color | AMD: `NON_PIXEL_SHADER_RESOURCE`; sample: combined pixel/non-pixel read | SRV / compute read | Restored to state declared in `FfxApiResource.state` | **CORROBORATED** |
| Depth | Same input-state rule | SRV / compute read | Restored to declared initial state | **CORROBORATED** |
| Motion vectors | Same input-state rule | SRV / compute read | Restored | **CORROBORATED** |
| Exposure, if supplied | Same input-state rule | SRV / compute read | Restored | **VERIFIED** |
| Reactive mask, if supplied | Same input-state rule | SRV / compute read | Restored | **VERIFIED**; FSR4 does not require it |
| T&C mask, if supplied | Same | SRV / compute read | Restored | **VERIFIED**; FSR4 does not require it |
| Output | No separate normative pre-state stated; official sample uses combined read and declares that state | Transitioned to UAV when written | Restored to declared initial state | **CORROBORATED backend + sample** |
| Internal history/scratch | SDK-owned | SDK-managed | SDK-managed | **VERIFIED** |

The crucial backend behavior is explicit: registration copies `FfxApiResource.state` into both `initialState` and `currentState`; internal jobs call `addBarrier`; and unregister walks application-owned dynamic resources and transitions each back to `initialState`.

Therefore **the application's supplied FFX state must truthfully represent the actual D3D12 state**. It is not decorative metadata: FidelityFX uses it as the `StateBefore` basis of later barriers.

**VERIFIED — FidelityFX inserts its own intra-dispatch transition and UAV barriers.** If a resource needs a different state, `addBarrier` emits a transition; if it is already UAV and another UAV ordering point is requested, it emits a UAV barrier. Compute jobs explicitly transition bound UAV textures to `UNORDERED_ACCESS`.

This does **not** replace queue-to-queue synchronization. D3D12 requires explicit queue/fence coordination when a resource written on one queue is subsequently accessed on another.

---

## 4. Resource creation constraints

**VERIFIED — core FSR4 inputs are 2D frame-image resources at render/presentation resolution.** AMD specifies application-defined color, single-component floating-point depth, two-component floating-point motion vectors, optional exposure, and optional masks; motion vectors may be render or presentation resolution according to creation flags.

**VERIFIED — FSR4 reactive and T&C masks are no longer inherently required.** The provider/context-specific `ffxQueryDescUpscaleGetResourceRequirements` returns `required_resources` and `optional_resources`; a wrapper should not hard-code historical FSR2 requirements.

**VERIFIED — inputs must be SRV-viewable.** `RegisterResourceDX12` constructs SRVs for registered external GPU resources. A depth texture that cannot legally have an SRV, for example because its creation flags/view format prohibit one, cannot be passed directly.

**CORROBORATED — the output must be UAV-capable.** The backend creates UAV descriptors for an external resource only if its D3D12 description contains `D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS`; compute jobs then bind output UAVs and transition them to `UNORDERED_ACCESS`. For M6, requiring `ALLOW_UNORDERED_ACCESS` on the upscale output is therefore necessary.

`D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET` is **not** an FSR output requirement found in the API. It is only needed if the application itself also uses that image as an RT.

**VERIFIED — the backend contains explicit typeless/depth → SRV-compatible format conversion**, including `R32_TYPELESS → R32_FLOAT`. This supports ordinary shader-readable depth-resource arrangements, but does not imply every typeless format is accepted by the selected ML provider.

For less ordinary resource shapes:

| Property | Finding |
|---|---|
| Sample count | **UNKNOWN provider contract.** M6 should require `SampleDesc.Count == 1`; UAV output is inherently non-MSAA and the backend's ordinary texture views are designed around that path. |
| Mip count | **UNKNOWN provider contract.** Generic backend can create UAV descriptors per mip, but FSR4 docs describe frame images, not mip chains. `MipLevels == 1` is the defensible M6 restriction. |
| Array slices | **UNKNOWN provider contract.** Generic backend machinery is broader than the FSR4 contract; don't expose array support merely because descriptors can represent it. |
| Texture dimension | **VERIFIED semantic requirement for M6:** FSR inputs/output are image textures; restrict to 2D rather than expose generic backend 1D/3D capability. |
| Committed vs placed | **INFERRED neutral:** registration receives an `ID3D12Resource*` and `GetDesc()`; it does not distinguish allocation origin. Normal D3D12 ownership/residency still applies. |
| Reserved/tiled | **UNKNOWN.** Nothing establishes ML-provider support or manages tile residency for the application. |
| Aliasing | **UNKNOWN as an upscaler guarantee.** Generic backend supports alias barriers internally, but external allocation aliasing remains an application/D3D12 responsibility. |
| `ALLOW_SIMULTANEOUS_ACCESS` | No FSR4 requirement found. Cross-queue access still follows D3D12 synchronization rules. |

---

## 5. Descriptor/binding behavior

**VERIFIED — the caller does not supply FidelityFX descriptor heaps.** Backend context state owns CPU SRV/UAV heaps and a shader-visible descriptor ring. The backend creates and destroys them itself.

**VERIFIED — FidelityFX actively changes command-list binding state.** A compute job calls `SetComputeRootSignature` followed by `SetDescriptorHeaps(1, &dx12DescriptorHeap)` and then binds its descriptor tables/PSO.

Microsoft specifies that **all previously set shader-visible heaps are unset by `SetDescriptorHeaps`**. Descriptor-table state is consequently undefined after the heap changes until rebound.

**CORROBORATED — AMD's own sample explicitly compensates for this.** Immediately after FidelityFX work it comments that FidelityFX contexts modify the set resource-view heaps, then calls `SetAllResourceViewHeaps` to restore Cauldron's heaps.

So M6 must document that `dispatch` is **not command-list-state transparent**. Commands already recorded before it are unaffected, but application commands recorded afterward must re-establish any descriptor heaps, compute root signature, tables, and compute PSO they rely on.

---

## 6. Lifetime matrix

| Object | Needed through `ffxDispatch` CPU call | Needed through close/submission | Needed until GPU completion |
|---|---:|---:|---:|
| Supplied command list object | Yes | Yes, if caller still needs to close/submit it | **No inherent need after submission**; D3D12 permits resetting a command list while its previous execution is running |
| Command allocator | Yes indirectly | Yes | **Yes; allocator memory cannot be reset/recycled early** |
| Input/output `ID3D12Resource`s | **Yes; raw pointers, no FFX `AddRef`** | Yes | **Yes** |
| D3D12 device | Backend holds its own COM ref | Through context | Through outstanding context-owned GPU work |
| Upscaler/context | Yes | Yes | **Yes for any work referencing its internal GPU objects** |
| DX12 backend/provider-owned GPU objects | Yes | Yes | **Yes** |
| FidelityFX runtime/DLL | Yes | Through context lifetime | Practically until outstanding work is complete and context has then been destroyed |

The command-list/allocator distinction is standard D3D12: Microsoft permits `ID3D12GraphicsCommandList::Reset` while its prior execution is still running, but calling `ID3D12CommandAllocator::Reset` while GPU work using it remains outstanding is undefined behavior.

**VERIFIED — FidelityFX does not retain external resource COM objects.** Registration simply stores the incoming `ID3D12Resource*` in backend state. D3D12 places resource-memory lifetime and CPU/GPU synchronization responsibility on the application; memory must remain available until the GPU no longer needs it.

**VERIFIED — the backend does retain the device.** On first backend-context creation it calls `ID3D12Device::AddRef`; backend teardown releases that reference.

**INFERRED, high confidence — the upscaler/context must survive GPU completion.** Context teardown releases FidelityFX-owned resources, descriptor heaps, constant-buffer storage and other GPU objects immediately; no internal fence wait occurs in that teardown. Releasing them while previously recorded commands still reference them would violate standard D3D12 object-lifetime requirements.

AMD's sample supports that inference: when it needs to recreate the FSR context, it first flushes all command queues. That is sample policy rather than an explicit API guarantee, but it matches the D3D12 requirement.

---

## 7. Fence/synchronization requirements

**VERIFIED — `ffxDispatch` does not wait for GPU work.** It only records commands; it may return before the list has even been closed or submitted.

There is no upscaler completion token, queue, or fence field in `ffxDispatchDescUpscale`. Normal D3D12 queue/fence tracking is therefore the completion mechanism.

The required synchronization distinction is:

| Operation after dispatch | What is required |
|---|---|
| GPU reads same resource later on correctly ordered same queue | Normal command/barrier ordering; no CPU wait inherently required |
| CPU overwrites or frees a resource | Fence/completion proof required |
| Recycle its backing allocation | Fence/completion proof required |
| Reset command allocator | Fence/completion proof required |
| Submit dependent work on another queue | Queue-to-queue fence/wait required as dictated by D3D12 hazards |
| Destroy/recreate FSR context | Drain all work referencing it first |
| Resize by destroying/replacing dispatch resources | Drain work referencing the old resources first |
| Destroy backend/provider-owned state | Drain first |
| Unload FidelityFX DLL/runtime | Complete GPU work, destroy dependent contexts/backend objects, then unload |
| Destroy device | Normal D3D12 teardown: outstanding dependent work/objects must already be dealt with |

**VERIFIED — `reset=true` is temporal-history reset, not GPU synchronization.** AMD says it tells FSR that a camera discontinuity occurred and causes additional internal resources to be cleared. It does not wait for or cancel an earlier dispatch.

Therefore reset only changes algorithmic history. The reset dispatch still has to execute in the correct GPU order relative to preceding work.

---

## 8. Temporal and concurrency hazards

**VERIFIED — the upscaler has persistent GPU history.** AMD explicitly states that context memory includes intermediate surfaces and surfaces persistent across many application frames. Camera reset clears previously accumulated data and additional internal resources.

**INFERRED — frame N+1 may be CPU-recorded before frame N completes.** Nothing about D3D12 recording requires the CPU to wait for earlier GPU work merely to record another list. What matters is execution ordering and object lifetime.

**UNKNOWN — arbitrary GPU overlap between two dispatches on one upscaler is supported.** AMD gives no such guarantee. Since hidden persistent history is shared between frames, the wrapper should assume temporal dispatches require ordered execution unless testing or AMD documentation establishes otherwise.

This is especially relevant when batching command lists: D3D12 explicitly permits work from the second list in a *single* `ExecuteCommandLists` call to start before the first list finishes. For a minimal wrapper, do not promise that independently recorded same-context dispatches may be arbitrarily batched/overlapped.

**UNKNOWN — concurrent CPU calls on one upscaler/context are thread-safe.** No FSR4 thread-safety guarantee was found. The backend also contains mutable shared scheduling state such as `gpuJobCount` with ordinary increments during job scheduling. That is evidence against assuming concurrency, but not itself a contractual proof of non-thread-safety.

Consequently, for M6:

- one upscaler concurrently dispatched from several CPU threads: **UNKNOWN; serialize**;
- same upscaler concurrently targeting several command lists: **UNKNOWN**;
- same upscaler across several command queues: **UNKNOWN**, plus explicit D3D12 cross-queue synchronization is mandatory;
- several independent upscaler instances: architecture supports multiple effect contexts, but no blanket concurrent-entry guarantee should be invented.

A Rust `&mut self` dispatch receiver would therefore encode a useful CPU-side serialization invariant without claiming more.

---

## 9. Consequences for Rust safety

Rust can genuinely enforce CPU-side ownership relationships: keeping the runtime/device alive with the `Upscaler`, preventing calls after destruction, validating dimensions/resource presence, and preventing simultaneous safe calls through a single `&mut Upscaler`.

Rust **cannot**, with a normal temporary borrow, prove that a resource survives until asynchronous GPU execution ends:

```text
dispatch(&mut self, cmd: &..., color: &..., output: &...) -> Result<()>
                    ^ borrows can end here
                    GPU execution may not even have started
```

D3D12 explicitly makes CPU/GPU lifetime and fencing the application's responsibility.

Cloning a COM interface only helps if the clone is retained **until the corresponding fence completes**. Cloning it for the duration of `dispatch()` and dropping the clone on return provides no GPU-lifetime guarantee. It also cannot solve state correctness, aliasing, conflicting writes, or cross-queue hazards.

A global Rust typestate such as `Resource<NonPixelShaderResource>` is not realistically authoritative unless `fsr-sdk` owns every transition of that resource. D3D12 applications commonly transition the same resource elsewhere and across other command lists/queues. FidelityFX itself relies on an explicit caller-supplied state value and restores that state afterward.

Given the project's stated rule that the high-level crate is safe only where Rust can genuinely enforce safety, **an immediately-returning M6 `dispatch` cannot fully enforce its GPU-lifetime preconditions without a fence/in-flight ownership abstraction**.

The defensible M6 consequence is therefore:

**INFERRED / API-SAFETY CONSEQUENCE — make the low-level DX12 dispatch boundary `unsafe` unless the wrapper also owns enough submission/fence machinery to retain every referenced object until completion.**

Its caller contract would need to require, at minimum:

1. a valid open DIRECT command list;
2. accurate actual D3D12 states matching the `FfxApiResource.state` declarations;
3. valid SRV-compatible inputs and UAV-capable output;
4. resources, allocator and context-owned GPU objects remaining valid for the necessary GPU lifetime;
5. correct queue/fence ordering, especially across queues;
6. no concurrent unsupported use of the same upscaler;
7. application descriptor/binding state being rebound after FidelityFX as needed.

The command list/resource COM interfaces should normally be **borrowed for CPU recording**, not blindly cloned as if that solved GPU lifetime. If a later API introduces an in-flight/fence token, that token could retain COM clones until completion.

Finally, **M6 should remain explicitly DX12-specific**. The contract being exposed is fundamentally about `ID3D12GraphicsCommandList`, resource states, descriptor heaps, command allocators, DIRECT queues and D3D12 fences. Abstracting that at this stage would hide rather than remove the safety obligations.

---

## 10. Experiments still required

The open backend settles most mechanics, but the ML 4.1.1 provider is not completely characterized by public C++ source. These native tests remain useful:

1. **Real output-image test:** DIRECT list, documented input states, UAV-capable output, submit/fence, verify reconstructed image under D3D12 debug layer + GPU validation.
2. **PIX state trace:** confirm actual ML-provider jobs transition output to UAV and that all external resources finish in their declared initial states.
3. **Output-state matrix:** try initial combined-read, pure compute-read, UAV, and another legal backend-supported state.
4. **COMPUTE-list experiment:** dispatch FSR4 on a COMPUTE list/queue with full validation. A pass establishes implementation behavior, not a public AMD guarantee.
5. **Descriptor-clobber test:** bind application CBV/SRV/UAV + sampler heaps, dispatch, verify rebinding requirements predicted by backend source.
6. **Premature-destruction tests:** release an external resource/context before its queue fence and verify debug-layer diagnostics; compare against fence-before-destroy.
7. **Two frames in flight:** record N+1 while N executes, submit in strictly ordered queue submissions, verify temporal correctness.
8. **Same-submit overlap:** put two same-context dispatch lists into one `ExecuteCommandLists` call and test separately; do not assume this is valid simply because ordinary D3D12 permits list overlap.
9. **Cross-queue dispatch:** only if later API scope needs it; synchronize with explicit queue fences and characterize provider behavior.
10. **Resource-shape tests:** reject/verify MSAA, mip chains, arrays, placed resources and aliasing cases before broadening the public resource contract.
11. **Provider resource-requirements query:** record the exact required/optional bitfields returned by the v2.3.0 ML 4.1.1 context; this is the authoritative way to decide mask/exposure requirements for the selected provider.

### Primary sources

- [FSR SDK v2.3.0 release/tag — commit 60f4ea8](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea8)
- [AMD FSR Upscaling 4.1.1 integration documentation — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/techniques/super-resolution-ml.md)
- [ffx_upscale.h — v2.3.0 dispatch/query ABI](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h)
- [Generic FSR API dispatch documentation — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md)
- [DX12 FidelityFX backend implementation — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp)
- [Official FSR DX12 sample — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp)
- [Cauldron2 framework command-list/submission path — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/Cauldron2/dx12/framework/core/framework.cpp)
- [Cauldron2 DX12 queue/list implementation — v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/Cauldron2/dx12/framework/render/dx12/device_dx12.cpp)
- [Microsoft — Creating and recording D3D12 command lists](https://learn.microsoft.com/en-us/windows/win32/direct3d12/recording-command-lists-and-bundles)
- [Microsoft — ExecuteCommandLists ordering semantics](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12commandqueue-executecommandlists)
- [Microsoft — SetDescriptorHeaps semantics](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-setdescriptorheaps)
- [Microsoft — ID3D12CommandAllocator::Reset synchronization requirement](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12commandallocator-reset)
- [Microsoft — D3D12 explicit synchronization/lifetime responsibilities](https://learn.microsoft.com/en-us/windows/win32/direct3d12/important-changes-from-directx-11-to-directx-12)
- [Microsoft — multi-queue execution and synchronization](https://learn.microsoft.com/en-us/windows/win32/direct3d12/executing-and-synchronizing-command-lists)
