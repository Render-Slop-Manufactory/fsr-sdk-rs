# Native colour controls implementation checks — 2026-10-15

[D027](../adr/D027.md) implements the maintainer-approved checked default and
explicit per-frame exception. This is development-source verification, not
provider/GPU acceptance or final release selection.

## Inputs and behaviour

Host: macOS ARM64, Rust/Cargo 1.98.1. SDK baseline: v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Commands run from the repository root.
No AMD runtime was loaded, no GPU work executed and no SDK/dependency/platform
selection changed. Header and sample provenance are retained in the
[source investigation](../research/records/2026-10-15-src-non-linear-colour-contract.md).

Portable `InputColorSpace` defaults to Linear; `ColorEncodingHint` defaults to
Unspecified. NonLinear maps to creation/memory-estimate bit 8. Srgb and Pq map
exclusively to dispatch bits 1 and 2. Unspecified supplies zero hint bits with
either classification. HDR and all other creation controls remain independent;
RGBA16F colour/output remain required. No conversion, PQ normalization,
pre-exposure rewrite, provider detection or history reset occurs.

The ordinary frame setter requires NonLinear for Srgb/Pq. Mismatch returns
`DispatchValidation::ColorEncodingContext` before COM/native inspection without
poisoning. The explicit `set_color_encoding_hint_allow_context_mismatch` setter
bypasses only pairing for its frame. A fresh frame checks by default; the ordinary
setter restores checking even for the same hint. Both setters are safe storage
operations; acquisition/dispatch retain their unsafe provider, signal and GPU
obligations. The policy follows D027 rather than claiming upstream universally
forbids a mismatch.

## Executed local checks

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --features movable --locked --offline
cargo check --workspace --all-targets --no-default-features --locked --offline
cargo check --workspace --all-targets --no-default-features --features movable --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --features movable --target x86_64-pc-windows-msvc --locked --offline
cargo clippy --workspace --all-targets --features movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --features movable --target x86_64-pc-windows-msvc --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features movable --target x86_64-pc-windows-msvc --locked --offline
python tools/verification/run.py run consumers --require-passed
```

All pass. Each host test suite passes 44 library tests, ten portable dimensions/
options tests, one portable query-value test, six raw ABI/constants tests and two
doctests. Added portable tests cover enum defaults/traits, clone independence,
the checked/overridden pairing matrix, exact hint bits, combined creation bits
and estimate forwarding independent of HDR. Windows compilation covers the
prepared fixture and both ownership families; it does not execute those tests.

The consumer profile passes all 83 host cases, including the added portable
colour API case, feature/dependency boundaries and complete portable/movable
examples. Python commands use the already bundled Python 3.12 interpreter.
The new Windows frame consumer case is selected on Windows and remains unexecuted
on this host. Existing verification orchestration is unchanged.

Package contents and repository checks also pass:

```sh
cargo package -p fsr-sdk-sys --list --allow-dirty --locked --offline
cargo package -p fsr-sdk --list --allow-dirty --locked --offline
cargo package --workspace --no-verify --allow-dirty --locked --offline
python tools/verification/repository_size.py
git diff --check
```

Both local archives were opened and their LICENSE and LICENSE-AMD bytes matched
the corresponding crate sources. The wrapper archive includes the exact new
colour fixture source. `--no-verify` packaging checks contents and notices, not
extracted-package build/native acceptance. Changed text files are UTF-8/LF and
local Markdown file targets exist. No commit or publication was made.

## Prepared checks and limits

`colour_flags_checked_pairing_and_per_frame_override_share_both_families` uses
the controlled Windows COM/runtime fixture, without GPU work. For each family,
eight contexts combine both input classifications, HDR selections and exposure
modes. Every hint is tried through both setters. The observer checks exact
creation/estimate/dispatch bits, pre-exposure `0.25`, omitted/present exposure and
unchanged reset. Checks are restored by the ordinary setter even for the same
hint. Later frames default to checking.

Separate failures cover a missing required mask, COM identity inspection, wrong
colour format, missing explicit exposure and oversized render region. The
override preserves each rejection. Pairing rejection precedes an injected COM
failure; valid retry reaches the fixture. A native failure preserves its code
and poisons later overridden frames before inspection/native entry. Reference
counts are balanced. Rust and independent C++ assertions cover the three exact
native constants without descriptor layout changes.

Windows fixture execution and independent Windows x64/MSVC C++ compilation remain
pending because this host lacks that execution environment. Rust 1.88 MSRV
execution remains pending because that toolchain is not installed. The existing
linear HDR GPU protocol is separate. No non-linear GPU protocol or acceptance
claim is introduced: actual provider encoding/normalization, pre-exposure and
transfer ordering, output interpretation and temporal compatibility must first
be established. Source mappings and successful compilation do not verify a PQ
pipeline. M16 and final release verification remain open.
