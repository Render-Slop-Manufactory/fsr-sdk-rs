# Supplied-mask implementation checks — 2026-10-15

[D025](../adr/D025.md) implements the M15 supplied-mask API for SDK v2.3.0.
This record distinguishes completed local checks from pending Windows/native
acceptance. The feature is implemented, not yet native-verified or selected for
release automatically. No quality study was run.

## Inputs and implemented contract

Host: macOS ARM64; Rust/Cargo 1.98.1 stable. SDK baseline: v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. No AMD runtime was loaded here.
Working directory for commands below is the repository root.

The actual source adds two optional borrowed mask inputs to the private
`UpscaleResources` bundle, with unchanged constructors and null defaults. R8_UNORM
maps to native format 25 using the existing resource/dispatch ABI layouts.
Every created context now queries and privately stores resource requirements;
unknown required inputs and query failures reject construction with terminal
cleanup. The public requirements method still queries live. Missing required
masks and supplied masks advertised as neither required nor optional reject
before COM/native recording, without poisoning a usable context.

Portable tests cover required/optional presence combinations and overlap, unknown
optional bits and unknown required-bit rejection. Prepared Windows fixtures cover
both ownership families and exposure modes, per-frame presence changes, completely
zero absent descriptors, exact present metadata, dynamic active dimensions,
format/shape/extent/readability/device/canonical-identity failures, balanced COM
references, retries and native-error poison. Constructor-query errors and unknown
required inputs check terminal cleanup under both exposure modes. Public consumer
tests compile getter/setter lifetimes and preserve existing input trait assertions.
Those Windows tests were compiled here; they were not executed.

## Completed checks

All commands below completed successfully unless explicitly qualified:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --locked --offline --features movable
cargo check --workspace --all-targets --no-default-features --features movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline
cargo clippy --workspace --all-targets --features movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features movable --target x86_64-pc-windows-msvc --locked --offline
python3 -m unittest discover -s tools/verification/tests -p test_infrastructure.py
python3 tools/verification/run.py run consumers --require-passed
cargo package -p fsr-sdk-sys --list --allow-dirty --locked --offline
cargo package -p fsr-sdk --list --allow-dirty --locked --offline
cargo package --workspace --no-verify --allow-dirty --locked --offline
```

The Python checks used the existing bundled Python 3.12 runtime explicitly,
without installation or dependency changes. The system Python 3.9 first failed
on the coordinator's existing `tomllib` dependency. The infrastructure suite then
passed all 26 tests with Python 3.12 outside the sandbox; the initial sandboxed
attempt passed 20 and failed six local-socket process tests with `PermissionError`.
The feature-specific infrastructure assertion confirms selection in both source
and extracted-package native profiles. Consumer verification passed all 81 cases,
including platform gates, dependency trees and complete portable/movable examples.
Windows-target dependency-tree cases inspect feature graphs; they execute no DLL.

Both Cargo test configurations passed 39 library tests, eight dimensions tests,
one portable query-input test, six raw ABI/constants tests and two doctests.
Windows-gated cases do not execute on macOS. Current Rust compilation and Clippy
for Windows include both the prepared mask fixtures and native Rust selectors.
Rustdoc compiled for the Windows public surface with warnings denied.

The locally generated archives were opened and their LICENSE and LICENSE-AMD
bytes checked against each crate's source copies. The wrapper archive includes
`tests/native/supplied_masks.h`. Packaging used `--no-verify`: this is an inventory/
notice check, not final package verification, extracted-package GPU acceptance
or publication simulation. No packages were published and no commit was created.

## Pending native confirmation

The new `native.dispatch.supplied-masks` case (also `artifact.dispatch.supplied-masks`)
selects `upscaler::dx12::native_dispatch::native_supplied_masks_dispatch`.
It prepares each mask alone and both together, under explicit and automatic
exposure: six contexts with three continuous frames each. Active sizes are
320×180 → 256×144 → 288×162 inside fixed 320×180 allocations and 640×360 output.
Distinct jitter-aligned R8_UNORM patterns include 0, 128 and 255 values and inactive
input guards. The helper records mask/input readbacks and compares uploaded bytes
after fencing. Output sentinel replacement, finite pixels, device/debug checks
and explicit context teardown reuse the ordinary integration protocol.

The helper intentionally allocates both mask inputs even when one is omitted;
this demonstrates neither allocation savings nor provider consumption. Readback
preservation proves read-only input handling, not which inactive texels were
sampled or better images. Supplied-mask GPU coverage uses the local family;
shared Movable mapping/cleanup is covered by the prepared controlled fixtures.

Windows x64/MSVC fixture execution, the independent C++ SDK-header probe including
format 25, native helper compilation and actual GPU execution remain unrun here.
Rust 1.88.0 is not installed; MSRV execution is pending too. Run on the Windows
verification host with the trusted pinned deployment:

```sh
python tools/verification/run.py run ci --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-upscaler --require-passed --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Select the actual SDK root and adapter. Final release-archive verification remains
separate from this feature checkpoint. Existing native results preceding M15 are
not evidence that the new mask path executed.
