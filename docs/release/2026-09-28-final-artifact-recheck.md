# Final release-artifact recheck — 2026-09-28

Both `fsr-sdk-sys 0.1.0` and `fsr-sdk 0.1.0` are **READY FOR RELEASE SEQUENCE**.
The validated source/package inputs are suitable to declare **RELEASE CANDIDATE
FREEZE**. The real release order remains sys, then wrapper.

This pass regenerated and verified source archives after the completed Windows
Gate A/Gate B verification. It did not publish, tag, reconstruct Git history,
rerun native GPU work, or change production code, API, versions, metadata,
dependencies, features, licensing, runtime/provider policy or D011/D012.

## Candidate and artifact identity

- Captured before packaging: `2026-09-28T17:32:09.1156151Z`.
- Source commit: `3844935a639948985e7266d0049d0bf351c51e36`.
- Source Git tree: `43de00f21c37cf4a33e4f72816355e53808ef906`.
- Working-tree status: clean; `git status --porcelain=v1` returned no entries.
- Uncommitted source changes: none; both package versions are exactly `0.1.0`.
- Toolchain: Rust 1.98.1 (`48a229cea`, 2026-09-01), Cargo 1.98.1
  (`797e8a9bc`, 2026-08-05); Windows host and installed target
  `x86_64-pc-windows-msvc`. No target/toolchain installation was needed.

The [source-state capture](2026-09-28-final-artifact-recheck/source-state.json)
includes SHA-256 fingerprints for all 57 tracked crate files and the
workspace manifest/lockfile. Its SHA-256 is
`0f1ade5094a186501b04dd2ce5711ad66ecc22618dce0a95b849eab90cebdde7`.
Every source fingerprint still matched after package, dry-run and consumer checks.
Both Cargo VCS records name the clean commit above, with the appropriate
`crates/fsr-sdk-sys` / `crates/fsr-sdk` path and no dirty flag.

Fresh archives are under `target/final-artifact-recheck/package/`. These are the
current pre-freeze archive identities; archives were not manually edited.

| Archive | Files | Compressed bytes | SHA-256 |
|---|---:|---:|---|
| `fsr-sdk-sys-0.1.0.crate` | 28 | 32,078 | `8c67214edf3cabfc577eace6b9798b37a9febf9898bd675495482378c1662cb0` |
| `fsr-sdk-0.1.0.crate` | 24 | 58,005 | `4eea0938c5da942d4b4f151cd2f989b6a95ca6629c24c629f59a92ab02c6eeda` |

Hashes remained identical after all package/dry-run/consumer checks. Local
registry copies and consumer lockfiles contain these exact checksums.

The only subsequent repository changes are this report and evidence, plus its
links/status in `docs/DEVELOPMENT.md` and `ROADMAP.md`. They are outside both
crate packages. The [post-report change manifest](2026-09-28-final-artifact-recheck/post-report-changes.json)
identifies those uncommitted documentation changes individually; they are not
silently represented by the packaged source Git tree.

## Windows changes and package contents

Compared with the preceding release simulation, the only changed crate files
were the following test/harness sources. Production API changes: **none**.
Production runtime behavior changes: **none**. Test/harness changes: **yes**.
The Rust module is included only by the existing `#[cfg(test)]` declaration in
`src/upscaler.rs`; the C++ source is an opt-in native test helper.

| Windows harness file | sys archive | wrapper archive |
|---|---|---|
| `src/upscaler/m6.rs` | Absent | Present, 20,382 bytes; test-only module |
| `tests/m6_gpu.cpp` | Absent | Present, 22,591 bytes; test-only source helper |
| `tests/run-m6.ps1` | Absent | Excluded by `tests/**/*.ps1` |

The established package rules intentionally retain test/fixture/helper source.
Simply excluding `m6.rs` would leave the package's unit-test module referring to
a missing source file. The extracted wrapper's Windows `--all-targets` check
passed, including that module and the integration example. No helper DLL is
packaged. This recheck preserved the current exclusions rather than changing
the distribution layout.

The [archive audit and complete per-file inventories](2026-09-28-final-artifact-recheck/archive-audit.json)
record every entry's size and SHA-256. Both entry sets match the post-D012
simulation: sys has 28 files; wrapper has 24. Rust sources, generated/original
Cargo manifests, generated lockfiles/VCS records, READMEs, byte-identical copies
of the root MPL-2.0 license, retained AMD MIT notices, tests/fixtures/native
source helpers and the wrapper example are present. No separate root project
documentation tree is part of either crate's packaging contract.

Every non-generated archived file matches its current Windows working-copy
bytes. Comparing those bytes with the recorded post-D012 source after CRLF-to-LF
normalization finds no sys source changes and exactly two wrapper source changes:
`src/upscaler/m6.rs` and `tests/m6_gpu.cpp`. Archive-byte differences also include
the new VCS revision and Windows line endings versus the earlier macOS run;
the sys hash therefore changes even though its source behavior is unchanged.
The previous archives are not present on this host, so this is an inventory and
source/metadata comparison, not an unverified claim that all archive-byte
differences arise solely from the two harness edits.

