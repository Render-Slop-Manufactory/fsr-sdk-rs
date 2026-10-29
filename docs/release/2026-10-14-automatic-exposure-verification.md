# Automatic exposure: implementation verification

**Follow-up, 2026-10-15:** the corrected source passes complete Windows CI/MSRV,
C++ ABI, controlled fixtures and native automatic-exposure dispatch in the
[combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The earlier failure, correction and execution limits below remain historical evidence.

- Date: 2026-10-14, Europe/Berlin.
- Source: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0.
- Contract: [D023](../adr/D023.md), explicit maintainer authorization for M13.
- October 14 result, corrected after CI diagnosis: **original host runs failed;
  the local harness fix then passed default and no-default/movable workspace tests.** Consumers
  and Windows compilation passed. Full CI/MSRV acceptance and native GPU execution
  remained pending at that point. The October 15 follow-up above closes this
  development-source checkpoint.

## Contracts and tests

Portable `ExposureMode::{Explicit, Automatic}` configures contexts and memory
estimation through the shared private flag conversion. Explicit remains default;
automatic adds bit 5. Existing layouts, SDK pin, exports and dependencies are
unchanged. Paired [Rust](../../crates/fsr-sdk-sys/tests/dispatch_abi.rs) /
[C++](../../crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp) assertions cover the
added constant. Rust assertions run after the harness correction; independent C++ probes also
pass in the failed Windows stable CI run, without making that whole run a pass.

[Resources](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs) retain the
five-resource explicit constructor and add `without_exposure` for automatic
contexts. A mode/presence mismatch fails before COM/native entry without poison.
Only present resources are inspected; automatic dispatch emits a null exposure
resource with zero metadata/state. Existing colour/depth/motion/output formats,
resource identity/device checks, fixed extents and GPU duties remain unchanged.
Pre-exposure still reports the factor already applied to colour independently.

Automatic construction queries the actual provider's required inputs before
returning a usable context. Exposure is satisfied by the automatic context flag;
required masks or unknown inputs outside this profile reject construction with
the full mask. Optional requirements do not force additional textures. Native
query errors preserve codes. Rejection makes ordinary terminal cleanup with no
wrapper GPU recording; failed cleanup retains the critical bundle under D005.
Explicit construction makes no extra query. Automatic correctness still depends
on the acquired trusted provider's flag/resource/query contract.

- [Portable tests](../../crates/fsr-sdk/tests/dimensions.rs): default, option
  cloning and traits.
- [Host fixtures](../../crates/fsr-sdk/src/query/tests.rs): estimate flags `32`
  and combined depth/motion/automatic flags `62`; accept supported required masks
  including exposure, reject required masks/unknown bits while preserving them.
- [Windows public fixtures](../../crates/fsr-sdk/src/upscaler/dx12/tests/automatic_exposure.rs):
  both families, creation query count/flags, exact explicit/null exposure mapping,
  unchanged pre-exposure, rejected mismatches without native recording, valid
  retry, unchanged COM counts, skipped inspection of omitted exposure, explicit
  metadata rejection, query errors, unsupported masks and post-create cleanup.
  Compiled here; this controlled test also passes in the failed Windows stable
  CI run with default and movable features. Full CI acceptance remains pending.
- The selected native automatic case uses
  [three-frame equipment](../../crates/fsr-sdk/tests/native/temporal_smoke.rs):
  real requirements/memory queries, moving/jittered frames, absent exposure,
  recording/submission/fencing, finite/written output, device/debug checks and
  explicit teardown. It uses the local ownership family; movable behaviour is
  covered by prepared controlled fixtures. The helper still owns an unused
  exposure allocation, which is never supplied to FSR. No renderer-allocation
  saving, exposure-value accuracy or image-quality comparison is asserted. Native
  execution is pending.
- [Native case definitions](../../tools/verification/native-cases.json),
  [profile selection](../../tools/verification/native.py) and the
  [registration regression](../../tools/verification/tests/test_infrastructure.py)
  include source and artefact automatic-exposure verification.

## Checks run here

macOS ARM (`aarch64-apple-darwin`), Rust/Cargo 1.98.1 and bundled Python 3.12.14
with existing NumPy; Windows Rust target installed. Rust 1.88, PowerShell,
Windows/MSVC and native DX12 execution are unavailable. Cargo used existing caches
offline. Commands from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline
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

The original blanket claim that all checks passed was incorrect: both local
workspace test commands failed in the ownership compilation harness (see the
correction below). All 80 portable consumer-profile checks pass. An isolated
Windows-target consumer compiles the exact `automatic-exposure-resources` snippet
in [consumer-cases.json](../../tools/verification/consumer-cases.json) with movable
enabled. The infrastructure suite runs outside the sandbox for localhost fixtures:
149 of 154 tests pass, with three unavailable PowerShell parser and two Linux-only
checks skipped. Changed local documentation links and repository budget pass.
Draft archive inspection checks changed source and licence bytes; `--no-verify`
packaging does not establish extracted-package builds or final release acceptance.

## CI failure and local correction

The M13 CI run on 2026-10-14 fails in Rust tests on Linux/Windows, stable and 1.88.
The common failing test is `context::tests::consumed_owner_and_private_handle_are_compile_errors`.
Its positive standalone harness imports the real `error.rs` and `context.rs`, but
M13's new error field refers to `crate::query::ResourceFlags`. Without a query
module, the supposedly valid harness fails before ownership diagnostics:

```text
error[E0433]: cannot find `query` in the crate root
required: crate::query::ResourceFlags
```

The same failure is present in the original local logs. Later successful commands
in the shell batch masked the failed test status; the earlier passing-test report
was wrong. The correction imports the real `query.rs` into the harness. It retains
the positive control and every negative ownership/privacy case; no assertions or
API/error contracts are removed. The existing positive control detects this bug.

After this correction, both complete local workspace test commands listed above
pass (default and no-default/movable), as do formatting and Windows-target Clippy.
At the time of this correction, the remote run had not been rerun with the fix.
The failed run's Windows stable results independently showed the automatic-exposure
controlled test passing for both families and the paired C++ ABI probes passing;
the overall profile still failed. Those individual checks did not establish native
GPU execution or full MSRV acceptance.
CI run `38581862201` is supplemental provenance; the failure and fix are
described here without relying on the hosted run.

Subsequent hosted CI passed on Windows and Linux with stable Rust and Rust 1.88,
including the October 15 run `38645911172` (Europe/Berlin).
The self-contained [October 15 Windows record](2026-10-15-windows-m11-m14-verification.md)
retains all 112 passing CI, 95 passing MSRV and 27 passing native-upscaler cases.
The corrected ownership harness, C++ ABI probes, controlled exposure fixtures and
native automatic-exposure case pass. Final package/release acceptance remains separate.

## Batch requested on October 14

This batch subsequently passed in the October 15 follow-up linked above. The
commands below retain the original request rather than an outstanding feature gate.

On the prepared Windows/MSVC/runtime/GPU environment:

```sh
python tools/verification/run.py run ci
python tools/verification/run.py run msrv
python tools/verification/run.py run native-upscaler --native --sdk-root /path/to/Kits/FidelityFX --adapter-index 0
```

The original request required the actual trusted root and device and proposed
combining native integration with M11/M12/U03. At that point no new platform
support, verified native behaviour, final release acceptance or publication was
claimed. The October 15 follow-up records the subsequent native result; final
release acceptance and publication remain separate.
