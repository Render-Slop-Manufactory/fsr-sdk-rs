# Provenance repair: lifecycle and fsr2-rs source records

- **Date:** 2026-09-22
- **Method:** GitHub release metadata, commit objects, commit history, commit patches, commit-pinned blob/raw files, and parent-repository submodule history. Branch-floating `master`/`main` content was not used as final provenance.
- **Scope:** Provenance repair only. The two research-record files themselves were not available in this conversation and were not discoverable publicly by filename, so the checks below cover the entries and minimum representative source set specified in the brief rather than a line-by-line audit of every sentence in those records.

## 1. FidelityFX v2.3.0 source URL repair

AMD's `v2.3.0` release identifies tag `v2.3.0` with commit `60f4ea8`; the full commit object is `60f4ea81909200d8542eca14dccb2628b763a9a3`. The release also identifies AMD FSR Upscaling 4.1.1, consistent with the research baseline.

| Existing entry | Verified v2.3.0 path | Commit/tag | Exact URL | Result |
|---|---|---|---|---|
| Release/tag metadata | release `v2.3.0` | `60f4ea81909200d8542eca14dccb2628b763a9a3` / `v2.3.0` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0); commit: [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3) | **Verified.** Release/tag and commit agree. |
| `ffx_api.h` | `Kits/FidelityFX/api/include/ffx_api.h` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h) | **Verified.** Contains `ffxCreateContext`, `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, `ffxDispatch`, including the descriptor-pointer lifetime statement. |
| `ffx_api_dx12.h` | `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h) | **Verified.** Contains `FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12` and `ffxCreateBackendDX12Desc`. |
| `ffx_upscale.h` | `Kits/FidelityFX/upscalers/include/ffx_upscale.h` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h) | **Verified.** Contains `ffxCreateContextDescUpscale` and `ffxDispatchDescUpscale`. |
| `ffx-api.md` | `Kits/FidelityFX/docs/getting-started/ffx-api.md` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md) | **Verified.** Documents context creation/destruction and query/configure/dispatch/version-selection flow. |
| `version_2_1_0.md` | `Kits/FidelityFX/docs/whats-new/version_2_1_0.md` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/whats-new/version_2_1_0.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/whats-new/version_2_1_0.md) | **Verified.** Historical API/version-transition material is present in the v2.3.0 tree. |
| `ffx_api.cpp` | `Kits/FidelityFX/api/internal/ffx_api.cpp` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp) | **Verified.** `ffxCreateContext` selects a provider and delegates creation; destruction recovers the associated provider and delegates destruction. |
| `ffx_provider.h` | `Kits/FidelityFX/api/internal/ffx_provider.h` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h) | **Verified.** Defines virtual `CreateContext`/`DestroyContext`, `InternalContextHeader`, `GetAssociatedProvider`, and provider selection. |
| `ffx_backends.h` | `Kits/FidelityFX/api/internal/ffx_backends.h` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_backends.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_backends.h) | **Verified.** Contains `CreateBackend`/`MustCreateBackend`; the path as recorded is correct. |
| `ffx_provider_fsr3upscale.cpp` | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp) | **Verified.** Its provider `CreateContext` allocates the internal context, associates the provider, creates the backend, and initializes FSR3 upscale state. |
| `ffx_provider_fsr2.cpp` | `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp) | **Verified.** Contains FSR2 provider `CreateContext` and `DestroyContext`, including backend creation and `ffxFsr2ContextCreate`. |
| Frame Generation comparison source | `Kits/FidelityFX/docs/techniques/frame-interpolation-api.md` | same | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md) | **Verified.** This is the exact adjacent v2.3.0 document containing the explicit thread-safety wording. |

For raw-content provenance, the corresponding immutable raw form is obtained by replacing the GitHub `blob/<SHA>/` prefix with `raw.githubusercontent.com/.../<SHA>/`. For example, `ffx_api.h` is [https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h?utm_source=chatgpt.com). The same transformation is valid for every file row above.

The two already-populated entries also exist at the same immutable revision: `Kits/FidelityFX/api/include/ffx_api.hpp` contains the C++ `CreateContext`/`DestroyContext` wrappers, and `Kits/FidelityFX/docs/techniques/super-resolution-ml.md` is the FSR Upscaling 4.1.1 document. Their existing URL strings were not supplied, so they should be retained unchanged if already pinned to `v2.3.0` or `60f4ea819...`; there is no evidence requiring a substantive citation change.

### Frame Generation comparison source

**Exact document:** `Kits/FidelityFX/docs/techniques/frame-interpolation-api.md`  
**Exact revision:** `60f4ea81909200d8542eca14dccb2628b763a9a3` (`v2.3.0`)  
**Relevant section/symbol:** `Thread safety`; specifically the synchronization requirements covering Frame Generation context APIs including context creation/destruction and dispatch operations.  
**Why it matches the historical claim:** The document explicitly states that the underlying context is not guaranteed thread-safe and that specified public operations require external synchronization. That makes it the appropriate contemporaneous comparison source for a lifecycle record distinguishing explicit Frame Generation synchronization language from more general FSR API lifecycle documentation.

## 2. fsr2-rs revision reconstruction

**Repository:** [https://github.com/NotAPenguin0/fsr2-rs](https://github.com/NotAPenguin0/fsr2-rs?utm_source=chatgpt.com)

**Historical revision investigated:** `3a928595f46818f7221f0ca28f00770b53ded850` is the strongest defensible reconstruction of the `master` state inspected on 2026-09-20. Classification: **Inference, high confidence**, not direct historical verification.

**Commit date:** GitHub groups the commit under 2023-05-24; its patch carries `Date: Wed, 24 May 2023 21:41:44 +0200`.

**Commit URL:** [https://github.com/NotAPenguin0/fsr2-rs/commit/3a928595f46818f7221f0ca28f00770b53ded850](https://github.com/NotAPenguin0/fsr2-rs/commit/3a928595f46818f7221f0ca28f00770b53ded850)

**Tree URL:** [https://github.com/NotAPenguin0/fsr2-rs/tree/3a928595f46818f7221f0ca28f00770b53ded850](https://github.com/NotAPenguin0/fsr2-rs/tree/3a928595f46818f7221f0ca28f00770b53ded850?utm_source=chatgpt.com)

**Evidence:** GitHub's `master` commit history currently has `3a92859` as its top commit and contains 36 commits in total; all listed commits predate the 2026-09-20 investigation. The historical record independently recorded `master` and “36 commits” on that date. The full SHA behind the top commit is `3a928595f46818f7221f0ca28f00770b53ded850`.

**Confidence / limitations:** This establishes the current `master` object and provides strong evidence that it is the same revision seen on 2026-09-20. It does not cryptographically prove the value of the branch ref on that date: a later force-push followed by restoration could leave today's history indistinguishable. Accordingly, the old “exact SHA unresolved” statement can be tightened substantially, but historical branch equivalence should remain an **Inference**, not be upgraded to **Verified**.

## 3. Native FidelityFX-FSR2 submodule revision

**Parent fsr2-rs commit:** `3a928595f46818f7221f0ca28f00770b53ded850`

**Submodule path:** `fsr2-sys/src/vendor/fsr2`

**.gitmodules URL:** [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/.gitmodules](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/.gitmodules)

**Gitlink SHA:** `35d136728c49b5c866517b906d4405b7bee583da`

**Resolved child commit URL:** [https://github.com/NotAPenguin0/FidelityFX-FSR2/commit/35d136728c49b5c866517b906d4405b7bee583da](https://github.com/NotAPenguin0/FidelityFX-FSR2/commit/35d136728c49b5c866517b906d4405b7bee583da)

**Child commit date:** 2023-05-24 according to the child repository's GitHub commit history.

**Evidence:** The pinned `.gitmodules` maps `fsr2-sys/src/vendor/fsr2` to `https://github.com/NotAPenguin0/FidelityFX-FSR2`. More importantly, parent commit `6f6909a2ff81e44b692918caeea7b87ead3e0196` changes the mode-160000 gitlink at that exact path from `750b60b09ae135975f3f60589c1db1f7b30e1f57` to `35d136728c49b5c866517b906d4405b7bee583da`. Inspection of the subsequent parent commits through `3a928595...` found no later gitlink modification.

