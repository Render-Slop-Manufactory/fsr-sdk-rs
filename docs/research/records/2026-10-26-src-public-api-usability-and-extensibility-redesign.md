# Public API redesign review: usability and future extensibility

- Investigation date and evidence cut-off: **2026-10-26**.
- Kind: **source investigation and architectural design review**.
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, Rust 1.88,
  edition 2024, AMD FidelityFX SDK v2.3.0 at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Question: which public API would best balance straightforward renderer
  integration and future growth if preserving current signatures has low weight?
- Maintainer direction: breaking changes are available because adoption is low;
  assess usability **and** extensibility, and reduce the preference for
  conservative changes. This is permission to reconsider the design, not an
  instruction to maximize change or implement the recommendations.
- Status: **recommendations for review**, not accepted decisions, an
  implementation, a release gate or a change to project scope.
- Method: inspect the public production surface, shared mechanics, examples,
  selected fixtures, decisions and topic findings; compare concrete caller
  workflows and extension scenarios; check identified primary external sources.
- Exclusions: no implementation, compilation of proposed APIs, native/ABI
  execution, GPU work, benchmark or user study. The findings below are source
  facts and design judgements, not measured developer satisfaction.
- Provenance: the [input inventory][inventory] identifies the inspected source
  baseline. All 95 inputs in the [earlier audit's inventory][prior-inventory]
  still matched their recorded sizes and SHA-256 hashes at this investigation.
  The worktree was clean at intake.

## 1. Verdict and disagreement with the first review

**The existing API is a viable native wrapper, but neither it nor the first
audit's R1–R4 package is the preferred long-term public design under this brief.**

The first [audit][prior] correctly identifies several local inconsistencies.
Its recommendation is nevertheless too incremental for the revised objective:
it repairs provider argument conventions, motion precedence and one module
boundary while retaining the larger sources of caller ceremony. In particular,
it treats portable frame data as an optional convenience and a setup scope as
something to consider only after further consumer growth. Both address workflows
already present in the repository and provide useful extension boundaries now.

The preferred direction is a **more substantial redesign of public assembly
and operation scopes**, reusing the private ownership, validation and native
mapping mechanisms where their contracts still fit. It does not require a new
crate, an engine abstraction or a universal backend trait.

| Recommendation | Usability benefit | Extensibility benefit | Assessment |
|---|---|---|---|
| U1: an effect/backend setup scope with one provider selection and context creation | Stop repeating device/provider arguments and move related operations together | Keep new setup queries out of a growing flat Runtime API | Recommend |
| U2: canonical portable frame parameters, separate resource bindings and command list | Prepare/update frame data without DX12; use a short recording call | Add frame controls without growing constructors or duplicating backend data | Recommend |
| U3: named resource assembly with an extensible builder | Make same-typed resource roles visible at the call site | Add optional resources without constructor proliferation | Recommend |
| U4: one motion choice and one complete colour-hint request | Inspect and replace effective intent without hidden permission/fallback state | New choices extend the relevant value instead of adding interacting setters | Recommend |
| U5: checked, portable tuning requests | Stage a complete desired configuration before any native mutation | Reusable request data composes with queues, editors and utilities | Recommend |
| U6: explicit usability status and local recording-abandonment invalidation | Represent the renderer's known terminal state in the owner | Centralize future terminal causes without pretending to track GPU completion | Recommend |
| U7: reorganize errors, metadata construction and canonical names | Improve pure testing, discovery and diagnostics | Avoid dispatch-shaped errors and unnecessary public trait restrictions | Recommend, with the bounded choices below |
| U8: separate loader trust from effect integration trust | Make unsafe assumptions reviewable where the effect is selected | A new effect need not extend every loader acquisition's contract | Recommend as an explicit safety-contract redesign, not a cosmetic move |

“Recommend” means a design preference supported by the analysis, not an
unconditional claim that the sketches are ready to implement. Sections 3–10
give the contracts, alternatives and costs. Section 12 identifies decisions
and checks required before implementation acceptance.

The retained parts also need a positive justification. Keep native ownership,
mandatory inspection, independently selectable native controls, renderer-owned
GPU orchestration and explicit unsafe callback integration because they express
the actual responsibilities. Keep concrete local/movable owners because their
trust and auto-trait differences are meaningful, not because their names have
already shipped. Reconsidering them remains legitimate; section 9 compares the
generic alternative directly.

There is no source-only way to prove a globally “best” user experience. This
report supplies a preferred, concrete design and the trade-offs against which
it should be judged. An engine integration walkthrough can test those judgements;
an absence of memory-safety findings cannot substitute for that assessment.

## 2. Evidence, criteria and complete surface coverage

**Verified** denotes an identified source fact. **Historical observation**
denotes an existing record's bounded result, not a rerun. **Inference** and
**recommendation** denote architectural judgement. Future scenarios are design
stress cases, not claims about a usable future SDK.

Native fidelity, soundness, backend isolation and truthful support claims are
constraints. Within those constraints, evaluate alternatives in this order:

1. Make the ordinary workflow discoverable and explicit about its effects.
2. Reduce repeated inputs, competing settings and avoidable application glue.
3. Give realistic additions a place without inventing their native contracts.
4. Make remaining native/renderer obligations and failure effects visible.
5. Keep costs and public concepts proportionate to their benefit.
6. Account for migration, with low weight for preserving existing syntax.

This does not equate fewer types, shorter snippets or more generics with better
design. A named binding builder introduces a type and a pure error, but can make
resource roles clearer and future inputs easier to add. A synchronization guard
can shorten a call while hiding obligations it cannot enforce.

The following covers the public wrapper groups, including advanced APIs. Private
inline tests and every raw ABI declaration were not independently re-proved.

| Surface inspected | Source and conclusion |
|---|---|
| Manifests, modules and reexports | [Workspace][workspace], [wrapper manifest][manifest], [sys manifest][sys-manifest], [crate root][lib], [upscaler root][upscaler]: retain two crates and backend isolation; choose clearer canonical paths |
| Local and movable runtime, load errors | [Runtime][runtime], [movable Runtime][movable-runtime], [state][runtime-state]: retain independent library ownership; reorganize effect setup and acquisition contracts |
| Pre-create provider, quality, size, jitter, memory and resource queries | [Routing][routing], [native queries][native-query], [utilities][native-utilities]: bind arguments once; keep real native calls and fresh explicit-ID membership checks |
| Live provider, memory, requirements and quality/size/jitter queries | [Local owner][dx12], [movable owner][movable]: retain actual-context routing and explicit native operations |
| Creation profile and checked scalar/enum values | [Common][common], [reactive controls][reactive-options], [sharpening][sharpening]: preserve native meanings; improve composition, placement and naming |
| Frame and GPU resources | [Frame/resources][frame], [inspection][inspection], [descriptor][descriptor]: make portable parameters canonical and resource roles named; retain compulsory inspection |
| Reactive-mask generation | [Generator][generator], [mapping][reactive-mapping]: apply the same three-part recording shape; preserve its separate resources, region and ordering obligations |
| Tuning | [Common values][common], [Configure][tuning], [recovery example][recovery-example]: use staged checked requests; retain nontransactional native calls |
| Failure state and teardown | [Eligibility][eligibility], [native owner][owner], [recovery guidance][recovery]: expose known usability; preserve terminal cleanup and failure retention |
| Callbacks and diagnostics | [Interop][interop], [diagnostics][diagnostics], [D040][d040], [verification][callback-record]: retain an explicitly unsafe native integration boundary |
| Error taxonomy and value results | [Errors][errors], [query values][query]: separate input rejection from dispatch, improve operation attribution and metadata construction |
| Optional provider-name resolver | [Resolver][resolver], [fixtures][resolver-tests], [example][provider-example]: keep policy optional, improve public testability and selection composition |
| Raw ABI and loader | [Raw root][sys-lib], [API][raw-api], [upscale ABI][raw-upscale], [DX12 ABI][raw-dx12], [loader][loader]: no ergonomic redesign of the C ABI; keep it faithful and distinct |

Current architecture context includes [D013][d013], [D014][d014], [D015][d015],
[D016][d016], [D028][d028], [D029][d029], [D031][d031], [D032][d032],
[D033][d033], [D034][d034], [D036][d036], [D040][d040] and [D041][d041].
They establish the baseline and explain costs; their acceptance does not make
their public syntax the optimum under this review's changed priorities.

External sources accessed on **2026-10-26**:

- Pinned AMD [common API][amd-api] and [upscaler header][amd-upscale]: distinct
  native operations, provider override and frame/creation controls. They do not
  establish closed-provider safety, callback shutdown or universal capabilities.
- Rust API Guidelines on [type safety/builders][rust-types],
  [predictability][rust-predictability] and [future proofing][rust-future]: useful
  design considerations, not a mandate to use a particular construction style.
- Rust Reference on [non-exhaustive types][rust-non-exhaustive] and
  [special traits][rust-traits]: construction and trait commitments matter in
  addition to private fields or non-exhaustive variants.
- [wgpu v27.0.1 render-pipeline source][wgpu]: a primary example of named
  descriptor fields. Its different validation/ownership model is not evidence
  that FSR can expose safe recording or copy its entire descriptor design.

Attempts to access versioned ash documentation/source did not yield readable
source in this investigation; no recommendation relies on those failed fetches.
Unversioned guideline/reference pages are access-date snapshots, not release pins.

## 3. U1: make planning and creation one discoverable setup scope

### Verified friction and why R1 alone is insufficient

The current [provider example][provider-example] repeats selection branching
for ratio/render-size and phase/offset queries, then supplies the same selection
through a different route for requirements, memory and creation. Runtime method
names repeat the effect and backend because neither is represented by a receiver.
The first audit's optional-provider argument removes branching but still passes
device/provider separately and leaves creation selection stored elsewhere.

**Inference:** the unit of caller intent is “work with this runtime/device,
this effect and this provider request while planning and creating contexts”.
A borrowed setup object represents that intent directly. It is core API
composition under D033, not a provider-selection policy or renderer service.

### Preferred public shape

All code below is **illustrative proposed Rust**, not implemented or compiled.
Names form one coherent target for review; they are not existing APIs.

```rust
let mut runtime = unsafe { Runtime::load(loader_path)? };
let mut setup = unsafe { runtime.upscaler_dx12(device) };
let providers = setup.providers()?;
let selection = select_provider(&providers, choice)?;
let mut setup = setup.with_provider(selection);

let render_max = setup.render_size_for_quality(output_max, QualityMode::Quality)?;
let config = upscaler::Config::new(render_max, output_max);
let requirements = setup.resource_requirements()?;
let memory = setup.estimate_memory(&config)?;
let mut context = setup.create(&config)?;
```

Proposed essential signatures for the local family, in signature notation
rather than complete Rust `impl` blocks:

```text
// Loader and effect acquisition duties are divided in section 9.
unsafe fn Runtime::upscaler_dx12<'a>(
    &'a mut self, device: &'a ID3D12Device,
) -> dx12::UpscalerSetup<'a>;

fn UpscalerSetup::with_provider(self, selection: ProviderSelection) -> Self;
fn UpscalerSetup::providers(&mut self) -> Result<Vec<ProviderInfo>, Error>;
fn UpscalerSetup::ratio(&mut self, mode: QualityMode) -> Result<f32, Error>;
fn UpscalerSetup::render_size_for_quality(
    &mut self, output: Extent2D, mode: QualityMode,
) -> Result<Extent2D, Error>;
fn UpscalerSetup::jitter_phase_count(
    &mut self, render: Extent2D, output: Extent2D,
) -> Result<JitterPhaseCount, Error>;
fn UpscalerSetup::jitter_offset(
    &mut self, index: i32, phases: JitterPhaseCount,
) -> Result<JitterOffsetPixels, Error>;
fn UpscalerSetup::resource_requirements(&mut self) -> Result<ResourceRequirements, Error>;
fn UpscalerSetup::estimate_memory(&mut self, config: &Config) -> Result<GpuMemoryUsage, Error>;
fn UpscalerSetup::create(&mut self, config: &Config) -> Result<Upscaler, Error>;
unsafe fn UpscalerSetup::create_with_callbacks(
    &mut self, config: &Config, callbacks: CreationCallbacks,
) -> Result<Upscaler, Error>;
```

Use `ProviderSelection::{Default, Explicit(ProviderId)}` as the named public
request. It makes the default's meaning readable without an argument-name hint;
the cost is a small enum rather than `Option`. The resolver would return this
type, retaining its current parsing/ambiguity policy. Do not put FSR generations
or name heuristics in the core selection enum.

**Contract choices:**

- The view holds an exclusive CPU borrow of the runtime and a device borrow.
  Constructing it or changing its requested selection performs no native call.
  `with_provider` consumes the old view; there is one stored selection, with no
  hidden runtime-global setting or independent provider field in `Config`.
- A view defaults to `Default`; rebinding is an explicit operation on the view.
  All subsequent setup queries, estimates and creates read that stored request.
  Enumeration itself remains a device/effect snapshot, not a selected query.
- Each explicit native operation still performs fresh membership validation;
  applicable pure input checks precede enumeration. Preserve default descriptor
  chains, native errors, no fallback and explicit-create live identity checking.
- `create(&Config)` snapshots configuration and selection into the new owner.
  It independently retains runtime/device/storage. Its return type does **not**
  borrow the view. Several contexts can be created and remain usable after
  dropping the view and public runtime, under their lasting contracts.
- Retained selection is requested identity, not a reserved provider or capability
  token. `Default` can resolve differently at different invocations. Rebinding a
  view does not reconfigure existing contexts or relabel earlier query results.
- The movable family receives a matching concrete setup view and return owner.
  No implicit local-to-movable conversion or feature-dependent auto traits.
- A view's exclusive borrow serializes its own calls only. Sibling contexts,
  independent acquisitions, native workers and external users still require
  common-domain coordination; the view is not a synchronization primitive.

This deliberately replaces the flat setup methods as the canonical public
route. Keeping eight old methods as equally prominent alternatives would dilute
the improvement. Migration aliases can be temporary; their existence is not a
design prerequisite under the maintainer's instruction.

### Alternatives assessed

| Alternative | Usability and growth trade-off |
|---|---|
| Flat methods with one optional provider argument | A good minimal patch, but repeats bindings and grows Runtime for each effect/backend; not preferred here |
| Device-bound scope without creation | Shorter query names, but leaves selection duplicated between scope and options; incomplete |
| Borrowed scope including creation | Preferred: covers the existing workflow without a new native lifetime |
| Owning factory holding cloned runtime/device references | Useful for a concrete long-lived factory consumer, but adds owned dependencies and transfer questions to a setup-only need |
| Immutable provider capability/session token | Misleading unless explicitly only a request binding; enumeration cannot establish durable capability |
| One generic backend/effect factory | Requires contracts for unimplemented backends/effects and makes present signatures more abstract |

Historical [setup timings][setup-record] make unnecessary repeated queries a
real integration concern, but this redesign promises no speedup. The measured
roughly 230/697–707 ms paths are six-query historical sequences, not isolated
enumeration cost or current benchmarks. A setup view must not silently cache
membership or query results. Any performance change needs a separate contract
and measurement; a faster-looking API is not evidence of faster native calls.

## 4. U2/U3: make frame data and named GPU bindings the primary recording API

### The present boundary is on the wrong side of frame composition

**Verified:** [UpscaleDispatch][frame] owns required jitter/time/camera/reset,
optional frame controls, a command-list borrow and a resource bundle. Required
values have getters but no setters. New frames restore all optional defaults.
The same-typed colour/depth/motion/exposure/output arguments in the resource
constructor are positional. No assembly step checks their semantic roles.

**Inference:** this shape makes consumers either invent another portable frame
struct and replay setters, or pull Windows/DX12 types into their preparation
functions. Extensibility is more than being able to add another setter: the
extension should have an obvious portable place and not add another piece of
application copying code for each backend.

### Preferred shape and realistic ordinary frame

```rust
let mut frame = upscaler::FrameParameters::new(
    jitter, frame_time, camera, HistoryAction::Continue,
);
frame.set_pre_exposure(pre_exposure);
frame.set_motion_scale(MotionVectorScale::Pixels);
frame.set_sharpening(Sharpening::Native(sharpness));

let resources = dx12::UpscaleResources::builder()
    .color(&color)
    .depth(&depth)
    .motion_vectors(&motion)
    .exposure(dx12::ExposureInput::Texture(&exposure))
    .output(&output)
    .build()?;

// Optional early metadata diagnosis; normally omit this immediately before record.
context.validate_upscale_inputs(command_list, &resources, &frame)?;
unsafe { context.record(command_list, &resources, &frame)?; }
// The renderer still closes, submits once, orders, retains and fences the work.
```

The preferred canonical operation has this signature:

```text
unsafe fn Upscaler::record(
    &mut self,
    list: &ID3D12GraphicsCommandList,
    resources: &UpscaleResources<'_>,
    frame: &FrameParameters,
) -> Result<(), Error>;
```

The old consuming `UpscaleDispatch` is unnecessary as the primary public
assembly type. A private temporary aggregate may reuse existing inspection and
descriptor code. Removal is justified by a clearer composition boundary; adding
a second equally canonical carrier/dispatch route would keep avoidable ceremony.

`FrameParameters` has private fields, `Debug + Clone`, checked required inputs
and complete getters/setters, including jitter, time, camera and history action.
It has no fabricated `Default` camera/time/jitter. Required choices are explicit
at construction. Reusing a value across frames is allowed, and **does not**
automatically advance jitter, clear reset or update time. The ordinary example
must demonstrate those updates; otherwise reuse could preserve a first-frame
reset unintentionally. `HistoryAction::{Continue, Reset}` replaces the trailing
positional bool at this boundary only. It adds no automatic history policy.

Keep all existing optional defaults and native scalar domains. Parameter setters
accept checked values and change local data only. Context pairing, active regions
and provider/resource eligibility remain recording/preflight checks. Borrowing a
parameter value does not certify that its pixels, camera or native domain match.
Do not promise `Copy` or `Eq` for the evolving aggregate; small checked scalar
types can retain their independently justified traits.

### Named resource construction and its deliberate cost

The builder stores required role borrows until `build` and returns a typed pure
missing-binding error if a required role was omitted. Exposure must be selected
explicitly as `ExposureInput::Texture(resource)` or `ExposureInput::Automatic`;
absence of the builder call is not an accidental choice of automatic exposure.
Optional masks default to absent and have named methods. The built value owns no
COM reference, performs no COM/FFX calls and promises no device/format/state
validity. Canonical recording still runs every mandatory inspection.

Named methods make accidental same-type argument swaps less likely to escape
visual review; they do not make semantic swaps impossible or validate content.
Automatic exposure must still match the context's mode. Do not infer defaults
from a live context during pure construction or relax actual-provider duties.

The additional builder and fallible assembly are real costs. They buy named roles
and allow optional resources to be introduced without another constructor or
mandatory struct-literal field. A straightforward runtime builder is preferred
over a long public typestate parameter list. Missing bindings are caught before
native work and can be checked while resolving a render graph.

Public literal descriptors are a reasonable alternative for passive data, as
[wgpu's named fields][wgpu] demonstrate. They are less attractive here when
new optional inputs would force literal updates. A non-exhaustive public struct
cannot simply be constructed externally with a literal; [the Rust Reference][rust-non-exhaustive]
makes that limitation explicit. Prefer private extensible storage and named
assembly rather than claiming all these properties simultaneously.

### Apply the same shape to generation without flattening the operations

Generation currently [combines][generator] command list, three resources,
active region and options. Use portable `ReactiveMaskParameters` for active
region plus `ReactiveMaskOptions`, a named DX12 `ReactiveMaskResources` builder,
and:

```text
unsafe fn Upscaler::record_reactive_mask(
    &mut self,
    list: &ID3D12GraphicsCommandList,
    resources: &ReactiveMaskResources<'_>,
    parameters: &ReactiveMaskParameters,
) -> Result<(), Error>;
```

Keep a matching `validate_reactive_mask_inputs` metadata operation. Both families
share implementation. Allow the two input colours to alias where the current
contract permits it; output distinctness and reviewed role/view rules remain.
Keep generation → renderer UAV ordering/transition → upscale explicit. Neither a
common call shape nor OK return establishes generator support or output writes.
The [historical provider results][generator-record] remain mixed.

## 5. U4: represent complete effective intent in portable values

### Motion and sharpening

Retain the first audit's `MotionVectorScale::{Uv, Pixels, Native(scale)}` and
portable `NativeMotionVectorScale`. There is one stored selection. UV derives
from active render/display-motion geometry, Pixels maps to `(1, 1)`, and Native
forwards accepted bits independently of active-size changes. Resolution and
jitter inclusion remain separate creation axes. No pixel/NDC conversion occurs.

Remove both old units and override accessors from the canonical frame API. An
old units getter cannot report `Native(scale)` faithfully. Keeping dormant units
only to answer it would recreate the state model being removed. Temporary
overrides require caller-owned save/restore of the prior enum value. Retain
signed zero, subnormal and all accepted finite native-scale bits; add no new
provider-safe numerical claim or unreviewed domain restriction.

Move `NativeSharpness` and `Sharpening` to portable upscaler code. Keep disabled
distinct from enabled zero, and retain direct native semantics. There is no
reason for CPU settings preparation to require a GPU backend. Whether old paths
temporarily reexport the same types is a migration choice, not a reason to keep
the canonical definitions backend-gated.

### Colour permission should travel with the hint

**Verified:** the ordinary [colour-hint setter][frame] clears the explicit
context-mismatch permission; the alternative setter sets it. The getter for the
hint alone does not describe that full request. This is intentional and more
predictable than motion's dormant fallback, but portable plans must still know
which of two setters conveys their intended permission.

Prefer one `ColorEncodingRequest` value containing the hint and pairing rule,
with explicit constructors such as `checked(hint)` and
`allow_context_mismatch(hint)`, one frame setter and getters for both components.
Its default is checked Unspecified. This preserves the existing explicit bypass
of only the pairing rule; it neither changes colour space nor certifies provider
support. Avoid folding HDR, input classification, exposure and frame hint into
one preset: the native controls are independently meaningful.

Other existing optional values do not need ceremonial replacement. `None` for
an active region naturally means the receiving context's maximum; an absent
optional mask naturally means no supplied mask. Keep booleans where names make
their independent meaning clear. The extra history enum is justified at the
positional required-input boundary, not as a rule for every boolean.

## 6. U5: use checked tuning requests as composable public data

**Verified:** [request_desired_at_idle][recovery-example] loops over raw
parameter/value pairs. [Configure][tuning] validates each pair immediately before
that native call. The first valid request can reach native code before a later
known-invalid request is rejected. The five parameter domains already exist;
there is no public checked pair for offline staging.

The first audit's pure validator helps, but a checked request is the stronger
composition boundary when source compatibility has low weight:

```rust
let desired = [
    TuningRequest::new(TuningParameter::VelocityFactor, TuningValue::Value(0.5))?,
    TuningRequest::new(TuningParameter::AccumulationAddedPerFrame, TuningValue::Value(0.25))?,
    TuningRequest::new(TuningParameter::ShadingChangeScale, TuningValue::ProviderDefault)?,
];
// Construct the complete list before entering the renderer-established idle interval.
for request in &desired {
    context.request_tuning(request)?;
}
```

`TuningRequest` is portable, owns a parameter and checked value, and has private
fields and accessors. Construction performs no native call. Canonical requests
use the same validation routine; mandatory entry validation and poison precedence
remain. There is no optional unchecked/checked sibling API, and no requirement
for five separate float wrapper types. Preserve exact accepted bits and explicit
ProviderDefault rather than substituting numerical header defaults.

This makes request lists useful as editor state, utility outputs and recreation
inputs without duplicating parameter ranges. It also changes D029's signature
deliberately. The reason is the concrete staged-list use, not abstract type purity.

Native calls remain ordered and nontransactional. A native error can occur
halfway through, poisons the context, and leaves earlier acknowledgements
unrolled back. Constructing a checked list proves neither GPU idleness nor
application/support. A batch convenience may later report failed index/completed
count, but no batch method or rollback framework is needed for this redesign.

## 7. U6: expose the unusable state the renderer already knows

**Verified:** [Eligibility][eligibility] stores usable/dispatch-failed/
Configure-failed internally. The [recovery example][recovery-example] must keep
an abandoned-but-OK context in an `Option` until list invalidation and prior GPU
completion succeed. The wrapper cannot observe abandonment and continues to
consider that context usable; avoiding later calls is an application convention.

Recommend a pure `usability()` snapshot and a safe local
`mark_recording_abandoned(&mut self)` transition. Use names such as
`ContextUsability::{Usable, Unusable(cause)}`, never `Idle` or `Ready` for a state
that says nothing about GPU completion. Causes include native upscaling error,
native generation error, tuning error and explicit abandonment. Preserve the
first terminal cause; repeated marking cannot erase the original native error.

The transition makes no native call, releases nothing, waits for nothing and
revives nothing. Once marked, reject recording, live queries, tuning and
context-dependent preflight, with local status/configuration access and terminal
destruction available under their existing duties. Destruction must still wait
for the renderer to establish the required completion/invalidation conditions.

This is useful even though it cannot detect submission: the renderer can report
an event it already knows and the owner can enforce no further ordinary use.
All affected contexts on an abandoned list must be marked; the wrapper has no
list-to-context registry. A successful recording token dropped without a submit
is not a solution: it cannot prove GPU/list invalidation, handle forgotten
tokens or safely perform fallible waiting in Drop.

Keep native-error terminality, single-attempt consuming destruction and exceptional
dependency retention. Do not change cleanup into a retry, automatic wait or an
apparently recoverable reset. Exposing usability adds useful state modelling
without asserting a new provider recovery capability.

## 8. U7: make discovery, testing and diagnostics fit the design

### 8.1 Canonical modules and names

Choose one canonical path per concept. A coherent proposed layout is:

```text
fsr_sdk
  runtime::{Runtime, MovableRuntime, LoadError}
  types::{Extent2D}
  provider::{ProviderId, ProviderInfo, ProviderSelection}
  diagnostics::{DebugLevel}
  error::{Error, InputError, NativeError, ...}
  upscaler::{Config, FrameParameters, ReactiveMaskParameters,
            CameraParameters, JitterOffsetPixels, FrameTimeMillis,
            PreExposure, NativeSharpness, Sharpening, MotionVectorScale,
            NativeMotionVectorScale, ColorEncodingRequest,
            TuningRequest, TuningParameter, TuningValue,
            QualityMode, JitterPhaseCount, ResourceRequirements, GpuMemoryUsage, ...}
    dx12::{UpscalerSetup, MovableUpscalerSetup, Upscaler, MovableUpscaler,
           UpscaleResources, ReactiveMaskResources, ExposureInput, interop, ...}
  utils::provider::{FsrChoice, SelectionError, select_provider}
```

`Extent2D` has only the current nonzero-width/height invariant. Moving this known
primitive to a neutral path now does not assert that future effects share camera,
history or dispatch contracts. Keep upscaler camera and temporal parameters
effect-specific. Provider IDs remain effect/device/runtime-dependent requests;
a neutral data module grants no cross-effect interchangeability guarantee.

Make portable effect data directly discoverable under `upscaler`; `common` need
not remain the primary public concept. Keep DX12 owners/resources under an
explicit backend path. Do not rebind a root `Upscaler` alias to whichever backend
feature is enabled. New consumers should not have to learn legacy reexports.

Use `max_output_size` consistently with `active_output_size`; rename
`InputColorSpace` to `InputColorClass` to express Linear/NonLinear classification
without implying a gamut. `Config` versus `UpscalerOptions` is a secondary naming
choice: the consequential changes are the receiver, data and ownership boundaries.
Do not let a broad naming pass consume the redesign without changing those.

For `Config`, require both maxima, keep private fields and allow named setters
to return `&mut Self`. Memory estimation and creation borrow the same config;
creation retains its own snapshot. Changing a config cannot reconfigure a live
context. Keep defaults as supported native-profile defaults, not quality advice.

Existing checked scalar domains are mostly appropriate. Retain them and clear
unit names. A checked `FrameTimeMillis::try_from(Duration)` is a reasonable small
addition with documented conversion/rounding and zero rejection; raw-millisecond
access remains. Camera construction still needs explicit units and finite/infinite
profile. A new generic camera framework is unjustified by these inputs.

### 8.2 Provider IDs are data, not unforgeable authority

**Verified:** `ProviderId` is a private `u64`; `ProviderInfo` has private ID/name
fields and no public assembly route. The [resolver fixtures][resolver-tests]
therefore live inside the query privacy boundary. Real selection checks
[freshly compare the numeric ID][native-query] against the actual runtime/device;
creation additionally checks effective live identity.

The first audit describes a raw constructor as risking a capability illusion.
That is a documentation problem, not a reason to deny pure construction when
the ID is already explicitly not a capability. Recommend safe, lossless
`ProviderId::from_raw(u64)` / `as_raw()` data conversion and
`ProviderInfo::new(id, name)` metadata assembly. They establish no provenance,
native membership, hardware support or validity interval. A zero integer is not
the Default selection; explicit operations still validate membership and the
existing live-zero invariant remains. Do not bypass those checks for any ID.

This allows downstream tests of the actual public resolver, custom selection
policies and round-tripping a preference without loading a DLL. A persisted ID
still needs fresh resolution and can be reused by native code. Display names
remain labels, not a stable version schema. Synthetic metadata must be described
as application-constructible; never advertise all `ProviderInfo` as authenticated
runtime output after adding this API.

An alternative is a separate public name-classification function returning a
generation, leaving metadata opaque. It is sufficient for testing only the name
parser, but not downstream selection/ambiguity and policy composition against
the actual public resolver. A generic provider trait is more machinery than
plain data constructors for this need. If the project later wants a provenance
token, give it a different type with an enforceable contract rather than treating
this integer wrapper as one retroactively.

Add `BitOr`/`BitOrAssign` and an empty value to the reported `ResourceFlags` mask
so multi-resource predicates are straightforward; retain every unknown native
bit. Additional bitwise operations should have explicitly documented semantics.
Do not turn resource-query bits into an allow-list or capability certificate.

### 8.3 Error contracts should describe input and operation, not historical layout

**Verified:** [Error][errors] calls pure camera/exposure construction failures
`InvalidDispatch`, groups both GPU operations under `Operation::Dispatch`, and
groups tuning/global diagnostics under Configure. Its main enum promises
`Copy + Eq`. These choices make aggregate diagnosis less informative and extend
an upscaler/dispatch vocabulary into otherwise portable input construction.

Prefer three distinct kinds of detail, composed into one owned public Error:

- **InputError:** scalar field/domain, dimensions, missing resource binding,
  context/frame pairing and query/tuning domain failures. Scalar construction
  does not pretend a dispatch exists. Recording rejection can add its operation
  context while retaining the underlying input reason.
- **NativeError:** exact code and the particular operation, such as record
  upscale, record reactive mask, query jitter, request tuning or global debug.
  This small value can remain `Copy + Eq`; it can also be retained as the first
  unusable cause. Do not erase unknown native codes.
- **Backend/invariant/unusable details:** exact HRESULT and resource role where
  applicable, native success-output violations and the stored terminal cause.

Main `Error` should guarantee `Debug`, `Display` and `std::error::Error`, without
a blanket `Copy` or `Eq` contract. Preserve matchable categories and source chains;
small reason/code values keep equality where meaningful. This leaves room for
owned diagnostics or future backend causes without forcing all errors into
small integer payloads. It requires no success-path allocation. The [Copy
restriction][rust-traits] applies to every future variant, so non-exhaustiveness
alone does not provide that freedom. Removing the trait now is a deliberate
extensibility choice, not a claim of a present soundness bug.

The separate owned LoadError remains justified by path/source information. Avoid
a universal string error, arbitrary dynamic errors at every boundary or a guessed
effect taxonomy for effects not implemented. The implemented upscaler operation
enum can grow non-exhaustively and later sit inside an effect-specific category.
Remove the dead `UnsupportedResource` rejection variant. Make format-query role
attribution consistent, including the older RG32F route.

## 9. U8 and retained ownership/safety choices

### 9.1 Separate loader acquisition from effect integration assumptions

**Verified:** [Runtime::load_unchecked][runtime] loads a library that
[RuntimeState][runtime-state] retains, but its public safety contract also
contains upscaler construction, numerical query, generation and Configure
assumptions. Adding each future effect to that contract would make every loader
acquisition responsible for operations it may never expose to the application.

In the target, rename it to unsafe `Runtime::load` and scope it to trusted code,
ABI/export validity, initialization, partial failure/unload and common native
loading/termination/thread obligations. It is not a safe loader or a check that
certifies an arbitrary DLL. The unsafe `upscaler_dx12` setup acquisition carries
the lasting effect/backend assumptions needed by its later safe queries,
construction and derived owners. Those cover all actual providers subsequently
used through that scope, including rebinding and SDK defaults.

Callbacks and global diagnostic installation retain their separate explicit
unsafe contracts. Recording retains its resource/content/access/completion
contract. Derived owners retain the effect assumptions after the setup view is
dropped; a lexical lifetime of the view does not terminate them. Moving work to
this boundary does not prove any closed-provider guarantee or reduce the total
caller obligations. It makes them selectable and reviewable by effect.

The cost is another explicit unsafe acquisition step in the ordinary workflow.
It is preferable for a growing SDK to silently extending one universal load
contract. Before implementation, audit **every** surviving safe method: none may
invoke effect-specific native work on a merely loaded Runtime without the
corresponding established trust. Audit Drop/unwind, failed setup/create and
retained bundles too. A rename or documentation move by itself is insufficient.
This change needs an explicit contract decision extending D004/D008/D018/D022/
D028/D029/D030; no existing safety precondition is relaxed by this report.

### 9.2 Concrete local/movable owners versus a public ownership generic

A serious alternative is `Runtime<Mode = Local>` and
`dx12::Upscaler<Mode = Local>`, with sealed Local/Movable modes and specialized
unsafe acquisition. Advantages: one method surface, generic downstream algorithms
over ownership mode, and fewer repeated docs as effects grow. Default type
parameters/aliases could keep ordinary call sites short. These are real benefits;
“already accepted” and “generics are complicated” are insufficient rebuttals.

The cost is a public ownership-mode contract and constrained storage/trait design.
The current implementation [deliberately audits Send only on concrete owners][movable],
while its generic native owner stays nontransferable. A public mode abstraction
must preserve that exact distinction through derived setup/context types, avoid
leaking private retention internals and not permit downstream modes or conversions
to bypass arbitrary-destination native trust. Type defaults do not solve those
requirements. Using Arc everywhere merely to simplify the generic introduces a
different storage/cost choice; it is not automatically free.

**Preference:** retain concrete local/movable owners in the proposed package,
with shared private implementation and consistent public contract generation
where worthwhile. The main consumer gains above do not depend on mode generics.
This is a judgement about keeping the unusual thread-trust choice explicit while
leaving ordinary effect parameters simple. It is not a compatibility veto.
A concrete consumer needing one generic algorithm over both modes would shift
the balance; a sealed mode design is a legitimate option, not forbidden research.
Do not make one owner always Send or change auto traits when a feature is enabled.

### 9.3 Keep native limits visible while improving usability

The [threading synthesis][threading] and [callback evidence][callback-record]
describe shared native state beyond any one owner. A setup scope, mutex attached
to one runtime, effect generic or callback RAII object cannot establish global
coordination or quiescence. Raw callbacks therefore remain under explicit interop,
paired allocation/free groups, externally owned targets and unsafe installation.

Do not use captured closures, inferred per-context callback identity, TLS current
context or automatic unregister-on-Drop. Their missing native premises are the
reason to reject them now, independent of whether their syntax breaks an API.
Useful future safe adapters require actual invocation/shutdown contracts and
must continue to expose the native route.

Preserve distinct creation flags, frame debug view, message level and modern/
legacy diagnostics. A unified “debug mode” preset must not replace independent
controls. Preserve callback-bearing failure retention and ordinary cleanup's
separate accepted policy. No closure wrapper can reclaim external targets merely
because the installing context was destroyed successfully.

## 10. Test the design against future additions

These are source-level extension walkthroughs, not compiled implementations or
promises that every future native change will be nonbreaking.

| Addition or consumer | How the target accommodates it | Boundary that must still be reviewed |
|---|---|---|
| New optional per-frame scalar | Checked portable value plus FrameParameters field/setter; record signature stays the same | Default/native mapping and supported domain; aggregate trait contracts |
| New optional frame resource | Named builder method and private resource storage; no positional constructor change | Required presence, role/view inspection, GPU/native safety |
| New mandatory native input | New profile/version or explicit missing-input rejection, with a migration if needed | A builder cannot invent a correct default or guarantee a nonbreaking change |
| Future Vulkan upscaling | Concrete Vulkan setup/resources/recording plus genuinely shared portable parameters | Actual command buffers/images/layouts/device loading, fields that differ, ownership and native evidence |
| Another SDK effect | Effect-specific setup, config, parameters and owner; neutral Extent2D/metadata/error mechanisms reused where valid | Creation, history, shutdown/presentation and failure contracts can differ |
| CPU settings editor or stateful utility | Build Config, FrameParameters and checked tuning requests without DX12 | Keep creation/frame/idle Configure phases separate; no policy provenance in core |
| Downstream provider policy tests | Public synthetic ProviderInfo/ID values exercise the actual resolver without a DLL | A synthetic or persisted ID proves nothing about later native selection |
| Renderer changes command list/resources each frame | Borrow current list/resources at recording; portable frame data has no GPU lifetime | Renderer must update actual state, content and frame-dependent values |
| Renderer discards an OK recording | Mark every affected context unusable; retain it through invalidation/completion and destroy | No detection, rollback, resurrection or automatic wait |
| Future owned backend diagnostics | Main Error can own context/cause; small NativeError remains inspectable | Public trait/source compatibility and preservation of exact native codes |

Concrete backend modules are preferable to `Upscaler<B>` today because no usable
new native contract was inspected here, not because a generic would be a breaking
change. If future code demonstrates an invariant set shared by backends, introduce
a narrower trait then. A renderer choosing a backend dynamically may still need
its own enum; that is not evidence for type-erasing every resource in this crate.

Vulkan is independent of operating system. Portable data must compile without
Windows/DX12 dependencies; native code/types stay gated. Keep the reserved Vulkan
feature's current deliberate failure under amended D031. No functional Vulkan
or SDK-baseline claim follows from this review, and no placeholder backend is
needed to reserve the proposed extension boundaries.

No quality algorithm, provider recommendation table, automatic motion conversion,
submission service or policy framework is introduced. Native operation modelling
and representational convenience remain core under D033. Optional independently
specified policies remain in utils; quality-policy work remains deferred under
the [project scope][scope].

## 11. Concrete workflow comparison and baseline delta

| Workflow | Current API | First audit's preferred package | This report's preferred target |
|---|---|---|---|
| Choose a provider, query sizes/memory, create | Repeated branches/arguments and separate selection storage | Uniform optional arguments; creation still separate | One selected setup view covers queries, estimate and creation |
| Prepare a frame in portable engine code | Application carrier plus replay, or backend-coupled preparation | Sharpening portable; frame carrier optional | Canonical FrameParameters with complete updates |
| Bind five DX12 resource roles | Positional same-typed constructor | Retained | Named builder, explicit exposure choice, pure missing-role error |
| Record one frame | Construct and consume a combined dispatch value | Retained; optional alternate assembly | Borrow list, resource bindings and parameters separately |
| Apply ordinary motion after a custom scale | Clear override and set units in the right combination | One enum | One enum, with old getters/setters deliberately replaced |
| Stage several tuning values | Duplicate domains or permit later local rejection after earlier native calls | Optional pure validation method | Checked request values are the canonical request data |
| Recover after abandonment | External Option/convention retains owner but wrapper still reports usable | Documented workflow; invalidation deferred | Explicit local unusable transition plus existing external cleanup |
| Test provider policies outside the crate | Nonempty fixtures need native IDs/private construction | Acknowledged friction, retain opacity | Application-constructible metadata and integer IDs; fresh native checks remain |
| Add another effect | Extend flat runtime API and potentially universal load contract | Effect-specific modules later | Effect setup boundary and trust acquisition already have a shape |

### Reassessment of earlier recommendations

- **R1:** correct diagnosis, insufficient target. Prefer a provider-bound setup
  view including creation over only normalizing arguments. A named selection
  enum earns its place in the cohesive target.
- **R2:** keep the single choice. Settle the complete accessor replacement and
  exact mapping now rather than leaving ambiguous compatibility shims.
- **R3:** keep portable sharpening, but make all portable frame data a canonical
  integration boundary, not just correct one misplaced type.
- **R4:** keep documentation/dead-variant fixes. They are necessary accuracy work,
  not an answer to the broader usability/extensibility question.
- **R5:** promote and strengthen. Replace the combined dispatch as the ordinary
  shape instead of layering an optional carrier onto it indefinitely.
- **R6:** prefer a checked request pair, changing the request signature. The
  staged-list workflow already supplies a concrete consumer rationale.
- **R7:** prioritize named resource assembly, useful error attribution, public
  pure metadata construction and explicit unusable state. They should not all
  wait for an unspecified future aggregate consumer.

The earlier response's statement “no blocking design errors; take R1–R4” was
reasonable as a narrow safety/consistency review, but too weak as a judgement
of the preferred caller experience. Likewise, “additive later” is not a positive
reason to postpone a structural improvement when a break is inexpensive now.

This record does not supersede accepted ADRs by itself. It explicitly recommends
amending several of their public shapes, while confirming much of their native
ownership and safety rationale. The old audit and the first review remain
historical inputs; their conservative priorities are not carried forward as
requirements.

## 12. Decisions, implementation sequence and verification boundaries

The design still needs maintainer acceptance. “Breaking changes allowed” removes
a major constraint; it does not automatically accept every sketch or authorize
implementation in this source investigation.

Recommended review/implementation units, after acceptance:

1. **Accept the target and trust distribution together.** Decide U1/U8 as one
   coherent setup design: selection stored once, creation included, contexts
   outlive views, loader/effect assumptions allocated explicitly. Amend the
   affected acquisition/provider/ownership decisions; do not implement only a
   renamed flat API and call the design complete.
2. **Implement portable data and recording assembly.** U2–U4, named bindings,
   generation symmetry and canonical paths. Reuse production inspection and
   descriptor mapping; settle required-input update methods and all defaults.
3. **Implement staged requests and usable-state reporting.** U5/U6 with one
   validation routine, original-cause retention and no implicit cleanup/wait.
4. **Reconcile metadata, errors, naming and migration.** U7, source consumers,
   examples and Rustdoc. Prefer one canonical route over permanent parallel
   old/new facades. Reexport-only aliases may ease migration without duplicating
   state or semantics. Wrapper changes alone do not require a sys version bump.
5. **Verify the resulting package pair.** Keep final archive verification
   separate from feature/source checks under the existing release preparation.

Validation proportional to these changes should cover:

- External consumers using setup selection for default and explicit planning,
  estimates and creation; invalid-input-before-enumeration order, fresh ID
  membership, native refusal and effective-provider mismatch. Confirm contexts
  survive the setup/public runtime and never borrow the setup lifetime.
- Loader-only failure/Drop and every moved effect acquisition duty; no safe
  effect entry without trust, including diagnostics and failure cleanup. Local
  remains nontransferable; movable/setup/owner field graphs require explicit
  Send/Sync audits on Rust 1.88 and supported features.
- Portable parameter/resource-builder assembly, required-role omissions,
  explicit automatic/texture exposure, required frame updates and default/native
  descriptor equivalence. Both recording routes must recheck after preflight.
- Motion mode transitions, active render/display geometry and literal scale
  bits; colour pairing permission and sharpening enabled-zero behaviour through
  both owners. Preserve reviewed formats, allocation sizes and view rules.
- Public pure resolver fixtures using assembled metadata; forged/stale IDs do
  not skip actual runtime/device membership; native codes/unknown mask bits stay
  intact. Do not describe data conversion as native validity validation.
- Entire tuning lists constructed before native mutation; parameter-domain and
  ProviderDefault mapping, entry poison precedence and native partial failure.
  Explicit abandonment stays terminal and preserves the first cause while leaving
  completion, command invalidation and cleanup with the renderer.
- Supported feature/no-default/consumer/MSRV configurations, deliberate Vulkan
  failure controls, relevant existing ABI/native regressions and exact final
  archive-native coverage. Compile sketches as real external consumers during
  implementation; do not infer their validity from this document.

Do not turn this into a new broad native research campaign, image-quality gate
or generic concurrency project. Investigate concrete mapping/safety failures if
they arise. Existing callback/generator/provider limitations survive renaming and
data regrouping. Measured startup, allocation or hot-path improvements require
their own evidence; none is claimed here.

## 13. Limits, provenance and record checks

The preferred target is based on source reasoning and repository-owned caller
examples. It has not been compared in a user study, implemented, compiled or
timed. In particular, builder inference/diagnostics, the borrowed setup ergonomics,
movable view traits and migration effort remain implementation checks, not passed
results. Public mode generics remain a viable competing design with different
costs; no claim of a universally optimal Rust API is made.

No new ABI, GPU support, provider capability, callback quiescence, recovery,
performance or shader-access safety was established. The current native-platform
boundary, pinned SDK, dependency versions and distribution/licensing terms remain
unchanged. This report proposes no dependency or binary redistribution.

The source inventory records actual inspected checkout bytes and external source
identifiers. Historical execution records are used only for their explicitly
bounded observations. All findings needed to assess the recommendations are
stated here; neither Git history nor an unpublished prototype is required.

Only this new report and its adjacent input inventory are retained outputs.
Current topic syntheses, the research index, roadmap, ADR acceptance and production
code are not changed.

Record checks on 2026-10-26 resolved all 59 local reference targets and 87
reference-style uses against 67 definitions, checked balanced code fences,
UTF-8/LF, no trailing whitespace and the two new files' size limits. The final
inventory contains 97 source identities; all 95 earlier baseline identities
still match, with no mismatches. `git diff --check` passes for tracked files;
the new untracked report/inventory receive the explicit text checks above.
`python tools/verification/repository_size.py` passes at 41.49 MB. Temporary
authoring/check equipment remains under ignored `target/`. No Rust tests,
compiled design probe, native calls or GPU checks were run for this record.

## Source references

[inventory]: 2026-10-26-src-public-api-usability-and-extensibility-redesign/sources.json
[prior]: 2026-10-26-src-public-api-consistency-and-architecture-audit.md
[prior-inventory]: 2026-10-26-src-public-api-consistency-and-architecture-audit/sources.json
[workspace]: ../../../Cargo.toml
[manifest]: ../../../crates/fsr-sdk/Cargo.toml
[sys-manifest]: ../../../crates/fsr-sdk-sys/Cargo.toml
[scope]: ../../../SCOPE.md
[lib]: ../../../crates/fsr-sdk/src/lib.rs
[upscaler]: ../../../crates/fsr-sdk/src/upscaler.rs
[common]: ../../../crates/fsr-sdk/src/upscaler/common.rs
[reactive-options]: ../../../crates/fsr-sdk/src/upscaler/common/reactive_mask.rs
[runtime]: ../../../crates/fsr-sdk/src/runtime.rs
[movable-runtime]: ../../../crates/fsr-sdk/src/runtime/movable.rs
[runtime-state]: ../../../crates/fsr-sdk/src/runtime/state.rs
[routing]: ../../../crates/fsr-sdk/src/runtime/queries.rs
[native-query]: ../../../crates/fsr-sdk/src/query/native.rs
[native-utilities]: ../../../crates/fsr-sdk/src/query/native/utilities.rs
[dx12]: ../../../crates/fsr-sdk/src/upscaler/dx12.rs
[movable]: ../../../crates/fsr-sdk/src/upscaler/dx12/movable.rs
[frame]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs
[inspection]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs
[descriptor]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs
[sharpening]: ../../../crates/fsr-sdk/src/upscaler/common/sharpening.rs
[generator]: ../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/reactive_mask.rs
[reactive-mapping]: ../../../crates/fsr-sdk/src/upscaler/reactive_mask.rs
[tuning]: ../../../crates/fsr-sdk/src/upscaler/tuning.rs
[eligibility]: ../../../crates/fsr-sdk/src/context_policy.rs
[owner]: ../../../crates/fsr-sdk/src/context.rs
[interop]: ../../../crates/fsr-sdk/src/upscaler/dx12/interop.rs
[diagnostics]: ../../../crates/fsr-sdk/src/upscaler/dx12/diagnostics.rs
[errors]: ../../../crates/fsr-sdk/src/error.rs
[query]: ../../../crates/fsr-sdk/src/query.rs
[resolver]: ../../../crates/fsr-sdk/src/utils/provider.rs
[resolver-tests]: ../../../crates/fsr-sdk/src/query/provider_selection_tests.rs
[provider-example]: ../../../crates/fsr-sdk/examples/provider_choice.rs
[recovery-example]: ../../../crates/fsr-sdk/examples/dx12_recovery.rs
[recovery]: ../../DX12_RECOVERY.md
[threading]: ../runtime-threading.md
[sys-lib]: ../../../crates/fsr-sdk-sys/src/lib.rs
[raw-api]: ../../../crates/fsr-sdk-sys/src/api.rs
[raw-upscale]: ../../../crates/fsr-sdk-sys/src/upscale.rs
[raw-dx12]: ../../../crates/fsr-sdk-sys/src/dx12.rs
[loader]: ../../../crates/fsr-sdk-sys/src/loader/windows.rs
[d013]: ../../adr/D013.md
[d014]: ../../adr/D014.md
[d015]: ../../adr/D015.md
[d016]: ../../adr/D016.md
[d028]: ../../adr/D028.md
[d029]: ../../adr/D029.md
[d031]: ../../adr/D031.md
[d032]: ../../adr/D032.md
[d033]: ../../adr/D033.md
[d034]: ../../adr/D034.md
[d036]: ../../adr/D036.md
[d040]: ../../adr/D040.md
[d041]: ../../adr/D041.md
[callback-record]: ../../release/2026-10-25-callbacks-diagnostics-verification.md
[setup-record]: 2026-10-19-exp-provider-setup-cost.md
[generator-record]: 2026-10-19-exp-reactive-mask-providers.md
[amd-api]: https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h
[amd-upscale]: https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h
[rust-types]: https://rust-lang.github.io/api-guidelines/type-safety.html
[rust-predictability]: https://rust-lang.github.io/api-guidelines/predictability.html
[rust-future]: https://rust-lang.github.io/api-guidelines/future-proofing.html
[rust-non-exhaustive]: https://doc.rust-lang.org/reference/attributes/type_system.html
[rust-traits]: https://doc.rust-lang.org/reference/special-types-and-traits.html
[wgpu]: https://raw.githubusercontent.com/gfx-rs/wgpu/v27.0.1/wgpu/src/api/render_pipeline.rs