There are no duplicate/escaping paths, symlinks, `external/` or FidelityFX SDK
checkout, AMD DLLs, other compiled binaries, build output, PDB/EXE/OBJ files,
local Cargo configuration/caches, credential files or temporary files. Focused
content scans found no credentials/private-key/token patterns, personal absolute
paths, local username or machine name in either archive or the newly committed
Windows verification material. GPU/device/driver/provider versions, hashes and
native error codes in the verification evidence remain intact. These are focused
sanity scans, not a guarantee that every possible secret form can be detected.

## Normalized manifests

The exact generated manifests are retained for
[sys](2026-09-28-final-artifact-recheck/fsr-sdk-sys-normalized-Cargo.toml) and
[wrapper](2026-09-28-final-artifact-recheck/fsr-sdk-normalized-Cargo.toml).
Both still use the same names, `0.1.0`, edition 2024 and `MPL-2.0` metadata.
Source crate manifests and workspace manifest/lockfile are unchanged from the
post-D012 candidate. Generated features, docs.rs metadata, dependency/dev-
dependency and target-specific dependency structure match those source manifests.

The wrapper registry dependency is:

```toml
[dependencies.fsr-sdk-sys]
version = "0.1.0"
default-features = false
```

The workspace `path` does not survive into the registry manifest. Wrapper default
features still enable `dx12`; sys has no default features. Both docs.rs default
targets remain Windows x64/MSVC, with sys's existing `dx12` docs feature. Vulkan
remains an intentionally unsupported compile-error feature; `--all-features`
was not used as a success gate.

## Packaging and publish simulation

All commands below ran from the repository root with ordinary extracted-build
verification and registry index access. Neither `--allow-dirty` nor `--no-verify`
was used. An initial sandbox connection failure for sys packaging was retried
with approved registry access; it was an environment restriction, not an artifact
defect. No additional dependency or tooling was introduced.

```sh
cargo package -p fsr-sdk-sys --locked --target-dir target/final-artifact-recheck --config net.retry=0
cargo package -p fsr-sdk --locked --target-dir target/final-artifact-recheck --config net.retry=0
cargo package --workspace --locked --target-dir target/final-artifact-recheck --config net.retry=0
cargo publish --dry-run -p fsr-sdk-sys --locked --target-dir target/final-artifact-recheck --config net.retry=0
cargo publish --dry-run -p fsr-sdk --locked --target-dir target/final-artifact-recheck --config net.retry=0
cargo publish --dry-run --workspace --locked --target-dir target/final-artifact-recheck --config net.retry=0
```

| Selection | Cargo exit | Result |
|---|---:|---|
| Sys package | 0 | PASS; archive and extracted build verified |
| Wrapper package alone | 101 | Expected unpublished sys registry dependency |
| Workspace package | 0 | PASS; both fresh archives and extracted builds verified |
| Sys publish dry-run | 0 | PASS; upload aborted due to dry run |
| Wrapper publish dry-run alone | 101 | Same expected release-order condition |
| Workspace publish dry-run | 0 | PASS; sys then wrapper verified, both uploads aborted |

The only cause reported by the standalone wrapper commands was:

```text
no matching package named `fsr-sdk-sys` found
location searched: crates.io index
required by package `fsr-sdk v0.1.0 (<repo>/crates/fsr-sdk)`
```

Cargo's joint commands supply a temporary local registry for the unpublished
workspace dependency. Each simulated upload ended with
`warning: aborting upload due to dry run`. Combined verification and the isolated
consumers below passed, so the wrapper's standalone registry condition is not
a release-sequence blocker. Authentication, ownership, actual server acceptance
and hosted docs.rs publication were not exercised by this local gate.

## Isolated registry consumers

Two fresh consumers outside the repository each have their own workspace, fresh
target output and only versioned FSR dependencies. A fresh temporary Cargo home
uses `crates-io` source replacement with a temporary local registry. The registry
contains byte-identical copies of these two archives, Cargo-generated FSR index
entries, and the 17 cached third-party archive/index versions from the unchanged
workspace lockfile. Every third-party checksum matched its archive, index entry
and workspace lockfile. [Registry inputs](2026-09-28-final-artifact-recheck/registry-inputs.json)
record the archive names and checksums.

The dependency specifications are `fsr-sdk = "=0.1.0"` and, in the independent
raw consumer, `fsr-sdk-sys = { version = "=0.1.0", features = ["dx12"] }`.
There are no path dependencies or patches. The actual
[consumer sources/manifests/lockfiles](2026-09-28-final-artifact-recheck/consumer-inputs.json)
and [metadata assertions](2026-09-28-final-artifact-recheck/isolated-validation.json)
are included. Metadata confirms one member per consumer workspace, registry
source IDs, extracted FSR manifests under the isolated Cargo home, exact archive
checksums, no manifest paths into the original repository, and no Vulkan feature.

In each consumer directory, these passed with exit 0:

```sh
cargo generate-lockfile --offline
cargo check --all-targets --locked --offline
cargo check --all-targets --locked --offline --target x86_64-pc-windows-msvc
cargo metadata --format-version=1 --locked --offline
```

