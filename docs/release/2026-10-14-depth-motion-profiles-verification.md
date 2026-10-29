# Depth and motion profiles: implementation verification

**Follow-up, 2026-10-15:** the deferred Windows fixture/MSRV/ABI and eight-profile
native GPU checks passed in the [combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The original observations and pending status below describe the October 14 run.

- Date: 2026-10-14, Europe/Berlin.
- Source: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0.
- Contract: [D021](../adr/D021.md). The maintainer requested the remaining M12
  controls after the [first slice](2026-10-14-depth-motion-verification.md).
- Result: **host tests, consumer checks and Windows Rust compilation pass;
  Windows fixture/MSRV execution, C++ ABI/helper compilation and native GPU
  execution remain pending.** M12 remains Active. Quality validation is on hold.

## Contracts and test coverage

[Portable configuration/camera types](../../crates/fsr-sdk/src/upscaler/common.rs)
provide four standard/reversed finite/infinite depth conventions, render/display
motion resolution and excluded/included projection jitter. Defaults preserve the
original standard finite, render-resolution, jitter-free profile. A shared flag
conversion feeds both creation and pre-create memory estimation. The raw crate
adds four existing SDK bits, with unchanged layouts, pin and dependencies.

`CameraParameters::new` keeps its original finite validation. `new_infinite`
checks near/FOV/metres and supplies `f32::MAX` as a native finite far placeholder,
with a separate private infinite marker. Dispatch rejects a camera-context
finite/infinite mismatch before COM inspection/native entry. Finite distances stay
ordered even for reversed depth. Motion metadata must match the selected render
or output dimensions. UV scaling uses that texture's dimensions; pixels use unit
scale. Applied jitter is forwarded unchanged. The wrapper does not rewrite pixels.

- [Portable tests](../../crates/fsr-sdk/tests/dimensions.rs): defaults/clone
  independence, traits, four depth modes, infinite camera scalar rejection and
  distinction from a finite camera with `far == f32::MAX`.
- [Host query fixtures](../../crates/fsr-sdk/src/query/tests.rs): all 16 context
  combinations, with independent expected depth flags `0/8/16/24`, display bit
  `2` and included-jitter bit `4`.
- [Windows controlled fixtures](../../crates/fsr-sdk/src/upscaler/dx12/tests/depth_motion.rs):
  all 16 context combinations, default/pixel/UV replacement for both ownership
  families, creation flags, exact scale/camera/jitter, unchanged active extents,
  reference counts and dispatch counts. Wrong-camera and wrong-motion-extent
  inputs are rejected without native entry; valid retries succeed. These tests
  were compiled here, not executed on Windows.
- Paired [Rust](../../crates/fsr-sdk-sys/tests/dispatch_abi.rs) /
  [C++](../../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp) constant assertions.
  Rust assertions ran on the host; native C++ compilation is pending.

The prepared `depth-motion` case uses
[three-frame equipment](../../crates/fsr-sdk/tests/native/temporal_smoke.rs) and
[matching uploads](../../crates/fsr-sdk/tests/native/gpu.cpp). Eight representative
profiles cover all four depth conventions and both motion resolutions, jitter
conventions and units; this native selection is not an exhaustive product matrix:

| Depth | Motion resolution | Units | Projection jitter in vectors |
|---|---|---|---|
| Standard finite | Render | UV | Excluded |
| Standard finite | Display | Pixels | Included |
| Reversed finite | Render | Pixels | Included |
| Reversed finite | Display | UV | Excluded |
| Standard infinite | Render | UV | Included |
| Standard infinite | Display | Pixels | Excluded |
| Reversed infinite | Render | Pixels | Excluded |
| Reversed infinite | Display | UV | Included |

Each profile has three moving/jittered frames, first-frame reset, memory query,
native recording, submission/fences, finite/written output, device/debug checks
and explicit teardown. Display profiles allocate a real 640×360 motion texture;
colour/depth remain 320×180. UV translation after frame zero is `-1/320`; pixel
translation is `-1` at render resolution or `-2` at display resolution. Included
jitter adds the actual previous-minus-current projection displacement, normalised
by the render dimensions, then converts to the selected texture's pixel units
when needed. Depth uploads use standard/reversed `0.25/0.75`; infinite cameras use
the same near/FOV/metres and the finite far placeholder. No quality equivalence,
jitter-cancellation accuracy or signed FSR4 internal equation is asserted. The
original native smoke still uses its original default profile and acceptance
criteria. C++ helper and GPU execution have not run here.

## Local checks

Host: macOS `aarch64-apple-darwin`, Rust/Cargo 1.98.1, bundled Python 3.12.14 and
existing NumPy. Windows Rust target is installed. Rust 1.88, PowerShell,
Windows/MSVC and native DX12 execution are unavailable here. Cargo checks used
existing caches offline. From the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
python tools/verification/run.py run consumers
python -m unittest discover -s tools/verification/tests -v
cargo package --workspace --locked --offline --allow-dirty --no-verify
cargo package -p fsr-sdk --list --locked --offline --allow-dirty
python tools/verification/repository_size.py
git diff --check
```

Final checks pass. All 77 consumer cases pass. An isolated Windows-target consumer
also compiles the exact `dispatch-inputs-positive` source from
[consumer-cases.json](../../tools/verification/consumer-cases.json) with `movable`.
The infrastructure suite runs outside the sandbox for localhost fixtures: 148 of
153 tests pass, with three unavailable PowerShell-parser and two Linux-only checks
skipped. Draft archive inspection checks packaged changed source and licence
contents. Packaging with `--no-verify` does not establish extracted-package build
or final release acceptance. Local documentation links and file-size budget pass.

## Remaining execution

On the prepared Windows/MSVC/runtime/GPU environment:

```sh
python tools/verification/run.py run ci
python tools/verification/run.py run msrv
python tools/verification/run.py run native-upscaler --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Select the actual trusted SDK root and adapter. The native profile includes
`depth-motion` and does not acquire AMD binaries implicitly. No additional
platform support, native verification, final release acceptance or publication is
claimed. Arbitrary scales and coordinate conversions remain deferred.
