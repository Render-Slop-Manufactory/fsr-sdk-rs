# Source investigation: M6 upscaler frame semantics

> Intake note (2026-09-24): renamed from `rename_me_4.md`. Opaque
> `:chatgpt-content-reference` markers had no supplied index-to-source map and
> were removed. The substantive findings and pinned source list were retained;
> this intake does not independently verify every claim.

Research is pinned to AMD FidelityFX-SDK `v2.3.0`, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. In that release the public upscaler API is **4.1.1**, and AMD’s ML documentation identifies it as **FSR Upscaling 4.1.1**. FSR4 is delivered as a signed binary, so exact FSR4 shader internals are not public; where I use the same-tag open FSR3.1.5 implementation to resolve mechanics, I label that evidence `INFERRED` rather than silently treating it as the FSR4 implementation. AMD explicitly directs FSR4 integrators to the FSR3 documentation for general integration guidance.

Useful exact source pins: [ffx_upscale.h @ v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com) · [FSR4.1.1 integration guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md?utm_source=chatgpt.com) · [DX12 sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp?utm_source=chatgpt.com) · [Cauldron motion-vector shader](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/rendermodules/gbuffer/shaders/gbufferps.hlsl?utm_source=chatgpt.com).

## 1. Field/resource semantics

| Input | Semantic/numerical contract | Format / resolution / lifetime | Status |
|---|---|---|---|
| `maxRenderSize` | Maximum active rendering size for this context. Per-frame `renderSize` must not exceed it. It is an allocation/configuration bound, not necessarily the current frame size. | Creation-time persistent. | **VERIFIED.** Header defines it as maximum rendering size. Same-tag open implementation rejects dispatch dimensions beyond the maximum. |
| `maxUpscaleSize` | Presentation/output allocation target and upper bound. | Creation-time persistent. | **VERIFIED.** |
| `color` | Current-frame rendered scene color. FSR4 strongly expects **linear** color. Normally pre-tonemap; AMD places application tonemapping after upscaling. HDR requires the HDR creation flag. Nonlinear input is supported only when declared, optionally identifying sRGB or PQ. | Application-specified texture format, active render resolution, current frame. Must be jittered. | **VERIFIED/CORROBORATED.** |
| `depth` | Hardware/device depth, not application-linearized view-space distance. Depth convention is communicated via creation flags. Standard and reversed/infinite variants are supported; inverted means `[1..0]`. | One floating-point channel; render resolution; current frame; jittered consistently with color. | **CORROBORATED.** FSR4 docs specify one FLOAT and depth flags; same-tag open depth reconstruction derives view depth from device depth plus near/far/FOV. |
| `motionVectors` | Current-frame pixel → location of that same surface point in the **previous frame**. After scaling, the semantic quantity is screen/pixel displacement. Default convention requires jitter removed from the vectors. | 2× FLOAT, render resolution unless display-resolution-MV flag; 16-bit precision is sufficient internally. | **VERIFIED.** |
| `motionVectorScale` | Converts the stored vector encoding to FSR's screen-space convention. The safest interpretation is `stored_mv * scale = current→previous displacement in pixels`. | Per frame; target dimensions depend on low- vs display-resolution MVs. | **CORROBORATED/INFERRED.** Public prose, sample shader and open implementation together establish this; see §2. |
| `exposure` | Current-frame scalar exposure multiplier that should match the exposure used by the application's later tonemapper. | `R32_FLOAT`, 1×1. May be null when auto exposure is enabled. | **VERIFIED.** |
| `preExposure` | Factor by which the supplied color was multiplied before FSR. FSR divides input color by this value to recover the original game signal. Must be `>0`. | Scalar, per frame. Dimensionless multiplier. | **VERIFIED.** |
| `reactive` | Controls how strongly current-frame samples replace/reduce reliance on temporal history. `0` = ordinary accumulation; `1` = fully reactive. | `R8_UNORM`, render resolution. Optional for FSR4. | **CORROBORATED.** FSR4 marks it optional; common same-release guidance specifies `[0,1]`. |
| `transparencyAndComposition` | Controls history protection/locking rather than simply accumulation weighting. Intended for content such as animated textures/reflections whose temporal behavior is poorly represented by ordinary geometry motion. | `R8_UNORM`, render resolution. Optional for FSR4. | **CORROBORATED.** |
| `output` | Current reconstructed image at presentation/upscale resolution. | Presentation resolution. Public FSR4 docs do **not** publish a format whitelist or alpha-preservation contract. | Dimensions **VERIFIED**; format/alpha **UNKNOWN**. |
| `renderSize` | Active dimensions actually rendered this frame. Not synonymous with allocation extent or context maximum. | Per frame. Nonzero should be treated as required. | **VERIFIED.** Header explicitly defines it; general docs say render-resolution resources match actual rendering resolution. |
| `upscaleSize` | Active output resolution this dispatch should produce. `{0,0}` means `maxUpscaleSize`. Exactly one zero dimension is nonsensical. | Per frame. | **VERIFIED/CORROBORATED.** |
| `jitterOffset` | **Pixel-space subpixel jitter actually used to render this frame**, not NDC/projection-matrix translation. | Per frame, floating pixel units. Recommended sequence is Halton `[2,3]`. | **VERIFIED.** |
| `enableSharpening`, `sharpness` | Enables the additional sharpening stage; `sharpness∈[0,1]`, 0 minimum and 1 maximum. It is RCAS in the supplied FSR4 material/sample. | Per dispatch. | **CORROBORATED.** |
| `frameTimeDelta` | Elapsed time since the previous frame, in **milliseconds**. FSR explicitly uses it in temporal auto exposure. | Per frame. ~16.67 at 60 Hz. | **VERIFIED.** |
| `reset` | Discards unusable temporal history after a discontinuous camera transformation. Assert on the **first frame** after the discontinuity. | Per dispatch boolean; normally one frame only. | **VERIFIED.** |
| `cameraNear` | Geometric distance to near clipping plane, in application view-space units. | Positive scalar; per frame. | **VERIFIED** meaning; exact FSR4 validation range beyond normal camera validity is not published. |
| `cameraFar` | Geometric far-plane distance. Reversed-Z is **not** signaled by swapping near/far; flags encode the convention. | Per frame. Infinite projection indicated separately by creation flag. | **CORROBORATED/INFERRED.** Same-tag reconstruction explicitly chooses transforms from flags and normalizes near/far ordering. |
| `cameraFovAngleVertical` | Vertical perspective FOV, **radians**, not horizontal FOV or degrees. | Per frame. Same-tag validator expects `0 < FOV <= π`. | **VERIFIED** units/direction; validator evidence is FSR3 implementation. |
| `viewSpaceToMetersFactor` | Multiplicative conversion from application's view-space unit to meters. If one view unit is one meter, pass `1.0`. | Per frame. | Meaning **VERIFIED**. Zero-default behavior **INFERRED only**: sample zero-initializes it while same-tag fallback maps nonpositive values to 1.0. Do not rely on this; explicitly pass a positive value. |
| dispatch colorspace flags | With nonlinear-color context mode, `SRGB` or `PQ` describes the actual perceptual encoding. They are mutually exclusive. | Per dispatch. | **VERIFIED.** |
| resource presence | At ABI level a missing resource is `FfxApiResource.resource == nullptr`; dimensions/format/state fields do not substitute for presence. | Resource descriptor. | **VERIFIED.** |

