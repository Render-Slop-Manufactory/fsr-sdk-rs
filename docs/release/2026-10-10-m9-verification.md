# M9 public native integration and release-candidate verification — 2026-10-10

## Result

**M9 is complete for the recorded Windows/DX12 configuration and local candidate.**
The development implementation and the unmodified extracted `fsr-sdk 0.2.0`
artifact each passed the selected [M9 integration protocol](2026-10-09-m9-native-sharpening-protocol.md):
three isolated process repetitions, 48 outputs per repetition, 144 outputs in
each run. Both runs also passed the required disabled lifecycle, single-frame,
three-frame and motion/jitter/history-reset regressions. There were no failed
native acceptance repetitions, automatic retries or relaxed image assertions.

The locally prepared candidate is `fsr-sdk 0.2.0` depending on `fsr-sdk-sys 0.1.1`.
Builds, fixtures, migration/consumer controls, Rust 1.88, warnings-denied Clippy
and Rustdoc, package/extracted-source checks, archive inventories and registry-only
consumers passed. The joint publication dry-run passed in sys-then-wrapper order;
both uploads were explicitly aborted by Cargo. **Nothing was published, committed,
pushed or tagged.** The published `0.1.0` remains the release baseline.

Successful recording, completed GPU execution, exact wrapper mapping and image
quality are separate findings. The original
[failed sharpening-effect experiment](../research/records/2026-09-30-exp-d014-native-sharpening.md)
is unchanged and remains failed. These M9 results add no quality guarantee,
general temporal-accuracy claim, GBV result or additional hardware support.

## Native inputs and execution

Both runs used the existing locally acquired signed SDK v2.3.0 deployment,
ordinary D3D12 debug layer, zero creation/dispatch flags, 320×180 render and
640×360 output, explicit exposure 1, depth 0.5, zero motion, 16.667 ms frame time
and the protocol's camera/jitter values. The frequency/slanted-edge scenes and
half-float upload conversion are unchanged. GBV and the recording control override
were unset. Each child retained the 180-second watchdog; each frame used the
equipment's existing 60-second fence wait. Every successful list was submitted
once, in context recording order, and fenced before the next frame.

| Input | Recorded value |
|---|---|
| Host/target | Windows 11 Pro, version 10.0.26300, build 26300; `x86_64-pc-windows-msvc` |
| GPU | AMD Radeon RX 9060 XT; vendor `0x1002`, device `0x7590`; hardware DX12 feature level 12_0 |
| Driver | `32.0.31041.1004` |
| Selected provider | Name `4.1.1`; ID `0xf5a5ca1e01001001` |
| Stable Rust / Cargo | `rustc 1.99.0 (b940084d7 2026-09-28)` / Cargo 1.99.0 |
| MSRV | Installed Rust 1.88.0 |
| MSVC / Windows SDK | Tools `14.44.35207`, compiler `19.44.35224.0`; SDK `10.0.22621.0` |
| PowerShell | 7.6.6 |
| Loader DLL SHA-256 | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| Upscaler DLL SHA-256 | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |
| AMD signatures | Both staged DLLs returned `Valid` |

The [development environment](2026-10-10-m9-verification/development-source/native.environment.txt)
and [artifact environment](2026-10-10-m9-verification/release-artifacts/native.environment.txt)
retain toolchain/environment details. Each run has its own source fingerprints,
header/helper/executable fingerprints, selected command and protocol hash,
per-child status and per-frame observations. No Git revision or external CI page
is needed to identify the tested inputs.

The [new runner implementation](../../crates/fsr-sdk/tests/native/sharpening_integration.rs)
constructs every frame with public `UpscaleResources` / `UpscaleDispatch`, sets
the public sharpening selection and calls public unsafe `Upscaler::dispatch`.
It uses no private descriptor patch or historical dispatch override. Only the
existing test-only provider-identification query uses internal access. The
historical sharpening runner and its effect assertions are unchanged.

For each scene, fresh switching A, switching B and always-Disabled contexts each
process eight ordered frames. Only frame 0 resets history. Frames 3–7 retain one
context through `Disabled → Native(0) → Native(0.35) → Native(1) → Disabled`.
Frame equipment and resource/list references remain retained through context
destruction. After successful fences, explicit terminal destruction succeeds,
all equipment is released and the live device is checked again.

