# Selected upscaler Query implementation verification — 2026-10-12

Follow-up: the [Windows checkpoint](2026-10-12-windows-m10-sdk-verification.md)
subsequently passed on 2026-10-12, including MSVC ABI, controlled DLL execution,
consumers, MSRV and both public query families followed by GPU regressions.
M10 is now Verified in unreleased source. The pending observations below describe
the earlier macOS attempt and remain historical evidence.

## Scope and inputs

Unreleased M10 implements the five methods accepted in [D018](../adr/D018.md)
for local and optional movable owners. The public results are portable, owned
Rust types. Safe methods rely on explicitly extended unsafe acquisition trust;
this record proves no universal closed-provider guarantee.

Checks used macOS ARM64, Rust 1.98.1 (`48a229cea`, 2026-09-01), Python 3.12.14
and Apple Clang 21.0.0. The installed Windows Rust target was
`x86_64-pc-windows-msvc`. SDK headers were from v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`, in the locally acquired SDK.
The native-profile attempt verified all four normalized header hashes against
[the existing pins](../../tools/native-headers.json).
No crate version, dependency version or SDK pin changed; nothing was published.

## Checks and observations

Commands were run from the repository root; `python` below denotes the existing
Python 3.12.14 interpreter, rather than the host's older default interpreter.

```sh
python tools/verification/run.py run ci
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --offline --locked -- -D warnings
cargo clippy --workspace --all-targets --features movable --target x86_64-pc-windows-msvc --offline --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features movable --target x86_64-pc-windows-msvc --offline --locked
cargo test -p fsr-sdk --lib --offline --locked query::
python -m unittest discover -s tools/verification/tests -v
```

The portable `ci` attempt completed 94 cases. Formatting, check/test/Clippy/docs
for all four configurations, downstream diagnostics, dependency trees, Vulkan
rejection, archive integrity and examples passed. Its aggregate status remained
failed because six infrastructure tests could not bind their loopback sockets
inside the sandbox. Re-running the infrastructure suite outside that sandbox
passed: 115 tests, three PowerShell-dependent skips. This does not retrospectively
turn the first attempt into a passed profile. Rust 1.88.0 was not installed;
MSRV execution remains unperformed.

The eight focused query tests passed. They exercise device/options forwarding,
count/fill shrinkage and bounded truncation, checked allocation, immediate/lossy
name copying, invalid successful outputs, unknown/overlapping masks, zero and
large byte values, every known non-OK code plus an unknown code, and poison
rejection without invocation. The movable host suite had 29 passing library
tests. Windows all-target Clippy passed for both default and movable builds;
Windows Rustdoc also passed. Compilation includes the new controlled DLL test,
which calls all five public methods and checks ordinary-error continuation and
poison for both families. That Windows fixture was compiled, not executed here.

Ten downstream query cases were additionally checked with `cargo check
--target x86_64-pc-windows-msvc --offline --locked` in an isolated path-dependent
consumer. Both public families compile without unsafe query calls, result types
remain available without `dx12`, fields/ID construction reject private access,
and shared references reject queries with E0596. The checked snippets and
expected diagnostics are retained in [consumer-cases.json](../../tools/verification/consumer-cases.json).

The independent pinned-header C++ query assertions passed with:

```sh
clang++ -std=c++17 -fdeclspec -Wno-ignored-attributes -fsyntax-only \
  -Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include \
  -Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/include \
  crates/fsr-sdk-sys/tests/query_native_abi.cpp
```

This is a host Clang header/layout check, not MSVC ABI evidence. Rust assertions
cover all added field types, offsets, sizes/alignment, tags and resource bits;
the runner now includes the independent query probe in Windows ABI preparation.

`cargo package --list --allow-dirty --offline --locked` passed for each crate.
Local archives generated with `CARGO_TARGET_DIR=target/query-package cargo
package -p <crate> --allow-dirty --no-verify --offline --locked` contained the new
sources/tests/example, both licence texts and the package README. Licence copies
matched the repository byte-for-byte. These were inclusion checks without
package build verification or registry/release compatibility acceptance.

## Pending native checkpoint

```sh
python tools/verification/run.py run native-upscaler --native \
  --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The attempt stopped at `windows.environment`: Windows x64/MSVC required. Header
hashes passed; all four MSVC probes, Authenticode checks, native fixture execution,
public queries and GPU regressions were not run. No native or GPU pass is claimed.
Full local attempts remain under ignored `target/verification`; the observations
and limits above stand independently of those temporary files.

[The native profile](../../tools/verification/native.py) now requires the public
five-query child for both families before existing lifecycle/dispatch regressions.
Gate A also queries the same context before dispatch; movable recording checks
the three live queries on transferred contexts under the existing common gate.
Run these on Windows with the signed pinned deployment, then retain actual
query/provider and dispatch/readback results. M10 remains **Active** until this
checkpoint passes. Release/version selection, MSRV and final artefact checks
remain separate from the implemented source surface.
