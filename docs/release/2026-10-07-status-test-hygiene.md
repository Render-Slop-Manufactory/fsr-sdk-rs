# Status and test hygiene verification

- Date: 2026-10-07, Europe/Berlin.
- Scope: roadmap step 1, on the current post-0.1.0 source workspace.
- Target/toolchains: Windows x64/MSVC, Rust 1.99.0 and minimum Rust 1.88.0.
- No SDK pin, dependencies, public API contract or production dispatch behavior
  changed. D013 and D014 remain proposed; this maintenance accepts neither design.

## Changes

- [Roadmap](../../ROADMAP.md), [decision index](../DECISIONS.md) and
  [D013](../adr/D013.md) now describe the completed
  [native transfer spike](../research/records/2026-09-30-exp-d013-native-thread-transfer.md):
  all three fenced transfer scenarios and controls passed, including siblings
  and destination-thread final loader release. Provider termination and the
  public conditional-contract/integration review remain open.
- [D014](../adr/D014.md) and the roadmap distinguish the unimplemented public
  API from its executed test-only sharpening protocol. Execution/repeat controls
  passed; the original effect gate failed. API-form review and effect-policy
  review remain separate. No scene, metric, threshold or result was changed.
- The [protocol snapshot](../research/records/2026-09-30-exp-d014-native-sharpening/protocol-before-run.md)
  has its broken relative link repaired with a dated clerical-correction note.
  Package README and Rustdoc now link the later moving-frame/controlled-semantic
  evidence without implying general image quality or published-archive revalidation.
- The roadmap preserves the agreed small M7–M9/0.2.0 sequence, followed by a
  realistic in-repo upscaler example/test harness. D013 stays independent; new
  effects follow broader evidence for the existing upscaler.
- [Context fixtures](../../crates/fsr-sdk/src/context/tests.rs) and
  [upscaler fixtures](../../crates/fsr-sdk/src/upscaler/tests/support.rs) share an
  exclusive [test-directory helper](../../crates/fsr-sdk/tests/fixtures/directory.rs).
  [Raw loader fixtures](../../crates/fsr-sdk-sys/tests/loader.rs) also retry only
  `AlreadyExists`. Stale paths are neither reused nor removed; other errors are
  returned. Two regression tests precreate conflicting paths and check preserved
  contents plus a non-collision I/O failure. No dependency was added.

## Commands and observations

Run from the repository root with cached dependencies:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --locked --offline
$env:CARGO_INCREMENTAL = '0'
cargo +1.88.0 test --workspace --locked --offline
cargo +1.88.0 check --workspace --all-targets --no-default-features --locked --offline
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --no-deps --locked --offline
cargo package --workspace --allow-dirty --locked --offline --no-verify
cargo package -p fsr-sdk --list --allow-dirty --locked --offline
tar -tf target/package/fsr-sdk-0.1.0.crate
tar -tf target/package/fsr-sdk-sys-0.1.0.crate
git diff --check
```

- All listed Cargo commands completed successfully. Stable and minimum-version
  default-feature runs each passed 34 wrapper unit/fixture tests, one public
  consumer test, 13 sys tests and four doctests: **52 passed, 19 ignored**.
- The stable feature-disabled run passed 19 wrapper tests, eight sys tests and
  two guarded doctests: **29 passed**. This checks the host fixture adapter,
  not DX12 runtime support without the feature.
- Tests used the normal temporary directory outside the desktop sandbox, with
  no `TEMP`/`TMP` override. The sandbox's earlier compiler-metadata access denial
  is separate from stale-path collisions. Both new collision regressions passed;
  existing lifecycle, threading fixtures, inspection and poison tests passed.
- Formatting and Clippy completed; Rustdoc denied warnings. Clippy emitted
  filesystem-related incremental hard-link fallback warnings, not failed code
  lints. Minimum-version tests/checks disabled incremental compilation.
- Both local archives were created and inspected: 45 wrapper files and 28 sys
  files. The wrapper archive contains `tests/fixtures/directory.rs` alongside
  both callers. Both archives retain LICENSE, LICENSE-AMD and README; no SDK
  checkout, DLLs or PowerShell runners are included. `--no-verify` checks archive
  creation/content only; no packaged compilation or release readiness is claimed.
- Repository-relative Markdown file targets and anchors were checked after
  editing; the protocol link resolves. The final diff passed whitespace checks.

## Limits

No AMD runtime/GPU test, new native experiment or hosted CI job was run for this
test-only/documentation maintenance. Existing dated GPU evidence remains evidence
for its recorded inputs. Historical records, failed sharpening metrics and archive
fingerprints were preserved. Local archive inspection does not compare against
published crates or validate a future 0.2.0 release. No commit, push or publication
was performed.
