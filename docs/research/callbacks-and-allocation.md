# Callbacks, diagnostics and custom allocation

- Updated: 2026-10-29 (current archive-bound callback/production-CB evidence integrated; original source/spike bounds retained).
- Scope: R03/U04 for complete upscaling access in `fsr-sdk 0.3.0` /
  `fsr-sdk-sys 0.1.2`, Rust 1.88, Windows x64/MSVC + DX12, SDK v2.3.0 at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Included records: [SDK contract investigation](records/2026-10-25-src-upscaler-callback-allocation-contracts.md)
  and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md).
  Both are source-only records dated 2026-10-25. Neither executes callbacks,
  proves signed-provider behaviour, accepts an API or implements a feature.
  The subsequent [ABI/native experiment](records/2026-10-25-exp-upscaler-callback-allocation.md)
  adds bounded execution on selected 3.1.5/4.1.1 and two Rust compilers;
  it implements no production callback API. The separate
  [implementation verification](../release/2026-10-25-callbacks-diagnostics-verification.md)
  covers the accepted D040 API and both public families.

## Current assessment

The source/practice investigations and bounded ABI/native spike are complete.
Maintainer authorisation after independent review accepts [D040](../adr/D040.md).
Both public families now implement its typed unsafe callback interface, paired
host/resource/heap tables, CB and message pointers, independent debug controls
and explicit modern/legacy diagnostic methods. External targets remain caller-owned;
no registry, closures, automatic unregister or target reclamation is implied.

The [implementation verification](../release/2026-10-25-callbacks-diagnostics-verification.md)
passes **192 CI** and **169 MSRV** stages separately from the public-native profile:
**15 stages**, 12 contexts, 20 fenced frames and 16 delivered messages on selected
3.1.5/4.1.1. Host/resource allocations balance; 80 earlier-set resources are freed
by compatible replacement callbacks. The legacy live-context route returns 0
on both providers, while deprecated null-context requests return 2 unchanged.
The production raw CB aggregate is exchanged through MSVC and Rust 1.99/1.88.
Heap callbacks pass ABI and controlled invocation checks, including a real
65,536-byte equipment-owned heap; SDK heap invocation remains unobserved.

The [2026-10-29 package checkpoint](../release/2026-10-29-package-coverage-verification.md)
now executes the public callback fixture from the extracted wrapper and imports
the production CB declaration from the extracted sys package. Both Rust compilers'
independent MSVC aggregate exchanges pass, alongside 12 contexts/20 fenced frames,
16 messages and balanced allocations. The
[consolidated candidate review](../release/2026-10-29-candidate-verification.md)
now includes the subsequent [rc1 Windows report](../release/2026-10-29-rc1-windows-verification.md),
which passes those archive-bound cases on the frozen pair shared by all three
hosts. Final P08 input reconciliation is [accepted](../release/2026-10-29-rc1-performance-acceptance.md); publication simulation passes separately.
It closes the prior source-only binding gap without proving native heap invocation
or adding quiescence, threading or failure-recovery guarantees.

