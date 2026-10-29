# runtime threading

Updated: 2026-10-27 (U07 accepted/implemented; prior evidence: D042 loader/setup trait allocation and controlled transfer integrated; native guarantees unchanged).

Extracted from the [API and safety synthesis](api-and-safety.md).
These findings retain their original evidence and limitations.

The [2026-10-09 maintenance recheck](../ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh)
repeated all six private transfer prerequisites, all 30 extended cleanup children
and the public movable regression on its captured baseline. The audit verified
96 selected payloads matching their controls and 12 unwind Drops. Provider
residency persisted after application cleanup; this adds no general threading
or termination guarantee and excludes later concurrent source edits.

## Loader and effect trust before safe transfer

The [accepted setup contract](../design/upscaler-setup.md) separates loader/final-
release compatibility from effect operations. A movable loaded Runtime is already
Send, so arbitrary-destination loader cleanup must be established at loading;
it cannot wait for later setup acquisition. Unsafe effect acquisition establishes
the additional lasting upscaler duties for every safe reachable selection and
derived owner, including after the setup/public runtime is dropped.

The [spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md) confirms
local runtime/setup/context `!Send + !Sync` and movable counterparts
`Send + !Sync` through positive/negative external consumers on Rust 1.99/1.88.
One joined-thread destruction executes through the production movable owner
under serialized controlled DLL/COM conditions. It establishes no new AMD
affinity, hidden-worker, arbitrary-thread or concurrent-use guarantee. Exclusive
setup borrowing still covers only one CPU view; all prior common-domain and
callback obligations remain. U07's concrete loader/setup contracts were approved
and implemented on 2026-10-27; the
[implementation record](../release/2026-10-27-upscaler-setup-verification.md)
checks production auto traits and existing transfer regressions separately.

## Callback registration and threading assessment

The [callback/allocation synthesis](callbacks-and-allocation.md) integrates the
[SDK contract record](records/2026-10-25-src-upscaler-callback-allocation-contracts.md)
and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md).
Open allocation/message slots use unsynchronized module-global storage. Two Rust
runtime handles can share one DLL image; an effect ID does not distinguish two
contexts. A Rust registry, TLS current-context marker or mutex supplies neither
missing identity nor callback quiescence. Loader and provider images must not be
assumed to share the same registration namespace.

The [ABI/native spike](records/2026-10-25-exp-upscaler-callback-allocation.md)
now confirms that two acquisitions share one loader image on the recorded stack,
and later resource/CB registration affects surviving contexts on both selected
providers. All observed callbacks run on the main thread at depth 1 during the
serialized protocol; no transfer, concurrent native call or callback-driven
reentry is tested. Provider-message routing differs, and the signed loader's
effect-aware rejection differs from the inspected open implementation. Neither
fact establishes a universal module boundary or supported-effect isolation.

D013's lasting common-domain coordination remains necessary. D040 adds the
concrete movable field/lifetime audit and explicit native affinity,
reentry and shutdown obligations; local `!Send` and movable `Send + !Sync` alone
do not control native callbacks. Avoid wrapper locks across native callback-capable
calls. Both public families now expose explicit unsafe integration, with callback
compatibility on every permitted destination thread. Pinned table/descriptor
addresses survive outer moves; the wrapper owns no external target state.
The [implementation verification](../release/2026-10-25-callbacks-diagnostics-verification.md)
records controlled destination-thread callback cleanup and public-native execution
for both families. The native workload remains serial on one thread. It proves
no general affinity, concurrency, reentry, shutdown or asynchronous-use guarantee.

## Current Configure coordination

[D029](../adr/D029.md) extends both unsafe acquisition contracts to the five
context-specific controls exposed as `request_tuning`. Domain validation now
precedes every native Configure call; it neither acquires a lock nor establishes
GPU completion. Common-domain serialization/non-reentrancy
and movable destination-thread/cleanup obligations include these calls.
Actual-provider trust requires synchronous descriptor/float consumption on both
success and error, no deferred GPU/shutdown work and compatibility with sibling
contexts and the retained profile. Prior unsafe dispatch must establish GPU
completion and no pending executable recording before safe Configure; CPU locks,
`&mut self` and ownership handoff do not establish that condition.

