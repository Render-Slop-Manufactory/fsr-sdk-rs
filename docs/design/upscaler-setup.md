# Upscaler setup and acquisition contract

- Drafted: 2026-10-26; accepted for U07 implementation: 2026-10-27.
- Status: **accepted, implemented and verified for U07 under [D042](../adr/D042.md)**,
  following the maintainer's explicit approval. U08 composition is now implemented separately; its
  [verification](../release/2026-10-27-frame-composition-verification.md) covers the
  production paths. The spike's usability guard remains prior candidate evidence;
  [U11 production verification](../release/2026-10-27-context-usability-verification.md)
  separately covers the public status and abandonment operations.
- Production evidence: [implementation verification](../release/2026-10-27-upscaler-setup-verification.md).
- Baseline: wrapper 0.3.0 / sys 0.1.2, Rust 1.88, AMD SDK v2.3.0,
  Windows x64/MSVC + DX12. No new dependency, backend or SDK pin.
- Evidence: [bounded source/compile/controlled experiment](../research/records/2026-10-26-exp-upscaler-setup-contract-spike.md).
  The spike uses real `windows` types and production ownership, routing,
  descriptor assembly and inspection in an isolated candidate. It does not run
  an AMD provider or GPU workload or establish closed-provider safety.

## Intended workflow and public signatures

Load the native library, acquire an upscaler integration scope, plan/create, then
record through independently owned contexts. Configuration and frame data remain
portable; concrete device/resources/list types remain DX12-specific.

The following is signature notation. `UpscalerOptions` is the retained private-field
creation profile, now without a provider field. `load` is deliberately unsafe despite
its short spelling. Neither naming choice changes an obligation.

```text
unsafe fn Runtime::load(path: impl AsRef<Path>) -> Result<Runtime, LoadError>;
unsafe fn MovableRuntime::load(path: impl AsRef<Path>) -> Result<MovableRuntime, LoadError>;

unsafe fn Runtime::upscaler_dx12<'r, 'd>(
    &'r mut self, device: &'d ID3D12Device,
) -> dx12::UpscalerSetup<'r, 'd>;
unsafe fn MovableRuntime::upscaler_dx12<'r, 'd>(
    &'r mut self, device: &'d ID3D12Device,
) -> dx12::MovableUpscalerSetup<'r, 'd>;

ProviderSelection::{Default, Explicit(ProviderId)}

fn UpscalerSetup::selection(&self) -> ProviderSelection;
fn UpscalerSetup::with_provider(self, request: ProviderSelection) -> Self;
fn UpscalerSetup::providers(&mut self) -> Result<Vec<ProviderInfo>, Error>;
fn UpscalerSetup::ratio(&mut self, mode: QualityMode) -> Result<f32, Error>;
fn UpscalerSetup::render_size_for_quality(
    &mut self, output: Dimensions, mode: QualityMode,
) -> Result<Dimensions, Error>;
fn UpscalerSetup::jitter_phase_count(
    &mut self, render: Dimensions, output: Dimensions,
) -> Result<JitterPhaseCount, Error>;
fn UpscalerSetup::jitter_offset(
    &mut self, index: i32, phases: JitterPhaseCount,
) -> Result<JitterOffsetPixels, Error>;
fn UpscalerSetup::resource_requirements(&mut self) -> Result<ResourceRequirements, Error>;
fn UpscalerSetup::estimate_memory(&mut self, config: &UpscalerOptions) -> Result<GpuMemoryUsage, Error>;
fn UpscalerSetup::create(&mut self, config: &UpscalerOptions) -> Result<dx12::Upscaler, Error>;
unsafe fn UpscalerSetup::create_with_callbacks(
    &mut self, config: &UpscalerOptions, callbacks: CreationCallbacks,
) -> Result<dx12::Upscaler, Error>;
unsafe fn UpscalerSetup::configure_debug(
    &mut self, level: DebugLevel, callback: Option<MessageCallback>,
) -> Result<(), Error>;
unsafe fn UpscalerSetup::configure_global_debug_legacy(
    &mut self, level: DebugLevel, callback: Option<MessageCallback>,
) -> Result<(), Error>;
```

