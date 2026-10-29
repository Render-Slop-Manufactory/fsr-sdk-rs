# Integrated API transition verification

- Date: 2026-10-27.
- Scope: authorised U12 under [D042](../adr/D042.md), wrapper 0.3.0 / sys 0.1.2,
  Rust 1.88+, pinned SDK v2.3.0, supported Windows x64/MSVC + DX12.
- Result: **U12 implemented and verified within the recorded configuration**.
  The final confirmation passes all 625/625 required cases, with unchanged inputs
  and independently verified artefact integrity.
- Follow-up: [P08](../../ROADMAP.md#performance) is separately required by
  [D043](../adr/D043.md) before final release qualification. This record claims
  no performance result, accepted final archives or publication.

## API consistency assessment and changes

The review covered exported wrapper types and methods in runtime/query/error,
portable upscaler values, both DX12 owner/setup families, resource builders and
explicit callback tables, plus optional utilities. The existing consumer inventory
already exercises feature gates, private checked fields, removed API names,
non-exhaustive enums, traits and borrowed-resource escape rejection.

| Area | Assessment and disposition |
|---|---|
| Constructors | Keep checked new -> Result and complete-value constructors. No blanket try_new rename or duplicate facade; required inputs stay explicit. |
| Getters/setters/builders | Getters observe, set_* mutates a borrowed receiver, with_provider consumes/returns a changed setup, resource builders consume/return assembly. Both families use the same mechanics. |
| Standard traits | Small checked values, IDs/masks and snapshots retain existing Copy/equality where appropriate. Frame/options/resource aggregates remain Debug + Clone without a blanket Copy/Default commitment. Sharpening remains extensible Clone. Native owners remain neither Clone nor Copy; main Error owns sources without Clone/Copy/Eq. No derive is added without a consumer need. |
| Discard diagnostics | Add targeted must_use to both resource-builder types, setup with_provider and both owner usability getters. Result already diagnoses discarded fallible calls. Ordinary mutating setters and marking remain unit-returning, without a blanket annotation pass. |
| Enum evolution | Open choices, diagnostics, errors, roles and terminal status remain non-exhaustive, preserving room for SDK/backend additions. Make DebugLevel non-exhaustive as well: future SDK filters can then extend this portable choice consistently with the other request enums. A negative consumer case requires a fallback. Private-field structs can grow internally. No new variants/defaults or native mappings are introduced. |
| Borrowing and ownership | Integrated flows need no new lifetime bounds, clones of native owners, generic ownership trait or conversion. Setups borrow runtime/device while returned contexts retain dependencies independently. Resource assembly only borrows; it does not AddRef or prolong GPU use. |
| Documentation | Replace the mixed historical/development package README with one 0.3 entry point, retain licence/provenance text, add a migration guide and complete example, fix obsolete preflight/reset terminology and document local abandonment consistently. Root README remains the published 0.2 baseline. |

A consuming with_provider call does not silently mutate a reusable old setup:
the old value is moved. The new warning catches accidentally discarding the
returned replacement. Builder type annotations cover intermediate consuming
steps; usability annotations cover pure snapshots. Explicit let _ discard remains
possible and grants no GPU readiness or safety. Six negative external cases use
deny(unused_must_use); one positive case permits deliberate discard.

No native ABI, dependency, package version, flags, resource mapping, mandatory
inspection, first-cause policy, failure-retention mechanism or Send/Sync
implementation changes. No universal backend/ownership/policy trait or automatic
wait, submission, unregister, rollback or recovery service is added.

## Complete external consumer flows

[upscaler_workflows.rs](../../crates/fsr-sdk/examples/upscaler_workflows.rs) is copied
from the package examples into an isolated downstream crate. Concrete local and
movable modules each expose:

1. first_frame: create once, retain the owner in the caller's Option before
   recording, invoke renderer completion only after record success, and leave
   the owner available on success/error/unwind. It outlives setup/public runtime.
2. selected_provider: one explicit selection through planning/create, preserving
   fresh membership and no fallback. generation_choice additionally composes the
   selected optional utils resolver for each family.
3. record_then_abandon: attempt recording, explicitly abandon the entire list,
   preserve the first terminal cause, invalidate executable recordings, complete
   earlier GPU work, then destroy once. Fallible callbacks leave a terminal
   retained owner; consuming destroy errors leave None and use existing retention.

Portable prepare_frame/update_frame take checked current jitter/time/camera and
explicit history, with no backend/runtime dependency. The bind helper assembles
named resources with explicit exposure. The migration guide distinguishes published
0.2 source changes from earlier unreleased APIs and explains required frame
updates, unsafe acquisition, native error ownership and renderer cleanup duties.

Eight named external cases require all six core entry points and both utility
compositions with actual typed calls. The example and complete README/Rustdoc
snippets also compile under all four supported native example feature combinations.
Stable/MSRV coverage, portable builds and existing negative ownership/trait/gate
cases remain required. These consumer checks compile functions; main starts no
GPU work. Existing controlled/native tests separately execute production paths.

## Reproduction and preparation

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/api-transition-verification
python tools/verification/repository_size.py
```

Full-feature Clippy and warning-free workspace Rustdoc pass during preparation.
An isolated preflight runs the actual example producer and all seven new discard
cases. Its first generated call-site wrapper incorrectly tried to name an impl
Trait parameter with turbofish syntax; ordinary typed wrapper calls correct that
harness error without changing the library or example API. All eight corrected
entry-point wrappers and seven discard cases pass. The package README and all
example feature combinations compile in that external preflight.

The first captured full attempt precedes the final restoration of the README's
complete fixture/Windows licence provenance paragraph and the final DebugLevel
extensibility correction. It is preparation evidence;
a fresh unchanged-input confirmation is required before acceptance. No stale
source identity or preparation failure is relabelled as a pass. The same
623-case preparation attempt also rejected the twelve expected lint failures
(six on each toolchain): the old generic diagnostic matcher only recognised
error[E-number], not lint diagnostics. The compiler emitted the intended errors.
The runner now checks Cargo JSON compiler-message records for the exact lint code,
error severity and nonzero exit. An end-to-end infrastructure test rejects wrong
codes, warning severity, successful exit, echoed source text and non-compiler
records. The corrected actual consumer producer passes all eight new cases in
preflight. One targeted test invocation initially lacked the fixture import path;
the correct test-directory invocation then required the permitted repeat for
restricted temporary-file writes and passes. Native cases in the first full
attempt were blocked by the source gate, not observed GPU failures.

## Draft package content inspection

`cargo package --list -p fsr-sdk --allow-dirty --offline --locked` includes the new
example, current README and licence files. Workspace packaging also succeeds:

```powershell
cargo package --workspace --no-verify --allow-dirty --offline --locked --target-dir target/api-transition-package-preview
```

The existing archive reader checks member safety/uniqueness. README, LICENSE and
LICENSE-AMD bytes in both draft archives match their source files; the wrapper's
new example also matches exactly. The wrapper MPL licence remains byte-identical
to the root licence. The complete constructor-fixture header/commit attribution
and Windows dependency terms remain in the package README.

| Draft archive | Files | SHA-256 |
|---|---|---|
| fsr-sdk-0.3.0.crate | 160 | `7d0b90df50c4902dd1afab48d58336cb57d05aef8804a853735ba988d43f6919` |
| fsr-sdk-sys-0.1.2.crate | 33 | `05fba7838b4c947b5ee9950e97ac1b2aa0091150057959e2072b29a682e549a5` |

These ignored target artefacts are content previews made with no-verify, not an
accepted final pair or package/native/registry execution result. Packaging the
workspace supplies the local raw dependency during assembly; the earlier
single-wrapper offline packaging limitation does not prevent these previews.
Final metadata/link changes still require candidate regeneration and qualification.

## Captured confirmation

Accepted attempt:
`20261027T180350Z-windows-upscaler-followup-f7f6edd3ffb3` under ignored
`target/api-transition-verification/`, 2026-10-27 18:03:52–18:12:51 UTC.
Status passed, completed 625/625, inputs_unchanged true. Independent verify_run
checks the artefact inventory/hashes, required case completeness and agreement
with all saved outcomes. Reproduction:

```powershell
python tools/verification/run.py verify-run --run target/api-transition-verification/20261027T180350Z-windows-upscaler-followup-f7f6edd3ffb3
```

| Identity | SHA-256 |
|---|---|
| Repository-only source | `4ed1fb98dabb2e4805bfbc9f3f38c7b28b0984dc81af2e8a2dee5e24ea1c7804` |
| Captured repository plus SDK source | `97daadd2c5c20b1a7e6f0a124d0cf27a29d96bfab04aef4ff36c202026b289d4` |
| Run inputs including configuration/environment | `a815512ad7d8684d085fcbc765578c6e4c99dbf2ff6947d7c434f40180ccf343` |

The preparation attempt is
`20261027T173409Z-windows-upscaler-followup-565378ddfcba`: failed, 623 required
outcomes recorded, inputs_unchanged false. Twelve lint-case evaluations fail for
the matcher issue above; dependent native cases are blocked. This attempt is
retained separately and is not part of the accepted result.

The final 625-case inventory includes supported feature combinations,
formatting/Clippy/Rustdoc, verification infrastructure and archive-integrity
controls, stable and Rust 1.88.0 consumers/examples/dependency graphs, independent
pinned-header ABI checks and the existing public query/lifecycle/dispatch/
sharpening GPU regressions. Both toolchains pass all six named core walkthroughs,
both utility compositions, the six exact lint rejections, deliberate-discard
positive case and DebugLevel non-exhaustive rejection. Native Configure also
repeats both-family completed-frame -> recorded abandonment -> invalidation ->
destroy checks. This does not execute the example callbacks as a standalone renderer.

Equipment: Rust/Cargo 1.99.0 and exact MSRV 1.88.0, Python 3.14.1,
Windows 11 build 26300, MSVC 14.44.35207, Windows SDK 10.0.22621.0,
AMD Radeon RX 9060 XT adapter 0 (1002:7590), driver 32.0.32015.7029.
Ordinary DX12 debug validation is enabled, GBV disabled. The abandonment case
observes provider display name 4.1.2 * with opaque ID 0xf5a5ca1e81001002 on both
families; names establish no general capability or module provenance.
SDK headers are pinned at 60f4ea81909200d8542eca14dccb2628b763a9a3.
The staged DLLs have valid AMD Authenticode signatures and SHA-256:

| DLL | SHA-256 |
|---|---|
| amd_fidelityfx_loader_dx12.dll | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| amd_fidelityfx_upscaler_dx12.dll | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

Text/link/diff checks pass; the repository remains below its 50 MB content budget.
Earlier supplementary format/callback profiles keep their original source bounds;
they are not rerun or attributed to this source. No new native failure,
callback-quiescence, provider capability or performance guarantee follows.

## Final release coverage reconciliation

U12 integrates the source API. The following existing equipment must be reconciled
with the exact final package pair; a checkout pass alone is insufficient:

| Selected surface | Current equipment | Final archive obligation |
|---|---|---|
| Canonical API, portable values, errors/traits/gates | Consumer inventory, stable/MSRV feature matrix and complete example checks | Registry-only consumers already select the inventory; ensure all six new walkthroughs and both utility compositions are selected against packaged examples on both toolchains. Current release examples only select dx12/dx12,movable. |
| Provider resolver | Portable resolver/error cases and both-family utility workflows | Keep utils off by default; include feature-on/off, portable dependency isolation and complete feature-enabled examples in exact-package checks. |
| Recording, setup, queries, tuning, first cause and abandonment | windows-upscaler-followup, controlled fixtures and existing native regressions | Bind extracted-package builds and outcomes to the same archive hashes; preserve both families, rejection order, literal mapping and teardown. |
| Formats/views, explicit motion, active output, masks/HDR and generator access | Existing selected feature/native profiles and their bounded records | Select relevant final-package cases without treating older identities as current results; preserve 3.1.5 generator writes and 4.1.1/2.3.4 no-write limits. |
| Explicit callbacks/diagnostics and CB aggregate ABI | native-callbacks plus the production callback ABI exchange | Bind the public callback builder and raw declaration import to the selected wrapper/sys archives. The current separate source profile/CB fixture does not do so. |
| Performance | Historical P01–P07 equipment | Settle and execute P08 on frozen candidate inputs and review regressions; no measurement is supplied by U12. |

The actionable release checklist remains in
[0.3 preparation](0.3.0-preparation.md#archive-coverage-before-final-verification).
Final package README/version links, candidate generation, coverage bindings,
exact-archive/registry/MSRV/native verification, P08 and publication simulation
are separate closure work. No registry download, upload or publication is
performed here. SDK/backend/platform, native safety and callback-quiescence bounds
remain unchanged.