The [implementation checks](../release/2026-10-18-upscaler-tuning-verification.md)
verify local mapping/lifecycle and compile both public families. The pointer-free
eligibility enum moves with the owner; temporary tuning data is not retained.
The local `!Send + !Sync` and movable `Send + !Sync` boundaries remain unchanged.
These results neither establish actual-provider Configure support nor a new
native threading guarantee. The later [coherence checkpoint](../release/2026-10-18-api-coherence-verification.md)
checks both families and the renamed preflight/query methods on current Rust and
MSRV, including isolated development packages. No concurrency permission, wait,
retention token or new native result follows from that checkpoint. The subsequent
[Windows follow-up](../release/2026-10-19-windows-upscaler-verification.md) executes
the fixtures, both-family tuning cycles with 20 fenced GPU frames on 4.1.1 and
the existing public movable handoff regressions. The new tuning control remains
on one thread; it adds no Configure handoff or concurrency proof.

The [reactive-mask experiment](records/2026-10-19-exp-reactive-mask-providers.md)
likewise keeps each public movable generator child on one thread. Both families
pass on 3.1.5; successful returns without writes on 4.1.1/2.3.4 remain failures.
Clean fenced cleanup across these children does not establish cross-thread
generator compatibility, reentrancy, sibling isolation or provider termination.
Common-domain serialization, GPU completion and terminal recording obligations
remain caller-established conditions.

## Active-output coordination follow-up

The [D024 output implementation](../release/2026-10-19-active-output-verification.md)
adds a checked optional dimension value to the existing borrowed frame, without
new native ownership, callbacks, locks or transfer mechanisms. Both public owners
share inspection and native mapping; the existing local/movable trait boundaries
and common-domain coordination requirements remain unchanged.

Both-family GPU coverage on selected 4.1.1/3.1.5 keeps every child on one CPU
thread. Each previous frame is CPU-fenced before recording with replacement
resources on a new per-frame queue. This establishes the recorded serial sequence,
not overlapping work, general cross-queue synchronization or active-output
handoff compatibility. The consolidated batch also passes existing movable
regressions; those do not extend their coverage to transferring this new profile.
Bounds checks, preflight and an owner move still establish no GPU completion.

## Post-v0.1.0 upscaler surface

The [2026-09-29 source record](records/2026-09-29-src-dx12-upscaler-api-surface.md)
maps the broader modern DX12 upscaler surface at the same pinned SDK v2.3.0
commit. It agrees with the earlier M5 inventory and D008/D011/D012; it adds no
runtime evidence or reason to reopen those decisions. Intake checked selected
pinned headers and Query/Configure routing against the current Rust surface,
not every claim in the report. Implementation-specific claims without direct
inline links remain reported findings; the
[prior provenance repair](records/2026-09-22-src-m5-construction-query-provenance-repair.md)
supplies relevant implementation references and qualifications.

**Verified source surface:** the ten creation bits, two generic/seven upscaler
Queries and global-debug/five-key Configure inventory match the pinned headers.
These are upstream capabilities, not complete sys bindings or high-level APIs.
Modern operations use the five generic exports; C++ helpers and legacy direct
FSR3 functions are distinct surfaces. Host allocation callbacks and DX12 backend
allocation extensions belong to ownership/infrastructure, with separate lifetime
contracts if exposed.

**Provider and query implications:** enumeration/override IDs should remain
opaque and runtime-obtained; native names should be copied into owned strings.
Neither names nor IDs establish persistent identity or universal capability.
Resource-requirement Queries report provider-dependent required/optional inputs,
including masks and exposure. They inform future validation without becoming a
new prerequisite for today's fixed profile. Nominal quality ratios do not
establish universal helper rounding or supported modes for every provider.

