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
| [Licensing and distribution](licensing-and-distribution.md) | Header/package audit, [v2.3.0 acquisition and artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md), AMD DLL terms and bounded deployment findings; archive verification and dependency closure remain open. |
| [API and safety](api-and-safety.md) | ABI, loader and lifecycle checks; [numerical construction research](records/2026-09-23-src-m5-numeric-construction-safety-contract.md) found no source-supported safe domain for the signed provider. Zero dimensions aborted on one configuration; a [width-boundary probe](records/2026-09-23-exp-m5-dx12-dimension-boundary.md) returned generic error `1` at all three widths. Two contexts coexisted sequentially. No general size, concurrency or dispatch contract follows. |

The [M4 Windows experiment](records/2026-09-22-exp-m4-windows-verification.md)
adds production-owner fixture, native ABI and successful AMD lifecycle evidence
to the API/safety topic; no GPU dispatch was submitted.

Three initial M5 source records add [construction validation](records/2026-09-22-src-m5-upscaler-construction-validation.md),
[Query/Configure scope](records/2026-09-22-src-m5-upscaler-query-configure.md) and
[runtime sharing/threading](records/2026-09-22-src-m5-runtime-sharing-threading.md).
The [integrated findings and provenance limits](api-and-safety.md#m5-source-intake-and-provenance)
include the later repair and signed-runtime experiments. No M5 API is selected.

The SDK version is already decided: AMD FSR SDK `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
The following are remaining research questions, not findings:

| Topic | Questions |
|---|---|
| API and safety | M5: a safe numerical construction domain remains unsupported by public v2.3.0 evidence; [D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract) is proposed. Establish any needed signed-provider limits or a reviewed trust assumption, selected Query/Configure lifetimes and semantics, and runtime-sharing/threading obligations. General returned-create-error and destroy-failure contracts, signed-provider chain/device retention and GPU synchronization remain open. |
| SDK acquisition and distribution | Private colocation sufficiency is resolved for the tested signed v2.3.0 inputs; exact loading calls, basename collisions and transitive dependencies remain unresolved. Further runtime tests depend on later deployment requirements. Resolve the [identified package licensing gaps](licensing-and-distribution.md); audit exact future binary bundles. Acquisition, caching, verified downloads and offline paths remain undecided. |

Add linked topic entries as investigations are integrated. Keep open questions
and findings separate; do not populate summaries from unverified plans.
