# Source investigation: published 0.2.0 compatibility and coherent 0.3.0 API design

- Investigation date: 2026-10-18.
- Question: Against the **publicly released** `fsr-sdk 0.2.0` interface, which changes in the supplied development snapshot already break source or behavioral/safety compatibility, and which changes suggested by the [ergonomics investigation](2026-10-18-src-error-recovery-and-preflight-ergonomics.md) would actually require a new breaking interface? If compatibility is secondary, what is the smallest *coherent, safety-oriented canonical API* worth shipping?
- Baseline: GitHub [`Render-Slop-Manufactory/fsr-sdk-rs` tag `v0.2.0`](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.2.0), release commit `fabb964`, dated **2026-10-10**, containing **`fsr-sdk 0.2.0` and `fsr-sdk-sys 0.1.1`**. Public source was read using exact `v0.2.0` GitHub raw URLs (accessed 2026-10-18). The second package's baseline is **0.1.1, not 0.2.0**.
- Candidate: supplied `fsr-sdk-rs-main` ZIP, SHA-256 `555f0357f7413a6400870a10c03f6c70dbef62c4618a17abdfec0a3a2e767366`; `CHANGELOG.md` labels it **unreleased `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`**. It contains no `.git` metadata. Decisions through D029 are accepted/implemented to their documented scope; **D030 is proposed, not implemented**.
- Method: manual cross-check of tagged public Rust sources, published release notes, development implementation, changelog, D011/D012/D013/D014/D018/D022/D024–D030, research, and Rust/Cargo compatibility guidance. Reviewed the six issues in the earlier research record; classified **signature/source**, **trait/type**, **behavioral**, **unsafe-contract**, and **unreleased-only** changes separately.
- Limits: no automated whole-tree `cargo-semver-checks` comparison or Rust compiler (toolchain absent); no tag archive or packaged crate was downloaded; no Windows DX12, native provider, or GPU tests were run here. Public tagged files are authoritative for the *manually inspected* surface; a complete absence-of-breaking-change claim for every export, feature combination, build target and `fsr-sdk-sys` declaration is **not established**. Links to source establish syntax, not runtime behavior.

## Inputs and provenance

**Published source, pinned to `v0.2.0`:**

- [Release and commit](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.2.0); [`fsr-sdk` manifest](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/Cargo.toml); [`fsr-sdk-sys` manifest](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk-sys/Cargo.toml).
- [Crate exports](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler.rs), [portable common inputs](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/common.rs), [DX12 owner](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12.rs), [dispatch implementation](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/dispatch.rs), [borrowed frame input](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs), [sharpening](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/dispatch/sharpening.rs), [error types](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/error.rs).
- [Local runtime](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/runtime.rs), [movable runtime](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/runtime/movable.rs), [movable upscaler](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/movable.rs), and [crate-level contracts](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/lib.rs).

**Supplied development snapshot:**

