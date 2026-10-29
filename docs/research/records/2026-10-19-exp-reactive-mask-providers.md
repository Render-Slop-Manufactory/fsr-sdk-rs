# Experiment: reactive-mask generation across pinned DX12 providers

- Date: 2026-10-19.
- Question: do native acknowledgements correspond to mask writes and successful
  generated-mask consumption through the raw path and both public families?
- Outcome: **3.1.5 passes both public families; 4.1.1 and 2.3.4 fail mask-write
  criteria despite returning OK.** Failed complete profiles remain failed.
- Exclusions: numerical reconstruction quality, provider consumption of particular
  mask texels, automatic support probing/fallback and other deployments.

## Setup and reproducible protocol

Use the [Windows batch equipment, runtime hashes and source-state description](../../release/2026-10-19-windows-upscaler-verification.md#equipment-and-tested-source).
The initial raw test uses the existing independent test-only descriptor. Public
tests add a shared equipment/oracle macro, calling `validate_generate_reactive_mask`
and `generate_reactive_mask` on `Upscaler` or `MovableUpscaler`. They retain the
raw descriptor branch independently. No production mapping or criterion changes.
The [source/command identities](../../release/2026-10-19-windows-upscaler-verification/checks.json)
identify each tested state and include independent MSVC ABI checks.

```powershell
python tools/verification/run.py run experiment-reactive-mask --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-public-reactive-mask --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-selected-reactive-mask --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Each provider/family arm has fresh-process no-generator and identical-colour
controls plus all 16 flag combinations. Each child uses a disposable context,
fixed 320×180 RGBA16F colour allocations, a distinct R8_UNORM mask seeded to 227,
and regions 320×180 → 256×128 → 320×180. Scalars are scale 1, cutoff 0.2 and
binary value 0.9. Inputs alias only in the identical-colour control.
Explicit providers are selected by exact name and checked against the live ID;
there is no fallback. Calls remain serialized, including the movable family.

The helper records UAV ordering and UAV → copy-source → shader-read transitions,
snapshots the mask, records its public upscaler consumer on the same list, submits
once and fences before readback/cleanup. Equipment controls require unchanged
seed and unchanged upscale output. Generation requires mask writes, untouched
inactive pixels, zero identical-colour reactivity, nonzero distinct-colour output
with flags zero, finite/written upscale output, clean diagnostics and cleanup.
Other flags retain their observed values without assuming an SDK formula.

## Observations

| Provider / ID | Route | Generation children | Result |
|---|---|---|---|
| 4.1.1 / `17700776140664213505` | Independent raw, local owner | 17 | All fail: OK, no mask writes |
| 4.1.1 / `17700776140664213505` | Public local and movable | 34 | All fail: OK, no mask writes |
| 3.1.5 / `17700776140660019205` | Explicit public local and movable | 34 | All pass, 102 generated/submitted/fenced frames |
| 2.3.4 / `17700776140655833092` | Explicit public local and movable | 34 | All fail: OK, no mask writes |

All seven no-generator controls pass. All 126 children complete their three
frames and terminal cleanup: 378 completed frames in total, including 21 control
frames. Diagnostics report zero errors and no device removal. No native error,
crash or timeout explains a failed write criterion.

On 4.1.1/2.3.4 every read mask pixel remains 227, including identical-colour
controls. All 255 affected generation frames retain zero changed pixels. On
3.1.5 identical-colour output is entirely zero. With flags zero and distinct
inputs, each maximum-size phase changes all 57,600 pixels, with 43,200 zero pixels
and maximum 127; the smaller phase changes all 32,768 pixels. Inactive pixels
remain seeded. The [complete compact measurements](2026-10-19-exp-reactive-mask-providers/measurements.json)
retain every flag/family/phase and cleanup result; their
[inventory](2026-10-19-exp-reactive-mask-providers/inventory.json) hashes normalized UTF-8/LF bytes.

The raw/default profile exits 1 (12 passed/17 failed), public/default exits 1
(13/34), and explicit-provider profile exits 1 (49/34). Passing 3.1.5 children
do not reclassify either complete failed profile. Original attempts remain local.

## Baseline delta and limits

Windows fixtures, production/probe MSVC ABI, public native recording and bounded
3.1.5 writes/cleanup are now observed. A successful optional dispatch return does
not certify generator capability. The public wrapper reproduces the independent
raw 4.1.1 result; the successful 3.1.5 arm also checks the equipment's ability to
observe writes. These observations do not identify the internal reason for
4.1.1/2.3.4's behaviour or establish support elsewhere.

No provider allow-list, automatic retry, capability cache, inferred error or
changed terminality contract is introduced. Keep the documented actual-provider
trust and successful-recording/GPU cleanup obligations. Final extracted packages
and any broader provider configuration require their own checks. Full original
logs remain under ignored `target/`; retained measurements omit personal paths
and transient addresses. Exact past GPU bytes are not promised by reproduction.
