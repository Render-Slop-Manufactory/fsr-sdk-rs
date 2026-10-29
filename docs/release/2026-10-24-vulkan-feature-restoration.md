# Vulkan feature restoration and sys 0.1.2 verification

Date: 2026-10-24. Development-source and local-package checks for
`fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, following the explicitly authorised
[D031 amendment](../adr/D031.md). This is not final release closure.

## Selected inputs and behaviour

The wrapper again declares `vulkan = ["fsr-sdk-sys/vulkan"]`; sys declares
`vulkan = []` and deliberately emits a Rust compile error whenever it is enabled.
The diagnostic identifies Vulkan as unimplemented for wrapper `0.3.0` and points
to the initial `0.4.0` upscaling target, subject to official SDK/runtime and native
evidence. No Vulkan implementation, dependency or SDK migration is introduced.

The raw package version, wrapper dependency and workspace lockfile now agree on
`0.1.2`. Existing raw layouts, loader contracts and Windows/DX12 gates are unchanged.
Current decisions, release notes, package documentation and topic summaries match
the amendment; historical source/experiment and verification records retain their
original version pairs and results. Wrapper `0.3.0`'s stronger unsafe acquisition
contracts remain a separate breaking change.

Verification ran on Windows x64/MSVC with Rust/Cargo 1.99.0, exact Rust 1.88.0
for MSRV cases, Python 3.14.1 and the installed PowerShell/MSVC tools. The four
C++ ABI probes used local, hash-verified SDK v2.3.0 headers pinned at
`60f4ea81909200d8542eca14dccb2628b763a9a3` by `tools/native-headers.json`.
No new SDK/runtime acquisition was needed.

## Commands and results

Run from the repository root. Cargo/profile checks used cached inputs offline.

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
python -m unittest discover -s tools/verification/tests -p test_infrastructure.py -v
python tools/verification/run.py run consumers --require-passed
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run release-artifacts --require-passed
cargo package --workspace --list --allow-dirty --locked --offline
git diff --check
python tools/verification/repository_size.py
```

| Check | Observed result |
|---|---|
| Formatting, workspace all-target compilation and whitespace | Passed |
| Focused infrastructure suite | 30/30 passed, including the new Vulkan rejection controls |
| Source consumers | 145/145 cases passed |
| Windows `ci` | 182/182 cases passed, including four independent C++ ABI probes, seven Rust feature configurations, Clippy, Rustdoc and consumers |
| Infrastructure suite within `ci` | 264 tests: 260 passed, four platform/symlink controls skipped on Windows; archived evidence integrity and required PowerShell syntax passed separately |
| Exact Rust 1.88.0 `msrv` | 159/159 cases passed |
| `release-artifacts` | 311/311 cases passed against the exact local archives and isolated registry |
| Package listing/content inspection | Expected source, manifests and project/AMD licence files present; archive/source identity checks passed |

For each crate, consumer checks first compile its implemented feature combination
and check the reserved manifest feature/forwarding. They then require a completed,
nonzero Rust compilation with ``error: the `vulkan` feature is intentionally unsupported``
for both Vulkan alone and all features. These controls pass in source, MSRV and
both packaged consumer profiles. Missing-feature errors, unrelated compiler
failures, successful commands, aborts and timeouts are rejected by the new
infrastructure regression test. Existing Linux/Windows production dependency
graph checks continue to verify Windows/DX12 isolation; they are not Linux execution.

The package validator's stale docs.rs expectation was aligned with the existing
manifest selection `movable,utils`; Vulkan remains excluded from docs.rs builds.
The initial restricted focused-test invocation failed on temporary-directory
permissions. Repeating it with normal temporary-directory access passed; the
complete profiles used that access too. No code workaround was needed.

Local attempts under `target/verification/` were:

- `20261024T132336Z-consumers-3c8646e23644`
- `20261024T132609Z-ci-e519a98298a0`
- `20261024T132655Z-msrv-4d56d1da0ea3`
- `20261024T132654Z-release-artifacts-26d3110cfd52`

All four reported unchanged inputs during execution. After finalisation,
`python tools/verification/run.py verify-run --run target/verification/<attempt>`
rechecked their inventories, required outcomes and retained file hashes. The
implementation checks preceded addition of this report and its documentation links;
no crate content changed afterwards.

## Local archive identity and limits

| Archive | SHA-256 |
|---|---|
| `fsr-sdk-0.3.0.crate` | `07fc296c5091e85a54d44a4aca67b94dddff34d17293156cfec032e4ab9df833` |
| `fsr-sdk-sys-0.1.2.crate` | `2f371e89d14def715bfa38133d5d61a36d8761d91d182d5b53d54677364a6e76` |

The package profile checked current/MSRV compilation and tests, current-toolchain
Clippy/Rustdoc, registry-only consumers, dependency provenance and unchanged
archive/extracted bytes. These are development archives. No actual AMD runtime/GPU
execution, publication simulation, publication or new native Vulkan support was
part of this change. Final `0.3.0` release acceptance remains tracked in
[release preparation](0.3.0-preparation.md).
