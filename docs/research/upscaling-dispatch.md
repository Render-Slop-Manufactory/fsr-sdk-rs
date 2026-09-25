# Upscaling dispatch: DX12 and frame semantics

- Updated: 2026-09-25.
- Scope: source research and bounded Windows/DX12 execution evidence for the
  first upscaler dispatch against AMD FSR SDK `v2.3.0`
  (`60f4ea81909200d8542eca14dccb2628b763a9a3`), whose upscaler API version
  is 4.1.1. Source analysis and executed GPU results are distinguished below.
- Included records: [dispatch ABI](records/2026-09-24-src-m6-upscale-dispatch-abi.md),
  [official DX12 sample and readback path](records/2026-09-24-src-m6-dx12-dispatch-example.md),
  [DX12 state and synchronization](records/2026-09-24-src-m6-dx12-state-and-synchronization.md),
  [frame semantics](records/2026-09-24-src-m6-upscale-frame-semantics.md),
  the [dispatch contract recommendation](records/2026-09-24-src-m6-dispatch-contract-recommendation.md),
  the [M6a native dispatch proof](records/2026-09-24-exp-m6a-native-dispatch-proof.md),
  the [bounded GPU-based-validation diagnostic](records/2026-09-24-exp-m6-gbv-dispatch-stall.md),
  the [recording-ahead experiment](records/2026-09-24-exp-m6-recording-ahead.md),
  the [motion-vector and jitter investigation](records/2026-09-24-src-m6-motion-vector-jitter-semantics.md),
  the [backend allocation investigation](records/2026-09-24-src-m6-backend-allocation-inflight-scope.md),
  and the [M6b public-route verification](records/2026-09-25-m6b-public-dispatch-verification.md).

The supplied records list pinned primary sources, but their original inline
citation tokens had no index-to-source map. Those tokens were removed at intake.
The later contract recommendation had the same citation limitation and was
integrated as an analysis of these records and the repository, not as a native
test. Selected ABI declarations were checked against the locally staged v2.3.0
headers; the full source claim set has not been independently reverified. M6a
tested a selected subset, including ABI, conversion and one submitted GPU
frame; M6b repeated the bounded GPU test through the public method. The signed
FSR4 provider's internal implementation is not available in
the public source. Same-tag open FSR3 behavior is useful corroboration, not
proof of FSR4 behavior. The two later source investigations had no repository
tree or native runtime in their research environment. Their repository baseline
came from a brief; this intake compared it with the records and proposed D011
here. Their source findings are not new GPU results.

## Current findings

**Verified source ABI:** `ffxDispatch` and `PfnFfxDispatch` already existed in the
sys loader/common declarations; M6a added the selected upscaler dispatch
descriptor, resource descriptions, states, formats and DX12 resource
conversion. M6b retained that conversion behind the public unsafe method. The
v2.3.0 header defines `ffxDispatchDescUpscale` with tag `0x00010001`, a raw command-list
pointer, inline `FfxApiResource` values, frame scalars and flags. Optional images
are inline resources with null native pointers, not null descriptor pointers.
The DX12 resource and format helpers are C++ `static inline` functions, not DLL
exports. The ABI record's x64 calculations passed paired Rust/MSVC checks in
M6a; x86 remains unverified.
[ABI record](records/2026-09-24-src-m6-upscale-dispatch-abi.md).

**Source-backed execution boundary:** AMD's DX12 sample passes a recording
graphics command list and separate color input/output textures. Dispatch
records commands; the application closes, submits and fences the list. An OK
return establishes CPU-side acceptance only. Input resources should enter in
`NON_PIXEL_SHADER_RESOURCE` and be described with matching FidelityFX state;
the sample also demonstrates a combined pixel/non-pixel read state. The output
needs UAV capability in the inspected backend and a truthful declared initial
state. The public FSR4 material does not establish a general output-state
guarantee; backend source restores registered external resources to their
declared initial states. M6a and M6b completed an output UAV-to-copy transition
without debug-layer errors on one configuration; D011 adopts restoration for
that bounded baseline, not as a universal provider guarantee. The backend
changes descriptor heaps and compute bindings, so later application commands
must rebind what they use. D011 requires a DIRECT list; COMPUTE-list support
is unestablished.
[Sample](records/2026-09-24-src-m6-dx12-dispatch-example.md) ·
[DX12 state](records/2026-09-24-src-m6-dx12-state-and-synchronization.md) ·
[M6a result](records/2026-09-24-exp-m6a-native-dispatch-proof.md) ·
[M6b result](records/2026-09-25-m6b-public-dispatch-verification.md).

