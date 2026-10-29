# Public API Consistency & Architecture Audit — fsr-sdk-rs

- Investigation date and evidence cut-off: **2026-10-26**.
- Kind: **source research**, including architectural reasoning.
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`; edition 2024;
  Rust 1.88; AMD FidelityFX SDK **v2.3.0**, upstream commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Question: is the complete public API a suitable long-term foundation, and
  which changes are justified before 0.3.0?
- Status: **findings and recommendations for maintainer review**. This record
  accepts no design, changes no API and selects no new release prerequisite.
- Method: inspect manifests, production definitions and reexports, shared
  implementation, consumer examples, selected tests, decisions and existing
  evidence; reason through rendering-engine workflows; consult identified
  primary Rust and pinned AMD sources. The worktree was clean at intake.
- Exclusions: no implementation, compilation experiment, ABI execution, DLL
  loading, GPU work, benchmark, package comparison or topic integration.
  Existing execution records are cited as historical evidence, not rerun.
- Input identity: [source inventory](2026-10-26-src-public-api-consistency-and-architecture-audit/sources.json)
  records paths, sizes and SHA-256 identities of the inspected baseline inputs.
  Findings below describe the relevant behaviour without requiring Git history.

## 1. Architectural verdict

**The current ownership and native-operation architecture is a suitable
foundation. The public configuration surface should receive a small, deliberate
consolidation before 0.3.0. A wholesale replacement is not justified.**

The design is more coherent than its feature-by-feature presentation suggests.
Native ownership, checked inputs, metadata inspection, descriptor translation
and terminal failure handling already have distinct responsibilities. Both
ownership families use the same implementation of consequential behaviour.
The wrapper does not pretend to own a renderer or turn a successful native
return into proof of GPU completion or feature application. These are valuable
foundational choices, not temporary limitations to remove for symmetry.
See [context ownership][owner], [eligibility][eligibility], [DX12 core][dx12],
[recording][dispatch] and [interop][interop].

The principal inconsistencies are at the boundary where applications express
intent:

1. **Provider selection has three public input conventions.** Four setup
   operations have separate default/explicit methods, resource requirements take
   `Option<ProviderId>`, and memory estimation takes options containing that
   option. The real provider-choice example consequently branches repeatedly.
2. **Motion scaling stores two competing intentions.** A unit selection and a
   native override coexist. A successful units setter can change only a dormant
   fallback. This is deliberately documented, but a single mutually exclusive
   choice is a better public model.
3. **One ordinary checked control remains backend-gated.** `NativeSharpness`
   and `Sharpening` contain no GPU objects, yet portable configuration code cannot
   use them alongside the other common values.

These are API consistency and composition problems, not newly demonstrated
memory-safety defects. They merit changes because they affect current consumer
workflows and would otherwise become patterns repeated across backends.
Sections 4–6 compare alternatives and describe exact replacements.

The genuinely difficult structural limitations are different: native global
state is wider than any Rust owner; callback targets can remain reachable after
context destruction; a successful recording cannot be rolled back by discarding
its Rust input; and the wrapper cannot certify closed-provider access domains.
No builder, universal trait or RAII callback guard fixes those facts.
Sections 7–8 explain where the current explicit unsafe model is defensible and
where further evidence is necessary.

### Recommended pre-0.3.0 package

| Priority | Recommendation | Meaning of priority |
|---|---|---|
| Essential design change | R1: one optional-provider argument convention for the four pre-create quality/size/jitter operations, in both families | Remove current branching and competing canonical routes before those new APIs ship |
| Essential design change | R2: one `MotionVectorScale` selection, replacing units-plus-override storage | Remove a current precedence trap; requires reviewed amendment of D034 |
| Essential boundary correction | R3: expose sharpening selections and their checked scalar through `upscaler::common` on every host | Complete the existing common/backend split, preserving old type aliases |
| Essential release clarity | R4: correct current API documentation contradictions and remove the obsolete, unreleased `UnsupportedResource` rejection variant | Make the shipped contract match implementation; preserve historical records |
| Desirable, additive | R5: portable `UpscaleFrameParameters` and an assembly constructor; R6: pure tuning validation; R7: small diagnostics/value ergonomics | Useful improvements, but no reason to delay release solely for them |
| Defer | Generic effects/backends, safe callback framework, automatic synchronization, provider cache, universal policy interfaces | Missing concrete contracts or evidence |

“Essential” here is this audit's recommendation, **not an already accepted
release gate**. Retaining the existing API is operationally viable if the
maintainer rejects the ergonomic trade-offs; it does not create a proven
soundness defect. The audit recommends taking the inexpensive foundational
improvements now, while accepting that most of the architecture should remain.

## 2. Evidence and assessment criteria

**Verified source fact** means directly supported by the identified source.
**Historical observation** means a result reported by an existing local record,
with its original scope. **Inference** and **recommendation** identify this
audit's architectural judgement. None means new native verification.

The primary implementation baseline is the [workspace][workspace] and two
[crate][wrapper-manifest] [manifests][sys-manifest], public source listed in
section 3, and the consumer workflows cited throughout. The lockfile agrees
with wrapper 0.3.0, sys 0.1.2, `windows 0.62.2` and `libloading 0.9.0`.
The published versions described by repository release documentation are
wrapper 0.2.0 / sys 0.1.1; the current root README intentionally documents that
published release. This audit does not independently compare registry archives.

The requested decisions were considered explicitly:

| Decision | Audit treatment |
|---|---|
| [D014][d014] | Keep private resource/frame fields, explicit required inputs, direct sharpening semantics and unsafe recording. Reconsider backend placement of sharpening and optional frame-data assembly; accepted details are context, not proof of optimality |
| [D016][d016] | Retain independent direct controls and distinguish project policy from provider requests. Do not force `Native`/`Preset` variants onto unrelated domains |
| [D031][d031] | Keep platform/backend dependency isolation. The deliberately failing Vulkan feature is an accepted compatibility choice, not Vulkan support |
| [D033][d033] | Retain core/utility/renderer responsibility boundaries. Test actual composition rather than assuming a generic policy interface is necessary |
| [D036][d036] | The provider-name resolver is a concrete, portable composition example; its application reveals friction in the core query API |
| [D040][d040] | Keep explicit unsafe callback installation and global diagnostics. Global registration and failure retention materially constrain alternatives |
| [D041][d041] | Assess safety, rendering meaning and unresolved native assumptions separately; the recent Rustdoc migration is part of the inspected baseline |

Additional relevant context is [D013][d013], [D015][d015] and [D034][d034],
the [prior full-surface review][prior-audit], the
[core/utility synthesis][core-utils] and [runtime threading synthesis][threading].
The earlier review already identified provider-signature inconsistency; current
examples demonstrate that it persists. This report does not present it as a
new discovery.

External primary sources, accessed **2026-10-26**:

- Pinned AMD [`ffx_api.h`][amd-api] and [`ffx_upscale.h`][amd-upscale],
  at the commit above. They corroborate distinct native creation, query,
  Configure and GPU-recording operations, provider overrides and frame controls.
  They do not provide a general closed-provider safety or callback-shutdown proof.
- Rust API Guidelines on [type distinctions][rust-types],
  [predictable receivers][rust-predictability], [extensibility][rust-future] and
  [documented failure/safety contracts][rust-docs]. These inform judgement, not
  mandatory uniformity. A custom enum helps when it removes conflicting intent;
  a builder or trait is not intrinsically more idiomatic.
- The Rust Reference's [unsafe obligations][rust-unsafe] supports distinguishing
  the caller's extra proof obligations from what a checked value actually proves.
- [ash 0.38.0+1.3.281][ash] separates loader-level, instance and device operations.
  This is a useful precedent for explicit native scopes, not evidence that FSR
  should copy ash's ownership, public generics or unsafe surface.

No new dependency, licence model, binary redistribution or SDK migration is
proposed. Existing project MPL-2.0 and retained AMD notices remain distinct.

## 3. Actual public API and conceptual model

### 3.1 Crates, modules and feature boundaries

| Surface | Actual responsibility and availability |
|---|---|
| `fsr-sdk-sys::{api, upscale}` | Portable selected raw ABI declarations: headers/tags, integer constants, opaque context, resource metadata, create/dispatch/query/Configure payloads, function and callback aliases. Some message-bearing declarations additionally require Windows because their ABI uses Windows `wchar_t` |
| `fsr-sdk-sys::{dx12, loader}` | Windows + `dx12`; pointer-only DX12 callback metadata and backend descriptors; `FfxLibrary` / raw `LoadError` |
| `fsr_sdk::error` | Portable owned operation, validation, inspection and native-invariant failures |
| `fsr_sdk::query` | Portable provider metadata, input-resource masks, memory values, quality modes and jitter counts; the native implementation is private |
| `fsr_sdk::upscaler::common` | Portable checked creation/frame values and selections |
| `fsr_sdk::upscaler::dx12` | Windows + `dx12` owners, borrowed GPU inputs, COM inspection, recording, callback integration |
| `fsr_sdk::upscaler` reexports | Same common types and legacy DX12 aliases; these are not another implementation or a backend-selected facade |
| `fsr_sdk::runtime` | Windows + `dx12` explicit trusted loader acquisition, pre-create upscaler queries and global diagnostics; optional movable sibling |
| `fsr_sdk::utils::provider` | Off-by-default `utils` feature; portable project-defined name recognition returning the existing optional provider ID |

The wrapper defaults to `dx12`; sys defaults to no backend. `movable` changes
available types, not existing types' auto traits. `utils` adds no backend or
dependency. `vulkan` deliberately produces a compile error, including with
`--all-features`. This is an exception to ordinary additive feature expectations,
but it is explicit and recently reconsidered in D031. Do not use all-features
success as the build criterion. Sources: [wrapper root][lib],
[upscaler exports][upscaler], [sys root][sys-lib] and manifests.

### 3.2 Complete conceptual inventory

The following groups account for the production public API, including the
ordinary and advanced routes; private test hooks are excluded.

| Group | Public types and operations |
|---|---|
| Common creation | `UpscalerOptions` with required render/upscale maxima; setters/getters for provider, depth convention, motion resolution/jitter inclusion, exposure, dynamic resolution, HDR, input classification, checking and visualization |
| Common selections | `DepthConvention`, `MotionVectorResolution`, `MotionVectorJitter`, `MotionVectorUnits`, `ExposureMode`, `InputColorSpace`, `ColorEncodingHint`, `DebugLevel` |
| Checked values | `Dimensions`, `JitterOffsetPixels`, `FrameTimeMillis`, `CameraParameters` including infinite projection, `PreExposure`, `NativeMotionVectorScale`; separately DX12-gated `NativeSharpness` |
| Direct operation configuration | `ReactiveMaskOptions` with three finite scalars/four independent flags; `TuningParameter` and `TuningValue`; separately DX12-gated `Sharpening` |
| Query values | `ProviderId`, `ProviderInfo`, `ResourceFlags`, `ResourceRequirements`, `GpuMemoryUsage`, `QualityMode`, `JitterPhaseCount` |
| Loader owners | `Runtime` and optional `MovableRuntime`: unsafe acquisition; provider enumeration; memory estimate; default and explicit ratio/render-size/phase/offset queries; optional-provider requirements; unsafe modern/legacy global diagnostic requests |
| Effect owners | `Upscaler` and optional `MovableUpscaler`: ordinary/callback creation; seven live queries; tuning; metadata preflight and unsafe recording for upscaling and reactive generation; unsafe live legacy diagnostics; consuming destruction |
| DX12 inputs | `UpscaleResources<'a>`, `UpscaleDispatch<'a>`, `GenerateReactiveMaskDispatch<'a>`; exact `windows` device/list/resource types and a selected barrier vocabulary are reexported |
| Callback assembly | `HostAllocationCallbacks`, `ResourceAllocationCallbacks`, `HeapAllocationCallbacks`, `Dx12AllocationCallbacks`, `CreationCallbacks`; eight non-null callback aliases; selected raw `interop::abi` payload/type/constant reexports |
| Core errors | `Error`, `Operation`, `DispatchScalar`, `DispatchValidation`, `ResourceRole`, `ResourceProperty`, `InspectionObject`, `InspectionOperation`, `InvariantViolation`, `QueryValidation`; separate owned `runtime::LoadError` |
| Optional utility | `FsrChoice`, `SelectionError`, `select_provider` |
| Raw loading | `FfxLibrary::load` plus unsafe forwarding of the five native exports; no public extracted function table or high-level context handle |

