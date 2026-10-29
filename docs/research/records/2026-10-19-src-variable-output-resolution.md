# Source-research record: Variable output resolution in AMD FidelityFX SDK 2.3.0

**Investigation date:** 19 October 2026  
**Status:** Source investigation; no implementation changes or native/GPU experiments  
**Target:** `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`, Windows x64/MSVC + DirectX 12, Rust MSRV 1.88

## 1. Scope, pinned inputs and method

The investigation distinguishes three independent concepts:

- **Context capacity:** `maxRenderSize` and `maxUpscaleSize`, fixed at context creation.
- **Resource allocation:** actual texture dimensions, supplied through resource descriptions.
- **Active image:** the region processed during a dispatch, specified by `renderSize` and `upscaleSize`.

The pinned upstream source is [FidelityFX-SDK commit `60f4ea81909200d8542eca14dccb2628b763a9a3`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3), selected as **SDK 2.3.0**. Its public upscaler header declares **upscaler API 4.1.1**. These versions are distinct from individual provider implementation versions.

The method is static inspection of the pinned public API, open FSR3 implementation, provider and backend code, shader sources and the supplied Rust project. Header declarations, executable validation, shader behaviour and implementation assumptions are treated as different grades of evidence.

Every substantive conclusion is classified as:

- **Verified source fact:** directly established by inspected code or declarations.
- **Upstream claim:** documented behaviour not independently established by the inspected implementation.
- **Inference:** a consequence or recommendation derived from source evidence.
- **Unresolved:** behaviour requiring further evidence, including closed-provider or GPU verification.

### 1.1 Initial contract established by the public API

**Verified source fact.** [`ffx_upscale.h`, lines 127–185](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L127-L185) declares:

- `ffxCreateContextDescUpscale::maxRenderSize`: maximum rendering dimensions.
- `ffxCreateContextDescUpscale::maxUpscaleSize`: target presentation dimensions.
- `ffxDispatchDescUpscale::renderSize`: dimensions used to render the inputs.
- `ffxDispatchDescUpscale::upscaleSize`: dimensions requested for upscaling, explicitly optional and defaulting to `maxUpscaleSize`.
- `ffxDispatchDescUpscale::output`: output colour resource at presentation resolution.

Consequently, **the native public ABI already exposes variable output resolution**. A wrapper that invariably supplies maximum output dimensions omits a documented dispatch capability. The header does not, however, establish which output allocation sizes are compatible with smaller active regions.

## 2. Native dispatch contract and provider differences

### 2.1 Zero, maxima and dimensional relationships

**Verified source fact.** The public declaration is at [`ffx_upscale.h:68–103`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L68-L103). The relevant native path is:

`ffxDispatchDescUpscale` → `ffxProvider_FSR3Upscale::Dispatch` → `ffxFsr3UpscalerContextDispatch` → `fsr3upscalerDispatch`.

The [FSR3 adapter, lines 321–345](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L321-L345), forwards both dimensions without modification. The [FSR3 public dispatch entry, lines 1201–1230](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1201-L1230), enforces four independent upper bounds:

\[
0\le R_x\le R_{\max,x},\qquad
0\le R_y\le R_{\max,y},\qquad
0\le U_x\le U_{\max,x},\qquad
0\le U_y\le U_{\max,y}.
\]

Exceeding any maximum returns `FFX_ERROR_OUT_OF_RANGE`. These comparisons are **not sufficient validity conditions**: they admit zero, and they do not compare dimensions with resource allocations.

The internal [defaulting logic, lines 870–885](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L870-L885), behaves as follows:

| Supplied `upscaleSize` | Open FSR3 behaviour |
|---|---|
| `(0, 0)` | Uses both components of `maxUpscaleSize`. |
| `(positive, positive)` | Uses the supplied components. |
| `(0, positive)` or `(positive, 0)` | Does **not** default the zero component; a debug warning exists, but ordinary upper-bound validation does not reject it. |

**Inference.** The wrapper should represent the default as `None`, resolve it to the context maximum and reject partially zero dimensions through its validated `Dimensions` type. Zero is not a useful active output extent: the native implementation divides by active dimensions when constructing scaling and motion constants. The absence of a rejection is not evidence of safety.

**Verified source fact.** Neither the public dispatch declaration nor the examined FSR3 dispatch validator enforces equal render/output aspect ratios, a particular upscale factor, `renderSize <= upscaleSize`, or divisibility by 8 or 16. Quality-mode ratios describe intended scaling modes, not an explicit general dispatch validation rule.

