# Upscaling dispatch: DX12 and frame semantics

- Updated: 2026-10-29 (current archive-native and provider-specific generation checks integrated; prior source/spike identities retained).
- Scope: source research and bounded Windows/DX12 public dispatch, recording-ahead
  temporal smoke, controlled semantic comparisons, a test-only sharpening
  experiment, public sharpening integration, V01 scene/reference design and
  two-scene temporal calibration and subsequent property-specific confirmations,
  plus the HDR/encoding source concept, linear HDR verification, bounded
  sRGB/PQ signal and exposure controls, standalone reactive-mask generation,
  raw variable-output geometry/allocation and public active-output verification,
  external-format/view and explicit motion-scale source research, and
  bounded Linux-hosted Proton execution of Windows/DX12 binaries
  against AMD FSR SDK `v2.3.0`
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
  the [M9 public native and candidate verification](../release/2026-10-10-m9-verification.md),
  the [V01 scene/reference source contract](records/2026-10-10-src-v01-scene-reference-contract.md),
  the [temporal calibration experiment](records/2026-10-11-exp-v01-verification-calibration.md),
  [metric diagnosis](records/2026-10-11-exp-temporal-quality-metric-diagnosis.md),
  [quality evaluator and confirmation](records/2026-10-11-exp-temporal-quality-validation.md),
  [continuous-history confirmation](records/2026-10-11-exp-temporal-long-sequence.md),
  [cut/reset confirmation](records/2026-10-11-exp-temporal-reset-recovery.md),
  [jitter sign/phase confirmation](records/2026-10-12-exp-temporal-jitter-phase.md),
  [Steam Deck Proton proof](records/2026-10-13-exp-proton-linux-cross-build.md),
  [Vulkan-validation follow-up](records/2026-10-13-exp-proton-vulkan-validation.md),
  [Steam Machine INT8 comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md),
  [Deck temporal regressions](records/2026-10-13-exp-proton-temporal-regression.md),
  [Machine temporal regressions](records/2026-10-14-exp-steam-machine-proton-temporal-regression.md),
  [exit-247 diagnosis](records/2026-10-14-exp-proton-exit-status.md),
  the [HDR/encoding source concept](records/2026-10-15-src-hdr-colour-encoding-concept.md),
  [selected exposure/active-render Windows verification](../release/2026-10-15-windows-m11-m14-verification.md),
  [supplied-mask implementation checks](../release/2026-10-15-supplied-masks-verification.md),
  [linear HDR implementation checks](../release/2026-10-15-linear-hdr-verification.md),
  [non-linear contract investigation](records/2026-10-15-src-non-linear-colour-contract.md),
  [colour implementation checks](../release/2026-10-15-colour-encoding-verification.md),
  [Windows feature batch](../release/2026-10-15-windows-feature-batch.md),
  [native sRGB/PQ signal experiment](records/2026-10-15-exp-non-linear-colour-signals.md),
  [variable-output source investigation](records/2026-10-19-src-variable-output-resolution.md),
  [variable-output native spike](records/2026-10-19-exp-variable-output-resolution.md),
  [public active-output implementation verification](../release/2026-10-19-active-output-verification.md),
  [external formats and motion-scale investigation](records/2026-10-19-src-upscaling-formats-and-motion-scale.md),
  [Windows selection/Configure verification](../release/2026-10-19-windows-upscaler-verification.md)
  and [reactive-mask provider experiment](records/2026-10-19-exp-reactive-mask-providers.md).

The initial M6 source records list pinned primary sources, but their original inline
citation tokens had no index-to-source map. Those tokens were removed at intake.
The later contract recommendation had the same citation limitation and was
integrated as an analysis of these records and the repository, not as a native
test. Selected ABI declarations were checked against the locally staged v2.3.0
headers; the full source claim set has not been independently reverified. M6a
tested a selected subset, including ABI, conversion and one submitted GPU
frame; M6b repeated the bounded GPU test through the public method. The signed
FSR4 provider's internal implementation is not available in
the public source. Same-tag open FSR3 behaviour is useful corroboration, not
proof of FSR4 behaviour. The two later source investigations had no repository
tree or native runtime in their research environment. Their repository baseline
came from a brief; this intake compared it with the records and proposed D011
here. Their source findings are not new GPU results.

## Current findings

**Current archive-native checkpoint, 2026-10-29:** the
[consolidated verification](../release/2026-10-29-candidate-verification.md)
records passing motion (106), typed-format (170), depth-format (86), BGRA/sRGB
(106) and remaining-resource-format (284) profiles through the selected Windows
archives. Together they retain 2,646 validated frame/readback records, including
C++ references and no-dispatch controls. Current generation passes both owners/
all flags on 3.1.5; 4.1.2 */2.3.4 acknowledge but leave masks seeded, preserving
their failed profiles. Earlier 4.1.1 failures remain historical observations.
Fresh Deck/Machine proofs refresh bounded translated-DX12 execution on a shared
Linux pair, distinct from Windows/P08. Final shipping-byte reconciliation, native
access/provider conditions and all prior semantic/safety limits remain separate.
These checks add neither image-quality evidence nor new performance measurements.

