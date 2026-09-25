# Milestone 6: minimal native DX12 dispatch proof

- Updated: 2026-09-25.
- Public-contract gate: [D011](adr/D011.md) accepted; M6b implementation and
  [public-route verification](research/records/2026-09-25-m6b-public-dispatch-verification.md)
  completed on one tested Windows/DX12 configuration.
- Status: M6a and M6b implemented and passed bounded Windows/DX12 GPU proofs with
  D3D12 debug layer; GPU-based validation timed out during `ffxDispatch` and
  remains inconclusive. See the [verification record](research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
  and [bounded GBV diagnostic](research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md),
  which isolated the timeout to the native call on the tested setup without
  diagnosing a provider or driver defect. This does not change M6a's bounded
  completion.
- Baseline: AMD FSR SDK `v2.3.0`, Windows/DX12, the unchanged public M5
  `Runtime::load` and `Upscaler::new` path.
- Basis: [M5 construction](MILESTONE_5.md),
  [dispatch source synthesis](research/upscaling-dispatch.md), and
  [dispatch contract recommendation](research/records/2026-09-24-src-m6-dispatch-contract-recommendation.md).
  That recommendation proposed a different auto-exposure profile; accepted
  [D011](adr/D011.md) owns the implemented public contract.

## Question and boundary

Can a context created by the public M5 constructor record an upscaler dispatch,
execute it on a DX12 GPU, and write a plausible output texture? M6 answers that
integration question for one recorded setup. It does not assess image quality or
establish a reusable renderer.

On the recorded setup, the dispatch ABI checks passed and the private production
dispatch path returned OK. The DX12 test harness submitted and fenced the list,
then read back 230,400 finite pixels. Every pixel differed from the sentinel,
the red channel varied spatially, and the D3D12 debug layer reported zero errors
with no device removal. These observations complete the bounded M6a proof. They
do not establish image quality,
temporal correctness, broad format or hardware/provider support, general
concurrency, a public dispatch contract, or GPU-based validation success.

The first implementation slice, **M6a**, comprised the missing dispatch ABI,
private production dispatch plumbing, and one opt-in synthetic DX12 test.
Completion required submission, fence completion and GPU readback, which the
recorded test delivered. `ffxDispatch` returning OK establishes CPU-side
acceptance, not completion.

The public M5 constructor fixes creation flags to zero. The native test supplied
an explicit 1×1 `R32_FLOAT` exposure resource with multiplier `1.0`; the selected
provider accepted that profile and completed the recorded dispatch. Changing the
constructor to auto exposure would exercise a different policy and remains a
separate public-contract decision.

## M6a implementation scope (completed)

| Location | Required work | Boundary |
|---|---|---|
| `fsr-sdk-sys` | Add `FfxApiFloatCoords2D`, `FfxApiResourceDescription` with its three union slots, `FfxApiResource`, the upscaler dispatch descriptor and tag `0x00010001`, and only the resource type, format, usage, flag, and state constants needed by this proof. | Mirror pinned AMD declarations; the existing loader already forwards `ffxDispatch`. Do not bind the entire SDK format universe. |
| `fsr-sdk` | Add a narrow internal `ID3D12Resource::GetDesc()` → FFX resource converter; build the complete native dispatch descriptor from the five required resources and supplied frame values; call `ffxDispatch` through the retained M5 owner and preserve its native return code. | Keep this private. Do not expose a public dispatch method, raw context handle, generic resource framework, or new creation policy. |
| `tests/` | Create a device, DIRECT queue, allocator and recording graphics list; upload synthetic textures; record state transitions and dispatch; close/submit, signal/wait a fence, and read back the output with the DX12 footprint row pitch. | Test equipment only. No scene renderer or library-level queue/fence ownership. |

Keep the DX12 harness under `tests/`; a crate-local opt-in test can invoke the
private dispatch path. An external integration test must not force a public
method into existence just to reach that path.

Use AMD's inline `ffxApiGetResourceDX12` as the conversion reference. Derive
dimensions, mips, format, and UAV usage from `GetDesc()`; do not guess actual
DX12 resource state from it. For this proof, accept the chosen single-sample,
single-mip Texture2D inputs and reject unsupported shapes or formats before
FFI. Keep those test restrictions private; they are not a public whitelist.
Check inspectable preconditions needed by the proof: a DIRECT list, resources
from the context's device, compatible extents within context maxima, distinct
input/output objects, and UAV capability on output. Do not claim that these
checks prove heap non-aliasing, residency, or actual GPU state.
The test must provide resources from the context's device and keep the list,
allocator, resources, backing memory, context, and runtime live through fence
completion. The internal call cannot turn a CPU borrow into GPU completion.
The owner's teardown safety reasoning now requires GPU completion before teardown.

The initial descriptor uses color, depth, motion vectors, exposure, and output.
Reactive and transparency masks are inline null resources. Set render/output
extents from the actual textures, motion-vector scale to the render dimensions
for UV-encoded vectors, sharpening and dispatch flags to zero, and
`preExposure = 1.0`. Supply valid positive frame time, near/far/FOV and
view-unit-to-metre values, plus a one-frame reset. The chosen jitter must match
how the synthetic color was generated; zero motion vectors describe a static
frame. These values are fixture inputs, not a public frame contract.

## Verification, in order

The following were M6a's verification gates. The
[native record](research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
documents their result on the tested setup; GPU-based validation remains
inconclusive as described above.

1. **Paired ABI checks.** Check the selected constants, types, field offsets,
   size/alignment, bool representation and `PfnFfxDispatch` signature in Rust
   and against the pinned headers with MSVC. Compare the converter's five test
   resources with AMD's inline DX12 helper. The [dispatch ABI record](research/records/2026-09-24-src-m6-upscale-dispatch-abi.md)
   supplies expected x64 offsets; the selected values passed the paired
   Rust/MSVC check on Windows x64.
2. **Private path checks.** Use a fixture to inspect the descriptor tag, five
   resource pointers/descriptions/states, null masks, extents, scalar values,
   one-call behavior and preservation of an unknown native error code. Include
   focused rejection of an invalid resource before native dispatch. Fixtures
   cannot establish provider acceptance or GPU behavior.
3. **One opt-in native test.** Start with approximately 320×180 → 640×360:
   spatially varying RGBA16F color, valid R32F depth, zero RG16F motion
   vectors, 1×1 R32F exposure, and a distinct UAV-capable RGBA16F output
   initialized to a known sentinel. Transition inputs to
   `NON_PIXEL_SHADER_RESOURCE` and declare matching
   `FFX_API_RESOURCE_STATE_COMPUTE_READ`; put output in its declared UAV state.
   Record dispatch on the open DIRECT list, then transition/copy the output
   for readback. The recorded UAV-to-copy transition completed with zero
   D3D12 debug-layer errors on the selected setup; this is not a universal
   output-state guarantee.
4. **Inspect the completed work.** Require an OK native dispatch, successful
   close/submit/fence, no relevant DX12 debug errors or device
   removal, and finite output that differs from the sentinel and has nontrivial
   spatial variation related to the input. A single frame establishes neither
   temporal correctness nor attractive reconstruction. Record DLL fingerprints,
   SDK/provider version, GPU/driver, texture/descriptor inputs, native codes,
   debug/GPU-validation observations, readback checks, commands and limitations in a
   self-contained verification record.

If native execution fails, distinguish ABI mismatch, input/state setup,
fixed-zero creation flags, provider behavior, and GPU validation failures.
Preserve the negative observation; do not substitute a private auto-exposure
context and call it proof of the public M5 path. Build or fixture success alone
does not mark M6 complete.

## Explicitly outside M6a

Scene renderer, meshes, materials, camera system, animation, game loop,
Bevy/wgpu, image-quality framework, complex temporal sequence, public
`UpscaleDispatch` API, generalized format/state support, optional masks,
sharpening, dynamic resolution, Query/Configure helpers, and automated SDK
acquisition.

M6b subsequently implemented [D011](adr/D011.md) with borrowed public inputs,
resource validation, checked frame values, unsafe GPU-lifetime obligations and
native-failure poisoning while preserving zero create flags and explicit
exposure. The [public-route record](research/records/2026-09-25-m6b-public-dispatch-verification.md)
documents its submitted, fenced readback on one configuration. The
[roadmap](../ROADMAP.md) remains the status authority.
