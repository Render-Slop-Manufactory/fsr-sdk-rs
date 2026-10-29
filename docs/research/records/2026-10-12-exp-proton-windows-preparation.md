# Experiment: Windows preparation for a preliminary Proton runtime spike

- Date: 2026-10-12.
- Question: Can the first Proton execution probe be prepared and its host-side
  integrity/failure controls checked on Windows, with Linux execution deferred?
- Scope: prepare exact previously verified Windows binaries and a small portable
  entry point for the existing public query test followed by unchanged Gate A.
- Success: source/build identity remains checked, the complete handoff bundle
  passes standalone inspection, corruption/stale-source controls reject inputs,
  and Windows execution or output replacement is rejected without launching AMD.
- Exclusions: Linux cross-compilation, Proton launch, new GPU execution,
  equivalent Linux signature policy, full Proton/runtime provenance and V07 completion.

## Setup and baseline

Host: Windows 11 build 10.0.26300, Python 3.14.1. The input executable/helper were
produced by the previously passed native attempt
`20261012T213634Z-native-upscaler-1e95c356ba8e`, with Rust/Cargo 1.99.0,
MSVC toolset 14.44.35207 and Windows SDK 10.0.22621.0. SDK v2.3.0 is pinned to
commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.

That earlier attempt's 22 passed cases on the AMD Radeon RX 9060 XT, driver
32.0.31041.1004, are retained in the
[Windows M10/SDK record](../../release/2026-10-12-windows-m10-sdk-verification.md).
They establish the Windows build/control baseline, not new Proton execution.
The source attempt's input identity is
`5c0280584460c18f5cc05184f05bcde88fdfbdcc9ddd175fc5ea8ed2c540b929`.

The repository starting revision was `0e5bde16eded14eedef2d2a710998efa88248e44`.
Existing uncommitted work consisted of the Windows verification reports/evidence
and linked README, roadmap, development and D017/D018 documentation, plus D019
proposal refinements. No crate, manifest or GPU-helper source was changed during
this spike. Preparation compares every current crate/manifests input with the
retained build hashes, including rejection of added source files.

Probe additions are [proton_spike.py](../../../tools/verification/proton_spike.py),
[its host controls](../../../tools/verification/tests/test_proton_spike.py) and
[its handoff protocol](../../../tools/verification/proton-spike.md).
The verification README and still-proposed D019 link the preliminary split.
The script is excluded from current profile composition: it reuses `Runner`,
watchdogs and record generation without registering a supported `proton` profile.

## Reproduction and actual results

From the repository root, using the completed native attempt and D017 installation:

```powershell
python tools/verification/proton_spike.py prepare --run target/verification/20261012T213634Z-native-upscaler-1e95c356ba8e --sdk-root target/sdk-windows-verification/v2.3.0 --output target/proton-spike/deck-handoff
python tools/verification/proton_spike.py inspect --bundle target/proton-spike/deck-handoff
python -m unittest discover -s tools/verification/tests -p test_proton_spike.py -v
python -m unittest discover -s tools/verification/tests -v
```

The initial temporary handoff was `target/proton-spike/windows-bundle-20261012`.
An intermediate bundle was `target/proton-spike/deck-bundle`. Final review added
rejection of result/prefix locations overlapping the immutable bundle, so repeat
execution cannot invalidate its inventory. The final fresh destination is
`target/proton-spike/deck-handoff`; earlier bundles were not overwritten.

| Control | Observation |
|---|---|
| Final Windows preparation | Exit 0; complete native attempt integrity, current crate/manifests, D017 installation and matching AMD payload identities passed |
| Repository-side inspection | Exit 0; complete handoff file inventory matches |
| Standalone copied script, from inside the final bundle | Exit 0 for `python tools/verification/proton_spike.py inspect --bundle .`; copied modules work without the original repository imports |
| Focused controls outside the sandbox | Five tests passed; no Proton/AMD execution |
| Full infrastructure suite outside the sandbox | 120 tests: 91 passed, 29 skipped, no failures |
| Run command on Windows with explicit native/network flags | Exit 1: `Execution requires Linux; Windows supports prepare/inspect only`; no child launched |
| Repeated prepare into the intermediate existing destination | Exit 1: destination must be new/separate; existing bundle preserved |
| Local final ZIP | 13 files, 17,949,036 bytes; ZIP CRC check passed |

The Windows rejection command supplied placeholder paths deliberately:

