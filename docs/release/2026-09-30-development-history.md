# Development verification history

Snapshot of the development guide before the 2026-09-30 maintenance changes.
Status and commands below describe the original runs. For current commands,
MSRV, CI and source locations, use [development](../DEVELOPMENT.md). Native runner
entry points remain available; links below follow relocated code, while recorded
command text retains its original paths.


This document covers local checks and native verification. Contribution policy
lives in [CONTRIBUTING.md](../../CONTRIBUTING.md); design guidance lives in
[CODE_HEURISTICS.md](../CODE_HEURISTICS.md). Manifests, source, and any future CI
configuration establish the checks and features actually implemented.

## Published release

Both crates were published as `0.1.0` on 2026-09-28, followed by the GitHub
`v0.1.0` release. Windows-target API documentation is available for
[`fsr-sdk`](https://docs.rs/fsr-sdk/0.1.0/fsr_sdk/) and
[`fsr-sdk-sys`](https://docs.rs/fsr-sdk-sys/0.1.0/fsr_sdk_sys/).
The [publication check](../release/2026-09-29-publication-check.md) records the public
registry entries, timestamps and checksums.

The packaging and release-simulation sections below preserve pre-publication
observations. Their missing registry dependency and pending hosted-documentation
statements describe those historical runs, rather than the current release status.

## Local setup and checks

Use a stable Rust toolchain supporting the workspace's Rust 2024 edition and
resolver 3, with rustfmt and Clippy available. No minimum Rust version is declared
in the manifests yet. Python is not required to build either crate.

Run from the repository root, selecting checks appropriate to the change:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --no-default-features --locked
cargo doc --workspace --no-deps --locked
```

Add `--offline` to Cargo build, test, lint, and documentation commands when all
required dependencies are cached. Keep `--locked` for validation; intentional
dependency changes may require updating the lockfile separately.

These commands were checked on macOS on 2026-09-16 with cached dependencies.
Clippy, tests, no-default-features checking, and documentation generation passed.
That historical pass established compilation only. The common ABI now has
Rust and paired C++ checks; the loader adds independent fixture tests and an
opt-in AMD DLL load test. CI runs the ordinary checks on Ubuntu and Windows
x64/MSVC.

### Continuous integration

The [CI workflow](../../.github/workflows/ci.yml) runs on pull requests, pushes to
`main`, and manual dispatch. Ubuntu checks formatting, Clippy, ordinary tests,
the no-default-features build, and documentation with warnings denied. Windows
x64/MSVC runs Clippy, ordinary tests, the no-default-features build, and
documentation with warnings denied so the Windows/DX12 public API is checked. The
shared Rust setup installs stable Rust with the required components and restores
a Cargo cache. Validation uses the lockfile.

The ordinary test command runs Rust-side ABI checks and local fixture tests. It
skips the ignored tests that require AMD DLLs or a DX12 device. CI does not
acquire the SDK, compile the paired native C++ header checks, load an AMD
runtime, or execute GPU work. Keep those forms of evidence separate from a
passing CI result. The `vulkan` feature deliberately fails compilation, so CI
does not use `--all-features`. No MSRV is declared yet.

### Consumer API documentation and example

`fsr-sdk` warns on missing public documentation. CI denies those warnings through
Clippy and Rustdoc on Windows as well as the host job. The runtime/upscaler API
is compiled only for Windows with `dx12`; host-only macOS/Linux documentation
does not include those modules. docs.rs uses `x86_64-pc-windows-msvc` as the
default documentation target, with default features and no `--all-features`.

The explicit public acquisition entry point is `runtime::Runtime::load_unchecked`.
Its name marks caller-established runtime trust and native integration obligations;
path canonicalization and required-export checks still run. See its Rustdoc for
the complete safety contract.

With the Windows standard library target already installed, these commands can
check the full public docs and example from a non-Windows host:

```sh
cargo check -p fsr-sdk --examples --target x86_64-pc-windows-msvc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --locked
```

The [`dx12_upscale` example](../../crates/fsr-sdk/examples/dx12_upscale.rs) contains
creation and first-frame recording functions that accept renderer-owned DX12
objects. Its executable only prints guidance, and its Windows functions are
gated by `windows` and `dx12`. The Windows CI `--all-targets` Clippy check compiles
those functions. Cross-checking them or building Rustdoc establishes compilation
only, not DLL loading, GPU execution or a successful hosted docs.rs build.

Local validation on 2026-09-28 used macOS ARM64, Rust 1.98.1 and the installed
Windows x64/MSVC standard library, with Cargo checks locked/offline:

- The local setup checks above passed; Clippy checked all targets with warnings
  denied. Host tests passed 10 wrapper tests, five sys tests and two doctests.
  The host doctests omit their Windows-only bodies.
- Windows-target workspace `check --all-targets`, Clippy `--all-targets -- -D
  warnings`, and Rustdoc with `RUSTDOCFLAGS="-D warnings"` passed. The two full
  crate-root doctest bodies, including their hidden setup, also compiled as
  temporary Windows-target examples. No missing-docs warnings remained.
- `cargo check -p fsr-sdk --examples --no-default-features --target
  x86_64-pc-windows-msvc --locked --offline` passed, checking the example's feature
  gate. Host and Windows-target Rustdoc both built with warnings denied.
- `cargo package -p fsr-sdk --list --allow-dirty --locked --offline` included the
  example, `LICENSE` and `LICENSE-AMD`. Full archive creation with `--no-verify`
  failed because `fsr-sdk-sys` was absent from the local crates.io index/cache;
  archive inspection was pending at that point and was completed in the later
  [crate packaging pass](#crate-packaging). Publish dry-run validation remained
  a separate release step.

These were compilation/fixture checks, not new Windows execution, hosted docs.rs
validation or native GPU evidence. The existing native records are unchanged.

### Crate packaging

Local `0.1.0` package hardening on 2026-09-28 used macOS ARM64, Rust/Cargo 1.98.1,
the locked dependencies and the installed Windows x64/MSVC standard library.

- Both descriptions identify the selected Windows/DX12 scope. Keywords and
  categories were checked against the crates.io metadata rules and category
  list. No project MSRV is declared and no dependency versions changed.
- Cargo detects each package's `README.md`; an explicit `readme` field is not
  required. The omitted `documentation` field lets crates.io link to docs.rs
  when the published documentation becomes available.
- Both docs.rs configurations select `x86_64-pc-windows-msvc` with `targets = []`.
  `fsr-sdk` keeps its default `dx12`; `fsr-sdk-sys` explicitly requests `dx12`.
  Neither requests all features. The raw crate's documentation was built
  separately with `dx12`, without relying on workspace feature unification.
- `tests/**/*.ps1` is excluded from both packages because those runners require
  the complete repository checkout. All Rust/C++ source, Rust tests, fixtures
  and the wrapper's DX12 integration example remain included.
- Both packages retain `MPL-2.0` metadata, a byte-identical copy of the root
  `LICENSE`, and the retained AMD MIT notice in `LICENSE-AMD`. The package
  READMEs distinguish original code from header-derived declarations. No
  licensing-policy change or new maintainer decision was needed for this pass.

Package commands run from the repository root:

```sh
cargo package --list -p fsr-sdk-sys --allow-dirty --locked --offline
cargo package --list -p fsr-sdk --allow-dirty --locked --offline
cargo package --workspace --allow-dirty --locked --target-dir target/package-hardening --config net.retry=0
cargo package --workspace --allow-dirty --locked --target x86_64-pc-windows-msvc --target-dir target/package-hardening --config net.retry=0
```

Both workspace packaging commands passed, including Cargo's build verification
of each extracted archive. They used Cargo's temporary local registry for the
unpublished workspace dependency and required crates.io index access; they did
not publish anything. Attempts with `--offline`, including a fresh target
directory, created temporary archives but failed wrapper verification with
Cargo's internal `no hash listed for fsr-sdk-sys v0.1.0` error. The index-enabled
passes completed without changing the workspace lockfile.

The inspected archives are in `target/package-hardening/package/`:

| Package archive | Files | Compressed bytes | Compressed size |
|---|---:|---:|---:|
| `fsr-sdk-sys-0.1.0.crate` | 28 | 31,268 | 30.54 KiB |
| `fsr-sdk-0.1.0.crate` | 24 | 53,601 | 52.34 KiB |

Actual archive entries matched the package listings. Source, fixtures, examples
and notices matched their repository bytes. Both archives include Cargo's
normalized manifest, original manifest, lockfile and VCS metadata. No SDK,
runtime DLL, executable, PDB, build output, experiment output, credentials or
local machine configuration was included. Package-local README links resolve
within the archives; repository documentation links are already public absolute
URLs and required no correction.

The normalized wrapper manifest retains the registry dependency
`fsr-sdk-sys = "0.1.0"` with default features disabled, while the development
manifest retains both `path` and `version`. Publish `fsr-sdk-sys 0.1.0` before
`fsr-sdk 0.1.0`; local packaging does not establish registry name ownership or
permission to publish.

Other checks passed with `--locked --offline` where applicable:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --no-default-features --target x86_64-pc-windows-msvc --locked --offline
RUSTDOCFLAGS="-D warnings" cargo doc -p fsr-sdk --no-deps --target x86_64-pc-windows-msvc --locked --offline
RUSTDOCFLAGS="-D warnings" cargo doc -p fsr-sdk-sys --no-deps --features dx12 --target x86_64-pc-windows-msvc --locked --offline
```

Host tests passed 10 wrapper tests, five sys tests and two doctests; the doctests
omit Windows-only bodies on macOS. The generated Windows HTML contains the
public runtime/upscaler API and the raw DX12/loader API. The negative check
`cargo check --workspace --all-features --locked --offline` failed intentionally
with the Vulkan diagnostic.

Both crates can proceed to release simulation/publish dry-run. These
`--allow-dirty` development archives do not verify final release contents,
hosted docs.rs builds, Windows execution, AMD runtime loading or GPU behavior.
Registry publication, temporal verification and public-history work remain
separate release steps.

### Release simulation

The local `0.1.0` release simulation on 2026-09-28 used macOS ARM64,
Rust/Cargo 1.98.1 and the already installed `x86_64-pc-windows-msvc` target.
The initial worktree was clean at commit
`c5b80852b3e5d2cdba22b7d37c18f96e69f1ceef`, tree
`f67ad5d90e03bc9a17cd54cb38e248887421f672`. Both crate versions were `0.1.0`;
the development dependency retained `path = "../fsr-sdk-sys"`,
`version = "0.1.0"` and `default-features = false`.
Only repository documentation changed during this pass. The generated VCS
records identify that commit and contain no dirty-package flag. No version,
dependency, API, feature or licensing change was needed.

#### Archives and manifests

New archives were generated in `target/release-simulation/package/`, replacing
neither the earlier development archives nor their historical record:

| Archive | Files | Compressed bytes | SHA-256 |
|---|---:|---:|---|
| `fsr-sdk-sys-0.1.0.crate` | 28 | 31,261 | `5e4271480bafc6c84754c6e09937fd59024f84e665c0c7ce352275908a8491b2` |
| `fsr-sdk-0.1.0.crate` | 24 | 53,591 | `5ac3c568a1ef229644bdd271032872fc30321ca83c1727c822b3f8db31549862` |

Archive entries exactly matched the intended crate files plus Cargo's generated
`Cargo.toml.orig`, `Cargo.lock` and `.cargo_vcs_info.json`. Rust/C++ source,
fixtures/tests, READMEs, both license files and the wrapper's `dx12_upscale`
example were present. Checkout-dependent PowerShell runners stayed excluded.
Source, example and notice bytes matched the repository files. There were no
unexpected or missing files, duplicate/escaping paths, SDK checkout, runtime
DLLs, executables, PDBs, build output or local configuration. Inventory and
content inspection found no credentials or private machine paths. No generated
archive was manually modified.

The normalized wrapper manifest contains:

```toml
[dependencies.fsr-sdk-sys]
version = "0.1.0"
default-features = false
```

There is no dependency path or inherited workspace metadata in that manifest.
The wrapper retains default `dx12`; the sys crate retains no default features.
Both retain the explicit unsupported Vulkan sentinel. Packaged names, versions,
descriptions, public repository URL, detected README, keywords and categories
match the current manifests. Documentation remains implicit through docs.rs.
Package-local README license links resolve; repository documentation links are
absolute public URLs. No stale pre-dispatch description was found. Both packages
retain `MPL-2.0` metadata, the root `LICENSE` bytes, AMD's retained MIT notice in
`LICENSE-AMD`, and the pinned SDK header provenance in their READMEs. This check
preserved the existing licensing decision and found no new licensing blocker.

#### Package and publish dry-runs

Commands were run from the repository root with normal build verification and
crates.io index access:

```sh
cargo package -p fsr-sdk-sys --locked --target-dir target/release-simulation --config net.retry=0
cargo package -p fsr-sdk --locked --target-dir target/release-simulation --config net.retry=0
cargo package --workspace --locked --target-dir target/release-simulation --config net.retry=0
cargo publish --dry-run -p fsr-sdk-sys --locked --target-dir target/release-simulation --config net.retry=0
cargo publish --dry-run -p fsr-sdk --locked --target-dir target/release-simulation --config net.retry=0
cargo publish --dry-run --workspace --locked --target-dir target/release-simulation --config net.retry=0
```

| Command selection | Result |
|---|---|
| Sys package | Exit 0; archive created and extracted build verified |
| Wrapper package alone | Exit 101; unpublished sys package absent from crates.io |
| Workspace package | Exit 0; both archives created and extracted builds verified |
| Sys publish dry-run | Exit 0; `warning: aborting upload due to dry run` |
| Wrapper publish dry-run alone | Exit 101; same unpublished sys dependency failure |
| Workspace publish dry-run | Exit 0; both extracted builds verified, sys then wrapper uploads both aborted |

The two individual wrapper failures reported:

```text
no matching package named `fsr-sdk-sys` found
location searched: crates.io index
```

The successful joint dry-run used Cargo's temporary registry for the unpublished
workspace packages. Together with the normalized manifest and isolated registry
consumer results below, this identifies the missing registry dependency as the
individual wrapper failure; no package defect remained. No `--no-verify` or
`--allow-dirty` flag was used. Initial sandbox DNS failures were retried with
index access and did not require changing the packages.

#### Isolated registry consumers and packaged checks

Two temporary consumers were created outside the repository, each with its own
workspace and only a versioned dependency:

```toml
# Wrapper consumer dependency
fsr-sdk = "=0.1.0"

# Separate raw consumer dependency
fsr-sdk-sys = { version = "=0.1.0", features = ["dx12"] }
```

They used a fresh temporary `CARGO_HOME` and target directory. A temporary local
registry held byte-identical copies of the two new archives and Cargo's generated
index entries, plus the 17 third-party archives/entries selected by the unchanged
workspace lockfile. Third-party bytes were taken from the existing Cargo cache;
their SHA-256 values matched both index entries and lockfile checksums. A local
`.cargo/config.toml` replaced `crates-io` with that registry using
`[source.crates-io] replace-with = "release-simulation"` and
`[source.release-simulation] local-registry = "<temporary registry directory>"`.
There were no path dependencies, patches or references to the original workspace
in either consumer. No toolchain or new tooling was installed.

In each consumer directory, these commands passed:

```sh
cargo metadata --format-version=1 --offline
cargo check --locked --offline
cargo check --locked --offline --target x86_64-pc-windows-msvc
```

Metadata identified FSR dependencies as registry packages extracted under the
temporary Cargo home. Consumer lockfiles recorded the exact archive checksums
above, each workspace contained only its consumer, and no package manifest path
referred to the original repository. The resolved feature graph omitted Vulkan.
The wrapper consumer compiled the host error API and, for Windows, the public
imports and construction/dispatch calls below.

This historical consumer excerpt predates [D012](../adr/D012.md). Current callers
use `UpscalerOptions::new(max_render_size, max_upscale_size)` in place of the
options literal; see the current [DX12 example](../../crates/fsr-sdk/examples/dx12_upscale.rs).

```rust
use fsr_sdk::{
    error::Error,
    runtime::Runtime,
    upscaler::{Dimensions, ID3D12Device, UpscaleDispatch, Upscaler, UpscalerOptions},
};

pub fn create(runtime: &Runtime, device: &ID3D12Device) -> Result<Upscaler, Error> {
    Upscaler::new(runtime, device, UpscalerOptions {
        max_render_size: Dimensions::new(320, 180)?,
        max_upscale_size: Dimensions::new(640, 360)?,
    })
}

/// # Safety
/// The renderer must uphold Upscaler::dispatch's resource and GPU obligations.
pub unsafe fn record(upscaler: &mut Upscaler, frame: UpscaleDispatch<'_>) -> Result<(), Error> {
    unsafe { upscaler.dispatch(frame) }
}
```

The raw consumer compiled `api::ffxApiHeader` on the host and additionally
`dx12::ffxCreateBackendDX12Desc` and `loader::FfxLibrary` for Windows. These were
compile checks only; neither consumer loaded AMD binaries or executed GPU work.

Both archives were also extracted into a separate `packages/` directory under
that temporary environment. For each extracted manifest, `cargo test`,
Windows `cargo check --all-targets`, and host/Windows `cargo check
--no-default-features` passed with `--manifest-path`, `--locked` and `--offline`.
Packaged host tests passed 10 wrapper tests, five sys tests and two wrapper
doctests; their Windows-only bodies are omitted on macOS.

With `RUSTDOCFLAGS="-D warnings"` and the same isolated registry, these passed:

```sh
cargo doc --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --no-deps --locked --offline --target x86_64-pc-windows-msvc
cargo doc --manifest-path packages/fsr-sdk-sys-0.1.0/Cargo.toml --no-deps --features dx12 --locked --offline --target x86_64-pc-windows-msvc
```

The generated HTML contained `Runtime`, `Upscaler`, `UpscalerOptions`,
`UpscaleDispatch`, the raw DX12 backend descriptor and `FfxLibrary`. Both packaged
docs.rs configurations select Windows/MSVC with `targets = []`; the wrapper
uses default DX12 and the raw package explicitly requests it. Neither enables
Vulkan/all features. An isolated negative `cargo check --manifest-path
packages/fsr-sdk-0.1.0/Cargo.toml --all-features --locked --offline` returned 101
with only the intentional Vulkan compile error. It is not a success gate.

The original workspace also passed formatting, host/Windows all-target checks
and Clippy with warnings denied, host tests (the same 10 + 5 + 2), host/Windows
no-default-feature checks and host/Windows Rustdoc with warnings denied. Cargo
validation used `--locked --offline`; the concrete command forms are in
[local setup](#local-setup-and-checks), [crate packaging](#crate-packaging) and
the Windows cross-check instructions below. The workspace lockfile and crate
source/manifests remained unchanged.

#### Decision and limits

Both `fsr-sdk-sys 0.1.0` and `fsr-sdk 0.1.0` are **READY FOR RELEASE SEQUENCE**
for the local Cargo/package gate, with sys published before the wrapper.
No concrete packaging, metadata, dependency, documentation or retained-notice
blocker was found. This is not an overall release approval.

No upload, tag, GitHub release, commit or history change was made. Local registry
consumption establishes package resolution and compilation; it does not establish
public crates.io availability, ownership, authentication or acceptance by the
server's additional checks. Local Windows-target Rustdoc approximates docs.rs;
hosted documentation was not tested. Windows binaries were not linked/executed,
and AMD DLL loading, GPU work and temporal behavior were not tested on this Mac.
Existing native records and the separate temporal/public-history release steps
remain unchanged.

### Runtime acquisition API rename

On 2026-09-28, the public Windows/DX12 loader was renamed to
`Runtime::load_unchecked(path)` before `0.1.0`. Its `unsafe` signature, return
type, error mapping, ownership and threading behavior are unchanged. The
existing `# Safety` text was compared with the previous source and is identical.
`_unchecked` names the caller's runtime/provider trust and native integration
obligations; it does not disable path canonicalization or export resolution.
The spelling leaves `load` available for a possible future safe acquisition API,
without promising or providing one. There is no compatibility alias.

Current READMEs, Rustdoc, example and executable test/harness sources use the new
name. D008's current API spelling was updated without changing its acquisition
contract. The old name remains intentionally in the completed M6 baseline in
[MILESTONE_6.md](../MILESTONE_6.md) and these historical native records:

- [M5 Windows native verification](../research/records/2026-09-24-exp-m5-windows-native-verification.md).
- [M6a native dispatch proof](../research/records/2026-09-24-exp-m6a-native-dispatch-proof.md).
- [M6 GBV dispatch stall](../research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md).

Pre-commit validation used macOS ARM64, Rust/Cargo 1.98.1 and the installed
Windows x64/MSVC target. These passed:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline -- -D warnings
cargo check --workspace --no-default-features --target x86_64-pc-windows-msvc --locked --offline
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --locked --offline
```

Host tests passed 10 wrapper tests, five sys tests and two doctests. Windows
Rustdoc exposes `load_unchecked` and no `load` method. In a temporary source
consumer outside the workspace, both complete crate-root doctest bodies
(including hidden setup, with DX12 enabled) and an explicit acquisition function
using `fsr_sdk::runtime::{Runtime, LoadError}` compiled for Windows. Its dependency
pointed at the current crate source; this was not a registry/package recheck.
The temporary checks used `cargo generate-lockfile --offline`, then `cargo check
--examples --lib --locked --offline --target x86_64-pc-windows-msvc`. A separate
negative `cargo check --lib` with the same target and flags rejected the removed
method with E0599, after the positive control passed.
Local documentation links resolved, and the changed repository files contained
no personal absolute paths or local username.

At the pre-commit checkpoint, final package regeneration, publish dry-runs and
consumers of the new archives were pending the maintainer's commit. That
checkpoint made no packages, uploads, tags or commits. The earlier release
simulation above preserves the pre-rename artifacts; the final recheck follows.

#### Final release recheck

After the maintainer committed the rename, the targeted recheck on 2026-09-28
used the same macOS ARM64 host, Rust/Cargo 1.98.1 and installed Windows x64/MSVC
target. The candidate worktree was clean at commit
`b480ae43af7ddd8630615d3382bde8a75a4c4bac`, tree
`36e3692d100e9a401a6ec8de76a43adf3aeffe66`. New archives are in
`target/runtime-load-rename-release/package/`; earlier artifacts were preserved.

| Archive | Files | Compressed bytes | SHA-256 |
|---|---:|---:|---|
| `fsr-sdk-sys-0.1.0.crate` | 28 | 31,260 | `bab1361353ccaeb2fa7e78e73808b1b1c616569da919cbc6603cb4eaea52ac36` |
| `fsr-sdk-0.1.0.crate` | 24 | 53,730 | `9efe82186a841302a0f87eba1f97e2b1865d55caa84296792f82177e5158d725` |

Both generated VCS records identify the clean candidate. Inventory matched the
intended crate files and Cargo's generated files. Source, README, example and
notice bytes matched the committed files; no archive was manually edited.
The normalized wrapper dependency remains version `0.1.0`, with
`default-features = false` and no path. Its packaged source exposes only
`Runtime::load_unchecked` as the public explicit acquisition method.

These commands ran from the repository root with normal build verification and
crates.io index access:

```sh
cargo package -p fsr-sdk-sys --locked --target-dir target/runtime-load-rename-release --config net.retry=0
cargo package -p fsr-sdk --locked --target-dir target/runtime-load-rename-release --config net.retry=0
cargo package --workspace --locked --target-dir target/runtime-load-rename-release --config net.retry=0
cargo publish --dry-run -p fsr-sdk-sys --locked --target-dir target/runtime-load-rename-release --config net.retry=0
cargo publish --dry-run -p fsr-sdk --locked --target-dir target/runtime-load-rename-release --config net.retry=0
cargo publish --dry-run --workspace --locked --target-dir target/runtime-load-rename-release --config net.retry=0
```

Sys packaging and its publish dry-run passed (exit 0). Workspace packaging and
the joint publish dry-run passed (exit 0), including both extracted archive
builds. Both uploads stopped with `warning: aborting upload due to dry run`.
Individual wrapper packaging and its dry-run again returned 101 solely because
`fsr-sdk-sys 0.1.0` was absent from crates.io:

```text
no matching package named `fsr-sdk-sys` found
location searched: crates.io index
```

The joint commands supplied that dependency through Cargo's temporary registry. Neither
`--allow-dirty` nor `--no-verify` was used.

Two fresh consumers outside the repository used only versioned dependencies:
`fsr-sdk = "=0.1.0"` and, separately,
`fsr-sdk-sys = { version = "=0.1.0", features = ["dx12"] }`. A fresh temporary
Cargo home and target directory used a local registry source replacement,
containing byte-identical copies of the new archives, Cargo's generated FSR
index entries and the same 17 cached third-party versions from the unchanged
workspace lockfile. There were no path dependencies or patches. Metadata
confirmed independent consumer workspaces, registry source IDs, extracted
manifest paths under the temporary Cargo home, exact archive checksums in both
consumer lockfiles and no Vulkan in the feature graph. No manifest path referred
to the original repository.

In each consumer directory, these passed:

```sh
cargo metadata --format-version=1 --offline
cargo check --locked --offline
cargo check --locked --offline --target x86_64-pc-windows-msvc
```

The wrapper consumer compiled the host error API and the Windows construction
and dispatch signatures. It also compiled this acquisition function against
the packaged registry dependency:

```rust
use fsr_sdk::runtime::{LoadError, Runtime};
use std::path::Path;

/// # Safety
/// The caller must uphold Runtime::load_unchecked's trust and integration contract.
pub unsafe fn acquire(path: &Path) -> Result<Runtime, LoadError> {
    unsafe { Runtime::load_unchecked(path) }
}
```

The raw consumer checked `api::ffxApiHeader` on the host and additionally
`dx12::ffxCreateBackendDX12Desc` and `loader::FfxLibrary` for Windows.
With both new archives separately extracted under `packages/` in the same
isolated environment, these also passed:

```sh
cargo check --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --examples --locked --offline --target x86_64-pc-windows-msvc
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --no-deps --locked --offline --target x86_64-pc-windows-msvc
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path packages/fsr-sdk-sys-0.1.0/Cargo.toml --no-deps --features dx12 --locked --offline --target x86_64-pc-windows-msvc
```

Packaged Windows Rustdoc contains `load_unchecked` and no `load` method, plus
the wrapper upscaler and raw DX12/loader pages. All nine targeted isolated checks
passed. Final archive checksums and registry copies remained unchanged. The
pre-commit workspace checks above cover the same committed source bytes; the
full licensing/package-design audit was not repeated. Current consumer
references use the new name, with only the four historical occurrences listed
above preserved. Changed repository documentation passed local-link and privacy
checks, with no personal absolute paths or local username.

Both **fsr-sdk-sys 0.1.0** and **fsr-sdk 0.1.0** remain **READY FOR RELEASE
SEQUENCE** for the local Cargo/package gate, in sys-then-wrapper order. No upload,
tag, release or agent-created commit occurred. Public registry availability,
ownership, authentication, server acceptance and hosted documentation remain
unverified. These Mac checks compiled Windows code without linking or executing
it; AMD DLL loading, GPU work and temporal behavior were not exercised. The
post-check edits affect only this report and the roadmap, outside both packages.

### Options construction API and native target policy

On 2026-09-28, [D012](../adr/D012.md) changed options construction to private fields,
an explicit constructor and getters, retaining `Debug`/`Clone` and removing
`Copy`/`PartialEq`/`Eq`. Native descriptors and zero flags are unchanged.
[SCOPE.md](../../SCOPE.md#platform-and-compatibility-boundaries) now explicitly owns
the distinction between exposed Windows/DX12 APIs and the supported native target.

Validation used macOS ARM64, Rust 1.98.1, cached dependencies and the installed
Windows x64/MSVC standard library, with Cargo checks locked/offline. Formatting,
host and Windows-target all-targets Clippy with warnings denied, host tests,
host and Windows-target no-default-features checks, and host/Windows Rustdoc
with warnings denied passed. Host tests passed 10 wrapper tests, five sys tests
and two doctests; the host doctests omit their Windows-only bodies. Both full
crate-root examples and external use of the options getters and `Clone` also
compiled in a temporary Windows-target consumer. These checks did not link or
execute Windows code, load AMD DLLs or run GPU work. At this API-change checkpoint,
the earlier release simulations predated the options change; archive rechecking
was completed in the following pass.

#### Post-D012 release simulation

On 2026-09-28, the release simulation was repeated on macOS ARM64 with
Rust/Cargo 1.98.1 and the installed `x86_64-pc-windows-msvc` target. The candidate
was clean at commit `8248672e0c3a647ea23cb738cf1ac6be1116114a`, tree
`d5c62c8070399979b63d8755a41a385ae5921dfc`. Both crates remain `0.1.0`.
The development wrapper dependency retains `path = "../fsr-sdk-sys"`,
`version = "0.1.0"` and `default-features = false`. Source and D012 agree on
private options fields, `new`, borrowed dimension getters and `Debug`/`Clone`.
`Runtime::load_unchecked` remains the public acquisition method.

Fresh artifacts are in `target/post-d012-release/package/`; previous simulation
artifacts were preserved:

| Archive | Files | Compressed bytes | SHA-256 |
|---|---:|---:|---|
| `fsr-sdk-sys-0.1.0.crate` | 28 | 31,564 | `b5c43f8806f92b8f620d6ec3a6d5203737ca2f6f51514c6344e834414d6916b5` |
| `fsr-sdk-0.1.0.crate` | 24 | 54,232 | `6bf05c402e9554d67cec60b3d26de778b881398b11fac220d88737df1739c271` |

Both generated VCS records identify the clean post-D012 commit. Inventory matched
the intended crate files plus Cargo's generated manifest, lockfile and VCS
metadata, with the same entry sets as the previous simulation. Rust sources,
READMEs, licenses, intended tests/fixtures and the wrapper example are present;
checkout-dependent PowerShell runners remain excluded. Source and notice bytes
match the committed files, including byte-identical root-license copies and the
retained AMD notice. There are no duplicate or escaping paths, SDK checkout,
AMD DLLs, EXE/PDB files, build output or local caches/configuration. The focused
content scan found no credentials or personal absolute paths.

Both normalized Cargo manifests are byte-identical to the previous simulation's
normalized manifests: metadata and features are unchanged. The wrapper registry
dependency contains `version = "0.1.0"` and `default-features = false`, with no
path. No package, API, dependency, feature or licensing correction was needed.

Package and publish commands ran from the repository root with index access and
normal extracted-build verification:

```sh
cargo package -p fsr-sdk-sys --locked --target-dir target/post-d012-release --config net.retry=0
cargo package -p fsr-sdk --locked --target-dir target/post-d012-release --config net.retry=0
cargo package --workspace --locked --target-dir target/post-d012-release --config net.retry=0
cargo publish --dry-run -p fsr-sdk-sys --locked --target-dir target/post-d012-release --config net.retry=0
cargo publish --dry-run -p fsr-sdk --locked --target-dir target/post-d012-release --config net.retry=0
cargo publish --dry-run --workspace --locked --target-dir target/post-d012-release --config net.retry=0
```

| Selection | Result |
|---|---|
| Sys package | Exit 0; archive and extracted build verified |
| Wrapper package alone | Exit 101; unpublished sys dependency absent from crates.io |
| Workspace package | Exit 0; both archives and extracted builds verified |
| Sys publish dry-run | Exit 0; upload aborted due to dry run |
| Wrapper publish dry-run alone | Exit 101; same missing sys dependency |
| Workspace publish dry-run | Exit 0; both extracted builds verified, sys then wrapper uploads aborted |

The individual wrapper commands reported only the known registry condition:

```text
no matching package named `fsr-sdk-sys` found
location searched: crates.io index
```

Joint commands used Cargo's temporary registry for the unpublished dependency.
Each simulated upload stopped with `warning: aborting upload due to dry run`.
Neither `--allow-dirty` nor `--no-verify` was used. An initial sandbox DNS failure
was retried with index access.

Two independent consumers outside the repository used only versioned
dependencies: `fsr-sdk = "=0.1.0"` and, separately,
`fsr-sdk-sys = { version = "=0.1.0", features = ["dx12"] }`. A fresh temporary
Cargo home and target directory used the established local registry replacement
for `crates-io`. That registry contained byte-identical copies of the new FSR
archives and Cargo-generated index entries, plus the same 17 cached third-party
versions from the unchanged workspace lockfile. Index, archive and lockfile
checksums agreed. There were no path dependencies or patches. Metadata confirmed
registry source IDs, FSR manifests extracted under the temporary Cargo home,
the exact new archive checksums, one consumer per workspace, no manifest paths
into the original repository and no Vulkan feature.

In each consumer directory, these passed:

```sh
cargo metadata --format-version=1 --offline
cargo check --locked --offline
cargo check --all-targets --locked --offline --target x86_64-pc-windows-msvc
```

The wrapper checked the host error API and Windows imports from
`fsr_sdk::runtime::{LoadError, Runtime}` and
`fsr_sdk::upscaler::{Dimensions, ID3D12Device, Upscaler, UpscalerOptions,
UpscaleDispatch}`. Its Windows creation function constructs options with
`UpscalerOptions::new(Dimensions::new(320, 180)?, Dimensions::new(640, 360)?)`,
uses both getters, `Debug` and explicit `Clone`, and passes the options to
`Upscaler::new`. Acquisition calls `unsafe { Runtime::load_unchecked(path) }`;
dispatch forwarding retains its unsafe renderer contract. The raw consumer
checks `api::ffxApiHeader` on the host and the DX12 backend descriptor and
`loader::FfxLibrary` on Windows. The wrapper consumer also compiles both complete
packaged crate-root `no_run` examples, including hidden setup, as Windows
examples with its local `dx12` feature enabled.

With the new archives separately extracted under `packages/` in that isolated
environment, these passed:

```sh
cargo check --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --examples --locked --offline --target x86_64-pc-windows-msvc
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path packages/fsr-sdk-0.1.0/Cargo.toml --no-deps --locked --offline --target x86_64-pc-windows-msvc
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path packages/fsr-sdk-sys-0.1.0/Cargo.toml --no-deps --features dx12 --locked --offline --target x86_64-pc-windows-msvc
```

All nine isolated checks passed. Packaged HTML exposes `load_unchecked` with no
`load` method, and the options constructor/getters and `Debug`/`Clone` without
`Copy`/`PartialEq`/`Eq`. Archive hashes and registry copies remained unchanged.

The workspace also repeated all ten command forms recorded under
[runtime acquisition API rename](#runtime-acquisition-api-rename): formatting,
host/Windows all-targets checks and Clippy with warnings denied, host tests,
host/Windows no-default-feature checks and host/Windows Rustdoc with warnings
denied. All passed with locked/offline Cargo validation. Host tests passed
10 wrapper tests, five sys tests and two doctests; Windows-only doctest bodies
were checked separately as described above. Vulkan/all features remain an
intentional compile error and were not used as a success gate.

The current-reference scan found no stale consumer API use. The options literal
in the earlier release simulation remains explicitly historical; other curly
options matches are type/impl declarations or a helper return type. The four old
loader-name occurrences in the M6 baseline and M5/M6 native records remain
historical, as listed in the rename record. Repository and packaged-file privacy
scans found no personal absolute paths or local username. The report and roadmap
passed local-link and whitespace checks.

Both **fsr-sdk-sys 0.1.0** and **fsr-sdk 0.1.0** are **READY FOR RELEASE SEQUENCE**
for the local Cargo/package gate, with sys first. Public registry availability,
ownership, authentication, server acceptance and hosted docs.rs remain unverified.
Windows code was compiled without linking or execution; no AMD runtime, GPU or
temporal verification ran. No upload, tag, release, commit or history change was
made. Only this report and the roadmap changed, outside both crate packages.

#### Final post-Windows artifact recheck

The [2026-09-28 final artifact record](../release/2026-09-28-final-artifact-recheck.md)
rechecks the clean committed candidate after the final Windows verification.
Fresh sys and wrapper archives, combined sys-then-wrapper package/publish
dry-runs, isolated registry consumers, Windows-target checks and the standard
release checks passed. The standalone wrapper commands retain only the expected
unpublished sys dependency condition. Both crates remain **READY FOR RELEASE
SEQUENCE**; the record identifies the source state and exact archive hashes
suitable as the reference point for **RELEASE CANDIDATE FREEZE**. It also records
the intentional test-source inclusions, unchanged normalized metadata, privacy
scan and documentation-only follow-up outside both packages. No upload, tag,
history reconstruction, production change or GPU rerun occurred.

### Common ABI verification

The [ABI coverage inventory](../ABI_COVERAGE.md) distinguishes raw declarations,
their verification, and wrapper support for each selected slice.

`crates/fsr-sdk-sys/tests/abi.rs` checks return constants, nullable function-pointer
layouts, exact Rust types/signatures, and Windows x64 MSVC struct sizes, alignment
and field offsets. Fixed numeric struct expectations are gated to that target;
tests on other hosts do not establish Windows layout compatibility.

The paired `crates/fsr-sdk-sys/tests/native_abi.cpp` checks the same expectations
against the local SDK v2.3.0 `Kits/FidelityFX/api/include/ffx_api.h`, including
native declarations and explicit `__cdecl` function-pointer signatures. It is
optional, compile-only, and separate from Cargo: no SDK download, linking, build
script or runtime invocation is involved. Keep the paired expectations aligned
when changing the ABI slice.

From the repository root in an **x64 Native Tools Command Prompt for VS 2022**:

```bat
if not exist target\abi-check mkdir target\abi-check
cl /nologo /std:c++17 /W4 /WX /c /Iexternal\FidelityFX-SDK\v2.3.0\Kits\FidelityFX\api\include crates\fsr-sdk-sys\tests\native_abi.cpp /Fotarget\abi-check\native_abi.obj
```

Verified on 2026-09-20 with Rust 1.98.1 (`x86_64-pc-windows-msvc`) and
VS 2022 Build Tools: all three ABI tests and the C++ header check passed.
This establishes the tested common ABI declarations, not runtime or GPU support.
The [verification record](../research/records/2026-09-20-exp-common-abi-verification.md)
preserves the tested revision, local-header fingerprint, observations and limits;
the [API/safety synthesis](../research/api-and-safety.md) maintains current findings
and remaining questions.

### Loader verification

On Windows with `dx12`, ordinary workspace tests compile small Rust fixture DLLs
using `rustc` from PATH (matching the test executable's toolchain/architecture).
They need no SDK, AMD runtime or GPU. Fixtures are emitted into unique directories
under the OS temporary directory, named `fsr-sdk-loader-<pid>-<counter>`.
Four tests cover a missing file, an invalid DLL, each of the five missing exports
(including partial-load cleanup), and argument/return forwarding plus unloading
after moving and dropping the owner. Two compile-fail doctests check owner
borrowing and private pointer access. No AMD API functions run in these tests.

The separate ignored integration test loads a trusted DLL, resolves its five
exports, and drops it without invoking any API function. From PowerShell at the
repository root, explicitly opt in with:

```powershell
$env:FSR_SDK_TEST_DLL = (Resolve-Path external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll).Path
cargo test -p fsr-sdk-sys --features dx12 --test loader --locked --offline loads_amd_runtime_without_api_calls -- --ignored --exact
```

This executes DLL initialization/termination code. Only supply a trusted,
ABI-compatible SDK DLL and dependencies. The environment variable is solely a
test input, not a library discovery mechanism. The local v2.3.0 DLL passed on
2026-09-20; details and limitations are in the
[loader record](../research/records/2026-09-20-exp-explicit-dll-loader.md).

### Native query smoke test

`tests/query.rs` adds one ignored count-only upscaler version query. It passes a
null context and device and checks `FFX_API_RETURN_OK` plus a positive count.
No contexts or GPU resources are created. The tested local setup returned 2.
The absolute path to the original SDK loader alone returned `NO_PROVIDER`;
staging the test executable with the loader and upscaler DLLs succeeded.
Use the exact PowerShell build/staging/run commands in the
[query record](../research/records/2026-09-20-exp-version-count-query.md#reproduction).
This is local test deployment, not automatic runtime discovery or installation.
The ordinary Cargo test run skips it. The paired ABI checks now include the
56-byte query descriptor, all field offsets/types and its two required tags.

### M4 context ownership checks

`fsr-sdk` now contains a private terminal lifecycle owner and Windows/DX12
upscaler creation state. Ordinary wrapper tests use a locally compiled fixture
library and no AMD runtime. On macOS a test-only runtime adapter exercises the
same owner; on Windows/DX12 the fixtures go through the production FfxLibrary.
Counters cover cleanup/retention of state, device and runtime ownership; Windows
also checks fixture DLL residency. The Mac adapter's drop signal does not prove
that dyld actually unloads the fixture. Compile checks include a positive control
before rejecting use after consuming destroy and private handle access.

The concrete Windows tests check stable descriptor addresses and owner trait
exclusions. New production ABI declarations are paired with the existing
`tests/context_lifecycle/native_abi.cpp`; provider identification remains
instrumentation-only. The ordinary native lifecycle now uses the production
wrapper owner, not a second raw ownership implementation.

On Windows with Rust and MSVC, run ordinary checks with:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1
```

From an **x64 VS developer shell**, with the trusted local SDK v2.3.0 and a
hardware DX12 device, explicitly opt into native verification:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1 -Native
```

The latter also compiles paired C++ ABI checks, builds the helper, stages SDK
DLLs and runs `upscaler::tests::native_lifecycle` in a separate process with a
60-second timeout. Stdout/stderr/status are saved under `target/m4-lifecycle/`.
The current test uses M5's public constructor and two sequential contexts; its
Windows execution is recorded in the
[M5 verification record](../research/records/2026-09-24-exp-m5-windows-native-verification.md).
The earlier M4 version used production
create/destroy policy, identified the provider while the context was live and
submitted no GPU work. Those Windows checks passed on 2026-09-22: 17 ordinary
executable tests, two compile-fail doctests, paired C++ ABI compilation and the
isolated production-owner lifecycle (provider 4.1.1,
RX 9060 XT, driver 32.0.31041.1004). The
[M4 verification record](../research/records/2026-09-22-exp-m4-windows-verification.md)
preserves inputs, results and the process-local PowerShell execution-policy
override needed on this machine. Historical raw-runtime evidence is
preserved in the [lifecycle record](../research/records/2026-09-20-exp-context-lifecycle.md).

The old `crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1` invocation without
switches delegates to this production-owner runner. Its research switches remain
separate from M4 verification.

Pass `-FailureInjection` to the sys research runner for the separate host-allocation
failure experiment. It measures an instrumented successful control, then runs
each observed allocation index and one unreached control in separate processes
with 60-second timeouts. Per-process status and output are retained under
`target/context-lifecycle/failure-N.*.txt`. Child crashes are observations, so
runner success does not imply that all native calls succeeded. Ordinary tests
skip this probe. The tested runtime crashed at all three injected indices;
see the [failure record](../research/records/2026-09-20-exp-create-allocation-failure.md)
for callback balances, reproduction and limits.
`-InputLifetime` likewise retains its separate raw probe, now with a production-
owner successful control. Historical records describe their original runner.

On a non-Windows host with the Windows standard library installed, this checks
Rust code and tests without linking or executing Windows binaries:

```sh
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked -- -D warnings
```

No Rust cross-check substitutes for the native C++/DLL/device checks above.

### M5 public construction checks

M5 adds a public Windows/DX12 `Runtime` and checked `Upscaler` constructor.
The ordinary Windows test suite includes a local fixture for the descriptor
chain, COM AddRef/Release, zero rejection and exceptional dependency retention,
plus shared-runtime sibling lifecycle tests. On Windows with Rust/MSVC, run:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m5.ps1
```

For the trusted local SDK v2.3.0, an x64 VS developer shell and a hardware DX12
device, opt into the two-context native cases (one explicit B destroy, one B
Drop) in separate 60-second processes:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m5.ps1 -Native
```

The runner saves DLL SHA-256 fingerprints, OS/GPU/driver data, native output
and exit status under `target/m5-lifecycle/`. It compiles the existing paired
native ABI check and builds the DX12 helper. The native cases create A and B
sequentially through the public path, drop the public runtime and caller device,
destroy A, query B for provider identity, then clean up B. Module residency is
recorded separately from the Rust runtime ownership assertion. Neither fixtures
nor native lifecycle tests exercise Dispatch or prove universal numerical
provider safety. The ordinary Windows CI suite ran these M5 fixtures successfully
in the [recorded run](../research/records/2026-09-24-exp-m5-windows-ci-verification.md).
Both opt-in native cases passed on the tested Windows x64/MSVC
machine on 2026-09-24; the [verification record](../research/records/2026-09-24-exp-m5-windows-native-verification.md)
includes the exact PowerShell 7 developer-shell command, inputs, outcomes, and
two setup failures before native execution.

### M6a private native dispatch proof

The M6a proof originally checked the selected dispatch ABI, private descriptor
construction, rejection paths and fixture forwarding. The opt-in test uses
the unchanged public M5 constructor, five synthetic DX12 textures, command-list
submission, fence and pitched readback. M6b switched it to the public dispatch
method; the historical M6a record still describes its original private path.
It requires the trusted local SDK v2.3.0 under
`external/FidelityFX-SDK/v2.3.0`, an x64 VS developer shell and a hardware DX12
GPU. From the repository root:

```powershell
pwsh -NoProfile -File crates/fsr-sdk/tests/run-m6.ps1 -Native
```

The runner compiles a paired C++ ABI check against the pinned headers, builds
the DX12 helper, stages the signed loader/provider DLLs beside an isolated
test executable, and records metadata, stdout, stderr and status under
`target/m6a/`. It enables the D3D12 debug layer; `-GpuValidation` additionally
enables GPU-based validation. On the [recorded setup](../research/records/2026-09-24-exp-m6a-native-dispatch-proof.md),
the debug-layer-only run passed with OK dispatch, fence completion, 230,400
finite changed pixels, spatial variation and zero debug errors. Two
GPU-based-validation attempts timed out inside native dispatch before GPU
submission; that mode remains inconclusive. The original M6a run provided
bounded private-path evidence, not an image-quality claim.
The [bounded GBV spike](../research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md)
uses [`run-m6-gbv-spike.ps1`](../../crates/fsr-sdk/tests/run-m6-gbv-spike.ps1):
its no-dispatch DX12 control completed under
GBV, while the provider call consumed CPU and timed out before submission.

### M6b public dispatch verification

The M6a harness now calls the public `unsafe Upscaler::dispatch` route while
retaining its five-resource parity check, submission, fence and pitched readback.
Run `run-m6.ps1 -Native` as above to execute this public route. The ordinary
suite checks the fixed descriptor, inspectable resource and scalar rejection,
public-route COM inspection with controlled DX12 objects, and fixture-driven
error/poison/teardown policy without AMD DLLs or GPU commands. The
[M6b record](../research/records/2026-09-25-m6b-public-dispatch-verification.md)
preserves the native inputs, command, outputs and limits. The method does not
submit or wait; the unsafe caller must keep the context, resources and backing
storage alive until GPU completion and obey D011's failure/abandonment rules.

### Final Windows release verification

`run-m6.ps1 -Native` now runs the existing public-route regression (Gate A) and
`upscaler::m6::native_temporal_smoke` (Gate B) in separate 180-second children.
Gate A retains its stronger sentinel/gradient assertions and explicit destroy.
Gate B uses one runtime/device/upscaler for exactly three analytically moving,
jittered frames, with fresh per-frame resources, serialized fence completion
and normal Drop. Per-frame fencing is this test's strategy, not a wrapper rule.
Both gates require the actually selected hardware adapter to be AMD and check
the ordinary debug layer/device status after context teardown. Fence waits
remain bounded at 60 seconds. GBV stays disabled unless explicitly requested
with the existing `-GpuValidation` diagnostic switch.

Use a unique `-Stage` directory for each verification run to preserve earlier
evidence; omitting it retains the `target/m6a` layout used by the other runners.
The directory contains separate `gate-a.*` and `gate-b.*` output/status files,
source/native input fingerprints, the harness patch, environment data and check
results. Sanitize the local transcript before retaining it in repository records.

The [2026-09-28 final Windows record](../research/records/2026-09-28-exp-final-windows-release-verification.md)
preserves the exact commands, executed state and sanitized evidence: standard
checks, paired ABI compilation, Gate A and all three Gate B frames passed on
the recorded RX 9060 XT/provider 4.1.1 configuration. Every output had 230,400
finite changed pixels, zero debug errors/corruption and no device removal.
Explicit destroy succeeded; post-Drop device/debug observations were clean,
with no claim of a native Drop return code. This establishes bounded temporal
smoke coverage, not temporal reconstruction accuracy or an image-quality oracle.

## Feature and platform boundaries

The supported native target is `x86_64-pc-windows-msvc` with the `dx12` feature.
Windows/DX12 APIs are exposed on other Windows targets, but those targets are
currently unverified and unsupported; compilation may succeed.
[SCOPE.md](../../SCOPE.md#platform-and-compatibility-boundaries) owns this support policy.
Native verification is limited to the SDK, runtime/provider, GPU, driver and
workload configurations recorded in the verification records, including
[M6b](../research/records/2026-09-25-m6b-public-dispatch-verification.md). Target
support does not imply compatibility with every hardware/provider configuration.

`fsr-sdk` enables `dx12` by default and forwards it to `fsr-sdk-sys`. The raw crate
has no default features. Its loader module is gated by Windows and `dx12`.
The `vulkan` feature intentionally raises a compile error; do not use
`--all-features` as the workspace's success check.

Host builds on macOS do not compile the Windows loader. Run the relevant checks
on Windows with DX12 enabled when changing that boundary. Cross-compilation may
check target-specific Rust code, but cannot establish that a DLL loads or a GPU
operation works. The native SDK baseline is `v2.3.0`
([D002](../DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).
No automated SDK acquisition or public Query/Configure operations are implemented.
The raw loader's forwarding methods do not validate native operation contracts.

## FFI safety and verification

For native changes, establish and document the applicable contracts:

- Compare layouts, alignment, integer widths, constants, and calling conventions
  against pinned AMD headers. Add meaningful ABI checks as bindings are added.
- Document unsafe public APIs with caller obligations. Explain unsafe blocks
  with `SAFETY` comments identifying the invariants that justify the operation.
- Keep libraries alive while native functions or objects depend on them. Verify
  context destruction, partial initialization cleanup, and descriptor lifetimes.
- Establish resource states, GPU completion, ownership, and threading guarantees;
  Rust borrowing alone cannot guarantee GPU synchronization or justify `Send`
  and `Sync`.
- Exercise relevant failure paths, including missing libraries or symbols and
  native errors. Preserve native error information.

Use focused tests for deterministic validation, conversions, and ownership logic.
Use native integration checks for ABI and runtime behavior; mocks cover wrapper
logic but cannot validate AMD's runtime. Add a regression test for a fixed bug when
it meaningfully captures the failure.

Report native evidence separately: Windows compilation, runtime loading and symbol
resolution, successful query, context lifecycle, and actual GPU dispatch. For
runtime results, identify the SDK/runtime version, OS, driver, and GPU when
relevant, plus inputs and observed outcomes. Correct output and performance need
their own evidence. If hardware or runtime access is unavailable, state exactly
what remains untested. Bounded investigations use the
[research process](../research/PROCESS.md); ordinary checks need only a clear report.