There is no separate viewport or scissor rectangle in the upscaler dispatch API. Consequently FSR only knows the resource and `renderSize`; it cannot be told that valid rendering occupies an arbitrary nonzero-origin viewport. For M6, using a top-left `(0,0)` active image whose extent exactly equals `renderSize` avoids an otherwise undocumented assumption. **INFERRED.**

There is also no published numerical minimum width/height in the FSR4.1.1 documentation. `renderSize==0` is not documented as a valid sentinel; the open FSR3 code happens to fall back to the color resource extent but simultaneously emits a diagnostic saying zero render dimensions are wrong. M6 should reject zero. **UNKNOWN for FSR4 implementation behavior; contract recommendation is nonzero.**

Dynamic render-resolution changes are a supported use case, indicated by `FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION`; AMD's sample sets it specifically for its DRS mode. Active dimensions still must remain inside the creation maxima. Resource/window resize in the official sample destroys and recreates the context and restarts the jitter index.

## 2. Motion-vector contract

The most useful canonical definition is:

\[
\Delta uv = uv_\mathrm{previous}-uv_\mathrm{current}
\]

with UV coordinates using the ordinary texture convention used by the sample: +X right, +Y down. If the MV texture contains values \(m=(m_x,m_y)\), define \(T=(W,H)\) as the MV target resolution—render size normally, presentation size with `DISPLAY_RESOLUTION_MOTION_VECTORS`. Then the intended pixel displacement is

