# Source investigation: AMD FSR SDK 2.4.0 Vulkan integration and D042 API compatibility

- **Investigation date:** 2026-10-29 (source access and evidence cut-off: 2026-10-29).
- **Topic and scope:** Whether AMD's official FidelityFX/FSR SDK **2.4.0** Vulkan API can be integrated into `fsr-sdk-rs` without redesigning its recently completed D042 upscaling API or breaking Windows/DX12 consumers; ABI provenance, platform support, safety, alternative architectures, migration and verification gates.
- **Project baseline:** Unpublished `fsr-sdk 0.3.0`, `fsr-sdk-sys 0.1.2`, Rust MSRV 1.88; pinned AMD SDK **v2.3.0** (`60f4ea81909200d8542eca14dccb2628b763a9a3`); verified native platform **Windows x64/MSVC + DX12**. D042's U07–U12 implementation is the integration baseline. D031 reserves Vulkan upscaling for wrapper v0.4.0; its Cargo feature intentionally fails in v0.3.0.
- **Question:** Can the current public API accommodate official Vulkan upscaling? What, if anything, must change **before** publishing v0.3.0, and how should v2.4.0/Vulkan be adopted?
- **Method:** Read the supplied repository's research template/process, current manifests, public and internal code, decisions, previous platform/licensing research and representative tests; compare version-pinned upstream GitHub **v2.3.0** and **v2.4.0** C headers, release notes, and AMD's published 2.4 documentation. Analyse interface obligations and alternatives; preserve negative findings. **No** modifications to production code, compilation, native loading, runtime query, hardware use or GPU experiments; no reliance on unpinned `main`.
- **Disposition:** Source-research findings and recommendations **only**; no ADR acceptance, release gate, new claimed platform support or change to the SDK pin.

## Inputs

All upstream links below were accessed **2026-10-29**. A URL to a tag identifies a source revision, **not** independently validated binary bytes.

