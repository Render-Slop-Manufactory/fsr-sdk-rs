# Source synthesis: proposed M6 DX12 dispatch contract

- Intake date: 2026-09-24; investigation date was not supplied.
- Scope: repository inspection and AMD FSR SDK `v2.3.0` source analysis for a proposed first Windows/DX12 FSR4 dispatch API.
- Provenance: supplied handoff analysis. Its opaque inline citation tokens had no index-to-source map and were removed at intake. The pinned primary-source links at the end and the existing [M6 source records](../upscaling-dispatch.md) provide context; this intake did not independently reverify every claim or run a GPU dispatch.
- Status: recommendations and inferred safety obligations, not an accepted API decision or implementation result.

The supplied analysis recommends this M6 boundary: **one real DX12 FSR4 dispatch path, deliberately narrow, with `unsafe fn dispatch`, borrowed Windows COM interfaces, strict pre-call validation, fixed resource conventions, and no queue/fence ownership.**

## 1. Current repository state relevant to M6

The workspace contains only `fsr-sdk-sys` and `fsr-sdk` (`Cargo.toml:3-7`). `fsr-sdk` explicitly says dispatch is not exposed (`crates/fsr-sdk/src/lib.rs:3-4`); `Upscaler` currently owns a `NativeContextOwner`, retains its own `ID3D12Device`, pins the creation descriptor chain, and is neither `Send` nor `Sync` (`upscaler.rs:45-60`).

`Upscaler::new` is safe, accepts a borrowed `ID3D12Device`, clones/AddRefs it, and currently creates the context with `flags: 0` (`upscaler.rs:75-90,106-131`). `NativeContextOwner` assumes its private API creates no GPU work, and therefore destroys immediately in `Drop` (`context.rs:31-42,93-145`). That assumption must explicitly change in M6.

`fsr-sdk-sys` is closer to dispatch-ready than the high-level crate: `PfnFfxDispatch` already exists (`api.rs:102-107`) and the DLL loader already resolves/forwards `ffxDispatch`, with an appropriately broad unsafe contract (`loader.rs:259-275`). What is missing is the dispatch payload/resource ABI: `upscale.rs` currently contains only creation/version descriptors (`upscale.rs:9-31`).

The error model is deliberately open over native `u32` values, but `Operation` only has `Create` and `Destroy`; M6 needs `Dispatch` plus deterministic validation errors (`error.rs:5-25`). The supplied analysis did not rerun workspace tests because `cargo` was unavailable in its environment. Its conclusions are repository inspection and source-based recommendations, not fresh test execution.

## 2. Conclusions already established by the repository research

The repository's M6 research is substantially correct and should be taken as baseline:

- **REQUIRED:** `ffxDispatchDescUpscale` contains seven inline `FfxApiResource` values, not pointers to resource descriptors. Exposure/reactive/transparency resources can therefore be represented by zero/null `FfxApiResource`s. AMD v2.3.0 confirms the exact descriptor shape.
- **REQUIRED:** `ffxApiGetResourceDX12` is a `static inline` header helper, not a DLL export. Rust must reproduce the required conversion internally. It derives the FFX description from `ID3D12Resource::GetDesc()`, including dimensions, format and UAV usage.
- **REQUIRED:** M6 needs real submission/fence/readback verification; `ffxDispatch == FFX_API_RETURN_OK` proves only CPU-side acceptance/recording, not successful GPU execution.
- **REQUIRED:** FSR4 no longer requires reactive or transparency/composition masks, so they do not need to enter the first dispatch API.
- **DESIGN CHOICE:** use automatic exposure and omit the explicit exposure image. This requires changing current creation flags from zero to `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE = 1 << 5`; AMD both documents and uses that flag in its v2.3.0 sample.
- **REQUIRED:** motion vectors describe current→previous motion; `motionVectorScale` converts the application's representation to FSR's screen-space range. AMD's documented NDC example uses render width/height as the scale.
- **REQUIRED:** jitter supplied to FSR is the actual subpixel offset used to render the frame, in pixel units; custom sequences should avoid `(0,0)`. `frameTimeDelta` is milliseconds and `reset` is a temporal-history discontinuity pulse.