Definitions and their member locations: [common inputs][common],
[query values][query], [runtime][runtime], [movable runtime][movable-runtime],
[DX12 owner][dx12], [movable owner][movable], [frame/resources][frame],
[generator][generator], [sharpening][sharpening], [interop][interop],
[errors][errors], [resolver][resolver], [raw loader][loader],
[raw API][raw-api], [raw upscaling][raw-upscale] and [raw DX12][raw-dx12].

### 3.3 The intended lifecycle

The actual model is:

1. Establish lasting native trust and coordination at unsafe acquisition.
2. Optionally enumerate/select a provider and perform planning queries.
3. Create an owner from a checked immutable creation profile and DX12 device.
   The owner retains a runtime reference, independent device reference and pinned
   creation storage, then queries requirements and checks explicit live identity.
4. Assemble borrowed resources plus frame values. Construction does not inspect
   GPU objects, record work or establish GPU safety.
5. Optionally inspect metadata early. The recording operation always repeats the
   mandatory inspection before FFX entry.
6. Record through an unsafe operation; the renderer submits, orders and retains
   everything through completion.
7. Query live results as permitted; tune only at the required idle interval;
   destroy once after safe cleanup is possible.

This is **checked native integration**, not a complete safe rendering subsystem.
The library's value is ownership, faithful mapping and useful rejection of
inspectable errors within explicit caller-established native invariants.

## 4. R1 — Consolidate provider selection at setup

**Classification:** API consistency/ergonomics; essential recommended
pre-release change. **Confidence:** high source confidence, moderate design
confidence. No native algorithm change is needed.

**Verified:** [runtime.rs:191–335][runtime] and its
[movable counterpart][movable-runtime] expose four pairs of default/explicit
utility methods. Requirements take `Option<ProviderId>`; estimates inspect
`options.provider()`. [provider_choice.rs:32–97][provider-example] branches
separately for quality queries and jitter planning. The utility's return type
already matches core options, but applying it consistently is unnecessarily
verbose.

Current consumer shape, with an already acquired runtime/device/options:

```rust
let selected = select_provider(&providers, choice)?;
options.set_provider(selected);
let render = match selected {
    Some(id) => runtime.upscaler_render_size_for_quality_for_provider_dx12(
        device, id, display, mode,
    )?,
    None => runtime.upscaler_render_size_for_quality_dx12(device, display, mode)?,
};
let requirements = runtime.upscaler_resource_requirements_dx12(device, selected)?;
let memory = runtime.estimate_upscaler_memory_dx12(device, &options)?;
```

The problem is not merely long names. The call that omits a provider can
silently ask a different provider from the one requested by creation options.
Internal refactoring cannot make a default method infer an option it never
receives. Documentation explains the distinction repeatedly but cannot remove
this representational opportunity for mistakes.

**Recommendation:** use one canonical `Option<ProviderId>` argument for each
pre-create utility, matching requirements. Preserve `None` as “SDK default at
this invocation”, and `Some(id)` as “freshly check and explicitly request this
ID”. Keep estimates taking the complete options: their flags and maxima are
genuine estimate inputs. Do not add a redundant separate estimate selector.

