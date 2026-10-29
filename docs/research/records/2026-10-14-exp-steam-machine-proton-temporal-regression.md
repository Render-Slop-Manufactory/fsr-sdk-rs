# Steam Machine Proton temporal regression validation

**Privacy correction, 2026-10-29:** the companion JSON is now a sanitized
derivative with workspace aliases. Measurements and verdicts are unchanged;
[correction scope and original/sanitized byte identities](../../release/2026-10-29-evidence-privacy-correction.md)
are recorded separately.

Validation date: 2026-10-14, Europe/Paris. Question: do the smoke and four selected Proton temporal profiles pass on this Steam Machine under the existing frozen criteria?

Overall temporal validation failed. Completed the smoke profile and all four selected temporal profiles with unchanged criteria. Of 76 required checks, 73 passed and three image audits failed; none were blocked.

## Equipment and execution

Attempts ran on 2026-10-13 from 23:32 to 23:58 UTC (14 October, 01:32–01:58 in Europe/Paris). Exact timestamps and component hashes are retained in the compact evidence. Exact repeated JSON values use documented `shared_value` references; expanding them restores the selected measurements and commands.

SteamOS 3.9.2; AMD Steam Machine / RADV NAVI33, vendor 0x1002, device 0x7481, adapter 0; installed Proton 11.0 (`1791559474 proton-11.0-2e-x86_64`); AMD SDK v2.3.0 and FSR provider 4.1.1. Used the locked verification Nix shell, prepared Microsoft cross toolchain, Rust 1.99.0, NumPy 2.5.2, and unwrapped umu 1.4.4. Cargo stayed locked/offline; launcher network access was acknowledged.

| Profile | Outcome | Detail |
|---|---|---|
| `proton` | PASSED | All eight checks passed. |
| `proton-temporal-quality` | FAILED | Foreground motion sensitivity failed in all three repetitions: reverse/correct error ratio 0.71519; required ≥1.1. All normal-output image budgets passed. |
| `proton-temporal-long` | FAILED | Foreground motion sensitivity failed in all 21 windows: ratios 0.68223–0.73953; required ≥1.1. All normal-output image budgets passed. |
| `proton-temporal-reset` | PASSED | All 18 checks passed, including frozen recovery and ghosting criteria. |
| `proton-temporal-jitter` | FAILED | Moving foreground alignment exceeded limits in seven of nine windows. Stationary alignment, normal-output quality budgets and all wrong-control sensitivity checks passed. All three repetitions completed. |

## Verification boundaries

All 26 validation-enabled children reported active Vulkan core and synchronization validation, with zero errors and zero warnings. All five attempts passed evidence integrity verification and retained unchanged source/external input identities. The three failures are image-criteria failures, not launcher failures. Repository source was not changed during execution. This report and its documentation links were added afterwards.

Windows D3D12 debug-layer/GBV coverage remains unavailable. These are bounded Proton observations on this GPU/provider, not native Linux support or complete native/temporal coverage. Jitter uses the declared phase-0 selection. Default compaction removed full temporal diagnostic images after analysis; compact measurements and commands are retained alongside this report; full local logs and payload inventories remain under ignored `target/`. Full image reanalysis requires a new GPU run.

[Compact commands, input identities, outcomes and measurements](2026-10-14-exp-steam-machine-proton-temporal-regression/results.json)

The selected profiles and criteria are described in the [verification guide](../../../tools/verification/README.md#selected-temporal-regression). The [earlier Deck run](2026-10-13-exp-proton-temporal-regression.md) remains unchanged. V07d acceptance remains open.

## Baseline delta and remaining questions

The existing Deck record covered provider 3.1.5: reset passed, quality/history and long-background criteria failed, and a launcher failure blocked the full jitter verdict. This run adds Steam Machine/provider 4.1.1 results with the same numerical criteria. Reset again passes; normal-output quality and long-history budgets pass, but foreground motion sensitivity fails. Jitter completes all three processes and produces a failed moving-alignment verdict. These observations do not identify a cause or justify changing the criteria. Resolving the failed criteria and broader host comparison remain separate work; no new research or synthesis was performed while assembling this record.

## Reproduction

From the repository root, enter the locked shell, use the existing prepared caches and run each profile from the table:

```sh
nix develop path:./tools/verification/nix
export RUSTUP_HOME="$PWD/target/rustup"
export CARGO_HOME="$PWD/target/cargo"
export RUSTUP_TOOLCHAIN=1.99.0
python3 tools/verification/run.py run proton-temporal-quality \
  --native --allow-launcher-network --toolchain 1.99.0 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --adapter-index 0 \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 11.0" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```
