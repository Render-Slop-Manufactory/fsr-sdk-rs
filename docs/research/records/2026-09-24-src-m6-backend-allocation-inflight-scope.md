# Source investigation: M6 backend allocation and recorded/in-flight scope

- **Investigation date:** 2026-09-24
- **Project:** `fsr-sdk-rs`
- **Decision gate:** D011, proposed public DX12 upscaler dispatch boundary
- **Pinned upstream:** AMD FSR SDK 2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`
- **Platform in scope:** Windows x64/MSVC, DirectX 12
- **Modern API in scope:** `ffxCreateContext`, `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, `ffxDispatch`
- **Research question:** What public AMD evidence establishes the ownership/sharing scope of transient DX12 backend allocation state relevant to recorded FidelityFX upscaler work, and what does AMD publicly require about recording ahead or GPU completion before that state is reused?

## Topic and scope

This record is evidence gathering for D011 only. It does **not** select or accept a D011 policy, define the final Rust safety contract, or turn an implementation constant into an application-facing limit.

The investigation separates six notions that are easy to collapse incorrectly: CPU API-call serialization, command-list recording, queue submission ordering, GPU execution ordering, GPU completion, and lifetime/reuse of application or FidelityFX-owned resources. “In flight” is avoided unless the relevant notion is stated explicitly.

The central distinction is between:

1. the **open DX12 backend and open FSR3 provider** present in the pinned SDK source; and
2. the **modern signed upscaler provider path**, which AMD documents as the required FSR API distribution and which contains FSR 4.1.1 in SDK 2.3.0.

Findings about the open backend are not generalized to the signed FSR 4.1.1 provider unless public AMD evidence connects the two.

## Baseline and evidence cutoff

The supplied project brief establishes the following repository baseline, treated here as **repository-provided evidence**, not independently reproduced runtime evidence:

- `fsr-sdk-rs` is a narrow Rust wrapper around FidelityFX SDK 2.3.0 for Windows/DX12.
- M5 established the bounded construction/lifecycle path; M6a added the minimum private resource/dispatch ABI and recorded one successful native dispatch/readback on the repository's tested configuration.
- The open DX12 backend has finite transient storage and defines `FFX_MAX_QUEUED_FRAMES` as `4`.
- A repository recording-ahead experiment reportedly recorded 2, 5, and 8 dispatches on one upscaler context before submission, and 2 and 5 sibling contexts before submission, followed by ordered submissions and fence completion with distinct finite outputs and no observed D3D12 debug error/device removal in those runs.
- Those observations are explicitly bounded: they do not establish a universal supported count, allocator ownership, signed-provider implementation details, arbitrary GPU overlap/cross-queue behavior, destruction safety, or temporal/image correctness.

**Repository inspection limitation.** The investigation request refers to a repository ZIP and specifically asks that D011, the M6 synthesis, the recording-ahead experiment, and research-process instructions be inspected first. In this execution environment, no repository ZIP or repository tree was supplied; the only local file available was the investigation brief itself. Therefore those local files and repository-specific source-research formatting conventions could not be independently inspected. This record preserves the supplied baseline verbatim in substance and follows the mandatory structure and evidence discipline requested in the brief, but it does not claim local repository verification.

**Public-source cutoff:** AMD/GPUOpen public material inspected on 2026-09-24, with source-code conclusions pinned to commit `60f4ea81909200d8542eca14dccb2628b763a9a3` wherever a revision-sensitive claim is made.

## Method and exclusions

Method:

1. Trace `FFX_MAX_QUEUED_FRAMES` from its definition into the pinned DX12 backend's allocation formulas and ring-index logic.
2. Identify which fields are stored in the backend-wide `BackendContext_DX12` versus per-effect `EffectContext` records.
3. Trace modern API dispatch to provider dispatch and, separately, the open FSR3 provider's backend creation path.
4. Search the pinned API documentation, FSR3/FSR4 documentation, headers, and implementation for explicit GPU-completion or queue-depth requirements.
5. Treat the signed/external provider boundary as opaque unless a public source exposes the relevant implementation.
6. Compare source facts with the repository-provided recording-ahead observations without deriving a numeric policy.

