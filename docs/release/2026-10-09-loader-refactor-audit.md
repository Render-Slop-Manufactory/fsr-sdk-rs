# Loader refactor audit — 2026-10-09

## Outcome and scope

Accept the loader refactor as implemented. No production correctness or public
API regression was found by source review and the available checks. Native
Windows execution remains unverified in this audit; compilation is not a
substitute. No production code, features, dependencies or tests were changed.

This is a fresh audit of current source, separate from the earlier
[implementation verification](2026-10-09-loader-platform-verification.md).
The worktree was clean before the audit. At audit completion, repository changes
were limited to this report and a navigation link in the earlier record. No new
architectural decision was made; [D015](../adr/D015.md) already accepts the loader
organization and dependency boundary.

## Architecture assessment

Confirmed from source and manifests:

- `loader/mod.rs` contains documentation, a private Windows implementation and
  public reexports. Existing imports remain
  `fsr_sdk_sys::loader::{FfxLibrary, LoadError}`.
- The crate-root gate is `all(windows, feature = "dx12")`. Raw `libloading 0.9.0`
  is optional under `cfg(windows)` and activated by `dx12`. The raw crate has
  empty defaults; the wrapper defaults to DX12 and forwards that feature.
- `windows.rs` handles path resolution, loading, five common C entry points,
  errors and DLL retention. It contains no DX12 device/resource integration.
  `fsr_sdk::runtime::RuntimeState` likewise retains the library rather than a
  graphics device. Context ownership and DX12 resource integration remain in
  their existing wrapper modules.
- [D004](../adr/D004.md), [D007](../adr/D007.md), [D010](../adr/D010.md),
  [D013](../adr/D013.md) and D015 explain acquisition, provider deployment,
  owned errors, threading/ownership and platform boundaries consistently with
  the implementation.

Assessment: requiring Windows and DX12 is appropriate for the only implemented
native runtime/provider profile. Operating system, graphics backend, provider
and dynamic loading are separate concerns; this gate selects their currently
supported combination, not an equivalence between Windows and DX12 or Linux
and Vulkan.

Future consideration: a Windows/Vulkan runtime with the same five-export ABI
could reuse the Windows loading mechanics without rewriting them solely to
separate graphics concerns. Its feature gates, provider compatibility, resource
and safety contracts would need independent review. No actual future runtime
has been verified, so compatibility or the absence of other changes cannot be
promised. No generic loader trait, Linux loader or runtime hierarchy is warranted
by this audit. Vulkan remains intentionally unsupported.

## Loader correctness and preserved contracts

A direct comparison with the pre-refactor implementation found byte-identical
contents after replacing its single module-description line. Raw ABI source,
runtime/context/upscaler implementation, native tests, lockfile and CI workflow
were unchanged by the refactor. The following review therefore also checks
existing behavior, without attributing it to newly executed native tests:

- The explicit path is canonicalized before loading. All five exact export
  names are resolved with their matching `PfnFfx*` types before constructing an
  owner. Symbol names cannot validate ABI, authenticity or provider version;
  those remain unsafe acquisition obligations.
- The locally cached, pinned `libloading 0.9.0` Windows implementation rejects
  `GetProcAddress` results of `None`. Successful lookups therefore establish the
  private non-null invariant used by forwarding methods. No redundant null
  check or changed public error variant is necessary.
- A failed lookup returns the named symbol error and drops the local library
  owner, including failures after earlier lookups succeed. Path, library and
  symbol failures preserve their paths and original error sources. The wrapper
  continues to map these into its own error types under D010.
- Private function pointers cannot be extracted through the API. Every call
  borrows the owner, which retains the DLL. Raw callers still own the obligations
  for contexts, callbacks and GPU work extending beyond a call.
- Library Drop uses `FreeLibrary`; its result is ignored by libloading's existing
  Drop implementation. This preserves behavior rather than guaranteeing that
  every release succeeds or every native provider unloads.
- Raw `FfxLibrary` remains `Send + Sync`, consistent with its library and pointer
  fields. This does not establish native call concurrency safety. The local
  wrapper family remains `!Send + !Sync`; the optional movable family remains
  `Send + !Sync`, with caller-established lasting coordination and arbitrary-
  thread native compatibility obligations. There is no new serialization.
