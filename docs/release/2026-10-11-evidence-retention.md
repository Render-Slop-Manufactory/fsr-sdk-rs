# Compact generated evidence retention

Date: 11 October 2026. The maintainer requested removal of unnecessary generated
data and cleanup of the published main history. No SDK/public API or native
execution contract changed.

## Problem and resulting behaviour

The previous committed tree contained 913,355,909 bytes. Of these,
902,842,304 bytes / 6,443 files belonged to one temporal calibration evidence
directory: three full exports, verbose analyses and repeated source snapshots.
The prior committed tree was 10,410,980 bytes. The original measurements and
duplicate-byte analysis are in the [size audit](../research/records/2026-10-11-src-repository-size-audit.md).

That evidence directory now contains approximately 0.39 MB / 14 files. The
candidate checkout is approximately 11 MB. It retains the historical protocol,
jitter, input identities, attempts/check outcomes, primary/pooled measurements,
package inspection and contact sheet. No separate data archive was created.
See the [compact evidence index](../research/records/2026-10-11-exp-v01-verification-calibration/README.md).

New calibration runs still independently validate every input and output and
compare all three repeats. After finalization they discard bulk frames and
verbose analyses by default, preserving failed outcomes too. An explicit
`--retain-diagnostics` keeps full local data for a concrete diagnosis. Exports
remain compact in either mode. Whole source snapshots are no longer generated.
Compact pooled diagnostics are computed before raw data disposal. Other native
profiles retain their existing bounded small-readback contracts.

CI rejects checkouts over 15,000,000 bytes, individual files over 1,000,000 bytes,
and raw payloads/source snapshots under `docs/`. The local check includes new
files that Git does not ignore. Build caches and SDK downloads are outside these
committed-file measurements.

## Validation and limits

- Before removal, `records.verify_artifacts` validated every file in the original
  evidence directory against its inventory.
- The new `compact_diagnostics` function recomputed the original pooled arm and
  paired-control measurements from the retained full analyses. Both dictionaries
  matched `diagnostic-summary.json` exactly. Numerical findings did not change.
- The bundled Python 3.12.14 / NumPy 2.3.5 interpreter ran
  `python -m unittest discover -s tools/verification/tests -v`: all 47 tests
  passed. Sandbox temporary-file restrictions required the permitted host rerun.
  Cases cover failed/successful retention, tamper rejection before deletion,
  immutable compact exports, inventory/run validity, pixel-weighted pooling,
  empty regions and total/file/generated-data size rejection.
- `run.py plan experiment-temporal-calibration` retained 20 ordered cases.
- A real CLI run without native opt-in finished as failed/blocked, created its
  retention disclosure and retained all 20 outcomes. `verify-run` and export were
  checked on that finalized failed attempt. No GPU execution was attempted.
- Compact evidence identities, changed navigation and the repository budget
  passed their checks. The final diff had no whitespace errors.

Full offline image reanalysis of the original attempts is no longer available
from the repository. Their hashes are identities, not substitutes for raw GPU
measurements. New questions require another run of the protocol and may yield
different output with another driver, provider or GPU. Historical negative and
inconclusive results remain unchanged. No new GPU correctness/support claim is
made by this storage change.

Only the latest offending main commit is replaced; its parent and earlier
history remain intact. The remote main identity is checked with an explicit push
lease, and a fresh remote clone is used to verify the resulting reachable tree.
The repository's private visibility is preserved.
