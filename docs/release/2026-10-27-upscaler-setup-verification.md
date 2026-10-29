# Upscaler setup implementation verification

- Date: 2026-10-27 (Europe/Berlin; run timestamps use UTC).
- Scope: approved U07 under [D042](../adr/D042.md), wrapper `0.3.0` /
  sys `0.1.2`, SDK v2.3.0, Windows x64/MSVC + DX12, Rust 1.88+.
- Result: **U07 implemented and verified within the recorded configuration**.
  Final confirmation: 469/469 cases; callback supplement: 15/15; both input
  inventories unchanged and independently integrity-checked. Final release
  archives and U08–U12 remain separate.

## Implemented boundary and migration

Unsafe `Runtime::load` / `MovableRuntime::load` acquire loader/ABI/init/failure/
release trust and common coordination. Unsafe `upscaler_dx12` acquires lasting
upscaler/provider trust and returns concrete `dx12::UpscalerSetup` /
`dx12::MovableUpscalerSetup`. Their independent runtime/device borrows establish
no global lock or GPU idleness. Movable loader cleanup compatibility is required
before the loaded runtime can enter safe Send code; effect acquisition adds
arbitrary-destination operation/COM/error/cleanup duties.

Setup owns one `query::ProviderSelection`, initially Default. Consuming
`with_provider` changes future requests without native work. Safe provider,
quality/size/jitter/resource queries, memory estimation and `create(&options)`
share that request. Preserve numerical rejection before fresh explicit-ID
membership, no fallback/cache/reservation, live identity checks and native errors.
Safe rebinding requires upfront trust for every safely reachable provider/domain.

`UpscalerOptions` retains its name and portable creation settings, without a
provider field/getter/setter. Static public context `new` / `new_with_callbacks`
and old flat Runtime effect methods are removed. Callback creation and global
diagnostics remain unsafe setup methods; live recording/diagnostics retain their
own unsafe duties. Live owners retain runtime/device/pinned storage independently
through the existing ordinary-error/one-attempt/exceptional-retention policies.
Private native routing requires explicit unsafe effect trust too.

The crate Rustdoc, development package README, current examples/native harness,
consumer inventory, decisions, setup contract and roadmap are migrated. Root
README still documents published 0.2.0. Historical records, inventories and the
frozen setup/composition experiment are unchanged. U08–U12 remain open.

## Equipment and source identity

Rust/Cargo 1.99.0, exact MSRV 1.88.0, Python 3.14.1, Windows 11
10.0.26300, MSVC 14.44.35207 and Windows SDK 10.0.22621.0. Adapter 0 is
AMD Radeon RX 9060 XT (`1002:7590`), driver `32.0.32015.7029`; ordinary
DX12 validation, GBV disabled. SDK headers are pinned at
`60f4ea81909200d8542eca14dccb2628b763a9a3`.

The combined source/SDK inventory identity shared by the clean confirmation and
callback run is `4c32c7f1e41ad2462f50d3828121bfb0fcbe0524827cea61d6cda10a9b786653`.
The repository-only identity is
`cc20e86c749877b4f7e25dc3e1b8ef8846ce4570929ba74d27db7d46a925e2c5`.
Both recorded repository inventories match the final files exactly. The combined
inventory additionally hashes the four SDK headers and two staged DLLs. The
coordinator hashes manifests/lockfile, Rust/examples/fixtures, verification
tools and CI inputs; `records.source_identity` defines that inventory. The main
new implementation is `crates/fsr-sdk/src/upscaler/dx12/setup.rs`; runtime,
private routing, options and concrete context constructors are migrated in place.
No raw declaration, SDK pin or dependency changes.

Both staged DLLs have valid AMD Authenticode signatures. SHA-256:

| Staged file | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The runtime reports default provider name `4.1.2 *` in the query/regression
children. Callback tests explicitly enumerate/select `3.1.5` and `4.1.1`.
Display names are observations, not an audit of every internally loaded module
or proof that all execution comes from the staged provider file. No wrapper
download/discovery policy is introduced.

## Checks and reproduction

