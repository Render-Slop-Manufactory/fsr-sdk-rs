# Roadmap

A rolling plan towards complete AMD FSR SDK coverage. Work is organized by
capability, with stable item IDs, explicit prerequisites and completion
checkpoints. Release versions are assigned when work is selected to ship.
[SCOPE.md](SCOPE.md) defines the goal; [decisions](docs/DECISIONS.md) own accepted
contracts; [roadmap history](docs/ROADMAP_HISTORY.md) retains completed work.

## Current state

The implementation baseline is **`fsr-sdk 0.2.0` / `fsr-sdk-sys 0.1.1`**:
partial DX12 upscaling, native sharpening, portable common inputs and optional
movable ownership. Native support remains Windows x64/MSVC with SDK v2.3.0.
The [release record](docs/release/2026-10-10-0.2.0-release-verification.md) describes
local verification; the maintainer's publication handoff remains open below.

The goal includes Upscaling, Frame Generation and presentation, Ray Regeneration,
Radiance Caching, runtime controls and upstream-supported backends. The queues
below distinguish further upscaler work from adding the other SDK capabilities.

## Next actions

The selected near-term queue, in priority order. An item appears in only one
table; promoting a later item moves its row here with the same ID.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| R01 — Publish verified baseline | Next | Maintainer publication | Recheck final-commit packages; publish sys 0.1.1 before wrapper 0.2.0; confirm hosted docs and matching GitHub release |
| V01 — Temporal/image baseline | Next | Existing upscaler harness | Repeatable longer motion, disocclusion, jitter and reset sequences; predeclared comparisons and retained readbacks |
| M10 — Provider metadata and selected Queries | Next | Contract review | Owned names/opaque IDs and selected read-only queries; ABI, lifetime, consumer and native checks; provider override stays separate |

Complete V01 before selecting another large effect implementation. Investigation
and smaller independent tasks can proceed alongside it. No later row is a
commitment to a particular crate version or an already accepted API design.

## Upscaling

Extend the existing effect in small, independently checked steps. M10–M16 keep
their existing milestone IDs; numbers are identifiers, not a global execution
order. The first rows are the preferred progression within this track.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| M11 — Pre-exposure | Planned | V01 | Finite-positive scalar, default 1.0; equivalent scenes remain comparable after normalization |
| M12 — Depth and motion-vector profiles | Planned | V01 | Typed conventions/units; controlled camera, motion-scale and jitter comparisons |
| M13 — Automatic exposure | Planned | M10, V01 | Explicit creation/input contract; provider-aware luminance comparisons and resource validation |
| M14 — Active render/output extents | Planned | V01 | Allocation/maxima/active bounds; sentinel checks and temporal size transitions |
| M15 — Supplied masks | Planned | M10, V01 | Provider-aware mask requirements; transparency and disocclusion comparisons |
| M16 — HDR and non-linear color | Planned | M11, V01 | Explicit encoding/format/exposure contract; bounded provider and tonemapping comparisons |
| U01 — Automatic reactive masks | Explore | M15 | Separate GPU operation and ownership contract; generated/supplied mask controls |
| U02 — Remaining Queries and Configure controls | Explore | M10 | Inventory remaining descriptors/helpers, select typed APIs and verify provider-specific semantics |
| U03 — Quality/size/jitter helpers and presets | Explore | V01, M10 | Justified helpers and reproducible calibration; preserve direct native controls and document policy separately |

[Existing surface research](docs/research/records/2026-09-29-src-dx12-upscaler-api-surface.md)
supplies candidates, not accepted contracts. [D016](docs/adr/D016.md) keeps direct
provider controls distinct from SDK-selected quality policies.

## Other SDK effects

Each effect has its own discovery, API and integration checkpoints. These tracks
are all in scope; table order does not choose which effect ships first. Discovery
must pin an upstream surface and the usable hardware/runtime requirements before
implementation scope or public contracts are accepted.

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

The [pinned AMD SDK inventory](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0)
lists these technologies; Radiance Caching is a preview in that baseline. Their
presence upstream does not establish Rust coverage here.

## Backends and runtime