- Contexts retain `Rc` or `Arc` ownership independently of the public runtime.
  Successful teardown destroys once and releases descriptor state and device
  before runtime ownership. Destroy failure and OK/null creation retain the
  critical dependency bundle; returned create errors follow D005's accepted
  bounded trust assumption. No teardown, retry or native safety contract changed.

The unchanged Windows loader fixture suite covers missing files, invalid images,
each missing export, forwarding of all arguments/returns and final module
release. Its source compiled for Windows; those tests were not executed here.

## Findings by severity

- **Critical/high/medium:** no concrete defect found in the reviewed refactor.
- **Low, verification coverage:** `tools/check-movable-api.ps1` checks production
  trees of a temporary consumer with both crates' defaults disabled. It does not
  directly assert each package's real default-feature dependency tree. Ordinary
  CI still builds default configurations. This audit separately checked the real
  package trees successfully. Extending the dedicated CI tree matrix is an
  optional follow-up, not a loader defect or acceptance blocker. The script change
  was not implemented during the audit; the later authorized follow-up is below.

## Dependency verification

Sixteen direct package production trees passed, excluding dev-dependencies:

| Package/configuration | Windows x64/MSVC: libloading | Linux x64: libloading |
|---|---|---|
| `fsr-sdk`, defaults | Present | Absent |
| `fsr-sdk`, no defaults | Absent | Absent |
| `fsr-sdk`, no defaults + DX12 | Present | Absent |
| `fsr-sdk`, defaults + movable | Present | Absent |
| `fsr-sdk`, no defaults + movable | Absent | Absent |
| `fsr-sdk-sys`, defaults | Absent | Absent |
| `fsr-sdk-sys`, no defaults | Absent | Absent |
| `fsr-sdk-sys`, no defaults + DX12 | Present | Absent |

Each command used:

```sh
cargo tree -p PACKAGE --target TARGET --edges normal --prefix none --locked --offline
```

Substitute either package and `x86_64-pc-windows-msvc` or
`x86_64-unknown-linux-gnu`; append the feature flags in the table. The wrapper's
direct `libloading` dev-dependency supports portable test fixtures and is
intentionally excluded. Lockfile presence does not establish production use.
Linux dependency resolution needs no installed Linux standard library.

## Executed checks and reproduction

Environment: macOS ARM64, Rust 1.98.1; installed targets
`aarch64-apple-darwin` and `x86_64-pc-windows-msvc`. All Cargo checks used cached
dependencies offline. No installation, runtime download or remote CI run was
performed. Matrix commands used `CARGO_INCREMENTAL=0`; Rustdoc warnings were
denied with `RUSTDOCFLAGS=-D warnings`.

The four workspace configurations were defaults, `--features fsr-sdk/movable`,
`--no-default-features`, and no defaults plus `--features fsr-sdk/movable`.

| Check | Fresh result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo test --workspace --locked --offline`, all four configurations | All pass; each executes 21 wrapper ownership/unit tests, 3 portable consumer tests, 4 raw ABI/constant tests, 1 dispatch constant test and 2 wrapper documentation compile tests |
| `cargo clippy --workspace --all-targets --locked --offline`, all four configurations, followed by `-- -D warnings` | All pass |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline`, all four configurations | All pass, including gated Windows test sources and public trait assertions |
| Windows-target Clippy, all targets, `--features fsr-sdk/movable`, denied warnings | Pass |
| `cargo doc --workspace --no-deps --locked --offline`, no defaults and movable, on host and Windows target | Four passes with denied warnings |
| `cargo test -p fsr-sdk-sys --doc --features dx12 --target x86_64-pc-windows-msvc --locked --offline` | Two compile-fail doctests pass: invocation cannot outlive owner; stored pointer is private |
| `cargo check -p fsr-sdk-sys --all-targets --features dx12 --target x86_64-pc-windows-msvc --locked --offline` | Pass independently of wrapper feature forwarding |
| Unsupported Vulkan, host and Windows target | Both return expected exit 101 and exact intentional diagnostic |
| Temporary downstream consumer controls | 60/60 pass, including expected diagnostic-specific failures |

These are 22 Cargo/formatting checks, 16 tree commands and 60 consumer controls.
Vulkan reproduction is `cargo check -p fsr-sdk --no-default-features --features
vulkan --locked --offline`, repeated with the Windows target. `--all-features`
is intentionally not a successful build configuration.