Run from the repository root with the installed toolchains and existing SDK:

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
cargo clippy --workspace --all-targets --features movable,utils --offline --locked -- -D warnings
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/upscaler-setup-verification
python tools/verification/run.py run native-callbacks --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/upscaler-setup-verification
python tools/verification/repository_size.py
```

`windows-upscaler-followup` includes current Rust check/test/strict Clippy/docs,
seven feature configurations, exact Rust 1.88 all-target/test/consumer/example
coverage, external positive and diagnostic controls, dependency isolation,
reserved-Vulkan rejection, infrastructure/archive checks and independent C++ ABI
before existing native lifecycle/query/dispatch/movable/Configure regressions.
`native-callbacks` adds production raw aggregate exchange with MSVC/current/MSRV
and both-family public callback/diagnostic/fenced cleanup coverage. Profiles
remain the existing coordinator's inventories; no new verification framework.

The dedicated production regression reuses a setup for two contexts, rebinds
Default/Explicit, rejects stale membership before another create, then queries
and destroys both after public-runtime/original-device loss. Controlled DLL/COM
observations also retain the existing validation-order, mapping, live mismatch,
ordinary create-error release, OK/null, callback error, failed destroy, poison
and unwinding tests. These are controlled fixtures, not AMD/GPU observations.

34 added U07 external cases (four positive and 30 negative per compiler) cover
unsafe load/setup/installer boundaries, private choice,
exclusive runtime and independent device borrows, setup-independent owner types,
local/movable family identity, Send/Sync, feature gates, portable non-exhaustive
selection and removed old entry points. Positive controls must compile; negative cases must produce their specific
diagnostic. Missing features or unrelated compiler failures cannot satisfy a
rejection.

## Recorded results

| Check | Outcome | Evidence boundary |
|---|---|---|
| Strict all-target Clippy; strict Rustdoc | Passed | Current compiler, both optional features; subsequent consolidated runs cover every selected configuration |
| Final `windows-upscaler-followup` | 469/469 passed; inputs unchanged | Seven current/MSRV configurations, strict docs/Clippy, consumers/examples, independent ABI and native regressions |
| `native-callbacks` | 15/15 passed; inputs unchanged | Independent C++ ABI and production raw exchange on current/MSRV, 12 public contexts and 20 fenced frames |
| Both final-run integrity checks | Passed | `run.py verify-run`, complete required outcomes and artefact hashes; repository inputs independently matched to final files |
| Changed Markdown links / diff whitespace | Passed | Relative file targets exist; no whitespace errors |

Final coordinator attempts under `target/upscaler-setup-verification`:

- `20261026T233648Z-windows-upscaler-followup-9c19920b805e` — passed, 469 cases.
- `20261026T234144Z-native-callbacks-535c8b18d6a8` — passed, 15 cases.

The native regression includes both-family default/explicit provider planning,
lifecycle, recording, Configure, movable transfer/cleanup and three sharpening
integration repetitions. The independent audit accepts 144 readbacks and 78
comparisons without image-quality thresholds.

Callback observations include 24 paired host allocations/frees, 240 paired
resource allocations/frees, 368 CB allocations, 16 messages and 80 compatible
cross-registration resource frees. Host/resource/heap live counts end at zero;
CB backing and external callback targets deliberately remain process-lived.
SDK heap-callback invocation remains unobserved, separately from passing ABI and
controlled/equipment coverage. Modern global and legacy live diagnostics succeed;
legacy null-context calls preserve native code 2 for both selected providers and
both families. Success and balanced allocations establish no generic callback
quiescence or provider capability service.

These counts describe coordinator cases, allocations, contexts and frames as
labelled; they are not interchangeable measures of test coverage.

## Preparation failures and limits

The first sandboxed Cargo-fix attempt could not open its local lock-server
socket (OS error 10013). The first workspace test attempt passed 127 tests but
one nested compiler control could not write its temporary output (OS error 5).
Repeating with the required local access succeeded. Strict Clippy rejected an
explicit drop of the non-Drop setup in an example; the example now relies on
its last use to end the borrow. The initial consolidated attempt
`20261026T233021Z-windows-upscaler-followup-9db9926e6f04` completed all 463
individual cases successfully but overlapped final source review/fixture and
consumer corrections. Its input-change rejection is preserved as failed; it is
not accepted as a passed profile. The fresh 469-case confirmation above replaces
it for acceptance. The final repository-size check stays below the 50 MB budget.
Only generated local build caches were removed to reclaim verification space.

No universal AMD construction, callback quiescence, native affinity, termination,
provider numerical/access safety or GPU/host-memory consequence is established.
A fixture observer retains its own DLL reference, so context retention witnesses
are distinct from observed final OS unload. Native execution is bounded to the
recorded deployment/GPU/driver/workloads. No new image-quality/performance,
Proton, broad provider support or final release-package claim follows.
