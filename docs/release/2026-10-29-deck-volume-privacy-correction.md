# Historical Deck volume privacy correction — 2026-10-29

The release-content reviewer identified eight occurrences of a local external
volume mount root in four October 13 Steam Deck compact evidence files. The
finding is confirmed: each occurrence is the target of a recorded Wine `D:`
drive mapping. The volume UUID is unnecessary for interpreting the experiment.
The affected mount root is now represented as `<external-volume>`.

| Sanitized derivative | Replacements |
|---|---:|
| [jitter-results.json](../research/records/2026-10-13-exp-proton-temporal-regression/jitter-results.json) | 2 |
| [long-results.json](../research/records/2026-10-13-exp-proton-temporal-regression/long-results.json) | 2 |
| [quality-results.json](../research/records/2026-10-13-exp-proton-temporal-regression/quality-results.json) | 2 |
| [reset-results.json](../research/records/2026-10-13-exp-proton-temporal-regression/reset-results.json) | 2 |

The [byte-identity receipt](2026-10-29-deck-volume-privacy-correction.json)
records original and sanitized sizes/SHA-256, the eight changed JSON field paths,
replacement counts and CRLF-to-LF normalization. Before writing, every parsed
JSON value was recursively compared: only the declared mount-root substitutions
change string values; structure, numbers, outcomes and all other values match.
Historical source/native hashes and inventory digests remain their original
values and are not relabelled as hashes of the sanitized derivatives.

Original bytes are retained under ignored
`target/privacy-correction-deck-volume-20261029/`, with backup hashes verified
against the receipt. The distributed derivatives use UTF-8/LF. There is no Git
history rewrite, rerun or change to the original experimental verdicts.

The [content review guide](../release-content-review.md) now includes a bounded
external-volume UUID mount-path search. Repeating it across the current
non-ignored repository text finds no matches after correction. This is a focused
data-minimization correction, not a complete privacy/security or Git-history
audit. Generic system/toolchain paths and relevant GPU, driver, provider and
package identities are retained.

The shared rc1 archive hashes remain unchanged; these historical research files
are outside both crate packages. Exact-pair host qualification and accepted P08
reconciliation remain valid. Publication simulation, final candidate archive
cleanup, the controlled top-level README pass and publication remain separate.