**Portable composition candidate, 2026-10-26:** accepted [D042](../adr/D042.md)
selects portable frame parameters, named resource assembly and a separate command
list as the ordinary recording API. The [contract with an early-U08 probe](../design/upscaler-setup.md) and
[spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md) exercise that
shape through production inspection/mapping in an isolated candidate. Stable
and MSRV controlled checks preserve negative-zero/subnormal literal scale bits,
replace native scaling with pixel/UV selection, clear the initial history reset
and reject metadata changed after successful preflight. Missing colour/exposure
is a pure binding error; automatic exposure must still match the receiving context.
Portable sharpening compiles without DX12. U08 now implements production
`FrameParameters` / `ReactiveMaskParameters`, complete named DX12 bindings and
separate list/resources/parameters recording, including generation. One motion
and colour request replace dormant setting pairs. The
[implementation verification](../release/2026-10-27-frame-composition-verification.md)
records compiler, controlled and native checks separately from this earlier probe.
The [API synthesis](api-and-safety.md#accepted-api-transition-and-setup-draft)
distinguishes those checks from contract acceptance and native coverage. No AMD
provider, shader access, GPU result, ABI or performance evidence is added, and
renderer submission/completion/abandonment duties remain unchanged.

**D040 diagnostics and callback-backed dispatch, 2026-10-25:** accepted
[D040](../adr/D040.md) adds independent checking/visualization creation flags
and a per-frame `draw_debug_view` request, default false. Estimate/create flags
share the same mapper; frame bit 0 merges with the existing colour hints without
changing reset or creation options. A frame is not rejected solely because
creation visualization was false; unsafe dispatch must establish compatibility
of the actual provider and requested combination. An OK return does not certify
the intended overlay or the effect of every flag.

The [public verification](../release/2026-10-25-callbacks-diagnostics-verification.md)
passes 20 fenced frames across both owners and selected 3.1.5/4.1.1: eight with
debug view, twelve without, with complete finite/written output and zero DX12
errors. The 0.5 ms warning stimulus delivers 16 messages; it is not an image
comparison. The caller retains custom CB upload backing/ranges and residency
through completion; CPU return and COM retention do not authorize reuse.

The [callback synthesis](callbacks-and-allocation.md) owns shared registration,
external-target lifetime and diagnostic routing. Global diagnostic changes need
all affected native users idle/coordinated, with GPU work complete and no
executable pending recordings. These are distinct from context-local tuning.
The later [archive checkpoint](../release/2026-10-29-package-coverage-verification.md)
repeats this public callback/debug-view fixture through extracted packages and
the packaged raw CB declaration. Final publication-pair reconciliation remains
separate from that completed current-candidate binding.


**D035 implementation, 2026-10-22:** the accepted
[typed colour/output/depth slice](../adr/D035.md) adds RGBA32F, R11G11B10F,
RGB10A2 UNORM and RGBA8 UNORM in selected colour/output/generation roles, and
R16F device depth. Additional formats share role/device checks and role-bearing
query errors; signals and provider compatibility remain caller-established.
The [implementation record](../release/2026-10-22-typed-format-verification.md)
records passing controlled, MSRV and bounded public/C++ checks on one configuration.
D038/D039 subsequently complete the other applicable format families.

**D037 depth-buffer implementation, 2026-10-22:** [D037](../adr/D037.md) adds
`R32_TYPELESS`, `D32_FLOAT`, `R32G8X24_TYPELESS` and `D32_FLOAT_S8X24_UINT` in
the depth role. The exact fixed depth SRV, conditional fully typed casting check,
DSV capability and whole-resource transition contract remain distinct from
the collapsed native surface enum. D16/D24 and view-only/stencil-only allocations
remain outside this slice. [Verification](../release/2026-10-22-depth-format-verification.md)
passes stable/MSRV and both-family native checks on selected 4.1.1/3.1.5: exact
shader depth samples, preserved stencil and matched C++/R32F outputs. This
bounded slice alone does not complete U05 or certify every provider/GPU;
D039 supplies the later complete accounting.

**D034 implementation, 2026-10-21:** the accepted
[native-scale/typed-motion slice](../adr/D034.md) is implemented for both owners.
[Verification](../release/2026-10-21-native-motion-verification.md) passes 129
stable, 112 MSRV and 102 public native cases: 252 dispatch/12 control frames on
4.1.1/3.1.5. This includes RG32F render/display motion, exact positive/mixed-sign
overrides, changing jitter with Included, UAV-capable inputs, override clearing
and active output. Equivalence/sensitive controls establish bounded mapping
consistency, not native jitter-cancellation consumption or a safe numerical
domain. D039 subsequently completes the applicable format families.

**U05 source intake, 2026-10-20:** the
[new record](records/2026-10-19-src-upscaling-formats-and-motion-scale.md), investigated
on 2026-10-19 at the same SDK pin, distinguishes external role semantics, generic
format conversion, backend-generated views and actual-provider support. It
reports independent signed motion-scale arithmetic in open FSR3/FSR2, different
display-motion denominators, and an unresolved NDC example conflict. The
[assessment below](#external-formats-and-explicit-motion-scales) retains those
limits and separates source-supported candidates from accepted formats. U05
contract selection, implementation and native verification were open at that
intake; D034's later accepted slice is distinguished below.

The [2026-10-18 coverage synthesis](upscaling-coverage.md) integrates the
[release-surface inventory](records/2026-10-18-src-upscaling-release-surface-inventory.md).
Its intake baseline distinguishes supplied masks from generated-mask recording,
fixed-output active render regions from variable active output,
and UV/pixel motion units versus explicit signed/asymmetric native scales.
Other gaps include reviewed alternative formats/usage and the native debug flags.
D030 generation is implemented and has [provider-specific GPU evidence](#standalone-reactive-masks-provider-specific-execution);
variable output is subsequently implemented under the accepted D024 amendment
([verification](../release/2026-10-19-active-output-verification.md)). Explicit
native motion scales remain missing. Applicable per-role/provider
support, resource-state/aliasing obligations and both-family verification must
be explicit when widening the profile; generic DX12 types alone establish none
of them. That source intake adds no dispatch execution or automatic conversion policy.

**Variable output source intake, 2026-10-19:** the
[new record](records/2026-10-19-src-variable-output-resolution.md) separates
context capacity, real texture allocation and active output dimensions. It
reports open FSR3 forwarding/defaults/bounds and active-output motion geometry,
while the FSR2 adapter omits the requested output size. Maximum-allocation
equality is an existing wrapper policy, not an established general AMD rule.
The [assessment below](#variable-output-resolution-source-contract) preserves
provider/allocation uncertainty, rounded-write concerns and the decisions that
were needed before widening D024. The subsequent
[native spike](#variable-output-native-follow-up-2026-10-19) passes 71 integration
cases: 4.1.1/3.1.5 vary written footprints, 2.3.4 retains maximum output and
padding writes occur. This adds raw GPU evidence, not a public implementation
or proof that active-sized allocations bound native access.
The later [accepted output slice](#accepted-maximum-allocation-output-slice)
adds the public API under explicit native trust; it accepts no allocation relaxation.

**Configure/dispatch ordering is now an accepted wrapper contract:**
[D029](../adr/D029.md) adds safe live-context tuning on both families. A prior
unsafe dispatch carries the lasting obligation to complete submitted work and
leave no executable unsubmitted recording before Configure: record → submit →
wait → configure → record again. Abandoned successful recordings and failed
native dispatches retain their terminal-use rules; Configure is not a recovery
path and does not automatically reset history. A Configure error additionally
blocks dispatch and all live queries before inspection/FFI.
The [implementation checks](../release/2026-10-18-upscaler-tuning-verification.md)
cover this shared eligibility policy with controlled host tests and Windows
compilation, including a renderer-supplied submit/fence example. They add no
actual-provider/GPU ordering, persistence or parameter-application verdict.
The subsequent [Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md)
passes all five value/default-request cycles and 20 submitted/fenced frames
across both families on 4.1.1. This adds bounded ordering/cleanup evidence;
parameter application, persistence and effective-default restoration remain unproven.
The later [coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
uses `request_tuning` with mandatory parameter-domain checks and
`validate_dispatch_inputs` for optional preflight at that checkpoint. D042 now
names it `validate_upscale_inputs` and replaces the combined frame/`dispatch` API
with portable parameters, borrowed resources and unsafe `record`. Recording
independently repeats inspection; preflight establishes no resource states,
backing-storage isolation or GPU completion. Development-package
checks and Windows compilation add no new GPU/state-restoration evidence.

**HDR and bounded non-linear native integration verified:** [D026](../adr/D026.md)
linear HDR passes Windows fixtures, C++ ABI, MSRV and six native frames in the
[Windows batch](../release/2026-10-15-windows-feature-batch.md).
[D027](../adr/D027.md)'s colour controls now have
[270 native signal/control frames](records/2026-10-15-exp-non-linear-colour-signals.md)
on one GPU/driver/provider. Native integration passes, while 15 initial
same-code predictions fail during changing pre-exposure; static/reset controls
pass the unchanged budgets. The [HDR assessment below](#linear-hdr-and-non-linear-colour)
distinguishes these observations from unresolved internal ordering, PQ scale and
general dynamic-pre-exposure semantics. M16's bounded SDK-access checkpoint is
complete under the [accepted completion boundary](../adr/D027.md#sdk-access-completion-boundary-2026-10-18);
those unresolved signal semantics remain explicit limitations.

**Scope update, 2026-10-18:** image-quality research and custom quality policies
are outside scope through 1.0.0 by maintainer direction
([scope](../../SCOPE.md#verification-priority)). SDK access and wrapper integration
take priority. V02/V07d and further quality studies are in the
[deferred backlog](../POST_1_0.md); do not carry them into active status summaries
or release prerequisites. ABI, mapping, safety, native execution and cleanup
checks continue. Existing experiment outcomes are preserved.

**Verified Proton single-frame coverage:** the same bounded numerical workload
passes on Steam Deck/provider 3.1.5 and Steam Machine/provider 4.1.1, with
separately observed Vulkan core/synchronization validation. The
[assessment below](#proton-single-frame-and-int8-evidence) distinguishes the
Machine's compiled INT8 evidence from instruction tracing and preserves the
missing Windows diagnostics and temporal-quality limits.

**Selected Proton temporal coverage, acceptance failed:** reset recovery passes
on Deck and Machine under unchanged criteria. Quality/long/jitter acceptance is
failed or blocked, with different failed predictions on the two configurations.
The [comparison below](#selected-proton-temporal-regressions) separates completed
GPU execution from image verdicts and the unresolved Deck launcher failure.

**Verified selected V01 baseline:** later, separately frozen confirmations establish
bounded motion/disocclusion, 256-frame continuous history, one camera-cut reset
and isolated jitter sign/phase sensitivity on the recorded Windows/DX12
configuration. The [integrated assessment](#confirmed-temporal-properties-and-metric-limits)
preserves contrary metric rankings and distinguishes exploration from confirmation.
The [roadmap](../../ROADMAP.md) already marks this selected checkpoint verified;
broader scenes, providers and perceptual quality remain unverified. No SDK quality
policy, public contract or ordinary/release gate follows from these experiments.

**Verified bounded temporal calibration:** the
[2026-10-11 experiment](records/2026-10-11-exp-v01-verification-calibration.md)
completed three independent 301-frame children through the public Upscaler,
with full RGBA16F readbacks, independently byte-checked inputs and clean ordinary
debug/device/fence observations on the recorded RX 9060 XT deployment. Motion
and jitter controls expose measurable differences, but correct inputs do not
universally minimize spatial, temporal or leakage error. Reset/fresh and repeat
outputs were locally bit-identical. The [calibration assessment below](#temporal-calibration-and-candidate-verification-criteria)
distinguishes completed execution, mixed diagnostic results and the then-unverified
candidate criteria. The subsequent confirmations below resolve selected questions
without accepting those original candidates retrospectively.

**V01 source intake:** the [2026-10-10 record](records/2026-10-10-src-v01-scene-reference-contract.md)
adds a proposed opaque, two-depth disocclusion scene, independent display
references and regional temporal diagnostics within the existing public profile.
It corroborates the UV-motion/jitter conventions and preserves the upstream
raw-NDC example discrepancy. It supplies no new execution evidence, numerical
quality gate or accepted implementation contract. The [intake assessment below](#v01-scene-and-reference-contract)
separates its source facts and proposals from the subsequent smaller experiment.

**Verified M9 public integration:** the
[2026-10-10 verification](../release/2026-10-10-m9-verification.md) passed the
selected protocol through both development source and the unmodified extracted
`fsr-sdk 0.2.0` candidate. Each run completed three isolated repetitions and
144 finite, written, fenced outputs, including same-context sharpening changes
with continuing history, clean ordinary debug/device checks and explicit cleanup.
This closes the public-API native integration and local artefact-verification
gap on the recorded AMD configuration; publication remains pending. The original
sharpening effect gate remains failed, and no general quality or GBV claim follows.

**Maintenance recheck:** the [2026-10-09 verification](../ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh)
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
[M6b](records/2026-09-25-m6b-public-dispatch-verification.md). Production behaviour
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
were preserved; broader profiles and provider allocation behaviour gained no
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
graphics command list and separate colour input/output textures. Dispatch
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
allocator ownership, sharing, capacity or reuse behaviour. The recording-ahead
GPU result fits either separate sibling allocations or shared storage with
sufficient capacity. No ordinary per-dispatch completion rule was found in the
inspected public material; the explicit FSR3 GPU-idle guidance concerns context
destruction. These source facts narrow D011's allocation question without
identifying signed-provider allocation behaviour or setting a numeric limit.
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

**Upstream frame contract and narrow test profile:** colour, depth and motion
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
single-sample 2D textures with one mip and distinct colour/output, and omit
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

**Original exposure policy:** the D008 constructor fixed creation flags
to zero. The source-backed null-exposure profile requires
`FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` at creation. To test the existing public M5
constructor end to end, M6a supplied the documented 1×1 `R32_FLOAT` exposure
resource and observed a successful dispatch. The older contract recommendation
instead proposes auto exposure and four textures. That is a different creation
policy, untested by M6a or M6b. D011 retained explicit exposure; subsequent
[D023](../adr/D023.md) accepts explicit/automatic exposure, and
[D026](../adr/D026.md) independently adds linear HDR. Those later contracts do
not extend M6a/M6b's observed coverage.
[Construction implementation](../../crates/fsr-sdk/src/upscaler/dx12.rs) ·
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
colour/output, R32F depth and RG16F motion vectors, all single-sample
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
universal SDK restriction. The [M6 plan](../ROADMAP_HISTORY.md#010--published-2026-09-28) and [D011](../adr/D011.md)
retain the boundary. M6b verified that route on one GPU configuration.

## Standalone reactive masks: provider-specific execution

The [2026-10-19 generator experiment](records/2026-10-19-exp-reactive-mask-providers.md)
checks accepted D030 through an independent raw descriptor and both public
families on SDK v2.3.0, Windows x64/MSVC, RX 9060 XT and driver 32.0.31041.1004.
Windows public fixtures and independent production/probe MSVC ABI checks pass
in the [execution report](../release/2026-10-19-windows-upscaler-verification.md).
The experiment uses disposable contexts and fresh processes, fixed maximum
allocations, all 16 flag combinations, identical-colour/no-generator controls,
active regions 320×180 → 256×128 → 320×180, explicit UAV ordering/transitions,
mask snapshots and public upscale consumers before fenced readback/cleanup.

| Observed provider | Route and result |
|---|---|
| 3.1.5 | Explicit selection, both public families: all controls/flags pass, with 102 generated/submitted/fenced frames, zero identical-colour reactivity and unchanged inactive regions. |
| 4.1.1 | Independent raw/default and both public/default families: return OK but leave every mask pixel at seed 227. All 51 generation children fail the write criterion. |
| 2.3.4 | Explicit selection, both public families: return OK but leave every mask pixel seeded. All 34 generation children fail the write criterion. |

All 126 children complete three frames, with clean debug diagnostics and terminal
cleanup. Successful recording/submission, valid upscale output and cleanup are
distinct from mask writes; failed complete profiles remain failed. The successful
3.1.5 arm demonstrates that this equipment can observe generated writes, while
the public/default route reproduces the independent raw 4.1.1 result. This does
not identify the internal cause of the other providers' behaviour or prove which
mask texels an upscaler consumes.

**Integration consequence:** enumeration, resource requirement bits and an OK
return are not generator capability certificates. There is no new allow-list,
fallback, implicit barrier or automatic probe. Keep D030's actual-provider trust,
shared terminal native-error policy and caller-owned successful-recording duties.
The new movable generator controls stay on one thread; they establish no handoff
or concurrent-use guarantee. Final extracted-package verification and wider
provider/state profiles remain separate; these native integration results do not
resume image-quality research or verify variable output.

## Variable output resolution: source contract

The [2026-10-19 source record](records/2026-10-19-src-variable-output-resolution.md)
examines SDK v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`,
upscaler API 4.1.1, for M14/P1-B. It supplies source evidence and recommendations,
not an accepted allocation policy or executed Windows/GPU result.

### Native access and provider limits

**Verified source facts reported by the record:** the public ABI already separates
creation maxima from dispatch `renderSize`/`upscaleSize`. Open FSR3 forwards the
requested output, defaults `(0, 0)` to its maximum and rejects either requested
axis above that maximum. A single zero component is not defaulted safely by that
logic; checked nonzero `Dimensions` remain appropriate. Absence of CPU rejection
does not certify arbitrary aspect ratios, render/output ratios, small dimensions
or workgroup alignment.

The FSR2 adapter does not forward `upscaleSize`; its context uses the creation
display size. Consequently, a common ABI field or successful return does not
certify that a selected provider honours a smaller active output. This matters
for resource coverage as well as the resulting image: a texture sized for the
request cannot be presumed sufficient when a provider retains maximum-output
behaviour. The native follow-up below bounds observations of the selected signed
providers; it does not resolve their general access semantics. No provider-name
allow-list, automatic fallback or live-context capability probe is selected.

**Implementation baseline at source intake:** D024 required maximum allocations and fixed
output. `descriptor.rs` derived native output dimensions from the inspected
output allocation; display-motion scaling used maximum output. The raw descriptor
already contains the necessary field, so no new ABI declaration is indicated.

### Allocation and active image are distinct

Let `U` be active output, `A` actual output allocation and `M` context maximum;
all comparisons are per axis. The record's assessment is:

| Relationship | Source conclusion and limit |
|---|---|
| `A = U = M` | Existing wrapper baseline; no new variable-output evidence. |
| `A = U < M` | Candidate in open FSR3; shader/provider compatibility unverified. |
| `A = M`, `U < M` | Candidate; pixels outside the active image need not be preserved. |
| `U <= A <= M` | Intermediate covering allocations are candidates, not certified support. |
| `U <= M < A` | No examined CPU allocation-maximum rejection; full compatibility unresolved. |
| `A < U <= M` | Insufficient for the requested image; reject before recording. |
| `U > M` | Rejected by the examined open FSR3 dispatch validator. |

A coverage condition is necessary but not a complete safety/capability contract.
Resource descriptions must retain actual dimensions; reporting `U` as the size
of a larger resource would misdescribe its allocation. The public descriptor
exposes no destination offset. The record describes origin-anchored processing;
it does not establish an arbitrary destination rectangle.

**Reported shader boundary concern:** output workgroups round up to 8 pixels,
and RCAS to 16. The record finds no active-output guard in the examined RCAS
stores. Padding inside a larger allocation may therefore be addressed, and
out-of-allocation execution semantics require further evidence. Do not promise
untouched padding or infer universal safety solely from `A >= U`. Source intake did not
independently inspect all shader entry points/permutations or execute this case.
This is a wrapper integration/safety question, not a deferred image-quality gate.

### Motion, helpers, history and ownership

Open FSR3 normalizes display-resolution motion against active output; render
motion uses active render size. Under the existing UV convention, forward the
corresponding active dimensions as scale; pixel motion retains `(1, 1)`.
Maximum-sized or intermediate display-motion allocations containing smaller
active images remain provider-sensitive candidates. Current-frame quality-size
and jitter-phase queries should use intended active dimensions; explicit
maximum-based planning queries remain meaningful. Query inputs do not resize a
context or inherit creation options/provider selection implicitly.

The examined open validator does not gate output variation on the dynamic flag.
That absence is not an AMD guarantee that every provider accepts it without the
flag. Extending existing render-size gating to output would be a wrapper policy
requiring a decision. Open FSR3 keeps previous output dimensions and does not
implicitly reset solely because output changes; neither fact proves every
transition valid. Leave any proposed reset rule separate from direct native
access. Changing context capacity requires recreation; changing external
resources still requires correct states, ordering and retention through GPU
completion. Existing native-error terminality and idle-Configure obligations
continue on both families; replacing textures or moving an owner is not a fence.
Reactive-mask generation remains governed by active render geometry.

### Variable-output native follow-up, 2026-10-19

The [completed spike](records/2026-10-19-exp-variable-output-resolution.md) tests
the same SDK v2.3.0 pin and signed DLL pair on Windows x64/MSVC + DX12, RX 9060 XT,
driver `32.0.31041.1004`, with ordinary debug validation. All 71 cases pass:
ten preparation/build checks, 60 native children/180 dispatched frames and one
three-frame no-dispatch control. All 61 contexts destroy successfully; no native
rejection occurs. Pass means completed integration and audited observations,
not uniform implementation of requested output.

The test-only raw route bypasses D024 geometry/allocation validation while using
real public construction and selected-provider queries. It independently checks
all five native resource descriptions, packed inputs and output metrics. Context
maxima are render 320×180/output 640×360; render inputs stay fixed while output
and display-motion allocations are maximum-sized, active-sized or 672×384.
The workload is one translating analytic plane, three frames per fresh context.

**Verified bounded write observations:** with maximum-sized output allocations,
the rounded shrink/grow sequence produces:

| Requested output | 4.1.1 / 3.1.5 written rectangle | 2.3.4 written rectangle |
|---|---|---|
| 637×357 | 640×360 | 640×360 |
| 511×287 | 512×288 | 640×360 |
| 637×357 | 640×360 | 640×360 |

These footprints occur with sharpening off/on at that allocation. At 511×287
that means 799 pixels written outside active output on 4.1.1/3.1.5 and 83,743
on 2.3.4. Aligned smaller
output is honoured by 4.1.1/3.1.5; width-only and height-only changes also round
beyond the requested image. No general alignment formula follows. For 2.3.4,
the maximum-sized crop under a stable small request is byte-identical to the
stable-maximum control in all three frames with matching inputs. This corroborates
the reported FSR2 omission of `upscaleSize`; it selects no provider-name allow-list.

**Correction at synthesis, 2026-10-19:** the original record overgeneralizes that
table to oversized allocations with sharpening. Its retained
[measurements](records/2026-10-19-exp-variable-output-resolution/measurements.json)
show a different result in `rounded-large-sharp`: all three providers write
through 640×368 for requested 637×357 (8,111 outside pixels). On the smaller
511×287 phase, 4.1.1/3.1.5 write 512×288; 2.3.4 writes 640×368 (88,863 outside
pixels). Oversized allocations without sharpening match the first table. Thus
the original record's statements that the same rectangles hold with sharpening
off/on in larger allocations are superseded here. Its bytes and inventory remain
unchanged. This correction strengthens the padding/access limitation; all 71
integration verdicts stand because padding preservation was not a pass criterion.

**Allocation safety remains unresolved:** active-sized resources also complete,
but readback cannot reveal attempted stores beyond the texture. Ordinary debug
validation is not GBV or an out-of-bounds detector. `A >= U` alone therefore
cannot certify native access, including display-motion reads when output size
is ignored. Observed seeded margins in larger allocations are not a padding
preservation guarantee. The source table above remains a source assessment,
not a table of universally safe allocation relationships.

**Flags and motion:** dynamic-flag true/false arms complete on all three providers
with matched inputs. 3.1.5/2.3.4 outputs match in all frames; 4.1.1 matches only
the first. The flag is neither proven irrelevant nor established as a universal
native prerequisite. Matched display UV/pixel arms produce identical output on
each provider; this supports bounded encoding/descriptor consistency, not general
sampling coverage or temporal quality.

Fresh resources replace prior ones only after CPU fencing; reset is true on the
first frame alone. These successful local-owner sequences establish neither
general no-reset correctness, overlapping/cross-queue execution, both-family
public variable-output behaviour nor native error-path safety. D024, renderer
ownership and the earlier GBV-stall limitation remain unchanged.

### Recommendations and decisions still open

The source record recommended optional active output, corrected display-motion
geometry and allocation coverage checks. Explicit maintainer acceptance of the
[D024 amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
subsequently selected this first public profile:

| Concern | Current selection and remaining limit |
|---|---|
| Output geometry | Optional active output defaults to the context maximum; each nonzero axis is bounded independently. Both public owners share preflight/dispatch inspection. |
| Allocation | Output and display-motion textures still equal context maxima. Smaller/intermediate/oversized allocations and render-side relaxation remain separate extensions; coverage alone certifies no native access boundary. |
| Provider | Actual-provider support and safe access are explicit unsafe-dispatch obligations. The recorded 2.3.4 smaller-output path does not meet that contract. There is no name-based rejection, fallback or capability inference from enumeration, masks or OK. |
| Dynamic flag | Smaller output requires the existing dynamic option as wrapper policy. The raw flag comparisons neither establish a universal AMD prerequisite nor make the flag irrelevant. |
| Native access | The whole supplied output allocation may be written. No untouched-padding or guessed alignment rule is promised; out-of-resource access safety remains empirical uncertainty under explicit native trust. |
| Renderer ownership | Reset is forwarded unchanged. Resource states, valid input sampling footprints, replacement lifetimes, submission/fences and terminal recording duties remain renderer-owned. |

The record calls `UpscaleDispatch` a portable frame description. At that
checkpoint it contained Windows/DX12 borrows. D042 now separates portable
`FrameParameters` from Windows/DX12 `UpscaleResources`; `Dimensions` and checked
geometry remain portable. Both ownership families use the same boundary.

The raw spike completes the bounded allocation/active, rounded/sharpening, motion,
replacement/fence and successful-cleanup observations proposed at source intake.
The subsequent public implementation also passes controlled checks for defaults,
independent-axis bounds, actual allocation metadata, motion mapping, poison
precedence and reset forwarding, followed by the bounded public GPU checks below.
These close the selected integration checkpoint, not the broader access-safety
question. Neither raw nor public passes establish a general rejection guarantee
or a generic safe allocation rule. Padding preservation is not a success criterion.

### Intake provenance and citation limitations

The incoming record was named
`fsr-sdk-2.3.0-variable-output-resolution-research.md`; intake renamed it to
`2026-10-19-src-variable-output-resolution.md`, preserving all 36,799 UTF-8/LF
bytes, SHA-256 `da2fa237ebb624e9eeb4c5e0affb1dc3ba85783e4e2cb8f190485809455112c7`.
Its investigation date is supplied, not inferred. No personal paths or obvious
credential material were found. The record states that an uploaded Rust project
was inspected but supplies no archive identity/hash; whole-tree provenance is
therefore incomplete.

Representative retained-cache checks corroborate public fields, FSR3 output
forwarding/defaults/motion denominator, independent maxima and the FSR2 omission.
They expose inaccurate citation ranges: the public creation/dispatch fields are
at physical header lines 76–110, not 127–185; FSR3 forwarding is at 375–378,
not 321–345; output defaulting is at 980–992 and maxima validation at 1351–1375,
not 870–885 and 1201–1230. Use the named symbols and commit-pinned files; the
record's line ranges are not independently validated. Its original text is
preserved, and the full upstream/shader investigation was not repeated. These
source-intake checks added no compilation or native/GPU evidence. The later native
record is integrated separately above; it does not repair the source citations.

### Accepted maximum-allocation output slice

Explicit maintainer review accepted the [D024 amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
on 2026-10-19. It is implemented in development: optional active output defaults
to the context maximum, smaller output requires the dynamic option, and actual
output/display-motion allocations still equal that maximum. Descriptors retain
real allocation metadata while output dimensions and display UV motion scale use
the active image. Both owners share inspection, preflight, terminal-error policy
and reset forwarding. The [implementation report](../release/2026-10-19-active-output-verification.md)
passes 62 public-output cases and 257 consolidated Windows stable/MSRV/ABI/
consumer/native cases. The public profile covers 52 fresh contexts: 144 dispatched
frames plus 12 no-dispatch control frames on selected 4.1.1/3.1.5, Windows x64/MSVC
DX12, RX 9060 XT and driver `32.0.31041.1004` at the same SDK v2.3.0 pin.

Stable/default, aligned/rounded and independent-axis output changes pass; rounded
output includes render/display UV/pixel motion with sharpening off/on. At maximum
allocations, both providers write 640×360 → 512×288 → 640×360 for requests
637×357 → 511×287 → 637×357, reproducing the raw spike's corresponding arm rather
than its oversized allocation arm. All 24 matched owner comparisons have equal
inputs/outputs, and 16 UV/pixel comparisons have equal outputs on the analytic
plane. This bounds encoding/descriptor consistency, not general motion consumption
or sampling coverage. CPU-fenced replacement and successful terminal cleanup pass;
the GPU run exercises no native rejection or failure-cleanup branch.

The retained source inventories differ only by two exact Rustdoc corrections
between the public GPU and consolidated runs; the report records those replacements
and hashes. Packaging/file/licence inspection passes, but no final extracted-package
verification follows from it. This intake checks the compact inventories and
outcomes without new compiler/GPU execution or changes to the original records.
This is an accepted conditional provider/access contract, not a proof of bounded
shader access or a new generic capability query. 2.3.4 smaller-output use remains
outside that contract; padding may be written and renderer GPU duties remain.

## External formats and explicit motion scales

The [U05 source record](records/2026-10-19-src-upscaling-formats-and-motion-scale.md)
examines SDK v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3` and a supplied
Rust archive. Its source findings are reported evidence; intake corroborates
selected public headers and the current wrapper without repeating the backend/
shader investigation. That source intake alone added no native/GPU work, format
widening or accepted API. The following intake assessment retains its dated
baseline; the later accepted D034 checkpoint is separate below.

### External role, format conversion and view legality

**Implementation at source intake:** the wrapper accepted RGBA16F colour/output and
generation colours, R32F device depth/1×1 exposure, RG16F motion and R8_UNORM masks/
generation output. Four concrete DXGI formats map to native formats; raw bindings
additionally declare UNKNOWN. Textures retain the accepted maximum allocations
and single-sample/mip/slice Texture2D shape. The new record corroborates these
code facts rather than replacing the prior public GPU evidence.

| Source-supported distinction | Consequence for U05 |
|---|---|
| Guides describe application-specified colour, one floating depth component and two floating motion components. | Exact existing formats are wrapper policy. RG32F motion is a narrow candidate; R16F depth, other floating/normalised colour and output alternatives require individual review. A documented family is not an exact provider format allow-list. |
| FSR3 documentation prescribes R32F/1×1 exposure and R8_UNORM supplied masks. | Generic converter entries do not justify R16F exposure, integer/SNORM masks or arbitrary motion-channel reinterpretation. Generation's input/output roles need their own operation-specific evidence. |
| Raw enum membership, converter implementation and legal external use are distinct. | New raw declarations must retain native values; binding a constant alone accepts no role/format pair. The record reports an enum entry for RGB32F whose DX12 converter cases are commented out. |
| The generic backend reportedly uses both the real allocation description and supplied native format, then derives SRV/UAV views. | Truthful allocation metadata is necessary but insufficient. Typeless/depth families and RGBA8 sRGB/UNORM interpretation cannot be reduced to an enum lookup. |
| Registration reportedly creates UAV descriptors when the real resource has UAV capability, including read-only input roles. | Review allocation flags and every generated view, not only the intended shader role. Device typed-store support and typed-load support are separate; require loads where the shader actually needs them. |

The report distinguishes R32 typeless/D32 and D32S8 depth families from an ordinary
R32F input. Native helper format collapse does not preserve the complete depth/
stencil view interpretation. D16/D24 converter cases do not establish the current
floating-depth contract. Keep those extensions explicit rather than accepting
them from helper coverage alone.

Likewise, an sRGB SRV's decoding and the algorithm's non-linear colour flags are
different controls; combining them without an established interpretation can
double-convert. Proposed sRGB/typeless/BGRA or packed output cases need exact
view/role review and selected-provider evidence. Existing D027 signal limitations
remain; no automatic colour conversion is selected.

### Explicit motion scale and unresolved conventions

**Reported open-source trace:** for each axis, open FSR3 uses a GPU scale constant
`S / T`, where `S` is the native dispatch scale and `T` is active render or active
output according to motion resolution. It multiplies sampled motion by that
constant, with a separate jitter-cancellation subtraction when enabled. This
explains the existing UV choice `S = T` and pixel choice `S = 1`; an explicit
native scale is not itself the final UV multiplier.

Open FSR2 reportedly uses configured display size for display-resolution motion,
while render motion uses active render size. This corroborates why FSR3's active-
output normalization cannot be transferred to the recorded 2.3.4 smaller-output
path. The source result neither traces that installed binary nor changes D024's
provider obligations or the current UV/pixel defaults.

The public ABI has two independent float components; the reported open paths
apply signs and unequal magnitudes without an absolute-value/clamping transform.
The header supplies no positive-only numerical domain or motion-scale `(0, 0)`
default sentinel. Open debug warnings for zero or positive values above maximum
render dimensions are diagnostics, not a dispatch validator or universal bound.
Finiteness is a proposed wrapper hygiene rule, not proof of finite downstream
products, meaningful motion or safe provider access. Whether to permit zero,
negative zero and extreme finite values still needs an explicit contract. Preserve
accepted bits if exposed; do not infer defaults from zero or invent an epsilon/cap.

**Unresolved source disagreement:** the record contrasts the FSR4 guide's raw
NDC-delta example with full positive dimension scales against Cauldron's explicit
half-scale/Y inversion before dimension scaling. These are not generally
equivalent producer conventions. No automatic NDC interpretation or universal
sign correction follows; exact producer direction, viewport and jitter semantics
remain renderer-owned. Closed-provider arithmetic is not visible in this record.

### Contract and verification implications

The record recommends explicit per-axis forwarding while preserving UV/pixel
behaviour, and a reviewed role/format/view matrix. Those recommendations do not
select an enum variant, optional override, zero policy, format list or new device
preflight. Both owners must share any eventual implementation and preserve backend
isolation, actual allocation metadata, aliasing policy and GPU ownership.

ABI/bit-forwarding and deterministic role/converter checks are ordinary
implementation verification. Additional native format tests should cover only
selected role/view/provider claims, with RG32F motion recommended as an initial
candidate. Typeless/DSV and sRGB extensions have distinct view questions. Native
signed/asymmetric motion consumption needs axis-specific evidence if claimed;
equal final images alone cannot prove it. Provider-conditional forwarding can
remain explicit without claiming universal consumption or a safe numerical range.

The proposed probes are future work, not selected or executed here. The report's
guard/readback and optional GBV suggestions must not be read as an established
way to prove out-of-resource safety: in-allocation guards and ordinary debug
silence cannot expose every attempted external access. The existing signed-provider
GBV stall remains separate. Existing 3.1.5 reactive-generation write/consumer
evidence stands; source uncertainty about a generic helper format does not
invalidate it or require automatically repeating that verified profile. The
4.1.1/2.3.4 no-write results also remain unchanged.

### Accepted native scale and typed-motion slice, 2026-10-21

[D034](../adr/D034.md) adds portable `NativeMotionVectorScale` and an optional
per-frame override, preserving UV/pixel fallback and exact finite bits, including
zero/negative zero and extreme finite values. NaN/infinities reject. The main
motion role accepts RG16F/RG32F; current mapping therefore covers five concrete
DXGI formats plus the existing raw UNKNOWN declaration. Other roles/generation
formats and maximum allocations remain unchanged. RG32F alone adds a device
format query after canonical membership checking: texture/load/sample and, for
an actually UAV-capable input, typed UAV-view support. This neither suppresses
real usage bits nor requires typed UAV shader loads/stores for a read-only role.
Preflight and dispatch share checks; errors retain eligibility and HRESULTs.

[Public-path evidence](../release/2026-10-21-native-motion-verification.md)
extends the earlier raw spike to both owners, render/display motion and actual
UAV-capable inputs on selected 4.1.1/3.1.5. All 56 encoding/sensitivity criteria
pass, including consistent scaling of the whole jitter contribution in a
mixed-sign case. Finite input bits, device support and matching outputs certify
no general native numerical/access domain, automatic NDC convention or native
jitter-cancellation consumption. Broader applicable formats and final release
archives remain separate; GBV's historical stall and 2.3.4 limits stand.

### U05 intake provenance and limitations

The record supplies investigation date 2026-10-19 and inspected archive
`fsr-sdk-rs-main(44)(1).zip`, SHA-256
`d6b6fbfa1bf94b527b72ad7c9fde512dada7c5bfe7963db7cb13575bc885d966`.
That archive was not available for independent intake validation; its hash is
reported provenance, not a Git identity or verified match to the current tree.
The record is preserved as 56,553 UTF-8/LF bytes, SHA-256
`7cff7058143552db0f2ebcb0d5f07a755566825375b6efacacaec877815724e1`.
No personal paths or obvious credentials were found.

Intake rechecks all four locally staged header hashes against the existing pin
manifest. Representative enum values and depth/UAV usage mapping agree with the
record. However, its claimed checked source spans are incomplete: in the pinned
DX12 header `ffxApiGetResourceDX12` starts at line 204, and format/additional-usage
assignment is at 267–268, beyond its cited 183–245 range. Resource-usage bits in
`ffx_api_types.h` extend through line 86, beyond its cited 25–79 span. Use symbols
and pinned files; the original record is unchanged, and the full backend/shader
source and Microsoft documentation were not independently revalidated at intake.
Source findings establish no new compiler, device/view or GPU result.

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
those capabilities. D014 first exposed native sharpening; later development adds
pre-exposure, depth/motion profiles, automatic exposure, active render regions,
supplied masks and linear HDR, tracked separately in the
[roadmap](../../ROADMAP.md#next-actions). Sharpening and pre-exposure are per-frame
scalars; automatic exposure, depth/MV conventions and
HDR are creation choices. Non-linear input combines a creation capability with
per-frame encoding flags. Reactive-mask generation is a separate tagged GPU
dispatch through the same export, not merely another main-frame input.

**Extent and resource qualification:** allocation dimensions, creation maxima
and active frame extents are distinct. Dynamic render resolution has a creation
flag; variable output uses `upscaleSize` without an analogous creation flag.
That shared field does not establish correct signed-provider history across
size changes. [D024](../adr/D024.md) now permits changing active render regions
within fixed maximum allocations and output; [D025](../adr/D025.md) checks actual
provider requirements for supplied masks and exposure. FSR3/FSR4 guidance differs.
The DX12 helper's format-conversion table is not an upscaler acceptance whitelist.
D011's original fixed profile remains the historical baseline, while the later
feature contracts define the development extensions.

**Earlier design inference and subsequent decision:** adding even an optional public
field to `UpscaleDispatch` breaks existing exhaustive struct literals. D012's
private creation options solve a different extensibility problem. Before the
first per-frame extension, the record recommends an explicit choice between
retaining the existing route plus an extensible advanced frame input, or a
breaking release with one private-field frame representation. The maintainer
subsequently accepted [D014](../adr/D014.md) for the latter `0.2.0` route;
M8 is now implemented and checked in the
[2026-10-09 implementation record](../ROADMAP_HISTORY.md#frame-and-sharpening-integration).
[M9](../release/2026-10-10-m9-verification.md) subsequently passed public native
integration and exact local candidate-artefact checks on the recorded configuration.
This preserves
`unsafe dispatch(&mut self, ...)`, renderer ownership, poison behaviour and
GPU-completion obligations.

**Subsequent experiments and remaining work:** the
[controlled semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md)
now cover motion/jitter direction, history sensitivity and reset for one scene.
The [D014 spike](records/2026-09-30-exp-d014-native-sharpening.md) adds stationary
sharpening comparisons with a failed proposed effect gate, detailed below. These
are later runtime evidence, not results of the source investigation. Subsequent
[Windows integration](../release/2026-10-15-windows-m11-m14-verification.md) covers
selected pre-exposure, exposure and active-render cases; it establishes no HDR
coverage. Later [mask verification](../release/2026-10-15-mask-forwarding-verification.md)
and the [HDR assessment below](#linear-hdr-and-non-linear-colour) supply separate
bounded native evidence. Broader format, temporal-quality and provider coverage
remains open.
GBV, signed-provider allocation/sharing, state restoration, COMPUTE-list support and callback/tuning
contracts remain unresolved within their existing scope.

## Linear HDR and non-linear colour

The [2026-10-15 source concept](records/2026-10-15-src-hdr-colour-encoding-concept.md)
examines SDK v2.3.0 at the same pinned commit against development source after
D025. It supplies source evidence and an additive API proposal, without native
execution. At that investigation's baseline HDR was unimplemented; subsequent
maintainer authorization accepted only the linear slice in [D026](../adr/D026.md).
The historical concept remains unchanged.

**Verified source findings:** HDR (`1 << 0`) and non-linear input (`1 << 8`)
are independent creation flags; optional sRGB/PQ hints occupy mutually exclusive
dispatch bits 1/2. The open FSR3 adapter forwards HDR but omits the non-linear
creation flag and both encoding hints. Shared declarations and an OK return
therefore establish no uniform provider interpretation. Resource requirements
contain no colour-capability bits; opaque provider IDs/names supply no generic
capability guarantee. These are findings reported by the source record, not new
signed-provider execution evidence.

**Accepted implementation:** portable `UpscalerOptions::high_dynamic_range` /
`set_high_dynamic_range` default to false. Both ownership families share the
creation/memory-estimate conversion, adding HDR bit 0; dispatch flags remain
zero by default. Linear RGBA16F colour/output and existing explicit/automatic exposure and
pre-exposure meanings are retained. The renderer supplies matching pixels; the
wrapper performs no colour conversion, tonemapping or HDR display setup. The
input-domain output contract follows upstream guidance and requires actual
provider support under the existing unsafe acquisition/dispatch obligations.
[D027](../adr/D027.md) additionally implements portable `InputColorSpace` and
`ColorEncodingHint`, exact non-linear creation bit 8 and exclusive sRGB/PQ
dispatch bits 1/2. Checked setters require NonLinear for either hint;
`set_color_encoding_hint_allow_context_mismatch` explicitly bypasses only that
pairing check for one frame. The ordinary setter restores checking, even for the
same hint. Defaults, other validation and poisoning remain unchanged.
The [follow-up source investigation](records/2026-10-15-src-non-linear-colour-contract.md)
records independent sample selections; the default is conservative wrapper
policy, not an established universal native requirement. The
[implementation checks](../release/2026-10-15-colour-encoding-verification.md)
add no provider/GPU confirmation. Wider formats remain outside the profile.

**Linear HDR verification:** the [implementation record](../release/2026-10-15-linear-hdr-verification.md)
establishes portable tests, consumer checks and prepared public fixtures. The
later [Windows batch](../release/2026-10-15-windows-feature-batch.md) executes
the fixtures, independent C++ ABI assertions, MSRV and six native HDR frames.
Explicit/automatic exposure and matching pre-exposure factors pass full input
readback, finite/written output and red strictly between 1 and 64, detecting
clipping and stale sentinels for the selected integration case. The batch's
unrelated original supplied-mask failures remain recorded; its overall failed
profiles do not invalidate the separately observed HDR results.

**Bounded non-linear execution:** the completed
[signal experiment](records/2026-10-15-exp-non-linear-colour-signals.md) adds
90 fresh processes and 270 GPU frames on Windows 11, RX 9060 XT, driver
`32.0.31041.1004`, signed provider 4.1.1 (`0xf5a5ca1e01001001`). Eight achromatic
stripes use HDR-enabled RGBA16F, 320x180 input and 640x360 output, three frames
per local-owner context, zero motion and fixed jitters. Independently implemented
inverse transfer functions audit the uploaded half-float codes. PQ uses the
declared test convention of one relative linear unit = 100 cd/m²; this does not
establish AMD's normalization. All tested RGB codes lie in `[0,1]`.

All 144 initial and 126 follow-up frames pass exact input readbacks, dispatch,
submission/fencing, finite/written output, ordinary debug/device checks and
explicit teardown; GBV remains off. Final CI has 115 passing cases and MSRV 98.
Native profile success includes complete numerical evaluation records, **not
acceptance of every signal hypothesis**. Production mapping and APIs are
unchanged. The compact measurements and repetition hashes preserve the results;
full pixel reanalysis requires the ignored local frames or a fresh run.

**Observed output and failed predictions:** neutral linear/sRGB/PQ outputs match
supplied code values; encoded neutral cases disagree with decoded-linear
candidates. Matching versus unspecified hints yield small reproducible output
differences, without proving complete standards-correct internal decoding.
Explicit exposure `0.25 → 1 → 4` and automatic-exposure controls also match
supplied codes. Byte-identical non-linear outputs across those exposure arms do
not establish ignored exposure: internally reversed operations can leave the
restored output invariant.

Changing pre-exposure `P=0.5 → 1 → 2` tests both `E(P*L)` and `P*E(L)` packing,
where `E` is the selected transfer encoder and `L` the relative linear signal.
The frozen per-patch/channel mean budget is `max(0.003, 0.03*abs(expected_code))`,
an exploratory domain discriminator, without a reconstruction-quality claim.

| Selected prediction | Initial continuous-history result | Follow-up under unchanged budgets |
|---|---|---|
| Linear and sRGB `P*E(L)` preserve supplied codes | Pass in every selected frame | Constant-P controls pass |
| sRGB `E(P*L)` preserves supplied codes | Zero-based frame 2 fails in each repetition; maximum error 0.003003919, marginally above the absolute budget | Constant-P and every-frame-reset controls pass |
| Both PQ packings preserve supplied codes | Zero-based frames 1 and 2 fail in each repetition; maximum errors 0.008828101 / 0.009876103 | Constant-P and every-frame-reset controls pass |

These are **15 failed frame predictions** across three repetitions, retained as
failures. All 126 follow-up predictions pass; maximum static/reset errors are
0.00126763 / 0.000488281. Within each arm, all three repetitions have identical
full output-hash triplets. Controls were selected after the initial failures;
they do not reclassify the original outcomes.

**Inference and unresolved contract:** the reset intervention and static controls
support dependence on continuous temporal state during the selected transitions.
They do not identify internal decode/pre-exposure order, select a universally
correct dynamic packing, establish a wrapper defect or justify automatic reset.
PQ luminance normalization/reference white, primaries, broader exposure
interactions and provider compatibility remain unresolved. Output-domain evidence
is bounded to these signals; preserved codes cannot determine reversible
internal operations or an absolute PQ scale. Clarification needs a provider
contract or a protocol with a discriminating observable. Under the maintainer's
2026-10-18 [completion boundary](../adr/D027.md#sdk-access-completion-boundary-2026-10-18),
these remain documented provider-specific limitations. M16's SDK-access contract
is complete without resolving internal processing or promising an end-to-end
non-linear pipeline. No converter, automatic policy, further signal experiment
or new release/quality gate follows; concrete wrapper defects remain in scope.

Coverage excludes coloured primaries, full PQ range, display integration, long
histories, movable non-linear GPU dispatch and universal provider semantics.
Optional hints describe supplied pixels; an unspecified hint does not mean
linear input. Resource masks and provider names do not settle colour capability.
Further quality studies are deferred until after 1.0.0.

## Remaining external format access, 2026-10-24

The [complete source accounting](records/2026-10-24-src-remaining-upscaling-formats.md)
partitions all 40 surface values, helper aliases and D035 residual families.
It identifies additional fixed typeless routes, normalized D16/D24 depth,
SNORM/shared-exponent colour and load-only RGB32F. RGB32F is a public-helper
omission with an existing native enum/backend view; D24's integer metadata does
not select an integer SRV. R16 typeless remains FLOAT and cannot expose D16
production. Typeless R8/R32 adds the documented mask/exposure interpretations,
not new encodings. Generic integer/normalized entries without an external role
and arbitrary view overrides are outside the documented contract.

The [independent experiment](records/2026-10-24-exp-remaining-upscaling-formats.md)
passes 119 direct-SDK contexts/357 fenced frames on selected 3.1.5/4.1.1, with
separate generation roles on 3.1.5. Device queries confirm load-only RGB32F and
optional shared-exponent stores on this adapter. Input integrity, shader depth,
planes, native calls and cleanup are distinct from FSR image semantics.
[D039](../adr/D039.md) accepts the checked routes and complete accounting under
the maintainer's U05 completion instruction; [public confirmation](../release/2026-10-24-resource-formats-verification.md)
passes 182 CI, 159 MSRV and 280 native stages, with 270 contexts and 810
fenced frames. Together with the earlier slices, U05 is complete for bounded
SDK access.
Earlier slices' remaining-family statements describe their bounded scope and
are superseded by this complete accounting. Provider/device trust and final
archive checks remain separate. No image/algorithm comparison gate is added.

## Typed BGRA/sRGB access, 2026-10-24

The [external source record](records/2026-10-24-src-bgra-srgb-dx12-upscaling-access.md)
is qualified by the [local/pinned-source audit](records/2026-10-24-src-bgra-srgb-audit.md).
Six LF-normalized source files match fresh content at the pinned SDK commit;
several claimed external citation ranges are wrong and the audit supplies actual
symbols/ranges. The reported project ZIP identity remains independently unverified.
Its supplied sys-version baseline predates amended D031. The original record
is unchanged; recommendations did not themselves accept a format contract.

The backend keeps typed sRGB SRVs but creates same-family UNORM UAVs; default
typeless colour registration chooses UNORM views and does not supply arbitrary
caller-selected SRVs. Logical RGB, physical byte order, transfer decoding and
native colour hints are distinct. The [independent DX12 view experiment](records/2026-10-24-exp-bgra-srgb-views.md)
confirms seven input and six store/read cases on the recorded adapter, retaining
the initial compile and overly strict sRGB-conversion criterion failures.

[Direct SDK integration](records/2026-10-24-exp-bgra-srgb-sdk-integration.md)
records 88 children, 252 main calls and 72 generator calls on selected
4.1.1/3.1.5, with generation only on 3.1.5. The original out-of-scope numerical
comparison failure remains historical; the separate authorized integration
review checks resource/native/write/fence/cleanup evidence and passes. These
records establish no public Rust-wrapper format admission or universal provider
support. They select no further quality or CPU mask-algorithm investigation.

The maintainer subsequently authorized [D038](../adr/D038.md) and implementation:
three typed formats, independent role selection, actual SRV/UAV support, conditional
casting, existing explicit colour controls and both-family shared validation.
[Public-wrapper verification](../release/2026-10-24-bgra-srgb-verification.md)
passes as a separate implementation checkpoint, covering both owners, exact
metadata, view/error paths, UAV-capable inputs and fenced SDK integration. Remaining
U05 candidates are subsequently resolved by D039; provider obligations and
final release archives remain separate.

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

**Verified enabled-zero behaviour:** enabled 0 differed from disabled in both
scenes (red MAE 0.0022058795 for frequency and 0.0010861201 for edge). Pass
enablement and strength therefore need distinct documentation; zero is the
parameter's lower endpoint, not a promise of disabled or identity output. The
edge profiles narrowed and showed overshoot, with a finite ROI maximum of
1.734375 at strength 1. Edge execution/profile measurements are diagnostics,
not an independent effect gate or a general image-quality result. Open FSR3/FSR2
RCAS behaviour does not explain the signed FSR4 internals by itself.

**Design inference and subsequent integration policy:** the source inventory's available
fields now have bounded enabled-pass runtime evidence. Successful execution
does not establish the proposed 1% gradient/0.005 mean-preservation policy.
The [2026-10-09 integration protocol](../release/2026-10-10-m9-verification.md#integration-protocol)
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
no new queue depth, cross-queue behaviour, thread transfer or general provider
support.

## M9 public sharpening integration and candidate verification

The [2026-10-10 record](../release/2026-10-10-m9-verification.md) executes the
unchanged [selected protocol](../release/2026-10-10-m9-verification.md#integration-protocol).
It uses public `UpscaleResources`, `UpscaleDispatch`, the sharpening setter and
unsafe `Upscaler::dispatch`; it does not use the historical spike's private
descriptor override. Both runs identify signed SDK v2.3.0/provider 4.1.1,
RX 9060 XT and driver `32.0.31041.1004`, with DLL fingerprints in the record.

**Verified bounded execution:** development source and the unmodified extracted
wrapper `0.2.0` artefact each completed three isolated repetitions of 48 outputs.
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

**Artefact boundary:** the local candidate pairs `fsr-sdk 0.2.0` with
`fsr-sdk-sys 0.1.1` and excludes D013's public `movable` family. Stable/MSRV,
Rustdoc, migration/consumer controls, unchanged archives/extracted files and the
publication dry-run passed; see [distribution findings](licensing-and-distribution.md).
It remains unpublished. The selected integration gate has no image-effect
thresholds and does not reinterpret the original failed experiment. GBV, moving
scene sharpening quality, broader providers, allocation capacity, recording ahead
with sharpening and cross-queue behaviour gain no new evidence.

[D016](../adr/D016.md) is the accepted cross-feature constraint for direct
provider controls and explicit SDK quality policies. It adds no experimental
result, future sharpening variant or quality guarantee and does not change
D014's API, native mapping or separately selected release protocol.

## V01 scene and reference contract

The [source record](records/2026-10-10-src-v01-scene-reference-contract.md) reports
GitHub HTML inspection on 2026-10-10 of six files at the same v2.3.0 commit.
It explicitly did not inspect the repository or supplied ZIP, obtain raw-file
hashes, build code or execute a GPU workload. Its reported project baseline and
GPU/driver/provider configuration were supplied context. Intake on 2026-10-11
preserved the record unchanged and found no secrets or personal machine paths.
Selected checks of the pinned [FSR4 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md),
[dispatch header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h)
and [GBuffer shader](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/Cauldron2/dx12/rendermodules/gbuffer/shaders/gbufferps.hlsl)
corroborated the central source claims; this was not a complete independent
reproduction of the six-file investigation or a binary/runtime check.

**Verified source facts and upstream claims:** the guide uses current-to-previous
motion, jittered render inputs with jitter-free motion for zero flags, optional
FSR4 masks, Halton-[2,3] jitter with 32 phases at 2x, and reset at camera cuts.
The shader explicitly converts NDC to top-down UV with `(0.5, -0.5)`; the guide's
raw-NDC example omits that conversion. Different projection/callback signs alone
do not demonstrate a defect. No reset/fresh-output equality guarantee or universal
image-error threshold was identified. Open FSR3 reset mechanics remain outside
proof of the signed FSR4 provider's behaviour.

**Repository comparison:** current [resource inputs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs),
[descriptor mapping](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs)
and [constructor](../../crates/fsr-sdk/src/upscaler/dx12.rs) agree with the supplied
five-texture profile: normal finite depth, UV motion scaled by render dimensions,
explicit exposure, pre-exposure 1, zero flags and absent masks. Caller-selected
camera/jitter/reset values and disabled sharpening can express the proposed first
scene without new public APIs. This establishes representation compatibility,
not successful execution of that scene. The public frame documentation asks for
reset on the first frame; the record's narrower statement that this is not an
explicit upstream FSR4 requirement does not relax that wrapper convention.

The existing [semantic experiment](records/2026-09-30-exp-dispatch-semantics.md)
and current [image generator](../../crates/fsr-sdk/tests/native/images.h) use one
constant-depth translating sinusoid, eight jitter samples and a final red-channel
comparison. They do not expose a previously occluded background. The earlier
zero reset/fresh difference is a measured result for that scene/configuration,
consistent with the new record's refusal to infer universal bit equality.
The helper's `half()` truncates mantissa bits and flushes small values to signed
zero; a V01 input oracle must specify the actual encoding and inspect stored
bytes rather than assume round-to-nearest IEEE half conversion. The subsequent
[calibration](#temporal-calibration-and-candidate-verification-criteria) checks
normal-half truncation and actual DX12 input bytes for its selected scenes.
The source intake changes neither that helper nor its historical observations.

**Inference and proposed test design:** colour, depth and motion must come from
one visible surface at the same jittered sample. A moving opaque foreground over
a static background gives distinct depth and object identity. Newly exposed
background keeps its own motion (zero for a fixed camera), even if its previous
location was occluded. Motion follows persistent surface correspondence, not
the object previously occupying the pixel. Offscreen previous projections need
a separate camera/object-entry case; the record identifies no invalid-motion
sentinel or provider guarantee. Its LH/+Z perspective depth formula and example
near/far/surface distances are mathematical test choices, not mandated SDK values;
the later input-only check must validate projection, depth, silhouette and motion.

The record separates quantized low-resolution inputs, an unjittered analytic
display point reference, an optional area-integrated reference and the actual
provider reconstruction. These references describe measurement targets; neither
is a promised FSR4 output. At edges, coverage differences, filtering and input
quantization must be distinguished from ghosting. Proposed RGB MAE/RMSE,
quantiles/maxima, temporal residuals and foreground-leakage traces use independent
visibility masks, edge distance and age since disocclusion. A global mean or the
existing red-channel final-frame comparison cannot replace those diagnostics.

**Broader source proposal (not executed in full):** use 96 frames across three stored 32-phase
jitter cycles at 320x180 to 640x360, with separate stationary, horizontal,
vertical/subpixel-diagonal and camera-cut arms. Keep scene motion continuous
through cycle boundaries. Pair history+reset with fresh+reset using identical
current input bytes and dispatch settings, then compare the cut and following
frames. Targeted foreground-motion inversion/freezing, incorrect dispatch jitter
and omitted/excess reset are sensitivity controls; their metric ranking is a
hypothesis, not a guaranteed provider response. Per-frame fencing, disabled
sharpening and the chosen lengths remain harness strategies. Retain complete
input/output readbacks, actual jitter values, masks, camera/scene settings and
SDK/runtime/provider/GPU/driver provenance. Calibrate repeatability, warm-up,
reference choice and local tolerances first; freeze criteria before separate
verification runs. [D016](../adr/D016.md) likewise separates calibrated policy
from validated quality outcomes. V01 remained open at source intake; this synthesis
accepts no quality policy and does not reinterpret the failed sharpening effect gate.
The later experiment deliberately selects a smaller 64-frame, two-scene matrix;
the remaining source proposals are not prerequisites silently imposed by intake.

## Temporal calibration and candidate verification criteria

The [2026-10-11 record](records/2026-10-11-exp-v01-verification-calibration.md)
adds execution evidence to the source contract without expanding the public
API. It uses SDK v2.3.0, the hash/signature-checked loader and actually observed
provider 4.1.1, RX 9060 XT and driver `32.0.31041.1004`, Windows x64/MSVC.
The current isolated profile is `experiment-temporal-calibration`; captured
attempts retain their original names. It is excluded from ordinary CI, release
and native acceptance profiles.

**Verified inputs and execution:** a stationary affine plane and a diagonally
moving opaque rectangle over a background at a different depth run at
320x180 to 640x360. Two saved 32-phase jitter cycles cover 64 frames with
continuous motion. Colour, perspective depth and jitter-free UV motion share
one visibility decision; newly exposed background retains zero motion with a
fixed camera. Sharpening is disabled, exposure/pre-exposure are 1, flags 0, no masks.
An initial 21-frame DX12 input-only child does not load AMD's runtime; independent
geometry and quantization checks match every readback byte. The same oracle
checks all 903 full-matrix input payloads and independently decodes all RGBA16F
outputs. Each final child completes 301 dispatches, submissions and confirmed
fences with zero debug/device errors, followed by context/resource cleanup.

The matrix contains A correct, B correct, B reversed foreground motion and B
negated dispatch jitter at unchanged image jitter (64 frames each), plus cut
history/reset (0..40) and fresh/reset (37..40). Absolute scene/jitter identity
continues across the cut. All three process repeats are bit-identical across
all arms; all twelve reset/fresh comparisons have identical current inputs and
full RGBA output. This is a local observation, not an upstream equality promise
or causal reset proof: no history/no-reset cut control was run.

**Mixed diagnostic findings:** measurements use independent analytic point and
area references, visibility/edge/age masks and reprojected error residuals,
without clipping. Neither reference is a promised FSR output. The following
repeat-1 values pool frames 32..63 by pixel count; other repeats are identical.
RGB errors are in linear colour units; leakage is dimensionless.

| Measurement | Pixel-frames or pixel-pairs | Correct | Reverse motion | Negated jitter |
|---|---:|---:|---:|---:|
| Foreground reprojected temporal residual MAE | 324,056 pairs | 0.000299944 | 0.000640252 | 0.000415732 |
| Silhouette area-reference MAE | 60,544 pixel-frames | 0.0489006 | 0.0513651 | 0.0882337 |
| Interior point-reference MAE | 6,808,448 pixel-frames | 0.000621367 | 0.000530946 | 0.000532336 |
| Age-2..5 signed foreground leakage outside the two-pixel edge band | 11,192 pixel-frames | 0.0276800 | 0.000756630 | 0.0164471 |

Foreground temporal residual and silhouette error distinguish the selected
wrong controls in the expected direction. Interior error, background temporal
residual and older-disocclusion/leakage measurements can improve with incorrect
inputs. Pure-background age-0/1 leakage and age-2..5 beyond eight pixels have
zero samples at this velocity: their results are null, not zero error. A's
phase-matched outputs also differ after different history lengths; two cycles
and an affine plane establish no convergence or textured-scene quality bound.

**Evidence and bounds:** the final attempt passed its 20 structural cases.
Three earlier attempts remain failed: missing offline-analysis parent,
Python case-setup shadowing before child execution, and loaded-DLL extended-path
comparison after complete GPU execution. The
[dated retention update](records/2026-10-11-exp-v01-verification-calibration.md#retention-update--11-october-2026)
removed raw payloads, source snapshots and verbose reanalyses after validation;
compact measurements and failed outcomes remain. Full historical image reanalysis
is unavailable. Each complete attempt originally captured approximately 2.52 GB
decoded inputs/outputs, losslessly compressed to 284,745,467 bytes. Final children took
13.265–13.406 seconds, with peak working sets 599,957,504–604,270,592 bytes;
offline analysis took 286.391 seconds. Per-frame fencing and retention through
context destruction bound this harness, not general performance or in-flight
capacity. No AMD timeout, uncertain-completion recovery or GBV case was induced.

**Historical calibration recommendation:** freeze structural completeness, exact input
conventions, named references, time windows, ROI counts and repeat handling
before a later acceptance run. The present evidence motivates paired foreground
temporal-residual sensitivity to reversed motion and silhouette area-error
sensitivity to negated jitter; it does not justify a universal error/leakage
ranking or an absolute quality threshold. Required empty regions must be
reported as missing coverage. A stronger reset-sensitivity scene with a
predeclared history/no-reset control and any expanded disocclusion variant need
new protocols and execution. Calibration data cannot retrospectively verify
criteria chosen from them. The later diagnosis below refutes the proposed general
jitter/silhouette ranking; subsequent confirmation selects separate properties.
Existing decisions are unchanged and the historical sharpening effect gate remains failed.

## Confirmed temporal properties and metric limits

Five later records distinguish diagnostic counterexamples from prospective
confirmation. All use the public Upscaler, SDK v2.3.0, the same measured loader/
upscaler DLL hashes, provider 4.1.1, RX 9060 XT and driver `32.0.31041.1004` on
Windows x64/MSVC. Full input-byte audits, finite complete RGBA readbacks, fences,
ordinary debug/device checks and cleanup accompany the GPU results. This synthesis
checks retained evidence and current profile/source correspondence; it reruns no
native experiment.

| Record | Confirmed observation | Qualification |
|---|---|---|
| [Metric diagnosis](records/2026-10-11-exp-temporal-quality-metric-diagnosis.md) | Two 1,216-frame matrices reproduce mixed rankings; fast-flat negated jitter improves silhouette MAE, contradicting the calibration's proposed ranking | Structural execution passes; no replacement quality criterion selected |
| [Evaluator/quality validation](records/2026-10-11-exp-temporal-quality-validation.md) | Ideal analytic image passes; twelve known defects fail their named properties; three fresh 576-frame blocks pass frozen textured-slow limits and motion/reset sensitivity | Other texture × speed × motion cells remain diagnostic, including improved foreground MAE with reversed motion in textured-fast |
| [Continuous history](records/2026-10-11-exp-temporal-long-sequence.md) | Three 512-frame blocks run correct/reversed 256-frame histories; all seven windows per block pass unchanged quality limits and both motion predictions | Reset only at frame zero; no unlimited-history or per-frame error bound |
| [Cut/reset](records/2026-10-11-exp-temporal-reset-recovery.md) | One exploration and three confirmations: reset matches fresh exactly for all 64 post-cut frames; retained history produces a measurable difference; all frozen quality windows pass | One camera displacement and cut phase; no-reset also fits the quality limits, so state comparison supplies reset sensitivity |
| [Jitter sign/phase](records/2026-10-12-exp-temporal-jitter-phase.md) | Three fresh confirmations yield 45 passing windows and 225 wrong-input sensitivity predictions across four stationary starts and one moving start | Independently validated periodic-scene position estimator; mixed edge/temporal rankings remain |

**Verified metric limits:** a missing object can have zero background error and
a constant foreground can have zero temporal residual. The validated evaluator
therefore tests distinct image properties together, rejects empty/nonfinite
required regions and checks known-defect severity. Correct inputs need not beat
wrong inputs on every scalar. In textured-fast, reversed motion improves
foreground MAE to .7753 times correct while worsening centred texture error to
1.4067 times and temporal residual to 3.0282 times. The affine-plane shift
diagnostic also confounds brightness bias with position. Neither counterexample
changes the documented motion/jitter convention.

**Verified scoped confirmation:** quality validation tests foreground/background,
silhouette, true-flow temporal residual and age-2..5 uncovered background against
separate analytic references. Textured-slow reversed motion exceeds correct
foreground and temporal error by 1.4334 and 2.5260 times; reset-every-frame
increases temporal error by 3.5570 times despite improving spatial error.
The unchanged limits pass through frame 255 in every declared window. These
cover useful disocclusion ages, while the original age-0/1 region gap is not
silently converted into coverage.

Cut/reset uses identical post-cut inputs for reset, no-reset and fresh arms.
Reset/fresh maximum RGBA difference is zero throughout; first-eight-frame
no-reset/fresh interior difference is .001449915 against the fixed .0001 minimum.
Jitter confirmation independently fits X/Y sinusoidal phase with brightness
and contrast separated. Correct radial RMS is at most .046551 stationary and
.033036 moving render pixel; every X/Y/both-sign, one-frame-lag and dispatch-only
zero control exceeds its paired correct alignment error by the frozen margin.
Jointly shifted image/dispatch start phases pass. The initial signed-zero input
encoding failure remains a failed oracle check, not an AMD failure.

**Inference and remaining scope:** property-specific sensitivity resolves the
earlier ranking ambiguity within these declared scenes. Exploration-derived
allowances were frozen before new reset/jitter confirmations; quality validation
used earlier measurements, and continuous history reused its limits unchanged.
The 25% allowances are local regression policies, not perceptual thresholds or
confidence intervals. Correlated frames and identical process repeats on one
deployment establish no population uncertainty or cross-driver guarantee.
Arbitrary images, other motion/scale/depth profiles, offscreen history, broader
cuts, hardware and longer durations remain follow-ups. Selected V01 completion
is already recorded in [roadmap history](../ROADMAP_HISTORY.md#v01--temporalimage-baseline--completed-2026-10-12).
The sharpening effect failure and GBV stall remain unresolved.

Protocols, criteria, source/native identities, compact measurements and failure
outcomes remain linked from each record. Generated full images were removed
after validated compaction; hashes cannot reconstruct historical GPU bytes.
See [evidence retention](evidence-retention.md) for the current boundary.

## Disagreements and limitations

- The [earlier sinusoidal semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md)
  rank matching horizontal motion/jitter below reversed inputs in analytic
  error. The [two-depth rectangle calibration](records/2026-10-11-exp-v01-verification-calibration.md)
  improves some spatial, background temporal and leakage measurements with
  deliberately wrong inputs. These are different scenes, masks, references and
  time windows on the recorded deployment, not grounds to erase either result
  or infer that wrong motion is valid. Sensitivity depends on the measurement.
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

## Proton single-frame and INT8 evidence

The [Steam Deck proof](records/2026-10-13-exp-proton-linux-cross-build.md),
[validation follow-up](records/2026-10-13-exp-proton-vulkan-validation.md) and
[Steam Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
reuse Windows Gate A's input, dispatch, fence/readback and numerical checks
through a separate Proton selector. Both hosts complete 640×360 finite, written
output with the expected gradient, no device removal and explicit teardown.
Vulkan core/synchronization activation and zero errors are observed; the Windows
case still requires its own D3D12 debug layer. Neither its missing info queue
nor GBV is counted as a pass under Proton.

**Verified:** each of the Machine's three shader-diagnostic runs compiled 30
compute shaders, 13 containing a total of 4,264 static packed INT8 dot-product
instructions. Experimental automatic/disabled upgrade generated identical
DXIL/SPIR-V pairs. Proton 11 generated the same DXIL inputs, with only 17 of 30
SPIR-V modules identical. All four successful GPU runs passed the unchanged
acceptance checks with matching aggregate readback metrics; full readback-byte
equality was not captured.

**Inference:** compiled packed INT8 shaders plus completed dispatch/fence/output
support selection of the 4.1.1 INT8 model path on this Machine. Per-shader dispatch
attribution and executed machine-instruction tracing were not obtained. Provider
version or shader dumps alone would not establish that execution. The direct
pinned SDK workload passes on Proton 11 without Experimental's contrib DLL and
with Experimental's upgrade disabled; older-game upgrades were not tested.

This record adds a second GPU/provider configuration for the single-frame proof;
it did not test temporal, sharpening-effect, image-quality or performance criteria.
The subsequent temporal records are assessed separately below.
The maintainer accepted [D019](../adr/D019.md#decision-status) and closed
the initial V07 proof and bounded V07e portability checkpoint on 2026-10-13.
V07d quality work is deferred until after 1.0.0. The later bounded V08 local-import/offline and V09
timeout
checkpoints pass on Deck, as assessed in [platform/backends](platform-and-backends.md#prepared-offline-runtime-and-timeout-containment).
Public safety contracts are unchanged.

## Selected Proton temporal regressions

The [Deck record](records/2026-10-13-exp-proton-temporal-regression.md) and
[Machine record](records/2026-10-14-exp-steam-machine-proton-temporal-regression.md)
reuse common orchestration, input oracles and Windows numerical criteria across
three process repetitions per profile. Jitter is limited to stationary and moving
phase 0, with six dispatch controls and 128 frames per arm. It is a subset of the
Windows phase study. Deck uses provider 3.1.5/Proton 10.0-4b; Machine uses provider
4.1.1/Proton 11.0. Their GPU/runtime differences are retained in the records.

| Selected protocol | Deck/provider 3.1.5 | Machine/provider 4.1.1 |
|---|---|---|
| Quality/history | Failed reset-every-frame sensitivity: reset/correct temporal-error ratio 0.808, required ≥1.1; normal quality and reversed-motion predictions pass | Failed foreground motion sensitivity in all repetitions: reverse/correct error ratio 0.71519, required ≥1.1; all normal-output budgets pass |
| Continuous 256-frame history | Failed background MAE from frame 64 onward; remaining budgets and motion sensitivity pass | All normal-output budgets pass; foreground motion sensitivity fails in all 21 windows, ratios 0.68223–0.73953, required ≥1.1 |
| Cut/reset | All 18 required checks pass; reset/fresh maximum difference is zero | All 18 required checks pass, including recovery and ghosting criteria |
| Selected jitter | First launcher returns 247 although all frame data and Rust completion are present; full image verdict blocked | All three launches complete; moving alignment exceeds limits in seven of nine windows; stationary alignment, normal quality and wrong-control sensitivity pass |

Deck has 64 passed, three failed and one blocked required case; Machine has 73
passed and three failed. The latter includes the eight-case smoke profile.
Counts do not imply acceptance: **V07d is deferred until after 1.0.0; quality acceptance remains unresolved.**
Separately observed
Vulkan core/synchronization activation has zero errors/warnings; Windows debug
and GBV remain unavailable. Completed frames, byte oracles and clean diagnostics
cannot substitute for image or launcher acceptance. The original Windows V01
confirmations remain valid for their recorded configuration; these newer
counterexamples limit transfer of those criteria across deployments.

**Unresolved launcher cause:** the [exit-247 diagnosis](records/2026-10-14-exp-proton-exit-status.md)
replays the exact Deck payload and seed-419 matrix five times with process/signal
tracing and once without, on historical and fresh prefixes. All six return zero,
and audited input/output inventories match the failed launch. Wine error 1726
also occurs in successful replays, so it is not a distinguishing marker. A POSIX
SIGKILL control demonstrates a possible -9-to-247 conversion, not the historical
cause; no original-failure trace exists. No image criteria were evaluated in
these diagnostic replays. Their pilot coordinator failures remain failures.

GPU/provider/driver/translation differences prevent causal attribution to Proton,
the wrapper or one provider. Keep the criteria unchanged pending a separately
declared diagnosis; a Machine jitter verdict does not unblock Deck acceptance.
If the launcher failure recurs during selected integration checks, capture
process/wait/signal events. Compact measurements retain
passes, failures and provenance; full temporal frames were removed, so full image
reanalysis needs a new GPU run. The exit diagnosis retains its observations and
reproduction recipe in the record, while its disposable coordinator and traces
remain under ignored `target/`; this synthesis does not reconstruct those files
or rerun GPU work.

## Open questions

1. **Signed-provider allocation behaviour:** [D011](../adr/D011.md) and M6b's
   public method use explicit bounded provider trust. The unknown allocation and
   sharing model remains an evidence limit, not a pending implementation gate.
   Neither source nor recording-ahead tests set a numeric capacity. Revisit the
   contract if new evidence identifies unsafe reuse or additional safety
   preconditions.
2. **Broader ABI coverage:** M6a passed the selected Rust/MSVC x64 dispatch
   checks; other SDK entities and x86 remain outside this proof.
3. **Further native validation:** if GBV success is required, obtain a native
   stack/trace or compare drivers/providers to distinguish its native-call
   stall from general GBV compute instrumentation. Other formats, states and
   broader provider workloads remain untested; Proton adds the selected
   single-frame and temporal comparisons above, with its own diagnostics.
   The debug-layer-only GPU
   evidence covers one resource profile and simple synthetic frames.
4. **Temporal correctness:** the final Windows three-frame moving smoke passed
   with applied jitter and current-to-previous UV motion. Its coarse finite/
   changed-output assertions do not establish correct reconstruction, history
   use, reset effectiveness or image quality. Recording ahead with zero motion
   also does not answer those questions. The [controlled semantic comparisons](records/2026-09-30-exp-dispatch-semantics.md)
   now cover horizontal motion direction, jitter direction, history sensitivity
   and reset for one scene/configuration; broader reconstruction remains open.
   The [V01 source intake](#v01-scene-and-reference-contract) supplies the proposed
   disocclusion inputs and independent references. The
   [bounded calibration](#temporal-calibration-and-candidate-verification-criteria)
   adds input-byte/projection checks, diagonal two-depth motion, regional
   diagnostics and local reset/repeat equality. The
   [later confirmations](#confirmed-temporal-properties-and-metric-limits) establish
   selected textured/disocclusion quality, 256-frame histories, causal cut/reset
   sensitivity and isolated jitter axes/phase under frozen criteria. Young
   age-0/1 disocclusion acceptance, motion-axis/scale profiles, offscreen history,
   broader content/cuts and cross-deployment tolerances remain open. The selected
   Proton comparison above preserves failed cross-deployment criteria and the
   unresolved Deck jitter launch; each expansion
   needs its own bounded protocol.
   No arbitrary overlap or general
   cross-queue support is established.
5. **Later scope:** output format range, alpha behaviour, aliasing, nontrivial
   resource shapes, COMPUTE lists, dynamic output size and general concurrency.
6. **Later per-frame extensions:** D014's breaking private-frame representation
   and native sharpening are implemented and checked under
   [M8](../ROADMAP_HISTORY.md#frame-and-sharpening-integration), with public GPU
   integration and local candidate artefacts verified under
   [M9](../release/2026-10-10-m9-verification.md). Later development implements
   pre-exposure, exposure/depth/motion profiles, active render regions, supplied
   masks and linear HDR under their own contracts. The
   [HDR assessment](#linear-hdr-and-non-linear-colour) distinguishes verified
   linear HDR and bounded non-linear integration from failed transition
   predictions and unresolved internal ordering/PQ normalization.
   Broader combinations and image semantics need provider-specific evidence;
   the [surface record](records/2026-09-29-src-dx12-upscaler-api-surface.md)
   supplies proposals, not implementation authorization.
7. **Sharpening acceptance policy:** the
   [D014 spike](records/2026-09-30-exp-d014-native-sharpening.md) supplies clean
   enabled-pass execution and measurable changes on one configuration, including
   enabled zero, but fails the proposed effect criteria. The selected
   [M9 protocol](../release/2026-10-10-m9-verification.md#integration-protocol) requires
   integration evidence without new sharpening image-effect thresholds and passed
   in [M9 verification](../release/2026-10-10-m9-verification.md). D014 was separately
   accepted by the maintainer on 2026-10-07. M8 fixtures establish exact wrapper
   mapping; M9 establishes bounded public-API recording, GPU execution and cleanup.
   Brightness preservation, moving-scene quality and broader provider coverage
   remain unresolved. The original failed effect gate supplies no quality promise.
