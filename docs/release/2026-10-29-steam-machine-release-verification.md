# Steam Machine package verification — 2026-10-29

**Passed: 12/12 required cases** for the bounded current-package Proton check,
02:46:25–02:46:56 UTC. Linux-hosted Windows Rust/C++ builds, public queries for
both ownership families, fenced single-frame GPU readback, explicit destruction,
Vulkan core/synchronization diagnostics and the final package audit pass.
The [compact evidence](2026-10-29-steam-machine-release-verification.json) retains
package/binary identities, equipment, metrics, case results and limitations.

## Package identity and scope

This host generated the following archives from its clean checkout and built
the Windows test executable and C++ helper from their extracted contents. Cargo
used a checksum-verified isolated local registry, with locked/offline builds.

| Archive | SHA-256 |
|---|---|
| `fsr-sdk-0.3.0.crate` | `56ca4a23e897063e7a0c975d6233919fa39f1db6e498c9c2b6977a709eefbdb5` |
| `fsr-sdk-sys-0.1.2.crate` | `af98d189914fbab70360056a40cdb1b044bae52b5757695d7b9fb8fade1f22d5` |

These hashes differ from the earlier Windows
[package-coverage pair](2026-10-29-package-coverage-verification.md). No archives
from the concurrently running Windows/Deck release checks were available here;
their byte equality is not established. This qualifies the pair above on this
Machine configuration. Final candidate freeze, common-pair reconciliation and
the other hosts' results remain separate release requirements.

Against the 187 crate-file hashes in the
[P08 closure](2026-10-29-performance-review-closure.json), 170 match byte for byte.
The other 17 match exactly after converting this checkout's LF to the baseline's
CRLF; there are no further content differences in that inventory. This is a
line-ending assessment, not byte identity with the measured archives. No runtime
code was edited and no new performance measurement was selected by this check.

## Equipment and reproduction

The host is SteamOS 3.9.5, build `20261023.100`, kernel
`7.2.7-valve2-2-neptune-72-g31504582a073`. Adapter 0 is AMD Steam Machine
(RADV NAVI33), vendor/device `1002:7481`; Vulkan API 1.4.350, driver version
26.1.99, Mesa `26.2.0-devel (git-5c65d4fc31)`.

The installed distribution reports `1791559474 proton-11.0-2e-x86_64`.
Observed components are VKD3D-Proton 3.0.0 build `212991fc2c266bc`, DXVK
`v2.7.1-498-ga6764047e587178`, umu 1.4.4 and steamrt4
`4.0.20260928.262390` with pressure-vessel `0.20260925.0`. Translator hashes are
retained in the JSON. Updates were disabled; launcher network access was
acknowledged and not isolated.

The locked [Nix environment](../../tools/verification/nix/README.md) supplies
Python 3.14.7, cargo-xwin 0.23.1, Clang/LLD 21.1.8, unwrapped umu and Khronos
validation layers 1.4.357.0. Rust/Cargo 1.99.0 and the Windows MSVC target were
already installed in `target/rustup`; the prepared Microsoft SDK/CRT cache is
`target/proton-toolchain`. AMD SDK v2.3.0 is the locally prepared D017 deployment.
The Nix environment fetched missing declared tools before the run; no tool or SDK
dependency was added to the repository.

From the repository root, inside `nix develop path:./tools/verification/nix`:

```sh
export RUSTUP_HOME="$PWD/target/rustup"
export CARGO_HOME="$PWD/target/cargo"
export RUSTUP_TOOLCHAIN=1.99.0
python3 tools/verification/run.py run proton --artifacts --require-passed \
  --native --allow-launcher-network --toolchain 1.99.0 \
  --output-root target/steam-machine/release-2026-10-29/runs \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --adapter-index 0 \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 11.0" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

This generates a pair; to recheck the recorded bytes, additionally supply both
`--wrapper-archive` and `--sys-archive` from the original attempt's
`package-build/package/` directory. Regeneration after later package changes
does not reproduce the recorded archive identity.

The local attempt is `20261029T024624Z-proton-0eecfa17b853` beneath the output
root above. `verify-run --run <attempt>` returned 0, and
`export --run <attempt> --output <new-directory>` completed successfully. Full
logs, binaries, extracted packages and the export remain under ignored `target/`;
the retained JSON makes the outcome understandable without those local files.

## Results and diagnostic boundaries

| Check | Observation |
|---|---|
| Toolchain, equipment, package creation, registry and binding | All passed |
| Pinned SDK and observed AMD module paths | Both DLL hashes verified; both GPU children load the staged files |
| Packaged Rust/C++ builds and Proton launch | Passed; translator bytes match the selected Proton distribution |
| Public queries, local/movable | Provider 4.1.1, ID `0xf5a5ca1e01001001`; each reports 80,740,352 bytes estimated/actual GPU memory |
| Single-frame dispatch/readback | Provider 4.1.1; 640 × 360, all 230,400 pixels finite and written; gradient checks pass |
| Readback red metrics | Minimum 0.10058594, maximum 0.84765625, left 0.193594, right 0.7556911 |
| GPU completion and cleanup | Device removal 0; explicit destruction succeeds with native code 0 |
| Vulkan validation in each GPU child | Core and synchronization active on all three observed instances; 0 errors, 0 warnings |
| Process containment | Distinct PID namespace and `--die-with-parent` observed for all four launcher invocations |
| Final archives/extracted contents | Unchanged; `inputs_unchanged=true`; evidence integrity passes |

Windows DirectX debug-layer creation returns `0x887a002d`; the info queue is
unavailable. Its error count remains `UINT64_MAX`, not zero. D3D12 GBV and Vulkan
GPU-assisted validation were not enabled. Linux `osslsigncode` observations both
exit 1 because the local issuer certificate is unavailable; this does not pass
Windows Authenticode policy. Execution trusts the pinned D017 DLL bytes.

Nonfatal launcher messages include a missing 32-bit MangoHud dependency, an
unavailable Nix ldconfig cache, a dconf symbol mismatch, and fresh-prefix AMD DLL
copy/game-drive warnings. All four launcher invocations exit 0, both GPU children
emit their required success markers, and the staged AMD module paths pass the
independent check. These messages are not Vulkan validation errors.

This bounded run does not execute the complete Windows release/ABI/advanced
feature matrix, MSRV suite, deferred temporal/image-quality work, offline-runtime
profile or forced-timeout cleanup profile. It adds current packaged Proton
evidence for this host, without adding native Linux/Vulkan support. Nothing was
published, committed or pushed.
