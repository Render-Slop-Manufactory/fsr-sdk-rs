# Historical evidence privacy correction — 2026-10-29

The external release-content review identified local workspace paths in the
October 14 Steam Machine compact record and unnecessary dependency-maintainer
email addresses in the October 10 verification output. Both checked-in JSON
files are now explicitly sanitized derivatives. This corrects retained text;
it neither reruns nor changes the original tests, measurements or verdicts.

The [byte-identity receipt](2026-10-29-evidence-privacy-correction.json) records
each file's original and sanitized size/SHA-256, replacement counts and line
ending conversion. Original inventories, source/native hashes and numerical
observations have not been regenerated or relabelled as sanitized-byte hashes.

| Retained file | Correction |
|---|---|
| [Steam Machine compact evidence](../research/records/2026-10-14-exp-steam-machine-proton-temporal-regression/results.json) | Replace 122 Windows/Wine workspace-root occurrences in shared values with two distinct `<workspace-N>` aliases. This includes the ten loaded-path occurrences highlighted by the review and related environment paths. Preserve drive notation, relative paths and the distinction between the two original roots |
| [October 10 check attempts](2026-10-10-m9-verification.md#observations-and-limits) | Replace 14 email occurrences in two Cargo-metadata output strings with `<redacted-email>`. Preserve other package metadata and check outcomes |

Both new derivatives use UTF-8/LF instead of the original CRLF. Parsed JSON
structure and every non-string value were compared before/after; changed string
fields are limited to the declared replacements. The original bytes remain in
ignored local diagnostics, outside the distributed repository. Their hashes
identify historical bytes and cannot reconstruct them.

This is a focused correction of the supplied findings, not a certification that
all repository content or Git history contains no personal information or
secrets. The repeatable [release-content review](../release-content-review.md)
also applies to final package contents before committing a Git-distributed
candidate. Git history and deliberate author identities require separate review;
this correction performs no history rewrite.

Retention note, 2026-10-29: the superseded October 10 operational verification
export, including `checks.json`, was subsequently retired during documentation
compaction. Its original/sanitized hashes remain in the correction receipt and
its release facts in the linked summary; the detailed bytes are in Git history.
The October 14 experimental record remains retained. No correction receipt or
experimental outcome was rewritten by that retirement.
