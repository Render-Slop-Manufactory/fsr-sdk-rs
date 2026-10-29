# Windows CI path-assertion correction — 2026-10-29

The two recent Windows CI failures have the same cause: P08 orchestration tests
compared canonical paths returned by the implementation with unresolved
temporary-directory spellings. The implementation's normalization is correct;
the fixtures' expectations were wrong. Real Windows short-path aliases reproduce
all six failing assertions. This changes tests, failure reporting and contributor
routing; library code, benchmark producers and recorded measurements are unchanged.

## CI evidence and recurrence

| CI run | Windows stable infrastructure outcome |
|---|---|
| 2026-10-28, run `40609590180` | 296 tests, six failures, 35 skips |
| 2026-10-29, run `40800881517` | 303 tests, the same six failures, 35 skips |

The retained `infrastructure.tests/00-command.stderr.txt` files identify:

- `Reconfirmation.test_fixed_batch_skips_pilot_and_selection`: the forwarded
  `--performance-plan` is canonical, but the expected `plan_path` was not.
- `UnattendedBatch.exercise`: the forwarded batch-output parent is canonical,
  but the expected `root / 'batches'` was not. This breaks the dispatch/sdk
  failure subtests, fixed reconfirmation, serial-batch completion and the
  speed-outcome/release-approval check: five failed assertions in total.

The latest run's Linux stable and both MSRV jobs passed. The MSRV profile does
not select the infrastructure suite, so its green Windows result does not
contradict the stable failure. The exported errors can display identical
`<user-home>` paths because sanitization removes differing home-directory
spellings; that display does not establish lexical equality of the inputs.

This assumption has appeared before. The October 13 Proton optional-upgrade
fixture was corrected to use `Path(directory).resolve()` when comparing against
canonical external inputs. On October 14, the Proton runtime fixture root was
similarly corrected to match canonical prepare/verify paths, including Windows
aliases and Unix temporary-root symlinks. These earlier fixes addressed individual
fixtures; there was no focused authoring rule or forced alias condition for the
new P08 tests. The two recent red runs are one unfixed defect across two revisions,
not evidence of random test flakiness or a defect in every prior CI failure.

## Reproduction and correction

On this Windows host with Python 3.14.1, the original complete 311-test suite
passed with four platform skips under the ordinary temporary directory. To
reproduce the CI condition, a long-named temporary directory was converted with
Windows `GetShortPathNameW`. The probe required that the short spelling differed
lexically while both paths resolved to the same directory, set `tempfile.tempdir`
to the short spelling, then ran the two affected modules with the unittest loader.
It restored the original temporary directory afterwards. All **six** CI assertion
failures reproduced in **14** tests, with no errors.

The corrected tests keep non-canonical input paths by creating a real
`path spelling` child directory and passing its parent through `path spelling/..`.
They compare forwarded destinations with the exact expected `.resolve()` result.
The fixed-plan mock also expects its canonical path. This exercises normalization
on every host without needing Windows short names or symlink privileges, while
still rejecting a wrong destination. Production path handling is unchanged.

The previous package-coverage validation ran a 68-test subset that omitted these
two modules. Even a full run under the ordinary local temporary path passed;
both validation selection and fixture/environment coverage needed attention.

## Focused routing and diagnostics

The [test-authoring guide](../../tools/verification/tests/README.md) now owns the
Python fixture rules. [Agent routing](../../.agents/INTRO.md#task-routing) selects
it only when changing Python verification tests or their producers;
[Contributing](../../CONTRIBUTING.md#verification-infrastructure) and the parent
verification guide link to it. It distinguishes canonical output, filesystem
identity and exact-spelling/security contracts, and requires the complete
infrastructure suite for verification-code changes. It does not prescribe
normalizing all path inputs.

On infrastructure-test failure, the coordinator now prints the sanitized last
16,000 stderr characters into the CI job log, in addition to retaining full
case stdout/stderr. A controlled failing-child test checks truncation,
sanitization, retained full output, preserved failure status and evidence integrity.

## Validation

Both complete runs passed on Windows/Python 3.14.1:

| Environment | Outcome |
|---|---|
| Ordinary Windows temporary paths | 312 tests: 308 passed, 4 skipped; no failures/errors |
| Actual Windows 8.3 temporary-root alias | 312 tests: 308 passed, 4 skipped; no failures/errors |

The four skips cover Linux prefix ownership, the Linux pidfd descendant oracle,
Linux mode/symlink identity, and the SDK symlink control that avoids requiring
Windows symlink privileges. The
failure-diagnostic test passed in both runs. The focused 46-test regression/
infrastructure run also passed before the full runs. The Windows short-path
run changed only the temporary-directory spelling; all GPU/native launches in
these orchestration tests remain mocked.

The ordinary CI command, from the repository root, is:

```sh
python -m unittest discover -s tools/verification/tests -v
```

This machine has Python 3.14.1; GitHub CI uses Python 3.11.9. A Python 3.11 run,
Linux execution and a remote CI rerun were not performed here. The fix still
needs its next CI confirmation after an authorized push. No push, publication,
GPU run or new performance measurement was performed for this correction.