Proposed signatures for **both** runtime families:

```rust
fn upscaler_ratio_dx12(
    &mut self, device: &ID3D12Device,
    provider: Option<ProviderId>, mode: QualityMode,
) -> Result<f32, Error>;
fn upscaler_render_size_for_quality_dx12(
    &mut self, device: &ID3D12Device,
    provider: Option<ProviderId>, display: Dimensions, mode: QualityMode,
) -> Result<Dimensions, Error>;
fn upscaler_jitter_phase_count_dx12(
    &mut self, device: &ID3D12Device,
    provider: Option<ProviderId>, render: Dimensions, display: Dimensions,
) -> Result<JitterPhaseCount, Error>;
fn upscaler_jitter_offset_dx12(
    &mut self, device: &ID3D12Device,
    provider: Option<ProviderId>, index: i32, phases: JitterPhaseCount,
) -> Result<JitterOffsetPixels, Error>;
```

Resulting workflow:

```rust
options.set_provider(select_provider(&providers, choice)?);
let selected = options.provider();
let render = runtime.upscaler_render_size_for_quality_dx12(
    device, selected, display, mode,
)?;
let requirements = runtime.upscaler_resource_requirements_dx12(device, selected)?;
let memory = runtime.estimate_upscaler_memory_dx12(device, &options)?;
let context = Upscaler::new(runtime, device, options)?;
```

All proposed API examples in this report are **illustrative source sketches**,
not implemented or compiled APIs. Existing-call examples are source-derived
excerpts with surrounding renderer inputs omitted.

| Alternative | Assessment |
|---|---|
| Retain paired methods | Valid but repeats branching and encourages a pattern that grows with effects/backends |
| Uniform optional argument | Preferred: removes four methods per runtime family and uses an existing core representation |
| `ProviderSelection::{Default, Explicit(id)}` | Clearer at isolated call sites, but adds a type and conversion boundary for little benefit over the existing `Option`. Reconsider only if selection gains real additional modes |
| Borrowed `runtime.upscaler_dx12(device, selected)` operation scope | Could bind repeated arguments and shorten names; useful if measured consumer complexity grows. It is not required for this fix and cannot reserve a provider |
| Owning backend session/factory that creates contexts too | More substantial redesign, borrowing/ownership decisions and another public lifecycle. Current evidence does not justify it |
| Cache enumeration or assume a selected scope proves capability | Reject as part of this change: alters an independent trust/freshness contract |

The optional argument does **not** enforce that all calls use the same ID.
It gives them the same vocabulary and makes reuse direct. `None` still need not
identify the same provider across calls. An explicit ID remains opaque, can go
stale or be reused, and is not a reservation or safety certificate.
[Fresh routing][runtime-queries] and [native membership checks][native-query]
must remain.

**Compatibility:** the affected methods are unreleased 0.3 additions. Replace
the explicit-provider names and add `None`/`Some(id)` to old default calls;
published 0.2 construction need not change. Amend D028 only after review.
Preserve input-validation-before-enumeration ordering, exact native errors,
live identity checks and the current coordination interval.

## 5. R2 — Express motion scaling as one choice

**Classification:** API consistency and state representation; essential
recommended change. **Confidence:** high source confidence, moderate design
confidence. This deliberately challenges an accepted D034 choice.

**Verified:** [frame.rs:435–469][frame] retains `MotionVectorUnits` alongside an
optional native override. [descriptor.rs:68–77][descriptor] ignores units while
the override is present. D034 explicitly requires that behaviour. By comparison,
the ordinary colour-hint setter clears its mismatch override.

An engine composing independent setting functions can currently write:

```rust
frame.set_motion_vector_scale_override(Some(custom_scale));
// A later component intends to select ordinary pixel motion.
frame.set_motion_vector_units(MotionVectorUnits::Pixels);
// custom_scale still determines the descriptor.
```

This is documented and memory-safe assembly, not an implementation bug.
It nevertheless stores a precedence rule in every consumer's mental model.
A utility that returns “pixel motion” cannot fully express its choice through
the units setter alone; it must also know to clear the other setting.

**Recommendation:** introduce a portable non-exhaustive selection:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum MotionVectorScale {
    Uv,
    Pixels,
    Native(NativeMotionVectorScale),
}
// Default is Uv.
// Frame getter: motion_vector_scale() -> MotionVectorScale
// Frame setter: set_motion_vector_scale(MotionVectorScale)
```

Then the current intentions have exact independent spellings:

```rust
frame.set_motion_vector_scale(MotionVectorScale::Native(custom_scale));
frame.set_motion_vector_scale(MotionVectorScale::Pixels);
```

`Uv` derives scale from active render/display-motion dimensions; `Pixels`
maps to `(1, 1)`; `Native` preserves literal accepted bits, independent of active
size. Context motion resolution and jitter inclusion remain separate axes.
The enum describes a descriptor scaling convention; it does not convert pixels,
choose resource resolution or prove provider support.

| Alternative | Assessment |
|---|---|
| Retain fallback plus override | Defensible when temporarily overriding and restoring a remembered fallback is a first-class requirement. No such consumer requirement is demonstrated in current examples |
| Make `set_motion_vector_units` silently clear the override | Simpler state, but changes the meaning of an existing setter and retains two public representations of one choice |
| Use one enum | Preferred: makes effective intent directly inspectable and avoids setter-order knowledge |
| Add an “effective units” getter only | Diagnoses precedence but leaves conflicting inputs and ambiguous utility application |

**Trade-off:** clearing the old override restored a saved UV/pixel choice.
With one enum, callers that need temporary overrides must save/restore the
previous selection themselves. That is explicit local application state, and
the cost is justified by simpler ordinary composition.

**Migration:** `(None, Uv)` becomes `Uv`; `(None, Pixels)` becomes `Pixels`;
`(Some(s), _)` becomes `Native(s)`. For published `set_motion_vector_units`,
either take the permitted 0.3 source break or retain a clearly documented
compatibility forwarding method that selects `Uv`/`Pixels`. Do not silently
preserve two conflicting canonical stores. Keeping an old method whose effect
changes when an override was set is a behavioural change, not “compatibility”.
The native override route itself is new in 0.3.

This requires maintainer review/amendment of **D034** and a migration note.
It changes no native fields or numerical domains. Verify that UV/pixel geometry
and native signed/zero/subnormal bits map exactly as before, through both owners.

## 6. R3/R5 — Finish the separation of frame data and GPU inputs

### 6.1 R3: sharpening belongs with common checked controls

**Classification:** current module-boundary inconsistency; essential recommended
correction. **Verified:** [sharpening.rs:3–60][sharpening] uses an error and an
existing portable scalar validator; neither public type contains DX12 objects.
Both definitions are reachable only through the gated DX12 dispatch module.
`NativeMotionVectorScale`, `ReactiveMaskOptions` and the other direct controls
are portable in [common.rs][common].

A configuration tool or backend-independent engine settings module can construct
pre-exposure, camera and native motion scale without DX12, but cannot construct
the corresponding sharpening choice. D033's illustrative portable plan is
therefore more awkward for this one setting than for the others.

Move the definitions to common code and export the **same types** from
`upscaler::common` and `upscaler` on every host. Retain
`upscaler::dx12::{NativeSharpness, Sharpening}` as same-identity aliases.
Do not change sharpening domains, defaults, borrowed getter, traits or enabled
zero semantics. This is additive availability and a definition move, not a new
quality policy or Vulkan capability. It needs no public trait or extra crate.

### 6.2 R5: a portable frame-parameter carrier is useful, but can be additive

**Classification:** composition/ergonomics, with a future-backend benefit;
desirable rather than an essential release condition.

**Verified:** [`UpscaleDispatch` fields and constructor][frame] combine two GPU
inputs (list and resource bundle) with required jitter/time/camera/reset and
optional sharpening, exposure, motion, geometry, colour and debug controls.
Each new frame restores all optional defaults. The bundle has no parameter-only
view; required values have getters but no setters.

For a rendering engine, a realistic flow is “prepare the next frame's numerical
settings in common code, then attach current GPU resources on the render thread”.
Today it needs an application-specific carrier plus a repeated setter sequence,
or a function tied to `UpscaleDispatch<'_>` and Windows/DX12. A stateful strategy
returning several checked settings is already possible, but consumers invent
the grouping and application code themselves.

