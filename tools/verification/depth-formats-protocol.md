# D32 depth-buffer integration protocol

`native-depth-formats` verifies [D037](../../docs/adr/D037.md) with the shared
[typed-resource equipment](typed-formats-protocol.md). It selects the same
pinned signed SDK, headers, adapter, both public owners and independent C++
contexts/descriptors on providers 4.1.1 and 3.1.5. Render/output maxima remain
320×180/640×360; each child fences three frames before cleanup.

## Workloads and frozen criteria

Per owner/provider: no-dispatch, R32F standard/reversed baselines, changed-colour
sensitivity, and all four D32/D32S8 typed/typeless allocations with standard,
reversed and dynamic geometry. The dynamic phase shrinks active render/output
to 160×90/512×288 and restores the maxima. Every format is a real
`ALLOW_DEPTH_STENCIL` allocation with no UAV or shader-read denial.

The helper creates the corresponding DSV and clears depth to 0.25/0.75 in
opposite horizontal halves (complemented for reversed depth). D32S8 stencil is
cleared to `0x5a`. It transitions the whole resource from depth-write to shader-
read. A separate HLSL compute shader loads the backend's exact depth SRV, plane
0, and writes floats to a buffer for fenced readback. DSV and SRV capability
queries must succeed; typed depth additionally requires OPTIONS3 casting support.
The helper records exact allocation/DSV/SRV/casting identity for each frame.

Require the sampled floats and depth-plane copy readback to equal the
independently encoded CPU depth bytes exactly. The stencil-plane copy after
upscaling must still equal the distinct `0x5a` pattern. These checks catch a
wrong plane, incorrect view, stale clear or mutation. Depth is produced through
the DSV rather than uploaded as scalar texture bytes. Other inputs retain exact
upload/readback audits from the typed-resource protocol.

Require selected live provider identity, three complete frames, finite/written
active output, unchanged no-dispatch output, changed-colour sensitivity, no
ordinary debug errors/device removal, fencing and explicit successful cleanup.
Require exact full-output public/C++ byte equality for every workload and phase.
For every fixed-geometry depth-family workload, require exact output equality
with the matched standard/reversed R32F baseline. These are workload-specific
integration criteria, not image-quality claims or universal numerical tolerances.

The Python evaluator's negative controls reject missing/wrong view or casting
evidence, incorrect sampled depth and changed stencil, in addition to the
existing finite/write/reference controls. Unsupported capability and native
errors fail. Keep failed attempts; do not change criteria to erase a failure.
Raw buffers remain under ignored `target/verification`; retain compact dated
commands, input identities, outcomes and limitations in a release record.

No stencil consumption by FSR, universal provider support, access bound,
padding preservation, GBV, performance or final-package claim follows.
