# Source investigation: the five upscaler Configure controls

- Investigation/source-access date: 2026-10-15.
- Question: what contract can support typed access to the five tuning controls,
  and which safety and policy decisions remain before implementation?
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, after D027;
  SDK v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Method: inspect pinned primary headers, loader, open providers, FSR3 setter,
  dispatch/backend copying, official sample, and current Rust ownership/errors.
- Exclusions: no SDK upgrade, implementation, accepted API decision, runtime
  loading, compilation, GPU probe, image-quality or performance experiment.
- Outcome: mapping and the open FSR3 contract are clear. FSR2 silently ignores
  these controls while returning OK. Closed-provider lifetime, synchronization
  and failure guarantees remain unresolved. Research recommendations below are
  not accepted design or native support claims.

## Inputs and provenance

All upstream links identify the pinned commit and were accessed on the date above.
Physical line anchors were checked against downloaded source, rather than folded
browser line counts.

| Source | Inspected evidence |
|---|---|
| [Shared upscaler header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h#L167-L183) | Descriptor, five keys, published ranges/defaults |
| [Common API header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h#L150-L155) and [Configure routing](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api.cpp#L88-L109) | Entry point, global/context distinction and provider routing |
| [FSR3 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp#L180-L213) | Key/value forwarding and descriptor rejection |
| [FSR2 adapter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_provider_fsr2.cpp#L132-L150) | Silent OK for the unhandled tuning descriptor |
| [FSR3 setter](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp#L1547-L1610), [declaration](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/include/ffx_fsr3upscaler.h#L571-L585) and [private context](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler_private.h#L77-L96) | Float consumption, null reset, context storage, invalid-key behavior |
| [FSR3 dispatch source](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/fsr3/internal/ffx_fsr3upscaler.cpp) and [DX12 staging](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/backend/dx12/ffx_dx12.cpp#L2833-L2856) | Creation defaults, history reset, per-dispatch constant-buffer copying |
| [CPU scalar helpers](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/gpu/ffx_core_cpu.h#L163-L194) and [error helper](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_api_helper.h) | Clamping implementation and `TRY2` error translation |
| [Official sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp#L1263-L1269) | Stack float passed by address; return code discarded |
| [Provider infrastructure](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/internal/ffx_provider.h) and [FSR4 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md) | Limits of available external-provider implementation and public guidance |

Twelve source files were retrieved directly over HTTPS into ignored
`target/research/configure-controls-sources/`; a local manifest retains URLs,
byte lengths and SHA-256 identities. Initial sandboxed retrieval failed DNS;
the authorized repetition succeeded. Additional helpers were read through the
web tool; sample/guide and a complete non-truncated pinned tree were reused from
the preceding source investigation. No native binaries were downloaded.

Selected downloaded SHA-256 identities:

- `ffx_upscale.h`: `65be36d5653eb71c10379e0204f7bb7cd6a8aeec96e6f8cffc5f59ffeb1f89bf`.
- `ffx_fsr3upscaler.cpp`: `0fd97a1016043cf3ec1b50c2112fc16f6f8702930ed54a166649a1d956a41e1a`.
- `ffx_provider_fsr3upscale.cpp`: `0572529154e534849823d86a7792f179deec89ef283c341b5917d696a69d2fde`.
- `ffx_provider_fsr2.cpp`: `7d878685169946962c607cc0abb1eba833a6b2da650c77610e2630e03696d4d3`.
- `ffx_dx12.cpp`: `3f6ffa9079caa62a10abb4e108fc833c34b555bc1affcec0007dbc623536be77`.

The cache is not needed to understand this record. The linked public source and
findings below establish the evidence boundary independently of local artifacts.

## Findings

### 1. Mapping is small; values are passed by address

**Verified source facts:** `ffxConfigureDescUpscaleKeyValue` contains the header,
64-bit `key`, 64-bit `u64` and `void* ptr`. The five keys are 0–4. On the inspected
FSR3 route, `ptr` points to a readable `float`; it does not encode float bits in
the pointer address. `u64` is unused on this route. The sample supplies an address
of a local float and a zero-initialized descriptor.

Context-specific Configure routes through the context's associated provider.
It is not a runtime-wide quality preset or a main dispatch parameter. The generic
entry point handles global debugging separately; those callbacks remain outside
this investigation's proposed implementation scope.

| Key | Control | Published clamping domain | Open FSR3 creation/null-reset value |
|---:|---|---|---|
| 0 | Velocity factor | `[0, 1]` | `1.0f` |
| 1 | Reactiveness scale | `[0, +∞)` | `1.0f` |
| 2 | Shading-change scale | `[0, +∞)` | `1.0f` |
| 3 | Accumulation added per frame | `[0, 1]` | `1.0f / 3.0f` |
| 4 | Minimum disocclusion accumulation | `[-1, 1]` | `-1.0f / 3.0f` |

The header's last two defaults are rounded to ±0.333. **Inference:** writing
those decimal literals is not an exact restoration of the inspected FSR3
defaults. A provider-default operation should be distinct from assigning a
wrapper-invented numeric default.

### 2. FSR3 copies persistent values; frame reset is different

**Verified source facts:** the FSR3 setter immediately dereferences a non-null
float pointer, clamps the value and stores it in the context's constants. Null
selects that key's compiled default. It retains no value pointer and makes no
backend or GPU-recording call on this path. Creation initializes the same five
fields at lines 617–621. The five fields are subsequently written only by the
setter in the inspected file.

Dispatch stages those constants at line 1127. The DX12 staging function copies
the bytes into its staging storage. The `resetAccumulation` paths reset temporal
history/jitter state without reinitializing the five tuning fields. Therefore
these overrides persist across frames and frame-history reset in this open
implementation. Configure does not itself request a temporal reset.

**Unresolved:** this does not establish signed-provider non-retention, identical
null-reset semantics, GPU completion requirements or visibility timing. The
sample's stack pointer is corroborating usage, not a universal lifetime contract.

### 3. OK is not support detection

**Verified source facts:** the open FSR3 adapter forwards the descriptor to the
five-key setter on the Windows path. An unknown descriptor returns
`FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE`. An invalid setter key fails before any
tuning assignment; `TRY2` maps that internal failure to public runtime error
`3`, losing the internal invalid-enum detail. Rust should preserve the public
code it actually receives rather than reconstructing a more specific one.

The open FSR2 Configure switch handles only the old global-debug descriptor,
has no rejecting default branch and returns OK afterward. A valid context and
the tuning descriptor therefore reach **OK without applying any tuning**.
During this investigation an initial progress message called this rejection;
inspection of the complete function corrected that statement. This record
retains the corrected source result.

**Inference:** a call returning OK establishes only native acknowledgment.
Neither successful trial calls, existing resource masks nor opaque provider
IDs/names supply a reliable five-control capability predicate. A wrapper must
not expose a cached value as the provider's observed effective state. The current
public query inventory offers no readback of these tuning constants.

**Unresolved:** FSR4/signed/driver providers' actual five-key, null-reset,
clamping and failure behavior is not established by the FSR3 implementation.
The external provider include refers to an `amdinternal` implementation absent
from the inspected complete public tree. No universal guarantee was found in
the inspected guide/header. No provider allowlist is justified here.

### 4. Numeric validation and failures need explicit policy

**Upstream claim:** finite out-of-range values are clamped into the table's
domains. The open setter implements this with the CPU min/max helpers. Those
helpers use comparisons rather than a finite-value check; NaN can propagate.
The two unbounded scales can also retain positive infinity. A mathematical
`+∞` bound does not establish a safe GPU numerical domain for literal infinity.

**Recommendation:** reject NaN/infinity in Rust. Then explicitly choose between
rejecting finite out-of-range values and forwarding them unchanged for native
clamping. The latter preserves the documented direct-control behavior; strict
domain types provide a narrower default. Do not silently clamp in Rust or turn
the controls into automatic quality policies under [D016](../../adr/D016.md).

The inspected FSR3 setter's invalid-key failure leaves its tuning fields alone;
FSR2's ignored descriptor also changes none. **Unresolved:** neither establishes
error atomicity or safe continuation for arbitrary providers. Several single-key
calls are not a transaction; if one succeeds and the next fails, earlier changes
must not be reported as rolled back. No native batch/rollback API is provided.

### 5. GPU ordering is a safety boundary, not a solved source fact

**Inference:** serialized FSR3 Configure followed by dispatch stages the updated
CPU constants; changing those CPU fields does not rewrite a prior staging copy.
This is narrower than permission to configure an arbitrary provider with earlier
work still in flight. The staging ring has its own finite-storage behavior, and
the setter itself contains no synchronization primitive.

**Recommendation:** preserve existing common-domain serialization and movable
thread obligations; do not interpret `&mut self` as GPU synchronization. An
initial API can require an idle, fenced context for configuration. That is a
conservative wrapper precondition, not a discovered universal AMD requirement.
It belongs at an unsafe boundary or in explicitly revised acquisition/dispatch
obligations; a safe method cannot silently assume the renderer already fenced.
Stack descriptors/values also need actual-provider synchronous non-retention.

## Baseline delta and decisions remaining

Current raw code already resolves and exposes `ffxConfigure` in
[`loader/windows.rs`](../../../crates/fsr-sdk-sys/src/loader/windows.rs).
The high-level [`runtime state`](../../../crates/fsr-sdk/src/runtime/state.rs)
and [`context owner`](../../../crates/fsr-sdk/src/context.rs) have no Configure
route; [`Operation`](../../../crates/fsr-sdk/src/error.rs) has no Configure case,
and the tuning descriptor/keys are not bound. Both ownership families would
need the same shared route. Current query descriptors are already covered;
there is no demonstrated additional ordinary upscaler query slice in these headers.

This confirms the synchronous FSR3/null-reset findings in the
[earlier Query/Configure investigation](2026-09-22-src-m5-upscaler-query-configure.md)
and [API inventory](2026-09-29-src-dx12-upscaler-api-surface.md). It adds exact
default/reset persistence analysis and the concrete FSR2 silent-OK limitation.
The earlier suggestion that trial-call return codes distinguish support must
be narrowed to acknowledgment/rejection; they cannot prove application.
Historical records and current syntheses are not rewritten by this investigation.

Before implementation, select:

1. **Safety boundary:** safe methods under explicitly extended unsafe runtime/
   dispatch trust, or typed unsafe context methods with call-specific provider,
   non-retention and GPU-idleness obligations. The latter is the smaller initial
   boundary; the former follows the conditional-trust pattern of
   [D018](../../adr/D018.md) but needs an accepted safety-contract change.
2. **Value policy:** finite domain validation versus finite passthrough to native
   clamping. Represent native-default reset separately; do not hardcode ±0.333.
3. **Failure policy:** safe continuation requires an explicit actual-provider
   contract. Conservative terminal eligibility after a native Configure error
   avoids assuming rollback; its effect on configure/dispatch/live queries,
   already recorded work and teardown must be specified. Local validation errors
   should remain pre-native and leave eligibility unchanged. Do not automatically
   copy the failed-command-list rules from dispatch: Configure receives no list.

**Recommended scope:** one typed control per call, an explicit native-default
selection, shared family handling, exact public errors and documented
acknowledgment semantics. No generic public pointer/key escape hatch, global
logging callback, cached effective-value getter, automatic reset, batch rollback
or provider-name heuristic is needed. These are recommendations, not acceptance.

## Limits and follow-up

Source work can proceed to the API/safety decision without Windows. Implementation
would need paired raw Rust/C++ descriptor/key checks and controlled fixtures
covering exact float/null forwarding, ignored-OK behavior, validation, failures,
poison eligibility and both families. None of those new tests ran here.

Later Windows work should separately observe each selected provider's return
codes, reset/persistence, safe recording/completion and cleanup. An OK call or
finite output alone cannot prove tuning application. Any application claim needs
provider-specific instrumentation or a justified isolated semantic observation;
this does not resume held perceptual-quality studies or impose a quality gate.
No finite probe can prove universal pointer non-retention or threading safety.
Those remain documented native trust requirements if not supplied upstream.

No runtime, native, GPU or performance result is claimed. SDK pin, dependencies,
platform support and redistribution policy are unchanged. Raw source snapshots
stay under ignored `target/`; no AMD source or binaries are added to distribution.
This investigation ends with this record; synthesis, roadmap status, accepted
design and implementation are separate steps.
