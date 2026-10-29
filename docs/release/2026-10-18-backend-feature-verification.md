# Backend feature and isolation checks

Date: 2026-10-18. Development-source checks for
[D031](../adr/D031.md), `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`.

## Inputs and change

The tested checkout contains implemented D028/D029 and proposed, unimplemented
D030. This change removes only the nonfunctional `vulkan` features and the raw
compile-error branch, documents Vulkan as a required future support target and
strengthens consumer checks for Windows/DX12 encapsulation. Existing DX12
platform gates, optional target dependencies, ownership and ABI declarations
remain in place. No dependencies or SDK artefacts were acquired.

Host: macOS ARM64; Rust/Cargo 1.98.1, Python 3.14.8, and installed
`x86_64-pc-windows-msvc` target. The separate MSRV check uses Rust 1.88.0.
No Windows host, AMD runtime or GPU workload was used.

## Commands and observations

Run from the repository root, with dependencies cached:

| Command | Observation |
|---|---|
| `python3.14 tools/verification/run.py run consumers --require-passed` | 88 cases pass. Portable positive controls, disabled-native-API negative controls, 24 Linux/Windows normal dependency graphs, both package all-feature builds and examples pass. |
| `python3.14 -m unittest discover -s tools/verification/tests -p test_infrastructure.py -q` | 27 tests pass, including negative controls for leaked/missing Windows/loader dependencies. |
| `cargo test --workspace --all-features --offline --locked` | 71 host tests/doctests pass; Windows-gated tests are not executed. |
| `cargo check --workspace --all-targets --all-features --target x86_64-pc-windows-msvc --offline --locked` | Cross-compilation passes; no Windows execution. |
| `cargo +1.88.0 check --workspace --all-targets --all-features --offline --locked` | Exact MSRV host compilation passes. |
| `cargo fmt --all -- --check` | Passes. |
| `cargo package --workspace --list --allow-dirty --offline --locked` | Package inventories inspected. |
| `cargo package --workspace --allow-dirty --offline --locked --no-verify --target-dir target/backend-features-package` | Both archives produced locally: 31 sys files and 102 wrapper files. Nothing published. |

Normal dependency graphs require both `windows` and `libloading` to be absent
for Linux and Windows common-only consumers, even when unrelated movable
support is enabled. Windows/DX12 wrapper consumers require both; raw
Windows/DX12 consumers require only the loader dependency. Host fixture
dev-dependencies are outside these normal dependency graphs.

Local archive inspection confirms that neither normalized manifest advertises
`vulkan`, project licence files match the root and the sys AMD notice is retained.
This is package-content inspection, not final extracted-package or registry-only
execution. Local logs and archives remain ignored under `target/`.

The initial default `python3` was version 3.9 and could not import `tomllib`;
checks were rerun with the required newer interpreter. Six infrastructure
process controls initially failed because the sandbox forbade their loopback
listener. The complete 27-test rerun outside that restriction passes. Formatting
also caught an extra trailing blank line after branch removal; it was corrected
before the passing check.

## Limits

These checks establish the current feature/dependency/API boundaries, not Vulkan
availability, Windows runtime loading or GPU correctness. D031 preserves the
Vulkan objective and mandatory platform/backend encapsulation. Native Vulkan
integration remains blocked on a usable upstream path. Final release checks,
including the sys crate's feature-break/version review, remain open.
