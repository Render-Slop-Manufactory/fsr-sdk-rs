# Finite depth and motion units: implementation verification

**Follow-up, 2026-10-15:** the extended M12 implementation passes Windows
fixture/MSRV/C++ ABI and eight representative native profiles in the
[combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The narrower implementation and pending status below describe the October 14 run.

- Date: 2026-10-14, Europe/Berlin.
- Source: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0.
- Contract: [D021](../adr/D021.md); maintainer selected reversed finite depth and
  UV/pixel motion units as the first M12 slice. Other conventions remain deferred.
- Result: **implementation/host checks and Windows Rust compilation pass;
  Windows fixture/MSRV, C++ ABI/helper and native GPU execution remain pending.**

This records the first slice before the maintainer requested the remaining M12
controls. See the [full-profile follow-up](2026-10-14-depth-motion-profiles-verification.md)
for current implementation and verification status. Descriptions below refer to
the earlier implementation; current linked source has since been extended.

## Implemented contracts and tests

Portable `DepthConvention` defaults to `StandardFinite`; `ReversedFinite` selects
the pinned inverted-depth bit (`1 << 3`). A shared private conversion feeds both
context creation and pre-create memory estimation. The raw crate adds only that
existing constant; descriptor layouts and dependencies are unchanged.

Per-frame `MotionVectorUnits` defaults to UV, mapping to render width/height.
Pixels map to unit scale. Render-resolution RG16F, current-to-previous direction,
+X right/+Y down and jitter-free content remain required. Camera distances remain
finite positive `near < far`; applied pixel jitter is forwarded unchanged.
The wrapper neither rewrites textures nor reconfigures existing contexts.

- [Portable tests](../../crates/fsr-sdk/tests/dimensions.rs): defaults, cloned
  options, portable traits and unchanged finite-camera validation.
- [Host query test](../../crates/fsr-sdk/src/query/tests.rs): independent expected
  memory-estimate flags `0`/`8`, confirming the selected depth option.
- [Controlled Windows tests](../../crates/fsr-sdk/src/upscaler/dx12/tests/depth_motion.rs):
  creation flags for both ownership families, default/pixel/UV scale replacement,
  unchanged camera/jitter/reset/pre-exposure/sharpening and COM reference counts.
  Compiled here; not executed on Windows.
- Paired [Rust](../../crates/fsr-sdk-sys/tests/dispatch_abi.rs) /
  [C++](../../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp) constant assertions.
  The Rust constant check runs on the host; C++ compilation remains pending.
- The prepared `depth-motion` case reuses
  [three-frame equipment](../../crates/fsr-sdk/tests/native/temporal_smoke.rs).
  Four context profiles each dispatch three jittered moving frames. Pending uploads
  use standard/reversed depth `0.25`/`0.75` and matching motion `-1/320` UV or `-1`
  pixel after frame zero. Each profile checks memory queries, native dispatch,
  submission/fencing, finite/written output, device/debug status and explicit
  destruction. No image-quality comparison is added; the original smoke path and
  criteria remain intact. The helper/case have not run on Windows or a GPU here.

## Local checks

macOS `aarch64-apple-darwin`; Rust/Cargo 1.98.1, bundled Python 3.12.14 with
existing NumPy. Windows Rust target is installed; Rust 1.88.0 and PowerShell are
not. Cargo used existing caches offline. Commands from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
python3 tools/verification/run.py run consumers
python3 -m unittest discover -s tools/verification/tests -v
cargo package --workspace --locked --offline --allow-dirty --no-verify
cargo package -p fsr-sdk --list --locked --offline --allow-dirty
```

Final checks pass. Initial host Clippy found the private native-flag helper unused
in portable production builds; it is now gated to Windows/DX12 and test builds,
and host/no-default confirmations pass without warning suppression.
All 77 portable consumer cases pass. A separate isolated Windows-target consumer
compiled the exact `dispatch-inputs-positive` source in
[consumer-cases.json](../../tools/verification/consumer-cases.json) with `movable`.
The infrastructure suite ran outside the sandbox for localhost fixtures: 148 of
153 tests passed; three unavailable PowerShell-parser and two Linux-only checks
were skipped. Draft archive inspection checks changed source and licence contents;
`--no-verify` packaging is not extracted-package build or final release acceptance.

## Remaining execution

On the prepared Windows/MSVC/runtime/GPU environment:

```sh
python tools/verification/run.py run ci
python tools/verification/run.py run msrv
python tools/verification/run.py run native-upscaler --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Select the actual trusted SDK root and adapter. The native profile includes the
`depth-motion` case and never acquires AMD binaries implicitly. M12 remains Active
until the applicable Windows/native checks pass. Infinite far planes, motion
resolution/jitter conventions and arbitrary scales remain deferred. No additional
platform support, quality validation, final package verification or publication is
claimed.
