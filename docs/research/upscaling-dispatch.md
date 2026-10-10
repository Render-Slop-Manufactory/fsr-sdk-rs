# Upscaling dispatch: DX12 and frame semantics

- Updated: 2026-10-10 (M9 public native integration and exact candidate-artifact verification integrated; original failed effect gate preserved).
- Scope: source research and bounded Windows/DX12 public dispatch, recording-ahead
  temporal smoke, controlled semantic comparisons, a test-only sharpening
  experiment and public sharpening integration against AMD FSR SDK `v2.3.0`
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
  the [M6b public-route verification](records/2026-09-25-m6b-public-dispatch-verification.md),
  the [final Windows regression/temporal smoke](records/2026-09-28-exp-final-windows-release-verification.md),
  the [broader DX12 upscaler surface](records/2026-09-29-src-dx12-upscaler-api-surface.md),
  the [controlled semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md),
  the [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md),
  and the [M9 public native and candidate verification](../release/2026-10-10-m9-verification.md).

The initial M6 source records list pinned primary sources, but their original inline
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

**Verified M9 public integration:** the
[2026-10-10 verification](../release/2026-10-10-m9-verification.md) passed the
selected protocol through both development source and the unmodified extracted
`fsr-sdk 0.2.0` candidate. Each run completed three isolated repetitions and
144 finite, written, fenced outputs, including same-context sharpening changes
with continuing history, clean ordinary debug/device checks and explicit cleanup.
This closes the public-API native integration and local artifact-verification
gap on the recorded AMD configuration; publication remains pending. The original
sharpening effect gate remains failed, and no general quality or GBV claim follows.

**Maintenance recheck:** the [2026-10-09 verification](../release/2026-10-09-windows-native-recheck.md)
reran public dispatch, temporal/semantic comparisons and all six recording-ahead
configurations on its captured pre-M7 baseline. These regressions passed. The
unchanged sharpening effect assertion failed in all three repeats, with the
independent readback audit confirming execution and metrics. The GBV control
passed, while dispatch again timed out at 90 seconds. Later concurrent public
sharpening edits are outside this verification; no release criterion changed.

**Verified controlled comparisons:** the
[2026-09-30 semantic experiment](records/2026-09-30-exp-dispatch-semantics.md)
found and corrected the synthetic image generator's jitter sign. On the recorded
provider, matching horizontal UV motion and projection jitter now have lower
analytic error than deliberately reversed inputs; accumulated history affects
output, and a reset matches a fresh reset context. This adds bounded semantic
coverage to the smoke tests, with 33 finite, fenced outputs and clean debug/device
checks. It is not a general temporal image-quality result.

**Verified 0.1.0 Windows release coverage:** the
[final verification record](records/2026-09-28-exp-final-windows-release-verification.md)
reports **WINDOWS RELEASE VERIFICATION PASSED** for its captured source and
harness state. It exercises `Runtime::load_unchecked`, D012's
`UpscalerOptions::new`, public construction and public unsafe dispatch on
`x86_64-pc-windows-msvc`. The selected RX 9060 XT, driver 32.0.31041.1004,
observed provider 4.1.1 and staged SDK v2.3.0 DLL fingerprints agree with
[M6b](records/2026-09-25-m6b-public-dispatch-verification.md). Production behavior
and the accepted D005/D007/D008/D011/D012 contracts were unchanged.

| Evidence | Verified result | Scope limit |
|---|---|---|
| Windows checks and native ABI/helper build | Standard checks, paired dispatch ABI compilation and five-resource AMD helper parity passed | Compilation/fixtures and real GPU results remain separate evidence |
| Gate A: public-route regression | One reset frame submitted, fenced and read back; stronger sentinel/gradient assertions and explicit destroy passed | Repeats the bounded M6b workload using the current API |
| Gate B: temporal smoke | Three moving, jittered frames passed through one runtime/device/upscaler; each fresh output was submitted, fenced and read back | Finite changed output establishes smoke coverage, not correct history use or reconstruction |
| Teardown observations | Zero debug errors/corruption and `S_OK` device status after explicit destroy, normal Drop and resource cleanup | The public API exposes no native Drop return code; failure postconditions were not tested |

All four outputs were 640 × 360 with 230,400 pixels finite across RGBA and
230,400 changed from the fresh sentinel. Every public dispatch succeeded;
both isolated children exited 0 without timeout. The ordinary debug layer was
enabled, GBV disabled, and the actually selected adapters were checked as AMD
hardware. The record preserves commands, inputs, fingerprints and sanitized
logs; its conclusion does not freeze a release or verify a later changed tree.

