# API and safety

- Updated: 2026-10-09 (D013 review, acceptance and separate public implementation verification recorded; D014 acceptance retained).
- Scope: common ABI, loading, provider deployment, version enumeration, context
  lifecycle contracts and isolated create-failure experiments; M5 construction,
  Query/Configure, runtime-sharing source findings and bounded signed-runtime
  experiments for FidelityFX SDK v2.3.0; post-dispatch teardown observations;
  the broader modern DX12 upscaler API surface and design implications;
  D013 trait compilation, serialized macOS/Windows fixtures and fenced native
  Windows/DX12 transfers, native Rust-unwind cleanup and staged residency,
  transfer/cleanup source limits, Windows failure/unwind fixtures and a supplied
  soundness/architecture review;
  D014's test-only native sharpening experiment; and older
  FSR2 Rust binding comparisons. AMD execution/native layout evidence is limited to
  Windows x64/MSVC; D013 fixture execution also covers macOS ARM64. Native D013
  transfer and loader-release observations cover one identified AMD configuration.
- Included evidence: [common ABI verification record](records/2026-09-20-exp-common-abi-verification.md)
  and [explicit loader verification](records/2026-09-20-exp-explicit-dll-loader.md),
  plus the [version-count query](records/2026-09-20-exp-version-count-query.md),
  [DX12 provider-DLL resolution](records/2026-09-21-src-dx12-provider-dll-resolution.md),
  [signed DX12 directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md),
  [lifecycle contract refinement](records/2026-09-22-src-lifecycle-contracts.md),
  [source provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md),
  [distribution/artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md),
  [v2.3.0 lifecycle investigation](records/2026-09-20-src-context-lifecycle.md),
  [destroy failure-path investigation](records/2026-09-20-src-destroy-failure-paths.md),
  [successful DX12 lifecycle](records/2026-09-20-exp-context-lifecycle.md),
  [M4 Windows production-owner verification](records/2026-09-22-exp-m4-windows-verification.md),
  [create-allocation failure spike](records/2026-09-20-exp-create-allocation-failure.md),
  [returned-create-error input lifetime spike](records/2026-09-22-exp-create-error-input-lifetime.md),
  [Embark fsr-rs comparison](records/2026-09-20-src-embark-fsr-rs.md) and
  [fsr2-rs raw-binding comparison](records/2026-09-20-src-fsr2-rs.md),
  [M5 source provenance repair](records/2026-09-22-src-m5-construction-query-provenance-repair.md),
  [M5 numerical construction investigation](records/2026-09-23-src-m5-numeric-construction-safety-contract.md),
  [signed-runtime dimension probe](records/2026-09-22-exp-m5-upscaler-construction-inputs.md),
  [signed-runtime DX12 width-boundary probe](records/2026-09-23-exp-m5-dx12-dimension-boundary.md),
  [sequential shared-runtime probe](records/2026-09-22-exp-m5-shared-runtime-provider-lifetime.md),
  [M5 Windows native verification](records/2026-09-24-exp-m5-windows-native-verification.md),
  [final Windows verification](records/2026-09-28-exp-final-windows-release-verification.md),
  the [DX12 upscaler API surface investigation](records/2026-09-29-src-dx12-upscaler-api-surface.md),
  the [D013 compile/host experiment](records/2026-09-29-exp-d013-movable-compile-host.md),
  [D013 Windows fixture execution](records/2026-09-29-exp-d013-windows-fixtures.md),
  [D013 native transfers](records/2026-09-30-exp-d013-native-thread-transfer.md),
  [D013 transfer/cleanup source intake](records/2026-10-07-src-d013-transfer-cleanup-contract.md),
  [D013 transferred failure/cleanup fixtures](records/2026-10-07-exp-d013-transfer-failure-cleanup.md),
  [D013 extended native transfers and cleanup](records/2026-10-08-exp-d013-native-transfer-cleanup.md),
  [D013 soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md),
  and the [D014 native sharpening spike](records/2026-09-30-exp-d014-native-sharpening.md).