\[
\Delta p = m \odot \texttt{motionVectorScale},
\]

and equivalently

\[
\Delta uv=\frac{m\odot\texttt{motionVectorScale}}{T}.
\]

The public FSR4 documentation directly establishes **current→previous direction** and the final screen-space range. The same-tag open implementation explicitly divides `motionVectorScale` by the target resolution before shader use, corroborating the UV interpretation. **CORROBORATED.**

For a texture already storing pixel displacement, use approximately:

```text
motionVectorScale = (1, 1)
```

For a texture storing UV displacement, use:

```text
motionVectorScale = (W, H)
```

The official DX12 sample takes the second route. Its GBuffer shader first calculates:

```text
previousNDC - currentNDC + jitterCancellation
```

and then multiplies by `(0.5, -0.5)`, explicitly saying it is converting from NDC to top-down UV space. CPU dispatch then supplies `(renderWidth, renderHeight)`. Thus the end result is correctly measured in pixels.

This exposes a genuine documentation bug. The FSR4 prose example shows raw `previousNDC-currentNDC` together with `motionVectorScale=(W,H)`. Raw NDC spans width `2`, and its Y axis has the opposite sign from top-down UV, so correct NDC→pixel conversion is actually:

\[
S=(+\tfrac12 W,-\tfrac12 H).
\]

The sample performs exactly those factors in the shader and then uses positive `(W,H)` on CPU. The prose omitted the `0.5,-0.5` transformation. This is **a real discrepancy**, not merely different terminology. The sample shader plus the stated screen-space range are stronger numerical evidence than that isolated example.

A simple sign sanity check follows directly from current→previous semantics: if an object moved four pixels to the right between frames, a currently covered pixel maps four pixels left to its previous position, so its X motion vector is `-4 px`.

Motion vectors should normally be **jitter-free**. AMD explicitly says every render-resolution input except motion vectors should be jittered. Cauldron calculates `PrevJitter-CurrJitter` as a cancellation term before exporting vectors. If an application deliberately supplies vectors containing the projection jitter, it must create the context with `FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION`; the same-tag fallback computes the temporal jitter difference internally in that case. **CORROBORATED.**

FSR wants valid motion wherever possible: opaque, alpha-tested and alpha-blended content, including vertex/procedural animation. There is no documented “invalid MV” sentinel. Zero therefore semantically means zero displacement, not “unknown”; missing vectors on moving content are a quality/correctness problem rather than a formally encoded state. **VERIFIED coverage requirement; invalid-sentinel semantics UNKNOWN.**

## 3. Jitter contract

The public query returns offsets in **unit pixel space**, using a Halton `[2,3]` sequence. The same-tag open helper is concretely:

\[
j_x=H_2(i)-0.5,\qquad j_y=H_3(i)-0.5,
\]

so its generated values lie approximately in `[-0.5,+0.5)` pixels. The older/open debug checker accepts the looser expected envelope `[-1,+1]`; FSR4 documentation itself does not declare `[-1,+1]` as a normative range.

For AMD's documented projection convention, pixel-space jitter `(jx,jy)` maps to projection translation:

\[
t_x=+2j_x/W,\qquad
t_y=-2j_y/H.
\]

`jitterOffset` passed to dispatch must then be the **same pixel-space jitter represented by that projection shift**. It is not the NDC translation.

The official sample initially looks contradictory:

```text
query -> q=(qx,qy)
camera projection translation = (-2*qx/W, +2*qy/H)
dispatch jitterOffset          = (-qx,-qy)
```

but it is internally consistent. Let the actual FSR-coordinate jitter be `d=(-qx,-qy)`. AMD's documented mapping gives:

\[
(+2d_x/W,-2d_y/H)=(-2q_x/W,+2q_y/H),
\]

which is exactly what Cauldron inserts into its projection matrix. The sample simply uses the negated Halton sequence. Therefore **do not copy the sample's negation as an API rule**; copy the invariant “dispatch the actual pixel jitter that your projection used.”