```powershell
python tools/verification/proton_spike.py run --bundle target/proton-spike/deck-bundle --umu absent --umu-version unselected --proton absent --proton-version unselected --native --allow-launcher-network
```

The final private ZIP is `target/proton-spike/deck-handoff.zip`, SHA-256
`58adec73a31a0cd28c86b04a1b12438f59869a0ec0dc636301eb17590d4c6dde`.
The [compact bundle manifest](2026-10-12-exp-proton-windows-preparation/bundle.json)
retains exact per-file identities, Windows build provenance identity, selectors
and required markers without retaining or publishing AMD payloads. The executable
SHA-256 is `8ee627b498e6579953941d93211fb93438be2f75c028506d66f5968212187543`.

The five controls cover changed/missing/extra bundle payloads, stale crate content
and inventories/manifests, schema rejection, Windows execution rejection and
required explicit native/network opt-ins and overlapping result/prefix rejection.
The 29 full-suite skips are the same
28 optional NumPy-based temporal oracles and one Windows-skipped portable symlink
control documented in the earlier Windows record. No new symlink/temporal result
is inferred from those skips.

The first focused test run inside the sandbox failed with temporary-file and
cleanup `PermissionError`/`WinError 5`. The permitted outside-sandbox focused and
full runs passed. The failed sandbox result remains failed; no source fix was
needed for it.

## Prepared Linux protocol and unresolved execution

The portable bundle contains the exact test executable, GPU helper and two AMD
DLLs beside the executable, full project/AMD notices, sanitized Windows provenance,
three project scripts and a file manifest. The intended Linux run creates a fresh
attempt and a separate fresh prefix, records Vulkan/GPU identity, then executes
the exact public-query selector followed by unchanged Gate A with 180-second
watchdogs. Exit 0 and the selected case/pass-count markers are required.

This initial case intentionally preserves the Windows debug-layer requirement.
No helper assertion is disabled. Missing diagnostics, unavailable Vulkan identity,
failed queries or a failed dispatch do not become a GPU pass. Actual working-
directory/DLL-path handling, Proton service cleanup and launcher exit propagation
remain unexecuted and therefore unverified on Linux.

Upstream source inspected on 2026-10-12:

- [umu manual](https://github.com/Open-Wine-Components/umu-launcher/blob/main/docs/umu.1.scd):
  explicit local Proton/prefix configuration, argument forwarding and disabled
  runtime updates are documented.
- [umu runtime source](https://github.com/Open-Wine-Components/umu-launcher/blob/main/umu/umu_runtime.py):
  `setup_umu` performs an upstream request before the existing-install
  `UMU_RUNTIME_UPDATE=0` early return, and handles missing runtime installation
  separately. This is source inspection of `main` at the access date, not execution
  of a pinned installed launcher.

Accordingly the prototype requires `--allow-launcher-network`; it does not claim
offline operation or bounded, pinned full-runtime acquisition. The operator must
select concrete installed tools on the Linux host. Their supplied version labels
are descriptive and need corroboration from actual launcher logs/component
identities. No Proton, umu or Linux runtime version has been selected, downloaded
or installed by this Windows preparation.

An initial `wsl --list --verbose` inside the sandbox returned access denied;
permitted enumeration identified a stopped Ubuntu WSL2 installation. A proposed
launch/inventory command was rejected and did not execute. The user then explicitly
excluded WSL and selected Windows infrastructure preparation with a later real
Linux/Steam Deck run. No WSL distribution was started and no Linux GPU probe ran.

## Baseline delta and next step

**Verified:** a private handoff can be prepared and independently inspected on
Windows from unchanged, previously verified native build inputs. Host-side
integrity and refusal controls pass. Both bundle and ZIP remain ignored local
artefacts; only this report and its compact manifest are retained in the repository.

**Unresolved:** all actual Proton/Linux behaviour. No Linux cross-build, AMD DLL
load under Proton, context, GPU completion or readback is claimed here. This is
preparation for the execution spike and does not complete V07a–V07c, accept D019
or extend platform support.

Next, copy the private bundle to the Deck, inspect its identity there and inventory
the available Vulkan/umu/Proton setup before selecting concrete local launch paths.
Run the frozen query/Gate A sequence and retain its actual pass/failure details.
The Linux cross-build is a separate remaining question even if Windows-built
binaries successfully execute there. Existing topic syntheses and roadmap status
were not updated by this experiment record.