## Topic map

Detailed findings are split by the contract they explain:

- [runtime loading](runtime-loading.md)
- [platform and graphics backends](platform-and-backends.md)
- [context lifecycle](context-lifecycle.md)
- [upscaler construction](upscaler-construction.md)
- [runtime threading](runtime-threading.md)
- [upscaling dispatch](upscaling-dispatch.md)

## Current findings

**Verified source facts:** `ffx_api.h` supplies the original common boundary:
context and scalar aliases, eight return constants, the base descriptor header
and four aliases, allocation callbacks and the five entry-point pointer types.
Resource and extended descriptors are not required to declare those signatures.
Context parameters are pointers to handles; allocation sizes and descriptor tags
are 64-bit unsigned integers. Create/query descriptor pointers are mutable;
configure/dispatch descriptors and allocation-callback struct parameters are const.
See the record's header findings and pinned local-header fingerprint.

**Verified test results:** Rust checks and a separate native C++ compile passed
the shared layout, constant and signature expectations. On the tested target the
base header is 16 bytes/alignment 8 and the allocation-callback struct is 24
bytes/alignment 8. The C++ file is a compile-only verification probe, not part of
the library or its Cargo build. Reproduction commands and setup failures are
preserved in the record.

**Upstream contracts:** pointers passed in creation descriptors must remain live
through destruction (the lifetime of chain-node storage is ambiguous; see below); allocation/deallocation have alignment, null and compatibility
obligations. Global query/configure allow a null context pointer. These comments
are evidence of the declared contract, not observed runtime behavior. Nullable
Rust function-pointer representation does not authorize passing null callbacks.

**Inference:** the paired checks support the selected Rust ABI mapping on Windows
x64/MSVC. They compare each language against shared expectations, not directly
against each other through a call. They do not establish a safe wrapper.

**Verified loader results:** the local `amd_fidelityfx_loader_dx12.dll` loaded
and all five API exports resolved, without invoking an AMD API function. Its
tested hash is in the loader record. The SDK documentation identifies it as the
entry DLL managing separate effect providers. The subsequent query experiment
adds evidence for one upscaler enumeration path.

**Verified query result:** one count-only `ffxQueryDescGetVersions` invocation
with null context/device returned OK and count 2. This requires one additional
56-byte descriptor and the version-query/upscaler-selector tags; paired ABI checks
cover fields, layout and constants against local headers. No name or ID arrays
were requested, so the count does not identify the providers or GPU compatibility.

**Deployment observation:** the initial call using the original absolute SDK
loader path returned NO_PROVIDER. Staging the test executable with the loader
and upscaler DLLs returned OK. This supports executable-adjacent deployment for
this test, not a comprehensive DLL search policy. No production loader change
or search-path mutation was made. Exact commands and both results are preserved
in the query record. The SDK's C example omits the descriptor tag; the probe sets
it explicitly according to the header and implementation.

**Verified ownership and failure checks:** `FfxLibrary` owns private typed
pointers and the library. Five unsafe forwarding methods borrow that owner;
there is no raw accessor. Fixture tests cover missing files, invalid images,
each missing export, release after partial resolution, forwarding and release
on drop. Compile-fail tests cover borrowing and private pointer access.

**Verified D013 compile/fixture results:** a sealed Rc/Arc retention parameter
works through the real private context owner, with serialized macOS ARM64 and
Windows x64/MSVC fixture transfers preserving terminal cleanup and runtime
retention. All eight D013 probes execute successfully on both platforms;
Windows additionally verifies fixture DLL residency through dependency cleanup
and release afterward. Compilation confirms the trait assumptions; public local
types remain `!Send + !Sync`.
This adds no AMD transfer or concurrency guarantee; see
[the bounded threading findings](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles).

