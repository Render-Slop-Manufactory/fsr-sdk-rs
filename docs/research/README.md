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
| [API and safety](api-and-safety.md) | ABI, loader, construction/lifecycle evidence and the [broader DX12 upscaler surface](records/2026-09-29-src-dx12-upscaler-api-surface.md). M5 lifecycle and final post-dispatch teardown passed on the recorded configuration; broader provider, callback, Configure and numerical contracts remain open. |
| [Runtime loading](runtime-loading.md) | Entry DLL, provider discovery and executable-adjacent deployment. |
| [Context lifecycle](context-lifecycle.md) | Retained inputs, failure paths, terminal teardown and observed native cleanup. |
| [Upscaler construction](upscaler-construction.md) | D008's constructor, numerical trust boundary, provider sharing and construction evidence. |
| [Runtime threading](runtime-threading.md) | Proposed API evolution and D013's bounded transfer fixtures; no accepted movable API. |
| [Upscaling dispatch](upscaling-dispatch.md) | D011's public route, the regression/three-frame smoke and [controlled motion/jitter/reset comparisons](records/2026-09-30-exp-dispatch-semantics.md). Broader frame capabilities are source research; general temporal quality, GBV and signed-provider allocation/sharing remain unresolved. |

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
are integrated in [API and safety](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles):
trait assumptions and all eight D013 probes pass on macOS ARM64 and Windows
x64/MSVC, including serialized Rc/Arc lifecycles and Windows DLL retention/release.
AMD transfer behavior and native refactor regressions remain unresolved.
[D013](../adr/D013.md) remains proposed and has no public movable API.

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | Public v2.3.0 evidence still defines no complete numerical construction domain; accepted D008 uses a bounded runtime trust assumption for M5. Future Query/Configure lifetimes and semantics, broader native concurrency, general returned-create-error and destroy-failure contracts, signed-provider chain/device retention and GPU synchronization remain open. Contradictory safety evidence requires revisiting the M5 contract. |
| Upscaling dispatch | Temporal/history/reset correctness, broader frame/provider profiles, signed-provider sharing and GBV remain open after the bounded smoke. Before a per-frame extension, choose an additive extensible input or a breaking private-frame API. Source inventory establishes no new runtime support, recording capacity or cross-queue guarantee. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for the tested signed v2.3.0 inputs; exact loading calls, basename collisions and transitive dependencies remain unresolved. Further runtime tests depend on later deployment requirements. Complete the [remaining package and distribution checks](licensing-and-distribution.md); audit exact future binary bundles. Acquisition, caching, verified downloads and offline paths remain undecided. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