Jitter applies to opaque, alpha-transparent and ray-traced rendering; depth and color consequently share the jittered sample location. Motion vectors normally remove it. A constant `(0,0)` sequence violates AMD's custom-sequence guidance and defeats the temporal subpixel sampling on which reconstruction depends.

Preset sequence lengths are explicitly documented as 18/23/32/72 for Quality/Balanced/Performance/Ultra Performance. For arbitrary ratios the prose says `ceil(8*n²)`. There is a minor source inconsistency: the same-tag open implementation converts `8*(displayWidth/renderWidth)^2` to integer by truncation, which also explains the documented Balanced result of 23. The robust M6 rule is therefore: **query `ffxQueryDescUpscaleGetJitterPhaseCount`; do not reimplement the formula.**

## 4. Depth/camera contract

Depth should be the render's normal **device depth value**, with the same jittered projection/sample location as color—not linear view-space Z. This is supported directly by the `[1..0]` reversed-depth description and by the open same-tag code reconstructing view-space depth from device depth using `cameraNear`, `cameraFar`, FOV and depth flags.

The four meaningful configurations are:

| Projection | Flags | Near/far interpretation |
|---|---|---|
| Standard finite | neither flag | Near maps toward depth 0, far toward 1. Supply geometric near/far distances. |
| Reversed finite | `DEPTH_INVERTED` | Near maps toward 1, far toward 0. Still supply geometric near/far distances. |
| Standard infinite | `DEPTH_INFINITE` | Standard depth orientation, infinite far projection. |
| Reversed infinite | both flags | Reversed-Z plus infinite far; this is AMD's strongly recommended general configuration. |

**Do not communicate reversed-Z by swapping the numeric near and far parameters.** The same-tag depth setup deliberately sorts those distances and chooses the reconstruction equation from the flags, making the flags authoritative.

For infinite depth, the same-tag sample uses `FLT_MAX` as its finite approximation when constructing the projection and passes that value as `cameraFar`, while creating FSR with both inverted and infinite flags. In the open reconstruction formula, the nominal far value drops out of the infinite projection coefficients. Thus for an M6 test using the same projection style, positive near + very large/`FLT_MAX` far + the infinite flag is strongly supported. Exact signed-binary FSR4 handling of arbitrary finite `cameraFar` while the infinite flag is set remains **INFERRED**, not publicly specified.

`cameraFovAngleVertical` is unambiguously **vertical FOV in radians**. It is not horizontal FOV. The open code derives horizontal FOV from vertical FOV and current render aspect ratio. No handedness field exists in the dispatch; Cauldron itself uses a `-Z` forward convention and works through the same scalar interface. Handedness therefore should not be encoded by negating near/far or FOV.

Orthographic-camera semantics are not documented for the upscaler. The interface supplies a perspective FOV and the open reconstruction explicitly uses `cot(FovY/2)`, so an orthographic projection should be considered **UNKNOWN/unsupported until empirically demonstrated**.

## 5. Color/HDR/exposure contract

For the ordinary path, color is scene color **before the application's final tonemapper**, preferably linear. AMD's suggested pipeline puts SSR/SSAO/denoisers before FSR and film grain, chromatic aberration, vignette, tonemapping, bloom, DoF and motion blur after it. Alpha-blended scene geometry is not something to exclude from the scene color; AMD explicitly asks that alpha-blended objects participate in jitter and preferably motion-vector generation. UI/HUD alpha semantics are not part of the FSR color contract; composing UI after reconstruction is the least ambiguous M6 choice.

Let the engine's un-pre-exposed linear signal be \(C\), and suppose its render target contains

\[
C_\mathrm{input}=P\,C.
\]

Then supply:

```text
preExposure = P
```

because AMD states that FSR divides the input by `preExposure` to recover the game's original signal. It subsequently multiplies the de-pre-exposed signal by the exposure value \(E\), which should match the exposure used by the later application tonemapper.

`preExposure=0` is invalid by public contract. The open FSR3 implementation happens to substitute `1.0` while its checker reports zero as an error; that fallback must not leak into the safe Rust contract.

Auto exposure absolutely exists in this API generation. With `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE`, the exposure texture can be null, and AMD recommends this path unless the application has a reason to provide its own exposure. The official v2.3.0 sample follows exactly that configuration: it enables auto exposure and dispatches a null exposure resource.