**Unresolved.** These checks do not establish that arbitrary aspect ratios, downscaling, exceptionally small dimensions or unaligned extents execute correctly on every provider. Such cases remain outside a proven cross-provider contract.

### 2.2 Provider routing is material

**Verified source fact.** The public API version is **4.1.1**; the bundled open FSR3 provider identifies its implementation as **3.1.5** in [`ffx_fsr3upscaler.h:32–44`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h#L32-L44).

The [FSR2 adapter, lines 251–284](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp#L251-L284), translates `renderSize` but **does not forward `upscaleSize`**. Consequently, the common public field cannot be assumed to provide equivalent variable-output behaviour across even the two open adapters.

On DX12, [`GetProvider`, lines 95–160](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h#L95-L160), can select a driver-side provider rather than the bundled implementation, while an explicit version override changes selection. The driver-side implementation is not established by the open FSR3 code.

**Unresolved.** A closed provider may impose additional extent, allocation, temporal-history or state requirements. Successful open-FSR3 validation must not be described as SDK-wide provider compatibility.

## 3. External output allocation and active geometry

### 3.1 Resource registration versus active output dimensions

**Verified source fact.** [`fsr3upscalerDispatch`, lines 802–850](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L802-L850), registers the caller's output as an external UAV. It does not compare the output resource description with active `upscaleSize`.

The [DX12 backend, lines 1959–1999 and 2037–2067](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L1959-L2067), creates views from the underlying `ID3D12Resource` but stores the supplied `FfxApiResourceDescription.width` and `.height` in its resource registry. Therefore, the wrapper must report **actual allocation dimensions**. Substituting the active output dimensions into that description would misrepresent the resource to callbacks and backend consumers.

**Verified source fact.** Internal textures are sized independently at context creation: accumulation and related render-side textures use `maxRenderSize`, while `newLocks` and the internal upscaled colour history use `maxUpscaleSize` ([`ffx_fsr3upscaler.cpp:467–488`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L467-L488)). Thus, permitting different external allocations does not imply that the native implementation reallocates its internal history per dispatch.

### 3.2 Allocation/active-size contract matrix

Let \(A\) be the actual external output allocation, \(U\) the active output and \(M\) the context's `maxUpscaleSize`. Comparisons are component-wise.

| Configuration | Open FSR3 CPU path | Source-level assessment |
|---|---|---|
| \(A=U=M\) | Within maximum; no allocation-equality objection | Established baseline; GPU behaviour still depends on valid resources/states. |
| \(A=U<M\) | Accepted by the inspected extent checks | **Candidate**, not GPU-verified; rounded shader dispatches matter. |
| \(A=M,\ U<M\) | Accepted by the inspected extent checks | **Candidate**; unused allocation pixels are not guaranteed untouched. |
| \(U\le A\le M\), arbitrary intermediate \(A\) | No direct rejection found | **Candidate**, subject to shader accesses and provider verification. |
| \(U\le M<A\) | No maximum-allocation comparison found | **Unresolved** compatibility; not natively rejected merely because \(A>M\). |
| \(A<U\le M\) | No direct CPU rejection found | **Reject in the wrapper**: the requested output exceeds its backing texture. |
| \(U>M\), regardless of \(A\) | `FFX_ERROR_OUT_OF_RANGE` | **Rejected by open FSR3**. |
| One zero output dimension | Upper-bound validation may pass | **Reject in the wrapper**; unsafe arithmetic/sampling assumptions. |
| Both output dimensions zero | Defaults to \(M\) | Supported default syntax; effective \(U=M\), so allocation must be assessed against \(M\). |

Here, *candidate* means that the inspected native CPU path does not reject the configuration. It does **not** mean successful GPU execution or guaranteed compatibility with closed providers.

### 3.3 Origin, dispatch rounding and writes outside the active region

**Verified source fact.** Open FSR3's processing is origin-anchored. Its render-region SPD rectangle is `{0, 0, renderWidth, renderHeight}` ([lines 979–992](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L979-L992)); output shader coordinates are likewise derived from unoffset dispatch pixels. The public dispatch structure has **no output offset or destination rectangle origin**. An application wanting another placement must perform its own subsequent operation.

**Verified source fact.** The main output pass uses `ceil(U/8)` workgroups; sharpening uses `ceil(U/16)` ([lines 930–935 and 1035–1050](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L930-L1050)). The open [RCAS shader, lines 78–104](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_rcas.h#L78-L104), issues four pixel operations for each thread's 16×16 workgroup area without an evident active-output bounds test. [`StoreUpscaledOutput`, lines 477–483](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h#L477-L483), writes directly to the output UAV.

**Inference.** With sharpening enabled and an active dimension not divisible by 16, shader invocations can address pixels beyond the requested active rectangle. If those addresses fall within a larger allocation, the corresponding padding cannot be promised unchanged; if outside the allocation, their execution semantics need GPU/backend verification. Merely requiring \(A\ge U\) is therefore not proof of bounded shader writes.

The output texture must be treated as **writable by native recording**, not as a resource whose contents outside the active rectangle are preserved. This applies especially to maximum-sized allocations holding smaller active images; no source-established preservation guarantee exists.

## 4. Display-resolution motion vectors and helper queries

**Upstream claim.** The [pinned SDK guide, lines 93–108 and 139–141](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md#L93-L141), specifies motion-vector images at render resolution unless `FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS` is enabled, in which case they should be at presentation resolution. It does not resolve every larger-allocation/smaller-active-image combination.

**Verified source fact.** Open FSR3 chooses the motion-normalisation denominator from the **effective active image dimensions**:

```text
render-resolution motion:  denominator = active renderSize
display-resolution motion: denominator = active upscaleSize
```

The denominator selection is in [`fsr3upscalerDispatch:894–908`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L894-L908). In the display-resolution shader permutation, [`GetMotionVector`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_reproject.h#L41-L49) loads motion at the high-resolution pixel coordinate rather than mapping it through render-resolution coordinates. The source resource is indexed directly in [`LoadInputMotionVector`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h#L402-L415).

The corresponding application-supplied scales should be:

| Motion encoding | Render-resolution motion | Display-resolution motion |
|---|---|---|
| Normalised UV displacement | `(active_render_width, active_render_height)` | `(active_output_width, active_output_height)` |
| Pixel displacement | `(1, 1)` | `(1, 1)` |

**Inference.** A larger display-motion allocation can potentially contain the active motion image in its top-left region, as high-resolution pixel addresses begin at zero. The allocation must still cover those addresses, and the native resource description must retain its actual dimensions. This is a source-supported candidate, not a cross-provider guarantee. The baseline's use of **maximum output dimensions for display-motion UV scaling becomes incorrect when active output differs from the maximum**.

**Verified source fact.** Quality and jitter helpers operate on dimensions supplied to each query, not on context allocations. The [public query descriptors, lines 111–137](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L111-L137), call their target dimensions `displayWidth`, `displayHeight` or `output resolution width`. The native quality helper divides both supplied output dimensions by the quality ratio; the jitter phase helper uses the supplied output/render width ratio ([`ffx_fsr3upscaler.cpp:1249–1278`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1249-L1278)).

**Recommendation — Inference.** Use the **current intended active output**, not the context maximum, when asking for a render resolution or jitter phase count for the current frame. Keep maximum-based queries available for callers deliberately planning context capacity. Jitter offsets remain pixel-space offsets. No new native query ABI is required.

## 5. Output changes, dynamic-resolution flags and temporal history

### 5.1 Flag requirements

**Verified source fact.** [`FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L42-L54) is described as indicating that the application uses dynamic resolution scaling. The [FSR3 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L28-L47) forwards the flag into the internal context. However, the examined [`ffxFsr3UpscalerContextDispatch` validator](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1201-L1230) does **not** condition either render-size or output-size changes on the flag.

The [open FSR3 shader-permutation selection, lines 321–338](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L321-L338), does not select a special permutation for this flag.

**Unresolved.** These facts do not establish that a closed provider ignores the flag, or that AMD guarantees output variation with the flag disabled. Nor does the pinned public header specifically say that output-size changes require it.

**Recommendation — Inference.** Treat enabling the flag for a variable-resolution context as a prudent configuration choice, but do not misrepresent it as a demonstrated native prerequisite for changing `upscaleSize`. Retaining the wrapper's existing render-size gating is a compatibility policy; extending that gating to output size would be another wrapper policy requiring an explicit decision.

### 5.2 History and recreation

**Verified source fact.** Open FSR3 records the previous active output dimensions before updating the current ones ([`ffx_fsr3upscaler.cpp:870–881`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L870-L881)). The reprojection shader explicitly uses `PreviousFrameUpscaleSize()` on one compilation path; another path uses the current size ([`ffx_fsr3upscaler_reproject.h:58–64`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_reproject.h#L58-L64)). This is positive evidence that changing output dimensions was contemplated, but not proof that every temporal transition is artefact-free.

**Verified source fact.** The accumulation-reset condition is `params->reset || context->firstExecution`, not ‘output dimensions changed’ ([`ffx_fsr3upscaler.cpp:791–803`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L791-L803)). A reset clears or reinitialises temporal state, and internal history resources remain allocated to context maxima. The native code does not automatically recreate them on an output-size change ([lines 467–488, 925–969](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L467-L488)).

**Inference.** Output shrink/grow **within both context maxima** is structurally possible without recreating the open FSR3 context. A change exceeding either maximum requires a larger-capacity context, because the dispatch validator rejects it and the internal allocations are fixed.

**Recommendation — Inference.** Leave `reset` under renderer control, consistent with the baseline. Document resetting on substantial output-size changes as an option for avoiding temporal discontinuities, not as a source-proven requirement. Neither the public API nor the inspected code establishes an exact reset threshold or mandates resetting on every resize.

## 6. Resource states, lifetime and synchronisation

### 6.1 DX12 state requirements

**Upstream claim.** The [pinned integration guide, lines 93–96](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md#L93-L96), instructs DX12 applications to transition upscaler inputs to `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE` before dispatch.

**Verified source fact.** The open DX12 backend registers each external resource with the `state` supplied in `FfxApiResource` ([`RegisterResourceDX12:1959–1984`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L1959-L1984)). Its compute-job path transitions bound SRVs to compute-read state and UAVs to unordered-access state ([`ffx_dx12.cpp:3226–3229, 3332–3335`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L3226-L3335)).

[`UnregisterResourcesDX12:2178–2200`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L2178-L2200) schedules and flushes barriers returning registered external resources to their *declared* initial states.

The supplied wrapper currently declares input resources as compute-readable and output resources as UAVs. Changing output dimensions or texture identity does not alter the meaning of these states.

**Inference.** The renderer must ensure that each newly supplied resource is actually in its declared state. It must also ensure that the output texture has UAV capability and a compatible format. Resource replacement is not an implicit transition, fence or ownership transfer.

**Unresolved.** The backend's ordinary successful-dispatch restoration path does not prove safe restoration after arbitrary native-recording failures, closed-provider failures or partially recorded command lists. The baseline's terminal-state policy following native recording errors remains justified.

### 6.2 Replacement textures and in-flight use

**Verified source fact.** `RegisterResourceDX12` records the caller's `ID3D12Resource` pointer for the current dispatch ([lines 1969–1984](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L1969-L1984)). Open FSR3 executes scheduled jobs and unregisters external resources at the end of its recording path ([`ffx_fsr3upscaler.cpp:1073–1082`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1073-L1082)).

**Inference.** Unregistration is not evidence of GPU completion. Replacing an output texture is permissible as an application resource-management operation, but an old texture must remain alive until all GPU commands referencing it have completed. Reusing it requires appropriate fences, resource-state tracking and prevention of conflicting in-flight accesses.

Internal resources and temporal history persist across dispatches; therefore, the context also requires correct serialisation of its recording and GPU-use obligations. The optional movable `Send + !Sync` wrapper variant does not remove these obligations.

No new SDK-owned submission, transition or fence responsibilities should be introduced into the Rust wrapper.

### 6.3 Clear/copy operations and preservation

**Verified source fact.** Open FSR3 schedules full-resource clears of internal surfaces during initialisation/reset and per-frame processing ([`ffx_fsr3upscaler.cpp:772–789, 938–977`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L772-L977)). The generic DX12 backend also contains whole-resource clear and texture-copy operations ([`ffx_dx12.cpp:3476–3559`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L3476-L3559)). Those generic operations must not be mistaken for proof that FSR3 copies the entire external output.

**Inference.** Neither the absence of an output clear in the examined ordinary path nor shader access patterns establishes preservation of output pixels beyond the active rectangle. Any engine requiring defined contents throughout a larger destination allocation should explicitly initialise or composite them.

## 7. Difference from the supplied Rust baseline

The uploaded project was inspected as a baseline; no files were changed.

| Existing behaviour | Source-research finding | Suggested treatment |
|---|---|---|
| `upscaleSize` always equals maximum output | Public ABI and open FSR3 accept a smaller effective active output | Add a per-frame active output dimension. |
| Output allocation must equal context maximum | No such equality test in inspected FSR3 registration/dispatch | Do not present equality as a native requirement. |
| Display-motion allocation must equal context maximum | Documentation specifies presentation-resolution *image* geometry; allocation flexibility is less explicitly defined | Separate active coverage from allocation size. |
| UV display-motion scaling uses maximum output | Native FSR3 divides motion by current active output | Use active output dimensions. |
| Render input allocations must equal context maximum | Open code recognises that allocation and active image may differ | Consider a separate, explicit relaxation; do not conflate it with output support. |
| Smaller active render requires dynamic-resolution flag | No corresponding runtime requirement found in the examined open validator | Preserve as an existing wrapper policy unless deliberately revised. |
| Renderer controls reset, fences and submission | Consistent with the examined ownership model | Preserve. |
| Native recording failures terminalise context | Source does not prove recovery or state restoration after partial failure | Preserve. |

The relevant local wrapper sites are `upscaler/dx12/dispatch/frame.rs` (`UpscaleDispatch`), `descriptor.rs` (`build_descriptor`), `resources.rs` (`check_extents`, resource inspection and `DispatchGeometry`) and `upscaler/common.rs` (`UpscalerOptions::check_active_render_size`). The two context ownership variants should share the same dispatch validation and descriptor construction.

## 8. Recommended minimum additive API and wrapper contract

These are **research recommendations, not an accepted API design**.

### 8.1 Additive surface

**Recommendation — Inference.** Extend the existing portable frame description with:

```rust
active_output_size: Option<Dimensions>
```

and methods analogous to the existing render-size API:

```rust
pub fn active_output_size(&self) -> Option<Dimensions>;
pub fn set_active_output_size(&mut self, value: Option<Dimensions>);
```

`None` resolves to `max_upscale_size`. The field belongs in portable dispatch geometry, not DX12-specific resource types. Existing callers retain maximum-output behaviour.

In the DX12 descriptor builder, emit the **resolved active output** into `ffxDispatchDescUpscale.upscaleSize`. Continue to derive `FfxApiResource.description.width` and `.height` from each resource's real `D3D12_RESOURCE_DESC`.

Expose this through shared internals so both thread-confined and optional movable contexts behave identically. No change to the faithful `fsr-sdk-sys` ABI is required.

### 8.2 Validation with an explicit evidence boundary

**Recommendation — Inference.** The minimum defensible validation is:

1. Resolve active render and output independently; require both active dimensions to be positive and within their respective context maxima, component-wise.
2. Require each resource allocation to cover its applicable active image region, including display-resolution motion resources where selected.
3. Retain actual-allocation metadata and existing format, usage, device, identity and state checks.
4. Do not invent an aspect-ratio rule, render/output ratio limit, power-of-two rule or workgroup-multiple requirement as an AMD-documented invariant.
5. Do not guarantee that writes are confined to the active rectangle.

The second condition requires a deliberate distinction between **memory coverage** and **provider compatibility**. The open sources support the rationale for coverage validation, but do not prove that every allocation satisfying it is compatible with every provider.

In particular, an allocation above `maxUpscaleSize` is not prohibited by the examined FSR3 CPU checks, but its complete provider/GPU compatibility remains unresolved. It should not be labelled ‘supported’ without qualification.

For the current release, the maintainer can choose either a permissive coverage-based policy with explicit compatibility limitations or a temporarily narrower tested-allocation policy. Such a restriction must be documented as a **wrapper policy**, never misreported as an upstream rule.

### 8.3 Decisions still requiring maintainer approval

**Unresolved.** The following are independent decisions:

- Whether to relax render-side allocation equality concurrently or keep that change separate from variable output.
- Whether allocations merely covering active regions are accepted immediately, accepted only for known providers, or restricted pending GPU evidence.
- Whether external allocations larger than context maxima are admitted.
- Whether output changes require the wrapper's dynamic-resolution option, despite the lack of an examined native enforcement rule.
- Whether the wrapper should offer an optional resize/reset convenience mechanism. An implicit mandatory reset is not source-justified.
- How explicitly to expose the possibility of shader writes outside the active rectangle, particularly with sharpening.

Reactive-mask generation should remain separate: its dispatch dimensions describe render-side inputs, not the upscaler's active output.

## 9. Verification plan and unresolved evidence

### 9.1 Portable and controlled checks — proposed, not executed

**Recommendation — Inference.** Add focused tests at the portable validation/descriptor boundary:

| Test | Expected property |
|---|---|
| No active output specified | Resolves to context maximum; backward-compatible descriptor. |
| Smaller active output | Native `upscaleSize` changes without modifying resource descriptions. |
| One output axis above maximum | Rejected before COM inspection/native recording. |
| Zero/mixed-zero dimensions | Cannot enter the validated native path. |
| Output allocation exactly active | Passes coverage validation; no claim of GPU support. |
| Output allocation equal maximum | Passes coverage validation for smaller active output. |
| Intermediate allocation | Not rejected solely for differing from maximum. |
| Allocation above context maximum | Behaviour follows an explicit wrapper-policy decision. |
| Allocation smaller than active | Rejected before native recording. |
| Display-motion UV encoding | Uses active output dimensions. |
| Render-motion UV encoding | Uses active render dimensions. |
| Pixel motion encoding | Uses `(1, 1)` regardless of active dimensions. |
| Output shrink/grow sequence | Correct per-dispatch descriptors; no implicit reset unless explicitly designed. |
| Invalid dimensions and both ownership variants | Same validation and errors; no accidental divergence. |
| Native recording error | Existing terminal-state semantics remain intact. |

Controlled descriptor tests should inspect the **actual raw fields**, including independently varied width and height, rather than infer behaviour from a successful wrapper return.

Further static review should examine the precise compiled entry-point guards for every relevant shader permutation, especially non-sharpened accumulation versus RCAS, and any provider-specific allocation constraints not visible in the examined adapters.

### 9.2 Later Windows/provider/GPU work — explicitly not performed

A Windows x64/MSVC host with DX12 is required to determine:

- Whether the bundled open FSR3 provider successfully records and executes with \(A=U<M\), \(A=M>U\), intermediate \(A\), and \(A>M\).
- Whether odd and non-multiple-of-8/16 active output dimensions produce debug-layer errors or out-of-bounds behaviour, with sharpening enabled and disabled.
- Whether padding inside larger output allocations is modified.
- Whether display-resolution motion sampling is correct for active images stored in larger textures.
- Whether repeated shrink/grow operations work across consecutive frames, with and without explicit temporal resets.
- Whether changing the actual output resource every frame preserves correct state transitions, lifetime and fence behaviour.
- Whether the selected closed/driver provider accepts the same descriptors and allocation combinations.
- Whether provider overrides alter validation, resource requirements or resize behaviour.
- Whether error paths leave command lists and external resources in usable states.

These tests should record the selected provider ID/version, native return codes, DX12 debug-layer messages and actual texture dimensions. GPU results must not be inferred from successful CPU dispatch validation.

### 9.3 Outstanding source limitations

**Unresolved.** No inspected public source provides a complete, provider-independent statement of external output-allocation sizes relative to active output and context maximum. The pinned integration guide uses ‘presentation resolution’ primarily as an image-geometry concept and does not explicitly settle all three allocation strategies.

**Unresolved.** The examined shader functions reveal a material sharpening boundary risk, but source inspection alone cannot establish the precise GPU semantics of out-of-allocation accesses or certify every compiled shader permutation.

**Unresolved.** The native code explicitly checks `upscaleSize` against its maximum, whereas the [internal dispatch-header error documentation, lines 350–359](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h#L350-L359), mentions only oversized `renderSize` in its `FFX_ERROR_OUT_OF_RANGE` description. The executable native check is stronger than that documentation; both pieces of evidence should be retained.

**Unresolved.** Dynamic-resolution flag semantics, compulsory resets and cross-provider compatibility are not completely specified by the examined public material. A recommendation based on the bundled open implementation cannot become a guarantee about closed providers.

### Research disposition

**Verified source fact:** SDK 2.3.0's common upscaler ABI already exposes `upscaleSize`; open FSR3 forwards it, defaults `(0,0)` to context maximum and validates positive requested extents against each maximum axis. Actual external allocation dimensions remain distinct from active dimensions.

**Inference:** The smallest useful wrapper extension is an optional active output size, independent coverage validation and corrected display-motion scaling. Maximum-allocation equality is a current wrapper simplification, not a demonstrated general native requirement.

**Unresolved:** Universal support for exact-active allocations, maximum allocations with smaller active images, intermediate allocations and allocations exceeding the context maximum cannot be certified without provider-specific and GPU evidence. In particular, preserving pixels outside the active output is not a justified guarantee.

No implementation changes, native calls or GPU experiments were performed. All upstream evidence cited here comes from the pinned SDK tree; no SDK upgrade or newer-documentation claim is used to establish the contract.