**Preferred optional addition:** one effect-specific
`upscaler::common::UpscaleFrameParameters` with private fields:

```rust
let mut parameters = UpscaleFrameParameters::new(jitter, time, camera, reset);
parameters.set_pre_exposure(pre_exposure);
parameters.set_sharpening(sharpening);
parameters.set_motion_vector_scale(MotionVectorScale::Pixels);
parameters.set_active_render_size(Some(active_render));
parameters.set_active_output_size(Some(active_output));
let frame = UpscaleDispatch::from_parameters(list, resources, parameters);
unsafe { context.dispatch(frame)?; }
```

Keep all current defaults and required renderer-selected values. No fabricated
`Default` camera, time, reset or jitter is justified. The carrier may be
`Debug + Clone` and portable; do not promise `Copy` for an evolving sharpening
selection. Context-dependent validation still occurs in canonical recording and
optional preflight. It is input data, not a validated context binding.

The existing six-argument `UpscaleDispatch::new` can delegate to this carrier,
so a break is unnecessary. Existing frame accessors/setters can delegate too,
with one underlying state. New examples would prefer parameter-first assembly.
Do not maintain two stores or add a generic `Frame<B, E>`.

**Alternatives and limits:** retaining the present frame is reasonable; the
concrete D036 utility only selects a provider, and no second backend currently
consumes frame parameters. A carrier earns its place by simplifying a real
multi-setting engine path, not by the mere prospect of Vulkan. If the maintainer
does not want the addition now, R3 and R2 solve the current portable-value and
precedence problems independently, and the carrier remains additive later.

The carrier would not contain generation's different GPU inputs, creation
options or tuning requests. A general “apply all settings” interface would erase
important phase boundaries.

## 7. Findings on ownership, safety and state

### 7.1 Keep Runtime/Upscaler and the separate movable family

**Classification:** deliberate, defensible architectural separation.
**Verified:** `RuntimeState` owns only [the loaded library][runtime-state].
Each [native context owner][owner] retains state, device and runtime in that
dependency order. `UpscalerCore<R>` contains options, a requirements snapshot
and eligibility. Local uses `Rc`; movable uses `Arc` plus a concrete audited
`Send` implementation and an explicit `!Sync` marker.

The runtime and effect context are different native lifetimes. Combining them
would either duplicate loader ownership per effect, force unnecessary device
binding into the loader, or hide lifetime sharing. The separate types also permit
several contexts to survive dropping the public runtime.

Public local/movable methods repeat, but [creation and queries][dx12],
[pre-create routing][runtime-queries], [recording][dispatch],
[tuning][tuning] and [eligibility][eligibility] share implementation.
The duplication is primarily the public contract and thin forwarding.
It has maintenance cost, including documentation drift, but does not imply
duplicated native state machines.

| Alternative | Assessment |
|---|---|
| Existing concrete local/movable types | Preferred: visible trust choice, straightforward signatures and auto-trait audit |
| `Runtime<Local>` / `Runtime<Movable>` and matching generic owners | Can encode the distinction, but exports ownership-mode machinery and complicates inferred/default types. It does not remove conditional native safety or operation documentation |
| One always-`Send` runtime/context | Reject: enabling a feature must not strengthen existing ownership guarantees or bypass the separate unsafe compatibility contract |
| Runtime enum deciding local/movable dynamically | The enum's auto traits must accommodate its least-transferable variant; dynamic choice cannot create conditional `Send` |
| Public common owner trait | Useful only for an actual downstream generic algorithm. Current shared implementation needs no public trait |
| `Deref` to a common operation target | Adds a public dereference relationship, method-resolution and mutability obligations merely to hide forwarding |

Do not replace the four public owners for 0.3. Private helpers or narrowly scoped
forwarding generation can reduce maintenance if needed, provided contracts stay
readable and concrete field audits remain. There is no demonstrated need to
introduce macros as part of this audit.

### 7.2 The safety model is coherent but conditional and expensive to integrate

**Classification:** genuine native architectural limitation; no newly proven
soundness defect. The [D041 migration][d041] correctly separates signal meaning
from memory/native access obligations.

| Boundary | Wrapper establishes | Caller/native assumptions that remain |
|---|---|---|
| Unsafe acquisition | Path resolution, five exports, owned loading error, library retention | Trusted ABI/code, initialization/termination, provider-safe query/Configure processing, non-retention, common-domain coordination |
| Safe ordinary creation | Checked option representation, owned device, pinned chain, owner establishment, requirements/live-ID checks | Acquisition invariants remain valid for the actual provider/profile; unsupported inputs can still cause native termination |
| Safe metadata preflight | Observable shape/view/device/identity and cross-field rules; no FFX call | No claim about open-list state, pixels, backing aliasing, real states, residency, native capability or GPU completion |
| Unsafe recording | Mandatory repeated inspection and exact descriptor assembly | GPU lifetimes/states/order, access footprints, safe provider domain, submission once, completion before cleanup/Configure |
| Safe live queries/tuning/destroy/Drop | Eligibility and owned lifetime mechanics; native errors preserved | Earlier lasting unsafe invariants, completed work where required, compatible cleanup and native continuation |
| Unsafe callback/global diagnostic installation | Stable wrapper-owned tables/chains and exact forwarding | External target lifetime, no unwind/reentry, allocator compatibility, global registration coordination and hidden native activity |

Safe tuning is not inherently inconsistent with unsafe dispatch: earlier
recording establishes lasting completion/no-pending-recording duties before a
later Configure. Likewise, safe destruction relies on those duties rather than
discovering fences itself. Making just `destroy` unsafe would not fix implicit
Drop. Making every query unsafe would repeat unchanged acquisition assumptions
at many call sites without providing stronger evidence.

Nevertheless, [`load_unchecked`][runtime] now carries a large effect-specific
trust contract, and [movable acquisition][movable-runtime] requires compatibility
with **any** thread where safe `Send` code may use or drop owners. A working
handoff between two chosen threads is insufficient. This is a real learning and
integration cost. It should be described as such, rather than advertised as
ordinary safe access after trusting a DLL file.

**Future design direction, not a 0.3 change:** when adding another effect,
separate loader/ABI acquisition from that effect's unsafe integration assumptions
if a concrete effect-specific acquisition boundary can express them better.
Do not simply append every future effect's assumptions to one universal load
contract. Moving obligations to a new method is useful only if it creates a
reviewable scope; it does not establish the missing guarantees.

### 7.3 Global state prevents owner-local synchronization guarantees

**Verified:** no wrapper lock or process registry exists. Mutable receivers
serialize only a particular Rust borrow. Multiple contexts and acquisitions can
share native state. The [threading synthesis][threading] and
[callback verification][callback-record] record shared replacement effects.

A renderer must coordinate construction, calls, error paths, implicit Drop and
final library release across all affected users. The
[movable integration example][movable-example] shows why wrapping only dispatch
in a mutex is insufficient. Local `!Send` does not prevent native workers,
reentry or other SDK users.

**Recommendation:** preserve the explicit common-domain contract. An injected
coordination object might reduce application boilerplate, but a private mutex
per runtime cannot govern separately acquired runtimes, external native users
or hidden workers. A global mutex can deadlock on callbacks/reentry and cannot
invent native affinity or shutdown guarantees. Do not present either as a safe
concurrency solution without a separate design.

### 7.4 Terminal failure policy is systematic; abandonment is an external state

