# Reactive-mask generation: local implementation checks

Date: 2026-10-19. Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`.
Host: macOS ARM64; Rust 1.98.1, exact Rust 1.88.0 and Python 3.14.
No Windows executable, AMD runtime or GPU workload ran.

## Selection, review and implementation

The maintainer selected U01 review and local implementation without Windows.
[D030](../adr/D030.md) is accepted with its existing separate unsafe operation,
explicit region and renderer-owned synchronization contract. Review confirmed
that generation can reuse the existing native owner and eligibility policy.
It needs operation-specific alias validation (two readers may alias) and a
shader-readable as well as UAV-capable output. No new context owner, callback,
allocation policy, capability inference or numerical clamp was introduced.

Both `Upscaler` and `MovableUpscaler` expose `generate_reactive_mask` and optional
`validate_generate_reactive_mask`. Portable `ReactiveMaskOptions` rejects each
non-finite scalar independently, defaults all four flags to false and preserves
finite bits, including negative zero and values outside `[0, 1]`. All 16 flag
combinations are forwarded, including both tonemap flags together. The borrowed
request is `Debug + Clone + Send + Sync`, without `Copy` or independent COM
retention. Common options retain their host/backend independence.

Inspection checks explicit top-left active regions against maximum allocations,
DIRECT list/device membership, canonical output identity, texture shape/format,
shader readability and UAV capability. Both colours are RGBA16F; output is
R8_UNORM, all allocated at maximum render size. Smaller active regions do not
require the main upscaler's dynamic-resolution flag. Provider resource masks do
not gate generation. Optional preflight makes no FidelityFX calls, preserves
eligibility and COM ownership, and recording repeats inspection.

A native generator error, including unsupported code 7 or unknown codes,
preserves `Operation::Dispatch` and makes generation, main dispatch, all live
queries and Configure terminal. Either prior poison origin rejects before
inspection. Preflight errors leave a usable context usable. Existing single
terminal destruction and failed-destroy dependency retention are reused.

Both unsafe acquisitions and recording contracts now cover generator compatibility,
synchronous descriptor non-retention, supported signal/scalar domains and trusted
success-state restoration. Both GPU routes must finish with no executable pending
recordings before safe Configure. These are caller-established native trust
requirements, not observed AMD guarantees. The movable field audit adds no owned
pointer or callback. The [renderer example](../../crates/fsr-sdk/examples/dx12_reactive_mask.rs)
records generation, UAV ordering, the shader-read transition and supplied-mask
upscaling; submission, completion and exceptional cleanup remain renderer-owned.

## ABI inputs and prepared assertions

Declarations were compared with cached SDK v2.3.0
`Kits/FidelityFX/upscalers/include/ffx_upscale.h`, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Its SHA-256 was rechecked as
`65be36d5653eb71c10379e0204f7bb7cd6a8aeec96e6f8cffc5f59ffeb1f89bf`.
The descriptor begins at physical line 153; the four flags are at lines 70–73.
No header or runtime was downloaded or added to packages.

The raw binding adds tag `0x10006`, all four flag bits and
`ffxDispatchDescUpscaleGenerateReactiveMask`. Paired Rust/C++ checks assert exact
field types and 64-bit layout: size 192, alignment 8; header/list/two colours/
output/region/scale/cutoff/binary/flags offsets
`0/16/24/72/120/168/176/180/184/188`. Host Rust execution checks its own layout;
independent MSVC assertion execution remains unrun.

## Executed checks

Commands ran from repository root using cached dependencies and no downloads.

| Command/check | Outcome |
|---|---|
| `cargo test --workspace --all-features --locked --offline` | 79 host tests/doctests pass after correcting the explicit common-options module path used by the standalone ownership compiler harness. Covers all 16 mappings, scalar bits, active-region/allocation distinction, loaded-owner forwarding, exact unsupported error, outer moves, explicit/Drop cleanup and failed-destroy retention. |
| `python3.14 tools/verification/run.py run ci --require-passed` | 108/108 pass, attempt `20261019T011754Z-ci-73aeb1dfc556`. Includes all four feature configurations, Clippy/docs, consumers, dependency isolation and examples. Infrastructure ran 241 tests with five explicit skips: three unavailable PowerShell-parser checks and two Linux-only process checks. |
| `python3.14 tools/verification/run.py run msrv --require-passed` | 97/97 pass, attempt `20261019T011721Z-msrv-03d753c9698b`, exact Rust 1.88.0 in all four configurations and downstream consumers. |
| `cargo check --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --locked --offline`, also with `cargo +1.88.0` | Both pass, including both public-family generator fixtures, public aliases/traits and the complete renderer example. No Windows linking or execution. |
| Windows-target Clippy with all targets/features and `-- -D warnings`; Windows-target Rustdoc with all features, no dependencies and `RUSTDOCFLAGS='-D warnings'` | Both pass. |
| `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` and sys counterpart; `cargo package --workspace --no-verify --allow-dirty --locked --offline` | New modules, tests and example are included. Opened archives contain byte-identical root MPL and retained AMD licence copies. Archive creation alone is not verification. |
| `python3.14 tools/verification/run.py run release-artifacts --require-passed --wrapper-archive target/package/fsr-sdk-0.3.0.crate --sys-archive target/package/fsr-sdk-sys-0.2.0.crate` | 203/203 pass, attempt `20261019T011839Z-release-artifacts-1b1df6dcde3d`: isolated local registry, extracted source/current/MSRV tests, consumers/examples, licences and unchanged-byte audit. |

Each successful coordinator run reports unchanged inputs. CI and MSRV evidence
integrity verification also passes. The initial sandboxed CI attempt
`20261019T011712Z-ci-6017447ec787` failed six existing watchdog controls because
local socket binding returned `PermissionError`. The authorized rerun outside
the sandbox passed unchanged tests. No assertion was weakened.

The first host test run exposed an ownership-harness module-resolution failure:
including `common.rs` through an absolute path selected the unrelated descriptor
mapping module for its new child. An explicit `common/reactive_mask.rs` path
fixes normal, standalone and extracted-package builds; the positive ownership
control and intended compile failures all pass. The Windows example's manually
dropped barrier union fields also needed explicit dereferences, then compiled
and passed Clippy. These were local implementation corrections, not AMD failures.

| Verified development archive | SHA-256 |
|---|---|
| `fsr-sdk-0.3.0.crate` | `0da751b820ac81dfcae3019b7b234f218e8ee6e4b43f8061eac9061ae8ffc5b6` |
| `fsr-sdk-sys-0.2.0.crate` | `9f58ee3aee765621b1ab415cb8658b6f4777519f4f970a92460db63056896a20` |

## Deferred verification and limits

Both-family Windows controlled fixtures compile but have not executed. They cover
resource/device/identity/inspection failures, read-only aliases, every flag and
active region, zero-call preflight, reinspection, unchanged COM ownership,
composition with ordinary supplied-mask dispatch, all terminal routes and failed
destruction retention. Compilation is not proof that those assertions pass.

Independent MSVC ABI, actual-provider support, safe numerical domains, mask writes,
GPU execution/state restoration, generated-to-consumer execution and both-family
native cleanup remain unrun. The [raw spike](../WINDOWS_FOLLOWUP.md#2-isolated-d030-raw-spike)
retains its independent descriptor and does not verify the new public path.
Successful recording would still not prove provider consumption of the mask.
Follow-up must exercise the public generator on both families and final release
archives. U01 remains implemented with native confirmation deferred, not Verified.

These are development archives, not final 0.3.0 release acceptance. No commit,
push or publication occurred. This report and final documentation-only status/
formatting corrections were added after the recorded checks; crate inputs and
archive bytes were unchanged.