Excluded: editing/accepting D011; proposing the final Rust API; production/test changes; native or GPU experiments; claiming reproduction of the signed-provider experiment; image-quality/temporal-correctness work; broader resource-profile design; Bevy/wgpu integration; and deriving a public dispatch limit from internal capacities.

Evidence labels used below:

- **Verified** — directly supported by inspected primary AMD source at the stated revision. This verifies a source fact, not a runtime result.
- **Upstream claim** — AMD states the behavior/requirement in public documentation or API comments; it was not independently reproduced here.
- **Inference** — a reasoned conclusion from identified source facts, not an explicit AMD contract.
- **Unresolved** — inspected public evidence does not answer the question.

## Pinned inputs and public source register

All sources below are AMD/GPUOpen primary sources and were accessed 2026-09-24.

**S1 — SDK 2.3.0 package/version manifest.** Exact commit. Establishes SDK “Redstone” 2.3.0, FSR3 Upscaler 3.1.5, ML Upscaler 4.1.1.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/readme.md#L1-L15

**S2 — internal FidelityFX limits.** Exact commit. Defines `FFX_MAX_QUEUED_FRAMES`, descriptor-ring and constant-buffer-ring sizing, resource/pass/job maxima.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_internal_types.h#L97-L133

**S3 — open DX12 backend implementation.** Exact commit. Establishes backend-context fields, scratch sizing, descriptor/constant/staging rings, GPU-job scratch, creation/destruction, and wrap behavior.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp

**S4 — modern DX12 backend construction helper.** Exact commit. Allocates one scratch buffer and initializes one `FfxInterface` for a requested context count.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp#L37-L64

**S5 — modern API implementation.** Exact commit. Establishes provider delegation, including `ffxDispatch`.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp#L39-L63  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp#L166-L172

**S6 — provider abstraction / external-provider boundary.** Exact commit. The public source includes an `amdinternal` DX12 external-provider header and considers the external provider during provider selection.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h#L81-L145

**S7 — open modern FSR3 upscaler provider.** Exact commit. Its modern context embeds an `FfxInterface`, creates that backend with `contexts = 1`, and frees that context's backend scratch at destruction.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L62-L92  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L141-L158

**S8 — FSR3 upscaler public header, version 3.1.5.** Exact commit. Documents context lifetime/destruction requirements and command-list recording semantics.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h#L223-L269

**S9 — open FSR3 implementation.** Exact commit. Contains the separate `FSR3UPSCALER_MAX_QUEUED_FRAMES = 16` implementation constant and advances its effect-level resource-frame index before executing scheduled backend jobs.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L50-L52  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1073-L1082

**S10 — AMD FSR API documentation.** Exact commit. States that applications use signed DLLs; identifies the upscaler DLL; and states that GPU dispatches encode commands into the caller-provided command list/buffer.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md#L4-L24  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md#L187-L198

**S11 — FSR 4.1.1 documentation.** Exact commit. States that FSR4 requires the AMD FSR API and signed binary distribution, and refers to the FSR3 documentation for general integration guidance.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#L1-L46

**S12 — FSR3 upscaler integration documentation.** Exact commit. Used as the public general-integration reference to which the FSR4 page points.  
https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md

No secondary source is needed for the material conclusions below.

## Findings

### 1. The open DX12 backend has an identifiable allocation owner: one `FfxInterface` scratch-backed `BackendContext_DX12`

**Verified** — `ffxGetScratchMemorySizeDX12(maxContexts)` sizes one scratch allocation as a `BackendContext_DX12` plus arrays for resources, effect contexts, constant-data staging, and GPU-job descriptions. `ffxGetInterfaceDX12` stores that scratch buffer in `FfxInterface::scratchBuffer` and writes `maxContexts` into `BackendContext_DX12::maxEffectContexts` (S3, lines 223–303).

