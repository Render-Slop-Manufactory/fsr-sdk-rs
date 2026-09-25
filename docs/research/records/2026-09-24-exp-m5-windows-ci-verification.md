# Verification: M5 Windows CI fixtures

- Verification date: 2026-09-24.
- CI run date: 2026-09-23.
- Question: Did the ordinary Windows CI checks pass for the M5 public
  construction path?

## Evidence and result

The Windows DX12 (x64 MSVC) CI job used the `windows-2025` runner and completed
successfully. It ran the following checks from the repository root; each step
passed:

```text
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --no-default-features --locked
```

The test output reported 15 passing wrapper tests, six passing ABI tests, four
passing loader tests, and two passing compile-fail doctests. Eight tests
were ignored by the ordinary suite, including the two M5 native lifecycle
cases. No test failed. These observations describe the CI run on 2026-09-23;
they do not establish results for later code changes.

The CI job did not run the opt-in native lifecycle cases or submit GPU work.
Both native cleanup cases and their distinct machine and runtime inputs are
documented in the [M5 native verification](2026-09-24-exp-m5-windows-native-verification.md).
