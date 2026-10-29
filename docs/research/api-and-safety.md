# API and safety

The [2026-10-09 maintenance recheck](../ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh)
refreshes the existing native tests on its captured pre-M7 baseline. Updated
observations are summarized in [loading](runtime-loading.md),
[lifecycle](context-lifecycle.md), [construction](upscaler-construction.md),
[threading](runtime-threading.md) and [dispatch](upscaling-dispatch.md).
Existing failure-contract gaps, sharpening's failed effect gate and GBV's
dispatch timeout remain; no architecture or safety contract changed.

- Updated: 2026-10-29 (U07–U12, common rc1 host qualification and SDK 2.4/Vulkan source compatibility integrated; publication simulation passes; publication remains separately authorised).
- Scope: common ABI, loading, provider deployment, version enumeration, context
  lifecycle contracts and isolated create-failure experiments; M5 construction,
  Query/Configure, runtime-sharing source findings and bounded signed-runtime
  experiments for FidelityFX SDK v2.3.0; post-dispatch teardown observations;
  the broader modern DX12 upscaler API surface and design implications;
  the selected M10 Query safety and conditional native-trust assessment;
  D013 trait compilation, serialized macOS/Windows fixtures and fenced native
  Windows/DX12 transfers, native Rust-unwind cleanup and staged residency,
  transfer/cleanup source limits, Windows failure/unwind fixtures and a supplied
  soundness/architecture review;
  D014's test-only native sharpening experiment and M9 public/candidate integration;
  published 0.2.0 compatibility, early validation and future-effect extensibility;
  the D042 public-API transition, concrete setup contract draft and isolated
  stable/MSRV composition and controlled-lifecycle evidence;
  and older FSR2 Rust binding comparisons. Native SDK layout evidence targets the
  Windows x64/MSVC ABI; experimental Linux-hosted execution translates those
  Windows binaries through Proton. D013 fixture execution also covers macOS ARM64. Native D013
  transfer and loader-release observations cover one identified AMD configuration.
- Included evidence: [common ABI verification record](records/2026-09-20-exp-common-abi-verification.md)
  and [explicit loader verification](records/2026-09-20-exp-explicit-dll-loader.md),
  plus the [version-count query](records/2026-09-20-exp-version-count-query.md),
  [DX12 provider-DLL resolution](records/2026-09-21-src-dx12-provider-dll-resolution.md),
  [signed DX12 directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md),
  [lifecycle contract refinement](records/2026-09-22-src-lifecycle-contracts.md),
  [source provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md),
  [distribution/artefact provenance](records/2026-09-22-src-distribution-artifact-provenance.md),
  [v2.3.0 lifecycle investigation](records/2026-09-20-src-context-lifecycle.md),
  [destroy failure-path investigation](records/2026-09-20-src-destroy-failure-paths.md),
  [successful DX12 lifecycle](records/2026-09-20-exp-context-lifecycle.md),
  [M4 Windows production-owner verification](records/2026-09-22-exp-m4-windows-verification.md),
  [create-allocation failure spike](records/2026-09-20-exp-create-allocation-failure.md),
  [returned-create-error input lifetime spike](records/2026-09-22-exp-create-error-input-lifetime.md),
  [Embark fsr-rs comparison](records/2026-09-20-src-embark-fsr-rs.md) and
  [fsr2-rs raw-binding comparison](records/2026-09-20-src-fsr2-rs.md),
  [M5 source provenance repair](records/2026-09-22-src-m5-construction-query-provenance-repair.md),
  [M5 numerical construction investigation](records/2026-09-23-src-m5-numeric-construction-safety-contract.md),
  [signed-runtime dimension probe](records/2026-09-22-exp-m5-upscaler-construction-inputs.md),
  [signed-runtime DX12 width-boundary probe](records/2026-09-23-exp-m5-dx12-dimension-boundary.md),
  [sequential shared-runtime probe](records/2026-09-22-exp-m5-shared-runtime-provider-lifetime.md),
  [M5 Windows native verification](records/2026-09-24-exp-m5-windows-native-verification.md),
  [final Windows verification](records/2026-09-28-exp-final-windows-release-verification.md),
  the [DX12 upscaler API surface investigation](records/2026-09-29-src-dx12-upscaler-api-surface.md),
  the [D013 compile/host experiment](records/2026-09-29-exp-d013-movable-compile-host.md),
  [D013 Windows fixture execution](records/2026-09-29-exp-d013-windows-fixtures.md),
  [D013 native transfers](records/2026-09-30-exp-d013-native-thread-transfer.md),
  [D013 transfer/cleanup source intake](records/2026-10-07-src-d013-transfer-cleanup-contract.md),
  [D013 transferred failure/cleanup fixtures](records/2026-10-07-exp-d013-transfer-failure-cleanup.md),
  [D013 extended native transfers and cleanup](records/2026-10-08-exp-d013-native-transfer-cleanup.md),
  [D013 soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md),
  the [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md),
  the [M9 public native and candidate verification](../release/2026-10-10-m9-verification.md),
  the [M10 DX12 Query safety review](records/2026-10-12-src-m10-dx12-query-safety.md),
  [early-error and recovery review](records/2026-10-18-src-error-recovery-and-preflight-ergonomics.md),
  [published-version compatibility review](records/2026-10-18-src-v020-api-breaks-and-coherent-030-design.md)
  and [full public-API extensibility review](records/2026-10-18-src-full-public-api-extensibility-and-v020-break-review.md),
  followed by the [API coherence implementation/verification](../release/2026-10-18-api-coherence-verification.md).
  The later [consistency/architecture audit](records/2026-10-26-src-public-api-consistency-and-architecture-audit.md),
  [usability/extensibility redesign](records/2026-10-26-src-public-api-usability-and-extensibility-redesign.md)
  and [setup contract/composition spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md)
  are integrated below with their separate source, decision and execution scopes.

## Topic map

Detailed findings are split by the contract they explain:

- [upscaling API coverage](upscaling-coverage.md)
- [callbacks, diagnostics and custom allocation](callbacks-and-allocation.md)
- [core and optional utilities](core-and-utilities.md)
- [runtime loading](runtime-loading.md)
- [platform and graphics backends](platform-and-backends.md)
- [context lifecycle](context-lifecycle.md)
- [upscaler construction](upscaler-construction.md)
- [runtime threading](runtime-threading.md)
- [upscaling dispatch](upscaling-dispatch.md)

## Current findings

**Current package/native checkpoint, 2026-10-29:** the
[consolidated review](../release/2026-10-29-candidate-verification.md) integrates
1,448 passing Windows archive/native checks, Deck's 16 offline plus 17 timeout
checks and Machine's 12 compatibility checks on the **same frozen rc1 pair**.
This includes archive-bound callbacks/production CB ABI on Stable/MSRV and
both-family examples/utils. The earlier Windows/Linux mismatch is historical.
Final P08 input/impact reconciliation is [accepted](../release/2026-10-29-rc1-performance-acceptance.md); publication simulation passes; publication remains separately authorised.
Provider-conditioned access, numerical
Configure uncertainty, callback quiescence and native failure recovery retain
their existing bounds. Neither an archive pass nor a translated DX12 run adds
a new native safety or platform guarantee.

