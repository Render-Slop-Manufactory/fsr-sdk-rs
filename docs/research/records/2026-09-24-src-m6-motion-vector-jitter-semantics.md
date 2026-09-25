# Source investigation: M6 motion-vector and jitter semantics

- **Investigation date:** 2026-09-24
- **Project:** `fsr-sdk-rs`
- **Topic:** FidelityFX SDK v2.3.0 / FSR Upscaling 4.1.1 motion-vector and dispatch-jitter semantics
- **Investigation type:** bounded public-source investigation; no implementation changes and no runtime/GPU experiment
- **Pinned upstream revision:** `60f4ea81909200d8542eca14dccb2628b763a9a3`
- **Evidence cutoff:** the pinned FidelityFX SDK revision above, inspected on 2026-09-24. No later upstream behavior is used to silently reinterpret that revision.

## Scope

This record investigates only the coordinate, direction, scale, resolution and jitter conventions needed to reason about the first public `fsr-sdk-rs` dispatch contract. It distinguishes three evidence classes that must not be conflated:

1. the **FSR4 public contract material** at the pinned SDK revision (`ffx_upscale.h` and the ML Super Resolution integration guide);
2. the **official AMD DX12/Cauldron sample behavior** at the same revision; and
3. the **open FSR3 implementation** shipped at the same revision, used only as supporting implementation evidence where the signed FSR4 provider's internals are not public.

The project baseline for this investigation is a Windows x64/MSVC, DirectX 12 wrapper around FidelityFX SDK 2.3.0 using the modern `ffxCreateContext` / `ffxDestroyContext` / `ffxConfigure` / `ffxQuery` / `ffxDispatch` API. The repository baseline reports one M6a native dispatch proof at render size `320x180` and output size `640x360`, with zero motion vectors, a nonzero matching jitter, `reset = true`, and plausible fenced readback. That existing proof establishes a real dispatch path but does **not** establish temporal motion-vector direction, scale, Y orientation, or multi-frame jitter semantics because the motion texture was zero and only one reset frame was exercised.

The current D011 proposal is treated only as a set of claims to compare with evidence. This record does not accept, reject, or rewrite D011 and does not design the Rust API.

A repository ZIP was expected by the investigation brief but was not available in the provided workspace: only the investigation brief itself was present. Therefore repository-local D011, M6/M6a records, and any additional repository research-process document could not be independently re-read. This record is self-contained and follows the research-process requirements explicitly available in the brief, but that missing repository-local inspection is a limitation recorded again below.

## Evidence labels

- **Verified** — directly supported by identified primary evidence inspected in this investigation.
- **Upstream claim** — AMD states the point, but this investigation did not independently reproduce it as a runtime result.
- **Inference** — derived from identified source evidence or coordinate mathematics rather than explicitly stated by AMD.
- **Unresolved** — the available public evidence does not settle the point.

A source fact marked **Verified** is not a newly verified runtime/provider result.

## Method and exclusions

The investigation inspected AMD's public files at the exact SDK commit, cross-checked documentation against the official DX12 sample and Cauldron shader/camera code, and then used the same-tag open FSR3 host/shader implementation only to illuminate scaling and jitter-cancellation mechanics that are not visible inside the signed FSR4 provider. Coordinate conversions were checked algebraically rather than inferred from names alone.

No AMD GPU was available. No signed-provider execution, temporal image comparison, GPU capture, debugger/shader inspection of proprietary binaries, production/test modification, code implementation, API redesign, renderer design, image-quality evaluation, allocation/in-flight research, exposure/sharpening/reactive-mask policy research, or Bevy/wgpu integration work was performed.

No secondary source was needed for the material conclusions below.

## Exact pinned inputs and source registry

All sources below were accessed on **2026-09-24** and are pinned to commit `60f4ea81909200d8542eca14dccb2628b763a9a3` unless explicitly stated otherwise.

