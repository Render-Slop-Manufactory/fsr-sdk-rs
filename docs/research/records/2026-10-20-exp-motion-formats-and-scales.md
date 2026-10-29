# Experiment: RG32F motion and signed per-axis native scales

- Investigation date: 2026-10-20.
- Question: do selected DX12 providers execute RG32F motion, and do signed,
  asymmetric scale encodings produce the same result as equivalent pixel motion?
- Result: **43/43 final cases pass, with 120 dispatched frames and four
  no-dispatch control frames. All 24 frozen hypotheses pass on 4.1.1/3.1.5.**
- Scope: disposable local contexts and raw descriptors. No production binding,
  public API, resource-profile or numerical-domain change is accepted.
- Exclusions: image quality, performance, typeless/sRGB/depth extensions,
  display-motion/variable-output rules, NDC conversion, provider 2.3.4,
  arbitrary zero/extreme scales, concurrency and release verification.

## Baseline, equipment and tested changes

The worktree was clean at the start, with development wrapper `0.3.0` and sys
`0.2.0`. Baseline revision `7494a9ff93aff0debd51f233e023aeb7b98ad8f4` is supplemental;
the retained [checks](2026-10-20-exp-motion-formats-and-scales/checks.json) contain
the tested source/configuration file hashes, commands, statuses, executable/helper
identities, selected providers and signatures. The completed spike's source
digest is `bfe65a7879acd7c76361046b82bdca0fd59e95ab4e991aa7f0bba6a158a70a75`;
its input/configuration identity is
`8a92b76dfe37b1c723c943d5af2a3493c4819692b7f9469dea3d1647e7467a09`.
All three finalized attempts report unchanged inputs.

The SDK remains v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`,
upscaler API 4.1.1. Four local headers match the existing normalized-text pin
manifest. Loader/provider DLL SHA-256 values are respectively
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`;
both staged Authenticode signatures are Valid AMD signatures. There was no SDK
download or strict online revocation verification.

| Equipment | Observed configuration |
|---|---|
| Platform | Windows 11, build 10.0.26300; x86_64-pc-windows-msvc; ordinary DX12 debug layer |
| GPU | Adapter 0, AMD Radeon RX 9060 XT; vendor `0x1002`, device `0x7590`; feature level 12_0 |
| Driver | `32.0.31041.1004` |
| Toolchain | Rust/Cargo 1.99.0; MSVC 14.44.35207; Windows SDK 10.0.22621.0 |
| Coordinator | Python 3.14.1; PowerShell 7.6.7 |
| Provider 4.1.1 | ID `17700776140664213505` |
| Provider 3.1.5 | ID `17700776140660019205` |

Probe code lives in the [Rust child](../../../crates/fsr-sdk/tests/native/motion_scale.rs),
[C++ input generator](../../../crates/fsr-sdk/tests/native/motion_scale.h) and
[coordinator/auditor](../../../tools/verification/motion_scale.py).
The [protocol](../../../tools/verification/motion-scale-protocol.md) was written
before the first native attempt. It includes a candidate contract without
selecting a public representation, zero policy or additional formats.

Registration adds one ignored test and one explicitly selected profile.
The existing output helper gains a motion-format argument with its original
RG16F default, format-aware readback stride and a separate motion export.
All production source and historical records remain unchanged. New RG32F native
format value `6` is probe-local, compared against pinned C++ conversion; it does
not extend production bindings. Existing formats use the production converter.
Every resource description matches the independent C++ inline helper. RG32F
motion is an ordinary typed, single-mip/slice/sample 320×180 Texture2D with no UAV
allocation flag and native read-only usage; no input-UAV or typeless claim follows.

## Method and frozen criteria

Each provider runs five arms for X-only, Y-only and combined motion: RG16F pixel
baseline, identical RG32F pixel input, RG32F positive asymmetric `(2,4)` scales,
RG32F signed asymmetric `(-2,-4)` scales, and deliberately wrong-sign controls.
Equivalent encodings divide stored X/Y motion by the corresponding scale.
Signed and wrong-sign arms have byte-identical GPU inputs and differ only in
descriptor scale signs. Baseline motion after reset is `(-4,0)`, `(0,-2)` or
`(-4,-2)` render pixels. All values are exactly representable dyadic numbers.

Each fresh child retains one context over four frames, reset only on frame zero.
Render is 320×180 and output 640×360, with maximum allocations, zero jitter,
patterned dyadic colour moving four X and/or two Y pixels per frame, depth 0.25,
explicit exposure 1, no masks/sharpening/HDR, and fresh output seeded to 65504.
Every previous submission is fenced before replacement resources or cleanup.

The helper reads back all five resources. Python independently reconstructs
packed colour/depth/motion/exposure bytes and recomputes finite/written output
counts. Integration requires exact inputs, complete finite output writes,
confirmed fences, no device removal/debug errors and terminal destruction.
A four-frame no-dispatch control must preserve every seed.

Frozen equivalence predicts byte-identical output in all four frames for the
float/asymmetric/signed arms. Wrong-sign output must match at reset and differ
in at least one subsequent frame for every axis/combined scene. Hypothesis
failures would remain `observed`, separately from integration failures; criteria
were not relaxed after execution. A 180-second child watchdog bounds execution.

## Commands and outcomes

```powershell
python tools/verification/run.py run experiment-motion-scale --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py verify-run --run target/verification/20261020T214515Z-experiment-motion-scale-9e540005eb20
```

Both native attempts used the same run command above. Their complete command
records and original run/inventory hashes are in the retained checks.

