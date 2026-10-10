# runtime threading

Updated: 2026-09-30 (native D013 evidence integrated).

Extracted from the [API and safety synthesis](api-and-safety.md).
These findings retain their original evidence and limitations.

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
the earlier discrepancy with release guidance and signed-runtime behavior
remains unresolved.

**Design inference:** D012 permits optional typed creation settings with defaults
preserving today's zero flags. Read-only/provider utilities can also be additive.
Each still needs its own supported contract: shared bits do not prove uniform
provider behavior. Callback lifetime/threading, Configure pointer consumption,
reset/persistence, numerical limits and broader native concurrency remain open.
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

**Verified host behavior:** eight probes passed on macOS ARM64. Same-thread
Rc/Arc controls and joined thread transfers preserve stable boxed pointers,
ordered destruction/dependency cleanup, sibling retention after the public-like
runtime is dropped, and final Rust ownership release on another thread.
Returned create errors, OK/null creation, destroy failure/no retry and unwinding
preserve D005's cleanup or exceptional-retention policy. The host library-drop
observer and weak references establish Rust ownership release, not independent
OS unmapping or AMD DLL termination behavior.

**Verified Windows fixture behavior:** the
[Windows x64/MSVC record](records/2026-09-29-exp-d013-windows-fixtures.md)
executed all eight probes with Rust 1.98.1 and default `dx12`, using the real
Windows `FfxLibrary` and the locally built lifecycle DLL. It verifies the same
Rc/Arc controls, serialized sibling transfers, stable handle, error retention,
unwinding and runtime-only transfer. Loaded-module observations require the
fixture DLL to stay loaded during callbacks/dependency cleanup, disappear after
successful final release, and remain loaded after intentional exceptional
retention. This complements the macOS observer; it does not establish AMD DLL
termination or arbitrary OS memory-unmapping behavior. All nine baseline input
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
[D013](../adr/D013.md) remains proposed; no public movable API or native transfer
contract is accepted by these experiments.

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

Provider/dependency termination on the destination thread, native error/unwind
and partial-load transfer paths, unfenced work, other configurations, callbacks,
custom allocators and coordination with other native users remain unverified.
Miri/ThreadSanitizer were not run. The source findings' missing universal AMD
transfer guarantees remain unresolved. D013's concrete-integration and lasting
caller-contract review gates still apply: [D013](../adr/D013.md) remains
**proposed**, with no public movable family or accepted native transfer contract.
