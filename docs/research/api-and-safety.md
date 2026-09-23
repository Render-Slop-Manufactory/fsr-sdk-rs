# API and safety

- Updated: 2026-09-23.
- Scope: common ABI, loading, provider deployment, version enumeration, context
  lifecycle contracts and isolated create-failure experiments; M5 construction,
  Query/Configure, runtime-sharing source findings and bounded signed-runtime
  experiments for FidelityFX SDK v2.3.0; and older FSR2 Rust binding comparisons.
  Execution/layout evidence is limited to Windows x64/MSVC.
- Included evidence: [common ABI verification record](records/2026-09-20-exp-common-abi-verification.md)
  and [explicit loader verification](records/2026-09-20-exp-explicit-dll-loader.md),
  plus the [version-count query](records/2026-09-20-exp-version-count-query.md),
  [DX12 provider-DLL resolution](records/2026-09-21-src-dx12-provider-dll-resolution.md),
  [signed DX12 directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md),
  [lifecycle contract refinement](records/2026-09-22-src-lifecycle-contracts.md),
  [source provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md),
  [distribution/artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md),
  [v2.3.0 lifecycle investigation](records/2026-09-20-src-context-lifecycle.md),
  [destroy failure-path investigation](records/2026-09-20-src-destroy-failure-paths.md),
  [successful DX12 lifecycle](records/2026-09-20-exp-context-lifecycle.md),
  [M4 Windows production-owner verification](records/2026-09-22-exp-m4-windows-verification.md),
  [create-allocation failure spike](records/2026-09-20-exp-create-allocation-failure.md),
  [returned-create-error input lifetime spike](records/2026-09-22-exp-create-error-input-lifetime.md),
  [Embark fsr-rs comparison](records/2026-09-20-src-embark-fsr-rs.md) and
  [fsr2-rs raw-binding comparison](records/2026-09-20-src-fsr2-rs.md),
  [M5 source provenance repair](records/2026-09-22-src-m5-construction-query-provenance-repair.md),
  [M5 numerical construction investigation](records/2026-09-23-src-m5-numeric-construction-safety-contract.md),
  [signed-runtime dimension probe](records/2026-09-22-exp-m5-upscaler-construction-inputs.md),
  [signed-runtime DX12 width-boundary probe](records/2026-09-23-exp-m5-dx12-dimension-boundary.md),
  and [sequential shared-runtime probe](records/2026-09-22-exp-m5-shared-runtime-provider-lifetime.md).

## Current findings

**Verified source facts:** `ffx_api.h` supplies the original common boundary:
context and scalar aliases, eight return constants, the base descriptor header
and four aliases, allocation callbacks and the five entry-point pointer types.
Resource and extended descriptors are not required to declare those signatures.
Context parameters are pointers to handles; allocation sizes and descriptor tags
are 64-bit unsigned integers. Create/query descriptor pointers are mutable;
configure/dispatch descriptors and allocation-callback struct parameters are const.
See the record's header findings and pinned local-header fingerprint.

**Verified test results:** Rust checks and a separate native C++ compile passed
the shared layout, constant and signature expectations. On the tested target the
base header is 16 bytes/alignment 8 and the allocation-callback struct is 24
bytes/alignment 8. The C++ file is a compile-only verification probe, not part of
the library or its Cargo build. Reproduction commands and setup failures are
preserved in the record.

**Upstream contracts:** pointers passed in creation descriptors must remain live
through destruction (the lifetime of chain-node storage is ambiguous; see below); allocation/deallocation have alignment, null and compatibility
obligations. Global query/configure allow a null context pointer. These comments
are evidence of the declared contract, not observed runtime behavior. Nullable
Rust function-pointer representation does not authorize passing null callbacks.

**Inference:** the paired checks support the selected Rust ABI mapping on Windows
x64/MSVC. They compare each language against shared expectations, not directly
against each other through a call. They do not establish a safe wrapper.

**Verified loader results:** the local `amd_fidelityfx_loader_dx12.dll` loaded
and all five API exports resolved, without invoking an AMD API function. Its
tested hash is in the loader record. The SDK documentation identifies it as the
entry DLL managing separate effect providers. The subsequent query experiment
adds evidence for one upscaler enumeration path.

**Verified query result:** one count-only `ffxQueryDescGetVersions` invocation
with null context/device returned OK and count 2. This requires one additional
56-byte descriptor and the version-query/upscaler-selector tags; paired ABI checks
cover fields, layout and constants against local headers. No name or ID arrays
were requested, so the count does not identify the providers or GPU compatibility.

**Deployment observation:** the initial call using the original absolute SDK
loader path returned NO_PROVIDER. Staging the test executable with the loader
and upscaler DLLs returned OK. This supports executable-adjacent deployment for
this test, not a comprehensive DLL search policy. No production loader change
or search-path mutation was made. Exact commands and both results are preserved
in the query record. The SDK's C example omits the descriptor tag; the probe sets
it explicitly according to the header and implementation.

**Verified ownership and failure checks:** `FfxLibrary` owns private typed
pointers and the library. Five unsafe forwarding methods borrow that owner;
there is no raw accessor. Fixture tests cover missing files, invalid images,
each missing export, release after partial resolution, forwarding and release
on drop. Compile-fail tests cover borrowing and private pointer access.

