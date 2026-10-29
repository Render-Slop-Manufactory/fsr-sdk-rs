# Source investigation: core/utility composability and future extension requirements

- **Investigation date:** 2026-10-19.
- **Question:** Can independently selected, optional utilities feed the existing FidelityFX core API without a competing effect API? Which present interfaces are ready, which concrete gaps exist, and what might future effects require?
- **Scope:** Implemented `fsr-sdk`/`fsr-sdk-sys` public boundaries, the proposed optional `fsr_sdk::utils` layer, Upscaler configuration/dispatch, and architecture implications for other FidelityFX effects. This is **not** a proposal to implement a utility or to reopen accepted SDK contracts.
- **Baseline and evidence cut-off:** Uploaded repository snapshot `fsr-sdk-rs-main(11)(4).zip`, SHA-256 `14d05e7ce7d36f07218810a0d6f649596a67265284209b538a4c0b00c6525b00` (intake 2026-10-19); this snapshot describes unreleased development `fsr-sdk 0.3.0` and the pinned FidelityFX SDK **2.3.0**, source commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. Separately supplied **proposed**, not accepted D033 text in `fsr-sdk-rs-D033-proposal.zip`, SHA-256 `652f2c1daa262e3f518af1cafb18e10c830c9e4cf369d0f8c26e393fa84f6bf1`. It is **not present in the baseline archive**. Earlier core/utilities audit supplied separately, SHA-256 `c4bb0518d1c1a511bc51ce274829c8f2d8116c6f9db82ebf360a1ef8e8f059b2`.
- **Method:** Inspected public exports, method signatures and selected implementations, manifests, ADRs, current roadmap, deferred-work boundaries and the prior source audit. Compared design constraints with the official Cargo feature documentation and AMD's API overview (consulted 2026-10-19). No production code was changed, no compile/native/GPU test run, and no unimplemented API was treated as available.

## Inputs and authority

**Repository baseline**

- [Scope](../../../SCOPE.md), [roadmap](../../../ROADMAP.md), [deferred quality work](../../POST_1_0.md), [coverage inventory](../upscaling-coverage.md) and [research process](../PROCESS.md).
- Accepted [D014](../../adr/D014.md) (dispatch/sharpening extensibility), [D015](../../adr/D015.md) (common/backend split), [D016](../../adr/D016.md) (direct provider controls versus authored policies), [D027](../../adr/D027.md) (bounded colour contract), [D032](../../adr/D032.md) (preflight/ergonomics). Proposed [D030](../../adr/D030.md) (native reactive-mask generator).
- [Wrapper manifest](../../../crates/fsr-sdk/Cargo.toml), [crate public exports](../../../crates/fsr-sdk/src/lib.rs), [upscaler exports](../../../crates/fsr-sdk/src/upscaler.rs), [common values/options](../../../crates/fsr-sdk/src/upscaler/common.rs), [DX12 frame/resources](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs), [sharpening](../../../crates/fsr-sdk/src/upscaler/common/sharpening.rs), [native descriptor conversion](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs), [query types](../../../crates/fsr-sdk/src/query.rs), [native quality/size/jitter queries](../../../crates/fsr-sdk/src/query/native/utilities.rs), [DX12 queries](../../../crates/fsr-sdk/src/upscaler/dx12.rs), [movable facade](../../../crates/fsr-sdk/src/upscaler/dx12/movable.rs).
- Earlier core-versus-utilities source audit (`2026-10-19-src-core-utils-boundary-audit.md`, separately supplied; SHA-256 recorded above) was **not** part of the baseline repository. Its classification is reassessed here against **composability and future requirements** rather than presumed normative.

**External primary references** (accessed 2026-10-19; relevant source version distinguished below)

