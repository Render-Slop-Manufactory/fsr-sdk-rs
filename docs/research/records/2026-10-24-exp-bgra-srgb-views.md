# Experiment: independent BGRA/sRGB DX12 reads and stores

- Investigation/execution date: 2026-10-24.
- Question: do the inspected backend-equivalent SRVs expose the expected RGB
  signal, and can the corresponding UNORM UAVs actually store expected bytes on
  the available adapter, including when allocation is typed sRGB?
- Scope: direct DX12 only; no AMD runtime loading, FSR context/dispatch, public
  wrapper format changes, image-quality or performance claims.
- Source basis: [audited mappings and corrected references](2026-10-24-src-bgra-srgb-audit.md).
- Frozen confirmation protocol: [BGRA/sRGB views](../../../tools/verification/bgra-srgb-protocol.md).

## Setup and reproduction

Repository baseline `7dbde52` plus the independent probe, Python coordinator,
protocol and acceptance tests named below; the incoming source record was
already untracked. All changes are inspectable project files rather than an
unpublished source snapshot:

- `tools/verification/bgra_srgb_probe.cpp`: independent HLSL compilation,
  resource/view creation, shader execution and fenced readbacks.
- `tools/verification/bgra_srgb.py`: shared-runner coordination and independent
  CPU upload/channel/transfer/store evaluation.
- `tools/verification/tests/test_bgra_srgb.py`: rejection/sensitivity controls.

Equipment: Windows 11 `10.0.26300`, adapter 0 AMD Radeon RX 9060 XT, vendor
`0x1002`, device `0x7590`, DXGI-reported driver `32.0.32015.7029`.
MSVC toolset `14.44.35207`, Windows SDK `10.0.22621.0`, Python `3.14.1`, x64.
The shader target is `cs_5_0`, independently compiled by D3DCompile.
Ordinary debug layer is enabled; GPU-based validation is not enabled.
OPTIONS3 reports fully typed casting support. Pinned public headers are checked
before compilation; the [source audit](2026-10-24-src-bgra-srgb-audit.md) supplies
the SDK commit and exact header/backend identities.

From the repository root:

```sh
python -m unittest discover -s tools/verification/tests -p test_bgra_srgb.py
python tools/verification/bgra_srgb.py --native --adapter-index 0 --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX
```

The coordinator acquires MSVC using existing `windows.ps1`, compiles with
`/std:c++17 /EHsc /W4 /WX`, links `d3d12.lib dxgi.lib d3dcompiler.lib`, and invokes
the resulting `view-probe.exe 0` in the exclusive run directory. GPU process
timeout is 90 seconds; each fence has a 15-second timeout. Buffers and textures
are released after their submitted work is fenced. No installation or SDK
runtime deployment is needed.

## Inputs and observations

Six 8×8 allocation routes: typed RGBA/BGRA UNORM, typed RGBA/BGRA sRGB, and
RGBA/BGRA typeless. Typeless routes intentionally use backend-default UNORM
views. Two distinct logical RGBA patches alternate by `(x+y)%2`:
`(48,128,224,160)` and `(192,64,16,127)`. Both Load and point SampleLevel read
all 64 texels to a structured float4 buffer. CPU evaluation checks all upload
bytes and 256-byte copy pitch independently, then all 128 float4 results.

T2 writes `(48/255,128/255,224/255,160/255)` through each exact UNORM UAV.
All 64 stored texels must equal `30 80 e0 a0` for RGBA or `e0 80 30 a0` for
BGRA (hex). Readback excludes undefined row padding. A subsequent read through
the exact SRV checks that sRGB decoding occurs on read, without mistaking it
for output encoding.

| Observation | Confirmation result |
|---|---|
| Six input allocation/SRV/helper-format/usage mappings | Pass |
| RGB order for RGBA and BGRA, all texels and both read instructions | Pass |
| Typed sRGB reads and undecoded alpha | Pass under source-defined code-space criterion |
| Typeless defaults read UNORM values | Pass; no implicit sRGB view |
| All six UAV allocations created and all six routes stored exact expected bytes | Pass on this adapter, including both typed-sRGB allocations |
| Subsequent SRV read of each written texture | Pass |
| Deliberately R/B-swapped upload | Matches swapped prediction and rejects ordinary RGB expectation |
| Unexpected debug errors/device removal/fence failures | None |

