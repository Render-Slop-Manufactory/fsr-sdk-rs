# Text line endings and retained byte identity — 2026-10-15

## Result and scope

New project text uses UTF-8/LF through the root `.gitattributes`. JSON record
writes and exported JSON/TXT/CSV produce UTF-8/LF bytes before inventorying.
Exports carry their own hashed `.gitattributes` to preserve every final byte,
including verbatim payloads that Git might otherwise identify as text.
The contribution and research rules document this boundary. No dependency,
verification profile, native contract or raw-data retention policy changed.

The [previous qualification](../research/evidence-retention.md#reproduction-limits)
identified 68 text files whose recorded sizes and hashes matched reconstructed
Windows CRLF bytes, rather than the LF checkout. Those exact CRLF bytes are now
restored, with scoped byte-preservation rules in
[`records/.gitattributes`](../research/records/.gitattributes). The original
inventories, measurements, outcomes and nine already matching files are unchanged.

| Evidence directory under `docs/research/records/` | Restored to CRLF | Already exact | Final exact |
|---|---:|---:|---:|
| `2026-10-11-exp-temporal-quality-metric-diagnosis` | 9 | 3 | 12 |
| `2026-10-11-exp-temporal-quality-validation` | 10 | 2 | 12 |
| `2026-10-11-exp-temporal-long-sequence` | 9 | 2 | 11 |
| `2026-10-11-exp-temporal-reset-recovery` | 19 | 2 | 21 |
| `2026-10-12-exp-temporal-jitter-phase` | 21 | 0 | 21 |
| **Total** | **68** | **9** | **77** |

All candidates were checked before any historical file was written. For each
non-matching file, reconstruction used
`raw.replace(b'\r\n', b'\n').replace(b'\n', b'\r\n')`; the result had to match both
the original inventory's byte count and SHA-256. No whitespace, JSON structure,
encoding or value was otherwise changed. Exact pre-restoration copies were kept
under ignored `target/`; the published original inventories remain the authority.

## Checks and reproduction

Executed on macOS with Python 3.14.8 and Git 2.54.0 (Apple Git-157), without downloads:

```sh
python3.14 -B -m unittest discover -s tools/verification/tests -p test_line_endings.py -v
python3.14 -B -m unittest discover -s tools/verification/tests -v
python3.14 -B tools/verification/archive/check.py
python3.14 -B tools/verification/repository_size.py
git diff --check
git diff --ignore-space-at-eol -- docs/research/records/
```

The four new [line-ending tests](../../tools/verification/tests/test_line_endings.py)
pass. They check exact UTF-8/LF JSON bytes, inclusion of the root attributes in
source identity, all 77 original historical sizes/hashes, and export/source
preservation through real Git index insertion and fresh file checkout in isolated
temporary repositories. Git runs with `core.autocrlf=true`, `false` and `input`,
each with `core.eol=crlf`. The tests cover normal LF source, existing archive
exceptions, all five historical exceptions, UTF-8 logs/CSV, binary and text-looking
verbatim payloads. Export hashes still match afterwards; a deliberately changed
file fails strict verification, and original attempt files remain unchanged.

The complete infrastructure suite ran 177 tests: 172 passed; two real PowerShell
parser tests and three Linux/Proton tests were skipped because their prerequisites
are unavailable here. The six process-tree tests need local sockets, so the full
suite ran with sandbox permission. Existing CI discovers the new tests on both
Ubuntu and Windows; no actual Windows execution is claimed by this local run.
Archive integrity and Python/JSON syntax pass; PowerShell parsing is explicitly
not run. Repository size and whitespace checks pass. Ignoring line-ending
whitespace leaves no change to the 68 historical payloads; all five original
inventory files remain byte-identical.

This is a storage/integrity correction, not a new GPU observation. It neither
recovers removed raw images nor verifies their past bytes or native behaviour.
