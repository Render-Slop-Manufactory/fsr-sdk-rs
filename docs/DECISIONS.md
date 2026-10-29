# Decisions

This is the index of architecture decision records (ADRs). Each record keeps
the original D-number, rationale, status, consequences, and evidence. The
[project scope](../SCOPE.md) defines goals, the [roadmap](../ROADMAP.md)
tracks progress, and source and manifests establish implemented behaviour.

## When to record a decision

Record choices affecting public contracts, native ownership or safety, SDK
version/binding strategy, runtime acquisition or redistribution, and platform
commitments. Small, reversible implementation details do not need an ADR.

Use the next D-number in `docs/adr/DNNN.md`. Record the date, status, context,
choice, rationale, consequences, and evidence. Status is `proposed`, `accepted`,
`rejected`, or `superseded`. Accept proposals after maintainer review or explicit
authorization for the choice; research alone does not accept a proposal.
When replacing a decision, retain its rationale, mark it superseded, and link
the replacement. Update SCOPE.md if project boundaries change.

## D001 — Separate raw bindings and ergonomic wrappers

**Status:** accepted

[Read D001](adr/D001.md)

## D002 — Pin AMD FSR SDK v2.3.0 as the initial native baseline

**Status:** accepted

[Read D002](adr/D002.md)

## D003 — Handwrite the first common ABI slice

**Status:** accepted

[Read D003](adr/D003.md)

## D004 — Load an explicit DLL with private entry points

**Status:** accepted

[Read D004](adr/D004.md)

## D005 — Own effect-independent context lifecycles with terminal teardown

**Status:** accepted

[Read D005](adr/D005.md)

## D006 — Curate ABI slices and permit reviewed generated bindings

**Status:** accepted

[Read D006](adr/D006.md)

## D007 — Keep runtime acquisition explicit and deploy the v2.3.0 DX12 upscaler DLL beside the executable

**Status:** accepted

[Read D007](adr/D007.md)

## D008 — Bound the first public DX12 upscaler construction contract

**Status:** accepted

[Read D008](adr/D008.md)

## D009 — Leave public error categories open for extension

**Status:** accepted

[Read D009](adr/D009.md)

## D010 — Own the public runtime load error in fsr-sdk

**Status:** accepted

[Read D010](adr/D010.md)

## D011 — Bound the first public DX12 upscaler dispatch contract

**Status:** accepted; M6b implemented and verified on one tested configuration;
public input representation and fixed sharpening partially superseded by D014
for 0.2.0

[Read D011](adr/D011.md)

## D012 — Keep upscaler options extensible

**Status:** accepted

[Read D012](adr/D012.md)

## D013 — Optional movable runtime/upscaler family

