# Experiment: BGRA/sRGB direct SDK integration

- Date: 2026-10-24.
- Question: can the pinned SDK be invoked with the candidate colour/output and
  generator-input formats on the recorded DX12 configuration, with correct
  resource metadata, native results, submission, synchronization and cleanup?
- Scope: SDK access and integration. Image reconstruction, provider quality,
  numerical equivalence of reconstructed images and mask-algorithm validation
  are excluded under the existing project scope and the maintainer's explicit
  reaffirmation during this experiment.
- Protocol: [direct SDK integration](../../../tools/verification/bgra-srgb-fsr-protocol.md).
  Earlier [T1/T2](2026-10-24-exp-bgra-srgb-views.md) cover independent view
  interpretation and storage. This record covers T3/T4 actual SDK calls.

## Equipment and tested changes

Baseline `10ea8a2` plus test-equipment changes recorded below. The worktree was
clean at start. Windows 11 `10.0.26300`, x64 MSVC `14.44.35207`, Windows SDK
`10.0.22621.0`, Python `3.14.1`; adapter 0 AMD Radeon RX 9060 XT, vendor
`0x1002`, device `0x7590`, DXGI driver `32.0.32015.7029`, feature level 12.0.
Ordinary debug layer enabled; GBV not enabled. Transient adapter LUID is omitted.

SDK v2.3.0, source commit `60f4ea81909200d8542eca14dccb2628b763a9a3`, with the
four independently hash-checked [public headers](../../../tools/native-headers.json)
and local AMD-signed DLLs:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The coordinator stages these exact bytes and checks their valid AMD Authenticode
signatures. Four independent C++ ABI compilation checks pass before native work.
Main calls select provider 3.1.5 (`17700776140660019205`) or 4.1.1
(`17700776140664213505`); generator calls select only 3.1.5. Live context identity
is checked by the C++ reference after creation.

Changes are confined to test equipment and documentation:

- Add four-byte BGRA/sRGB allocations to the existing helper's stride table in
  `crates/fsr-sdk/tests/native/typed_formats.h`.
- Add `tools/verification/bgra_srgb_fsr.py`, its protocol and acceptance tests.
- Add `tools/verification/bgra_srgb_fsr_check.py` for read-only review of existing
  native evidence under the authorized scope.

All native creation, main dispatch, generation and destruction use the existing
separate `typed_reference.cpp` with pinned C headers and AMD's resource helper.
The existing Rust laboratory orchestrates lifetime and provider enumeration in
`MODE=reference`; it does not record the candidate workloads through the public
Rust upscaler/generator methods. No production converter, API, dependency or
accepted format list changes. No runtime download or publication occurs.

## Commands and inputs

From the repository root:

```sh
python tools/verification/bgra_srgb_fsr.py --native --adapter-index 0 --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/bgra_srgb_fsr_check.py --run target/verification/20261024T144428Z-bgra-srgb-fsr-d5a92e935a85
```

The initial native attempt is
`20261024T144428Z-bgra-srgb-fsr-d5a92e935a85`, input identity
`85a50a0e271f07fc3b60b410e961c193401332429397163204a143b39ce30432`.

Main render/output allocations are 320×180 / 640×360. Depth R32F, motion RG16F,
explicit exposure one, pre-exposure one, zero jitter and pixel-space motion.
Inputs are fully initialized, independently packed moving RGB tiles with distinct
channels. UNORM and encoded sRGB byte configurations use matching explicit
float32 source configurations. No automatic colour conversion or hint inference
is introduced.

Each main input/output role is varied separately: BGRA8 UNORM, RGBA8 sRGB and
BGRA8 sRGB, with existing RGBA8 UNORM controls. Outputs are exercised with native
sharpening disabled/enabled, without requiring any image effect. Generator
opaque-only and pre-upscale colour roles are independently varied through all
three new formats; active render is 320×180, 160×90, 320×180 within fixed maximum
allocations. Black and contrasting inputs are valid configurations; no numerical
mask response is required. The generated mask receives an explicit UAV barrier
and state transition before consumption by main dispatch.

