# Source investigation: non-linear colour and the remaining signal contract

- Investigation and source-access date: 2026-10-15.
- Question: after linear HDR, do pinned SDK sources establish enough provider,
  sRGB/PQ, pre-exposure and output semantics to implement the next M16 slice?
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, accepted
  [D026](../../adr/D026.md), SDK v2.3.0 at commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Method: inspect current Rust source, pinned headers, FSR4 guide, open provider
  adapters, official sample and its display pipeline. No runtime loading,
  compilation, GPU probe, image-quality study or implementation change.
- Outcome: flag mapping is clear. A sample/guide mismatch weakens the proposed
  context/hint validation rule; a complete PQ/exposure contract remains unresolved.
  This record supplies recommendations, not a new decision or support claim.

## Inputs and provenance

All external paths below are in the pinned commit, accessed on the date above.
Line anchors refer to physical source lines; browser text extraction folds some
blank lines and must not supply GitHub line numbers.

| Source | Inspected surface |
|---|---|
| [Upscaler header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h) | Creation/dispatch bits, descriptors, resource query |
| [FSR4 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md) | Input resources, exposure, HDR and non-linear colour |
| [Open FSR3 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp) | `ConvertFlags` and main dispatch translation |
| [Open FSR2 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp) | Same creation/main-dispatch boundaries |
| [Sample implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp), [header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.h) | Colour selector, reinitialization, creation and dispatch |
| [Sample configuration](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/config/fsrapiconfig.json) | Render-module order |
| [Tonemapping module](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/render/rendermodules/tonemapping/tonemappingrendermodule.cpp), [shader](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/framework/shaders/tonemapping.hlsl) | Display gamut, luminance scale and PQ encoding |

The cached header's LF-normalized SHA-256 matches `tools/native-headers.json`:
`65be36d5653eb71c10379e0204f7bb7cd6a8aeec96e6f8cffc5f59ffeb1f89bf`.
Direct HTTPS retrieval also identified the sample implementation as
`924fed10c199eb4f7014bbc6d21e7adf9f6d58424853fdab4f37a634b0080e36`
and guide as
`9c761f10d1a0a6aba02cde98adef10229f4800500e36b58f0bcb5424778f2d02`.
Raw sources and the complete, non-truncated pinned GitHub tree were inspected in
ignored `target/research/non-linear-colour-sources/`; that cache is not required
to understand this record. Initial sandboxed retrieval failed DNS resolution;
authorised HTTPS retrieval succeeded. No new SDK binaries were acquired.

Repository checks confirm portable private HDR options and the common
`native_flags()` path used by creation and memory estimates. The
[dispatch converter](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs)
still emits zero dispatch flags; non-linear bindings/enums are absent.
[Linear HDR checks](../../release/2026-10-15-linear-hdr-verification.md) retain
their pending Windows/native and MSRV status. The earlier
[HDR concept](2026-10-15-src-hdr-colour-encoding-concept.md) is the proposal being
refined, not evidence that its complete API was accepted.

## Findings

### Native declarations and provider limits

**Verified source facts:** the header gives non-linear creation bit 8 and
sRGB/PQ dispatch bits 1/2, independently of HDR bit 0. It has no fields for
primaries, white luminance, PQ scale or the stage at which pre-exposure is applied.
Its resource query reports resources rather than colour capabilities.

The open FSR3 adapter translates HDR, but drops non-linear creation and both
encoding hints: main dispatch starts flags at zero and adds only debug drawing.
The open FSR2 adapter likewise omits non-linear creation and does not forward
these main-dispatch hints. This establishes absence of those explicit controls
in the inspected adapters, not that their algorithms cannot process any
non-linear numerical input.

**Upstream claim:** the FSR4 guide describes non-linear input and optional,
mutually exclusive sRGB/PQ hints. Its signed provider is the documented candidate
for these controls. **Unresolved:** no FSR4 implementation inspected here proves
its transfer calculations or complete combination support. Native success,
resource masks and provider names/opaque IDs are not capability detection.

### The sample does not establish the proposed context/hint invariant