Confirmation totals: seven T1 input cases including the swapped control, six
T2 store-and-read cases, and four passed coordinator stages. Exact output byte
checks pass for every texel. Maximum UNORM/alpha absolute error is
`2.9685450542160652e-8`. Maximum sRGB RGB absolute error is
`0.0006895404596125587`; its maximum inverse-transfer code error is
`0.09171959119959183`, within the source-defined 0.5-code budget.
For code tuple `(48,128,224,160)`, measured sRGB-view values are
`(0.029541015625,0.2158203125,0.74609375,0.6274510025978088)`.
Both channel families and both read instructions agree in this workload.

There are 13 ordinary debug warnings, ID 1328, that buffer creation ignores
the requested COPY_DEST initial state because buffers are effectively COMMON.
They are preserved as warnings, not described as a silent debug layer. There
are no error/corruption-severity diagnostics. Full source tables and individual
case observations are retained in [compact evidence](2026-10-24-exp-bgra-srgb-views/summary.json).

## Attempts and failed criteria

Attempts used the same explicit native command above, each in a new directory:

1. `20261024T140617Z-bgra-srgb-views-074fee44f6e0`: compilation failed because
   the probe used `ffx_api_dx12.h` rather than its `dx12/` include path. GPU
   execution/evaluation were blocked. Corrected the include; also corrected
   independently transcribed enum expectations before the next attempt.
2. `20261024T140805Z-bgra-srgb-views-24d659b64cc3`: compilation and GPU
   execution passed, evaluation failed. The original `2e-5` absolute sRGB
   precision criterion rejected `0.0006895404596125587` deviation from the ideal.
   This attempt remains failed. Source review then established the applicable
   code-space criterion; the correction was documented before confirmation.
3. `20261024T143052Z-bgra-srgb-views-38c8d4b061ca`: all four stages passed,
   process exit 0, with source inputs unchanged during the run. Input identity
   `f74da09ff74b48cd83dd8d8ac81ab7399111ecbcfb47fc68b501e2988bba1f7e`.

The six acceptance tests initially hit restricted temporary-directory access
errors, then passed with the required directory access. They check stale,
non-finite and partial data, wrong alpha, missing sRGB decoding, an excessive
sRGB error and the swapped-channel negative control. This is distinct from
GPU execution and is not counted as additional native coverage.

## Evidence and limits

The adjacent compact summary retains source/probe hashes, attempt outcomes,
equipment, per-route support bits, measurements and hashes of readbacks. Full
uploads/readbacks and compiler/process logs remain under ignored
`target/verification/<attempt>/`; they are not shipped in this record. Compact
hashes cannot reconstruct those GPU bytes. Rerunning the retained probe permits
new observations, not guaranteed byte-identical historical hardware results.

**Verified here:** direct DX12 allocation, exact selected views, shader-visible
channel/transfer behaviour, UNORM storage, independent CPU checks and fenced
cleanup on this configuration. The probe executes an independently specified
view table; it does not link or execute AMD's generic backend. Its public helper
metadata check does execute the pinned inline helper.

**Still unverified:** any FSR provider's acceptance or processing of these
formats, the main output callback of a shipped provider, sharpening interaction,
both generator input roles and their exact mask arithmetic, wrapper preflight/
dispatch parity, both Rust ownership families and broader devices/drivers.
The selected patches do not test full-range sRGB conformance or endpoints.
No generic access-safety, alpha-preservation, presentation encoding or provider
compatibility guarantee follows. Existing failed generator providers remain
failed historical results.

The next native step is T3 direct FSR main-input/output probes with independently
audited signals, followed by T4 generation on the known-writing provider 3.1.5.
No new public API decision is accepted by completing T1/T2.