**Delta from M6b:** the previous public-route proof covered a zero-motion reset
frame. Gate B adds a reset frame followed by two non-reset frames, each moving
one render pixel right with jitter-free current-to-previous UV motion
`(-1/320, 0)` and native scale `(320, 180)`. Applied jitter affects the analytic
image and matches the dispatch value. This completes the bounded temporal smoke
check on the recorded configuration. At that stage, history use and reset
effectiveness still needed targeted comparisons. The later
[controlled semantic experiment](records/2026-09-30-exp-dispatch-semantics.md)
adds bounded horizontal-motion/jitter/history/reset evidence; general temporal
reconstruction accuracy remains unverified.
The fixed extents/formats, explicit exposure 1.0, pre-exposure 1.0 and zero flags
were preserved; broader profiles and provider allocation behavior gained no
new evidence. Per-frame fencing is this test's strategy, not a wrapper rule or
general cross-queue support claim.

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
motion, one-reset-frame result did not exercise nonzero-motion temporal inputs.
The later final Windows smoke exercised this UV/jitter profile over moving
frames; its output assertions do not establish numerical motion/jitter
correctness or resolve the guide's raw-NDC discrepancy.
[Motion/jitter investigation](records/2026-09-24-src-m6-motion-vector-jitter-semantics.md) ·
[M6a result](records/2026-09-24-exp-m6a-native-dispatch-proof.md) ·
[Final Windows result](records/2026-09-28-exp-final-windows-release-verification.md).

**Exposure policy:** the current `Upscaler::new` fixes creation flags
to zero. The source-backed null-exposure profile requires
`FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` at creation. To test the existing public M5
constructor end to end, M6a supplied the documented 1×1 `R32_FLOAT` exposure
resource and observed a successful dispatch. The older contract recommendation
instead proposes auto exposure and four textures. That is a different creation
policy, untested by M6a or M6b; D011 retains explicit exposure and leaves a
later public auto-exposure option undecided.
[Current construction](../../crates/fsr-sdk/src/upscaler/dx12.rs) ·
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
one-frame GPU test through the public API. The final Windows verification adds
the completed three-frame temporal smoke; reconstruction and image-quality
verification remain open.

**Scope assessment:** the crate's job is the ABI, ownership, descriptor
conversion, error reporting and dispatch boundary. The renderer owns image
content, motion vectors, jitter generation, resource transitions, queue
submission and fences. The native image scene and readback are project test
equipment. D011 explicitly selected a fixed first public format, exposure and
motion-vector support profile using the M6a evidence; this does not make it a
universal SDK restriction. The [M6 plan](../MILESTONE_6.md) and [D011](../adr/D011.md)
retain the boundary. M6b verified that route on one GPU configuration.

## Broader frame surface and API evolution