The movable setup has matching operations and returns `MovableUpscaler`. No public
mode generic, owner conversion or raw-state/function-table escape is introduced.
Global diagnostics are discovered on the effect setup and remain explicitly
unsafe, without a provider selector or automatic fallback. The live legacy
diagnostic route remains on the context with the existing terminal Configure
policy. A merely loaded runtime exposes no safe effect operation.

Setup creation and selection changes perform no native call. There is one
selection store, and no provider in `UpscalerOptions`. Estimation and creation borrow
the same profile; creation snapshots it. Selection changes affect subsequent
setup operations only. Enumeration is an unselected device/effect observation.
For explicit operations, preserve pure-input-check-before-enumeration order,
fresh count/fill/name copying, membership and exact override forwarding. Creation
additionally checks the effective live ID. No provider caching, reservation,
fallback, ID-order policy or capability inference is added. `Default` can resolve
differently on each invocation.

## Borrowing, ownership and auto traits

The setup holds `&'r mut Runtime` and `&'d ID3D12Device` with independent
lifetimes. Its exclusive runtime borrow intentionally limits overlapping setup
views from that owner and keeps the existing mutable planning convention.
It establishes no coordination with sibling contexts, independent acquisitions,
callbacks, hidden workers or external users.

`create` returns an owner with no setup/runtime/device borrow in its type. Before
entering native creation, the existing ownership path retains a strong runtime
reference, independently clones the device and pins descriptor/host-table storage.
The returned owner retains all of them. Two contexts can outlive the setup,
configuration, original device reference and public runtime independently.

| Type | Required traits and lifetime consequence |
|---|---|
| Local runtime | `!Send + !Sync`, no Clone; loader and final release stay on the acquiring thread |
| Local setup | `!Send + !Sync`; borrows its runtime/device; dropping it only ends those CPU borrows |
| Local upscaler | `!Send + !Sync`, no Clone; independently retained production owner |
| Movable runtime | `Send + !Sync`, no Clone; loading establishes arbitrary-destination loader/release compatibility before it can be sent |
| Movable setup | `Send + !Sync`; its device borrow also requires the real device type's transfer/share contracts, and setup trust covers all destination-thread effect operations |
| Movable upscaler | `Send + !Sync`, no Clone; concrete production owner's existing field audit and lasting coordination duties remain |
| Portable frame data | No GPU borrows; checked required inputs, private fields and explicit updates |
| Resource bundle | Borrows exact renderer resources; cannot outlive them or extend GPU completion lifetime |

Forgetting a setup releases no native object and grants no quiescence. Forgetting
a context can leak its retained bundle but cannot invalidate it. Setup Drop has
no native teardown or callback deregistration. None of these lifetimes permits
premature allocator/resource reuse after recording.

## Accepted safety contracts

The allocation below is a wrapper argument under explicit native assumptions.
It does not manufacture AMD guarantees. Apply [D041](../adr/D041.md): preserve
uncertain safety-sensitive properties as labelled assumptions, and describe
rendering interpretation separately. The production loading/setup Rustdoc now
allocates these duties at their corresponding unsafe boundaries.

### Unsafe loading: common loader and family obligations

The caller establishes all of the following:

1. The DLL, dependencies and any providers loaded during initialization are
   trusted code with the pinned ABI, exports/calling conventions and valid
   initialization/termination behaviour. Export lookup is no authentication or
   native-behaviour check. Dependency/provider resolution during later effect
   operations remains subject to the setup's effect trust too.
2. Initialization, missing-symbol and partial-loading failure cleanup are safe
   on the loading thread. Final release/unload and native worker activity are
   compatible with retained dependencies, including exceptional retention.
3. The common native domain remains serialized and non-reentrant through load,
   all native use, explicit/implicit cleanup, unwinding and final library release.
   Coordinate siblings, other acquisitions, both ownership families and external
   native users. A different path/device or effect does not prove isolation.
4. For the local family, all wrapper native use/release stays on the acquiring
   thread. For the movable family, loader/dependency ownership, implicit Drop and
   final release must already be compatible with every possible destination
   thread before the loaded runtime can enter safe `Send` code. A later setup
   acquisition cannot retroactively establish this loader-level property.

These duties last through every retained runtime reference; dropping the public
runtime does not end them. No upscaler query/create/tuning processing assumption
is established merely by load. No safe effect method is exposed at this stage.

### Unsafe setup acquisition: lasting upscaler integration obligations

The caller additionally establishes:

