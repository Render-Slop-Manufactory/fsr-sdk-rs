# Experiment: selected Proton temporal regressions on Steam Deck

- Investigation dates: 2026-10-13–14, Europe/Paris. Attempts ran on 2026-10-13
  21:38–22:27 UTC, spanning local midnight.
- Question: can the selected Windows temporal protocols run through common
  Proton orchestration, with unchanged numerical criteria and explicit diagnostic
  differences, on the available SteamOS host?
- Result: cut/reset recovery passed. Quality/history sensitivity and long-history
  background budgets failed reproducibly. Jitter produced complete data, but one
  launcher failure blocked the required full image verdict. **V07d acceptance
  remains open.** Initial V07/V07e proofs are not reopened.
- Exclusions: V08 acquisition/offline execution, V09 forced timeout cleanup,
  full native regression, Steam Machine execution, native Linux/Vulkan support,
  Windows diagnostic equivalence and universal image quality.

## Declaration and implementation

The [selection declared before execution](2026-10-13-exp-proton-temporal-regression/protocol-before-run.md)
requires the original quality, long and reset matrices and a bounded jitter
selection. Budgets, sensitivity predictions, windows and three process blocks
were frozen before observing results. No criterion was relaxed after failure.

The starting worktree was clean at `b0b856ec4d9099aeca77668eebf6291913ffc200`.
Current source adds four `proton-temporal-*` profiles through the existing
cross-build/launcher, temporal matrix generator, input oracle, offline analysers
and compact retention. Windows selectors still require DirectX diagnostics;
separate `proton_temporal_diagnosis[_inputs]` selectors share the same workload
while accepting only zero or unavailable `UINT64_MAX` info-queue counts.
Device-removal checks remain required, alongside Vulkan core/synchronization
activation without errors in every GPU child. Numerical criteria are unchanged.

[Quality evidence](2026-10-13-exp-proton-temporal-regression/quality-results.json),
[long evidence](2026-10-13-exp-proton-temporal-regression/long-results.json),
[reset evidence](2026-10-13-exp-proton-temporal-regression/reset-results.json) and
[jitter evidence](2026-10-13-exp-proton-temporal-regression/jitter-results.json)
retain commands, selected source/component hashes, identities, outcomes and
compact measurements. Exact repeated measurement fields are stored once and
referenced from later blocks through `identical_measurements`; each block's
commands and execution outcomes remain separate. Long/reset evidence retains
the criterion-relevant values rather than duplicate diagnostic metrics. These
are not complete verification exports. Signature observations retain the method,
exit, error lines and a hash of the full normalized diagnostic text; redundant
certificate dumps remain local. A later
sanitizer fix, before reset/jitter execution, normalizes Wine DOS paths backed by
Linux directories; it changes evidence formatting, not native execution or image
criteria. Source/external inputs stayed unchanged within all four attempts.
No public API, crate dependency, shader generator or supported target changed.

## Equipment and trust

| Input | Observed configuration |
|---|---|
| Host | SteamOS 3.9.2, build 20260925.101; x86_64; kernel 7.2.7-valve1-1-neptune-72-gc8730d37f9c6 |
| GPU | AMD Custom GPU 0405 / RADV VANGOGH; vendor `0x1002`, device `0x163f`; adapter 0, DX12 feature level 12_0 |
| Driver | Mesa 26.2.0-devel, git-6c05e09fac; Vulkan 1.4.350, driver 26.1.99 |
| Provider | `3.1.5`, ID `0xf5a5ca1e00c01005`, in queries and matrix contexts |
| Rust/Cargo | 1.99.0; rustc b940084d7eb6a299eb4bfeb8e34901bc051e7ac4; Windows/MSVC cross target |
| Source crate versions | `fsr-sdk 0.3.0`, `fsr-sdk-sys 0.1.2`; unreleased checkout |
| Coordinator | Python 3.14.7; existing NumPy 2.5.2 from the locked verification shell |
| Cross tools | cargo-xwin 0.23.1; Clang/LLD 21.1.8; prepared Microsoft SDK 10.0.22621 and CRT component 14.44.17.14 |
| Proton/launcher | Installed Proton 10.0-4b (`1785139158 proton-10.0-4b`); VKD3D-Proton 3.0.0; unwrapped umu 1.4.4 / steamrt3 |
| Vulkan layer | Khronos validation 1.4.357.0; core and synchronization required; GPU-assisted validation disabled |
| AMD | D017 pinned SDK v2.3.0; all eight installation files verified |