**Verified bounded native execution:** paired Rust/MSVC x64 layout checks and
the five-resource comparison with AMD's inline DX12 helper passed. Through the
unchanged public M5 constructor with zero create flags, the private path used
explicit 1×1 R32F exposure and a 320×180 → 640×360 synthetic frame. Native
`ffxDispatch` returned OK; the DIRECT list closed, submitted and reached a
fence. Pitched readback found 230,400 finite pixels, all changed from the
sentinel, with a red gradient related to the input. The D3D12 debug layer
reported zero errors and no device removal on the tested RX 9060 XT, driver
`32.0.31041.1004`, signed provider 4.1.1 configuration. This establishes one
real write, not image quality, temporal correctness, broad format support or a
public API. [M6a record](records/2026-09-24-exp-m6a-native-dispatch-proof.md).

**Verified bounded public-route execution:** M6b used the same fixed-zero
constructor and five-texture profile through public `unsafe Upscaler::dispatch`.
The paired dispatch ABI check and AMD helper comparison passed. The DX12 list
submitted and reached a fence; readback again found 230,400 finite pixels, all
changed from the sentinel, with spatial variation, zero debug-layer errors and
no device removal on the tested configuration. Wrapper fixtures checked
pre-native rejection and native-failure poisoning, but an injected COM failure
does not exercise a failing real DX12 call. This establishes the first public
recording path on one configuration, not temporal quality, broad support or GBV
success. [M6b record](records/2026-09-25-m6b-public-dispatch-verification.md).

**Verified bounded recording ahead:** on the same RX 9060 XT, driver and
signed provider 4.1.1, the private path recorded 2, 5 and 8 dispatches on one
context before submitting any list. It also recorded 2 and 5 dispatches on
distinct sibling contexts sharing one runtime and device. Each recording had
its own list, allocator and resources with a distinct red input level. Separate
ordered submissions on one queue reached a final fence; every output was
finite, changed from its sentinel and reflected its distinct input. Native
dispatches returned OK, with zero D3D12 debug errors and no device removal.
This verifies that the fifth CPU-recorded call did not necessarily fail in
this fixture. It does not identify the signed provider's transient allocation
or sharing scope, establish an in-flight capacity, or validate temporal image
quality or arbitrary GPU overlap. The experiment used explicit exposure and
zero creation flags, so it adds no auto-exposure evidence.
[Recording-ahead record](records/2026-09-24-exp-m6-recording-ahead.md).

**Pinned source allocation scope:** the open DX12 backend keeps its descriptor
ring, fallback upload constant-buffer ring and their mutable heads in one
scratch-backed `BackendContext_DX12` associated with an `FfxInterface`. Effect
contexts deliberately hosted by that interface share those heads. The modern
open FSR3 3.1.5 provider instead creates a separate one-context interface for
each modern upscaler context, so its sibling contexts have separate fallback
backend rings. The internal `FFX_MAX_QUEUED_FRAMES = 4` sizes open-backend
storage; it is not a public dispatch count or fence cadence. The inspected
GPU-visible rings wrap without an explicit completion check, unlike temporary
CPU job and staging scratch, which can be reused after command encoding.
Public source does not expose the signed FSR4 4.1.1 provider's equivalent
allocator ownership, sharing, capacity or reuse behavior. The recording-ahead
GPU result fits either separate sibling allocations or shared storage with
sufficient capacity. No ordinary per-dispatch completion rule was found in the
inspected public material; the explicit FSR3 GPU-idle guidance concerns context
destruction. These source facts narrow D011's allocation question without
identifying signed-provider allocation behavior or setting a numeric limit.
[Allocation investigation](records/2026-09-24-src-m6-backend-allocation-inflight-scope.md) ·
[recording-ahead experiment](records/2026-09-24-exp-m6-recording-ahead.md).

**Verified GBV boundary; cause unresolved:** with GPU-based validation (GBV)
enabled, the same provider context, textures, uploads, barriers, submission,
fence and sentinel readback completed when FSR dispatch was omitted. With
dispatch included, Rust resource conversion and pre-call checks completed,
but the native `ffxDispatch` call did not return within 90 seconds and no GPU
work was submitted by the application. The child used roughly one CPU core
during that interval. A normal debug-layer run without GBV passed again. This
narrows the trigger to native dispatch under GBV on the tested setup; it does
not assign fault to the provider, driver, GBV instrumentation or a
provider-sensitive input. The control did not execute a compute shader. Earlier
120- and 180-second GBV timeouts are consistent with this narrower result.
[GBV diagnostic](records/2026-09-24-exp-m6-gbv-dispatch-stall.md) ·
[M6a record](records/2026-09-24-exp-m6a-native-dispatch-proof.md).

