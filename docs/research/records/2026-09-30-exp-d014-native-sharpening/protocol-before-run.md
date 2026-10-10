# Predeclared D014 sharpening comparison protocol

Snapshot taken from D014 before native execution on 2026-09-30. Outcomes and
configuration are recorded in the adjacent experiment record. The text below
is the policy evaluated by that spike; it is not an accepted release requirement.

This is a proposed fixture-specific acceptance policy, not measured data, an
AMD guarantee or an image-quality standard. Approve and record the fixture,
metrics and thresholds before inspecting enabled-sharpening outputs. The
thresholds below apply to the normalized grayscale frequency scene and require
a directional detail response in addition to output differences.

Use 320x180 render and 640x360 output, the existing fixed formats/flags, explicit
exposure `1.0`, ordinary constant depth `0.5`, camera/time values and eight jitter
pairs from the [semantic experiment](../research/records/2026-09-30-exp-dispatch-semantics.md).
Keep motion zero and the scene stationary. For unjittered render coordinates
`u = x + 0.5 - jitter_x`, `v = y + 0.5 - jitter_y`, use two separate scenes:

- Frequency detail: RGB `0.5 + 0.4*sin(0.8*(u + 0.37*v))`, alpha `1`.
- Slanted edge: RGB `0.1` for `u < 128 + 0.37*v`, otherwise `0.9`, alpha `1`.

Each repetition runs both scenes. For each scene create five fresh contexts:
disabled A, disabled B, enabled zero, enabled `0.35` and enabled `1`. Each context
processes the same eight-frame sequence, resetting only on frame zero. Keep
resources/list/allocator lifetimes, submission order and fence/readback cleanup
identical. Use three independent isolated process repetitions, each with the
180-second watchdog. Record SDK/DLL/provider/GPU/driver and fixture fingerprints.

The primary comparison uses final-frame red-channel values in the output ROI
`x=[32,608), y=[32,328)`. First compare the raw half-float red-component bits
of disabled A and B inside that ROI, excluding row padding. Bit-identical values
are the primary repeat control. Compute the following metrics in f64 from the
read-back half-float values as well:

- `N`: mean absolute difference between disabled A and B (repeat noise).
- `D_v`: mean absolute difference between strength `v` and disabled A.
- `C_v`: fraction of ROI pixels with absolute difference at least `1/1024`.
- `G_v`: mean absolute horizontal neighboring-pixel difference, with both
  neighboring pixels inside the ROI; `G_off` is the disabled-A value.
- `B_v`: absolute difference of ROI mean red between `v` and disabled A.

For the frequency scene, **each of the three repetitions** must meet:

| Measure | Predeclared criterion |
|---|---|
| Repeat control (default) | Bit-identical disabled-A/B red components throughout the ROI |
| Repeat control (predeclared numerical fallback) | `N <= 0.0001`, only under the fallback policy below |
| Nonzero-strength change | `D_v >= 0.001` for both `v=0.35` and `v=1` |
| Changed region | `C_v >= 0.01` for both nonzero strengths |
| Detail response | `G_v >= 1.01*G_off` for both nonzero strengths, with `G_off > 0` |
| Mean preservation | `B_v <= 0.005` for both nonzero strengths |
| Execution | Finite RGBA throughout every output, completed fences/readbacks, clean debug/device checks and cleanup |

Bit equality tests deterministic repetition directly. Record `N` even when it
is zero. The numerical fallback is available only for a configuration whose
non-bit-identical repetition has been investigated and documented separately,
with that policy chosen before the validation run. It is not an automatic pass
when the default control fails. Report which policy was used; a failed default
run stays a failure and a changed policy requires a new complete run.
Because `N` averages the ROI, it may be below a single half-float spacing when
only some pixels differ; it is not a claim that each pixel changed by that amount.

The difference floor is ten times the permitted fallback repeat noise; `1/1024` is an
explicit pixel-level difference cutoff for this normalized half-float signal.
The 1% gradient increase measures response in the expected sharpening direction;
the mean bound prevents a simple brightness shift satisfying the effect gate.
These are deliberately selected test thresholds, not inferred SDK requirements.

Report all metrics for enabled zero as well, without requiring a difference from
disabled. For the edge scene, report repeat noise, profiles normal to the known
edge, transition width and bright/dark overshoot for every setting. The edge
scene must pass execution checks, but no universal edge-width, overshoot or
strength-monotonicity bound is asserted by this proposal. It is an execution
and diagnostic case, not an independent sharpening-effect gate. Its successful
execution alone must not be described as evidence of sharpening effect or quality.

If repeat noise or effect criteria fail, investigate fixture, mapping and provider
behavior and report the failure. Changing thresholds/scene requires an explicit
documented amendment and a new complete run; do not relax thresholds after seeing
results and report the original gate as passed. This protocol establishes a
bounded sharpening response on its recorded configuration, not broad quality.