**Verified:** [context_policy.rs][eligibility] has `Usable`, `DispatchFailed` and
`ConfigureFailed`. Both GPU recording routes use the same terminal policy.
Native query errors do not change it. Poison is checked before later validation
or native entry. The first returned native error carries its operation/code;
later attempts receive a poison category, so applications should retain the
original error if needed.

| Event | Resulting contract |
|---|---|
| Invalid scalar/options/metadata before native recording | Correct input and retry a usable context; no new FFX recording in this call |
| Ordinary query error | Preserve code; leave existing eligibility unchanged |
| Native upscaling or generation error | Terminal recording/query/Configure eligibility; failed list must never execute |
| Native context Configure error, including live legacy diagnostics | Terminal eligibility; no rollback of prior acknowledged settings |
| Runtime-global Configure error | No owned context is automatically poisoned; unsafe caller must establish safe shared continuation |
| Native recording OK, later list abandonment/Close failure | Caller must stop use and recreate affected contexts; wrapper cannot observe abandonment |
| Destroy error | One attempt only; retain dependencies, do not retry |

This distinction is deliberate and coherent. It is also a material limitation for
engines that discard recorded frames frequently: native temporal/allocator state
can advance during recording even if GPU work never executes. There is no
wrapper state representing externally known abandonment.

Keep the [documented abort/recreate workflow][recovery]. An optional future
`invalidate`/`mark_unusable` operation could prevent accidental reuse after a
renderer reports abandonment, but would not prove list invalidation or GPU
completion. It is not essential now. A recording token cannot detect actual
submission or make a fallible wait happen safely in Drop. Reusable abandonment
needs provider-specific rollback/recovery evidence, not a new Rust name.

### 7.5 Callback assembly is a justified unsafe island

**Classification:** defensible exception to ordinary safe-type isolation.
**Verified:** [interop][interop] exposes exact raw callback payloads only under
an explicit module. Pure constructors assemble non-null allocation/free pairs;
installation occurs through unsafe construction or diagnostic methods.
`CreationCallbacks` is neither a lifetime owner nor a registration guard.

The [D040 verification record][callback-record] reports calls through later
registrations affecting surviving contexts, provider-specific message routing,
and bounded callback/cleanup execution. General quiescence and affinity are not
established, and SDK heap-callback invocation was not observed.

The conservative failure policy retains wrapper-owned chain/table/device/runtime
storage on callback-bearing native create failure, OK/null creation and destroy
failure. **It does not retain externally owned userdata or executable callback
targets.** Success on destruction ends access to wrapper-owned storage under the
contract; it does not imply copied global callback functions are unreachable.
Repeated failing callback creates can accumulate retained bundles.

Retain the separate unsafe constructor, paired groups and unsafe modern/legacy
diagnostic routes. Do not replace them with a context-owned captured closure,
`'static` reference alone, TLS dispatch or an unregister-on-Drop guard.
Those alternatives need native invocation identity, allocator compatibility and
shutdown properties absent from this baseline.

Modern global diagnostics and legacy live/null routes are intentionally distinct.
An upscaler method named `configure_global_debug_legacy` really can have
global effects; the name is appropriately explicit even though its receiver is
a context. No automatic fallback is justified. Debug checking, visualization
allocation, per-frame debug view and message filtering are separate native
controls; combining them into one bool or mandatory preset would lose access.

## 8. Configuration, queries, errors and utility composition

### 8.1 Checked types are mostly systematic; their guarantees are deliberately narrow

**Verified:** constructors use private fields, return errors rather than clamp,
and preserve accepted scalar bits. Different domains reflect different contracts:

| Input | Guaranteed locally | Not guaranteed |
|---|---|---|
| `Dimensions` | Both `u32` components nonzero | Device limit, render/output ratio, provider-safe arithmetic |
| Jitter / native motion scale / reactive scalars | Finite values; signs and accepted bits retained | Useful magnitude, finite downstream products, safe provider sampling |
| `FrameTimeMillis` / `PreExposure` | Positive finite values | Correct measured time or matching pixels |
| `NativeSharpness` | Finite inclusive `[0, 1]` | Quality gain; enabled zero is distinct from disabled |
| `CameraParameters` | Ordered finite distances or explicit infinite profile; valid FOV/unit factor | Projection correspondence, native format/domain support |
| `JitterPhaseCount` | Positive native `i32` count | Particular sequence or correct index wrapping |
| `TuningValue::Value` | No invariant by itself; canonical request checks parameter-specific domain | Application, support or effective value |

Sources: [common][common], [reactive options][reactive-options],
[sharpening][sharpening], [query values][query], [tuning][tuning].
Do not impose a universal `[0, 1]` type on every scalar, infer “zero means
default” for literal controls, or use quality expectations to tighten native
access domains without evidence. The tuning header's parameter-dependent
domains support different checks from unbounded finite motion scale.

Likewise, `None` has several **locally distinct** meanings: provider default at
invocation; maximum active size from the receiving context; absence of an exposure
texture/mask; no new callback group. This is not inherently inconsistent.
Treating callback omission as “restore default” would be wrong. R2 differs
because two fields express competing choices for the same descriptor value.

Keep documented defaults: explicit exposure, finite standard depth,
render-resolution jitter-excluded UV motion, linear classification, optional
features off, pre-exposure one and sharpening disabled. Required camera/time/
jitter/reset choices should remain explicit. More `Default` implementations
or a universal builder do not make those contracts better.

### 8.2 R6: tuning should support pure validation without inventing transactions

**Classification:** current composition friction; desirable additive improvement.
The canonical operation already validates every request and should continue to.
It is coherent for `TuningValue` to be unvalidated because its domain depends on
`TuningParameter`. Five distinct public scalar wrappers are unnecessary.

A concrete issue appears in [`request_desired_at_idle`][recovery-example]:
given `[(VelocityFactor, 0.5), (AccumulationAddedPerFrame, 2.0)]`, the first
request may enter native code before the second is rejected locally. There is
no public pure parameter-domain validator, so an engine wanting to reject all
known-invalid settings before entering its idle interval must duplicate ranges.

**Preferred minimal addition:**

```rust
// Proposed pure method; no native call, no eligibility or GPU claim.
TuningParameter::validate_value(self, value: TuningValue) -> Result<(), Error>;
```

A utility or application can validate every request first, then apply them in
order through `request_tuning`. Native failure can still occur halfway through;
earlier acknowledgements are not rolled back, and failure is terminal.
Reset stays an explicit `ProviderDefault` request, not a guessed numeric value.

A `TuningRequest::new(parameter, value)` checked pair is a reasonable alternative
only if reusable staged requests become a real API need. Adding it and changing
the canonical request signature is unnecessary now. Do not add a
`checked_request_tuning` sibling or promise a batch transaction. This
recommendation differs from the earlier rejected optional “checked value”
design: mandatory entry validation remains the sole native-operation policy.

### 8.3 Utility composition works through values; phases must stay visible

**Verified:** D036's [resolver][resolver] uses only public metadata getters and
returns `Option<ProviderId>` accepted by options. It does not use private FFI,
change defaults or introduce a second effect owner. That is a sound architecture.
The selected utility's parsing/ambiguity policy belongs in `utils` rather than
native provider identity.

A representative future multi-setting plan can remain utility-owned:

```rust
struct FramePlan {
    active_render: Dimensions,
    pre_exposure: PreExposure,
    sharpening: Sharpening,
}
// A stateful strategy may return FramePlan from its own history/query inputs.
// The renderer explicitly applies each value, then records normally.
```

R3 makes this example portable; R5 can remove repeated application boilerplate.
Neither requires core to know which algorithm produced the values.
If the same strategy also wants new creation settings or tuning, return separate
outputs for those phases. Creation changes require recreation; tuning requires
idle ordering; generation requires explicit GPU resources/barriers. A plan
combining them is not an atomic native operation.

