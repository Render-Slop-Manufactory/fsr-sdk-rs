# Loader platform organization verification — 2026-10-09

The later [loader refactor audit](2026-10-09-loader-refactor-audit.md) independently
reviews behavior, ownership and architecture and records freshly executed checks
with their native-platform limits.

## Change and responsibility boundaries

Implementation follows maintainer approval after task-prompt review and the
[D015 follow-up](../adr/D015.md#loader-organization-follow-up-2026-10-09).
Current source is distinct from the published `0.1.0` packages; this is not a
release or a native execution record.

`fsr-sdk-sys/src/loader/mod.rs` contains interface documentation, one private
Windows module and reexports of `FfxLibrary`/`LoadError`. The existing public
paths remain `fsr_sdk_sys::loader::{FfxLibrary, LoadError}`. The Windows module
contains all existing path resolution, five-export validation, forwarding,
errors and retained DLL ownership. A comparison against the pre-refactor file
confirmed identical contents except the Windows-specific module description.
No native function or safety-contract implementation changed.

Raw `libloading 0.9.0` is optional and Windows-target-specific, activated by
`dx12`. The wrapper retains its direct fixture dev-dependency. Production
dependency trees exclude dev-dependencies. The crate-root
`all(windows, feature = "dx12")` gate, defaults, Vulkan diagnostic and movable
semantics are unchanged. `Runtime` stays in its existing module; its state owns
no DX12 device. No shared platform framework, backend trait or Linux file exists.

## Exact changed files

Paths below are relative to the repository root. The source-investigation files
were already present as uncommitted work from the preceding analysis; they were
preserved, with current-source links/status updated where required.

| File | Reason |
|---|---|
| `crates/fsr-sdk-sys/src/loader.rs` → [loader/windows.rs](../../crates/fsr-sdk-sys/src/loader/windows.rs) | Move the existing native implementation; only its module description changes |
| [loader/mod.rs](../../crates/fsr-sdk-sys/src/loader/mod.rs) | Minimal organization and public reexports |
| [raw Cargo.toml](../../crates/fsr-sdk-sys/Cargo.toml) | Optional Windows dependency, enabled by DX12 |
| [raw README.md](../../crates/fsr-sdk-sys/README.md) | Explain current-source loader and dependency boundaries separately from published packages |
| [check-movable-api.ps1](../../tools/check-movable-api.ps1) | Preserve existing controls; add raw import, production-tree and Vulkan controls to the already-configured CI entry point |
| [D015](../adr/D015.md) | Explicit dated acceptance and resolution of the original narrower scope |
| [DECISIONS.md](../DECISIONS.md) | Summarize D015's loader follow-up |
| [DEVELOPMENT.md](../DEVELOPMENT.md) | Explain the extended CI consumer checks |
| [D010](../adr/D010.md), [D013](../adr/D013.md) | Repair links to the relocated implementation; contracts unchanged |
| [2026-09-20 loader record](../research/records/2026-09-20-exp-explicit-dll-loader.md), [2026-09-29 fixture record](../research/records/2026-09-29-exp-d013-windows-fixtures.md), [2026-10-09 soundness review](../research/records/2026-10-09-src-d013-soundness-architecture-review.md), [platform investigation](../research/records/2026-10-09-src-platform-backend-separation.md) | Navigation-only repair to the current implementation; historical observations, paths in recorded commands and fingerprints remain unchanged |
| [platform-and-backends.md](../research/platform-and-backends.md) | Replace the pending-cleanup status with implementation and verification limits |
| [research README.md](../research/README.md) | Update topic coverage |
| This record | Self-contained outcomes, reproduction commands and limits |

`Cargo.lock`, raw ABI declarations, runtime/context/upscaler source, native tests
and the CI workflow itself are unchanged. No new dependency version or licensing
model is introduced.

## Production dependency differences

The [preceding investigation](../research/records/2026-10-09-src-platform-backend-separation.md#2-actual-dependency-and-api-boundaries)
records the pre-refactor trees. Fresh locked offline trees confirm:

| Wrapper target/configuration | Before | After |
|---|---|---|
| Linux, defaults or explicit DX12 | `sys → libloading → cfg-if` | Only `fsr-sdk → fsr-sdk-sys` |
| Linux, no defaults | `sys → libloading → cfg-if` | Only `fsr-sdk → fsr-sdk-sys` |
| Windows, no defaults | `sys → libloading → windows-link` | Only `fsr-sdk → fsr-sdk-sys` |
| Windows, defaults/DX12 | Loader plus Windows interface dependencies | Same dependency versions and native path |

Both raw and wrapper packages were checked for default, no-default and explicit
DX12 configurations against both targets: 12 trees. Raw defaults enable no
backend; its Windows default tree also now omits `libloading`. No overhead was
benchmarked. Lockfile presence is not evidence of target dependency selection.

Reproduce each tree using the appropriate package/target and optional feature
flags:

```sh
cargo tree -p fsr-sdk --target x86_64-unknown-linux-gnu --edges normal --prefix none --locked --offline
cargo tree -p fsr-sdk --target x86_64-pc-windows-msvc --no-default-features --edges normal --prefix none --locked --offline
cargo tree -p fsr-sdk-sys --target x86_64-pc-windows-msvc --no-default-features --features dx12 --edges normal --prefix none --locked --offline
```

## Executed checks

Host: macOS ARM64, Rust `1.98.1`. Installed targets:
`aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. Dependencies were cached; no SDK,
toolchain or runtime was downloaded. Matrix checks used `CARGO_INCREMENTAL=0`;
documentation checks used `RUSTDOCFLAGS=-D warnings`.

Every Cargo command below used `--locked --offline`, except temporary downstream
consumers used `--offline` and generated their own lockfiles. Formatting used
`cargo fmt --all -- --check`.

| Check | Outcome |
|---|---|
| `cargo test --workspace` with defaults, `--features fsr-sdk/movable`, `--no-default-features`, and no defaults plus movable | All four pass: each runs 21 wrapper unit/ownership tests, 3 common consumer tests, 4 raw ABI/constant tests, 1 dispatch constant test and 2 wrapper doctests |
| `cargo clippy --workspace --all-targets` with movable, and separately without defaults, both with `-- -D warnings` | Both host checks pass |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` with default, movable, no defaults, no defaults plus movable | All four pass; Windows Rust compilation, including loader and gated test source where enabled |
| Windows-target Clippy with movable, all targets and denied warnings | Pass |
| `cargo doc --workspace --no-deps` with no defaults and with movable on host; Windows target with movable | All three pass with denied Rustdoc warnings |
| `cargo test -p fsr-sdk-sys --doc --features dx12 --target x86_64-pc-windows-msvc` | Both existing loader compile-fail doctests pass; private function extraction and invocation after owner drop remain rejected |
| 49 temporary downstream import controls on host/Windows target | All pass: common/legacy/raw imports, local and movable positive cases, unavailable-native `E0432` negatives and private-platform `E0603` negatives |
| Vulkan check on host and Windows target | Both return expected exit 101 and exact unsupported-feature message |
| `cargo package -p fsr-sdk-sys --list --allow-dirty` | Includes both new loader files and all licence texts; old `src/loader.rs` absent |
| `cargo package -p fsr-sdk-sys --allow-dirty --no-verify --target-dir target/loader-platform-verification/package` | Local archive generated; inspected loader files and normalized feature/optional target dependency wiring; MPL and AMD notices present and byte-identical to repository copies |

The temporary consumer manifest depended directly on both path crates with
`default-features = false`, exposing features `dx12 = ["fsr-sdk/dx12"]` and
`movable = ["fsr-sdk/movable"]`. For each host/Windows target and each of empty,
movable, DX12, DX12+movable feature sets, `cargo check --manifest-path ...`
compiled common inputs. Enabled Windows controls additionally imported
`loader::{FfxLibrary, LoadError}`, `runtime::Runtime`, both upscaler paths and,
where enabled, both movable paths with `Send` assertions. Other cells individually
rejected runtime, backend, legacy owners, raw loader and raw DX12 imports.
Windows/DX12 also rejected `loader::windows` as private. Exit status and exact
diagnostic codes were checked; these are compilation controls, not native calls.

Vulkan reproduction:

```sh
cargo check -p fsr-sdk --no-default-features --features vulkan --locked --offline
cargo check -p fsr-sdk --no-default-features --features vulkan --target x86_64-pc-windows-msvc --locked --offline
```

## Checks not executed and remaining limits

- Windows executable loader fixtures were not run: DLL load failures, missing
  exports, forwarding and final module release remain covered by their unchanged
  Windows tests. Host `tests/loader.rs` runs zero tests because its gate is false;
  that is not a Windows loader pass. Windows compilation and two compile-fail
  doctests passed separately.
- MSVC compilation against pinned AMD headers was not run; Windows SDK/MSVC and
  local SDK headers are unavailable. Raw ABI Rust tests passed on the host, but
  they do not replace paired native header verification. Existing C++ probes and
  Windows CI coverage remain unchanged.
- AMD loading, native lifecycle/provider behavior, GPU dispatch/readback and
  native movable transfers were not executed. Their opt-in tests are unchanged;
  earlier native evidence is not claimed as verification of this working tree.
- Linux Rust compilation was not run because its target standard library is not
  installed. Linux production dependency resolution was checked. Native Linux
  and Vulkan are still unsupported.
- Rust 1.88 and PowerShell are unavailable locally. The extended PowerShell
  consumer/CI script was inspected but not executed; direct Cargo checks and
  temporary consumer controls supply the local evidence above. CI still invokes
  that script on Ubuntu and Windows; no remote CI pass is claimed.
- Archive generation used `--no-verify`; it is a contents/notice check, not a
  packaged-release validation or publication.

A future Linux module must independently establish callable non-null exports,
native ABI/callback representation, runtime/provider artefacts, deployment and
cleanup contracts. The Windows implementation's symbol guarantees must not be
assumed for Unix. Vulkan requires an actual upstream integration plus its own
resource ownership and synchronization contract. This organization changes
neither the pinned SDK nor those requirements.