Callback-bearing native create failures retain the pinned wrapper bundle without
operating on the returned handle. Pre-native failures use ordinary cleanup;
post-create query failures use terminal one-attempt destruction. Successful
teardown releases wrapper storage, not externally retained global targets.
Native failure recovery, callback quiescence and general threading guarantees
remain unproved. These limits are explicit unsafe obligations rather than a
missing safe adapter. The [coverage matrix](upscaling-coverage.md#remaining-work-and-ordering)
and [release preparation](../release/0.3.0-preparation.md) distinguish feature
verification from final exact-package closure.

## Evidence progression

| Input | What it establishes | Limit |
|---|---|---|
| [SDK contracts](records/2026-10-25-src-upscaler-callback-allocation-contracts.md) and [Rust FFI practices](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md) | Pinned source contracts, callback identity, ownership risks and suitable Rust boundaries. | Source/practice analysis does not prove the signed provider's behaviour. The practices study is not an independent SDK audit. |
| [Direct-SDK spike](records/2026-10-25-exp-upscaler-callback-allocation.md) | 17 stages, 10 contexts and 16 fenced frames; aggregate exchange, shared allocation replacement and provider-specific messages. | Uses probe callbacks and an experiment-only aggregate; it does not verify the public wrapper. |
| [Accepted D040](../adr/D040.md) | Explicit unsafe installation, external targets, compatible shared allocation, independent debug controls and exceptional retention. | Maintainer acceptance selects the contract; it does not supply new AMD guarantees. |
| [Implementation verification](../release/2026-10-25-callbacks-diagnostics-verification.md) | 192 CI, 169 MSRV and 15 native stages; production raw aggregate, both public families, 12 contexts/20 frames, messages and legacy live routing. | Native success is bounded to the recorded deployment; injected failures and destination-thread cleanup are controlled-fixture evidence. Final package-native coverage remains separate. |
| [Archive-bound checkpoint](../release/2026-10-29-package-coverage-verification.md) | Selected Windows archives supply the wrapper fixture and raw CB declaration; Stable/MSRV aggregate exchange and the 12-context/20-frame callback case pass. | Final publication-pair reconciliation remains open; heap invocation, quiescence and failure-recovery limits are unchanged. |

## Combined findings

| Concern | Evidence and consequence |
|---|---|
| Callback identity | The SDK record finds userdata only in the host allocator pair. Messages and all five DX12 callbacks lack it; resource/heap `effectId` identifies an effect type, not a context. CB `data` is payload. The practices record therefore supports explicit function pointers, with no inferred per-context closure identity, TLS routing or generated trampoline registry. |
| Shared registration | The open backend stores all five pointers in module-static slots; complete pairs/non-null CB replace slots, while omission/incomplete pairs leave them installed. The native spike observes resource/CB replacement, omission and incomplete-pair persistence on both selected providers, including freeing earlier set-1 resources through set 2. Heap execution was not observed. Separate Rust handles are not independent callback domains. |
| Diagnostic scope | Open FSR2/FSR3 install creation messages globally; inspected open-loader Configure ignores `effectId` and accepts both descriptors. The signed loader instead rejects an unknown effect and the deprecated null-context request with code 2, while modern upscaling-global configuration reaches both selected providers. Creation-message routing differs between 3.1.5 and 4.1.1; neither per-context identity nor supported-effect isolation follows. |
| Host allocation | Compatible callbacks/state must reach create and destroy; free accepts null and has no size/alignment argument. An owned adapter would need allocation metadata and retained userdata. The documented permission to return null is qualified by unchecked zeroing/placement-new in open code and historical native access violations. A recoverable OOM contract is not established. |
| DX12 ownership | Resource and heap callbacks return usable COM objects; custom free replaces the corresponding release/accounting in inspected paths. Placed resources are released directly. Heap free receives the logical requested size, which need not equal the aligned native heap size. Keep resource, heap, offset and allocation lifetimes distinct; do not invent automatic AddRef/Release semantics for the closed provider. |
| Constant buffers | The 56-byte/alignment-8 result and `handle` offset 48 now pass independent C/Rust by-value exchange on Rust 1.99 and 1.88, including corrupted-field controls. Open DX12 uses `handle` as a root-CBV GPU address and ignores `resource`. Native C++ callbacks copy payload to aligned upload backing and complete fenced dispatch on both providers. No status or paired CB-free callback exists; backing must survive GPU use. |
| Error channels | Open resource-allocation paths assert success before using output; heap allocation and resource/heap deallocation statuses are ignored; CB has no defined failure sentinel. The wrapper must preserve native signatures without promising transactional failure, graceful allocation recovery or a fabricated `Result` channel. |
| Debug controls | Creation checking (`1 << 7`), visualization (`1 << 9`) and per-frame debug view (`1 << 0`) are separate controls. Match estimate/create flags, dimensions and provider selection. Open providers differ in forwarding these bits; acknowledgement does not prove application. Debug view may modify the output by adding an overlay. |
| Rust practice | ash, glfw-rs, windows-core, gpu-allocator and libloading supply bounded comparison patterns, not FidelityFX guarantees. Explicit unsafe operations, owned-versus-borrowed COM conversion, durable callback code/state and separate GPU completion are applicable lessons. No new dependency or MSRV change is justified by the comparison. |

The source-specific findings come from the
[contract record](records/2026-10-25-src-upscaler-callback-allocation-contracts.md);
their Rust API implications are inferences informed by the
[practices record](records/2026-10-25-src-rust-ffi-callback-allocation-practices.md).
The [experiment](records/2026-10-25-exp-upscaler-callback-allocation.md) supplies
the separately identified native/ABI observations, without exposing the signed
provider's private implementation or establishing universal guarantees.

## Direct-SDK spike and source/runtime reconciliation

The final spike passes **17/17 stages**, **10 contexts** and **16 fenced frames**
on one recorded Windows/Radeon deployment. Host allocation/free balances 30/30,
custom resources 160/160, with 220 CB uploads retained through GPU completion
and context teardown. Forty set-1 resources are freed by compatible set-2
callbacks. All tracked host/resource/heap blocks are gone at successful teardown;
heap callback counts are zero, so no heap invocation is verified. The Rust/MSVC
aggregate exchange is separate from the native callbacks, which are C++ probe
functions; neither result verifies a production Rust callback API.

On both selected providers, creating B changes surviving A's CB route to set 2;
omitting the extension or supplying incomplete pairs does not restore set 1.
Destroying the installing contexts also does not restore it. On 3.1.5, the
surviving A additionally invokes distinct message callback 3 after its installing
context D has been destroyed. On 4.1.1, the tested warning stays with the explicit
global callback despite later creation-message pointers. Modern upscaling-global
configuration changes warning delivery in both provider controls.

The signed loader returns code **2** for the unknown effect and deprecated
**null-context** global-debug request, unlike the inspected open-loader route.
The unknown query stimulus also returns 2 without emitting a message. The
standalone global case establishes no selected-provider identity; identity is
verified only in the context children. Preserve the effect field and native
errors, without claiming it is universally ignored or that rejection proves
isolation between supported effects. The spike did not test the live-context deprecated route. The later
[public implementation check](../release/2026-10-25-callbacks-diagnostics-verification.md)
observes code 0 for that route on both selected providers and both families. The pinned verbose level is **`0x0fffffff`**, correcting the source
record's `UINT32_MAX` description; historical evidence remains unchanged.

The three earlier failed attempts and their incorrect fixture/source predictions
are retained in the experiment record. The final passed observational protocol
does not turn rejected routes, absent heap callbacks or missing synthetic
diagnostics into successes. Debug checking/visualization creation and estimate
flags 640 plus ordinary/debug-view frames execute, without isolating each flag's
effect or proving estimate accuracy. No new execution was performed at synthesis.

## Accepted ownership and error policy

[D040](../adr/D040.md) extends D005 with the following implemented boundary.
It preserves the ordinary constructor and empty callback bundle's default
allocation/error policy. Callback assembly itself invokes no native code.

| Dependency or event | Owner and consequence |
|---|---|
| Descriptor chain and optional host table | The wrapper pins them through create/destroy and forwards the same host table. Successful destroy releases this storage; native access to it must then cease. |
| Callback code, userdata, allocator bookkeeping and device references | The integration retains them for every possible native user, including surviving/future ordinary contexts and error returns. Context Drop, replacement and null configuration do not prove quiescence. |
| CB upload resources, ranges and backing | The renderer retains them through native use and GPU completion, with residency and reuse coordination. There is no paired native CB-free callback or automatic wrapper retirement. |
| Callback-bearing native create error | Preserve the code, perform no operation on the failed output and retain the complete pinned state/device/runtime bundle. This does not promise native rollback. |
| Pre-native rejection or post-create query error | Pre-native rejection uses ordinary cleanup. A live owner rejected by a post-create query uses ordinary one-attempt terminal cleanup. |
| OK/null creation or failed destroy | Retain critical dependencies; do not retry destroy or manufacture a usable owner. External targets remain separately retained. |

All five DX12 callback roles and the host pair are accessible through both
families. Pairs are indivisible; absent DX12 groups do not reset earlier native
registrations. The callback payload boundary includes the named buffer,
generic-read and message constants needed without a direct sys dependency.
There are no captured closures, TLS routing, registration guards, automatic
unregister, hidden wrapper locks or recoverable-OOM promises.

Modern upscaling-global and deprecated null/live configuration remain distinct
unsafe calls. Runtime-global errors preserve their Configure code without
fabricating per-context poison; live-context errors follow the existing terminal
Configure policy. All affected native users must be idle and coordinated, with
GPU work complete and no executable pending recordings. Callback code must not
unwind or reenter the SDK domain. Movable installation additionally requires
compatibility with every permitted destination thread and cleanup there.

## Remaining work and milestone boundary

R03/U04 are complete for the accepted bounded SDK-access contract. The 192/169/15
stages above verify source integration, not the final 0.3.0/0.1.2 package pair.
The next package is P2-C release closure:

1. Reconcile the complete API inventory, provider limitations and public release
   documentation, retaining U01's mixed provider results.
2. The archive-native binding is implemented and verified against the selected
   Windows pair: `release` now selects the callback case and production CB fixture.
   Reconcile any final package changes with the completed checks under
   [release preparation](../release/0.3.0-preparation.md#archive-coverage-before-final-verification).
   The ordinary standalone callback profile remains source-based unless archive
   mode or the explicit pair is selected.
3. The frozen rc1 pair passes the consumer/native checks and its P08 input
   reconciliation is accepted. Publication simulation now passes with matching
   regenerated hashes and both uploads aborted; repeat affected checks only if
   later changes warrant them.
   Publication remains separately authorized.

Heap SDK invocation remains unobserved; its ABI and controlled calls are covered.
General quiescence, affinity/concurrency and native failure recovery remain
explicit unsafe assumptions. These limits do not reopen implementation or require
another broad online run, stress campaign or OOM crash probe. New investigation
is warranted only for a concrete defect or a separately selected stronger
contract. Image-quality and provider-output comparison gates remain outside scope.

## Intake and remaining limits

The incoming filenames and dates already follow the research convention.
Intake checked the records for private paths/secrets, inspected current source
and manifests, and compared the findings with the existing lifecycle/threading
evidence. The three research records, the experiment's compact evidence and the
separate implementation record/evidence remain byte-for-byte unchanged. The
spike retains its recorded 17-stage outcome; the public implementation summary
separately records 192/169/15 stages. This synthesis reruns neither workflow nor
reconstructs unretained frames.
The practices record explicitly did not audit SDK internals, so its
SDK assumptions are now corroborated or qualified by the separate contract record,
not counted as independent SDK confirmation.

Narrow pinned-source spot-checks corroborate the
[conditional registration path](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp),
[static slots and registration setters](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp)
and [loader-global Configure routing](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp).
The original source intake did not repeat either full investigation, validate
every source anchor/comparison implementation or authenticate the supplied
project archive. The later spike adds the bounded ABI/native evidence above.
Its signed-loader and provider-message differences are material scope
qualifications to the open-source model, not grounds to overwrite that model's
historical observations. General unregistration/quiescence, failure recovery,
SDK heap invocation and concurrent/reentrant behaviour remain unresolved; observing
callbacks on the main thread at depth 1 supplies no general native thread rule.