Unexpected dispatch failure invalidates the never-submitted list before terminal
context destruction. Unknown GPU completion exits the isolated child while
preserving possibly in-flight dependencies. Neither failure route can produce
an acceptance marker. Existing controlled fixtures separately check unknown
native errors, poison-first rejection, preflight recovery and terminal retention.

## Observations and independent readback audit

In **each** run, all 144 recordings returned success, all 144 fences completed,
and all 144 readbacks had the expected 640×360 dimensions, 230,400 finite RGBA
pixels and 230,400 pixels whose output sentinel was replaced. Every frame had
zero ordinary debug errors and no device removal. All 18 context destructions
and their post-teardown device checks succeeded. The five prerequisite native
children also passed in each run. The integration repetitions each ran exactly
one selected test and exited 0 without a timeout.

Each output's unpadded red half-float payload is 460,800 bytes. All 288 payload
fingerprints are retained. The raw images remain local in their target stages;
they are not added as repository or crate binaries. The preserved records include
all per-frame observations and numerical comparisons needed to understand the
result; the workload and [audit script](2026-10-10-m9-verification/collect-evidence.py)
make the calculation reproducible.

An independent Python half-float audit recalculated all 78 diagnostic comparisons
per run over `x=[32,608), y=[32,328)` (170,496 samples), confirming logged bit
mismatch counts and f64 mean absolute differences. No equality or image-effect
threshold was used as a native acceptance criterion. Switching A/B were bit-equal
in every compared ROI on this configuration. Return to Disabled was also bit-equal
to the always-Disabled control at frame 7 in these runs; this remains an observed
result, not a guarantee about arbitrary temporal histories.

The first repetition's A-versus-Disabled ROI diagnostics were identical between
the development and artifact runs and across their repetitions:

| Scene | Frame/mode | Half-bit mismatches | Mean absolute red difference |
|---|---|---:|---:|
| Frequency | 3 / Disabled | 0 | 0 |
| Frequency | 4 / Native(0) | 162,937 | 0.00212254789140489 |
| Frequency | 5 / Native(0.35) | 163,721 | 0.00292916734655340 |
| Frequency | 6 / Native(1) | 170,354 | 0.03004050147426021 |
| Frequency | 7 / Disabled | 0 | 0 |
| Slanted edge | 3 / Disabled | 0 | 0 |
| Slanted edge | 4 / Native(0) | 98,787 | 0.00110541234861265 |
| Slanted edge | 5 / Native(0.35) | 170,247 | 0.00297079608963059 |
| Slanted edge | 6 / Native(1) | 170,246 | 0.00425082767331922 |
| Slanted edge | 7 / Disabled | 0 | 0 |

Enabled zero differed from Disabled on both selected scenes. The recorded
`M9_RECORD` scalar fields identify the selected public input; they do not inspect
AMD's internal processing. Exact descriptor forwarding remains established by
the deterministic scalar/descriptor and controlled-FFI tests, including negative
zero, invalid values, unchanged other fields and disabling after enabling.

Full observations and audit data:

- Development run: [native cases](2026-10-10-m9-verification/development-source/native-cases.json),
  [frames](2026-10-10-m9-verification/development-source/frame-observations.json),
  [comparison audit](2026-10-10-m9-verification/development-source/diagnostic-audit.csv).
- Artifact run: [native cases](2026-10-10-m9-verification/release-artifacts/native-cases.json),
  [frames](2026-10-10-m9-verification/release-artifacts/frame-observations.json),
  [comparison audit](2026-10-10-m9-verification/release-artifacts/diagnostic-audit.csv),
  [exact extracted-package inputs](2026-10-10-m9-verification/release-artifacts/test-package-inputs.json).

## Candidate preparation, versions and boundaries

The development checkout retains its `0.1.0` manifest versions and separate
accepted D013 implementation. A reproducible
[candidate preparation script](2026-10-10-m9-verification/prepare-candidate.py)
creates a plain source projection under `target/m9-release-20261010-final/source`.
It does not change another Git checkout or remove D013 work from development.
The [projection record](2026-10-10-m9-verification/projection.json) records every
input/output hash, omission and transformation.

