# Active render-size implementation checks — 2026-10-14 UTC

**Follow-up, 2026-10-15:** Windows fixture/MSRV/C++ ABI and four native active-region
shrink/grow profiles passed in the [combined Windows batch](2026-10-15-windows-m11-m14-verification.md).
The earlier pending status below remains historical; performance and variable
output remain open.

[D024](../adr/D024.md) implements the first M14 slice for SDK v2.3.0: dynamic
active render sizes inside fixed maximum allocations, with fixed output.
This is implementation and compilation evidence, not Windows/native acceptance.
The work continued into 2026-10-15 in the maintainer's local timezone.

The maintainer explicitly deferred P01's pre-M14 performance baseline because
no Windows test machine is currently available. Performance is unmeasured;
future measurements must identify the feature set actually measured. Image
quality validation remains on hold under [scope](../../SCOPE.md#verification-priority).

## Contract and test inputs

- `UpscalerOptions::dynamic_resolution()` defaults false; setter adds bit 6 to
  both creation and memory estimates. Estimates retain context maxima. Controlled
  host tests check flags 0, 64, 126 and 62 and maxima 320×180 / 640×360.
- `UpscaleDispatch::active_render_size()` defaults None, resolving to the context
  maximum. Setter accepts optional checked `Dimensions`; None restores the default.
  Fixed contexts accept only the maximum. Dynamic contexts check both axes before
  COM inspection. Host bounds tests cover 1×1, independent edge axes, maxima,
  smaller regions and overflow on either/both axes, including `u32::MAX`.
- Exact allocation validation is retained for all roles. Native allocation
  metadata stays unchanged; render size and render-motion UV scale use the active
  region. Display-motion UV scale uses fixed output; pixels use `(1, 1)`.
- Windows controlled public fixtures are implemented for both families with
  fixed/dynamic and render/display profiles. They check creation flags, unchanged
  allocation metadata, UV/pixel scaling, shrink/grow/default restoration, reset
  forwarding, bounds before failing COM inspection, allocation rejection, no
  native entry on rejection, reference balance and valid retry. They compiled
  here but were not executed.
- Raw Rust assertions and the paired C++ header probe include bit 6. Existing
  descriptor layouts are unchanged; the C++ probe remains unexecuted locally.

## Checks run

Working directory: repository root. Host: macOS ARM; Rust/Cargo 1.98.1 stable.
Dependencies and Windows target libraries were already cached. Commands below
completed with exit 0; each result was checked independently.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline
cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features fsr-sdk/movable --locked --offline
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features movable --locked --offline
python tools/verification/run.py run consumers
PYTHONPATH=tools/verification/tests python -m unittest test_infrastructure.InfrastructureTests.test_deferred_features_are_selected_in_source_and_artifact_native_profiles -v
cargo package --workspace --locked --offline --allow-dirty --no-verify
```

Use Python 3.11+ for verification commands. This run used the already installed
Python 3.12.14. The host test configurations each passed 38 library tests, eight
portable dimension/configuration tests, one query-value test and applicable raw
ABI/doc tests. The ownership compile-error harness's positive control also passed.
The consumer profile passed all 81 selected host cases, including the new portable
configuration snippet and existing feature gates/dependency/example checks.

Windows all-target compilation covers the new fixtures and native Rust case;
default+movable compilation is also covered by Windows Clippy. The Windows-only
`active-render-frame` consumer snippet was separately compiled against the local
crates with `dx12,movable`, using the same source from
[consumer-cases.json](../../tools/verification/consumer-cases.json). It accepted
the public setter/getter/default restoration and emitted only an unused test
function warning. No Windows process was executed.

The focused infrastructure regression confirms exact selectors, markers, scopes
and prerequisites in both source and artefact native profiles. This is registration
evidence only; the full infrastructure suite was not rerun for this change.

Draft packaging produced both archives with `--no-verify`. Archive inspection
checked byte-identical MPL/AMD notices and inclusion of the new Rust fixtures and
native helper header. It is not extracted-package, registry, release or publication
verification. Changed documentation links, repository budget and diff whitespace
were checked separately.

## Prepared native confirmation

The ignored `upscaler::dx12::m6::native_active_render_dispatch` case is selected
as `native.dispatch.active-render` and `artifact.dispatch.active-render`.
It reuses the existing trusted DX12 helper and pinned signed loader/provider.

Four fresh contexts cover render/display motion in UV/pixel units, with standard
finite depth, jitter-excluded motion, explicit exposure and default pre-exposure.
Each context dispatches three moving/jittered frames with active sizes 320×180,
256×144 and 288×162, fixed 320×180 render allocations, 640×360 output and matching
motion allocations. Only the first frame resets history. The helper writes each
active image into the top-left region and fills inactive colour/motion with finite
65504 guards and inactive depth with 1.0.

After every successful recording, the helper submits, fences and reads full
output plus colour/depth/motion. Criteria require finite output, replacement of
every output sentinel pixel, byte-exact preservation of all supplied input pixels
including inactive guards, no device removal/debug errors and explicit context
destruction before resource cleanup. Input readback establishes preservation of
read-only resources; it does not prove that inactive texels were never sampled.
There is no reconstruction-quality comparison or timing criterion.

```sh
python tools/verification/run.py plan native-upscaler
python tools/verification/run.py run native-upscaler --native --sdk-root <local-sdk>
```

The native case, new C++ helper code and independent C++ assertion have not run
here: Windows, MSVC, signed native deployment and a hardware DX12 device are absent.
Rust 1.88/MSRV execution is also unavailable locally. Native/provider transition
acceptance, performance and variable output remain open; this record closes none
of those checkpoints.
