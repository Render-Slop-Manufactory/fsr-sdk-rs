# Source investigation: full public API, extensibility and justified breaks from 0.2.0

- **Investigation date:** 2026-10-18.
- **Question:** If preserving compatibility has negligible present-day user cost, what should actually change in the whole `fsr-sdk`/`fsr-sdk-sys` public API to make it sound, coherent, hard to misuse, and extensible to the intended SDK coverage? Which proposed changes *really break the published 0.2.0 baseline*, and which affect only unreleased work?
- **Scope:** Both crates, Cargo features and target gates, exposed Rust modules/types/methods and unsafe contracts, runtime and provider boundaries, ownership/teardown/threading, numerical inputs, DX12 resources/command recording, Query/Configure/error semantics, backend and future-effect extensibility, D030. The audit evaluates design and visible implementation, **not** all possible native execution paths for memory soundness.
- **Published baseline:** [`Render-Slop-Manufactory/fsr-sdk-rs` v0.2.0](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.2.0), commit `fabb964`, published 2026-10-10; packages **`fsr-sdk 0.2.0`** and **`fsr-sdk-sys 0.1.1`**. The two packages do *not* share the same version.
- **Development candidate:** user's ZIP `fsr-sdk-rs-main(8)(2).zip`, SHA-256 `555f0357f7413a6400870a10c03f6c70dbef62c4618a17abdfec0a3a2e767366`. `CHANGELOG.md`/`ROADMAP.md` target unreleased `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`; D001–D029 include accepted decisions; **D030 is still proposed**. ZIP lacks Git metadata; do not represent its tree as a particular Git commit.
- **AMD ABI reference:** FidelityFX SDK **v2.3.0**, pinned source commit [`60f4ea81909200d8542eca14dccb2628b763a9a3`](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3), not AMD repository `main` at an unspecified later date.
- **Method:** Read public Rust source (including sys exports), key private implementations/tests, Cargo manifests, D001–D030 as relevant, SCOPE, ROADMAP, existing research records and release docs; compare the **published** v0.2.0 tag's manually accessed source/manifest with current development source; examine the pinned AMD Upscale and Frame Generation headers for extension constraints; consult Rust/Cargo and DX12 primary docs. Source lines below refer to the **supplied development ZIP** unless prefixed `v0.2.0`.
- **Evidence status:** **Verified** means source-level fact only; **Inference** means architectural consequence or recommended design; **Unresolved** means empirical/native proof or full mechanical compatibility diff is missing.

## Inputs and primary sources