**Verified** — The helper used by modern open providers allocates one scratch buffer for the requested number of contexts and initializes one `FfxInterface` around it (S4, lines 37–64).

This gives a precise source-level owner for the **open backend implementation**: the transient backend state is associated with a particular `FfxInterface` and its scratch-backed `BackendContext_DX12`, not directly with the process or DX12 device as such.

The relevant state divides as follows:

| State/allocation | Source-level owner in open DX12 backend | Sizing/indexing | Relevance after command encoding |
|---|---|---|---|
| GPU-job description queue (`pGpuJobs`, `gpuJobCount`) | `BackendContext_DX12` | scratch storage sized with `maxEffectContexts * FFX_MAX_GPU_JOBS`; one shared `gpuJobCount` | CPU recording scratch; queue count is reset after jobs are encoded into the supplied command list |
| Constant-data staging ring (`pStagingRingBuffer`, `stagingRingBufferBase`) | `BackendContext_DX12` | scratch storage allocated as `maxEffectContexts * FFX_CONSTANT_BUFFER_RING_BUFFER_SIZE`; one shared base; the staging function wraps at `FFX_CONSTANT_BUFFER_RING_BUFFER_SIZE` | CPU staging data used while constructing/encoding jobs; not itself the GPU constant-buffer resource |
| Shader-visible CBV/SRV/UAV descriptor ring/heap (`descRingBuffer`, `descRingBufferBase`) | `BackendContext_DX12` | one heap; ring capacity `FFX_RING_BUFFER_DESCRIPTOR_COUNT * maxEffectContexts`; one shared base wraps to zero | GPU-visible descriptor entries referenced by recorded root descriptor tables |
| Fallback upload constant buffer (`constantBufferResource`, `constantBufferMem`, `constantBufferOffset`) | `BackendContext_DX12` | one lazy-created upload resource; size `align(FFX_BUFFER_SIZE,256) * maxEffectContexts * FFX_MAX_PASS_COUNT * FFX_MAX_QUEUED_FRAMES`; one shared offset wraps to zero | GPU-visible addresses are placed into recorded root CBVs |
| Persistent/dynamic resource index ranges and bindless ranges | per `BackendContext_DX12::EffectContext` inside the same backend | per-effect fields/ranges | context/resource ownership metadata, distinct from the shared transient ring heads |

Evidence: S3, `BackendContext_DX12` lines 115–203; scratch mapping lines 1197–1231; descriptor heap/ring creation around lines 1233–1250; fallback constant allocator lines 414–453; staging lines 2568–2588; job scheduling/execution lines 3149–3177 and 3577–3646.

**Verified** — The open backend therefore does **not** place the descriptor-ring head, fallback constant-buffer head, staging-ring head, or GPU-job count inside each `EffectContext`; those mutable heads are backend-wide fields (S3, lines 115–203).

**Inference** — If one open `FfxInterface` is intentionally configured to host multiple effect contexts (`maxEffectContexts > 1`), those effect contexts share these backend-level transient heads and backing allocations. Their persistent resource/bindless subranges can still be partitioned per `EffectContext`. This is narrower than saying “all contexts on the device share storage”: the source establishes sharing only among effect contexts placed in the same backend-interface instance.

### 2. `FFX_MAX_QUEUED_FRAMES = 4` is an internal capacity multiplier, not a public dispatch-count contract

**Verified** — In `ffx_internal_types.h`, AMD comments `FFX_MAX_QUEUED_FRAMES` as the “Maximum number of queued frames in the backend” and defines it as `4` (S2, lines 97–100).

**Verified** — The same internal header defines:

