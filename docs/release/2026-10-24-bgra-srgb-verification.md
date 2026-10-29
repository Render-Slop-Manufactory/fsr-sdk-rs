# Typed BGRA/sRGB wrapper verification

- Date: 2026-10-24.
- Scope: [D038](../adr/D038.md), development `fsr-sdk 0.3.0` / `fsr-sdk-sys
  0.1.2`, SDK v2.3.0, Windows x64/MSVC + DX12.
- Result: implemented and verified within the recorded configuration. Other U05
  candidates and final release archives remain separate.

## Implemented contract and inputs

Both public owners accept BGRA8 UNORM and RGBA8/BGRA8 sRGB as main colour,
main output and either generator colour input. The three raw constants have
paired Rust/MSVC assertions; native layouts and public method signatures are
unchanged. Shared preflight/recording checks actual sRGB SRVs, matching UNORM
UAVs and conditional fully typed casting, including on UAV-capable read-only
inputs. Main outputs require typed stores; input roles require neither typed
UAV loads nor stores. Errors retain role, operation and exact HRESULT; rejected
inspection enters no SDK code and permits retry.

No format/view override, conversion, software swizzle, automatic output encode,
inferred colour flag, reset, fallback or provider allow-list is added. Typeless
colour and other role formats remain outside this slice. Renderer coordination
and actual-provider obligations remain explicit at the unsafe native boundary.

Baseline `ec1a184` plus the inspectable D038 constants, validation, controlled
tests, public native profile and documentation changes. The
[compact evidence](2026-10-24-bgra-srgb-verification/summary.json) identifies
tested source/equipment hashes and all run identities. Accepted runs report
unchanged source inputs during execution. Subsequent edits only finalize
documentation/status and clarify the Rustdoc wording about software conversion;
the executable implementation and test-equipment hashes remain as recorded.

SDK commit `60f4ea81909200d8542eca14dccb2628b763a9a3`, four hash-verified
[public headers](../../tools/native-headers.json), exact AMD-signed local DLLs:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

Windows 11 `10.0.26300`; RX 9060 XT, adapter 0, vendor `0x1002`, device
`0x7590`, driver `32.0.32015.7029`; ordinary debug layer, GBV disabled.
MSVC `14.44.35207`, Windows SDK `10.0.22621.0`, Python `3.14.1`, current
Rust/Cargo `1.99.0`, exact MSRV Rust `1.88.0`.

## Commands and outcomes

From the repository root, selecting the actual SDK directory and adapter:

```sh
python tools/verification/run.py run ci --require-passed --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run native-bgra-srgb --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

| Attempt | Status | Stages |
|---|---|---|
| `20261024T154746Z-ci-79c48ffc5b04` | Passed; inputs unchanged | 182 |
| `20261024T154802Z-msrv-1ebfd4d90365` | Passed; inputs unchanged | 159 |
| `20261024T154849Z-native-bgra-srgb-2552e14ec651` | Passed; inputs unchanged | 102 |

CI covers infrastructure, formatting, seven feature configurations, ordinary
tests, Clippy, Rustdoc, public consumers/examples, platform/feature dependency
graphs and four independent MSVC header assertions. MSRV covers corresponding
source/tests/consumers/examples on 1.88.0. The reserved Vulkan compile error is
preserved; Linux dependency graphs establish isolation, not native execution.
Checks are offline and publish nothing. Accepted CI/MSRV runs had the temporary
Windows directory access their tests require.

Both-family controlled tests cover exact native format/usage/state, separate
SRV/UAV capabilities, query order, missing bits, conditional casting/errors,
same-HRESULT role attribution, metadata/device rejection before queries, retry,
no SDK entry on rejection, reference balance, poison precedence and terminal
native failure. Fake sRGB capabilities intentionally exclude UAV support; UNORM
UAV capabilities exclude unrelated load/sample bits. Other roles and typeless
colour reject. Existing baseline/RG32F/depth checks also pass.

The [public native protocol](../../tools/verification/bgra-srgb-public-protocol.md)
has ten equipment stages and 92 isolated workloads: both owners on selected
3.1.5/4.1.1 for main input/output, generation only on 3.1.5. All 92 contexts
report explicit successful destruction. There are 276 fenced submissions,
264 main SDK calls, 108 generator SDK calls and 12 no-dispatch control frames.
Generator frames also contain one main call.

Per-role candidate formats, output sharpening selections and UAV-capable inputs
execute through production preflight/recording. Generator render shrinks/grows
within fixed maximum allocations and its output is consumed after explicit UAV
ordering/transition. Require actual live provider identity, complete format/role
inventory, unchanged input bytes, complete readbacks, active writes, finite
floating output, zero ordinary error/device removal, fencing and cleanup. Main
native descriptors additionally match AMD's inline resource helper. The movable
cases are serial, with no new handoff/concurrency claim.

There are no reconstructed-image equality, numerical quality, sharpening-effect,
CPU mask-algorithm or provider-quality criteria. Hashes are observations;
seed-write checks are bounded to these inputs and establish no universal
mask-value rule or shader access bound.

## Preserved earlier attempts

- `20261024T154138Z-ci-257d43dff221`: failed (174 stages passed, eight failed).
  Restricted temporary-directory access broke infrastructure/compile-control
  tests. Two existing HDR/colour negatives also still rejected now-admitted BGRA;
  they now use still-unselected RGBA8 SNORM. Source changed during this attempt.
- `20261024T154436Z-ci-ee6d6dabeda0`: all 182 stages passed with temporary
  access, but source changed while correcting those tests. Integrity correctly
  rejects the overall run. The final CI run repeats all checks with unchanged inputs.
- The focused inventory test initially omitted the coordinator's common equipment
  stage (101 versus 102); the count was corrected. All six focused Python tests
  pass. This changed neither native criteria nor an observed native outcome.

The earlier direct-SDK record's out-of-scope comparison failure is unchanged
and is no wrapper acceptance gate. No further quality work was performed.

## Retention and limits

The compact summary retains attempt outcomes/identities, source/helper hashes,
case settings, allocation capability observations, per-frame resource hashes
and write/fence/cleanup outcomes. Full inputs/readbacks and logs remain ignored
under `target/verification/<attempt>/`. No source snapshots, generated images or
AMD DLLs are redistributed; compact hashes cannot reconstruct historical GPU bytes.

This is bounded wrapper integration on one configuration, not universal provider/
driver compatibility, arbitrary access safety, padding/alpha preservation or a
presentation colour policy. Existing 4.1.1/2.3.4 generator limitations remain
recorded and were not reclassified. Final extracted-package/native release
checks and remaining U05 families remain separate.