**Routing qualification:** section 5.4's generic chained-device description
must not imply that memory V2 needs an additional backend node. Both GetVersions
and memory V2 embed a device; pinned `GetDevice` recognizes each. Other
null-context utilities need the chained DX12 device to reach external providers.
The intake also confirms the public loader ignores global-debug `effectId`;
the earlier discrepancy with release guidance and signed-runtime behaviour
remains unresolved.

**Design inference:** D012 permits optional typed creation settings with defaults
preserving today's zero flags. Read-only/provider utilities can also be additive.
Each still needs its own supported contract: shared bits do not prove uniform
provider behaviour. D029 now supplies the explicit wrapper contract and local
implementation for Configure; closed-provider consumption/reset/persistence
properties remain unverified. Callback lifetime/threading and broader native
concurrency remain open.
The report's proposed sequence is a recommendation, not a roadmap change.
Per-frame extensibility and image-semantic experiments are integrated in the
[dispatch topic](upscaling-dispatch.md#broader-frame-surface-and-api-evolution).

## D013: compiled traits and serialized host lifecycles

The [2026-09-29 experiment](records/2026-09-29-exp-d013-movable-compile-host.md)
investigates an optional movable family after the released `0.1.0`. It uses the
real private owner with sealed Rc/Arc retention, not a copied implementation.
It supplements the earlier [threading source findings](records/2026-09-22-src-m5-runtime-sharing-threading.md)
without resolving their native synchronization gaps.

**Verified by compilation:** with locked `libloading` 0.9.0 and `windows`
0.62.2, the actual Windows `RuntimeState` and `Arc<RuntimeState>` are
`Send + Sync`, and `ID3D12Device` is `Send + Sync`. The test runtime's
`PhantomData<Cell<()>>` excludes Sync; a narrow manual Send implementation
suffices for the concrete fixture owner. Raw pointers and the generic owner's
local marker still exclude Send/Sync. The real public `Runtime` and `Upscaler`
remain `!Send + !Sync`; Rc/Arc owner mismatch and custom retention implementations
fail the compiler controls. The original Windows cross-checks compiled these
assertions; the later Windows fixture run also executed the probes. An initial
disjoint closure capture bypassed the Send wrapper and failed compilation;
wrapper methods fixed capture without widening
the generic owner's traits.

**Verified host behaviour:** eight probes passed on macOS ARM64. Same-thread
Rc/Arc controls and joined thread transfers preserve stable boxed pointers,
ordered destruction/dependency cleanup, sibling retention after the public-like
runtime is dropped, and final Rust ownership release on another thread.
Returned create errors, OK/null creation, destroy failure/no retry and unwinding
preserve D005's cleanup or exceptional-retention policy. The host library-drop
observer and weak references establish Rust ownership release, not independent
OS unmapping or AMD DLL termination behaviour.

**Verified Windows fixture behaviour:** the
[Windows x64/MSVC record](records/2026-09-29-exp-d013-windows-fixtures.md)
executed all eight probes with Rust 1.98.1 and default `dx12`, using the real
Windows `FfxLibrary` and the locally built lifecycle DLL. It verifies the same
Rc/Arc controls, serialized sibling transfers, stable handle, error retention,
unwinding and runtime-only transfer. Loaded-module observations require the
fixture DLL to stay loaded during callbacks/dependency cleanup, disappear after
successful final release, and remain loaded after intentional exceptional
retention. This complements the macOS observer; it does not establish AMD DLL
termination or arbitrary OS memory-unmapping behaviour. All nine baseline input
hashes match after CRLF-to-LF normalization, so these are observations of the
same source inputs rather than a changed probe. The workspace run passed 33
wrapper tests, 12 sys tests and four doctests; Clippy with warnings denied and
the feature-disabled all-targets check passed. Eleven AMD/native tests remained
ignored. Two intentional unwind panics were handled successfully; the Rust
launcher path-canonicalization warning did not fail any command.

**Design inference and limits:** these results support the shared private core
and test-only movable shapes. They neither audit a real movable upscaler bundle
nor establish AMD thread affinity, sibling/global isolation, non-reentrancy or
GPU completion. The D013 probes use no AMD DLL, provider, real COM device or GPU
work; other Windows workspace fixtures use controlled COM objects and cover
existing dispatch poison/terminal retention, without adding a transferred
dispatch-poison case. Miri and ThreadSanitizer were not run. This is compatible
with the earlier sequential native results, which test a different scope and
predate the shared-core refactor.
These fixture results alone leave native regressions and fenced thread transfer
unverified; the later native experiment below supplies bounded evidence for
those paths.
These experiments themselves accept no native transfer contract. The later
[D013 decision](../adr/D013.md) accepts the conditional design; at experiment time no public movable
API is implemented.

## D013: fenced native transfers and final release

The [2026-09-30 native experiment](records/2026-09-30-exp-d013-native-thread-transfer.md)
extends the fixture evidence with actual AMD execution on Windows x64/MSVC,
SDK v2.3.0 (`60f4ea81909200d8542eca14dccb2628b763a9a3`), RX 9060 XT and
driver `32.0.31041.1004`. Each context reported provider `4.1.1`, ID
`0xf5a5ca1e01001001`. The record fingerprints the signed loader/upscaler DLLs
and all 76 executed source inputs; these matched across the spike and regressions.
It tests post-publication source, not a new validation of the published archives.

**Verified native execution:** all three transfer cases and their equal
same-thread controls passed in six isolated children. The controls use the same
Arc-retained private owner, frame inputs and cleanup strategy on one worker.

| Case | Transferred operations | Observed result |
|---|---|---|
| A/B/C lifecycle | A loads/creates; B dispatches and fences; C explicitly destroys and releases the final runtime Arc | Passed with matching same-thread readback metrics and loader release |
| One context, three frames | A/B/C each dispatch and fence one successive moving frame; C destroys/releases | All three frames passed on the same context; matching control metrics |
| Two siblings, early runtime drop | A creates both and drops external runtime; B dispatches/fences/destroys the first; C dispatches/fences and normally Drops the last | Retention survives early runtime and first-sibling cleanup; final release on C passed |

Joined workers synchronize exclusive ownership of the whole family and test
equipment; no worker/native operation overlaps another. Each submitted frame
is fully fenced before any further transfer or cleanup. All 12 readbacks were
640×360 with 230,400 finite and changed pixels, zero D3D12 errors/corruption and
`S_OK` device status. Post-cleanup device/debug checks passed. No fence or
180-second child-watchdog timeout occurred; GBV was off. Equal printed metrics
are observed, without a pixel-by-pixel equivalence or image-quality claim.

**Verified retention and loader release:** the runtime weak reference expires
at final cleanup, and the loader DLL is no longer observed loaded on C in all
transfer cases. Scenario 3 includes ordinary Drop of the last sibling; its
native return code is not exposed. Successful Arc expiry and loader release
exclude the owner's exceptional retained-runtime outcome on this path.

**Unresolved provider unload:** the upscaler DLL remains loaded after final
wrapper release in both controls and transfers while the caller device is still
retained. The remaining module reference was not attributed. These observations
establish release/unload of the wrapper-owned loader on C; they do not establish
termination or unmapping of the provider/dependency closure on C. Provider
residency alone neither identifies a transfer-specific fault nor proves that
its full termination path permits transfer.

**Verified regression scope:** on the same changed source, both public Rc
lifecycle cases and all three public dispatch gates (single frame, three-frame
smoke, motion/jitter/reset comparisons) passed. All eight D013 lifecycle fixtures,
ordinary workspace checks, native ABI probes and feature-disabled all-targets
compilation also passed. This supplies the previously missing shared-core native
regression evidence for this configuration; it does not cover a future API/tree.

**Implementation inference and remaining gates:** the bounded results support
continued evaluation of the shared private owner and fenced, serialized transfers.
The only production extraction is the existing dispatch inspection into a private
function, shared by the local API and the test bundle. Send is implemented only
for that concrete test bundle; public local types and the generic owner keep
their existing traits. This is consistent with the fixtures and earlier
same-thread evidence, not an upstream native threading guarantee.

At this September stage, native Rust-unwind transfers had not been tested; the
[October follow-up](#d013-extended-native-transfers-unwind-cleanup-and-staged-residency)
adds bounded evidence for that path. Provider/dependency termination on the
destination thread, native returned-error and partial-load cleanup paths,
unfenced work, other configurations, callbacks, custom allocators and coordination
with other native users remain unverified. Miri/ThreadSanitizer were not run.
The source findings' missing universal AMD transfer guarantees remain unresolved.
The later [D013 acceptance](../adr/D013.md#acceptance-rationale-and-remaining-assumptions)
selects an explicit caller-established trust boundary, not a verified AMD
transfer guarantee. No public movable family was implemented by that experiment.

## D013: transfer contract source intake

The [2026-10-07 source record](records/2026-10-07-src-d013-transfer-cleanup-contract.md)
examines serialized transfer and final cleanup. **Provenance limit:** retrieval
of the requested v2.3.0 commit failed. Its accessible AMD SDK 2.3.0 manuals
were not matched byte-for-byte to the pin; those manuals, Microsoft platform
documentation and Rust/dependency sources were not independently revalidated
during this synthesis.
The record treats prior project experiments as supplied evidence; it adds no
execution or fresh pinned implementation audit. Its failure to establish an
affirmative AMD transfer guarantee is neither an exhaustive absence finding nor
evidence that the tested transfers fail.

**Reported platform findings and design inference:** Arc retention and
`Send + !Sync` do not coordinate independently owned siblings, other runtime
families or external native users. Coordination must survive safe operations,
implicit Drop and unwind; loader notifications/internal workers are not brought
under an application mutex merely by serializing exports. D3D12's recording,
queue and fence permissions do not establish AMD affinity or a blanket apartment
contract for every retained helper. GPU completion and CPU ownership handoff
remain separate requirements. Review actual interfaces, guard/drop order and
loader-lock ordering before extending the contract.

**Cleanup interpretation:** loader disappearance on C remains the earlier
bounded observation. It does not attribute provider termination, dependency
cleanup or a DLL callback's executing thread to C. Continued upscaler residency
in both arms is unattributed, not a transfer-specific failure or demonstrated
leak. Device-associated references, caching, other owners and delayed release
are hypotheses only. Same-adapter D3D12 acquisition need not create independent
devices; separate runtime/device values do not prove native-state isolation.
The [loading synthesis](runtime-loading.md#d013-acquisition-failures-and-release-observations)
retains the associated acquisition and observation limits.

**Policy tension:** the report recommends quarantine for ambiguous native
create-error ownership. Accepted D005 already permits ordinary RAII cleanup of
caller dependencies after a returned create error under a bounded runtime trust
assumption; it does not destroy an ambiguous failed handle. The report identifies
no new contrary observation or pinned provider postcondition, so this
recommendation does not replace D005. Native dispatch/destroy error cleanup,
hidden work, affinity and provider termination remain unresolved.

## D013: transferred failure state and coordinated fixture cleanup

The [2026-10-07 experiment](records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
extends the earlier eight probes through the real terminal owner and a shared
private dispatch policy. **Verified fixture scope:** all 14 isolated owner
children (seven scenarios, each with a control/transfer arm) and six acquiring-
worker loader-failure observations passed on Windows x64/MSVC. No AMD DLL,
real transferred COM device or GPU work was used. All 39 LF-normalized source
fingerprints matched the implementation at the 2026-10-07 synthesis intake.

| Scenario | Added observation |
|---|---|
| Known/unknown dispatch poison | Codes 1 and 4294967295 survive transfer; later rejection occurs before inspection/FFI; one dispatch and one successful terminal destroy, followed by complete release |
| Preflight rejection/recovery | CommandListType and CommandList/GetDevice HRESULT 0x80004005 remain exact; two rejections make no native dispatch and do not poison; later recovery succeeds |
| Explicit/Drop destroy failure after poison | Destroy code 4294967295 is terminal with no retry; explicit destruction reports it, Drop discards it; complete state/device-surrogate/runtime/DLL bundle remains retained |
| Ready/poisoned unwind with a sibling | Destination cleanup holds the common mutex; a waiting sibling recovers the poisoned mutex, dispatches successfully and releases the final runtime; exact panic payloads and event threads match |
| Worker acquisition failures | Missing file preserves owned io/NotFound; each missing required export preserves its exact symbol and libloading source; no runtime is published and the acquired fixture module is absent after failure |

Joined moves, channels and the shared mutex supply ordering. The external runtime
owner is dropped after creation. Handle/input/callback pointers remain stable;
native callbacks and dependency destructors observe a live library. Each owner
child has a 60-second watchdog and required completion marker. Intentional
retention ends with process isolation, not an invented cleanup attempt. Module
residency checks establish presence/absence after calls, not detach-thread
attribution; missing-export cleanup is acquisition-local, not transfer of an
incomplete runtime.

**Verified regression scope:** stable 1.99.0 and MSRV 1.88.0 workspace reruns
passed, including local inspection/poison/ownership and trait controls. Initial
sandbox compiler-control metadata writes failed with access denied; unchanged-
source reruns outside the sandbox passed. Clippy, formatting, Rustdoc and feature-
disabled checks passed. The no-default-features fixture run used the host adapter
on Windows, not Linux/macOS. Nineteen AMD/hardware/research tests stayed ignored. No new AMD
regression was executed as part of this fixture experiment after its dispatch-
policy extraction; the September native result covers its recorded earlier
source state. The October native follow-up below closes this bounded regression
gap on its own fingerprinted source.

**Remaining evidence limits:** the inspection HRESULT is injected
at the seam, and fixture descriptors/device surrogates do not audit a real movable
DX12 bundle. The source report arrived after execution; its larger matrix was
not the fixture experiment's oracle. Prior successful dispatch before fixture
poison, extra native-slot mutations, independently acquired families/external
fixture clients and detach instrumentation remain follow-ups. The native
follow-up below executes creation after runtime transfer, longer history,
independently held siblings, destination Rust unwind and staged residency.
Native returned-error cleanup, provider/device termination, hidden work, other
configurations, outstanding GPU work and sanitizers remain unverified. The later
D013 acceptance does not convert these gaps into verified properties;
public `Runtime`/`Upscaler` remain `!Send + !Sync`. That experiment added no public movable family.

## D013: extended native transfers, unwind cleanup and staged residency

The [2026-10-08 native experiment](records/2026-10-08-exp-d013-native-transfer-cleanup.md)
extends the September execution and October fixtures on Windows x64/MSVC,
SDK v2.3.0 at the same pin, RX 9060 XT, driver `32.0.31041.1004` and provider
`4.1.1`/`0xf5a5ca1e01001001`. AMD DLL hashes and hardware match September;
Rust is now 1.99.0 and the helper, test executable and source changed. All 83
executed source fingerprints match at this synthesis intake. The complete
preregistered protocol, per-child outcomes, hashes and audits remain in the record.

**Verified native execution:** all 11 prerequisites passed on the executed
source, including public local lifecycle/dispatch regressions and the earlier
six transfer/control children. All 30 new children passed: five cases, each with
a same-thread control and transfer arm, repeated in three isolated processes.

| Added case | Transfer observation |
|---|---|
| Runtime before creation | A loads, B creates, C dispatches/fences, D explicitly destroys and releases the final external runtime |
| Continuous history | One context executes 64 moving frames with 63 handoffs across four persistent workers; reset only on frame 0 |
| Independent siblings | Distinct workers retain siblings; one transfers and is destroyed while the other stays live, dispatches again and later Drops elsewhere |
| Context unwind | A fenced context transfers to D and its actual native owner Drops during a deliberate Rust panic outside FFI |
| Sibling unwind | Destination unwind destroys one sibling; the survivor recovers the poisoned common mutex, dispatches again and finally Drops on B |

The common domain serializes application calls, inspection and all cleanup,
including automatic Drop. Guard/drop order keeps native destruction inside the
domain during unwind. Owning channels establish CPU handoff; every submitted
frame completes its GPU fence before transfer, another frame or cleanup. This
coordinates application operations, not unknown provider workers or external
native users. Narrow Send implementations apply only to audited test bundles;
the public types and generic owner gain no traits.

All 516 new frames completed with finite/changed output, zero debug-layer
errors/corruption and no device removal. All 42 actual native destroy results
were OK, including 12 unwind cleanups. There were no unexpected panics or
timeouts. All 96 selected complete packed RGBA16F payloads match corresponding
controls and repetitions by SHA-256; unselected frames have status/metric checks,
not full image-equivalence evidence. No temporal accuracy or quality claim follows.
Stable/MSRV, ABI, fixture, trait and feature-disabled checks also passed on the
recorded source. Local archive inspection used `--no-verify`; it establishes
contents, not package-build verification or publication.

**Verified residency, unresolved termination:** all applicable checkpoints agree
in controls and transfers. After final context/runtime release, the loader is
absent and the provider remains. Releasing all audited frame, caller-device and
helper references makes the helper absent while the provider remains; ordinary
worker exit does not change this. The observer adds no module reference. The
previously retained application equipment/device refs are therefore not necessary
for provider persistence in this experiment. This neither proves a leak nor
attributes persistence to a cache, driver or native owner, and does not establish
actual COM device destruction, provider termination or its executing thread.

**Remaining evidence and decision boundary:** successful fenced Rust-unwind
cleanup, longer history,
creation after runtime transfer and independent siblings now have bounded native
evidence. Native returned-error/partial-load cleanup, hidden
work/TLS, provider/dependency termination, external-user coordination, outstanding
GPU work, callbacks/allocators, other configurations and sanitizers remain open.
The missing pinned implementation audit and affirmative AMD transfer guarantee
are unchanged. No observation requires revising D005/D011's tested policy.
[D013](../adr/D013.md) subsequently accepts the conditional design after review,
without resolving those native uncertainties. That experiment implemented no
public movable API; blanket Send implementations remain prohibited.

## D013: soundness and architecture review

The [review imported on 2026-10-09](records/2026-10-09-src-d013-soundness-architecture-review.md)
recommends **acceptance with revisions**, finding no confirmed Rust soundness
defect in the proposed architecture. **Provenance limit:** no investigation date,
reviewer identity or reviewed revision was supplied. It reviews existing records
without rerunning them and adds no AMD guarantee or pinned upstream audit.
Intake compared the cited current source paths and prior evidence; the verdict
is a design assessment, not verification of a production movable family.

**Required contract refinements:** unsafe acquisition must cover every thread
where a general `Send` owner can later be used or dropped, including unwind,
through the lifetime of all derived contexts. A promise limited to a predefined
A-to-B handoff cannot support unrestricted safe Rust movement. Common-domain
serialization and non-reentrancy must cover independently held siblings,
separate acquisitions, local/movable families, external native users and all
explicit/implicit cleanup. `&mut self` and Arc reference counting enforce
neither that shared-state coordination nor native thread compatibility.

**Implementation direction:** retain separate public families without safe
conversion, sealed Rc/Arc retention and shared private construction, validation,
dispatch and terminal cleanup. Use an explicit `PhantomData<Cell<()>>` marker
for a `Send + !Sync` movable runtime; audit every retained field and cleanup path
before adding only a concrete `unsafe impl Send for MovableUpscaler`. Do not
widen the generic owner's traits. These shapes agree with the existing fixture
source; at review intake production used Rc and exposed neither a `movable`
feature nor public movable types. D005's ordered cleanup, terminal no-retry and exceptional
retention and D011's unsafe GPU boundary remain the review's baseline.

**Cleanup clarification:** partial symbol-resolution failure is cleaned up
inside acquisition before a movable value exists; it is not transfer of an
incomplete runtime. Native failure cleanup remains a trust obligation. Final
wrapper-owned loader release and actual provider termination are separate events.
The proposed wording must assume any native termination is safe on its actual
executing thread, without promising provider unload or its thread.

**Evidence sufficiency is a recommendation:** this review considers the existing
serialized, fenced evidence sufficient for the explicitly
unsafe design if the strengthened caller-established assumptions are accepted.
It does not resolve missing AMD guarantees, native returned-error behaviour,
hidden work, provider termination, external-user coordination, outstanding-work
transfers or other configurations. Those remain uncertainties bounded by the
contract, rather than newly verified properties. It identifies no reason to
replace D005's returned-create-error policy, differing from the earlier source
report's quarantine recommendation without adding contrary execution evidence.

**Decision and verification boundary:** the review supplies technical contract
feedback, not acceptance by itself. Under explicit authorization,
[D013 was accepted on 2026-10-09](../adr/D013.md#acceptance-rationale-and-remaining-assumptions)
with the revisions and framework-independent renderer-state migration rationale.
Its [unsafe contract](../adr/D013.md#conditional-safety-contract) is authoritative;
the remaining native assumptions are not design soundness blockers. Acceptance
required no further general research phase. Implementation verification required the actual public types:
Windows traits and family separation, feature/host boundaries, shared-core
failure/retention fixtures, bounded public-path native transfers including final
Drop, and existing local-family regressions. Historical test bundles cannot
substitute for those checks. The recommendation selects no release and changes
neither D014 nor `0.2.0` scope.

## M10: Query coordination

The [2026-10-12 source review](records/2026-10-12-src-m10-dx12-query-safety.md)
applies the existing coordination distinction to the five proposed queries.
`&mut self` protects one Rust object; separate siblings and foreign/native users
can still touch shared state. Enumeration's reused global name buffer requires
the coordination domain to cover count, fill and immediate copying, not merely
the native calls. Movable queries also inherit D013's arbitrary-thread
and cleanup obligations, with no new implicit mutex or blanket Send impl.

The reported open paths perform CPU queries without recording GPU work; those
facts establish neither a universal fence requirement nor closed-provider
compatibility with GPU work in flight. A CPU lock/handoff is not GPU completion,
and a GPU fence does not serialize CPU access. Query-specific pointer and
GPU-state assumptions need deliberate contract review; D013 acceptance does not
automatically extend them. See the [five-method safety assessment](api-and-safety.md#m10-selected-dx12-query-safety).
The intake adds no execution result or AMD threading guarantee and leaves the
accepted movable contract unchanged.

[D018](../adr/D018.md) subsequently accepts explicit query-specific trust in both
unsafe acquisition contracts, including live-query compatibility with permitted
work in flight and ordinary-error continuation. Unreleased source implements
the five methods without adding locks. The later
[Windows checkpoint](../release/2026-10-12-windows-m10-sdk-verification.md) passes
both query families and transferred-owner regressions on one configuration.
M10 is selected for [0.3.0](../release/0.3.0-preparation.md); final release checks
remain separate. Neither those bounded results nor historical D013 checks
establish universal native query/GPU guarantees.

## D013: public implementation verification

Current source now implements the optional family, shared sealed private core
and concrete Send audit. The [2026-10-09 implementation check](../ROADMAP_HISTORY.md#movable-implementation-checkpoint)
records consumer traits/feature/family boundaries, failure fixtures, automatic
cleanup coordination and selected native public-path checks. Historical records
above remain evidence for their original test-only source. The new checks
establish no universal AMD compatibility or provider termination guarantee;
the maintainer explicitly included the optional family in `0.2.0` on 2026-10-10,
superseding the earlier exclusion. The
[combined release check](../ROADMAP_HISTORY.md#release-qualification)
verifies the selected archives and public movable paths. Historical experiments
above retain their original inputs and limitations.