**Safety boundary:** explicit input paths are canonicalized before loading the
main DLL. Dependency resolution still follows Windows loading rules. Loading
requires trusted initialization/termination and ABI-compatible exports; no
authenticity check occurs. Native contexts and work may outlive individual calls,
so the raw caller must preserve library and resource lifetimes and synchronization.
This bounded design is authorized by
[D004](../DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points).

## DX12 provider discovery and deployment

The [provider-resolution investigation](records/2026-09-21-src-dx12-provider-dll-resolution.md)
examines SDK v2.3.0 at commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
The [signed-runtime experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md)
now resolves the bounded private-directory question for the tested v2.3.0 DLLs.
It agrees with the earlier deployment observation; the source record's unresolved
status reflected missing implementation evidence, not a contradictory result.

**Reported source facts:** creation enters `GetProvider`; a null-context version
query enters `GetProviderCount` or `GetProviderVersions`. Existing contexts retain
their selected provider. Selection uses effect/descriptor and provider-version
IDs and can consider an external/driver provider. `ffxLoadFunctions` only resolves
the five exports from an already-loaded module; it does not load effect DLLs.
Neither the inspected version-query descriptor nor `ffxOverrideVersion` supplies
a provider path. Version selection does not establish filesystem isolation.

**Upstream deployment guidance:** the tagged DX12 sample copies loader and effect
DLLs to its executable output directory; the denoiser guide explicitly requires
executable-adjacent placement. This agrees with the successful staged query.
It does not establish that colocating loader and provider in a private directory
away from the executable is sufficient.

**Unresolved implementation:** the public source omits the `amdinternal` layer
needed to identify the physical provider-DLL opening call, its path/flags and
eager versus lazy loading. The record's supplementary third-party binary report
is not tied to the pinned signed artefact and supplies no call arguments; its
imports/strings do not verify a v2.3.0 bare-name `LoadLibraryA` mechanism.

**Conditional Windows inference:** explicitly loading the API DLL by absolute
path does not itself add its directory to the search path for a later independent
bare-name load. If the private provider loader uses such a call, executable
placement and already-loaded same-basename modules can affect resolution.
Neither that call form nor deterministic coexistence of multiple FSR runtime
copies is established. No supported provider-directory control was identified
in the inspected public surface; this is not proof against private mechanisms.

**Verified runtime behavior:** the signed v2.3.0 loader and upscaler DLLs were
tested on Windows build 26200.9457 with Rust x64/MSVC, using the existing null-
context/device count-only query. Each layout ran in a fresh process with an empty,
separate working directory and an absolute loader path. PATH was checked for
provider copies and inherited unchanged; the probe/runner made no DLL-directory
API calls. Both source DLLs had valid AMD signatures; fingerprints and logs are
preserved in the experiment record.

| Layout | Loader acquisition | Query return / count | Child exit |
|---|---|---|---:|
| A: loader and provider beside executable | succeeded | 0 (OK) / 2 | 0 |
| B: loader and provider together in private `runtime/` | succeeded | 4 (NO_PROVIDER) / 0 | 101 |
| C: private loader, provider beside executable | succeeded | 0 (OK) / 2 | 0 |
| D: private loader, no staged provider | succeeded | 4 (NO_PROVIDER) / 0 | 101 |

The provider was absent before and immediately after loader acquisition in every
case. After successful queries, module diagnostics identified exactly the staged
executable-adjacent provider. It remained absent in B and D. Negative exits were
the existing test assertion reacting to the observed return code, not loader
failures or native crashes. D and the successful module paths confirm dependence
on the staged provider under this environment.

**Verified conclusion:** colocation in the tested private directory alone was
insufficient; placing the provider beside the executable succeeded even with a
private loader. This establishes version enumeration for these binaries on this
configuration, not provider identity, GPU compatibility or dispatch.

**Inference and limits:** the results are consistent with executable-directory
resolution, but do not reveal AMD's exact loading API, argument, flags or full
search/timing algorithm. One run per layout on one Windows configuration does
not establish a contract for future releases or arbitrary process environments.
Same-basename coexistence, undocumented controls and transitive dependencies
remain unresolved; no clean-machine deployment was tested.