**Real but bounded testability friction:** `ProviderId` and `ProviderInfo` have
no public synthetic constructors. [Resolver tests][resolver-tests] live inside
the owning query module to build fixtures privately. Downstream tests of
nonempty synthetic snapshots therefore cannot directly use this concrete API
without native-obtained IDs. That limits pure consumer testing, but does not
justify a public `ProviderId::from_u64` capability illusion. Keep opacity now;
if external resolver extensions need fixtures, design a name-classification or
test-data seam with that concrete consumer. Do not introduce a generic provider
trait just to anticipate it.

No utility quality algorithm, calibration or resumed image-quality research is
recommended here. Those remain outside the selected through-1.0 priority.

### 8.4 Errors preserve native information; some taxonomy is upscaler-shaped

**Verified:** [`Error`][errors] preserves native `u32` code and operation and
DX12 HRESULT bits with inspection object/operation. Load failures retain owned
underlying sources independently of the sys error type.
Error/reason enums are non-exhaustive; the main error is `Copy + Eq`.
These are useful contracts. Do not replace them with strings or erase them behind
a general dynamic error in core.

There are smaller inconsistencies:

- `InvalidDispatch::Scalar` also reports construction of portable camera,
  exposure and generator values, before any dispatch exists.
- `Operation::Dispatch` covers both upscaling and generation;
  `Operation::Configure` covers tuning and diagnostic routes. Aggregated logs
  need call-site context to identify the exact operation.
- Most new format-support errors identify `ContextDeviceForResource(role)`,
  but the earlier RG32F-motion path can still use `ContextDevice`; this historical
  exception is documented in the [package README][wrapper-readme].
- `DispatchValidation::UnsupportedResource` is retained for compatibility with
  an earlier **unreleased** mask policy but is no longer emitted.

**R4 recommendation:** remove that dead rejection category before 0.3 unless a
specific downstream development-snapshot promise exists. It describes a rule the
wrapper deliberately stopped enforcing. No production branch uses it.

**R7 desirable improvements:** give native diagnostic context a more precise
operation subtype when a real aggregate consumer needs it; make role attribution
consistent. Do not redesign every error now, remove `Copy` speculatively, or
add effect IDs to every scalar failure before another effect exists. An
`InvalidInput` taxonomy might eventually be clearer, but changing variant names
alone brings less value than R1–R3.

Loading's separate non-Copy error is a defensible exception: it owns paths and
external causes, while operational errors currently contain small owned codes
and reasons. Future owned diagnostic payloads require a deliberate compatibility
review rather than an assumption that `#[non_exhaustive]` makes trait changes
harmless.

### 8.5 Value ergonomics and naming: improve selectively

`ResourceFlags::contains` accepts a mask, but consumers cannot combine the
public constants with `|`. Adding minimal bitwise composition is useful if a
consumer needs multi-resource predicates; a new dependency is unnecessary.
Unknown reported bits must survive. Do not add arbitrary ID construction under
the guise of uniform value conversion.

`UpscaleDispatch` lacks `Debug` while the generator input has `Debug + Clone`.
A useful `Debug` implementation is an additive diagnostic improvement.
Their different Clone contracts are not a GPU-safety mechanism: even an
uncloneable borrowed input can be reconstructed. Adding Clone for visual
symmetry provides little architectural value.

`max_upscale_size` and `active_output_size` describe related output concepts
with different words; `InputColorSpace` means linear/non-linear classification,
not a gamut; `UpscaleResources` also includes output. Their docs establish the
meaning. Prefer a coherent glossary and canonical examples over broad renaming
of published types with no new guarantee. In particular, do not replace every
boolean setter with a two-variant enum merely to follow a guideline.

The two preflight names differ (`validate_dispatch_inputs` versus
`validate_generate_reactive_mask`), but their nonmutating, mandatory-recheck
semantics are consistent. A future naming pass may align them; it is lower value
than correcting input representations.

## 9. Performance and backend/effect extensibility

### 9.1 Keep hot-path checks; do not infer a need for public prepared resources

**Verified source:** [resource inspection][resources] performs COM/device/
identity and format-support calls before native entry. It builds temporary
descriptors and releases inspection references. Calling optional preflight
immediately before every dispatch repeats work; it is diagnostic, not a fast path.
The [preflight fixture][preflight-tests] explicitly checks that a prior validation
does not bypass fresh inspection.

**Historical observation:** the [2026-10-16 performance record][performance-record]
met its selected average additional CPU cost budget for five workloads on one
recorded earlier stack. It does not measure every later format/callback path or
prove current zero overhead. The [setup-query record][setup-record] reports
six-query medians of roughly 230 ms default and 697–707 ms explicit selection.
Those are whole setup paths, not isolated enumeration timings.

Consequences:

- Keep planning/setup queries outside the per-frame path when their results are
  reusable under application semantics. Live jitter queries remain explicit
  native operations; the wrapper does not secretly substitute a sequence.
- R1 removes API branching, **not** native enumeration work or a promised latency.
- Pure parameter grouping and R2 require no new heap owner, dynamic dispatch,
  locking, GPU wait or automatic synchronization in their proposed design.
  Generated-code/latency claims still require measurement.
- Public prepared-resource caches would need identity/lifetime/device/role/view
  contracts and invalidation reasoning. They must not turn cached metadata into
  a certificate of current GPU states or provider compatibility.
- If optimizing repeated device-format queries is later justified, begin with
  measured private caching keyed by the actual stable inputs. Do not redesign
  the public resource model solely on an assumption of excessive overhead.

### 9.2 Vulkan: concrete backend modules, not a universal backend trait now

The existing common/DX12 split and target-specific dependencies provide a useful
seam. The incomplete part is R3; the larger loader gate remains honest about the
only implemented native acquisition path.

Keep backend-specific resources, list/command-buffer contracts and inspection
under `upscaler::dx12` and a future concrete `upscaler::vulkan` when supported.
The existing root DX12 aliases must continue to mean DX12 even when both
features are enabled. A Cargo feature must not silently rebind a public type
name to another backend.

Do not design `Upscaler<B>` or a universal resource trait before inspecting
actual Vulkan device/queue, image-layout, lifetime and function-loading inputs.
Private shared scalar mapping and eligibility can be reused where their
invariants match. Vulkan can run on Windows; OS selection and graphics backend
selection must stay separate. Existing raw message ABI gates must be revisited
against the actual future native ABI, not assumed portable by changing `cfg`.

The reserved Vulkan feature is awkward for blanket all-features tooling.
Retaining it is a deliberate, recently accepted D031 trade-off. This audit finds
no new evidence sufficient to reopen that decision. It does not justify
placeholder modules, fictional devices or a new backend-neutral runtime promise.

### 9.3 Additional effects: reuse mechanics, not unproven public semantics

The generic [`NativeContextOwner<S, D, R>`][owner] is a good **private** seam
for retained storage/dependencies and single-attempt cleanup. It is not proof
that Frame Generation/presentation, Ray Regeneration or Radiance Caching can
use the same lifecycle. Native worker shutdown, presentation, callbacks or
additional failure channels may need different owners and teardown.

Prefer effect-specific public modules/owners and input types, with shared
internal mechanisms after checking their preconditions. Move genuinely shared
types, such as nonzero two-dimensional extents, to a neutral module when a
second implemented effect confirms identical semantics; reexport the same
nominal type from old paths. Do not force all camera, temporal reset or dispatch
inputs into universal traits merely because upstream uses similar fields.

`query` is currently upscaler-heavy despite its neutral name; `DebugLevel` is a
global diagnostic concept located among upscaler values; host callbacks are
inside DX12 interop. These are discoverability debts for future effects, not
current dependency leaks. Add neutral canonical paths when actual second
consumers need them, preserving aliases. A wholesale top-level module move now
would spend compatibility and maintenance effort without settling future
native contracts.

## 10. Proposed coherent target

