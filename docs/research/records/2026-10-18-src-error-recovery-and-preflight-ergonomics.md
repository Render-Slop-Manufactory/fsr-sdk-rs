# Source investigation: early errors and non-terminal recovery in DX12 upscaling

- Investigation date: 2026-10-18.
- Topic and scope: Six developer-experience and safety boundaries in the current
  `fsr-sdk` DX12 API: D030 native-error poisoning, D011 abandoned recording,
  D029 numeric tuning, D029 Configure ordering, D028 repeated provider
  enumeration, and late dispatch resource validation.
- Baseline and evidence cut-off: supplied `fsr-sdk-rs-main` ZIP snapshot,
  archive SHA-256 `555f0357f7413a6400870a10c03f6c70dbef62c4618a17abdfec0a3a2e767366`;
  internal AMD SDK baseline **v2.3.0**, source commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`, API version 4.1.1.
  D011/D028/D029 are accepted; D030 is **proposed and not implemented**.
- Method: inspected repository implementation, tests, ADRs, historical research
  and primary pinned AMD sources; consulted Microsoft D3D12 reference material
  on command recording, ordering, barriers and fences (accessed 2026-10-18).
  Analysed alternative API contracts against the established unsafe GPU and
  native-provider trust boundaries. No source was changed other than this record.
- Exclusions: no native execution, GPU probe, performance measurement, provider
  reverse engineering, or implementation of the proposed APIs. No Rust toolchain
  is available in this review environment; the ZIP has no `.git` history.
  Prior recorded test results are historical evidence, not tests rerun here.

## Question and evaluation criteria

Can the SDK expose programming errors at construction or preparation time,
permit correction without discarding otherwise usable state, and preserve its
current Rust/native soundness contract? In particular, when is a failure an
ordinary inspectable input error, an unsupported native feature, a possibly
partially mutated provider context, or an unsubmitted GPU recording?

**Desired ordering of detection:** invalid typed value at creation or input
validation; immutable resource/configuration incompatibility at preparation;
per-frame numerical/geometry checks before FFI; genuinely provider-dependent
failures at native entry; and GPU execution failures at completion. The last
category cannot generally be moved to compile time. A useful safe abstraction
must prove its guarantees rather than merely move `unsafe` into its internals.

Terminology: **poison** means the private context eligibility is terminal for
new live operations; it does **not** mean the native handle has been destroyed,
that the GPU work completed, or that destruction can occur immediately.

## Inputs and provenance

Repository links below are relative to this record. Primary sources:

- [D011](../../adr/D011.md#native-errors-and-abandoned-recordings),
  [D016](../../adr/D016.md), [D018](../../adr/D018.md),
  [D028](../../adr/D028.md), [D029](../../adr/D029.md),
  [D030](../../adr/D030.md), and
  [code heuristics](../../CODE_HEURISTICS.md).
- [Shared eligibility and error transition](../../../crates/fsr-sdk/src/context_policy.rs),
  [DX12 dispatch orchestration](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs),
  [dispatch inspection](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs),
  [dispatch inputs](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs),
  [tuning mapping](../../../crates/fsr-sdk/src/upscaler/tuning.rs),
  [runtime provider routing](../../../crates/fsr-sdk/src/runtime/queries.rs),
  [enumeration and provider checks](../../../crates/fsr-sdk/src/query/native.rs),
  [upscaler core](../../../crates/fsr-sdk/src/upscaler/dx12.rs),
  [controlled tuning tests](../../../crates/fsr-sdk/src/upscaler/dx12/tests/tuning.rs).
- Existing [Configure source investigation](2026-10-15-src-upscaler-configure-contract.md),
  [complete upscaling surface inventory](2026-10-18-src-upscaling-release-surface-inventory.md),
  [D029 implementation verification](../../release/2026-10-18-upscaler-tuning-verification.md),
  and [D028 implementation verification](../../release/2026-10-18-provider-selection-verification.md).
- AMD pinned [common API header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h),
  [upscaler header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h),
  [FSR3 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L1597-L1736),
  [FSR3 implementation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp),
  and [AMD 3.1.5 technique guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md#constant-overrides).
- Microsoft [recording command lists](https://learn.microsoft.com/en-us/windows/win32/direct3d12/recording-command-lists-and-bundles),
  [command queue execution](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12commandqueue-executecommandlists),
  [resource barriers](https://learn.microsoft.com/en-us/windows/win32/direct3d12/using-resource-barriers-to-synchronize-resource-states-in-direct3d-12),
  [multi-engine synchronization](https://learn.microsoft.com/en-us/windows/win32/direct3d12/user-mode-heap-synchronization),
  [command list Close](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-close),
  and [fence completed value](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12fence-getcompletedvalue).

The code, existing records and version-pinned AMD sources define the substantive
baseline; the Microsoft documentation describes public D3D12 guarantees rather
than observed behaviour of the proprietary FSR provider.

## Findings

### 1. D030: unsupported native generator poisons the upscaler context

**Verified (repository source and proposal).**
`context_policy.rs:7–50` holds the common `Usable`, `DispatchFailed` and
`ConfigureFailed` states. It checks eligibility, performs Rust inspection and
only then enters the native call; any non-OK `Dispatch` return poisons the
context. Rust/COM preflight errors do **not** poison. D030:202–237 proposes
reusing this transition for `generate_reactive_mask`, including unsupported
operations; the method does not yet exist. D029 similarly poisons after any
native Configure error. After poison, live dispatch, Configure and queries are
rejected, while terminal teardown remains possible.

**Verified (upstream source).** The pinned common `ffx_api.h` defines error
codes 0–7, including `FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE` (2),
`FFX_API_RETURN_ERROR_RUNTIME_ERROR` (3),
`FFX_API_RETURN_ERROR_PARAMETER` (6) and
`FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE` (7). The open FSR3 adapter
has a separate dispatch branch for the generator and forwards it to an FSR3
function; an unknown tag returns 2 at that adapter switch. An open adapter
rejecting an unknown descriptor at its switch does **not** prove that every
provider emitting code 2 or 7 has left all CPU state, internal GPU allocators
and the recorded command list untouched. The header describes codes, not
per-error atomicity or rollback guarantees.

**Problem.** A *feature-discovery* attempt with a genuinely unsupported
provider can sacrifice a working main-upscaler context. If unsupported
behaviour is frequent across configurations, the error path becomes a routine
integration hazard rather than exceptional recovery. However, blindly ignoring
native non-OK because the code sounds like “unsupported” can reuse a partially
mutated context/list. D030's refusal to guess safety from a return code is
well-founded.

**Recommended near-term improvement — avoid predictable native failures:**

1. Keep the current terminal policy for **every native non-OK** until an
   identifiable provider/version-specific no-side-effect guarantee exists.
   Preserve exact `Operation::Dispatch` and native code, as now.
2. Add an entirely non-mutating `validate_generate_reactive_mask(...)` /
   `prepare_generate_reactive_mask(...)` step for flags, scalar finiteness,
   region, DX12 metadata, usage, device and COM identity. Expose structured
   reusable errors, and run it *before* native entry. This cannot detect native
   feature support.
3. Document provider support as **unconfirmed** unless tested for the exact
   provider/runtime combination. Offer an explicit renderer-level startup
   compatibility probe **on a sacrificial context**, with a real valid list and
   correct submission/fencing/cleanup. A probe is expensive and may fail to
   generalise; its result is evidence, not a permanent capability certificate.
4. Provide a recovery recipe or optional helper operating on a caller-owned
   creation recipe (device, runtime, options, provider, valid GPU lifecycle):
   invalidate a failed recording; complete previously submitted work; destroy
   the poisoned context; create a **new** context; restart temporal history.
   There must be no hidden submission, wait, retry of the failed list, or
   automatic fallback on a potentially damaged context. A separate sacrificial
   generator context may isolate main-upscaler history, but has unverified
   cost, provider and resource-sharing assumptions and needs GPU tests.

**Rejected shortcuts.** A query-requirements bitfield is not a generator
capability oracle (D025/D030). A trial dispatch on the live context is not
non-mutating. An error-code allow-list is not sound without proving rejection
occurs before mutation for **each** covered provider/version and failure path.
Calling reset or trying Configure cannot unpoison a context. No promise of
“always no runtime failures” is possible at the DX12/device/provider boundary.

**Verification gate.** Controlled fixtures for all error codes, eligibility
priority, no-FFI preflight failure and shared main/generator poison; separate
bounded native tests with supported and unsupported providers (where
available), using debug layer/GBV, tracking list invalidation and cleanup.
Specifically seek source/provider evidence of side-effect-free unsupported
rejection before considering a non-terminal `Unsupported` outcome. Differentiate
source-level absence of a branch, native acknowledgement, recorded commands,
completed GPU output and subsequent-context viability. D030 remains proposed.

### 2. D011: abandoning an acknowledged recording ends context use

**Verified (repository source and D011).** D011:286–333 requires terminal
cleanup when `ffxDispatch` returned OK but the application discards or cannot
submit that command list. The Rust method owns neither queue nor fence and
cannot observe submission. `dispatch.rs:21–80` exposes `unsafe` recording and
assigns list/allocator/resources/context completion to the caller; the shared
eligibility implementation has no `Abandoned` transition, because abandonment
is currently a **caller-side unsafe contract**, not detectable poison. The
open backend's historical recording-ahead experiment supports multiple pending
recordings in one bounded setup, **not** arbitrary rollback or a fixed ring
capacity (D011:179–236).

**Upstream fact.** D3D12 separates recording from `ExecuteCommandLists`.
Microsoft documents that recording calls can defer failure to `Close`, and
that command-list APIs do not retain all referenced COM objects. `Reset` of a
list and `Reset` of its allocator have distinct lifetime conditions; cancelling
an unsubmitted list is **not** a FidelityFX provider-state rollback operation.

**Problem.** A renderer may intentionally discard a frame during resize,
recovery, device-loss handling, or command-list recording failure. Requiring
upscaler-context replacement is costly but presently necessary because a
successful native call may already have advanced provider-side temporal or
transient state. There is also a **visibility gap**: the Rust API does not
actually mark a context as abandoned and cannot independently prevent later
use if the unsafe caller breaks this rule.

**Recommended staged approach:**

- **P0:** Preserve terminal semantics and add a precise abort/recreation guide
  with a state table covering: pure preflight failure (no new commands), native
  failure (do not submit affected list), successful FSR recording plus later
  renderer failure (must discard **all** affected FSR contexts), successful
  recording submitted (complete GPU work before destruction), and normal
  `Close`/`ExecuteCommandLists` errors. Do not claim errors are all
  recoverable on the same context.
- **P1:** Optional **renderer-integrated recording session** that owns, or is
  explicitly entrusted with, the **whole** list/submission lifecycle. It tracks
  recorded FSR contexts and marks them terminal on session abort. A returned
  `Recorded` token by itself is *not* enough: a user can submit the raw list
  without the token, fail to report a submission, or drop a token while GPU
  work is running. A proof-carrying safe session needs exclusive mediation of
  queue submission and fences, resource retention, allocator reuse, all
  referenced contexts and the relevant GPU states; otherwise its
  `submitted`/`aborted` assertions must remain `unsafe` obligations.
- **P2:** Investigate non-terminal abandonment **only** if native source or
  exact-provider experiments establish rollback of temporal and transient
  state, including after mixed recordings and across queues. A frame-history
  `reset` pulse is **not** proof of such rollback.

**Alternative.** For a renderer-independent low-level SDK, a small sample
`abort_upscaler_frame` / `recreate_context` workflow is preferable to introducing
an illusory safe submission-owner type. Recoverability by **replacement** is
useful; recoverability by **reusing the native handle** is unproven.

**Verification gate.** Fixtures for preflight/no poison, native failure,
multiple successes then abort, multiple contexts on one aborted list, owner
Drop/destroy retention, and renderer `Close` failure. GPU tests must cover
record-ahead, failed/abandoned recorded batches, completion fences and fresh
context output without assuming temporal continuity. A safe session requires a
full ownership and aliasing soundness review, not just a compile-fail test.

### 3. D029: finite but out-of-domain tuning is passed through unchanged

**Verified (repository source).** `upscaler/tuning.rs:40–61` rejects only
non-finite `TuningValue::Value`. For any finite `f32`, including `-0.0`, values
outside the documented bounds and `f32::MAX`, it sends the original bits in a
stack-allocated descriptor. `tests/tuning.rs:73–79` explicitly verifies this;
this is **intentional D029 policy**, not missing input checking. The null
`ProviderDefault` route preserves the actual provider default.

**Verified (upstream claim and open implementation).** The pinned AMD guide
lists clamping domains `VelocityFactor` and `AccumulationAddedPerFrame` [0,1],
`ReactivenessScale` and `ShadingChangeScale` [0,+infinity), and
`MinDisocclusionAccumulation` [-1,1]. The inspected open FSR3 setter clamps
finite values. The existing Configure research establishes that open FSR2 can
return **OK without applying any** of these settings. Hence an `Ok(())`
does not certify effective support, clamping or resulting values on all
providers.

**Problem.** An accidental `5.0` passed for `VelocityFactor` can be silently
clamped **by the provider**, hiding an application bug; the Rust wrapper itself
neither clamps nor transforms. Replacing the accepted raw provider control
with unconditional strict rejection would, however, violate D029 and D016's
policy that direct controls preserve SDK-level behaviour. Strict `f32`
range checks are run-time checks, not automatic Rust compile-time proofs.

**Recommended non-breaking two-tier API:**

- Retain `configure_tuning(parameter, TuningValue)` as the explicit,
  documented **direct provider/clamping** control. Consider clearer Rustdoc
  calling it pass-through and noting unconfirmed provider behaviour.
- Add a **pure** checked factory, for example
  `TuningValue::checked_for(parameter, value) -> Result<TuningValue, Error>`
  (illustrative, not selected), or named domain wrappers
  `UnitInterval::new(f32)`, `NonNegativeFinite::new(f32)` and
  `SignedUnitInterval::new(f32)`. Reject NaN/±infinity and wrong domain
  *before* any native call, without poison. Retain negative zero if desired
  since IEEE -0 compares equal to zero, and test it explicitly.
- Prefer an optional `configure_tuning_checked(...)` convenience only if its
  provenance is clear (published AMD/Open-FSR3 domain versus arbitrary
  selected-provider guarantees). A checked value is **within a documented
  range**, not necessarily numerically safe or applied by a closed provider.
- Avoid a silent wrapper-side `clamp()`, provider-default emulation with numeric
  constants, a cached “effective value”, and provider-keyed guesswork. If the
  project later wants a separate **quality policy**, name it as such under
  D016; do not redefine the five direct controls.

**Verification gate.** Portable exhaustive boundary/property tests for all
five domain mappings, ±0, negative finite values, upper bounds,
subnormals, maximum finite, NaN/±infinity, and no-native-call-on-rejection;
controlled native descriptors to establish exact direct-control passthrough.
No GPU/closed-provider support claim follows from those unit tests.

### 4. D029: changing tuning requires an idle context

**Verified (repository decision and code).** D029:134–156 requires that all
previously submitted work on that context has completed and that no executable
unsubmitted recording can subsequently execute. This condition is carried by
the **prior unsafe dispatch contract**, not an undocumented extra precondition
on the safe `configure_tuning` method (`dx12.rs:125–138,261–289`). A newly
created context is idle, and several tuning changes can be made before its
first dispatch. The wrapper performs no wait or queue submission.

**Verified (bounded upstream evidence).** Open FSR3 immediately copies updated
CPU-side tuning into persistent context constants; a later dispatch stages
these values. This suggests narrower scheduling may work there but cannot
establish that all signed/closed providers permit Configure while previous
GPU work is pending. Microsoft D3D12 documents explicit fence/queue coordination:
`&mut self` is CPU exclusivity, **not** evidence of completed GPU work. A fence
value alone also does not prove there is no as-yet-unsubmitted executable list.
On device removal, `ID3D12Fence::GetCompletedValue` can return `UINT64_MAX`;
naïvely interpreting that as “all GPU work completed” would be wrong.

**Problem.** Frequent interactive tuning between frames can cause a full
in-flight drain, destroying CPU/GPU pipelining and raising latency. But this is
an **intentional current safety bound**, not a known mandatory hardware stall
for every provider or every control. No measured overhead or real application
frequency is established in this investigation.

**Recommended paths in increasing risk:**

1. **Without contract change:** supply a builder/recipe for initial tuning,
   applying values to the newly created idle context before first dispatch.
   Cache **desired application settings**, not claimed effective provider
   values; batch user-interface changes into a single scheduled idle window.
   Five calls remain five independent Configure operations, without atomicity
   or rollback: if the third fails, the context poisons and prior changes
   cannot be treated as transactional.
2. **Explicit staging helper:** a purely Rust-side `PendingTuning` collection
   which validates options up front and applies a sequence only when the
   renderer confirms an already-established idle interval. Never imply that
   collecting five changes makes five native calls atomic. This improves
   diagnostics and reduces avoidable waits only when the application batches
   idle periods; it does not relax safety by itself.
3. **Optional renderer/queue integration:** an owner that mediates recording,
   submission, all relevant queues and fences and guards an idle phase may
   provide a genuinely safe state transition for Configure. A caller-supplied
   fence or `GpuIdleToken` without controlled submission is at most an
   **unsafe assertion**, not a sound proof.
4. **Conditional relaxation:** if exact-provider native tests plus sufficient
   implementation evidence establish in-flight Configure safety and visibility
   ordering, propose a separately reviewed, explicitly provider-scoped path.
   Do not quietly relax D029 for all providers based on open FSR3 staging.

**Verification gate.** Test before-first-dispatch, repeated changes on a new
context, poisoned-state precedence, partial-success sequence, and explicit
provider no-op acknowledgement. For a relaxation, require dispatch→Configure
with controlled in-flight work, fence/data capture and later dispatch/readback
across repeated and cross-queue sequences, with native debug/GBV where available.
Results must distinguish correct output ordering from mere return-code success.

### 5. D028: fresh enumeration before every explicit-provider query

**Verified (repository source).** `runtime/queries.rs:77–174` calls
`require_provider` before each explicit provider ratio/render-size/jitter,
resource-requirement or memory-estimate query. `query/native.rs:34–50`
implements that by enumerating and checking membership. D028:66–96 and
158–170 explicitly select this over a token. Explicit creation repeats the
check and subsequently verifies the actual live provider identity, whereas
pre-create queries have no equivalent effective-provider output.

**Verified (upstream).** The pinned AMD common header requires override IDs to
come from `ffxQueryDescGetVersions`; it does not guarantee that an earlier
observed ID is a durable reserved provider handle. D028 correctly recognises
that even fresh count/fill enumeration is **not** a reservation or protection
against identity reuse or provider changes after the observation.

**Problem.** Multi-query setup makes repeated native count/fill and name-copy
calls, and repeatedly passes the same ID through multiple APIs. This may cost
time or create repetitive error handling. No benchmark establishes that it is
material in typical startup flows; it is usually outside the per-frame hot path.
Replacing the check with a plain cached ID would remove a useful early
`ProviderNotEnumerated` result without proving better safety.

**Recommended, conditional ergonomics improvement:**

- Retain existing checked individual methods and post-create **effective
  identity** verification. Avoid a semantically misleading type called
  `VerifiedProvider` or `StableProvider`: an ID cannot become stable by Rust
  lifetime alone.
- After **measurement**, consider an optional short-lived
  `ProviderQuerySession` / `with_provider_dx12` (illustrative) bound to the
  actual runtime plus DX12 device. Enumerate once, preserve the selected opaque
  ID and group related pre-create queries. Be explicit that this is a **cached
  observation**, not a reservation; each native call can still fail and can
  observe a provider change. Do not make native override selection global.
- A batch entry point may instead execute ratio, dimensions, jitter and
  resource queries in a specified sequence under one selection step and common
  native coordination. It must return independent results, not fabricate a
  transaction or support certificate, and it should not invalidate the
  caller's ability to use simpler independent methods.
- If binding an `&mut Runtime` session, audit borrow/lifetimes against
  creation on both local/movable families, device COM lifetime, and the D013
  cross-thread/common-domain rules. Reuse the existing chain-building helpers.

**Verification gate.** Benchmark full startup cost with 1 versus multiple
related pre-create queries (report absolute and relative latency, allocation
counts, and native call counts). Controlled fixtures should include provider
removal/change between session calls, enumeration truncation/failure, ID reuse,
no silent fallback, identity mismatch and retained provider error codes. Do not
assert the cached session fixes TOCTOU; it changes cost and ergonomics only.

### 6. DX12 resource/configuration errors are sometimes detected at frame dispatch

**Verified (repository source).** `UpscaleResources::new` and
`without_exposure` in `dispatch/frame.rs:20–70` only assemble borrowed resource
references; `set_reactive_mask` and the composition-mask setter likewise defer
inspection. `dispatch/resources.rs:398–475` checks pure frame/options conditions,
exposure mode/resource presence (`432–435`), DIRECT-list type, canonical COM
identity, immutable texture metadata, extent/format/usage and device membership
immediately before native descriptor construction. `convert_desc:123–183`
verifies immutable texture descriptions but does not inspect actual GPU state.
Preflight returns structured errors without poisoning. Even a preflight call
cannot validate pixel content, queue order, backing-heap aliasing, current
resource state or future residency. D030 extends this inspection for generator
roles but is still a proposal.

**Problem.** Context-option/resource-bundle incompatibilities such as explicit
versus automatic exposure are usually discoverable during renderer setup, not
necessarily on the first frame. Receiving them at the first `dispatch()` makes
integration feedback slower, even though failure is non-terminal. Re-checking
immutable resource metadata every frame might also have a cost, but it is not
measured here and must not be claimed material.

**Recommended incremental API:**

1. Expose a **side-effect-free** preparation or validation method on the
   upscaler, e.g. `validate_dispatch(&self, &UpscaleDispatch) -> Result<(), Error>`
   using existing inspection and error types. This can be called during setup
   to produce errors earlier, and `dispatch()` **must still validate** when
   given arbitrary borrowed inputs. Do not represent the method as verifying
   real GPU states, texture pixels, submission, or provider support.
2. For frequently reused resources, consider `PreparedUpscaleResources`
   produced by one complete canonical identity, immutable descriptor and
   device check; tie it to the **specific context/profile** and resource
   lifetime, with an explicit safe or unsafe contract for mutable/rebound
   descriptors and heap aliasing. Retain relevant COM owners if it outlives
   caller borrows. A prepared object is **metadata-valid**, not
   `GpuStateValid` or `GpuSafe`; the dispatch remains `unsafe` unless queue,
   barriers and GPU-completion ownership are also addressed. Avoid an
   unnecessarily large cache or per-frame dynamic allocations.
3. Express reliable cross-field invariants with named construction modes where
   beneficial (e.g. explicit/automatic exposure preparation). Do not encode
   runtime DX12 device identity or current resource states in compile-time
   Rust generics. Keep a low-level borrowed route for integration flexibility.
4. Split pure frame validation from DX12 COM inspection so bad combinations
   fail before costly external calls. Add D030 generator-specific role/alias
   rules: two read-only colour roles may be the same COM object, but output
   must be distinct and shader-readable for subsequent upscale consumption.
   The existing generic `convert_desc` only checks the deny-SRV flag when
   `output == false`; simply reusing `output == true` without a new predicate
   would miss this D030 output requirement.
5. Correct the stale `dispatch/frame.rs:166` Rustdoc claiming a supplied mask
   must appear in provider required/optional bits: D025's accepted rule and
   current runtime behaviour do **not** impose that filter. This is a concrete
   documentation inconsistency worth fixing independently of any API change.

**Verification gate.** Tests must establish parity of prepared versus direct
validation for format, extent, array/mip/sample, flags, canonical identity,
wrong device/list, exposure mode, optional masks, active-size changes, and
poison precedence. Run controlled DX12 fixtures on Windows to exercise real
COM inspection, including resource/allocator reuse and D030's same-read-only
input allowance. For any cached metadata, prove that its invariants cannot go
stale or re-inspect on the fast path; do not label an unsafe GPU contract safe.

## Error-stage and recoverability matrix

| Event | Earliest trustworthy detection | Native FFI? | Retain context? | Caller duty |
|---|---|---|---|---|
| Invalid scalar or documented-range violation in optional strict API | Typed/checked construction | No | Yes | Fix the value |
| Mismatched exposure mode, format, extent, resource/device identity | Preflight/prepare | No FSR dispatch | Yes | Correct input; respect earlier recorded work |
| Unsupported generation **suspected** from requirements/version | No reliable generic predicate | Unknown until actual attempt | Not certified | Avoid guessing; use verified setup/probe or accept risk |
| Generator/main dispatch native non-OK | Native return after entry | Yes | **No** under D011/D030 | Discard affected list; drain earlier work; destroy/recreate |
| Configure native non-OK | Native Configure after entry | Yes | **No** under D029 | Already idle; terminal teardown/recreate |
| Acknowledged recording later abandoned | Renderer submission lifecycle | Already called | **No** under D011 | Invalidate never-submitted list; drain previous work; destroy/recreate |
| All GPU work completed and no executable unsubmitted list | Renderer-owned ordering/fence proof | Not required | Yes | Configure is allowed on a usable context |
| GPU device removed/command execution failed | DX12 runtime/debug/fence reporting | Depends | No automatic recovery claim | Follow device-loss/queue recovery separately |

The table concerns **eligibility and stated preconditions**, not an AMD native
atomicity guarantee. “No FSR dispatch” still permits DX12 `GetDesc`, `GetDevice`
or `QueryInterface` inspection. Past successful recordings must still be
submitted/finished or handled under D011 even if a *later* preflight fails.

## Trade-offs and recommended sequence

These recommendations are **not** accepted design decisions, implementation
permissions, or universal provider safety guarantees.

| Priority | Change | Benefit | Safety/cost qualification |
|---|---|---|---|
| P0 | D030: complete preflight, clear unsupported/poison diagnostics, recovery recipe | Prevent avoidable terminal failures; make unknown capability explicit | No unproven post-FFI unpoison |
| P0 | D011: documented aborted-frame lifecycle and testable renderer example | Make painful exceptional recovery deterministic | Context replacement remains required; no fabricated rollback |
| P1 | Optional strict tuning-value constructors beside native pass-through | Earlier, local errors without changing D029 behaviour | Source-documented bounds are not universal provider guarantees |
| P1 | Explicit validation/setup path and D025 Rustdoc fix | Detect predictable pipeline configuration mistakes before first frame | Dynamic GPU state still unsafe; dispatch still rechecks unless prepared invariants proven |
| P1 | Initial-tuning/batched-idle workflow | Avoid repeated GPU drains during routine setup and UI updates | No native transaction or arbitrary in-flight Configure |
| P2, measure first | Grouped provider-query session | Fewer repeated enumerations and less selection boilerplate | Not a provider reservation, can still fail, adds lifetimes/API complexity |
| Separate R&D | Truly recoverable errors / non-terminal abandonment / live in-flight Configure | Potentially large runtime ergonomic/performance gains | Needs provider-specific rollback, ordering and GPU tests, with explicit new contract |

### Suggested implementation seams (not public API commitments)

- `context_policy.rs`: preserve a **single** eligibility transition owner; do
  not add generator-specific poison state without new evidence. Existing
  `dispatch(eligibility, state, inspect, invoke)` already provides the
  preflight/no-poison separation.
- `upscaler/tuning.rs` plus `upscaler/common.rs`: put strict range validation in
  pure constructors/helpers, **outside** `configure`'s FFI mapping and outside
  the current pass-through contract.
- `upscaler/dx12/dispatch/resources.rs`: expose private reusable pure/COM
  inspection units and role-aware generator predicates, then optional public
  preparation API. Avoid storing unchecked native descriptors whose borrowed
  pointers outlive resources.
- `runtime/queries.rs` plus `query/native.rs`: group enumeration and provider
  query invocation at one seam only if a benchmark supports the added session
  API; preserve device matching and post-create effective-provider checks.
- An optional renderer-facing utility/example can centralise completion,
  invalidation and context recreation. It must never imply that `Drop` itself
  waits for GPU work or that a token proves a fence without owning submission.

## Baseline delta

1. **Confirmed:** accepted D011/D029 poisoning is intentionally terminal
   **after native entry**; Rust preflight errors are already recoverable.
   D030 proposes the same behaviour for generation, not an implemented feature.
2. **Corrected earlier framing:** D029 does **not** accidentally omit
   range validation. Finite passthrough and native clamping were explicitly
   accepted. Strict validation is an *optional additional policy* proposal,
   not a bug fix or a reason to silently break the direct control.
3. **Qualified earlier framing:** an idle context is a caller obligation after
   *unsafe dispatch*, not a new independent hidden obligation on safe
   `configure_tuning`; pre-first-dispatch Configure requires no drain.
4. **Qualified earlier framing:** D028 enumeration incurs repeated queries,
   but the performance cost has not been measured, and a cached token does
   not strengthen provider identity guarantees. It is a **lower-priority
   candidate**, not a demonstrated defect.
5. **New actionable point:** `frame.rs:166` contradicts D025's current supplied
   mask allowance. This is a documentation correction, independent of the six
   research questions, and is not a generator capability rule.
6. **Not established:** no general provider-wide side-effect-free native error
   categories, aborted-recording rollback, in-flight Configure safety, or
   generator feature predicate were found in the reviewed primary material.

## Limits, unanswered questions, and bounded next investigations

- Can a specific provider **provably** return code 2/7 before any context or
  list mutation? It requires version-pinned implementation evidence and
  controlled rejection tests including mixed prior recordings, not just the
  symbolic code name. How do signed FSR4/runtime providers behave?
- Does a disposable generator-context probe predict support in the main
  context for a given runtime/device/profile without altering shared provider
  state? Measure creation overhead and resource sharing; avoid a perpetual
  separate context without a demonstrated benefit.
- Is there any supported way to roll back a successful native recording after
  command-list abortion, and under what ring-buffer, history and multi-context
  conditions? No such mechanism is currently established.
- Can a specific provider safely Configure while its earlier submissions remain
  in flight, and how are changes sequenced with already staged GPU constants?
  Does a conservative idle window impose consequential real-world stalls?
- What are the actual costs of repeated provider enumeration and per-frame COM
  metadata inspection on supported deployments? Current evidence does not
  justify a performance assertion or cached `Prepared` complexity.
- Can a future renderer-independent optional session soundly own command-list
  submission, resource state, COM/allocator lifetimes and GPU completion?
  Without that scope, no new **safe** GPU-recording guarantee is credible.

A targeted experiment should use the repository's controlled-fixture and native
verification profiles, record the exact signed DLL/provider identity, DX12
runtime, hardware, command-list ordering, fences, negative outcomes and
readbacks. Windows native fixtures, independent MSVC ABI and actual GPU checks
for these proposals were **not executed** in this source investigation.
There is no identified change to project licensing, external SDK acquisition,
or binary redistribution from the proposals alone. Any future new native
components or packaged provider support matrix requires a separate audit.
