# Linear HDR implementation checks — 2026-10-15

[D026](../adr/D026.md) implements M16's first linear HDR slice under maintainer
authorization following review of the [concept](../research/records/2026-10-15-src-hdr-colour-encoding-concept.md).
This is a development-source implementation record, not native acceptance or
automatic selection for publication. Non-linear colour remains open.

## Inputs and implemented behaviour

Host: macOS ARM64, stable Rust/Cargo 1.98.1. SDK baseline: v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Commands run from the repository root.
No AMD runtime was loaded. No dependencies, SDK pin or platform support changed.

Portable `UpscalerOptions::high_dynamic_range` / `set_high_dynamic_range` default
to false. Enabling HDR adds only the existing creation bit 0 through the shared
context/estimate conversion. Both ownership families use those paths. RGBA16F
linear colour/output, exposure/pre-exposure meanings and zero dispatch flags
remain. Context options cannot be reconfigured through a caller's options clone.
Native acquisition/dispatch documentation explicitly requires actual provider
support for the selected HDR/exposure profile; resource queries prove no colour
capability. Native ownership, poisoning and GPU obligations are unchanged.

Portable tests cover defaults, clone independence, other creation bits and memory
estimation under explicit/automatic exposure. Prepared controlled Windows fixtures
exercise both families' public creation/estimate/dispatch paths, exact flags,
pre-exposure forwarding, exposure omission, colour-format rejection without
poison, successful retry and balanced references. Rust and independent C++ probes
assert `FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE == 1 << 0`; layouts are unchanged.

## Local checks

The following completed successfully:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --features movable --locked --offline
cargo check --workspace --all-targets --no-default-features --features movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline
cargo clippy --workspace --all-targets --features movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features movable --target x86_64-pc-windows-msvc --locked --offline
python -m unittest discover -s tools/verification/tests -p test_infrastructure.py
python tools/verification/run.py run consumers --require-passed
cargo package -p fsr-sdk-sys --list --allow-dirty --locked --offline
cargo package -p fsr-sdk --list --allow-dirty --locked --offline
cargo package --workspace --no-verify --allow-dirty --locked --offline
```

The final default workspace run passes 41 library tests, nine portable dimensions/
options tests, one portable query-value test, six raw ABI/constants tests and two
doctests. Movable workspace tests also pass; the additional HDR memory-estimate
test was subsequently run in the final default suite. Windows-gated fixtures and
native selectors compile but do not execute on macOS. All 82 consumer cases pass,
including the new portable HDR option case, feature/dependency boundaries and
complete portable/movable examples.

Python checks used the already bundled Python 3.12 runtime. The initial sandboxed
infrastructure run passed 20 tests and failed six process-fixture tests because
local sockets were denied. The authorised unsandboxed repetition passed all 26.
The profile assertion checks HDR case selection and prerequisites for both source
and extracted-package native profiles. No GPU acceptance is inferred.

Both local archives were opened and their LICENSE and LICENSE-AMD bytes compared
with the source copies; the wrapper includes the new HDR fixture source. These
`--no-verify` packages establish contents and notices only, not final package or
extracted-package native acceptance. Changed Markdown local links and
`git diff --check` also pass. No commit or publication was made.

## Prepared native protocol and remaining checks

`native.dispatch.linear-hdr` and `artifact.dispatch.linear-hdr` select
`upscaler::dx12::native_dispatch::native_hdr_dispatch`. Two contexts use explicit
and automatic exposure respectively. Each runs three continuous frames, reset
only on frame zero, at 320×180 render / 640×360 output. Scene RGB is constant
`(8,4,2)`; matching pre-exposure `0.25 → 1 → 4` packs red values `2 → 8 → 32`.
Alpha is 1, depth is 0.25, motion is zero, and the explicit exposure texture is 1.
Automatic dispatch omits exposure; the helper still retains its unused texture.

The helper extends the existing full-input readback machinery to include exposure
for HDR fixtures. After fencing, all colour/depth/motion/exposure bytes must match
the pending uploads. Output must replace every sentinel pixel, remain finite,
and have red strictly above 1 and below 64. Those broad signal-domain bounds reject
clipped `[0,1]` output and stale red sentinel values; they do not establish
reconstruction quality or numerical equivalence across exposure factors.
All frame dependencies survive explicit context teardown; device/debug and
post-resource-cleanup checks reuse the existing harness.

No Windows x64/MSVC host is currently available. Controlled fixture execution,
independent C++ SDK-header assertions, native helper compilation and GPU recording/
submission/readback/cleanup therefore remain unrun. Rust 1.88.0 is not installed,
so MSRV execution also remains pending. Run on the trusted Windows/GPU host:

```sh
python tools/verification/run.py run ci --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-upscaler --native --require-passed --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Select the actual pinned SDK root and hardware adapter. Native HDR coverage uses
the local ownership family; both families have prepared controlled mapping tests.
M16 remains active until its selected checks and separate non-linear contract
are resolved. Existing M15 deferral and held image-quality work are unchanged.