- `FFX_MAX_RESOURCE_COUNT = 512` per effect context;
- `FFX_MAX_PASS_COUNT = 50`;
- `FFX_RING_BUFFER_DESCRIPTOR_COUNT = FFX_MAX_QUEUED_FRAMES * FFX_MAX_PASS_COUNT * FFX_MAX_RESOURCE_COUNT`, explicitly described as the descriptors needed “for a single effect context”;
- `FFX_CONSTANT_BUFFER_RING_BUFFER_SIZE = FFX_MAX_QUEUED_FRAMES * FFX_MAX_PASS_COUNT * FFX_BUFFER_SIZE`, also described for a single effect context (S2, lines 102–123).

**Verified** — The open DX12 backend then multiplies the descriptor-ring capacity by `maxEffectContexts` when creating the shader-visible heap, while keeping one backend-wide `descRingBufferBase` (S3, around lines 1245–1248). Before emitting a UAV or SRV table it tests whether the next table fits; if not, it sets the shared base back to zero. After copying descriptors it advances that same shared base (S3, lines 3216–3224, 3290–3295, 3319–3327, 3391–3396).

**Verified** — The fallback constant allocator uses one backend-wide upload buffer. Its size includes `maxEffectContexts * FFX_MAX_PASS_COUNT * FFX_MAX_QUEUED_FRAMES`; it wraps the shared `constantBufferOffset` to zero when the next aligned allocation would reach/exceed the buffer size (S3, lines 414–453). During compute-job encoding, the resulting GPU virtual address is written into a root constant-buffer view (S3, lines 3445–3460).

**Verified** — Neither wrap path performs a fence query, waits for GPU completion, associates an allocation with a command-list submission, or tracks a per-frame completion token in the inspected open backend source.

**Inference** — The value `4` is therefore evidence about how the **open backend implementation sizes worst-case ring capacity**, not evidence that one dispatch consumes exactly one quarter of either ring. Actual descriptor and constant-buffer consumption depends on the passes/bindings encoded by the effect. A fifth CPU-recorded dispatch need not coincide with a ring wrap.

**Unresolved** — The public sources do not establish that the signed FSR4 provider uses this backend allocator, this constant, these formulas, or these wrap rules.

A second constant must not be conflated with it. **Verified:** the open FSR3 implementation separately defines `FSR3UPSCALER_MAX_QUEUED_FRAMES = 16` with the comment “max queued frames for descriptor management,” and wraps an FSR3 `resourceFrameIndex` modulo 16 (S9). This is effect-level open FSR3 implementation state. It is not a public modern-API dispatch limit and does not convert the backend's `FFX_MAX_QUEUED_FRAMES = 4` into such a limit.

### 3. CPU recording scratch is not the same thing as GPU-referenced transient storage

**Verified** — `ScheduleGpuJobDX12` writes job descriptions into the backend-wide `pGpuJobs` array and increments `gpuJobCount`; `ExecuteGpuJobsDX12` walks that array to emit API commands into the supplied command list and then resets `gpuJobCount` to zero (S3, lines 3149–3177 and 3577–3646).

**Inference** — This queue is temporary **CPU command-recording scratch**. Its reset after encoding is not evidence of GPU completion; the commands have merely been written to the application's command list.

**Verified** — `StageConstantBufferDataDX12` similarly copies CPU constant data into the backend's staging ring and advances/wraps one staging offset (S3, lines 2568–2588). When compute jobs are encoded, the open backend copies the staged bytes into either a registered constant allocator or its fallback GPU upload allocation and binds the resulting address (S3, lines 3445–3460).

**Inference** — Reuse of the CPU staging/job scratch can be safe earlier than reuse of the shader-visible descriptor entries or fallback upload-buffer bytes, because recorded GPU commands retain the latter's descriptor handles/GPU virtual addresses rather than the former's CPU staging pointers.

This distinction matters: a single phrase such as “backend transient storage” covers objects with materially different lifetime requirements.