### S1 — public upscaler API header

- Path: `Kits/FidelityFX/upscalers/include/ffx_upscale.h`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: public upscaler version is 4.1.1; display-resolution-MV flag; motion-vector-jitter-cancellation flag; `jitterOffset` is the camera subpixel jitter; `motionVectorScale` is the scale factor applied to motion vectors.

### S2 — FSR4 / ML Super Resolution integration guide

- Path: `Kits/FidelityFX/docs/techniques/super-resolution-ml.md`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: motion direction, expected screen-space range, render/display MV resolution, jitter-free default, NDC example, camera-jitter units and projection conversion, requirement to set `jitterOffset` to the jitter actually applied to rendering.

### S3 — official DX12 FSR sample render module

- Path: `Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: jitter query; projection-jitter callback; context flags; GBuffer MV resource passed to FSR; dispatch `jitterOffset = -queriedJitter`; dispatch `motionVectorScale = (renderWidth, renderHeight)`.

### S4 — official Cauldron GBuffer motion-vector shader

- Path: `Kits/Cauldron2/dx12/rendermodules/gbuffer/shaders/gbufferps.hlsl`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/rendermodules/gbuffer/shaders/gbufferps.hlsl
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: previous-minus-current projected displacement; explicit jitter cancellation; explicit `float2(0.5f, -0.5f)` NDC-to-UV conversion; comment that UV `+Y` is top-down.

### S5 — Cauldron camera component

- Path: `Kits/Cauldron2/dx12/framework/core/components/cameracomponent.cpp`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/core/components/cameracomponent.cpp
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: the sample's jitter callback values are installed as projection jitter and multiplied into the projection matrix; previous jittered projection is retained.

### S6 — Cauldron scene camera constants

- Path: `Kits/Cauldron2/dx12/framework/core/scene.cpp`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/core/scene.cpp
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: current and previous projection jitter values are placed in `SceneInfo.CameraInfo.CurrJitter` / `PrevJitter`, which the GBuffer shader uses to cancel jitter.

### S7 — Cauldron transform vertex shader

- Path: `Kits/Cauldron2/dx12/framework/shaders/transformVS.hlsl`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/shaders/transformVS.hlsl
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: current and previous projected positions used by the GBuffer MV shader come from current and previous camera view-projection transforms.

### S8 — same-tag open FSR3 host implementation

- Path: `Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: host `motionVectorScale` is divided by the MV target dimensions before shader consumption; the target dimensions are render or upscale size depending on the display-resolution-MV flag; jitter cancellation uses `(previousJitterOffset - currentJitterOffset) / targetSize`.
- Evidence role: supporting open FSR3 implementation only; not evidence of proprietary FSR4 shader internals.

### S9 — same-tag open FSR3 input-MV shader callbacks

- Path: `Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_callbacks_hlsl.h
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: source MV is multiplied by the normalized `MotionVectorScale()` to form a UV motion vector; when jittered-MV mode is enabled, `MotionVectorJitterCancellation()` is subtracted.
- Evidence role: supporting open FSR3 implementation only.

### S10 — same-tag open FSR3 reprojection shader

- Path: `Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_reproject.h`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler/ffx_fsr3upscaler_reproject.h
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: history is sampled at `currentUv + motionVector`, corroborating a current-position-to-previous-position vector.
- Evidence role: supporting open FSR3 implementation only.

### S11 — same-tag open FSR3 public header

- Path: `Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant facts: the FSR3 public flag names and jitter documentation match the generic upscale API's concepts; the jitter helper returns unit-pixel offsets and the dispatch field is to be programmed with the jitter actually applied.
- Evidence role: supporting open FSR3 evidence only.

### S12 — SDK revision identity

- Path: `readme.md`
- URL: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/readme.md
- Revision: `60f4ea81909200d8542eca14dccb2628b763a9a3`
- Relevant fact: the pinned tree identifies the current SDK documentation as AMD FSR SDK 2.3.0. S1 independently defines the public upscaler API version as 4.1.1.

