# Verification script archive — 2026-10-10

This archive retires 19 legacy entry points and preserves four context files whose
originals remained active at intake. It captures the exact working-tree bytes available on
2026-10-10, before relocation. It does not reconstruct every earlier tested source
state or establish a fresh runtime result.

The `.txt` suffix deliberately makes each payload a source artifact rather than an
ordinary executable entry point. The original repository layout is mirrored under
`files/`. [manifest.json](manifest.json) records original paths, byte counts, SHA-256
hashes, dependency groups, direct script dependencies and referring documents.

Documentation compaction, 2026-10-29: superseded release/architecture reports
and verbose 0.1.0/0.2.0 verification exports are retired from the current tree.
Their relevant outcomes and limits are retained in
[release history](../../../../docs/ROADMAP_HISTORY.md) and the
[M9 integration summary](../../../../docs/release/2026-10-10-m9-verification.md).
The manifest's `referenced_by` and `repository_inputs` now provide current
navigation; `intake_referenced_by` and `intake_repository_inputs` preserve the
original lists where navigation changed. Payload paths, bytes, hashes, dependencies
and intake dispositions remain unchanged. No archived source was executed.

## Retired scripts

| Original path | Frozen source | Purpose | Historical references |
|---|---|---|---|
| `crates/fsr-sdk/tests/run-m4.ps1` | [source](files/crates/fsr-sdk/tests/run-m4.ps1.txt) | M4 lifecycle alias | [MILESTONE_4](../../../../docs/ROADMAP_HISTORY.md#010--published-2026-09-28), [2026-09-30-development-history](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-09-22-exp-m4-windows-verification](../../../../docs/research/records/2026-09-22-exp-m4-windows-verification.md) |
| `crates/fsr-sdk/tests/run-m5.ps1` | [source](files/crates/fsr-sdk/tests/run-m5.ps1.txt) | M5 lifecycle alias | [2026-09-30-development-history](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-09-30-review-verification](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-09-24-exp-m5-windows-native-verification](../../../../docs/research/records/2026-09-24-exp-m5-windows-native-verification.md), [2026-09-30-exp-dispatch-semantics](../../../../docs/research/records/2026-09-30-exp-dispatch-semantics.md) |
| `crates/fsr-sdk/tests/run-m6.ps1` | [source](files/crates/fsr-sdk/tests/run-m6.ps1.txt) | M6 dispatch alias | [2026-09-30-development-history](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-09-30-review-verification](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-09-24-exp-m6a-native-dispatch-proof](../../../../docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md), [2026-09-25-m6b-public-dispatch-verification](../../../../docs/research/records/2026-09-25-m6b-public-dispatch-verification.md), [2026-09-28-exp-final-windows-release-verification](../../../../docs/research/records/2026-09-28-exp-final-windows-release-verification.md), [2026-09-30-exp-d013-native-thread-transfer](../../../../docs/research/records/2026-09-30-exp-d013-native-thread-transfer.md), [2026-09-30-exp-dispatch-semantics](../../../../docs/research/records/2026-09-30-exp-dispatch-semantics.md) |
| `crates/fsr-sdk/tests/run-m6-recording-spike.ps1` | [source](files/crates/fsr-sdk/tests/run-m6-recording-spike.ps1.txt) | Recording-ahead control and bounded same/sibling matrix | [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-24-exp-m6-recording-ahead](../../../../docs/research/records/2026-09-24-exp-m6-recording-ahead.md) |
| `crates/fsr-sdk/tests/run-m6-gbv-spike.ps1` | [source](files/crates/fsr-sdk/tests/run-m6-gbv-spike.ps1.txt) | GBV control/dispatch diagnostic | [2026-09-30-development-history](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-24-exp-m6-gbv-dispatch-stall](../../../../docs/research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md) |
| `crates/fsr-sdk-sys/tests/context_dimensions/run.ps1` | [source](files/crates/fsr-sdk-sys/tests/context_dimensions/run.ps1.txt) | Dimension cases and raw create outcomes | [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-22-exp-m5-upscaler-construction-inputs](../../../../docs/research/records/2026-09-22-exp-m5-upscaler-construction-inputs.md), [2026-09-23-exp-m5-dx12-dimension-boundary](../../../../docs/research/records/2026-09-23-exp-m5-dx12-dimension-boundary.md) |
| `crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1` | [source](files/crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1.txt) | Allocation failure and returned-error lifetime probes; ordinary lifecycle alias | [2026-09-30-development-history](../../../../docs/ROADMAP_HISTORY.md#september-maintenance-and-test-hygiene), [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-20-exp-context-lifecycle](../../../../docs/research/records/2026-09-20-exp-context-lifecycle.md), [2026-09-20-exp-create-allocation-failure](../../../../docs/research/records/2026-09-20-exp-create-allocation-failure.md), [2026-09-22-exp-create-error-input-lifetime](../../../../docs/research/records/2026-09-22-exp-create-error-input-lifetime.md) |
| `crates/fsr-sdk-sys/tests/provider_resolution/run.ps1` | [source](files/crates/fsr-sdk-sys/tests/provider_resolution/run.ps1.txt) | Four executable-adjacent/provider deployment layouts | [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-22-exp-dx12-provider-dll-resolution](../../../../docs/research/records/2026-09-22-exp-dx12-provider-dll-resolution.md) |
| `crates/fsr-sdk-sys/tests/shared_runtime/run.ps1` | [source](files/crates/fsr-sdk-sys/tests/shared_runtime/run.ps1.txt) | Sequential shared runtime lifetime probe | [2026-10-09-windows-native-recheck](../../../../docs/ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh), [2026-09-22-exp-m5-shared-runtime-provider-lifetime](../../../../docs/research/records/2026-09-22-exp-m5-shared-runtime-provider-lifetime.md) |
| `docs/release/2026-10-10-m9-verification/prepare-candidate.py` | [source](files/docs/release/2026-10-10-m9-verification/prepare-candidate.py.txt) | Prepare the historical movable-excluding projection | [2026-10-10-m9-verification](../../../../docs/release/2026-10-10-m9-verification.md) |
| `docs/release/2026-10-10-m9-verification/verify-candidate.py` | [source](files/docs/release/2026-10-10-m9-verification/verify-candidate.py.txt) | Verify projected candidate, archives and isolated consumers | [2026-10-10-m9-verification](../../../../docs/release/2026-10-10-m9-verification.md) |
| `docs/release/2026-10-10-m9-verification/collect-evidence.py` | [source](files/docs/release/2026-10-10-m9-verification/collect-evidence.py.txt) | Independent half decoding and native evidence collection | [2026-10-10-m9-verification](../../../../docs/release/2026-10-10-m9-verification.md) |
| `docs/release/2026-10-10-m9-verification/save-artifact-evidence.py` | [source](files/docs/release/2026-10-10-m9-verification/save-artifact-evidence.py.txt) | Export projected artifact evidence into the historical directory | [2026-10-10-m9-verification](../../../../docs/release/2026-10-10-m9-verification.md) |
| `docs/release/2026-10-10-0.2.0-release-verification/verify-release.py` | [source](files/docs/release/2026-10-10-0.2.0-release-verification/verify-release.py.txt) | Verify complete release packages and registry consumers | [2026-10-10-0.2.0-release-verification](../../../../docs/ROADMAP_HISTORY.md#release-qualification) |
| `docs/release/2026-10-10-0.2.0-release-verification/collect-release.py` | [source](files/docs/release/2026-10-10-0.2.0-release-verification/collect-release.py.txt) | Export complete release evidence and invoke M9 audit | [2026-10-10-0.2.0-release-verification](../../../../docs/ROADMAP_HISTORY.md#release-qualification) |
| `docs/release/2026-10-10-0.2.0-release-verification/audit-release.py` | [source](files/docs/release/2026-10-10-0.2.0-release-verification/audit-release.py.txt) | Content/navigation/provenance audit with historical output | [2026-10-10-0.2.0-release-verification](../../../../docs/ROADMAP_HISTORY.md#release-qualification) |
| `docs/research/records/2026-09-30-exp-d014-native-sharpening/audit-readback.ps1` | [source](files/docs/research/records/2026-09-30-exp-d014-native-sharpening/audit-readback.ps1.txt) | Independent sharpening half decoder and original metrics | [2026-09-30-exp-d014-native-sharpening](../../../../docs/research/records/2026-09-30-exp-d014-native-sharpening.md) |
| `docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/audit-results.ps1` | [source](files/docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/audit-results.ps1.txt) | D013 cleanup matrix/payload audit and evidence export | [2026-10-08-exp-d013-native-transfer-cleanup](../../../../docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md) |
| `docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/run-ordinary-checks.ps1` | [source](files/docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/run-ordinary-checks.ps1.txt) | Dated supplementary Cargo check sequence | [2026-10-08-exp-d013-native-transfer-cleanup](../../../../docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md) |

## Context snapshots, with active originals retained

| Original at intake | Frozen intake source |
|---|---|
| [crates/fsr-sdk/tests/native/run.ps1](files/crates/fsr-sdk/tests/native/run.ps1.txt) | [source](files/crates/fsr-sdk/tests/native/run.ps1.txt) |
| [tools/check-native-abi.ps1](files/tools/check-native-abi.ps1.txt) | [source](files/tools/check-native-abi.ps1.txt) |
| [tools/check-movable-api.ps1](files/tools/check-movable-api.ps1.txt) | [source](files/tools/check-movable-api.ps1.txt) |
| [tools/native-headers.json](../../../native-headers.json) | [source](files/tools/native-headers.json.txt) |

## Dependency and outcome boundaries

### milestone-wrappers

Retired M4/M5/M6 aliases; lifecycle/dispatch remain available through the active shared native runner.

Wrappers delegate to lifecycle or dispatch; they are not independent verification cases.

- Original crate-relative layout; PowerShell; Cargo and cached declared dependencies. Native mode additionally needs the native-regressions requirements.

Repository context: [`crates/fsr-sdk/tests/native/run.ps1`](files/crates/fsr-sdk/tests/native/run.ps1.txt).

### native-diagnostics

Frozen recording-ahead, GBV, dimension, allocation/input-lifetime, provider-deployment and shared-runtime experiments. Their Rust/C++ equipment remains in the repository for future migration.

Recording-ahead aggregates selected failures; GBV gates its control but records dispatch timeout; dimensions/shared-runtime record child statuses; provider deployment includes expected negative layouts; allocation/input-lifetime controls and raw aborts retain their original meanings. Parent exit 0 is not a universal child-pass claim.

- Windows x64/MSVC developer shell; trusted local SDK v2.3.0 headers and signed loader/provider deployment; applicable hardware/debug-layer requirements.
- Original script paths and a compatible source state. Each script owns its staging, watchdog and environment rules; fixed stages can overwrite earlier attempts.

Repository context: [`Cargo.toml`](../../../../Cargo.toml), [`Cargo.lock`](../../../../Cargo.lock), [`crates/fsr-sdk/Cargo.toml`](../../../../crates/fsr-sdk/Cargo.toml), [`crates/fsr-sdk-sys/Cargo.toml`](../../../../crates/fsr-sdk-sys/Cargo.toml), [`crates/fsr-sdk/tests/native`](../../../../crates/fsr-sdk/tests/native), [`crates/fsr-sdk/src`](../../../../crates/fsr-sdk/src), [`crates/fsr-sdk-sys/src`](../../../../crates/fsr-sdk-sys/src), [`crates/fsr-sdk-sys/tests/context_lifecycle`](../../../../crates/fsr-sdk-sys/tests/context_lifecycle), [`crates/fsr-sdk-sys/tests/context_dimensions.rs`](../../../../crates/fsr-sdk-sys/tests/context_dimensions.rs), [`crates/fsr-sdk-sys/tests/context_lifecycle.rs`](../../../../crates/fsr-sdk-sys/tests/context_lifecycle.rs), [`crates/fsr-sdk-sys/tests/query.rs`](../../../../crates/fsr-sdk-sys/tests/query.rs), [`crates/fsr-sdk-sys/tests/provider_resolution/modules.rs`](../../../../crates/fsr-sdk-sys/tests/provider_resolution/modules.rs), [`crates/fsr-sdk-sys/tests/shared_runtime.rs`](../../../../crates/fsr-sdk-sys/tests/shared_runtime.rs).

### m9-release

Historical projected candidate that excluded the public movable family; superseded for publication by the complete 0.2.0 record, not rewritten to match it.

Projected candidate/archive/consumer acceptance is separate from image diagnostics and publication. The preparation script expects pre-0.2.0 source and is not a current release recipe.

- Python standard library; recorded Cargo/stable/MSRV tools and cached registry archives; the pre-projection source expected by prepare-candidate.py.
- Generated target/m9-release-20261010-final source, archives, isolated registry, check logs and native stage/readback inputs. Collectors write beside their original scripts.

Repository context: [`Cargo.toml`](../../../../Cargo.toml), [`Cargo.lock`](../../../../Cargo.lock), [`LICENSE`](../../../../LICENSE), [`crates`](../../../../crates), [`docs/release/2026-10-10-m9-verification.md`](../../../../docs/release/2026-10-10-m9-verification.md), [`docs/release/2026-10-09-m9-native-sharpening-protocol.md`](../../../../docs/release/2026-10-10-m9-verification.md#integration-protocol).

### complete-release

Historical complete 0.2.0/0.1.1 verification, including movable, archive/registry isolation, content audit and evidence collection.

Expected compiler errors are passing negative controls. Resume reuse is label/exit based, not input-checked. Package verification, native integration, content audit and publication observation remain distinct.

- Python standard library; recorded Cargo/stable/MSRV tools and cached registry archives; exact candidate/package inputs.
- Generated target/release-020-20261010 archives, registry, check/dry-run logs, API controls, source/artifact movable stages and M9 artifact readbacks.
- collect-release.py imports the frozen M9 collector at its original path. Collectors/audits write into their original historical evidence directories; use a disposable reconstruction only.

Repository context: [`Cargo.toml`](../../../../Cargo.toml), [`Cargo.lock`](../../../../Cargo.lock), [`LICENSE`](../../../../LICENSE), [`README.md`](../../../../README.md), [`crates`](../../../../crates), [`docs/release/2026-10-10-0.2.0-release-verification.md`](../../../../docs/ROADMAP_HISTORY.md#release-qualification), [`docs/release/2026-10-09-m9-native-sharpening-protocol.md`](../../../../docs/release/2026-10-10-m9-verification.md#integration-protocol), [`docs/research/records/2026-09-30-exp-d014-native-sharpening.md`](../../../../docs/research/records/2026-09-30-exp-d014-native-sharpening.md), [`crates/fsr-sdk/tests/native/sharpening.rs`](../../../../crates/fsr-sdk/tests/native/sharpening.rs).

### dated-audits

Original independent sharpening decoder and D013 cleanup matrix collector/ordinary checks.

Preserve failed sharpening criteria and the D013 collector's bounded cleanup/payload comparisons. These are not general native safety or image-quality gates.

- PowerShell 7/.NET for the half decoder; Cargo/stable/MSRV tools for ordinary checks.
- Recorded target-stage logs, source identities and raw readbacks; D013 collector uses the repository as its working directory and writes beside its original script.

Repository context: [`docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md`](../../../../docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md), [`docs/research/records/2026-09-30-exp-d014-native-sharpening.md`](../../../../docs/research/records/2026-09-30-exp-d014-native-sharpening.md), [`crates/fsr-sdk/tests/native/thread_transfers.rs`](../../../../crates/fsr-sdk/tests/native/thread_transfers.rs), [`crates/fsr-sdk/tests/native/sharpening.rs`](../../../../crates/fsr-sdk/tests/native/sharpening.rs), [`Cargo.toml`](../../../../Cargo.toml), [`Cargo.lock`](../../../../Cargo.lock).

### native-regressions

Archive-time context snapshots; original files remain active and are not retired by this change.

Active shared runner/ABI/API checks retain their behavior. Frozen copies explain dependencies of retired scripts, not the exact equipment used by every earlier run.

- PowerShell 7; Cargo; Windows x64/MSVC/local SDK and recorded hardware for native execution. CI ABI mode may fetch hash-pinned headers.

Repository context: [`Cargo.toml`](../../../../Cargo.toml), [`Cargo.lock`](../../../../Cargo.lock), [`crates`](../../../../crates), [`.github/workflows/ci.yml`](../../../../.github/workflows/ci.yml), [`docs/DEVELOPMENT.md`](../../../../docs/DEVELOPMENT.md).

## Reading and reconstruction

Use the linked records for actual commands, environment, results and limitations.
Original experiment logs, JSON, CSV, protocols and fingerprints remain with the
retained research records. Superseded operational release exports are now in Git
history; their summaries distinguish qualification, failed diagnostics and actual
publication. Current navigation changes do not alter frozen script command text
or the original experimental measurements.

To inspect or reconstruct an old workflow, first verify the archive, then copy the
needed payloads into a disposable source tree at their `original` manifest paths,
removing only the final `.txt` suffix. Include each direct `depends_on` entry and
the repository/external/generated inputs listed for its group. Restore the original
relative layout: PowerShell uses `$PSScriptRoot`, Python calculates its repository
root from `__file__`, and the complete-release collector imports the M9 collector.

This archive contains scripts/configuration, not a complete historical source tree,
AMD distribution, compiler, registry cache, package archives or raw GPU readbacks.
Current Rust/C++ equipment remains active and can evolve. Intake snapshots alone
cannot recreate the pre-projection M9 source or prove historical hash identity;
compare the relevant record before claiming a reproduction.

Restored collectors can replace the evidence beside their original paths. Keep
reconstructed runs and exports in that disposable tree, never in these completed
repository records. Resume and diagnostic outcome limitations are preserved above.

## Verification of this relocation

The [relocation checks](relocation-checks.json) record the archive-only validation:
all 23 payload identities matched, 24 referring historical documents changed only
in script navigation and a dated note, and 438 other historical evidence files
remained byte-identical. Recorded command blocks, test logic/selectors, active
runner implementations and CI inputs were preserved. Formatting, Windows test
compilation with movable enabled, both package file inventories and the archive
checker passed. No archived workflow or native/GPU case was executed.

From the repository root:

```sh
python tools/verification/archive/check.py
```

The read-only checker verifies payload identity, inventory completeness, direct
dependency closure, referring-document navigation and frozen Python/PowerShell
syntax. It executes no archived verification workflow, AMD code or GPU workload.

Ordinary Cargo tests, ABI/consumer CI checks and the shared native runner remain
active. Standalone experiment launchers and dated release recipes are retired;
their coverage is preserved as source/evidence, not claimed to have a new active
replacement. New orchestration is a separate follow-up.

## Current orchestration after intake — 2026-10-10

The native runner, ABI launcher and consumer launcher above are now retired too.
Their existing frozen intake copies remain byte-identical; the manifest records
intake disposition, not current command availability. `tools/native-headers.json`
remains active. [Current verification](../../README.md) owns replacement commands
and case inventories. Statements about active runners above describe the separate
archive-only intake, before this orchestration migration. Historical findings and
relocation checks are unchanged.