This means M6 does not need Query/Configure, mask generation, or a general GPU abstraction.

## 3. Safety-invariant classification

Here A = statically enforceable by Rust ownership/types, B = pre-dispatch runtime validation, C = external D3D12/GPU/rendering obligation, D = private ABI concern.

| Invariant | Class | Domain / M6 handling |
|---|---|---|
| Upscaler context uniqueness / no CPU use-after-destroy | A | Existing owner model |
| Device COM lifetime owned by upscaler | A | Existing cloned `ID3D12Device` |
| Command-list/resource COM lifetime during the CPU call | A | Borrowed `&ID3D12…` |
| CPU-side concurrent dispatch on one upscaler | A | `&mut self`, existing `!Send + !Sync` |
| Command-list type | B | Require `DIRECT`; check `GetType()` |
| Command list belongs to same device | B | `GetDevice` + COM identity check |
| Resources belong to same device | B | Same |
| Texture dimension / sample count / mips / array size | B | `GetDesc()` |
| Resource formats | B | Restrict M6 to fixed formats |
| Render/output dimensions | B | Derive from textures; cross-check + context maxima |
| Output UAV capability | B | Check `ALLOW_UNORDERED_ACCESS` |
| Same COM resource supplied in two roles | B | Reject exact identity duplicates |
| Command list is currently open/recording | C | D3D12 has no useful query proving this |
| Actual resource state equals declared FFX state | C | Caller obligation; D3D12 state is not introspectable |
| Backing-memory aliasing | C | Exact-object duplicates can be rejected; heap aliasing cannot generally be inferred |
| Resource residency / external queue hazards | C | Caller |
| Resource lifetime through GPU completion | C | Caller after unsafe dispatch |
| Command allocator lifetime through GPU completion | C | Caller |
| Queue submission / fence / cross-queue waits | C | Caller |
| Upscaler/context lifetime while FSR work is outstanding | C | Caller |
| Runtime/DLL lifetime while FSR work is outstanding | C | Follows from keeping `Upscaler` alive |
| Temporal execution order between frames | C | Caller |
| Reset pulse correctness | C | Image/temporal correctness, not Rust memory safety |
| MV direction/units/jitter convention | C | Algorithmic correctness |
| Camera/depth convention | C | Algorithmic correctness, with numerical validation in B |
| Exposure/masks | B/C | Fixed M6 profile eliminates them from public input |
| Native tags/layout/discriminants/null-resource representation | D | `fsr-sdk-sys` only |

The important distinction is that a bad jitter vector normally causes bad reconstruction, whereas destroying a resource or context while recorded GPU commands still use it crosses an FFI/native-lifetime safety boundary.

## 4. Recommended dispatch safety model

**SAFETY-REQUIRED: `Upscaler::dispatch` should be `unsafe fn`.**

A safe immediate-return method over temporary resource borrows would imply more than Rust can enforce: once it returns, safe Rust could drop the textures or `Upscaler` before the submitted GPU work completes. Merely `Clone`ing the COM interfaces inside `dispatch` does not solve this unless those clones are retained until a known fence value completes; likewise, a command-list wrapper does not solve allocator/context lifetime.

A safe API becomes plausible only if this crate owns or tracks submission/fence completion and retains every required COM/context dependency until that point. That is deliberately outside M6.

The exact contract should be approximately:

```rust
/// Records one FSR upscale dispatch into `dispatch.command_list`.
///
/// # Safety
///
/// The caller must ensure that:
/// - `command_list` is currently recording and remains valid until it has been
///   closed/submitted as required by D3D12;
/// - each input resource is in the documented M6 pre-state and no other queue
///   mutates or aliases its backing storage without correct synchronization;
/// - the output resource is in its documented M6 pre-state;
/// - all referenced resources and backing allocations remain alive until every
///   GPU command recorded by this dispatch has completed;
/// - the command allocator backing the list is not reset or destroyed before
///   that GPU work completes;
/// - this `Upscaler`, and therefore its context, device, provider and runtime
///   library, remain alive until that GPU work completes;
/// - dispatches using this upscaler execute in temporal order and are not made
///   to overlap in an unsupported way.
///
/// The caller is responsible for queue submission, fences and cross-queue
/// synchronization.
pub unsafe fn dispatch(
    &mut self,
    dispatch: UpscaleDispatch<'_>,
) -> Result<(), Error>;
```

