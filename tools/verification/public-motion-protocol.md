# Public native motion-scale and RG32F verification

Frozen before first public execution, 2026-10-21. Implements the bounded native
verification in accepted [D034](../../docs/adr/D034.md). This is public-path
integration verification; the earlier raw spike remains separate evidence.

## Matrix and ownership

Pinned signed SDK v2.3.0, explicit 4.1.1/3.1.5 identities, both local/movable
owners; all calls remain serial on one thread. Each child owns a fresh context
for three frames, reset only frame zero, with an explicit helper queue fence
before replacing resources or destroying the context. Render/colour/depth stay
320×180, output allocation 640×360; motion allocation equals maximum render or
maximum display according to context options. All textures retain one
mip/slice/sample. No masks, sharpening, HDR or concurrent GPU work.

Twenty-one dispatch workloads per provider/family, plus an independent equipment
control per family/provider (84 dispatch children/252 frames and four controls/
12 frames). Ten preparation/build cases plus four group comparisons give
102 total cases. Each native child has a 180-second watchdog.

| Workload per group | Cells | Selection |
|---|---:|---|
| Fixed render/display RG32F | 10 | Each resolution: pixel reference, positive `(2,4)`, mixed-sign `(-2,4)`, wrong-X and wrong-Y sign controls |
| Existing RG16F UV defaults | 2 | Render/display, constructor defaults, no override; integration checked without a precision-equivalence claim |
| Jitter-inclusive RG32F | 6 | Each resolution: pixel reference, mixed-sign encoding and deliberately misencoded jitter control |
| Dynamic display RG32F with UAV allocation flag | 3 | Reference, mixed-sign and explicit override clearing; output 640×360 → 512×288 → 640×360 inside maximum allocations |

## Generator and independent oracle

RGBA16F colour is a continuous periodic triangle pattern: red/green use
`0.125 + 0.75 * abs(((position mod 16) - 8) / 8)`, position is pixel centre plus
current projection jitter minus four X/two Y pixels per frame. Blue is 0.25,
alpha 1.0. All colour arithmetic is dyadic and exactly representable in half
precision. Depth is R32F 0.25, exposure R32F 1.0; each fresh output is seeded to
half-float 65504. Python generates expected bytes independently, including the
declared truncating half packing for RG16F motion and binary32 rounding for RG32F.

Scene displacement after reset is render-pixel `p=(-4,-2)`. For display motion,
use `(-4*active_width/320, -2*active_height/180)`, with binary32 arithmetic.
Jitter-inclusive cells use pixel offsets `(0.25,-0.25)`, `(-0.25,0.25)`,
`(0.125,-0.375)` and enable `MotionVectorJitter::Included`. Define
`delta_j = previous_j - current_j`; on reset use previous=current and p=0.
Both matched arms have identical reported jitter, colour, geometry and context
flags. The encoded native pixel contribution is `p + delta_j` under the declared
open-provider convention. For scale S, store the whole `(p + delta_j) / S`
componentwise, including the jitter term. The pixel reference uses S=(1,1).
This defines the fixture's convention, not a universal renderer NDC/jitter rule.

Wrong-axis controls keep the mixed-sign texture bytes but change just that
axis's descriptor sign. The jitter control deliberately stores `p/S + delta_j`,
leaving the jitter term unscaled. Exactly representable scale components avoid
division-rounding confounds. RG32F is used in all matched comparisons; RG16F UV
defaults are separate integration cells. Override clearing must report None and
restore pixel scaling. Explicit scales remain literal during output transitions.

## Required criteria and limits

All five GPU resources are read back after each fence. Python requires exact
input bytes, recomputes output finite/written counts over the active image and
audits complete metadata, native scale/jitter bits, owner family, live provider,
format, actual UAV usage and cleanup. Pinned C++ helper conversion checks all
resource descriptions. An independent C++ device query reports view capability
bits; the production preflight and unsafe dispatch independently perform their
own new-format validation. Controls preserve the entire output seed. Native
recording rejection, missing evidence, diagnostics or incomplete cleanup fail.

Positive/mixed encodings, jitter-inclusive mixed encoding and dynamic mixed/
cleared overrides must match their corresponding pixel reference byte-for-byte
in all three output allocations. Wrong-axis and misencoded-jitter controls must
match at reset and differ in at least one subsequent frame. Freeze these
criteria; retain failures rather than relaxing a threshold after execution.
There are 56 group criteria and 168 paired frame comparisons.

These compare equivalent encodings and detect a sensitive mapping interaction;
they assess no reconstruction quality. Equivalence alone does not prove native
jitter cancellation consumption, nor internal closed-provider arithmetic.
Rounded/padding writes are observations, not a preservation or safety claim.
Zero/extreme scales remain controlled bit tests, not a native domain campaign.
Typeless/depth/sRGB/BGRA formats, provider 2.3.4, GBV, performance, overlapping
execution and final package verification stay outside this batch.

```powershell
python tools/verification/run.py run native-motion-scale --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The local run retains commands, exact source/runtime identities, metadata,
packed inputs/readbacks and comparison results under ignored `target/`.
Permanent verification records retain compact sanitized metrics/hashes, not raw
payloads or SDK binaries. Reproduction does not guarantee identical GPU bytes
on another driver/device/provider configuration.
