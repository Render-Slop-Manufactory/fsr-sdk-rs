# Roadmap

This document owns current project status, priorities, and sequencing.
[SCOPE.md](SCOPE.md) defines durable goals and boundaries. Source and
manifests remain authoritative for implemented behavior. Technical choices belong
in [decisions](docs/DECISIONS.md), with supporting [research](docs/research/README.md).

## Current state

Both crates are published as `0.1.0` on crates.io, with Windows-target API
documentation on docs.rs and the public GitHub `v0.1.0` release dated 2026-09-28.
The [publication check](docs/release/2026-09-29-publication-check.md) records the
public links, publication order and registry checksums.

The project implements DX12 upscaler construction and a narrow public unsafe
dispatch method. The supported native target is `x86_64-pc-windows-msvc` with
`dx12`. Windows/DX12 APIs are exposed on other Windows targets, but those targets
are currently unverified and unsupported; compilation may succeed.
[SCOPE.md](SCOPE.md#platform-and-compatibility-boundaries) owns this support policy.
Native verification is limited to the SDK, runtime/provider, GPU, driver and
workload configurations recorded in the verification records; target support
does not imply compatibility with every hardware/provider configuration.
[M6b](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md) recorded
one public-route native dispatch and plausible GPU readback on a tested
configuration; no image-quality or broad hardware-support claim is available.
The [final Windows verification](docs/research/records/2026-09-28-exp-final-windows-release-verification.md)
repeated that public-route regression and passed three ordered moving frames
through one upscaler on the recorded AMD configuration.

| Area | Current status |
|---|---|
| Selected FidelityFX ABI and explicit DLL loader | Implemented; bounded Windows x64/MSVC checks passed |
| Public DX12 runtime and upscaler construction | Implemented; native lifecycle passed on one tested configuration |
| Public DX12 upscaler dispatch | Implemented under D011; one submitted, fenced frame and readback passed on that configuration |
| Temporal multi-frame behavior | Three-frame moving smoke and [controlled motion/jitter/history-reset comparisons](docs/research/records/2026-09-30-exp-dispatch-semantics.md) passed on the recorded AMD configuration; general temporal reconstruction accuracy and image quality remain unverified |
| Consumer documentation and examples | Windows-target Rustdoc published on docs.rs for both `0.1.0` crates; DX12 example compiled locally for Windows |
| Packaging and release validation | Both `0.1.0` crates published on 2026-09-28, sys then wrapper; [publication verified](docs/release/2026-09-29-publication-check.md) |
| Public Git history | Public repository and `v0.1.0` release/tag available |
| Vulkan and other SDK effects | Not implemented; future work |

**Current focus:** Plan a small `0.2.0` around dispatch-input extensibility and
one contained addition: sharpening. Maintain the published `0.1.0` as the
regression baseline. [D014](docs/adr/D014.md) was accepted on 2026-10-07;
M7's API and migration contract is complete. Implementation and the separate
native release protocol remain pending. The original native effect gate failed
and stays failed. Complete and verify the small `0.2.0`, then broaden existing
upscaler evidence before opening new effects.

Current source also separates common checked upscaler inputs from the concrete
DX12 backend ([D015](docs/adr/D015.md)), preserving existing imports. Common
types are available on every host without backend features; native APIs retain
their Windows/DX12 gates. [Portable-common verification](docs/release/2026-10-09-portable-common-verification.md)
records consumer, feature, ABI and selected native regressions. It adds no Vulkan
support or D014 functionality and does not select a release.

- [x] Two-crate Cargo workspace: `fsr-sdk-sys` and `fsr-sdk`.
- [x] Initial module scaffolding for ABI, loading, context, errors, queries, and upscaling.
- [x] DX12 feature wiring and Windows loader gating; Vulkan explicitly rejects
  compilation and is not an implemented backend.
- [x] Contribution, development, decision, research, and agent-routing setup.
- [x] Consolidate durable goals in SCOPE.md and finish documentation routing.
- [x] Pin AMD FSR SDK `v2.3.0` as the initial native baseline
  ([D002](docs/DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).

The first common ABI slice is implemented: context and scalar aliases, return
constants, base descriptor headers, allocation callbacks, and the five native
function-pointer types ([D003](docs/DECISIONS.md#d003--handwrite-the-first-common-abi-slice)).
The explicit-path Windows/DX12 loader now owns the library and exposes five
unsafe forwarding methods with private function pointers
([D004](docs/DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points)).
The private M4 lifecycle owner, retained upscaler creation state and DX12 device
ownership are implemented. M5 adds explicit public runtime acquisition and safe
checked upscaler construction, with shared loader retention and independent COM
device ownership. M6b adds the bounded unsafe public dispatch contract. M5 has
Windows-target Rust compilation, host ownership fixtures, Windows CI fixture execution, and both
opt-in native lifecycle cases passing on the tested Windows x64/MSVC machine
([M5 verification](docs/research/records/2026-09-24-exp-m5-windows-native-verification.md)).
Windows M4 fixture execution, paired
native ABI checks and the real production-owner lifecycle now pass on the tested
Windows x64/MSVC configuration
([M4 verification](docs/research/records/2026-09-22-exp-m4-windows-verification.md)).
An opt-in, experiment-only Windows/DX12
[context lifecycle probe](docs/research/records/2026-09-20-exp-context-lifecycle.md)
has successfully created, identified and destroyed a provider 4.1.1 context.
Its creation ABI is now promoted into sys, while provider-identification ABI
remains test-local. The ordinary lifecycle runner now targets the production
owner; its Windows execution succeeded with provider 4.1.1. The later
[M6a proof](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
established one GPU dispatch and readback through the public M5 constructor.
The first ABI verification is preserved in an
[experiment record and API/safety synthesis](docs/research/api-and-safety.md).
The local AMD loader DLL was loaded and all five symbols resolved.
[Loader evidence](docs/research/records/2026-09-20-exp-explicit-dll-loader.md)
covers fixture-based errors, forwarding and library release. A separate
[count-only version query](docs/research/records/2026-09-20-exp-version-count-query.md)
now succeeds without a context/device, using a locally staged loader and upscaler
DLL beside the test executable. This does not establish GPU compatibility.
The common ABI has Rust tests and a
compile-only local header check. Checks and their limits
are documented in [development](docs/DEVELOPMENT.md); passing builds do not imply
runtime support.

## Implementation sequence

Milestone 1 has established the baseline needed for the common ABI and explicit
loader; broader investigation remains open. Milestone 2 is complete for that
bounded scope. Milestone 3 has its count-only smoke test completed. Milestone 4
is complete for its bounded private Windows/DX12 ownership scope; M5's public
construction path has Windows fixtures and both opt-in native cases passing on
the tested configuration; [M6a](docs/MILESTONE_6.md) has a private dispatch
implementation and one submitted/fenced GPU readback from the
[recorded run](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md).
The public dispatch contract is accepted in [D011](docs/adr/D011.md); M6b
implementation and [public-route verification](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
are complete on one tested Windows/DX12 configuration. These milestones
are not release dates or support promises.

| Milestone | Work | Completion evidence |
|---|---|---|
| 1. Establish the native baseline | **Done:** pin SDK `v2.3.0` (D002), verify the common ABI (D003), identify the API DLL and select explicit-path loading (D004), select the handwritten-by-default policy with reviewed generated exceptions ([D006](docs/DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)), and select explicit acquisition and executable-adjacent upscaler deployment for the bounded DX12 baseline (D007). **Pending:** broader safety contracts, effect runtime requirements beyond that baseline, applicable distribution terms and concrete additional binding slices. | Version-pinned research and decisions in the [API/safety synthesis](docs/research/api-and-safety.md). |
| 2. Minimal ABI and loader | **Done for Windows x64/MSVC:** common ABI and explicit-path loader with five private entry points and retained library ownership. | Paired ABI checks, Windows build, successful local AMD DLL symbol resolution, fixture-based failure/forwarding/unload checks and ownership compile-fail tests. |
| 3. Native query | **Done for Windows x64/MSVC:** one global count-only upscaler version query with its descriptor and two tags. | Opt-in local runtime test returned OK and count 2; paired ABI checks and both failed/successful deployment observations are [recorded](docs/research/records/2026-09-20-exp-version-count-query.md). |
| 4. Context ownership | **Done for the tested Windows x64/MSVC configuration:** private creation, terminal destruction, dependency retention and error conversion. | Windows fixtures, paired native ABI compilation and production-owner AMD create/query/destroy passed; [recorded inputs and limits](docs/research/records/2026-09-22-exp-m4-windows-verification.md). No dispatch or public constructor. |
| 5. Public DX12 construction | **Done for the tested Windows x64/MSVC configuration:** shared explicit runtime owner, checked dimensions, borrowed device with independent COM retention, safe upscaler constructor and terminal teardown. Query/Configure remain outside the accepted D008 construction scope. | Host ownership fixtures and Windows-target compilation passed; ordinary Windows CI fixtures passed in the [recorded run](docs/research/records/2026-09-24-exp-m5-windows-ci-verification.md). Both opt-in two-context native cases passed with [recorded inputs and limits](docs/research/records/2026-09-24-exp-m5-windows-native-verification.md). No dispatch. |
| 6. Minimal native DX12 dispatch proof | **M6a and M6b done for the tested Windows x64/MSVC configuration:** narrow dispatch ABI, private proof followed by D011's borrowed public unsafe dispatch input, validation, and poison/error contract ([M6 plan](docs/MILESTONE_6.md)). | Paired ABI checks and AMD helper parity passed. The [M6a record](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md) and [M6b public-route record](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md) each show OK dispatch, submission/fence and a 230,400-pixel finite changed readback with zero debug errors and no device removal on one setup. GPU-based validation remains inconclusive. |

The first native milestone is loading, resolving symbols, and querying the
runtime. It does not require image reconstruction or a complete public API.

## Proposed 0.2.0 milestones

The [2026-09-30 maintenance verification](docs/release/2026-09-30-review-verification.md)
adds dimensions accessors, smaller implementation/test modules, pinned-header CI
checks and Rust 1.88 CI coverage. The [semantic comparison](docs/research/records/2026-09-30-exp-dispatch-semantics.md)
adds bounded motion/jitter/history-reset evidence and corrects the test image
generator. These maintenance changes do not implement the 0.2.0 frame or
sharpening extension below.

Replace public `UpscaleDispatch` literals with a private, extensible frame
representation and keep one dispatch path. Sharpening is the only new feature:
a useful frame scalar requiring no new resources or creation flags.

Basis: the [dispatch API synthesis](docs/research/upscaling-dispatch.md#broader-frame-surface-and-api-evolution)
and [post-release surface record](docs/research/records/2026-09-29-src-dx12-upscaler-api-surface.md).
These repository milestones continue M1–M6, independently of the record's
numbering. D011 remains the implemented 0.1.0 baseline; D014 selects its limited
0.2.0 replacement. D012 already covers creation-option extensibility.

The accepted [D014 design](docs/adr/D014.md) supplies M7's API contract,
migration, alternatives and verification gates. M7 was completed on 2026-10-07;
the frame redesign and sharpening are not implemented.
Its [native sharpening spike](docs/research/records/2026-09-30-exp-d014-native-sharpening.md)
passed execution and repeat controls, but failed the predeclared effect criteria
in all three repetitions, confirmed by an independent readback audit. The original
failure remains evidence. API acceptance does not select the new release protocol.
Its scenes, controls, metrics, thresholds and repeat policy must be recorded
before execution, explicitly informed by the known results and including
same-context sharpening switches. No new run retrospectively passes the original gate.

| Milestone | Scope | Done when |
|---|---|---|
| M7 — Dispatch API contract | **Done 2026-10-07:** D014 accepted for private fields, construction/access methods, optional defaults and migration from `0.1.0` literals. | Contract approved; one unsafe dispatch route; GPU-lifetime, poison and cleanup obligations preserved. Implementation remains M8. |
| M8 — Extensible frame and sharpening | Implement the frame API and typed sharpening: disabled by default, finite `[0,1]` strength. Migrate examples. | Host/Windows checks cover defaults, rejection before FFI, descriptor mapping and existing failure paths. |
| M9 — Verification and release | Predeclare the separate sharpening release protocol; repeat native regressions and public-route comparisons, including mode switching within one context. | Exact descriptor mapping and recorded native execution/effect under the selected protocol; finite readback, fenced cleanup, no debug errors/device removal; migration, Rustdoc and package/consumer checks complete. Publication remains separately authorized. |

`0.2.0` does not include pre-exposure, automatic exposure, masks, changing active
extents, new depth/MV conventions, HDR/non-linear color, public Query/Configure,
provider override, movable public types, callbacks or custom allocation.
Future mandatory inputs can still require a break; native comparisons establish
only their recorded scope.

## Milestones after 0.2.0

Proposed scopes, without committed release versions. Compatible additions can
use `0.2.x`; a breaking contract needs another minor version. Reorder independent
work for concrete requirements and add one bounded capability at a time.

Before choosing the next large SDK slice or new effect, extend the existing
upscaler's in-repository example/test harness with longer moving sequences and
image comparisons. Keep it part of `fsr-sdk-rs` and preserve reproducible workload,
runtime/provider, GPU and driver inputs. Seek a second hardware/provider
configuration where available; record unavailable coverage explicitly. This
is a later evidence step after M7–M9, not a separate renderer project or a completed
image-quality claim.

| Milestone | Scope | Required boundary/evidence |
|---|---|---|
| M10 — Provider metadata and selected Queries | Owned names/opaque IDs; choose needed resource, memory, quality or jitter utilities. | Required ABI slices, routing and lifetimes verified. Provider override reviewed separately. |
| M11 — Pre-exposure | Finite-positive frame scalar, default `1.0`. | Equivalent scenes at different factors produce comparable normalized outputs. |
| M12 — Depth/MV profiles | Typed creation conventions and matching frame units; one profile at a time. | Controlled depth/motion tests cover camera rules, MV scale and jitter handling. |
| M13 — Automatic exposure | Typed creation mode and matching exposure input contract. | Provider resource requirements plus controlled luminance comparisons; explicit mode remains the default. |
| M14 — Active extents | Dynamic render resolution first; variable output verified separately. | Allocation/maxima/active bounds, sentinel regions and temporal size transitions checked. |
| M15 — Supplied masks | Optional reactive and transparency/composition resources. | Provider-aware requirements and transparency/disocclusion comparisons. |
| M16 — HDR/non-linear color | Bounded creation, encoding and format/provider profile. | Comparative image-semantic tests with explicit exposure/tonemapping rules. |

Configure tuning, debug callbacks/visualization, automatic mask generation,
custom allocators, SDK acquisition, other effects and backends remain separate
backlog items. Assign their milestones when a use case and contract justify
them. Controlled temporal tests accompany relevant capabilities.

### Independent D013 track

[D013](docs/adr/D013.md) was accepted on 2026-10-09 after soundness review and
contract revisions. Implementation remains outside `0.2.0` release scope and
is not a gate for M7–M9; no later release version is selected.

1. **D013-A — Evidence:** [compile/host probes passed](docs/research/records/2026-09-29-exp-d013-movable-compile-host.md).
   [Windows fixtures passed](docs/research/records/2026-09-29-exp-d013-windows-fixtures.md).
   The [bounded native spike](docs/research/records/2026-09-30-exp-d013-native-thread-transfer.md)
   passed all three serialized, fenced transfer scenarios, their same-thread
   controls and existing native regressions. Siblings, early external-runtime
   drop and final loader release on another thread were covered. The
   [failure/cleanup fixtures](docs/research/records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
   add transferred poison, preflight/recovery, terminal retention and coordinated
   unwind. The [extended native follow-up](docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md)
   passes all 11 prerequisites and 30 new children, covering creation after
   runtime transfer, 64-frame history, independent siblings and successful fenced
   Rust-unwind cleanup. The provider persists after all audited application refs
   and worker exit. These bounded spikes are complete; provider/dependency
   termination, native returned-error/partial-load cleanup, hidden work,
   external-user coordination, outstanding GPU work, other configurations and
   sanitizer coverage remain open.
2. **D013-B — Review: done 2026-10-09.** The
   [soundness review](docs/research/records/2026-10-09-src-d013-soundness-architecture-review.md)
   revisions are incorporated and the design accepted for framework-independent
   renderer-state migration. Remaining native assumptions are explicit in
   [D013's safety contract](docs/adr/D013.md#conditional-safety-contract);
   no unresolved design soundness blocker or further general research gate remains.
3. **D013-C — Optional API: done 2026-10-09.** The optional `movable` family,
   concrete field/cleanup audit, Rustdoc and consumer coordination example are
   implemented. [Verification](docs/release/2026-10-09-d013-public-api-verification.md)
   passes public traits/features/family controls, shared-core failure fixtures,
   independently held siblings, unwind/failed-handoff cleanup and selected fenced
   native transfers, including last runtime release and local-family regressions.
   Select its later release and verify artifacts separately; these results
   establish no universal AMD compatibility.

## Deferred work

- Broader dispatch behavior: temporal reconstruction, varying exposure, other
  formats and providers need separate evidence before broadening D011's profile.
- Automated SDK acquisition: decide download permission, pinned versions,
  checksums, cache behavior, offline use, and redistribution before implementing.
  This is separate from dynamic runtime loading.
- CI: keep the existing host and Windows compilation checks useful as implementation grows;
  keep hardware-dependent evidence separate from ordinary build jobs.
- Future releases: maintain compatibility documentation, package checks,
  examples, and required third-party notices as the implementation grows.
- Additional backends and broader SDK coverage: revisit only when upstream
  capabilities and a concrete use case justify them.

## Next actions

1. **Status/test hygiene — completed 2026-10-07:** current plans now reflect
   completed D013/D014 evidence, the broken protocol link is repaired and
   fixture directories tolerate stale paths.
   Verification is recorded in the [2026-10-07 maintenance check](docs/release/2026-10-07-status-test-hygiene.md).
2. **D014 / M7 — completed 2026-10-07:** API, migration and bounded native-pass
   trust accepted after review. The original failed experiment remains failed;
   the separate release protocol is still open.
3. **M8–M9:** implement and verify a deliberately small `0.2.0`: extensible frame
   API, sharpening and migration. Select and record the separate release
   protocol before native execution, including same-context mode switching.
   Repeat native regressions, the selected
   sharpening protocol, documentation and package/consumer checks on the final
   implementation and artifacts. Maintain published `0.1.0` consumer guidance.
   Publication remains separately authorized.
4. **Broader existing-upscaler evidence:** extend the in-repo example/test harness
   with longer moving sequences, image comparisons and, where available, a second
   hardware/provider configuration before choosing the next large SDK slice.
5. **D013 independently:** design, optional API and bounded public-path
   verification completed on 2026-10-09. Select a later release and verify its
   artifacts separately. Native assumptions remain caller obligations under
   [D013](docs/adr/D013.md), not verified AMD guarantees. This track stays
   outside `0.2.0`.

The [GBV spike](docs/research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md)
localized a timeout to native `ffxDispatch` on the tested setup; its
provider/driver/validation cause remains unresolved. Prior
[candidate artifact checks](docs/release/2026-09-28-final-artifact-recheck.md)
and the separate [publication check](docs/release/2026-09-29-publication-check.md)
remain historical baselines, not validation of future releases.
