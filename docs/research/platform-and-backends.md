# Platform and graphics backends

- Updated: 2026-10-29 (common rc1 Windows/Proton qualification, SDK 2.4 Vulkan/API
  source findings and conditional release sequencing integrated).
- Evidence: [platform/backend investigation](records/2026-10-09-src-platform-backend-separation.md),
  [D015](../adr/D015.md), and
  [portable-common implementation verification](../ROADMAP_HISTORY.md#portable-inputs-and-loader-checkpoint),
  followed by [loader refactor verification](../ROADMAP_HISTORY.md#portable-inputs-and-loader-checkpoint),
  the [independent loader audit](../ROADMAP_HISTORY.md#portable-inputs-and-loader-checkpoint),
  [SDK acquisition/platform source review](records/2026-10-12-src-fidelityfx-sdk-acquisition.md),
  [Windows handoff preparation](records/2026-10-12-exp-proton-windows-preparation.md),
  [Steam Deck cross-build/GPU proof](records/2026-10-13-exp-proton-linux-cross-build.md),
  [Vulkan-validation follow-up](records/2026-10-13-exp-proton-vulkan-validation.md),
  [Steam Machine/provider comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md),
  [Deck temporal regressions](records/2026-10-13-exp-proton-temporal-regression.md),
  [Machine temporal regressions](records/2026-10-14-exp-steam-machine-proton-temporal-regression.md),
  [exit-247 diagnosis](records/2026-10-14-exp-proton-exit-status.md),
  [offline runtime confirmations](records/2026-10-14-exp-proton-offline-runtime.md),
  [timeout cleanup confirmations](records/2026-10-14-exp-proton-timeout-cleanup.md),
  [AMD Vulkan announcement research](records/2026-10-23-src-amd-fsr-vulkan-announcement.md),
  and [SDK 2.4 Vulkan/API compatibility research](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md).
- Scope: current Rust dependencies, module/API gates and the pinned AMD FSR SDK
  v2.3.0 path, plus source findings for SDK v2.4.0 Vulkan integration. Recommendations do
  not accept a new SDK baseline, platform or backend.

## Current findings

**Common rc1 qualification, 2026-10-29:** the
[Windows report](../release/2026-10-29-rc1-windows-verification.md) passes
1,448 required case checks on RX 9060 XT, including exact-package Stable/MSRV
consumers and selected DX12 native profiles. The
[Deck report](../release/2026-10-29-rc1-deck-verification.md) passes 16 offline
and 17 timeout checks on VANGOGH/provider 3.1.5, Proton 10.0-4b/steamrt3.
The [Machine report](../release/2026-10-29-rc1-steam-machine-verification.md)
passes 12 ordinary compatibility checks on NAVI33/provider 4.1.1,
Proton 11.0-2e/steamrt4. All three select identical frozen archives; the
[synthesis](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
retains exact hashes and distinguishes this from the historical mismatched pair.

Proton executes Windows cross-built consumers, with active Vulkan core/sync
validation reporting zero errors/warnings. DirectX debug/info-queue counts are
unavailable, not zero; Linux signature diagnostics do not establish Windows
Authenticode policy. Only Deck establishes offline/forced-timeout coverage;
Machine acknowledges launcher networking. Remote raw inventories were verified
on their hosts, not independently re-read during this synthesis. These are
bounded translated-DX12 results, with no native Linux/Vulkan, general hang
recovery, temporal quality or performance claim. Separate [rc1 performance acceptance](../release/2026-10-29-rc1-performance-acceptance.md)
closes P08 input reconciliation; publication simulation now passes; final release closeout remains.

**Accepted follow-up:** the 2026-10-24 [D031 amendment](../adr/D031.md) restores the
reserved Vulkan Cargo feature and deliberate compile error for wrapper `0.3.0` /
sys `0.1.2`, reversing the unreleased removal. Initial usable Vulkan upscaling is
selected for wrapper `0.4.0`, subject to official SDK/runtime availability and
the evidence gates below. Windows/DX12 code, native types, loaders and dependencies
must remain encapsulated; common and future
Vulkan consumers must not require them. Usable upstream integration and separate
native verification remain necessary. Feature retention changes neither the pinned
SDK capabilities nor the existing native support evidence below.
The [coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
used wrapper `0.3.0` / sys `0.2.0` before the amendment. Its passing local
dependency/consumer and isolated-package checks describe that historical version
pair, not the restored feature or sys `0.1.2`. Windows compilation is not Windows
execution. Restoring a feature name supplies no usable Vulkan integration,
additional platform or native safety guarantee.

**Verified:** common checked upscaler inputs and owned errors are available
without native backend gates. DX12 resources, runtime and raw loader require
Windows plus `dx12`. Optional `movable` adds a conditional thread-transfer family
within those gates. Windows interface dependencies are optional and restricted
to Windows already.

**Resolved dependency issue:** the investigation found unconditional raw
`libloading` in Linux and Windows common-only production trees. The authorized
D015 loader follow-up now makes it optional under `cfg(windows)`, activated by
`dx12`. Verified production trees omit it for common-only use and Linux, including
Linux with DX12 enabled; Windows/DX12 retains it. Fixture dev-dependencies remain.
These trees establish dependency selection, not native execution or measured
overhead.

**Implemented organization:** `loader/mod.rs` reexports the unchanged public
types from private `loader/windows.rs`. The existing Windows/DX12 gate and all
native contracts remain. This adds no Linux module or common loading framework.

**Verified upstream statements:** pinned v2.3.0 documentation explicitly excludes
Vulkan and describes a signed Windows/DX12 DLL integration. Older SDK Vulkan
paths and retained Vulkan ABI identifiers do not establish a compatible native
Linux/Vulkan implementation for the selected path.

**Verified porting hazard:** the raw loader relies on Windows symbol resolution
to establish non-null functions. The inspected `libloading 0.9.0` Unix code can
successfully resolve a null symbol on Linux. Removing the gate alone would not
preserve acquisition invariants. Callback character ABI, native artefacts,
provider resolution and cleanup also need separate review.

## Announced Vulkan path

The [October 23 announcement record](records/2026-10-23-src-amd-fsr-vulkan-announcement.md)
and October 24 intake established an announced path, with no public 2.4 release
found at that earlier cut-off. The
[October 29 source investigation](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md)
supersedes that availability assessment: it identifies official SDK **v2.4.0**,
released **2026-10-28**, at commit
`7539ae8b6568ad29ed78a753404351daf3b5cca7`, and inspects its modern Vulkan header.
The historical records remain unchanged. This synthesis imports those source
findings; it performs no new download, compilation or native experiment.

**Verified in the source record:** the Vulkan creation descriptor holds
`VkDeviceContext` (device, physical device and device-proc-address function).
Its modern descriptor tag is `0x01000003`; the deprecated `0x00000003` layout is
distinct. SDK 2.4 swaps the Vulkan/Xbox backend IDs relative to 2.3, while the
DX12 backend ID stays zero. The five common entry points and shared upscaler
dispatch envelope remain visible, but this is not an exhaustive selected-ABI diff
or proof of DX12 binary compatibility. The currently curated raw crate does not
export the swapped `FFX_API_BACKEND_ID_VK`/`XBOX` macros; this upstream change
alone does not establish a breaking Rust package change.

**Upstream claims, not project execution evidence:** Windows Vulkan 1.2 support
covers upscaling and named Frame Generation paths. Ray Regeneration and Radiance
Caching remain DX12-only. Hardware, driver and extension eligibility must be
established per effect/provider; the report supplies no native Linux runtime.
The release's signed binary contents, per-DLL hashes, signatures, terms and
dependency closure have not been independently inspected in this research.

| Integration route | Evidence and remaining boundary |
|---|---|
| Windows x64/MSVC + DX12 | Project verification remains on SDK 2.3.0; SDK 2.4 needs its own ABI, provider, native regression and performance review |
| Windows + native Vulkan | Official 2.4 source interface and upstream support claim make this the first proposed validation cell; acquisition, ABI and GPU proof remain open |
| Linux + native Vulkan | No authorised native runtime or project execution proof established; independent investigation required |
| Linux + Proton/DX12 | Existing Deck/Machine results use translated DX12 and Windows binaries; neither native Vulkan SDK cell is established |

### SDK 2.4 and release sequencing

**Architectural inference:** D042's portable frame/options/query values and
concrete setup/resource/recording boundary accommodate a separate Vulkan
upscaler. There is no demonstrated Vulkan-driven reason to rename `Runtime`,
change DX12 signatures or add a public `Upscaler<B>` before 0.3.0. Existing
`Runtime::load` explicitly promises Windows/DX12 and the pinned 2.3 ABI. Preserve
that meaning; Vulkan requires explicit matching loader acquisition and its own
device/proc-address lifetime. Backend-specific runtime acquisition is the
report's preferred starting design, not an accepted public API.

Share portable values where their meanings match. Do not reuse DX12 COM
retention, DXGI inspection, opaque query-device casts, resource-state mapping,
terminal-error classification or movable-thread guarantees by analogy. Provider
IDs require fresh validation for the actual runtime/backend/device. Vulkan image
metadata, command-buffer encoding, layouts/barriers, queue ownership and failed
recording/cleanup remain unresolved native contracts. Start the design with
local ownership; movable Vulkan and allocation callbacks require separate
evidence. Some Vulkan allocation callbacks are marked unimplemented in the
inspected header. Frame Generation queue/presentation contracts are not ordinary
upscaler requirements.

The maintainer requested this staged planning direction on 2026-10-29:

| Wrapper target | Bounded work and release condition |
|---|---|
| **0.3.0** | Finish the accepted SDK 2.3/DX12 release. Reconcile explicit loader docs, portable dependencies/error extensibility and existing feature controls in final review. No SDK repin, new Vulkan API or extra broad test campaign follows from this report |
| **0.3.1, conditional** | R06 audits exact SDK 2.4 artefacts and every selected ABI declaration; R07 may then adopt SDK 2.4 for DX12 with compatible acquisition, provider/deployment, native regression and recurring performance review. B03 prepares Vulkan contracts and a bounded spike/design separately. Preserve existing DX12 call sites, safety duties, defaults and supported 2.3 consumer behaviour; keep `vulkan` deliberately unsupported. This is a planning target, not an accepted SDK pin |
| **0.4.0** | B01 delivers the selected concrete Vulkan upscaler after design review and source, ABI, acquisition, actual GPU/readback, lifecycle and package evidence. Windows/Vulkan is the first proposed cell; native Linux, further effects and movable Vulkan are separate scopes |

The report recommends moving SDK 2.4 adoption together with Vulkan to 0.4.0.
Splitting out an earlier SDK-only 0.3.1 is a reasonable **inference**, conditional
on the missing compatibility evidence. Audit public raw declarations **and**
wrapper raw-interop reexports, layouts/constants, trait and feature contracts,
MSRV, runtime eligibility and unsafe preconditions. A new SDK number does not
select Rust crate versions. If raw changes require a compatibility-series bump,
version `fsr-sdk-sys` accordingly and assess its wrapper exposure separately.
If wrapper compatibility or continued 2.3 use cannot be preserved, move the
incompatible work to 0.4.0 rather than shipping it as 0.3.1. Amend D002/D017 only
after the actual audited adoption is reviewed; D031/D042 remain unchanged.

This intake accepts no new runtime guarantees or binary redistribution. The
report's broad temporal/output proposals do not reopen deferred image-quality
research: select bounded mapping, GPU-write/readback, synchronization and cleanup
controls under the existing scope. An API success code alone proves no GPU write.

### R06 comparison checklist

SDK 2.4 source is now inspectable; binary acquisition and the complete selected
comparison remain pending. Compare the exact candidate with v2.3.0:

- Pin the tag/full commit, release date, exact assets and measured hashes;
  identify signatures, package members, notices and runtime dependencies under
  the [distribution review](licensing-and-distribution.md#future-sdk-and-vulkan-artefacts).
- Diff every wrapper-used common, upscaler and backend header. Check entry
  points, descriptor tags/chains, layouts, calling conventions, callbacks and
  platform types against compiled C/C++ and Rust ABI checks. Establish the
  actual effect/provider versions, query/override rules and failure behaviour.
- Establish the OS/architecture, driver/ICD, GPU/provider and Vulkan feature
  matrix from versioned documentation, samples and binaries. Review loader and
  provider discovery separately from OS library loading.
- Retain existing DX12 ABI, consumer and native integration checks as migration
  controls. A bounded feasibility probe can precede a project-wide migration;
  adoption and a coherent shipped baseline still need a separate decision.

### B01 evidence gates

These requirements apply the amended D031 checkpoint and the selected `0.4.0`
initial slice: public context creation, bounded upscaling dispatch/readback and
clean teardown for a named OS/architecture/provider cell. They select no Vulkan
types, dependencies, public backend trait or experiment for implementation now.

1. **Source contract:** identify usable official Vulkan headers, samples and
   runtime. Establish device ownership, command-buffer/queue rules, image/view
   metadata and provenance, formats, layouts/barriers, queue-family transfers,
   submission responsibility, GPU completion, callback/thread rules and cleanup
   after failure. DX12 COM retention, metadata checks and `movable` assumptions
   require independent review before reuse.
2. **Build and acquisition:** verify native ABI, target/feature dependency gates,
   trusted runtime loading, provider queries/selection and bounded failure cases
   on a named OS/architecture. Compilation and queries do not establish GPU work.
3. **Native proof:** create a context and real Vulkan images, record and submit
   through the official developer interface, wait for completion, check readback
   against a bounded control and verify clean teardown with Vulkan diagnostics.
   Record SDK/provider, GPU and driver/ICD identities. Linux requires actual
   native Linux artefacts and a separate proof.
4. **Public support:** verify the public wrapper's creation/dispatch, documented
   unsafe boundaries, error/poisoning/cleanup, pending GPU lifetimes and
   representative motion/jitter/reset, extent/format and provider cases; check
   packages and consumers. A single-frame proof covers only its tested cell.
   These are wrapper integration checks, not deferred image-quality research.

## SDK acquisition and Proton boundary

The [2026-10-12 source review](records/2026-10-12-src-fidelityfx-sdk-acquisition.md)
separates obtaining Windows SDK artefacts on Linux from executing them. Download,
hashing and byte-preserving extraction need no Windows process or Proton.
That source review established feasibility; the later experiments below actually
acquire and verify the selected local subset.
Linux Authenticode tooling can assess PE signatures under explicit trust inputs,
but does not establish equivalence to Windows trust policy. Acquisition and
distribution qualifications are integrated in [licensing/distribution](licensing-and-distribution.md#explicit-developer-sdk-acquisition).

### Windows preparation followed by verified Proton execution

The [2026-10-12 preparation record](records/2026-10-12-exp-proton-windows-preparation.md)
verifies a private bundle assembled from unchanged, previously verified Windows
executables and the pinned v2.3.0 SDK inputs. Repository-side and standalone
inspection pass; five host controls pass, and the full infrastructure suite reports
91 passes and 29 skips. Stale/corrupt inputs, existing or overlapping destinations
and attempted Windows execution are rejected. The initial sandbox test failure
remains recorded separately from the successful permitted runs.

The retained compact manifest records file identities, selectors and required
markers; the binary bundle and ZIP remain ignored local artefacts. Intake checked
that manifest and the record without revalidating the private ZIP or rerunning
the controls. This record captures the preparation stage; subsequent records
establish actual Linux cross-build and Proton/GPU execution.

The [Deck proof](records/2026-10-13-exp-proton-linux-cross-build.md) and
[Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
subsequently verify cross-builds, staged DLL loading, public queries and
single-frame GPU dispatch/readback/teardown on both hosts. The later
[offline-runtime](records/2026-10-14-exp-proton-offline-runtime.md) and
[timeout-cleanup](records/2026-10-14-exp-proton-timeout-cleanup.md) experiments
verify their bounded checkpoints on Deck. Those execution questions are no longer
open within the tested scope. The GPU proof uses Linux-built binaries and a
separate Proton selector with explicit diagnostic limits; Windows Gate A
debug-layer equivalence remains unverified. D019 acceptance remains a separate
maintainer decision.

## Observed Proton execution

**Current release boundary (2026-10-29):** the historical October 13/14 proofs
below retain their original scope. Fresh current-API package-mode runs now pass
on Deck (16 offline and 17 timeout cases) and Machine (12 compatibility cases),
as reviewed in the [rc1 synthesis](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification).
Their archive hashes now match Windows rc1 as well. The earlier pairs differed;
their retained LF/CRLF assessment remains historical. P08 input reconciliation
is now accepted; publication simulation now passes. These are real Linux cross-build
and Proton/GPU results, beyond the earlier mocked build-binding checks. Preserve
V07/V07e, V08/V09 and the separate historical temporal failures. No new native
Linux/Vulkan support or full temporal/image-quality study is implied.

**Verified:** the [Steam Deck proof](records/2026-10-13-exp-proton-linux-cross-build.md)
and [Steam Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
both cross-compile the Windows/MSVC Rust tests and C++ helper on Linux, load the
pinned v2.3.0 AMD DLLs, exercise both public query families and complete the shared
single-frame dispatch, fence, checked readback and teardown. The eight-case
profile is implemented in [proton.py](../../tools/verification/proton.py).

| Configuration | Observed provider | Passing execution environment |
|---|---|---|
| Steam Deck, RADV VANGOGH / device `0x163f` | `3.1.5`, ID `0xf5a5ca1e00c01005` | Proton 10.0-4b, VKD3D-Proton 3.0.0, umu steamrt3 |
| Steam Machine, RADV NAVI33 / device `0x7481` | `4.1.1`, ID `0xf5a5ca1e01001001` | Proton 11.0-2e / VKD3D-Proton 3.0.0 and Experimental 11.0-20261008 / VKD3D-Proton 3.1.0, umu steamrt4 |

Both use the recorded SteamOS 3.9.2 and Mesa 26.2.0-devel git-6c05e09fac inputs.
The Deck's [validation follow-up](records/2026-10-13-exp-proton-vulkan-validation.md)
and four successful Machine runs observe Vulkan core/synchronization activation
in queries and dispatch, with zero reported errors or warnings. Machine setup
and intentional-validation-error attempts remain failed. Windows D3D12
debug/info-queue and GBV coverage remain unavailable; Vulkan diagnostics are
separate evidence.

**Verified compiled shader evidence; inference about execution:** Machine shader
dumps contain packed INT8 dot products, alongside completed GPU output. The
[dispatch assessment](upscaling-dispatch.md#proton-single-frame-and-int8-evidence)
retains the distinction between compiled operations and individual executed
instructions. Experimental's automatic upgrade, disabled upgrade and installed
Proton 11 all pass the pinned SDK workload; Experimental is not required on this
configuration. This says nothing about upgrading games with older runtimes.

The maintainer [accepted D019](../adr/D019.md#decision-status) on
2026-10-13 and closed the initial V07 proof and bounded V07e portability checkpoint.
Regression (V07d), runtime bootstrap/offline execution (V08) and timeout cleanup
(V09) have separate [roadmap checkpoints](../../ROADMAP.md#verification-and-compatibility),
assessed below.
Acceptance does not change native Windows support or add a native Linux/Vulkan
SDK backend.

### Selected temporal coverage

The [Deck](records/2026-10-13-exp-proton-temporal-regression.md) and
[Machine](records/2026-10-14-exp-steam-machine-proton-temporal-regression.md)
records run four common temporal profiles with unchanged numerical criteria;
jitter selects stationary/moving phase 0 rather than the full Windows matrix.
Cut/reset passes on both. Deck/provider 3.1.5 fails history sensitivity and
long-background budgets; Machine/provider 4.1.1 passes normal-output budgets
but fails foreground motion sensitivity and moving jitter alignment. Deck's
nonzero jitter launcher exit blocks its full image verdict. The
[dispatch comparison](upscaling-dispatch.md#selected-proton-temporal-regressions)
retains those distinctions. **V07d is deferred until after 1.0.0; quality acceptance remains unresolved**
under the maintainer's [verification priority](../../SCOPE.md#verification-priority).

Vulkan core/synchronization diagnostics report no errors or warnings in these
selected runs; that does not satisfy failed image criteria or unavailable
Windows diagnostics. Provider, GPU, driver and translation inputs differ, so
the failures do not isolate a Proton or wrapper cause. The exit-247 follow-up
does not reproduce the launcher failure or establish its cause.

### Prepared offline runtime and timeout containment

**Verified bounded V08:** the [offline experiment](records/2026-10-14-exp-proton-offline-runtime.md)
passes two fresh 12-case confirmations on Deck using explicit local import of
Proton 10.0-4b/steamrt3 and the installed umu 1.4.4 package. Full distribution
inventories and a caller-supplied manifest checksum select those local inputs;
network namespaces enforce offline execution, and missing inputs reject without
downloads. Import bounds are 4 GB, 100,000 entries and 300 seconds. This is local
trust, not upstream authentication, remote acquisition, a relocatable runtime
bundle or a pinned host-library/driver closure. Ordinary Proton profiles still
require network acknowledgement. See the [distribution assessment](licensing-and-distribution.md#prepared-local-proton-runtime).

**Verified bounded V09:** the [timeout experiment](records/2026-10-14-exp-proton-timeout-cleanup.md)
preserves a failed process-group-only baseline with 12 survivors. Bubblewrap PID
containment then passes two fresh forced-expiry confirmations: all observed
owned processes exit, unrelated-prefix activity survives, and same-prefix
query/GPU controls pass afterwards. Prepared ordinary, temporal and offline
launches now require PID containment; offline launches also isolate networking.
Missing isolation blocks launch. This process-cleanup proof does not establish
provider unloading, deliberate GPU/kernel-hang recovery, arbitrary-host cleanup
or the cause of the earlier jitter exit. Namespace/pidfd availability and the
recorded Deck stack bound the result.

## Recommendations

Keep the two crates, portable common types and concrete backend modules.
The scoped loader/dependency cleanup is implemented under D015. Making the
wrapper's raw-crate dependency optional remains a separate, lower-priority
choice if completely dependency-free common usage is required.

Use the Rust target for OS selection and features for backend capabilities.
Do not couple Linux to Vulkan or add OS features. Preserve current DX12 defaults,
legacy imports and D031's reserved Vulkan feature/compile error for `0.3.0`.
Share private lifecycle mechanisms where contracts match; defer public backend traits and
additional runtime modules until a second usable native path exists.

A future Vulkan backend needs device-lifetime ownership, trusted image metadata
and provenance, native resource mapping, command-buffer and queue-family rules,
layout/access synchronization, and its own native/threading/cleanup evidence.
Cloned handle storage does not provide COM-style device retention. Current
DX12 trust and `Send` audits do not transfer automatically.

## Evidence limits and open choices

The implementation follow-up passes macOS workspace tests, Clippy, Rustdoc,
Windows compilation with and without DX12/movable, two Windows-target loader
compile-fail doctests, and 49 downstream import controls. Linux/Windows production
trees and unsupported Vulkan diagnostics also pass their controls. Linux Rust
compilation, Windows executable tests, MSVC header ABI checks and native AMD/GPU
execution were not performed in that loader follow-up. Later Proton execution
is assessed separately above. The PowerShell CI script was extended but could
not run locally; remote success is not claimed.

The subsequent independent audit also records passing source/consumer checks
and eight real package configurations against Linux and Windows production
dependency trees. Its authorized CI follow-up added those 16 package/target
checks to the existing PowerShell script, with separate mutation controls for
defaults and feature forwarding. That audit still executed no Windows binaries,
MSVC header probes or AMD/GPU work; the script itself was not run locally there.
These are documented follow-up results, not fresh execution in this synthesis.

Before native Linux implementation, choose whether the requirement is portable
configuration, Windows compatibility-layer use, or native Vulkan execution.
Select and pin actual upstream artefacts and technology coverage for the last
case. An older integration must not silently change D002 or claim modern-provider
equivalence. Preserve applicable third-party notices and review any new native
distribution separately.
