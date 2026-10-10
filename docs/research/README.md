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
| [API and safety](api-and-safety.md) | ABI, loader, construction/lifecycle evidence, broader DX12 upscaler surface and bounded D013/D014 native evidence. Sharpening execution passed but its proposed effect gate failed; broader provider, callback, Configure and numerical contracts remain open. |
| [Runtime loading](runtime-loading.md) | Entry DLL, provider discovery and executable-adjacent deployment. |
| [Context lifecycle](context-lifecycle.md) | Retained inputs, failure paths, terminal teardown and observed native cleanup. |
| [Upscaler construction](upscaler-construction.md) | D008's constructor, numerical trust boundary, provider sharing and construction evidence. |
| [Runtime threading](runtime-threading.md) | D013 traits/fixtures and three fenced native transfer scenarios with controls; loader release observed, provider unload unresolved; no accepted movable API. |
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

The [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md)
is integrated in [upscaling dispatch](upscaling-dispatch.md#d014-sharpening-native-execution-and-failed-effect-gate)
and [API and safety](api-and-safety.md). Execution and repeat controls passed;
enabled zero differed from disabled. The proposed effect gate failed in all three
repetitions, confirmed by an independent readback-audit run. D014 remains proposed
and no public sharpening API was implemented.

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | Public v2.3.0 evidence still defines no complete numerical construction domain; accepted D008 uses a bounded runtime trust assumption for M5. Future Query/Configure lifetimes and semantics, broader native concurrency, general returned-create-error and destroy-failure contracts, signed-provider chain/device retention and GPU synchronization remain open. Contradictory safety evidence requires revisiting the M5 contract. |
| Runtime threading | Provider/dependency termination on destination threads, native error/unwind and partial-load transfers, unfenced work, other configurations and sanitizer coverage remain unverified. D013 still requires concrete-integration and lasting caller-contract review before acceptance or public implementation. |
| Upscaling dispatch | Broader temporal quality, frame/provider profiles, signed-provider sharing and GBV remain open. D014 proposes a breaking private-frame API; its sharpening effect policy needs review after the failed native gate. Controlled semantics and sharpening evidence establish no general recording capacity or cross-queue guarantee. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for the tested signed v2.3.0 inputs; exact loading calls, basename collisions and transitive dependencies remain unresolved. Further runtime tests depend on later deployment requirements. Complete the [remaining package and distribution checks](licensing-and-distribution.md); audit exact future binary bundles. Acquisition, caching, verified downloads and offline paths remain undecided. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
