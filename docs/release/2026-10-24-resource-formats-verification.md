# Remaining external format implementation verification

- Date: 2026-10-24.
- Scope: [D039](../adr/D039.md), U05, wrapper 0.3.0 / sys 0.1.2,
  SDK v2.3.0, supported Windows x64/MSVC + DX12 target.
- Result: implemented and verified within the recorded configuration; U05 complete for bounded SDK access.

## Implemented behaviour and evidence boundary

Both public owners now accept the complete applicable allocation/role matrix
in [the source accounting](../research/records/2026-10-24-src-remaining-upscaling-formats.md):
fixed typeless colour/motion/scalar/exposure/mask views, RGBA8 SNORM,
shared-exponent RGB and D16/D24 normalized depth. Load-only typed/typeless
RGB32F inputs explicitly use the existing native enum omitted by AMD's helper;
they cannot be output or UAV-capable. All 40 pinned native surface constants
are bound with Rust/MSVC assertions. Layouts and public method signatures are
unchanged. No arbitrary view override, software conversion, automatic encoding,
provider fallback, hidden GPU synchronization or image-analysis policy is added.

Shared inspection queries the fixed typed view after metadata/device checks,
preserves exact role/HRESULT errors, permits retry before native entry and
rechecks at recording. D16/D24 use depth-plane SRVs, independently of the
helper's collapsed surface metadata. Typed depth requires casting support.
R16 typeless depth/stencil production is rejected because its fixed view is
FLOAT. R32 typeless ordinary depth/exposure is distinguished from its existing
D32 depth-buffer route. Integer/generic format declarations do not widen roles.

Baseline `a6ca2b7` plus the inspectable raw constants, shared resource checks,
controlled fixtures, reusable native equipment, profile and documentation.
The [independent experiment](../research/records/2026-10-24-exp-remaining-upscaling-formats.md)
records source/device evidence and 119 separate C++ SDK contexts. Public
confirmation is separately recorded here; no FSR image equality or quality
criterion is imported from earlier format profiles.

## Setup and commands

SDK commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; four hash-pinned public
headers. `native.py` checks exact local AMD loader/provider DLL hashes and
signatures. Windows 11 `10.0.26300`, RX 9060 XT adapter 0, vendor `0x1002`,
device `0x7590`, driver `32.0.32015.7029`; ordinary debug layer, GBV disabled.
MSVC `14.44.35207`, Windows SDK `10.0.22621.0`, Python `3.14.1`, Rust/Cargo
`1.99.0` and exact MSRV Rust `1.88.0`. No network acquisition or publication.