### 4. The open backend's GPU-visible rings wrap without an explicit completion gate

**Verified** — The shader-visible descriptor ring and fallback constant upload ring use monotonic offsets with wrap-to-zero conditions (S3). The inspected wrap code contains no fence value, event, queue, command allocator, submission serial, or completion check.

**Inference** — Source alone therefore shows a potential reuse boundary: once a shared ring wraps, newly recorded commands may overwrite descriptors/upload bytes that earlier recorded or submitted command lists could still reference. The source does not itself prove when that situation is safe; it only shows that the open allocator does not enforce the safety condition internally.

This is **not** equivalent to saying “wait every four dispatches.” Capacity is expressed in descriptor/constant-buffer allocation units and worst-case pass/resource maxima, not dispatch count. It is also not equivalent to saying “four frames are always supported concurrently,” because no public contract inspected states that interpretation.

### 5. AMD's modern FSR API describes command encoding, not a per-dispatch completion wait

**Upstream claim** — The pinned AMD FSR API documentation states that GPU rendering dispatches encode their commands into the command list/buffer supplied in the dispatch descriptor (S10, lines 187–198).

**Verified** — The pinned API implementation makes `ffxDispatch` a direct delegation to the provider associated with the modern context (S5, lines 166–172). Thus any provider-specific transient allocation policy can live behind that provider boundary.

The following searches were made across the inspected pinned API docs, FSR3/FSR4 documentation, relevant headers, and open implementation paths. No explicit AMD requirement was found that an application must establish GPU completion:

- after every `ffxDispatch`;
- after exactly four dispatches or another fixed number;
- before recording another upscaler dispatch on the same modern context;
- before recording/using a sibling modern upscaler context;
- specifically before the open descriptor or fallback constant ring wraps.

This is a **negative source finding**, not proof that no such requirement exists in unpublished material. It must not be upgraded into a guarantee of unlimited recording ahead.

### 6. Destruction is the one inspected upscaler path with explicit GPU-use lifetime guidance

**Upstream claim** — The public FSR3 header states that before destroying an FSR3 context, care should be taken to ensure that the GPU is not accessing resources created or used by FSR3, and therefore recommends that the GPU be idle before context destruction (S8, lines 223–238).

**Verified** — The open DX12 backend's `DestroyBackendContextDX12` releases the shared fallback constant-buffer resource and descriptor heaps when its backend refcount reaches zero, but the function itself performs no GPU wait (S3, lines 1421–1479).

**Inference** — The explicit lifetime burden is placed on the caller/integration around destruction rather than enforced by the open backend. This destruction guidance does not imply a wait after ordinary dispatches.

For application-owned command allocators and resources, no additional FidelityFX-specific completion cadence was found in the inspected sources. Their ordinary DX12 reuse/lifetime requirements remain separate from the FidelityFX source question and are not evidence for a FidelityFX ring limit.

### 7. Generic open-backend sharing and modern open-FSR3 sharing are different questions

**Verified** — The generic DX12 backend is capable of hosting multiple effect contexts in one backend interface: scratch sizing and several heaps scale with `maxEffectContexts`; effect contexts occupy slots/ranges inside that one backend context (S3).

**Verified** — The modern **open FSR3 3.1.5 provider**, however, embeds an `FfxInterface` in each `InternalFsr3UpscalerUContext` and calls `MustCreateBackend(..., &internal_context->backendInterface, 1, ...)` during each modern context creation (S7, lines 62–92). At destruction it frees that context's `backendInterface.scratchBuffer` (S7, lines 141–158).

**Inference** — For this open FSR3 modern provider path, sibling modern API upscaler contexts have separate backend scratch/interface instances, each provisioned for one effect context. The open backend's fallback descriptor/constant rings are consequently independent between those sibling modern FSR3 contexts, even though the generic backend implementation could share rings if multiple effects were deliberately hosted in one interface.

