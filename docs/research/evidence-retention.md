# Research evidence retention

- Updated: 2026-10-19 (maintainer-selected checkout budget increase).
- Evidence: [repository size audit](records/2026-10-11-src-repository-size-audit.md),
  [implemented compact retention](../release/2026-10-11-evidence-retention.md),
  and the [calibration retention update](records/2026-10-11-exp-v01-verification-calibration.md#retention-update--11-october-2026).
- Scope: source checkout size and retained experiment evidence; native findings
  and historical GPU output identity are separate questions.

## Current findings

**Verified historical cause:** the audited tree grew from 10,410,980 to
913,355,909 file-content bytes. One calibration evidence directory contributed
902,842,304 bytes / 6,443 files, including three complete exports, compressed
inputs/outputs, verbose analyses and repeated source snapshots. This was evidence
placement and duplication, rather than a missing compression feature. Identical
Git blobs reduce object storage but still occupy every path in a checkout.
Ignored build and SDK directories are separate local disk use. The audit did not
measure registry packages or establish a final cleanup size.

**Superseded recommendation:** the audit initially proposed an external lossless
archive. Its dated update records the maintainer's choice to keep compact results
and remove generated raw data instead. No separate archive was created. The
[implementation report](../release/2026-10-11-evidence-retention.md) records original
inventory validation and exact recomputation of retained pooled measurements
before deletion, plus 47 passing infrastructure tests and blocked-run retention
checks. Those are recorded implementation results, not fresh GPU execution.

**Implemented boundary:** current [research guidance](PROCESS.md),
[contribution rules](../../CONTRIBUTING.md) and
[verification/export guidance](../../tools/verification/README.md#evidence-and-reanalysis)
keep raw diagnostics under ignored `target/`. Compact records preserve protocols,
commands, source/native identities, measurements and failed outcomes. Temporary
full retention needs a concrete diagnosis; exports remain compact. The current
[size check](../../tools/verification/repository_size.py) enforces 50,000,000 bytes
total, 1,000,000 bytes per file and no raw payloads/source snapshots under `docs/`.
Its local inventory includes new non-ignored files, so thousands of small files
cannot bypass the total budget. The maintainer raised the total limit from
30 MB to 50 MB on 2026-10-19 for continued source and compact evidence growth;
the per-file limit and raw-diagnostic restrictions remain unchanged.

## Reproduction limits

Historical full-image reanalysis is unavailable after compaction. Hashes identify
past bytes but cannot reconstruct them; a fresh run needs the recorded SDK and
suitable equipment and may produce different GPU output. Negative and inconclusive
outcomes remain evidence. The [dispatch synthesis](upscaling-dispatch.md#confirmed-temporal-properties-and-metric-limits)
uses the later compact records within these limits.

The audit's pre-cleanup sizes must not be reported as current checkout sizes.
The implementation report's approximate 11 MB candidate likewise is a dated
observation. Run the current size check for the current tree. Neither record
alone verifies all remote refs, clones or final history cleanup; this synthesis
performs no history rewrite, publication or archival operation.

**Intake checksum qualification, 2026-10-12:** across the five later temporal
evidence directories linked from the dispatch synthesis, nine inventoried files
match their retained byte counts/hashes exactly. Another 68 text files match
both only after reconstructing Windows CRLF line endings from this checkout's
LF text. Their JSON parses and the reset/jitter criteria match the current
criteria files. This explains the strict byte-verifier failure on this checkout;
it does not establish an exact retained-byte match as checked out. No historical
inventory or file was rewritten, and no numerical-content discrepancy was found.

**Byte-identity restoration, 2026-10-15:** the 68 qualified text files now retain
their reconstructed Windows CRLF bytes, each verified against its original
recorded size and SHA-256 before replacement. The nine already matching files
and all five inventories are unchanged. All 77 inventoried files now match
byte for byte; scoped `-text` rules preserve those bytes across checkouts.
The [verification record](../release/2026-10-15-line-ending-verification.md)
documents the restoration, UTF-8/LF generation policy and Git round-trip tests.
This resolves the qualified line-ending mismatch, not the earlier removal of
full raw image data. Historical observations and raw-data retention are unchanged.