Motion-vector, jitter and camera semantics belong in ordinary API documentation, not in `# Safety`, because violating them generally damages image quality rather than Rust memory safety.

## 5. Concrete public Rust API

**DESIGN CHOICE:** do not build a resource wrapper framework. Use the exact Windows interfaces already used by the crate, derive physical metadata from `GetDesc()`, and add only semantic types that eliminate plausible unit/convention mistakes.

```rust
pub use windows::Win32::Graphics::Direct3D12::{
    ID3D12Device,
    ID3D12GraphicsCommandList,
    ID3D12Resource,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JitterOffsetPixels {
    x: f32,
    y: f32,
}

impl JitterOffsetPixels {
    pub fn new(x: f32, y: f32) -> Result<Self, Error>;
    pub fn x(self) -> f32;
    pub fn y(self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTimeMillis(f32);

impl FrameTimeMillis {
    pub fn new(value: f32) -> Result<Self, Error>;
    pub fn get(self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParameters {
    near_plane: f32,
    far_plane: f32,
    vertical_fov_radians: f32,
    view_space_to_meters: f32,
}

impl CameraParameters {
    pub fn new(
        near_plane: f32,
        far_plane: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
    ) -> Result<Self, Error>;
}

#[non_exhaustive]
pub struct UpscaleDispatch<'a> {
    pub command_list: &'a ID3D12GraphicsCommandList,
    pub color: &'a ID3D12Resource,
    pub depth: &'a ID3D12Resource,
    pub motion_vectors: &'a ID3D12Resource,
    pub output: &'a ID3D12Resource,
    pub jitter: JitterOffsetPixels,
    pub frame_time: FrameTimeMillis,
    pub camera: CameraParameters,
    pub reset_history: bool,
}

impl Upscaler {
    pub unsafe fn dispatch(
        &mut self,
        dispatch: UpscaleDispatch<'_>,
    ) -> Result<(), Error>;
}
```

`JitterOffsetPixels` prevents projection/NDC translations from looking interchangeable with FSR's pixel-space jitter. `FrameTimeMillis` prevents the common seconds-vs-milliseconds error. `CameraParameters` makes radians and view-space scale explicit and allows finite/range validation. AMD's native descriptor itself specifies milliseconds, positive `preExposure`, radians for vertical FOV, and a view-space-to-metres factor.

I would **not** expose `motion_vector_scale` in M6. Fix the profile to low-resolution, jitter-free current→previous motion vectors encoded as normalized/screen-relative displacement and derive `(render_width, render_height)` internally. Likewise fix `preExposure = 1.0`, sharpening off, dispatch flags zero, and omit exposure/masks. This removes several invalid combinations without pretending to solve general FSR integration.

For the initial validated profile:

- `color`: `DXGI_FORMAT_R16G16B16A16_FLOAT`, render dimensions.
- `depth`: `DXGI_FORMAT_R32_FLOAT`, render dimensions.
- `motion_vectors`: `DXGI_FORMAT_R16G16_FLOAT`, render dimensions.
- `output`: `DXGI_FORMAT_R16G16B16A16_FLOAT`, output dimensions, UAV-capable.
- all: Texture2D, one mip, one array slice, sample count 1, distinct backing resources.
- inputs: documented `NON_PIXEL_SHADER_RESOURCE` boundary state → `FFX_API_RESOURCE_STATE_COMPUTE_READ`.
- output: use the official sample-established shader-readable boundary state initially; encode the corresponding FFX state, and make this one item an explicit native-validation gate before freezing the contract.

`Error` should gain `Operation::Dispatch` and `Error::InvalidDispatch(DispatchValidationError)`. Native failures remain losslessly represented as `Error::Native { operation: Dispatch, code }`.