This is a material narrowing of “backend allocation domain” for the open provider: the source-identifiable domain is the **backend-interface/scratch instance**, and this provider instantiates one per modern FSR3 context.

### 8. The signed FSR4.1.1 provider does not expose an equivalent public ownership trace

**Upstream claim** — SDK 2.3.0 identifies ML Upscaler/FSR4 version 4.1.1 (S1). AMD's FSR4 documentation says FSR4 requires the AMD FSR API and signed binary distribution (S11, lines 42–46). The API documentation says applications using the AMD FSR API must use provided signed DLLs and identifies `amd_fidelityfx_upscaler_dx12.dll` as containing the latest FSR4 provider plus legacy FSR3/FSR2 providers (S10, lines 13–24).

**Verified** — The public provider abstraction includes `../../amdinternal/api/internal/dx12/ffx_provider_external.h` for the DX12 external-provider implementation and considers that external provider during provider selection (S6, lines 81–145). The implementation of that `amdinternal` provider is not exposed by the inspected public source path.

**Unresolved** — Public source therefore does not establish for the signed FSR4.1.1 provider:

- whether its transient descriptor/constant storage is per modern context, per provider instance, per DLL/runtime, per device, or otherwise shared;
- whether sibling modern contexts share a ring or pool;
- whether it uses `FFX_MAX_QUEUED_FRAMES`, the open DX12 allocator, or a different allocator;
- what capacity/wrap policy it uses;
- whether it internally tracks GPU completion or submission state;
- whether cross-queue use has provider-specific restrictions beyond normal DX12 synchronization;
- what exact provider-owned data remain referenced by previously recorded command lists.

It would be an unsupported generalization to import the open FSR3 provider's one-backend-per-context result into the signed FSR4.1.1 provider.

### 9. Ordering concepts kept separate

| Concept | What the inspected evidence establishes |
|---|---|
| **CPU API-call serialization** | **Unresolved as a public general upscaler contract.** The open backend contains shared mutable recording counters/heads; only the fallback constant allocator has a mutex in the inspected code. No broad public statement was found that defines supported concurrent `ffxDispatch` calls on one modern context. |
| **Command-list recording order** | **Upstream claim / Verified implementation path.** `ffxDispatch` reaches a provider; GPU dispatches encode commands into the caller-provided command list/buffer (S5, S10). |
| **Submission order** | Not chosen by `ffxDispatch`; the application owns command-list submission. The repository experiment's ordered submissions are a bounded local observation, not an AMD contract. |
| **GPU execution order** | Depends on DX12 queue/submission synchronization. No FidelityFX-specific cross-queue ordering rule relevant to this allocator question was found. |
| **GPU completion** | No ordinary per-dispatch/fixed-count completion gate found. Context destruction has explicit FSR3 guidance that the GPU should no longer access FSR3 resources, with GPU idle recommended (S8). |
| **Application resource lifetime** | Separate from backend allocator reuse. Inputs are referenced by recorded GPU work; this investigation found no additional AMD queue-depth number governing caller resources. |
| **Effect/context lifetime** | FSR3 header explicitly requires safe GPU lifetime before destruction (S8); open backend destroy does not wait itself (S3). |
| **Backend transient allocator reuse** | Open descriptor/upload rings advance and wrap by allocation offsets with no explicit fence check (S3). Signed-provider behavior is unresolved. |

### 10. Comparison with the repository recording-ahead evidence

The repository-provided recording-ahead observations and the public source findings are compatible; they answer different questions.

**Narrowed, not contradicted.** The source shows why `FFX_MAX_QUEUED_FRAMES = 4` does not imply “the fifth CPU-recorded dispatch must fail.” The open backend's rings are sized from worst-case pass/resource quantities, and their heads advance by the descriptors/constant bytes actually allocated, not by one fixed slot per `ffxDispatch`. The signed provider may also use a different implementation entirely.

