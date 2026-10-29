# Windows follow-up and raw reactive-mask spike preparation

- Date: 2026-10-18.
- Question: how much of the pending Windows verification and D030 spike can be
  prepared and checked on the available Mac?
- Scope: executable test equipment, evidence evaluation, a consolidated Windows
  profile and reproducible handoff. D030 acceptance, public generation APIs,
  native support and image-quality conclusions are excluded.
- Outcome: preparation and portable checks pass; actual Windows/MSVC helper
  compilation, fixture execution, provider loading and GPU execution remain unrun.

## Setup and tested source

Host: ARM64 macOS 27.0.1, build 26A434; Rust/Cargo 1.98.1, exact Rust 1.88.0,
Python 3.14.8 and Apple Clang 21.0.0. Both Rust toolchains have the Windows MSVC
target. No Windows runtime/device or native MSVC environment is available.

The baseline is revision `7b904ca96bc77e7321234d7e23a4e4c8aa9ef4a2`, with the
following local preparation changes. The revision is supplementary: the scope
and results below describe the actual tested changes without requiring history.

- `tests/native/configure.rs` and shared `followup.rs`: a native test of all five
  tuning parameters, checked value and provider-default reset, both owner
  families, dispatch/fence/readback and terminal cleanup. Twenty frames are
  required on Windows; none ran here. The case is included in source and
  extracted-package native regressions.
- Probe-only `reactive_mask_descriptor.rs`, `reactive_mask.rs`,
  `reactive_mask_abi.cpp`, `reactive_mask.h` and host layout integration test;
  additions to the shared native entry points/helper. Production sys bindings
  and public generator methods are unchanged.
- `tools/verification/reactive_mask.py`, native/profile inventory and negative
  controls: 18 independent native children, structured completion/readback
  evidence and source-before-GPU gating in `windows-upscaler-followup`.
- [Windows handoff](../../WINDOWS_FOLLOWUP.md), profile documentation and a D030
  preparation note; the proposal remains proposed.

SDK inputs are v2.3.0 at
`60f4ea81909200d8542eca14dccb2628b763a9a3`, API 4.1.1. The existing local
`target/sdk-acquisition-check/v2.3.0/Kits/FidelityFX` cache supplies the four
hash-verified headers from `tools/native-headers.json`. No SDK acquisition or
dependency installation was performed. The native runner still requires exact
loader/provider hashes from `tools/verification/native.py` and valid AMD
signatures before loading anything; these DLL checks did not execute here.

## Protocol and observable outcomes