**Verified source facts:** the sample header has four distinct selector values.
The UI callback requests recreation for values 0/1 only. Creation enables the
non-linear bit only for value 1; dispatch sets the sRGB/PQ hints for values 2/3.
See [selector/callback](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp#L409-L425),
[creation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp#L960-L972)
and [dispatch](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp#L1474-L1489).

**Source-derived inference:** with no unrelated reinitialization, two UI paths
can reach the same sRGB hint with different retained creation settings:

| Path from the default linear context | Non-linear creation bit | Dispatch hint |
|---|---|---|
| Select sRGB directly | Off | sRGB |
| Select non-linear, recreate, then select sRGB | On | sRGB |

The guide describes hints in addition to non-linear creation, while this sample
can exercise them independently. Sources do not settle whether that is intended
provider behavior, a sample defect or a diagnostic choice. The earlier concept's
rule requiring a NonLinear context for Srgb/Pq is therefore a proposed
conservative wrapper policy, not an established universal native precondition.
Do not silently relax or accept that policy as a result of this investigation.

### Display PQ is separate from upscaler input PQ

**Verified source facts:** the sample configuration orders FSR before
tonemapping. The FSR module copies its colour input into an intermediate texture;
its colour selector affects flag assignments, not an encoding pass in that module.
It forwards scene exposure as pre-exposure and always requests automatic exposure.

The later tonemapper reads the HDR target and writes the swapchain proxy. It
selects a REC709-to-REC2020 matrix for HDR display output; its shader scales by
display maximum luminance divided by 10,000 before PQ encoding.

**Inference:** those explicit display conventions are useful for constructing a
separately declared test hypothesis. They do not specify the FSR4 provider's PQ
input normalization, primaries, output re-encoding or pre-exposure order. The
sample supplies no matched encoded-input/linear-reference proof here.

### Exposure and output are only partially specified

**Upstream claims:** the guide describes division by pre-exposure to recover the
original signal, subsequent exposure multiplication, and reversal of internal
tonemapping before output in the input signal domain. This supports retaining
the existing linear contract. It is not a per-encoding transfer formula.

**Unresolved:** the inspected sources do not choose between decoding before
pre-exposure removal and arithmetic on encoded values, define how changing
pre-exposure interacts with PQ, or specify every automatic/explicit exposure
combination. The same-domain statement supports an encoded-output expectation
as an inference; it does not establish a verified PQ output transfer or gamut.
Do not redefine `PreExposure` as a multiplier of sRGB/PQ code values, apply a
silent factor of 1, or treat an absent encoding hint as linear input.

## Baseline delta and recommendation

The investigation confirms the additive native mapping and provider limits from
the earlier concept. It adds two material qualifications: the sample does not
establish its proposed context/hint restriction, and the sample's concrete PQ
luminance/gamut conversion belongs to display output after FSR.

**Recommendation:** a direct native-control API can be designed with the
documented flags and explicit caller-established provider compatibility. Sources
do not block descriptor plumbing. Before implementation, explicitly decide
whether context/hint mismatch rejection is the supported wrapper profile and
what non-linear signal/exposure domain it claims. A general verified PQ contract
is not ready from this source evidence alone. Do not invent automatic provider
selection, colour conversion or numerical normalization to fill the gaps.

The next useful evidence would be provider-specific documentation or a bounded
native semantic probe on the pinned signed FSR4 runtime. A proposed first probe
would use known constant/patch inputs in RGBA16F, pre-exposure and explicit
exposure both 1, and independently checked encoders/readbacks. Compare declared
linear, non-linear-only and non-linear-plus-hint cases, with a separately trusted
sample-style hint-only diagnostic if safe for the selected provider. Record
actual creation bits, provider identity and whether output is consistent with
encoded or linear candidate domains. Only then vary exposure/pre-exposure to
distinguish operation order. This is a recommendation, not an accepted protocol,
new release gate or authorization to execute a probe.

## Limits

No signed provider was executed or reverse-engineered. Standard transfer-function
names and generic display helpers do not establish native acceptance. No broad
provider/driver/format guarantee, PQ implementation or new compatibility claim
follows. Linear HDR native confirmation remains separate and pending.

No code, accepted decision, roadmap status, synthesis or research index changed.
The record contains original analysis and links; no AMD source snapshot or binary
is redistributed. Inspected upstream files retain their existing AMD notices;
the SDK pin, dependencies and project licensing are unchanged.
