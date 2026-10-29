# Research

Research for the engine-independent AMD FSR SDK Rust wrapper. Project intent is
in [SCOPE.md](../../SCOPE.md); source and manifests establish implemented
behaviour. Findings here do not imply SDK support or accepted design decisions.

## Start here

Read the [research process](PROCESS.md) for evidence standards and integration.
Use the [source record](templates/source.md),
[experiment record](templates/experiment.md), or
[topic synthesis](templates/topic.md) template as appropriate.
Historical evidence goes in [records/](records/); current topic syntheses live
beside this file and link the records they use.

Retired verification scripts are preserved in the
[indexed 2026-10-10 archive](../../tools/verification/archive/2026-10-10/README.md).
It records exact intake bytes, dependencies, referring records and reconstruction
limits. Historical command text and observations remain in the records; script
links point to archived source and affected records have a dated navigation note.
Current [verification profiles](../../tools/verification/README.md) orchestrate
routine checks and separately named experimental protocols; they do not import
archived scripts or overwrite these records.

The [2026-10-09 Windows native regression recheck](../ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh)
reran existing tests and protocols without changing historical records. It
refreshes bounded execution evidence for loading, construction, lifecycle,
threading and dispatch; the sharpening effect gate, GBV dispatch timeout and
native failure/termination uncertainties remain open. Its captured baseline
excludes later concurrent M7 edits. Routine verification reports remain separate
from the historical research records below.

## Coverage