| Attempt | Exit/status and observations |
|---|---|
| `20261020T214434Z-experiment-motion-scale-6217bcc3984f` | Exit 1, failed profile: 10 preparation cases passed; equipment control completed/audited four frames but stayed `observed`; 30 dispatch children and two comparisons were blocked. The new coordinator omitted promotion from observation-only subprocess status after successful independent evaluation. No provider dispatch occurred. |
| `20261020T214515Z-experiment-motion-scale-9e540005eb20` | Exit 0, passed 43/43, 21:45:15–21:46:42 UTC. The coordinator explicitly marks independently accepted children passed. Thirty dispatch contexts/120 frames, one control context/four frames, two provider comparisons. No native rejection, timeout, abort, device removal, debug error, input mismatch or failed cleanup. |
| `20261020T214841Z-motion-equipment-regression-514de40cb4d8` | Exit 0, passed 12/12: rebuild without the new motion-export flag, existing three-frame output control and existing three-frame 4.1.1 stable-maximum workload through the original export. Both contexts destroy successfully; default RG16F geometry equipment is unchanged in this bounded check. |

Integrity verification passes for all three finalized attempts, including the
failed first profile. The initial error was coordinator bookkeeping, not an SDK
rejection. The corrected attempt retains the same workload, oracle and frozen
hypotheses; the earlier failed attempt is not overwritten.

Supplementary helper regression is reproducible by copying the retained
[driver](2026-10-20-exp-motion-formats-and-scales/equipment-regression.py) to
`target/motion-equipment-regression.py` and running `python target/motion-equipment-regression.py`.
It selects only two existing geometry children and their prerequisites, not the
complete variable-output profile. A preliminary driver invocation exited 1 on
incorrect `Runner` argument order before any case/GPU execution; the corrected
driver is retained byte-identically to its tested external input.

## Measured results

[Measurements](2026-10-20-exp-motion-formats-and-scales/measurements.json) retain
all descriptors, GPU metrics and input/output SHA-256 values for both motion
attempts, including the first control. [Comparisons](2026-10-20-exp-motion-formats-and-scales/comparisons.json)
retain all 96 paired frame results and all 24 hypotheses. Exact-equivalence
comparisons are 72/72 equal. All six wrong-sign reset comparisons are equal;
all 18 subsequent wrong-sign comparisons differ.

| Wrong-sign scene | 4.1.1 changed pixels, frame 3 | 3.1.5 changed pixels, frame 3 |
|---|---:|---:|
| X only | 230,393 / 230,400 | 219,573 / 230,400 |
| Y only | 230,348 / 230,400 | 220,317 / 230,400 |
| X and Y | 230,316 / 230,400 | 220,285 / 230,400 |

**Verified observation:** both selected providers complete the tested typed
RG32F render-motion path. Changing scale signs alone changes post-reset output;
compensating the stored motion restores exact baseline output. Independent
X-only/Y-only results support consumption of both scale components for these
small values. This is integration evidence, not an assessment of reconstruction
quality or an exact trace of internal closed-provider arithmetic.

Supplementary checks pass: all 255 verification-infrastructure tests complete
(four intentional skips), including negative controls for wrong scale bits,
provider identity, input corruption, incomplete fences, duplicate completion,
insensitive output and false equivalence. `cargo fmt --all -- --check`,
`cargo clippy -p fsr-sdk --features dx12,movable --lib --tests --offline -- -D warnings`
and `cargo test --workspace --features fsr-sdk/movable --offline` pass.
Rust test compilation includes both owner families, but this new native spike
records only raw local contexts. Independent MSVC ABI probes pass in each run;
no new MSRV or final-package verification was performed.

Initial Python temporary-file writes and a Rust compile fixture were denied by
the Windows sandbox; the corresponding checks were rerun outside the sandbox
and passed. An initial new profile unit assertion also forgot the automatically
added equipment prerequisite; it was corrected before the native run, with all
four new tests then passing. Cargo reported incremental hard-link fallback to
copying; no compilation failure followed from that warning.

## Baseline delta, retention and remaining choices

The [source record](2026-10-19-src-upscaling-formats-and-motion-scale.md) supplied
candidate format and independent signed-scale semantics. This experiment adds
actual-provider execution and sensitive equivalence evidence on one GPU/driver.
The original source record and accepted UV/pixel/allocation contracts are intact.
The inputs are also exactly representable in half precision; no additional
RG32F precision or numerical-range benefit is established.

The next design review can use RG32F render motion and exact explicit-scale
forwarding as evidenced candidates. Public API shape, per-role format/view/device
validation, zero/negative-zero/extreme finite scale policy and both-family
implementation remain unaccepted. This does not justify all converter formats,
display-motion equivalence, other providers, generic NDC conversion, safe native
numerical bounds or a capability allow-list. Ordinary debug silence does not
prove bounded resource access; GBV and its historical stall were not revisited.

Compact evidence uses UTF-8/LF with a scoped byte-preservation rule and
[inventory](2026-10-20-exp-motion-formats-and-scales/inventory.json). Personal
paths and transient pointer identities are sanitized; adapter LUID is replaced
by `<adapter-luid>`, while provider IDs, return codes and hardware IDs remain.
Raw payloads and full local attempts remain under ignored `target/` and are not
copied into the repository. Retained hashes cannot recover those original GPU
bytes if local diagnostics are later removed, and reruns need not reproduce
identical outputs on another configuration. This record does not integrate a
topic synthesis, change roadmap completion or accept the U05 design.
