# Common/DX12 module separation verification — 2026-10-09

## Change and inputs

[D015](../adr/D015.md) separates checked upscaler inputs in
[`common`](../../crates/fsr-sdk/src/upscaler/common.rs) from the concrete
[`DX12 backend`](../../crates/fsr-sdk/src/upscaler/dx12.rs). The original module
reexports the same public types. Configuration domains, descriptors, resource
validation, ownership, native calls and Windows/feature gates are preserved.
Vulkan and D014 remain unimplemented. No dependency or ABI declaration changed.

This is routine implementation verification, not a new native experiment or a
release. Checks ran against the current working tree containing the relocated
sources and the consumer import-compatibility test. Eleven relocated files
(descriptor translation, movable ownership and all nine DX12 test modules)
were also compared with their pre-change contents and matched after line-ending
normalization. Existing assertions were not weakened.

- Native target: `x86_64-pc-windows-msvc`, Windows 11 Pro build 26300.
- Rust: stable 1.99.0; minimum-version compilation with 1.88.0.
- MSVC tools 14.44.35207, compiler 19.44.35224.0; Windows SDK 10.0.22621.0.
- GPU: AMD Radeon RX 9060 XT; driver 32.0.31041.1004.
- AMD SDK v2.3.0, pinned headers at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Selected upscaler provider: 4.1.1, ID `0xf5a5ca1e01001001`.
- Loader SHA-256: `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler DLL SHA-256: `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
- Both AMD DLL signatures were valid. Existing locally acquired SDK files were
  used; no new SDK download. Ordinary DX12 debug layer enabled, GBV disabled.

## Commands and outcomes

Commands are relative to the repository root. Supplemental quality/package
checks used `CARGO_INCREMENTAL=0`; Rustdoc used `RUSTDOCFLAGS=-D warnings`.

| Check | Outcome |
|---|---|
| `cargo test --workspace --features fsr-sdk/movable --locked --offline` before and after the source refactor | Pass outside the sandbox, including fixture, public consumer and compile-fail tests; hardware tests remain ignored in this command |
| `cargo check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings` | Pass |
| `cargo check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo doc --workspace --features fsr-sdk/movable --no-deps --locked --offline` | Pass with Rustdoc warnings denied |
| `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo +1.88.0 check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Pass |
| `./tools/check-native-abi.ps1 -SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX -Stage target/common-dx12-20261009/abi` | Pass, three independent C++ probes against hash-verified pinned headers; initialized the x64 compiler environment |
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode movable -Native -Stage target/common-dx12-20261009/native` | Pass, exit 0; default fmt/Clippy/tests/no-default compilation, ABI checks, helper build and all six native children |
| `./tools/check-movable-api.ps1 -Stage target/common-dx12-20261009/consumer-controls` | Pass, two positive and six diagnostic-specific negative consumer controls |
| `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` | Pass |
| `cargo package -p fsr-sdk --allow-dirty --no-verify --locked --offline` | Pass, archive created and selected contents inspected; no package-build verification or publication |
| `git diff --check` | Pass |

The consumer test accepts the same sizes/options/scalars, constructor signatures,
dispatch input types and DX12 interfaces through both old and explicit module
paths, including `MovableUpscaler` when enabled. Archive inspection confirmed
matching LICENSE, LICENSE-AMD and selected relocated implementation/test files.

The initial sandboxed baseline failed because an existing compiler-control test
could not write its temporary `.rmeta` file. The authorized unsandboxed baseline
and post-change tests passed. An intermediate Rustdoc run found unresolved short
module links; these were qualified and the final warning-denying build passed.

## Native observations

All six children exited 0 without timeout, using the updated exact filters under
`upscaler::dx12`:

| Child | Observation |
|---|---|
| `tests::native_lifecycle` | Two contexts, early runtime/device release, successful explicit destruction |
| `tests::native_lifecycle_drop` | Same lifecycle with automatic final cleanup |
| `m6::native_dispatch_readback` | Submitted and fenced 640×360 output; 230,400 finite, changed pixels; zero debug errors and no device removal |
| `m6::native_temporal_smoke` | Three moving, individually fenced frames; each had 230,400 finite, changed pixels; zero debug errors and no device removal |
| `m6::native_semantic_comparisons` | Existing motion/jitter/reset checks passed; analytic red MAE 0.01334815 for correct inputs, 0.25143567 for reversed motion, 0.04247920 for reversed jitter; reset delta 0.00000000 |
| `m6::movable_public::public_siblings_dispatch_and_cleanup_after_thread_handoffs` | Two fenced public movable frames; independent siblings; coordinated destination-thread cleanup during deliberate caught unwind and failed handoff; zero debug errors and no device removal |

These results establish the selected existing regressions after the refactor.
They do not establish pixel-for-pixel equivalence with a separate pre-refactor
GPU run, general image quality, performance, arbitrary AMD threading guarantees,
other GPU/driver/provider support, Vulkan, or D014 sharpening. No remote CI,
non-Windows execution or new release validation is claimed.
