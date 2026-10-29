# Typed resource integration protocol

This opt-in Windows x64/MSVC + DX12 protocol implements the bounded verification
selected in [D035](../../docs/adr/D035.md). The `native-formats` coordinator owns
the executable inventory in [typed_formats.py](typed_formats.py); it does not
publish packages, download runtimes or select quality/performance criteria.

## Inputs and native routes

Use the four header hashes in [native-headers.json](../native-headers.json),
signed pinned local loader/provider DLLs and an explicitly selected AMD adapter.
Both public owners select recorded providers 4.1.1 and 3.1.5 through enumerated
IDs and verify live identity. Generation uses only selected 3.1.5; 4.1.1/2.3.4's
existing generator no-write failures remain separate evidence.

The DX12 helper creates single-sample/mip/slice maximum allocations, queues,
lists, uploads and tight readback copies. Render size is 320×180, output 640×360,
and exposure 1×1. A separate C++ translation unit uses the pinned public headers
to create/query/dispatch/destroy its own context and retain its own loader. It
constructs resource descriptors with AMD's helper and scalar/creation mappings
independently of Rust. Public main descriptors are additionally compared with
AMD's resource helper before recording.

The coordinator generates uploaded bytes, including independently packed/decodeable
RGBA32F, RGBA16F, R11G11B10F, RGB10A2 UNORM, RGBA8 UNORM and R16F values.
Uploads contain a spatial RGB pattern moving four pixels per frame, two device
depth levels, pixel-space motion, and explicit exposure one. Reversed depth uses
complementary stored values with the existing ordered finite camera parameters.
Generator contrast differs in a known region; zero-mask controls use identical
decoded black in both colour formats, avoiding quantization disagreement.

Each workload runs an isolated public child and an isolated C++ reference child,
each with one reset frame followed by two continuing frames. Workloads vary one
role at a time, cover UAV-capable versus ordinary inputs, standard/reversed R16F
depth, each new output with sharpening disabled/enabled, above-one floating
colour and a mixed-format active-output shrink/grow sequence. Generator inputs
are varied independently, with contrast/zero controls, render-region shrink/grow
and subsequent generated-mask consumption. All sequences are serial and fenced.

## Frozen acceptance criteria

- Exact child selection, exit success, three frame records, selected provider
  identity, complete capability records and explicit successful cleanup.
- Exact input byte readbacks after typed upload/copy, including tight row counts
  and native resource-helper metadata parity. The output format may differ from
  the input without changing either descriptor's actual metadata.
- Required per-role device support bits, no ordinary debug errors or device
  removal, and confirmed completion before reading or destroying resources.
- Every active main-output pixel differs from its finite magenta seed and
  decodes to finite components. A no-dispatch control preserves the entire seed.
  The selected RGB patterns do not contain that seed; this is a workload-specific
  write observation, not a general detector for arbitrary image content.
- Each generated active mask has no retained `0x55` sentinel; equal-black
  controls are zero, and contrast controls have a nonzero nonsentinel response.
  UAV ordering and transition precede supplied-mask consumption.
- Public and C++ routes have byte-identical complete same-format output and,
  where generated, mask readbacks at every frame. This exact criterion is frozen
  before the first run. No cross-format equality or new numerical tolerance is
  selected. A separately changed-RGB control must change at least one output.

Unsupported device bits, native errors, incomplete writes, missing controls or
missing reference results cannot pass. Failed recording invalidates its list
before terminal cleanup. Unproven GPU completion ends the isolated process
without unwinding native objects. Destruction errors retain the reference's
loader/context dependencies. The coordinator preserves failed attempt outcomes.

These observations establish bounded mapping/integration, not reconstruction
quality, universal format compatibility or access bounds. D024 permits writes
outside active output within the full allocation; whole-resource comparisons
record the tested routes' agreement without promising untouched padding. No GBV
run, new provider allow-list, colour conversion or performance gate is selected.

Raw uploads/readbacks remain ignored local diagnostics under `target/verification`.
Retain dated commands, input/tool/provider identity, compact measurements and
failure/limitation summaries in a self-contained release record. Final archive
and extracted-package verification remain separate.
