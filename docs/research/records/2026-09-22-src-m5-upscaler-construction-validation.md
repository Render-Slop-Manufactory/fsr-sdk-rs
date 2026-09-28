# Source investigation: v2.3.0 upscaler construction validation

> Intake note (2026-09-22): renamed from `2026-09-22-rename_me_1.md`.
> The supplied source table identifies a commit and file paths but contains no
> direct source URLs; several implementation entries use descriptive names only.
> Findings below are preserved as received, not independently reverified at intake.
> This citation gap is tracked in the [current synthesis](../api-and-safety.md#m5-source-intake-and-provenance).

**Investigation date:** 2026-09-22  
**Target record:** `docs/research/records/2026-09-22-src-m5-upscaler-construction-validation.md`  
**Topic:** Public construction and validation contract for a modern FidelityFX upscaler context on DirectX 12.  
**Authoritative baseline:** AMD FidelityFX SDK v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.  
**Evidence cut-off:** Pinned commit above. Current upstream was inspected only for release/delta checking; no post-baseline behavior is projected backwards.

## Scope and method

This investigation covers only context creation through the modern FidelityFX API and the construction-time contract relevant to:

- `ffxCreateContextDescUpscale`;
- `ffxCreateContextDescUpscaleVersion`;
- `ffxCreateBackendDX12Desc`;
- creation flags;
- `maxRenderSize` and `maxUpscaleSize`;
- DX12 device requirements;
- API-version/provider-selection interaction;
- creation-time failure semantics.

Dispatch behavior is considered only where necessary to establish what the creation-time maxima mean. No GPU/runtime experiment was performed. Source acceptance is not treated as proof of behavior of the separately distributed signed runtime.

Evidence labels used below:

- **Verified** — directly supported by identified pinned primary source.
- **Upstream claim** — stated by AMD documentation but not independently reproduced here.
- **Inference** — follows from source/documentation but is not stated as a normative rule.
- **Unresolved** — public evidence does not establish the point.

## Pinned source inputs

All repository files below are from commit `60f4ea81909200d8542eca14dccb2628b763a9a3`, accessed 2026-09-22.

| Source | Purpose |
|---|---|
| `Kits/FidelityFX/api/include/ffx_api.h` — | Generic descriptor model, return codes, context creation, descriptor-pointer lifetime |
| `Kits/FidelityFX/api/include/ffx_api.hpp` — | C++ descriptor-chain linking used by samples |
| `Kits/FidelityFX/api/include/ffx_api_types.h` — | `FfxApiDimensions2D` representation |
| `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` — | DX12 backend descriptors |
| `Kits/FidelityFX/upscalers/include/ffx_upscale.h` — | Upscaler create descriptor, flags, version, dispatch dimensions |
| `Kits/FidelityFX/docs/getting-started/ffx-api.md` — | Public modern-API descriptor/version/provider documentation |
| `Kits/FidelityFX/docs/techniques/super-resolution-ml.md` — | FSR4/upscaler integration and version-descriptor requirement |
| `Kits/FidelityFX/docs/techniques/super-resolution-upscaler.md` — | FSR3 provider requirements |
| `Kits/FidelityFX/docs/techniques/super-resolution-temporal.md` — | FSR2 provider requirements |
| `Kits/FidelityFX/docs/whats-new/version_2_1_0.md` — | Introduction of mandatory upscaler-version descriptor; FSR4 alignment bug history |
| DX12 FSR sample `fsrapirendermodule.cpp` — | Official construction sequence |
| `api/internal/ffx_api.cpp` — | Provider selection and root-descriptor handling |
| `api/internal/ffx_provider.h` — | Provider enumeration/selection and device support filtering |
| `api/internal/ffx_backends.h` — | Required-backend helper |
| `backend/dx12/ffx_backends_dx12.cpp` — | Backend-chain validation, null/duplicate device handling |
| `backend/dx12/ffx_dx12.cpp` — | DX12 device retention and capability queries |
| FSR3 modern provider adapter — | Translation of modern creation descriptor into legacy FSR3 context |
| FSR3 implementation — | Allocation and maximum-dimension behavior |
| FSR3 implementation, dispatch validation — | Runtime dimensions versus creation maxima |
| SDK v2.3.0 release — | Release/tag cross-check |

The public repository does not expose the complete implementation of the signed FSR4/external provider. Consequently, open FSR2/FSR3 implementation behavior is evidence for those providers, not proof of signed FSR4 behavior.

## Findings

### 1. Creation descriptors and chain contract

#### `ffxCreateContextDescUpscale`

**Verified.** This is the root creation descriptor for the upscaler effect. It contains the generic descriptor header, creation flags, `maxRenderSize`, `maxUpscaleSize`, and optional message callback. Its `flags` field is explicitly documented as “zero or a combination” of the defined upscaler creation flags. `maxRenderSize` is described as the maximum render size, while `maxUpscaleSize` is the presentation-resolution target.

The root descriptor is materially different from extension nodes: `ffxCreateContext` uses the root descriptor's `type` to select an appropriate provider. **API validity rule.**

The generic API requires each descriptor header's `type` to match the actual concrete descriptor. AMD's API documentation explicitly states that supplying a descriptor with an incorrect type is undefined behavior and is likely to crash. **Safety requirement.**

#### `ffxCreateContextDescUpscaleVersion`

**Verified.** In v2.3.0 this node is not merely a sample convention. The FSR4/upscaler documentation states that, as of SDK 2.1, applications must create an `ffxCreateContextDescUpscaleVersion`, set its `version` member to `FFX_UPSCALER_VERSION`, and link it into the creation chain. The SDK 2.1 change log likewise says the descriptor “must be linked” during creation to ensure compatibility with future API updates.

`FFX_UPSCALER_VERSION` in this pinned header is 4.1.1. The field describes the version of the upscaler API against which the application was built; it is not the provider-selection ID.

Therefore:

- inclusion of the version descriptor is a **v2.3.0 API validity requirement**;
- its `version` value must be `FFX_UPSCALER_VERSION`;
- the exact failure code or behavior of the signed provider when this node is missing or contains another value is **Unresolved**.

The open FSR3 adapter does not consume this descriptor while translating its creation data. That implementation detail does not override the explicit SDK 2.1+ public requirement.

A documentation inconsistency exists inside the pinned tree: the generic `ffx-api.md` creation example shows only the upscaler root and backend descriptor. The more specific upscaler documentation and SDK 2.1 change log explicitly require the version node. The specific, later requirement controls this investigation; the generic example is incomplete for the v2.3 upscaler contract.

#### `ffxCreateBackendDX12Desc`

**Verified.** The DX12 backend descriptor contains an `ID3D12Device *` described as the device on which the backend runs. The official v2.3 DX12 FSR sample links this backend descriptor during context creation.

The open DX12 backend scanner rejects a recognized backend descriptor whose `device` is null with `FFX_API_RETURN_ERROR_PARAMETER`; it also rejects a second DX12 backend descriptor in the same chain with the generic `FFX_API_RETURN_ERROR`. The open FSR3 provider calls the backend-required helper, which fails creation if no backend can be constructed.

Thus a DX12 backend descriptor with a non-null valid device is a **Verified API validity requirement for the open DX12 provider path** and the construction form used by the official sample. The exact missing-backend behavior of every external/signed provider is not publicly inspectable.

#### Other applicable creation descriptors

`ffxCreateBackendDX12AllocationCallbacksDesc` is an optional DX12 extension allowing custom resource, heap, and constant-buffer allocation hooks. Individual callback pairs may be absent; the open backend installs resource/heap hooks only when the corresponding allocation/deallocation pair is complete.

`ffxOverrideVersion` is an optional generic extension used to request a particular provider implementation/version. Its ID is conceptually separate from `FFX_UPSCALER_VERSION`. Public documentation requires override IDs to originate from the version-query API rather than being hard-coded.

The `ffxAllocationCallbacks *memCb` argument to `ffxCreateContext` is not a `pNext` node. It may be null, in which case the generic API uses its default allocation behavior.

#### Descriptor ordering

**Verified:** the root upscaler descriptor must be the descriptor passed directly to `ffxCreateContext`; extensions hang from its `pNext` chain.

**Verified:** the official pinned DX12 sample constructs the chain in the order:

```text
ffxCreateContextDescUpscale
    -> ffxCreateBackendDX12Desc
    -> ffxCreateContextDescUpscaleVersion
    [-> ffxOverrideVersion]
```

because the C++ helper links the descriptor headers in argument order.

This differs from the project's currently successful:

```text
upscale -> version -> backend
```

The open generic/provider/backend implementation searches the chain for descriptors it needs instead of relying on the backend/version nodes occupying those particular relative positions.

No pinned public contract states that recognized extension nodes may be put in arbitrary order, however. Therefore:

- there is evidence that **version-before-backend is not a required ordering**;
- there is evidence that open implementations are insensitive to these two nodes' relative ordering;
- a general guarantee that arbitrary extension ordering is supported by every signed/external provider is **Unresolved**.

#### Duplicate and unknown nodes

| Situation | v2.3 evidence |
|---|---|
| Duplicate DX12 backend descriptor | **Verified:** open DX12 backend returns `FFX_API_RETURN_ERROR` rather than accepting a second backend. |
| Missing DX12 backend | **Verified for open FSR3:** backend-required helper returns an error. Exact behavior for every signed provider is not public. |
| Duplicate upscaler-version descriptor | **Unresolved.** No public rule or common failure behavior was found. |
| Duplicate version-override descriptor | **Source-supported:** generic source takes the first matching override encountered. No public duplicate-node contract was found. |
| Unknown extension descriptor | **Unresolved generically.** Public return codes include unknown-descriptor/provider-does-not-support-descriptor errors, while the open DX12 backend scanner simply skips unrecognized chain nodes. These facts do not establish one universal behavior. |
| Header `type` inconsistent with actual object layout | **Verified undefined behavior**, not a reliably rejectable input. |

#### Descriptor-data lifetime

`ffx_api.h` states broadly that pointers supplied through the creation descriptor must remain live until `ffxDestroyContext`. This is the strongest public lifetime statement and must be treated as a safety-relevant FFI contract.

There is some ambiguity concerning the storage of the extension descriptor nodes themselves. The official sample treats its provider override object as needing to live through the `CreateContext` call, while open providers synchronously scan/copy descriptor values rather than retaining the complete linked list.

Accordingly:

- validity of all pointers while native creation traverses the chain is a **Safety requirement**;
- lifetime of pointer-valued objects that the context/backend retains or can later call is a **Safety requirement**;
- whether the storage backing every `pNext` descriptor node itself must remain allocated until destruction, despite the sample's call-local usage pattern, is **Unresolved by the public wording**.

A non-null `fpMessage` can be retained and used by provider diagnostics, so any supplied function target must remain valid whenever native code may invoke it. Null is expressly permitted.

### 2. `maxRenderSize` and `maxUpscaleSize`

#### Public meaning

**Verified.** `maxRenderSize` is explicitly the maximum rendering size supplied at creation. `maxUpscaleSize` is the presentation-resolution target. At dispatch, `renderSize` represents the actual rendering resolution; `upscaleSize` may specify an output size and otherwise defaults to the creation-time maximum upscale size.

`FfxApiDimensions2D` itself contains only two unsigned 32-bit members, `width` and `height`; the generic type defines no minimum, maximum, alignment, or relationship constraint.

#### Zero dimensions and minimum dimensions

No pinned public upscaler header or documentation found in this investigation explicitly states a numerical minimum such as `width > 0`, `height > 0`, `>= 2`, or another provider-independent minimum.

The open FSR3 implementation uses the creation maxima to construct native resources, including resources whose dimensions are derived from fractions of `maxRenderSize`. A zero or sufficiently tiny size can therefore reach invalid or zero-sized underlying resource requests. That supports the conclusion that arbitrary zero/tiny maxima are not generally useful inputs, but it does not establish a provider-independent public numerical minimum.

Classification:

- “both creation dimension pairs must be non-zero” — **Unresolved as a generic documented API rule**;
- zero/tiny values may validly fail native/provider resource creation — **source-supported**;
- non-zero dimensions being sufficient for successful construction — **not established**.

Nothing found makes zero dimensions a Rust memory-safety prerequisite by themselves when a structurally valid descriptor is passed.

#### Maximum dimensions

No fixed numerical maximum for `maxRenderSize` or `maxUpscaleSize` is stated by the pinned public upscaler API.

The implementation allocates resources derived from the supplied maxima and therefore remains subject to provider, DX12 resource, device, memory, and implementation limits. Those constraints are not reduced to one public SDK-wide dimension constant.

A fixed provider-independent maximum is therefore **Unresolved/not specified**.

#### Alignment and granularity

No generic creation-time alignment or granularity requirement was found for either dimension pair.

The SDK 2.1 change log specifically records an FSR4 fix for a rendering error when surface dimensions were not multiples of eight. This is evidence against treating “multiple of 8” as a v2.3 FSR4 public validity requirement: the limitation was recorded as a defect that had been fixed, not as an API precondition.

No other general width/height divisibility rule was found.

#### Relationship between render and upscale dimensions

The public upscaler API includes a `NATIVEAA` quality mode with a 1.0 scale ratio, and the pinned FSR3/FSR4 material describes Native AA operation. Therefore equal render and upscale dimensions are an intended configuration for providers supporting Native AA.

FSR2's pinned provider documentation lists the traditional upscale ratios but does not advertise the same Native AA mode. This is provider-specific and must not be generalized from FSR3/FSR4 to FSR2.

The open FSR3 dispatch implementation independently checks actual render size against `maxRenderSize` and actual upscale size against `maxUpscaleSize`. It does not contain an explicit `renderSize <= upscaleSize` rejection before calculating scale factors. Lack of such a check is not a public guarantee that downscaling (`render > upscale`) is supported.

Therefore:

- `render == upscale`: **Verified intended usage for Native-AA-capable providers**;
- `render < upscale`: normal documented upscale usage;
- `render > upscale`: **Unresolved as a public supported configuration**;
- fixed matching aspect ratios: no public restriction found, but unrestricted arbitrary aspect-ratio support is likewise not explicitly guaranteed.

#### Are the values exact sizes or upper bounds?

For render resolution, the public name/documentation and FSR3 source are consistent: `maxRenderSize` is an upper bound, not necessarily the exact size of every later render. FSR3 rejects actual render dimensions greater than this creation maximum.

For output resolution, the public description calls `maxUpscaleSize` the presentation target, but the dispatch API can carry an explicit `upscaleSize`; the open FSR3 implementation permits explicit output dimensions up to the creation maximum and substitutes `maxUpscaleSize` when dispatch supplies zero for the optional output size.

Thus **FSR3 source supports variable render and output sizes below the creation maxima**. The closed signed FSR4 provider's exact validation path cannot be derived from this source.

#### Resolution changes and recreation

A requested size above a provider's creation-time maximum cannot be represented as a within-context FSR3 dispatch: the implementation explicitly rejects dimensions exceeding the corresponding maximum. A larger maximum therefore requires a new context for that provider.

Conversely, no generic v2.3 contract was found saying that *every* resolution change within the established maxima requires recreation. The API contains per-dispatch actual dimensions, and FSR3 explicitly handles values below its maxima.

The official sample recreates its context as part of its resize handling, but that sample policy is not proof of a universal native requirement.

### 3. Creation flags

The complete `ffxCreateContextDescUpscale::flags` set in the pinned public ABI is:

| Value | Public name | Contract/effect |
|---:|---|---|
| `1 << 0` (`0x001`) | `FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE` | Indicates HDR input/content. Provider documentation requires corresponding HDR signaling when HDR input is supplied. |
| `1 << 1` (`0x002`) | `FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS` | Motion vectors are supplied at display/upscale resolution rather than render resolution. |
| `1 << 2` (`0x004`) | `FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION` | Indicates motion vectors contain jitter cancellation/offset behavior described by the upscaler interface. |
| `1 << 3` (`0x008`) | `FFX_UPSCALE_ENABLE_DEPTH_INVERTED` | Indicates inverted-depth convention. |
| `1 << 4` (`0x010`) | `FFX_UPSCALE_ENABLE_DEPTH_INFINITE` | Indicates an infinite depth/far-plane convention. |
| `1 << 5` (`0x020`) | `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` | Requests provider-generated automatic exposure; affects whether an application exposure resource is necessary. |
| `1 << 6` (`0x040`) | `FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION` | Declares use of dynamic resolution scaling. |
| `1 << 7` (`0x080`) | `FFX_UPSCALE_ENABLE_DEBUG_CHECKING` | Enables additional diagnostic validation/checking. |
| `1 << 8` (`0x100`) | `FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE` | Indicates non-linear/perceptual color-space input. |
| `1 << 9` (`0x200`) | `FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION` | Enables debug-visualization facilities; header warns this can increase memory usage. |

The field documentation expressly permits **zero** or a combination of defined flag values. Therefore `flags = 0` is a documented syntactically valid combination.

That does not mean zero correctly describes every application's inputs. HDR, motion-vector resolution, depth convention, exposure mode, dynamic-resolution behavior, and color-space convention must agree with the resources/semantics presented to the selected provider. Such mismatches are functional/API-semantic issues; no public evidence found here turns an incorrect semantic flag into Rust memory unsafety.

No pinned public rule declares any two creation flags mutually exclusive.

Provider differences matter:

- FSR4 documentation expects linear input where possible and documents the non-linear-colorspace flag for cases where that cannot be supplied. It recommends inverted depth and recommends automatic exposure unless the application has a reason to provide its own exposure. Those recommendations are not generic hard validity constraints.
- The open FSR3 modern-to-legacy adapter maps creation bits 0 through 7 but does not propagate bits 8 or 9 into the legacy FSR3 creation flags. Thus behavior of those two flags is provider-specific in v2.3 rather than a uniform FSR3 feature.
- Debug checking is documented as a development diagnostic facility, not a prerequisite for a valid production context.

Use of unknown bits outside the defined mask is not documented as valid. The exact native rejection behavior for such bits is **Unresolved**.

### 4. DX12 device/backend requirements

#### Nullability and pointer validity

The backend descriptor declares an `ID3D12Device *`. The open DX12 backend explicitly checks the pointer and returns `FFX_API_RETURN_ERROR_PARAMETER` when it is null.

This produces two distinct classifications:

- non-null backend device: **API validity rule** for the inspected DX12 backend;
- the pointer actually referring to a valid compatible COM object and remaining safe for native use: **Safety requirement**.

A dangling, forged, or wrong-interface pointer is not an ordinary dimensions/flags validation error.

#### Device lifetime

The open DX12 backend calls `AddRef` on the supplied `ID3D12Device`, stores it in backend state, and releases it during backend destruction. This directly demonstrates that the in-tree backend uses the device beyond the `ffxCreateContext` call.

Separately, the public generic API states that pointer data passed during creation must remain live through context destruction.

The open backend's `AddRef` behavior does not justify weakening that public lifetime requirement for other providers, particularly an external/signed provider whose implementation is unavailable.

#### Feature level and capabilities

No pinned public DX12 backend header establishes one minimum `D3D_FEATURE_LEVEL` for every upscaler provider.

The backend queries device capabilities such as shader model, wave-lane properties, FP16 support, and ray-tracing support. Provider documentation imposes additional requirements:

- FSR2: Shader Model 6.2 baseline and documented texture/UAV-format capabilities, with higher shader-model behavior on some hardware.
- FSR3: similarly documents Shader Model 6.2 and resource-format/UAV requirements.
- FSR4 4.1.1: documents Shader Model 6.6 and a substantially narrower supported AMD GPU set.

These are **provider-specific compatibility requirements**, not generic requirements of `ffxCreateBackendDX12Desc`.

#### Can compatibility be queried before creation?

The generic version query accepts a device, and provider enumeration filters candidates through provider `CanProvide`/`IsSupported` logic. The public documentation also uses the device when querying available versions.

This is useful evidence of provider availability for a supplied device, but it is not a complete universal preflight test. In the open tree, for example, the FSR3 provider does not override the base `IsSupported` implementation even though its documentation separately lists GPU capability requirements.

Therefore “version/provider appears in a device-scoped query” must not be interpreted as proof that all dimensions, resources, flags, or subsequent operations are compatible.

A context can also encounter provider/resource requirements later that are not established merely by successfully creating a provider object. Creation success is not documented as a blanket guarantee for all possible future dispatch inputs.

### 5. API version versus provider version

Two different notions of version exist and must not be conflated.

`ffxCreateContextDescUpscaleVersion.version = FFX_UPSCALER_VERSION` communicates the **upscaler API contract version the application was built against**. At this baseline that macro is 4.1.1.

`ffxOverrideVersion.versionId`, by contrast, optionally selects a particular **provider implementation/version**. Public guidance requires those IDs to come from `ffxQuery` version enumeration rather than being hard-coded. Omitting an override allows provider selection to occur normally.

The v2.3 package contains multiple generations of upscaler providers; the pinned documentation identifies FSR2 2.3.4, FSR3 3.1.5, and FSR4 4.1.1 material. Their capability and validation contracts are not identical.

The open provider manager may consider internal and external/driver providers and uses device/provider support information while selecting or enumerating them. That implementation policy does not make the provider observed in one successful creation representative of every machine or provider configuration.

Consequently:

- API-version compatibility and provider selection are separate concerns;
- provider availability does not imply complete device/input compatibility;
- the project's observed 4.1.1 provider is compatible with the pinned upscaler versioning evidence, but does not establish behavior of FSR2, FSR3, other FSR4 builds, or driver/external providers.

### 6. Failure semantics relevant to construction

The generic modern API defines these return codes:

| Code | Name | Public meaning |
|---:|---|---|
| `0` | `FFX_API_RETURN_OK` | Success |
| `1` | `FFX_API_RETURN_ERROR` | Unspecified/general error |
| `2` | `FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE` | Unknown descriptor type |
| `3` | `FFX_API_RETURN_ERROR_RUNTIME_ERROR` | Underlying runtime/effect failure |
| `4` | `FFX_API_RETURN_NO_PROVIDER` | No provider available |
| `5` | `FFX_API_RETURN_ERROR_MEMORY` | Memory failure |
| `6` | `FFX_API_RETURN_ERROR_PARAMETER` | Invalid parameter, including null/empty/out-of-bounds examples |
| `7` | `FFX_API_RETURN_PROVIDER_NO_SUPPORT_NEW_DESCTYPE` | Provider does not support a newer descriptor type |

Specific inspected paths provide more precision:

- null root descriptor or context output pointer: `FFX_API_RETURN_ERROR_PARAMETER`;
- no selectable provider: `FFX_API_RETURN_NO_PROVIDER`;
- null device in a recognized DX12 backend descriptor: `FFX_API_RETURN_ERROR_PARAMETER`;
- duplicate DX12 backend descriptor: generic `FFX_API_RETURN_ERROR`;
- missing backend in the inspected provider path: generic error from the required-backend helper;
- underlying legacy FSR3 creation/resource failures passing through the modern adapter are converted to `FFX_API_RETURN_ERROR_RUNTIME_ERROR`.

Not all invalid construction inputs have a documented deterministic return code:

- incorrect descriptor `type` versus actual memory layout is explicitly undefined behavior, so ordinary error handling cannot be assumed;
- missing or malformed upscaler-version nodes have a documented contract requirement but no public signed-provider failure code;
- duplicate version descriptors have no documented behavior;
- unknown extension nodes do not have one observed universal path despite generic error constants existing;
- zero, tiny, unusually large, or otherwise unsupported dimensions do not have one documented generic creation return code;
- unsupported provider/device combinations may be rejected at provider selection, provider creation, underlying backend/resource creation, or later capability-sensitive operations.

Assertions in open implementation code are implementation evidence only. They do not establish the behavior of the signed release runtime in every build configuration.

## Construction-rule classification

| Material rule/question | Evidence | Classification |
|---|---|---|
| Root object is an actual `ffxCreateContextDescUpscale` and header `type` matches its concrete layout | Wrong descriptor type is explicitly UB/likely crash. | **Safety requirement** |
| All `pNext` pointers traversed during creation point to valid, correctly laid-out descriptor objects | Native code directly walks descriptor chains. | **Safety requirement** |
| Device pointer denotes a valid `ID3D12Device` object and remains valid according to native lifetime contract | Public pointer-lifetime rule; backend retains device with `AddRef`. | **Safety requirement** |
| Any supplied callback remains a valid callable target for the period native code can invoke it | Provider retains message callback state; generic pointer lifetime applies. | **Safety requirement** |
| Include `ffxCreateContextDescUpscaleVersion` | Explicitly mandatory since SDK 2.1. | **API validity rule** |
| Set version node to `FFX_UPSCALER_VERSION` | Explicit requirement in pinned public header/docs. | **API validity rule** |
| Exact native result when mandatory version node/value is absent/wrong | No signed-provider validation path is public. | **Unresolved** |
| Supply DX12 backend descriptor for inspected DX12 provider path | Official sample plus required-backend source. | **API validity rule** |
| DX12 backend `device != NULL` | Explicit source validation to `ERROR_PARAMETER`. | **API validity rule** |
| Root descriptor is chain head | Root type drives provider selection. | **API validity rule** |
| Version descriptor must precede backend descriptor | Official sample uses opposite order from project baseline; scanners search the chain. | **No such requirement established** |
| Arbitrary ordering of all valid extension descriptors is guaranteed | Not stated publicly; external provider unavailable. | **Unresolved** |
| Duplicate DX12 backend descriptors | Open backend explicitly rejects them. | **API validity rule** |
| Duplicate version nodes | No public behavior found. | **Unresolved** |
| Unknown extension nodes | Different components can ignore or reject; generic error codes exist. | **Unresolved** |
| Descriptor-node storage itself must remain allocated until context destruction | Header lifetime wording and sample call-local extension usage are not fully aligned. | **Unresolved** |
| `maxRenderSize.width/height > 0` | No explicit generic public numerical rule; provider resource creation implies zero is problematic. | **Unresolved** |
| `maxUpscaleSize.width/height > 0` | Same evidence situation. | **Unresolved** |
| Non-zero dimensions alone are sufficient | No source establishes this. | **Unresolved** |
| Fixed generic minimum dimensions | None found. | **Unresolved** |
| Fixed generic maximum dimensions | None found; device/provider/resource limits remain. | **Unresolved** |
| Dimensions must be multiples of 8 | FSR4 non-multiple-of-8 behavior was fixed as a bug before v2.3. | **Not a v2.3 generic validity rule** |
| Other fixed alignment/granularity rule | None found. | **Unresolved** |
| Render and upscale dimensions may be equal | Native AA 1.0 is public for supporting providers. | **API validity rule / supported provider mode** |
| Render dimensions may exceed upscale dimensions | FSR3 lacks an explicit rejection, but documentation does not guarantee downscaling. | **Unresolved** |
| Fixed aspect-ratio relationship | None found. | **Unresolved** |
| Later render dimensions may vary below `maxRenderSize` | Explicitly supported/validated by FSR3 implementation. | **API validity rule, provider-supported** |
| Later output dimensions may vary below `maxUpscaleSize` | Explicitly supported by FSR3 implementation. | **API validity rule, provider-supported** |
| Increasing beyond creation maxima without recreation | FSR3 rejects dispatch above maxima. | **API validity rule for FSR3** |
| Every resolution change requires recreation | API/source evidence does not support this blanket claim. | **Unresolved as a provider-general rule** |
| `flags = 0` | Header expressly permits zero. | **API validity rule: valid combination** |
| Only defined creation-flag bits are part of documented domain | Header defines “zero or combination” of enum values. | **API validity rule** |
| Exact rejection of unknown flag bits | No common path found. | **Unresolved** |
| Flags correctly describe HDR/MV/depth/exposure/DRS/color-space semantics | Provider docs depend on those interpretations. | **API validity rule / functional contract** |
| Enable inverted depth | Strongly recommended by FSR4 but alternatives are supported. | **Ergonomic/recommended rule** |
| Enable automatic exposure | Recommended in FSR4 unless application supplies another exposure strategy. | **Ergonomic/recommended rule** |
| Enable debug checking in development | Diagnostic recommendation, not construction prerequisite. | **Ergonomic/recommended rule** |
| DX12 device must satisfy one SDK-wide fixed feature level | No such generic fixed feature level found. Provider requirements differ. | **Unresolved as a generic rule** |
| Device satisfies selected provider's documented shader/resource capabilities | FSR2/3/4 each publish provider-specific requirements. | **API validity rule, provider-specific** |
| Provider returned by version query is necessarily usable for every valid-looking configuration | Provider query support checks are not exhaustive in open implementations. | **Unresolved / not established** |
| Use `ffxOverrideVersion` IDs only from a version query | Explicit public instruction. | **API validity rule** |

## Provider-specific boundaries

The public modern upscaler descriptor is shared, but the providers behind it do not have identical construction/capability contracts.

**FSR2:** pinned documentation identifies version 2.3.4 and documents its own shader/UAV/resource requirements and conventional upscale ratios. Native AA should not be inferred merely because newer upscaler providers expose it.

**FSR3:** pinned documentation identifies version 3.1.5. Its open implementation provides the strongest inspectable evidence for dimension maxima and variable dispatch sizes, but that evidence remains FSR3-specific where no public generic rule says otherwise.

**FSR4:** pinned documentation identifies 4.1.1, requires the API-version descriptor, and documents substantially narrower GPU/shader requirements. The actual signed/external provider's creation validation implementation is not present in the inspected public source, so exact rejection behavior cannot be reconstructed from the repository.

The existence or selection of one provider therefore cannot establish construction validity for another provider.

## Baseline delta

### Confirmed

The supplied project baseline is consistent with the pinned public ABI in the following respects:

- `ffxCreateContextDescUpscale`, `ffxCreateContextDescUpscaleVersion`, and `ffxCreateBackendDX12Desc` are all applicable descriptors for the modern DX12 upscaler creation path.
- The version descriptor must contain `FFX_UPSCALER_VERSION`; at the pinned baseline this is 4.1.1.
- `flags = 0` is explicitly a valid flag-field combination.
- `fpMessage = null` is permitted.
- null/default host allocation callbacks are supported by the generic creation API.
- the inspected DX12 backend requires a non-null device.
- the two creation dimension pairs are construction-time sizing inputs and are not merely descriptive metadata.
- retaining ownership of the DX12 device is consistent with both the generic pointer-lifetime contract and the fact that the open backend uses the device after creation.

### Contradicted

No supplied baseline claim is directly contradicted.

In particular, the project only states that its current chain is:

```text
upscale -> version -> backend
```

not that this ordering is mandatory. The pinned official sample actually links:

```text
upscale -> backend -> version
```

so any stronger interpretation that version-before-backend were required would be contradicted by the official v2.3 sample.

### Refined

The evidence materially refines the baseline in these areas:

1. The version node is **mandatory public v2.3 contract**, not merely a descriptor that happened to be present in the successful M4 experiment.

2. Relative ordering of the version and backend extension nodes is not established as a public requirement. Open source scans the chain and the official sample uses the opposite order from the project's current chain. A blanket arbitrary-order guarantee remains unproven.

3. The current non-zero dimension checks remain a defensible project-side precondition but are **not established by pinned public evidence as the complete native dimension contract**. Neither a generic minimum nor sufficiency of non-zero values was found.

4. `flags = 0` is structurally permitted, but zero correctly describes an application only when none of the flag-controlled input conventions/features apply.

5. `maxRenderSize` is demonstrably a maximum rather than a fixed dispatch render size. For FSR3, `maxUpscaleSize` also operates as an output ceiling even though the public field description calls it the presentation target.

6. Device-scoped provider enumeration is not sufficient evidence of every provider capability or later-operation requirement.

7. The open DX12 backend itself retains a COM reference, but that source observation does not supersede the generic public lifetime contract or establish behavior of every external provider.

### Left unresolved

Public v2.3 material does not settle:

- exact generic zero-dimension rejection rules;
- any fixed numerical creation minimum or maximum;
- a general alignment/granularity constraint;
- unrestricted aspect-ratio behavior;
- whether `maxRenderSize > maxUpscaleSize` is a supported generic use case;
- exact signed-FSR4 behavior for missing, duplicate, or incorrect version descriptors;
- behavior of duplicate version descriptors;
- a universal rule for unknown extension descriptors;
- a universal arbitrary-extension-order guarantee;
- the exact lifetime requirement for the storage of each `pNext` node after `ffxCreateContext` returns;
- one generic DX12 feature-level/capability threshold valid for FSR2, FSR3, FSR4, and external/driver providers;
- whether provider enumeration proves all creation-time compatibility;
- exact return codes for every invalid dimension or flag combination.

The successful M4 signed-runtime lifecycle establishes none of these unresolved cases.

## Current/upstream delta

As checked on 2026-09-22, AMD's public release listing still identifies FSR SDK v2.3.0 as the relevant released SDK, so no later released SDK contract was imported into this record.

One historical difference inside the pinned documentation is relevant: SDK 2.1 introduced the mandatory upscaler-version creation node and fixed an FSR4 issue with dimensions not divisible by eight. Those are historical explanations of the v2.3 contract, not newer behavior projected onto it.

## Limits and follow-up

The principal evidence boundary is the signed/external FSR4 provider: its complete creation validator is not available in the inspected public repository. Open FSR3 behavior can clarify the shared API's mechanics but cannot establish undocumented FSR4 rejection rules.

If the unresolved cases become material to safe-construction guarantees, narrow experiments against the exact signed v2.3.0 Windows/DX12 runtime would be needed. The smallest useful tests would be:

1. independently vary each creation dimension through `0`, `1`, a normal non-multiple-of-eight value, equality between render/upscale maxima, and `maxRenderSize > maxUpscaleSize`, recording provider identity and `ffxReturnCode_t`;
2. compare otherwise-identical chains with the version node absent, with an incorrect version value, with two version nodes, and with backend/version node order swapped;
3. if exact invalid-flag rejection matters, test one otherwise-unused bit outside the defined creation mask.

A deliberately mismatched descriptor `type`, dangling pointer, malformed object layout, or other case already documented as undefined behavior should **not** be used as a validation experiment.

Those tests would characterize the signed runtime only. They would not retroactively turn provider-specific runtime behavior into a public v2.3 source contract.

The source-supported construction boundary is therefore narrower than “all fields non-zero and creation succeeded”: descriptor structural correctness and pointer validity are safety obligations; the version node and valid DX12 backend are explicit API requirements; many dimension and provider-compatibility limits are intentionally or effectively delegated to native/provider validation; and several edge conditions remain undocumented in publicly available v2.3 material.