```sh
python tools/verification/run.py run ci --require-passed --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run native-resource-formats --native --require-passed --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The [frozen protocol](../../tools/verification/resource-formats-protocol.md)
and `resource_formats.py::workloads` identify all 84 configurations. Each runs
on both public owners; 51 main configurations select 3.1.5/4.1.1 and 33 generator
configurations select 3.1.5. This is 270 contexts, 810 fenced submissions,
798 main SDK calls, 198 generator calls and 12 no-dispatch control frames.
Each context runs reset then two continuing frames and explicitly destroys.

## Checks and results

| Attempt | Status | Stages |
|---|---|---|
| `20261024T164832Z-ci-149118fffd42` | Passed; inputs unchanged | 182 |
| `20261024T163619Z-msrv-4393140859ca` | Passed; inputs unchanged | 159 |
| `20261024T164826Z-native-resource-formats-a7b599b98c16` | Passed; inputs unchanged | 280 |

The [compact evidence](2026-10-24-resource-formats-verification/summary.json)
retains complete outcomes, source/equipment identities and measurements. Final
artifact inventories validate independently. CI covers infrastructure, seven
feature configurations, formatting, ordinary tests, Clippy, Rustdoc, public
consumers/examples, feature/platform graphs and MSVC ABI. MSRV runs the
corresponding Rust/tests/consumer checks on exactly 1.88.0. The executable Rust
implementation/tests are unchanged afterwards. Python/protocol successful-payload
retention changed and was rechecked by final CI/native confirmation. The final
text-only encoding restoration below is recorded separately from those source
snapshots. Resource hashes are pooled in the JSON's `resource_sha256_table`;
each frame's indices identify its seven resources without repeating hash strings.

Controlled checks cover exact metadata, fixed-view queries, every required
support bit, exact failed-query HRESULT, role attribution, legal UAV inputs
without typed shader access bits, output store requirements, RGB32F rejection
of UAV/output, depth casting/flags, wrong-role formats, device/metadata precedence,
preflight/recording repetition, retry, terminal failure and reference balance.
Both ownership families execute these checks. Newly accepted formats replace
old negative controls with still-unselected RGBA8 UINT or RGBA16 UNORM.

Native acceptance checks provider identity, complete allocation/readback
inventories, pinned-header metadata parity with the explicit RGB32F exception,
unchanged uploaded inputs, finite floating output, complete active writes,
ordinary diagnostics, device status, fences and cleanup. Both generator inputs
are independently exercised, including legal UAV-capable allocations. Generated
masks are consumed after renderer-owned ordering/transition. D16/D24 standard
and reversed active geometry includes independent shader depth and copied
depth/stencil planes. Ordinary R32 typeless depth is tested separately without
DSV flags. No input/output/provider image equality, numerical reconstruction,
sharpening-effect or CPU mask prediction is evaluated.

## Earlier checks and corrections

The first focused build missed the RGB32 typeless DXGI import; compilation
failed and the explicit import was added. Initial ordinary library tests also
found old negative controls that now accepted the selected format, and an
incorrect reference-count snapshot before context acquisition in the new scalar
fixture. The controls/snapshot were corrected; all eight focused view tests then
passed. The broad ordinary test additionally hit restricted Windows temporary
file access in an existing compile-control test; final CI/MSRV use the approved
access needed by that test. These are not failed native format observations.
A focused Python invocation initially used the wrong module search directory;
running from `tools/verification` passes all five tests. Final CI repeats them.

The preliminary public smoke attempt
`20261024T163348Z-resource-formats-public-475c70bee482` passed its 49 stages,
including both owners, load-only RGB32F, shared-exponent output/generator UAV,
ordinary R32 typeless depth and D16/D24. Its separate source identity precedes
the final evaluator's additional retained depth-view/hash observations.

The first final CI attempt `20261024T163620Z-ci-3e889a31d5d3` remains
failed: 179 stages passed and three failed while writing Rust metadata/docs
because the local disk was full (Windows error 112). The first full public native
attempt `20261024T163616Z-native-resource-formats-fb11f69ec902` also remains
failed: 196 stages passed, 29 failed and 55 did not complete. The first failed
child completed two frame markers before readback persistence failed with the
same storage error; subsequent file creation failed with errno 28. This is not
reclassified as native format rejection or a passing whole run.

Approved cleanup removed only raw payload directories of the four completed/incomplete GPU attempts
created today, freeing about 6.9 GB. Case records, logs,
original inventories and compact measurements remain. The coordinator now
discards each successful exclusive child payload only after full evaluation
and capturing hashes/measurements, retaining failures for diagnosis. Final CI
and all public native cases were repeated, with unchanged inputs and no storage
failure. No library or native acceptance criterion changed for this correction.

## Retention, completion boundary and limits

Final text review detected UTF-8 text decoded through Windows' legacy code page
in the editing helpers. The introduced display-character corruption was repaired
across the affected current documentation. Three files in the final tested source
inventory changed: the crate README, verification README and two Rustdoc comment
lines in `dispatch/frame.rs`. No executable Rust, tests, Python coordinator,
protocol or ABI changed after final confirmation. The compact summary records
tested/final hashes and affected lines. Formatting passes, and Rustdoc with
`-D warnings`, `--features movable,utils --no-deps --offline` passes on current
Rust and exact 1.88.0 after restoration:

```sh
cargo doc -p fsr-sdk --features movable,utils --no-deps --offline
cargo +1.88.0 doc -p fsr-sdk --features movable,utils --no-deps --offline
```

Retain compact source/configuration/equipment identities, complete case outcomes,
capability observations, input/output hashes and per-frame writes. Native
logs and retained failed readbacks remain under ignored `target/verification/<attempt>/`; no source
snapshots or runtime binaries are distributed. Hashes cannot recreate old frames.

This matrix and the final checks pass alongside the already verified
D034/D035/D037/D038 slices, completing U05 for bounded SDK access. Final extracted release packages and R03/U04
remain separate. Results are bounded to this one deployment; optional UAV
capabilities and load-only RGB32F need the actual provider/device contract.
Existing 4.1.1/2.3.4 generator no-write results remain failures. No general access
bound, padding/alpha guarantee, image-quality promise or concurrency claim follows.