This relationship is **Verified** from the parent repository history. The fact that the child repository's current `master` also starts at `35d1367` is corroboration, not the basis for the provenance assignment.

## 4. Branch-floating citation verification

| Existing cited file | Pinned repository | Pinned commit | Content matches historical claim? | Notes |
|---|---|---|---|---|
| `fsr2-sys/src/lib.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Defines FSR2 version `2.2.0` and context size `16536`, together with create/dispatch/destroy FFI declarations. |
| `.gitmodules` | `fsr2-rs` | `3a928595...` | **Yes** | Exact path and NotAPenguin0/FidelityFX-FSR2 URL confirmed. |
| `fsr2-sys/Cargo.toml` | `fsr2-rs` | `3a928595...` | **Yes** | `fsr2-sys` 0.1.6; backend feature declarations are present. |
| `fsr2-sys/build.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Uses `src/vendor/fsr2`, CMake/Release configuration and backend feature selection. |
| `fsr2-sys/src/error.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Rust error-code representation matches the pinned native error definitions. |
| `fsr2-sys/src/interface.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Contains the backend-interface callback/function-pointer bindings represented by the native interface header. |
| `fsr2-sys/src/types.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Pinned types include the resource definitions and public null-resource constant. |
| `fsr2-sys/src/backend/vk.rs` | `fsr2-rs` | `3a928595...` | **Yes** | Vulkan function-pointer table and FSR2 Vulkan wrapper bindings are present, including context-aware texture/buffer conversion. |
| `fsr2-sys/src/backend/dx12.rs` | `fsr2-rs` | `3a928595...` | **Yes** | File is empty at the pinned revision; this supports, rather than changes, the historical contrast about missing Rust DX12 backend bindings. |
| `src/ffx-fsr2-api/ffx_fsr2.h` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Native macros identify FSR2 `2.2.0` and context size `16536`; lifecycle declarations are present. |
| `src/ffx-fsr2-api/ffx_error.h` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Native `FfxErrorCode` constants are present. |
| `src/ffx-fsr2-api/ffx_fsr2.cpp` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Contains `ffxFsr2ContextCreate`, `ffxFsr2ContextDestroy`, and `ffxFsr2ContextDispatch` implementations. |
| `src/ffx-fsr2-api/ffx_fsr2_interface.h` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Defines the backend interface callback contract, including backend create/destroy/resource/job callbacks. |
| `src/ffx-fsr2-api/vk/ffx_fsr2_vk.h` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Declares `ffxFsr2GetInterfaceVK`, the Vulkan function-pointer table, and Vulkan resource conversion helpers. |
| `src/ffx-fsr2-api/vk/ffx_fsr2_vk.cpp` | `FidelityFX-FSR2` | `35d136728...` | **Yes** | Implements `ffxFsr2GetInterfaceVK` and populates the native backend callback table. |

For this tested set, the newly pinned parent/child objects contain the expected material. Because today's `master` refs point to the same parent and child commits, replacing the floating URL component with these SHAs changes provenance precision, not the referenced file content. Historical equality of the parent `master` ref on exactly 2026-09-20 remains the high-confidence inference described in §2.

## 5. Required corrections to the historical records

### 2026-09-20-src-context-lifecycle.md

Use full commit `60f4ea81909200d8542eca14dccb2628b763a9a3` for the AMD baseline. Replace the empty provenance fields as follows:

| Entry | Replacement `URL:` |
|---|---|
| v2.3.0 release/tag metadata | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0) and, for immutable object provenance, [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3) |
| `Kits/FidelityFX/api/include/ffx_api.h` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h) |
| `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h) |
| `Kits/FidelityFX/upscalers/include/ffx_upscale.h` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h) |
| `Kits/FidelityFX/docs/getting-started/ffx-api.md` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md) |
| `Kits/FidelityFX/docs/whats-new/version_2_1_0.md` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/whats-new/version_2_1_0.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/whats-new/version_2_1_0.md) |
| `Kits/FidelityFX/api/internal/ffx_api.cpp` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp) |
| `Kits/FidelityFX/api/internal/ffx_provider.h` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h) |
| `Kits/FidelityFX/api/internal/ffx_backends.h` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_backends.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_backends.h) |
| `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp) |
| `Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp` | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp) |
| Frame Generation synchronization comparison | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/frame-interpolation-api.md) |

No path correction is required for any of those entries.

### 2026-09-20-src-fsr2-rs.md

Replace **Repository revision/commit inspected** with:

> **Inference — high confidence:** `NotAPenguin0/fsr2-rs`, commit `3a928595f46818f7221f0ca28f00770b53ded850`. This is the current `master` HEAD and is the strongest defensible reconstruction of the revision inspected on 2026-09-20: the historical record observed `master` with 36 commits, and GitHub's present `master` history contains the same 36-commit history with `3a92859` as the newest commit, dated 2023-05-24. Exact historical branch-ref identity is not independently time-stamped and therefore remains an inference. Commit: [https://github.com/NotAPenguin0/fsr2-rs/commit/3a928595f46818f7221f0ca28f00770b53ded850](https://github.com/NotAPenguin0/fsr2-rs/commit/3a928595f46818f7221f0ca28f00770b53ded850).

Replace **Native source revision** with:

> **Verified:** At parent revision `3a928595f46818f7221f0ca28f00770b53ded850`, submodule path `fsr2-sys/src/vendor/fsr2` resolves through the parent gitlink to `NotAPenguin0/FidelityFX-FSR2` commit `35d136728c49b5c866517b906d4405b7bee583da`. `.gitmodules` maps that path to the NotAPenguin0 repository, and parent history directly records the mode-160000 gitlink update to that exact object. Child commit: [https://github.com/NotAPenguin0/FidelityFX-FSR2/commit/35d136728c49b5c866517b906d4405b7bee583da](https://github.com/NotAPenguin0/FidelityFX-FSR2/commit/35d136728c49b5c866517b906d4405b7bee583da).

The old statement that the **native gitlink SHA is unresolved** can therefore be removed. The old statement that the **parent HEAD SHA is unresolved** should be replaced by the high-confidence **Inference** above rather than by an unconditional `Verified` classification.

For the branch-floating raw URLs, replace `/master/` with the appropriate repository object's SHA:

| File | Commit-pinned replacement |
|---|---|
| `fsr2-sys/src/lib.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/lib.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/lib.rs) |
| `.gitmodules` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/.gitmodules](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/.gitmodules) |
| `fsr2-sys/Cargo.toml` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/Cargo.toml](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/Cargo.toml) |
| `fsr2-sys/build.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/build.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/build.rs) |
| `fsr2-sys/src/error.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/error.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/error.rs) |
| `fsr2-sys/src/interface.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/interface.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/interface.rs) |
| `fsr2-sys/src/types.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/types.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/types.rs) |
| `fsr2-sys/src/backend/vk.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/backend/vk.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/backend/vk.rs) |
| `fsr2-sys/src/backend/dx12.rs` | [https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/backend/dx12.rs](https://raw.githubusercontent.com/NotAPenguin0/fsr2-rs/3a928595f46818f7221f0ca28f00770b53ded850/fsr2-sys/src/backend/dx12.rs) |
| `src/ffx-fsr2-api/ffx_fsr2.h` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2.h](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2.h) |
| `src/ffx-fsr2-api/ffx_error.h` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_error.h](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_error.h) |
| `src/ffx-fsr2-api/ffx_fsr2.cpp` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2.cpp](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2.cpp) |
| `src/ffx-fsr2-api/ffx_fsr2_interface.h` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2_interface.h](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/ffx_fsr2_interface.h) |
| `src/ffx-fsr2-api/vk/ffx_fsr2_vk.h` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/vk/ffx_fsr2_vk.h](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/vk/ffx_fsr2_vk.h) |
| `src/ffx-fsr2-api/vk/ffx_fsr2_vk.cpp` | [https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/vk/ffx_fsr2_vk.cpp](https://raw.githubusercontent.com/NotAPenguin0/FidelityFX-FSR2/35d136728c49b5c866517b906d4405b7bee583da/src/ffx-fsr2-api/vk/ffx_fsr2_vk.cpp) |

## 6. Claims affected by provenance differences

No material discrepancy was found in the representative claims tested. Pinning changes provenance precision but does not change the substantive findings: the Rust side is FSR2 2.2.0-era binding material, the native submodule is the corresponding NotAPenguin0 fork object, the Vulkan bindings/backend sources align, and `fsr2-sys/src/backend/dx12.rs` is empty at the pinned parent revision.

The only classification change warranted is provenance-specific: the native submodule SHA moves from **Unresolved** to **Verified**; the reconstructed parent `master` SHA moves from **Unresolved** to **Inference — high confidence**, not to direct historical **Verified**.

## 7. Remaining unresolved provenance

1. **Exact historical `master` ref value on 2026-09-20:** no dated Git ref snapshot/reflog or equivalent archived branch-ref evidence was available. `3a928595f46818f7221f0ca28f00770b53ded850` is strongly supported by the unchanged 36-commit history, but strict historical branch-ref identity remains an **Inference**.
2. **Already-populated URLs in Record A:** the actual URL strings currently stored for `ffx_api.hpp` and `super-resolution-ml.md` were not supplied. Both sources are verified at `60f4ea819...`; whether their existing textual URLs already use that commit/tag cannot be checked from the brief alone.
3. **Full prose-level Record B audit:** because the historical record itself was not available, this repair verifies the complete minimum source set requested and the provenance relationships, but cannot assert that unquoted claims elsewhere in the file were individually compared against the pinned sources.