**Accepted policy and remaining research:**
[D007](../DECISIONS.md#d007--keep-runtime-acquisition-explicit-and-deploy-the-v230-dx12-upscaler-dll-beside-the-executable)
selects explicit, application-controlled acquisition and executable-adjacent
upscaler deployment for this bounded baseline. The loader may reside in a private
directory and is loaded through an explicit path. No additional experiment is
needed to answer private-directory colocation sufficiency for these inputs.
Alternate loading mechanisms, runtime coexistence, other systems and dependency
closure remain unresolved and would need their own evidence. The decision does
not establish AMD's internal search algorithm or authorize binary redistribution.

## Context lifecycle: v2.3.0 source findings

The [lifecycle report](records/2026-09-20-src-context-lifecycle.md) examines tag
`v2.3.0`, commit `60f4ea8`. These are reported source findings, not new runtime
results; the signed FSR4 and external/driver providers are not characterized by
the open FSR2/FSR3 implementations.

**Upstream contract:** ordinary DX12 upscaler creation needs an upscaler root,
`ffxCreateContextDescUpscaleVersion` with `FFX_UPSCALER_VERSION`, and a DX12
backend descriptor containing a live device. The effect-specific documentation
requires the version node even though the generic example omits it. Optional
`ffxOverrideVersion` instead selects an enumerated provider ID; it does not
replace the API-version node. The existing count-only query supplies no IDs.
Descriptor tags must match their actual payload types; mismatch is not merely a
recoverable validation error.

**Reported source behavior:** creation clears a valid output slot after checking
both input pointers, then delegates to a provider. Open FSR2/FSR3 providers publish
the handle only on success; there is no provider-independent failed-create
postcondition. Destruction in these open paths frees the context without clearing
the caller's handle, contradicting the generic documentation. Provider lookup
assumes a live handle: destroying a null handle or destroying twice is hazardous.
A null pointer-to-handle has a distinct error path and is not equivalent to a
pointer containing a null handle.

**Unresolved cleanup:** a create error does not establish that destruction is
valid or required. A destroy error does not establish that the context is still
usable, that teardown did nothing, or that retrying is safe. Native return codes
must remain representable beyond today's known constants. Exact-once teardown
needs independent Rust state; testing the raw handle for null is insufficient.

**Lifetime constraints:** keep the device and retained callback code live through
native use. Open providers retain the message callback. The destroy-path report
identifies device reference retention/release in the open v2.3.0 DX12 backend;
equivalent ownership in the signed/external 4.1.1 provider remains unresolved.
The header's generic pointer-lifetime wording does not establish a uniform
chain-node lifetime: the override node is explicitly call-scoped, while the
concrete upscaler/version/backend chain remains unresolved for signed providers.
Host allocation callbacks passed separately to create/destroy need compatible allocation state,
not the same callback-struct address. They are distinct from DX12 backend
allocation callbacks carried in the chain.

**Inference:** live contexts depend on callable provider code, so library ownership
must extend beyond individual forwarding calls. Provider selection can vary with
the device and driver; record the selected provider with lifecycle observations.
No usable upscaler concurrency guarantee was found. Neither Rust auto traits nor
successful stress tests establish that contract.

## Descriptor, callback, device and threading contracts

The [2026-09-22 lifecycle contract investigation](records/2026-09-22-src-lifecycle-contracts.md)
refines the earlier report against the same v2.3.0 baseline. These are reported
source findings, not additional runtime verification.

- **Descriptor storage:** tagged documentation explicitly permits the
  `ffxOverrideVersion` node to expire after creation. This contradicts a blanket
  contractual requirement to retain every chain node until destruction. Open
  FSR2/FSR3+DX12 paths consume/copy the inspected descriptors without retaining
  their addresses. Neither fact establishes call-only lifetime for the concrete
  `UpscaleVersion` or `BackendDX12` nodes in a signed provider. Conservative
  retention remains a wrapper policy, not a demonstrated universal AMD rule.
- **Message callback:** open FSR2/FSR3 providers retain `fpMessage` per context
  and install it in unsynchronized module-global state. Destroy does not clear
  that global pointer, so the dependency can outlive its installing context.
  Inspected debug dispatch calls it synchronously, but no public lifetime,
  calling-thread, reentrancy or serialization guarantee was found for signed
  providers. Null is explicitly permitted and introduces no callback dependency
  of its own; it does not characterize separately configured global callbacks.
- **Device ownership:** the open DX12 backend takes one COM reference when its
  scratch/backend instance gains its first effect context and releases it after
  the last. This is not necessarily one reference per API context. The public
  descriptor promises no ownership transfer or permission to release the caller's
  reference immediately; signed-provider COM retention remains unresolved.
- **Concurrency:** no upscaler contract guaranteeing or prohibiting same-context,
  cross-context, creation/destruction or provider-enumeration concurrency was
  found. Mutable context state, global callbacks and shared version-name storage
  prevent inferring isolation from separate Rust owners. Frame Generation has
  explicit synchronization rules, but those do not transfer to Upscaling.

**Design implication:** D005's null callback and retained dependencies avoid
some unresolved dependencies, but excluding Send/Sync alone supplies no global
native threading contract. Non-null callback support needs a broader lifetime
model than context-local ownership. The failed-create spike does not resolve
successful-context descriptor lifetimes, callback behavior or threading. D005
is now accepted; suggested instrumentation in the source report is future
research, not executed evidence or authorization to run it.

## M5 source intake and provenance

Four source-only records dated 2026-09-22 examine the same SDK v2.3.0 commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`:

- [Upscaler construction validation](records/2026-09-22-src-m5-upscaler-construction-validation.md).
- [Upscaler Query/Configure surface](records/2026-09-22-src-m5-upscaler-query-configure.md).
- [Runtime sharing and threading](records/2026-09-22-src-m5-runtime-sharing-threading.md).
- [Construction and Query/Configure provenance repair](records/2026-09-22-src-m5-construction-query-provenance-repair.md).

**Provenance:** the later repair supplies direct links to pinned headers,
documentation, samples and implementation sources and narrows several earlier
claims. The original construction report still lacks direct URLs and the
original Query/Configure report lacked an index-to-source map for its opaque
citation markers. Those markers were removed as clerical cleanup; the repair's
source table does not reconstruct a claim-by-claim map for every original
statement. The source findings below are not signed-runtime verification and
do not establish a new public safety contract by themselves.

### Construction and validation

**Reported source findings:** the upscaler root, mandatory API-version node and
DX12 backend form the relevant creation chain. The API-version value describes
the interface built against, not the selected provider ID. Explicit provider
overrides instead use IDs obtained through version enumeration. The official
sample puts backend before version; M4's successful version-before-backend chain
is not contradicted, and neither order establishes arbitrary extension ordering.

The source record finds no complete provider-independent numerical contract for
creation dimensions: nonzero values alone are not a documented sufficient
condition, and generic minima, maxima, aspect-ratio limits and downscaling
support remain unresolved. Native AA permits
equal render/output sizes for supporting providers. Open FSR3 accepts variable
dispatch dimensions within creation maxima and rejects dimensions exceeding them;
that implementation does not establish signed FSR4 validation behavior.

The header reportedly permits zero or combinations of ten defined creation bits.
Flags must describe the application's actual input conventions. The FSR3 adapter
maps bits 0–7 but not the non-linear-colorspace/debug-visualization bits 8–9, so
shared flag declarations do not establish uniform provider behavior. Device
requirements likewise differ by provider; enumeration with a device is useful
availability evidence, not exhaustive compatibility validation.

**Safety interpretation:** correctly tagged layouts, valid pointers, retained
dependencies and callable callback targets are distinct from functional input
choices. Failure to find a numerical safety rule does not prove that arbitrary
dimensions safely return an error. The report's suggestion that limits can be
delegated to provider validation is not a verified general rejection guarantee.
M4's unsafe constructor and D005 runtime assumption remain unchanged.

**Reconciliation with prior evidence:** the earlier lifecycle-contract record
identifies an explicit call-only lifetime for `ffxOverrideVersion`. The new
construction record describes that exception less precisely; it does not undo
the earlier finding. Lifetime of the actual `UpscaleVersion` and `BackendDX12`
node storage for signed providers remains unresolved. M4 retains both. Open
backend COM retention is per backend's live-effect population, not a universal
one-AddRef-per-modern-context contract.

### Query and Configure scope

**Reported inventory:** two generic and seven upscaler Query descriptors cover
version enumeration, live provider metadata, quality ratio/render resolution,
jitter phase/offset, live and pre-create GPU memory, and resource requirements.
Configure adds two global debug descriptors and one upscaler key/value descriptor
with five float tuning keys. This inventories upstream surface, not implemented
wrapper support or a requirement to expose it all.

| Concern | Current source conclusion and boundary |
|---|---|
| Minimal use | No unconditional Query/Configure prerequisite was found for ordinary creation or the first dispatch. An explicit provider override requires a queried ID. Temporal input jitter is still needed; AMD permits an application-owned generator. |
| Null context | Supported only for specific queries. Utility queries need a chained DX12 device to reach external/driver providers; GetVersions and memory V2 embed their own device. The existing null-device count test does not prove that other queries are device-free. |
| Quality helpers | Quality is a utility-query input, not a creation field. Helpers are provider-routed; arithmetic-looking outputs do not establish provider-independent pure functions. |
| Output lifetimes | Query arrays/buffers need valid writable storage and correct capacity. Enumeration names can be overwritten by later queries and should be copied. Exact live-provider-name lifetime and cross-provider pointer retention remain unresolved. |
| Memory/resources | Live memory Query requires a context; V2 estimates before creation. Resource requirements depend on the selected provider. FSR3's estimator reportedly ignores V2 flags; do not infer uniform semantics from the field alone. |
| Configure | Debug/tuning is optional. FSR3 consumes float pointers synchronously and supports null reset for its five keys; these are implementation details, not a universal pointer-lifetime/reset contract. Public loader debug state is module-static; signed-runtime and cross-module scope remain unresolved. |

**Source discrepancy:** jitter documentation specifies `ceil(8*n²)` whereas
the inspected FSR3 code truncates to an integer; custom scale factors and other
providers remain unresolved. The v2.3.0 release describes `effectId` as enabling
per-effect debug routing, while the pinned public loader ignores the field and
uses module-static callback state. The signed runtime's exact behavior and
cross-module scope remain unresolved. No Query purity, Configure visibility,
CPU overlap or GPU-completion guarantee follows from these reports.

### Runtime sharing and CPU threading

**Reported positive evidence:** multiple coexisting contexts are part of the SDK
architecture: lower-level DX12 supports multiple effect contexts and modern
Frame Generation uses distinct live context types. Open FSR3 allocates separate
context/backend state, and context-bound calls recover a per-context provider
association. These observations distinguish coexistence from simultaneous CPU
execution. They do not guarantee arbitrary numbers of upscaler contexts or
concurrent use of different providers through one runtime.

**Unresolved:** same-context calls, independent-context calls, overlapping
create/destroy, and provider enumeration versus lifecycle operations have no
general upscaler concurrency guarantee in the inspected material. API diagnostics,
version-name storage and DX12 allocation callbacks include shared global state.
Different devices do not remove those interactions. Frame Generation's explicit
external-synchronization requirements remain effect-specific.

Provider-object reference counting does not establish effect-DLL residency or
loader synchronization. The effect of destroying one context on module state
used by another remains insufficiently specified for arbitrary runtime sharing.
D005 permits future sharing but does not select it; neither `!Send/!Sync` nor
exclusive Rust library owners prove global native isolation.

### Signed-runtime construction inputs

The [dimension experiment](records/2026-09-22-exp-m5-upscaler-construction-inputs.md)
used the signed v2.3.0 loader/upscaler DLL pair on one Windows x64/MSVC system
with an RX 9060 XT. Each of nine structurally valid cases ran in a fresh process.
The 1280×720 → 1920×1080 control and four nonzero variants (1×1 → 2×2,
non-multiples of eight, equal maxima, and render maxima larger than upscale
maxima) each returned OK, reported provider 4.1.1 from a live-context query,
and destroyed successfully. These results establish construction and teardown
only; downscaling and GPU dispatch were not tested.

Each case with zero in one of the four width/height components instead aborted
before `ffxCreateContext` returned. The child exit was `0xC0000409`, with a Rust
foreign-exception report; the native exception type, throw site and selected
provider for those failed cases were not established. The current private
constructor already uses `NonZeroU32`. For this tested signed path, zero input
cannot safely be delegated to a returned native error. The result does not
establish a complete nonzero size domain or behavior on other configurations.
This corrects any reading of the earlier source-only report as promising an
ordinary error return for zero dimensions.

### Numerical construction safety boundary

The [2026-09-23 source investigation](records/2026-09-23-src-m5-numeric-construction-safety-contract.md)
finds no public v2.3.0 contract or auditable signed-provider source defining a
complete safe numerical domain for the four creation dimensions. The generic
upscaler ABI specifies their meanings but no universal minimum, maximum,
alignment, aspect ratio or render-versus-upscale ordering rule. The decision in
[D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
is now accepted with an explicit bounded numerical runtime trust assumption;
the production constructor remains private and unsafe pending M5 implementation.

**Open-provider source behavior:** FSR2 and FSR3 derive half-resolution resource
extents from `maxRenderSize`, so a coordinate of 1 yields a zero extent in those
paths. They also assign some unsigned creation dimensions into signed constants.
The signed-runtime experiment succeeded at 1×1 → 2×2, demonstrating that the
open-provider resource envelope cannot be transferred to the selected closed
provider. It does not prove that all other small or nonzero values are safe.

**Backend and error limits:** Microsoft's DX12 Texture2D coordinate range of
1–16384 constrains actual texture descriptions at the relevant feature levels;
it does not directly bound upstream upscaler creation dimensions. The pinned
open DX12 backend throws on failed resource-creation HRESULTs, and the inspected
generic create path does not show conversion of that throw to an API return code.
The pre-create V2 memory Query can expose some invalid resource descriptions
for open providers but is not documented as a complete signed-provider
construction validator. None of these observations establishes the signed
provider's internal arithmetic, failure behavior or a safe wrapper range.

**Verified signed-runtime boundary observation:** the
[three-point experiment](records/2026-09-23-exp-m5-dx12-dimension-boundary.md)
ran `16383x720`, `16384x720` and `16385x720` as equal render/upscale maxima,
one fresh child per case, on the same signed v2.3.0 DLL pair and RX 9060 XT
configuration. All three entered and returned from `ffxCreateContext` with
`FFX_API_RETURN_ERROR` (`1`) and a null output; each child exited normally and
none attempted destruction. No successful context existed to query provider
identity. Earlier successful controls identified 4.1.1 on this configuration,
but do not identify the provider selected during these rejected calls.

**Inference and limits:** the beyond-limit `16385` case returned an ordinary
native error on this run. Because the below-limit and at-limit cases returned
the same generic error, the experiment neither shows that `16384` is accepted
nor isolates the D3D12 width limit as the rejection cause. The error does not
identify an out-of-memory, aspect-ratio or provider-specific rule; the
recorded GPU-memory readings do not exclude every resource-pressure contribution.
These observations do not establish a safe width interval or a general
ordinary-error contract for unsupported sizes. D008 now selects an explicit,
bounded trust assumption for the supported runtime path and its actually
selected providers. This closes the policy question without converting these
observations into proof of a safe numerical interval; no further numerical
research is required for D008. Implementation remains pending.

### Sequential shared-runtime lifetime

The [shared-runtime experiment](records/2026-09-22-exp-m5-shared-runtime-provider-lifetime.md)
used one `FfxLibrary`, one DX12 device and two live upscaler contexts on the
same signed DLL pair. A and B both created, identified provider 4.1.1 and were
destroyed once. After A was destroyed, B's live-context provider query returned
the same ID/name. The staged upscaler DLL remained resident at every observed
point while B lived, and also after B and the loader owner were dropped.

This verifies a two-context **sequential** path on the tested configuration.
It neither establishes concurrent-call safety nor a normative module residency
rule, provider DLL reference counts, dispatch correctness or the identity of
all participating driver code. Source evidence still leaves cross-context
synchronization and arbitrary provider/runtime sharing unresolved.

**Integration outcome:** the source repair and experiments refine M5 inputs
without completing M5. They select no public runtime, device, sharing or
synchronization API. The remaining helper, retention, ordering and threading
questions need separate evidence where the selected wrapper surface requires it.

## Destroy failures: source paths and limits

The [destroy-path investigation](records/2026-09-20-src-destroy-failure-paths.md)
supplies commit-pinned sources for v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
These are reported static findings about the common layer, open FSR2/FSR3
providers and their DX12 backend, not additional runtime observations or an
implementation description of the signed 4.1.1 provider.

**Reported source behavior:** the common destroy entry point checks the outer
pointer, reads the associated provider through the handle, and forwards the
provider's result unchanged. It then performs zero-refcount provider cleanup
even if that result is an error. The inspected static FSR2/FSR3 providers do not
reach zero on this path; external-provider reference accounting is unavailable.
An error alone therefore does not establish that provider code/state survives.

| Inspected path | Error handling and post-state | Reachability limit |
|---|---|---|
| Null pointer-to-handle argument | Parameter error 6 before teardown | No live context involved; does not test post-failure liveness |
| Pointer to a null, stale or invalid handle | Provider lookup dereferences the handle before inner provider checks | No controlled error or safe validation established |
| Open FSR2 valid context | Core release ignores backend cleanup results and returns success; provider then frees scratch/context | No legitimate ordinary-use destroy error identified |
| Open FSR3 shared-resource loop | Backend errors become public runtime error 3; earlier entries may already be released, while core/scratch/context teardown has not completed | Stock DX12 trigger found is an out-of-range private resource index; no valid-state supported-use trigger identified |
| Host deallocation callback | Returns void, with no error channel | Allocator incompatibility is a contract violation, not a recoverable destroy-error mechanism |

Both open core destroy functions have a null-core-pointer error, but valid
providers pass the address of an embedded object. That branch is not a supported
failure trigger for a live context. FSR2/FSR3 core safe-release helpers discard
backend destruction results; this differs from the FSR3 provider's earlier
shared-resource loop, which propagates them. The mere existence of an API error
constant does not show that a destroy path can emit it.

**Inference:** destruction is not established as transactional. The conditional
FSR3 error path can follow partial teardown; it does not justify treating an
error as an untouched, usable context. Neither inspected documentation nor the
source establishes a general retry contract. This conditional source path was
not exercised and must not be presented as an observed valid-context failure.

**Unresolved:** the signed loader's exact forwarding/validation and the
FSR4/ML/external provider's destruction implementation are unavailable in the
inspected public source. Open-provider behavior cannot classify a hypothetical
4.1.1 destroy error as live, dead or partially destroyed. No legitimate,
non-corrupting valid-context failure trigger suitable for a runtime experiment
was identified. The report's null-outer-pointer case concerns API conformance
only; it does not close that gap or authorize a new experiment.

The report also confirms that the C++ convenience wrapper merely forwards
destroy and does not clear the handle. This reinforces the existing
documentation conflict and successful lifecycle observation without proving
that the signed provider uses the same internal implementation. No Context,
Drop or failure policy is accepted by integrating these findings.

## Context lifecycle: observed runtime behavior

The [successful lifecycle experiment](records/2026-09-20-exp-context-lifecycle.md)
and [allocation-failure experiment](records/2026-09-20-exp-create-allocation-failure.md)
use the local SDK v2.3.0 loader/upscaler DLLs on Windows x64/MSVC with an AMD
Radeon RX 9060 XT and driver `32.0.31041.1004`. Both retain the complete
root -> version -> DX12 descriptor chain, device and libraries. No dispatch is
submitted. The creation descriptors have since been promoted into sys for the private M4
owner; provider-query instrumentation remains test-local. These historical
results do not themselves verify the new production owner or establish public construction.

**Verified M4 production-owner execution:** the
[Windows verification](records/2026-09-22-exp-m4-windows-verification.md)
now closes the implementation's Windows execution gap on build 26200.9457 with
the same RX 9060 XT, driver and fingerprinted SDK inputs. Formatting, Clippy,
17 ordinary executable tests, two compile-fail doctests, no-default-feature
checking and paired MSVC C++ ABI compilation passed. Windows fixtures exercise
the production loader and verify terminal teardown, ordered release, exceptional
retention and DLL residency. These are wrapper-policy results, not AMD failure
postconditions. The isolated native test created through the production
`Upscaler` / `NativeContextOwner`, moved the outer owner, identified provider
`0xf5a5ca1e01001001` / `4.1.1`, and successfully destroyed it; exit 0 without
timeout. No production change was needed. One successful lifecycle does not
establish dispatch, image correctness, native failure cleanup, measured COM
reference counts or general threading guarantees. D005's runtime trust assumption
and remaining provider-contract questions are unchanged.

**Verified successful lifecycle:** creation returned OK with a non-null context;
the provider query identified `0xf5a5ca1e01001001`, name `4.1.1`; destruction
returned OK without nulling the handle. This independently reproduces the
source-reported non-nulling behavior on the tested runtime path and contradicts
the generic documentation's null-after-destroy claim for that path. It does not
establish minimum descriptor/device lifetimes or general GPU compatibility.

**Documented allocation contract:** the local `ffx_api.h` explicitly permits
the host allocator to return null to indicate failure. It requires suitable
alignment, acceptance of null free arguments and compatible create/destroy
allocators. It supplies no universal failed-create handle or rollback guarantee.

**Verified allocation observations:** instrumented successful controls reached
three host allocations of 1,099,864, 1,634,664 and 16 bytes. Create and destroy
returned 0; destroy freed all three exactly once, in order 2, 1, 3. The same
callbacks were used for both calls; replacement-callback compatibility was not
tested. The following outcomes were reproduced in two process-isolated sweeps:

| Injected null at attempt | Successful allocations / observed frees | Outcome |
|---|---|---|
| 1 | 0 / 0 | Access violation before create returned |
| 2 | 1 / 0 | Access violation before create returned |
| 3 | 2 / 0 | Access violation before create returned |
| 4 (not reached) | 3 / 3 after destroy | Successful control |

All injected cases ended with process status `0xC0000005`, not an FFX return
code. Their handles were null before create; neither a returned error nor an
after-create handle state was observable. No free callback occurred before the
crashes, and no destroy-after-failure or speculative manual cleanup was attempted.
The successful controls identified provider 4.1.1; failed processes could not
independently query provider identity. Neither experiment identifies whether the
implementation came from the bundled provider DLL or the driver.

**Inference:** on this tested path, permitted host-allocation failures did not
become recoverable FFX errors, and indices 2 and 3 showed no callback-visible
partial cleanup before termination. This is not evidence of a usable failed
context or permission to destroy it. OS process reclamation is not SDK cleanup.

**Unresolved:** exact fault location/cause was not captured; the results do not
attribute a defect to a specific module. Accounting covers only these host
callbacks, not GPU allocations or other internal allocators. Those crashes supply
no returned-error postconditions; the separate experiment below covers one
returned rejection. Other providers/builds and failed destruction remain untested.
A successful control does not
establish general leak freedom, and a crashing injection does not establish a
universal failure contract.

## Returned create error: caller input lifetime

The [D005 input-lifetime spike](records/2026-09-22-exp-create-error-input-lifetime.md)
adds a normally returned error from the signed v2.3.0 runtime on Windows
x64/MSVC, RX 9060 XT and driver `32.0.31041.1004`. It uses a correctly tagged
`upscale -> version -> DX12 -> DX12` chain with the same live device in both
backend nodes and null host, message and backend callbacks. Local source
explicitly rejects the duplicate backend; no invalid pointer or mismatched
descriptor payload induces the error.

**Verified observations:** discovery and all three isolated lifetime variants
returned 1 (`FFX_API_RETURN_ERROR`) with a null output. No failed output was
destroyed, dereferenced or queried. Each variant exited normally after a
two-second observation window, with no observed fault or timeout:

| Variant | Action after returned error | Result |
|---|---|---|
| Retain | Keep descriptor storage, device reference and runtime | Observation completed |
| Protect | Apply and verify PAGE_NOACCESS on dedicated descriptor pages; retain device/runtime | Observation completed |
| Cleanup | Free descriptor pages, release caller device reference, then drop FfxLibrary | Cleanup and observation completed |

Separate successful controls identified provider `4.1.1`, returned OK from create
and destroy, and again left the handle unchanged after destruction. The record
preserves input fingerprints, reproduction commands and the linked event log.

**Source/binary distinction:** the open FSR3 path allocates internal state and
initializes its first backend interface before rejecting the duplicate. The
signed runtime may reject it earlier; neither its failure depth nor the failed
call's provider identity or supplying module was established. Successful control
identity cannot resolve those unknowns.

**Inference and limits:** this is consistent with no continuing native use of
caller inputs on this tested rejection path. Unlike the allocation crashes, it
exercises the proposed caller cleanup order after an actual returned error.
Survival does not establish rollback, leak freedom, absence of internally handled
access faults, later asynchronous use, COM destruction or provider DLL unload.
No debug layer or exception debugger was active. Other failures, callbacks,
providers and threading remain outside the evidence.

**D005 implication:** no observation contradicts ordinary RAII for this request.
[D005](../DECISIONS.md#d005--own-effect-independent-context-lifecycles-with-terminal-teardown)
now accepts it for the supported runtime path as an explicit, revisable runtime
trust assumption, not a provider-independent guarantee. The evidence limits
above remain unchanged; this is no longer a general M4 blocker. No public context
ownership implementation follows. Repeating this wait or adding a staged cleanup
matrix cannot establish the missing universal contract. Further investigation
would need provider-specific source/contract evidence or an independently
justified deeper returned-failure path.

## Older Rust bindings: useful patterns and limits

Both comparisons concern standalone FSR2 2.2.0, whose caller-allocated opaque
context storage differs from v2.3.0's pointer handle. They inform wrapper design,
not the modern SDK's native contracts.

| Concern | Embark `fsr-rs` | NotAPenguin0 `fsr2-rs` |
|---|---|---|
| Ownership | Non-copyable high-level context retains interface and scratch storage. | Thin raw layer exposes copyable opaque context storage; no owner wrapper. |
| Teardown | Unsafe `destroy(&mut self)` neither consumes nor invalidates the object; no context `Drop`. | Free create/destroy functions leave all live/dead state to callers. |
| Failure handling | Returns create errors without wrapper rollback; DX12 interface initialization discards its native result. | Exposes native errors directly; partial-create cleanup remains unresolved. |
| ABI | Generated raw bindings still permit copying native state. | Handwritten declarations are auditable, but closed error enums, callback nullability/unsafe typing and enum widths need scrutiny. |
| Threading | Explicit interface/scratch `Send + Sync` propagate to context without an identified native guarantee. | Opaque storage gains auto traits; descriptor has explicit `Send`, without establishing native concurrency safety. |

Sources: [Embark report](records/2026-09-20-src-embark-fsr-rs.md), pinned to
`2ec24b8abae1d6013d444cd472fa9712c584a21b`, and
[fsr2-rs report](records/2026-09-20-src-fsr2-rs.md), supplemented by the
[provenance repair](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md):
parent `3a928595f46818f7221f0ca28f00770b53ded850` is a high-confidence historical
reconstruction; its native gitlink is `35d136728c49b5c866517b906d4405b7bee583da`.

**Inference for this project:** retain backing storage and native dependencies
with the context owner, prevent duplicate ownership and post-destroy use, preserve
native errors, and audit callback types and layouts against the pinned ABI.
A constructor borrow alone does not encode a context-long dependency. Neither
comparison supplies a sound ready-made destruction/failure policy. Embark's old
DX12 backend `AddRef`/`Release` behavior does not resolve modern v2.3.0 device
ownership; the old Vulkan paths likewise establish no new backend support here.
These are design inputs, not accepted API or binding-strategy decisions.

## Disagreements and limitations

No discrepancy was observed within the experimentally checked ABI/query subset.
The lifecycle source report identifies documentation conflicts about the required
version node, post-destroy nulling, and chain-node lifetime. Runtime evidence now
includes successful context lifecycles, host-allocation injection crashes and
one returned-error input-lifetime experiment;
it confirms non-nulling but leaves chain-node lifetime unresolved. The documented
permission to return null from allocation contrasts with crashes on the tested
path; the separate duplicate-backend rejection returned normally and caller
cleanup caused no observed fault within two seconds. This does not establish a
general failure cleanup contract. No upscaling
dispatch or image output was tested. No 32-bit or other-platform claim follows. The local header was
fingerprinted but its correspondence to the upstream release tree was not
independently authenticated in this work.

[D003](../DECISIONS.md#d003--handwrite-the-first-common-abi-slice) authorizes
handwritten bindings only for this slice; successful tests do not settle the
full-SDK binding strategy. The Rust module is production-intended implementation;
the native probe is verification evidence. Evidence records preserve the state
at investigation time and do not independently accept architecture changes.

The ABI checks themselves offer no licensing conclusion. Current notice,
acquisition and signed-runtime distribution findings are maintained in the
[licensing and distribution synthesis](licensing-and-distribution.md), including
the distinction between source-built components and the supplied signed runtime.

**Provenance refinement:** the
[repair record](records/2026-09-22-src-lifecycle-fsr2-provenance-repair.md)
supplies full-commit AMD citations at
`60f4ea81909200d8542eca14dccb2628b763a9a3` for the older lifecycle record's
missing URLs. Its two already populated URLs are locally confirmed as v2.3.0
links. The repair reports verification of the fsr2-rs parent's native gitlink;
the parent's identity on 2026-09-20 remains a high-confidence inference from
matching 36-commit history, not a dated branch snapshot. Representative pinned
sources support the earlier findings without a reported substantive discrepancy.
The repair did not have the full historical records and is not a complete prose
re-audit. Original records remain historical; the supplementary record carries
the corrections. Neither repaired citations nor open source resolve the signed
provider's implementation or establish runtime behavior.

## Open questions

- **M5 source and runtime follow-up:** the later provenance repair supplies
  pinned source links and corrects broad earlier claims, but does not recover
  every original claim-to-source mapping. Zero components aborted on one signed
  configuration; a later three-width probe returned generic error `1` below,
  at and above the D3D12 Texture2D width limit without identifying the rejection
  cause or an accepted maximum. The numerical source investigation and these
  probes establish no complete safe domain; [D008](../DECISIONS.md#d008--bound-the-first-public-dx12-upscaler-construction-contract)
  remains proposed, and public safe construction awaits further evidence or a
  reviewed trust decision. Resolve provider-specific helper rounding, output
  retention, global debug scoping and runtime/module lifetime where selected public
  operations require them. The two-context sequential result does not resolve
  general concurrency or choose an API.
- **Policy selected, broader questions open:** D007 selects explicit acquisition
  and executable-adjacent upscaler deployment for the tested v2.3.0 DX12 baseline;
  automatic runtime discovery is not part of that policy. Other effect artifacts,
  runtime coexistence and dependency closure remain open. Distribution terms
  remain separate from the accepted acquisition/deployment policy.
- **Provenance:** exact historical fsr2-rs branch identity remains inferred;
  the repaired source set is not a complete audit of every historical claim.
- **Source clarification and design:** chain-node lifetime, signed/external 4.1.1
  device COM ownership,
  callback/threading guarantees, and universal failure-state contracts. D005 now
  selects failed-create cleanup and exceptional dependency retention under its
  explicit runtime assumption; these remaining research gaps do not reopen that
  M4 policy by themselves. New contradictory evidence requires policy review.
- **Follow-up investigation after authorization:** locate the observed allocation-
  failure faults and distinguish loader/provider/driver behavior. Establish a
  deeper returned-failure path or obtain provider-specific lifetime evidence;
  the duplicate-backend spike now covers one returned error and bounded caller
  cleanup, but not a universal contract. Replacement allocator compatibility remains
  untested. For destroy, first establish a documented legitimate failure trigger
  or obtain provider-specific evidence: the new source report found no supported
  valid-context trigger, and a null outer pointer cannot answer liveness. Do not
  corrupt private state to manufacture one, blindly destroy failed-create output,
  or retry failed destruction without a contract.
  Check library/context lifetime invariants without deliberate use-after-free.
- **Later source work and experiments:** result-array queries, GPU resource and
  synchronization contracts, and dispatch. Successful context creation does not
  establish executed upscaling. Stress tests cannot establish an absent threading guarantee.
- **Policy selected, implementation deferred:**
  [D006](../DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)
  establishes handwritten bindings as the default and permits reviewed generated
  exceptions with equivalent ABI verification, including for first-class support.
  Concrete additional slices and any generation setup remain to be selected.
