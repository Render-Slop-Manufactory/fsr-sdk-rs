# Experiment: controlled dispatch semantics

- Date: 2026-09-30, Europe/Berlin.
- SDK: v2.3.0, pinned commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Tested source: repository maintenance work after the released 0.1.0, including
  dispatch/test module splits, public dimensions getters and a shared native runner.
  These changes do not alter the native dispatch descriptor contract.
- Scope: one analytic image, horizontal translation, two-dimensional projection
  jitter, history accumulation and reset through the public unsafe dispatch route.
- Reproduction code: [semantic comparisons](../../../crates/fsr-sdk/tests/native/semantics.rs),
  [analytic images](../../../crates/fsr-sdk/tests/native/images.h),
  [GPU equipment](../../../crates/fsr-sdk/tests/native/gpu.cpp), and
  [shared runner](../../../crates/fsr-sdk/tests/native/run.ps1).

## Question and conditions

Can controlled comparisons detect a reversed motion direction or reversed jitter,
and does resetting accumulated history match a fresh context on the same input?
Success requires finite output, zero debug errors/device removal, observable
differences for incorrect inputs and accumulated history, lower analytic error for
the matching inputs, and agreement between the two reset cases. This is a bounded
regression test, not a general FSR quality benchmark or provider support claim.

The scene uses linear grayscale RGBA16F color, ordinary R32F depth 0.5, RG16F UV
motion vectors, R32F exposure 1.0 and RGBA16F output. Creation flags remain zero.
Render dimensions are 320×180; output is 640×360; frame time is 16.667 ms. The
camera values are near 0.1, far 100, vertical FOV π/3, and unit scale 1.

For render pixel center `(x+0.5,y+0.5)` and frame `n`, the sampled scene is:

```text
u = x + 0.5 - jitter_x - 2*n
v = y + 0.5 - jitter_y
red = green = blue = 0.5 + 0.4*sin(0.8*(u + 0.37*v))
alpha = 1
motion = (-2/320, 0), except frame zero uses (0,0)
```