The exposure scalar is a multiplier, not an EV value or value in nits. No upscaler dispatch field specifies HDR luminance bounds. `HIGH_DYNAMIC_RANGE` describes the signal's dynamic-range characteristics; nonlinear sRGB/PQ flags describe encoding. These are separate issues. FSR's internal exposure/tonemapping is undone before output, and AMD explicitly states that the output returns to the same signal domain as the input rather than becoming the application's final tonemapped image.

There is no documented semantic use or preservation guarantee for the input/output alpha channel. Treat alpha as **UNKNOWN** for M6 rather than depending on it.

## 6. Mask semantics

For FSR4 specifically, reactive and transparency/composition masks are **not required**. The API provides `ffxQueryDescUpscaleGetResourceRequirements` precisely because resource requirements vary by selected upscaler version; its required bitmap means “required for effect correctness,” while its optional bitmap identifies resources that will be consumed if supplied.

Reactive mask:
- `R8_UNORM`, render resolution.
- Continuous `[0,1]`, not merely binary.
- `0`: ordinary/default temporal composition.
- `1`: maximally reactive/current-frame weighted.
- Typical high-reactivity content: fast alpha-blended particles/transparency, especially content without reliable depth/MVs.
- Alpha used for transparency is a useful proxy; FSR3 general guidance recommends not routinely pushing values all the way to 1, suggesting ~0.9 as a practical upper value.
- Helper generation is optional, not required; it compares opaque-only color against opaque+translucent color.

Transparency/composition mask:
- `R8_UNORM`, render resolution.
- Also conceptually `[0,1]`.
- It is not “another reactive mask”: it weakens/removes history-lock protection and luminance-instability behavior.
- `0`: no additional lock modification.
- `1`: remove that history lock completely.
- Examples: ray-traced reflections, animated textures and other special composition whose shading changes are not explained by conventional geometry motion.

The same-release general guide says null masks use cleared internal 1×1 fallback textures; the open implementation does the same. Because AMD explicitly points FSR4 integration back to that general guide and its own FSR4 sample passes null masks, this is strong evidence for null semantics, though the signed FSR4 implementation is not source-visible.

For the smallest FSR4 M6 experiment, omitting both masks is therefore valid. They become quality experiments rather than prerequisites for first correct reconstruction.

## 7. Temporal/reset contract

The upscaler retains temporal state internally; there is **no application-supplied frame index** in `ffxDispatchDescUpscale`. Consecutive dispatches implicitly define consecutive reconstruction frames.

Across frames, color, depth, MVs, jitter, camera data, pre-exposure/exposure and active dimensions must describe the same current frame. Motion vectors must describe precisely the interval from that frame to the immediately preceding upscaler input; depth and color must correspond to the same jittered projection. This follows from AMD's current-frame resource requirement and current→previous MV definition.

`reset=true` means history prior to this dispatch is unusable. AMD explicitly prescribes it on the **first frame** of a discontinuous camera transformation. The sample combines its manual reset button with `WasCameraReset()` and clears its reset boolean after the dispatch, confirming one-frame pulse semantics.

Appropriate reset cases are therefore:
- first M6 dispatch: prudent even though the provider necessarily has no useful history;
- camera cut/teleport;
- any equivalent whole-view temporal discontinuity where the previous-frame correspondence is meaningless.

A mere supported DRS change inside context maxima is not, by itself, evidence that history should be discarded; DRS exists specifically to permit changing active render size. Conversely, the official sample **recreates the context** on actual resource resize and resets the jitter index. Reset is not a substitute for recreation when creation-time maxima, flags, resource allocation or other persistent context properties need to change.

The same-tag open implementation also internally treats first execution as accumulation reset, but this is implementation evidence rather than the FSR4 binary contract.

`frameTimeDelta` should correspond to elapsed time between the two successive rendered frames entering the upscaler. The documentation only calls it “time elapsed since the last frame”; it does not distinguish simulation versus presentation clocks. The sample passes its frame `deltaTime * 1000`. For M6, a real render-frame delta is the least ambiguous interpretation. Zero is not documented as valid, and the same-tag checker warns for values below 1 ms.

## 8. Contradictions discovered