The [2026-09-29 source investigation](records/2026-09-29-src-dx12-upscaler-api-surface.md)
agrees with the narrow D011 profile at the same SDK commit. Its inventory and
design proposals add no GPU execution or temporal-quality evidence. Selected
header declarations were checked at intake; the complete source claim set was
not independently reverified. Provider/Query/Configure findings are integrated
in [API and safety](runtime-threading.md#post-v010-upscaler-surface).

**Verified source surface:** the existing raw dispatch descriptor already has
sharpening, pre-exposure, masks, active render/output extents and dispatch flags.
At the time of the source investigation, the public Rust method fixed or omitted
those capabilities. Current M8 source exposes native sharpening; the other
controls remain fixed or omitted. Sharpening and pre-exposure are per-frame
scalars; automatic exposure, depth/MV conventions and
HDR are creation choices. Non-linear input combines a creation capability with
per-frame encoding flags. Reactive-mask generation is a separate tagged GPU
dispatch through the same export, not merely another main-frame input.

**Extent and resource qualification:** allocation dimensions, creation maxima
and active frame extents are distinct. Dynamic render resolution has a creation
flag; variable output uses `upscaleSize` without an analogous creation flag.
That shared field does not establish correct signed-provider history across
size changes. Resource requirements should inform future mask/exposure policy;
FSR3/FSR4 guidance differs. The DX12 helper's format-conversion table is not an
upscaler acceptance whitelist. D011's fixed extents, formats and exposure remain
the supported wrapper profile.

**Earlier design inference and subsequent decision:** adding even an optional public
field to `UpscaleDispatch` breaks existing exhaustive struct literals. D012's
private creation options solve a different extensibility problem. Before the
first per-frame extension, the record recommends an explicit choice between
retaining the existing route plus an extensible advanced frame input, or a
breaking release with one private-field frame representation. The maintainer
subsequently accepted [D014](../adr/D014.md) for the latter `0.2.0` route;
M8 is now implemented and checked in the
[2026-10-09 implementation record](../release/2026-10-09-m8-implementation-verification.md).
[M9](../release/2026-10-10-m9-verification.md) subsequently passed public native
integration and exact local candidate-artifact checks on the recorded configuration.
This preserves
`unsafe dispatch(&mut self, ...)`, renderer ownership, poison behavior and
GPU-completion obligations.

**Subsequent experiments and remaining work:** the
[controlled semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md)
now cover motion/jitter direction, history sensitivity and reset for one scene.
The [D014 spike](records/2026-09-30-exp-d014-native-sharpening.md) adds stationary
sharpening comparisons with a failed proposed effect gate, detailed below. These
are later runtime evidence, not results of the source investigation. Equivalent
scenes under different pre-exposure/exposure modes, mask fixtures and sentinel
checks outside changing active extents still need experiments. Broader
HDR/non-linear/format, temporal-quality and provider coverage remains open.
GBV, signed-provider allocation/sharing, state restoration, COMPUTE-list support and callback/tuning
contracts remain unresolved within their existing scope.

## D014 sharpening: native execution and failed effect gate

The [2026-09-30 spike](records/2026-09-30-exp-d014-native-sharpening.md) tests the
existing native fields after shared descriptor inspection, through the existing
owner/poisoning core. At the time, production dispatch fixed sharpening off.
Current source implements D014's public frame API and native control under M8;
the spike does not verify that later public implementation.

**Verified bounded execution:** on the RX 9060 XT, driver `32.0.31041.1004`,
signed SDK v2.3.0/provider 4.1.1 configuration, two stationary grayscale scenes
ran at 320x180 render and 640x360 output with explicit exposure 1, zero motion
and eight matched jittered frames per fresh context. Disabled A/B and enabled
0, 0.35 and 1 were compared in three isolated 180-second-watchdog processes.
A second complete run added saved half-bit readbacks for independent auditing.
Across both runs, 480 submitted/fenced outputs had finite RGBA, clean ordinary
debug/device checks and completed explicit teardown. GBV stayed disabled.
Disabled A/B red bits were identical in the measured ROI, with repeat noise
`N=0` in every repetition.

**Verified failed effect criteria:** both nonzero settings produced differences
above the predeclared magnitude and changed-area floors, but neither satisfied
all frequency-scene requirements. Every repetition gave the same results:

| Enabled setting | Horizontal gradient relative to disabled | Absolute mean-red shift | Failed criterion |
|---|---:|---:|---|
| 0.35 | 1.0058436764 | 0.0013968057 | Gradient must be at least 1.01 times baseline |
| 1 | 1.0653898496 | 0.0195217544 | Mean shift must be at most 0.005 |

All three original children and all three audit children exited 101 at the final
effect assertion, after successful execution and cleanup; none timed out. The
independent .NET half decoder and f64 metric loops reproduced the Rust values to
ten decimal places. This rules out repeat noise and a discrepancy between those
two metric implementations as explanations for the recorded failure. It does
not identify the closed provider's internal response or prove the policy suitable
for broader scenes. The original thresholds remain unchanged and the gate failed.

**Verified enabled-zero behavior:** enabled 0 differed from disabled in both
scenes (red MAE 0.0022058795 for frequency and 0.0010861201 for edge). Pass
enablement and strength therefore need distinct documentation; zero is the
parameter's lower endpoint, not a promise of disabled or identity output. The
edge profiles narrowed and showed overshoot, with a finite ROI maximum of
1.734375 at strength 1. Edge execution/profile measurements are diagnostics,
not an independent effect gate or a general image-quality result. Open FSR3/FSR2
RCAS behavior does not explain the signed FSR4 internals by itself.

**Design inference and subsequent integration policy:** the source inventory's available
fields now have bounded enabled-pass runtime evidence. Successful execution
does not establish the proposed 1% gradient/0.005 mean-preservation policy.
The [2026-10-09 integration protocol](../release/2026-10-09-m9-native-sharpening-protocol.md)
for D014 now selects exact mapping and bounded successful native/GPU execution
without additional sharpening image-effect thresholds, while this original
result stays failed. The original thresholds were not adopted as release gates.
The maintainer accepted D014's API/migration and bounded native-pass trust on
2026-10-07; M8 is implemented and the selected M9 protocol subsequently passed
through public development and extracted-candidate APIs, as detailed below.
D011's unsafe
boundary, application GPU lifetimes and poison/cleanup obligations persist;
its bounded allocator trust is extended to the native sharpening pass as a
maintainer decision, not an experimental guarantee. The experiment establishes
no new queue depth, cross-queue behavior, thread transfer or general provider
support.

## M9 public sharpening integration and candidate verification

The [2026-10-10 record](../release/2026-10-10-m9-verification.md) executes the
unchanged [selected protocol](../release/2026-10-09-m9-native-sharpening-protocol.md).
It uses public `UpscaleResources`, `UpscaleDispatch`, the sharpening setter and
unsafe `Upscaler::dispatch`; it does not use the historical spike's private
descriptor override. Both runs identify signed SDK v2.3.0/provider 4.1.1,
RX 9060 XT and driver `32.0.31041.1004`, with DLL fingerprints in the record.

**Verified bounded execution:** development source and the unmodified extracted
wrapper `0.2.0` artifact each completed three isolated repetitions of 48 outputs.
For each of the two stationary scenes, switching A/B and always-Disabled contexts
process eight matched jittered frames. Only frame 0 resets history; frames 3–7
retain one context through `Disabled → Native(0) → Native(0.35) → Native(1) → Disabled`.
All 288 outputs had completed fences, 230,400 finite RGBA pixels and full sentinel
replacement. Each run had zero ordinary debug errors/device removal and 18
successful explicit context destructions with post-teardown checks. Existing
disabled lifecycle, single-frame, three-frame and semantic regressions also passed.

**Verified diagnostic comparisons:** independent half-float audits
confirmed 78 ROI comparisons per run, with matching diagnostics across both runs
and their repetitions. Switching A/B were bit-equal; enabled zero differed from
Disabled on both scenes. Returning to Disabled at frame 7 matched the control's
ROI bits on this setup. This does not promise equality for arbitrary temporal
histories. Exact native parameter forwarding, including negative zero, is
established separately by scalar/descriptor and controlled-FFI fixtures; GPU
success or output differences alone cannot establish mapping.

**Artifact boundary:** the local candidate pairs `fsr-sdk 0.2.0` with
`fsr-sdk-sys 0.1.1` and excludes D013's public `movable` family. Stable/MSRV,
Rustdoc, migration/consumer controls, unchanged archives/extracted files and the
publication dry-run passed; see [distribution findings](licensing-and-distribution.md).
It remains unpublished. The selected integration gate has no image-effect
thresholds and does not reinterpret the original failed experiment. GBV, moving
scene sharpening quality, broader providers, allocation capacity, recording ahead
with sharpening and cross-queue behavior gain no new evidence.

[D016](../adr/D016.md) is the accepted cross-feature constraint for direct
provider controls and explicit SDK quality policies. It adds no experimental
result, future sharpening variant or quality guarantee and does not change
D014's API, native mapping or separately selected release protocol.

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
  response to the literal pair remain unknown. The final Windows analytic
  moving scene exercised the accepted UV profile at smoke level. The subsequent
  [controlled comparisons](records/2026-09-30-exp-dispatch-semantics.md) distinguish
  matching from reversed horizontal motion and jitter on one scene/configuration.
  They do not resolve the guide's discrepancy or establish all-axis motion/scale
  accuracy and general reconstruction quality.
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
- The final Windows smoke complements the earlier proofs without resolving
  their failure or quality limits. Successful calls add no native evidence for
  dispatch poisoning, abandoned recordings or D005's exceptional retention on
  destroy failure; those wrapper policies still have fixture/source evidence.
  GBV stayed disabled, so its historical native-call stall remains unresolved.

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
4. **Temporal correctness:** the final Windows three-frame moving smoke passed
   with applied jitter and current-to-previous UV motion. Its coarse finite/
   changed-output assertions do not establish correct reconstruction, history
   use, reset effectiveness or image quality. Recording ahead with zero motion
   also does not answer those questions. The [controlled semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md)
   now cover horizontal motion direction, jitter direction, history sensitivity
   and reset for one scene/configuration; broader reconstruction remains open.
   No arbitrary overlap or general
   cross-queue support is established.
5. **Later scope:** output format range, alpha behavior, aliasing, nontrivial
   resource shapes, COMPUTE lists, dynamic output size and general concurrency.
6. **Later per-frame extensions:** D014's breaking private-frame representation
   and native sharpening are implemented and checked under
   [M8](../release/2026-10-09-m8-implementation-verification.md), with public GPU
   integration and local candidate artifacts verified under
   [M9](../release/2026-10-10-m9-verification.md). Shared flag support, changing active extents,
   exposure/mask requirements and image semantics need provider-specific evidence;
   the [surface record](records/2026-09-29-src-dx12-upscaler-api-surface.md)
   supplies proposals, not implementation authorization.
7. **Sharpening acceptance policy:** the
   [D014 spike](records/2026-09-30-exp-d014-native-sharpening.md) supplies clean
   enabled-pass execution and measurable changes on one configuration, including
   enabled zero, but fails the proposed effect criteria. The selected
   [M9 protocol](../release/2026-10-09-m9-native-sharpening-protocol.md) requires
   integration evidence without new sharpening image-effect thresholds and passed
   in [M9 verification](../release/2026-10-10-m9-verification.md). D014 was separately
   accepted by the maintainer on 2026-10-07. M8 fixtures establish exact wrapper
   mapping; M9 establishes bounded public-API recording, GPU execution and cleanup.
   Brightness preservation, moving-scene quality and broader provider coverage
   remain unresolved. The original failed effect gate supplies no quality promise.