**Inference about Rust safety:** a borrow ending at `ffxDispatch` return does
not prove GPU completion. External textures, command allocator, context-owned
GPU objects and runtime must remain valid through the submitted work; state,
residency and queue ordering are application facts that this wrapper cannot
currently enforce. D011 selected an immediate-return `unsafe` boundary; a
future safe alternative would need to own or track in-flight objects and
fences. `&mut self` can serialize CPU calls on one upscaler but cannot establish
GPU ordering. D011 and the public method now expose the completion obligation
before teardown; immediate `Drop` is not a fence. The recording-ahead
experiment did not relax resource-lifetime or fence obligations.
[Synchronization record](records/2026-09-24-src-m6-dx12-state-and-synchronization.md) ·
[recording-ahead record](records/2026-09-24-exp-m6-recording-ahead.md) ·
[current owner](../../crates/fsr-sdk/src/context.rs).

**Upstream frame contract and narrow test profile:** color, depth and motion
vectors describe the same jittered current frame; motion vectors point from
current to previous positions and normally exclude jitter. For UV-encoded
motion vectors, scale by the motion texture's target dimensions. Raw NDC vectors
need the additional `(0.5, -0.5)` conversion shown in AMD's sample shader.
Dispatch jitter is the pixel offset actually applied to the projection, not
the projection translation. Use nonzero subpixel jitter for an integration
test, a positive `preExposure`, frame time in milliseconds, positive near/far
distances with matching depth flags, vertical FOV in radians, and an explicit
positive view-unit-to-metre factor. A first frame or camera cut can pulse
`reset=true`; reset is not a fence. Keep render/output extents exact, use
single-sample 2D textures with one mip and distinct color/output, and omit
optional masks for the selected FSR4 profile. D011 adopted a narrow five-texture
wrapper whitelist; these restrictions are not a published universal SDK whitelist.
[Frame record](records/2026-09-24-src-m6-upscale-frame-semantics.md).

**Pinned source motion and jitter semantics:** the FSR4 public guide defines
motion as current position to previous position, normally with projection
jitter removed. The official Cauldron sample stores top-down normalized UV
displacement after an explicit NDC-to-UV conversion of `(0.5, -0.5)` and uses
`motionVectorScale = (renderWidth, renderHeight)` for render-resolution motion
vectors. That is a coherent fixed wrapper profile, while the public guide
allows other stored representations through the scale. Raw standard NDC
displacement needs `(W/2, -H/2)` to become top-down pixel displacement; the
guide's raw-NDC example paired with `(W,H)` is numerically inconsistent with
that coordinate mapping and the official sample. The most direct source-based
explanation is an omitted NDC-to-UV conversion, not a demonstrated alternate
convention. Dispatch `jitterOffset` is the pixel-space jitter actually applied
to the current projection. The sample negates both its queried offset when
applying projection jitter and the value it passes at dispatch, so its signs
agree at the semantic boundary. The zero-flags profile requires already
de-jittered motion vectors; the jitter-cancellation creation flag declares the
other input mode. Open FSR3 code corroborates normalization and cancellation
mechanics but does not reveal the signed FSR4 provider's internals. M6a's zero
motion, one-reset-frame result has not tested these temporal semantics.
[Motion/jitter investigation](records/2026-09-24-src-m6-motion-vector-jitter-semantics.md) ·
[M6a result](records/2026-09-24-exp-m6a-native-dispatch-proof.md).

**Exposure policy:** the current `Upscaler::new` fixes creation flags
to zero. The source-backed null-exposure profile requires
`FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` at creation. To test the existing public M5
constructor end to end, M6a supplied the documented 1×1 `R32_FLOAT` exposure
resource and observed a successful dispatch. The older contract recommendation
instead proposes auto exposure and four textures. That is a different creation
policy, untested by M6a or M6b; D011 retains explicit exposure and leaves a
later public auto-exposure option undecided.
[Current construction](../../crates/fsr-sdk/src/upscaler.rs) ·
[D008](../adr/D008.md).

## Public dispatch decision and earlier recommendation

