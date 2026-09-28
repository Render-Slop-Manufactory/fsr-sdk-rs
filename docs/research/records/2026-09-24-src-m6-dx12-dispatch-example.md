# Source investigation: M6 DX12 dispatch example and GPU readback

> Intake note (2026-09-24): renamed from `rename_me_2.md`. Opaque
> `:chatgpt-content-reference` markers had no supplied index-to-source map and
> were removed. The substantive findings and pinned source list were retained;
> this intake does not independently verify every claim.

The normative baseline is AMD FSR SDK **v2.3.0 / commit `60f4ea8`**. In that release the public upscaler ABI is **4.1.1** (`FFX_UPSCALER_VERSION_MAJOR=4`, minor 1, patch 1), and the DX12 sample is `Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp`.

## 1. Official execution trace

**VERIFIED — provider/backend/context creation.** `FSRRenderModule::UpdateFSRContext(true)` constructs `ffx::CreateBackendDX12Desc`, sets `device = GetDevice()->GetImpl()->DX12Device()`, then constructs `ffx::CreateContextDescUpscale` with `maxRenderSize`, `maxUpscaleSize`, and flags. The sample normally enables auto-exposure, HDR and debug-visualization; inverted/infinite depth, dynamic resolution, nonlinear color and debug checking are conditional. It also supplies `ffxCreateContextDescUpscaleVersion.version = FFX_UPSCALER_VERSION`. Finally it calls `ffx::CreateContext(m_UpscalingContext, nullptr, createFsr, backendDesc, headerVersion [, versionOverride])`. The VRAM/version queries around this are sample diagnostics and are not prerequisites for dispatch.

The C++ `CreateContext` wrapper merely chains the descriptor headers and invokes `ffxCreateContext`; this is convenience code, not another semantic layer. At the core API level, `ffxCreateContext` selects a provider using descriptor type, optional version override and device, then invokes that provider's `CreateContext`. The DX12 backend descriptor contains only the header plus `ID3D12Device*`; backend creation converts that device to an FFX device and obtains the DX12 `FfxInterface`.

**VERIFIED — per-frame setup.** The sample queries the upscaler context for jitter phase count and jitter offset. The resulting jitter is applied to the camera projection, while the dispatch descriptor later receives its negative: `{-m_JitterX,-m_JitterY}`. This query machinery is useful for a representative temporal test, but a first smoke experiment can use a controlled zero-jitter frame.

Immediately before the actual upscaler, Cauldron copies its color target into a separate temporary input texture so that input and output are different resources. It transitions source/destination to `COPY_SOURCE`/`COPY_DEST`, copies, and returns both to its generic shader-read state.

**VERIFIED — actual dispatch.** `FSRRenderModule::Execute` constructs `ffx::DispatchDescUpscale`, fills it, and calls:

`ffx::Dispatch(m_UpscalingContext, dispatchUpscale)`

which reduces to `ffxDispatch(&context, &descriptor.header)`. The C entry point performs parameter checks and immediately calls the associated provider's `Dispatch`.

Critically, the API documentation explicitly says GPU dispatches **encode commands into the supplied command list**. They do not submit that list themselves. Thus an `FFX_API_RETURN_OK` proves CPU-side acceptance/recording, not GPU execution.

---

## 2. Resource inventory

The ABI has exactly these external resources for an upscale dispatch: `color`, `depth`, `motionVectors`, optional `exposure`, optional `reactive`, optional `transparencyAndComposition`, and `output`.