1. **Safe reachability, not only the initial selection.** All providers reachable
   through the scope's safe default/explicit selection and every wrapper-accepted
   input domain used by its safe operations must permit safe processing or
   ordinary native rejection. Safe `with_provider`, setters and later data
   construction must not introduce a new hidden safety precondition. Restricting
   trust to the initial default or one preferred ID is insufficient. Keep a
   compatible provider configuration while derived objects remain. This broad
   conditional assumption is a real integration cost; membership is not proof.
2. **Queries.** Count/fill capacities are respected, names are readable and
   NUL-terminated through immediate copying, and descriptor/backend/override/
   device/output storage is synchronously consumed without retained CPU addresses.
   Preserve [D018](../adr/D018.md)/[D022](../adr/D022.md)/[D028](../adr/D028.md)'s
   numerical domains: nonzero `u32` creation/estimate dimensions, utility display
   dimensions up to `i32::MAX`, jitter width ratios in `[1, 16383]`, positive
   `i32` phase counts and indices in `0..phase_count`. Success outputs must also
   satisfy the documented result contract; wrapper checks do not protect against
   earlier invalid native writes. Queries on live contexts must be safe in all
   GPU states permitted by recording, and ordinary query errors preserve liveness.
3. **Creation and lifetime.** Checked profiles, flags, backend and override chains
   are handled safely. Owned device and pinned chain/table storage are sufficient
   for all their native accesses. Ordinary returned create errors allow wrapper
   dependency release under [D005](../adr/D005.md)'s accepted conditional policy;
   do not claim provider-internal rollback. OK/null creation and destroy failure
   retain the complete bundle. The chain and its dependencies remain valid after
   moves of the outer owner, setup loss and public-runtime loss.
4. **Configure.** Every safe exposed parameter domain and null-default request is
   processed safely or ordinarily rejected. Temporary descriptor/float storage
   is not retained, including on error; no deferred GPU work or callback creates
   an unowned lifetime. Profile/resource requirements and siblings remain
   compatible. A failed context remains terminally destroyable. GPU idleness
   follows from prior unsafe recording duties, never from the setup borrow.
5. **Cleanup and threading.** Destruction can finish under the existing
   single-attempt/retention policy after recording duties are discharged. All
   effect operations and cleanup obey the common native coordination domain.
   Movable scope/owner use includes arbitrary destination threads, COM release,
   unwinding, error paths and retained bundles; one successful handoff is not a
   general guarantee. Callback installers keep their separate obligations.

These assumptions last through all derived owners and exceptional retention,
including owners created before later selection changes. Dropping or forgetting
the view does not revoke them. No new operation may silently rely on an
unallocated property: a new effect or safe capability needs its own review.

### Recording, callbacks and diagnostics keep operation-specific obligations

Unsafe upscaling/generation still establishes actual-provider numerical/access
support for that request, valid open command lists, truthful states, residency,
non-overlapping backing, resource/list/allocator lifetime and ordered once-only
submission through GPU completion. Complete submitted work and leave no
executable unsubmitted recording before safe Configure. Failed or abandoned
recordings require invalidation, prior-work completion and terminal cleanup.
Generation also keeps its provider-domain, boundary-state-restoration and
generation-to-consumer ordering duties. None is inferred from a resource mask.

Callback-bearing creation and modern/legacy diagnostics retain
[D040](../adr/D040.md)'s external target lifetime, allocator compatibility,
non-unwinding/non-reentry, hidden-activity and shared-registration conditions.
Targets may remain reachable after errors and successful context destruction.
Global installation coordinates every affected CPU/GPU user. The wrapper keeps
only its own chain/host-table/device/runtime storage on exceptional failure;
it does not own external callback targets. No closure guard or setup lifetime
can establish quiescence or automatic unregister.

## Safe-operation coverage and failure audit

