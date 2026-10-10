# D013 — Soundness and Architecture Review

## Intake provenance

- Intake date: **2026-10-09**, Europe/Berlin; no investigation date, reviewer
  identity or reviewed source revision was supplied.
- Imported from `neuer_record.md`; the supplied review below is preserved.
- Method and scope: supplied architecture/soundness review of proposed
  [D013](../../adr/D013.md), its Windows/DX12 movable family and existing evidence.
  Its acceptance verdict and proposed safety text are **design recommendations**,
  not a maintainer decision, AMD guarantee or new execution result.
- Referenced implementation: [runtime](../../../crates/fsr-sdk/src/runtime.rs),
  [context owner](../../../crates/fsr-sdk/src/context.rs),
  [upscaler](../../../crates/fsr-sdk/src/upscaler.rs),
  [movable fixtures](../../../crates/fsr-sdk/src/context/d013_movable.rs),
  [native transfer harness](../../../crates/fsr-sdk/tests/native/thread_transfers.rs)
  and [loader](../../../crates/fsr-sdk-sys/src/loader/windows.rs). The review's abbreviated
  paths and line numbers refer to these files; their original revision is unknown.
- Referenced execution evidence: [compile/host probes](2026-09-29-exp-d013-movable-compile-host.md),
  [Windows fixtures](2026-09-29-exp-d013-windows-fixtures.md),
  [September native transfers](2026-09-30-exp-d013-native-thread-transfer.md),
  [failure/cleanup fixtures](2026-10-07-exp-d013-transfer-failure-cleanup.md) and
  [October native follow-up](2026-10-08-exp-d013-native-transfer-cleanup.md).
  The supplied review reports that these results were reviewed, not rerun.
- Intake compared the cited ownership, trait and acquisition paths with the
  current repository and the relevant records. No new compilation, AMD source
  investigation or native execution was performed. D013 remains **proposed**;
  its concrete-integration and explicit maintainer acceptance gates remain open.

## Supplied review

Verdict: Accept with specific revisions. D013 is technically ready for implementation.

No confirmed Rust soundness defect was found in the proposed architecture. `Send + !Sync` can be implemented with a narrowly audited `unsafe impl Send` for the concrete movable upscaler, while retaining the existing `Rc` family unchanged.

The existing native evidence is sufficient to justify proceeding with this explicitly unsafe API. An affirmative AMD threading guarantee, provider termination observations, and further exploratory native experiments are not prerequisites, provided the caller-side contract explicitly assumes the unverified behavior.

Formal ADR acceptance still requires the maintainer decision specified in `docs/adr/D013.md:271–287`.

## 1. Prioritized findings

### F1 — High: The unsafe acquisition contract must cover every subsequent safe operation

Classification: Required contract clarification, not a confirmed implementation defect.

D013 correctly locates the unsafe boundary at `MovableRuntime::load_unchecked` (`docs/adr/D013.md:59–63,79–127`).

However, once acquired, safe Rust may transfer or destroy a `Send` value on another thread. The contract therefore cannot merely authorize a predefined A→B transfer.

It must establish that the selected native implementation tolerates operations and destruction on every thread on which the movable value may subsequently be used or dropped, including during unwinding.

If compatibility exists only for a restricted subset of threads, that restriction cannot be enforced by the proposed general `Send` API.

The acquisition contract must also extend through the lifetime of all derived contexts, not just the public runtime object.

Disposition: Strengthen the thread-affinity wording. No per-transfer unsafe method is needed if the stronger acquisition assumption is adopted.

### F2 — High: Independent movable owners do not enforce native serialization

Classification: Known unenforced invariant, acceptable under the unsafe contract.

`&mut self` excludes simultaneous Rust access to one upscaler, but separate contexts can be used or destroyed concurrently.

`Arc<RuntimeState>` ensures thread-safe reference counting, not native synchronization.

This matters across separately acquired runtimes, local and movable instances, and external FidelityFX users potentially sharing provider/global state.

D013 already recognizes these cases (`docs/adr/D013.md:86–119`). The October native experiment demonstrates a coordinated application domain, rather than proving independent contexts are intrinsically concurrency-safe (`tests/native/thread_transfers.rs:59–101,260–282`).

Disposition: Caller-managed common-domain serialization is sufficient. An internal mutex would not solve interactions with unknown external users.

The documentation must explicitly make simultaneous native operations a violation of the acquisition's safety precondition, including implicit destruction.

### F3 — High: The manual `Send` implementation must be limited to the audited concrete type

Classification: Confirmed implementation requirement.

`NativeContextOwner<S,D,R>` intentionally remains `!Send + !Sync` through `PhantomData<Rc<()>>` (`context.rs:54–57`).

Its `Arc` specialization is not automatically `Send`, because the context handle, pinned descriptor structures and local marker require separate examination.

The correct implementation is a narrow `unsafe impl Send for MovableUpscaler`, justified by its exact private fields and their ownership invariants—not a blanket implementation for `NativeContextOwner`, `RuntimeState` or arbitrary dependencies.

