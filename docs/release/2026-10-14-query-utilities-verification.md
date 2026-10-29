# SDK quality, size and jitter utilities: implementation verification

**Follow-up, 2026-10-15:** the deferred Windows fixture/MSRV/C++ ABI and native
query checks passed in the [combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The original observations and pending status below describe the October 14 run.
Custom quality policies remain on hold.

- Date: 2026-10-14, Europe/Berlin.
- Source: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0.
- Contract: [D022](../adr/D022.md), explicitly authorised four-helper extension.
- Result: **host tests, consumers and Windows Rust compilation pass. Windows
  fixtures/MSRV, independent C++ ABI and actual native query execution remain
  pending.** U03 remains Active for direct SDK helpers; quality policies are held.

## Implementation and prepared tests

Four faithful raw descriptors, tags and five quality constants are added to
[the raw upscaler ABI](../../crates/fsr-sdk-sys/src/upscale.rs), with paired
[Rust](../../crates/fsr-sdk-sys/tests/query_abi.rs) /
[C++](../../crates/fsr-sdk-sys/tests/query_native_abi.cpp) layout, offset and type
assertions. Existing layouts, SDK pin, exports and dependencies are unchanged.
Rust assertions ran here; the independent Windows/MSVC probe has not run.

Portable `QualityMode` and checked `JitterPhaseCount` feed four shared
[descriptor/result helpers](../../crates/fsr-sdk/src/query/native/utilities.rs).
Both runtime owners expose device-backed pre-create calls through a temporary
[DX12 backend extension](../../crates/fsr-sdk/src/runtime/queries.rs); both
upscaler owners expose live calls through the existing context owner. The SDK
supplies ratios, render rounding, phase counts and jitter components. Nothing
reconfigures options, modifies a frame, selects a provider override or records GPU
work. Ratios are positive finite scalars; dimensions are checked nonzero and no
larger than display; phase counts are positive i32; jitter is finite pixel offset.

Signed utility dimensions and jitter width ratios have explicit input guards;
negative/out-of-phase indices fail before FFI. Missing/invalid successful outputs
are invariant failures. Non-OK native codes take precedence over error outputs.
Live poison rejects before validation/invocation; query errors leave poison
unchanged. The conditional unsafe acquisition contract extends to numerical
safety for this bounded domain and synchronous consumption of backend extensions.
Open source and fixtures do not prove arbitrary-provider safety.

- [Host fixtures](../../crates/fsr-sdk/src/query/utility_tests.rs): all five mode
  mappings, exact descriptor inputs/tags, non-formula provider outputs, odd render
  sizes, signed numerical boundaries, input/poison rejection without invocation,
  native error preservation and invalid/missing result checks.
- [Portable input tests](../../crates/fsr-sdk/tests/query_values.rs): checked
  positive count construction, rejection and portable traits.
- [Windows public fixtures](../../crates/fsr-sdk/src/upscaler/dx12/tests/utility_queries.rs):
  all four owners, correct device/backend or live-context routing, reference counts,
  returned errors, valid retry, poison and independent runtime queries. Compiled
  here; not executed on Windows.
- The same Windows file contains a separate ignored native case: all five quality
  modes/queried sizes, phase counts and three offsets for pre-create/live use,
  both families, provider logging and explicit destruction. It records observations
  without requiring provider equality, fixed formulas or image-quality criteria.
  No GPU commands are submitted; actual native execution has not run here.

## Deferred native profile correction

The existing M11/M12 selectors were present in
[native-cases.json](../../tools/verification/native-cases.json), but omitted by
`regression_cases` when constructing executable profiles. Earlier implementation
records accurately retain pending execution; their statement that the profiles
included these cases was incomplete. The
[coordinator](../../tools/verification/native.py) now selects pre-exposure,
depth/motion and utility queries in both source and artefact native profiles.
A [regression test](../../tools/verification/tests/test_infrastructure.py) checks
selectors, markers and prerequisites for `native-upscaler` and `release`.
The utility case has `native-query` scope. This fixes registration only; none of
these tests is retroactively claimed as executed.

## Checks run here

macOS ARM (`aarch64-apple-darwin`), Rust/Cargo 1.98.1 and bundled Python 3.12.14
with existing NumPy. Windows Rust target is installed. Rust 1.88, PowerShell,
Windows/MSVC and native DX12 execution are unavailable. Existing caches are used
offline. Commands run from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
python tools/verification/run.py run consumers
python -m unittest discover -s tools/verification/tests -v
cargo package --workspace --locked --offline --allow-dirty --no-verify
cargo package -p fsr-sdk --list --locked --offline --allow-dirty
python tools/verification/repository_size.py
git diff --check
```

Final checks pass. Host tests include five new utility fixture tests and one
portable-input test. All 79 consumer-profile checks pass. Isolated Windows-target
consumers compile the exact `queries-local` and `queries-movable` sources in
[consumer-cases.json](../../tools/verification/consumer-cases.json), exercising
all new methods with the existing local/movable boundaries. The infrastructure
suite runs outside the sandbox for localhost fixtures: 149 of 154 tests pass;
three unavailable PowerShell parser and two Linux-only checks are skipped.
Changed documentation links and repository size pass. Draft archive inspection
checks changed source bytes and licence contents. Packaging with `--no-verify`
does not establish extracted-package builds or final release acceptance.

## Remaining batch

On the prepared Windows/MSVC/runtime/GPU configuration:

```sh
python tools/verification/run.py run ci
python tools/verification/run.py run msrv
python tools/verification/run.py run native-upscaler --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

Select the actual trusted root and device. Ordinary builds/profile runs do not
acquire AMD binaries. Native query execution and M11/M12 dispatch integration can
be verified together; upscaling image-quality validation remains on hold. No new
platform support, final release acceptance or publication is claimed.