| Entry/path | Enforced wrapper property | Earlier or explicit unsafe boundary |
|---|---|---|
| Loading success, missing symbol, partial failure, runtime Drop | Explicit path, required exports, retained library/source error | Loader trust, release and family-wide coordination |
| Setup construction/rebinding/Drop | CPU borrows and one request; no native work | Setup acquisition establishes persistent effect trust; no revocation on Drop |
| Enumeration and six planning routes | Existing routing, pure checks, bounded copied results; selected checks stay fresh | Setup query domains/non-retention/capacities and common coordination |
| Ordinary create and post-create identity/requirements failures | Retain runtime/device/pinned chain before create; use production cleanup | Setup creation/query/ordinary-error release assumptions |
| Callback-bearing create, OK/null, destroy failure | Existing whole-bundle retention, no output-handle destroy on create failure | Setup lifetime plus explicit installer duties for external targets |
| Live provider/requirements/memory and four utility queries | Eligibility and checked outputs; query errors do not newly poison | Lasting setup trust, including queries while permitted GPU work is in flight |
| Optional metadata preflight | No FFX call, same context checks; no independent GPU retention | No added native trust; successful metadata inspection certifies no states/content/completion |
| Upscale and generation recording | Fresh mandatory inspection and production descriptor mapping | Explicit per-call recording/access/GPU and failure duties plus setup trust |
| Context Configure and live legacy diagnostics | Eligibility first; native Configure failure is terminal | Setup Configure processing; earlier recording supplies idleness; installer supplies diagnostic/global duties |
| Pure usability/abandonment | Preserve first cause and reject later live use | No native operation, wait or release; recording lifetime/cleanup obligations persist |
| Consuming destroy, normal Drop, unwind | One attempt; retained state/device before runtime release; failure retains bundle | Lasting setup/loader contracts and recording completion/invalidation; no GPU wait |
| Movable transfer, setup transfer, destination-thread Drop | Concrete `Send + !Sync` types, no conversion from local | Loader and setup arbitrary-destination assumptions established before transfer |

The spike reaches these edges through production mechanisms where implemented;
the experiment record distinguishes executed cases from source-only analysis.
It does not independently re-prove the shared native owner's pre-existing
conditional contracts or every callback registration route.

## U08 production composition and earlier probe

The U08 production recording shape is:

```text
unsafe fn Upscaler::record(
    &mut self, list: &ID3D12GraphicsCommandList,
    resources: &UpscaleResources<'_>, frame: &FrameParameters,
) -> Result<(), Error>;
fn Upscaler::validate_upscale_inputs(&self, list, resources, frame) -> Result<(), Error>;
unsafe fn Upscaler::record_reactive_mask(&mut self, list, resources, parameters) -> Result<(), Error>;
```

Required frame values remain explicit and updatable. Native motion choice
replaces units/override storage; inspection borrows the actual portable values,
so no dormant override can survive a later pixel/UV request. Sharpening
uses the same implementation relocated into portable common code. Named bindings
require every role and an explicit texture/automatic exposure choice. Construction
and cloning only assemble borrows. Preflight never skips later inspection.

The probe preserves a renderer-known terminal cause after recording failure or
abandonment and rejects later live operations. This small U11-shaped guard is
experimental scaffolding to test the third workflow; it does not select a new
recovery contract or complete U11. The spike did not implement U09 checked
tuning or U10's final error/metadata taxonomy; it retains the baseline tuning pair
and an owned experiment-only error. Production U09 now supplies checked requests;
its [verification](../release/2026-10-27-tuning-request-verification.md) remains
separate from that probe. U08 completes generation's named builder and private-field portable
parameters. The earlier probe checked only its three-way call shape.

## Acceptance and remaining work

The recorded spike and production checks support the structure: owned contexts need no factory lifetime, two independent setup
borrows are expressible, no safe effect route leaks from the loader-only type,
and local/movable trait differences survive. This is a conditional wrapper
argument, not a completed native soundness proof.

Production uses `UpscalerOptions`, concrete local/movable setup types and complete
public Rustdoc. Old flat Runtime effect methods and public static context
constructors are removed. Production reuses the private routing/owning core;
the frozen spike retains its baseline API internally for reproducibility.
U10/U11 production checks are tracked separately. U12 now supplies
[integrated consumer/migration checks](../release/2026-10-27-api-transition-verification.md);
[P08 is closed with accepted limits](../release/2026-10-29-performance-review-closure.md);
[the common rc1 archives are qualified](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification).
Publication remains separate.

If future evidence shows safe provider rebinding cannot be supported under an adequately bounded
upfront trust contract, revisit that acquisition/selection boundary before
implementation. Do not add hidden safety duties to a safe selection method.
Likewise, contrary native lifetime/threading/cleanup evidence requires revising
the supported scope or contract. Compile success cannot overrule it.
