# Checked pre-exposure: implementation verification

**Follow-up, 2026-10-15:** the deferred Windows fixture/MSRV/ABI and native GPU
checks passed in the [combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The original observations and pending status below describe the October 14 run.

- Date: 2026-10-14, Europe/Berlin.
- Source: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0.
- Contract: [D020](../adr/D020.md), authorised by the maintainer's M11
  implementation request. No raw ABI, dependency or creation-flag changes.
- Result: **implementation and host/Windows compilation checks pass;
  Windows controlled-fixture execution and AMD/GPU confirmation remain pending.**

## Implemented behaviour

Portable `PreExposure` rejects positive/negative zero, negative values, NaN and
infinities with `DispatchScalar::PreExposure`. Positive finite values retain their
bits, including the smallest positive subnormal and `f32::MAX`. Default is `1.0`.
The private frame field has a setter and copying getter; both upscaler families
share its exact native mapping. Content matching and provider-supported numerical
inputs remain caller obligations at unsafe dispatch, separate from scalar checks.

The [portable tests](../../crates/fsr-sdk/tests/dimensions.rs) verify these domains
and common/reexport paths. [Controlled Windows tests](../../crates/fsr-sdk/src/upscaler/dx12/tests/pre_exposure.rs)
capture default and selected native bits for ordinary/movable public dispatch,
check replacement, unchanged reset/sharpening and unchanged COM reference counts.
Those tests compile here but have not executed on Windows.

The existing [single-frame harness](../../crates/fsr-sdk/tests/native/single_frame.rs)
adds a named native case with three fresh contexts and matching RGB factors
`0.25`, `1.0`, `4.0`. Its helper scales RGB only, before submission; the exposure
texture remains `1.0`. Recording, submission, fencing, finite/written output,
device/debug status and explicit cleanup must pass. The control-only environment
cannot satisfy this case. It adds no quality comparison and changes no default
single-frame acceptance criteria. Neither the modified C++ helper nor this native
case has been built/executed on Windows here.

## Checks performed

Host: macOS `aarch64-apple-darwin`, Rust/Cargo 1.98.1, bundled Python 3.12.14;
installed Windows Rust
target `x86_64-pc-windows-msvc`. Cargo commands used the existing cache, offline.
From the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo check --workspace --all-targets --no-default-features --locked --offline
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
python3 tools/verification/run.py run consumers
python3 -m unittest discover -s tools/verification/tests -v
cargo package -p fsr-sdk --list --locked --offline --allow-dirty
cargo package --workspace --locked --offline --allow-dirty --no-verify
```

All listed checks passed. Python used the existing bundled interpreter, including
NumPy, outside the sandbox for localhost watchdog fixtures. The infrastructure
suite ran 153 tests: 148 passed, three PowerShell-parser checks skipped because
`pwsh` is absent, and two Linux ownership/pidfd checks skipped on macOS.
All 76 consumer cases pass, covering portable positive/private-construction boundaries,
feature gates, dependency trees and host integration examples. A separate isolated
consumer cross-check compiled the exact `dispatch-inputs-positive` source from
[consumer-cases.json](../../tools/verification/consumer-cases.json) against the
Windows target with `movable`, covering the new public getter/setter.
Draft archive inspection confirms the new scalar/tests/helper bytes and both
licence texts are included. `--no-verify` packaging is not extracted-package
build or final release verification.

## Remaining confirmation

Rust 1.88.0 is not installed locally; no MSRV execution is claimed. Windows
execution of the controlled fixtures, C++ helper compilation, ABI checks and
native GPU execution need the prepared Windows/MSVC/runtime/GPU environment:

```sh
python tools/verification/run.py run ci
python tools/verification/run.py run msrv
python tools/verification/run.py run native-upscaler --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Choose the actual adapter index and trusted SDK root. `native-upscaler` includes
the `pre-exposure` case; it does not download AMD binaries. M11 remains Active
until those applicable execution checks pass. This record verifies development
source, not final packages or publication; quality validation remains on hold.
