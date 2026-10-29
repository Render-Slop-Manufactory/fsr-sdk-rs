# API coherence and independent sys version checks

Date: 2026-10-18. Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`.
Host: macOS ARM64; Rust/Cargo 1.98.1, exact Rust 1.88.0 and Python 3.14.8.
SDK reference remains v2.3.0; no AMD runtime or GPU operation ran.

## Implemented contract

The maintainer authorized amendments to D022/D028/D029/D031/D032, without a new
Decision. Both context families now expose `request_tuning`, compulsory finite
parameter-domain validation and a single `InvalidTuningValue` error. Direct
enum construction and cross-parameter reuse are checked at the native-entry
boundary. Accepted bits, including negative zero, and null provider-default
requests remain unchanged. Invalid input makes no native call and does not poison;
existing poison still takes precedence. Native codes and terminal cleanup remain.
Success acknowledges the request, without certifying support or application.
No support probe, effective-value cache or checked/unchecked method pair was added.

Both families use `validate_dispatch_inputs` for optional metadata preflight and
`render_size_for_quality` for quality-derived dimensions. Runtime equivalents are
`upscaler_render_size_for_quality_dx12` and
`upscaler_render_size_for_quality_for_provider_dx12`. Examples, controlled/native
workloads and external consumer sources use the same names. No compatibility
aliases retain the unreleased names. Published dispatch/frame constructors and
ownership, recording, queue-completion and terminality contracts remain intact.

Sys `0.2.0` independently accounts for D031's published feature removal; wrapper
`0.3.0` depends on it. The lockfile changes only that workspace version. Naming,
tuning validation and error consolidation affect unreleased APIs, not published
wrapper `0.2.0`. The overall release still has the separately documented Vulkan
feature break and stronger unsafe acquisition contract. D030 remains proposed;
its verification gates a release shipping generation. Current complete-upscaling
scope still includes U01 and is not narrowed here.

## Executed checks

Commands ran from repository root with cached dependencies and no downloads:

| Check | Outcome |
|---|---|
| `cargo test --workspace --all-features --locked --offline` | 72 host tests/doctests pass, including actual tuning-boundary range/bit checks, cross-parameter reuse, no-native-entry errors, reset after invalid input, acknowledgement-only fixtures, both poison origins and loaded-owner cleanup. |
| `python3.14 tools/verification/run.py run ci --require-passed` | 108/108 pass; final attempt `20261018T205700Z-ci-9ad62838ce11`. Infrastructure: 241 tests, five explicitly skipped. |
| `python3.14 tools/verification/run.py run msrv --require-passed` | 97/97 pass; attempt `20261018T205540Z-msrv-e4268460e40f`. |
| `cargo check --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked --offline` and exact `+1.88.0` counterpart | Both pass, including Windows-gated fixtures and all renderer examples. No Windows linking/execution. |
| Windows-target Clippy with all targets/features and `-- -D warnings`; Windows-target Rustdoc with all features and `RUSTDOCFLAGS='-D warnings'` | Both pass. |
| Thirteen positive Windows consumer sources copied verbatim from `consumer-cases.json` into a disposable downstream package, with path dependencies on both crates | All compile for Windows on current Rust and 1.88.0. Ordinary unused-function warnings remain in compile-only consumers. |
| `release-artifacts --require-passed`, selecting the exact archive pair below with `--wrapper-archive` / `--sys-archive` | 203/203 pass; attempt `20261018T205712Z-release-artifacts-e87bd5361cb8`. Includes isolated registry, extracted-package tests/Clippy/docs, current/MSRV consumers, examples, licences and final unchanged-byte checks. |

All three successful coordinator runs report unchanged inputs. The initial CI
attempt `20261018T205540Z-ci-532582eac106` failed six loopback-listener controls
because the sandbox prohibited socket binding. The authorized outside-sandbox
rerun passes unchanged tests. Skips cover three PowerShell-parser and two
Linux-only process checks, not passed native/platform verification.

The initial archive attempt `20261018T205626Z-release-artifacts-2beffbc052e2`
created both archives, then Cargo's built-in workspace verification failed with
`no hash listed for fsr-sdk-sys v0.2.0` in its temporary registry. The existing
selected-archive flow independently verifies those same bytes; no verifier
assertion was weakened. An initial selected-path attempt named the wrong Cargo
output directory; the successful attempt uses `package/tmp-crate` from the
archive attempt. The disposable Windows consumer initially omitted its direct
raw dependency; adding that dependency fixes the consumer setup without changing
crate code or inventory sources.

| Checked development archive | SHA-256 |
|---|---|
| `fsr-sdk-sys-0.2.0.crate` | `a8a3a2c25cdc7c3ea599735aaab5e0d8223ba7262e72dad46286ff62d1ceb9bb` |
| `fsr-sdk-0.3.0.crate` | `d639f9857e3133fb2c19f2313e46a378c9e493cdc662c44ae2dcd4da899d48c8` |

## Remaining boundary

These are development archives, not final release or publication verification.
Windows fixture execution, independent MSVC assertions and actual-provider/GPU
confirmation remain unrun for these changes. Cross-compilation and host fixtures
cannot prove provider non-retention, application, state restoration or GPU safety.
Use the [Windows follow-up](../WINDOWS_FOLLOWUP.md) and final release profile after
the selected surface is complete. No D030 public API was implemented, no package
was published and no commit was created. This report and navigation links were
added after the successful runs; packaged source and manifests were unchanged.
