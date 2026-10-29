# Roadmap

Work items, prerequisites and completion checkpoints for the [project scope](SCOPE.md).
[Current coverage](README.md#what-works-today) · [Decisions](docs/DECISIONS.md) ·
[History](docs/ROADMAP_HISTORY.md) · [Maintenance rules](CONTRIBUTING.md#roadmap-maintenance).

## Next actions

Priority order. Release versions are assigned when work is selected to ship.

The next release is **fsr-sdk 0.3.0**, with **fsr-sdk-sys 0.1.2**.
The [release package table](docs/release/0.3.0-preparation.md#release-packages)
contains the current archive sizes and SHA-256 values.
The maintainer selected **complete upscaling access to the pinned SDK v2.3.0**
for this release on the supported Windows x64/MSVC + DX12 target, 2026-10-18.
This includes the remaining upscaling APIs, not only the already verified slices.
R05's [upscaling source inventory](docs/research/upscaling-coverage.md) is integrated.
It maps the applicable public creation, dispatch, Query, Configure, helper and
shared runtime/backend capabilities to raw bindings, wrapper access and remaining
integration verification. The applicable interfaces are now implemented with
bounded feature evidence. The selected API transition, exact rc1 host coverage,
accepted performance reconciliation and publication simulation are complete;
release closeout remains.
The [release preparation](docs/release/0.3.0-preparation.md) tracks readiness;
feature checkpoints do not verify the final release packages.

On 2026-10-26 the maintainer selected [D042](docs/adr/D042.md)'s coherent API
transition for **0.3.0**, prioritizing caller experience and extensibility over
preserving current wrapper signatures. U07–U12 are verified within their recorded bounds.
P08 is [closed with accepted limits](docs/release/2026-10-29-performance-review-closure.md).
Complete final archive coverage and qualification before release. Earlier
verified feature rows retain their original evidence; they do not verify the
replacement API. The native SDK-access target remains unchanged.

U07 now implements the [accepted contract](docs/design/upscaler-setup.md),
authorized on 2026-10-27: loader-only unsafe loading, unsafe effect acquisition,
setup-bound planning/create with one request, and independently retained contexts.
The [implementation verification](docs/release/2026-10-27-upscaler-setup-verification.md)
records production checks separately from the
[2026-10-26 spike](docs/research/records/2026-10-26-exp-upscaler-setup-contract-spike.md).
U08 now implements the complete composition routes; its
[verification](docs/release/2026-10-27-frame-composition-verification.md) records production evidence.
U09 now implements portable checked tuning requests and canonical operations on
both families; its [verification](docs/release/2026-10-27-tuning-request-verification.md)
records production checks. U10 now implements structured owned errors, retained
first native causes and provider/mask composition; its
[verification](docs/release/2026-10-27-error-metadata-verification.md) records the
source and native bounds. U11 now adds pure usability and local abandonment
marking; its
[verification](docs/release/2026-10-27-context-usability-verification.md) passes
593 stable/MSRV/consumer/ABI/native cases, including both-family abandonment.
U12 now closes source API integration, with complete external workflows,
[migration guidance](docs/MIGRATING_0_3.md) and
[625 passing stable/MSRV/consumer/ABI/native cases](docs/release/2026-10-27-api-transition-verification.md).
The spike remains prior evidence. [P08](#performance), the first mandatory
candidate performance review under [D043](docs/adr/D043.md), is closed with
maintainer-accepted temporal/deployment limits. Exact-package callback/CB ABI,
examples/utils and native-profile bindings are now implemented; the
[package coverage verification](docs/release/2026-10-29-package-coverage-verification.md)
records their checks and remaining execution limits. On 2026-10-29 the maintainer
selected a bounded internal cleanup before final release preparation. U13 resource
inspection is now [verified](docs/release/2026-10-29-resource-inspection-verification.md);
U14 portable common types now pass [targeted checks](docs/release/2026-10-29-common-organization-verification.md).
U15 descriptor/test alignment and Rustdoc reconciliation now pass
[targeted integration checks](docs/release/2026-10-29-composition-cleanup-verification.md).
The internal cleanup preserves the public API, native contracts and workload
inputs. The [fresh post-cleanup functional batch](docs/release/2026-10-29-candidate-verification.md#post-cleanup-windows-functional-qualification)
now passes all eight Windows source/native profiles, **1,434/1,434** required
cases, with stable inputs and independently verified retained inventories.
The [release-facing documentation/metadata pass](docs/release/2026-10-29-candidate-verification.md#release-documentation-and-metadata)
is complete with six passing focused checks. The maintainer approved the updated
top-level README on 2026-10-30. The [shared rc1 pair](docs/release/2026-10-29-candidate-package-preparation.json)
is prepared from clean committed sources; Cargo package verification,
four focused archive/registry/integrity cases and eight bounded content screens
pass. The [rc1 cross-host synthesis](docs/release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
now qualifies that same exact pair on Windows **1,448/1,448**, Deck **16/16
offline + 17/17 timeout** and Machine **12/12**: **1,493 required case checks**
across ten stages. The earlier Windows/Linux archive mismatch is historical;
rc1 hashes match on all three hosts. Provider-specific generator failures remain
failed and native safety/platform limits are unchanged. The
[rc1 performance acceptance](docs/release/2026-10-29-rc1-performance-acceptance.md)
now closes final P08 reconciliation after the 122-case short timing follow-up.
Accepted CPU nominal one-sided 95% upper bounds are 4.046–4.618 us in P08's
retained subset; no wrapper-specific GPU overhead or cleanup regression is
detected, with the recorded statistical limits preserved. The
[publication simulation](docs/release/2026-10-29-candidate-verification.md#rc1-publication-dry-run)
now passes: both uploads aborted and both regenerated hashes match rc1. The
temporary candidate directory is removed and the controlled README review is
complete. The final documentation/link review corrects stale API descriptions
and navigation.
Publication requires separate authorisation and registry/docs verification.
October 13/14 results remain historical evidence. The
maintainer selected recurring pre-release performance checks on 2026-10-27;
no further performance experiment is currently selected for 0.3.0. Assess any
later change to measured paths and repeat affected measurements if warranted.

By maintainer direction on 2026-10-29, U14/U15 use
[targeted development checks](docs/verification-selection.md). Batch full
qualification after the cleanup and documentation integration; U13's 1,426
checks are not a per-item requirement. Repeat only affected checks after later edits.
The post-U15 functional batch is now complete; prose-only release preparation
does not require repeating that source/native matrix.

By maintainer selection on 2026-10-24, `0.3.0` retains the reserved `vulkan`
feature and its deliberate compile error. Initial usable Vulkan upscaling is
targeted at **fsr-sdk 0.4.0** under [B01](#backends-and-runtime) and the
[D031 amendment](docs/adr/D031.md). Official SDK/runtime availability, concrete
platform selection and native verification remain prerequisites.

The [SDK 2.4/Vulkan source assessment](docs/research/platform-and-backends.md#sdk-24-and-release-sequencing)
is integrated on 2026-10-29. It finds no Vulkan-driven API blocker for 0.3.0;
keep SDK 2.3/DX12 and the reserved failing feature. After 0.3.0, target a
**conditional 0.3.1** for compatible SDK 2.4 DX12 adoption (R06/R07) and Vulkan
contract/preparation work (B03), followed by usable Vulkan in **0.4.0** (B01).
The SDK-only patch must preserve existing consumers and native trust contracts;
move incompatible changes to 0.4.0. Exact raw-crate versioning and SDK adoption
remain subject to the ABI/API audit and an explicit decision update. Source
availability is established; signed binary acquisition and native compatibility
are not. Preparation does not enable Vulkan or promise native Linux support.

SDK access and wrapper integration are the priority through 1.0.0. Image-quality
research is outside this roadmap and deferred until after 1.0.0 by maintainer
direction, 2026-10-18 ([scope](SCOPE.md#verification-priority),
[deferred backlog](docs/POST_1_0.md)). ABI, mapping, safety and bounded native
integration checks remain required.

The accepted [core/utility boundary](docs/adr/D033.md) keeps native helpers and
validation in core. The [audit synthesis](docs/research/core-and-utilities.md)
finds no API requiring extraction. The separately accepted and implemented
[D036 provider resolver](docs/adr/D036.md) is tracked under optional utilities;
its release inclusion in 0.3.0 was selected on 2026-10-27. It remains opt-in.
Complete native upscaling access, the D042 API transition and that utility are
the selected release scope.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| R05 — Complete SDK-surface inventory | ✓ Verified (upscaling source slice); other effects open | Separate inventory for other SDK effects | [Inventory and intake assessment](docs/research/upscaling-coverage.md) map the 0.3.0 upscaling surface and remaining gaps; source-only, no new native checks. Other SDK effects remain to be inventoried |
| M10 — Provider metadata and selected Queries | ✓ Verified | [D018](docs/adr/D018.md) | Selected for 0.3.0. Owned names/opaque IDs and five queries for both families; [Windows ABI, controlled/consumer/MSRV and native GPU checks](docs/release/2026-10-12-windows-m10-sdk-verification.md) pass on one configuration. Final release checks/publication remain open; provider override is tracked by R04 for this release |
| M11 — Pre-exposure | ✓ Verified | [D020](docs/adr/D020.md) | Positive-finite scalar, default `1.0` and shared exact mapping; [Windows fixture/MSRV/ABI and three-factor native GPU confirmation](docs/release/2026-10-15-windows-m11-m14-verification.md) pass on one configuration |
| M12 — Depth and motion-vector profiles | ✓ Verified | [D021](docs/adr/D021.md) | Standard/reversed finite/infinite depth, render/display and jitter-excluded/included motion, UV/pixel units and matching estimate flags; [Windows fixture/MSRV/ABI and eight native profiles](docs/release/2026-10-15-windows-m11-m14-verification.md) pass |
| U03 — SDK quality-mode/size/jitter helpers | ✓ Verified | Final release-archive verification | [D022](docs/adr/D022.md) implements four provider-routed helpers; [Windows fixture/MSRV/ABI and native pre-create/live queries for both families](docs/release/2026-10-15-windows-m11-m14-verification.md) pass |
| M13 — Automatic exposure | ✓ Verified | [D023](docs/adr/D023.md) | Explicit/automatic options, provider requirement checks and optional/null exposure mapping; [Windows fixture/MSRV/ABI and three-frame native GPU confirmation](docs/release/2026-10-15-windows-m11-m14-verification.md) pass |
| M14 — Active render/output extents | ✓ Verified (bounded integration) | Final release-archive verification; accepted [D024 amendment](docs/adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19) | Dynamic active render/output within exact maximum allocations, shared preflight/dispatch and active-output display-motion mapping. [Windows verification](docs/release/2026-10-19-active-output-verification.md) passes 62 public-output cases on 4.1.1/3.1.5 across both families and 257 consolidated stable/MSRV/ABI/consumer/native cases. Whole output may be written; actual-provider support/access remains conditional, 2.3.4 smaller output is outside the contract, and wider allocation support remains separate. [P05 render timing](docs/research/records/2026-10-16-exp-active-render-timing.md) retains an inconclusive transition penalty; it measures no variable output |
| M15 — Supplied masks | ✓ Verified (bounded integration) | Final release-archive verification | [D025's amendment](docs/adr/D025.md#amendment-rationale-and-implementation-status) is implemented: require mandatory masks, validate/forward supplied masks irrespective of optional query bits, without a bypass mode. [Windows fixtures/MSRV/C++ ABI and 18 native frames](docs/release/2026-10-15-mask-forwarding-verification.md) pass on provider 4.1.1, including both masks separately/together, explicit/automatic exposure and active-render shrink/grow. Actual provider consumption is not established |
| M16 — HDR and non-linear colour | ✓ Verified (bounded SDK access) | Final release-archive verification | [D027's completion boundary](docs/adr/D027.md#sdk-access-completion-boundary-2026-10-18) documents the colour/exposure contract and provider-specific limits. [Windows fixtures/MSRV/C++ ABI and six linear-HDR frames](docs/release/2026-10-15-windows-feature-batch.md) pass; the [signal experiment](docs/research/records/2026-10-15-exp-non-linear-colour-signals.md) adds 270 integration-passing frames on one configuration. Fifteen changing-pre-exposure predictions remain failures. No universal non-linear pipeline, conversion rule or automatic reset policy is promised |
| U01 — Automatic reactive masks | Active (native results mixed) | Accepted [D030](docs/adr/D030.md); final archive checks and recorded provider limits | Both public families/fixtures and independent MSVC ABI pass. [Native experiment](docs/research/records/2026-10-19-exp-reactive-mask-providers.md): provider 3.1.5 passes all 16 flags, identical-colour control, active-region shrink/grow and generated-mask consumption; 4.1.1/2.3.4 return OK but fail every mask-write criterion. Full experimental profiles remain failed; no fallback or generic support claim |
| U02 — Remaining Query routes and Configure controls | ✓ Verified (bounded integration) | Final release-archive verification; [D028](docs/adr/D028.md), [D029](docs/adr/D029.md) | All five typed tuning controls, mandatory domains/default requests and selected Query routes are implemented for both families. [Windows fixtures/MSRV/MSVC ABI and native integration](docs/release/2026-10-19-windows-upscaler-verification.md) pass: 20 value/default → dispatch/fence frames on 4.1.1. Acknowledgement does not establish numerical application or effective defaults |
| R04 — Provider/version selection | ✓ Verified (bounded integration) | Final release-archive verification; accepted [D028](docs/adr/D028.md) | Both-family selected/default setup queries, retained chains and live identity pass [Windows fixtures/MSRV/MSVC ABI and native execution](docs/release/2026-10-19-windows-upscaler-verification.md), including two GPU frames per family with the explicitly selected observed default ID. Generator probes additionally select 3.1.5/2.3.4 and retain their different write outcomes. No universal provider compatibility claim |
| R03 — Callbacks and custom allocation | ✓ Verified (bounded SDK access; selected for 0.3.0) | Final release-archive verification | Accepted [D040](docs/adr/D040.md) implements both families with external targets, all host/DX12 callback routes and conservative failure retention. [192 CI, 169 MSRV and 15 native stages](docs/release/2026-10-25-callbacks-diagnostics-verification.md) pass: 12 contexts/20 fenced frames, production CB exchange, balanced host/resource cleanup and compatible shared replacement. Heap ABI/controlled invocation passes; SDK heap invocation remains unobserved |
| U04 — Upscaler diagnostics | ✓ Verified (bounded SDK access; selected for 0.3.0) | Final release-archive verification | [D040](docs/adr/D040.md) implements independent creation/frame flags, matching memory estimates and unsafe modern/legacy message routes in both families. [Verification](docs/release/2026-10-25-callbacks-diagnostics-verification.md) observes 16 messages, modern global success and legacy live code 0 on both providers; null-context legacy code 2 is preserved. General quiescence/threading guarantees remain caller obligations |
| U05 — Resource formats and explicit motion scales | ✓ Verified (bounded SDK access) | Final release-archive verification | D034/D035/D037/D038 plus [D039](docs/adr/D039.md) complete the applicable role/format/view accounting and explicit scales. [Final verification](docs/release/2026-10-24-resource-formats-verification.md) passes 182 CI, 159 MSRV and 280 native stages: both public owners, 270 contexts and 810 fenced frames. Fixed typeless views, SNORM/shared-exponent colour, load-only RGB32F and D16/D24 are implemented; device/provider-conditional limits and documented non-applicability are explicit. No image/algorithm comparison gate |
| U06 — Early validation and recovery ergonomics | ✓ Verified (bounded integration) | Final release-archive verification; [D032](docs/adr/D032.md) | Mandatory tuning domains and both-family preflight share production validation/eligibility. [Windows fixtures, stable/MSRV consumers/examples and Configure/dispatch execution](docs/release/2026-10-19-windows-upscaler-verification.md) pass. [Renderer workflows](docs/DX12_RECOVERY.md) retain explicit idle/abort/recreate obligations; no recovery guarantee or implicit GPU wait is added |
| U07 — Upscaler setup and acquisition boundaries | ✓ Verified (bounded integration) | Accepted [D042](docs/adr/D042.md) and [concrete contract](docs/design/upscaler-setup.md), approved 2026-10-27 | Setup-bound planning/create, one selection, independent context retention, split loader/effect trust and removed old facade. [469 stable/MSRV/consumer/ABI/native cases plus 15 callback cases pass](docs/release/2026-10-27-upscaler-setup-verification.md) with unchanged inputs. Final archives remain separate |
| U08 — Portable frame parameters and named resources | ✓ Verified (bounded integration) | [D042](docs/adr/D042.md); verified U07 | Portable upscale/generator parameters, named DX12 resources, explicit exposure, one motion/colour request, portable sharpening and mandatory reinspection. [541 source/MSRV/consumer/native, 166 format/generator, 102 motion and 15 callback cases pass](docs/release/2026-10-27-frame-composition-verification.md). Final archives remain separate |
| U09 — Checked tuning requests | ✓ Verified (bounded integration) | [D042](docs/adr/D042.md); existing U02 domains and Configure contract | Portable private-field `TuningRequest`, one constructor validator, complete-list staging and canonical both-family operations. [557 stable/MSRV/consumer/ABI/native cases pass](docs/release/2026-10-27-tuning-request-verification.md), including exact bits/defaults, ordered native partial failure, terminal cleanup and 20 native value/default frames. Final archives remain separate |
| U10 — Composable errors and provider metadata | ✓ Verified (bounded integration) | [D042](docs/adr/D042.md); [accepted contract](docs/design/errors-and-provider-metadata.md) | Six owned error categories, exact codes/roles/original sources, retained first terminal cause and portable provider/mask composition. [583 stable/MSRV/consumer/ABI/native, 166 format/generator and 15 callback cases pass](docs/release/2026-10-27-error-metadata-verification.md), with the recorded callback-test-helper source delta. U11 and final archives remain separate |
| U11 — Terminal usability and recording abandonment | ✓ Verified (bounded integration) | [D042](docs/adr/D042.md); U10 terminal-cause representation; existing U06 recovery contract | `ContextUsability`, pure `usability()` and local `mark_recording_abandoned()` retain the first cause and reject live use before inspection/FFI. [593 stable/MSRV/consumer/ABI/native cases pass](docs/release/2026-10-27-context-usability-verification.md), including both-family abandoned native recordings and controlled cleanup/retention. No GPU-readiness or recovery claim |
| U12 — API transition integration and consumer migration | ✓ Verified (bounded source integration) | U07–U11; existing bounded feature evidence | Canonical API, current package README, [migration guide](docs/MIGRATING_0_3.md), targeted discard diagnostics and extensible DebugLevel. [625 stable/MSRV/consumer/ABI/native cases pass](docs/release/2026-10-27-api-transition-verification.md), including six core workflows and both optional utility compositions. P08 is closed with accepted limits; common rc1 host/archive qualification now passes. Final performance-input reconciliation is accepted; publication simulation passes; release closeout remains |
| U13 — Resource inspection module organisation | ✓ Verified (bounded source/native integration) | U12; existing D035–D039 resource contracts | Ordered frame inspection with private format/view, metadata conversion and COM/device inspection modules. All 20 function bodies/signatures and three data declarations preserved; public paths, validation/error order, COM guard lifetimes and descriptor bits unchanged. [625 integrated, 794 targeted native and seven archive checks pass](docs/release/2026-10-29-resource-inspection-verification.md), plus successful Cargo package verification. Common rc1 exact-package qualification now passes; final performance-input reconciliation is accepted |
| U14 — Portable common type organisation | ✓ Verified (targeted source checks) | U13; D042 canonical portable paths | Common facade reduced from 1,050 to 67 lines with private camera, diagnostics, dimensions, options, profile, scalar and tuning modules. Canonical re-exports, privacy, defaults and mapping bodies preserved. [Targeted Stable/MSRV compilation, Clippy, common/integration tests, source-fixture loading, Rustdoc and package listing pass](docs/release/2026-10-29-common-organization-verification.md). The subsequent common rc1 qualification now passes on Windows, Deck and Machine |
| U15 — Internal composition and documentation integration | ✓ Verified (targeted source integration) | U13, U14 | Descriptor construction borrows `FrameParameters`; old test owner methods and fake resource constructors are removed in favour of direct canonical calls/builders. Workload values and raw routes preserved; active-output Rustdoc/package/migration wording reconciled. [Targeted Clippy/MSRV, controlled and portable tests, example compilation and Rustdoc checks pass](docs/release/2026-10-29-composition-cleanup-verification.md). Source-level performance impact recorded; subsequent common rc1 qualification passes. Final P08 input/impact reconciliation is accepted; publication simulation passes separately |

The [2026-10-19 Windows follow-up](docs/release/2026-10-19-windows-upscaler-verification.md)
closes R04/U02/U06's bounded Windows evidence gaps after the earlier deferral.
U01 has passing Windows fixtures/MSVC ABI and public 3.1.5 generation, while
4.1.1/2.3.4 mask-write failures remain explicit. M14 variable output (P1-B) now
implements its accepted contract with [both-family public-path verification](docs/release/2026-10-19-active-output-verification.md).
U05 resource formats and explicit motion scales are complete for bounded SDK
access under D034/D035/D037/D038 and [D039](docs/adr/D039.md). The complete
source accounting and [public implementation verification](docs/release/2026-10-24-resource-formats-verification.md)
resolve the former residual families. No image-quality or provider-algorithm
research is required for this checkpoint. R03/U04 allocation, callback ownership
and diagnostics are now implemented under accepted [D040](docs/adr/D040.md).
Their [feature verification](docs/release/2026-10-25-callbacks-diagnostics-verification.md)
passes 192 CI, 169 MSRV and 15 public-native stages, including 12 contexts,
20 fenced frames, actual messages, exact production CB ABI and compatible
replacement/cleanup. Heap SDK invocation and general callback shutdown/failure
properties remain unproved; the exposed heap route has ABI/controlled coverage
and the unsafe contract retains those limits. D042's U07–U12 API transition now
precedes final coverage/release reconciliation and exact package verification.
Another broad online run or safe callback framework is not a prerequisite.
The subsequent
[archive-native coverage](docs/release/0.3.0-preparation.md#archive-coverage-before-final-verification)
tooling now binds callbacks and production CB exchange to extracted packages.
The common rc1 Windows batch passes those cases on Stable/MSRV and both public
families. Exact-pair host qualification, accepted P08 input reconciliation
and publication simulation are complete; publication remains separately authorised.

The [D024 output amendment](docs/adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19)
was accepted by maintainer review on 2026-10-19: optional active output, existing
maximum allocations, dynamic-option gating and explicit provider/write obligations.
Implementation and bounded both-family public-path verification are complete;
broader allocation access safety and final release archives remain separate.

The [API coherence checkpoint](docs/release/2026-10-18-api-coherence-verification.md)
passes local CI/MSRV, Windows compilation and isolated package checks for the
renamed methods, mandatory tuning validation and independently versioned sys
`0.2.0`. The subsequent Windows follow-up above passes the deferred fixtures and
bounded native checks; final archive verification remains separate.

The [Windows follow-up](docs/WINDOWS_FOLLOWUP.md) now has one consolidated
ABI/CI/MSRV/fixture/native command, including actual-provider Configure followed
by dispatch and fencing, plus a separate test-only D030 generator spike.
The raw spike remains independent of U01's public API. Separate public/default
and explicitly selected generator profiles now retain the provider-specific
passing and failing results linked above.

[API candidates](docs/research/records/2026-09-29-src-dx12-upscaler-api-surface.md) ·
[Direct provider controls](docs/adr/D016.md).

## Other SDK effects

### Frame Generation and presentation

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| FG01 — Effect and presentation contract | Explore | — | Map interpolation, preparation inputs, callbacks, swapchain/frame pacing and supported providers; choose an engine-independent integration boundary |
| FG02 — Bindings and ownership | Planned | FG01, V01 | Selected ABI with independent native checks; owned context/presentation types and explicit callback, thread and resource lifetimes |
| FG03 — Public integration | Planned | FG02 | Deterministic real/generated-frame sequence; presentation order, pacing, enable/disable, resize and cleanup exercised through the public API |

### Ray Regeneration

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| RR01 — Denoising contract | Explore | — | Pin inputs/signals, history, camera/material conventions and provider requirements; select a bounded first profile |
| RR02 — Bindings and ownership | Planned | RR01, V01 | Verified ABI, checked profile/context inputs and explicit resource/history ownership |
| RR03 — Public integration | Planned | RR02 | Repeatable noisy-scene input; recorded/submitted GPU output, history/reset controls and clean teardown |

### Radiance Caching

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| RC01 — Cache integration contract | Explore | — | Pin available SDK API, training/query phases, scene data and hardware requirements; select the renderer boundary |
| RC02 — Bindings and ownership | Planned | RC01, V01 | Verified ABI and cache lifecycle; explicit scene/buffer lifetime and update/query ordering |
| RC03 — Public integration | Planned | RC02 | Bounded scene update and cache-query sequence; native/GPU observations, reset/rebuild and cleanup controls |

## Backends and runtime

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| B01 — Vulkan native path | Selected for initial `fsr-sdk 0.4.0` slice; source path identified, implementation pending | R06, B03; trusted SDK 2.4 Vulkan runtime and named validation host; [SDK/API assessment](docs/research/platform-and-backends.md#sdk-24-and-release-sequencing); [D031](docs/adr/D031.md) | Keep the feature unsupported in 0.3.x. For `0.4.0`, verify concrete public context creation, bounded upscaling dispatch/readback and clean teardown, initially targeting Windows x64/Vulkan. Preserve Windows/DX12 isolation and meet the [source, ABI, native GPU and public-wrapper evidence gates](docs/research/platform-and-backends.md#b01-evidence-gates); native Linux needs its own artefacts and proof |
| B02 — Additional native targets | Explore | Suitable upstream artefacts | Select a concrete OS/architecture + effect/backend combination; verify loader ABI, deployment, execution and cleanup before support claims |
| R02 — Explicit SDK acquisition/deployment | Active | Runtime dependency closure/deployment audit | [D017](docs/adr/D017.md) selects the explicit pinned maintainer bootstrap; [real Windows installation, strict trust, offline reuse and acquired-subset native checks](docs/release/2026-10-12-windows-m10-sdk-verification.md) pass; broader deployment work remains open |
| R06 — Next SDK baseline review | Explore (2.4 source findings integrated; artefact/ABI audit open) | Acquire and inspect exact signed SDK 2.4 assets; pinned source commit identified in the [record](docs/research/records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md) | Complete the [v2.3.0 comparison checklist](docs/research/platform-and-backends.md#r06-comparison-checklist): exact artefacts/terms, all selected ABI layouts/tags and exposed Rust/interop changes, providers, deployment and OS/backend requirements. Deliver an explicit compatible-adoption or breaking-change disposition before R07; this is not an accepted pin or native proof |
| R07 — Compatible SDK 2.4 DX12 adoption | Planned; conditional `fsr-sdk 0.3.1` target | R06, released 0.3.0; reviewed D002/D017 adoption update | Adopt only with unchanged DX12 call sites, feature/MSRV and unsafe contracts and preserved 2.3 consumer behaviour; independently version any raw-crate change. Verify ABI, acquisition/dependency/provider changes, native DX12 and affected Proton regressions, exact packages and recurring D043 performance review. Move incompatible wrapper changes to 0.4.0; do not enable Vulkan here |
| B03 — Vulkan contract and feasibility preparation | Planned after 0.3.0; alongside conditional 0.3.1 work | R06 source/artefact basis; trusted runtime and suitable Windows/Vulkan host for the spike | Draft concrete loader/device/setup/resource/record usage, then a bounded raw native spike for handle encoding, image state/queue synchronization, real readback and cleanup. Feed findings into reviewed public design before B01. No generic backend trait, published Vulkan API, native Linux claim or movable promise follows from preparation |

## Verification and compatibility

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| V03 — Second hardware/provider configuration | Planned | Select applicable native integration protocols and acceptance scope | Deck and Steam Machine provide two bounded Proton GPU/provider baselines; [comparison](docs/research/platform-and-backends.md#observed-proton-execution) records their queries, single-frame execution and limits. Broader native integration coverage and acceptance remain open |
| V04 — GPU-based validation stall | Planned | Reproducible instrumented setup | Reduce the [known stall](docs/research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md); record cause/fix or a bounded unresolved result; rerun ordinary controls |
| V05 — In-flight recording and shared-context limits | Explore | V01 | Bounded recording-ahead, sibling and ordering cases; no inferred unlimited capacity or cross-queue guarantee |
| V06 — Movable native lifetime follow-up | Explore | Instrumented native setup | Investigate hidden activity/provider retention, worker exit and outstanding-work transfers; record limits and revisit [D013](docs/adr/D013.md) if contrary evidence appears |
| V07 — Proton DX12 compatibility verification | ✓ Verified | Accepted [D019](docs/adr/D019.md) | Initial V07a–V07c proof passes on [Deck](docs/research/records/2026-10-13-exp-proton-linux-cross-build.md) and [Machine](docs/research/records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md): cross-build, public queries, single-frame GPU/readback/cleanup and Vulkan core/synchronization checks. [Comparison](docs/research/platform-and-backends.md#observed-proton-execution) preserves Windows diagnostic limits; follow-ups below do not reopen this checkpoint |
| V07e — Proton bounded portability | ✓ Verified | V07 | The same query/single-frame protocol passes on Deck/provider 3.1.5 and Machine/provider 4.1.1; [environment/provider comparison](docs/research/platform-and-backends.md#observed-proton-execution) records differences and limits. This covers the bounded proof, not full regression portability |
| V08 — Proton runtime bootstrap and offline execution | ✓ Verified (local import) | Prepared local Proton/steamrt3 and Linux GPU host | Explicit bounded local import, full distribution inventories and enforced offline launcher execution pass on Deck; missing-input controls reject without downloads. [Two confirmations and policy limits](docs/research/records/2026-10-14-exp-proton-offline-runtime.md); remote acquisition and a pinned OS closure are outside this policy |
| V09 — Proton timeout cleanup | ✓ Verified (tested stack) | Linux bubblewrap/PID namespace and pidfds | Original group-only cleanup leaves 12 Wine/container processes; PID containment passes two fresh forced-expiry confirmations, preserves unrelated-prefix activity and reruns query/GPU controls. [Failure, correction and limits](docs/research/records/2026-10-14-exp-proton-timeout-cleanup.md); GPU/kernel hangs remain outside this proof |

## Performance

Completed bounded progression: **P03 → P04 → P05 → P02**, building on the completed
P01 baseline. P02 retains its ID for the extended supplied-mask/linear-HDR
comparison; its historical parity conclusion remains inconclusive. P06 completes
the separately selected bounded native characterisation: all five workloads meet
the 5 µs average additional CPU dispatch budget on the recorded stack. GPU
equivalence remains unproven. P06 is historical experiment evidence, not the
current candidate check. [D043](docs/adr/D043.md) now requires fresh performance
verification before every release; P08 tracks the first 0.3.0 check. This sequence
follows the
[performance synthesis](docs/research/performance.md#recommended-next-experiments)
and [source report](docs/research/records/2026-10-15-src-performance-testing-strategy.md).

P01's original pre-M14 measurement was deferred on 2026-10-14. Its subsequent
[CPU baseline](docs/research/records/2026-10-15-exp-wrapper-cpu-performance.md)
measures post-M14 source with fixed render/output, not pre-M14 performance or
active-size transitions. P03–P05 complete bounded comparisons/timing for the
implemented fixed-output profile; P05 uses M14's render slice. P02 completes the
selected masks/linear-HDR cases. Their completed checkpoints do not themselves
require repetition, but do not fulfil the newly selected P08 candidate check.
Variable output, M16's documented signal limitations and final release
scope/archive checks remain separate.

Use optimized builds, warm-up, repeated independent process blocks and recorded
SDK/provider, hardware, driver, workload and measurement boundaries. Keep CPU
recording, GPU queue intervals and lifecycle costs separate. Check matched inputs,
valid output and safe completion independently of speed. Initial results are
descriptive, including inconclusive comparisons; equivalence or non-inferiority
claims require a separately justified practical margin and uncertainty analysis.
P01–P05/P02 select no numerical speed gate. P06 separately selects a 5 µs
upper average CPU budget for its experiment; D043 does not automatically adopt
that number universally. The maintainer now explicitly selects the existing
5 us CPU criterion for P08, with the [P06/P07 adaptation](docs/design/release-performance.md),
an initial fresh pilot/confirmation and descriptive review of setup/GPU/lifecycle
results. Its inconclusive CPU outcome now leads to a separate fixed 256-block
confirmation; old observations supply planning data only.
No broad resolution/feature matrix or image-quality study is selected here.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| P01 — Wrapper Performance Baseline | ✓ Verified (bounded CPU) | Current post-M14 source; tested Windows/DX12 configuration | [Three controlled/native repetitions](docs/research/records/2026-10-15-exp-wrapper-cpu-performance.md) retain dispatch/reference/validation distributions, COM and Rust-allocation counts and separate lifecycle costs. Fixed-size local-owner profile only; no GPU timing, dynamic-size performance or independent C++ parity claim |
| P03 — Independent C++ CPU comparison | ✓ Verified (bounded comparison) | P01; recorded Windows/DX12 configuration | [Independent C++ and Rust comparison](docs/research/records/2026-10-15-exp-independent-cpp-cpu-comparison.md) passes all 45 protocol cases: matched descriptors/provider/device/output, six balanced process blocks and three warm-state controls. CPU differences remain noisy/sign-changing; precise overhead and speed parity are inconclusive. No GPU-time or full-P02 claim |
| P04 — 1080p GPU timing pilot | ✓ Verified (bounded timing) | P03's matched consumers/workload specification; native DX12 timestamp queries | [Matched Rust/C++ 1080p pilot](docs/research/records/2026-10-15-exp-1080p-gpu-timing.md) passes all 22 cases: provider-reported 1280×720 render, coherent jitter/motion, same-queue timestamps/frequency, empty brackets, fenced resolve/readback and matching output. Four process blocks retain separate CPU/GPU distributions; GPU medians 0.93808–0.94564 ms on the recorded stack. No speed-parity, FPS or quality claim |
| P05 — Active-render transition performance | ✓ Verified (bounded timing) | P04; M14 render slice; recorded Windows/DX12 configuration | [Matched Rust/C++ shrink/grow experiment](docs/research/records/2026-10-16-exp-active-render-timing.md) passes all 42 cases: six debug controls, four balanced blocks and 3,864 submitted/fenced frames, fixed allocations/output and independently audited matched-size inputs. Separate first-transition/settling/steady CPU/GPU measurements retain substantial variability; a consistent transition penalty or speed-parity claim is inconclusive. No variable output or allocation resizing |
| P02 — Native C++ Performance Parity | ✓ Verified (bounded comparison); parity inconclusive | P03–P05; M15 and M16 linear-HDR slice; recorded Windows/DX12 configuration | [Supplied-mask/linear-HDR comparison](docs/research/records/2026-10-16-exp-mask-hdr-performance-comparison.md) passes all 62 cases, with fixed/active-render controls, ten debug and forty timed children, 6,440 submitted/fenced frames, exact mask/input audits and matched Rust/C++ descriptors/outputs. Signed CPU/GPU differences vary across four balanced blocks; no equivalence margin or speed-parity claim. Broader M16 non-linear semantics and release acceptance remain separate |
| P06 — Native performance characterisation | ✓ Verified (bounded CPU budget and characterisation) | P01–P05/P02 matched consumers and local Windows/DX12 configuration | [Completed confirmation](docs/research/records/2026-10-16-exp-native-performance-confirmation.md): fresh pilot/confirmation pass 122/1,220 cases; 112 frozen randomised blocks establish all five one-sided CPU upper 95% bounds below 2.78 µs against the selected 5 µs average budget. Separate GPU/lifecycle intervals remain descriptive; zero Rust allocator events observed across 1,920 warmed calls. Earlier interrupted evidence preserved; current candidate release check is P08 |
| P07 — Provider setup-query cost | ✓ Verified (bounded measurement) | Recorded Windows/MSVC runtime/device; separate decision before optimization | [Three optimized native processes](docs/research/records/2026-10-19-exp-provider-setup-cost.md) pass all 11 cases each: four warm-ups per arm, 32 alternating paired blocks per process, 192 six-query timings. Per-process medians: default 229.55–230.74 ms, selected 696.98–707.10 ms. Whole-path costs, no GPU dispatch, speed gate or cache/session decision |
| P08 — Release candidate performance verification | ✓ Verified (bounded review; accepted exceptions) | U12 API closure; recorded candidate inputs; [D043](docs/adr/D043.md) | [Closure and maintainer disposition](docs/release/2026-10-29-performance-review-closure.md): fixed 2,660-case confirmation plus three 11-case setup runs pass; original full CPU outcome inconclusive, accepted 222-block temporal subset has nominal upper bounds below 5 us with selection limits disclosed. Supplements and 30-case provider comparison reviewed; remaining deployment/provider differences accepted as non-blocking. Common rc1 host qualification passes; [final rc1 reconciliation is accepted](docs/release/2026-10-29-rc1-performance-acceptance.md), retaining numerical confidence bounds and uncertainty. Publication simulation passes; release closeout remains |

## Optional utilities

These independently selected utilities preserve the native core and do not
resume deferred quality-policy work. UT01 is selected for 0.3.0; other utility
release inclusion needs separate selection.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| UT01 — FSR generation provider resolver | ✓ Verified (scoped implementation checks) | Accepted [D036 release amendment](docs/adr/D036.md#release-selection-amendment--2026-10-27); selected for 0.3.0 | Selected to ship with 0.3.0, still off by default. Portable `utils` resolver, strict name recognition, explicit ambiguity/no-fallback errors and public core composition. Seven stable/MSRV configurations and both-family fixtures pass; fresh stable/MSRV consumers each pass 141 cases. [Verification](docs/release/2026-10-22-provider-utils-verification.md) preserves initial broad-profile failures, focused rechecks and non-Windows/GPU limits |

## Status

| Status | Meaning |
|---|---|
| Explore | First deliverable is a scoped investigation/design; implementation details remain open |
| Planned | A completion checkpoint is described; work has not started |
| Next | Selected to start next; any required contract review is part of the work |
| Active | Work is actually underway |
| On hold | Work is deliberately paused; resumption needs explicit selection, and recorded failures remain unresolved |
| Blocked | The Needs cell names a missing external input |
| Verified (✓) | The row's checkpoint is checked; release/publication handoff remains where relevant |
