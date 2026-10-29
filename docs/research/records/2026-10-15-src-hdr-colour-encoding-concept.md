# Source investigation: HDR and non-linear colour API concept

- Investigation date and source access: 2026-10-15.
- Question: extend the existing upscaler with M16 HDR/encoding controls while
  preserving constructor calls, native parameter meaning and future extension.
- Baseline: current `fsr-sdk` 0.3.0 / `fsr-sdk-sys` 0.1.2 development source,
  pinned AMD SDK v2.3.0, after D025's supplied-mask implementation. M15 Windows
  confirmation is explicitly deferred because no Windows host is available.
- Method: inspect current Rust options/dispatch/query implementations, accepted
  decisions and pinned upstream headers, guides and open FSR3 adapter. No code
  experiment, native execution, ABI compilation or image-quality study.
- Status: **proposal**, not an accepted decision, implementation or release
  selection. Existing roadmap/release deferral edits are preserved separately.

## Evidence and current implementation

All upstream paths below refer to commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`, not a floating SDK revision.

1. **Verified source facts:** the [shared header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L42-L60)
   defines independent creation flags HDR = 1 and non-linear colour = 256,
   and per-dispatch sRGB = 2 / PQ = 4. These are existing descriptor fields,
   not new exports or descriptor chains.
2. **Upstream claims:** the [FSR4 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#hdr-support)
   prefers linear input; non-linear input uses the creation flag, with optional,
   mutually exclusive sRGB/PQ hints. Its [exposure section](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#exposure)
   distinguishes pre-exposure from exposure, describes pre-exposure before
   lower-precision packing, and says internal tonemapping is reversed on output.
   This is not an independently established signed-provider transfer/output
   contract for every encoding/exposure combination.
3. **Verified source limitation:** the [open FSR3 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L28-L47)
   forwards HDR but omits non-linear creation support; its
   [dispatch translation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L323-L356)
   starts dispatch flags at zero and forwards only debug visualization. An OK
   native return therefore cannot prove sRGB/PQ hints were consumed.
4. **Verified source fact:** the [resource query](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L185-L200)
   reports required/optional resources, without colour-capability bits.
   [D018](../../adr/D018.md) likewise gives provider IDs/names no generic capability
   or semantic-version guarantee.
5. **Current code:** [common options](../../../crates/fsr-sdk/src/upscaler/common.rs)
   have private fields and one shared creation/memory-estimate flag conversion.
   [Frame inputs](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs) use
   private fields and additive setters. [Resource inspection](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs)
   fixes colour/output to RGBA16F; [descriptor conversion](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs)
   currently sets dispatch flags to zero. Both ownership families share these paths.
   [D014](../../adr/D014.md), [D015](../../adr/D015.md),
   [D016](../../adr/D016.md), [D020](../../adr/D020.md) and
   [D023](../../adr/D023.md) govern extension, platform boundaries, direct controls
   and existing exposure semantics.

## Proposed API

Keep HDR range, context input classification and per-frame encoding hints separate,
matching their distinct native meanings. In portable `upscaler::common`, with the
existing `upscaler` reexports:

```rust
#[non_exhaustive]
pub enum InputColorSpace { Linear, NonLinear }