Consumer controls used a separate workspace with direct path dependencies on
both crates, `default-features = false`, and features
`dx12 = ["fsr-sdk/dx12"]` and `movable = ["fsr-sdk/movable"]`. Each control ran
`cargo check --manifest-path MANIFEST --offline`, with the selected target and
features. Exact coverage:

- Eight common positive controls: host/Windows crossed with empty, movable,
  DX12 and DX12+movable features. Common and legacy type identities, checked
  constructors and raw common ABI imports compile.
- Thirty-six independent `E0432` controls reject runtime, backend, legacy owners,
  raw loader and raw DX12 imports in each disabled-native configuration.
- Six Windows/DX12 controls check local API/type identity, raw owner
  `Send + Sync`, private `loader::windows` (`E0603`), and movable unavailable
  without its feature (`E0432`). The enabled movable positive additionally
  checks both public `Send` types and the existing constructor reexport.
- Four crossed-family/conversion controls require `E0308` or `E0277`.
- Six `E0277` controls reject `Send` and `Sync` for local Runtime/Upscaler and
  `Sync` for both movable owners.

## Environment limits and remaining risks

- No Windows executable ran. Loader error/forwarding/unload fixtures, public
  movable Windows fixtures, AMD loading/query/context lifecycle, GPU regression
  readback and native transfers were **not executed**. Host-gated loader tests
  execute zero cases; Windows compilation and compile-fail doctests do not
  replace those native checks. Earlier native records were not reused as fresh
  execution evidence. Windows follow-up uses the existing fixture tests and
  `tests/native/run.ps1 -Mode movable -Native` from an x64 VS developer shell.
- Host ABI tests establish selected scalar/signature/nullable-pointer facts.
  Windows-specific layout test source compiled but its numeric assertions did
  not execute. Independent pinned-header C++/MSVC ABI checks could not run.
  `tools/check-native-abi.ps1` remains the existing Windows verification path.
- Linux compilation was not run: its target standard library is not installed.
  Linux production dependency resolution was verified, not native support.
- Rust 1.88 and PowerShell are unavailable. The actual PowerShell CI script was
  inspected but not executed; direct Cargo consumer/tree controls were run
  separately. No complete CI-equivalent or remote CI pass is claimed.
- Native ABI compatibility, provider behavior and thread compatibility remain
  caller-established/accepted bounded assumptions. Final wrapper-owned library
  release is distinct from provider termination; the latter is not promised.
  These are existing limitations, not newly introduced regressions.

Local raw logs and detailed command/control results are in
`target/loader-refactor-audit`; they are optional diagnostics, not required to
understand this report. Audited loader SHA-256:
`9ac41b93178d6d4d3db25e9c6503798e5c361490ad69e531d8e985e9d87bf4f4`.
Raw manifest SHA-256:
`e2ae5e62c3f913612754f2a6dc49b60b43cc969d729d70aaed28686a0cbf020e`.

## Authorized CI coverage follow-up — 2026-10-09

After the audit, the maintainer authorized the proposed small CI fix.
`tools/check-movable-api.ps1` now also checks the eight real package
configurations in the dependency table against both target triples. These
16 additional commands use the repository manifest, select each package directly
and pass `--edges normal --locked --offline`. Expected loader presence is
specified independently of the manifests: Windows defaults for the wrapper
require it, raw defaults do not, explicit DX12 requires it only on Windows, and
movable without DX12 does not require it. Existing consumer controls and the
unsupported-Vulkan check remain intact. The CI workflow already invokes this
script on Ubuntu and Windows; no workflow or runtime change is needed.

Fresh local verification extracted the package/configuration arguments from the
edited script's table and executed all 16 dependency commands successfully.
Four isolated dependency-only workspace copies then changed one manifest fact
at a time: removed wrapper DX12 defaults, enabled raw DX12 defaults, moved the
optional loader dependency out of its Windows target section, or removed DX12
forwarding from the wrapper. Every modified tree resolved successfully but
violated its expected loader-presence assertion. This checks that the matrix
detects those concrete regressions rather than merely accepting the current
manifests. No production manifests or sources were modified by these controls.

PowerShell remains unavailable: the PowerShell script itself was not executed,
and the local argument/assertion checks are not a claim of a CI pass. Script
syntax and final changes were reviewed; whitespace and documentation links were
checked. Rust/native tests were not repeated because the follow-up changes only
dependency-check orchestration and documentation. Local diagnostic results are
under `target/loader-defaults-ci-check`; this section preserves the inputs,
observations and limits independently of those optional logs.
