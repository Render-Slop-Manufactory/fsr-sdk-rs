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
| [Licensing and distribution](licensing-and-distribution.md) | Header/package audit, [v2.3.0 acquisition and artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md), AMD DLL terms and bounded deployment findings; wrapper archive inspection and runtime dependency closure remain open. |
| [API and safety](api-and-safety.md) | ABI, loader and lifecycle checks; [numerical construction research](records/2026-09-23-src-m5-numeric-construction-safety-contract.md) found no source-supported safe domain for the signed provider. Two contexts coexisted sequentially in a raw probe, and the [public M5 lifecycle](records/2026-09-24-exp-m5-windows-native-verification.md) passed both cleanup cases on one Windows/DX12 configuration. M5 alone establishes no general size or concurrency guarantee; dispatch is tracked in its own topic. |
| [Upscaling dispatch](upscaling-dispatch.md) | M6 source records, [motion/jitter semantics](records/2026-09-24-src-m6-motion-vector-jitter-semantics.md), [open-backend allocation scope](records/2026-09-24-src-m6-backend-allocation-inflight-scope.md), accepted [D011](../adr/D011.md), and [public-route M6b GPU verification](records/2026-09-25-m6b-public-dispatch-verification.md). Signed FSR4 allocator sharing remains unknown under explicit bounded provider trust; no numeric recording limit is promised. |

The [M4 Windows experiment](records/2026-09-22-exp-m4-windows-verification.md)
adds production-owner fixture, native ABI and successful AMD lifecycle evidence
to the API/safety topic; no GPU dispatch was submitted.

Three initial M5 source records add [construction validation](records/2026-09-22-src-m5-upscaler-construction-validation.md),
[Query/Configure scope](records/2026-09-22-src-m5-upscaler-query-configure.md) and
[runtime sharing/threading](records/2026-09-22-src-m5-runtime-sharing-threading.md).
The [integrated findings and provenance limits](api-and-safety.md#m5-source-intake-and-provenance)
include the later repair and signed-runtime experiments. Accepted
[D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
selects the public construction contract; M5 implements it in Rust. Ordinary
Windows CI fixtures passed in the [recorded run](records/2026-09-24-exp-m5-windows-ci-verification.md);
both opt-in native lifecycle cases passed on the tested Windows x64/MSVC
configuration ([native record](records/2026-09-24-exp-m5-windows-native-verification.md)).

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | Public v2.3.0 evidence still defines no complete numerical construction domain; accepted D008 uses a bounded runtime trust assumption for M5. Future Query/Configure lifetimes and semantics, broader native concurrency, general returned-create-error and destroy-failure contracts, signed-provider chain/device retention and GPU synchronization remain open. Contradictory safety evidence requires revisiting the M5 contract. |
| Upscaling dispatch | D011's bounded public method and one submitted GPU readback are complete. The open FSR3 backend trace does not establish signed FSR4 sharing or a public recording limit. Temporal correctness, varying exposure, broader provider support and the native GBV timeout remain open on the tested setup. The [M6 plan](../MILESTONE_6.md) separates completed proof from these follow-ups. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for the tested signed v2.3.0 inputs; exact loading calls, basename collisions and transitive dependencies remain unresolved. Further runtime tests depend on later deployment requirements. Complete the [remaining package and distribution checks](licensing-and-distribution.md); audit exact future binary bundles. Acquisition, caching, verified downloads and offline paths remain undecided. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