#[non_exhaustive]
pub enum ColorEncodingHint { Unspecified, Srgb, Pq }
```

Proposed traits for both simple selections: `Debug + Clone + Copy + Default +
PartialEq + Eq`. Defaults are Linear and Unspecified. HDR uses a boolean like the
existing dynamic-resolution switch, rather than a new type for one independent
native bit.

Add getters and setters to the existing private-field objects:

```rust
// Creation options; default false.
options.set_high_dynamic_range(true);
// Optional non-linear context; default Linear.
options.set_input_color_space(InputColorSpace::NonLinear);
// Frame hint; default Unspecified.
frame.set_color_encoding_hint(ColorEncodingHint::Pq);
```

Getter names are `high_dynamic_range`, `input_color_space`, and
`color_encoding_hint`. Constructors, dispatch signatures, resource borrows,
existing defaults and Send/Sync/input Clone contracts remain unchanged. Mutating
options never reconfigures an existing context. Portable selections contain no
DX12 types; resource/native work retains its current gates. No builder, raw flag
bitset or generic colour-management framework is needed.

`Unspecified` means exactly no encoding hint: it is not a synonym for Linear.
It preserves zero dispatch flags for both the existing linear context and the
SDK's documented non-linear-without-a-specific-hint route. Srgb/Pq require a
NonLinear context under the proposed supported wrapper profile; reject mismatches
before COM/native dispatch without poison. An enum prevents both hints being set.
HDR is independent: do not invent a blanket HDR+PQ incompatibility or silently
change HDR when an encoding hint changes. No automatic history reset is inferred;
callers retain responsibility for discontinuities and compatible temporal input.

Creation and pre-create memory estimation must use the same shared flag conversion.
Per-frame conversion changes only the two hint bits, preserving future unrelated
flags. Add only the four missing constants to raw bindings, with paired Rust/C++
assertions. Provider memory estimates stay provider results, not verified budgets.

## Resource, signal and provider contract

**Recommendation:** keep RGBA16F colour and output for the first scope. Float
storage can carry linear or encoded numerical values, but format alone proves
neither encoding nor provider acceptance. Keep explicit formats and avoid adding
sRGB views that could introduce a second decode. Wider format/view support can
be selected later without changing the setters.

The caller supplies already-correct pixels. Flags describe those pixels; the
wrapper performs no encoding, decoding, tonemapping, clamping, gamut conversion,
pre-exposure multiplication, HDR display setup or swapchain configuration.
The HDR flag describes input dynamic range, not the monitor. For the linear HDR
slice, preserve D020's existing pre-exposure meaning and explicit/automatic
exposure choices, with bounded provider-supported signals above 1.0.

**Unresolved for non-linear acceptance:** exact signal units/primaries and PQ
normalization, pre-exposure placement relative to transfer encoding, output domain
and exposure-mode interactions need a pinned provider-specific contract and
bounded integration evidence. Do not promise an output encoding from an input
hint alone, reinterpret pre-exposure as multiplying PQ/sRGB code values, or claim
all combinations work because flags exist. Do not silently substitute a factor
of 1.0 or change the exposure mode to conceal this gap.

Resource requirements cannot act as a colour capability check. Do not detect
support by provider-name substrings or silently retry in linear mode. Any supported
non-linear profile must identify the compatible actual provider and extend the
existing unsafe acquisition/dispatch obligations accordingly. Native support may
remain caller-established, as for other provider-sensitive flags; context creation
or resource-query success alone must never be advertised as capability detection.
If a robust automatic refusal mechanism is selected, its identity/override contract
must be decided explicitly rather than smuggled into this feature.

## Suggested implementation order and checks

1. **Linear HDR first:** add the HDR option, shared creation/estimate mapping,
   portable tests and public fixtures for both families. Preserve all non-HDR
   defaults. Prepare a three-frame bounded HDR case with input signals above 1.0,
   known explicit/automatic exposure, selected pre-exposure factors, full input
   readback, finite/written output, diagnostics, fencing and terminal cleanup.
2. **Non-linear extension:** add the context classification and encoding hint
   using the same additive API pattern once the initial signal/provider contract
   is explicit. Source-level mapping can be developed here; a verified support
   claim still requires the provider/runtime and output-domain checks above.
3. Check default mappings, independent/combined creation bits, exact mutually
   exclusive dispatch bits, context/hint mismatch rejection and retry, unchanged
   exposure forwarding, trait/reexport/consumer compatibility and both families.
   Fixture calls verify descriptor plumbing, not actual signal interpretation.
4. On an available Windows host, run C++ ABI, controlled fixtures, MSRV and
   bounded native cases. Keep temporal quality studies on hold. No performance
   assertion, full factorial matrix or image-quality calibration is selected.

This ordering advances M16 without waiting for a Windows machine and avoids
presenting the whole non-linear contract as already settled. M15's deferred
verification remains open and is not silently waived. Future encodings can add
non-exhaustive enum variants; format broadening can extend validation. Genuinely
new mandatory inputs would still need a separate compatibility review.

## Baseline delta and limits

HDR and non-linear support remain unimplemented in the inspected source. This
concept confirms the existing creation/frame split, narrows the first executable
slice to linear HDR and identifies non-linear provider/signal gaps that resource
metadata cannot resolve. No repository code, accepted ADR, roadmap selection or
research synthesis was changed. No new dependency, SDK pin, redistributed AMD
material or platform support is proposed. All implementation and verification
steps above are recommendations, not checks performed by this investigation.