**Motion-vector NDC scaling — real discrepancy.** FSR4 docs show raw NDC `previous-current` and CPU scale `(W,H)`. The official shader actually converts NDC with `(0.5,-0.5)` before the same CPU `(W,H)` scale. The latter is numerically consistent with the public screen-space range and texture-coordinate orientation. For M6, follow the sample transformation, not the isolated prose example.

**Jitter signs — only apparent discrepancy.** Documentation says projection translation for dispatch jitter `(jx,jy)` is `(+2jx/W,-2jy/H)`. Sample appears to use the opposite projection sign, but it also dispatches the negative query result. Those two negatives exactly cancel at the semantic level. The invariant is “dispatch the jitter actually represented by the rendered projection.”

**Jitter phase formula — real minor discrepancy.** Documentation says custom length `ceil(8 n²)`, yet documents Balanced as 23; the same-tag helper truncates `8 n²`, yielding 23 for the nominal 1.7 ratio. Use the query API instead of reproducing either formula.

**Depth format wording — apparent/stale-header discrepancy.** The public struct comment says “32bit depth values”; FSR4's dedicated integration guide says application-specified single-component floating depth whose precision is chosen by the application. The FSR4 integration guide is more specific and later-facing; do not encode a needless “must be D32” restriction in a semantic type.

**Masks — version distinction, not contradiction.** The FSR3 general guide strongly recommends masks; the FSR4-specific section explicitly says FSR4 no longer requires them. FSR4-specific text wins for FSR4 correctness.

**`viewSpaceToMetersFactor` — underspecified sample behavior.** Header defines the factor but no default. The sample zero-initializes the dispatch and never assigns it; same-tag open FSR3 maps nonpositive values to 1. This strongly suggests “zero → default one” compatibility behavior, but that is not an explicit FSR4 API guarantee. M6 should write `1.0` explicitly when view units are meters.

**Zero `renderSize` / zero `preExposure` — implementation fallback is not contract.** Open FSR3 contains fallback code for both, while its own debug checks call them erroneous. A safe wrapper should enforce the documented semantics rather than expose those accidents as features.

## 9. Rust semantic-type implications

These types are justified by concrete failure modes, not aesthetics:

- `RenderDimensions` versus `OutputDimensions`: prevents swapping render and presentation sizes and allows enforcing `render <= max_render`, `output <= max_output`. Your existing validated `Dimensions` can remain the common representation internally, but role-specific wrappers at dispatch boundaries have real value.
- `JitterOffsetPixels { x, y }`: prevents passing the projection/NDC translation `(2j/W,-2j/H)` where FSR expects pixel jitter.
- `MotionVectorScale` or, better, an encoding-aware constructor such as `MotionVectorEncoding::{Pixels,Uv}`: prevents the realistic `(W,H)` versus `(W/2,-H/2)` versus `(1,1)` error. This is probably the highest-value semantic type.
- `FrameTimeMilliseconds`: prevents the sample-documented seconds-versus-milliseconds error.
- `VerticalFovRadians`: prevents both degrees and horizontal-FOV substitution.
- `CameraParameters { near, far, vertical_fov, view_space_to_meters }`: keeps the mutually dependent depth-reconstruction values together.
- `DepthConvention::{StandardFinite,StandardInfinite,ReversedFinite,ReversedInfinite}` at context configuration: prevents trying to communicate reversed-Z by swapping near/far.
- `PreExposure`: positive finite newtype; reject zero. `ExposureMultiplier` should be a different type if manual exposure is exposed, because exposure and pre-exposure have opposite roles and are easy to confuse.
- `Sharpness01` if sharpening enters M6: prevents out-of-range values.
- Typed resource roles for `ColorInput`, `DepthInput`, `MotionVectorsInput`, `UpscaledOutput`, and the two masks are useful if they can validate descriptors; they prevent resource-role swaps. They cannot make resource-state/GPU-lifetime correctness safe by themselves.

A dedicated newtype around `reset: bool` provides little real safety. A named `HistoryReset::{Continue,Discard}` enum may improve readability, but I would not introduce it solely for type purity.

For raw D3D12 resources, Rust cannot verify that GPU writes have completed, that the stated state matches the actual D3D12 state, or that the texture's pixels obey the semantic conventions above. Until there is a synchronization/resource abstraction, some dispatch boundary should remain explicitly unsafe or carry clearly documented external invariants rather than claiming full safety.

