# Direct FSR BGRA/sRGB SDK-integration probes

This is T3/T4 after the [independent DX12 view probes](bgra-srgb-protocol.md).
It changes test equipment only; the public wrapper still rejects these formats.
The existing Rust laboratory selects the provider and controls resource lifetime,
but `MODE=reference` sends all creation/dispatch/generation/destruction through
the separate pinned-header C++ reference. No Rust resource converter or public
upscale/generator method records these candidate workloads.

## Execution and input boundary

```sh
python tools/verification/bgra_srgb_fsr.py --native --adapter-index 0 --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
```

Select the actual adapter index. Require existing Windows x64/MSVC, ordinary
debug layer, the four pinned v2.3.0 headers, exact signed local loader/provider
DLLs and offline Cargo dependencies. Existing `native_setup`, ABI checks, runner,
watchdogs and source-integrity checks remain in use. No new automatic profile,
download, provider fallback or feature acceptance is added.

Main probes explicitly select 3.1.5 and 4.1.1; generation selects only the
previously writing 3.1.5. Every workload runs twice in independent processes,
each with one reset frame followed by two continuing frames and explicit
successful context destruction. All calls/submissions are serial and fenced.
Each process has a 180-second watchdog. The reference verifies actual live
provider ID after creation. The expected inventory is 44 workloads, 88 children
and 264 submitted/fenced frames, including no-dispatch controls. The current
inventory has 53 coordinator stages. The initial attempt additionally ran two
unrequested numerical-comparison stages; they are excluded from the authorized
scope under the maintainer's explicit correction on 2026-10-24.

## Signal construction

Render/output maxima remain 320×180 / 640×360, depth R32F, motion RG16F,
explicit R32F exposure one, pre-exposure one, zero jitter and pixel-space motion.
RGB 16×16 tiles move four input pixels per continuing frame. Logical RGBA codes
alternate `(48,128,224,255)` and `(128,64,16,255)`; a known centre rectangle adds
64 red codes to the pre-upscale colour, staying in range. RGB is deliberately
distinct. Alpha is one and is not treated as a reconstruction input.

For UNORM trials, the float32 reference contains `code/255`. For sRGB trials,
the float32 reference contains independently calculated ideal decoded RGB;
typed sRGB textures contain the original encoded bytes. BGRA packs physical
bytes B,G,R,A. No non-linear creation flag or per-frame hint is set: the intended
provider-facing signal is linear after the selected SRV. This selects a bounded
probe configuration, not a universal provider colour policy.

The coordinator generates and audits tight input bytes. The existing native
helper audits all post-dispatch input copies, states, ordinary error diagnostics,
device status, fence completion and cleanup. The new four-byte formats are
admitted only in the test helper's stride table.

## T3 main roles

Vary only one resource role at a time. Main colour is RGBA8 UNORM (paired
control), BGRA8 UNORM, RGBA8 sRGB or BGRA8 sRGB; output remains RGBA16F. Main
output uses the same four eight-bit allocations, each with native sharpening
disabled/enabled, while input remains the existing RGBA32F format. Include
no-dispatch and changed-RGB sensitivity controls and UNORM/decoded float32
references.

Frozen criteria:

- Exact process/provider/frame/capability inventory, native success, unchanged
  input bytes, zero ordinary debug errors/device removal and explicit cleanup.
- Every active output pixel differs from the finite magenta seed. A no-dispatch
  control preserves the whole seed. Float outputs must have finite components.
- Repeated inputs exercise independently created native contexts; their output
  hashes are observations. No image equivalence, numerical reconstruction bound,
  determinism, sharpening effect or provider-algorithm expectation is required.
- Channel/transfer semantics are independently checked at the DX12 view boundary
  in T1/T2. Do not use comparisons of FSR's reconstructed images to retest them.

## T4 both generator colour roles

Independently replace opaque-only role 5 and pre-upscale role 0 with each new
format 87/29/91. The other colour input remains RGBA32F with matched UNORM or
ideal-decoded linear signals. Each selection has a contrast trial and an
equal-black trial. Active render shrinks to 160×90 in phase one and returns to
320×180; allocations stay at maxima. Generated R8 output starts at `0x55`.
An explicit UAV barrier and transition precede its use as the main reactive mask.

Use unit scale and **no generator flags**. Require complete active writes
(no retained `0x55` in this bounded workload) and the same native/status/input/
output/cleanup checks as T3. A sentinel check is a workload-specific write
observation, not a guarantee that arbitrary legitimate masks avoid that value.
Do not predict mask values, compare the provider with a CPU algorithm, require
an image effect or impose repeated-output equality. Black inputs are another
valid input configuration, not an algorithm-quality acceptance criterion.

## Review of an existing attempt

```sh
python tools/verification/bgra_srgb_fsr_check.py --run target/verification/<native-attempt>
```

This separate read-only checker verifies the original artefact inventory,
unchanged native equipment and recorded GPU commands, then evaluates their SDK
integration observations under the current scope. It writes a new exclusive
result and makes no GPU calls. Original failed comparison stages remain failed
in their original attempt; the checker neither overwrites nor relabels them.

## Recording and limits

Preserve every failed attempt; do not substitute an OK status for written or
correctly mapped output. Retain compact commands, source/runtime identities,
per-frame hashes/measurements and outcomes; full raw readbacks stay under ignored
`target/verification/`. A native failure or unproven completion cannot pass;
the existing isolated laboratory discards unsubmitted recordings and avoids
unwinding native objects with unproven GPU completion.

This does not test public wrapper admission, either ownership family's new
format validation, arbitrary signals/providers, alpha preservation, PQ/exposure
semantics, access bounds, untouched padding, GBV, image quality or performance.
The prior 4.1.1/2.3.4 generator no-write findings remain unchanged.
