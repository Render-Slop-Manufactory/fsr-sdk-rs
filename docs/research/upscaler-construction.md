# Upscaler construction

- Updated: 2026-09-30 (reorganized; evidence dates remain as recorded).
- Scope: public construction, numerical trust, provider sharing and M5 evidence.
- Overview: [API and safety](api-and-safety.md).

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
provider for those failed cases were not established. The M4 private
constructor already used `NonZeroU32`; the M5 public `Dimensions::new` rejects
zero before native creation. For this tested signed path, zero input
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
is accepted with an explicit bounded numerical runtime trust assumption. M5
now exposes safe construction from checked dimensions, a borrowed DX12 device
and an unsafely acquired `Runtime`. Windows-target compilation and host/Windows
ownership fixtures pass. The public path's native lifecycle evidence is
synthesized below; it does not establish a complete nonzero numerical domain.

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
research is required for D008. M5 implements the selected contract without
claiming that these observations establish a universal safe numerical domain.

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

**Integration outcome:** these source records and experiments informed, but did
not themselves select, the public API. Accepted D008 subsequently selected
runtime sharing, device retention and thread confinement; M5 implements those
choices. The earlier raw two-context experiment did not verify the new public
path; the later experiment below does so within its tested configuration.

### Public M5 construction verification

**Verified:** the [M5 Windows experiment](records/2026-09-24-exp-m5-windows-native-verification.md)
ran the production `Runtime` and `Upscaler` path against the signed SDK v2.3.0
loader/upscaler DLL pair on Windows x64/MSVC, an RX 9060 XT and driver
`32.0.31041.1004`. Ordinary tests, paired C++ ABI compilation and the native
helper build passed. Two isolated native cases each created A and B on one DX12
device, dropped the public runtime and caller device, destroyed A, queried live
B for provider `0xf5a5ca1e01001001` / `4.1.1`, then cleaned up B. Explicit B
destruction and B `Drop` each exited `0` without timeout. This adds public
wrapper execution to the earlier raw shared-runtime and private M4 lifecycle
evidence. The native run exercised one nonzero size pair and no Dispatch.

**Ownership observation:** the loader remained resident while B lived and was
no longer resident after B cleanup in both cases. The test's internal runtime
ownership assertions passed, and B remained usable after the caller's runtime
and device values were dropped. This does not independently measure COM
reference counts. The earlier raw shared-runtime experiment observed the
*upscaler provider DLL* remaining resident after its loader owner was dropped;
the M5 test checked the *API loader DLL*. Their residency results concern
different modules and do not conflict. Neither probe identifies every selected
implementation or establishes a general module-unloading rule.

**Limits:** calls were sequential, and each case ran once on one configuration.
No native concurrency, broader input domain, dispatch, image correctness,
performance, or other GPU/provider compatibility follows. The first two M5
runner attempts stopped at local PowerShell command availability and restricted
OS inventory access before reaching native execution; the record retains them
separately from the passing hardware cases.