- [Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html), especially union/unification, additive behaviour and SemVer; [Rust API Guidelines: naming/features](https://rust-lang.github.io/api-guidelines/naming.html).
- AMD [FFX API overview](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md) and pinned [upscaler header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h). The public upstream `main` documentation also describes `ffxCreateContext`, `ffxDispatch`, `ffxQuery`, `ffxConfigure`, `ffxDestroyContext` and descriptor chaining; that description is corroborative, **not** a substitute for the pinned header/ABI.
- Upstream SDK documentation distinguishes upscaling, frame generation, denoising/ray regeneration and radiance caching, with different effect DLLs. This supports future **effect-specific** public contracts, not a generic utility trait. [Upstream API guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/main/Kits/FidelityFX/docs/getting-started/ffx-api.md) (mutable `main`; version-qualified interpretation required).

## Evaluation framework

A utility is *directly composable* when it can produce existing **public, checked core inputs** that a caller can explicitly supply, without gaining privileged FFI access, changing defaults, bypassing validation, or acquiring renderer-owned GPU objects. Four independent questions matter:

1. **Value composability:** Can a utility return `Sharpening`, `Dimensions`, `JitterOffsetPixels`, `PreExposure`, `ColorEncodingHint`, or another public value already accepted by the relevant operation?
2. **Placement and timing:** Is it a creation-time `UpscalerOptions` setting, per-frame `UpscaleDispatch` input, live `&mut Upscaler` query/Configure operation, or a separate GPU dispatch? Producing the right Rust type does **not** make these interchangeable.
3. **Authority and evidence:** Is the value directly supplied by AMD, a mathematically defined conversion, a caller rule, a maintainer preset, or an empirically calibrated strategy? D016 requires provenance and appropriate evidence.
4. **Safety and ownership:** Who owns the GPU textures, state transitions, native thread co-ordination, provider validity, and history reset? A utility cannot silently discharge those obligations by returning a checked scalar.

Do not use `enum Native | Calibrated` or a generic `SharpeningStrategy` trait as the default: it would couple the core to optional policy provenance without resolving any of these four questions.

## Findings: present API and immediate needs

| Concern | Direct source fact (**verified as source**) | Utility integration and required core change |
|---|---|---|
| **Native sharpening** | `NativeSharpness::new(f32) -> Result<_, Error>` checks `[0,1]`; `Sharpening::{Disabled, Native(NativeSharpness)}` is public, `#[non_exhaustive]` and derives `Clone`; `UpscaleDispatch::set_sharpening(Sharpening)` exists (`sharpening.rs:8–59`, `frame.rs:272–281`). | **Ready, no core change.** Any selector can return `Sharpening`. Unlike an earlier conversational shorthand, `Sharpening::Native(0.35)` is **not** the current API; the checked scalar must be constructed. Current API uses a setter, not `.with_sharpening()`. |
| **Per-frame composability** | `UpscaleDispatch::new` takes command list, borrowed resources, jitter, frame time, camera, reset. Separate setters support sharpening, pre-exposure, motion units, active render size, encoding hints (`frame.rs:189–380`). | **Ready for individual settings.** A higher-level policy may compute several values; caller applies each explicitly. No universal `apply_policy` is needed. Creation-only settings cannot be applied at dispatch time. |
| **Creation profile** | `UpscalerOptions` has private fields; public setters/getters for fixed maxima, depth, motion, exposure, dynamic-resolution enablement, HDR, colour classification and provider (`common.rs:200–418`). | **Ready for caller-authored presets.** A utility can construct/modify options before context creation. Changes to `UpscalerOptions` do not modify an existing context; a utility must not imply hot reconfiguration. |
| **Native quality and jitter** | `QualityMode` and `JitterPhaseCount` are checked; `Upscaler`/`MovableUpscaler` expose actual provider ratio/render-size/phase/offset queries, requiring `&mut self` (`query.rs`, `dx12.rs:300–365`, `movable.rs:97–135`). | **Core by provenance.** A custom jitter sequence or a frame counter is a utility candidate. A helper that wraps native queries remains provider-dependent, must respect serialisation/poison, and must not silently substitute local formulas when unavailable. |
| **Active render size and motion** | `set_active_render_size(Option<Dimensions>)` selects a subregion at fixed maximum allocation/output; `MotionVectorUnits` changes native scale mapping but does not convert texture pixels (`frame.rs:350–380`, `descriptor.rs`). | **Current single-field planning is feasible.** Arbitrary per-axis MV scaling is still U05; variable output and allocation ownership remain M14 work. A controller must not advertise these as implemented merely because it computes dimensions. |
| **Exposure and colour** | `PreExposure`, `InputColorSpace`, `ColorEncodingHint`, `ExposureMode` express native controls, not pixel transforms; mismatch handling is explicit (`common.rs`, `frame.rs:283–347`, `upscaler.rs:17–61`). | **Scalar planning is possible, universal colour-correct behaviour is not established.** sRGB/PQ math may be separately specified, but texture conversion and any provider-specific exposure ordering cannot be invented by a utility. D027 retains failed changing-pre-exposure predictions. |
| **Masks and generation** | `UpscaleResources` borrows renderer textures and accepts optional supplied masks; D030 describes a separate *native* generator, still proposed/unimplemented in this snapshot (`frame.rs:11–145`, D030). | A helper selecting mask-generation *parameters* could be pure utility. Actual generation is native **core** once implemented; creating GPU textures, transitions and submission remain renderer integration. No utility should present generated pixels or capabilities as implemented today. |
| **Preflight and failure paths** | `validate_dispatch_inputs` is a core method for both ownership families (D032); dispatch performs its own validation; native failures can poison subsequent operations. | **Remain core.** A utility can construct values before preflight, but cannot turn an unsafe dispatch into a safe one or automatically promise recovery after native error/abandoned recording. |
| **Movable and provider selection** | `MovableUpscaler` is opt-in `Send + !Sync` with native caller-side co-ordination. Provider selection and live configuration are separate from per-frame inputs (`lib.rs`, `movable.rs`, D013/D028/D029). | An optional utility must not add hidden shared state, implicit provider queries, automatic fallback, cross-thread Drop or default synchronisation. Any stateful helper's own `Send`/`Sync` behaviour is independent of native owner's contract. |
| **Platform and feature boundary** | Common types can compile without DX12; `dx12` is default, `movable` optional; **no `utils` feature/module exists** (`Cargo.toml`, `upscaler.rs`). | Introduce only alongside a concrete utility. Keep a genuinely portable utility usable under `--no-default-features --features utils`; gate DX12-specific functionality separately if later justified. |

### Exact present-day sharpening composition

The following snippet reflects the **existing** setter/type signatures; it does not implement calibration:

```rust
use fsr_sdk::upscaler::{NativeSharpness, Sharpening, UpscaleDispatch};

fn choose_sharpening(value: f32) -> Result<Sharpening, fsr_sdk::error::Error> {
    Ok(Sharpening::Native(NativeSharpness::new(value)?))
}

fn apply(frame: &mut UpscaleDispatch<'_>, selection: Sharpening) {
    frame.set_sharpening(selection);
}
```

A future `fsr_sdk::utils::sharpening::calibrated(&inputs)` *could* return `Result<Sharpening, UtilityError>` and feed `set_sharpening` unchanged. This is **hypothetical and deferred**; neither the function nor its evidence exists. `Disabled` and `Native(0)` must remain distinct. D016 explicitly forbids treating the old failed sharpening-effect gate as proof of calibration.

## Long-term requirements and likely pressure points (**inference**, not approved work)

| Likely use case | What would be reusable | Core sufficiency / possible trigger for a later API review | Evidence and ownership boundary |
|---|---|---|---|
| **Custom jitter orchestration** | Sequence index, wrap/phase management, deterministic alternative sequences, portable validation. | The current typed `JitterOffsetPixels` already feeds frame creation. Additional core API is justified only by an actual provider-query limitation; the jitter sequence manager belongs outside core. | The renderer must apply the same offset to projection and dispatch; utility cannot verify camera-side application. Native `jitter_offset` already exists in core. |
| **Sharpening presets or calibration** | Provider/workload-specific selection function returning `Sharpening`, opt-in and provenance-labelled. | **No new core dispatch API needed.** A compact policy metadata type may be needed within `utils` to report target, applicability, evidence revision and explicit unverified fallback. | Per-provider, colour, exposure and temporal context matter. Quality research and calibration are explicitly deferred past 1.0.0 in `POST_1_0.md`. |
| **Dynamic resolution and render-size policies** | Frame-time filter, hysteresis, quantisation, bounds, proposed `Dimensions`. | Existing `UpscalerOptions`/frame setters suffice **only** for fixed output/max allocation and supported active render sizes. M14 variable output and U05 MV rules are separate core work. | Renderer allocates/reallocates textures, schedules history reset and owns GPU fences; choosing numbers is not allocating resources. |
| **Multi-field frame configuration** | A transparent, validated *plan* of sharpen/pre-exposure/encoding/active region settings. | Existing setters are adequate initially. If applications repeatedly need consistent atomic assembly, consider a **utility-owned** checked plan applied to newly constructed frames or options, without making core depend on the plan. Never make `request_tuning` and dispatch frame setters appear atomic. | Explicitly separate creation-time choices, per-frame values and native Configure calls; native Configure has poison/idleness semantics, no transaction/rollback. |
| **Colour and exposure maths** | Explicit scalar transforms and colour-transfer utilities with named units, reference luminance and operating domain. | Core types are inputs, not calibrated signal converters. A new conversion would need independent, testable mathematical semantics; `InputColorSpace`/`ColorEncodingHint` do not imply it. | CPU scalar arithmetic can be portable. Editing textures, gamut/HDR output pipelines or trusting provider-internal exposure ordering is renderer/domain work. |
| **Motion-vector adapters** | Coordinate/units/scaling maths for application-owned motion data. | Pure coefficient conversions are utilities. Exact native descriptor mapping stays core. Arbitrary signed/asymmetric per-axis scale awaits U05 if supported. | Full MV texture transforms need GPU access and belong in an integration layer; distinguish pixel/UV/NDC, direction, jitter convention and active region. |
| **Provider-aware profiles** | Optional selection using `ProviderInfo`/ID, supported requirements and explicit policy inputs. | Query access exists; however, `&mut self` native calls and provider identity are not pure getters or capability guarantees. A utility that queries automatically could impose undocumented serialisation and failure costs; favour caller-supplied **owned query snapshots** when a specific need emerges. | No provider/version stability assumption across runs; provider results are not a complete capability oracle. Snapshot freshness, failure and device identity must be documented. |
| **Other SDK effects: frame generation** | Possibly shared timing, dimensions and a small number of pure scalar conversions. | Do **not** predesign shared `EffectConfig`, `EffectPolicy`, trait-object registries or universal dispatch wrappers. Different resource, presentation, queue and lifetime contracts will need effect-specific core types; evaluate extraction after an implemented second effect. | GPU scheduling, swapchain and presentation are not safe generic numeric utilities. |
| **Ray regeneration / radiance caching** | Some portable geometry, exposure, signal and parameter calculations may overlap with upscaling. | Wait for actual bindings and stable effect-specific types. Shared calculation may be free-standing or duplicated briefly rather than a premature generic trait. | Their GPU inputs and work scheduling differ; supporting their upstream effect names is not evidence of wrapper coverage. |
| **wgpu/Bevy integration** | Adapter code for native handles and frame input production. | Deliberately outside `fsr-sdk::utils`, as the consumer owns encoder access, texture states, history and dispatch completion. | An independent `fsr-sdk-wgpu`/`fsr-bevy` layer could use both core and portable utilities later. |

### Specific interface design recommendations

1. **Keep the return-to-core pattern as the default.** A one-parameter policy returns a checked core value. Multiple related values can be returned as an ordinary utility-defined struct, with explicit per-field application. Avoid a second `Upscaler` type, a `Sharpening::Calibrated` core variant or a dynamic strategy registry.
2. **Do not move validation into optional utilities.** Constructors, invariants, direct provider queries, error mapping and lifetime controls must remain available without `utils`. A policy may reject its own domain independently; that is not a replacement for core validation.
3. **Preserve lifetime and phase distinctions.** Clearly mark *creation-time* (`UpscalerOptions`), *per-frame* (`UpscaleDispatch`), *live native Configure* (`request_tuning`) and *GPU-operation* boundaries. One global `apply(config)` obscures incompatibilities and could conceal native failure/poison behaviour.
4. **Keep policy selection inspectable.** Prefer a pure, deterministic function when possible; return a typed selection and, if needed, separate provenance/diagnostics. Explicit fallback and provider matching belong to the policy, not silently to the core.
5. **Prioritise composability over universal helpers.** A utility need not use a trait, builder or object hierarchy to compose. Add helper traits or callbacks only after two concrete consumers demonstrate common needs; composition by value is already supported by Rust's ordinary APIs.
6. **Future utility error handling:** Prefer feature-local `utils::...Error` types for policy-domain failures rather than continually expanding core `Error` with policy categories. Do not hide or convert native errors so their operation and return code are lost.
7. **Possible tiny future core additions (not currently justified):** only when a *real* utility cannot express a supported native configuration with existing checked types, or when immutable inspection/reuse of options is materially insufficient. Require a minimal downstream consumer demonstrating the gap before exposing new setters, getters, generic traits or effect cross-products.

## Cargo feature and verification implications

The manifest currently offers `default = ["dx12"]`, `dx12`, and `movable`; a `utils` gate is **not implemented**. Cargo unifies features across dependent packages, so they must be additive and must not alter defaults, the direct-control API, safety requirements or native dispatch behaviour ([Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)). `utils = []` in the existing crate is an acceptable **future** first step, not a mandate to publish another crate. If a utility depends on DX12, either expose only its portable calculation outside that gate or give the DX12-specific operation an explicit platform gate. A feature named `utils` should not imply universal GPU support.

When the first utility is authorised, proportionate acceptance checks should cover:

- Linux/host `--no-default-features` and `--no-default-features --features utils` for pure utilities; Windows default and DX12 + utils; Windows `movable` + utils where relevant. Respect actual supported targets/feature combinations; do not assert native Linux execution.
- Downstream consumer compilation of **both** direct Core and utility-selected values through the same real public `set_*`/constructor paths; core behavioural equivalence with and without the feature.
- Edge cases and numerical domains, reproducibility and property tests for pure policies; state transitions and negative controls for any stateful controller.
- No new native work or `unsafe` hidden behind a nominally safe convenience method. Any invocation of native queries/Configure/dispatch must retain all applicable contracts and failure semantics. Prove safety independently of its convenience label.
- Provider/evidence matching tests for claims of calibrated behaviour. Keep provider-independent mathematical correctness separate from native-provider image/performance claims.

## Alternatives and trade-offs

| Alternative | Assessment |
|---|---|
| Add `Sharpening::Calibrated` or other SDK-policy variants to core now | Unnecessary coupling of core inputs to authored policy/evidence revisions; would risk D016's direct-control separation and introduce versioning pressure. |
| Generic `EffectPolicy` / `ConfigurableEffect` traits today | No demonstrated common lifetime, query or configuration phase across even two implemented effects. Deferred until justified by concrete overlap. |
| Expose only pure functions without a feature | Technically feasible for tiny, dependency-free maths. An opt-in module/feature is preferable when it avoids confusing native SDK controls with authored choices; do not force a gate for every trivial helper. |
| Separate published `fsr-sdk-utils` crate immediately | No clear dependency graph, independent release cadence or multiple consumers justify the extra package yet. May become appropriate for sizeable dependencies, non-FSR reuse or independent semver requirements. |
| Have utilities own DX12 resources and renderer lifecycle | Would break the existing engine-independent scope and disguise caller GPU obligations. Place backend/engine integration elsewhere. |

## Baseline delta and verdict

- **Confirmed:** D014/D015 already provide the essential *value* and *configuration-phase* modularity for prospective sharpening utilities, common render values and caller-defined presets. No confirmed core API defect or required public refactor was identified. The native helper queries are core regardless of their internal filename.
- **Refined relative to the earlier audit:** Composability is not one uniform slot. Creation, frame inputs, queries, Configure and GPU operations have distinct timing/safety semantics. Utility output types must correspond to those phases; a policy returning several settings is a planning operation, not a reason to turn the core into a policy engine.
- **Proposed refinement to D033:** Add an explicit design invariant: *Optional utilities should preferentially produce existing checked core input types, leaving core independent of policy provenance; additional policy-specific core hooks require a demonstrated missing capability.* Also state that this is a preference, **not** a universal trait/builder requirement, and expressly preserve creation/frame/Configure/GPU-operation boundaries.
- **Recommendation:** Accept D033 as a scope boundary after any desired wording refinement, but do **not** add `utils`, migrate existing APIs or start calibration now. Complete the selected native Upscaler scope and release checks first; pilot exactly one small, demonstrated utility when post-1.0 work is explicitly reopened or an independent non-quality need is authorised.

## Limitations and follow-up questions

- This is an **architecture/source** investigation, not a native performance, GPU image-quality or Rust-compilation result. Examples were checked against declared API signatures, not compiled here; the environment has no Rust/Cargo toolchain.
- The separately supplied D033 is **proposed**, not present in the audited repository baseline. This record does not accept it, add it to the repository or change `ROADMAP.md`/topic syntheses. The existing 0.3.0 release objectives and post-1.0 deferral remain controlling.
- Future-effect API details may evolve with the pinned SDK and later effect-specific decisions; external `main` documentation is not pinned evidence of an implemented wrapper feature.
- **Concrete test still needed before any utility API approval:** build a downstream miniature caller using only public `fsr-sdk` types, showing both an explicit native configuration and one utility-selected configuration. Record the exact friction (if any) without adding a generic trait speculatively.
- **Separate research if calibration is later authorised:** define calibration target, hardware/provider/version domain, data split, controls, failure/negative evidence and applicability/fallback rules under D016; do not infer this from the already failed V02 sharpening-effect criterion.