The compile and native fixtures demonstrate this technique (`context/d013_movable.rs:122–152,264–293`; `tests/native/thread_transfers.rs:269–295`).&#x20;



Disposition: The proposed approach is sound if the production implementation preserves that field-level audit.

### F4 — Medium: `Arc` does not provide the required public `!Sync` guarantee

Classification: Confirmed trait-design requirement, already identified by D013.

`RuntimeState` and `Arc<RuntimeState>` compile as `Send + Sync` with the locked dependencies.

Consequently, a `MovableRuntime` containing only that `Arc` would also be `Sync`.

Use a private `PhantomData<Cell<()>>` marker so that the runtime is `Send + !Sync`; no unsafe `Send` implementation is necessary for this runtime shape.

This is already demonstrated in `context/d013_movable.rs:125–131,264–293` and documented in `docs/adr/D013.md:129–169`.&#x20;



### F5 — Medium: Provider termination must not be conflated with wrapper-owned loader release

Classification: Undocumented native behavior, not a blocker.

The October experiment observed successful destruction and release of the Rust-owned runtime, but the provider DLL remained loaded after audited application references and workers were released.

Therefore, the wrapper can describe the executing thread of its last library release, but cannot claim that this is the thread on which the provider terminates.

Reference: `docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md:204–236`.

Disposition: Require the caller to assume that any native termination which does occur is safe on its actual executing thread. Do not promise provider unload or its thread.

## 2. Ownership and lifetime

The existing ownership structure supports D013 without redesigning terminal cleanup.

PROPOSED MOVABLE OWNERSHIP GRAPH

MovableRuntime

`Arc<RuntimeState>` → `FfxLibrary` → `libloading::Library` and private function pointers

independent strong reference on creation

MovableUpscaler

`NativeContextOwner<Pin<Box<UpscaleCreateState>>, ID3D12Device, Arc<RuntimeState>>`

- Native context handle
- Pinned root/version/backend descriptor chain
- Independent owned COM device reference
- Shared runtime/library retention

Dispatch command lists and texture resources are borrowed, not owned. Their GPU lifetimes remain the caller's responsibility. The selected provider's independent module residency is not represented by a Rust-owned handle.

Source: `runtime.rs:105–119`, `upscaler.rs:94–165,194–218`, `context.rs:40–57`.

The terminal cleanup path in `context.rs:119–163,207–210` is coherent:

1. Native destruction is attempted at most once.
2. On success, pinned state is released before the owned device, and the runtime reference is released last.
3. On destroy failure, the entire critical dependency bundle is retained.
4. Successful creation with a null handle also retains dependencies; unsuccessful creation follows D005's existing returned-error trust assumption.

Moving the outer owner does not move its pinned descriptor allocation. Independent `Arc` references keep the library alive when the public runtime is dropped before its contexts.

The final `Arc` release may unload the wrapper-owned DLL on a different thread. There is no guaranteed unload thread and no reason to introduce one.

These conclusions preserve the accepted D005 ownership policy (`docs/adr/D005.md:21–90`). No double-destruction, descriptor-movement, or Rust reference-counting defect was identified.&#x20;



The caller cannot use the public API to enforce provider-global isolation, hidden native worker quiescence, or coordination with uncooperative external users. These are genuine limits of the chosen unsafe contract.

## 3. Native evidence assessment

The recorded evidence is stronger than a basic thread-transfer smoke test.

| Property                                   | Assessment                               |
| ------------------------------------------ | ---------------------------------------- |
| Rust `Send`/`Sync` feasibility             | Verified by compile probes               |
| `Rc`/`Arc` retention and terminal policies | Verified by host and Windows fixtures    |
| A→B→C→D load/create/dispatch/destroy       | Observed successful                      |
| History-preserving transfers               | 64 frames, 63 handoffs, successful       |
| Independently held sibling contexts        | Successful under shared coordination     |
| Native destruction during Rust unwind      | Successful                               |
| Selected output equivalence                | 96 matching payload hashes               |
| Provider-global thread safety              | Not established                          |
| Provider termination on destination thread | Not observed                             |
| Transfers with outstanding GPU work        | Not tested; outside the verified profile |

The October record reports 30 successful new native children, 516 completed frames and 42 successful native destructions on Windows 11, an RX 9060 XT, driver `32.0.31041.1004`, selected provider 4.1.1 and the recorded signed AMD DLLs (`docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup.md:3–21,61–85`).

These are bounded observations, not universal AMD guarantees.&#x20;



Rust establishes exclusive ownership, atomic `Arc` reference counting, and happens-before edges when correctly synchronized. It does not establish that FidelityFX ignores thread affinity, serializes hidden provider state, or safely handles driver callbacks.

The absence of published guarantees can be accommodated by explicitly unsafe acquisition. It should not be described as a proven property of the tested binary.

## 4. Required design changes

Only the following implementation constraints are necessary.

Retain separate public types. `MovableRuntime` and `MovableUpscaler` should remain distinct from `Runtime` and `Upscaler`, without safe conversion in either direction. The `Rc` family must not inherit movability accidentally.

