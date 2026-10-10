# Portable common upscaler inputs — 2026-10-09

## Change and inputs

The accepted follow-up to [D015](../adr/D015.md) makes `upscaler`, its
[`common`](../../crates/fsr-sdk/src/upscaler/common.rs) module and common-type
reexports available without platform or backend gates. `upscaler::dx12`, its
backend reexports, `runtime` and the raw loader/DX12 modules still require
Windows plus `dx12`; movable owners additionally require `movable`.

Runtime stays in its existing module: its state owns the library, not GPU
resources. Only explanatory runtime module documentation changed. No acquisition,
threading, cleanup or dispatch safety contract changed. No dependency, ABI,
resource validation or native call changed. Vulkan and D014 remain unimplemented.

The consumer tests now separate common values from native import compatibility.
Existing scalar assertions moved from the DX12 unit tests into the public
consumer test without weakening their domains or expected errors. The downstream
control script now checks absent native modules directly and absent legacy owners
separately. CI adds these controls on Ubuntu, executes common tests without
default features, and builds documentation with and without native features.

Checks ran against the current working tree. This is bounded implementation
verification, not a release or a new native experiment.

- Host: `x86_64-pc-windows-msvc`, Windows 11 Pro build 26300.
- Rust stable 1.99.0; minimum-version compilation with 1.88.0.
- PowerShell 7.6.6; MSVC tools 14.44.35207, compiler 19.44.35224.0;
  Windows SDK 10.0.22621.0.
- GPU: AMD Radeon RX 9060 XT, driver 32.0.31041.1004.
- AMD SDK v2.3.0; headers pinned at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Provider 4.1.1, ID `0xf5a5ca1e01001001`.
- Loader SHA-256: `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler DLL SHA-256: `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
- Both AMD signatures were valid. Existing locally acquired SDK files were used;
  no SDK download. Ordinary DX12 debug layer enabled, GBV disabled.

## Commands and outcomes

Commands below are relative to the repository root. Quality, documentation,
consumer, native and packaging checks used `CARGO_INCREMENTAL=0`. Rustdoc used
`RUSTDOCFLAGS=-D warnings`. Existing fixture/compiler tests ran outside the
sandbox because their temporary compiler outputs require that access.

| Check | Outcome |
|---|---|
| `cargo test --workspace --locked --offline` | Pass through the native runner; native tests ignored here |
| `cargo test --workspace --features fsr-sdk/movable --locked --offline` | Pass, including six public movable consumer tests and compile-fail doctests |
| `cargo test --workspace --no-default-features --locked --offline` | Pass; three common consumer tests run, native consumer tests omitted |
| `cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline` | Pass; common values remain usable, movable native APIs omitted |
| `./tools/check-movable-api.ps1 -Stage target/portable-common-20261009/consumers` | Pass: four positive and eighteen diagnostic-specific negative controls on Windows |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass through the native runner |
| `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings` | Pass |
| `cargo doc --workspace --no-default-features --no-deps --locked --offline` | Pass, warnings denied |
| `cargo doc --workspace --features fsr-sdk/movable --no-deps --locked --offline` | Pass, warnings denied |
| `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass |
| `cargo +1.88.0 check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Pass |
| `./tools/check-native-abi.ps1 -SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX -Stage target/portable-common-20261009/abi` | Pass, three independent C++ probes against hash-verified pinned headers |
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode movable -Native -Stage target/portable-common-20261009/native` | Pass, exit 0; ordinary checks, ABI probes, helper build and six native children |
| `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` | Pass |
| `cargo package -p fsr-sdk --allow-dirty --no-verify --locked --offline` | Pass; local archive inspected, no package-build verification or publication |
| `cargo fmt --all -- --check`, `git diff --check` and changed-document local link checks | Pass |

The downstream controls accept common inputs and common raw ABI types without
DX12. Each absent native module and legacy owner is checked independently under
no features and `movable` alone, requiring `E0432`. Existing movable feature,
crossed-family and conversion controls still require `E0432`, `E0308` or `E0277`
as appropriate. Positive baselines establish working compiler/dependency inputs.

Archive inspection confirms the LICENSE matches the root copy, LICENSE-AMD is
retained, and changed source, consumer tests and README match the workspace.
One incremental-cache hard-link warning occurred during the standalone movable
test run; the tests passed. Warning-denying quality checks disabled incremental
compilation and passed.

## Selected native regressions

All six isolated children exited 0 without timeout. The explicit/drop lifecycle
tests created two contexts, retained the loader after early public-owner release,
and completed cleanup. Single-frame readback and all three moving frames each
produced 230,400 finite, changed pixels at 640x360. Debug errors were zero and
no device removal occurred.

The existing motion/jitter/reset comparisons passed: analytic red MAE was
0.01334815 for correct inputs, 0.25143567 for reversed motion and 0.04247920 for
reversed jitter; reset delta was 0.00000000. The public movable regression passed
two fenced frames, independent siblings and coordinated cleanup on destination
threads during a deliberate caught unwind and failed handoff.

## Limits

Local Linux tests were skipped at the maintainer's request. Ubuntu CI is
configured for portable consumers and absent native APIs even with `dx12`
enabled, but was not executed in this verification. No Linux native support or
remote CI success is claimed. The native results cover only the recorded
configuration and existing workloads; they establish neither general image
quality, arbitrary provider/thread compatibility, Vulkan nor D014 sharpening.
No commit, publication or release validation was performed.