The [contract recommendation](records/2026-09-24-src-m6-dispatch-contract-recommendation.md)
turned the source findings into a concrete candidate for an API decision. It
proposed one immediate-return unsafe `Upscaler::dispatch` method using borrowed
Windows command-list and texture COM interfaces. An unsafe caller would keep
resources, backing allocations, allocator, upscaler/context and runtime alive
through GPU completion; provide truthful resource states, residency and
cross-queue synchronization; and submit same-context work in temporal order.
`&mut self` and CPU borrows constrain calls, not GPU execution. It called for a
post-dispatch completion obligation before teardown. This was research input;
D011 selected the public contract described below.

[D011](../adr/D011.md) was accepted on 2026-09-25 after maintainer policy review.
It retains zero flags and explicit exposure, fixes the first public UV/jitter
profile, uses a borrowed input struct and crate-owned errors, and makes native
dispatch failure terminal for dispatch while retaining D005 teardown. It imposes
no per-dispatch fence cadence or numeric recording limit. Instead, it explicitly
trusts provider-internal transient allocation correctness within the supported
signed runtime boundary while preserving application lifetime and temporal-order
obligations. Signed FSR4 allocation topology remains unknown; acceptance is a
policy decision, not new allocator evidence or an unlimited-capacity claim.
The experiment's earlier recommendation to retain a conservative fence rule is
superseded as policy by D011; its observations and limits remain unchanged.
M6b implementation and [bounded public-route verification](records/2026-09-25-m6b-public-dispatch-verification.md)
completed on 2026-09-25. This did not resolve signed-provider allocation
topology or the earlier GPU-based-validation stall.

The earlier handoff proposed fixed auto exposure with a null exposure resource,
no masks, sharpening or dispatch flags, and a four-texture profile. RGBA16F
color/output, R32F depth and RG16F motion vectors, all single-sample
Texture2D resources with one mip and slice, give the native image test a
specific setup. Their extents and UAV capability can be inspected. The
wrapper can derive native resource metadata from `GetDesc()` and validate
known SDK preconditions such as device identity, supported conversions,
extents and output UAV capability. It cannot prove actual D3D12 state,
backing-heap non-aliasing or GPU completion. The test's exact formats and
shape, and the handoff's fixed UV motion-vector convention, are not evidence
that the SDK rejects other formats or motion-vector encodings. D011 nevertheless
selected a narrower initial wrapper support profile.
Unlike that four-texture auto-exposure recommendation, the M6a proof of M5's
fixed-zero constructor used a fifth, explicit exposure texture. Its success
does not decide whether a future extension should offer auto exposure.

The handoff's proposed semantic inputs were pixel-space jitter, milliseconds
of frame time, camera near/far/FOV and view-unit-to-metre factor, plus a reset
pulse. The suggested types were `JitterOffsetPixels`, `FrameTimeMillis`,
`CameraParameters` and `UpscaleDispatch<'_>`; M6b implements them with checked
scalar constructors. M6a used `preExposure=1.0` for its synthetic scene, and
D011 fixes that value for the first public profile. Other motion-vector encodings
or pre-exposure would need a separate contract. The handoff also proposed
`Operation::Dispatch` and a distinct validation-error category while preserving
unknown native return codes. It also proposed fake-runtime descriptor/error
checks, paired Rust/MSVC layout checks, parity with AMD's inline DX12 resource
helper, and submitted/fenced image and temporal tests. M6a completed the
selected layout, parity and private one-frame GPU checks; M6b repeated the
one-frame GPU test through the public API. Temporal verification remains open.

**Scope assessment:** the crate's job is the ABI, ownership, descriptor
conversion, error reporting and dispatch boundary. The renderer owns image
content, motion vectors, jitter generation, resource transitions, queue
submission and fences. The native image scene and readback are project test
equipment. D011 explicitly selected a fixed first public format, exposure and
motion-vector support profile using the M6a evidence; this does not make it a
universal SDK restriction. The [M6 plan](../MILESTONE_6.md) and [D011](../adr/D011.md)
retain the boundary. M6b verified that route on one GPU configuration.

## Disagreements and limitations

- The [sample/readback record](records/2026-09-24-src-m6-dx12-dispatch-example.md)
  proposes zero jitter for a one-frame smoke test. The
  [ABI](records/2026-09-24-src-m6-upscale-dispatch-abi.md) and
  [frame-semantics](records/2026-09-24-src-m6-upscale-frame-semantics.md)
  records report AMD's nonzero jitter guidance. A zero-jitter diagnostic might
  exercise CPU/GPU plumbing, but it should not be the integration correctness
  gate. M6a used an applied nonzero jitter and matching dispatch value.