## 10. Correctness-test plan

**Motion direction/sign.** Render an analytic high-contrast rectangle translating exactly +4 render pixels/frame over a static background, with a high-resolution ground-truth renderer. Correct current→previous vectors inside the rectangle are `(-4,0)` pixel-equivalent. Compare correct, negated and zero MVs after 8–16 warm-up frames. Wrong sign should produce a trail on the opposite side of the edge.

**Motion scale.** Store UV vectors (`-4/W,0`) and dispatch `(W,H)`. Run variants `(1,1)`, `(W/2,H/2)`, `(2W,2H)` and correct `(W,H)`. Edge ghost displacement should scale correspondingly. This distinguishes scale errors from direction errors.

**NDC conversion.** Generate exactly the same scene with raw NDC vectors and test `(W,H)` versus `(W/2,-H/2)`. The latter should match the UV-vector reference; this directly detects the documentation trap.

**Jitter sign/units.** Use an analytically sampled near-Nyquist grid or one-pixel high-resolution diagonal. Generate low-resolution frames at known ±0.25-pixel sample offsets. Correct FSR jitter should converge to a stable fixed high-resolution reference; negating only dispatch jitter creates periodic double edges, while passing NDC values instead of pixels produces a scale-dependent phase error.

**Double jitter.** Produce MVs once with jitter removed and once containing the current/previous jitter difference while leaving the cancellation flag disabled. Then repeat the latter with cancellation enabled. Only `(jitter-free,no flag)` and `(jittered,cancellation flag)` should agree.

**Depth convention.** Two planes with a sharp depth discontinuity, foreground object moving laterally against a textured far background. Run ordinary finite, reversed finite and reversed infinite projections with their corresponding flags. Wrong reversed-Z configuration should show obvious disocclusion history around the silhouette.

**Near/far sensitivity.** Hold the actual depth texture fixed but intentionally supply 2×/½× incorrect near or far values. Use large depth separation and moving occlusion boundaries so erroneous reconstructed view depth gives measurable history rejection differences. For infinite projection vary the nominal huge far value; output should be far less sensitive than a finite-depth run.

**HDR/pre-exposure.** Render a linear HDR wedge plus a moving emissive patch. Alternate pre-exposure between, e.g., `P=1` and `P=4` while multiplying the input signal by P and passing matching P. After normalizing outputs back to the same physical signal they should remain temporally consistent. Re-run with deliberately wrong `preExposure=1`; history/exposure instability becomes measurable.

**Manual versus auto exposure.** With fixed scene luminance, compare auto-exposure/null texture against a fixed explicit `R32_FLOAT` exposure and matching later tonemapper. This validates the resource binding separately from pre-exposure.

**Dimensions.** First establish the baseline with resources whose physical extents exactly equal render/output dimensions. Then deliberately lie about `renderSize` by a small amount. A poison-colored border outside the declared active area is useful for subsequent experiments with oversized allocations: any leakage proves an unsupported resource-extent assumption.

**Reset.** Warm history on pattern A, instantaneously replace it with unrelated pattern B/camera pose, and compare reset false versus a one-frame reset pulse. The reset frame should contain no measurable A-history. Repeating reset for several frames can additionally quantify the cost/quality penalty and confirm it is intended as a pulse.

**Reactive mask.** Moving translucent/additive particle with deliberately absent MV/depth footprint. Compare mask 0 to alpha-derived ~0.8–0.9. Evaluate trailing energy behind the particle.

**T&C mask.** Use fixed geometry with an independently scrolling/animated texture or rapidly changing reflection. Compare T&C 0/1 while geometry MVs remain zero. Measure persistence of stale pattern content.

Use deterministic readback plus per-frame image-error metrics against an analytic high-resolution reference; do not make “looks reasonable” the pass criterion. A semantic bug can still produce a visually plausible image.

## 11. Unknowns that should remain explicit in M6