### SDK 2.4 and Vulkan compatibility

The [2026-10-29 source investigation](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md)
supports D042's portable values and concrete backend boundaries. Its
architectural inference is that Vulkan can be added without redesigning the
0.3.0 DX12 API. It supplies no new compilation, ABI or native execution proof.
Current `Runtime::load` explicitly means a trusted Windows/DX12 loader with the
pinned 2.3 ABI; preserving that contract is preferable to a speculative rename
or generic backend trait. Vulkan needs explicit matching runtime acquisition,
its own device/resource/queue contracts and separately justified unsafe
recording, failure retention and threading. The non-exhaustive backend error
envelope leaves room for concrete future errors without adding Windows types
to portable inputs.

The [release assessment](platform-and-backends.md#sdk-24-and-release-sequencing)
keeps 0.3.0 on SDK 2.3, plans compatible SDK 2.4 adoption conditionally for
0.3.1 and retains Vulkan implementation for 0.4.0. Audit raw-interop exposure
and native safety/runtime compatibility as well as Rust signatures before
calling the SDK-only release nonbreaking. Backend-ID changes in upstream
headers do not alone establish a public Rust break or safe compatibility.
The report's Vulkan resource metadata, command-buffer encoding, queue ownership,
callback implementation and Linux runtime questions remain open. DX12/Proton
success and copied Vulkan handles settle none of them.

### Accepted API transition and setup draft

The two 2026-10-26 reviews assess the same production baseline. The
[consistency audit](records/2026-10-26-src-public-api-consistency-and-architecture-audit.md)
prefers targeted provider, motion, portable-sharpening and documentation fixes.
The [redesign review](records/2026-10-26-src-public-api-usability-and-extensibility-redesign.md)
builds on it with less weight on preserving signatures, promoting setup scopes,
portable frame data and named resource assembly to canonical boundaries. This
is a difference in design priorities, not contradictory native observations or
two independent proofs of one design.

**Accepted target, not implementation:** [D042](../adr/D042.md) selects the
structural transition for wrapper 0.3.0: one setup-bound provider request through
planning/create, portable parameters and separate backend resources/list,
complete effective motion/colour requests, checked tuning inputs, targeted
error/metadata improvements and explicit terminal usability. It preserves the
two crates, concrete local/movable owners, shared private mechanics, mandatory
inspection, renderer-owned orchestration and unsafe native boundaries. It does
not adopt every review sketch, create a universal backend/effect trait or select
a quality policy. [U07–U12](../../ROADMAP.md#next-actions) track delivery.

| Layer | Current state |
|---|---|
| Production | U07 setup-bound planning/create and provider-free options are implemented. U08 portable parameters, named resources and one motion/colour request now replace combined dispatch and dormant settings. Native trust and cleanup limits still apply |
| Accepted direction | D042 replaces those public shapes during the selected 0.3.0 transition; earlier feature evidence remains evidence of the preceding API |
| Concrete contract | The [U07 contract](../design/upscaler-setup.md), approved on 2026-10-27, assigns loader/effect/recording/installation duties and uses independent runtime/device setup lifetimes |
| Executed candidate | The [bounded spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md) reuses production routing/ownership/inspection in an isolated copy with real Windows types and controlled DLL/COM inputs; it changes no production API |

**Verified within that spike:** Rust 1.99 and exact 1.88 each compile the three
local/movable consumer workflows, portable data and movable trait assertions.
Each passes 17 selected tests, one separate no-default portable test and 20
expected compiler rejections. The coordinator reports 64/64 passing process
cases, including provenance checks and strict stable Clippy; this is not 64
AMD/GPU tests. Weak runtime-state witnesses and COM counts confirm independent
context retention, while fresh selection, preflight reinspection, literal
scale-bit mapping, cleanup/unwind and terminal-cause rules pass controlled
execution. The DLL observer itself retains a library reference, so those
witnesses do not independently observe final operating-system DLL unload.

Two contract consequences are now explicit. Safe provider rebinding requires
the setup's prior trust to cover providers and input domains reachable through
later safe operations, not just the initial selection; default remains an
invocation-time request. Movable loader/final-release compatibility must already
hold before the loaded runtime can enter safe `Send` code. Effect trust can be
acquired at setup, but persists after the view/public runtime is dropped.
An exclusive setup borrow provides neither global serialization nor GPU idleness.

**Production follow-up, 2026-10-27:** U07 was approved and implemented with
public Rustdoc and migrated consumers/examples. Its
[implementation verification](../release/2026-10-27-upscaler-setup-verification.md)
records compiler, controlled and native checks separately. U09 now implements
portable checked tuning requests and full-list preparation; its
[verification](../release/2026-10-27-tuning-request-verification.md) passes current
Rust/MSRV, controlled/consumer/ABI and bounded native checks. U10 adds structured
owned errors, retained first native causes and provider/mask composition; its
[verification](../release/2026-10-27-error-metadata-verification.md) records the
new production surface and source bounds. U11 now implements pure usability and
local abandonment; its [production verification](../release/2026-10-27-context-usability-verification.md)
records both-family guards and bounded cleanup. U12 now integrates the API and
complete external workflows; [migration](../MIGRATING_0_3.md) and
[verification](../release/2026-10-27-api-transition-verification.md) distinguish
that source checkpoint from the separately
[closed P08 review](../release/2026-10-29-performance-review-closure.md) and still-open
final archive qualification.

**Limits of the earlier spike:** U09 checked tuning,
U10 error/metadata restructuring and the full U08 generator builder were not
implemented in the probe; its small abandonment/first-cause guard does not
complete U11. No AMD provider, GPU, independent ABI, callback quiescence,
performance or final-package evidence was added. Earlier native assumptions and
failed observations remain binding; the spike does not turn them into guarantees.
The record preserves its preparation/Clippy failures and disk-exhausted repeat
separately from the final successful confirmation. This synthesis reruns none of
those experiments.

Intake on 2026-10-26 matched all 53 crate inputs in each source-review inventory
and all 178 recorded spike inputs against the checkout. Decision/planning files
have intentionally evolved since the reviews; historical inventories and records
are preserved. At that intake the contract remained a draft; the later U07
approval/implementation is recorded above and does not complete U08.
External references and native observations were not independently re-established
during this intake.

### Implemented SDK-access baseline and earlier evidence

The [2026-10-25 callback/allocation assessment](callbacks-and-allocation.md)
integrates the [SDK contract investigation](records/2026-10-25-src-upscaler-callback-allocation-contracts.md)
and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md),
followed by the [bounded ABI/native spike](records/2026-10-25-exp-upscaler-callback-allocation.md).
Accepted [D040](../adr/D040.md) implements faithful ABI and a typed unsafe wrapper
interface for both families, with separate debug controls. Open module-global registration persists
after context teardown and omitted DX12 callbacks do not reset it. Safe closures,
automatic reclamation, recoverable OOM and effect-isolated messages are therefore
not established contracts. D040 explicitly retains wrapper dependencies on
callback-bearing native create errors while preserving D005's ordinary path. The spike's 17 stages,
10 contexts and 16 fenced frames resolve the tested CB aggregate ABI and observe
resource/CB replacement on both providers plus 3.1.5 message survival after
installing-context destruction. Signed-loader handling qualifies the open-source
ignored-effect/deprecated-route model. Separate [public-wrapper verification](../release/2026-10-25-callbacks-diagnostics-verification.md) now records 12 contexts/20 fenced frames, actual message delivery and legacy
live-context code 0 on both providers; null-context legacy remains code 2.
Heap SDK invocation and general shutdown/failure/threading guarantees remain
unverified. Controlled ABI/mapping covers the exposed heap pair; exact final
release archives remain separate.

The [complete-upscaling intake](upscaling-coverage.md) integrates the
[release-surface inventory](records/2026-10-18-src-upscaling-release-surface-inventory.md)
and [Configure study](records/2026-10-15-src-upscaler-configure-contract.md).
At intake it corroborated missing provider/query/configuration, diagnostic and
allocator routes, including retained optional chains. [D028](../adr/D028.md)
subsequently accepted the provider-selection contract; its
[implementation checks](../release/2026-10-18-provider-selection-verification.md)
pass host tests and Windows compilation, followed by the
[Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md).
[D029](../adr/D029.md) now accepts public Configure synchronization/failure
semantics and is implemented in development ([checks](../release/2026-10-18-upscaler-tuning-verification.md));
bounded Windows/native confirmation now passes. Upscaling callback contracts are accepted and implemented under D040.
The study's FSR2 silent-OK finding rules out treating acknowledgement as effective
support. Acceptance comes from the maintainer's D028/D029 decisions, not the
source findings or controlled fixtures.

**Current implementation evidence:** the [API coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
passes 108 local CI, 97 exact Rust 1.88 MSRV and 203 isolated development-package
cases. Both families' Windows code/tests and downstream consumers/examples
compile on current Rust and MSRV. Loaded host fixtures exercise actual wrapper
forwarding and terminal ownership, not an AMD provider. Independent MSVC and
Windows/native execution subsequently pass in the Windows follow-up: a 256-case
consolidated batch and 31-case native profile, including selected-provider dispatch
for both families and 20 tuning value/default-request → dispatch/fence frames
on 4.1.1. Numerical application, persistence and effective defaults remain
unproven. Development archive checks do not identify final release inputs or
complete a published-package API comparison.

**Observed acknowledgement limits:** the
[reactive-mask experiment](records/2026-10-19-exp-reactive-mask-providers.md) adds
native evidence for D030. Both public families/all flags produce masks on 3.1.5;
4.1.1 and 2.3.4 return OK but leave output seeded. Their complete experimental
profiles remain failed despite successful submission, upscale output and cleanup.
The [current archive-bound repetition](../release/2026-10-29-candidate-verification.md#generator-outcomes-retained-as-failures)
again verifies 3.1.5 for both families/all flags, and observes no writes on the
current default 4.1.2 * and explicit 2.3.4. All 68 failing generation cases retain
native success/cleanup observations and failed write criteria; historical 4.1.1
results are not attributed to that newer default-provider identity.
This is generation evidence, distinct from the Configure study's source finding
that open FSR2 ignores tuning. Neither optional operation acquires an `Applied`
guarantee, generic support query or provider allow-list from these results.
Successful cleanup supplies bounded observations rather than native error
atomicity or universal destroyability. See [dispatch](upscaling-dispatch.md#standalone-reactive-masks-provider-specific-execution).

**Variable-output access limits, integrated 2026-10-19:** the
[raw native spike](records/2026-10-19-exp-variable-output-resolution.md) passes
71 integration cases on the recorded Windows/DX12 stack. 4.1.1/3.1.5 vary written
footprints but write rounded padding; 2.3.4 retains maximum output despite a
smaller request. Thus OK does not certify that requested geometry was applied.
Actual resource metadata and coverage of the requested image remain necessary
checks, but cannot alone certify provider access boundaries. Active-sized
readbacks cannot detect attempted out-of-resource stores; ordinary debug validation
does not settle that question. All contexts clean up successfully, with no native
error branch exercised. The [dispatch assessment](upscaling-dispatch.md#variable-output-native-follow-up-2026-10-19)
retains the allocation/provider/flag decisions needed before public implementation.
That raw evidence did not extend D024. Subsequent maintainer review accepted its
[maximum-allocation output amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19),
now implemented with [public-path verification](../release/2026-10-19-active-output-verification.md).
Actual-provider support/safe access remains explicit at unsafe dispatch; safe
frame setters/preflight add no hidden GPU-safety precondition or capability promise.
The public profile passes 62 cases, including 144 dispatched frames and 12 controls
across both owners on 4.1.1/3.1.5; the consolidated Windows profile passes 257 cases.
Controlled tests verify independent output bounds, preflight reinspection and
native-error poison precedence. The GPU runs observe only successful native
returns and cleanup. Neither their finite/written output nor matched owner/encoding
hashes proves bounded shader access, general motion consumption or error atomicity.
Only two Rustdoc replacements distinguish the recorded public GPU and later
consolidated source inventories. Package inspection is separate from final
extracted-package verification; broader allocations and generic provider support
remain outside the accepted profile.

**U05 source intake, 2026-10-20:** the
[formats/views and motion-scale record](records/2026-10-19-src-upscaling-formats-and-motion-scale.md)
reports that generic backend view creation depends on real allocation flags and
supplied native format, including UAV creation for UAV-capable inputs. Format
enum membership, resource-query bits and OK returns therefore certify neither
legal views nor provider support. Actual device/view requirements need review
when widening the current role profile; no new preflight is implemented here.
Explicit scale finiteness is a proposed input rule, not a safe numerical-domain
proof. Zero/negative-zero handling and the raw-scale API remain unaccepted.
Existing UV/pixel defaults, both owner families, allocation/aliasing rules and
renderer GPU duties stand. The [dispatch assessment](upscaling-dispatch.md#external-formats-and-explicit-motion-scales)
preserves the FSR2/FSR3 denominator difference, NDC source conflict and intake
provenance/citation limits. This source record adds no format/native support or
new proof of resource-access safety. These are the intake's historical limits;
the accepted implementation checkpoint follows.

**Accepted D034 slice, 2026-10-21:** both owners now share the optional native
scale override and RG32F main-motion checking under [D034](../adr/D034.md).
Finite signed bits, including zeros/extremes, are accepted literally; NaN/infinity
reject. RG32F alone queries actual device SRV/possible UAV-view support, after
role/shape/device checks, without treating feature bits as provider permission.
Optional preflight and mandatory dispatch agree; inspectable errors do not poison
and preserve HRESULTs. The unsafe caller retains actual-provider numerical,
format/view, content, backing-storage and GPU-completion obligations.
[Stable/MSRV/public verification](../release/2026-10-21-native-motion-verification.md)
passes including mixed-sign jitter, UAV-capable motion and active output.
Encoding equivalence does not prove native jitter cancellation or bounded access.
Other format families, native error-branch cleanup guarantees and final release
archives gain no new verification claim.

**Accepted conditional contract, amended by D042/U09 on 2026-10-27:** portable
`TuningRequest::new` now checks each parameter/value pair before native mutation.
Both families accept only `request_tuning(request)`, forwarding accepted values
unchanged or sending a null-default request. Prepare complete lists first; native
calls remain ordered and nontransactional. The earlier optional `checked_for`
constructor remains removed. These provider-independent wrapper input rules come from the shared
v2.3.0 header, not a claim that every provider implements them. Future widening
requires behavioural/native-trust review; it must not follow guessed provider
capabilities. No cached effective value or support predicate is introduced.
Native errors preserve their code and prevent dispatch, live queries
and further Configure, while normal terminal destruction remains available.
Pointer non-retention, parameter-domain/reset safety, profile compatibility and
post-error destroyability are actual-provider acquisition obligations. Prior
unsafe dispatch must establish GPU completion and no executable unsubmitted
recording before safe Configure. Neither `&mut self` nor acknowledgement proves
those conditions. The [dispatch](upscaling-dispatch.md),
[lifecycle](context-lifecycle.md) and [threading](runtime-threading.md) topics
state the corresponding boundaries; D040 now specifies externally owned callback
targets and conservative wrapper-bundle retention.

**Names distinguish promises:** successful `request_tuning` acknowledges a native request;
`set_*` changes local Rust options/frame data; `validate_upscale_inputs` checks
inspectable inputs and does not certify GPU state or provider support.
`render_size_for_quality` reports a provider's quality-derived size rather than
the context's current active size. Diagnostic errors retain the precise
`ConfigureDebug`, `ConfigureGlobalDebugLegacy` or `ConfigureContextDebugLegacy`
operation; tuning errors use `Operation::Tuning`.
Open FSR2's silent no-op and open FSR3's setter are source-supported differences;
closed-provider application remains unestablished. There is no generic per-key
capability query; trial OK codes and resource masks cannot justify an `Applied`
result.

### Published compatibility, recovery and extensibility

The 2026-10-18 findings below describe their reviewed implementation and prior
selection. The [D042 assessment above](#accepted-api-transition-and-setup-draft)
now owns the preferred replacement direction; it does not relabel these earlier
checks as verification of the replacement API. U07 production checks are linked
above; the remaining transition is still open.

The three 2026-10-18 reviews above compare the same supplied development ZIP
(SHA-256 `555f0357f7413a6400870a10c03f6c70dbef62c4618a17abdfec0a3a2e767366`)
with manually inspected `v0.2.0` source where applicable. The published baselines
are **`fsr-sdk 0.2.0` and `fsr-sdk-sys 0.1.1`**, released on 2026-10-10;
the development targets are independently `0.3.0` and, after the 2026-10-24 D031
amendment, `0.1.2`. The original sys `0.2.0` plan was unpublished and reversed. AMD remains
SDK v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`.

**Verified source findings, bounded to the reviewed surface:** existing owner,
frame/resource constructor, dispatch and consuming-destroy signatures remain.
Private fields, non-exhaustive diagnostics and separate local/movable ownership
already permit extension. The reviews identify no demonstrated need to replace
these signatures for earlier validation. This is not complete compatibility or
native soundness certification.

| Boundary | Current finding | Remaining qualification |
|---|---|---|
| Unsafe runtime acquisition | D018 and later decisions expand actual-provider Query/Configure/selection trust without changing `load_unchecked` signatures. | A safety-contract change needs explicit migration guidance; distinguish obligations for old default-only use from new optional operations. |
| Context creation | The added requirements query can reject native query failures or unsupported required inputs. | Potential behavioural incompatibility depends on provider responses; unchanged constructor syntax does not establish unchanged behaviour. |
| D028/D029/D030 | Provider queries, tuning and generation are post-0.2.0; D030 generation is accepted and locally implemented as of 2026-10-19. | Revising them affects development users and accepted decisions, even though it need not break published 0.2.0 source. |
| `vulkan` feature | The reviewed snapshot retained a deliberately failing marker. The 2026-10-24 [D031 amendment](../adr/D031.md) restores that published feature contract for wrapper `0.3.0`, reversing its unreleased removal. | Vulkan alone and all-features builds still deliberately fail. Initial usable upscaling is targeted at `0.4.0`, subject to a usable official SDK/runtime and native evidence; Windows/DX12 encapsulation remains mandatory. |
| Raw bindings | Sys returns to `0.1.2`; wrapper `0.3.0` depends on it. The feature-removal sys `0.2.0` bump is reversed. No native ABI/SDK version change follows. | Existing declarations/loader contracts are retained and new bindings are additive; final package compatibility and verification remain release-closure work. |

**Recovery contract:** compulsory inspection already runs before native dispatch.
On a usable context, Rust validation failure leaves eligibility unchanged;
native dispatch/Configure non-OK makes live operations terminal and preserves
the operation/code. Existing poison is checked first. A successfully recorded
but abandoned dispatch requires the renderer to call
`mark_recording_abandoned()` on every affected context. The wrapper records
`TerminalCause::RecordingAbandoned` locally, retaining any earlier native cause;
`usability()` is a pure snapshot and does not establish GPU readiness. Recovery requires invalidating affected
unsubmitted recordings, completing earlier GPU work and destroying/recreating
affected contexts. A Configure failure has no failed command list of its own.
Neither a return-code name nor a history reset proves native rollback.

Configure before first dispatch needs no earlier-work drain. Later Configure
requires completed submitted work and no executable unsubmitted recording,
under the previous unsafe dispatch contract. CPU exclusivity and a fence number
alone cannot establish this. Multiple Configure calls are not an atomic batch.
Provider enumeration is likewise an observation, not a reservation or a
generator-support certificate. Enumeration/inspection costs and the cost of idle
Configure were not measured by these reviews.

**Disagreement in recommendations:** the first review proposes optional checked
tuning, preparation and staged/session helpers. The later two instead prefer
mandatory internal validation and one canonical public route, under their
explicit coherence criterion. They favour strict per-parameter tuning rejection
for early diagnosis, while recognizing finite passthrough as another coherent
choice. This is a design-preference difference, not contradictory runtime
evidence. The subsequent D029 API coherence amendment selects strict tuning
checks; D028 retains additive provider helpers with fresh checks. D016 remains
binding. The historical source reviews are unchanged.

**Subsequent maintainer selection:** [D032](../adr/D032.md) originally selected
optional setup/value checks. Its implemented API coherence amendment removes
`TuningValue::checked_for` in favour of mandatory request-boundary checks; both families' renamed
`validate_upscale_inputs` reuse obligatory inspection without FSR calls or
eligibility changes. [Recovery workflows](../DX12_RECOVERY.md) and
an opt-in startup-cost profile address the other findings within existing safety
bounds. D030 was subsequently accepted and implemented locally on 2026-10-19
([checks](../release/2026-10-19-reactive-mask-generation-verification.md)). Windows
fixtures and bounded native execution now follow in the verification above.
The [setup-query measurement](records/2026-10-19-exp-provider-setup-cost.md) supplies
192 whole-path samples across three processes, without isolating enumeration
or selecting caching. No rollback/in-flight Configure rule follows.

**Extensibility inference:** retain effect-specific owners and backend-specific
resource/recording contracts. The reviewed Frame Generation header introduces
callbacks, presentation and asynchronous work requiring a separate lifecycle
audit. It does not justify a generic all-effect dispatch facade or automatic
reuse of the upscaler's teardown/threading proof. Shared scalar types could
retain old paths through reexports if a second implementation establishes
identical invariants. Prepared resources or recording tokens cannot prove GPU
states, submission and completion without controlling those relationships.

**Earlier documentation discrepancies:** the source reviews found stale mask
and provider-selection Rustdoc; D032 corrected those descriptions. The later
coherence checkpoint also corrects method names and tuning-domain documentation.
The historical findings remain unchanged and do not describe the current mask
filter. The retained `UnsupportedResource` diagnostic was a separate review
suggestion, not a demonstrated reason to redesign published dispatch.

**Open evidence:** comparisons against each published baseline, including
symbols, default descriptors and unsafe obligations, remain separate from the
passed current-source/MSRV and development-package checks. The Windows follow-up
adds bounded native evidence; final extracted-release verification remains open.
Error-code-specific rollback, abandonment reuse, in-flight Configure and broader
D030 support/state restoration still need pinned evidence before relaxing contracts.
D030's implemented role-aware aliasing and shader-readable/UAV-output validation
pass Windows fixtures; provider-specific write results do not replace them. These are
integration/safety questions; image-quality research remains deferred by scope.
Generation verification is required for a release shipping generation, not by
D030's existence alone; the current complete-upscaling scope still includes U01.

Intake corroborated current policy, manifests and the documentation discrepancies;
it did not revalidate the supplied archive, fetch the published packages or repeat
the external-source investigation. The three records retain their original
observations. No compatibility tool, compiler, native provider or GPU experiment
ran during this synthesis, and no design or release status changed at intake.
D031 and the subsequent coherence amendments are separate maintainer decisions.
This synthesis reconciles those accepted amendments and their completed local
checks; it makes no new design decision or native verification claim.

### Common ABI baseline

**Verified source facts:** `ffx_api.h` supplies the original common boundary:
context and scalar aliases, eight return constants, the base descriptor header
and four aliases, allocation callbacks and the five entry-point pointer types.
Resource and extended descriptors are not required to declare those signatures.
Context parameters are pointers to handles; allocation sizes and descriptor tags
are 64-bit unsigned integers. Create/query descriptor pointers are mutable;
configure/dispatch descriptors and allocation-callback struct parameters are const.
See the record's header findings and pinned local-header fingerprint.

**Verified test results:** Rust checks and a separate native C++ compile passed
the shared layout, constant and signature expectations. On the tested target the
base header is 16 bytes/alignment 8 and the allocation-callback struct is 24
bytes/alignment 8. The C++ file is a compile-only verification probe, not part of
the library or its Cargo build. Reproduction commands and setup failures are
preserved in the record.

**Upstream contracts:** pointers passed in creation descriptors must remain live
through destruction (the lifetime of chain-node storage is ambiguous; see below); allocation/deallocation have alignment, null and compatibility
obligations. Global query/configure allow a null context pointer. These comments
are evidence of the declared contract, not observed runtime behaviour. Nullable
Rust function-pointer representation does not authorize passing null callbacks.

**Inference:** the paired checks support the selected Rust ABI mapping on Windows
x64/MSVC. They compare each language against shared expectations, not directly
against each other through a call. They do not establish a safe wrapper.

**Verified loader results:** the local `amd_fidelityfx_loader_dx12.dll` loaded
and all five API exports resolved, without invoking an AMD API function. Its
tested hash is in the loader record. The SDK documentation identifies it as the
entry DLL managing separate effect providers. The subsequent query experiment
adds evidence for one upscaler enumeration path.

**Verified query result:** one count-only `ffxQueryDescGetVersions` invocation
with null context/device returned OK and count 2. This requires one additional
56-byte descriptor and the version-query/upscaler-selector tags; paired ABI checks
cover fields, layout and constants against local headers. No name or ID arrays
were requested, so the count does not identify the providers or GPU compatibility.

**Deployment observation:** the initial call using the original absolute SDK
loader path returned NO_PROVIDER. Staging the test executable with the loader
and upscaler DLLs returned OK. This supports executable-adjacent deployment for
this test, not a comprehensive DLL search policy. No production loader change
or search-path mutation was made. Exact commands and both results are preserved
in the query record. The SDK's C example omits the descriptor tag; the probe sets
it explicitly according to the header and implementation.

**Verified ownership and failure checks:** `FfxLibrary` owns private typed
pointers and the library. Five unsafe forwarding methods borrow that owner;
there is no raw accessor. Fixture tests cover missing files, invalid images,
each missing export, release after partial resolution, forwarding and release
on drop. Compile-fail tests cover borrowing and private pointer access.

**Verified D013 compile/fixture results:** a sealed Rc/Arc retention parameter
works through the real private context owner, with serialized macOS ARM64 and
Windows x64/MSVC fixture transfers preserving terminal cleanup and runtime
retention. All eight D013 probes execute successfully on both platforms;
Windows additionally verifies fixture DLL residency through dependency cleanup
and release afterward. Compilation confirms the trait assumptions; public local
types remain `!Send + !Sync`.
This adds no AMD transfer or concurrency guarantee; see
[the bounded threading findings](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles).

**Verified bounded D013 native results:** the September spike passed three
serialized, fenced AMD transfer scenarios and same-thread controls on RX 9060 XT,
driver `32.0.31041.1004`, SDK v2.3.0/provider 4.1.1. The October follow-up passes
all 11 prerequisites on its changed source, closing the native regression gap
after dispatch-policy extraction, then all 30 new children. These add creation
after runtime transfer, 64-frame continuous history, independently held siblings
and actual automatic native cleanup during Rust unwind. All 516 new frames and
42 destroy calls pass, including 12 unwind cleanups; 96 selected packed RGBA16F
payloads match controls/repetitions by SHA-256. Unselected frames have metric/
status checks rather than full image-equivalence evidence.

The provider remains loaded after all audited application frame/device/helper
refs are released and workers exit; the loader and helper are absent. This does
not attribute native retention, prove actual COM device destruction or establish
provider termination on the destination thread. See
[extended native threading results and remaining gates](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency).
D013's conditional design is now accepted; the existing public types remain
`!Send + !Sync`. This experiment implemented no public movable API and establishes
no universal AMD threading guarantee.

**Verified D013 failure-fixture results:** 14 control/transfer owner children and
six worker acquisition failures pass on Windows x64/MSVC. Shared dispatch policy
preserves known/unknown errors, poison, preflight rejection/recovery and terminal
no-retry retention; coordinated ready/poisoned unwind preserves sibling usability
and final fixture release. All 39 recorded source fingerprints matched at the
2026-10-07 intake.
Stable/MSRV regressions pass after preserved sandbox metadata-write failures;
AMD tests remain ignored. The injected inspection error and device surrogate
are not transferred real COM operations. See
[failure-fixture coverage](runtime-threading.md#d013-transferred-failure-state-and-coordinated-fixture-cleanup).

**Source intake limit and policy tension:** the source report could not inspect
the requested pinned implementation and establishes no affirmative AMD transfer
guarantee. Its platform sources were not independently revalidated during intake.
The native follow-up supplies its own preregistered matrix and does not complete
the missing pinned implementation audit. Loader residency is distinct from
provider termination/thread attribution. Its ambiguous-create-error quarantine
recommendation supplies no new contrary observation to D005's bounded returned-
error RAII policy and does not replace it. Native returned-error cleanup remains
unverified; see [source intake](runtime-threading.md#d013-transfer-contract-source-intake).

**D013 review recommendation:** the
[supplied soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md)
finds no confirmed Rust soundness defect and recommends acceptance with revisions.
Unsafe acquisition must support every subsequent destination thread and maintain
shared-state coordination through safe calls, implicit Drop/unwind and all derived
contexts. A concrete upscaler field audit and narrow Send implementation, explicit
runtime `!Sync` marker and separate local/movable families remain necessary.
The review considers further exploratory native experiments unnecessary before
implementation if those caller assumptions are accepted; this is a design judgement,
not new native evidence. Its date, author and source revision were not supplied;
2026-10-09 is the intake date. Existing D005/D008/D011 policies remain intact.
[D013](../adr/D013.md#acceptance-rationale-and-remaining-assumptions) was separately
accepted on 2026-10-09 with the revisions and renderer-state migration rationale.
No design soundness blocker remains. The subsequent optional public implementation
and [bounded verification](../ROADMAP_HISTORY.md#movable-implementation-checkpoint)
record the actual field audit, traits/features/family boundaries, shared failure
fixtures and coordinated native cleanup separately from this review.
The [ADR safety contract](../adr/D013.md#conditional-safety-contract)
owns the requirements; see [review synthesis](runtime-threading.md#d013-soundness-and-architecture-review).

**Verified bounded D014 sharpening results:** the
[native spike](records/2026-09-30-exp-d014-native-sharpening.md) changes only the
already-bound native sharpening fields in test code after shared inspection,
through the existing context owner and dispatch-poisoning core. On SDK
v2.3.0/provider 4.1.1, RX 9060 XT and driver `32.0.31041.1004`, two complete
three-repetition runs submitted/fenced 480 finite outputs with clean ordinary
debug/device checks and explicit teardown. Disabled repeat controls were
bit-identical. Enabled zero differed from disabled in both stationary scenes;
zero strength cannot promise pass disablement or identity output.

The proposed frequency-scene effect gate failed in every repetition: strength
0.35 increased horizontal gradient by only 0.584% against a 1% requirement;
strength 1 shifted mean red by 0.01952 against a 0.005 bound. Independent half
decoding and metric computation confirmed the results. Edge overshoot is a
diagnostic observation, not a quality guarantee. See the
[dispatch synthesis](upscaling-dispatch.md#d014-sharpening-native-execution-and-failed-effect-gate)
for the original effect failures and scope limits. [D014](../adr/D014.md) was
accepted on 2026-10-07 as an API/migration and bounded native-pass trust decision.
Current source implements its public frame and native sharpening API under
[M8](../ROADMAP_HISTORY.md#frame-and-sharpening-integration), whose controlled
FFI and consumer checks cover scalar validation, exact descriptor forwarding,
same-context switching, traits and existing error paths. The separate
[M9 integration protocol](../release/2026-10-10-m9-verification.md#integration-protocol)
was subsequently executed in
[2026-10-10 verification](../release/2026-10-10-m9-verification.md). Development
source and the unmodified extracted wrapper `0.2.0` candidate each passed three
isolated repetitions, 144 finite/written/fenced outputs, same-context mode changes
with continuing history, zero ordinary debug/device errors and explicit cleanup.
Native regressions, exact candidate archives, registry-only consumers, Rust 1.88,
Rustdoc and publication dry-run checks also passed; the candidate remains unpublished
and excludes D013's public movable family.

The historical spike supplies private native execution/effect observations;
M8 fixtures establish exact validation, mapping and error behaviour; M9 adds
bounded public-API AMD/GPU and extracted-artefact evidence. Independently audited
M9 diagnostics again show enabled-zero differences, and return to Disabled matched
the control ROI on this setup. Neither observation is a universal output guarantee.
No new effect threshold was adopted; the original failed effect gate is unchanged.
Successful native paths establish no general failure cleanup contract, safe GPU
lifetimes, signed-provider allocation capacity or broader hardware support. See
[the M9 dispatch findings](upscaling-dispatch.md#m9-public-sharpening-integration-and-candidate-verification).

**Safety boundary:** explicit input paths are canonicalized before loading the
main DLL. Dependency resolution still follows Windows loading rules. Loading
requires trusted initialization/termination and ABI-compatible exports; no
authenticity check occurs. Native contexts and work may outlive individual calls,
so the raw caller must preserve library and resource lifetimes and synchronization.
This bounded design is authorized by
[D004](../DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points).

## Older Rust bindings: useful patterns and limits

Both comparisons concern standalone FSR2 2.2.0, whose caller-allocated opaque
context storage differs from v2.3.0's pointer handle. They inform wrapper design,
not the modern SDK's native contracts.

| Concern | Embark `fsr-rs` | NotAPenguin0 `fsr2-rs` |
|---|---|---|
| Ownership | Non-copyable high-level context retains interface and scratch storage. | Thin raw layer exposes copyable opaque context storage; no owner wrapper. |
| Teardown | Unsafe `destroy(&mut self)` neither consumes nor invalidates the object; no context `Drop`. | Free create/destroy functions leave all live/dead state to callers. |
| Failure handling | Returns create errors without wrapper rollback; DX12 interface initialization discards its native result. | Exposes native errors directly; partial-create cleanup remains unresolved. |
| ABI | Generated raw bindings still permit copying native state. | Handwritten declarations are auditable, but closed error enums, callback nullability/unsafe typing and enum widths need scrutiny. |
| Threading | Explicit interface/scratch `Send + Sync` propagate to context without an identified native guarantee. | Opaque storage gains auto traits; descriptor has explicit `Send`, without establishing native concurrency safety. |

Sources: [Embark report](records/2026-09-20-src-embark-fsr-rs.md), pinned to
`2ec24b8abae1d6013d444cd472fa9712c584a21b`, and
[fsr2-rs report](records/2026-09-20-src-fsr2-rs.md), supplemented by the
[provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md):
parent `3a928595f46818f7221f0ca28f00770b53ded850` is a high-confidence historical
reconstruction; its native gitlink is `35d136728c49b5c866517b906d4405b7bee583da`.

**Inference for this project:** retain backing storage and native dependencies
with the context owner, prevent duplicate ownership and post-destroy use, preserve
native errors, and audit callback types and layouts against the pinned ABI.
A constructor borrow alone does not encode a context-long dependency. Neither
comparison supplies a sound ready-made destruction/failure policy. Embark's old
DX12 backend `AddRef`/`Release` behaviour does not resolve modern v2.3.0 device
ownership; the old Vulkan paths likewise establish no new backend support here.
These are design inputs, not accepted API or binding-strategy decisions.

## Disagreements and limitations

No discrepancy was observed within the experimentally checked ABI/query subset.
The lifecycle source report identifies documentation conflicts about the required
version node, post-destroy nulling, and chain-node lifetime. Runtime evidence now
includes successful context lifecycles, host-allocation injection crashes and
one returned-error input-lifetime experiment;
it confirms non-nulling but leaves chain-node lifetime unresolved. The documented
permission to return null from allocation contrasts with crashes on the tested
path; the separate duplicate-backend rejection returned normally and caller
cleanup caused no observed fault within two seconds. This does not establish a
general failure cleanup contract. Those ABI/query/lifecycle and failure records
did not test upscaling dispatch or image output; the separate M6 and final
Windows records supply the bounded GPU evidence described above. No 32-bit or
other-platform claim follows. The local header was
fingerprinted but its correspondence to the upstream release tree was not
independently authenticated in this work.

[D003](../DECISIONS.md#d003--handwrite-the-first-common-abi-slice) authorizes
handwritten bindings only for this slice; successful tests alone did not settle
the broader binding strategy. [D006](../adr/D006.md) subsequently selected the
handwritten-by-default policy with reviewed generated exceptions; concrete
additional slices remain undecided. The Rust module is production implementation;
the native probe is verification evidence. Evidence records preserve the state
at investigation time and do not independently accept architecture changes.

The ABI checks themselves offer no licensing conclusion. Current notice,
acquisition and signed-runtime distribution findings are maintained in the
[licensing and distribution synthesis](licensing-and-distribution.md), including
the distinction between source-built components and the supplied signed runtime.

**Provenance refinement:** the
[repair record](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md)
supplies full-commit AMD citations at
`60f4ea81909200d8542eca14dccb2628b763a9a3` for the older lifecycle record's
missing URLs. Its two already populated URLs are locally confirmed as v2.3.0
links. The repair reports verification of the fsr2-rs parent's native gitlink;
the parent's identity on 2026-09-20 remains a high-confidence inference from
matching 36-commit history, not a dated branch snapshot. Representative pinned
sources support the earlier findings without a reported substantive discrepancy.
The repair did not have the full historical records and is not a complete prose
re-audit. Original records remain historical; the supplementary record carries
the corrections. Neither repaired citations nor open source resolve the signed
provider's implementation or establish runtime behaviour.

## M10: selected DX12 Query safety

The [2026-10-12 source review](records/2026-10-12-src-m10-dx12-query-safety.md)
examines provider enumeration, pre-create memory estimation and three live
queries: provider metadata, resource requirements and GPU memory usage, for
both ownership families. It uses the same pinned SDK v2.3.0 commit and reports
direct inspection of AMD headers, routing, fallback, FSR3 and DX12 sources.

**Intake and provenance:** the new record was renamed from
`2026-10-12-src-m10-dx12-query-safety-EN.md` without changing its contents.
Its ZIP identity/hash and primary-source retrieval are reported provenance;
intake did not independently revalidate that archive or replay the upstream
investigation. At intake, source agreed with the reported implementation gaps:
`query.rs` contained only a licence header, native query forwarding was test-only,
the selected additional ABI declarations were absent and `Operation::Query`
did not exist. Intake performed no builds, native calls or GPU experiments.

**Reported source findings:** enumeration uses a reused external-name buffer
and requests IDs and names together. Names must be copied while shared CPU
coordination remains held. The inspected open paths consume query outputs
synchronously; FSR3 live memory reads CPU backend accounting, V2 asks the device
for allocation sizes, and resource requirements are CPU-side masks. These paths
record no GPU dispatch. Closed-provider name validity, pointer non-retention,
error consequences and compatibility with in-flight GPU work remain unresolved.
Immediate copying and `&mut self` alone do not establish those properties.

| Proposed method, including movable | Wrapper obligations and remaining native assumption |
|---|---|
| Runtime provider enumeration | Checked, initialized count/fill storage; copy paired names/IDs under coordination; define truncation/completeness policy. Native names must be readable and terminated, with no query-pointer retention. |
| Runtime pre-create memory estimate | Correct V2 device/options descriptor and initialized output; preserve native errors. Query-specific numerical/device/pointer safety needs explicit trust; D008's construction assumption does not automatically cover this path. |
| Upscaler provider metadata | Retain context/runtime; reject successful zero ID or null name before copying. External names need a valid copying window. |
| Upscaler resource requirements | Check return code before reading masks; retain both raw `u64` masks, including unknown bits. Closed-provider live-state/GPU compatibility remains an assumption. |
| Upscaler GPU memory usage | Retain context/backend and copy initialized accounting output. Closed-provider synchronization with GPU work in flight is the review's strongest live-query concern. |

**Design assessment, not acceptance:** the review finds all five methods
conditionally defensible as safe operations following unsafe acquisition,
provided a deliberate contract extension covers query-specific name validity,
non-retention, numerical safety, continuation after ordinary query errors and
the permitted GPU states. Existing D008/D013 trust does not silently accept
those additional obligations. GPU-quiescent measurement is a possible
integration restriction; it does not establish arbitrary-provider safety.
The review supplies no unavoidable blocker merely from missing documentation,
and no universal safe-query guarantee. At intake, M10 remained subject to contract review.
If the additional trust is rejected, the review offers unsafe query methods or
deferring individual methods as alternatives; neither is selected by this intake.

**Recommended error/eligibility policy:** preserve every non-OK code through
`Error::Native` with a Query operation, and interpret outputs only after OK.
Successful null live names or zero live IDs are invariant failures; valid zero
memory values and unknown resource bits must not be rejected without a supported
semantic rule. Error-path all-ones resource masks are not valid results. The
review recommends rejecting all three live queries before FFI after dispatch
poison, while keeping runtime queries separate. Ordinary query errors should
preserve dispatch eligibility only under an accepted continuation assumption;
queries must never clear dispatch poison. These are proposed policies, not
implemented behaviour or amendments to D011.

The review also reports a suspicious null-context fallback comparison and a V2
shared-resource loop indexing the other resource array. Those are scoped open
source findings, with no reproduced signed-provider fault or new wrapper
soundness finding. [Construction/planning](upscaler-construction.md#m10-provider-discovery-and-memory-planning)
details the enumeration and estimate limits;
[threading](runtime-threading.md#m10-query-coordination) covers copying and GPU
coordination. Further native checks could reveal configuration-specific faults,
but cannot prove universal name validity, non-retention or synchronization.

**Implementation follow-up:** [D018](../adr/D018.md) accepts the five safe methods
under explicit query-specific acquisition trust, bounded enumeration and the
error/poison policy above. Unreleased source implements both families through
shared private functions. The later
[Windows checkpoint](../release/2026-10-12-windows-m10-sdk-verification.md) passes
MSVC ABI, controlled/consumer/MSRV and native query/GPU checks on one configuration.
M10 is selected for [0.3.0](../release/0.3.0-preparation.md); final package checks
and publication remain open. Neither the source review nor the bounded native
checks prove universal closed-provider guarantees.

## Open questions

The later [M6 dispatch synthesis](upscaling-dispatch.md) collects source
findings about DX12 states and synchronization and tracks accepted D011, M6b's
public-route proof and the final Windows regression/three-frame smoke. The new
post-dispatch teardown observations complement this topic's historical M5
lifecycle results; their native failure and ownership limits remain unchanged.

- **M5 evidence limits and future API work:** the later provenance repair supplies
  pinned source links and corrects broad earlier claims, but does not recover
  every original claim-to-source mapping. Zero components aborted on one signed
  configuration; a later three-width probe returned generic error `1` below,
  at and above the D3D12 Texture2D width limit without identifying the rejection
  cause or an accepted maximum. The numerical source investigation and these
  probes establish no complete safe domain. Accepted
  [D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
  instead adopts a bounded runtime trust assumption, and M5 implements the
  public constructor. Ordinary Windows CI fixtures for that path passed in the
  [recorded run](records/2026-09-24-exp-m5-windows-ci-verification.md); both
  opt-in native lifecycle cases passed on the tested configuration
  ([native record](records/2026-09-24-exp-m5-windows-native-verification.md)). Resolve
  provider-specific helper rounding, output retention and global debug scoping
  where later public operations require them. The two-context sequential result
  does not establish general native concurrency safety.
- **D013 native transfer:** the [compile/host findings](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles)
  resolve the Rust trait questions for the tested shapes. The
  [September native spike](runtime-threading.md#d013-fenced-native-transfers-and-final-release)
  passed three fenced transfer scenarios, controls and then-current native
  regressions; the [October fixtures](runtime-threading.md#d013-transferred-failure-state-and-coordinated-fixture-cleanup)
  add transferred poison, exact preflight, terminal-retention and coordinated
  unwind coverage. The [extended native follow-up](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency)
  closes the extraction's native regression gap and adds creation after runtime
  transfer, longer history, independent siblings and successful fenced Rust-unwind
  cleanup. Native returned-error cleanup, partial-load cleanup, provider/
  dependency termination, hidden work, external-user coordination, outstanding
  GPU work, other configurations and sanitizer evidence remain open.
  The access-limited source intake establishes
  no universal transfer guarantee. Concrete renderer-state migration and the
  lasting caller-contract review informed D013 acceptance. The
  [new review](runtime-threading.md#d013-soundness-and-architecture-review)
  recommends accepting strengthened unsafe assumptions without further exploratory
  execution; the native uncertainties remain unresolved. Acquisition failures
  occur before publication of a movable owner, rather than transferring incomplete
  state. The subsequent [public-path verification](../ROADMAP_HISTORY.md#movable-implementation-checkpoint)
  checks the actual optional API, consumer traits/features/families, coordinated
  automatic cleanup and selected fenced native regressions; it resolves none of
  the missing native guarantees.
- **Policy selected, broader questions open:** D007 selects explicit acquisition
  and executable-adjacent upscaler deployment for the tested v2.3.0 DX12 baseline;
  automatic runtime discovery is not part of that policy. Other effect artefacts,
  runtime coexistence and dependency closure remain open. Distribution terms
  remain separate from the accepted acquisition/deployment policy.
- **Provenance:** exact historical fsr2-rs branch identity remains inferred;
  the repaired source set is not a complete audit of every historical claim.
- **Source clarification and design:** chain-node lifetime, signed/external 4.1.1
  device COM ownership,
  callback/threading guarantees, and universal failure-state contracts. D005 now
  selects failed-create cleanup and exceptional dependency retention under its
  explicit runtime assumption; these remaining research gaps do not reopen that
  M4 policy by themselves. New contradictory evidence requires policy review.
- **Follow-up investigation after authorization:** locate the observed allocation-
  failure faults and distinguish loader/provider/driver behaviour. Establish a
  deeper returned-failure path or obtain provider-specific lifetime evidence;
  the duplicate-backend spike now covers one returned error and bounded caller
  cleanup, but not a universal contract. Replacement allocator compatibility remains
  untested. For destroy, first establish a documented legitimate failure trigger
  or obtain provider-specific evidence: the new source report found no supported
  valid-context trigger, and a null outer pointer cannot answer liveness. Do not
  corrupt private state to manufacture one, blindly destroy failed-create output,
  or retry failed destruction without a contract.
  Check library/context lifetime invariants without deliberate use-after-free.
- **Later source work and experiments:** result-array queries and broader GPU
  resource, temporal and synchronization behaviour remain open. The separate
  [dispatch topic](upscaling-dispatch.md) includes the public-route regression
  and completed three-frame moving smoke. Neither those bounded results nor
  recording-ahead tests establish general threading, synchronization or
  image-quality guarantees. Temporal reconstruction accuracy remains open.
- **D014 coverage and remaining limits:** the
  [sharpening experiment](records/2026-09-30-exp-d014-native-sharpening.md)
  resolves whether the selected enabled passes execute and whether enabled zero
  can differ from disabled, but its proposed effect criteria failed. The selected
  [M9 integration protocol](../release/2026-10-10-m9-verification.md#integration-protocol)
  requires correct native control and completed GPU execution through the final
  public API, without additional sharpening image-effect criteria. The original
  failed measurements remain unchanged; they are not an observed native safety
  failure. D014's public API is implemented and checked under
  [M8](../ROADMAP_HISTORY.md#frame-and-sharpening-integration), with the selected
  public-API GPU protocol and local candidate-artefact checks passed in
  [M9](../release/2026-10-10-m9-verification.md). Existing D011 lifetime, ordering
  and terminal-error obligations remain in force; HDR, exposure modes, GBV,
  moving-scene sharpening quality and other providers gained no new coverage.
  Publication is separate from verification and remains pending.
- **Policy selected, implementation deferred:**
  [D006](../DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)
  establishes handwritten bindings as the default and permits reviewed generated
  exceptions with equivalent ABI verification, including for first-class support.
  Concrete additional slices and any generation setup remain to be selected.