## Findings

### 1. Motion-vector direction

**Verified — FSR4 public contract.** The FSR4 integration guide states that the 2D vector encodes motion **from a pixel in the current frame to the position of that same pixel in the previous frame** [S2]. This is unambiguous current -> previous semantics.

**Verified — official sample behavior.** The Cauldron GBuffer shader computes projected displacement as `previousPosition / previousW - currentPosition / currentW`, then performs jitter cancellation and NDC-to-UV conversion [S4]. The ordering agrees with current -> previous.

**Verified — supporting open FSR3 behavior, not FSR4 internals.** The same-tag FSR3 reprojection code computes the history UV as `currentUv + motionVector` [S10]. That sign only reprojects to the prior location when the vector maps current -> previous. This corroborates, but is not needed to establish, the public FSR4 direction.

**Inference — directional sanity check.** If an object moves right between frame N-1 and frame N, its current X position is larger than its previous X position. A current -> previous displacement therefore has negative X at the current sample: it points left, back to the object's prior location. This follows from the direction definition; it is not quoted upstream text.

**Finding:** motion-vector direction is settled by primary AMD public evidence: **current frame -> previous frame**.

### 2. Coordinate space of stored motion vectors

The sources distinguish the upscaler's **expected scaled range** from the application's **stored resource representation**.

**Verified — FSR4 public contract.** The integration guide says FSR expects motion vectors in a screen-space range `[-width,-height] .. [width,height]`, while explicitly allowing an application to compute/store vectors in another space and use `motionVectorScale` to adjust them [S2]. It gives NDC as an example of such another space. Therefore the public contract does **not** say that the resource must always store normalized UV values.

The guide's full-screen example also fixes the orientation of its expected screen-space result. A current pixel at the upper-left with vector `(width,height)` maps back across the full surface to the bottom-right prior position [S2]. Combined with current -> previous direction, that expected screen-space convention is `+X` right and `+Y` down.

**Verified — official sample convention.** The Cauldron shader first computes a previous-minus-current projected/NDC displacement, manually cancels camera jitter, and then explicitly multiplies XY by `(0.5,-0.5)` under the source comment that this transforms motion vectors from NDC to UV and that `+Y` is top-down [S4]. Thus the **resource that the official sample hands to the upscaler stores current-to-previous, top-down UV displacement**, not raw NDC displacement.

For standard NDC `(x_ndc,y_ndc)` and top-down UV `(u,v)`, the coordinate map is

```text
u = (x_ndc + 1) / 2
v = (1 - y_ndc) / 2
```

so a displacement obeys

```text
DeltaUV = ( 0.5 * DeltaNDC.x,
           -0.5 * DeltaNDC.y )
```

which is exactly the sample shader's `(0.5,-0.5)` factor [S4].

**Unresolved / unsupported by identified primary evidence.** No inspected FSR4 source establishes raw **clip-space** displacement as an accepted stored representation. The sample divides current and previous clip positions by their respective `w` values before differencing [S4, S7]. In general, raw clip-space XY displacement cannot be converted to screen displacement by one constant 2D scale because perspective division depends on `w`.

**Finding:** AMD's public contract permits a scaled input representation and describes the target as top-down screen space; the official sample's actual stored representation is specifically **top-down normalized UV displacement**. Treating “stored vectors are UV” as the only upstream representation would overstate the documentation.

### 3. Meaning of `motionVectorScale`

**Verified — FSR4 public contract.** S1 calls `motionVectorScale` the scale factor applied to motion vectors. S2 says it is used when the application's vectors use another range so that they match FSR's expected screen-space range [S1, S2]. Conceptually, it is the component-wise conversion from the application's stored MV units into the upscaler's target displacement units.

