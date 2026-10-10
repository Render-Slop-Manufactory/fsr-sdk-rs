# Research

Research for the engine-independent AMD FSR SDK Rust wrapper. Project intent is
in [SCOPE.md](../../SCOPE.md); source and manifests establish implemented
behavior. Findings here do not imply SDK support or accepted design decisions.

## Start here

Read the [research process](PROCESS.md) for evidence standards and integration.
Use the [source record](templates/source.md),
[experiment record](templates/experiment.md), or
[topic synthesis](templates/topic.md) template as appropriate.
Historical evidence goes in [records/](records/); current topic syntheses live
beside this file and link the records they use.

## Coverage

| Current topic | Evidence and coverage |
|---|---|
| [Licensing and distribution](licensing-and-distribution.md) | Header/package audit, [v2.3.0 acquisition and artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md), AMD DLL terms and bounded deployment findings. Released wrapper archives were inspected; broader runtime dependency closure remains open. |
| [API and safety](api-and-safety.md) | ABI, loader, construction/lifecycle evidence, broader DX12 surface, D013 source/fixture/native evidence and conditional-contract review. Sharpening's proposed effect gate failed; broader provider contracts remain open. |
| [Runtime loading](runtime-loading.md) | Entry DLL, provider discovery, executable-adjacent deployment, D013 worker acquisition failures and staged native residency after application refs/worker exit; provider termination remains unattributed. |
| [Context lifecycle](context-lifecycle.md) | Retained inputs, native cleanup, D013 transferred failure fixtures and successful fenced native Rust-unwind destruction/sibling survival; D005's bounded trust remains in force. |
| [Upscaler construction](upscaler-construction.md) | D008's constructor, numerical trust boundary, provider sharing and construction evidence. |
| [Runtime threading](runtime-threading.md) | D013 traits/failure fixtures, extended fenced native transfers and a soundness review recommending acceptance with contract revisions. AMD guarantees/termination remain unresolved; maintainer acceptance and public API verification remain open. |
| [Upscaling dispatch](upscaling-dispatch.md) | D011's public route, temporal smoke, controlled semantics and D014's native sharpening comparisons. Enabled zero differs from disabled; the proposed effect gate failed. General temporal quality, GBV and signed-provider allocation/sharing remain unresolved. |

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

The [D013 compile/host experiment](records/2026-09-29-exp-d013-movable-compile-host.md)
and [Windows fixture execution](records/2026-09-29-exp-d013-windows-fixtures.md)
are integrated in [runtime threading](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles):
trait assumptions and all eight D013 probes pass on macOS ARM64 and Windows
x64/MSVC, including serialized Rc/Arc lifecycles and Windows DLL retention/release.
These fixtures establish ownership behavior, not AMD transfer compatibility.
[D013](../adr/D013.md) remains proposed and has no public movable API.

The [native D013 experiment](records/2026-09-30-exp-d013-native-thread-transfer.md)
is integrated in [runtime threading](runtime-threading.md#d013-fenced-native-transfers-and-final-release)
and summarized in [API and safety](api-and-safety.md). Three fenced transfer
scenarios, their same-thread controls and current native lifecycle/dispatch
regressions pass on one recorded AMD configuration. Final loader release/unload
was observed on the destination thread; the provider DLL remained loaded in
both controls and transfers. General transfer guarantees and provider termination
remain unresolved; D013 remains proposed.

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
accepted bounded RAII trust assumption. D013 remains proposed.

The [2026-10-08 native follow-up](records/2026-10-08-exp-d013-native-transfer-cleanup.md)
is integrated in [runtime threading](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency),
API/safety, lifecycle and loading. All 11 prerequisites and 30 new children pass,
including 12 actual Rust-unwind cleanups and 96 selected payload comparisons.
It closes the dispatch-policy extraction's native regression gap. The provider
persists after all audited application refs and worker exit; its termination
remains unresolved. D013 remains proposed.

The [D013 soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md)
was renamed from `neuer_record.md` and integrated in
[runtime threading](runtime-threading.md#d013-soundness-and-architecture-review),
API/safety, lifecycle and loading. No investigation date or reviewed revision was
supplied; 2026-10-09 is the intake date. It recommends acceptance with stronger
arbitrary-destination and lasting serialization/cleanup obligations, without
another exploratory native phase. It adds no execution evidence or AMD guarantee.
Concrete integration and explicit maintainer acceptance remain open; D013 remains
proposed, with actual public-path verification required after implementation.

The [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#d014-sharpening-native-execution-and-failed-effect-gate)
and [API and safety](api-and-safety.md). Execution and repeat controls passed;
enabled zero differed from disabled. The proposed effect gate failed in all three
repetitions, confirmed by an independent readback-audit run. The maintainer
separately accepted [D014](../adr/D014.md) on 2026-10-07; its public sharpening
API is unimplemented and its new release protocol remains open. This status
update adds no research evidence.

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | Public v2.3.0 evidence still defines no complete numerical construction domain; accepted D008 uses a bounded runtime trust assumption for M5. Future Query/Configure lifetimes and semantics, broader native concurrency, general returned-create-error and destroy-failure contracts, signed-provider chain/device retention and GPU synchronization remain open. Contradictory safety evidence requires revisiting the M5 contract. |
| Runtime threading | Provider/dependency termination and retention cause, native returned-error/partial-load cleanup, hidden work/TLS, external-user coordination, outstanding GPU work, other configurations and sanitizers remain open. The review recommends a strengthened unsafe contract without further exploratory execution; it resolves no missing AMD guarantee. Concrete integration and maintainer acceptance remain required; implementation/public support verification are separate. |
| Upscaling dispatch | Broader temporal quality, frame/provider profiles, signed-provider sharing and GBV remain open. D014's breaking private-frame API is accepted but unimplemented; its separate sharpening release protocol remains open after the failed native gate, including same-context switching comparisons. Controlled semantics and sharpening evidence establish no general recording capacity or cross-queue guarantee. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for the tested signed v2.3.0 inputs; exact loading calls, basename collisions and transitive dependencies remain unresolved. Further runtime tests depend on later deployment requirements. Complete the [remaining package and distribution checks](licensing-and-distribution.md); audit exact future binary bundles. Acquisition, caching, verified downloads and offline paths remain undecided. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
