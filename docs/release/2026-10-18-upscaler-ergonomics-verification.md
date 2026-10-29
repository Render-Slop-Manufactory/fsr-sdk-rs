# Upscaler early-validation and recovery checks

Date: 2026-10-18. Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, implementing
[D032](../adr/D032.md) and refining still-proposed [D030](../adr/D030.md).
SDK baseline remains v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`.

## Change and source state

The tested source includes accepted/implemented D028/D029 and D031's removed
Vulkan marker. This change adds no native ABI, dependency, platform or relaxed
safety guarantee. It adds portable optional tuning-domain checks and an owned
error category; both DX12 families' borrowed `validate_dispatch` share the same
inspection as compulsory dispatch. Existing test-only owned inspection calls
delegate to that borrowed inspection without changing public dispatch signatures.

| Reviewed problem | Implemented response | Remaining boundary |
|---|---|---|
| Native failure makes a context terminal | Earlier metadata diagnosis, explicit error-stage/recovery guide and refined D030 support-probe limits | No side-effect-free native error category or generator-support predicate established. |
| Successful recording later abandoned | Compilable renderer-owned invalidation/completion/destruction/recreation example, retaining the old owner until cleanup succeeds | Abandoned native state is not rolled back; renderer owns submission and GPU completion. |
| Out-of-domain tuning hidden by native clamping | `TuningValue::checked_for` checks documented per-parameter domains and preserves valid bits, including negative zero | Direct finite Configure passthrough remains; checking does not prove actual application or retain parameter association. |
| Configure requires GPU idle | Initial/batched-idle example validates all desired settings before native mutation, then applies them in one established idle interval | No atomic batch, implicit wait or in-flight Configure permission. |
| Repeated provider enumeration | Opt-in optimized native startup-cost profile and bounded protocol prepared | Actual Windows/provider timings are unavailable; no cache or session selected. |
| Resource errors discovered during dispatch | Both families expose optional setup preflight; dispatch independently rechecks changed metadata | GPU states/content/lifetimes and provider capability remain outside metadata inspection. |

Stale supplied-mask and provider-selection Rustdoc is corrected. D030's generator
and optional generator validator remain proposed/unimplemented. The
[setup/recovery guide](../DX12_RECOVERY.md) is the current renderer workflow.

## Environment and executed checks

Host: macOS ARM64; Rust/Cargo 1.98.1, exact Rust 1.88.0 and Python 3.14.8.
Both Rust toolchains have the Windows MSVC target installed. No Windows runtime,
AMD provider execution or GPU workload ran. Commands use cached dependencies:

| Command | Observation |
|---|---|
| `python3.14 tools/verification/run.py run ci --require-passed` | 108 cases pass: four feature configurations, host tests/Clippy/docs, portable consumers/examples and dependency graphs. Infrastructure suite: 234 tests, 229 passing and five skipped. |
| `python3.14 tools/verification/run.py run msrv --require-passed` | 97 cases pass on exact Rust 1.88.0, including four configurations and public consumers/examples. |
| `cargo test --workspace --all-features --offline --locked` | 72 host tests/doctests pass, including the public tuning-domain/bits test and existing finite-passthrough fixtures. |
| `cargo check --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --offline --locked` | Windows paths, controlled fixtures, native timing workload and renderer examples compile. |
| Same Windows check prefixed with `cargo +1.88.0` | Passes on exact MSRV. |
| `cargo clippy --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --offline --locked -- -D warnings` | Passes. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --target x86_64-pc-windows-msvc --offline --locked` | Windows public API documentation passes. |
| `cargo fmt --all -- --check` and `git diff --check` | Pass. |
| `python3.14 tools/verification/run.py plan experiment-provider-setup` | Enumerates Windows/ABI/signed-runtime prerequisites and the exact optimized startup-query child. This is a plan, not execution. |

The initial CI attempt retained six sandbox loopback-listener errors and one
new profile-test prerequisite mismatch. The expectation was corrected to include
the coordinator's common toolchain prerequisite; a complete permitted rerun
passes. Failed attempts and full local logs remain under ignored `target/`.

`cargo package --workspace --list --allow-dirty --offline --locked` and local
`cargo package --workspace --allow-dirty --offline --locked --no-verify --target-dir target/ergonomics-package`
produce inspected inventories and archives: 31 sys files and 105 wrapper files.
Project licences match the root, the sys AMD notice is retained, and the new
example/fixture sources match the checkout. No Vulkan marker reappears. Packaging
does not publish or replace final extracted-package/native release verification.

## Unexecuted evidence

The Windows preflight fixtures check parity across metadata failures, no FSR
calls, COM-reference balance, reinspection after input changes, unsupported code
terminality and both poison origins. They compile but have not executed here.
Neither public preflight runtime behaviour nor DX12 resource inspection is
claimed verified by host tests.

Provider setup timing has four warm-ups per arm and 32 alternating paired blocks
of six-query default/selected sequences, with 64 timing rows per process. It
measures the whole setup path and does not reserve default-provider routing.
No native samples, enumeration-cost conclusion or caching benefit is reported.
Native failure rollback, abandonment reuse and in-flight Configure require
separate provider-pinned evidence; D030 acceptance/implementation remain separate.