## 6. Exact `fsr-sdk-sys` additions

**REQUIRED:** no new function-loader symbol is required; `PfnFfxDispatch` and `FfxLibrary::ffxDispatch` already exist.

Add to the raw ABI:

```rust
#[repr(C)]
pub struct FfxApiFloatCoords2D {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct FfxApiResourceDescription {
    pub r#type: u32,
    pub format: u32,
    // three 32-bit anonymous-union storage slots:
    pub width_or_size: ...,
    pub height_or_stride: ...,
    pub depth_or_alignment: ...,
    pub mipCount: u32,
    pub flags: u32,
    pub usage: u32,
}

#[repr(C)]
pub struct FfxApiResource {
    pub resource: *mut c_void,
    pub description: FfxApiResourceDescription,
    pub state: u32,
}
```

The raw binding should model the three C anonymous unions as `#[repr(C)] union`s rather than inventing a semantically different layout. AMD's v2.3.0 definition is exactly eight 32-bit descriptor slots followed by the outer resource pointer/state.

Add `ffxDispatchDescUpscale` in the exact upstream field order from header through `flags`; do not reorder the optional resources.

Minimal constants needed by this M6 profile are:

```text
FFX_API_DISPATCH_DESC_TYPE_UPSCALE       = 0x0001_0001
FFX_UPSCALE_ENABLE_AUTO_EXPOSURE         = 1 << 5

FFX_API_RESOURCE_TYPE_TEXTURE2D          = 2
FFX_API_RESOURCE_FLAGS_NONE              = 0
FFX_API_RESOURCE_USAGE_READ_ONLY         = 0
FFX_API_RESOURCE_USAGE_UAV               = 2

FFX_API_RESOURCE_STATE_COMPUTE_READ      = 4
FFX_API_RESOURCE_STATE_PIXEL_COMPUTE_READ= 12

FFX_API_SURFACE_FORMAT_UNKNOWN           = 0
FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT= 4
FFX_API_SURFACE_FORMAT_R16G16_FLOAT      = 18
FFX_API_SURFACE_FORMAT_R32_FLOAT         = 28
```

Those resource-state/usage values come directly from the normative header.

Do not bind AMD's complete format universe merely to implement these four textures. Internally accept these three DXGI formats and emit their matching FFX values; AMD's helper confirms that `GetDesc()` is the canonical source for texture type, extents, mip count, format and UAV usage.

Required x64 ABI assertions from the repository synthesis are:

```text
FfxApiFloatCoords2D:       size 8,   align 4
FfxApiResourceDescription: size 32,  align 4
FfxApiResource:            size 48,  align 8
  resource      @ 0
  description   @ 8
  state         @ 40

ffxDispatchDescUpscale:    size 432, align 8
  header                    0
  commandList              16
  color                    24
  depth                    72
  motionVectors           120
  exposure                168
  reactive                216
  transparencyAndComposition 264
  output                  312
  jitterOffset            360
  motionVectorScale       368
  renderSize              376
  upscaleSize             384
  enableSharpening        392
  sharpness               396
  frameTimeDelta          400
  preExposure             404
  reset                   408
  cameraNear              412
  cameraFar               416
  cameraFovAngleVertical  420
  viewSpaceToMetersFactor 424
  flags                   428
```

These must be confirmed by the repository's paired MSVC fixture against the pinned AMD headers before merging; they should not be accepted merely because Rust's calculated layout happens to match the synthesis.

## 7. Rust-to-ABI conversion

The conversion should be one narrow internal function:

```text
UpscaleDispatch
  ↓
validate semantic values
  ↓
validate DIRECT command list + same device
  ↓
GetDesc() on four resources
  ↓
validate format / Texture2D / 1 mip / 1 slice / sample=1 /
exact extents / output UAV / context maxima / exact duplicate objects
  ↓
construct four stack FfxApiResource values
  ↓
construct three zero/null optional FfxApiResource values
  ↓
construct stack ffxDispatchDescUpscale
  ↓
NativeContextOwner::dispatch(...)
  ↓
FfxLibrary::ffxDispatch(&mut context, &desc.header)
  ↓
Error::Native { operation: Dispatch, code } or Ok(())
```

