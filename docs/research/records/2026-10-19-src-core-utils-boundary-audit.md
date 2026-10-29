# Source investigation: core and optional utility responsibilities in `fsr-sdk-rs`

- **Investigation date:** 2026-10-19.
- **Question:** Which implemented Rust APIs belong in native ABI bindings, the faithful/sound Rust core, or optional project-defined utilities, and is a restructuring justified?
- **Baseline:** Supplied `fsr-sdk-rs-main(11)(4).zip`, SHA-256 `14d05e7ce7d36f07218810a0d6f649596a67265284209b538a4c0b00c6525b00`; development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`, pinned FidelityFX SDK v2.3.0. The **separate** `fsr-sdk-rs-D033-proposal.patch` (SHA-256 `cdc225ebf5d747b55fc747f4d579e688531dea45dbbab9898d08f615ceca53ca`) is a proposed architecture decision, *not* part of the baseline archive or an accepted decision.
- **Method:** Read public exports, implementation files, manifests, D015/D016/D032, D033 proposal, project scope, the deferred backlog and relevant existing research. Checked external Cargo feature and SemVer guidance on 2026-10-19. No repository files modified, and no compilation, native execution or GPU experiments performed.

## The operative distinction

The right division is **source of semantics**, not API convenience, code complexity or mathematical content.

1. **`fsr-sdk-sys`:** Curated ABI surface, exact upstream types/layouts/symbols, without authored application policy (`SCOPE.md`, crate responsibilities).
2. **`fsr-sdk` core:** A faithful Rust representation of an AMD operation or control plus the checks, transformations, resources, lifetime handling and safety conditions required to expose it coherently. An additional public convenience method can remain core when it is only an alternate way of expressing the same native operation, with no new choice or behaviour.
3. **Future optional `fsr_sdk::utils`:** A deliberately selected *new* computation or strategy (choice, smoothing, sequence, conversion, orchestration) whose behaviour is specified by this project rather than the native provider. It must be optional, explicit and not replace a direct native control.
4. **Renderer/integration layer:** GPU resource creation and transitions, camera projection, queues/fences, render graph, history ownership, engine lifecycles or presentation. This is neither generic wrapper core nor a generic numerical utility.

The third layer is a **module/feature boundary within `fsr-sdk`**, not necessarily a third crate. D016 already separates native provider controls from authored SDK-level quality policies; D033 would sharpen the module boundary without changing D016. The post-1.0 quality-policy deferral remains binding (`docs/POST_1_0.md:1–18`).

## Implemented API inventory and classification

| Current surface | Classification | Evidence and explanation |
|---|---|---|
| `fsr-sdk-sys` headers, resource descriptors, query/configure tags and loader symbols | **ABI / sys** | `crates/fsr-sdk-sys/src/`; `SCOPE.md`. The exact interface remains upstream-defined. |
| `Runtime`, `MovableRuntime`, DLL retention, `NativeContextOwner`, poisoned states, native error codes | **Core: essential safety and lifetime** | `src/runtime.rs:122–140`, `src/context.rs:20–78`, `src/context_policy.rs`, `src/error.rs`. Movable is not a convenience policy; its explicit unsafe acquisition and `Send + !Sync` contract remain core. |
| `Dimensions`, `FrameTimeMillis`, `JitterOffsetPixels`, `PreExposure`, `CameraParameters` | **Core: checked representations** | `src/upscaler/common.rs:43–78,473–667`. Constructors validate native-facing domains; they do **not** generate a jitter sequence, convert elapsed seconds implicitly, or transform colour data. The infinite-far placeholder is native mapping, not an independently authored rendering algorithm. |
| `UpscalerOptions`, depth/MV/exposure flags, HDR and non-linear classification | **Core: direct configuration** | `src/upscaler/common.rs:81–423`. Construction defaults define the supported wrapper profile; they are not a calibrated quality preset. `InputColorSpace`/`ColorEncodingHint` forward native controls without decoding input pixels. |
| `UpscaleResources::new/without_exposure`, setters for supplied masks | **Core: native resource representation** | `src/upscaler/dx12/dispatch/frame.rs:11–145`. The constructors express different native input requirements and add no image generation, mask heuristics or GPU resource ownership. |
| `UpscaleDispatch` frame parameters, sharpening, motion units and active render extent | **Core: direct dispatch and unit mapping** | `src/upscaler/dx12/dispatch/frame.rs:147–…`, `src/upscaler/dx12/dispatch/descriptor.rs:36–…`. Converting UV-vs-pixel MV units to native `motionVectorScale` is required for faithful representation (`descriptor.rs:56–63`). It does not rewrite motion-vector texture pixels. |
| Optional `validate_dispatch_inputs` | **Core: optional preflight for compulsory checks** | `src/upscaler/dx12/dispatch.rs:21–39`; D032. It repeats the dispatch metadata inspection without calling FidelityFX, but it does not select a policy, change native defaults or prove GPU states. Moving it would split safety-relevant validation from dispatch. |
| Provider enumeration/identity/selection, requirements and memory queries | **Core: provider-defined facts and explicit selection** | `src/query.rs`, `src/runtime.rs:142–208,386–…`, `src/upscaler/dx12.rs:378–…`. Explicit provider selection is caller choice, not an automatic wrapper recommendation. |
| `QualityMode`, ratio/render-size/jitter-phase/jitter-offset query methods | **Core: *native* utility queries** | `src/query.rs:16–71`, `src/query/native/utilities.rs:3–153`, `src/runtime.rs:142–291`; these form and invoke AMD query descriptors, check outputs and preserve provider routing. No substitute ratio formula, local Halton sequence or rounding policy is implemented. The internal file name `utilities.rs` is **not** evidence of a `utils`-layer responsibility. |
| `TuningParameter`, `TuningValue`, `request_tuning` | **Core: native Configure** | `src/upscaler/common.rs:7–42`, `src/upscaler/tuning.rs`, D029/D032. Checked domains and the null-pointer provider-default route are native-facing constraints; no locally chosen preset is applied. |
| Native automatic exposure, explicit pre-exposure, native sharpening, supplied reactive/T&C masks | **Core: native operations/inputs** | `src/upscaler/common.rs`, `src/upscaler/dx12/dispatch/sharpening.rs`, `src/upscaler/dx12/dispatch/frame.rs`. `Automatic` is AMD's mode; `Native(0)` differs from `Disabled`. A wrapper-authored exposure controller, sharpening recommendation or mask synthesiser would be a separate feature. |
| Research scripts, calibration Halton sequence, sample GPU pixels and test harnesses | **Test/research equipment, not public utility APIs** | `tools/verification/temporal_calibration.py:37–47`, `crates/fsr-sdk/tests/native/`, verification protocols. Their maths is designed for controlled fixtures, not necessarily a general-purpose renderer algorithm or supported production API. |

**Result:** No confirmed production function currently implements an independently designed SDK-level utility policy. The code already respects the proposed semantic boundary more consistently than the names ‘helper’ and ‘utility’ might suggest. There is nothing that demonstrably **must** move from core to a new module now.

## Borderline cases and concrete restructuring recommendations

**A. Do not relocate checked types or preflight.** `Dimensions::new`, `PreExposure::new`, `CameraParameters::new_infinite`, `ResourceFlags::contains`, the exposure-specific resource constructor and `validate_dispatch_inputs` make direct native access viable. Relocation would fragment contracts, increase feature combinations and risk an avoidable break in the established import paths (D015). A simple method is not necessarily optional policy.

**B. Do not relocate the four native “utility” queries.** Despite the folder name, `query/native/utilities.rs` calls `ffxQuery` with actual SDK descriptor tags and checks provider outputs. Replacing these with locally computed approximations would be a semantic regression. Renaming the private file to avoid confusion is optional documentation hygiene, not a priority refactor.

**C. Keep low-level unit conversion in core; classify *content* conversion separately.** The descriptor currently maps `MotionVectorUnits::Uv` to active-region dimensions and `Pixels` to `(1,1)` (`descriptor.rs:56–63`). That is required descriptor mapping. A separately specified conversion of entire motion-vector fields, camera-space/NDC coordinates or image pixels would be a possible utility if CPU-only and renderer-independent; GPU texture transitions/shader execution belong to an integration layer.

**D. Separate provider outputs from independent policy.** `render_size_for_quality` returning the native size is core. A controller that picks a quality mode to satisfy a target GPU time, smooths resolutions or handles hysteresis would be a utility. It should expose the rule and its inputs; never silently change `UpscalerOptions` or overwrite native recommendations.

**E. Distinguish representation defaults from recommended presets.** Existing core defaults select a documented, deliberately narrow supported profile (`common.rs:220–238`). They are not claims of optimal quality. A `recommended_preset`, provider-based auto-selection or generated jitter scheme would choose additional behaviour; this belongs in utils only after separate review and evidence. Custom quality policies and calibration are explicitly deferred until after 1.0.0.

**F. Treat exposure/colour helpers carefully.** A precisely documented sRGB/PQ transfer function, encoding conversion or exposure arithmetic *could* qualify as a pure, independent utility, but must specify signal domain, primaries, luminance units and ordering. It must not purport to reconstruct a provider's internal transfer/pre-exposure processing. M16's unresolved native behaviour does not itself justify adding a conversion helper, or moving current `InputColorSpace`/`ColorEncodingHint` from core.

**G. Avoid convenience wrappers that launder unsafe work.** Helpers managing submission, barriers, GPU resources, temporal history, or async cleanup need renderer ownership and GPU coordination. They cannot make `dispatch` or `MovableRuntime` safe merely by providing a nicer signature. Consumer-owned cleanup and cross-thread native obligations remain. A separate `fsr-bevy`/wgpu adapter is the appropriate home for corresponding renderer lifecycle operations.

**H. Shared provider/core mechanics remain internal.** There is some deliberate public method repetition across `Runtime` and `MovableRuntime` and across `Upscaler` and `MovableUpscaler`. Existing shared query functions and the generic sealed retention core already mitigate semantic divergence (`runtime/queries.rs`, `query/native/utilities.rs`, `upscaler/dx12.rs`, `context.rs`). This is an **internal maintenance** consideration, not a reason to invent a public utils trait or move native APIs. Further consolidation should be driven by concrete divergence, not by the new layer label.

## Cargo and platform boundary

The current `[features]` is `default = ["dx12"]`, `dx12 = ["fsr-sdk-sys/dx12", "dep:windows"]`, `movable = []`; **no `utils` feature exists** (`crates/fsr-sdk/Cargo.toml:20–32`). Common typed inputs are available without Windows/DX12, while runtime and native upscaler remain gated (`src/lib.rs:184–208`, D015/D031).

An eventual `utils = []` plus `#[cfg(feature = "utils")] pub mod utils;` is an implementable *option*, not a requirement of this source audit. Only introduce it together with a real, reviewed utility, and keep it off by default. Pure numeric utilities should build with `--no-default-features --features utils` on Linux and Windows, and not pull in `windows`, provider DLLs, GPU code or new allocator/coordination guarantees. Backend-specific utilities require their own justified gating. Test the public configurations `none`, `utils`, `dx12`, `dx12+utils` and `dx12+movable+utils` on supported hosts/CI. Existing consumer compile-fail tests should continue to enforce native gates.

