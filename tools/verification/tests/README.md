# Authoring Python verification tests

Read this guide when changing tests in this directory or the Python verification
code they exercise. The parent [verification guide](../README.md#architecture-and-platform-boundaries)
owns execution, evidence and platform boundaries; this file owns test-fixture
and validation practices.

## Filesystem path assertions

Choose assertions from the operation's contract:

- If the operation returns or forwards a canonical path, compare it with
  `expected.resolve()`. A temporary directory's returned spelling need not be
  canonical: Windows 8.3 aliases and Unix symlinked temporary roots are examples.
- If the contract concerns an existing filesystem object's identity, use an
  identity comparison such as `samefile` where appropriate.
- If the contract concerns exact CLI spelling, archive-member syntax, traversal,
  symlinks or containment, preserve those spellings and test that contract.
  Normalizing them before validation can hide the defect being tested.

For a canonicalizing boundary, supply a deliberately non-canonical fixture path
and assert the exact expected canonical destination. This makes the regression
reproducible on ordinary development machines. For example:

```python
spelling = Path(temporary) / 'path spelling'
spelling.mkdir()
root = spelling / '..'
expected_output = (root / 'batches').resolve()
# Pass root / 'batches' to the operation, then compare its forwarded destination.
```

The intermediate directory must exist. This fixture works without Windows
symlink privileges or availability of short-path names. Keep the input
non-canonical so the test actually exercises normalization; resolving the root
before the call would remove that coverage. The P08 orchestration tests use this
pattern for both fixed-plan and batch-output forwarding.

## Validation after changing verification code

Run the affected tests while developing, then run the complete infrastructure
suite from the repository root before reporting the change as verified:

```sh
python -m unittest discover -s tools/verification/tests -v
```

This applies to Python verification producers as well as their tests: a producer
can be imported or exercised by another test module. Record the Python version,
host, total and skipped tests. CI uses Python 3.11 on Windows and Linux; use that
version when available and state any interpreter/platform coverage not executed.
A passing local subset or a GPU profile does not establish a full suite pass.

When fixing a CI-only failure, read the retained case output, reproduce the
triggering condition, and retain it in the relevant fixture or regression test.
The coordinator prints a bounded, sanitized infrastructure-test stderr tail on
failure; full stdout/stderr and case outcomes remain in the attempt's evidence.
Use those logs to distinguish assertion failures from unavailable tools or
unexecuted platform checks.