**Verified — official sample convention.** The sample stores top-down UV displacement [S4] and dispatches `motionVectorScale = (renderWidth, renderHeight)` [S3]. For a render-resolution UV vector `d_uv`, this gives the ordinary top-down pixel displacement

```text
d_pixel = d_uv * (W, H).
```

Thus `(W,H)` is internally coherent for the sample because the stored values are UV displacements.

**Verified — supporting open FSR3 implementation.** The same-tag FSR3 host chooses an MV target size of render size or upscale/display size depending on `FFX_FSR3UPSCALER_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS`, then stores

```text
internalScale = dispatch.motionVectorScale / motionVectorTargetSize
```

[S8]. Its shader computes

```text
uvMotionVector = sourceMotionVector * internalScale
```

[S9]. Therefore, for top-down UV source vectors and `motionVectorScale = targetSize`, the internal scale becomes `(1,1)` and the UV displacement is preserved. This is strong supporting evidence for the scale interpretation, but it is not a disclosure of signed FSR4 provider internals.

**Inference — mathematically correct scales for common representations.** Let the public target be top-down pixel displacement over an MV target of dimensions `(W,H)`. Then:

| Stored resource representation | Component-wise scale to top-down pixel displacement |
| --- | --- |
| top-down pixel displacement | `(1, 1)` |
| top-down normalized UV displacement | `(W, H)` |
| standard NDC displacement (`+X` right, `+Y` up, span 2) | `(W/2, -H/2)` |

These are coordinate conversions, not separate AMD API modes. They follow from the standard NDC-to-UV map and the sample's explicit `(0.5,-0.5)` conversion [S4].

For a fixed `fsr-sdk-rs` profile that **chooses** render-resolution, top-down UV vectors, deriving the native scale as `(renderWidth, renderHeight)` is therefore consistent with the official sample and coordinate mathematics. That derivation is a wrapper-profile choice; the broader AMD contract itself permits other stored ranges via `motionVectorScale`.

### 4. The raw-NDC + `(W,H)` documentation discrepancy

This is a real source-level inconsistency and should not be silently reconciled.

**Verified — documentation text.** The FSR4 guide labels its shader value an NDC-space motion vector and computes previous NDC minus current NDC. It then shows host `motionVectorScale = (renderWidth, renderHeight)` [S2].

**Verified — official sample text/code.** The Cauldron shader performs the same previous-minus-current projected/NDC subtraction, adds its jitter-cancellation term, and then explicitly converts NDC displacement to top-down UV by multiplying `(0.5,-0.5)` [S4]. The sample host then sets the same `(renderWidth,renderHeight)` scale [S3].

**Inference — numerical consequence.** Standard NDC spans two units across either screen axis before the Y inversion into top-down texture/screen coordinates. Therefore

```text
DeltaPixel.x =  (W/2) * DeltaNDC.x
DeltaPixel.y = -(H/2) * DeltaNDC.y
```

whereas the documentation example as written implies

```text
DeltaPixel_doc.x = W * DeltaNDC.x
DeltaPixel_doc.y = H * DeltaNDC.y.
```

The latter is a factor of two too large in both axes and has the wrong Y sign relative to the guide's own top-down screen-space interpretation and the sample's explicit NDC-to-UV transformation.

A one-pixel horizontal displacement illustrates the factor directly: one screen pixel corresponds to `2/W` in NDC X, so multiplying raw NDC by `W` yields magnitude `2`, not `1` pixel. Likewise, top-down Y requires the sign inversion encoded by `-H/2`, not `+H`.

**Finding / characterization.** The **example as written is not numerically consistent** with standard NDC, the guide's stated screen-space range, and the pinned official sample. The strongest source-based explanation is that the documentation example omits the NDC-to-top-down-UV factor `(0.5,-0.5)` (or equivalently should have shown `(W/2,-H/2)` for raw NDC). Calling this a distinct but internally consistent convention is not supported by the pinned sample. There is no inspected evidence proving that the text is stale from an earlier API, so the “stale text” explanation remains **Unresolved** rather than asserted.

