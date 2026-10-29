# Experiment: native variable-output geometry and allocation boundaries

- Date: 2026-10-19.
- Question: how do the three pinned DX12 providers handle requested output
  shrink/grow, output/display-motion allocations, motion geometry, sharpening
  and writes beyond the requested active rectangle?
- Outcome: **all 71 integration/evidence cases pass**, including 60 native
  children/180 dispatched frames and one three-frame no-dispatch control.
  **4.1.1/3.1.5 change their write footprints with requested output;
  2.3.4 retains the context-maximum footprint. Padding is not preserved.**
- Scope: test-only raw descriptors beyond D024's public fixed-output profile.
  No public API, allocation policy, provider gate or reset contract is accepted.
- Exclusions: reconstruction quality, performance, out-of-resource write safety,
  overlapping GPU execution, general cross-queue support and release archives.

## Setup and tested source

The worktree was clean before the spike. Baseline revision
`98fa751e40c2f663b42e4ff1ffc74341e31e64e9` is supplemental. The tested implementation
is development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`, with SDK v2.3.0 pinned at
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The
[source inventory and execution checks](2026-10-19-exp-variable-output-resolution/checks.json)
identify source/configuration, all commands, signatures, helper/executable hashes
and the original run/inventory hashes. The finalized run reports unchanged inputs.

| Equipment | Observed configuration |
|---|---|
| Platform | Windows 11 build 10.0.26300; x86_64-pc-windows-msvc, DX12 debug layer |
| GPU | Adapter 0, AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, feature level 12_0 |
| Driver | `32.0.31041.1004` |
| Compilers | Rust/Cargo 1.99.0; separate Rust 1.88.0 compilation; MSVC 14.44.35207; Windows SDK 10.0.22621.0 |
| Orchestration | Python 3.14.1; PowerShell 7.6.7 |
| 4.1.1 ID | `17700776140664213505` |
| 3.1.5 ID | `17700776140660019205` |
| 2.3.4 ID | `17700776140655833092` |

Preparation checks four pinned header hashes and Valid AMD signatures for the
exact loader/provider DLLs, SHA-256 respectively
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
No SDK download or strict online revocation verification was performed.

Probe code is retained in the [Rust child](../../../crates/fsr-sdk/tests/native/variable_output.rs),
[dedicated C++ equipment](../../../crates/fsr-sdk/tests/native/variable_output.h),
[coordinator/independent auditor](../../../tools/verification/variable_output.py)
and [frozen protocol](../../../tools/verification/variable-output-protocol.md).
Registration adds a named opt-in profile and guarded helper build; it does not
change production dispatch, D024 validation, raw ABI types or native criteria.
Public construction/selected-provider queries retain the real context, but raw
recording deliberately bypasses the allocation/geometry policy under investigation.
All five resources are compared with the pinned independent C++ inline conversion.

## Reproduction and evidence

```powershell
python tools/verification/run.py run experiment-variable-output --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py verify-run --run target/verification/20261019T203246Z-experiment-variable-output-baf868a0a066
python tools/verification/run.py export --run target/verification/20261019T203246Z-experiment-variable-output-baf868a0a066 --output target/verification-evidence/NEW-EXCLUSIVE-DESTINATION
```

The recorded invocation exits 0 with `status=passed`, 71/71 cases and unchanged
inputs. Integrity verification and sanitized export also pass. No failed,
aborted, timed-out, blocked or native-rejected child occurred in this attempt.
Passing here means completed integration and audited observations, not a promise
that every provider honours `upscaleSize`.

Every provider runs the protocol's 20 workloads in fresh isolated processes:
stable maximum/small output, aligned and rounded transitions, width/height-only
changes, maximum/active/oversized allocations, dynamic flag true/false,
sharpening off/on, render UV motion and display UV/pixel motion. Each child
retains one context for three frames. Maxima are render 320×180 and output 640×360;
render/colour/depth stay fixed. Output and display-motion allocations are maximum,
current active dimensions or 672×384. Active output never exceeds context maxima.
Output is reseeded to half-float 65504 in fresh resources each frame.

The linear plane translates by 1/128 normalized units per axis/frame. Selected
jitter queries use current requested output. Motion is jitter-excluded UV,
scaled by active motion dimensions, or its corresponding display-pixel encoding
with descriptor scale `(1,1)`. Reset is true only on frame zero. Every previous
frame is CPU-fenced before the next records, including across replacement textures
and separate per-frame queues. There is no overlap or automatic resize reset.

The helper compares every GPU-read input byte with its upload. Python independently
reconstructs binary32/truncated-binary16 colour, depth, motion and exposure and
recomputes output finite/written counts, outside writes and bounding coordinates.
A pixel counts as written when any channel differs from the seed; finiteness
checks all four channels. These are integration criteria, not reconstruction tests.
All 183 frames pass these checks, with zero debug errors, device removals and
input mismatches. All 61 contexts complete terminal destruction.
[Measurements](2026-10-19-exp-variable-output-resolution/measurements.json) retain
every selection, frame, descriptor, metric and input/output SHA-256;
[comparisons](2026-10-19-exp-variable-output-resolution/comparisons.json) retain
matched byte comparisons. Their [inventory](2026-10-19-exp-variable-output-resolution/inventory.json)
hashes normalized UTF-8/LF bytes with scoped Git preservation.

## Observed write footprints

For the rounded sequence with maximum or 672×384 output allocations:

| Requested active output | 4.1.1 / 3.1.5 written rectangle | Pixels written outside active | 2.3.4 written rectangle / outside pixels |
|---|---|---|---|
| 637×357 | 640×360 | 2,991 | 640×360 / 2,991 |
| 511×287 | 512×288 | 799 | 640×360 / 83,743 |
| 637×357 | 640×360 | 2,991 | 640×360 / 2,991 |

The larger allocation also leaves an observable seeded margin beyond these
rectangles. These patterns hold with sharpening off/on in the measured cases;
no general alignment formula or preservation rule follows from this small geometry selection.
Aligned 640×360 → 512×288 → 640×360 produces corresponding active footprints on
4.1.1/3.1.5. On those providers, width-only shrink to 511×357 writes through
512×360, and height-only shrink to 637×287 through 640×288. All requested active
pixels are finite and changed from the seed.

With active-sized allocations, all providers overwrite the available requested
rectangle and no outside pixels exist to observe. This is **not evidence that
their shader stores stay within those allocations**. In particular, 2.3.4's
maximum-sized write behaviour remains visible whenever sufficient allocation
exists; a smaller readback cannot expose an attempted store outside the texture.
Ordinary debug validation is not GBV or an out-of-bounds detector.

## Provider, flag and motion distinctions

**2.3.4:** requesting stable 511×287 in a 672×384 allocation still writes 640×360.
For all three phases its complete 640×360 crop is byte-identical to the matching
stable-maximum workload. The paired colour/depth/motion/exposure hashes match.
This corroborates the source intake's reported FSR2 omission of active output;
it does not establish a universal provider-name capability rule.

**Dynamic flag:** the rounded sequence submits/fences with the flag both enabled
and disabled on all providers. 3.1.5/2.3.4 output bytes match between these arms
for all three frames. On 4.1.1 the first frame matches, but frames one and two
differ despite matched packed inputs. Thus successful execution without the flag
does not establish that it is irrelevant. No image-quality or internal-history
explanation is selected from those differences.

**Display motion:** maximum-, active- and oversized-motion allocation cases
complete with both sharpening settings. Matched maximum-allocation UV/pixel
motion arms produce identical output bytes in all three frames on each provider.
This is bounded encoding/descriptor consistency on the analytic plane, not proof
of general motion correctness, exact sampling coverage or a temporal-quality result.
For a provider ignoring requested output, requested-sized motion allocations
therefore still cannot be declared generically sufficient from these passes.

## Baseline delta, limitations and contract questions

The earlier source intake now has actual Windows GPU observations for its
main candidate combinations. Faithful raw access can vary output on 4.1.1/3.1.5
in this bounded workload; no-ops and rounded padding writes need explicit contract
treatment. **Inference:** metadata coverage of only the requested active region
is not, by itself, a sufficient general claim about native access boundaries.

Before public implementation, decide:

- Which output/display-motion allocations are permitted, including above-maxima
  allocations, and which actual-provider compatibility/write obligations remain
  with the caller. Active-sized successes do not settle out-of-resource safety.
- How to expose requested output when a provider demonstrably retains maximum
  output, without claiming generic support from enumeration or OK returns.
- The wrapper's dynamic-flag policy. This spike supplies no universal prerequisite
  or permission to ignore the flag; 4.1.1 has observed output differences.
- How to document native access to the whole supplied output allocation while
  preserving renderer-owned resource states, fences, replacement lifetimes and reset.

D024's fixed-output implementation and render-side allocation rules remain intact.
No automatic reset, guessed alignment restriction, provider fallback/allow-list,
new public unsafe contract or API implementation is introduced. Successful native
recording/cleanup is distinct from rejection safety; the prepared terminal native
error branch did not execute here. Other providers, dimensions, input profiles,
concurrent/overlapping frames and exact historical GPU bytes remain unverified.
GBV's earlier stall is not reopened by this ordinary debug-layer experiment.

Stable movable/all-target Clippy, exact Rust 1.88.0 all-target compilation, five
auditor negative-control tests and the complete ordered profile-inventory check
pass. No additional dependencies were introduced. Raw input/output payloads,
full logs and the verified export remain under ignored `target/`; only compact
measurements/provenance are retained here. Local paths and transient addresses
are normalized. This record does not synthesize the findings into a new decision
or complete final release verification.
