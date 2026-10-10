# Repository review maintenance verification

- Date: 2026-09-30.
- Scope: review-driven maintenance of the current source workspace, after release
  0.1.0. No publication, commit or change to the published archives was performed.
- Native baseline: SDK v2.3.0, Windows x64/MSVC; the supported target policy is unchanged.

## Changes and rationale

| Area | Result |
|---|---|
| Public dimensions | Add `width()` and `height()` so renderers can recover the checked configuration. The external [consumer test](../../crates/fsr-sdk/tests/dimensions.rs) checks cloned options, values, zero rejection and `u32::MAX` roundtrips. These accessors do not certify that a size is accepted by AMD. |
| Production modules | Split dispatch into [frame values](../../crates/fsr-sdk/src/upscaler/dispatch/frame.rs), [DX12 inspection/conversion](../../crates/fsr-sdk/src/upscaler/dispatch/resources.rs), [descriptor construction](../../crates/fsr-sdk/src/upscaler/dispatch/descriptor.rs) and [orchestration/poison policy](../../crates/fsr-sdk/src/upscaler/dispatch.rs). Existing public paths and behavior remain intact. |
| Ownership/inspection tests | Separate construction, dispatch failure policy, lifecycle, traits and fixture equipment under `src/upscaler/tests`. Preserve documented ignored-test filters. |
| Native equipment | Move GPU equipment to `tests/native`, split Rust cases and analytic image generation, and share one staging/signature/timeout runner. Keep milestone-named runners as compatibility entry points. |
| Toolchain | Declare Rust 1.88 in both manifests. CI runs exactly 1.88.0 on Ubuntu and Windows in addition to current stable checks. `libloading` 0.9.0 already requires 1.88, so the declaration exposes the existing dependency floor. |
| Independent ABI | Windows CI compiles common, creation and dispatch probes against four hash-verified headers at the pinned commit. [The script](../../tools/check-native-abi.ps1) also accepts the local SDK; Cargo builds still acquire no SDK or AMD DLLs. |
| Documentation | Separate current development guidance from [historical verification](2026-09-30-development-history.md); split API/safety details into loading, lifecycle, construction and threading topics. Correct stale publication, licensing and D013 fixture status. Add short decision summaries without changing ADR acceptance. |
| D013 seam | Retain the small sealed ownership parameter because it tests the real lifecycle implementation with both retention mechanisms. Production still instantiates `Rc`; `Arc` instantiations and manual transfer wrappers remain experimental tests. No movable public family is implemented. |
| Semantic evidence | Add [controlled comparisons](../research/records/2026-09-30-exp-dispatch-semantics.md) for horizontal motion, jitter and history/reset. The comparisons exposed and corrected a jitter-sign error in synthetic input generation; no production descriptor sign was changed. |

The ordinary production Rust modules now stay below 350 lines (the largest
dispatch inspection module is 332 lines). Larger test-equipment files and dated
records retain their coherent responsibilities. The current development guide
is about 200 lines; its former release history remains available as a dated
snapshot. This is a responsibility-based split, not a universal line-count rule.
M7–M9 and D013 remain proposals; this maintenance does not implement sharpening
or the breaking frame representation.

## Commands and observed results

Run from the repository root; dependencies were cached except for the registry
lookup needed by package verification. No AMD binaries were downloaded.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --no-deps --locked --offline
cargo +1.88.0 check --workspace --all-targets --locked --offline
cargo +1.88.0 test --workspace --locked --offline
cargo +1.88.0 check --workspace --all-targets --no-default-features --locked --offline
./tools/check-native-abi.ps1 -SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
./tools/check-native-abi.ps1 -Stage target/repo-review/downloaded-abi
./crates/fsr-sdk/tests/run-m6.ps1 -Native -Stage target/repo-review/semantic-corrected
./crates/fsr-sdk/tests/run-m5.ps1 -Native -Stage target/repo-review/lifecycle-final
cargo package --workspace --allow-dirty --locked --config net.retry=0
```

- Rust 1.98.1 and 1.88.0 passed: 33 wrapper unit/fixture tests, one public consumer
  test, 12 sys tests, and four doctests. Ignored native cases were run separately
  rather than counted as ordinary successes. Minimum-version all-targets checks
  passed with and without default features. Clippy and Rustdoc denied warnings.
- All three native ABI probes passed using the local SDK and separately using
  freshly downloaded headers. A modified cached copy of `ffx_api.h` was rejected
  with `Pinned header hash mismatch` before compilation; that copy was restored.
  The installed local SDK was not modified.
- Gate A, all three Gate B frames and 33 Gate C frames passed on the recorded
  Radeon RX 9060 XT/driver 32.0.31041.1004/provider 4.1.1 configuration. Matching
  motion/jitter beat the deliberate-error comparisons, reset matched a fresh
  reset, outputs were finite, and debug/device checks were clean. Both two-context
  lifecycle cases passed. [The experiment](../research/records/2026-09-30-exp-dispatch-semantics.md)
  retains exact inputs, numerical results, DLL fingerprints and the initial
  unsuccessful jitter assumption.
- PowerShell scripts parsed successfully, and repository-relative Markdown
  links/anchors were checked after relocation. Historical command text retains
  original paths; actual links follow the current locations.
- Workspace package creation and verification passed for both archives. The
  flattened manifests declare `rust-version = "1.88"`; the wrapper registry
  dependency remains `fsr-sdk-sys = "0.1.0"` with default features disabled.
  The sys archive contains 28 files and wrapper 41. Both include their licenses;
  the wrapper includes relocated Rust test modules, the public API test and the
  GPU helper/header. Checkout-only PowerShell runners are excluded. No SDK
  checkout, DLLs, build output or local configuration is packaged.

An earlier offline workspace packaging attempt produced both archives but Cargo
1.98.1 failed verification with its internal `no hash listed for fsr-sdk-sys`
error. The separate offline wrapper attempt lacked cached registry data. The
listed online metadata lookup resolved package verification; no publishing was
performed. This distinction avoids presenting an archive inventory alone as a
successful package build.

## Remaining verification boundaries

The modified GitHub workflow has not been dispatched here; Ubuntu and hosted
Windows job results must come from the next CI run. Local Windows checks establish
the stated local results. The controlled image covers one workload/configuration,
not broad temporal quality or vertical-motion semantics. GPU-based validation,
additional providers/hardware and native thread transfer remain unverified.
Package checks use the existing 0.1.0 source version for maintenance; any future
release needs its own version choice and final artifact verification.