Backend support and effect coverage are separate dimensions. A working upscaler
does not establish another effect or backend, and compiling on an OS does not
establish native runtime support. Keep portable inputs usable independently.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| B01 — Vulkan native path | Blocked | Usable upstream integration | Pin a supported SDK/runtime path; then define device retention, resource metadata, layouts, queue-family/synchronization and cleanup contracts |
| B02 — Additional native targets | Explore | Suitable upstream artifacts | Select a concrete OS/architecture + effect/backend combination; verify loader ABI, deployment, execution and cleanup before support claims |
| R02 — Explicit SDK acquisition/deployment | Explore | Distribution review | Choose local/offline and optional verified acquisition paths; pin artifacts, integrity, cache behavior, notices and runtime dependency closure |
| R03 — Callbacks and custom allocation | Explore | Concrete ownership use case | Separate host/backend allocation and message callbacks; prove callback lifetimes, panic containment, threading and teardown |
| R04 — Provider/version selection | Explore | M10 | Explicit override semantics, retained identity and provider compatibility checks; no silent version-dependent policy |
| R05 — Complete SDK-surface inventory | Planned | Pinned SDK v2.3.0 | Map all public slices; distinguish raw declarations, wrapper APIs and native coverage; turn remaining gaps into scoped queue items |
| R06 — Next SDK baseline review | Explore | Candidate upstream version | Compare one candidate with the pin; review migration/artifacts/notices and verify affected ABI, consumers and native paths before adopting it |

B01 is blocked by the selected upstream path: AMD's
[pinned v2.3.0 documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0)
explicitly marks Vulkan unsupported. An older SDK path or a new pin needs its
own compatibility decision. [Platform findings](docs/research/platform-and-backends.md)
retain the current evidence and loader hazards.

## Verification and compatibility

V01 starts this track from the near-term queue. The work below expands what can
be claimed about correctness, provider behavior and compatibility independently
of adding API surface. Native results remain scoped to recorded configurations.

| Item | Status | Needs | Completion checkpoint |
|---|---|---|---|
| V02 — Sharpening image-effect follow-up | Planned | V01 | Predeclare bounded scenes/metrics; distinguish native execution from image effect and retain failures as well as passes |
| V03 — Second hardware/provider configuration | Blocked | Access to another configuration | Run the same protocols, record SDK/provider/GPU/driver inputs and compare results/limitations |
| V04 — GPU-based validation stall | Planned | Reproducible instrumented setup | Reduce the known dispatch stall; record cause/fix or a bounded unresolved result; rerun ordinary controls |
| V05 — In-flight recording and shared-context limits | Explore | V01 | Bounded recording-ahead, sibling and ordering cases; no inferred unlimited capacity or cross-queue guarantee |
| V06 — Movable native lifetime follow-up | Explore | Instrumented native setup | Investigate hidden activity/provider retention, worker exit and outstanding-work transfers; record limits and revisit the accepted contract if contrary evidence appears |

V02 starts from the [failed original effect criteria](docs/research/records/2026-09-30-exp-d014-native-sharpening.md),
which remain failed. V04 retains the [GBV stall](docs/research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md).
V06 follows [the accepted movable contract](docs/adr/D013.md); these investigations
are not new blockers imposed on the already verified release.

## Keeping the roadmap rolling

| Status | Meaning |
|---|---|
| Explore | First deliverable is a scoped investigation/design; implementation details remain open |
| Planned | A completion checkpoint is described; work has not started |
| Next | Selected to start next; any required contract review is part of the work |
| Active | Work is actually underway |
| Blocked | The Needs cell names a missing external input |
| Verified | The row's checkpoint is checked; release/publication handoff remains where relevant |

1. Keep one row per item. Preserve IDs when moving rows; never renumber M10–M16
   or reuse an archived ID. New ideas go at the bottom of the relevant track.
2. Promote a row into **Next actions**, rather than copying it. Dependencies
   refer to stable IDs, including completed prerequisites in roadmap history;
   the Needs cell also names any external requirement.
3. Update status and checkpoint as work progresses. Accept consequential
   contracts through the decision process; a roadmap row alone accepts no API,
   SDK pin, safety or platform claim.
4. A feature checkpoint needs source, ABI/consumer checks, documentation and
   native evidence appropriate to the claimed behavior. An investigation ends
   with its bounded findings and a recorded follow-up or limitation.
5. At release close, move shipped/finished rows to **roadmap history**, recording
   the release/date and completion evidence. Update the baseline and README
   coverage table; keep unfinished rows and their order. Completed investigations
   may be archived independently, without inventing a release assignment.
6. Append new scoped work below the remaining queue. Routine releases change
   rows and baseline metadata; they do not replace the plan with a new before/
   after-release narrative. Repeat release checks as needed without making a
   permanent milestone for every routine test run.
