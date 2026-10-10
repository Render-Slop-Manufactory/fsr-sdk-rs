# Roadmap history

Completed checkpoints, grouped by release baseline. Publication is recorded
separately from implementation and verification. Open work lives in the
[rolling roadmap](../ROADMAP.md); dated records below retain the tested inputs,
observations and limits.

## 0.2.0 — verified 2026-10-10

Baseline: `fsr-sdk 0.2.0` with `fsr-sdk-sys 0.1.1`, verified on 2026-10-10.
The maintainer selected optional movable ownership for this release, superseding
the earlier exclusion. Publication was pending at verification; `R01` tracks
the maintainer's publication handoff.

| Checkpoint | Completed work | Record |
|---|---|---|
| M7 — Dispatch contract | Extensible frames and explicit native sharpening; migration from public frame literals | [D014](adr/D014.md) |
| M8 — Frame implementation | Private resources/frame construction, accessors, exact sharpening forwarding and failure controls | [Implementation checks](release/2026-10-09-m8-implementation-verification.md) |
| M9 — Native integration and artifacts | Public sharpening, same-context mode changes and candidate packages; original projection excluded movable | [Original M9 checks](release/2026-10-10-m9-verification.md) |
| Movable ownership | Optional `Send + !Sync` family, coordinated handoffs and cleanup; included in 0.2.0 | [Public API checks](release/2026-10-09-d013-public-api-verification.md) |
| Common inputs and loader organization | Portable checked values, preserved imports, Windows/DX12-only production loader dependency | [Common input checks](release/2026-10-09-portable-common-verification.md), [loader audit](release/2026-10-09-loader-refactor-audit.md) |
| Combined release verification | Actual source/packages, registry-only consumers, stable/MSRV, native movable + sharpening and extracted-package GPU checks | [Complete release record](release/2026-10-10-0.2.0-release-verification.md) |

Native support remains Windows x64/MSVC with the pinned SDK v2.3.0. Results cover
the recorded SDK/provider/GPU/driver/workload, not general hardware compatibility
or image quality. The publication dry-run aborted both uploads; no hosted
0.2.0 publication is established by these records.

The failed original sharpening-effect criteria and unresolved GPU-based
validation stall remain open work (`V02` and `V04`). The combined release passed
its separately selected execution gates; it did not change those earlier results.
Movable's accepted native-compatibility and coordination assumptions remain in
force; `V06` tracks further investigation.

## 0.1.0 — published 2026-09-28

Both crates shipped as `0.1.0`, followed by GitHub `v0.1.0`.
The [publication record](release/2026-09-29-publication-check.md) identifies the
public registry entries and hosted documentation.

| Checkpoint | Completed work | Record |
|---|---|---|
| M1–M3 — Native foundation | Selected common ABI, explicit DLL loading and provider/version query probes | [ABI inventory](ABI_COVERAGE.md), [development history](release/2026-09-30-development-history.md) |
| M4 — Context ownership | Runtime retention and terminal native context cleanup | [Windows verification](research/records/2026-09-22-exp-m4-windows-verification.md) |
| M5 — Public construction | Checked dimensions/options, retained runtime/device, owned errors | [Native verification](research/records/2026-09-24-exp-m5-windows-native-verification.md) |
| M6 — Public dispatch | Fixed-profile recording, submitted/fenced readback and a moving smoke | [Public dispatch](research/records/2026-09-25-m6b-public-dispatch-verification.md), [final Windows checks](research/records/2026-09-28-exp-final-windows-release-verification.md) |
| Release artifacts | Extracted packages, consumer checks and retained license notices | [Artifact recheck](release/2026-09-28-final-artifact-recheck.md) |

These checkpoints establish the initial Windows/DX12 path. They do not establish
complete SDK coverage or general provider, performance or image-quality support.