Cargo's reference explicitly requires additive features because dependency resolution **unifies** enabled features (accessed 2026-10-19: <https://doc.rust-lang.org/cargo/reference/features.html>). The Cargo SemVer guide notes that adding a feature is usually compatible while removing one is generally breaking (accessed 2026-10-19: <https://doc.rust-lang.org/cargo/reference/semver.html>). Thus, avoid predeclaring empty or speculative individual utility features, and never let feature unification change core defaults or dispatch safety.

## D033 proposal assessment

**Source review supports the central boundary.** It is consistent with the accepted D016 separation and the existing D015 common/DX12 split. However, D033 is **proposed only**, supplied in a separate patch. No actual `utils` module is implemented or authorised.

Two small clarifications would improve D033 *before accepting it*, without redesign:

1. State expressly that **representational convenience and small helper-like constructors are core** whenever they enforce/faithfully present a native contract, even when a user could implement the same calculation externally. Examples: `without_exposure`, `validate_dispatch_inputs`, `CameraParameters::new_infinite` and simple flag accessors.
2. Define the **allocation/GPU boundary more sharply**: pure scalar/CPU calculations may become generic utilities; ownership of textures, command lists, state transitions, queues, temporal histories and native cross-thread cleanup remains with the renderer/integration layer or the core's existing explicit unsafe contract. Optional utilities must never silently absorb `unsafe dispatch` obligations.

Optionally, replace ‘SDK-level policies’ with ‘project-authored optional policies’ in cross-references where readers could confuse such algorithms with AMD provider policy. This is terminology clarification only: D016 does not need reopening.

## Baseline delta, limits and next decision

- **Confirmed:** Production code contains direct native operations, representation/validation and shared owner mechanics, but no substantial implementation of a project-authored convenience policy needing extraction.
- **Not established:** GPU behaviour, quality, performance, or compatibility of hypothetical utilities; no prototype was built and no new native tests were run. This report is a static source/architecture audit only.
- **Unresolved choice:** Whether/when a concrete utility should exist is a product decision requiring an actual repeated integration problem, not a discovery made by this audit.

**Recommendation:** **Accept or lightly clarify D033 as a future architectural constraint, but make no structural code migration now.** The present core is correctly scoped. Complete selected native upscaler coverage and existing release verification. After a concrete renderer-independent pain point is observed, design a single optional utility against the public core API, specify its semantics and tests, and only then add the `utils` feature/module. Keep image-quality policies deferred as already agreed.