Because the FSR4 provider is signed/proprietary, this source investigation cannot independently prove how that provider would behave if an application literally supplied raw NDC plus `(W,H)`. The contradiction is nevertheless present in AMD's public source material.

### 5. `jitterOffset`: units and semantic meaning

**Verified — FSR4 public contract.** The FSR4 guide states that jitter values returned by `ffxQueryDescUpscaleGetJitterOffset` are in **unit pixel space** [S2]. It shows their conversion into projection-space offsets as

```text
projectionJitter.x =  2 * pixelJitter.x / renderWidth
projectionJitter.y = -2 * pixelJitter.y / renderHeight
```

and requires the application to set dispatch `jitterOffset` so FSR is informed of the jitter offset **that has been applied to render each frame** [S2]. S1 describes the field as the subpixel jitter offset applied to the camera.

This fixes two points:

- `jitterOffset` is **not** normalized UV and is **not** the NDC/projection-matrix translation itself; it is a subpixel offset in pixel units.
- Semantically it denotes the jitter actually represented by the current rendered projection/camera, regardless of whether the sequence came from AMD's helper or an application-defined sequence.

**Verified — rendered inputs.** S2 says FSR relies on the application applying subpixel jitter while rendering and says render-resolution inputs other than motion vectors should be rendered with jitter; motion vectors are normally the exception [S2].

**Finding:** the invariant for an external renderer is: dispatch the **pixel-space jitter corresponding to the projection actually used for the current rendered frame**.

### 6. Jitter sign conventions in the official sample

The official sample appears contradictory only if the queried jitter pair is mistaken for the semantic API value without following how Cauldron applies it.

Let AMD's queried pixel-space pair be

```text
q = (qx, qy).
```

**Verified — documented conversion.** The integration guide's illustrated pixel-to-projection conversion is

```text
P(q) = ( 2*qx/W, -2*qy/H ).
```

[S2].

**Verified — sample projection behavior.** The DX12 sample's jitter callback instead passes

```text
(-2*qx/W, +2*qy/H) = P(-q)
```

into the Cauldron camera [S3]. The camera component installs those callback values into its jitter translation matrix used for the jittered projection [S5]. Thus the sample actually renders with the pixel-space jitter `-q` under the documentation's own conversion convention.

**Verified — sample dispatch behavior.** The sample dispatches

```text
jitterOffset = (-qx, -qy)
```

[S3]. That is the same pixel-space jitter represented by the projection it actually used.

**Finding:** the sample does **not** contradict the semantic API contract. Its local variables `m_JitterX/Y` hold the helper/query output, but Cauldron negates that sequence when forming its projection and then dispatches the correspondingly negated pixel-space value. The stable external invariant is not “use the query sign unchanged”; it is “`jitterOffset` must encode, in pixel units, the jitter actually present in the current rendering.”

This conclusion concerns source-level sign consistency only. It is not a new signed-provider runtime validation.

### 7. Motion-vector jitter cancellation

**Verified — FSR4 public contract.** The FSR4 guide states that motion vectors should normally **not** have jitter applied, unless `FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION` is present [S2]. S1 defines that creation flag as indicating that the motion vectors have the jittering pattern applied to them [S1]. Therefore the default contract is jitter-free MVs; the flag exists for inputs whose vectors retain projection jitter.

**Verified — official sample convention.** The sample context-creation flags in S3 do not enable the MV jitter-cancellation bit. Instead, the GBuffer shader explicitly computes a `PrevJitter - CurrJitter` term and incorporates it before the NDC-to-UV conversion, under a `cancelJitter` variable name [S4]. The current and previous jitter values come from the current/previous jittered projection state [S5, S6]. Thus the sample manually produces jitter-free vectors before dispatch and does not ask FSR to remove jitter internally.