The loader SHA-256 was
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`;
the provider SHA-256 was
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Linux execution explicitly trusts these pinned bytes. Optional osslsigncode
inspection does not establish Windows Authenticode policy; its certificate-chain
failures remain observations. DirectX debug/GBV coverage is unavailable here;
missing error counts remain `18446744073709551615`, never zero errors.

Tools, SDK, Microsoft cache, Proton and launcher runtime were already prepared.
No new dependency or bootstrap was introduced. Ordinary Cargo work stayed
locked/offline. Launcher network acknowledgement remains required; this is not
proof of offline launcher execution or complete runtime pinning.

## Reproduction

From the repository root, using the existing locked shell and prepared local
inputs, run each profile separately. Replace the profile with `long`, `reset`
or `jitter` for the other protocols:

```sh
nix develop path:./tools/verification/nix
python3 tools/verification/run.py plan proton-temporal-quality
python3 tools/verification/run.py run proton-temporal-quality \
  --native --allow-launcher-network --toolchain stable \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 10.0" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

The actual commands used the four profile names below and the installed home
path represented above by `$HOME`. Each attempt independently rebuilt/staged the
C++ helper and exact Rust executable, repeated both public-query families and
single-frame control, then ran the matrix's input-only oracle and three fresh
process blocks. GPU access required execution outside the agent sandbox.
`--allow-launcher-network` and `--online` remain separate permissions.

| Profile | Attempt suffix | Matrix frames | Input-only frames | Exit/outcome |
|---|---|---:|---:|---|
| `proton-temporal-quality` | `213757Z-…-18b628a1929c` | 576 × 3 | 45 | 1; image audit failed |
| `proton-temporal-long` | `214753Z-…-02f977d093fa` | 512 × 3 | 34 | 1; image audit failed |
| `proton-temporal-reset` | `220347Z-…-f88f658f33b8` | 320 × 3 | 25 | 0; all 18 cases passed |
| `proton-temporal-jitter` | `221247Z-…-a44cf99c6b36` | 1,536 × 3 | 108 | 1; first launcher failed, full image audit blocked |

All suffixes belong to UTC `20261013T` attempts; complete names and timestamps
are in the evidence. Integrity checks on all four finalized attempts passed,
including the failed outcomes. Combined inventory: 68 required cases, 64 passed,
three failed and one blocked; this is not a combined acceptance pass.

## Observations

### Quality and history control

All three 576-frame blocks passed GPU execution, cleanup, the independent byte
oracle and finite/sentinel/device checks. Correct textured-slow output passed
all five quality budgets. Reversed motion exceeded correct foreground error by
3.3213× and temporal error by 4.7966×, satisfying the frozen 1.1× predictions.
The per-frame-reset prediction failed in all three blocks:

| Temporal error against analytic reference | Observed MAE |
|---|---:|
| Correct continuous history | 0.0007192973817121195 |
| Reset every frame | 0.0005811916656266194 |
| Reset/correct ratio | 0.807999139720526; required ≥ 1.1 |

This is a failed local sensitivity prediction, not evidence of a Vulkan error
or proof that resets universally improve visual quality. Repeated selected
outputs and measurements were identical on this configuration.

### Continuous 256-frame histories

The three 512-frame blocks (correct/reversed arms, 256 frames each) passed the
structural audits and motion sensitivity in every window. Correct background
MAE exceeded the unchanged limit `0.003138194689362351` from frames 64 onward:

| Window | Correct background MAE, each repeat |
|---|---:|
| 32–63 | 0.0025151512477614504 |
| 64–95 | 0.0034314412492463123 |
| 96–127 | 0.004052738662820886 |
| 128–159 | 0.004477010349137812 |
| 160–191 | 0.004762042568514721 |
| 192–223 | 0.004952566874157036 |
| 224–255 | 0.005085796385769922 |

Foreground, edge, temporal and disocclusion budgets passed in all 21 windows.
The background failure is retained separately in all three blocks; no whole-run
average replaces the window criteria.

### Cut/reset recovery

