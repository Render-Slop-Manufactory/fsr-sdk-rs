# Upscaling API coverage

- Updated: 2026-10-29 (common rc1 cross-host qualification integrated; performance-input reconciliation accepted; publication simulation passes; release closeout remains).
- Scope: the complete public upscaling surface of SDK v2.3.0 at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`, Windows x64/MSVC + DX12,
  for the selected `fsr-sdk 0.3.0` release target. The upscaler API version is
  separately 4.1.1; the raw development package is `fsr-sdk-sys 0.1.2` under
  the 2026-10-24 [D031 amendment](../adr/D031.md).
- Included records: the [release-surface inventory](records/2026-10-18-src-upscaling-release-surface-inventory.md)
  and its prerequisite [five-control Configure study](records/2026-10-15-src-upscaler-configure-contract.md).
  Both are source investigations. Later implementation evidence is supplied by
  the [provider-selection checks](../release/2026-10-18-provider-selection-verification.md)
  and [typed Configure checks](../release/2026-10-18-upscaler-tuning-verification.md),
  followed by [API coherence verification](../release/2026-10-18-api-coherence-verification.md).
  Integrated on 2026-10-18; these local checkpoints add no Windows/native execution.
  The [variable-output investigation](records/2026-10-19-src-variable-output-resolution.md)
  is integrated on 2026-10-19 in the [dispatch assessment](upscaling-dispatch.md#variable-output-resolution-source-contract).
  Subsequent executed evidence comes from the [Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md),
  [reactive-mask provider experiment](records/2026-10-19-exp-reactive-mask-providers.md)
  and [setup-query measurement](records/2026-10-19-exp-provider-setup-cost.md),
  followed by the [variable-output native spike](records/2026-10-19-exp-variable-output-resolution.md)
  and [public active-output implementation verification](../release/2026-10-19-active-output-verification.md).
  The [U05 formats/views and motion-scale source record](records/2026-10-19-src-upscaling-formats-and-motion-scale.md)
  is integrated on 2026-10-20, without new implementation or native execution.

## Current findings

**Current candidate qualification, 2026-10-29:** the
[consolidated verification](../release/2026-10-29-candidate-verification.md)
qualifies the same frozen rc1 pair on Windows **1,448/1,448**, Deck **16/16
offline + 17/17 timeout** and Machine **12/12**. Windows covers exact-package
consumers, callbacks/production CB ABI, both-family examples, optional `utils`
and selected advanced-native paths. The earlier Windows/Linux archive mismatch
is historical; the three new reports agree on both rc1 hashes. P08 input/impact
reconciliation is [accepted](../release/2026-10-29-rc1-performance-acceptance.md); publication simulation passes; release closeout remains. These are bounded
integration results, not universal provider capability.
The earlier generator repetition confirms 3.1.5 writes for both families/all
flags, while 4.1.2 */2.3.4 acknowledge without writes and their profiles remain
failed. Historical 4.1.1 no-write evidence retains its original identity.

**API transition selected, U07–U12 implemented:** [D042](../adr/D042.md) adds the
U07–U12 caller-facing transition to the complete native-upscaling release target.
The [review/setup synthesis](api-and-safety.md#accepted-api-transition-and-setup-draft)
reconciles both source audits, the [accepted contract](../design/upscaler-setup.md)
and the [stable/MSRV controlled spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md).
The latter verifies selected type/lifetime and controlled integration properties,
not new SDK-surface coverage or AMD capability. U07 was approved and implemented
on 2026-10-27; its [verification](../release/2026-10-27-upscaler-setup-verification.md)
covers the production setup path. U08 adds complete portable recording composition;
[verification](../release/2026-10-27-frame-composition-verification.md) records its checks. U09 adds canonical
checked tuning requests; [verification](../release/2026-10-27-tuning-request-verification.md)
passes compiler, controlled/consumer/ABI and native integration checks. U10 supplies
structured errors with original sources/first native causes and public provider/
mask data composition; [verification](../release/2026-10-27-error-metadata-verification.md)
records source/native bounds. U11 adds pure usability and local abandonment;
[verification](../release/2026-10-27-context-usability-verification.md) records its
both-family checks. U12 adds integrated consumer workflows, a consistency pass
and [migration guidance](../MIGRATING_0_3.md); its
[verification](../release/2026-10-27-api-transition-verification.md) records the
source checkpoint and remaining candidate/archive coverage obligations. The existing feature checkpoints below retain
their original API/source identities and provider limits. Current archive-native
coverage is now recorded above; final performance reconciliation is accepted and publication simulation passes, with closeout tracked in
[release preparation](../release/0.3.0-preparation.md).

**R03/U04 research integrated, 2026-10-25:** the
[callback/allocation synthesis](callbacks-and-allocation.md) combines the
[SDK contract investigation](records/2026-10-25-src-upscaler-callback-allocation-contracts.md)
and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md),
now followed by the [ABI/native spike](records/2026-10-25-exp-upscaler-callback-allocation.md).
Its final 17 stages, 10 contexts and 16 fenced frames pass on selected 3.1.5/4.1.1;
current/MSRV aggregate exchange and shared resource/CB replacement are verified
within that scope. Signed-loader effect/deprecated-route handling and provider
message routing qualify the open-source model. Heap invocation remains unobserved.
Accepted [D040](../adr/D040.md) now implements both public families. Its separate
[verification record](../release/2026-10-25-callbacks-diagnostics-verification.md)
tracks CI/MSRV and 15 public-native stages: 12 contexts, 20 fenced frames, 16
messages, production raw CB exchange and compatible shared allocation cleanup.
Legacy live-context configuration returns 0 on both providers; null-context
legacy returns 2. Heap ABI/controlled calls pass, with SDK invocation unobserved.
Final exact archives and release reconciliation remain separate.

**U05 complete, 2026-10-24:** the [remaining-format source accounting](records/2026-10-24-src-remaining-upscaling-formats.md)
and [independent SDK experiment](records/2026-10-24-exp-remaining-upscaling-formats.md)
resolve all D035 residual families. Accepted [D039](../adr/D039.md) implements
the applicable fixed views, SNORM/shared-exponent colour, load-only RGB32F and
D16/D24 normalized depth, including typeless motion/exposure/masks. Its
[verification](../release/2026-10-24-resource-formats-verification.md) passes
182 CI, 159 MSRV and 280 public native stages with 270 contexts/810 fenced
frames. Together with D034/D035/D037/D038, P1-C/U05 is complete for bounded
SDK access. Generic formats outside the documented external roles and arbitrary
view overrides are explicitly accounted for, not unresolved omissions.
Device/provider trust and final extracted archives remain separate. No image
or mask-algorithm comparison is an acceptance condition.


The inventory's [full matrix](records/2026-10-18-src-upscaling-release-surface-inventory.md#coverage-matrix-every-public-upscaling-descriptor-route-and-applicable-commondx12-extension)
is the historical source-to-binding-to-wrapper comparison at intake. Later
D028/D029 changes and the accepted D024 output extension are reconciled below. The inventory confirms the
implemented M10–M13/U03 controls, fixed-output M14 render regions, M15 supplied
masks and bounded linear/non-linear integration. Those completed slices do not
establish complete upscaling access or general provider compatibility.

**Current implementation:** P0-A and P0-B are implemented for both ownership
families under accepted [D028](../adr/D028.md) and [D029](../adr/D029.md).
Provider selection, selected/default query routing, pre-create requirements and
all five typed controls are available through `request_tuning`, with compulsory
parameter-domain checks. The [coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
passes 108 local CI, 97 exact Rust 1.88 MSRV and 203 isolated development-package
cases, plus Windows-target compilation and external consumers. These establish
source/host/compiler and package behaviour. The subsequent Windows follow-up
passes the 256-case consolidated batch and 31-case native profile, including fixtures,
independent MSVC ABI, both-family selected-provider dispatch and all five tuning
controls followed by dispatch/fencing. Numerical tuning application remains unproven.
U01 generation is implemented under accepted D030. Windows fixtures/ABI and
both-family native generation on 3.1.5 pass; 4.1.1/2.3.4 acknowledge but leave
masks unchanged. Those write criteria and complete experimental profiles remain
failed. M14 variable output is now implemented under the subsequently accepted
[D024 amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19):
optional active output, exact maximum allocations, dynamic-option gating and
active-output display-motion scaling. The raw spike itself accepted no API;
explicit maintainer review accepted the later design. Public-path and regression
checks pass in the [implementation verification](../release/2026-10-19-active-output-verification.md):
62 public-output cases and 257 consolidated Windows cases, including stable/MSRV,
independent MSVC ABI, consumers/examples and native regressions. The public GPU
profile covers both owners on 4.1.1/3.1.5 with 144 dispatched and 12 control frames.
The smaller-output provider and access limits remain: 2.3.4 ignores the request,
padding can be written and metadata/active-sized passes cannot certify access safety.

**U05 source assessment:** the [dispatch synthesis](upscaling-dispatch.md#external-formats-and-explicit-motion-scales)
separates documented external roles, generic conversion, generated DX12 views
and actual-provider support. Subsequent [D034](../adr/D034.md) is accepted and
implemented: a finite native scale override and typed RG32F motion. Shared
role/device/view checks and [both-family public verification](../release/2026-10-21-native-motion-verification.md)
pass, including mixed-sign jitter and active output. Zero/negative-zero/extreme
finite bits are deliberately accepted, without a provider-safe numerical-domain
claim. The open FSR2/FSR3 denominator difference and NDC source conflict remain;
D039 below resolves the remaining formats; final release verification remains open.

**U05 BGRA/sRGB slice, 2026-10-24:** the
[source audit and qualified findings](upscaling-dispatch.md#typed-bgrasrgb-access-2026-10-24)
integrate the external record, corrected pinned-source references and independent
DX12/direct-SDK probes. [D038](../adr/D038.md) is now accepted with implementation
authorization for typed BGRA UNORM and RGBA/BGRA sRGB colour/output/generator
inputs, separate view support and conditional casting. Direct C++ success does
not establish public-wrapper verification; the separate
[implementation checkpoint](../release/2026-10-24-bgra-srgb-verification.md)
now passes current/MSRV/ABI and both-family public native checks, including
UAV-capable inputs. No image or algorithm criterion is selected.

**U05 typed-format implementation, 2026-10-22:** accepted
[D035](../adr/D035.md) implements four additional typed colour/output and
generation-colour formats plus R16F depth through the existing resource APIs.
Shared role/device checks preserve actual metadata, and new query failures
retain the affected role and HRESULT. The
[implementation verification](../release/2026-10-22-typed-format-verification.md)
records passing controlled/MSRV and independent-C++/public native checks on one
configuration. Remaining
BGRA/sRGB is implemented under D038; D039 completes the other applicable candidates. [D037](../adr/D037.md)
adds the next D32/D32S8 typed/typeless depth-buffer slice with fixed depth SRVs,
conditional fully typed casting checks and renderer-owned DSV/transitions.
Its [bounded verification](../release/2026-10-22-depth-format-verification.md)
passes stable/MSRV and both-family native DSV/shader/plane/C++ checks on selected
4.1.1/3.1.5. D039 supplies the subsequent complete U05 accounting.

**Source facts at inventory intake, before D028/D029 implementation:**

- [`api.rs`](../../crates/fsr-sdk-sys/src/api.rs) had the Configure function
  signature and CPU allocation callback types, but no provider-override or debug
  descriptors. [`upscale.rs`](../../crates/fsr-sdk-sys/src/upscale.rs) lacked the
  reactive-generation and key/value Configure descriptors and the three debug
  flags. [`dx12.rs`](../../crates/fsr-sdk-sys/src/dx12.rs) lacked the custom
  allocation extension.
- The [creation chain](../../crates/fsr-sdk/src/upscaler/dx12.rs) owned the root,
  fixed API-version and backend nodes, with no provider override and no message
  callback. [Runtime state](../../crates/fsr-sdk/src/runtime/state.rs) and the
  [context owner](../../crates/fsr-sdk/src/context.rs) did not forward Configure;
  context creation/destruction always used the null host allocator.
- [Resource validation](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs)
  fixed one format per role, one mip/slice and non-MSAA 2D textures. The
  [dispatch mapping](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs)
  derived output size from the allocation and motion scale from UV/pixel units;
  explicit signed/asymmetric native scales were inaccessible.
- Both runtime families exposed provider enumeration, memory estimates and
  utilities, but no pre-create resource-requirements query. The existing raw
  resource-requirements descriptor therefore does not mean every public route
  using it was implemented.

The record reports twelve upscaler-specific descriptors, tags `0x10000` through
`0x1000b`. At intake the missing two were generated-reactive dispatch (`0x10006`)
and key/value Configure (`0x10007`). D029 binds/exposes the latter; D030 now also binds/exposes generated
reactive dispatch. D040 subsequently fills the shared diagnostics/allocation
routes and flags. At intake, further gaps existed in shared descriptors,
flags, query routing and the supported input profile; counting descriptors alone
would miss them. C++ inline DX12 conversion helpers are not exported ABI calls.

## Remaining work and ordering

This maps the inventory's recommended packages to the roadmap. It identifies
current implementation and remaining work within the selected full-access
target. Acceptance comes from the linked decisions; completed source slices
do not establish native verification.

| Inventory package | Roadmap owner | Current coverage and remaining completion boundary |
|---|---|---|
| P0-A | R04, U02 | Implemented under D028: provider override, retained chains, selected/default routes, effective live identity, internal Configure forwarding and pre-create resource requirements. Windows fixtures/MSRV/MSVC ABI and bounded native queries/dispatch pass, including two frames per family with the explicitly selected observed default ID; no universal provider compatibility claim. |
| P0-B | U02 | Implemented under D029: five typed controls through `request_tuning`, mandatory finite parameter-domain validation, null-default reset, acquisition/dispatch safety and terminal native-error eligibility. Windows fixtures/MSRV/MSVC ABI and 20 value/default-request → dispatch/fence frames pass on 4.1.1 across both families. Application, persistence and effective defaults remain unproven. |
| P1-A | U01 | Implemented under accepted [D030](../adr/D030.md): standalone unsafe generation, optional preflight and shared terminal/idle-Configure policy. Windows fixtures/MSVC ABI pass. Both families/all flags pass actual mask-write/consumer checks on 3.1.5; 4.1.1/2.3.4 return OK without writes. Native results remain mixed; supplied masks remain independently usable. |
| P1-B | M14 | Verified for bounded integration under the accepted [D024 output amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19): active output within exact maximum allocations, dynamic-option gating, shared preflight/dispatch and corrected motion mapping. [Verification](../release/2026-10-19-active-output-verification.md) passes 62 public-output and 257 consolidated Windows cases. Both owners run on 4.1.1/3.1.5; no generic capability or bounded-access proof. Allocation relaxation and final extracted-package verification remain separate. |
| P1-C | U05 | Complete for bounded SDK access under D034/D035/D037/D038 and [D039](../adr/D039.md). Exact native scale overrides and all applicable external role/view routes are implemented and verified on the recorded deployment; [182 CI, 159 MSRV and 280 native stages](../release/2026-10-24-resource-formats-verification.md) pass. Device/provider limits, generic non-applicability and unchanged maximum-allocation/safety boundaries are explicit. Final release archives remain separate. |
| P1-D | M16 | Complete for bounded SDK access under [D027's completion boundary](../adr/D027.md#sdk-access-completion-boundary-2026-10-18): public colour/exposure contract, caller obligations and provider-specific signal limits. No universal non-linear pipeline or guessed transfer rule is promised; final archive checks remain separate. |
| P2-A | U04 | Implemented and verified for bounded SDK access under [D040](../adr/D040.md): independent debug flags with estimate/create parity and faithful modern/legacy message routes. [192 CI, 169 MSRV and 15 native stages](../release/2026-10-25-callbacks-diagnostics-verification.md) pass. Both providers accept live legacy code 0; null-context legacy code 2 is preserved. No universal effective-level/delivery or quiescence guarantee. |
| P2-B | R03 | Implemented and verified under [D040](../adr/D040.md) through both public families: paired host/resource/heap, CB/message callbacks, stable tables and conservative exceptional retention. The [verification](../release/2026-10-25-callbacks-diagnostics-verification.md) records production CB exchange on current/MSRV and 12 public contexts/20 fenced frames, balanced host/resource cleanup and compatible shared replacement. Heap ABI/controlled calls pass; SDK invocation remains unobserved. |
| P2-C | Release closure | [Common rc1 qualification](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification) passes exact-package native cases, callbacks/production CB, registry-only Stable/MSRV consumers, examples/utils and bounded Proton checks. Package metadata/notes are prepared. P08 input/impact reconciliation is accepted; publication simulation passes; archive cleanup and the explicitly deferred root README review remain. |

**Next remaining release work:** P2-C, archive cleanup and the controlled root
README review; publication simulation passes and final P08
input/impact reconciliation is accepted. The callback case and production CB fixture now use the
selected extracted packages and pass on rc1. P2-A/U04, P2-B/R03, P1-C/U05 and P1-D's bounded
SDK-access contracts are implemented and verified. P1-A is implemented under D030 with mixed provider
results retained for release review; rc1 extracted-package qualification is complete.
No additional format or quality research is a U05 completion prerequisite.
The P1-B source record recommends optional active output and coverage checks, not
maximum-allocation-only access. Coverage alone does not certify provider support
or bounded shader writes, as the native padding/provider observations reinforce.
The accepted D024 amendment selects maximum allocations, dynamic-option gating
and explicit actual-provider trust. Wider allocation support and render-side
allocation relaxation remain separate scope choices. Intake provenance/citation limitations and the current
DX12-gated frame boundary are recorded in the linked dispatch assessment.

The maintainer accepted [D028](../adr/D028.md) with review clarification on
2026-10-18: optional provider IDs, additive selected-provider helpers, fresh
membership/post-create identity checks, retained concrete chains and a private
Configure seam. Provider-specific native trust belongs at unsafe acquisition.
Related pre-create queries and creation must use matching selection; default
utilities do not inherit options. This is a separate accepted design decision,
not an implementation or native result supplied by the source investigation.

## D028 implementation follow-up

[D028 implementation checks](../release/2026-10-18-provider-selection-verification.md)
now record the raw override, options, selected-provider helpers, pre-create
resource requirements, post-create identity validation and private Configure
forwarding for both families. Host/MSRV tests and Windows-target compilation
pass, including Rust 1.88.0 in all four feature configurations. The
[Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md) subsequently
passes fixtures/MSVC ABI, selected setup/live queries and explicit-provider GPU
dispatch for both families. Generator probes additionally verify exact live
selection of 3.1.5/2.3.4, retaining their different write outcomes.
The missing-source findings above describe the intake baseline, not this later
implementation. The five public Configure
controls were subsequently implemented under D029 below; diagnostics and
allocation callbacks remain separate work.

## D029 implementation follow-up

The [typed Configure record](../release/2026-10-18-upscaler-tuning-verification.md)
closes the five-key binding/wrapper gap under the original finite-passthrough
contract. The [coherence amendment/checkpoint](../release/2026-10-18-api-coherence-verification.md)
supersedes that value policy with mandatory finite parameter domains, including
direct enum construction and reuse across parameters. D042/U09 now places the
single check in portable `TuningRequest::new`; both families accept the checked
pair through `request_tuning(request)`, allowing complete-list preparation before
native mutation. The [U09 record](../release/2026-10-27-tuning-request-verification.md)
keeps production evidence separate from the earlier API.
Invalid values return `InvalidTuningValue` without FFI or poison; accepted bits
remain unchanged and explicit default requests still send null rather than
rounded numeric constants. The optional `checked_for` constructor is removed.
Domains are explicit wrapper policy derived from the shared v2.3.0 header,
independent of provider selection; support and application remain separate.
Success means acknowledgement only. A native Configure error preserves its code and prevents dispatch, all seven live queries and further
Configure; query/validation errors do not poison a usable context. Existing
single-attempt destruction and failed-destroy retention remain in force.

Safe methods depend on extended actual-provider trust at unsafe acquisition,
including synchronous non-retention, profile compatibility and destroyability
on error. GPU idleness before Configure is a lasting obligation of the prior
unsafe dispatch. The current 72-test host suite covers production forwarding
through a loaded controlled library, domain boundaries and parameter mismatch;
both Windows public-family fixtures now execute in the Windows follow-up.
All five value/default-request cycles also complete 20 submitted/fenced frames
on 4.1.1 across both families. These checks establish wrapper mapping and bounded
integration, not closed-provider application, clamping, persistence or effective defaults.

## Configure findings and unresolved contracts

The [Configure study](records/2026-10-15-src-upscaler-configure-contract.md)
reports synchronous float copying, persistence across history resets and null
default restoration in the pinned open FSR3 implementation. Rounded header
defaults are not exact reset values. The same study finds that the open FSR2
adapter returns OK while ignoring these controls: acknowledgement is not proof
of application, and there is no effective-value readback or justified provider
name allow-list. These are source findings, not newly observed native results.

Its earlier statement that no additional ordinary Query descriptor was missing
is compatible with the inventory's missing pre-create resource route: that route
uses an already bound descriptor. The later inventory expands access coverage;
it does not invalidate the completed M10 Query slice.

D028 settles provider-ID observation and mismatch policy. Configure value policy,
GPU ordering, non-retention and error eligibility are now accepted in
[D029](../adr/D029.md) by explicit maintainer instruction on 2026-10-18;
implementation was subsequently authorized and is complete in development
([checks](../release/2026-10-18-upscaler-tuning-verification.md)), followed by the
bounded Windows/native confirmation above. D040 now selects the local/movable callback ownership contract; broader output
allocation and alternative resource/state contracts remain unselected. Successful native calls alone do not prove
every optional control was applied. Both families must preserve coordinated
native access and terminal cleanup; movable types do not authorize concurrent use.

## Intake provenance and limitations

The subsequent [compatibility and ergonomics synthesis](api-and-safety.md#published-compatibility-recovery-and-extensibility)
integrates three further 2026-10-18 source reviews. They identify no demonstrated
need to break published dispatch/frame-construction signatures, but retain the
expanded unsafe acquisition contract, possible construction behaviour changes
and incomplete package/API comparison. Their strict-tuning and provider-query
consolidation suggestions originally differed from D029/D028. The maintainer's
later D029/D032 amendments select and implement mandatory tuning validation;
provider-query consolidation was not selected at that stage; D042 later
implements setup-bound planning and creation. D030 was subsequently accepted and implemented locally on 2026-10-19,
before the subsequent Windows follow-up and provider-specific generation experiment.

Amended [D032](../adr/D032.md) exposes optional metadata preflight, now named
`validate_upscale_inputs`, using compulsory recording inspection. Setup-bound
`render_size_for_quality` reports quality-derived dimensions without resizing a
context. Current recovery examples prepare the complete list of checked tuning
requests before native mutation. Native calls remain ordered and nontransactional;
a later native failure leaves earlier requests acknowledged and makes the context terminal.
The [setup-query experiment](records/2026-10-19-exp-provider-setup-cost.md) now
passes three optimized native processes and retains 192 whole-path timings.
Default medians are 229.55–230.74 ms, selected 696.98–707.10 ms per six-query
sequence. This does not isolate enumeration or select a cache/session design.

**2026-10-19 execution intake:** the canonical Windows report covers the earlier
sandbox-interrupted attempt separately from eight finalized attempts with
unchanged inputs. The generator record preserves 126 children/378 completed
frames: 3.1.5 passes 102 generation frames; 4.1.1/2.3.4 retain 255 unchanged-mask
generation frames, alongside 21 passing control frames. Successful acknowledgements
and clean cleanup do not reclassify the failed mask-write criteria. Intake checks
compact inventories, case outcomes and measurement arithmetic without new native
execution. Historical records, safety contracts and final archive gates remain unchanged.

The inventory identifies its input archive by SHA-256
`ccd27091a9c517e3602ad518d00c511a9629b70791e0e77653e6d9dab8fc3f14`,
without identifying an independent Git revision. Intake did not revalidate that
archive or repeat the full upstream investigation. The current source checks
above corroborate the main implementation gaps; they do not establish whole-tree
identity with the research input.

The retained `ffx_upscale.h` cache matches the Configure study's recorded SHA-256
`65be36d5653eb71c10379e0204f7bb7cd6a8aeec96e6f8cffc5f59ffeb1f89bf`.
Its declaration positions expose a citation defect in the new inventory:
reactive generation begins at physical line 153 and key/value Configure at 168,
rather than the inventory's 138 and 152. The cached common header also places
`ffxOverrideVersion` at 109 rather than 95. Use the pinned file links and symbol
names for navigation; do not treat its reported line ranges as independently
validated. Both historical records are preserved unchanged.

No compilation, runtime loading, GPU execution or new benchmark ran during
intake. Existing feature evidence retains its original configuration limits;
the later D028/D029 reports separately record ABI declarations, controlled host
checks and selected-provider/Configure compilation. D040's separate implementation
record adds public callback/native checks; final packaged native confirmation
remains release work. M16's recorded failed signal
predictions remain unresolved; no wrapper defect or automatic policy follows.
They remain documented limitations under M16's accepted SDK-access completion
boundary, rather than a requirement to reverse-engineer provider internals.

Generation verification gates a release shipping generation, not an unshipped
proposal. The release target remains complete upscaling access, including U01.
Generic types unused by upscaling, renderer-owned barriers/submission/tonemapping and other effects or
backends are outside this inventory. Any proposal to omit an applicable public
route requires an explicit scope decision; research does not authorize an
exception or a raw-only substitute for wrapper access. See the
[release checklist](../release/0.3.0-preparation.md#upscaling-completeness-checkpoint).
