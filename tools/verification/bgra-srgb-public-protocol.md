# Public BGRA/sRGB wrapper verification

`native-bgra-srgb` verifies [D038](../../docs/adr/D038.md) through both public
ownership families. It reuses the [typed resource laboratory](typed-formats-protocol.md)
and [direct SDK inputs](bgra-srgb-fsr-protocol.md) with `MODE=public`.
No reconstructed-image equality, numerical tolerance, CPU mask algorithm,
sharpening effect or quality criterion is selected.

```sh
python tools/verification/run.py run native-bgra-srgb --require-passed --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Select the actual adapter index. Require Windows x64/MSVC, ordinary debug layer,
four pinned headers and exact signed local loader/provider DLLs. The coordinator
runs independent C++ ABI checks, builds/stages the Rust laboratory and DX12
helper, and uses the shared watchdog, source identity and evidence inventory.
It downloads and publishes nothing. Raw evidence stays under ignored `target/`.

## Inventory and acceptance

There are ten equipment stages and 92 isolated public workloads (102 total).
Each workload has a reset and two continuing frames, fences every submission,
checks the selected live provider and explicitly destroys its context.

- Main inputs/outputs: BGRA8 UNORM, RGBA8 sRGB and BGRA8 sRGB, local/movable
  owners on 3.1.5/4.1.1; each output with sharpening disabled/enabled.
- Each main colour candidate also runs as a UAV-capable input.
- Each generator colour role independently uses each candidate on 3.1.5 with
  ordinary/UAV-capable and all-black input configurations. Active render shrinks
  320×180 → 160×90 → 320×180 within maximum allocations.
- No-dispatch controls preserve complete output seeds; an existing float32
  colour/float16 output supplies the baseline resource configuration.

Optional preflight and actual recording use production inspection. The laboratory
compares each main native resource descriptor with AMD's inline helper before
recording. Generation uses public validation/generation methods; a UAV barrier
and transition precede mask consumption. Require exact format/role inventory,
post-dispatch input bytes, complete readback sizes, active output/mask writes,
finite floating output, successful native statuses, zero ordinary error/device
removal, fence completion and cleanup. Hashes are observations, not expected
image values or equivalence gates. Seed checks are bounded to these inputs.

Main maxima remain 320×180 / 640×360, zero jitter, explicit exposure/pre-exposure
one and pixel-space motion. Native colour flags remain explicit; format infers
no hint. The movable cases make no new handoff or concurrent-use claim.

## Controlled and ordinary checks

Local/movable COM fixtures model sRGB SRVs without UAV support and matching
UNORM UAVs with only their required capabilities. Cover exact metadata, query
order, role-specific HRESULTs, missing bits, conditional casting/errors, both
recording operations, no native entry on rejection, metadata/device precedence,
retry, reference balance and terminal native failure. Existing baseline/RG32F/
depth checks remain applicable. Stable/MSRV, consumers, docs and independent
header assertions complete the source checks.

Retain compact commands, input identities, outcomes and limits in a dated
implementation record. Public admission establishes no universal provider support,
arbitrary shader access bounds, untouched padding, alpha preservation or new
renderer ownership. Final extracted-package verification remains separate.