| Resource | Minimal DX12 choice | Size | Dispatch state | Status |
|---|---|---|---|---|
| Color | `R16G16B16A16_FLOAT` is a conservative test choice | render | `NON_PIXEL_SHADER_RESOURCE` (or sample's combined pixel+compute read) | required |
| Depth | `R32_FLOAT` texture is simplest; sample renderer may use a depth-target format | render | shader read | required |
| Motion vectors | `R16G16_FLOAT` | render unless display-resolution-MV flag | shader read | required |
| Exposure | 1×1 floating exposure texture | 1×1 | shader read | optional; **omit with AUTO_EXPOSURE** |
| Reactive | `R8_UNORM` | render | shader read | optional/provider-dependent |
| Transparency/composition | `R8_UNORM` | render | shader read | optional/provider-dependent |
| Output | `R16G16B16A16_FLOAT`, `ALLOW_UNORDERED_ACCESS` | upscale | imported in its pre-dispatch state | required |

The SDK documentation specifies application-defined color format, one-component floating depth, two-component floating motion vectors and render/presentation resolution rules; it also explicitly requires DX12 inputs to be in `NON_PIXEL_SHADER_RESOURCE` before dispatch. The sample config explicitly makes both masks `R8_UNORM`; its mask textures also have UAV capability because the sample can generate them.

**VERIFIED — masks can be omitted.** The sample passes a null `FfxApiResource` when reactive masks are disabled and independently does the same for transparency/composition. The public API now also exposes `ffxQueryDescUpscaleGetResourceRequirements`; therefore the most robust standalone test should query the selected provider after creation and verify what it considers required/optional rather than hard-code assumptions across FSR2/3/4 providers.

**VERIFIED — exposure can be null in the official path.** The sample enables `FFX_UPSCALE_ENABLE_AUTO_EXPOSURE` and passes `nullptr` as `dispatchUpscale.exposure`.

**INFERRED — synthetic inputs.** Color, depth, MV and masks have no requirement that they originate from rasterization. Consequently committed textures uploaded with deterministic values are sufficient for an ABI/GPU reconstruction experiment. For the first frame use a spatially varying color image, constant physically consistent depth, zero MV, no masks, and auto-exposure. Avoid a constant color input because it makes "nontrivial reconstruction" difficult to distinguish from trivial behavior.

The public `FfxApiResourceDescription` contains dimensions, mip count, format, usage and type but **no sample-count field**. The official path is therefore naturally based on resolved single-sample textures; use `SampleDesc.Count=1`, one mip, one array slice for the experiment.

---

## 3. Dispatch descriptor construction

The exact v2.3.0 ABI is at `Kits/FidelityFX/upscalers/include/ffx_upscale.h:78-103`. Do not import fields from old FSR2 structs.

The official sample writes:

| Field | Sample value | Character |
|---|---|---|
| `commandList` | raw `ID3D12GraphicsCommandList*` | per dispatch |
| `color` | temporary pre-upscale color | resource |
| `depth` | `DepthTarget` | resource |
| `motionVectors` | `GBufferMotionVectorRT` | resource |
| `exposure` | null | constant because auto-exposure enabled |
| `output` | main color target | resource |
| `reactive` | mask or null | option/resource |
| `transparencyAndComposition` | mask or null | option/resource |
| `jitterOffset` | `(-m_JitterX,-m_JitterY)` | per frame |
| `motionVectorScale` | `(RenderWidth,RenderHeight)` | resolution-dependent |
| `reset` | manual reset OR camera reset | per frame |
| `enableSharpening` | UI RCAS setting | option |
| `sharpness` | UI value | option |
| `frameTimeDelta` | `deltaTime*1000` | per frame, milliseconds |
| `preExposure` | scene exposure | per frame/scene |
| `renderSize` | current render resolution | resource/frame |
| `upscaleSize` | current upscale resolution | resource/frame |
| `cameraFovAngleVertical` | camera vertical FOV, radians | camera |
| `cameraFar` | camera far plane | camera |
| `cameraNear` | camera near plane | camera |
| `flags` | debug-view/nonlinear-colorspace flags | options |

All of those assignments occur consecutively in `FSRRenderModule::Execute`.

One subtle point: `viewSpaceToMetersFactor` exists in the v2.3.0 descriptor, but the upscaler sample does **not** explicitly write it, so the C++ zero-initialized descriptor leaves it `0`. That is direct evidence against inventing a value solely because the field exists.

For a minimal controlled test, use `enableSharpening=false`, `sharpness=0`, `flags=0`, `preExposure=1`, and `frameTimeDelta≈16.6667 ms`. The documentation explicitly states milliseconds and warns through the debug checker about implausibly small values.

---

## 4. D3D12/FidelityFX boundary

**VERIFIED.** Cauldron's helper is almost entirely transparent:

`SDKWrapper::ffxGetResourceApi(cauldronResource,state,additionalUsages)`
→ obtains its `ID3D12Resource*`
→ calls `ffxApiGetResourceDX12(...)`.

It only adds one Cauldron-specific correction for buffer stride. Textures receive no additional magic.

`ffxApiGetResourceDX12` itself:

1. stores the native resource pointer and caller-specified FFX state;
2. calls `ID3D12Resource::GetDesc()`;
3. converts DXGI format to `FfxApiSurfaceFormat`;
4. fills width/height/depth-or-array-size/mips/type;
5. infers depth-target usage from depth formats;
6. adds UAV usage if `D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS` is present.

Therefore **VERIFIED**: Rust can populate `FfxApiResource` directly if it reproduces this mapping faithfully. There is no registration, allocation, descriptor creation or synchronization hidden in this helper. For M6, however, reproducing the tiny helper semantics in `fsr-sdk-sys` or an internal high-level conversion routine is less error-prone than exposing users to raw description construction.

The command-list conversion is even thinner: the sample simply extracts the underlying `ID3D12GraphicsCommandList*` and assigns it to the ABI's `void* commandList`.

---

## 5. Post-dispatch execution path

**CORROBORATED.** `ffxDispatch` records FidelityFX GPU work into the application-supplied command list; it does not execute that command list or signal/wait a fence. This follows both the API documentation and the sample architecture.

The sample also notes an important side effect: FidelityFX changes the currently set resource-view heaps, so Cauldron calls `SetAllResourceViewHeaps(pCmdList)` afterward. That restoration is **sample-renderer state repair**, not required by a standalone program that has no subsequent engine rendering.

The surrounding command allocator/list creation, reset, close, queue execution and frame fencing live in Cauldron rather than `FSRRenderModule`. Thus they are framework mechanics, not FidelityFX API requirements. For standalone DX12 the equivalent is conventional:

`allocator->Reset()` → `list->Reset()` → barriers/uploads → `ffxDispatch` → readback barrier/copy → `list->Close()` → `queue->ExecuteCommandLists()` → `queue->Signal(fence)` → wait.

No FidelityFX-created descriptor heap needs to be installed by the application. Indeed, the fact that the sample restores *its own* heap after FFX is evidence that the backend manages its descriptor binding internally.

Before destruction, GPU completion must be established by the application. Nothing in Rust ownership or `ffxDestroyContext` establishes that an earlier submitted command list has completed.

---

## 6. One-frame vs multi-frame conclusion

**VERIFIED:** the upscaler API is temporal; official documentation explicitly describes temporal accumulation/history, and the ABI has `reset`, motion vectors, jitter, depth and frame-time inputs for that reason.

**INFERRED, strongly:** a fresh-context first dispatch is nevertheless expected to produce an output. Otherwise the normal renderer could not display the first frame after context creation/reset. `reset=true` explicitly means discontinuous history, so absence of prior useful history is a supported operating condition. Nothing in v2.3.0 states that several dispatches are required before output becomes valid.

Therefore:

- **one dispatch is sufficient to prove actual GPU image reconstruction;**
- it does **not** prove useful temporal accumulation;
- the first frame should be interpreted as the upscaler's reset/no-history path;
- multiple frames are preferable only for the stronger "temporal behavior works" milestone.

For the first deterministic temporal extension, use 3–8 frames. Frame 0: `reset=true`, zero MV, first official jitter sample. Subsequent frames: `reset=false`, successive jitter samples. Move a sharp synthetic object by exactly e.g. one render-resolution pixel/frame and supply the matching motion vector and depth. Keep `frameTimeDelta=16.6667 ms`. Compare this against a deliberately zero-MV sequence. A measurable output difference demonstrates history/MV dependence without requiring a game renderer. The official sample obtains jitter through `GETJITTERPHASECOUNT` and `GETJITTEROFFSET`, so using those queries is preferable to inventing a Halton implementation.

---

## 7. Minimal standalone native experiment

The smallest source-backed experiment I would implement is:

```text
enable D3D12 debug layer

create ID3D12Device
create one DIRECT command queue
create one DIRECT allocator
create one DIRECT graphics command list
create fence + event

load amd_fidelityfx_loader_dx12.dll
resolve ffxCreateContext / ffxDispatch / ffxDestroyContext / ffxQuery

create:
    color     = renderW × renderH, RGBA16_FLOAT
    depth     = renderW × renderH, R32_FLOAT
    motion    = renderW × renderH, RG16_FLOAT
    output    = upscaleW × upscaleH, RGBA16_FLOAT, ALLOW_UAV

create upload resources
upload deterministic color/depth/MV
transition input textures -> NON_PIXEL_SHADER_RESOURCE
put output in a known state consistent with the FfxApiResource state

createBackend:
    device = ID3D12Device*

createUpscale:
    maxRenderSize  = render size
    maxUpscaleSize = output size
    flags = AUTO_EXPOSURE
            [+ DEBUG_CHECKING during bring-up]

createUpscaleVersion:
    version = FFX_UPSCALER_VERSION

ffxCreateContext(... createUpscale -> backend -> version ...)

optionally query resource requirements
assert selected provider accepts null exposure/reactive/transparency

build FfxApiResource values from native ID3D12Resource descriptions

build ffxDispatchDescUpscale:
    commandList = graphics command list
    color       = color
    depth       = depth
    motionVectors = motion
    exposure    = null
    reactive    = null
    transparencyAndComposition = null
    output      = output

    jitterOffset = {0,0}             // first smoke test
    motionVectorScale = {renderW,renderH}
    renderSize = {renderW,renderH}
    upscaleSize = {upscaleW,upscaleH}
    enableSharpening = false
    sharpness = 0
    frameTimeDelta = 16.6667
    preExposure = 1
    reset = true
    cameraNear = chosen near plane
    cameraFar  = chosen far plane
    cameraFovAngleVertical = chosen FOV in radians
    viewSpaceToMetersFactor = 0       // matches sample zero-init
    flags = 0

ffxDispatch
assert FFX_API_RETURN_OK

transition output -> COPY_SOURCE
CopyTextureRegion(output -> readback buffer footprint)

close command list
ExecuteCommandLists
Signal fence
wait for fence

Map(readback)
validate pixels

only after fence completion:
    ffxDestroyContext
    release runtime/DLL/device resources
```

This deliberately omits a swapchain, RTVs, scene renderer, UI, frame generation, reactive-mask generation, presentation and Cauldron's resource classes. The descriptor fields and resource conversion come directly from the v2.3.0 sample/API.

For initial dimensions I would use something moderate such as **320×180 → 640×360**. Tiny textures may exercise provider/model constraints that are irrelevant to the wrapper test.

---

## 8. Output validation plan

For readback, use the standard DX12 texture-to-buffer path: obtain the output texture's copyable footprint, allocate a READBACK-heap buffer of the required total size, transition output to `COPY_SOURCE`, and issue `CopyTextureRegion` from texture subresource 0 to the placed footprint. Only map after the queue fence proves completion.

The important detail is that CPU rows are separated by the returned `Footprint.RowPitch`, not `width * bytesPerPixel`. Iterate `height` rows using that pitch.

For `R16G16B16A16_FLOAT`, don't make PNG writing part of the first pass. Decode IEEE binary16 values directly and compute:

- number/fraction of finite pixels;
- min/max/mean luminance;
- nonzero pixel count;
- a hash over canonicalized raw output bytes or decoded values;
- optionally MSE against a CPU nearest-neighbor copy of the input.

That avoids introducing a half-float→8-bit color conversion as another possible failure source. If an inspectable image is wanted, convert finite linear RGB half floats to float, clamp/tonemap as appropriate, encode sRGB, then write an 8-bit PNG. The FFX output itself does not need to be a conventional image-file format.

The M6 gate should have four distinct assertions:

**API success:** `ffxDispatch == FFX_API_RETURN_OK`.

**GPU execution:** fence completes, `GetDeviceRemovedReason()==S_OK`, and debug/GBV produces no relevant errors.

**Actual output:** sentinel-clear the output before dispatch and verify a substantial fraction changed, values are finite, and there is meaningful spatial variance.

**Repeatability:** identical fresh-context runs give the same result or, if driver/model arithmetic prevents bit identity, metrics within a narrowly measured tolerance.

I would make **GPU execution + nontrivial deterministic output** the first M6 acceptance criterion. Temporal-history verification should be a subsequent test, because requiring it immediately couples the new Rust dispatch API to multi-frame resource lifetime and synchronization design before the basic dispatch boundary is proven.

---

## 9. What the AMD sample does that M6 can omit

The following are sample/framework concerns, not requirements of the basic upscale API: Sponza and its G-buffer renderer; lighting/translucency/particles; swapchain and presentation; tone mapping/UI; frame generation and its proxy swapchain; distortion fields; HUD-less buffers; version-selection UI; VRAM queries; RCAS controls; dynamic-resolution UI; automatic reactive-mask generation; copies made solely because Cauldron's renderer wants its existing color target to become the output; Cauldron resource wrappers; profiler markers; and restoring Cauldron descriptor heaps after dispatch.

The only useful pieces to preserve conceptually are: distinct input/output resources, correct states, real DX12 command list, exact `FfxApiResource` descriptions, correctly populated dispatch constants, queue submission, explicit GPU completion, and delayed destruction.

---

## 10. Open questions

**UNKNOWN — provider-specific resource requirements.** The ABI itself labels several resources optional, but v2.3.0 deliberately provides `ffxQueryDescUpscaleGetResourceRequirements`, presumably because requirements can vary with the selected provider/version. The experiment should query this rather than assume FSR4.1.1 has exactly the same optional set as FSR2/3.

**UNKNOWN — best golden-image tolerance across AMD GPU/driver generations.** Source establishes the algorithmic interface but does not promise bit-identical floating-point output across hardware/drivers. Start with invariants/statistics plus same-machine repeatability before committing a pixel-perfect golden hash.

**INFERRED — output state after FFX.** The application tells FFX the imported resource's state and the backend necessarily performs internal accesses/barriers while recording its passes, but the public contract is less useful as a general Rust state-tracking guarantee than the sample's simple "resources arrive in generic read" convention. For M6, treat FFX dispatch as an explicit state-boundary operation and validate the subsequent `COPY_SOURCE` transition under the D3D12 debug layer rather than building a general resource-state abstraction around undocumented assumptions.

**UNKNOWN — minimal dimensions accepted by the ML provider.** Nothing useful is gained by probing pathological tiny sizes in M6. Use a normal small image and make dimension-limit discovery a separate test.

For diagnostics, enable the ordinary D3D12 debug layer for every development run and GPU-based validation for the small standalone fixture; Microsoft specifically notes GBV catches shader descriptor/resource-state errors and that its diagnostics become available on the GPU timeline after execution. Enable GBV **before** device creation. Also enable FidelityFX `FFX_UPSCALE_ENABLE_DEBUG_CHECKING` with `fpMessage` during bring-up; AMD explicitly documents this facility for detecting invalid dispatch inputs. DRED is worth enabling/capturing for device-removal failures.

The key M6 architectural consequence is fairly narrow: **you do not need a GPU-resource framework to dispatch.** The minimum new boundary is essentially a borrowed raw DX12 command list plus borrowed DX12 resources with enough explicit state/metadata to construct `FfxApiResource`; correctness outside the call still requires the caller to keep those resources and the FFX context alive until the submitted GPU work has completed. That synchronization obligation should remain explicit rather than being disguised as an ordinary Rust lifetime.

Primary v2.3.0 sources: [upscaler ABI (`ffx_upscale.h`)](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h?utm_source=chatgpt.com) · [DX12 API helper (`ffx_api_dx12.h`)](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h?utm_source=chatgpt.com) · [official FSR DX12 sample](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Samples/Upscalers/FidelityFX_FSR/dx12/fsrapirendermodule.cpp?utm_source=chatgpt.com) · [Cauldron FFX/DX12 conversion helper](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/Cauldron2/dx12/framework/core/backend_interface.h?utm_source=chatgpt.com) · [core FFX API routing](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/api/internal/ffx_api.cpp?utm_source=chatgpt.com) · [DX12 backend creation](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/backend/dx12/ffx_backends_dx12.cpp?utm_source=chatgpt.com).