The projection sets wrapper `0.2.0`, sys `0.1.1`, and the wrapper's sys requirement
to `0.1.1`. D014 permits retaining sys `0.1.0` when its raw source/API is unchanged.
Here the already accepted [D015 loader follow-up](../adr/D015.md#loader-organization-follow-up-2026-10-09)
changed raw implementation/manifest packaging and made `libloading` optional and
Windows/DX12-only. The patch version identifies those new source/dependency bytes;
it introduces no new ABI or native behavior and is not a bump merely to match the
wrapper. Third-party dependency versions/checksums and SDK pin are unchanged.

D013 is outside this release: the candidate has no `movable` feature or public
MovableRuntime/MovableUpscaler modules, reexports, example or public consumer tests.
Six corresponding files are omitted; remaining feature-gated fixture/block items
are explicitly inactive. Private Rc/Arc ownership infrastructure and its existing
tests are retained. Candidate Rustdoc omits the D013 public overview, and the
candidate README uses `0.2.0` in its explicitly unpublished install guidance.
Feature/API absence is checked by real downstream diagnostic controls. This
projection selects no later D013 release.

These are Cargo-generated, verified archives; no archive was manually edited:

| Artifact | Files | Bytes | SHA-256 |
|---|---:|---:|---|
| `fsr-sdk-sys-0.1.1.crate` | 28 | 33,054 | `4f29475b2aef3527072111bd0f4f5e7c1828c4881eaa72a3ba96d7e275f54465` |
| `fsr-sdk-0.2.0.crate` | 55 | 93,701 | `a8b293ec70c0b6e52897fba12e04b105154cd12caa95f18277ec6ed9dacd1276` |

Archives remain under `target/m9-release-20261010-final/build/package/`. The
[complete inventories](2026-10-10-m9-verification/archive-audit.json) preserve
every entry's size and hash. Byte-identical MPL licence copies, README files and
the raw crate's retained AMD licence are included. There are no packaged DLLs,
executables, compiled artifacts, SDK trees, source-tree runner scripts, escaping
paths, duplicate entries or symlinks. Focused scans found no personal absolute
paths in the archives or retained logs. Staged AMD DLLs remain local test inputs.
The MPL licence does not relicense AMD binaries.

The dry-run regenerated byte-identical archives. All projected source hashes,
archive hashes and **every extracted archived file** still matched after build,
tests, documentation, consumers and native GPU execution. Packaging from the
ignored projection includes no Cargo VCS record; the explicit file inventories
and projection hashes are its source provenance. A future regenerated or edited
candidate must be reidentified and verified before publication.

## Checks and reproduction

Run the native commands from an x64 Visual Studio developer shell at repository
root, using trusted local SDK inputs. `CARGO_INCREMENTAL=0` was set. Every Cargo
check used cached dependencies and `--locked --offline`, except the joint
publication dry-run needed registry index access. Rustdoc used `-D warnings`.

```powershell
./crates/fsr-sdk/tests/native/run.ps1 -Mode m9 -Native -Stage target/m9-20261010-attempt-2
cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings
./tools/check-movable-api.ps1 -Stage target/m9-api-controls
python docs/release/2026-10-10-m9-verification/prepare-candidate.py
cargo package --workspace --manifest-path target/m9-release-20261010-final/source/Cargo.toml --locked --offline --allow-dirty --target-dir target/m9-release-20261010-final/build
python docs/release/2026-10-10-m9-verification/verify-candidate.py
$previousCargoHome = $env:CARGO_HOME
$env:CARGO_HOME = (Resolve-Path target/m9-release-20261010-final/isolation/cargo-home).Path
./crates/fsr-sdk/tests/native/run.ps1 -Mode m9 -Native -Stage target/m9-20261010-artifact -TestManifest target/m9-release-20261010-final/isolation/packages/fsr-sdk-0.2.0/Cargo.toml
# Restore the ordinary Cargo home before the registry-index dry-run.
$env:CARGO_HOME = $previousCargoHome
cargo publish --dry-run --workspace --manifest-path target/m9-release-20261010-final/source/Cargo.toml --locked --allow-dirty --target-dir target/m9-release-20261010-final/dry-run --config net.retry=0
python docs/release/2026-10-10-m9-verification/collect-evidence.py target/m9-20261010-attempt-2 development-source
python docs/release/2026-10-10-m9-verification/collect-evidence.py target/m9-20261010-artifact release-artifacts
python docs/release/2026-10-10-m9-verification/save-artifact-evidence.py
```

Use fresh target stages in a fresh checkout; preparation/native stages refuse
overwriting earlier attempts. The verifier's `--resume` keeps completed checks
and prior failures when repairing its test environment; no candidate source or
acceptance threshold is changed. [All check attempts](2026-10-10-m9-verification/checks.json)
and the [38 final artifact gates](2026-10-10-m9-verification/artifact-summary.json)
are retained. The dry-run [log](2026-10-10-m9-verification/publish-dry-run-online.log)
records two aborted uploads.

| Gate group | Result |
|---|---|
| Formatting; default/no-default Clippy; default/no-default tests; default/no-default Rustdoc | Pass on projected candidate; warnings denied |
| Rust 1.88 all-target check, default tests and no-default tests | Pass on projected candidate |
| Deterministic wrapper and DLL fixtures | Pass: complete disabled descriptor, exact native sharpening bits, resource-role rejection/recovery, poison and terminal paths |
| Four pinned-header fingerprints and three MSVC ABI probes | Pass in each native run; no AMD runtime linked by those probes |
| Unmodified extracted wrapper/sys all targets, tests and Windows Rustdoc | Pass; extracted packages placed in their own test workspace |
| Independent registry consumer | Pass with exact `=0.2.0` / `=0.1.1` dependencies, no paths or patches; fresh lock/checks/MSRV; archive checksums confirmed |
| Complete consumer examples | Pass for the DX12 example and both full crate-root `no_run` examples, including hidden Windows setup |
| Consumer negative controls | Expected feature-resolution failure for movable; E0432 for absent movable/native imports; E0616 for a private frame field |
| Packaged production dependency trees | 12 passing Linux/Windows default/no-default/DX12 trees; loader appears only on Windows with DX12 |
| Current development Clippy with movable and full consumer controls | Pass; [diagnostic controls](2026-10-10-m9-verification/development-consumer-controls.txt) retain public-input/private-field/trait/family/dependency boundaries |
| Joint Cargo package and publish dry-run | Pass; both packages verified, two uploads aborted |
| Native public API and native extracted artifact | Both pass: five regression children plus three M9 repetitions each |

The consumer's isolated Cargo home contains a temporary local registry made
from the two exact candidate archives and 17 cached third-party archives/index
entries, each checked against the unchanged dependency lock checksums. Its
[manifest, source and lockfile](2026-10-10-m9-verification/consumer-inputs.json)
and [registry inputs](2026-10-10-m9-verification/registry-inputs.json) are retained.
Metadata confirms one consumer workspace member, registry source IDs, matching
archive checksums and no dependency paths/patches. The extracted-source tests
use a separate test workspace without modifying either normalized manifest.
Linux dependency-tree inspection establishes selection only, not Linux native
compilation or GPU support.

Final [documentation/evidence QA](2026-10-10-m9-verification/final-qa.json)
confirms local links, evidence syntax, unchanged historical/selected protocols,
unchanged historical effect runner and failed result, matching diagnostics across
repetitions/source/artifact runs and unchanged final archive hashes.

## Preserved unsuccessful setup attempts and limits

The first development attempt stopped **before any GPU child**: the sandbox
denied a temporary metadata-file write in an existing context compile test.
The unchanged test passed outside the sandbox. The first joint offline package
verification similarly could not initialize Cargo's temporary-registry cache;
the identical command passed outside the sandbox. No native acceptance failure
was retried or discarded.

Early candidate preparation exposed Rustfmt resolving removed inactive modules;
the preparation script now removes their inactive module/reexport declarations
and formats/records the result before packaging. Two extracted-package checks
initially found the enclosing development workspace; the verifier now supplies
an explicit test workspace without changing packaged manifests. A negative
feature test initially expected Cargo's plural diagnostic, while registry
resolution uses the singular form; its exact expected diagnostic was corrected.
These earlier check failures remain in the check record. The archive/candidate
contents were unchanged during those verifier repairs. A fully offline publish
dry-run requested HTTP index access and stopped; the authorized index-enabled
dry-run passed without uploading.

The [setup-attempt summary](2026-10-10-m9-verification/setup-attempts.json)
identifies those unsuccessful setup commands and their observed failures.

Bounded support remains one provider/GPU/driver/workload configuration. Existing
GBV dispatch-timeout evidence remains unresolved; no second GPU/provider,
performance benchmark or general image-quality validation was added. D013
native assumptions and its eventual release remain separate. Real registry
authentication/server acceptance, publication, hosted docs.rs and a public
`0.2.0` tag/release were not exercised. Publication still requires separate
authorization, in sys-then-wrapper order.