**Verified — supporting open FSR3 implementation.** When the corresponding FSR3 creation flag is enabled, the host computes

```text
motionVectorJitterCancellation =
    (previousJitterOffset - currentJitterOffset) / motionVectorTargetSize
```

[S8], and the input-MV shader subtracts that cancellation quantity from the scaled UV motion vector [S9]. This makes the relevant temporal difference explicitly **previous jitter minus current jitter**, normalized by the MV target dimensions in that implementation.

**Finding:** without the flag, supply vectors with projection jitter removed. With the flag, the public meaning is that supplied vectors include the jitter pattern and FidelityFX is asked to cancel it. The open same-tag FSR3 code corroborates that the cancellation is based on the previous-to-current jitter difference, but it must not be presented as proof of signed FSR4 shader internals.

### 8. Render-resolution versus display-resolution motion vectors

**Verified — FSR4 public contract.** The normal MV resource resolution is render resolution. If `FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS` is set, the resource should instead be at presentation/display resolution [S1, S2]. Neither S1 nor S2 defines a different temporal direction for display-resolution vectors; current -> previous is stated as the general MV semantic [S2].

**Verified — supporting open FSR3 implementation.** The same-tag FSR3 code switches the `motionVectorsTargetSize` used for both MV-scale normalization and jitter-cancellation normalization from render size to upscale size when the display-resolution-MV flag is set [S8]. Therefore the same normalized displacement semantics are applied relative to the selected MV target resolution in the open FSR3 implementation.

**Inference for a UV profile.** If an application stores top-down normalized UV displacement at display resolution and uses the same scale-to-pixel convention as the sample, the corresponding scale is `(displayWidth, displayHeight)`, not `(renderWidth, renderHeight)`. For render-resolution top-down UV, it is `(renderWidth, renderHeight)`.

**Caveat.** This investigation does not expose the signed FSR4 provider internals. The public FSR4 source establishes the resource-resolution switch and unchanged direction semantics; the exact internal normalization behavior is corroborated by open FSR3, not directly inspected in FSR4.

The current first-profile choice to expose only render-resolution vectors is therefore narrower than AMD's capability, not evidence that AMD itself requires render-resolution MVs in all modes.

### 9. Comparison with the current D011 proposal

The following comparison evaluates evidence only; it does not select architecture.

| Current proposed semantic claim | Evidence status | Source-based assessment |
| --- | --- | --- |
| Motion is current -> previous | **Verified** | Directly and explicitly stated by FSR4 documentation [S2]; official sample [S4] and open FSR3 reprojection [S10] agree. |
| Stored MVs use top-down UV axes | **Verified as official sample convention; not the sole FSR4 public representation** | Cauldron explicitly converts NDC to top-down UV with `(0.5,-0.5)` [S4]. FSR4 docs, however, allow another stored range/space via `motionVectorScale` and even claim NDC as an example [S2]. A wrapper may deliberately fix UV as its profile, but that is narrower than the upstream contract. |
| UV vectors use `(W,H)` scale | **Verified for the sample convention; mathematically supported** | Sample stores UV [S4] and dispatches render `(W,H)` [S3]. Open FSR3 normalization [S8,S9] corroborates the interpretation. Raw NDC would instead require `(W/2,-H/2)` by coordinate mathematics. |
| Vectors are normally jitter-free | **Verified** | FSR4 guide states this directly, with the jitter-cancellation flag as the exception [S2]. The sample manually cancels jitter and does not set the flag [S3,S4]. |
| Dispatch jitter is the actual pixel-space rendering jitter | **Verified** | FSR4 guide says helper values are unit-pixel and `jitterOffset` must report the jitter applied to render each frame [S2]. The sample's matched projection/dispatch negation demonstrates a local-sign convention that preserves this semantic invariant [S3,S5]. |
| A wrapper with a fixed render-resolution UV profile may derive native scale from render size | **Inference strongly supported by official sample convention** | For top-down UV, `(renderWidth,renderHeight)` is the direct UV-to-pixel conversion and exactly what the sample dispatches [S3,S4]. This is a legitimate profile derivation, but AMD's broader API permits other representations and display-resolution MVs. |

