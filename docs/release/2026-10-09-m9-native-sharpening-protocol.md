# M9 native sharpening integration protocol

- Selected: 2026-10-09, under the authorized direct-native-control strategy.
- Status: **selected, not executed**. M8 implementation checks are separate from
  this M9 GPU gate. Publication and final package validation remain separate.
- Contract: [D014](../adr/D014.md); direct-control invariant: [D016](../adr/D016.md).

## Acceptance contract and evidence

The wrapper guarantees checked construction and exact descriptor mapping:
`Disabled -> (false, +0.0)` and `Native(v) -> (true, v)`, preserving the `f32`
bits, including negative zero. Invalid values never enter FFI. Native recording
may fail; its error, poison and cleanup contract remains in force.

This protocol selects no wrapper filter, quality profile, additional quality
parameter or image-quality promise. The original
[2026-09-30 protocol and failed results](../research/records/2026-09-30-exp-d014-native-sharpening.md)
remain historical evidence. Its gradient, mean-shift, image-difference and
changed-area thresholds were not adopted as release gates. None is a condition
of this new gate. This selection is explicitly informed by those known failures;
it does not make the original experiment pass.

Before GPU execution, require the final public API's deterministic scalar,
descriptor and controlled-FFI tests, including default Disabled, enabled positive
and negative zero, `0.35`, `1`, disabling after enabling, invalid values and
preservation of other descriptor fields. Keep existing preflight, unknown-error,
poison-first and terminal cleanup checks. Controlled fixtures establish wrapper
behavior, not AMD runtime or GPU behavior. An OK return or image difference
alone cannot establish exact parameter forwarding.

## Workload and controls

Use the signed SDK v2.3.0 Windows/DX12 profile, zero creation/dispatch flags,
320x180 render and 640x360 output, and the existing five texture roles/formats.
Use zero motion, explicit exposure `1.0`, ordinary depth `0.5`, frame time
`16.667` ms, camera near `0.1`, far `100.0`, vertical FOV `pi/3` and metres factor
`1.0`. All renderer resource-state, residency, aliasing and lifetime obligations
continue to apply. Enable the ordinary D3D12 debug layer; GBV is excluded as in
the recorded sharpening profile.

Retain the two stationary scenes from the original workload. At unjittered
coordinates `u = x + 0.5 - jitter_x`, `v = y + 0.5 - jitter_y`, use RGB
`0.5 + 0.4*sin(0.8*(u + 0.37*v))` for the frequency scene and RGB `0.1` when
`u < 128 + 0.37*v`, otherwise `0.9`, for the slanted edge. Alpha is `1`.
Upload through the existing test equipment's half-float conversion.

For each scene create three fresh public `Upscaler` contexts: switching A,
switching B with identical history, and an always-Disabled control. Construct
every frame with the public `UpscaleResources` and `UpscaleDispatch` API and
select sharpening with the public setter. Do not patch a native descriptor or
use the historical spike's private dispatch override.

| Frame | Jitter `(x, y)` | Switching A and B | Always-Disabled control | Reset |
|---|---|---|---|---|
| 0 | `(0.25, -0.25)` | Disabled | Disabled | `true` |
| 1 | `(-0.25, 0.25)` | Disabled | Disabled | `false` |
| 2 | `(0.125, -0.375)` | Disabled | Disabled | `false` |
| 3 | `(-0.375, 0.125)` | Disabled | Disabled | `false` |
| 4 | `(0.375, -0.125)` | Native(0) | Disabled | `false` |
| 5 | `(-0.125, 0.375)` | Native(0.35) | Disabled | `false` |
| 6 | `(0.0625, -0.0625)` | Native(1) | Disabled | `false` |
| 7 | `(-0.0625, 0.0625)` | Disabled | Disabled | `false` |

`Native(v)` abbreviates `Sharpening::Native(NativeSharpness::new(v)?)`.
Frames 3-7 verify the required same-context sequence with continuing temporal
history. The first three frames establish history; do not recreate the context
or add resets at mode changes. Record each sequence in order and submit each
successful recording exactly once. Complete each frame's fence and readback
before reusing its list, allocator or textures.

Run both scenes and all three contexts in each of three independent isolated
process repetitions: 48 outputs per repetition, 144 total. Retain the existing
180-second child watchdog. Every repetition must complete; there is no automatic
retry, numerical fallback or dropping of a failed repetition.

## Pass, failure and diagnostics

Every valid-input native case must record successfully and complete GPU
execution. Check every frame for the expected output dimensions, completed
fence/readback, finite RGBA over the entire output, and replacement of the
equipment's output sentinel at every pixel. Sentinel replacement checks output
writes, not a minimum visual difference between modes. Require zero ordinary
debug errors, no device removal, and successful explicit terminal cleanup after
completed work. After teardown, inspect the live device's debug/error state.

Compare matching frames of switching A/B, and switching A against the
always-Disabled control at frames 3-7, including the return to Disabled. Save
red half-float components without row padding and report bit equality and f64
mean absolute difference over `x=[32,608), y=[32,328)`. These comparisons are
diagnostics with no equality, difference, gradient, brightness, monotonicity or
image-quality pass threshold. In particular, frame 7 does not promise identity
with the always-Disabled control's different temporal history. Preserve all
observed differences rather than interpreting them as proof of mapping.

An unexpected native error, GPU failure, timeout or failed execution/cleanup
check fails the integration run. Never submit a failed native recording;
invalidate it, complete previously submitted work when completion can be
established, and destroy the poisoned context according to D011/D014. Preserve
possibly in-flight dependencies when completion is unknown and let the isolated
process contain that failure. Expected errors are checked separately with the
controlled fixtures; they do not turn a failed valid-input AMD run into a pass.
Revise this protocol explicitly before any changed acceptance run; retain failed
runs and explain changed inputs or conditions.

M9 also requires the existing disabled lifecycle, single-frame, three-frame and
motion/jitter/reset regressions with their own existing assertions. Stable/MSRV,
feature, Rustdoc, ABI, migration and final package/consumer checks remain gates.
This protocol does not add GBV, another provider, thread transfer or general
quality coverage. Contradictory safety evidence still requires contract review.

## Execution record

Implement a separate M9 test/runner entry for this public-API protocol; the
historical sharpening runner keeps its original effect assertions. Before
execution, record the concrete commands/test selectors and source/fixture
fingerprints. Preserve sanitized commands, settings, native return codes,
per-frame fence/readback/write/debug/device observations, comparison data,
child exits/timeouts and cleanup results in a dated repository release record.
Identify SDK, DLL hashes/signatures, provider, OS, GPU and driver. Report
compilation, controlled FFI, AMD recording and completed GPU execution separately.

Neither selecting this protocol nor completing M8 supplies this native evidence
or marks M9 complete. The report must be understandable without external CI,
unpublished history or a Git revision as its only input description.