The wrapper consumer imports the actual current module paths:

```rust
use fsr_sdk::{
    runtime::Runtime,
    upscaler::{Dimensions, Upscaler, UpscalerOptions},
};
```

It compiles `UpscalerOptions::new(max_render_size, max_upscale_size)` with checked
`Dimensions`, both getters, `Debug` and explicit `Clone`, `Upscaler::new`,
`unsafe { Runtime::load_unchecked(path) }` and unsafe public dispatch forwarding.
Both complete packaged crate-root `no_run` examples, including hidden setup and
their Windows/DX12 bodies, compile as consumer examples. The raw consumer checks
`ffxApiHeader`, the DX12 backend descriptor and `FfxLibrary`.

With the archives independently extracted under `packages/` in the isolated
environment, these also passed:

```sh
cargo check --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --all-targets --locked --offline --target x86_64-pc-windows-msvc
cargo doc --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --no-deps --locked --offline --target x86_64-pc-windows-msvc
cargo doc --manifest-path packages/fsr-sdk-sys-0.1.0/Cargo.toml --no-deps --features dx12 --locked --offline --target x86_64-pc-windows-msvc
```

Both documentation commands used `RUSTDOCFLAGS=-D warnings`. Packaged Windows
HTML exposes `load_unchecked`, with no `load` method, and the options constructor,
getters, `Debug` and `Clone`, without `Copy`, `PartialEq` or `Eq`. Windows target
compilation passed. Since this host is Windows x64/MSVC, the host and explicit
target checks cover the same platform. No GPU execution was requested or run.

## Standard release checks and regression scans

Every workspace check below passed. `cargo doc` commands used
`RUSTDOCFLAGS=-D warnings`; Clippy denied warnings. Validation used the unchanged
lockfile and cached dependencies.

| Command | Cargo exit |
|---|---:|
| `cargo package --workspace --locked --target-dir target/final-artifact-recheck --config net.retry=0` | 0 |
| `cargo publish --dry-run --workspace --locked --target-dir target/final-artifact-recheck --config net.retry=0` | 0 |
| `cargo fmt --all -- --check` | 0 |
| `cargo check --workspace --all-targets --locked --offline` | 0 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 |
| `cargo test --workspace --locked --offline` | 0 |
| `cargo check --workspace --no-default-features --locked --offline` | 0 |
| `cargo doc --workspace --no-deps --locked --offline` | 0 |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline` | 0 |
| `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline -- -D warnings` | 0 |
| `cargo check --workspace --no-default-features --target x86_64-pc-windows-msvc --locked --offline` | 0 |
| `cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --locked --offline` | 0 |
| `cargo test --workspace --no-default-features --locked --offline` | 0 |
| `cargo clippy --workspace --all-targets --no-default-features --locked --offline -- -D warnings` | 0 |
| `cargo doc --workspace --no-deps --no-default-features --locked --offline` | 0 |

Default-feature tests passed 25 wrapper executable tests, six sys ABI tests,
two dispatch ABI tests and four loader fixture tests. Four doctests passed.
The five wrapper native tests and six sys native/experiment tests remained
ignored. The no-default-feature suite passed ten wrapper executable tests,
six sys ABI tests and two dispatch ABI tests; two configuration-gated wrapper
doctest shells passed. Windows/DX12 doctest bodies were separately compiled with
DX12 enabled as described above. Fixture DLL tests do not load AMD runtimes or
perform GPU work.

The focused stale-API scan found no current `Runtime::load(` calls or old options
construction literals in source, READMEs or examples. The old options literal
in the earlier release simulation in `docs/DEVELOPMENT.md` remains historical;
other `UpscalerOptions {` matches are declarations/impls or a test helper return
type. Historical loader-name mentions in earlier records remain accurate and
were not rewritten. No current syntax correction was needed.

The [command results](2026-09-28-final-artifact-recheck/command-results.json)
retain exact arguments, native Cargo exit codes and sanitized outputs; check
runner records also carry UTC timestamps. `<repo>`, `<isolation>` and
`<user-home>` substitute unrelated personal path prefixes.

## Final provenance and classification

```text
source commit: 3844935a639948985e7266d0049d0bf351c51e36
source tree:   43de00f21c37cf4a33e4f72816355e53808ef906
uncommitted packaged changes: none
fsr-sdk-sys .crate SHA-256: 8c67214edf3cabfc577eace6b9798b37a9febf9898bd675495482378c1662cb0
fsr-sdk .crate SHA-256:     4eea0938c5da942d4b4f151cd2f989b6a95ca6629c24c629f59a92ab02c6eeda

fsr-sdk-sys: READY FOR RELEASE SEQUENCE
fsr-sdk:     READY FOR RELEASE SEQUENCE
```

The current candidate's validated package source and archive hashes are suitable
as the reference point for **RELEASE CANDIDATE FREEZE**. The documentary follow-up
is explicitly listed above and in its change manifest. This pass supplies the
artifact gate; it does not perform publication, tagging or the separately planned
public-history preparation.