**Verified bounded D013 native results:** the September spike passed three
serialized, fenced AMD transfer scenarios and same-thread controls on RX 9060 XT,
driver `32.0.31041.1004`, SDK v2.3.0/provider 4.1.1. The October follow-up passes
all 11 prerequisites on its changed source, closing the native regression gap
after dispatch-policy extraction, then all 30 new children. These add creation
after runtime transfer, 64-frame continuous history, independently held siblings
and actual automatic native cleanup during Rust unwind. All 516 new frames and
42 destroy calls pass, including 12 unwind cleanups; 96 selected packed RGBA16F
payloads match controls/repetitions by SHA-256. Unselected frames have metric/
status checks rather than full image-equivalence evidence.

The provider remains loaded after all audited application frame/device/helper
refs are released and workers exit; the loader and helper are absent. This does
not attribute native retention, prove actual COM device destruction or establish
provider termination on the destination thread. See
[extended native threading results and remaining gates](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency).
D013's conditional design is now accepted; the existing public types remain
`!Send + !Sync`. This experiment implemented no public movable API and establishes
no universal AMD threading guarantee.

**Verified D013 failure-fixture results:** 14 control/transfer owner children and
six worker acquisition failures pass on Windows x64/MSVC. Shared dispatch policy
preserves known/unknown errors, poison, preflight rejection/recovery and terminal
no-retry retention; coordinated ready/poisoned unwind preserves sibling usability
and final fixture release. All 39 recorded source fingerprints matched at the
2026-10-07 intake.
Stable/MSRV regressions pass after preserved sandbox metadata-write failures;
AMD tests remain ignored. The injected inspection error and device surrogate
are not transferred real COM operations. See
[failure-fixture coverage](runtime-threading.md#d013-transferred-failure-state-and-coordinated-fixture-cleanup).

**Source intake limit and policy tension:** the source report could not inspect
the requested pinned implementation and establishes no affirmative AMD transfer
guarantee. Its platform sources were not independently revalidated during intake.
The native follow-up supplies its own preregistered matrix and does not complete
the missing pinned implementation audit. Loader residency is distinct from
provider termination/thread attribution. Its ambiguous-create-error quarantine
recommendation supplies no new contrary observation to D005's bounded returned-
error RAII policy and does not replace it. Native returned-error cleanup remains
unverified; see [source intake](runtime-threading.md#d013-transfer-contract-source-intake).

**D013 review recommendation:** the
[supplied soundness/architecture review](records/2026-10-09-src-d013-soundness-architecture-review.md)
finds no confirmed Rust soundness defect and recommends acceptance with revisions.
Unsafe acquisition must support every subsequent destination thread and maintain
shared-state coordination through safe calls, implicit Drop/unwind and all derived
contexts. A concrete upscaler field audit and narrow Send implementation, explicit
runtime `!Sync` marker and separate local/movable families remain necessary.
The review considers further exploratory native experiments unnecessary before
implementation if those caller assumptions are accepted; this is a design judgment,
not new native evidence. Its date, author and source revision were not supplied;
2026-10-09 is the intake date. Existing D005/D008/D011 policies remain intact.
[D013](../adr/D013.md#acceptance-rationale-and-remaining-assumptions) was separately
accepted on 2026-10-09 with the revisions and renderer-state migration rationale.
No design soundness blocker remains. The subsequent optional public implementation
and [bounded verification](../release/2026-10-09-d013-public-api-verification.md)
record the actual field audit, traits/features/family boundaries, shared failure
fixtures and coordinated native cleanup separately from this review.
The [ADR safety contract](../adr/D013.md#conditional-safety-contract)
owns the requirements; see [review synthesis](runtime-threading.md#d013-soundness-and-architecture-review).

**Verified bounded D014 sharpening results:** the
[native spike](records/2026-09-30-exp-d014-native-sharpening.md) changes only the
already-bound native sharpening fields in test code after shared inspection,
through the existing context owner and dispatch-poisoning core. On SDK
v2.3.0/provider 4.1.1, RX 9060 XT and driver `32.0.31041.1004`, two complete
three-repetition runs submitted/fenced 480 finite outputs with clean ordinary
debug/device checks and explicit teardown. Disabled repeat controls were
bit-identical. Enabled zero differed from disabled in both stationary scenes;
zero strength cannot promise pass disablement or identity output.

The proposed frequency-scene effect gate failed in every repetition: strength
0.35 increased horizontal gradient by only 0.584% against a 1% requirement;
strength 1 shifted mean red by 0.01952 against a 0.005 bound. Independent half
decoding and metric computation confirmed the results. Edge overshoot is a
diagnostic observation, not a quality guarantee. See the
[dispatch synthesis](upscaling-dispatch.md#d014-sharpening-native-execution-and-failed-effect-gate)
for the policy conflict and scope limits. Production still fixes sharpening
off. [D014](../adr/D014.md) was accepted on 2026-10-07 as an API/migration
and bounded native-pass trust decision; implementation and the new release
protocol remain pending. These results add bounded execution evidence;
they do not establish new public scalar validation, trait guarantees, safe GPU
lifetimes, signed-provider allocation capacity or broader hardware support.

**Safety boundary:** explicit input paths are canonicalized before loading the
main DLL. Dependency resolution still follows Windows loading rules. Loading
requires trusted initialization/termination and ABI-compatible exports; no
authenticity check occurs. Native contexts and work may outlive individual calls,
so the raw caller must preserve library and resource lifetimes and synchronization.
This bounded design is authorized by
[D004](../DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points).

## Older Rust bindings: useful patterns and limits

Both comparisons concern standalone FSR2 2.2.0, whose caller-allocated opaque
context storage differs from v2.3.0's pointer handle. They inform wrapper design,
not the modern SDK's native contracts.

| Concern | Embark `fsr-rs` | NotAPenguin0 `fsr2-rs` |
|---|---|---|
| Ownership | Non-copyable high-level context retains interface and scratch storage. | Thin raw layer exposes copyable opaque context storage; no owner wrapper. |
| Teardown | Unsafe `destroy(&mut self)` neither consumes nor invalidates the object; no context `Drop`. | Free create/destroy functions leave all live/dead state to callers. |
| Failure handling | Returns create errors without wrapper rollback; DX12 interface initialization discards its native result. | Exposes native errors directly; partial-create cleanup remains unresolved. |
| ABI | Generated raw bindings still permit copying native state. | Handwritten declarations are auditable, but closed error enums, callback nullability/unsafe typing and enum widths need scrutiny. |
| Threading | Explicit interface/scratch `Send + Sync` propagate to context without an identified native guarantee. | Opaque storage gains auto traits; descriptor has explicit `Send`, without establishing native concurrency safety. |

Sources: [Embark report](records/2026-09-20-src-embark-fsr-rs.md), pinned to
`2ec24b8abae1d6013d444cd472fa9712c584a21b`, and
[fsr2-rs report](records/2026-09-20-src-fsr2-rs.md), supplemented by the
[provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md):
parent `3a928595f46818f7221f0ca28f00770b53ded850` is a high-confidence historical
reconstruction; its native gitlink is `35d136728c49b5c866517b906d4405b7bee583da`.

**Inference for this project:** retain backing storage and native dependencies
with the context owner, prevent duplicate ownership and post-destroy use, preserve
native errors, and audit callback types and layouts against the pinned ABI.
A constructor borrow alone does not encode a context-long dependency. Neither
comparison supplies a sound ready-made destruction/failure policy. Embark's old
DX12 backend `AddRef`/`Release` behavior does not resolve modern v2.3.0 device
ownership; the old Vulkan paths likewise establish no new backend support here.
These are design inputs, not accepted API or binding-strategy decisions.

## Disagreements and limitations

No discrepancy was observed within the experimentally checked ABI/query subset.
The lifecycle source report identifies documentation conflicts about the required
version node, post-destroy nulling, and chain-node lifetime. Runtime evidence now
includes successful context lifecycles, host-allocation injection crashes and
one returned-error input-lifetime experiment;
it confirms non-nulling but leaves chain-node lifetime unresolved. The documented
permission to return null from allocation contrasts with crashes on the tested
path; the separate duplicate-backend rejection returned normally and caller
cleanup caused no observed fault within two seconds. This does not establish a
general failure cleanup contract. Those ABI/query/lifecycle and failure records
did not test upscaling dispatch or image output; the separate M6 and final
Windows records supply the bounded GPU evidence described above. No 32-bit or
other-platform claim follows. The local header was
fingerprinted but its correspondence to the upstream release tree was not
independently authenticated in this work.

[D003](../DECISIONS.md#d003--handwrite-the-first-common-abi-slice) authorizes
handwritten bindings only for this slice; successful tests alone did not settle
the broader binding strategy. [D006](../adr/D006.md) subsequently selected the
handwritten-by-default policy with reviewed generated exceptions; concrete
additional slices remain undecided. The Rust module is production implementation;
the native probe is verification evidence. Evidence records preserve the state
at investigation time and do not independently accept architecture changes.

The ABI checks themselves offer no licensing conclusion. Current notice,
acquisition and signed-runtime distribution findings are maintained in the
[licensing and distribution synthesis](licensing-and-distribution.md), including
the distinction between source-built components and the supplied signed runtime.

**Provenance refinement:** the
[repair record](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md)
supplies full-commit AMD citations at
`60f4ea81909200d8542eca14dccb2628b763a9a3` for the older lifecycle record's
missing URLs. Its two already populated URLs are locally confirmed as v2.3.0
links. The repair reports verification of the fsr2-rs parent's native gitlink;
the parent's identity on 2026-09-20 remains a high-confidence inference from
matching 36-commit history, not a dated branch snapshot. Representative pinned
sources support the earlier findings without a reported substantive discrepancy.
The repair did not have the full historical records and is not a complete prose
re-audit. Original records remain historical; the supplementary record carries
the corrections. Neither repaired citations nor open source resolve the signed
provider's implementation or establish runtime behavior.

## Open questions

The later [M6 dispatch synthesis](upscaling-dispatch.md) collects source
findings about DX12 states and synchronization and tracks accepted D011, M6b's
public-route proof and the final Windows regression/three-frame smoke. The new
post-dispatch teardown observations complement this topic's historical M5
lifecycle results; their native failure and ownership limits remain unchanged.

- **M5 evidence limits and future API work:** the later provenance repair supplies
  pinned source links and corrects broad earlier claims, but does not recover
  every original claim-to-source mapping. Zero components aborted on one signed
  configuration; a later three-width probe returned generic error `1` below,
  at and above the D3D12 Texture2D width limit without identifying the rejection
  cause or an accepted maximum. The numerical source investigation and these
  probes establish no complete safe domain. Accepted
  [D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
  instead adopts a bounded runtime trust assumption, and M5 implements the
  public constructor. Ordinary Windows CI fixtures for that path passed in the
  [recorded run](records/2026-09-24-exp-m5-windows-ci-verification.md); both
  opt-in native lifecycle cases passed on the tested configuration
  ([native record](records/2026-09-24-exp-m5-windows-native-verification.md)). Resolve
  provider-specific helper rounding, output retention and global debug scoping
  where later public operations require them. The two-context sequential result
  does not establish general native concurrency safety.
- **D013 native transfer:** the [compile/host findings](runtime-threading.md#d013-compiled-traits-and-serialized-host-lifecycles)
  resolve the Rust trait questions for the tested shapes. The
  [September native spike](runtime-threading.md#d013-fenced-native-transfers-and-final-release)
  passed three fenced transfer scenarios, controls and then-current native
  regressions; the [October fixtures](runtime-threading.md#d013-transferred-failure-state-and-coordinated-fixture-cleanup)
  add transferred poison, exact preflight, terminal-retention and coordinated
  unwind coverage. The [extended native follow-up](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency)
  closes the extraction's native regression gap and adds creation after runtime
  transfer, longer history, independent siblings and successful fenced Rust-unwind
  cleanup. Native returned-error cleanup, partial-load cleanup, provider/
  dependency termination, hidden work, external-user coordination, outstanding
  GPU work, other configurations and sanitizer evidence remain open.
  The access-limited source intake establishes
  no universal transfer guarantee. Concrete renderer-state migration and the
  lasting caller-contract review informed D013 acceptance. The
  [new review](runtime-threading.md#d013-soundness-and-architecture-review)
  recommends accepting strengthened unsafe assumptions without further exploratory
  execution; the native uncertainties remain unresolved. Acquisition failures
  occur before publication of a movable owner, rather than transferring incomplete
  state. The subsequent [public-path verification](../release/2026-10-09-d013-public-api-verification.md)
  checks the actual optional API, consumer traits/features/families, coordinated
  automatic cleanup and selected fenced native regressions; it resolves none of
  the missing native guarantees.
- **Policy selected, broader questions open:** D007 selects explicit acquisition
  and executable-adjacent upscaler deployment for the tested v2.3.0 DX12 baseline;
  automatic runtime discovery is not part of that policy. Other effect artifacts,
  runtime coexistence and dependency closure remain open. Distribution terms
  remain separate from the accepted acquisition/deployment policy.
- **Provenance:** exact historical fsr2-rs branch identity remains inferred;
  the repaired source set is not a complete audit of every historical claim.
- **Source clarification and design:** chain-node lifetime, signed/external 4.1.1
  device COM ownership,
  callback/threading guarantees, and universal failure-state contracts. D005 now
  selects failed-create cleanup and exceptional dependency retention under its
  explicit runtime assumption; these remaining research gaps do not reopen that
  M4 policy by themselves. New contradictory evidence requires policy review.
- **Follow-up investigation after authorization:** locate the observed allocation-
  failure faults and distinguish loader/provider/driver behavior. Establish a
  deeper returned-failure path or obtain provider-specific lifetime evidence;
  the duplicate-backend spike now covers one returned error and bounded caller
  cleanup, but not a universal contract. Replacement allocator compatibility remains
  untested. For destroy, first establish a documented legitimate failure trigger
  or obtain provider-specific evidence: the new source report found no supported
  valid-context trigger, and a null outer pointer cannot answer liveness. Do not
  corrupt private state to manufacture one, blindly destroy failed-create output,
  or retry failed destruction without a contract.
  Check library/context lifetime invariants without deliberate use-after-free.
- **Later source work and experiments:** result-array queries and broader GPU
  resource, temporal and synchronization behavior remain open. The separate
  [dispatch topic](upscaling-dispatch.md) includes the public-route regression
  and completed three-frame moving smoke. Neither those bounded results nor
  recording-ahead tests establish general threading, synchronization or
  image-quality guarantees. Temporal reconstruction accuracy remains open.
- **D014 release protocol:** the
  [sharpening experiment](records/2026-09-30-exp-d014-native-sharpening.md)
  resolves whether the selected enabled passes execute and whether enabled zero
  can differ from disabled, but its proposed effect criteria failed. Review the
  intended acceptance policy before any threshold/scene amendment and new full
  run. This is not an observed native safety failure. The maintainer separately
  accepted D014's public API and bounded native-pass trust on 2026-10-07;
  release verification remains pending. Existing D011 lifetime, ordering and terminal-error obligations
  remain in force; HDR, exposure modes, GBV, moving scenes and other providers
  gained no new coverage.
- **Policy selected, implementation deferred:**
  [D006](../DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)
  establishes handwritten bindings as the default and permits reviewed generated
  exceptions with equivalent ABI verification, including for first-class support.
  Concrete additional slices and any generation setup remain to be selected.