A separate source conflict must travel with any maintainer review: S2's illustrative **raw NDC + `(W,H)`** example contradicts the explicit `(0.5,-0.5)` conversion in S4 and the coordinate mathematics. The record should not quote the documentation example as proof that raw NDC with `(W,H)` is correct.

## Baseline delta

Relative to the supplied repository baseline, this investigation adds or sharpens the following evidence without changing policy:

- The proposed **current -> previous** direction is no longer merely inferred from sample behavior; it is explicitly stated in the pinned FSR4 integration guide [S2].
- The official sample's stored representation is pinned precisely: previous-minus-current projected displacement, manual jitter cancellation, then `(0.5,-0.5)` into **top-down UV** [S4].
- The documentation/sample discrepancy is localized: the FSR4 guide's NDC example omits the conversion needed before an `(W,H)` scale can mean pixels. The example is numerically inconsistent as written; the sample is internally coherent.
- `motionVectorScale` is better separated into (a) the FSR4 public role of mapping application MV range to the expected displacement range, (b) the official sample convention of UV multiplied by target dimensions, and (c) the open-FSR3 internal normalization by the selected MV target dimensions [S2,S3,S8,S9].
- Jitter sign handling is resolved at the semantic level: Cauldron negates the helper output both when constructing the projection and when filling dispatch `jitterOffset`; therefore its API value still denotes the actual pixel-space rendering jitter [S2,S3,S5].
- The jitter-cancellation flag's public meaning is separated from the sample's manual cancellation. The sample supplies already de-jittered MVs; the same-tag open FSR3 implementation shows how a flagged path uses previous-minus-current jitter internally [S1,S2,S4,S8,S9].
- Display-resolution MVs are explicitly an upstream-supported mode, while a render-resolution-only first wrapper profile remains a narrower project choice [S1,S2].

The existing M6a one-frame proof remains unchanged in evidentiary scope: zero MVs plus reset cannot validate temporal direction, scale, Y orientation or cancellation behavior.

## Limits and follow-up

1. **No current repository ZIP was available.** The local D011 text, M6/M6a research records, native proof source, and any additional repository research-process instructions could not be independently inspected. The supplied baseline was preserved in this record, but repository-local cross-checking remains outstanding.
2. **No AMD GPU / no signed-provider experiment was performed.** Nothing here newly runtime-validates current -> previous, `(W,H)` UV scaling, Y orientation, jitter signs, the cancellation flag, or display-resolution MVs against FSR4 4.1.1.
3. **The signed FSR4 implementation is not public.** Open FSR3 sources [S8-S11] are used only as supporting implementation evidence and must not be described as the FSR4 binary implementation.
4. **The NDC documentation discrepancy is source-settled but provider behavior is not.** The raw-NDC + `(W,H)` example is mathematically inconsistent with the sample and standard NDC-to-screen conversion. Public sources do not reveal whether the signed provider has any undocumented accommodation for that literal input; relying on such accommodation would require a targeted runtime experiment.
5. **“Stale from an earlier API” is not established.** No inspected pinned source proves the provenance of the inconsistent NDC example. The best-supported description is an omission/inconsistency in the example, while its historical cause is unresolved.
6. A future native temporal experiment, if maintainers choose to run one, can separate the remaining runtime questions efficiently: a two-frame known horizontal/vertical translation to catch direction, factor-of-two and Y-sign errors; a stationary scene with changing jitter to compare manual MV de-jittering against the cancellation flag; and, separately, a display-resolution-MV case. Such tests are follow-up suggestions only and were not performed here.

## Narrow evidence summary

