# Remaining external format integration protocol

Frozen 2026-10-24 before public confirmation. Baseline: SDK v2.3.0 pinned by
`native.py`, D039, Windows x64/MSVC + DX12, explicit adapter index 0 for this
confirmation. Python supplies bytes; Cargo runs the public owner; C++ owns DX12
equipment and independent pinned-header descriptors. No new dependency.

```sh
python tools/verification/run.py run native-resource-formats --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed
```

`resource_formats.py::workloads` is the exact inventory. Each new colour
allocation is varied independently as main input and, except load-only RGB32F,
output with sharpening disabled/enabled. Both generator colour roles vary
independently, including UAV-capable inputs where legal. Scalar/vector,
exposure, supplied-mask and generated-output typeless routes are separate arms.
R32 typeless ordinary depth explicitly disables DSV equipment; existing D32
typeless production remains covered by D037. D16/D24 include standard depth
and reversed depth with active render/output shrink/grow. Every arm runs three
fenced frames, reset then continuing. Both public owners select 3.1.5/4.1.1;
generation selects only the separately verified 3.1.5 operation.

Acceptance requires exact selected provider identity, real allocation inventory,
correct pinned-helper resource mapping (explicit RGB32F enum exception),
unchanged uploaded inputs, correct readback sizes, finite floating outputs,
complete active output writes, zero ordinary debug errors, no device removal,
completed fences and explicit native cleanup. No-dispatch controls preserve
their output seeds. Generated active masks must replace their seed; no mask
formula, threshold prediction or expected-zero algorithm is evaluated.

Depth is produced by DSV clears with independently representable UNORM values.
The shader samples the exact backend depth SRV. CPU plane readback independently
normalizes copied D16/D24 codes, and stencil stays 0x5a. These are view/encoding
checks, not comparisons of FSR reconstructions. Typeless payloads use the exact
fixed FLOAT/UNORM representation; RGB32F is never supplied with UAV flags.

There is no same-format/cross-format/provider image equality, numeric FSR-output
tolerance, sharpening-effect, quality, performance, mask-algorithm or padding
oracle. Raw inputs/readbacks are transient under ignored `target/`; successful
child payloads are discarded only after complete evaluation and capturing their
measurements/hashes. Failed payloads remain available for diagnosis. Retain compact case settings,
source/runtime identity, capability observations, hashes, counts and outcomes.
Failed attempts remain recorded. Source changes during a run invalidate its
acceptance. Device bits and one passing deployment certify neither other
providers/devices nor general native memory safety.
