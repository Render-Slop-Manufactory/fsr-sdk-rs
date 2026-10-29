# Raw motion formats and per-axis scale experiment

Frozen before first execution, 2026-10-20. This is a candidate contract and
experimental protocol, not acceptance of a public API or new resource profile.

## Candidate contract and questions

Preserve current UV/pixel defaults. A future explicit choice would forward two
native per-axis scale components unchanged, sharing mapping across both owners.
Finiteness, zero/negative-zero and extreme finite values still require design
review. This probe uses only small, finite, exactly representable dyadic values;
it establishes no general numerical or access-safety domain. RG32F render-motion
input is the sole new role/format candidate. All other formats, maximum
allocations, ownership and GPU duties stay at the existing baseline.

Questions: do selected providers execute RG32F motion inputs, and do signed,
unequal native scales behave like their corresponding motion encodings?
Distinguish acknowledged dispatch, completed integration and scale-consumption
hypotheses. Equal outputs alone are insufficient without sensitive controls.

## Frozen workload and criteria

Use the existing pinned signed SDK v2.3.0 deployment and explicit 4.1.1/3.1.5
selection, with live identity matching. Fresh child/context for every arm;
render 320×180, output 640×360; render-resolution motion; four serial frames,
reset only frame zero; zero jitter; no masks, sharpening, dynamic output or HDR.
Colour is a periodic dyadic RGB pattern translated four X pixels and/or two Y
pixels per frame. Constant R32F depth is 0.25, exposure is 1.0. Output is freshly
seeded to half-float 65504 each frame. Resources retain one mip/slice/sample.

For each X-only, Y-only and combined scene, baseline stored motion is respectively
`(-4,0)`, `(0,-2)` or `(-4,-2)` after frame zero, in render-pixel units. Five arms:

| Arm | Motion format | Native scale | Stored motion relative to baseline |
|---|---|---|---|
| half | RG16F | (1,1) | unchanged |
| float | RG32F | (1,1) | unchanged |
| asymmetric | RG32F | (2,4) | divide X by 2, Y by 4 |
| signed | RG32F | (-2,-4) | divide X by -2, Y by -4 |
| wrong | RG32F | (2,4) | same bytes as signed; deliberately reversed motion |

Frame-zero stored motion is positive zero on both axes. Signed zero arising on
an inactive later axis is preserved in its input bytes; it is not an API policy
or a test of arbitrary all-zero scales. The signed and wrong arms have identical
textures; only their scale signs differ. Native descriptors preserve scale bits.

Thirty native children (15/provider), plus one four-frame no-dispatch control.
Require all GPU inputs to match independent Python-generated packed bytes,
C++ pinned-helper resource parity, completed fences, zero device/debug errors,
finite/full output writes and terminal context destruction. The no-dispatch
control must preserve every output seed. Per-child watchdog is 180 seconds.

Frozen hypotheses compare all four output frames byte-for-byte to the RG16F
baseline: float/asymmetric/signed encodings should be identical. Wrong-sign arms
must match reset-frame output and differ in at least one subsequent frame for
each axis/combined scene. A false hypothesis is retained as `observed`, separate
from failed integration; exact equality is an experimental prediction, not an
SDK guarantee or quality criterion. No threshold tuning or silent reruns.

## Execution, evidence and exclusions

```powershell
python tools/verification/run.py run experiment-motion-scale --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The probe uses raw descriptors in isolated local contexts. It does not widen
production bindings or validation. Both public families remain implementation
work after design acceptance. Helper changes retain existing geometry exports
and their default RG16F motion behaviour; the new profile opts into extra exports.
The coordinator captures all preparation/build/child commands, source identities,
selected provider, input/output hashes, diagnostics, fences and comparisons.
Raw payloads stay under ignored `target/`; retain compact sanitized measurements.

Exclude typeless/depth/sRGB/BGRA extensions, display-motion/variable-output
semantics, NDC conversion, zero/extreme scale-domain policy, provider 2.3.4,
image-quality/performance claims, concurrent execution and release acceptance.
Ordinary debug silence and finite readbacks do not prove bounded native access.
GBV is not selected; its existing signed-provider stall remains unresolved.