Retain the sealed ownership choices. Reuse the existing private `Rc`/`Arc` retention seam in `context.rs:14–38`. Factor shared construction, validation, dispatch policy and terminal cleanup instead of duplicating them. No public generic `Effect` abstraction or blanket unsafe trait implementation is justified.

Keep trait and construction invariants explicit. The runtime requires its `!Sync` marker; the upscaler requires a narrowly audited `Send` implementation while remaining `!Sync`. Safe construction must accept only `&MovableRuntime` and retain an `Arc` from precisely that runtime.

Tighten the public contract. State the arbitrary-destination requirement, common coordination obligations, automatic Drop/unwind requirements and distinction between loader release and provider termination. Partial load failures are cleaned up within the acquisition call before a movable value exists, not through a transferred partially constructed runtime (`fsr-sdk-sys/src/loader.rs:118–183`).

No internal scheduler, conversion API, new provider abstraction or change to `unsafe dispatch(&mut self)` is required.

## 5. Proposed final safety contract

The following text is suitable for the eventual `MovableRuntime::load_unchecked` Rustdoc, alongside the existing loader trust requirements.

`# Safety`

The caller must guarantee that:

1. The loader and all resolved dependencies/providers are trusted, ABI-compatible, and safe to initialize, operate and release, including unsuccessful acquisition and terminal error paths.
2. The selected loader, provider, driver and relevant native dependencies permit use, movement, context creation, dispatch, destruction, COM-reference release and final wrapper-owned DLL release on any thread where these `Send` owners may subsequently be used or dropped. No same-thread affinity may be required for those operations. These guarantees are caller assumptions, not established AMD guarantees.
3. All native operations potentially touching shared FidelityFX runtime, provider or global state are serialized, non-reentrant and ordered by synchronization establishing happens-before across threads. This includes all contexts, separate runtime acquisitions, existing local-family instances, external native users, error cleanup, explicit destruction, implicit `Drop` and final loader release. Independent locks are sufficient only when their isolation is established.
4. These invariants are maintained for the entire lifetime of the runtime and every derived context, even after the public runtime is dropped. Ownership transfers and normal or unwinding destruction must preserve the required coordination. Exceptional native failures may retain dependencies indefinitely; they do not establish native quiescence or authorize other users to disregard shared-state restrictions.
5. The caller independently upholds the dispatch contract for D3D12 resource states, lifetime, residency, recording/submission ordering and GPU completion. CPU synchronization does not imply GPU completion. Submitted work must complete and invalid recordings must be discarded before context destruction. Any native transfer preconditions involving outstanding GPU work must also be satisfied; such transfers are not established by the existing experiments.

If these requirements cannot be upheld for the chosen native configuration and all subsequent uses, this API must not be called.

This contract places the persistent responsibility at one unsafe acquisition boundary while allowing safe construction and consuming destruction. It does not make dispatch safe or guarantee that a third party can disregard the originating unsafe obligation.

## 6. Verification gaps

No further exploratory native experiment is necessary before implementation. The existing records cover the relevant Rust ownership mechanisms and successful serialized, fenced native transfers.

Once the real public types exist, the minimal implementation/release verification is:

| Test                                  | Purpose                                                                                                                |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Windows compile tests                 | Movable `Send + !Sync`, local `!Send + !Sync`, cross-family construction rejection                                     |
| Feature-disabled and host compilation | Ensure no unintended public API or portability regression                                                              |
| Shared-core fixtures                  | Verify error/poison policy, pointer stability, final release and exceptional retention through actual movable wrappers |
| Public-path native transfer test      | Exercise the real `MovableRuntime` and `MovableUpscaler` API across distinct workers, including final Drop             |
| Existing Windows regressions          | Ensure local construction/dispatch and D005/D011 behavior remain unchanged                                             |

The previous native matrix does not need to be redesigned or expanded; selected cases can be reused for public-path verification.

Testing actual AMD returned-error recovery, further provider unload behavior, arbitrary driver configurations, unsynchronized calls, or outstanding-work transfers is not a precondition for this design. Their uncertainty must instead remain explicitly bounded by the unsafe contract and supported configuration.

The recorded Rust and native results were reviewed, not independently re-executed here.

## 7. Next implementation steps

1. Approve the tightened caller-side contract and explicitly accept the conditional native trust boundary in D013.
2. Introduce the feature-gated `MovableRuntime`, using `Arc<RuntimeState>` and an explicit `!Sync` marker.
3. Extract a shared private upscaler core while preserving the existing public API and sealed retention variants.
4. Implement the concrete `MovableUpscaler`, audit its complete field graph, and add only its narrow `unsafe impl Send`.
5. Add public Rustdoc and trait/API compile tests, including family separation and feature gating.
6. Run shared-core fixtures, bounded public-path native tests, and existing local-family regressions on the selected Windows configuration.

Final assessment: D013 needs precise contractual and implementation safeguards, not another native research phase. The existing evidence supports proceeding with implementation once the maintainer explicitly accepts the conditional unsafe trust model. No concrete defect requires overturning D005, D008 or D011, and D014 and the `v0.2.0` scope can remain untouched.