- [`CHANGELOG.md`](../../../CHANGELOG.md), [`crates/fsr-sdk/src/upscaler.rs`](../../../crates/fsr-sdk/src/upscaler.rs), [`upscaler/common.rs`](../../../crates/fsr-sdk/src/upscaler/common.rs), [`upscaler/dx12.rs`](../../../crates/fsr-sdk/src/upscaler/dx12.rs), [`dispatch/frame.rs`](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs), [`dispatch.rs`](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs), [`context_policy.rs`](../../../crates/fsr-sdk/src/context_policy.rs), [`upscaler/tuning.rs`](../../../crates/fsr-sdk/src/upscaler/tuning.rs), [`runtime/queries.rs`](../../../crates/fsr-sdk/src/runtime/queries.rs).
- [D011](../../adr/D011.md), [D012](../../adr/D012.md), [D013](../../adr/D013.md), [D014](../../adr/D014.md), [D018](../../adr/D018.md), [D022](../../adr/D022.md), [D025](../../adr/D025.md), [D028](../../adr/D028.md), [D029](../../adr/D029.md), [D030](../../adr/D030.md); [previous ergonomics source record](2026-10-18-src-error-recovery-and-preflight-ergonomics.md).
- [Rust/Cargo SemVer compatibility guide](https://doc.rust-lang.org/cargo/reference/semver.html) (accessed 2026-10-18). In Cargo's `0.y.z` convention a change in `y` (`0.2` to `0.3`) is the incompatible-release boundary; this is a **permitted** break, not evidence that a particular API break occurred.

## Classification: what counts as a break?

1. **Compile/source break:** a previously compiling public call, import, trait bound or pattern no longer compiles (e.g. `dispatch()` signature changed, constructor now returns `Result`, removal of an exposed trait).
2. **Behavioral break:** the same source compiles but a valid previously accepted use returns a new error, clamps differently, changes provider routing, changes resource default, or changes temporal results. Safety-improving rejection can still be a behavior change.
3. **Unsafe-contract break:** an `unsafe` caller must establish materially stronger conditions than in the release, even if signatures and default runtime behavior remain unchanged. This matters in an FFI/GPU wrapper, irrespective of compiler checks.
4. **Unreleased-only redesign:** a change to something introduced after 0.2.0 is **not** a breaking change relative to 0.2.0; it may nonetheless be a design reversal against an accepted ADR or affect consumers of Git/main revisions.
5. **Additive/non-breaking:** new methods, new optional configuration with the same defaults, tighter validation that rejects only cases already expressly outside 0.2.0's contract, internal implementation changes and corrected documentation. Distinguish “not source-breaking” from “no potential behavioral impact.”

This record does not use download counts, stars or presumed user adoption as evidence of API compatibility. A no-user assumption can inform migration cost, not soundness.

## 0.2.0's established public contract

**Verified (tag source):** `fsr-sdk 0.2.0` exposes `Runtime::load_unchecked(path)`, `Upscaler::new(&Runtime, &ID3D12Device, UpscalerOptions)`, `unsafe Upscaler::dispatch(&mut self, UpscaleDispatch<'_>) -> Result<(), Error>` and consuming `Upscaler::destroy(self)`. Under `movable`, matching `MovableRuntime::load_unchecked`, `MovableUpscaler::new`, `dispatch`, and `destroy` exist. The two owner families have different threading contracts (`!Send` local versus `Send + !Sync` movable).

`UpscalerOptions::new(Dimensions, Dimensions)` has private fields and `Debug + Clone`; `Dimensions::new(u32,u32) -> Result`; `UpscaleResources::new(color,depth,motion,exposure,output) -> Self` and `UpscaleDispatch::new(command_list, resources, jitter, frame_time, camera, reset) -> Self` have private fields. `UpscaleResources::exposure()` **already returns `Option<&ID3D12Resource>`** in 0.2.0, despite always being `Some` then; changing that getter's observable result under explicit automatic-exposure opt-in is not a signature change. `UpscaleDispatch::set_sharpening` and checked `NativeSharpness` are already published. Default creation profile is explicit exposure, linear non-HDR, ordinary finite depth, fixed dimensions and no optional masks.

A successful `dispatch()` was already **recording only**: caller-controlled recording list, queue, fence, states, residency, lifetimes and ordered submission, no automatic waiting; native failure poisons; abandoning an acknowledged recording requires retiring the context. Those are **0.2.0 contracts**, not novel 0.3.0 defects. Native `Ok` does not establish GPU execution or image quality.

The baseline `Error`, `Operation`, `DispatchValidation`, `ResourceRole` and other diagnostic enums were already marked `#[non_exhaustive]` in the public source. Adding variants does **not normally break** well-formed external exhaustive matching, because external callers already needed a fallback; field/variant structural changes and new observable errors still merit review.

## Findings: confirmed/current-source differences from 0.2.0

| Boundary | 0.2.0 | Supplied development snapshot | Classification | Recommendation |
|---|---|---|---|---|
| `load_unchecked` (local and movable) | Unsafe native ABI, threading, lifetime and coordination trust | D018/D022/D028/D029 extend trust to native query, Configure, selected provider and numerical/temporary-pointer/GPU-state details | **Confirmed unsafe-contract break**, same Rust signatures | Acknowledge explicitly in 0.3.0 migration; audit call-site safety comments |
| Owner/dispatch constructors, principal frame inputs | Signatures listed above | Existing signatures/getters retained, additional modes/setters and methods | **No identified source break on inspected core public calls** | Preserve unless a proven unsound representation requires replacement |
| Creation-time resource requirements | 0.2 context creation did not ask for/validate live requirements | Snapshot performs an additional post-create native requirements query and rejects unknown/unfulfilled required inputs | **Potential behavioral compatibility break** on providers with those responses, even with unchanged constructor signature | Specify fail-fast rationale; test default 0.2 profile and provider variations |
| Error enumerations | Public `#[non_exhaustive]` | More operations, validations, invariants and error cases | Primarily **additive API**, changed runtime error possibilities | Test diagnostic precedence and downstream error handling |
| Profile defaults | Only fixed explicit-exposure profile | Opt-in automatic exposure, depth/motion, HDR, dynamic resolution, encoding, masks, provider | **Generally additive**; defaults intentionally preserved | Golden-test default native descriptors against tag |
| `fsr-sdk-sys` | `0.1.1` | Planned `0.1.2` adds raw descriptors/constants | Additive for inspected declared surface; **full ABI/source diff not verified** | Independently ABI-check; audit separate `0.1.1` baseline |

**Crucial qualification:** strengthening `Runtime::load_unchecked` does **not** automatically make every previous caller unsound. It adds proof obligations when using newly available native operations or providers; some may be conditional. D018's accepted wording and the development changelog correctly classify this as a contract change. Precisely enumerate which obligations apply to old default-only 0.2 integrations when finalizing release notes; do not portray all new optional operations as unavoidable requirements on integrations that never invoke them unless the actual acquisition contract says so.

**Not confirmed:** deletion, renaming or mandatory signature replacement of the reviewed 0.2.0 core construction/dispatch methods. The tag was not locally compiled against the development build, so this is a **targeted source finding**, not a complete SemVer certification. Public `#[non_exhaustive]` and retained defaults mitigate many foreseeable changes, but do not eliminate behavior/safety compatibility questions.

## Six proposed improvements, measured **against 0.2.0**

### A. Native failure/poisoning, including proposed D030

**0.2.0 anchor:** `unsafe Upscaler::dispatch` poisons after native non-OK, but not after pre-FFI metadata/scalar inspection failure (tag dispatch, lines 63–76; tag owner, lines 39–54). The development `context_policy.rs` generalizes this to `Eligibility::{Usable,DispatchFailed,ConfigureFailed}`. D030 would dispatch a *new* standalone generator operation through the same context and eligibility state; **it is not implemented**.

**Break analysis:** inspecting more validatable conditions internally before calling FFI, preserving exact native error codes, improved error text and a recovery example are **not** signature breaks. A new `generate_reactive_mask` method/type would be additive relative to 0.2.0. Reclassifying selected native errors as non-poisoning would change the **established terminal safety contract** of the 0.2.0 upscaler if applied to its `dispatch()`; changing only new Configure/generator paths would be unreleased-only.

**Coherent canonical path:** retain the single `unsafe dispatch(...)` shape and the strict native-error rule. Make full inspectable preflight **internal and compulsory** for each operation. Preserve a structured `Error::Native { operation, code }`, with context-lifecycle diagnostics and clear recovery instructions; do not require callers to remember a separate `validate_*` method. D030 should reuse the same state machine, share resource inspection primitives, but apply **its own role-aware** alias and output-SRV rules. Unsupported provider features are not reliably knowable from requirements bits or source-independent type information; a speculative live trial is not harmless. Do not add a mandatory `try_dispatch`/`checked_dispatch` alternative.

**Only break the 0.2.0 terminal contract if:** named provider/runtime-specific evidence demonstrates *every relevant early error path* leaves provider, allocator and recorded command list in a safely reusable state. Symbolic return codes do not prove atomicity. Even then a separately scoped, documented recovery rule may be preferable to silently weakening `dispatch()`.

### B. Abandoned successful command recording: D011

**0.2.0 anchor:** `dispatch()` returns `Result<(), Error>` on recording, not a submission proof; abandonment requires the caller to stop using the context. The SDK does not own `ID3D12CommandQueue`, fence or allocator (tag dispatch, lines 27–55). The prior research shows list reset is not provider-state rollback.

**Break analysis:** documenting invalidation/recreation and implementing diagnostics **does not** change the API. Replacing `dispatch()` with `begin_recording`/`submit`/`finish` or making `dispatch()` return a mandatory submission token would be a **real 0.2.0 signature, lifecycle and borrow-model break** for both owner families. A token around an externally owned command list **does not itself prove** submission or completion; the renderer can submit behind the wrapper's back or drop a token while work is running.

**Coherent canonical path:** leave the low-level recording operation unsafe and direct. Offer a single tested abort/recreate workflow in the examples; preserve poison/terminal semantics after an acknowledged but discarded recording. *Do not* add a new public session abstraction only for comfort: making it genuinely safe demands mediating every relevant list, queue/fence, resource lifetime and ordering path, defeating the current renderer-independent scope.

**Conditional redesign trigger:** reproducible evidence that real integrations cannot satisfy the lifecycle contract or an agreed scope change that lets the wrapper own **the entire submission/fencing relationship**. A mandatory token justified only by nicer call syntax fails that test.

### C. Tuning value domain policy: D029

**0.2.0 anchor:** no `TuningParameter`, `TuningValue`, or `configure_tuning` existed. D029 currently accepts all finite `TuningValue::Value(f32)` and forwards bits for native clamping; upstream documented ranges differ per parameter. `Value(f32)` is a public enum payload and cannot enforce a parameter-dependent invariant at enum construction.

**Break analysis:** making `configure_tuning(parameter, value)` reject documented out-of-range finite values *before FFI* would be a **behavior change relative to unreleased D029**, **not a 0.2.0 break**. Replacing the public `TuningValue::Value(f32)` representation by private-field typed values or parameter-specific constructors would be a **source break for current main/Git users only**. Retaining both `configure_tuning` and `configure_tuning_checked` creates competing pathways; not necessary for 0.2 compatibility.

**Coherent canonical choices (choose ONE):**

- **Recommended if the library prioritizes detecting application mistakes:** one `configure_tuning(parameter, TuningValue)` method; validate the key-specific documented domain internally *before* FFI; reject on violation without poison. Retain `ProviderDefault`, preserve in-domain `f32` bits, make documentation explicit that bounds are documented for the selected SDK, **not universal effective-value guarantees**. A private `validate_tuning(parameter, value)` provides one source of truth and catches even direct enum construction. Provide a new precise `Error`/reason for the parameter and bad domain. Accept a deliberate D029 policy reversal.
- **Recommended if the library prioritizes strict FidelityFX pass-through:** keep one `configure_tuning` and deliberately preserve finite native clamping, document its behavior unequivocally. Avoid a second checked API. Native clamping is an upstream behavior and not automatically incorrect; application-level strictness can be imposed before calling the SDK.

**Avoid:** silently Rust-side clamping; claiming `Ok(())` means value applied; baking FSR3 defaults into `ProviderDefault`; a provider-independent compile-time proof of support. A strict rule improves early *runtime* diagnosis, not compile-time knowledge of driver/provider state.

### D. GPU idle before tuning: D029

**0.2.0 anchor:** no Configure API. Existing unsafe `dispatch()` already requires GPU completion before destroy and guarantees neither submission nor fences.

**Break analysis:** keeping `configure_tuning(&mut self, ...)` and requiring an idle context after earlier unsafe dispatches affects only newly introduced Configure use, thus **no 0.2.0 source break**. Tightening its ordering contract is a new safety obligation for users adopting it, not a compile-time change. Replacing the published `dispatch()` with an owning typestate system that controls queue/fences would be a major 0.2.0 break and a separate architectural commitment.

**Coherent canonical path:** retain a single `configure_tuning` and document a normal **configure-before-first-dispatch** sequence; permit batching desired settings into one idle interval at the renderer level. Five native Configure calls are not an atomic transaction. `&mut self` and even a caller-provided fence value cannot certify GPU idle while unsubmitted executable lists exist. No `GpuIdleToken` or `PendingTuning` public type without a real proof or measured need. A safely relaxed in-flight Configure contract needs *specific provider* staging/order evidence; FSR3 source alone cannot certify closed providers.

### E. Provider enumeration and routing: D028

**0.2.0 anchor:** no public provider enumeration/selection/utility-query API. D028 adds option selection, repeated explicit-provider query entry points and enumeration-before-override to current unreleased code.

**Break analysis:** changing or replacing those D028 method names/signatures, introducing a canonical provider-query selection representation, or removing the repeated convenience variants would break **current main**, **not 0.2.0**. Existing `Upscaler::new` and `UpscalerOptions::new` can remain untouched. Repeated count/fill cost and repeated selection boilerplate are not yet measured as serious performance defects. A cached `ProviderId` or `ProviderQuerySession` is not a provider reservation and can become stale.

**Coherent canonical path:** choose one primary way to specify a provider for setup queries and context creation, consistent with existing `UpscalerOptions::provider() -> Option<ProviderId>` and `set_provider`. Keep native membership/identity checks internal, and preserve default-provider semantics. Before adding sessions or caching, measure cost and list a *real* application with repeated setup queries. If the four `_for_provider` methods duplicate a default path without adding correctness, consider a **single provider-selection argument model** before 0.3.0 publication; no need to support both styles. This is an architectural preference requiring API review, **not a proven security defect**.

**False guarantee to reject:** the fact that a provider was enumerated, even just now, does not mean the provider/version is reserved or that every subsequent operation supports it.

### F. Earlier resource validation: core dispatch and D030

**0.2.0 anchor:** `UpscaleResources::new(...) -> Self` and `UpscaleDispatch::new(...) -> Self` assemble borrows *without COM inspection*. Actual device identity, texture shape/format/extent and role checks run within `unsafe dispatch()` before native FSR entry. The caller supplies actual states, aliasing, contents and GPU lifetime (tag frame, lines 8–25, 74–98).

**Break analysis:** moving pure checks earlier *inside dispatch* or factoring common inspection functions is not a signature break. Changing these constructors to return `Result`, requiring a `PreparedResources`/`PreparedDispatch` wrapper at every call, or making `dispatch` a method of a prepared-only type **would break 0.2.0 source**. Construction-time DX12 inspection may also duplicate per-frame work, require an owner/device reference absent from the constructors and still cannot prove actual resource states or GPU completion.

**Coherent canonical path:** keep existing constructors and `dispatch`. Reject any inspectable invalid combination in the **operation itself** before FSR FFI. Split CPU-only cross-field checks (e.g. exposure profile, size, encoding mismatch) from COM inspection for clear error precedence; optionally validate static creation/profile combinations when the owner is constructed, but never promise resource state knowledge from metadata. Avoid making preflight an opt-in safety condition. If caching is later justified by measurements, prove resource descriptors, identity, feature requirements and active sizes cannot go stale; do not market a prepared resource as GPU-safe.

**Concrete non-API fix:** `dispatch/frame.rs` currently claims supplied masks must appear in provider required/optional bits; accepted D025 permits forwarding valid supplied masks even without those bits. Fix the stale Rustdoc independently.

## Compatibility matrix and priority

| Candidate decision/change | Breaks public 0.2.0 Rust source? | Changes 0.2.0 unsafe/behavior contract? | Suggested outcome |
|---|---|---|---|
| Keep existing signatures + mandatory internal preflight (main/generator) | **No** | Earlier recoverable errors only within already-invalid inputs | **Do, P0** |
| Additional `generate_reactive_mask` method and request | **No** (new surface) | Shares terminal semantics; new feature only | **Proceed only after D030 review, P0** |
| Improve native-poison diagnostics and renderer recovery guidance | **No** | Preserve terminality | **Do, P0** |
| Strengthened unsafe acquisition trust in current D018+ | **No** | **Yes: confirmed safety-contract expansion** | **Keep with explicit migration warning, P0** |
| Post-create required-input query/rejection | **No** | **Possibly**, conditional provider-specific construction failure | **Review/verify, P0** |
| Enforce strict tuning ranges in canonical `configure_tuning` | **No** | **No, feature new after 0.2.0** | **Resolve D029 policy once, P1** |
| Rework D028 query naming/provider parameter representation | **No** | **No, methods new after 0.2.0** | **Consider before shipping only if simpler, P2** |
| Document idle-Configure sequencing / batch during setup | **No** | Additional new-feature usage preconditions, not 0.2 existing use | **Do, P0** |
| Replace published `dispatch` with recording/commit/fence tokens | **Yes** | **Yes; substantial** | **Reject without end-to-end soundness proof** |
| Make `UpscaleResources::new` / `UpscaleDispatch::new` fallible or mandatory prepared | **Yes** | Additional construction-time side effects/constraints | **Not justified by current evidence** |
| Permit non-terminal errors after *published* `dispatch` native failure | No signature change | **Yes; weakens existing terminality** | **Reject without pinned native proof** |
| Allow unsubmitted successful recordings to be silently abandoned | No signature change | **Yes; unsound unless native rollback established** | **Reject** |
| Silently allow Configure against outstanding GPU work | No existing signature | New-feature safety contract weaker; native race risk | **Reject without evidence** |

**Interpretation:** the earlier six usability findings do **not** establish a necessary *compile-time breaking change* against `fsr-sdk 0.2.0`. The **one confirmed cross-release safety-contract change** in the reviewed current source is the expanded native trust of `load_unchecked`; a new construction-time requirements query has plausible behavioral compatibility effects on certain providers. Breaking redesigns of published dispatch and frame constructors are conceivable, but the evidence in the first record does **not justify** them now.

## Recommendation: a coherent 0.3.0, not six parallel public APIs

**Design invariants proposed for decision review (not accepted):**

1. One canonical low-level DX12 `dispatch` route and one corresponding optional-generator route; never require a separately invoked validator for safety. `unsafe` explicitly delimits states/content, queue ordering and GPU lifetime that Rust cannot observe.
2. Invalid pure input is rejected immediately, invalid inspectable resources before native FSR entry, and errors that might have mutated native state remain terminal. A rejected preflight must not poison; native `Ok` cannot be treated as GPU completion or feature-effectiveness proof.
3. No duplicate normal/checked/fast/provider-specific-overload API families merely to avoid breaking unreleased code. For truly distinct semantic operations use distinct names; for a single operation agree one value and provider-selection policy *before release*.
4. Default v0.2.0 behavior remains the baseline until an explicit safety, correctness or user-facing evidence justifies replacing it: fixed explicit exposure, no optional masks, sharpening disabled, ordinary depth, initial `reset` caller-driven. Opt-in settings should not change default descriptors.
5. An upstream unsafe native boundary cannot be made safe by renaming functions or hiding a fence precondition in a type. Avoid adding `Prepared`, `GpuIdle`, `RecordingToken` or `ProviderSession` as public types unless their invariants are formal, enforceable and needed. Lightweight private factoring and tested examples are preferable to speculative new types.
6. Do not privilege preservation of main-branch 0.3.0 signatures over a coherent released API, but treat **accepted ADR reversals** (especially D029's deliberate native clamping) as explicit decisions requiring rationale. A source investigation cannot silently amend an ADR.

**Concrete design review queue:**

- **P0 — Confirm and disclose:** D018 expanded `unsafe` contract; construction-time requirements-query behavior; verify old default-path dispatch semantics with the published tag. Preserve the 0.2.0 core API provisionally.
- **P0 — Make D030 complete:** internal role-aware preflight, single poison policy, no fabricated capability detection, clear unsupported/recovery behavior, native resource-state and ABI verification gates. D030 is proposal only.
- **P1 — Settle one D029 policy:** strict documented-domain rejection *versus* intentional full-finite native pass-through; do not ship two normal routes. Exercise all five parameters and both owner families in controlled fixtures, then native tests.
- **P1 — Clarify D011 lifecycle:** error/abandonment recipes, multiple contexts per list and recreated-context temporal reset; no public recording-session type unless the wrapper takes real submission/fence responsibility.
- **P2 — Reconsider D028 ergonomics only against concrete setup flows/performance observations.** If the current explicit-provider methods are sensible, keep them. Do not introduce a cache or new session just to avoid repeated native calls whose cost is unmeasured.
- **P2 — Move reusable validation logic privately, not into a second required public preparation workflow.** Fix the D025 stale mask documentation.

## Reproducible API-compatibility verification before publication

When a Rust/Windows toolchain and the actual tag/package artefacts are available:

1. Materialize `v0.2.0` (**commit `fabb964`**) and the chosen 0.3 release commit as separate trees. Compare `fsr-sdk 0.2.0` to the new wrapper and independently compare `fsr-sdk-sys 0.1.1` to the new sys crate; record file hashes/revisions, feature gates and MSRV. Do **not** compare development source to a moving `main` URL as if that were 0.2.0.
2. Check public surface both structurally (`cargo-semver-checks` if available) and by compiling minimal **external consumers** using only `0.2.0` published items. Compile on the supported `x86_64-pc-windows-msvc` target (actual Windows execution separately), with `dx12`, `movable`, and portable no-DX12 configurations. Account for `#[non_exhaustive]`, trait/auto-trait changes, blanket impl/coherence conflicts and Cargo feature resolution.
3. Include consumer cases for `Runtime::load_unchecked`, `UpscalerOptions::new`, `UpscaleResources::new`, `UpscaleDispatch::new`, `set_sharpening`, `Upscaler::new`, `dispatch`, consuming `destroy`, the movable pair, portable scalar constructors and error matches with fallbacks. Run the same consumer source against the intended new release with no modifications; document each required edit.
4. Compare **default creation and frame descriptors** under controlled fixtures: creation flags/version/chain, exposure texture pointer, dimensions, sharpening defaults, GPU-state assumptions, native error/poison precedence. Test creation against providers with unknown/required resource bits and expose new early `Err` cases; do not assume identical native behavior across drivers.
5. Audit `unsafe` `# Safety` on tagged and candidate `load_unchecked`, `dispatch` and movable variants line by line. Produce a separate safety-obligations delta even if ordinary source compilation passes. Windows native confidence requires loading/querying, actual recording, submission, fences and (for D030) generated-mask readback; cross-compilation is not execution.
6. Maintain an explicit migration section: **source break**, **behavior break**, **unsafe-contract break**, **new opt-in feature**. Every incompatible change should cite its actual failure/misuse scenario and why a private fix is insufficient. If no such scenario exists, favor preserving the existing public call.

## Baseline delta and unresolved questions

- **Verified:** 0.2.0 already exposes its private-input constructors and poisoning/abandonment obligations. A mandatory session or fallible-constructor redesign would be a genuine 0.2.0 break; preflight improvements do not require it.
- **Verified:** D028/D029/D030 surface is post-0.2.0; changing it before 0.3.0 release is not a break relative to the public version. D030 itself has not been implemented.
- **Verified (current source/decision):** D018 expands unsafe runtime acquisition obligations, acknowledged in `CHANGELOG.md` as a public safety-contract break. New required-resource pre-create/post-create checks potentially reject configurations that creation did not previously reject; provider-specific effect is unresolved.
- **Corrective synthesis:** the previous ergonomics research favored additive two-tier tuning/preflight/session APIs in some sections. Under the maintainer's **one coherent canonical API** criterion, these should instead be **internal obligatory checks** or **one explicitly selected D029 policy**, unless a second API has genuinely distinct semantics. This source record records a design recommendation, **not** a revision to the earlier historical evidence or to D029.
- **Unresolved:** absence of a complete automated tagged-package API diff; changes outside the audited exports; Rust auto-trait changes in all feature configurations; 0.2 versus 0.3 driver/provider behavior; D030 generator support, error atomicity, and GPU state restoration; performance costs of enumeration, COM inspection and idle Configure; existence of a provably safe renderer-agnostic recording session.

**Research outcome:** There is currently **no evidenced reason to break the published 0.2.0 dispatch or frame-construction signatures for the six ergonomics issues**. Prefer a coherent 0.3.0 with a deliberately reviewed native safety-contract expansion, mandatory early validation inside operations, a resolved D029 single-policy choice, and D030 implemented only with its stated native verification gates. If future GPU/FFI evidence establishes that an old signature encodes a false safety guarantee, break it without hesitation; this source investigation has not established that case.