| ID | Input and identity | Relevance / inspection level |
|---|---|---|
| U23 | [AMD SDK v2.3.0 tag](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0), commit [`60f4ea81909200d8542eca14dccb2628b763a9a3`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3) | Project's pinned DX12 ABI baseline; selected GitHub-rendered C headers inspected |
| U24 | [AMD FSR SDK v2.4.0 release](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.4.0), [tag](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.4.0), commit [`7539ae8b6568ad29ed78a753404351daf3b5cca7`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/7539ae8b6568ad29ed78a753404351daf3b5cca7) | Release dated **28 October 2026**, reported at 13:18 in GitHub UI (timezone not established); official release notes and pinned source |
| A23/A24 | [`ffx_api.h` v2.3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api.h) / [`v2.4`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.4.0/Kits/FidelityFX/api/include/ffx_api.h) | Common five entry points, descriptor conventions, backend IDs, version/query interfaces |
| T24 | [`ffx_api_types.h` v2.4](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.4.0/Kits/FidelityFX/api/include/ffx_api_types.h) and [v2.3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/ffx_api_types.h) | Shared resource and format types; selective inspection, **not** a mechanically exhaustive struct-by-struct diff |
| V24 | [`vk/ffx_api_vk.h` v2.4](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.4.0/Kits/FidelityFX/api/include/vk/ffx_api_vk.h) | Real Vulkan device/context descriptor, legacy variant, allocation callbacks and resource helpers. A corresponding `v2.3.0` new-API path was not found: do not mistake prior legacy Vulkan implementations for this new contract |
| D24 | [`dx12/ffx_api_dx12.h` v2.4](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.4.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h) and [v2.3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h) | Concrete DX12 counterpart; selected interface comparison, not an exhaustive change audit |
| S23/S24 | [`upscalers/include/ffx_upscale.h` v2.3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h) / [`v2.4`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.4.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h) | Shared upscaler creation, query and dispatch, version/flags and frame fields |
| G24 | [Official AMD FSR SDK page](https://gpuopen.com/amd-fsr-sdk/) | Published platform/hardware/driver matrix, DLL loading and version-selection policy; **upstream claims**, not reproduced GPU observations |
| N24 | [AMD's SDK 2.4 announcement](https://gpuopen.com/learn/amd-fsr-sdk-2-4-rx-6000-ryzen-apus/) | Release context and feature/driver distinctions; upstream claims |
| P | Local repo: `docs/research/PROCESS.md`, `docs/research/templates/source.md`, `docs/adr/{D015,D016,D031,D033,D040,D041,D042}.md`, `docs/research/platform-and-backends.md`, `docs/research/licensing-and-distribution.md`, `docs/ABI_COVERAGE.md`, and the crate manifests/source paths listed below | Actual supplied checkout, not assumed current upstream `main`; local file content directly inspected |

**Provenance limits:** GitHub rendered source/header and release-note pages were readable, but the complete v2.4.0 downloadable release payload and signed DLLs were **not** acquired or hashed. The release page advertises a ~200 MiB minimal package with signed DLLs and sample library sources; its dynamic asset listing did not provide a reliable per-file manifest in this inspection. Therefore no individual DLL file identity, Authenticode result, executable export table, binary license inventory or native behaviour is established here. Neither a local SHA-256 for upstream archives nor a complete SDK source-tree machine diff is claimed. Local checkout revision/hash was not independently pinned; identified file paths and version fields, rather than a fabricated checkout commit, are the local evidence identity.

Evidence labels throughout:

- **Verified:** directly inspected source, header, repository content or dated release metadata; *a verified header fact is not a verified runtime result*.
- **Upstream claim:** AMD-documented compatibility, requirements, binary content or behaviour not independently exercised.
- **Inference:** architectural consequence or design recommendation derived from those facts.
- **Unresolved:** insufficient source, artifact inspection or native execution evidence.

## Findings

### 1. Answers to the three decisions

1. **D042 public API is a sound foundation for an independent Vulkan upscaler, subject to Vulkan-specific native contracts.** **Inference.** The existing separation of portable options/frame parameters from DX12 setup/resources, and of setup/queries/creation from `record`, matches the shared FFX API and the Vulkan backend descriptor. **It is not a proof** that current DX12 resource inspection, ownership or GPU synchronization can be reused. Keep a concrete `upscaler::vk`/`vulkan` surface and internal sharing only where audited. [P, A24, V24, S24]
2. **No Vulkan-motivated breaking change to v0.3.0 is justified by current evidence.** **Inference.** Preserve DX12 names/signatures, the 2.3 pin, default feature and deliberate `vulkan` compile error for publication. The **nonadditive** v2.4 backend tag changes make an automatic v0.3 re-pin particularly unwise. Pre-release work, if desired, is a bounded **audit**, not API surgery: verify `Runtime` naming/documentation, cargo feature combinations, portable type dependencies, error extensibility and hidden DX12 assumptions. [P, A23/A24]
3. **Adopt SDK v2.4.0 on a separate v0.4.0 implementation/verification track:** pin tag and signed artifact hashes; review the common ABI delta; preserve the existing DX12 2.3 release line; add a concrete Vulkan backend plus compatible runtime acquisition; establish Windows/Vulkan compilation, loading, provider queries and actual GPU-output tests in separate stages; only then claim v0.4.0 Windows/Vulkan support. Treat Linux/Vulkan as a separate investigation without an official support claim. **Inference.** [U24, V24, G24, P]

These are research recommendations, **not** amendments to D031/D042.

### 2. Release identity, supported scope and SDK delta

**Verified:** The official release is `v2.4.0`, released on **2026-10-28**, with tag commit `7539ae8b6568ad29ed78a753404351daf3b5cca7`. The pinned project baseline is AMD `v2.3.0` commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. AMD publishes a minimal package described as signed runtime DLLs plus sample library source, without built samples/media; the release does not supply independently checked individual DLL identities in this investigation. [U23, U24]

**Upstream claim:** SDK 2.4 contains FSR Upscaling **4.1.2**, analytical Super Resolution **3.1.6** (also includes older **2.3.4**), ML Frame Generation **4.0.2**, analytical Frame Generation **3.1.6**, ML Frame Generation Swapchain **4.0.0**, analytical Swapchain Frame Generation **3.1.7**, Ray Regeneration **1.2.1**, and Radiance Caching **0.9.0 technical preview**. The release adds Vulkan to upscaling and frame-generation families; **Ray Regeneration and Radiance Caching remain DX12-only**. Claimed DX12/Vulkan *effect feature parity* is not a guarantee that native backend contracts, synchronization or hardware eligibility are interchangeable. [U24, G24]

| Dimension | AMD SDK v2.3.0 / wrapper v0.3.0 baseline | SDK v2.4.0 upstream evidence | Consequence |
|---|---|---|---|
| Backend API | Selected DX12 FFX API, wrapper Windows/DX12-only | New `ffx_api_vk.h` modern Vulkan backend with common FFX entry points | Add backend-specific FFI and resources, not a universal renderer object [A24, V24] |
| Upscaler | Shared effect descriptor/dispatch and checked wrapper model, release pin 2.3 | Header version `4.1.2`, analytical fallback `3.1.6` | Audit effect version/queries and provider differences; no reason to replace `FrameParameters` [S23/S24, G24] |
| Backend tag constants | `DX12=0x00000000`, `XBOX=0x01000000`, `VK=0x02000000` | **`DX12=0x00000000`, `VK=0x01000000`, `XBOX=0x02000000`** | **Not a purely additive ABI/constant delta**; backend-tagged chain constants must be selected from the exact version, not mixed [A23/A24] |
| Native export architecture | Five common FFX API operations | The same five named operations and descriptor-chain model remain visible | Retain shared call machinery *after* versioned signatures/layouts and loader are checked [A23/A24] |
| Debug | Existing global debug and configuration support | `ffxConfigureDescGlobalDebug1` deprecated in favour of `ffxConfigureDescGlobalDebug`; no effect removed | Check whether curated bindings/reference code rely on deprecated variant; no automatic DX12 API break [U24] |
| Effect DLL lookup | Prior project documented executable-directory/provider resolution | First already-loaded module, then executable directory, **then loader DLL directory** | Test actual DLL provenance/dependency resolution; avoid assuming all found effects come from desired SDK [G24, U24] |
| Driver providers | Queries, explicit provider IDs, override and default selection already exist | AMD documents driver-provided newer versions and explicit version override; driver-supplied name has ` *` suffix | Version/provider source must remain contextual to runtime/backend/device and each query; strict policy must not assume stable names or IDs [A24, G24] |
| GPU pipelines | Only DX12 verified by project | Vulkan 1.2 support documented for selected effects; Vulkan shader blob decompression added | New shader/runtime path requires native load and GPU evidence [U24, G24] |

**Verified source incompatibility worth isolating:** The `FFX_API_BACKEND_ID_VK` and `..._XBOX` macro values exchange positions between the two pinned common headers. The DX12 ID stays zero. This alone does **not** establish a broken SDK 2.3→2.4 DX12 binary interface; it does refute an unqualified claim that v2.4 only added tags. Whether any compiled v2.3 DX12 effect/descriptor was impacted needs a comprehensive definition diff and **binary** probe. Never pair a v2.4 Vulkan descriptor with v2.3 ID definitions or pre-2.4 tutorials. [A23/A24]

**Not established:** Exhaustive changed struct layouts, enum discriminants, packing and `sizeof`/`alignof` values, loader export implementations, native error distributions or backward binary compatibility. The visible five entry-point signatures are the **same form**, not evidence that all effect descriptor chains or provider protocols are identical. `fsr-sdk-sys` should remain a *curated* ABI slice rather than importing the entire v2.4 SDK. [A23/A24, T24]

### 3. Actual native Vulkan interface

#### 3.1 Device, backend and context creation

**Verified in `V24`:** The **current** Vulkan create descriptor uses:

```c
/* Simplified field-preserving excerpt; see pinned V24 for definitions. */
typedef struct VkDeviceContext {
    VkDevice vkDevice;
    VkPhysicalDevice vkPhysicalDevice;
    PFN_vkGetDeviceProcAddr vkDeviceProcAddr;
} VkDeviceContext;

#define FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_VK \
    FFX_API_MAKE_BACKEND_SUB_ID(FFX_API_BACKEND_ID_VK, 0x03)

struct ffxCreateBackendVKDesc {
    ffxCreateContextDescHeader header;
    VkDeviceContext deviceContext;
};
```

For **v2.4.0**, the current descriptor type evaluates to `0x01000003`; its device context has **three** Vulkan handles/function-pointer fields, not a DX12 COM device. A separate deprecated `ffxCreateBackendVKDesc_DEPRECATED` with tag **`0x00000003`** directly stores `VkDevice`, `VkPhysicalDevice`, `PFN_vkGetDeviceProcAddr`. The header says it remains for older FSR3 Vulkan compatibility. Do not expose the deprecated layout as the new wrapper's preferred API, and never interchange these two descriptor layouts based on C/Rust naming similarity. [V24, A24]

**Verified in `A24`, `S24`:** `ffxCreateContext`, `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, `ffxDispatch` are common named functions. A create chain joins an effect-specific upscaler descriptor, backend descriptor and optional version/override/callback descriptors. The FFX query version descriptor's `.device` field is documented as an `ID3D12Device` pointer on DX12 and a **`VkDeviceContext` pointer** on Vulkan; upscaler GPU memory-usage V2 query carries the same backend-specific meaning. Hence these queries are conceptually reusable, but a DX12 pointer cannot be cast to the Vulkan query payload. Backend/device binding and temporary storage lifetime must be audited anew.

**Inference:** A concrete Vulkan acquisition may take a valid `VkDeviceContext`-equivalent checked/unsafe wrapper, and can return a Vulkan-specific `UpscalerSetup` without changing the shared portable query result *types*. Device handles, proc addresses and loader lifetime belong to the concrete backend; creating a public `Upscaler<B>` or an object-safe `Device` trait adds no native safety. Since the current runtime is explicitly DX12-pinned, *load the correct loader* (`amd_fidelityfx_loader_vk.dll` per AMD documentation), rather than assuming passing a Vulkan device into a DX12 runtime suffices. [P, G24, V24]

#### 3.2 Resources, command buffers and recording

**Verified:** The upscaling dispatch remains `ffxDispatchDescUpscale` with an opaque `void* commandList`, a group of `FfxApiResource` inputs/output (`color`, `depth`, `motionVectors`, optional `exposure`, `reactive`, `transparencyAndComposition`, `output`) and scalar fields such as jitter, motion-vector scaling, timing, camera, sharpening/flags and render dimensions. It is a **shared FFX envelope**, not a native `ID3D12GraphicsCommandList` or `VkCommandBuffer` API typed by C. [S24]

**Verified:** The v2.4 Vulkan header has Vulkan-to-FFX format/resource-description helpers that accept `VkFormat`, and an image description helper accepting `VkImageCreateInfo` plus usage metadata. `ffxApiGetResourceVK(void*, FfxApiResourceDescription, uint32_t)` simply packages a caller-supplied pointer, description and state into `FfxApiResource`. It does **not** prove the image's current layout, ownership, format-view compatibility, contents, access rights or synchronization. In particular, a Vk image create-info describes creation, not necessarily the state when the upscaler reads/writes it. A public helper accepting arbitrary `void*` would invite unverified invariants. [V24, T24]

**Inference / required design:** Implement Vulkan-only resource/view/format/usage mapping and dispatch descriptor construction, with the same role-specific validation discipline as DX12 where Vulkan permits it. Explicitly account for resource image usage (`SAMPLED`/`STORAGE` as appropriate), depth/stencil aspects, image-view compatibility, subresource ranges, Vk image format and create flags, layouts, access/stage masks and queue-family ownership. Determine whether AMD's runtime expects image handles encoded as pointer values, precise initial/final layouts, particular states and internally managed transitions by **pinned source inspection and native testing** before defining the public recording contract. Do not invent a Vk image-view parameter solely to make it look like a DX12 resource builder; do not omit views where actual effect internals require them. [V24, S24, P]

**Unresolved:** The required `VkCommandBuffer` encoding and exact `.commandList` convention are not established merely by its `void*` C declaration; the sample/implementation is needed. Likewise, the observed header does not establish who makes which image layout transition or whether recorder rollback is legal after an error. The production Vulkan `record` must remain **unsafe**, as DX12 `record` is, unless actual end-to-end ownership/synchronization evidence supports narrower safe obligations. It must state separate pre-record, post-record, submission, abandonment, re-recording, teardown and GPU-completion duties, not assume `ffxDispatch` submission or a Rust borrow is a Vulkan memory barrier. [S24, P]

#### 3.3 Queues, callbacks and synchronization

**Verified:** `VkQueueInfoFFXAPI` in the new Vulkan header contains a `VkQueue`, family index and an optional queue-submit callback (`PFN_vkQueueSubmitFFXAPI`) described for concurrent submission. That is relevant especially to *frame generation/swapchain* paths; do not infer it is mandatory for ordinary upscaling context creation. The Vulkan backend allocation callback descriptor exists, but its resource allocator/deallocator functions are annotated **`TODO` / `NOT IMPL YET`**; a constant-buffer allocator function pointer is present. The declaration of a callback type is not evidence it is supported or safe to call. [V24]

**Inference:** Keep memory allocation and diagnostics interop backend-specific, with opt-in unsafe scope only when actual support is verified. Preserve common host callback retention/unwind restrictions, and establish Vulkan-specific calling convention, reentrancy, queue and external synchronization obligations. Require callbacks and any retained device/proc-address table to outlive native contexts, pending commands and exception/failed-destroy retention. [A24, V24, P]

**Unresolved:** Supported queue families, a mandatory compute queue versus graphics queue, timeline-semaphore or binary-semaphore protocol, shader module loading, required Vulkan feature bits/extensions, exact `VkQueueSubmitFFXAPI` use, the meaning of individual `FfxApiResource.state` bits on Vulkan, thread safety of driver providers and error/abandonment behaviour. Upstream's **Vulkan 1.2** supported API statement (and SPIR-V shader requirements described in SDK materials) is a starting point, not an exhaustive feature/extension list for all devices/effects. Verify against the exact backend implementation and sample before advertising a minimal Vulkan device profile. [G24, V24]

#### 3.4 Providers, errors and effect boundaries

**Verified:** The common version discovery/override path survives in the 2.4 headers, while AMD's 2.4 documentation describes driver-delivered versions and automatic newer-version selection unless explicitly pinned. It also states driver-provided version names carry a trailing ` *` and a version ID can be used in override. The lookup and availability are dependent on SDK, device, driver, backend and effect; nothing in these facts guarantees an ID is portable across DX12 and Vulkan or two process runs. [A24, S24, G24]

**Inference:** Keep `ProviderSelection` and `ProviderId` as portable *values*, but enumerate and validate providers for the **current Vulkan device/backend, runtime, effect and query call**. Preserve fresh membership checks and no fallback on explicit selection; do not cache provider identity globally, compare raw version names as stable keys, or silently promote a user-selected provider. Keep distinct failure provenance for loader, API return code, Vk resource inspection, Vulkan failure and context terminal state. Vulkan's native recoverability and terminal-usability classifications require verification: copying DX12 errors verbatim is not sufficient. [P, G24]

**Effect extensibility:** Standard upscaling has one context plus an application-recorded dispatch, congruent with D042. Frame Generation may require additional context, preparation/dispatch ordering and multiple queues; swapchain Frame Generation can interpose on presentation and swapchain ownership. Ray Regeneration/Radiance Caching are documented DX12-only in 2.4. No existing evidence justifies adding generic public effect or swapchain abstraction to the **upscaler** for v0.4. [G24, N24, V24]

### 4. Audit of existing Rust boundaries against Vulkan

The following references are to paths in the **supplied checkout**, not a future code design.

| Current source / contract | Evidence and concern | Decision |
|---|---|---|
| `crates/fsr-sdk/src/upscaler/common/{frame,options,scalars,camera,sharpening,...}.rs` | Portable `FrameParameters`, `UpscalerOptions`, motion-vector selection, checked dimensions and frame scalar semantics contain no DX12 COM resources. Native FFX shared dispatch fields match their intent. | **Keep** as shared Core; add no Vulkan fields merely for backend resources. Test each enum's native mapping on VK. [P, S24] |
| `upscaler/dx12/{setup,dispatch,interop}.rs` and `upscaler/dx12/dispatch/{resources,descriptor,frame}.rs` | Setup scope and separate unsafe recording already express backend context and renderer duties. But DX12 resource state/format/descriptor checking uses COM, DXGI and D3D12 interfaces. | **Parallel concrete Vulkan module**; share portable scalar/descriptor-planning logic only after auditing it for backend assumptions. [P, V24] |
| `crates/fsr-sdk/src/runtime.rs`, `runtime/state.rs` | `Runtime::load(path)` explicitly loads a Windows DX12 DLL, resolves five exports, retains library and native callback state. `Runtime` is not publicly backend-neutral despite its name. | **Preserve existing DX12 signature and semantics**. Add a backend-specific Vulkan loader/acquisition path; choose whether to share an internal state core after verifying its native invariants. [P, A24] |
| `upscaler/dx12/setup.rs`: `unsafe Runtime::upscaler_dx12(&ID3D12Device)`; `MovableRuntime` analogue | Concrete acquisition binds one native device and reachable-provider trust to a setup, followed by creation. | Vulkan counterpart with a typed Vulkan device context and separate safety contract; do not alter `upscaler_dx12` or reinterpret its argument. [P, V24] |
| `runtime/queries.rs` + `query` | Query result types and selection value are portable; query payload device pointer is not. | Reuse **results/policy**, not DX12 descriptor casts or provider guarantees. [P, A24, S24] |
| `error/backend.rs`, `error` | Owned backend error is `#[non_exhaustive]` and currently includes DX12 inspection data without Windows dependencies. | Add VK-specific variants/owned source only as observed; no retroactive DX12 error change needed. [P] |
| `upscaler/dx12.rs`, `dx12/movable.rs`, `runtime/movable.rs` | Local `!Send + !Sync` and opt-in movable `Send + !Sync` require audited arbitrary-thread native compatibility; neither guarantees GPU completion or mutual exclusion. | Begin with local Vulkan ownership. Gate movable VK separately until native threading/drop, queue and driver provider rules are established. [P, V24] |
| `crates/fsr-sdk/{Cargo.toml,src/lib.rs}`, `crates/fsr-sdk-sys/{Cargo.toml,src/lib.rs}` | Wrapper default `dx12`; sys default empty; `vulkan` reserves compile error; DX12/Windows imports and loader are `cfg(all(windows,feature="dx12"))`. `runtime` exported only under that cfg, so current portable use cannot load Vulkan alone. | In v0.4 gate runtime/VK separately from Windows/DX12; ensure `--no-default-features --features vulkan` needs no `windows` crate/DX12 exports. Preserve DX12-only and combined modes. No need to change failing feature in v0.3. [P] |
| `utils` policy modules | Produce portable provider/option values without owning the GPU or a backend. | Retain composability; do not impose Vulkan policy or automatic provider fallback. [P] |

#### Hidden DX12 assumptions requiring explicit audit

1. **DLL path and dependency identity.** A path to a valid DX12 loader is not a backend-independent runtime. `Runtime::load` docs explicitly promise DX12 v2.3 signatures and DLL provenance; changing this implicit contract could regress existing users. v2.4's effect search adds another directory and can resolve differently than v2.3. [P, G24]
2. **Query `device: void*`.** The Rust type alone looks generic, while its meaning is either a COM device pointer or a **pointer to `VkDeviceContext`**. No generic `device as *mut c_void` conversion until backend-specific lifetime and layout are reviewed. [A24, S24]
3. **Provider identity.** An `u64` remains a portable ID representation, **not** a globally meaningful provider key. Version override and default may be driver/backend-dependent, as already reflected by D042's fresh validation. [P, G24]
4. **Resource-format policy.** `FfxApiResourceDescription` can be shared at ABI level, but Vulkan `VkFormat`/usage/mutable-format mappings, aspect/mips and resource state may diverge from DXGI. Existing DX12 `TypedResourceFormat` and inspection must not leak into `common`. [P, T24, V24]
5. **Opaque command list.** `void*` does not mean `ID3D12GraphicsCommandList` everywhere; Vulkan command-buffer representation, state transitions and synchronization must be observed. [S24]
6. **Native coordination.** The existing process-wide unsafe serialization, callback lifetime, failed-destroy retention, and local/movable contracts should be **conservatively extended** to Vulkan; they must not be lifted because Vulkan handle values are `Copy`, `Send` elsewhere, or a shared Rust state is `Arc`. [P, V24]
7. **Error and terminal state.** A failed Vulkan record may leave GPU commands in an application-owned buffer; no safe automatic rollback or ordinary retry can be inferred from an FFX return code. [P]

### 5. Credible integration architecture alternatives

| Option | Compatibility | Safety/ownership and dependency isolation | Cost and recommendation |
|---|---|---|---|
| **A. Extend `Runtime` with concrete `upscaler_vulkan(...)`** | DX12 source signatures can remain unchanged **if** the existing `load(path)` contract stays DX12 and an explicit Vulkan loader constructor/path is added. | Backend discriminator and loader identity must be enforced internally; a single generic-sounding loader method that accepts either DLL by accident is unsafe. Current `runtime` Windows/DX12 gate must be revised. | Moderate, usable if naming/loader semantics stay explicit. Do not implement `upscaler_vulkan` on a DX12-only loaded runtime. |
| **B. Concrete backend-specific runtime acquisition; common portable Core (preferred starting design)** | Existing `Runtime`/`upscaler_dx12` remain exactly as v0.3. A `vulkan::Runtime`/`VulkanRuntime` or equivalent `Runtime::load_vulkan` can be added without altering DX12 callers. | Native ownership and loaded DLL match the backend by construction; VK setup/resource/record logic is isolated. Share private entry-point lookup and safe portable query/result machinery only after source/ABI audit. | **Smallest defensible proof-of-concept architecture** and the easiest to reason about. Select exact names after a small public-use example and v0.4 design review. |
| **C. Completely separate public DX12/VK contexts with no shared implementation** | Compatible. | Strong isolation, but redundant queries/lifecycle and opportunities for divergent semantics. | Appropriate **temporarily** for first spike; deduplicate only proven common invariants later. |
| **D. Public `Upscaler<B>`, generic GPU resource/queue traits or effect hierarchy** | Risks public D042 churn and unnecessary generic constraints. | Does not prove equivalent GPU resource ownership, queue sync or provider semantics. Generic context abstraction can obscure backend-specific unsafe guarantees. | **Reject for 0.3/0.4 absent new native evidence**; revisit only once multiple concrete effects demonstrate a stable common contract. |

**Recommended shape (Inference):** Keep D042's `UpscalerOptions`/`FrameParameters`/`ProviderSelection` and the local DX12 `Runtime::load` + `upscaler_dx12(...)` API. In v0.4 add one explicitly Vulkan-loaded runtime and backend-specific `UpscalerSetup`, `Upscaler`, resource bundle and unsafe `record`. Reuse portable query values, error envelopes and perhaps internally the five function pointers/context-storage patterns, but only where the actual VK ABI and lifetime model matches. Avoid a forced public `Runtime` rename or `Upscaler<B>` migration. If future evidence shows an internal runtime can safely hold either backend while preserving old `Runtime::load` exact behaviour, choosing A instead of B is an additive implementation decision, not a reason to block v0.3.

**Comparison of risk:** B offers easier DLL/device provenance, feature gating, local/movable separation and independent native tests; A may be slightly more ergonomic for programs using both backends but requires clearer state discrimination. D has an unproven generality benefit and highest breakage risk. None eliminates cross-backend *process-global* driver/runtime interactions: explicit non-reentrancy/external coordination may remain even with distinct Rust wrappers. [P, V24]

### 6. Platforms and runtime eligibility must be assessed separately

| Target | SDK 2.4 published support vs wrapper evidence | Claim currently justified |
|---|---|---|
| **Windows x64/MSVC + DX12** | AMD v2.4 supports DX12. Project has **existing native DX12 verification only with pinned v2.3.0**. | Existing v0.3.0 DX12 baseline only; upgrading its runtime to v2.4 needs new regression evidence. |
| **Windows x64/MSVC + Vulkan 1.2** | **Upstream claim:** officially supported for upscaling and named frame-generation effects. Vulkan GPU/driver/feature qualifications apply. | Suitable **first VK validation target**, **not yet verified by this project**. |
| **Linux x64 + native Vulkan** | **Unresolved / not an official support claim.** AMD's 2.4 supported OS list names Windows 10/11, not native Linux. DLL deployment and Windows-specific host/runtime assumptions remain. | No current native Linux support. Source build portability does not imply viable native loading or successful image output. |
| **Linux host + Proton/DX12** | Previously recorded host-driven DX12/Proton execution in repo; dispatches via DX12-on-Proton, not native Vulkan SDK loader. | **Does not establish** native Linux/Vulkan. |

**Upstream claim:** AMD lists Vulkan **1.2** for upscaling, analytical FSR 3.1.6, Frame Generation and swapchain frame-generation variants; the neural FSR Upscaling **4.1.2** path runs on qualifying AMD RX 6000/7000/9000 desktop-class discrete GPUs and specified Ryzen APUs, with analytical fallback on other eligible GPUs. Newly listed Vulkan/APU support requires **AMD Adrenalin 26.10.1** (released 2026-10-20) or later. ML Frame Generation 4.0.2 is more restrictive: Windows 11 and RX 9000 series, with fallback to analytical FG on other supported hardware. Do not treat driver 26.10.1, RX 9000, Windows 11 or DX12 Agility SDK as universal requirements for **all** Vulkan upscaling providers; determine eligibility per effect/provider/driver. [G24, U24]

**Cargo gate requirement (Inference):** A later native VK feature should compile with `--no-default-features --features vulkan` without importing DX12/`windows`; `dx12` and `vulkan` together should not create duplicate public `Runtime` types or incompatible native singleton assumptions. For `fsr-sdk-sys`, expose common C declarations without backend feature; make VK FFI/loader depend on `vulkan`, DX12 FFI/loader on Windows + `dx12`. On Windows, a `libloading` implementation is available, but changing it to an unconditional dependency would violate the current no-backend isolation. On Linux, a compile-only gate (with or without a `dlopen` mechanism) cannot override the Windows-only upstream support and binary provenance issues. [P, G24]

### 7. Safety, ABI, licensing and distribution consequences

**`fsr-sdk-sys` migration.** Capture a complete **v2.3→v2.4 audited inventory** before changing the raw crate pin: C entry-point signatures and calling conventions; every selected struct field, size, alignment and descriptor chain tag; `FFX_API_BACKEND_ID_*` reassignment; resource/format enums and states; result code/error aliases; versions and version-override payloads; provider count/fill query ownership, name storage and `VkDeviceContext` pointer; VK callbacks and loader exports. Produce layout/offset assertions or Windows C/Rust ABI comparisons for a minimal selected slice. Any additional native declarations must be annotated with the precise upstream version and retained notices. **Do not** mix headers from versions 2.3 and 2.4 in one tested backend contract. [A23/A24, V24, T24, P]

**Loading/provider provenance.** `fsr-sdk-sys` and the wrapper do not bundle AMD runtime DLLs in current crates. SDK 2.4's loader/effect DLL search order can select an already-loaded provider binary, then one next to the executable, then one next to the loader. A successful `LoadLibrary`/symbol lookup is therefore **not** verification of effect DLL revision, signing, driver provider or actual effect used; record exact resolved module paths, file hashes, signature and queried provider version for a native test. Do not imply a Rust wrapper can authenticate signed AMD binaries from an API success code. Avoid shipping user-downloaded or reconstructed substitutes for AMD's official binaries. [G24, U24, P]

**Licensing:** The wrapper is MPL-2.0; selected AMD headers contain MIT-style copyright/license notices, as does the inspected upscaler header. This does **not** grant blanket permission to repackage every v2.4 signed proprietary effect DLL, driver component, model or sample asset on identical terms. The repository's `docs/research/licensing-and-distribution.md` remains the existing policy baseline; **recheck the v2.4 release package's exact license/notices and AMD distribution conditions** before packaging or redistributing any binary. The source page's statement that signed DLLs are provided is not evidence of unrestricted redistribution. No v2.4 signed-DLL signatures or licensing attachments were verified in this investigation. [S24, U24, P]

**Crate version sequence:** Keep `fsr-sdk 0.3.0` + `fsr-sdk-sys 0.1.2` pinned to SDK v2.3.0. For v0.4, release an appropriately versioned raw crate with audited 2.4 backend declarations, and a wrapper compatible with it; decide exact semver bumps after a package API diff, not by assuming native SDK version equals crate version. Preserve DX12 feature semantics and tests; if 2.4 DX12 is required by the combined backend installation, **regress all DX12 behaviours against the new binary** before switching the crate's documented SDK pin. Published 0.3 installations must remain valid; do not make v0.4's Vulkan availability retroactively change v0.3's contract. [P, A23/A24]

### 8. Verification and migration sequence

These are **proposed gates**, not tasks already completed. Pass each evidentiary stage independently; recording a successful later stage must not erase earlier failures.

| Gate | Work / evidence to retain | What it would prove (and what it would not) |
|---|---|---|
| **G0 — Publication of v0.3.0** | Keep the pinned 2.3 DX12 baseline, accepted D042, default `dx12`, reserved failing `vulkan`. Review docs for explicit DX12-only `Runtime::load` semantics and audit no hidden DX12 dependencies in portable common modules; no implementation necessary. | Existing supported configuration remains documented; **no** Vulkan claim. |
| **G1 — SDK 2.4 provenance** | Acquire AMD official release asset and source tag; store archive/individual DLL SHA-256, digital signature/certificate, license notices, loader/effect names, build IDs, source commit, access date. Check linked dependencies and actual package contents. | Binary/source identity for future tests; **not** compatible execution. |
| **G2 — ABI/library audit** | Compare selected 2.3/2.4 header declarations *completely*; use C `static_assert` + Rust layout checks for selected C-compatible records on Windows/MSVC, symbol/export checks, correct new Vulkan tag. Record original/expected symbols and failure cases. | A defined ABI basis; **not** runtime/provider safety or correct GPU work. |
| **G3 — Configuration/gating build** | Build `fsr-sdk-sys` + wrapper `dx12` only, `vulkan` only (`--no-default-features`), `dx12,vulkan` together, portable/no backend, and optional `utils`,`movable` where relevant; run examples/tests and check Windows dependency trees. | Compilability and feature isolation on tested targets; **not** native Linux support or successful Vulkan context creation. |
| **G4 — Windows/Vulkan load and discovery** | On Windows 10/11 x64 with Vulkan 1.2 capable device + known physical device, driver version (include AMD 26.10.1+ when testing new ML VK), signed AMD VK loader/effect DLLs: load, resolve all five entry points, record actual resolved modules; query versions/count/name/IDs; select both default and explicit versions with supported and unsupported configurations. | Loader/provider/query behaviour of **that device+driver+runtime**. Still **not** verified native context, recording or image. |
| **G5 — Vulkan context + GPU execution** | Build descriptor chain using **modern** `ffxCreateBackendVKDesc`; create/destroy repeated contexts; test injected invalid tags/versions and failures; record real Vk commands into known-valid buffers; submit/synchronize with correct queue ownership/layout/access barriers; inspect validation layers; compare output pixel statistics/reference images and temporal behaviour. Include color/depth/motion, exposure, reactive/composition, sRGB, HDR/dynamic resolution/sharpening on supported providers. Retain test inputs, output hashes, GPU/driver/SDK IDs, commands and error traces. | Actual Vulkan recording and **correct output** only for tested effects/configurations. An FFX success code alone does not prove correctness. |
| **G6 — Lifecycle and cross-backend regression** | Concurrent/serialized DX12+VK module acquisition, runtime teardown while contexts retained, failed create/record/destroy, abandoned command buffers, GPU work in flight, debug callback lifetime, provider variation; DX12 native 2.4 regression against 2.3. Only then consider opt-in movable Vulkan and callback support. | Bounded safety/retention and compatibility claims for tested native domains; still not universal provider certification. |
| **G7 — Separate Linux investigation** | Establish an authorized **native Linux** runtime and legally distributable artifact first; test Linux loader/export/ABI and Linux `wchar_t`, Vulkan device/driver, execution and outputs independently. Clearly distinguish Wine/Proton from native implementation. | Only such data could support a separately qualified Linux/Vulkan claim; Windows Vulkan proof does not. |

**First useful Windows/Vulkan proof of concept** requires more than successful `cargo check`: exact signed 2.4 VK DLLs; a `VkDevice`/`VkPhysicalDevice` and valid `vkGetDeviceProcAddr`; correct `VkDeviceContext` descriptor ID and lifetime; a real provider query and upscaler context; application-owned Vk images with recorded format/usage/layout state; command-buffer recording; submit/wait and validation output; and an image readback demonstrably changed/correct across representative frames. A single successful context create/query or dispatch return is insufficient. Test a minimal analytical provider if needed for hardware eligibility, then eligible ML on a documented AMD device; do not conflate the two. [V24, S24, G24]

### 9. Baseline delta and release-sensitive choices

**Confirmed:** D042's portable values + concrete backend resource/recording boundary closely match AMD's shared effect API plus explicit Vulkan device backend. `Runtime::load` is currently *DX12-specific* despite its unqualified Rust name. `fsr-sdk-sys` is deliberately incomplete, and `vulkan` is not implemented in v0.3. The first official SDK 2.4 Vulkan support does not require renaming `FrameParameters`, making `Upscaler<B>`, or changing DX12 consumer signatures. [P, A24, V24, S24]

**Superseded:** Pre-2.4 project notes that no official current Vulkan backend exists are now historical; SDK **v2.4.0 exists** as of 28 Oct 2026. They are not evidence that the project already supports Vulkan. [U24, G24, P]

**Potentially conflicting with an “additive release” assumption:** The **actual backend ID macro reassignment** and a deprecated Vulkan descriptor layout necessitate a precise v2.4 ABI binding update, even if the DX12 ABI remains compatible on tested paths. The driver-upgrade mechanism and added DLL search directory also warrant renewed runtime/provenance assumptions. [A23/A24, V24, G24]

**Not contradicted:** D031's choice to defer until v0.4 and D042's reluctance to generalise GPU backend types are strengthened by these source findings. AMD's Windows-only published OS matrix leaves the Linux target explicitly **unverified** despite native Vulkan as a graphics API. [P, G24]

**Action before publishing v0.3.0:** No new Vulkan API and no SDK repin. A **read-only audit** of documentation/public names, common-module DX12 imports and feature combinations is justified and cheap; implement any unrelated genuine v0.3 corrections only under their existing decision process, not as speculative Vulkan preparation. A documented DX12-specific `Runtime::load` is not in itself a defect requiring a breaking rename. [P]

## Limits and follow-up

### Unresolved questions, prioritized

1. **Highest — Full 2.3→2.4 ABI diff:** Besides the confirmed VK/Xbox ID reassignment, are any selected common resource enums, structs, packing, error codes or DX12 descriptors changed? Verify with exact headers and C/Rust ABI tests before migrating `fsr-sdk-sys`.
2. **Highest — Vulkan command buffer and resource encoding:** What exact handle representation does `.commandList` accept? What do `ffxApiGetResourceVK`, `FfxApiResource.state` and the VK backend require for Vk images, image views, subresources, layout transitions, and barriers? Check exact v2.4 implementation/examples *and* execute a minimal real GPU case.
3. **Highest — Binary provenance:** What actual DLL filenames, signatures, package SHA-256s and distribution terms are in the official downloadable **2.4.0** archive? Which transitive dependencies and runtime/driver versions actually load?
4. **High — Context/query provider behaviour:** Which provider IDs and version override codes appear on Vulkan vs DX12 on the same GPU? Does version count/fill, default version selection and lifetime mirror the documented logic for 2.4? Measure, don't extrapolate.
5. **High — GPU synchronization and failure:** Which resource layouts/queue capabilities does AMD's VK upscaler require? What is the state of an application command buffer on partial failure? When can `request_tuning`, destruction and callbacks be safely invoked after in-flight work?
6. **High — Vulkan-only feature graph:** Can `fsr-sdk --no-default-features --features vulkan` build/run on Windows without DX12 dependencies, and can both backends be loaded in one process without unwanted cross-effects? Determine actual target support through builds.
7. **High — Native threading:** Can VK loading, context operations, teardown and any callback be moved to arbitrary threads for an opt-in `Send + !Sync` family? Existence of Vulkan dispatchable handles is no proof. Initial local-only support is defensible.
8. **Medium — Effect scope:** Which specific device extensions/SPIR-V capabilities, image usage and queue families are required per upscaler version/provider? What changes for swapchain FG? Avoid adding unsupported future abstractions.
9. **Medium — Native Linux:** Is there an actual AMD-authorized Linux runtime/loader and license path? How do calling convention, wide strings, dynamic library loading and driver dependencies behave? A Proton DX12 test does not answer this.
10. **Medium — Packaging:** Does a revised AMD license/notice or signed-binary redistribution condition alter current source-only crate publishing? Check exact 2.4 release bytes before any new binary shipment.

### Evidence ladder and nonclaims

1. **Header/source inspection:** accomplished for selected pinned headers and local code; establishes declaration facts only.
2. **Host compilation:** **not performed** for SDK 2.4/Vulkan.
3. **Native signed-library loading:** **not performed**.
4. **Provider queries and context creation:** **not performed**.
5. **GPU command recording and submission:** **not performed**.
6. **Correct reconstructed GPU output, lifecycle and repeated-frame behaviour:** **not performed**.

No full release artifact manifest, native Vulkan safety guarantee, supported Linux SDK, universal driver/provider guarantee, ABI equivalence proof, or passed GPU validation is claimed. Release notes announcing “feature parity” and an exposed Vulkan C structure cannot substitute for these separate observations. Any native GPU probe is a separately authorised experiment and should be retained as `exp` research with exact host/GPU/driver/runtime identities and observed output.

**Recommendation for maintainer decision:** Publish v0.3.0 with D042 and SDK 2.3.0 intact. Start v0.4.0 with audited v2.4.0 declarations and a concrete **Windows/Vulkan upscaler**, favoring backend-specific runtime acquisition and portable common types. Do not preemptively redesign D042, claim native Linux, enable the reserved feature, silently amend ADRs or repin the existing release based on this research alone.
