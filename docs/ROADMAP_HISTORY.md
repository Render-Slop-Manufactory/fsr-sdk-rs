# Roadmap history

Completed checkpoints, grouped by release baseline. Current work lives in the
[roadmap](../ROADMAP.md); implemented contracts live in the ADRs and source.

On 2026-10-29 the maintainer selected compaction of superseded 0.1.0/0.2.0
release preparation, architecture checks and repeated verification exports.
This page retains their release identities, relevant observations and limits.
The old command/log collections remain in Git history; they are not current
verification recipes. No historical pass qualifies a later candidate. Original
SDK investigations, unresolved negative evidence and the current 0.3.0 records
remain separately available. One [M9 integration summary](release/2026-10-10-m9-verification.md)
keeps its existing public Rustdoc URL.
The original M4–M6 implementation-plan files are also retired: their module/API
representations have been superseded. The ADRs and original experiments linked
below retain the construction, lifetime and native-dispatch evidence.
Verbatim pre-run protocols keep their original intake references; dated notes
in their parent records explain navigation to retired plans. Their criteria and
byte identities are not rewritten to make historical references look current.

## V01 — Temporal/image baseline — completed 2026-10-12

Selected bounded checkpoint completed:
[quality/motion/disocclusion](research/records/2026-10-11-exp-temporal-quality-validation.md),
[continuous history](research/records/2026-10-11-exp-temporal-long-sequence.md),
[cut/reset](research/records/2026-10-11-exp-temporal-reset-recovery.md) and
[jitter sign/phase](research/records/2026-10-12-exp-temporal-jitter-phase.md).
The records retain measurements and limits; no release gate or universal quality
claim is added. Subsequent quality work is deferred under [scope](../SCOPE.md).

## Verification infrastructure — implemented 2026-10-10

Named source/MSRV/ABI/consumer/native/artefact and experimental profiles replaced
milestone launchers. The coordinator introduced exclusive attempts, captured
inputs, separate execution/evaluation, diagnostic-specific compiler controls,
offline analysis, safe archive extraction, isolated registry consumers and
integrity-checked export. Cargo compilation caches were reusable; case passes
were not. Actual publication remained separate from uploads-aborted simulation.
Current contracts and commands are in [verification](../tools/verification/README.md).

The original Windows acceptance recorded 289 passing cases with generated
archives and another 289 with selected archives, followed by a 164-case
publication simulation with identical regenerated hashes and both uploads
aborted. Fourteen infrastructure tests passed. Source/extracted native arms
each retained 144 sharpening-integration readbacks and 78 diagnostic comparisons;
the separate cleanup check passed 41 children, including 30 matrix children,
516 fenced frames and 96 selected payloads. These counts describe that dated
checkpoint, not today's case inventory or a new run.

The original sharpening-effect children remained failed and the GBV dispatch
remained timed out at 90 seconds. Corrected audit mistakes concerned selected
images versus executed frames and Cargo's `tmp-crate` lookup; the initial
mistakes were not native successes. The helper-residency control prevented a
vacuous unload observation. No Linux execution, general failure-recovery,
provider-unload or image-quality guarantee followed.

The [script archive](../tools/verification/archive/2026-10-10/README.md) retains
exact source payloads and their SHA-256 identities. Its historical intake
references are distinct from the current navigation to these compact summaries.
The 212-file old verification export is retired; its full logs and original
inventories are available only in Git history. No frozen script payload or
retained experimental measurement was rewritten during compaction.

## 0.2.0 — published 2026-10-10

Baseline: `fsr-sdk 0.2.0` with `fsr-sdk-sys 0.1.1`. The maintainer selected the
optional movable family for the actual release, superseding the earlier
movable-excluding M9 projection. Public package/download observations completed
the separate `R01` handoff after pre-publication verification.

