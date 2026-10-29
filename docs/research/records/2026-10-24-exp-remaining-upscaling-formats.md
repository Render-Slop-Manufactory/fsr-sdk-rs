# Experiment: remaining external upscaling format access

- Investigation/execution date: 2026-10-24.
- Question: which remaining fixed DX12 resource/view routes execute through
  the pinned SDK independently of the public wrapper's format restrictions?
- Source basis: [complete source accounting](2026-10-24-src-remaining-upscaling-formats.md).
- Success: exact provider identity, native calls, input integrity, complete
  fenced active writes, independent depth-plane checks, ordinary diagnostics and
  explicit cleanup. No image equality, numerical reconstruction, sharpening
  effect, mask-algorithm, quality or performance criterion.

## Setup and reproduction

Baseline `a6ca2b7` plus the inspectable `format_capabilities.cpp/.py`,
`resource_formats.py`, expanded native texture/depth equipment and independent
`typed_reference.cpp`. The wrapper implementation was being prepared separately;
reference mode constructs its own native contexts/descriptors and never invokes
public Rust dispatch. The [compact evidence](2026-10-24-exp-remaining-upscaling-formats/summary.json)
retains exact source hash inventories, attempt identities, case inventories,
settings, capabilities, exit codes and per-frame resource hashes. Each accepted
attempt reports unchanged inputs; different attempts have different identified
source states rather than an invented common final state.

Windows 11 `10.0.26300`, adapter 0 AMD RX 9060 XT, vendor `0x1002`, device
`0x7590`, driver `32.0.32015.7029`. MSVC `14.44.35207`, Windows SDK
`10.0.22621.0`, Rust/Cargo `1.99.0`, Python `3.14.1`. Ordinary debug layer;
no GBV. Exact SDK commit `60f4ea81909200d8542eca14dccb2628b763a9a3` and
four [hash-pinned public headers](../../../tools/native-headers.json).
`native.py` checks the local AMD signatures and hashes before SDK execution:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

From the repository root:

```sh
python tools/verification/format_capabilities.py --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/resource_formats.py --native --direct --smoke --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/resource_formats.py --native --direct --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The current coordinator includes subsequent public-confirmation coverage too;
the retained case settings identify the exact historical selections. Reference
contexts run selected 3.1.5/4.1.1, with generator operations only on 3.1.5.
Each child executes three separately fenced frames, reset then continuing.
The independent C++ descriptor helper uses AMD's inline resource helper, with
an explicit existing RGB32F enum for the two missing public-helper cases.
Inputs use distinct finite RGB tiles, explicit exposure 1, pixel motion,
representable depth, finite output seeds and mask seed 0x55. DSV clears produce
D16/D24 depth; a separate shader reads the precise depth SRV. CPU readback
normalizes copied plane codes and checks stencil 0x5a. These view/encoding
observations are independent of the FSR reconstruction.

## Observations

| Attempt | Result | Stages / actual execution |
|---|---|---|
| `20261024T161615Z-format-capabilities-5c75a3b5125a` | Passed, unchanged inputs | 4 stages; helper mappings and adapter bits for DXGI values 1–115; no SDK context or GPU dispatch |
| `20261024T162032Z-resource-formats-direct-93cf31e62099` | Passed, unchanged inputs | 24 stages; 15 reference contexts, 45 fenced frames |
| `20261024T162703Z-resource-formats-direct-685cb074cea3` | Passed, unchanged inputs | 128 stages; 119 reference contexts, 357 fenced frames |

All children return exit 0. The full reference matrix has 351 main calls,
57 generator calls and six no-dispatch control frames. It varies each added
typeless colour, SNORM/shared-exponent colour and RGB32F input independently,
including legal main-input UAV flags and output sharpening disabled/enabled.
Both generator colour roles are separate arms. It also exercises typeless
motion, R16F typeless depth, R16 UNORM depth, D16/D24 standard/reversed active
geometry, typeless exposure, each supplied mask role and typeless generated
output. The historical R32 typeless depth arm uses DSV equipment; ordinary
non-DSV R32 typeless depth and additional generator-UAV arms were subsequently
added to public confirmation and are not attributed to this historical run.

The adapter reports RGB32F load capability without filtering or typed UAV
capability. Its direct main inputs pass on both selected providers and each
generator input passes on 3.1.5. Thus the commented public-helper cases are an
access omission rather than a demonstrated inability to use RGB32F colour.
Shared-exponent RGB reports a typed UAV view and stores, and both input/output
SDK arms pass. D16/D24 shader depth is normalized floating data despite the
helper's collapsed D24 integer metadata; source interpretation and independent
plane observations agree. Other source exclusions are not native rejection
experiments and are not relabelled as such.

## Baseline delta and limits

These observations resolve the concrete runtime unknowns identified by D035's
residual list. They establish bounded SDK integration, not public wrapper
implementation or design acceptance. Those are separate under
[D039](../../adr/D039.md) and its implementation verification.

The retained cases select one GPU/driver/runtime deployment. Load-only RGB32F
may be incompatible with a provider using filtering; optional shared-exponent
stores may be absent elsewhere. Device bits, clean diagnostics and seed writes
prove no universal shader access bound, padding behaviour, alpha semantics,
numerical reconstruction, cross-provider compatibility or concurrency contract.
The existing 4.1.1/2.3.4 generator limitations remain unchanged. No dependency,
download, redistribution or SDK pin changed. Full readbacks and native binaries
are excluded from the repository; retained hashes identify but cannot recreate
them. After this experiment, the public confirmation filled the local disk.
With approved cleanup, its generated raw payloads and the three completed GPU
payload directories here were removed; case records, logs, original inventories
and the compact evidence remain. The current coordinator discards successful
child payloads after evaluation. No historical outcome is reclassified.
