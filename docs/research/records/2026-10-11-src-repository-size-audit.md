# Source investigation: repository growth from retained calibration evidence

Retention decision update, 11 October 2026: the maintainer selected compact
results and temporary generated raw data. The separate-archive recommendation
below was superseded before implementation. See the
[dated calibration retention update](2026-10-11-exp-v01-verification-calibration.md#retention-update--11-october-2026).
The original size measurements below describe the pre-cleanup state.

- Investigation date: 11 October 2026.
- Question: why did the last commit make the project hundreds of megabytes,
  and how can the repository remain small without losing experiment evidence?
- Baseline: `338db68`, compared with `13e06347fc48be15d2344b459001160ba2b94e49`
  (`implement V01 spike and synthesis`). The initial worktree was clean.
- Method: Git tree sizes and blob identities, local directory sizes, manifests,
  evidence records and current export code. No GPU execution, deletion,
  history rewrite, upload or publication was performed.
- Units: MB and GB below are decimal; tree sizes are file-content bytes,
  excluding filesystem allocation and Git storage overhead.

## Findings

**Verified:** the committed tree increased from 10,410,980 bytes / 868 files
to 913,355,909 bytes / 7,317 files. The increase is 902,944,929 bytes.
The immediate preceding committed tree was therefore 10.4 MB, rather than 1 MB.

The adjacent evidence directory for the
[temporal calibration experiment](2026-10-11-exp-v01-verification-calibration.md)
accounts for 902,842,304 bytes across 6,443 files: more than 99.98% of the net
growth. Its principal subdirectories are:

| Directory | File-content bytes | Files |
|---|---:|---:|
| `attempt-1` | 289,964,249 | 2,009 |
| `attempt-3` | 287,884,440 | 2,004 |
| `attempt-4` | 305,097,392 | 2,036 |
| `reanalysis-attempt-1` | 15,149,393 | 30 |
| `check-renamed-inputs` | 2,381,014 | 171 |

The experiment record explains the growth: attempts 1 and 3 failed evidence
processing after completing the native matrices; attempt 4 completed the
structural profile. All three complete matrices were exported and committed.
Each includes 903 output images, 903 input payloads and 21 prerequisite inputs.
The failures and their data are legitimate evidence, but full retention does
not require inclusion in every source checkout.

**Verified:** within this new evidence directory, compressed RGBA16F outputs
occupy 731,348,325 bytes and compressed inputs 124,304,087 bytes. Together they
occupy 855,652,412 bytes. Removing only these payloads would still leave a
57,703,497-byte project. Full frame analyses, reanalyses, source snapshots,
logs and inventories also require a deliberate retention boundary.

**Verified:** grouping this directory's files by identical Git blob identity
yields only 729 distinct blobs, totalling 94,906,960 bytes. Distinct compressed
payload blobs total 82,404,655 bytes. About 807.9 MB of checkout content repeats
already present bytes. This is byte duplication, not an assertion that distinct
attempts or their provenance are interchangeable. A deduplicated archive must
preserve every original path, identity and attempt outcome.

**Verified:** local `.git` file contents totalled 78,263,450 bytes at inspection;
`git count-objects -v` reported about 74.4 MiB of loose objects and 279 KiB of
packs. Git already shares identical blobs. Packing Git objects cannot eliminate
the 902.8 MB of checked-out evidence. Download size was not measured.

Local `target` file contents separately totalled 13,332,349,899 bytes and
`external` 661,350,603 bytes. These are ignored build/verification outputs and
SDK material. They explain additional local disk use, but are separate from
the committed-tree regression. Cleanup must preserve the desired local evidence
before removing disposable builds; blindly clearing `target` is inappropriate.

**Verified:** the remote `refs/heads/main`, read through `git ls-remote`, points
to the same full commit ID as the audited local HEAD. The offending commit is
already published. No tag points directly at local HEAD. Other remote refs,
forks and existing clones were not exhaustively inventoried.

## Cause and existing boundaries

[records.py](../../../tools/verification/records.py) exports complete validated
readbacks and source snapshots into an explicitly selected destination. The
calibration implementation creates deterministic, losslessly compressed
payloads; compression is already present. This is an evidence placement and
retention problem, rather than a missing compression feature.

The [research process](../PROCESS.md) asks for proportional, self-contained
records and useful sanitized supporting artefacts. It does not require every
raw frame to be committed. The [verification documentation](../../../tools/verification/README.md)
currently permits publication exports into repository evidence directories and
does not specify a repository size budget. [.gitignore](../../../.gitignore)
ignores `target` and downloaded SDK material but does not guard research exports.
The reviewed CI workflow has no committed-tree size check.

The large directory is outside both crate package roots. It is not automatically
part of downstream crate archives through those manifests. No new package build
or registry-package size check was performed for this source-only audit.

## Recommended repair, not yet implemented

1. Preserve and validate the complete evidence directory before removal. Export
   it to an explicitly fetched archive, with SHA256, byte size, original file
   inventory, retained licences and exact restoration instructions. Start with
   a complete lossless archive; deduplication is optional and must reconstruct
   the original directory and pass its inventory checks. Do not discard failed
   attempts or infer that hashes alone preserve raw images.
2. Keep the self-contained experiment report, concise diagnostic results,
   attempt identities/statuses, frozen protocol, representative contact sheet,
   and archive reference in Git. Full raw frames, verbose per-frame analysis,
   repeated source snapshots and complete export inventories belong in the
   archive. Update affected navigation/reanalysis commands transparently.
   Preserve historical conclusions and their limitations.
3. Host the archive separately from Git objects, for example as an explicitly
   labelled research-evidence GitHub Release asset. GitHub documents release
   assets as an alternative to tracking large files. This is a proposed storage
   choice; no destination, URL or upload has been created. Ordinary builds and
   CI must remain independent of this optional evidence download.
4. Introduce a portable CI check of the whole tracked-file budget, covering
   staged additions as well as existing files. A proposed initial budget is
   15 MB for the entire committed tree and 1 MB per file, with deliberate,
   documented review for exceptions. A total limit is essential: thousands of
   sub-megabyte files caused this regression. Keep raw exports under ignored
   locations; ignore rules alone do not protect already tracked files or forced
   additions. Document this boundary in contribution/research/export guidance.
5. After verifying preservation and the replacement tree, replace the last
   published commit with one containing the implementation and compact records.
   Earlier history need not be rewritten for this particular addition.
   Inspect affected refs first and use a push lease pinned to the observed old
   main commit. Coordinate existing clones and retain a recoverable backup.
   Published crate releases and older release tags need no change merely to
   remove this later evidence addition. A subsequent deletion commit alone
   retains the large blobs in main's reachable history.

Excluding the entire new evidence directory gives a measured tree size of
10,513,605 bytes before adding any retained compact evidence. Approximately
10–12 MB is a reasonable candidate target, rather than a verified final size.
Measure the actual replacement checkout and a fresh clone before declaring the
repair complete. Check links, restored archive identities, evidence availability
and the size guard's acceptance/rejection cases; rerunning GPU experiments is
not necessary for a lossless storage migration.

Git LFS is an alternative if these files become regularly edited test inputs.
For optional historical evidence, an explicit archive better matches the desired
small ordinary checkout. Merely putting one large ZIP in Git also fails the
repository-size objective. CI's existing 14-day artefact retention is unsuitable
as the only durable home of completed research evidence.

## Reproduction and limits

The measurements can be reproduced with `git ls-tree -r -l HEAD` and the same
command for `HEAD^`: sum the size field, group by path, and group by blob ID to
measure duplicate file content. Use `git count-objects -v` for object storage and
`git ls-remote --heads origin main` for the published main identity. The tables
above retain the relevant observed values independently of access to these
historical commits.

Primary external sources, accessed 11 October 2026:

- [GitHub: large files and removal from history](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)
  explains why adding a later deletion commit is insufficient and recommends
  release assets for large distributable files.
- [GitHub: releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
  documents downloadable release assets separately from repository contents.

This investigation establishes the cause and a bounded repair plan. It does not
establish archive integrity, a final replacement size or completed history
cleanup. Those remain implementation checks. No existing evidence was altered.