### 10.1 Required target versus optional extension

After R1–R4, the target keeps the present ownership and operation model:

```text
fsr-sdk-sys
  api / upscale                 faithful selected ABI
  dx12 / loader                 gated native backend and loading

fsr-sdk
  error                         owned core/native rejection information
  query                         owned native query values
  runtime                       local and movable loader owners
                                uniform optional-provider setup queries
  upscaler
    common                      creation options, checked values, selections
                                including Sharpening and MotionVectorScale
    dx12                        local and movable context owners
                                borrowed resources and recording input
      interop                   explicit raw callback integration
    legacy reexports            same type identities and DX12 meaning
  utils
    provider                    optional name-selection policy
```

R5 would add `common::UpscaleFrameParameters` and
`dx12::UpscaleDispatch::from_parameters` without changing the owners.
R6 would add pure tuning validation. No target public `Backend`, `Effect`,
`Policy` or `Allocator` trait is selected. No new crate is warranted.

The organising rules are:

1. **Data composition is portable where it contains no backend objects.**
2. **Creation, queries, live requests and GPU recording remain separate phases.**
3. **Each setting has one canonical representation of effective intent.**
4. **The renderer binds resources and owns submission/completion.**
5. **Native shared-state and callback effects stay explicit unsafe boundaries.**
6. **Reuse implementation without exporting its ownership machinery.**

### 10.2 Cross-workflow outcome

| Rendering-engine workflow | Current friction/limit | Target treatment |
|---|---|---|
| One local upscaler, explicit exposure | Required inputs and two unsafe boundaries are understandable; resource constructors are positional | Keep workflow and mandatory metadata checks; no builder requirement |
| User chooses FSR generation, plans resources, creates context | Repeated provider branching; default helper can differ from creation selection | R1 reuses one optional value everywhere; no fallback/reservation claim |
| Portable engine settings and backend-specific render thread | Sharpening is unavailable in common-only code; frame grouping is application-specific | R3 fixes portable selection; optional R5 centralizes frame data |
| Stateful strategy changes several frame controls | Possible through existing checked values, but motion override needs ordering knowledge | R2 removes precedence; R5 is optional convenience |
| Apply several tuning requests | Later invalid value can follow earlier native acknowledgements | R6 permits validation of the whole input list first; native requests remain nontransactional |
| Generate reactive mask then upscale | Must align signals/region and record explicit UAV ordering/transition; second failure does not undo first | Keep explicit two operations and renderer barriers, as in the composition example |
| Abort a frame after native recording | Context must be terminally recreated even after an OK CPU return | Keep honest limitation; investigate rollback before any reuse API |
| Move renderer between workers | `Send` owners still need arbitrary-thread compatibility and cleanup coordination | Keep optional family and the coordinated wrapper example |
| Install allocator/message callbacks | Registration can outlive installing context and affect others | Keep explicit unsafe island; no fake closure lifetime or unregister guard |

## 11. Migration, review and implementation sequence

This is a proposed sequence for subsequent authorised work, not implementation
performed by this audit.

| Step | Work and dependency | Completion evidence appropriate to the change |
|---|---|---|
| 0 | Maintainer reviews R1–R7; explicitly accept/reject R1/R2 and decide optional R5/R6 scope | Amend relevant decisions only after acceptance; record preserved native meanings and planned breaks |
| 1 | R3 definition move and same-identity aliases; R4 obsolete variant/documentation cleanup | Portable imports, old-path type identity, links and accurate current contracts |
| 2 | R1 canonical query signatures and shared optional-selection routing, both runtime families | Default/explicit downstream consumers, same pre-enumeration input checks, fresh membership, exact codes and no fallback |
| 3 | R2 one motion selection and mapping; migration documentation | All three modes, active render/display geometry, literal finite bits, both families and trait/feature boundaries |
| 4 | If selected, R5 shared frame carrier; R6 pure tuning validation; targeted R7 ergonomics | One state/default source, portable composition and meaningful invalid-list/native-partial-failure checks |
| 5 | Reconcile current public docs/examples and exact final release packages | Existing supported feature/MSRV/consumer checks, ABI/native regressions proportional to changed mappings, final archive coverage separately verified |

Step 3 can proceed independently of step 2 after design acceptance. Optional R5
should follow the selected value vocabulary to avoid rewriting its API twice.
R6 is independent of both. No production changes are needed to accept the
source record itself.

Compatibility implications must be explicit:

- R1 changes new 0.3 query APIs; preserve `None`/explicit-provider behaviour.
- R2 changes the published UV/pixel setter surface if replaced outright and
  the new override composition semantics. Wrapper 0.3 is the appropriate
  opportunity for an intentional break. Do not change resource formats or
  motion encoding under the same change.
- R3 is additive portability if aliases and all existing traits survive.
- R4 removes an unreleased obsolete variant. Documentation corrections do not
  relax accepted safety requirements.
- R5/R6 can be additive. Do not break the six-argument constructor solely to
  introduce a carrier.
- Keep sys at its independently justified compatibility/version boundary unless
  its actual public declarations change. Wrapper ergonomics alone need not alter
  the raw ABI, SDK pin, `windows` version or package licensing.

For subsequent implementation, use the repository's supported configurations:
default, no-default, movable and utils combinations, with deliberate Vulkan
failure controls rather than blanket `--all-features` success. Check exact Rust
1.88 consumers and Windows public APIs, not only unit-module fixtures. R1 needs
selected/default path fixtures; R2 needs descriptor equivalence and applicable
existing native motion regressions. Preserve negative native results and
distinguish source, ABI, loading and GPU evidence.

These are proposed checks for changed contracts, not a request for another broad
native research campaign. The [0.3 preparation record][release-prep] already
requires final archive verification; source feature checks, including callback
checks, are not a substitute. No new image-quality acceptance condition follows.

## 12. R4 detail: current documentation contradictions

**Verified source facts at this baseline:**

- [`UpscalerOptions` type documentation, common.rs:249–254][common] says output
  stays fixed. Its dynamic-resolution setter and implemented dispatch allow
  smaller active output within maximum allocations.
- [`DepthConvention` documentation, common.rs:84–90][common] says callers supply
  R32F depth, while [`UpscaleResources`][frame] and production inspection accept
  several reviewed ordinary/depth-buffer formats.
- [`UnsupportedResourceRequirements` documentation][errors] describes an
  automatic-exposure provider, while [construction][dx12] checks unknown
  required resources for every context.
- The [package README introduction][wrapper-readme] still calls selected
  Windows/native checks deferred, despite the subsequently completed
  [Windows follow-up][windows-record]; pre-exposure has a similar stale pending
  statement. Historical release/decision observations should stay dated, but
  current user guidance should not contradict the implemented baseline.

Fix these during implementation/release documentation work. Do not “repair”
the root README's intentional published-0.2 view into a development status
inventory, and do not rewrite completed historical records. The audit has not
modified any of these files.

## 13. Baseline delta, disagreements and uncertainties

### 13.1 What this investigation confirms or changes

- **Confirmed:** the prior ownership/unsafe-recording/terminal-cleanup design is
  worth retaining. Source sharing is substantial; four concrete owners do not
  establish four architectural implementations.
- **Confirmed, still unresolved as an API choice:** the earlier provider
  consistency recommendation remains relevant. D036's current example now makes
  repeated branching especially concrete. R1 favours the minimal optional
  argument over a new backend session.
- **New comparative finding:** D034's fallback-plus-override scheme contrasts
  with the clearing behaviour of colour-hint override and complicates composed
  setting application. R2 proposes replacing that accepted representation,
  without suggesting its implementation is accidental or incorrect.
- **New boundary finding:** sharpening's backend gate is inconsistent with
  current portable direct controls. R3 needs no hypothetical Vulkan API.