All descriptor structs, dimensions and null-resource values are stack temporaries whose CPU lifetime ends after `ffxDispatch`. The COM pointers inside `FfxApiResource` are borrowed raw pointers; the wrapper does **not** claim that FFX AddRefs them. AMD's helper itself simply places `pRes` into the resource structure and derives the remaining metadata from `GetDesc()`.

The descriptor should be filled with:

```text
exposure/reactive/transparencyAndComposition = null resources
motionVectorScale = { render_width, render_height }
renderSize        = derived render extent
upscaleSize       = derived output extent
enableSharpening  = false
sharpness         = 0
frameTimeDelta    = validated milliseconds
preExposure       = 1.0
reset             = caller value
camera*           = validated CameraParameters
viewSpaceToMetersFactor = validated caller value
flags             = 0
```

`Upscaler` should retain its `UpscalerOptions` maxima in addition to the owner so dispatch validation does not require reconstructing creation state. `NativeContextOwner` needs only a crate-private dispatch forwarding method and, preferably, a crate-private borrowed-device accessor for device-identity validation; do not expose the raw context handle.

## 8. GPU lifetime and destruction

Yes, after M6 it is mechanically possible for `Upscaler::drop` to occur while previously recorded FSR GPU work is still outstanding. The current comment that teardown is safe because “the private API submits no GPU work” (`context.rs:98-105`) therefore becomes false and must be rewritten.

**SAFETY-REQUIRED:** after a successful unsafe dispatch, destroying/dropping the `Upscaler` before GPU completion violates the caller contract. Rust cannot enforce the fence condition with the proposed immediate-return API.

The existing ownership topology otherwise remains correct: if the caller keeps `Upscaler` alive, it keeps the context, retained device, runtime state and loaded DLL alive. No new fence ownership should be invented.

`&mut self` is still useful: it rules out overlapping CPU calls through safe Rust. It does **not** prove that dispatch N has completed—or even executed—before dispatch N+1 is recorded/submitted.

Therefore M6 needs documentation changes, not a lifecycle redesign. A later safe execution API could return/own an in-flight object retaining resources/context until a fence completion, but that would be a different abstraction and should not be smuggled into this milestone.

## 9. Test plan

### Pure Rust / fake runtime

Extend the existing fake DLL/fixture so `ffxDispatch` snapshots the descriptor. Verify tag, command-list pointer, four resource pointers/descriptions/states, three null optionals, derived render/output sizes, derived MV scale, `preExposure = 1`, sharpening off, flags zero, camera/jitter/time/reset, and exact propagation of an unknown native error code.

Validation tests must prove the native dispatch counter remains zero for bad format, bad dimensions, arrays/mips/MSAA, output without UAV support, duplicate resource identity, dimensions exceeding context maxima, wrong command-list type/device, and invalid semantic scalar constructors.

### ABI

Use both Rust `size_of/align_of/offset_of` assertions and the existing paired native-fixture style. The C++ side should `static_assert` all struct sizes/alignments/offsets, `sizeof(bool) == 1`, descriptor ID/constants, and the `ffxDispatch` function-pointer signature against the pinned v2.3.0 headers.

Additionally, a small native parity check should compare this crate's generated M6 `FfxApiResource` fields with AMD's inline `ffxApiGetResourceDX12` for each of the four supported resource roles. That directly tests the one helper Rust is reproducing.

### Native DX12 experiment / real runtime gate

Use a small deterministic case such as 320×180 → 640×360:

1. Create RGBA16F color, R32F depth, RG16F motion-vector, and UAV-capable RGBA16F output textures.
2. Populate a deterministic spatial image, valid depth, zero/static motion or a controlled known displacement, and render/sample it using a nonzero known jitter.
3. Put resources in the exact documented M6 boundary states.
4. Record `dispatch(reset_history = true)` into an open DIRECT list.
5. Transition output to `COPY_SOURCE`, copy via a correctly pitched readback footprint, close, submit, signal and wait.
6. Keep allocator, resources, `Upscaler` and `Runtime` dependencies alive through the fence.
7. Verify no relevant D3D12 debug/GBV failure or device removal, output differs from its sentinel clear, contains finite/nontrivial spatial data, and is repeatable.
8. Only then destroy the upscaler.