By maintainer direction on 2026-10-18, image-quality research and custom quality
policies are outside scope through 1.0.0. The
[scope](../../SCOPE.md#verification-priority) prioritizes SDK access and wrapper
integration; [deferred work](../POST_1_0.md) is separate from the active roadmap
and requires explicit selection after 1.0.0. Existing findings remain historical
evidence. ABI, mapping, safety, native execution and cleanup verification continue.

| Current topic | Evidence and coverage |
|---|---|
| [Licensing and distribution](licensing-and-distribution.md) | Existing v2.3/Proton provenance retains its bounds. SDK 2.4 release/header availability is now recorded, but exact binary hashes, signatures, package terms and dependency closure remain unaudited; no new redistribution approval. |
| [Upscaling API coverage](upscaling-coverage.md) | U07–U12 and bounded SDK access are implemented. [Common rc1 qualification](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification) passes Windows 1,448, Deck 16 + 17 and Machine 12 required checks on identical archives. Historical generator failures remain. P08 input reconciliation is accepted; publication simulation passes; publication remains separately authorised. |
| [Callbacks, diagnostics and custom allocation](callbacks-and-allocation.md) | D040 source/spike/implementation evidence and the earlier package checkpoint are followed by [rc1 archive-bound checks](../release/2026-10-29-rc1-windows-verification.md). Both families and Stable/MSRV production CB exchange pass; native heap invocation, quiescence and failure recovery remain unproved. |
| [API and safety](api-and-safety.md#accepted-api-transition-and-setup-draft) | D042/U07–U12 and archive-native evidence retain their bounds. The [Vulkan compatibility assessment](api-and-safety.md#sdk-24-and-vulkan-compatibility) supports concrete backend extension; a nonbreaking SDK-only update still needs raw-interop and native-contract review. No broader native safety guarantee follows. |
| [Core and optional utilities](core-and-utilities.md) | D033 boundaries and opt-in D036 selection for 0.3.0 remain. [Archive consumers](../release/2026-10-29-candidate-verification.md) compile both utility compositions on Stable/MSRV and feature-on/off configurations. No new policy hook, utility algorithm or default activation is selected. |
| [Runtime loading](runtime-loading.md) | Existing Windows/Proton deployment and termination limits remain. D042's loader-only/setup trust split is implemented under U07; the loaded runtime has no safe effect escape. Weak state retention is distinguished from actual DLL unload; no new acquisition/deployment policy is selected. |
| [Platform and graphics backends](platform-and-backends.md) | Common rc1 hashes now match across Windows/Deck/Machine; bounded Proton proof adds no native Linux/Vulkan support. SDK 2.4 source findings require no demonstrated pre-0.3.0 redesign. Conditional 0.3.1 SDK adoption/preparation precedes 0.4.0 Vulkan, with separate ABI/acquisition/native gates. |
| [Context lifecycle](context-lifecycle.md) | D042 candidate contexts retain production ownership after setup/runtime loss; controlled unwind, failure retention and first-cause/abandonment checks pass. U11 now exposes pure usability and local abandonment with [separate production checks](../release/2026-10-27-context-usability-verification.md), without native recovery, callback quiescence or heap-invocation guarantees. |
| [Upscaler construction](upscaler-construction.md) | Existing selected-provider native evidence remains bounded. D042 selects a borrowed setup; its controlled candidate preserves fresh membership, live identity and setup-independent owners. P07 supplies no cache or latency argument for that design. |
| [Runtime threading](runtime-threading.md) | Existing D013/D040 native limits remain. The setup spike compiles local/movable owner/view traits and executes one controlled transfer; loader release trust must precede Send, while effect trust persists after setup loss. No new AMD concurrency/affinity guarantee follows. |
| [Upscaling dispatch](upscaling-dispatch.md) | U08 composition and existing unsafe recording duties remain. [Archive-native qualification](../release/2026-10-29-candidate-verification.md) covers motion, typed/depth/BGRA/resource formats and current provider-specific generation, preserving failed no-write outcomes and historical identities. Readback success adds no shader-access, image-quality or broader semantic guarantee. |
| [Performance](performance.md) | P01–P07 retain historical bounded results. [P08 is closed](../release/2026-10-29-performance-review-closure.md) with accepted temporal/deployment limits; [final rc1 input/impact reconciliation is accepted](../release/2026-10-29-rc1-performance-acceptance.md) after a separate 122-case timing follow-up. Accepted P08 nominal one-sided 95% CPU upper bounds are 4.046–4.618 us; no wrapper-specific GPU overhead is detected. Functional host checks add no timings. The 30/30 provider comparison shows no clear GPU difference, but higher default-provider context costs; driver/wallpaper causation and universal parity remain unestablished. |
| [Evidence retention](evidence-retention.md) | Repository growth audit and implemented compact retention/size budgets. Failed outcomes and measurements remain; removed raw GPU frames cannot be reconstructed from hashes. |
| [Language conventions and migration](language-conventions.md) | Both language audits reconciled with naming verification and current policy. The three follow-up prose findings are corrected; legacy names, metadata compatibility and frozen protocol identities remain intentional. No complete spelling proof or new migration is implied. |

The [short rc1 timing follow-up](records/2026-10-29-exp-rc1-performance-check.md)
and [maintainer acceptance](../release/2026-10-29-rc1-performance-acceptance.md)
are integrated on 2026-10-29 in performance and current release status. The
source/package reconciliation is complete. The accepted claim retains numerical
CPU/GPU confidence bounds and their limits; no samples or original verdicts change.

The three rc1 reports — [Windows](../release/2026-10-29-rc1-windows-verification.md),
[Deck](../release/2026-10-29-rc1-deck-verification.md) and
[Machine](../release/2026-10-29-rc1-steam-machine-verification.md) — are integrated
on 2026-10-29 in coverage, API/safety, platform/backends and the release status.
Their [cross-host synthesis](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
distinguishes exact archive identity, host-specific scope and remote evidence
verification limits. Historical mismatched archives and failures remain visible;
this intake runs no tests, benchmarks or publication.

The [SDK 2.4/Vulkan source report](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md)
is integrated on 2026-10-29 in platform/backends, API/safety and licensing. The
[release sequencing assessment](platform-and-backends.md#sdk-24-and-release-sequencing)
records the conditional SDK-only patch target separately from the report's
recommendation to adopt 2.4 with Vulkan in 0.4.0. No SDK pin, public API or native
support claim changes; no experiment runs as part of this intake.

The [consistency audit](records/2026-10-26-src-public-api-consistency-and-architecture-audit.md),
[usability/extensibility redesign](records/2026-10-26-src-public-api-usability-and-extensibility-redesign.md)
and [setup contract/composition spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md)
are integrated on 2026-10-26 in the [API assessment](api-and-safety.md#accepted-api-transition-and-setup-draft)
and the affected construction, lifecycle, loading, threading, dispatch, coverage
and core/utility topics. [D042](../adr/D042.md) accepts the direction; the
[concrete contract](../design/upscaler-setup.md) was approved and U07 implemented
on 2026-10-27; [production verification](../release/2026-10-27-upscaler-setup-verification.md)
remains distinct from the spike and final-package checks. Historical records and negative outcomes
are preserved; this integration runs no new experiment.

The [SDK callback/allocation investigation](records/2026-10-25-src-upscaler-callback-allocation-contracts.md)
and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md)
are integrated on 2026-10-25 in the [callback/allocation synthesis](callbacks-and-allocation.md),
coverage, API/safety, lifecycle and threading. The subsequent
[ABI/native spike](records/2026-10-25-exp-upscaler-callback-allocation.md) is now
integrated in those topics: its bounded checkpoint passes, including current/MSRV
aggregate exchange and native host/resource/CB/message observations. Heap calls
remain unobserved, and signed-loader/provider diagnostic differences qualify the
open-source model. Accepted [D040](../adr/D040.md) and the separate
[implementation verification](../release/2026-10-25-callbacks-diagnostics-verification.md)
now close R03/U04 for bounded SDK access: 192 CI, 169 MSRV and 15 native stages,
including 12 public contexts/20 fenced frames, 16 messages and legacy live code 0.
Heap ABI/controlled calls pass; SDK invocation remains unobserved. D042's
production API transition now precedes final
[release archive coverage](../release/0.3.0-preparation.md#archive-coverage-before-final-verification),
not another source investigation or general spike.

The [remaining-format accounting](records/2026-10-24-src-remaining-upscaling-formats.md)
and [independent SDK experiment](records/2026-10-24-exp-remaining-upscaling-formats.md)
resolve D035's residual format families and the RGB32F helper omission.
[D039](../adr/D039.md) implements the checked routes; [public verification](../release/2026-10-24-resource-formats-verification.md)
passes, completing U05 for bounded SDK access. The dispatch and coverage syntheses distinguish source applicability,
actual native observations and implementation status, without image/algorithm
comparisons.

The [BGRA/sRGB assessment](upscaling-dispatch.md#typed-bgrasrgb-access-2026-10-24)
integrates the external source record, corrected source audit, independent views
and direct SDK execution. D038 accepts the typed-resource contract and authorizes
implementation; the separate [public-wrapper checkpoint](../release/2026-10-24-bgra-srgb-verification.md)
now passes stable/MSRV/ABI and both-family native checks. Verification is limited
to SDK integration, without image/algorithm acceptance criteria.

The [2026-10-23 AMD Vulkan announcement record](records/2026-10-23-src-amd-fsr-vulkan-announcement.md)
is integrated on 2026-10-24 in
[platform/backends](platform-and-backends.md#announced-vulkan-path) and
[licensing/distribution](licensing-and-distribution.md#future-sdk-and-vulkan-artefacts).
AMD's commitment is distinguished from secondary reporting of SDK 2.4 and its
timing. Primary announcement/download spot-checks agree with the record; its
source ZIP, video/slides and driver-DLL claims were not independently validated.
B01 remains technically blocked, R06 awaits inspectable candidate artefacts,
and the v2.3.0 pin and accepted decisions remain. No native work was rerun.
A dated navigation correction supplies missing source-link definitions in the
incoming record without changing its findings or evidence cut-off.

The [U05 formats/views and motion-scale record](records/2026-10-19-src-upscaling-formats-and-motion-scale.md),
investigated on 2026-10-19, is integrated on 2026-10-20 in
[dispatch](upscaling-dispatch.md#external-formats-and-explicit-motion-scales),
[coverage](upscaling-coverage.md) and [API/safety](api-and-safety.md).
It identifies candidate format families and open-provider scale semantics while
preserving unknown closed-provider behaviour, generated-view risks, zero/extreme
value questions and an NDC guide/sample disagreement. Its archive identity is
reported but not independently revalidated; representative cached-header checks
also expose incomplete citation ranges. The original UTF-8/LF record is unchanged.
That intake alone accepted no implementation or spike and added no GPU execution.
The later [D034](../adr/D034.md) slice is accepted and implemented, with
[stable/MSRV and public native verification](../release/2026-10-21-native-motion-verification.md)
on 2026-10-21. Its 252 dispatch/12 control frames include mixed-sign jitter,
RG32F render/display motion and UAV-capable inputs. D039 subsequently completes the applicable formats;
final release verification remains open; source/experiment records are unchanged.

The [2026-10-19 variable-output record](records/2026-10-19-src-variable-output-resolution.md)
is integrated in [dispatch](upscaling-dispatch.md#variable-output-resolution-source-contract)
and [coverage](upscaling-coverage.md). It identifies native per-frame output access
while preserving provider, allocation and rounded-write uncertainty. The dispatch
assessment records the byte-preserving rename, missing project-archive identity,
inaccurate source line ranges and the distinction between portable geometry and
the current DX12-gated frame type. The subsequent
[native spike](records/2026-10-19-exp-variable-output-resolution.md) is integrated
on 2026-10-19 in dispatch, coverage and API/safety: all 71 integration cases pass,
including 180 dispatched frames. 4.1.1/3.1.5 vary written footprints with rounded
padding writes; 2.3.4 retains maximum output. Active-sized passes do not certify
out-of-resource access safety, and dynamic-flag effects differ between providers.
At that intake M14's public contract/implementation remained open. Intake verified compact
inventory hashes and outcomes without rerunning native work or altering the record.
The dispatch synthesis also [corrects the record's oversized/sharpened footprint
summary](upscaling-dispatch.md#variable-output-native-follow-up-2026-10-19): retained
measurements show writes through 640×368 in that arm. The historical prose
overgeneralizes the maximum-allocation table; its original bytes remain preserved.
Subsequent explicit maintainer review accepted the [D024 output amendment](../adr/D024.md#amendment-active-output-within-maximum-allocations-2026-10-19).
Its [implementation verification](../release/2026-10-19-active-output-verification.md)
is now synthesized in dispatch, coverage, API/safety, construction, lifecycle and
threading. It passes 62 public-output cases (144 dispatched and 12 control frames)
and 257 consolidated Windows stable/MSRV/ABI/consumer/native cases. Both public
owners run on 4.1.1/3.1.5 within exact maximum allocations under dynamic-option
gating and explicit native trust. This closes M14's selected integration checkpoint;
it adds no smaller/oversized allocations or generic access-safety guarantee.
Intake verifies compact inventory hashes, outcomes, comparisons and the exact
two-Rustdoc source delta between runs without new execution. The completed report,
supporting evidence, raw spike and its correction remain unchanged. Package
inspection does not complete final extracted-package verification or full upscaling coverage.

The [2026-10-18 release-surface inventory](records/2026-10-18-src-upscaling-release-surface-inventory.md)
and its [Configure-contract prerequisite](records/2026-10-15-src-upscaler-configure-contract.md)
are integrated in [upscaling coverage](upscaling-coverage.md). The source intake
corroborated main code gaps and documented inaccurate upstream line ranges;
use pinned files and symbol names. The archive identity was not revalidated, and
no new compilation/native execution or API acceptance followed from that intake.
The subsequent [D028 implementation record](../release/2026-10-18-provider-selection-verification.md)
and [D029 implementation record](../release/2026-10-18-upscaler-tuning-verification.md)
are now synthesized in coverage, API/safety, construction, lifecycle, threading
and dispatch. Their accepted designs and local checks supersede the missing
implementation baseline. The later [API coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
records implementation of the maintainer's D022/D028/D029/D031/D032 amendments:
compulsory
parameter-domain checks, `request_tuning`, `validate_dispatch_inputs`,
quality-derived size names and then-selected independent sys `0.2.0`. It passes 108 CI, 97 exact
Rust 1.88 MSRV and 203 isolated development-package cases, plus Windows compilation.
Intake checks the reported verdicts and archive hashes against retained local
artefacts; no new compiler/native/GPU run is part of this synthesis. The later
2026-10-24 D031 amendment restores the Vulkan feature and sys `0.1.2`; that
historical checkpoint does not verify the amended version pair.

The subsequent [2026-10-19 Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md)
is integrated into coverage, API/safety, construction, lifecycle, threading and
dispatch: the 256-case consolidated batch and 31-case native profile pass.
This closes the bounded Windows fixture/MSVC/selection/tuning gaps; native tuning
acknowledgement still proves neither numerical application nor effective defaults.
Development archives are not final release verification or a complete comparison
against published packages. Routine reports remain canonical in `docs/release/`.

The [reactive-mask provider experiment](records/2026-10-19-exp-reactive-mask-providers.md)
and [provider setup-query measurement](records/2026-10-19-exp-provider-setup-cost.md)
are also integrated on 2026-10-19. Both public generator families/all flags pass
on 3.1.5; 4.1.1/2.3.4 leave masks unchanged despite OK returns, preserving failed
write criteria and failed complete profiles. P07 retains 192 whole-path timings
from three optimized processes, without attributing enumeration cost or choosing
caching. Intake checks compact inventories, outcomes and arithmetic without new
native execution. Historical records and accepted safety contracts remain unchanged;
U01's mixed provider results and final archive verification remain release-review inputs.

The [2026-10-15 HDR/encoding concept](records/2026-10-15-src-hdr-colour-encoding-concept.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#linear-hdr-and-non-linear-colour)
alongside accepted [D026](../adr/D026.md) and the
[linear HDR implementation checks](../release/2026-10-15-linear-hdr-verification.md).
Native colour controls additionally follow [D027](../adr/D027.md), based on the
[non-linear contract investigation](records/2026-10-15-src-non-linear-colour-contract.md);
see [colour implementation checks](../release/2026-10-15-colour-encoding-verification.md).
The [Windows batch](../release/2026-10-15-windows-feature-batch.md) subsequently
verifies linear HDR with six GPU frames, fixtures, C++ ABI and MSRV checks.

The completed [native sRGB/PQ signal experiment](records/2026-10-15-exp-non-linear-colour-signals.md)
is integrated on 2026-10-15: all 270 frames pass native integration, but 15
changing-pre-exposure predictions fail. Static/reset controls pass unchanged
budgets, supporting a temporal-state dependency without determining internal
ordering, correct dynamic packing or provider PQ normalization. M16 remains
Active. Intake checks retained measurements, repetition hashes and evidence
integrity without new GPU execution; historical records remain unchanged.

The [2026-10-15 performance source investigation](records/2026-10-15-src-performance-testing-strategy.md)
is integrated with the existing [CPU experiment](records/2026-10-15-exp-wrapper-cpu-performance.md)
in the [performance topic](performance.md). Intake preserves the source record
unchanged and reconciles its older ZIP's missing P01 files with this checkout's
record, harness and six retained measurement matrices. No new GPU execution or
external-source investigation was performed. A subsequent maintainer request
converted the proposed sequence into [performance roadmap entries](../../ROADMAP.md#performance);
planning remains separate from implementation and performance acceptance.

The completed [P03 independent C++ CPU comparison](records/2026-10-15-exp-independent-cpp-cpu-comparison.md)
and [P04 1080p GPU timing pilot](records/2026-10-15-exp-1080p-gpu-timing.md) are
integrated in the performance topic on 2026-10-15. All 45 P03 and 22 P04 protocol
cases pass on the recorded Windows configuration; CPU differences remain noisy,
while P04 establishes one matched GPU command-interval baseline. Close GPU
medians do not establish parity. Intake checks compact verdicts, cohort metadata/
output matching and block arithmetic without new native execution; records stay
unchanged. P01/P03 use different reference and sampling conditions, so their
overhead estimates are not pooled.

The completed [P05 active-render timing experiment](records/2026-10-16-exp-active-render-timing.md)
is integrated on 2026-10-16. All 42 cases pass, covering six debug controls and
four balanced Rust/C++ blocks with 3,864 submitted/fenced frames. Matched
stable-size inputs and independent GPU input audits support bounded transition
measurement; substantial timing variability leaves a consistent transition
penalty inconclusive. Intake checks retained evidence and arithmetic without
new native execution. Historical measurements remain unchanged.

The completed [P02 supplied-mask/linear-HDR comparison](records/2026-10-16-exp-mask-hdr-performance-comparison.md)
is integrated on 2026-10-16. All 62 cases pass, with ten debug and forty timed
children, 6,440 submitted/fenced frames, independently checked masks/inputs and
matched Rust/C++ execution. Four balanced blocks retain variable, sign-changing
CPU/GPU differences; speed parity remains inconclusive. Intake checks evidence
integrity, process audits and summary arithmetic without new native execution.
The selected P01/P03–P05/P02 checkpoints are complete; actual mask consumption,
broader non-linear colour semantics and final release scope/checks stay separate.

The [P06 implementation/pilot record](records/2026-10-16-exp-native-performance-characterization.md)
and separate [completed confirmation](records/2026-10-16-exp-native-performance-confirmation.md)
are integrated on 2026-10-16. The earlier stopped 110-block run remains historical.
A fresh unattended pilot/confirmation passes 122/1,220 cases with 112 frozen
randomised blocks. All five workloads meet the selected 5 µs average additional
CPU budget; every one-sided upper 95% bound is below 2.78 µs. GPU equivalence
remains unproven; lifecycle intervals span zero and allocation probes observe
zero Rust allocator events over 1,920 warmed calls. Intake audits full local
inventories, native logs, matching inputs/outputs and retained arithmetic without
new native execution. Historical records and release gates remain unchanged.

The [M4 Windows experiment](records/2026-09-22-exp-m4-windows-verification.md)
adds production-owner fixture, native ABI and successful AMD lifecycle evidence
to the API/safety topic; no GPU dispatch was submitted.

Three initial M5 source records add [construction validation](records/2026-09-22-src-m5-upscaler-construction-validation.md),
[Query/Configure scope](records/2026-09-22-src-m5-upscaler-query-configure.md) and
[runtime sharing/threading](records/2026-09-22-src-m5-runtime-sharing-threading.md).
The [integrated findings and provenance limits](upscaler-construction.md#m5-source-intake-and-provenance)
include the later repair and signed-runtime experiments. Accepted
[D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
selects the public construction contract; M5 implements it in Rust. Ordinary
Windows CI fixtures passed in the [recorded run](records/2026-09-24-exp-m5-windows-ci-verification.md);
both opt-in native lifecycle cases passed on the tested Windows x64/MSVC
configuration ([native record](records/2026-09-24-exp-m5-windows-native-verification.md)).

The [2026-09-29 API surface record](records/2026-09-29-src-dx12-upscaler-api-surface.md)
is integrated in both topics. It corroborates the intentional v0.1.0 profile
and proposes future APIs/experiments; it accepts no new decision or milestone.

The [2026-10-12 M10 Query safety review](records/2026-10-12-src-m10-dx12-query-safety.md)
is integrated in API/safety, construction/planning and threading. The new record
was renamed from its `-EN.md` intake filename without content changes. It reports
pinned primary-source inspection and supplies the project ZIP identity/hash;
intake checked current implementation gaps without independently revalidating
the ZIP or repeating that investigation. It refines enumeration truncation,
global-name copying and CPU memory-query paths, while closed-provider pointer
and GPU guarantees remain unresolved. Safe methods under extended unsafe
acquisition trust and live-query rejection after dispatch poison are
recommendations at intake. Subsequent [D018](../adr/D018.md) accepts the contract
and unreleased implementation. The subsequent
[Windows checkpoint](../release/2026-10-12-windows-m10-sdk-verification.md) passes
ABI, controlled/consumer/MSRV and native query/GPU checks on one configuration.
M10 is selected for [0.3.0](../release/0.3.0-preparation.md); final package checks
and publication remain open.

The [D013 compile/host experiment](records/2026-09-29-exp-d013-movable-compile-host.md)
and [Windows fixture execution](records/2026-09-29-exp-d013-windows-fixtures.md)
are integrated in [runtime threading](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles):
trait assumptions and all eight D013 probes pass on macOS ARM64 and Windows
x64/MSVC, including serialized Rc/Arc lifecycles and Windows DLL retention/release.
These fixtures establish ownership behaviour, not AMD transfer compatibility.
[D013](../adr/D013.md) now accepts the conditional design; this experiment added no public movable API.

The [native D013 experiment](records/2026-09-30-exp-d013-native-thread-transfer.md)
is integrated in [runtime threading](runtime-threading.md#d013-fenced-native-transfers-and-final-release)
and summarized in [API and safety](api-and-safety.md). Three fenced transfer
scenarios, their same-thread controls and current native lifecycle/dispatch
regressions pass on one recorded AMD configuration. Final loader release/unload
was observed on the destination thread; the provider DLL remained loaded in
both controls and transfers. General transfer guarantees and provider termination
remain unresolved; later D013 acceptance adds no native guarantee.

The [2026-10-07 source report](records/2026-10-07-src-d013-transfer-cleanup-contract.md)
and [failure/cleanup experiment](records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
are integrated in [runtime threading](runtime-threading.md#d013-transfer-contract-source-intake),
API/safety, lifecycle and loading. The source report was moved byte-for-byte
from the nested `fsr-sdk-rs/docs/research/records/` intake directory. Its requested
pinned implementation retrieval failed; no new AMD transfer guarantee was found.
The experiment passed 14 owner children and six worker load-failure observations
on Windows, adding transferred poison, preflight/recovery, terminal retention
and coordinated unwind. It does not execute AMD or the report's larger suggested
matrix. The create-error quarantine recommendation does not replace D005's
accepted bounded RAII trust assumption. Later D013 acceptance preserves that policy.

The [2026-10-08 native follow-up](records/2026-10-08-exp-d013-native-transfer-cleanup.md)
is integrated in [runtime threading](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency),
API/safety, lifecycle and loading. All 11 prerequisites and 30 new children pass,
including 12 actual Rust-unwind cleanups and 96 selected payload comparisons.
It closes the dispatch-policy extraction's native regression gap. The provider
persists after all audited application refs and worker exit; its termination
remains unresolved, including after D013 design acceptance.

The [D013 soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md)
was renamed from `neuer_record.md` and integrated in
[runtime threading](runtime-threading.md#d013-soundness-and-architecture-review),
API/safety, lifecycle and loading. No investigation date or reviewed revision was
supplied; 2026-10-09 is the intake date. It recommends acceptance with stronger
arbitrary-destination and lasting serialization/cleanup obligations, without
another exploratory native phase. It adds no execution evidence or AMD guarantee.
Under explicit authorization, [D013 was accepted on 2026-10-09](../adr/D013.md#acceptance-rationale-and-remaining-assumptions)
with those revisions and a framework-independent renderer-state migration
rationale. The [ADR contract](../adr/D013.md#conditional-safety-contract) owns the
requirements. No design soundness blocker or further general research gate
remains. The subsequent [public implementation verification](../ROADMAP_HISTORY.md#movable-implementation-checkpoint)
records the optional API, consumer boundaries and coordinated cleanup checks
separately. It establishes no missing AMD guarantee and selects no release.

The [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#d014-sharpening-native-execution-and-failed-effect-gate)
and [API and safety](api-and-safety.md). Execution and repeat controls passed;
enabled zero differed from disabled. The proposed effect gate failed in all three
repetitions, confirmed by an independent readback-audit run. The maintainer
separately accepted [D014](../adr/D014.md) on 2026-10-07. Its public frame and
sharpening API is implemented and checked under
[M8](../ROADMAP_HISTORY.md#frame-and-sharpening-integration). The separate
[M9 integration protocol](../release/2026-10-10-m9-verification.md#integration-protocol)
subsequently passed in [2026-10-10 verification](../release/2026-10-10-m9-verification.md),
integrated in dispatch, API/safety and licensing/distribution. Development source
and the unmodified extracted wrapper candidate each completed 144 outputs with
continuing-history mode changes, clean ordinary debug/device checks and cleanup.
Exact local `fsr-sdk 0.2.0` / `fsr-sdk-sys 0.1.1` archives and registry-only consumers
passed verification with D013's public movable family excluded from that
historical projection. The maintainer explicitly superseded this exclusion on
2026-10-10; [combined release verification](../ROADMAP_HISTORY.md#release-qualification)
covers the complete `0.2.0` release. Mapping fixtures, GPU execution and quality remain distinct evidence;
the original failed effect results are unchanged.

The [2026-10-10 V01 scene/reference record](records/2026-10-10-src-v01-scene-reference-contract.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#v01-scene-and-reference-contract)
on 2026-10-11. It proposes opaque disocclusion inputs, independent point/area
references, regional temporal diagnostics and longer matched controls within the
existing profile. The record was preserved unchanged; selected pinned source and
repository checks support intake, without reproducing the complete investigation.
It adds no GPU results or quality thresholds. Its proposed input/projection
checks and native calibration are addressed for selected scenes by the bounded
experiment below. Separate verification against frozen criteria remains open;
the historical sharpening effect gate remains failed.

The [2026-10-11 temporal calibration](records/2026-10-11-exp-v01-verification-calibration.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#temporal-calibration-and-candidate-verification-criteria).
Three independent 301-frame children completed with full RGBA16F readbacks and
independently byte-checked inputs on the recorded AMD deployment. Repeat and
cut reset/fresh outputs were locally identical. Foreground temporal residual
and silhouette error distinguish selected wrong controls, while several other
error/leakage rankings favour wrong inputs and young pure-background regions
are empty. Failed attempt outcomes and compact measurements remain preserved; generated
raw frames were removed under the dated retention update in the record. These
findings motivated prospective criteria and further bounded controls. The
[subsequent diagnosis and confirmations](upscaling-dispatch.md#confirmed-temporal-properties-and-metric-limits)
are integrated on 2026-10-12: evaluator/quality validation, continuous history,
cut/reset and isolated jitter sign/phase resolve the selected V01 questions
under frozen criteria. Mixed rankings and failed attempts remain preserved;
this does not retrospectively accept the calibration's candidate rules.

The [2026-10-12 SDK acquisition review](records/2026-10-12-src-fidelityfx-sdk-acquisition.md)
is integrated in licensing/distribution and platform/backends. It corroborates
pinned provenance/licence findings and proposes explicit developer acquisition.
Those source recommendations are historical; D017 subsequently selected the
maintainer bootstrap, and the Proton records below verify its local Linux reuse
and bounded GPU execution. Wider dependency/redistribution audits, remote
launcher-runtime acquisition and complete host dependency closure remain open;
bounded local import/offline execution is verified below.

The [Windows handoff preparation](records/2026-10-12-exp-proton-windows-preparation.md)
is integrated in [platform/backends](platform-and-backends.md#windows-preparation-followed-by-verified-proton-execution)
on 2026-10-15. Bundle inspection and five host integrity/refusal controls pass;
this preparation was followed by verified Linux cross-builds, public queries and
GPU execution on Deck and Machine, then bounded offline execution and timeout
cleanup on Deck, as recorded below. The later GPU proof uses a separate Proton
diagnostic selector; Windows Gate A debug-layer equivalence remains unverified.
Intake checked the retained manifest; private payloads and controls were not
revalidated.

The [Steam Deck cross-build/GPU proof](records/2026-10-13-exp-proton-linux-cross-build.md),
[Vulkan-validation follow-up](records/2026-10-13-exp-proton-vulkan-validation.md) and
[Steam Machine/provider comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
are integrated in platform/backends, dispatch, loading and licensing/distribution
on 2026-10-13. The same bounded eight-case profile passes on Deck/provider 3.1.5
and Machine/provider 4.1.1. Machine passes under both Proton 11 and Experimental,
including Experimental with automatic upgrading disabled; its shader diagnostics
verify compiled packed INT8 operations, with model-path selection retained as an
inference rather than instruction tracing. Failed attempts stay failed. Windows
debug/GBV equivalence, temporal/full regression, full runtime pinning/offline
execution and timeout cleanup remained open at that intake. The maintainer
subsequently accepted
[D019](../adr/D019.md#decision-status) on 2026-10-13, closing the initial V07 proof
and bounded V07e portability checkpoint while tracking V07d/V08/V09 separately.
Intake checked the retained six-attempt evidence and probe hash without rerunning GPU work or
reconstructing unretained logs/shaders.

The 2026-10-14 records are integrated in those topics. The
[Deck temporal attempt](records/2026-10-13-exp-proton-temporal-regression.md) and
[Machine temporal comparison](records/2026-10-14-exp-steam-machine-proton-temporal-regression.md)
pass reset under unchanged criteria; other image criteria fail, and Deck jitter
remains blocked. The [exit-status diagnosis](records/2026-10-14-exp-proton-exit-status.md)
does not reproduce or explain the earlier exit 247. Further V07d quality work
is deferred until after 1.0.0 under the current scope.
The [offline-runtime record](records/2026-10-14-exp-proton-offline-runtime.md)
completes bounded V08 local import/offline execution on Deck. The
[timeout-cleanup record](records/2026-10-14-exp-proton-timeout-cleanup.md)
completes bounded V09 on that stack, preserving the failed group-only baseline
alongside two successful PID-contained confirmations and subsequent GPU controls.
Remote acquisition, a pinned host closure, general regression portability and
GPU/kernel-hang recovery remain outside those proofs.

The [repository size audit](records/2026-10-11-src-repository-size-audit.md) is
integrated in evidence retention, preserving its original measurements and the
later compact-retention choice that superseded its external-archive proposal.

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | D018 accepts M10 query trust, bounded enumeration and error/poison policy; Windows native verification passes on one configuration; rc1 exact-package host checks pass and performance-input reconciliation is accepted; publication simulation passes; publication remains separately authorised; closed-provider names, non-retention, numerical safety and in-flight GPU compatibility remain unresolved. Public v2.3.0 evidence still defines no complete numerical construction domain; accepted D008 uses bounded trust. D028/D029 now define and implement provider/Configure contracts; their closed-provider non-retention, safe rejection, idle-context compatibility and terminal-error assumptions remain unproven native properties. D040 defines the explicit unsafe callback contract; general native quiescence, concurrency and failure guarantees remain unproved. Contradictory safety evidence requires policy review. |
| Runtime threading | Provider/dependency termination and retention cause, native returned-error/partial-load cleanup, hidden work/TLS, external-user coordination, outstanding GPU work, other configurations and sanitizers remain open. D013 accepts explicit caller assumptions and its optional public implementation has bounded verification; no missing AMD guarantee is resolved. Release selection and artefact verification remain separate. |
| Upscaling dispatch | Selected V01 criteria and motion/disocclusion/history/reset/jitter sensitivity are confirmed on one deployment. Young age-0/1 disocclusion acceptance, broader content/cuts/profiles and cross-deployment tolerances remain open, alongside signed-provider sharing and GBV. The original sharpening effect gate remains failed; no general recording capacity or cross-queue guarantee follows. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for tested signed v2.3.0 inputs; basename collisions and wider transitive dependencies remain unresolved. D017's subset is reused in the Deck/Machine proofs; bounded local Proton import/offline execution passes on Deck. Remote acquisition, a complete host closure and Windows/Linux trust equivalence remain open. Audit exact future binary bundles. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