- **Refined composition finding:** existing setters permit multi-setting plans,
  as D033 says. Portable sharpening, one motion choice and optional pure tuning
  validation improve concrete composition without requiring policy-specific core
  hooks. A universal policy framework remains unjustified.
- **Updated safety context:** callbacks/diagnostics and D041 are implemented in
  this baseline. Their shared-state and lifetime constraints rule out simplistic
  safe closure/registration designs; documentation separation does not remove
  obligations.
- **Disagreement with an older recommendation:** do not renew removal of the
  Vulkan feature as a 0.3 goal. Amended D031 deliberately restored it; this audit
  establishes no new reason to reverse that decision.

### 13.2 Evidence limits and remaining investigation

1. **No independent soundness proof or current execution verdict.** This is a
   public-design/source audit. Tests were inspected, not run. No compilation
   success is claimed for the illustrative replacement APIs.
2. **No mechanical published-API comparison.** Compatibility classifications use
   current source and repository release/decision context. Exact registry
   archives, target-specific API differences and any semver tooling output
   remain final release work.
3. **Closed-provider assumptions remain conditional.** Numerical domains,
   transient allocations, boundary-state restoration, query behaviour while GPU
   work is in flight and safe cleanup after native error are not proved for
   every provider/profile by the public headers.
4. **No new callback shutdown/threading proof.** Historical bounded successful
   calls do not establish quiescence, arbitrary worker affinity or safe
   replacement under concurrent hidden native activity.
5. **No universal generator capability.** The [existing provider experiment][generator-record]
   observed mask writes on 3.1.5 and OK returns without writes on 4.1.1/2.3.4.
   The target API must preserve that limitation; neither R1 nor a provider-name
   choice certifies capability.
6. **No current performance guarantee.** Earlier measurements have identified
   workloads/source identities. R1 does not optimize native routing; R2/R5
   require measurement before claiming zero cost or improved latency.
7. **Vulkan and additional effects are architectural stress cases.** Their
   usable future SDK interfaces, device/resource lifetimes, callback identity,
   presentation/shutdown and distribution inputs need independent investigation.
   No FSR SDK 2.4 adoption or upstream availability claim is made here.
8. **The nonessential proposals need consumer judgement.** A concrete engine
   walkthrough should settle whether R5/R6 reduce enough boilerplate to justify
   their public surface. Rejecting them leaves a coherent smaller API.

The actionable conclusion is therefore **retain the foundation, consolidate
selection and frame-control intent before expanding it**. Specifically, accept
or reject R1/R2 consciously before 0.3, complete portable sharpening placement
and remove current contract contradictions, and keep broader generic/runtime/
callback redesigns contingent on real native contracts. Research recommendations
remain separate from maintainer decisions and subsequent implementation.

### 13.3 Record verification

Documentation checks on 2026-10-26 resolved all 59 local link targets and 99
reference-style link uses, checked balanced code fences, UTF-8/LF and trailing
whitespace, and revalidated all 95 source-inventory sizes/hashes against the
unchanged input files. These checks used Python's standard library; they did
not compile the Rust sketches. `python tools/verification/repository_size.py`
passed at 41.41 MB. Final worktree inspection showed only this new record and
its adjacent source inventory; no existing production, decision, roadmap or
topic-synthesis file was modified. No Rust tests or native checks were run.

## Source references

All repository links describe the inspected 2026-10-26 baseline. Source line
numbers in the text are baseline locations, not promises that later revisions
retain those numbers.

[workspace]: ../../../Cargo.toml
[wrapper-manifest]: ../../../crates/fsr-sdk/Cargo.toml
[sys-manifest]: ../../../crates/fsr-sdk-sys/Cargo.toml
[lib]: ../../../crates/fsr-sdk/src/lib.rs
[upscaler]: ../../../crates/fsr-sdk/src/upscaler.rs
[common]: ../../../crates/fsr-sdk/src/upscaler/common.rs
[reactive-options]: ../../../crates/fsr-sdk/src/upscaler/common/reactive_mask.rs
[query]: ../../../crates/fsr-sdk/src/query.rs
[runtime]: ../../../crates/fsr-sdk/src/runtime.rs
[movable-runtime]: ../../../crates/fsr-sdk/src/runtime/movable.rs
[runtime-state]: ../../../crates/fsr-sdk/src/runtime/state.rs
[runtime-queries]: ../../../crates/fsr-sdk/src/runtime/queries.rs
[native-query]: ../../../crates/fsr-sdk/src/query/native.rs
[dx12]: ../../../crates/fsr-sdk/src/upscaler/dx12.rs
[movable]: ../../../crates/fsr-sdk/src/upscaler/dx12/movable.rs
[owner]: ../../../crates/fsr-sdk/src/context.rs
[eligibility]: ../../../crates/fsr-sdk/src/context_policy.rs
[dispatch]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs
[frame]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs
[descriptor]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs
[resources]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs
[sharpening]: ../../../crates/fsr-sdk/src/upscaler/common/sharpening.rs
[generator]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/reactive_mask.rs
[tuning]: ../../../crates/fsr-sdk/src/upscaler/tuning.rs
[interop]: ../../../crates/fsr-sdk/src/upscaler/dx12/interop.rs
[errors]: ../../../crates/fsr-sdk/src/error.rs
[resolver]: ../../../crates/fsr-sdk/src/utils/provider.rs
[resolver-tests]: ../../../crates/fsr-sdk/src/query/provider_selection_tests.rs
[provider-example]: ../../../crates/fsr-sdk/examples/provider_choice.rs
[movable-example]: ../../../crates/fsr-sdk/examples/movable_upscale.rs
[recovery-example]: ../../../crates/fsr-sdk/examples/dx12_recovery.rs
[preflight-tests]: ../../../crates/fsr-sdk/src/upscaler/dx12/tests/preflight.rs
[wrapper-readme]: ../../../crates/fsr-sdk/README.md
[sys-lib]: ../../../crates/fsr-sdk-sys/src/lib.rs
[loader]: ../../../crates/fsr-sdk-sys/src/loader/windows.rs
[raw-api]: ../../../crates/fsr-sdk-sys/src/api.rs
[raw-upscale]: ../../../crates/fsr-sdk-sys/src/upscale.rs
[raw-dx12]: ../../../crates/fsr-sdk-sys/src/dx12.rs
[d013]: ../../adr/D013.md
[d014]: ../../adr/D014.md
[d015]: ../../adr/D015.md
[d016]: ../../adr/D016.md
[d031]: ../../adr/D031.md
[d033]: ../../adr/D033.md
[d034]: ../../adr/D034.md
[d036]: ../../adr/D036.md
[d040]: ../../adr/D040.md
[d041]: ../../adr/D041.md
[prior-audit]: 2026-10-18-src-full-public-api-extensibility-and-v020-break-review.md
[core-utils]: ../core-and-utilities.md
[threading]: ../runtime-threading.md
[recovery]: ../../DX12_RECOVERY.md
[callback-record]: ../../release/2026-10-25-callbacks-diagnostics-verification.md
[windows-record]: ../../release/2026-10-19-windows-upscaler-verification.md
[setup-record]: 2026-10-19-exp-provider-setup-cost.md
[performance-record]: 2026-10-16-exp-native-performance-confirmation.md
[generator-record]: 2026-10-19-exp-reactive-mask-providers.md
[release-prep]: ../../release/0.3.0-preparation.md
[amd-api]: https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h
[amd-upscale]: https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h
[rust-types]: https://rust-lang.github.io/api-guidelines/type-safety.html
[rust-predictability]: https://rust-lang.github.io/api-guidelines/predictability.html
[rust-future]: https://rust-lang.github.io/api-guidelines/future-proofing.html
[rust-docs]: https://rust-lang.github.io/api-guidelines/documentation.html
[rust-unsafe]: https://doc.rust-lang.org/reference/unsafe-keyword.html
[ash]: https://docs.rs/ash/0.38.0%2B1.3.281/ash/