**Unresolved** — Recording 2/5 sibling signed-provider contexts successfully does not identify their allocator ownership. That observation is compatible with at least two materially different implementations: independent per-context transient allocations, or a shared allocation with sufficient capacity. Without provider tracing/source, the experiment cannot distinguish them.

**Supported only for an open comparison path.** The open modern FSR3 provider gives each modern FSR3 context its own one-context backend interface. That demonstrates that independent per-modern-context backend allocation is a real design in the pinned SDK, but it is not evidence that the selected signed FSR4.1.1 provider does the same.

**No universal numeric limit follows.** Neither the source nor the bounded experiment establishes a supported public number of CPU-recorded dispatches, submitted-but-incomplete dispatches, or simultaneously executing GPU dispatches.

## Baseline delta

The investigation changes or sharpens the supplied baseline as follows without choosing D011 policy:

1. **Open-backend owner identified more precisely.** The generic pinned DX12 backend's relevant mutable transient state is owned by one scratch-backed `BackendContext_DX12` associated with an `FfxInterface`. Descriptor and fallback constant rings have backend-wide heads. This is more precise than an unspecified “backend allocation domain.”
2. **Generic sharing scope identified.** If multiple effect contexts occupy the same open `FfxInterface`, they share those ring heads/backing allocations; per-effect resource/bindless metadata is separate.
3. **Modern open-FSR3 scope identified.** The open FSR3 3.1.5 modern provider creates a distinct `FfxInterface` with capacity for one effect context for each modern context, so its fallback backend transient rings are per modern FSR3 context.
4. **`FFX_MAX_QUEUED_FRAMES` characterized.** `4` is an internal backend capacity multiplier used in descriptor and constant-buffer ring sizing; the inspected source does not make it a dispatch count or completion cadence.
5. **Reuse mechanism characterized.** Open descriptor/upload ring indices wrap to zero without an explicit GPU-completion check. This identifies a possible resource-reuse hazard boundary but not the application-visible condition under which AMD guarantees it safe.
6. **Completion contract search narrowed.** No ordinary upscaler requirement was found to wait after each dispatch or after a fixed count. The inspected explicit completion/lifetime guidance is destruction-specific: ensure the GPU is no longer accessing FSR3 resources; GPU idle is recommended.
7. **Signed-provider scope remains open.** The source boundary to the signed/external provider is visible, but its allocator ownership/sharing implementation is not public. The open FSR3 result cannot resolve the FSR4.1.1 signed-provider case.

## Limits and follow-up

### Limits

- No repository ZIP/tree was available in this execution environment, so D011, the M6 synthesis, the native experiment implementation/results, and repository research-convention files were not independently inspected. Repository baseline statements in this record come from the supplied investigation brief only.
- No AMD GPU, Windows/DX12 runtime experiment, native trace, debugger capture, or provider instrumentation was used.
- No claim is made that the repository's native observations were reproduced.
- The signed FSR4.1.1 provider implementation is not exposed by the inspected public source path. Its allocator topology and reuse synchronization remain **Unresolved**.
- Absence of a completion rule from the inspected public corpus is a negative finding, not proof that no contractual or implementation constraint exists in proprietary/licensed material.
- Open-source implementation behavior is not itself a public ABI/safety promise unless AMD documents it as such.

### Evidence that would resolve remaining questions

Without selecting a policy, the unresolved signed-provider questions would require evidence not available from the public path inspected here, such as:

- provider implementation/source or authoritative AMD documentation stating transient allocator ownership and queue-depth/completion semantics;
- native/provider tracing that can distinguish per-context from shared transient allocations and detect reuse across recorded/submitted work;
- debugger/PDB-level inspection sufficient to identify provider-owned allocation instances and wrap state, if AMD's distributed debugging material exposes it;
- targeted Windows/DX12 experiments designed around actual ring reuse, sibling-context interaction, overlapping submissions, destruction, and cross-queue cases.

