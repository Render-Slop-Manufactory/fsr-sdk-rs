# RC1 Steam Machine verification — 2026-10-29

**Passed: 12/12 required cases** against the exact shared `0.3.0-rc1` archive
pair, 17:00:27–17:00:57 UTC (18:00:27–18:00:57 Europe/Paris). Both public query
ownership families, the Linux-hosted Windows Rust/C++ build, fenced single-frame
GPU dispatch/readback, explicit destruction and required Vulkan core/synchronization
validation pass. The final audit confirms unchanged archives, extracted contents
and recorded inputs. Independent `verify-run` and evidence export both return 0.

The [compact evidence](2026-10-29-rc1-steam-machine-verification.json) retains
case outcomes, package and executable hashes, equipment, SDK identities,
measurements, diagnostic limits and evidence inventory digests. This report
qualifies the bounded Steam Machine path for these bytes. Overall release
acceptance remains dependent on the other host reports and final consolidation.

## Exact candidate inputs

The run explicitly selected both checked-in archives from
`release-candidates/0.3.0/rc1/`; it did not regenerate packages on this host.
Both hashes and sizes match the shared candidate manifest.

| Archive | Bytes | SHA-256 |
|---|---:|---|
| `fsr-sdk-0.3.0.crate` | 318,967 | `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766` |
| `fsr-sdk-sys-0.1.2.crate` | 43,576 | `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042` |

The candidate identifies clean packaging source
`6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`. This verification ran from clean
distribution checkout `1b1c7a3fc1fa3877972aa1111eb79e7916c3732c`. The registry
audit validates all 171 wrapper and 33 raw archive members against the checkout
and builds a checksum-verified isolated registry. Rust tests and the C++ helper
use the verified extracted sources. Cargo builds remain locked/offline.

The earlier Machine check used a different archive pair. Its passing outcome
does not establish RC1 byte identity; this fresh exact-pair run supplies that
qualification. No previous report or project status document was changed.

## Equipment

SteamOS 3.9.5, build `20261023.100`, kernel
`7.2.7-valve2-2-neptune-72-g31504582a073`; adapter 0, AMD Steam Machine
(RADV NAVI33), vendor/device `1002:7481`, DX12 feature level 12_0. Host Vulkan
reports API 1.4.350, driver version 26.1.99 and Mesa
`26.2.0-devel (git-5c65d4fc31)`.

The selected Proton 11.0 distribution reports
`1791559474 proton-11.0-2e-x86_64`. Observed components are VKD3D-Proton 3.0.0
build `212991fc2c266bc`, DXVK `v2.7.1-498-ga6764047e587178`, umu 1.4.4,
steamrt4 `4.0.20260928.262390` and pressure-vessel `0.20260925.0`.
The prefix translator hashes match the selected distribution.

The locked Nix verification shell supplies Python 3.14.7, cargo-xwin 0.23.1,
Clang/LLD 21.1.8, unwrapped umu and Khronos validation layers 1.4.357.0.
Prepared Rust/Cargo 1.99.0 cross-compiles for `x86_64-pc-windows-msvc` using
the existing Microsoft SDK/CRT cache. AMD SDK v2.3.0 uses the local pinned D017
deployment. No dependency or verification infrastructure was changed.

## Results

| Check | Observation |
|---|---|
| Toolchain, equipment, selected packages, isolated registry and binding | All passed |
| SDK/module identity | Pinned loader/upscaler hashes pass; both GPU children load the staged modules |
| C++ helper, Rust executable and Proton launch | All passed |
| Local and movable public queries | Provider 4.1.1, ID `0xf5a5ca1e01001001`; each reports 80,740,352 bytes estimated/actual GPU memory |
| Single-frame dispatch/readback | 640 × 360; all 230,400 pixels finite and written; gradient checks pass |
| Red readback metrics | Minimum 0.10058594; maximum 0.84765625; left 0.193594; right 0.7556911 |
| Device and teardown | Device-removal code 0; explicit destruction native code 0 |
| Vulkan validation | Core and synchronization active on all three observed instances in each GPU child; 0 errors, 0 warnings |
| Process containment | All four launcher invocations observe distinct PID namespaces and `--die-with-parent`; all exit 0 |
| Final integrity | Selected archives and extracted bytes unchanged; `inputs_unchanged=true` |

Windows DirectX debug-layer creation returns `0x887a002d`; the info queue is
unavailable. Its `UINT64_MAX` error count means unavailable, not zero errors.
D3D12 GBV and Vulkan GPU-assisted validation were not enabled. Both Linux
`osslsigncode` observations exit 1 because the local issuer certificate is
unavailable; they do not establish Windows Authenticode-policy success. Execution
uses the independently verified D017 DLL hash pins recorded in the JSON.

Nonfatal launcher diagnostics include unavailable Nix ldconfig cache, a missing
32-bit MangoHud dependency, a dconf symbol mismatch and Wine/OpenVR/OpenXR
initialization messages. Required process outcomes, GPU markers, staged-module
identity and Vulkan validation pass. Launcher network access was acknowledged;
updates were disabled, but this profile does not isolate networking.

## Reproduction and retained evidence

From the repository root, enter `nix develop path:./tools/verification/nix`, then:

```sh
export RUSTUP_HOME="$PWD/target/rustup"
export CARGO_HOME="$PWD/target/cargo"
export RUSTUP_TOOLCHAIN=1.99.0
python3 tools/verification/run.py run proton --artifacts --require-passed \
  --native --allow-launcher-network --toolchain 1.99.0 \
  --output-root target/steam-machine/rc1-2026-10-29/runs \
  --wrapper-archive release-candidates/0.3.0/rc1/fsr-sdk-0.3.0.crate \
  --sys-archive release-candidates/0.3.0/rc1/fsr-sdk-sys-0.1.2.crate \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --adapter-index 0 \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 11.0" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

The attempt is `20261029T170025Z-proton-0273c5f1a932` under the output root.
`verify-run --run <attempt-directory>` returns 0;
`export --run <attempt-directory> --output target/steam-machine/rc1-2026-10-29/evidence`
also returns 0. Full logs, extracted sources, binaries and the export remain
under ignored `target/`. The compact JSON removes personal Wine stage paths
while preserving independently checked stage-relative module identities.

The first Nix shell entry was blocked by its cache lock being outside the
restricted writable directories. Retrying with the required access succeeded;
there was one verification attempt, with no failed test case or modified input.

## Scope limits

This is bounded Linux-hosted Windows/DX12 execution through Proton. It does not
establish native Linux/Vulkan target support, complete Windows ABI/advanced-feature
or MSRV coverage, temporal/image-quality coverage, offline-runtime execution or
forced-timeout cleanup. Those profiles were not run here.

No new performance measurement was selected or performed. This report does not
reconcile P08 performance inputs or certify cross-host performance; release
consolidation retains that responsibility. Publication simulation and overall
release acceptance remain separate. Nothing was published or pushed.