The [bounded raw protocol](../../WINDOWS_FOLLOWUP.md#2-isolated-d030-raw-spike)
uses RGBA16F colours, a distinct R8_UNORM mask seeded to 227, scale `1`, cutoff
`0.2`, binary `0.9`, all 16 flag combinations and active regions
320×180 → 256×128 → 320×180 within fixed allocations. Equipment and
identical-colour controls distinguish uploads from actual generation. Successful
children must produce mask writes, preserve inactive pixels and complete a
following public upscale through submission/fence/readback with clean debug
diagnostics. The proposed UAV boundary and renderer barriers are exercised
explicitly. Forwarding the mask does not prove which texels the provider uses.

A genuine nonzero native generator return preserves its exact code as
`observed`, discards the unsubmitted list and attempts terminal destruction
once. Failed cleanup is `failed`. Crashes/timeouts, missing or duplicate
completion evidence, unchanged masks and bad GPU controls cannot become support
observations. Rejected children do not suppress other flag combinations. The
control must pass first; no generator call ran on this host.

## Executed commands and observations

Commands run from the repository root, offline unless stated otherwise.

```sh
cargo test -p fsr-sdk --test reactive_mask_probe --offline --locked
clang++ -std=c++17 -fdeclspec -Wno-ignored-attributes -Wall -Wextra -Werror -fsyntax-only -Itarget/sdk-acquisition-check/v2.3.0/Kits/FidelityFX/api/include -Itarget/sdk-acquisition-check/v2.3.0/Kits/FidelityFX/upscalers/include crates/fsr-sdk/tests/native/reactive_mask_abi.cpp
cargo +1.88.0 check --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --offline --locked
cargo clippy --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --offline --locked -- -D warnings
python3.14 tools/verification/run.py run ci --require-passed
python3.14 tools/verification/run.py run msrv --require-passed
cargo package --workspace --allow-dirty --offline --locked --no-verify --target-dir target/windows-followup-package
python3.14 tools/verification/run.py run experiment-reactive-mask --native --sdk-root target/sdk-acquisition-check/v2.3.0/Kits/FidelityFX --adapter-index 0
```

| Check | Actual result |
|---|---|
| Host Rust layout test | Exit 0, one passed; tag, four flags, 192-byte/8-aligned descriptor and offsets |
| Host C++ header assertions | Exit 0; exact field types, offsets, layout, tag and four bits against the pinned header. ARM64 host ABI only |
| Windows Rust compilation | Exact 1.88 all-target/all-feature check and current all-target/all-feature Clippy exit 0; no Windows linking/execution |
| Local CI | Exit 0, all 108 cases passed, inputs unchanged. Infrastructure ran 241 tests successfully with five intentional skips |
| Exact MSRV profile | Exit 0, all 97 cases passed, inputs unchanged |
| Local archives | Exit 0, sys 31 files and wrapper 112 files; both licences present, all new probe/helper files retained in wrapper archive. `--no-verify` is packaging only |
| Raw profile platform guard | Exit 1; 29 cases recorded, two passed (toolchain and header identity), 27 blocked by missing Windows/MSVC or dependencies. No native helper, DLL or GPU child ran |

Negative controls exercise genuine rejection versus successful GPU evidence,
cleanup failure, invalid metrics/regions, crashes/timeouts and missing completion.
A simulated consumer failure in the real composed follow-up graph blocks every
GPU child while independent MSRV checks still execute. This is orchestration
evidence, not a provider experiment.

Unsuccessful preparation attempts are preserved as limits:

- Initial host C++ syntax checking without `-fdeclspec` failed because the AMD
  header uses `__declspec(dllexport)`. Enabling that syntax and suppressing the
  host's ignored export-attribute warning allowed the independent assertions;
  this does not substitute for MSVC.
- Initial Windows Clippy exited 101 on two unnecessary `drop` calls on a borrowed
  non-Drop request. Removing the calls preserved borrow checking and the final
  strict Windows Clippy check passed.
- Initial sandboxed CI exited 1: 107 cases passed, infrastructure failed on six
  loopback-based watchdog tests (`PermissionError: Operation not permitted`).
  The authorized rerun with loopback available passed all 108 cases. Both attempts
  are retained locally; no test requirement was weakened.

The successful CI/MSRV source identity is
`9b257a7be35d295096b9f65308b5cb7c051ddffc9cfa4047d93b3bb16f2ad30c`.
Local attempts are `20261018T204121Z-ci-cae146162699`,
`20261018T204133Z-msrv-863d7052a38c` and
`20261018T204133Z-experiment-reactive-mask-01bcc7d03f1d` under ignored
`target/verification`. Integrity verification and exclusive export of the blocked
raw attempt succeeded. Verbose logs/package outputs remain ignored; no GPU
images were generated or retained. The table above is the portable result record.

## Baseline delta and remaining work

Windows now has a prepared combined run, actual-provider Configure workload and
isolated raw generator protocol, rather than only proposed manual checks. Local
compilation, evidence rejection controls and packaging pass. The full native
C++ helper still needs MSVC compilation; a host header assertion alone cannot
establish that it builds or executes on Windows.

Run the [handoff commands](../../WINDOWS_FOLLOWUP.md) on Windows next. Record
Configure acknowledgement separately from numerical application/default effects,
and native generator rejection separately from successful writes/execution.
D030 still needs review and any subsequent public both-family implementation,
fixtures, lifetime audit and final package verification. These preparations
close no native support, Windows fixture or release-acceptance checkpoint.