The eight jitter pairs, in pixel units, are `(0.25,-0.25)`, `(-0.25,0.25)`,
`(0.125,-0.375)`, `(-0.375,0.125)`, `(0.375,-0.125)`, `(-0.125,0.375)`,
`(0.0625,-0.0625)`, `(-0.0625,0.0625)`. Motion excludes jitter. Positive
projection displacement moves the projected image positively, so a fixed pixel
samples the scene at its center minus that displacement. This follows the
[pinned jitter investigation](2026-09-24-src-m6-motion-vector-jitter-semantics.md#5-jitteroffset-units-and-semantic-meaning)
and is corroborated by the pinned open FSR3 upsample calculation
`fSrcUnjitteredPos = inputPosition + 0.5 - Jitter()`; open FSR3 is not the signed
FSR4 implementation.

Five fresh contexts run separately: matching inputs, reversed X motion, negated
dispatch jitter with unchanged image sampling, matching history with reset at
frame seven, and a fresh context processing only frame seven with reset. The
first four have eight frames each, giving 33 submitted/fenced frames overall.
Each frame has its own textures, command list, allocator and upload storage.
All remain live until context destruction. GPU completion is fenced before
readback and the next frame; uncertain completion terminates the isolated child.

Comparisons use red-channel mean absolute difference in output pixels
`x=[32,608), y=[32,328)`, excluding borders. The unjittered analytic reference
samples `(x+0.5)/2 - 14` and `(y+0.5)/2`. Half-float input quantization and
reconstruction filtering remain part of the observed error.

## Environment and commands

- Windows 11 Pro, version/build 10.0.26300 / 26300.
- Hardware adapter: AMD Radeon RX 9060 XT, vendor 0x1002/device 0x7590.
- Driver: 32.0.31041.1004; provider 4.1.1, ID `0xf5a5ca1e01001001`.
- Rust 1.98.1, x86_64-pc-windows-msvc; MSVC tools 14.44.35207, compiler 19.44.35224.0.
- Windows SDK 10.0.22621.0; PowerShell 7.6.6.
- D3D12 debug layer enabled; GPU-based validation disabled.
- Both staged AMD DLL Authenticode signatures were valid.
- Loader SHA-256: `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler SHA-256: `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.

Run from the repository root with the trusted local SDK under
`external/FidelityFX-SDK/v2.3.0`. The runner initializes MSVC through its ABI
check when needed, stages helpers/DLLs and clears experimental control variables.

```powershell
./crates/fsr-sdk/tests/run-m6.ps1 -Native -Stage target/repo-review/semantic-native
./crates/fsr-sdk/tests/run-m6.ps1 -Native -Stage target/repo-review/semantic-corrected
./crates/fsr-sdk/tests/run-m5.ps1 -Native -Stage target/repo-review/lifecycle-final
```

The first semantic run intentionally remains documented below as an unsuccessful
image-convention assumption. The corrected run exited zero for Gate A, Gate B
and Gate C. Each child had a 180-second timeout; none timed out. The runner also
passed ordinary Cargo checks and all three paired C++ ABI probes. Source
fingerprints are retained in [inputs.json](2026-09-30-exp-dispatch-semantics/inputs.json).
The record reports the observations directly; transient staging logs and a Git
revision are not required to understand the result. Fingerprints normalize CRLF
to LF for text sources, matching the header-verification convention.
The staged whitespace check subsequently removed a trailing blank line from
`images.h`; its fingerprint reflects that final text. Executable code is unchanged.

## Observations, including the initial negative result

The initial image generator used `pixel_center + jitter`, repeating the earlier
smoke harness's assumption. The comparisons were sensitive, but the negated
dispatch jitter had lower analytic error: matching-labelled MAE 0.04515318 versus
0.01372520 for negated jitter. Reversed motion MAE was 0.24308752. This demonstrated
that smoke plausibility and sensitivity alone were insufficient. No production
descriptor sign was changed to hide the result.

The generator was corrected to `pixel_center - jitter`, consistent with projected
image displacement. The earlier single-frame and temporal smoke image generators
were corrected as well; their previous records remain historical execution evidence.
The new semantic test additionally requires matching-input error to be lower than
both deliberate-error cases. The corrected run observed:

| Comparison on final frame | Red-channel mean absolute difference |
|---|---:|
| Matching versus reversed motion | 0.24122791 |
| Matching versus reversed jitter | 0.03519135 |
| Accumulated history versus reset | 0.01101808 |
| Reset after history versus fresh reset | 0.00000000 |

| Input convention | Analytic red-channel MAE |
|---|---:|
| Matching motion and jitter | 0.01334815 |
| Reversed motion | 0.25143567 |
| Reversed jitter | 0.04247920 |

All 33 semantic outputs had 230,400 finite pixels, zero debug errors/corruption
and no device removal; every finish/fence/readback returned success. All five
explicit context destroys succeeded; the final device/debug checkpoints were clean.
The regression Gate A and all three Gate B frames also passed after the correction.
The separate two-context lifecycle cases passed with both explicit destroy and
ordinary Drop: the loader stayed resident after dropping the public runtime and
destroying the first context, then became nonresident after final context cleanup.
Drop does not expose a native return code.

## Limits

The result supports the tested motion direction, jitter convention, history
sensitivity and reset behavior for this horizontal, constant-depth scene on one
provider/GPU/driver configuration. It does not establish arbitrary temporal
reconstruction quality, vertical-motion direction or scale, dynamic resolution,
other formats/flags, general queue overlap, AMD thread transfer, performance or
GPU-based-validation success. Thresholds are regression comparisons for these
inputs; they are not universal image-quality tolerances. Public dispatch remains
unsafe with renderer-owned states, submission and GPU lifetimes.