| Checkpoint | Completed work | Retained explanation |
|---|---|---|
| M7 — Dispatch contract | Extensible frames and exact native sharpening; migration from public literals | [D014](adr/D014.md) |
| M8 — Frame implementation | Private resources/frame construction, checked scalars, descriptor forwarding and controlled failure checks | [Frame and sharpening integration](#frame-and-sharpening-integration) |
| M9 — Native integration and artefacts | Selected public/native and extracted-package execution; original projection excluded movable | [M9 summary](release/2026-10-10-m9-verification.md) |
| Movable ownership | Optional `Send + !Sync` family, coordinated handoffs and cleanup | [Movable implementation checkpoint](#movable-implementation-checkpoint) |
| Common inputs and loader organization | Portable values and isolated Windows/DX12 production loading | [Portable inputs and loader checkpoint](#portable-inputs-and-loader-checkpoint) |
| Combined release verification | Actual packages, Stable/MSRV registry consumers, native movable/sharpening and extracted-package GPU checks | [Release qualification](#release-qualification) |
| R01 — Published baseline | Downloaded registry bytes, tag provenance, hosted movable documentation and consumer checks | [Published package identity](#published-package-identity) |

### Published package identity

The public 2026-10-10 check observed both crates not yanked, GitHub `v0.2.0`
public/non-draft/non-prerelease, successful versioned Rustdoc and release CI.
The tag and both packaged VCS members identify
`fabb9640d22abec64b2f12e5c06eb89e57a9bce9`. Six actual-registry consumer
compilation gates and one provenance gate passed. These are dated observations,
not a fresh internet availability check.

| Published crate | Published UTC | Files | Bytes | Downloaded SHA-256 |
|---|---|---:|---:|---|
| fsr-sdk-sys 0.1.1 | 13:03:10 | 29 | 32,711 | `0c8949d411c7e470d99f3a186e1111f6303bea65d0b58b9e86491c450f291c0f` |
| fsr-sdk 0.2.0 | 13:03:27 | 62 | 106,130 | `b98e9c27c0266a998f7165cd28a57ccd4e0d7ce133d8d732778d6ac42b18ffec` |

Downloaded hashes matched the sparse index and crates.io API. The wrapper used
`fsr-sdk-sys ^0.1.1` with raw defaults disabled, default DX12 and optional movable;
both crates retained Rust 1.88 and MPL-2.0 with AMD notices. All source members
matched the public tag after line-ending normalization. Published versus final
candidate differences were Git provenance, CRLF/LF and the wrapper's generated
lockfile sys checksum; no other parsed lockfile difference was observed.

Registry and hosted references:
[wrapper](https://crates.io/crates/fsr-sdk/0.2.0),
[raw crate](https://crates.io/crates/fsr-sdk-sys/0.1.1),
[wrapper Rustdoc](https://docs.rs/fsr-sdk/0.2.0/fsr_sdk/),
[raw Rustdoc](https://docs.rs/fsr-sdk-sys/0.1.1/fsr_sdk_sys/),
[release tag](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.2.0).

### Release qualification

The final complete candidate passed 64 package/source/consumer gates, including
12 expected compiler failures. Stable/MSRV source and extracted-package checks,
warning-denied Clippy/Rustdoc, exact-version isolated-registry consumers and full
examples passed. The same candidate included public movable ownership; the
earlier M9 projection did not. Joint package/publication simulation ran raw first,
then wrapper, regenerated identical candidate bytes and aborted both uploads.
That check itself did not publish.

| Final pre-publication candidate | Bytes | SHA-256 |
|---|---:|---|
| fsr-sdk-sys 0.1.1 | 33,188 | `88cc65fcdd46905e4ff4ab55d7a127e260700493bdc281c1686523d05864a883` |
| fsr-sdk 0.2.0 | 106,953 | `076020dc8e0f28f3acabd37ed7dc17cdebc0e3b07ae942cbba44df10502faefc` |

Source and extracted-package native arms each passed seven isolated children:
explicit/Drop lifecycle, one-frame and three-frame dispatch, motion/jitter/reset,
public movable and public sharpening. The extracted candidate additionally
passed three selected M9 repetitions. Licence/README/source inclusion, archive
integrity and bounded private-data screens passed; no DLLs/native build products
were included. Source, archive acceptance and actual publication are distinct.

### Portable inputs and loader checkpoint

October 9 separated portable checked dimensions/options and their errors from
Windows/DX12 resources, contexts and loading. The native re-exports then preserved
the preceding imports. Later D042 intentionally replaces several wrapper shapes;
this checkpoint is not a current compatibility promise.

Raw `libloading` became optional under Windows, activated by `dx12`; normal
Linux/common-only production dependency trees omitted it. Native layout checks,
Windows feature configurations, denied-warning documentation and 60 local
consumer controls passed in the loader source audit; the portable-value native
refresh separately passed six isolated children and four positive/eighteen
negative consumer controls. Cross-target compilation was not Linux execution.

The loader audit found acquisition, five exact exports, error mapping and DLL
ownership preserved by the private Windows module move. The loader retained no
graphics device; contexts retained runtime/device independently. This supported
[D015](adr/D015.md)'s concrete backend separation, not generic loading traits,
native Unix support or an assumption that Windows means DX12 forever. Provider
termination, arbitrary thread compatibility and future Vulkan acquisition/ABI
still required separate evidence. Current findings live in
[platform/backends](research/platform-and-backends.md).

### Movable implementation checkpoint

October 9 public integration verified local `!Send + !Sync`, optional movable
`Send + !Sync`, no Clone and no automatic family conversion. Six controlled
public consumer cases covered retained device/runtime ownership, siblings,
failed handoffs and caught unwind; positive controls preceded exact negative
diagnostic checks. Stable and Rust 1.88 compilation/tests passed. The fixture
compiler itself remained the installed default Rust 1.99, not an MSRV-native
fixture-compiler claim.

The six-child native refresh included the actual public movable family: loader
acquisition, creation, two fenced dispatches and sibling destruction occurred
on coordinated distinct threads under one caller-owned mutex domain. Final
destruction during caught Rust unwind and after failed handoff returned native
code zero. Both 640x360 readbacks had all 230,400 pixels finite/written, zero
reported debug errors and no device removal. A weak observer could no longer
upgrade after final cleanup; that proved application runtime-reference release,
not provider/dependency termination or a DLL-unload thread.

The CI shell initially propagated an expected compiler failure despite validated
controls. The corrected wrapper returned zero only after validation; simulated
positive-control failure, unexpected negative success and wrong diagnostics
still failed. Initial restricted Temp write failures remained failed setup
attempts; unrestricted reruns passed without relaxing native gates.

[D013](adr/D013.md) retains caller-established native thread compatibility,
external-user coordination and completion obligations. These checks add no
concurrency, outstanding-GPU-work transfer, universal recovery or arbitrary
provider guarantee. The original
[transfer/cleanup experiments](research/records/2026-10-08-exp-d013-native-transfer-cleanup.md)
remain the deeper experimental evidence.

### Frame and sharpening integration

M8 used private resource/frame fields, checked finite sharpness in `[0,1]`,
exact `f32` forwarding including signed zero, `Disabled -> (false, +0.0)` and
enabled zero distinct from Disabled. Controlled DLL tests covered the complete
descriptor, unchanged other inputs, invalid-input rejection before native calls,
unknown native errors, poison-first rejection and terminal cleanup. Metadata
role checks rejected incompatible formats/extents; equal-metadata swaps remained
caller semantic obligations. These were source/controlled checks, not AMD GPU
or reconstruction-quality proof.

The later [M9 integration summary](release/2026-10-10-m9-verification.md)
retains the selected public same-context sequence, output/cleanup criteria and
observations. Its execution gate intentionally differed from the unchanged
[failed original effect experiment](research/records/2026-09-30-exp-d014-native-sharpening.md).
There is no new sharpening-gain, brightness, monotonicity or universal enabled-zero
identity guarantee. Current contracts follow D014/D016 and D042, not the retired
0.2.0 constructor examples.

### October 9 native diagnostic refresh

The regression refresh used Windows 11 x64/MSVC, RX 9060 XT (`1002:7590`,
feature level 12_0), driver `32.0.31041.1004`, provider 4.1.1
(`0xf5a5ca1e01001001`) and SDK v2.3.0 headers at
`60f4ea81909200d8542eca14dccb2628b763a9a3`. DLL SHA-256 values were:

- Loader: `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler: `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.

Both Windows signatures were Valid. These are shared deployment identities,
not proof that every checkpoint executed the same workloads.

The disabled native regressions passed, including motion/jitter sensitivity and
history-reset controls. Public movable exercised two fenced frames and coordinated cleanup.
Raw provider layouts A/C found two versions; B/D returned `NO_PROVIDER` (4)
with count zero. Sequential shared-runtime queries/destroys passed while provider
residency persisted beyond loader drop. No provider-termination attribution followed.

Important negative results remain:

- All three original sharpening-effect repetitions failed their unchanged effect
  assertion. Native acknowledgement/output differences were not promoted to
  image-quality acceptance; the original numerical record remains linked above.
- The no-dispatch GBV control passed, while native GBV dispatch timed out after
  90 seconds and required termination. A successful diagnostic parent exit
  did not mean dispatch passed. [Original GBV evidence](research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md) remains.
- Three raw allocation-failure cases aborted with `0xC0000005` before create
  returned, with no observed frees; these were process faults, not FFX error
  returns or successful recovery. [Allocation evidence](research/records/2026-09-20-exp-create-allocation-failure.md) remains.
- Raw zero-dimension cases aborted with `0xC0000409`/foreign-exception diagnostics;
  unusual positive-dimension acceptance did not establish dispatch support.
  [Dimension evidence](research/records/2026-09-23-exp-m5-dx12-dimension-boundary.md) remains.
- The returned-create-error lifetime probe passed its control and retain/protect/
  cleanup variants after a runner marker fix. The earlier marker failure was
  not native failure evidence; the bounded successful trigger did not establish
  universal failed-create safety. [Lifetime evidence](research/records/2026-09-22-exp-create-error-input-lifetime.md) remains.

No second GPU/provider, general concurrency, provider/dependency shutdown,
unknown-completion teardown, complete numerical domain or universal native
failure cleanup was established. These limitations remain relevant to the
current [safety synthesis](research/api-and-safety.md).

## 0.1.0 — published 2026-09-28

Both crates shipped as `0.1.0`, followed by public GitHub `v0.1.0` at
19:47 UTC. The September 29 observation found both registry entries not yanked
and both Windows-target Rustdoc roots available.

| Published crate | Published UTC, September 28 | Registry-reported SHA-256 |
|---|---|---|
| fsr-sdk-sys 0.1.0 | 19:30:01 | `a7e0ba27a9cc3b95dcb8c4d26c37bc04b2961babd6cd089d63ecde0e705f734e` |
| fsr-sdk 0.1.0 | 19:34:03 | `4a20a64583400cbc39cadb72b73d05c7ba6e6367eedb91fc39ae203aa20a5314` |

The wrapper depended on `fsr-sdk-sys ^0.1.0` with raw defaults disabled and
default DX12. The public check read registry/API/Rustdoc responses; it did not
download or compare the published archives. The preceding verified local pair
had different hashes: raw `8c67214edf3cabfc577eace6b9798b37a9febf9898bd675495482378c1662cb0`
and wrapper `4eea0938c5da942d4b4f151cd2f989b6a95ca6629c24c629f59a92ab02c6eeda`.
Its sys/workspace package verification and joint dry-run passed with both uploads
aborted, alongside isolated-registry and recorded workspace/Windows checks and retained licence
notices. Those local candidate bytes were not certified as registry-identical.

| Checkpoint | Completed work | Retained evidence |
|---|---|---|
| M1–M3 | Selected common ABI, explicit loading and provider/version queries | [ABI inventory](ABI_COVERAGE.md) |
| M4 | Runtime retention and terminal native context cleanup | [Windows experiment](research/records/2026-09-22-exp-m4-windows-verification.md) |
| M5 | Checked construction, runtime/device retention and owned errors | [Native experiment](research/records/2026-09-24-exp-m5-windows-native-verification.md) |
| M6 | Fixed-profile recording, submitted/fenced readback and moving smoke | [Public dispatch](research/records/2026-09-25-m6b-public-dispatch-verification.md), [final native check](research/records/2026-09-28-exp-final-windows-release-verification.md) |

These establish the original Windows/DX12 path, not complete SDK coverage or
general provider, performance or image-quality support. The old `load_unchecked`,
frame literal and milestone-runner examples are retired documentation, not
instructions for the current API. Use [migration](MIGRATING_0_3.md) and
[development](DEVELOPMENT.md) for current entry points and commands.

### September maintenance and test hygiene

The September 30 reorganization separated ownership/failure tests and shared
native equipment without changing production `Rc` ownership; manual `Arc`
transfer was still experimental. Source/consumer/ABI checks and selected native
regressions passed on the recorded SDK v2.3.0 deployment. Dispatch semantic
measurements remain in the [original experiment](research/records/2026-09-30-exp-dispatch-semantics.md).
One Cargo 1.98.1 standalone wrapper package attempt failed with its internal
`no hash listed for fsr-sdk-sys` condition; successful joint package verification
did not establish a general Cargo fix.

The October 7 hygiene follow-up recorded 52 passing default-feature checks and
29 feature-disabled checks, plus collision-safe exclusive fixture directories.
An initial Temp metadata-write AccessDenied remained a failed environment attempt;
the authorised unrestricted rerun passed. D013 transfer, D014 effect failures
and provider-termination uncertainty retained their separate evidence. These
old maintenance passes neither verify today's API nor require preserving another
copy of an obsolete development guide.