Published v0.2.0 source (accessed 2026-10-18): [release](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.2.0), [`fsr-sdk` manifest](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/Cargo.toml), [`fsr-sdk-sys` manifest](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk-sys/Cargo.toml), [`fsr-sdk` exports](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/lib.rs), [`upscaler` exports](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler.rs), [common scalars/options](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/common.rs), [DX12 upscaler](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12.rs), [frame/resource API](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs), [dispatch](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/dispatch.rs), [errors](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/error.rs), [runtime](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/runtime.rs), [movable runtime](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/runtime/movable.rs), [movable upscaler](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk/src/upscaler/dx12/movable.rs), [sys exports](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk-sys/src/lib.rs), [sys loader](https://raw.githubusercontent.com/Render-Slop-Manufactory/fsr-sdk-rs/v0.2.0/crates/fsr-sdk-sys/src/loader/windows.rs).

Development: [`SCOPE.md`](../../../SCOPE.md), [`ROADMAP.md`](../../../ROADMAP.md), [`crates/fsr-sdk/Cargo.toml`](../../../crates/fsr-sdk/Cargo.toml), [`crates/fsr-sdk-sys/Cargo.toml`](../../../crates/fsr-sdk-sys/Cargo.toml), [`crates/fsr-sdk/src/lib.rs`](../../../crates/fsr-sdk/src/lib.rs), [`runtime.rs`](../../../crates/fsr-sdk/src/runtime.rs), [`runtime/movable.rs`](../../../crates/fsr-sdk/src/runtime/movable.rs), [`upscaler.rs`](../../../crates/fsr-sdk/src/upscaler.rs), [`upscaler/common.rs`](../../../crates/fsr-sdk/src/upscaler/common.rs), [`upscaler/dx12.rs`](../../../crates/fsr-sdk/src/upscaler/dx12.rs), [`dispatch.rs`](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs), [`dispatch/frame.rs`](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs), [`dispatch/resources.rs`](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs), [`query.rs`](../../../crates/fsr-sdk/src/query.rs), [`error.rs`](../../../crates/fsr-sdk/src/error.rs), [`context.rs`](../../../crates/fsr-sdk/src/context.rs), [`context_policy.rs`](../../../crates/fsr-sdk/src/context_policy.rs), [`sys API`](../../../crates/fsr-sdk-sys/src/api.rs), [`sys upscale ABI`](../../../crates/fsr-sdk-sys/src/upscale.rs), [`sys loader`](../../../crates/fsr-sdk-sys/src/loader/windows.rs).

Decision evidence: [D005](../../adr/D005.md), [D008](../../adr/D008.md), [D009](../../adr/D009.md), [D011](../../adr/D011.md), [D012](../../adr/D012.md), [D013](../../adr/D013.md), [D014](../../adr/D014.md), [D015](../../adr/D015.md), [D016](../../adr/D016.md), [D018](../../adr/D018.md), [D021](../../adr/D021.md), [D022](../../adr/D022.md), [D023](../../adr/D023.md), [D024](../../adr/D024.md), [D025](../../adr/D025.md), [D027](../../adr/D027.md), [D028](../../adr/D028.md), [D029](../../adr/D029.md), [D030](../../adr/D030.md). Related records: [early-error/recovery investigation](2026-10-18-src-error-recovery-and-preflight-ergonomics.md), [focused 0.2.0 compatibility investigation](2026-10-18-src-v020-api-breaks-and-coherent-030-design.md), [upscaling coverage inventory](2026-10-18-src-upscaling-release-surface-inventory.md), [upscaling synthesis](../upscaling-coverage.md).

Upstream and conventions (accessed 2026-10-18): [pinned AMD Upscale header](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h), [pinned AMD Frame Generation header](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/framegeneration/include/ffx_framegeneration.h), [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html), [Cargo feature conventions](https://doc.rust-lang.org/cargo/reference/features.html), [Rust API Guidelines: future proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html), [Microsoft command-list closing/allocator completion](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-close).

## Scope and criteria: a meaningful breaking change

A **source break** removes/renames an import or method, changes its arguments/types/trait bounds, requires a formerly unnecessary feature, or makes construction impossible. A **safety-contract break** strengthens `unsafe` preconditions even when signatures are unchanged. A **behavioral break** newly rejects previously accepted input or changes observable failure/selection/default behavior. A **manifest/feature break** removes or changes the meaning of a published Cargo feature. A **development-only break** affects D018–D030 features introduced after 0.2.0, not the published baseline.

Rank a break as *justified* only if it removes a demonstrated unsoundness, removes a misleading guarantee, or resolves an actual extensibility collision that cannot reasonably be corrected internally or additively **without producing competing canonical APIs**. “Could be nicer,” “could wrap more in a builder,” “could model GPU state in typestate” are not sufficient. Distinguish native guarantees, wrapper checks, and caller `unsafe` obligations.

## 1. Public surface inventory and 0.2.0 boundary

**Verified:** The publicly shipped 0.2.0 `fsr-sdk` had `error`, `upscaler::{common, dx12}`, and Windows+DX12 `runtime` modules. Portable checked common values were `Dimensions`, `JitterOffsetPixels`, `FrameTimeMillis`, `CameraParameters`, `UpscalerOptions`. DX12 exported `Upscaler`, `UpscaleDispatch`, `UpscaleResources`, `Sharpening`, `NativeSharpness`, `ID3D12Device`, `ID3D12GraphicsCommandList`, `ID3D12Resource`. `Runtime`, and under optional `movable`, `MovableRuntime`/`MovableUpscaler` were public; `Runtime::load_unchecked`, `Upscaler::new`, `unsafe dispatch`, `destroy`, constructors/getters/setters and error enums formed the principal contract. See pinned source above.

**Verified:** Unreleased main adds `query` public module; `QualityMode`, `JitterPhaseCount`, `ProviderInfo`/`ProviderId`, `ResourceRequirements`, `ResourceFlags`, `GpuMemoryUsage`; runtime and live-context Queries; options for reversed/infinite depth, motion profile, HDR/nonlinear/auto exposure/dynamic resolution/provider selection; additional per-frame settings/masks; context Configure; D030 only proposed. These *do not automatically count as breaking changes from v0.2.0*.

**Verified:** The `fsr-sdk-sys` **0.1.1** baseline exposes C ABI types, loader and DX12 integration; development **0.1.2** adds more upstream declarations. This is a raw crate with an independently evolving public ABI, not a hidden implementation detail for direct sys consumers; a complete sys symbol/layout diff is not established by this manual review.

| Surface | Published 0.2.0 | Development snapshot | Compatibility finding |
|---|---|---|---|
| crate/module backend gates | portable `error`, `upscaler::common`; DX12 gated; optional `movable` | same, plus portable `query` | additions, not an identified source break |
| local DX12 runtime acquisition | `unsafe Runtime::load_unchecked(path)` | same signature; broader provider/Query/Configure trust | **confirmed strengthened safety contract (D018+)** |
| local + movable context ownership | distinct `!Send/!Sync` and `Send/!Sync` families; consuming `destroy` | same published types/signatures | no source break identified |
| creation | `Upscaler::new(runtime, device, UpscalerOptions)` | same, more validation/provider capabilities | potential **behavioral** changes for some provider configurations |
| frame assembly/recording | private-field frame/resource constructors; `unsafe dispatch` | constructors/signature retained, many setters added | no established source break |
| scalar contracts | nonzero sizes; finite jitter/frame/camera; checked sharpness | extensions; older constructors retained | no identified source break |
| native terminal state | dispatch failure poisons; abandoned successful recording terminal by caller contract | also Configure failure poisons | extended behavior only for new Configure; original dispatch behavior intact |
| Query/Configure/provider | not in 0.2.0 | new and still redesignable | **development-only compatibility** |
| `vulkan` Cargo feature | present but deliberately causes compile error | same | **existing feature-design problem, not a newly introduced break** |
| sys crate | `fsr-sdk-sys 0.1.1` | `0.1.2` planned; new ABI slice | raw API must be independently compared |

**Limits:** No downloaded tagged source archive, no `cargo-semver-checks`, no builds or GPU execution; therefore this is *not* a claim of exhaustive symbol-by-symbol or target-by-target parity. A release gate should compare both published crates mechanically.

## 2. Core design judgment: preserve the good invariants

### 2.1 Ownership and terminal destruction — keep the model

**Verified:** `context.rs:46–63,78–121,124–169` owns pinned descriptor storage, retained device/runtime and native context together; `destroy(self)` consumes its owner; `Drop` calls the terminal teardown path and cannot retry after a failed native destroy. On failed destroy it deliberately retains dependencies, avoiding a potentially unsafe DLL unload. D005 records this rationale. `upscaler/dx12.rs:213–257` builds one context ownership path shared by both threading families.

**Assessment:** This is a sound *structural direction*, not a complete proof that arbitrary providers behave safely. Do **not** change `destroy(self)` to an endlessly retryable `&mut self` operation or add a safe `recover()` without native rollback evidence. A leaking exceptional path is unpleasant but can be safer than unloading dependencies that a failed native destroy still needs. No 0.2.0 signature break justified here.

### 2.2 Threading — separate local and movable families are deliberate

**Verified:** `Runtime` retains `Rc<RuntimeState>` and is local; `MovableRuntime` retains `Arc<RuntimeState>`, is `Send`/`!Sync`, and requires an extensive unsafe acquisition contract covering all callbacks, cleanup and sibling operations (`runtime/movable.rs`, D013). `MovableUpscaler` delegates to the same core (`upscaler/dx12/movable.rs`). D013 includes proof obligations for implicit `Drop`, which matters even when explicit calls are serialized.

**Assessment:** A single globally `Send` context behind a mutex would promise more than the wrapper can establish about native providers; a generic `Threading` type parameter exposed in every consumer signature would add noise and trait-bound complexity. The duplication of **public wrappers** is a maintainability cost, but it is currently backed by shared implementation. Keep distinct ownership contracts unless concrete code duplication becomes error-prone. Adding future effects doubles these outward wrappers; that is acceptable if common native code stays shared internally. No required 0.2.0 break.

### 2.3 `unsafe dispatch` is an honest boundary, not an ergonomic defect

**Verified:** `dispatch.rs:24–96` documents resource states, residency, heap aliasing, CPU vs GPU lifetimes, submission ordering, discarded recordings, subsequent Configure, and teardown. `frame.rs:18–65,189–235` intentionally borrows resources for the CPU call only, not until the GPU fence. `dispatch/resources.rs` inspects COM identity/metadata/device but cannot know the command list's actual state transitions, overlapping placed heaps, queue submission or GPU completion. D011 covers the partial native trust assumption.

**Assessment:** Turning `dispatch()` safe solely because it accepts `&ID3D12Resource` or `&mut Upscaler` would be unsound: Rust borrow lifetimes do not cover asynchronous GPU execution. A safe `RecordedFrame` token cannot prevent submitting a previously cloned DX12 list or certify that a caller waited on the correct fence. A *real* safe recording API would need exclusive control of command-list execution, resource ownership/storage and fence completion, i.e. an actual renderer/queue integration, not just phantom markers. Preserve `unsafe dispatch`; optional higher-level integrations may be separate products after evidence. **A mandatory 0.2.0-breaking recording-session replacement is not justified.**

## 3. Findings: problems, extensibility stress and recommended treatment

Severity labels: **P0** fix/decide before declaring 0.3 complete; **P1** high-value design decision before shipping new interfaces; **P2** defer absent measured pain. “Break?” distinguishes *published 0.2.0* from unreleased main.

### F1 — Native poison vs predictable application errors (P0, no necessary 0.2 break)

**Problem:** `context_policy.rs:7–50` sets `DispatchFailed` or `ConfigureFailed` on any non-OK native mutating call; future operations receive poisoned errors. D030 proposes applying the same model to mask generation. A missing native capability could thereby make an otherwise useful upscaler terminal. **This is a real ergonomic hazard, but not proof that a specific native failure is harmless.**

**Safe approach:** Validate every *knowable* contract before FFI **inside the canonical operation**, return precise nonpoisoning validation errors, and retain terminal state when the native provider may already have mutated/recorded work. Distinguish missing provider enumeration (pre-create), resource/flag/size errors (pre-native), and native failure (terminal unless proved otherwise). For D030 verify negative-path generator support when possible; a query indicating capability is not a promise of later success. Do not add `checked_dispatch` plus unchecked default or a `clear_poison()` bypass. For any proposal to treat a particular native return code as recoverable, require pinned provider-specific evidence that the *failed operation* did not mutate CPU/GPU state, plus repeated-use tests.

**Break:** No mandatory v0.2.0 source break. Error classification/poison rules can constitute *behavioral* changes, especially if changing the handling of a previously terminal native return; do not silently relax them. See earlier [recovery record](2026-10-18-src-error-recovery-and-preflight-ergonomics.md).

### F2 — Abandoned successful command recordings (P0 safety documentation, P2 alternative design)

**Problem:** D011 requires terminal cleanup if an already successful recording is abandoned. The wrapper cannot observe whether that command list was later submitted or reset. This is particularly inconvenient in a renderer that frequently cancels/rebuilds frames.

**Safe approach:** Clearly distinguish (a) error before FFI, (b) native error with unknown recording side effects, (c) successful recorded commands later abandoned, (d) successful submitted commands. Provide one explicit documented recovery procedure (invalidate abandoned/failed list, fence prior work, destroy, create fresh context); add an example demonstrating ordinary frame abort, not merely normal submission. Investigate whether the *specific provider* has reliable discard/rollback before offering reusable aborted contexts.

**Break:** Replacing the published `Upscaler::dispatch` signature with mandatory recording tokens is a **major source/contract break**; recommend **against it now**. A constructor returning `Result<RecordingGuard>` is not proof that previously submitted GPU work was abandoned. A type-state guard may be useful only with actual queue integration and explicit fallible terminal pathways.

### F3 — Context Configure after previous GPU work (P0, no 0.2 break)

**Problem:** `upscaler/dx12.rs:261–288` exposes **safe** `configure_tuning(&mut self, ...)`; previous **unsafe** dispatch carries lasting caller obligations to finish submitted GPU work and leave no executable unsubmitted list before Configure. `&mut self` cannot prove either. Native Configure may fail and poison the context. This is new in D029.

**Safe approach:** Keep one canonical `configure_tuning`; document and test *before first dispatch* as the default sequence. Allow multiple configuration calls in one renderer-controlled idle interval, explaining that there is no native atomic batch. Require caller established ordering as part of earlier unsafe dispatch contracts. Do not invent `GpuIdleToken` solely from an integer fence value, or silently wait on a queue the wrapper does not own. Evaluate an initialization-time tuning config only if Configure is guaranteed safe and correctly ordered at that phase for real providers; this would not be an automatic GPU synchronization solution.

**Break:** No published v0.2.0 API change, because Configure did not exist. Changing 0.3's API is permitted; no extra public abstractions are required yet. **Unresolved:** whether safety obligations of accepted native Configure are satisfiable for each provider/profile and whether an FSR context can safely handle specific Configure requests when pending work exists.

### F4 — Tuning accepts finite but out-of-domain values (P1, unreleased-only)

**Problem:** `upscaler/common.rs:7–39` exposes `TuningValue::Value(f32)`; `upscaler/tuning.rs:34–61` rejects NaN/Inf but forwards finite out-of-documentation-domain values unchanged, relying on AMD's stated native clamping. D029 selected that deliberately; it is not an accidental unchecked bug. `Ok(())` does **not** imply the provider applied the requested value.

**Choice, not an automatic finding:** For the stated “find programmer errors early” objective, prefer **one canonical strict policy** that rejects documented out-of-domain finite values before FFI, by *parameter* (`VelocityFactor`, etc.) and preserves exact valid bits. Keep `ProviderDefault` as a genuinely native reset. Reject negative zero only if a domain rule genuinely disallows it; do not invent unsupported precision/domain claims. Document that a documented range is not a universal provider-support guarantee. Alternatively deliberately retain pass-through for ABI fidelity; both approaches are coherent. Do not add parallel `configure_tuning_checked` and `configure_tuning_raw` for the same common use or silently clamp in Rust; D016 requires native-control fidelity where supported.

**Break:** Behavior/source changes only relative to **unreleased D029**, *not* v0.2.0. Strictness needs a **separate amendment to D029**, not a claim that its earlier reasoning was mistaken.

### F5 — Provider selection is inconsistent across setup paths (P1, unreleased-only)

**Problem:** `runtime.rs:141–289,391–420` has four default utility queries and four parallel `_for_provider_dx12` methods; `upscaler_resource_requirements_dx12(device, Option<ProviderId>)` uses a third form; memory estimates receive `UpscalerOptions`, and `UpscalerOptions::set_provider()` determines actual create override. These affordances risk a caller querying one provider and constructing another. Fresh provider enumeration (`runtime/queries.rs:16–31`) is nontransactional and not a reservation. D028 explicitly accepts this limitation.

**Coherent choice:** Before publishing D028, select **one canonical source of provider selection for each setup operation**. A modest uniform explicit `Option<ProviderId>` input across device-backed utilities is easier to reason about than eight parallel names, though default queries then require an explicit `None`. Another coherent possibility is a single *setup-scoped selection* used by queries and creation—but only if the scope demonstrably reduces errors and does not falsely promise stable device/provider identity. Do **not** expose a long-lived `ProviderQuerySession` as a capability certificate or performance optimization without a need/benchmark. Cross-provider comparisons must still intentionally select an ID for each request. Keep provider ID opaque; no `ProviderId::from_u64` default that invents cross-runtime validity.

**Break:** Any consolidation affects new D028 APIs only. `UpscalerOptions::new` and published context creation need not change. It is **not proven** that repeated enumeration materially costs performance or can be safely cached.

### F6 — No preflight obligation should leak into the canonical runtime contract (P0)

**Problem:** Resources are assembled without COM inspection (`dispatch/frame.rs:18–65,212–235`), and `dispatch/resources.rs` inspects them inside `dispatch`. Exposure mode, HDR colour hint and active region may reject during frame recording even when the renderer can discover configuration mistakes at startup. D030 adds a second GPU operation with different aliasing/role rules and a UAV output that must also be readable later.

**Safe approach:** Preserve mandatory validation *within each operation*; split pure cross-field validation from COM inspection and native mapping, share low-level role checks without sharing invalid upscaler-specific assumptions. Catch obviously inconsistent creation options at construction where reliable; reject stale/incorrect per-frame metadata at dispatch. For D030 permit identical read-only inputs **only** if the provider semantics allow it, forbid output aliasing, check both UAV and shader-read flags as the proposal requires, and verify state restoration on the pinned native backend. A preflight helper may exist for error reporting but must never be a prerequisite for safety. Existing role-specific errors should remain precise.

**Break:** Internal reordering is not a v0.2.0 source break; turning `UpscaleDispatch::new`/`UpscaleResources::new` into fallible or mandatory prepared types **would break v0.2.0**, and has no established safety benefit for actual GPU-state/lifetime invariants. Avoid it absent concrete failure evidence.

### F7 — Portable scalar types live under an effect-specific name (P1 extensibility, avoid forced break)

**Problem:** `Dimensions`, `CameraParameters`, `FrameTimeMillis`, and `JitterOffsetPixels` are `upscaler::common` in v0.2.0 (`upscaler.rs:15–32`). The pinned Frame Generation header also has render sizes, frame time, projection fields and jitter (though **not always identical contracts or use sites**). Users of future effects should not have to import a semantically generic dimension via an upscaler-only path.

**Coherent approach:** Introduce a genuine shared `types`/`frame` module **only when** a second effect has been implemented and exact invariants align; reexport the **same underlying type identity** from old `upscaler` paths, rather than defining duplicate nominal `Dimensions` types. If FG uses different validation domains, introduce effect-specific wrappers, not misleading universal types. No speculative `EffectFrame` trait or generic camera type now. D015 already separated common inputs from DX12; this is a naming/extensibility issue, not a present soundness hole.

**Break:** Moving a type with old-path `pub use` retained need not break v0.2.0. **Removing** old aliases would be a source break; no compelling need today.

### F8 — Backend-qualified vs legacy root names (P1 design consistency, no forced break)

**Problem:** `upscaler::Upscaler` and `upscaler::UpscaleDispatch` are aliases to DX12 implementations under Windows `dx12`; separately `upscaler::dx12` exposes them (`upscaler.rs:15–33`, D015). `runtime::Runtime` currently denotes explicit DX12 native loader ownership. Future Vulkan support cannot *simply* redefine these names to mean “whatever backend is enabled” without ambiguity, especially with both features enabled.

**Coherent approach:** Keep DX12-specific behavior under `upscaler::dx12` as an unambiguous canonical backend-qualified API and treat root DX12 reexports as **permanent aliases** or deprecate them deliberately only when a second backend exists. Future `upscaler::vulkan` should have its own device/resource/command-list and synchronization contract; do not unify opaque native object ownership with a macro-generated generic `Upscaler<B>` purely for aesthetics. For runtime organization, choose a backend-qualified module on the actual second backend, reusing private loader/storage primitives where valid.

**Break:** Removing 0.2.0 root reexports or renaming `runtime::Runtime` would be **real source breaks**; current evidence does not justify doing so. Decide not to rebind old root names to new semantics later.

### F9 — The deliberately failing `vulkan` feature is a real feature-design defect (P1, *potential* published manifest break)

**Verified:** Both published 0.2.0 manifests offer a public `vulkan` feature; the `fsr-sdk` feature activates `fsr-sdk-sys/vulkan`. Both current manifests retain it; `fsr-sdk-sys/src/lib.rs:46–51` has unconditional `compile_error!` if this feature is enabled. Consequently `cargo --all-features` on either crate necessarily activates the deliberately failing path (**deduction from manifests plus conditional compile; not an executed cargo test**). `SCOPE.md` explicitly says Vulkan is not implemented.

**Recommendation:** Consider **removing** the placeholder features from both crates *until there is a working backend* rather than representing a nonexistent capability as a selectable Cargo feature. Do not silently provide stub Rust Vulkan types as a replacement. Add a real `vulkan` feature with actual scope later. An alternative is to keep the explicit compile-time diagnostic because users who manually enable unsupported Vulkan receive a clearer message; weigh this against all-features CI and feature-unification ergonomics.

**Break:** Removing a published Cargo feature is **formally a 0.2.0 manifest/feature break**, even though this particular feature never built successfully. This is one **concrete candidate for an intentional breaking change**, unlike most earlier six. Check the project's intended all-features policy before accepting it. Cargo's feature system and SemVer rules support treating feature removals as breaking.

### F10 — Error types and provider introspection are largely extensible already (P0 audit, no redesign)

**Verified:** Public `Error`, `Operation`, `DispatchValidation`, `ResourceRole`, `ResourceProperty`, `InvariantViolation`, `QueryValidation` are `#[non_exhaustive]` (`error.rs:6–261`), as D009 envisages; native error codes remain `u32` and HRESULT remains `i32` bit-equivalent. `runtime::LoadError` is a separate owned type, and D010 avoids exposing sys load errors as the wrapper's surface. `query.rs:74–185` makes provider identity opaque, copies names immediately, preserves **unknown resource bits** and reports memory as observational rather than a reservation.

**Assessment:** Preserve these choices. Adding a new error variant is forward-compatible with wildcard-matching callers; avoid renaming existing variants or turning stable errors into strings. Improve operation-specific diagnostics rather than a second parallel error hierarchy. A small **unreleased cleanup** is that `DispatchValidation::UnsupportedResource` documents a prior mask policy that D025 already rejected (`error.rs:98–102`); remove it before 0.3 *if no public 0.2 usage exists*. Do not invent an external numeric `ProviderId` constructor as a solution to the scope problem.

**Break:** No necessary 0.2.0 change. Changes to newly introduced error variants may break Git/main users but not the published release. Raw `fsr-sdk-sys` error enums/type representations require separate compatibility checks.

### F11 — Numeric and color/profile contracts: early detection without false type proofs (P0/P1)

**Verified:** `upscaler/common.rs:45–67` makes dimensions nonzero; `:476–553,568–662` checks finite jitter/frame/camera/pre-exposure values; creation options have private fields and meaningful defaults (`:201–352`). `upscaler/dx12.rs:401–434` still delegates provider-specific numerical limits and some color compatibility to an unsafe-acquisition trust contract. `dispatch/frame.rs:290–320` allows an explicit color hint/context mismatch override (D027). The provider's actual interpretation of HDR/non-linear signals is not detectable from resource dimensions.

**Recommendation:** Enforce every **provable invariant** at its earliest canonical boundary; do not manufacture bounds from a single tested GPU/provider. Preserve explicit `unsafe` obligations where pixel content or native provider-specific numerical behavior is unknown. Do not introduce a combinatorial `Upscaler<Hdr, Depth, Exposure, Motion, Provider>` type-state matrix: flags and formats are not all known at compile time, and new providers could add combinations. If a runtime profile mismatch is detectable from options and dispatch before native work, return an ordinary `InvalidDispatch` or creation error rather than terminal poison. Reassess color-mismatch override naming as intentionally hazardous but honest; it is a new D027 API, so there is no 0.2 break if renamed for clarity.

**Break:** No fundamental v0.2.0 redesign required. Raising validation strictness for formerly accepted **but unsupported** configurations is potentially a *behavior change*; document specific cases and validate before claiming compatibility.

### F12 — Future effects have different asynchronous/ownership models (P1 architecture, P2 public changes)

**Verified upstream:** Pinned Frame Generation header includes context create flags, a dispatch with up to four output resources, `frameID`, UI/presentation callback descriptors, swapchain fields, user pointers, callback functions and async-workload selection. This is visibly not the same lifecycle as an upscaler's single synchronous CPU recording method; see pinned [`ffx_framegeneration.h`](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/framegeneration/include/ffx_framegeneration.h). `SCOPE.md` explicitly targets Frame Generation (including presentation), Ray Regeneration, Radiance Caching and future backends.

**Architecture consequence (Inference):** Keep **effect-specific** public owners (`Upscaler`, future `FrameGenerator`, etc.) and backend-specific GPU resource descriptors, while sharing **private** `NativeContextOwner`, runtime loading and error-code mapping only to the extent their native lifecycle rules match. Callback registration needs actual stable callback/user-state ownership, native unregistration/drain guarantees and unwind handling; it cannot simply re-use D011's “no callbacks” owner assumption. Swapchain replacement or async workers may need a genuinely distinct owner with different teardown/Send rules. A generic `Effect::dispatch` trait would either hide important obligations or force a large unsafe contract unrelated to many effects. Do not create public effect/back-end traits until two real implementations establish their common invariants.

**Break:** Existing DX12 Upscaler constructors/dispatch need not change to accommodate *distinct* future effect owners. A future shared Rust module for truly common scalars can be additive through reexports. This is the most important reason **not** to perform a speculative unified-API rewrite today.

### F13 — `fsr-sdk-sys` exposes a deliberately narrow ABI, not a universal upstream SDK (P1)

**Verified:** `fsr-sdk-sys/src/lib.rs:3–51` exposes `api`, `upscale`, and gated `loader`/`dx12`; `api.rs`, `upscale.rs` use native C naming, `repr(C)` structures/unions and raw pointers. The manifest's crate version is independent of the native SDK and of `fsr-sdk`. D001/D003/D006 explicitly choose curated ABI slices rather than claiming complete generated bindings. D002 pins v2.3.0.

**Recommendation:** Preserve raw naming/layout in `-sys`, independently test `size_of`, `align_of`, offsets, constants, pointer signatures for each new descriptor (including D030), and keep policy/high-level value checks in `fsr-sdk`. Do not expose `-sys` structs in the safe wrapper to achieve easy extensibility. When implementing a new SDK release, distinguish *new raw declarations* from incompatible changes to existing native layouts. Changing the `windows` crate version on exposed DX12 interfaces is a legitimate Rust source compatibility event requiring explicit decision; avoid speculative dependency upgrades.

**Break:** Full raw sys ABI/symbol compatibility against published `0.1.1` requires a separate mechanical diff, not established here. Adding new FFX descriptors to `0.1.2` is not itself a 0.2.0 wrapper break.

### F14 — Documentation consistency affects API usability (P0, no break)

**Verified mismatch:** `dispatch/frame.rs` still says supplied masks “must be listed as required or optional by the selected provider” in its public Rustdoc near the frame resource contract (around lines 158–172). D025 amendment and `upscaler/common.rs:440–466` say the opposite: required bit implies *must supply*; absence of both bits does **not** prohibit supplying a mask. Also `runtime.rs:122–129` still says the runtime “does not expose ... provider-selection APIs,” although `runtime.rs:141–206,391–420` now does. These comments are stale, not evidence that the implementation follows the old rule.

**Recommendation:** Treat public Rustdoc as part of the API contract and add a documentation/behavior consistency check before release. Avoid allowing accepted ADR amendments and public docs to diverge during fast iteration.

**Break:** Documentation-only corrections, none.

## 4. D030 in the full architecture

D030's **proposed** `unsafe generate_reactive_mask(&mut self, ...)` is appropriately a separate GPU operation, not a hidden side effect of `Upscaler::dispatch()`. Its outputs may feed a later Upscale frame; it needs distinct role validation and GPU sequencing. On native failure, use the same eligibility state until a provider-specific nonmutating failure contract is proved. Define FFI operations and error variants by actual operation, preserving the *recovery state* and raw codes. Do not add `validate_generate_reactive_mask` as a caller-required second API. Do not promise that generator support follows from upscaler provider identity alone. Verify on supported native hardware: output shader readability/UAV flags, aliasing of two readers, no output/input aliasing, resource post-state restoration, actual provider availability, failure poisoning and successful mask use in a following frame. D030 is additive relative to 0.2.0 and may freely be revised before acceptance/implementation.

## 5. Compatibility and decision matrix

“Must break?” is a design recommendation, **not** a statement about what SemVer permits in 0.3.0.

| Candidate | Breaks published `fsr-sdk 0.2.0`? | Value / evidence | Recommendation |
|---|---|---|---|
| Strengthen `load_unchecked` native Query/Configure/provider trust (already current) | **Safety contract YES**, signature NO | D018+ | Keep, clearly publish migration/safety note; ask if API should require documented trusted deployment |
| New creation-time required-resource rejection | **Behavior MAYBE** | native provider dependent | Keep conservative checks; enumerate affected scenarios in release notes |
| Replace DX12 `dispatch` with `RecordingSession` | **Source YES** | no full renderer/queue control | **Do not do** absent an end-to-end proof |
| Make frame/resource constructors fallible or mandatory prepared | **Source YES** | cannot establish GPU states/submit | **Do not do** merely for earlier errors |
| Allow native-error continuation / abandoned-frame reuse | **Safety behavior YES** | missing rollback proof | **Do not do** without pinned proof |
| Rename/remove `upscaler::Upscaler` DX12 reexport | **Source YES** | theoretical second backend naming issue | Preserve alias; leave second backend qualified |
| Rename/remove `runtime::Runtime`, merge local+movable ownership | **Source YES** | breaks deliberate D013 trust split | Do not do |
| Remove always-failing `vulkan` feature marker | **Manifest YES** (both `0.2.0` wrapper / `0.1.1` sys) | concrete all-features issue; feature never worked | **Consider intentional break now** if all-features cleanliness is a goal |
| Strict D029 per-parameter tuning rejection | **NO**; new feature | programmer-error diagnosis vs native pass-through | Choose one coherent policy in D029 amendment |
| Consolidate D028 provider query signatures | **NO**; new feature | removes selection ambiguity/duplication | Decide before 0.3; avoid parallel canonical routes |
| D030 generator as separate unsafe operation | **NO**; new feature | models different GPU operation | Accept after role/state/native checks |
| Improve internal preflight/poison explanations | No signature break | real diagnostic value | **Do** |
| Move shared scalar *definition* while reexporting old paths | **NO**, if same type identity/aliases | future shared types | Only on second effect implementation |
| Add Frame Generation, RR, RC owner modules | **NO** | effect-specific native lifetimes | Add incrementally with their own audits |
| Remove old `UnsupportedResource` unreleased enum case | **NO** (not present 0.2) | obsolete D025 policy | Remove before shipping if no downstream snapshot promise |
| Change exposed `windows` 0.62 interface types | **Potential source YES** | no need established | Do not casually upgrade |

**Net finding:** The entire-source review does **not** reveal a compelling reason to break v0.2.0's central ownership, frame-construction or dispatch **signatures**. It does uncover the already-strengthened runtime safety contract and a plausible *additional* intentional **Cargo-feature break** (remove the unusable Vulkan markers). Substantial design choices remain for **new/unreleased Query, Configure and D030 APIs**. “No recommended central signature break” is not an independent native soundness proof.

## 6. Preferred API policy for long-term coherence

1. **One authoritative operation per capability.** `dispatch` validates before its own native call; one `configure_tuning` policy; one provider-selection convention for setup queries. Do not add parallel “checked” and “raw but identical” methods in the safe wrapper merely to preserve development syntax. The separate `fsr-sdk-sys` crate is already the genuinely raw layer.
2. **Choose stable semantic boundaries, not a universal facade.** Backend-specific resources/recording; effect-specific owners; shared truly identical scalars and *private* machinery. Avoid public generic traits whose only common property is “AMD calls this a context.”
3. **Expose only guarantees the wrapper can actually uphold.** Borrowed DX12 resources do not imply GPU lifetime ownership; `Send` does not imply native synchronization; enumerated providers are not reserved; successful native calls do not imply GPU completion, actual tuning efficacy, or correct colors.
4. **Prefer early recoverable failure to unearned compile-time fiction.** Stateless input errors must be nonpoisoning and precise. Native state-uncertain errors can remain terminal. The unsafe trust boundary cannot disappear just by adding checked value types.
5. **Make extension-friendly types private-field and open where needed.** Existing D009/D012/D014 practice is good. Preserve unknown native flags/codes in result reporting; use exact mapping for known fields; add new effect/backend modules without changing existing semantic identities.
6. **A break needs a failure demonstration.** For any proposed change to an existing public signature, provide (i) an invalid or misleading call site under the old API, (ii) why internal/preflight correction cannot solve it, (iii) the replacement call site and new guarantee, (iv) Windows/native verification of the new path, (v) 0.2.0 migration guidance. Aesthetic uniformity alone is insufficient.

## 7. Suggested release-oriented work package, not accepted decisions

**Gate A — correctness before 0.3 (P0):** Resolve D030 ABI/role/state checks and provider failure expectations; fix public docs diverging from D025/D028; centralize pre-FFI reject-before-poison precedence; explicitly document `configure_tuning` idle obligations and `load_unchecked` widened native trust; audit every new API for preserving native error codes and deterministic ownership on failures. Expand test matrix to cover local/movable, default/selected provider, mask/auto exposure, poison transitions and teardown error paths.

**Gate B — deliberately settle unreleased API choices (P1):** Decide **once** whether D029 tuning is strictly validated or faithful finite native pass-through. Decide **once** the canonical D028 setup-query provider argument pattern. Decide whether placeholder `vulkan` features should be removed as a conscious 0.2.0 manifest-level incompatibility. Review public names for D030 and any remaining upscaler-surface inventory items before declaring 0.3 done. These should be documented in new or amended decisions only by the project's normal authorization process; research itself does not accept changes.

**Gate C — do not pre-design hypothetical layers (P2):** Do not replace `dispatch` with GPU tokens without owning the actual submission/fence boundary; do not introduce provider-session caches without measurements; do not genericize all effects/backends before two real implementations; do not start arbitrary sys dependency/platform upgrades. Revisit when implementing Frame Generation, Vulkan, RR or RC with pinned upstream requirements.

**Release mechanical checks required:** download the two *published crate artifacts* or tagged checkout; run API-diff tooling such as `cargo-semver-checks` for `fsr-sdk 0.2.0` and **separately** for `fsr-sdk-sys 0.1.1`; inspect `--all-features` versus supported-feature builds and feature unification; compile consumer examples importing all old public types/constructors on Windows x64/MSVC and host-only portable inputs; test MSRV 1.88 and ordinary supported matrices; execute bounded native provider/GPU tests for changed error/teardown/callback semantics. Verify `cargo-semver-checks` limitations around safety docs, `unsafe` preconditions, callbacks, and behavioral changes separately. Record observed commands, versions and results rather than assuming success.

## Baseline delta

- **Confirmed from published source:** the original API is already intentionally extensible through private options/frame fields, non-exhaustive errors, explicit thread-family separation, DX12-specific module and honest unsafe GPU recording. These are not temporary design mistakes that must be replaced simply because 0.3 is allowed to break.
- **Confirmed current behavior change:** `Runtime::load_unchecked` now carries additional native trust requirements for queries/Configure/selected providers (D018 onward), despite unchanged function signature. Creation now can fail after newly required provider/resource introspection.
- **New cross-repo finding compared to the two earlier records:** The `vulkan` placeholder Cargo feature is available in published manifests but always emits `compile_error!`, creating a concrete all-features/extensibility tension. This is a *choice* whether to take a manifest break, not evidence of unsafe code.
- **Broader extensibility finding:** the pinned Frame Generation headers show a substantially different callback/presentation/async owner model, favoring separate effect-specific public APIs over a generic all-effect `dispatch` facade. Common input type paths can be shared later by reexports without necessarily breaking 0.2.0.
- **New consistency evidence:** current public documentation retains superseded mask/provider-selection statements; correct before release, independent of signature changes.
- **Unchanged from prior records:** D029/D028/D030 are development-only; no case proven where breaking the published `UpscaleDispatch::new`, `UpscaleResources::new`, `Upscaler::dispatch` or `Upscaler::destroy` is necessary.

## Limits and open questions

1. **No toolchain, ABI checker or native execution in this environment.** This record does *not* prove complete source compatibility, safety, ABI/layout correctness, provider support or GPU behavior. A full raw API symbol+layout diff against the actual published `fsr-sdk-sys 0.1.1` crate is still required; a manual review of selected raw files cannot certify it.
2. **Published tag was inspected via pinned web source**, not locally downloaded and mechanically compiled; exact GitHub release source and published crates should be compared, especially target and feature combinations.
3. **Native provider contracts are bounded:** cannot infer that all providers restore states, that Configure/Query work with all in-flight modes, that a particular error is rollback-safe, or that async/callback teardown can use D005 unchanged. Tests on one driver/provider do not establish a universal guarantee.
4. **Backend-evolution check:** when Vulkan becomes real, verify upstream API/runtime support and design device/synchronization ownership; do not infer availability from a Cargo feature name. If the placeholder is removed, determine how to communicate that to anyone using the (already failing) marker.
5. **Future-effect check:** separately investigate pinned Frame Generation callback reentrancy, callback pointer ownership, Unwind/Drop, swapchain lifetime, native worker quiescence and presentation threading before generalizing `NativeContextOwner` to that effect. Equivalent investigation applies to Ray Regeneration/Radiance Caching.
6. **Data for performance proposals is missing:** measure repeated enumeration and per-frame inspection before adding caches/prepared-resource sessions; report throughput, overhead and stale-state behavior explicitly.
7. **Coherence approval is not automatic:** this is source research. Do not silently amend D029/D028/D030 or alter accepted D011/D013/D015 policy. Any chosen tradeoff goes through a reviewed decision and verified implementation.