**Status:** accepted 2026-10-09 after soundness review and contract revisions;
caller-established native transfer/coordination assumptions explicitly accepted.
Optional API implemented; [bounded public-path verification](ROADMAP_HISTORY.md#movable-implementation-checkpoint)
completed separately. Included in `0.2.0` by explicit 2026-10-10 maintainer
selection; combined release evidence is recorded
[here](ROADMAP_HISTORY.md#release-qualification)

[Read D013](adr/D013.md)

## D014 — Extensible DX12 dispatch inputs and sharpening

**Status:** accepted 2026-10-07; 0.2.0 API, migration and bounded native-sharpening
trust selected. M8 implemented and checked on 2026-10-09; the direct-native M9
integration protocol and final local candidate checks passed on 2026-10-10
([M9 verification](release/2026-10-10-m9-verification.md)). The
[combined 0.2.0 release check](ROADMAP_HISTORY.md#release-qualification)
includes the optional D013 family. Publication remains a maintainer action.
Original native effect criteria failed and stay failed.

[Read D014](adr/D014.md)

## D015 — Separate common upscaler inputs and the DX12 backend

**Status:** accepted 2026-10-09, including the follow-up that makes common inputs
available without platform/backend gates. Existing imports, native gates and
safety contracts are preserved. The later authorized loader follow-up isolates
Windows implementation and makes raw `libloading` optional and target-specific.
Vulkan remains unsupported.

[Read D015](adr/D015.md)

## D016 — Separation of Native Provider Controls and SDK-Level Policies

**Status:** accepted 2026-10-09 after consistency review and explicit maintainer
authorization. Owns the cross-feature direct-control and SDK quality-policy
invariant; existing feature contracts and release scope are unchanged.

[Read D016](adr/D016.md)

## D017 — Explicit pinned maintainer SDK acquisition

**Status:** accepted 2026-10-12 by explicit maintainer authorization after prompt
review. A developer-only CLI installs the pinned subset; ordinary verification
is offline, and strict Windows trust verification may access the network.
Published crates and D007's runtime/deployment boundary remain unchanged.

[Read D017](adr/D017.md)

## D018 — Selected DX12 upscaler Queries and owned results

**Status:** accepted 2026-10-12 under maintainer implementation authorization;
query-specific native trust extends unsafe acquisition. M10 implementation and
Windows native verification are complete; the maintainer selected it for
`fsr-sdk 0.3.0` on 2026-10-13. [Release preparation](release/0.3.0-preparation.md)
records the subsequent 2026-10-18 selection of complete upscaling access for
0.3.0, with remaining implementation, final artefact checks and publication open.

[Read D018](adr/D018.md)

## D019 — Proton as an explicit DX12 verification environment

**Status:** accepted 2026-10-13 by explicit maintainer authorization. The initial
V07 cross-build/query/single-frame proof and bounded V07e portability checkpoint
are verified on Steam Deck and Steam Machine
([integrated evidence](research/platform-and-backends.md#observed-proton-execution)).
V07d temporal quality regression is deferred until after 1.0.0 under the maintainer's
[verification priority](../SCOPE.md#verification-priority), with failed image
criteria and a blocked Deck jitter verdict preserved. Bounded V08 local-import/offline
execution and V09 timeout cleanup
are verified on the recorded Deck stack
([runtime and cleanup evidence](research/platform-and-backends.md#prepared-offline-runtime-and-timeout-containment)).
Native platform support is unchanged.

[Read D019](adr/D019.md)

## D020 — Checked per-frame pre-exposure

**Status:** accepted 2026-10-14 under maintainer implementation authorization.
Portable positive-finite scalar, default `1.0`, per-frame setter/getter and exact
shared DX12 mapping. [Windows fixture/MSRV/ABI and bounded native confirmation](release/2026-10-15-windows-m11-m14-verification.md)
pass on one configuration.

[Read D020](adr/D020.md)

## D021 — Depth and motion-vector profiles

**Status:** accepted 2026-10-14 under explicit M12 slice selection and subsequent
request to implement the remaining controls. Standard/reversed finite/infinite
depth, render/display and jitter-excluded/included motion at creation; UV/pixel
units per frame. Defaults and applied pixel jitter are preserved.
[Windows fixture/MSRV/ABI and eight-profile native confirmation](release/2026-10-15-windows-m11-m14-verification.md)
pass on one configuration.

[Read D021](adr/D021.md)

## D022 — Provider-routed quality, size and jitter Queries

**Status:** accepted 2026-10-14 under explicit implementation authorization.
Four pre-create/live SDK helpers, portable checked inputs, direct provider results
and bounded numerical acquisition trust. Both ownership families share mappings.
[Windows fixture/MSRV/ABI and native pre-create/live queries](release/2026-10-15-windows-m11-m14-verification.md)
pass for both families on one configuration.

[Read D022](adr/D022.md)

## D023 — Explicit and automatic exposure

**Status:** accepted 2026-10-14 under explicit M13 implementation authorization.
Explicit remains the default; automatic contexts check provider requirements and
omit the exposure texture. Shared creation/estimate flags and null-resource
mapping preserve pre-exposure and terminal cleanup contracts.
[Windows fixture/MSRV/ABI and three-frame native confirmation](release/2026-10-15-windows-m11-m14-verification.md)
pass; the earlier CI harness failure does not recur.

[Read D023](adr/D023.md)

## D024 — Dynamic active render regions with fixed output

**Status:** accepted 2026-10-14 under explicit M14 implementation authorization,
including deferral of the pre-M14 performance baseline. In the original slice,
contexts opt into dynamic resolution; frames select an active render region inside
fixed maximum allocations. Output stays fixed in that slice; validation and motion scaling preserve
default behaviour and caller-controlled reset.
[Windows fixture/MSRV/ABI and four-profile native confirmation](release/2026-10-15-windows-m11-m14-verification.md)
pass for the fixed-output render slice. [P05's bounded transition timing](research/records/2026-10-16-exp-active-render-timing.md)
is verified; a consistent transition penalty remains inconclusive. The
[variable-output source assessment](research/upscaling-dispatch.md#variable-output-resolution-source-contract)
is integrated on 2026-10-19. It distinguishes context capacity, allocation and
active output, FSR2/FSR3 behaviour and rounded-write risks. The subsequent
[native spike](research/records/2026-10-19-exp-variable-output-resolution.md)
passes 71 raw integration cases, observing variable footprints on 4.1.1/3.1.5,
maximum output on 2.3.4 and padding writes. Research alone did not amend the
contract. Subsequent maintainer review accepted the [active-output amendment](adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19):
exact maximum allocations, dynamic-option gating, active-output display-motion
mapping and explicit provider/access trust. It is implemented with
[public-path verification](release/2026-10-19-active-output-verification.md);
broader allocation safety/support and final release archives remain separate.

[Read D024](adr/D024.md)

## D025 — Supplied upscaler masks and provider-aware dispatch validation

**Status:** accepted 2026-10-15 under explicit implementation authorization after
API/extensibility review. Optional borrowed R8_UNORM masks, mandatory construction
requirements query and shared per-frame checks. **Amended with maintainer approval
2026-10-15:** query consumption is not a forwarding allow-list. Require the
presence of mandatory masks, validate and forward supplied masks irrespective of optional
bits, and add no bypass mode. The amendment is implemented;
[Windows fixtures/MSRV/C++ ABI and all 18 native mask frames pass](release/2026-10-15-mask-forwarding-verification.md)
on the recorded 4.1.1 configuration. Actual mask consumption is not established.

[Read D025](adr/D025.md)

## D026 — Opt-in linear HDR upscaling

Accepted on 2026-10-15 after concept review and explicit implementation
authorization. Portable opt-in HDR options share creation/estimate mapping;
linear RGBA16F and existing exposure contracts remain.
[Windows fixture/MSRV/ABI and six native HDR frames pass](release/2026-10-15-windows-feature-batch.md).
Provider-specific non-linear signal limits are documented under D027.

[Read D026](adr/D026.md)

## D027 — Checked colour hints with an explicit per-frame override

Accepted on 2026-10-15 under explicit implementation authorization. Non-linear
creation classification and sRGB/PQ hints are independent native controls.
Default pairing requires NonLinear for either hint; an explicit per-frame setter
bypasses only that check. [Windows controlled mapping/validation, MSRV and C++ ABI pass](release/2026-10-15-windows-feature-batch.md).
The maintainer accepted M16's bounded SDK-access completion on 2026-10-18:
document the colour/exposure contract and provider-specific signal limitations,
without promising an end-to-end non-linear pipeline. Existing native evidence
and failed signal predictions retain their limits; final archive checks remain separate.

[Read D027](adr/D027.md)

## D028 — Explicit upscaler provider selection and shared native routing

**Status:** accepted with review clarification, 2026-10-18, confirmed by the
maintainer. Optional provider IDs, selected-provider utilities, pre-create resource
requirements and a private Configure seam share explicit acquisition-time native
trust. Related pre-create queries and creation must use matching provider
selection; default utilities do not inherit options. Implementation and
[host/MSRV tests and Windows compilation checks](release/2026-10-18-provider-selection-verification.md)
are complete. The subsequent [Windows verification](release/2026-10-19-windows-upscaler-verification.md)
passes fixtures, independent MSVC ABI and selected-provider GPU execution. D042
now binds the provider request to setup rather than creation options.

[Read D028](adr/D028.md)

## D029 — Typed upscaler Configure controls

**Status:** accepted, 2026-10-18, by explicit maintainer instruction. Selects one
typed parameter/value or provider-default request per context call through
`request_tuning`, with mandatory parameter-domain checks and acknowledgement-only
success. D042/U09 now makes portable private-field `TuningRequest` the canonical
input, with mandatory constructor validation before native mutation; the earlier
optional checked/unchecked pair remains removed. Safe methods rely on
explicitly extended unsafe acquisition and dispatch contracts, including
completion before Configure; a native Configure error makes the context
terminal for dispatch, live queries and further configuration. Both ownership
families share the contract. After initial deferral, implementation was explicitly
authorized on the same date and is complete in development; [implementation
checks](release/2026-10-18-upscaler-tuning-verification.md) record local results.
The subsequent [Windows verification](release/2026-10-19-windows-upscaler-verification.md)
confirms both-family value/default tuning cycles followed by fenced dispatch.

[Read D029](adr/D029.md)

## D030 — Standalone reactive-mask generation

**Status:** accepted and locally implemented, 2026-10-19, under maintainer
instruction to proceed with U01 without Windows. Both families expose unsafe
`record_reactive_mask`, optional nonmutating metadata preflight, portable finite
options and four independent SDK flags. Explicit active regions, output alias
rejection and renderer-owned barriers preserve the shared terminal dispatch policy.
Configure idleness covers both GPU recording routes. [Implementation checks](release/2026-10-19-reactive-mask-generation-verification.md)
separate local execution/Windows compilation from deferred MSVC, Windows-fixture
and actual-provider/GPU execution.

[Read D030](adr/D030.md)

## D031 — Retain the Vulkan feature and require backend isolation

**Status:** accepted 2026-10-18, amended with implementation authorization on
2026-10-24. Retain the published `vulkan` feature and its deliberate compile error
for wrapper `0.3.0` / sys `0.1.2`; the original removal and sys `0.2.0` bump are
reversed. Initial usable Vulkan upscaling is selected for wrapper `0.4.0`, subject
to a usable official SDK/runtime and native evidence. Windows/DX12 code, native
types, loaders and dependencies remain encapsulated; common and future Vulkan
consumers must not require them.
The [implementation checks](release/2026-10-18-backend-feature-verification.md)
record the original removal's host, Windows compilation, MSRV and package-content
results separately; those historical inputs precede the amendment.

[Read D031](adr/D031.md)

## D032 — Early input validation and explicit renderer recovery

**Status:** accepted, 2026-10-18, under the instruction to address the six
ergonomics findings. Amended to require D029 tuning-domain checks and expose
optional `validate_upscale_inputs` metadata preflight, renderer-owned recovery/
idle-Configure examples and a prepared startup-query timing profile. Native
terminality, GPU ownership and D028 selection remain unchanged. D030 was later
accepted and implemented separately on 2026-10-19;
Windows/native measurements remain separate.

[Read D032](adr/D032.md)

## D033 — Boundary between native SDK abstractions and optional utilities

**Status:** accepted, 2026-10-19, under maintainer instruction to integrate the
boundary audit and update project scope. Native operations, checked representations,
necessary mappings and safety/preflight remain core; project-defined algorithms
are candidates for future opt-in utilities within the same crate. Renderer
ownership stays with integrations. No existing API needs moving and no utility
feature or implementation is selected. The same-date composability clarification
prefers checked core outputs, preserves operation phases and requires a demonstrated
gap before adding policy-specific core hooks. D016 and the through-1.0.0 quality-policy
deferral remain binding.

[Read D033](adr/D033.md)

## D034 — Explicit native motion scales and typed motion-format validation

**Status:** accepted and implemented, 2026-10-21, under explicit maintainer
instruction to accept and implement the reviewed proposal. First U05 slice: portable checked native scale
with an optional per-frame override, unchanged UV/pixel fallback, finite signed
bits including zero/negative zero, and RG32F motion with role/device/view checks.
Shared mapping/preflight, both-family fixtures and
[stable/MSRV/public native verification](release/2026-10-21-native-motion-verification.md)
pass, including the mixed-sign jitter case. D039 subsequently completes U05's
applicable format accounting and verification; later rc1 archive qualification
is recorded in the consolidated release report.

[Read D034](adr/D034.md)

## D035 — Typed colour, output and half-float depth formats

**Status:** accepted and implemented, 2026-10-22, under explicit maintainer
instruction, including the reviewed role-bearing device-query errors. This U05
slice adds four typed colour/output
alternatives, R16F device depth and separately verified generation colour inputs,
with role-specific device/view checks and existing borrowed-resource APIs.
D037 separately adds D32/D32S8 depth-buffer access; D038/D039 complete the other
applicable candidate families. [Stable/MSRV/ABI and public/C++ native verification](release/2026-10-22-typed-format-verification.md)
passes within one recorded configuration. D039 closes U05; later rc1 archive
qualification is recorded in the consolidated release report.

[Read D035](adr/D035.md)

## D036 — Optional FSR generation provider-choice utility

**Status:** accepted and implemented, 2026-10-22, under explicit maintainer
instruction after review. The portable, off-by-default `utils` resolver maps
strict numeric provider names to an explicit generation choice, returning the
existing optional opaque ID. Exact parsing/ambiguity rules and standard resolver
errors preserve core selection/defaults. [Implementation verification](release/2026-10-22-provider-utils-verification.md)
records checks and limits. The maintainer selected release inclusion in 0.3.0
on 2026-10-27; the feature remains off by default. Deferred quality work remains
unselected.

[Read D036](adr/D036.md)

## D037 — D32 and D32S8 depth-buffer allocations

**Status:** accepted and implemented, 2026-10-22, under explicit maintainer
instruction. Four typed/typeless allocation routes use fixed depth-plane SRVs,
role-bearing device errors and conditional fully typed casting checks, preserving
renderer-owned DSV production and whole-resource transitions.
The [bounded verification](release/2026-10-22-depth-format-verification.md) passes
178 stable CI, 155 MSRV and 82 native cases with frozen inputs. Remaining U05
families and final release archives remain separate.

[Read D037](adr/D037.md)

## D038 — Typed BGRA and sRGB colour resources

**Status:** accepted, 2026-10-24, under explicit maintainer instruction to
formulate, implement and verify. Adds typed BGRA8 UNORM and RGBA8/BGRA8 sRGB
to main colour/output and both generator colour inputs, with exact native
mapping, separate SRV/UNORM-UAV checks and conditional fully typed casting.
Colour controls remain explicit. [Current/MSRV/ABI and both-family native
verification](release/2026-10-24-bgra-srgb-verification.md) passes within the
recorded configuration, without image/algorithm comparisons. D039 subsequently
resolves the remaining U05 candidates; final release archives remain separate.

[Read D038](adr/D038.md)

## D039 — Remaining external upscaling format access

**Status:** accepted, 2026-10-24, under the maintainer's instruction to finish
U05, implement and verify. Adds fixed typeless colour/motion/scalar/exposure/mask
views, SNORM/shared-exponent colour, load-only RGB32F and D16/D24 normalized
depth. All 40 native surface values are accounted for and bound; exact role/view
checks preserve metadata and role-bearing errors. [Verification](release/2026-10-24-resource-formats-verification.md) passes
182 CI, 159 MSRV and 280 native stages, completing U05 for bounded SDK access.
No image, algorithm or automatic format-conversion policy is selected.

[Read D039](adr/D039.md)

## D040 — Explicit callback integration and upscaler diagnostics

**Status:** accepted, 2026-10-25, after maintainer authorisation and independent review. Concrete R03/U04 contract based on the source,
practice and bounded native findings: portable debug controls, a separate unsafe
callback constructor, paired allocation tables and explicit modern/legacy debug
methods. The wrapper retains descriptor/host-table storage; callback targets stay
externally owned under lasting shared-state obligations. Callback-bearing create
errors conservatively retain the wrapper bundle. Both public families are
implemented; the [verification record](release/2026-10-25-callbacks-diagnostics-verification.md) tracks source and native checks. The ordinary constructor's D005 policy is preserved.

[Read D040](adr/D040.md)

## D041 — Distinguish safety obligations from rendering and SDK requirements

**Status:** accepted, 2026-10-26, under explicit maintainer instruction after the
[ecosystem follow-up](research/records/2026-10-26-src-rustdoc-requirements-ecosystem.md).
Extends D011/D014's distinction into a common documentation convention: complete
`Safety` contracts, correctness explained in prose or descriptive sections, and
explicitly labelled uncertain native assumptions retained at the safety boundary.
`Requirements` is allowed without prescribing its spelling. The
[source investigation](research/records/2026-10-26-src-safety-and-rendering-requirements.md)
classifies the proposal-baseline local/movable contracts. The Rustdoc migration
implements the convention without changing executable code, public signatures,
accepted safety obligations or release prerequisites.

[Read D041](adr/D041.md)

## D042 — Coherent upscaling setup and frame composition

**Status:** accepted, 2026-10-26, under maintainer instruction after comparing
both same-date API reviews. Selects the 0.3.0 transition to setup-bound planning
and creation, canonical portable frame data, named resources, complete effective
settings, checked tuning requests, targeted error/metadata improvements and
explicit terminal usability. Keeps the existing native ownership and unsafe
integration boundaries; U07 now supplies the concrete loader/effect trust split.
Partially supersedes D014/D028/D029/D034's public shapes, preserving historical
evidence and tracking each production slice separately. [U07–U12](../ROADMAP.md#next-actions)
track the selected work before final release verification.

The [concrete U07 contract](design/upscaler-setup.md) was approved for
implementation on 2026-10-27. Production now exposes loader-only unsafe `load`,
unsafe effect setup acquisition and one setup-bound provider request, retaining
`UpscalerOptions` without provider storage. The old Runtime effect routes and
public static owner constructors are removed. See the
[U07 implementation verification](release/2026-10-27-upscaler-setup-verification.md). U08 now implements portable frame/generator parameters, named resource builders,
one motion/colour request and separate recording arguments; see its
[verification](release/2026-10-27-frame-composition-verification.md).
U09 now implements checked tuning requests with full-list preparation; see its
[verification](release/2026-10-27-tuning-request-verification.md). U10 implements
structured errors, first native causes and provider/mask composition; see its
[verification](release/2026-10-27-error-metadata-verification.md). U11 adds pure
usability and explicit local abandonment on both owners; see its
[verification](release/2026-10-27-context-usability-verification.md). U12 integrates
the canonical API, targeted must_use diagnostics, extensible DebugLevel and
complete external workflows; see [migration](MIGRATING_0_3.md) and
[verification](release/2026-10-27-api-transition-verification.md).
[P08 is closed](release/2026-10-29-performance-review-closure.md) with accepted
limits; [common rc1 archive qualification](release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
is complete. The frozen spike is prior evidence.

[Read D042](adr/D042.md)

## D043 — Performance verification before every release

**Status:** accepted, 2026-10-27, under explicit maintainer instruction. Every
release requires fresh candidate performance verification and review of
regressions. P08 tracks the first 0.3.0 check after U12; its
[adaptation](design/release-performance.md) reuses P06/P07 and explicitly selects
the 5 us CPU criterion for 0.3.0. P08 is now
[closed with maintainer-accepted exceptions](release/2026-10-29-performance-review-closure.md),
including temporal selection and observed provider/deployment cost limits.
Historical/full-run verdicts remain unchanged; final archive qualification and
publication remain separate. Future releases require their own selected review.

[Read D043](adr/D043.md)

## Open choices

D042 accepts the API target, not every review sketch. U07's concrete setup
borrowing and loader/effect allocation are accepted and implemented. U08/U09
implement composition and checked requests. U10 supplies the reviewed error/
metadata contract. U11 implements the terminal-usability boundary. U12 resolves
integration and migration with existing type-identity reexports and one canonical
implementation, without a blanket module/type rename or compatibility facade.
No additional ownership/backend trait or mass constructor rename is selected. U10 now has a
[concrete error/metadata contract](design/errors-and-provider-metadata.md),
approved for implementation on 2026-10-27, including the Windows `std` feature
needed to retain original backend causes. Its
[production verification](release/2026-10-27-error-metadata-verification.md) records
the bounded completion. D036 utility inclusion in 0.3.0 is now selected, with
`utils` remaining off by default. D043 selects recurring release performance
checks; [P08 is closed](release/2026-10-29-performance-review-closure.md) with the
maintainer-accepted temporal subset and provider/deployment differences. Full-run
CPU outcomes remain inconclusive; subset bounds retain their disclosed post hoc
limits. Common rc1 candidate/archive coverage and publication simulation are
complete; publication remains open.

R03/U04's accepted [D040 contract](adr/D040.md) now supplies the concrete callback and
diagnostic contract. Maintainer authorisation after independent review accepts
it; implementation and public-native checks are recorded separately from the
preceding direct-SDK spike. Final package/release closure remains separate.

M14's [D024 output amendment](adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
is accepted and implemented within exact maximum allocations. Smaller/intermediate/
oversized output or display-motion allocations and render-allocation relaxation
remain separate unselected extensions. The [dispatch assessment](research/upscaling-dispatch.md#accepted-maximum-allocation-output-slice)
retains empirical access limits: no generic shader-boundary proof, provider
allow-list/capability detection or padding preservation follows from the evidence.
The accepted contract uses explicit actual-provider trust and renderer-owned GPU
duties; native success is no certificate that requested geometry was applied.

U05's accepted [D034 slice](adr/D034.md) uses the
[source record](research/records/2026-10-19-src-upscaling-formats-and-motion-scale.md)
and [raw format/scale spike](research/records/2026-10-20-exp-motion-formats-and-scales.md).
Its explicit override precedence, finite-bit policy and RG32F role/device/view
rules are implemented and verified within the recorded boundaries. D042 selects
one effective motion choice to replace that precedence during U08; numerical
and resource/view contracts remain. [D035](adr/D035.md), D037/D038 and accepted [D039](adr/D039.md)
account for the remaining role/view families. D039's explicit mapping and
verification checkpoint owns their closure; converter membership accepts no
role automatically. The open FSR2/FSR3 display-motion difference and conflicting
NDC examples change no
existing UV/pixel defaults. Current accepted contracts remain in force.

The dispatch-input and sharpening design in [D014](adr/D014.md) is accepted;
M7's contract decision and M8 implementation are complete. Its separate
[native integration protocol](release/2026-10-10-m9-verification.md#integration-protocol)
passed in [M9 verification](release/2026-10-10-m9-verification.md), including
same-context mode switching and final local candidate checks. Publication remains
separate. The historical M9 projection excluded D013; the selected combined
release includes it and uses sys `0.1.1` for existing loader maintenance.
The original sharpening effect gate failed in the
[native spike](research/records/2026-09-30-exp-d014-native-sharpening.md)
and remains failed. D011 remains the published 0.1.0 dispatch baseline;
D014 selects its limited 0.2.0 replacement and provider-trust extension.

The optional movable family in [D013](adr/D013.md) is accepted after the
[soundness review](research/records/2026-10-09-src-d013-soundness-architecture-review.md).
Renderer-owned state migration motivates the separate `Send + !Sync` types.
The lasting unsafe acquisition contract carries native compatibility and
coordination assumptions; missing AMD guarantees and unobserved provider
termination remain unverified facts, not design-acceptance blockers. The optional
public family, concrete field audit and bounded public-path verification are
complete in current source; the existing D008 types stay confined to their
acquiring thread. Explicit 2026-10-10 selection includes movable in `0.2.0`;
the combined release record verifies its actual artefacts.

Further binding slices under D006 and other SDK effects require their own
decisions; D042 now selects the current upscaling API transition.
[D011](adr/D011.md) accepts the first public DX12 dispatch
contract; M6b implementation and bounded public-route GPU verification are
complete. D008 selects the first construction
contract; implementation and
bounded [Windows native verification](research/records/2026-09-24-exp-m5-windows-native-verification.md)
are complete. D007 resolves runtime discovery/acquisition policy for its bounded
baseline; its explicitly deferred runtime questions remain open. Record further
choices when supported by the relevant investigation; research recommendations
are not decisions.