1. **Exact FSR4 output-format whitelist.** The release says application-specified input color but does not publish a complete accepted output-format table. Use a format demonstrated by the native sample/runtime experiment rather than promising arbitrary DXGI formats.
2. **Alpha preservation/meaning.** Not documented. Do not expose a guarantee.
3. **Input/output aliasing.** No public semantic guarantee. The official sample explicitly copies color to a temporary texture “so that the input and output texture … is different.” M6 should require distinct resources until proven otherwise.
4. **Oversized resource allocations with a smaller active `renderSize`.** The API conceptually separates extent from active size, but FSR4's signed implementation does not publicly define every cropping/origin rule. Exact matching is the conservative M6 contract.
5. **Minimum dimensions.** No FSR4.1.1 minimum is documented.
6. **Orthographic projections.** No documented contract.
7. **Invalid/no-motion sentinel.** None documented.
8. **Exact behavior for `frameTimeDelta=0`.** It violates intended elapsed-frame semantics, but FSR4's acceptance/error behavior is unpublished.
9. **History preservation across changing `upscaleSize`.** Dynamic render resolution is supported, but arbitrary per-frame presentation-resolution changes are not sufficiently documented. The sample recreates context on resource resize.
10. **Exact FSR4 behavior for null exposure when AUTO_EXPOSURE is off, zero `viewSpaceToMetersFactor`, zero `renderSize`, etc.** Same-tag FSR3 has compatibility fallbacks, but they are not FSR4 guarantees.
11. **Exact quantitative effect of optional masks in FSR4.** Their roles are defined through the common API/general guidance, but ML internals are signed-binary-only.

For the **smallest defensible M6 DX12 path**, the evidence points to a deliberately narrow profile: fixed exact-match resource dimensions; distinct input/output textures; linear pre-tonemap scene color; `preExposure=1` unless the test renderer genuinely pre-exposes; auto exposure + null exposure resource; jittered device depth with explicitly matching depth flags; low-resolution jitter-free motion vectors stored as top-down UV displacement with `motionVectorScale=(renderWidth,renderHeight)` exactly like AMD's sample; queried Halton jitter with the identical pixel offset represented in rendering and dispatch; vertical FOV radians; positive explicit `viewSpaceToMetersFactor=1`; one-frame reset on initial dispatch/cuts; and no reactive/T&C masks for the first FSR4 reconstruction. That profile removes essentially every semantic ambiguity identified above while staying well short of an engine abstraction.
I embedded the citations inline, but I omitted a consolidated source list at the bottom. That was a miss. Here are the primary `v2.3.0` sources used:

### Sources

1. `ffx_upscale.h` — public FSR upscaler API, dispatch descriptors, flags, field semantics  
[AMD FidelityFX SDK v2.3.0 — ffx_upscale.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com)

2. FSR4 / ML Super Resolution integration documentation — input signal, HDR, motion vectors, jitter, exposure, resource requirements  
[AMD FidelityFX SDK v2.3.0 — Super Resolution ML documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md?utm_source=chatgpt.com)

3. General Super Resolution / FSR3 integration documentation — reactive/T&C masks and common temporal integration rules  
[AMD FidelityFX SDK v2.3.0 — Super Resolution Upscaler documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md?utm_source=chatgpt.com)

4. Official DX12 FSR sample — actual dispatch values, jitter setup, auto exposure, resets, context resize handling  
[AMD FidelityFX SDK v2.3.0 — DX12 FSR sample render module](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp?utm_source=chatgpt.com)

5. Cauldron DX12 GBuffer shader — authoritative sample-side motion-vector calculation, including NDC→top-down-UV conversion and jitter cancellation  
[AMD FidelityFX SDK v2.3.0 — gbufferps.hlsl](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/rendermodules/gbuffer/shaders/gbufferps.hlsl?utm_source=chatgpt.com)

6. Same-tag open FSR3 implementation — validation, motion-vector scaling, depth reconstruction, first-frame/reset behavior; used only where FSR4 binary internals are unavailable  
[AMD FidelityFX SDK v2.3.0 — ffx_fsr3upscaler.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp?utm_source=chatgpt.com)

7. Cauldron camera implementation — projection construction and infinite/reversed-Z behavior used by the official sample  
[AMD FidelityFX SDK v2.3.0 — cameracomponent.cpp](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/core/components/cameracomponent.cpp?utm_source=chatgpt.com)

8. FidelityFX API resource definitions — null resource representation and ABI resource structure  
[AMD FidelityFX SDK v2.3.0 — ffx_api_types.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h?utm_source=chatgpt.com)

All of those are pinned to commit `60f4ea81909200d8542eca14dccb2628b763a9a3`, the commit behind AMD's `v2.3.0` tag.