Every workload has two separate processes, each with one reset frame and two
continuing frames. The C++ reference owns its own context/loader and destroys
the context explicitly. Each submitted list is fenced before readback or release;
the existing laboratory discards failed unsubmitted recordings and avoids native
destruction if completion is unproven. Child watchdog is 180 seconds.

## Integration results

The independent integration review passes all 44 workloads / 88 children and the
nine equipment prerequisites. These are 264 submitted/fenced frames: 252 actual
main-upscaling SDK calls, 72 generator SDK calls and 12 no-dispatch control frames
(generator frames also contain one main call). All 88 C++ contexts report
successful explicit destruction.

| Contract observation | Result |
|---|---|
| Exact pinned headers/runtime, ABI compilation and selected live provider | Pass |
| Actual per-role allocation formats and complete resource inventory | Pass |
| Native creation, main calls on 3.1.5/4.1.1 and generation on 3.1.5 | Pass |
| Exact post-submission input-byte integrity and readback sizes | Pass |
| Full active output writes; no-dispatch seed preservation | Pass |
| Active generator writes through shrink/grow and subsequent main consumption | Pass |
| Ordinary error diagnostics/device removal, fence completion and cleanup | Pass |

Output hashes and finite/write observations are retained as integration evidence.
They are not compared against an expected FSR image, another provider's image or
a CPU implementation of AMD's algorithm. The generator seed check is bounded to
these selected inputs; it is not a universal mask-value or write-detection rule.

## Scope correction and preserved failure

The agent initially added an unrequested reconstructed-image numerical gate and
a CPU mask-algorithm comparison, contrary to the already applicable scope. The
maintainer explicitly required their removal. This was an agent scope error,
not a new project policy or an SDK defect.

The original native attempt remains **failed**: 54 coordinator stages passed and
one reconstructed-image comparison failed. Its numerical threshold was 0.006;
the largest separately observed difference was 0.14111328125. These historical
observations are not relabelled as passing, used as support criteria or selected
for further investigation. The temporary numerical analysis also remains local
historical output. The active comparison code was removed.

The separate integration review
`20261024T145844Z-bgra-srgb-sdk-check-a9f27b17bfd0` verifies the original artefact
inventory, its unchanged native inputs and the exact current hashes of native
equipment. It rechecks the recorded native command results, input bytes, writes,
fences and cleanup under the authorized scope. It writes a new result, **passed**,
without changing the original attempt or making further GPU calls. Thus the
integration result is distinct from the failed out-of-scope experiment.

Five focused acceptance tests pass, including explicit acceptance of arbitrary
nonseed generator values instead of predicted mask values. Three existing
packed-format/write/inventory regression checks pass. Their first invocation
had an incorrect Python module search path, corrected before successful execution;
this was a test invocation error, not a native failure.

## Retention and limits

[Compact evidence](2026-10-24-exp-bgra-srgb-sdk-integration/summary.json) retains
the original status/identity, exact native equipment hashes, integration-review
identity, per-role capability observations and per-frame input/output hashes,
write counts and cleanup observations. Full raw inputs/readbacks, environment
and process logs stay under ignored `target/verification/<attempt>/`. They are
not redistributed; compact hashes cannot reconstruct historical GPU images.

These results establish bounded actual SDK invocation and resource/lifetime
integration on the named configuration. They do not prove arbitrary shader access
bounds, universal provider/hardware support or the public Rust wrapper's future
admission of these formats. Renderer-owned states, resource lifetimes, submission
and actual-provider obligations remain explicit. Existing generator failures on
4.1.1/2.3.4 are not revisited or reclassified.

The completed T3/T4 integration observations are evidence for a subsequent
role/format contract and implementation decision. No reconstruction-quality,
sharpening-effect, numerical-image or CPU algorithm work is a prerequisite.