1. **What motion-vector direction does AMD's public evidence establish?**  
   **Verified:** current-frame position -> previous-frame position. The FSR4 guide states this explicitly [S2], and the official sample's previous-minus-current calculation agrees [S4].

2. **What coordinate representation does the official sample actually provide to FidelityFX?**  
   **Verified:** current-to-previous **top-down normalized UV displacement**, with projection jitter manually removed. Cauldron converts the projected/NDC displacement by `(0.5,-0.5)` and explicitly identifies the result as UV with `+Y` top-down [S4].

3. **What exactly does `motionVectorScale` convert, and when is `(W,H)` correct?**  
   **Verified / Inference:** the public role is to scale the application's stored MV range to FSR's expected displacement range [S1,S2]. `(W,H)` is correct for normalized top-down UV displacement relative to an MV target of width `W`, height `H`; it converts UV displacement to pixel displacement. Open FSR3's `scale/targetSize` normalization corroborates this interpretation [S8,S9].

4. **Is the raw-NDC + `(W,H)` documentation example numerically consistent?**  
   **No, by coordinate mathematics.** Raw standard NDC requires `(W/2,-H/2)` to become top-down pixel displacement. `(W,H)` doubles magnitude and leaves Y with the wrong sign.

5. **If not, what source evidence best explains the discrepancy?**  
   **Inference:** the pinned official sample supplies the missing transformation explicitly: NDC displacement is multiplied by `(0.5,-0.5)` before the host uses `(W,H)` [S4,S3]. The best-supported characterization is therefore an omitted NDC-to-top-down-UV conversion in the documentation example. A “stale text” origin is **Unresolved**.

6. **What are the units and semantic meaning of `jitterOffset`?**  
   **Verified:** subpixel **pixel units**, representing the jitter actually applied to the current rendered camera/projection. It is not NDC/projection translation itself [S1,S2].

7. **Does the sample's jitter sign behavior actually contradict the documented contract?**  
   **No at the semantic level.** The sample turns helper output `q` into projection jitter corresponding to `-q` and dispatches `jitterOffset = -q`; projection and dispatch therefore describe the same actual pixel-space jitter [S2,S3,S5].

8. **Are motion vectors normally expected to have projection jitter removed?**  
   **Verified:** yes. S2 says MVs should normally be jitter-free. The sample cancels jitter in the GBuffer shader [S4].

9. **What does the jitter-cancellation flag change?**  
   **Verified public meaning:** it declares that the supplied MVs include the jittering pattern, so the upscaler is asked to cancel it instead of requiring already de-jittered vectors [S1,S2]. **Supporting open FSR3 evidence:** cancellation uses `(previousJitter - currentJitter) / MVTargetSize`, then subtracts that normalized term from the scaled MV [S8,S9].

10. **Which D011 motion/jitter statements are well-supported enough to carry into maintainer review?**  
    The evidence strongly supports carrying forward for review: current -> previous direction; a **deliberately fixed** top-down-UV wrapper profile matching the official sample; `(renderWidth,renderHeight)` for that render-resolution UV profile; jitter-free MVs by default; and pixel-space dispatch jitter that matches the actual rendered projection. The qualifier matters: top-down UV is the official sample convention and a coherent wrapper profile, not the only representation the broader FSR4 documentation claims to permit. Deriving native scale from render size is therefore well-supported **for that fixed profile**, not an upstream requirement for every valid integration.

11. **Which points remain unresolved without a targeted native temporal experiment?**  
    Runtime behavior of the signed FSR4 4.1.1 provider under nonzero temporal motion has not been newly validated; neither has its response to the documentation's literal raw-NDC + `(W,H)` pair, the jitter-cancellation flag path, or display-resolution MVs. The historical cause of the inconsistent NDC example is also unresolved. Those limits do not erase the public-source contradiction; they limit claims about proprietary-provider runtime behavior.

Research supplies the evidence above; it does not select or accept the final D011 policy.
