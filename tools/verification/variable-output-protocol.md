# Variable-output native spike

Selected by the maintainer on 2026-10-19 after the variable-output source intake.
This is test equipment beyond D024's current fixed-output public contract.
It neither implements a public variable-output API nor accepts an allocation,
provider-support, dynamic-flag or history-reset policy.

## Question and observable outcomes

On the pinned Windows/DX12 deployment, determine which requested active regions
are actually written, which padding pixels change, and whether recording,
submission, fencing, immutable inputs, diagnostics and cleanup succeed with
different output/display-motion allocations. Native rejection remains an
observation with its exact code and terminal cleanup. Crashes, timeouts, incomplete
active writes, bad inputs, non-finite active output, diagnostic or cleanup errors
fail; they are never converted to unsupported-provider success.

```powershell
python tools/verification/run.py run experiment-variable-output --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The existing coordinator checks the signed pinned SDK v2.3.0 deployment, four
header hashes and independent MSVC ABI, then builds a dedicated helper with
`FSR_OUTPUT_PROBE` and stages its paired Rust executable. Each workload runs in
a separate watchdog-bounded process. Cargo is offline. No GBV is enabled: its
recorded dispatch stall remains separate from ordinary debug-layer execution.

## Frozen workload selection

Three exact, enumerated providers: 4.1.1, 3.1.5 and 2.3.4. Missing providers fail
explicitly; there is no fallback. The local public owner creates/retains contexts,
but recording uses a complete raw ABI descriptor to bypass only the public
geometry/allocation restrictions being investigated. All resource metadata keeps
the actual DX12 dimensions and is compared with the pinned C++ inline helper.

Every context has maximum render 320×180 and maximum output 640×360. Colour,
depth and render-motion allocations stay fixed at 320×180. RGBA16F colour/output,
RG16F motion and 1×1 R32_FLOAT exposure retain the established profile. Render
size is always 320×180. No beyond-capacity active output is requested.

| Selection | Output / display-motion allocations | Variation |
|---|---|---|
| `stable-max` | Maximum | Three 640×360 frames; provider equipment baseline |
| `stable-small-large` | Output 672×384; render motion | Three requested 511×287 frames |
| `aligned-max` | Maximum | 640×360 → 512×288 → 640×360 |
| `rounded-max`, `rounded-fixed-flag` | Maximum | 637×357 → 511×287 → 637×357; dynamic flag true/false, each sharpening off/on |
| `rounded-active`, `rounded-large` | Active dimensions / 672×384 | Rounded sequence, sharpening off/on |
| `display-max`, `display-active`, `display-large` | Matching maximum / active / 672×384 output and display-motion allocations | Rounded sequence, UV motion, sharpening off/on |
| `display-max-pixels` | Maximum | Matched rounded display-motion workload encoded in pixels; sharpening off |
| `width-only`, `height-only` | Maximum | Change only width to 511 or height to 287 and grow back; sharpening off |

There are 20 workloads per provider, three frames per child, plus one independent
three-frame no-dispatch equipment control. The global control must pass before
any provider workload; each provider's stable-maximum baseline must pass before
its variants. A failed variant does not suppress another independent variant.
The 71-case profile includes 10 preparation/build cases and 61 isolated children.

## Inputs, ordering and audits

The analytic linear plane translates by 1/128 normalized units per axis/frame.
Colour samples include current queried jitter. Motion is current-to-previous,
jitter-excluded, zero for frame zero and −1/128 UV afterward; display-pixel
encoding multiplies each axis by the requested active output. UV descriptor
scales use active motion dimensions; pixel scales are `(1,1)`. Selected-provider
jitter queries use each current requested output. Reset is true only on the
first frame, including across shrink/grow; no automatic history claim follows.

Each frame receives fresh resources, including replacement output/motion
textures. The previous frame is CPU-fenced before recording the next. This is
serialized execution, not overlapping resource replacement or general cross-queue
support. The helper uploads inputs and output seed 65504, transitions inputs to
shader-read/output to UAV, then records fenced full-resource snapshots. Readback
checks every colour/depth/motion/exposure byte against uploads; Python independently
reconstructs binary32/truncated-binary16 inputs and rechecks packed snapshots.
Full output snapshots independently reproduce finite/written active counts,
outside writes and rightmost/bottommost written coordinates. SHA-256 identifies
each input/output. Matched UV/pixel output hashes are comparisons, not a mandatory
quality/equivalence gate.

Native OK recordings are submitted once and fenced before resources/context
cleanup. Native failure invalidates the unsubmitted list before one terminal
destroy attempt, with no reuse/retry. Failed/uncertain completion terminates the
isolated process without unwinding GPU dependencies. Successful completed frames
and failed attempts remain recorded.

Padding writes are measured without requiring preservation. A debug-layer pass
and in-resource snapshots cannot prove absence or safety of out-of-resource
shader stores, especially when a provider ignores requested dimensions. No
general allocation rule, native rejection guarantee, reconstruction-quality,
history-correctness or API-support claim follows from this bounded spike.
Raw payloads remain under ignored `target/`; retain compact measurements/identities
with the final experiment record, preserving unsuccessful outcomes.