“`ffxDispatch` returned OK” is explicitly not an M6 success criterion.

A second short temporal run should exercise at least two ordered frames plus reset behavior and a nonzero jitter sequence; this is where MV direction, jitter convention and history ordering stop being paper assumptions.

## 10. Ordered implementation plan

1. **SAFETY-REQUIRED:** write/accept the M6 ADR first: unsafe immediate-return dispatch, caller-owned fence lifecycle, fixed FSR4 resource profile, fixed auto exposure, and the post-dispatch destruction obligation.
2. **REQUIRED:** add only the dispatch/resource ABI to `fsr-sdk-sys`; add Rust + paired-MSVC layout/constants/signature tests.
3. **REQUIRED:** implement the private narrow DX12 resource converter from `ID3D12Resource::GetDesc()` and its role-specific validation.
4. **DESIGN CHOICE:** add `JitterOffsetPixels`, `FrameTimeMillis`, and `CameraParameters`; keep motion-vector scale and pre-exposure fixed internally.
5. **REQUIRED:** retain creation maxima in `Upscaler`, switch creation from `flags = 0` to fixed `AUTO_EXPOSURE`, and add `Operation::Dispatch`.
6. **SAFETY-REQUIRED:** add the crate-private owner dispatch bridge and public `unsafe fn Upscaler::dispatch`.
7. Extend the fake runtime and validation/error tests.
8. Add the native DX12 record/submit/fence/readback experiment and validate the selected resource-state boundary against the signed v2.3.0 provider.
9. Run the multi-frame temporal case, then update `ABI_COVERAGE.md`, milestone docs, README/roadmap, and lifecycle comments to state what has actually been verified.

Each stage can compile/test independently; in particular, do not publish the high-level method before the ABI fixture is green.

## 11. Work explicitly deferred from M6

**DEFERRED:** Vulkan; generalized GPU/resource abstractions; arbitrary texture formats and resource states; MSAA/arrays/mips; Query/Configure APIs; resource-requirements query as public API; manual exposure textures; reactive/transparency masks; sharpening; HDR/nonlinear-color options; inverted/infinite depth; display-resolution MVs; dynamic resolution; automatic jitter/MV generation; automatic command-list submission; queue/fence ownership; upload/readback APIs; compute-list support; multi-queue overlap; arbitrary same-context concurrent frames; placed-resource/heap aliasing support; and packaging/distribution automation.

None of those are required for the fixed FSR4 dispatch profile. FSR4's masks are explicitly optional, and auto exposure removes the need for an exposure resource.

## 12. Remaining blockers / unresolved questions

**UNKNOWN until native verification:** the exact public pre/post-state contract for the output texture. The research establishes the input recommendation and observes the official sample's broader shader-readable usage, but this is the one place where I would not freeze a generalized public state abstraction before the real 4.1.1 DX12 experiment.

**UNKNOWN until paired fixture runs:** the new Rust ABI layout is strongly determined by the headers and repository synthesis, but the native MSVC assertions still need to be executed.

**DESIGN CHOICE requiring an ADR:** current M5 deliberately fixes creation flags to zero; M6's smallest clean null-exposure path should instead fix `AUTO_EXPOSURE`. AMD recommends automatic exposure and its v2.3.0 sample enables it.

**UNKNOWN/provider-validation item:** the sample's handling of `viewSpaceToMetersFactor` is less explicit than the field contract. The M6 API should not depend on a zero default: require a positive explicit value and use `1.0` in the meter-based native fixture. The upstream descriptor defines it as the view-space-to-metres scale.

The normative upstream files used to resolve those points are pinned to commit `60f4ea81909200d8542eca14dccb2628b763a9a3`: [ffx_upscale.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h), [ffx_api_types.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h), [ffx_api_dx12.h](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h), [official DX12 FSR sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp), and [FSR4 integration documentation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md).