All frozen recovery, history sensitivity and image budgets passed in all three
blocks. Across each block's 64 paired post-cut frames, the maximum reset/fresh
RGBA16F difference was exactly zero (allowed maximum `0.000001`). Early reset
MAE was zero; early no-reset/fresh MAE was `0.0021661568853853885`, above the
required `0.0001` difference. All 960 matrix frames had complete byte audits.

### Selected jitter and launcher failure

The selection contains stationary phase 0 and moving phase 0, all six dispatch
controls and 128 frames per arm. Stationary starts 8/16/24 are excluded to bound
temporary disk use. Windows group criteria and three acceptance windows per
group are unchanged.

All three processes wrote 1,536 frames and reported a passing Rust test. Vulkan
core/synchronization activation was observed with zero errors/warnings. The
first umu invocation nevertheless returned **247** after 99.144 seconds; no
watchdog expired (limit 360 seconds). Its Rust log reported one passed test and
ended with a Wine service error 1726. The runner retained this launcher failure;
Rust's success marker alone is insufficient. Kernel-journal inspection for OOM
or killed processes in the bounded interval returned no entries; the cause is
unresolved and is not inferred from the numeric exit alone.

The second and third launcher invocations returned zero and passed the complete
input/output audits. The required full jitter image audit was **blocked** by the
first execution failure. No jitter image-criterion verdict is claimed.

Before default compaction, an additional read-only
[oracle probe](2026-10-13-exp-proton-temporal-regression/audit_failed_jitter_inputs.py)
validated the rejected first block's 1,536 frames and 1,415,583,744 complete input
bytes. Its digest was
`64cf9f37a91946a3d0d8d28768fc2a76635c4af472b2a43d9c51e502ee90cae6`.
That supplemental result is stored separately in the jitter evidence and does
not change the failed launcher outcome or unblock three-process acceptance.
The exact diagnostic command used the same prepared shell:

```sh
nix develop path:./tools/verification/nix --command python3 \
  docs/research/records/2026-10-13-exp-proton-temporal-regression/audit_failed_jitter_inputs.py \
  <uncompacted-jitter-attempt> <new-output-outside-attempt.json>
```

The diagnostic originally ran from a temporary copy under `target/`. The
retained copy additionally enforces a fresh output outside the original attempt.
It performs no GPU launch and requires retained, unmodified diagnostic payloads.

## Checks and limits

All four attempts re-established public queries and single-frame controls.
Every selected GPU/input-only child observed Vulkan core/synchronization
validation, with zero reported errors or warnings. Across matrix datasets,
metadata reports zero device removal, completed fences, successful finish and
230,400 finite/written pixels per native output. All 8,832 matrix frames passed
independent input/output oracles, including the separately audited rejected
jitter block; this does not substitute for execution or image acceptance.
All 212 input-only frames were independently checked before their matrices.

The final implementation passed:

```sh
nix develop path:./tools/verification/nix --command python3 -m unittest discover -s tools/verification/tests -q
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo +stable clippy -p fsr-sdk --lib --tests --features dx12,movable --target x86_64-pc-windows-msvc --locked --offline -- -D warnings
```

Infrastructure: 136 tests, no skips. Host Cargo: 38 tests and two documentation
examples; this is separate from GPU evidence. Windows-target Clippy compiled
the affected tests but does not establish native Windows execution. No native
Windows or second GPU run was performed here.

The original Windows confirmations used RX 9060 XT/provider 4.1.1; this run uses
Deck/provider 3.1.5. **Unresolved:** provider, hardware, driver and translation
all differ, so neither failed prediction is isolated to Proton or the Rust
wrapper. Repeating the same declared protocols on a provider-4.1.1 Proton host
is a useful next comparison; budgets must not be retrofitted to these outputs.
Jitter additionally needs a clean three-process execution and investigation of
the launcher status. Correlated frames and repeated processes on one device are
not population confidence intervals or universal image-quality evidence.

Full payloads/logs/binaries stayed under ignored `target/`. Default compaction
removed the generated image/input bytes after successful and failed finalized
attempts; permanent evidence keeps compact measurements, frame totals, hashes
and outcomes. Full offline image reanalysis now requires a new GPU run with
`--retain-diagnostics`; hashes cannot reconstruct the old outputs. Paths and
transient pointers in retained evidence are normalized; hardware/provider IDs,
return codes and hashes are preserved. V08 and V09 remain unexecuted.