- The [ABI record](records/2026-09-24-src-m6-upscale-dispatch-abi.md) leaves
  general output pre/post states unresolved; the
  [backend inspection](records/2026-09-24-src-m6-dx12-state-and-synchronization.md)
  finds restoration to the declared state. These have different scope:
  inspected generic backend mechanics versus a public guarantee for the
  selected signed FSR4 provider. M6a's UAV-to-copy transition completed with
  zero debug-layer errors for one setup; GBV never reached submission in the
  provider case. This does not establish a universal wrapper state contract.
- The public FSR4 prose pairs raw-NDC motion vectors with `(W,H)` scale. For
  standard NDC this doubles X displacement and reverses and doubles Y relative
  to top-down pixels. The official shader's `(0.5,-0.5)` NDC-to-UV conversion
  makes its subsequent `(W,H)` scale consistent. An omitted conversion is the
  best-supported explanation; the text's historical cause and signed-provider
  response to the literal pair remain unknown. Test motion direction and scale
  against an analytic temporal scene.
- Jitter signs in the sample are internally consistent because both its
  projection offset and dispatch value negate the queried offset. The API
  invariant is that dispatch receives the pixel offset actually used to render,
  not necessarily the unmodified query result.
- The sample leaves `viewSpaceToMetersFactor` at zero, while the header only
  states its meaning and the same-tag open FSR3 implementation has a fallback.
  Neither establishes a signed FSR4 default. Set it explicitly to `1.0` when
  one view unit is one metre. Keep pre-exposure distinct from the separate
  exposure multiplier; the sample record's shorthand does not define a new
  scalar contract.
- General FSR3 guidance encourages masks; FSR4-specific documentation and the
  sample permit omitting them. Do not extrapolate this to other providers.
- The RGBA16F/R32F/RG16F test profile and chosen UAV output boundary worked
  with the signed provider on one configuration. The handoff's fixed format
  whitelist, UV motion-vector scale and pre-exposure are test choices;
  promoting them to public restrictions narrows an engine-independent
  SDK wrapper. D011 explicitly selects that narrow first support profile; it is
  not an upstream-wide restriction. Exact COM identity checks cannot detect two
  different placed resources sharing backing memory.
- M6a established one private `ffxDispatch`, GPU execution and structurally
  plausible output on the recorded Windows configuration. M6b repeated the
  bounded result through the public method. GBV control work passed, but
  native provider dispatch under GBV consumed CPU until timeout before
  submission. Neither result establishes image correctness, temporal quality
  or a general provider support range.
- The earlier [state and synchronization record](records/2026-09-24-src-m6-dx12-state-and-synchronization.md)
  inferred that a later frame might be CPU-recorded before earlier GPU work
  completed. The [recording-ahead experiment](records/2026-09-24-exp-m6-recording-ahead.md)
  directly verifies several such calls on one tested provider and fixture.
  Its separate ordered queue submissions and coarse output signatures do not
  prove general temporal correctness, provider ring capacity, or isolation
  between sibling contexts.
- The open DX12 backend's shared rings are scoped to an `FfxInterface`, while
  the modern open FSR3 provider provisions one such interface per context.
  Neither fact identifies the signed FSR4 provider's allocation topology.
  `FFX_MAX_QUEUED_FRAMES = 4` and the successful eight-call recording case
  establish no public queue-depth or fence cadence.

## Open questions

1. **Signed-provider allocation behavior:** [D011](../adr/D011.md) and M6b's
   public method use explicit bounded provider trust. The unknown allocation and
   sharing model remains an evidence limit, not a pending implementation gate.
   Neither source nor recording-ahead tests set a numeric capacity. Revisit the
   contract if new evidence identifies unsafe reuse or additional safety
   preconditions.
2. **Broader ABI coverage:** M6a passed the selected Rust/MSVC x64 dispatch
   checks; other SDK entities and x86 remain outside this proof.
3. **Further native validation:** if GBV success is required, obtain a native
   stack/trace or compare drivers/providers to distinguish its native-call
   stall from general GBV compute instrumentation. Other providers, formats
   and states remain untested. The debug-layer-only GPU evidence covers one
   resource profile and simple synthetic frames.
4. **Temporal experiment:** jitter/MV direction and scale, reset behavior and
   image correctness over ordered moving frames. Recording ahead with zero
   motion and a changing color gradient does not answer those questions. No
   arbitrary overlap or cross-queue support is established.
5. **Later scope:** output format range, alpha behavior, aliasing, nontrivial
   resource shapes, COMPUTE lists, dynamic output size and general concurrency.