Those would be new evidence-gathering activities, not consequences that can be inferred safely from `FFX_MAX_QUEUED_FRAMES` or the existing recording-ahead observation.

## Narrow evidence summary

1. **Is D011's current concept of a shared `backend allocation domain` identifiable from public AMD evidence?**  
   **Partly.** For the **open DX12 backend**, the source-identifiable domain is one `FfxInterface` and its scratch-backed `BackendContext_DX12`; effect contexts hosted inside that interface share backend-wide descriptor/fallback-constant ring state. For the **selected signed FSR4.1.1 provider**, an equivalent domain is **Unresolved**. “Backend allocation domain” is not established as a public AMD API concept or contract.

2. **What is the narrowest transient-allocation ownership/sharing scope that the evidence actually establishes?**  
   **Verified for open source:** backend-wide transient ring state is per open DX12 backend-interface/scratch instance, not per `EffectContext`; persistent resource/bindless ranges have per-effect substate. **Verified for the open modern FSR3 provider:** each modern FSR3 context creates its own backend interface with `contexts = 1`, yielding independent fallback backend rings between sibling modern FSR3 contexts. **Unresolved for signed FSR4.1.1.**

3. **What does `FFX_MAX_QUEUED_FRAMES` establish, and what does it explicitly not establish?**  
   **Verified:** it is `4` in the pinned internal header and multiplies descriptor-ring and constant-buffer-ring capacity formulas, which are described for a single effect context before the open backend additionally scales capacity by `maxEffectContexts`. It does **not** establish a public supported number of recorded dispatches, submitted/incomplete dispatches, simultaneously executing dispatches, a fence cadence, a “wait every four” rule, or signed-provider behavior.

4. **Does AMD publicly specify any GPU-completion gate that `fsr-sdk-rs` must expose?**  
   No ordinary dispatch-count gate was found in the inspected public AMD sources. AMD does explicitly state for FSR3 context destruction that the GPU must no longer be accessing FSR3-created/used resources and recommends GPU idle before destruction. That is a destruction/lifetime condition, not a per-dispatch or four-dispatch rule.

5. **Does public evidence establish whether sibling modern-API contexts share the relevant transient storage?**  
   **For the open FSR3 3.1.5 provider: yes, they are independently provisioned** because each modern context creates and owns a one-context backend interface/scratch allocation. **For the signed FSR4.1.1 provider: Unresolved.** The public provider layer does not expose enough implementation to establish sharing or independence.

6. **Which parts remain unknowable for the signed provider path without additional native/provider evidence?**  
   The allocator owner, sibling-context sharing, exact ring/pool capacities, whether `FFX_MAX_QUEUED_FRAMES` participates at all, wrap/reuse logic, internal GPU-completion tracking, cross-queue assumptions, and precise lifetime of provider-owned objects referenced by previously recorded command lists all remain **Unresolved**.

7. **Which exact source facts should the maintainer carry back into the D011 decision?**  
   Carry only these facts, without turning them into a policy: (a) open-backend descriptor and fallback constant rings are owned at `FfxInterface`/`BackendContext_DX12` scope and use shared heads; (b) their capacities scale with `FFX_MAX_QUEUED_FRAMES = 4` and `maxEffectContexts`; (c) ring wrap has no explicit fence/completion check in the open backend; (d) CPU GPU-job/staging scratch is distinct from GPU-referenced descriptor/upload storage; (e) the open FSR3 modern provider provisions one backend interface per modern context with `contexts = 1`; (f) AMD's modern API defines dispatch as command encoding and no fixed ordinary completion gate was found; (g) FSR3 destruction requires safe GPU lifetime and recommends GPU idle; and (h) none of those open-source facts establishes the allocator-sharing scope or queue-depth contract of the signed FSR4.1.1 provider.

This evidence record intentionally stops there. It supplies facts and unresolved boundaries for D011; it does not select the final D011 policy.
