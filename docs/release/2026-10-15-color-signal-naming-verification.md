# Colour-signal naming migration verification

Date: 2026-10-15. Development-source verification on Windows x64/MSVC with
SDK v2.3.0, AMD Radeon RX 9060 XT (`1002:7590`), driver `32.0.31041.1004`,
and provider `4.1.1` (`0xf5a5ca1e01001001`). This checks naming, compatibility
and existing integration contracts on one configuration.

## Change and compatibility

Current producers use `color_signal.py`, `native_color_signal`,
`gpu_set_color_signal`, `FSR_SDK_COLOR_*`, `COLOR_*` labels and
`analysis/color-signal.json`. The canonical profile names are
`experiment-color-signal` and `experiment-color-signal-controls`.

Existing command/module/test names, native selector and helper export remain
forwarding aliases. Native environment inputs accept either spelling, accept
matching old/new values and reject conflicting values before native setup.
The metadata reader accepts recorded `COLOUR_*` labels with the same context,
frame, input-readback and provider checks; a completion marker is required
exactly once across either spelling.

Stable `colour.*` case IDs are preserved. Both protocol files have new
American-spelled paths with exactly the same bytes as their frozen originals.
The originals remain for historical links and the controls' `base_protocol`
reference. Their SHA-256 values are:

| Protocol | SHA-256 |
|---|---|
| `color-signal-protocol.json` | `43943ce35ce913c1854fa40c3e4027fa2bd1aa9da67dbab1d768b0090705094f` |
| `color-signal-controls-protocol.json` | `b2f2a3b64c36d23f8ad2103b6aa59d35e5e28a91cda98748412fd3e8a23f935e` |

The numerical generators, transfer inverses, input/output audits, hypotheses
and arm definitions are unchanged. Completed evidence and recorded failures
were not rewritten. Historical source links and reproduction commands remain
usable through the forwarding files.

## Observed checks

| Check | Observation |
|---|---|
| Canonical Python test entry point | 12 tests pass |
| Recorded legacy Python test entry point | The same 12 tests pass; broad discovery does not duplicate them |
| Rust environment compatibility tests, current compiler and Rust 1.88.0 | Three tests pass on each; the GPU test remains ignored during ordinary unit execution |
| Clippy, all wrapper targets with `movable` | Passes with warnings denied |
| Rust formatting and diff checks | Pass |
| Native initial signal profile, canonical command | All 60 cases pass, including independent C++ ABI compilation, rebuilt helper and 144 GPU frames |
| Native static/reset profile, legacy command alias | All 54 cases pass, including rebuilt helper and 126 GPU frames |
| Legacy selector with only old environment names | Three additional sRGB frames pass metadata, input-readback and finite-output checks |
| Both helper export names | Both resolve from the rebuilt DLL; null input returns `E_INVALIDARG` through each |
| Contradictory old/new encoding values | Rejected before context creation |
| Full Python infrastructure suite | Final rerun: 192 tests pass, four skipped; 196 discovered |

The first infrastructure run exposed the old export-consistency test's
assumption that every export has a current Rust caller. It rejected the retained
`gpu_set_colour_signal` alias. The test now permits that one explicit compatibility
export and still requires its canonical target to have a current caller. Both
targeted native-equipment contract tests pass after the correction.
The subsequent complete infrastructure rerun also passes, with four skipped
tests reported separately from the 192 passing tests.

The two native profiles retain 15 initial same-code hypothesis failure frames
and zero static/reset-control failure frames, matching the earlier bounded
observations. Integration passing does not turn those failed predictions into
accepted signal semantics. No numerical budget, quality gate, wrapper API or
platform support claim changes.

Compact [summary](2026-10-15-color-signal-naming-verification/summary.json),
[initial case outcomes](2026-10-15-color-signal-naming-verification/initial-run.json)
and [control case outcomes](2026-10-15-color-signal-naming-verification/controls-run.json)
are retained. Full local logs/readbacks remain under ignored `target/`; those
files are not required to understand the reported outcomes. Native runs preceded
the final consistency-test and documentation edits; their workload source was
unchanged afterwards.

## Reproduction

With the pinned SDK already prepared locally, run from the repository root:

```sh
python -m unittest discover -s tools/verification/tests -p test_color_signal.py -v
python -m unittest discover -s tools/verification/tests -p test_colour_signal.py -v
python -m unittest discover -s tools/verification/tests -q
cargo test -p fsr-sdk --features movable --offline --locked color_signal
cargo +1.88.0 test -p fsr-sdk --features movable --offline --locked color_signal
cargo clippy -p fsr-sdk --all-targets --features movable --offline --locked -- -D warnings
cargo fmt --all -- --check
python tools/verification/run.py run experiment-color-signal --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-colour-signal-controls --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The full Python suite needs its documented NumPy/PowerShell prerequisites and
working temporary-directory access. SDK/runtime acquisition is separate; these
commands do not select a new SDK download or change its trust policy.

To repeat the direct legacy-native check, use the first profile's staged
`native/colour-fsr_sdk.exe`, helper and AMD DLLs, with its
`readbacks/colour.neutral-srgb.r1/input` directory and a fresh output directory.
Clear inherited `FSR_SDK_*` controls, then supply the standard `FSR_SDK_TEST_DLL`,
`FSR_SDK_TEST_GPU_DLL` and adapter-index values from that stage, plus:

| Legacy variable suffix after `FSR_SDK_COLOUR_` | Value |
|---|---|
| `ENCODING` | `srgb` |
| `AUTOMATIC` | `0` |
| `PRE_EXPOSURE` | `1,1,1` |
| `EXPOSURE` | `1,1,1` |
| `RESET` | `1,0,0` |
| `INPUT` | Absolute path to the prepared input directory |
| `OUTPUT` | Absolute path to the fresh output directory |

Run the staged executable with `--ignored --exact
upscaler::dx12::native_dispatch::native_colour_signal --nocapture
--test-threads=1`, using the native stage as its working directory. It must
report one passing test and three validated frames. Adding
`FSR_SDK_COLOR_ENCODING=pq` to the old `srgb` configuration must fail with the
conflicting-values message before a context is created.

The direct export check loads the staged helper and calls each upload export
with `(nullptr, nullptr, 0, 1.0f)`; both must return `E_INVALIDARG`. No GPU work
is needed for that invalid-input control. Source-only alias checks do not replace
native execution; the results above separately identify both forms of evidence.
