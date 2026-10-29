# Experiment: Vulkan validation of the Proton single-frame workload

- Date: 2026-10-13 local time (execution timestamps are 2026-10-12 UTC).
- Question: can the prepared Steam Deck path run the pinned AMD DX12 provider
  with Vulkan core and synchronization validation, without relaxing its existing
  numerical, device-removal or teardown checks?
- Success requires cross-build, exact staged AMD modules, public Queries and
  single-frame GPU execution; every reported validation instance must show core
  and synchronization enabled, with zero validation errors. Missing activation,
  nonzero child exit or missing workload markers fail.
- Baseline: [initial Proton proof](2026-10-13-exp-proton-linux-cross-build.md).
  D019 remains proposed; no support-policy or public API change is accepted here.

## Setup and method

Source baseline `3cf429c3f0af832058e752437f50dc724caded69`, plus the local
explicit validation option, host `vulkaninfo` selection, locked Nix shell and
tests. The only Rust workload edit changes its diagnostic sentence to identify
Vulkan diagnostics as launcher-owned. GPU inputs and thresholds are unchanged.
[Compact results](2026-10-13-exp-proton-vulkan-validation/results.json) preserve
all three attempts, final source/component hashes and observed metrics. The
passing attempt's frozen input identity is
`048e4abd584cdda9c9cc136245ef1675040cfb9de39bfbdcf2e32cd90fd0638b`.

The x86_64 Steam Deck runs SteamOS 3.9.2, build 20260925.101, kernel
7.2.7-valve1-1-neptune-72-gc8730d37f9c6. GPU: AMD Custom GPU 0405 / RADV VANGOGH,
vendor 0x1002, device 0x163f; Mesa 26.2.0-devel git-6c05e09fac, Vulkan 1.4.350.
The Nix Python process observes Nix glibc 2.42; the host supplies the driver.
Proton version is `1785139158 proton-10.0-4b`, with VKD3D-Proton 3.0.0 and
DXVK 2.6.2-23-g3cb664e1260926e. The prepared umu runtime is steamrt3/sniper,
depot 3.0.20260928.262393; launcher 1.4.4.

The shell pins nixpkgs `7a0f122f5090cf4c2ade2a13a0e229d4e19ba71f`. It supplies
cargo-xwin 0.23.1, Clang/LLD 21.1.8, Python 3.14.7, NumPy 2.5.2, Pillow 12.3.0,
PowerShell 7.6.6, rustup 1.29.1 and Khronos validation layers 1.4.357.0.
The existing stable compiler is Rust/Cargo 1.99.0; Rust 1.88.0 remains installed
separately. Cross inputs are the prepared MS SDK 10.0.22621 / CRT component
14.44.17.14 cache and D017's eight-file AMD v2.3.0 installation. No SDK payload
is distributed by the flake. Selected provider: 3.1.5 / `0xf5a5ca1e00c01005`.

The runner strips inherited validation settings, enables core and synchronization,
includes submit-time synchronization checks, enables info/warning/error reports,
disables duplicate-message limiting and sets no message-ID filter. GPU-assisted
validation remains disabled. Diagnostics go to stdout, captured across the entire
child lifetime; opening one shared log separately from each Vulkan instance could
overwrite earlier diagnostics. The combined log is retained per child.

Source guidance accessed 2026-10-13:
[VKD3D-Proton debugging controls](https://github.com/HansKristian-Work/vkd3d-proton#environment-variables),
[Khronos settings](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/main/layers/vk_layer_settings.txt)
and [layer configuration](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/main/docs/khronos_validation_layer.md).
Runtime observations below, rather than setting an environment variable alone,
establish activation on this machine.

## Reproduction

Use the [locked shell and explicit bootstrap](../../../tools/verification/nix/README.md).
From the repository root, with the recorded SDK/cache and an installed Proton:

```sh
nix develop path:./tools/verification/nix
python3 tools/verification/run.py run proton --native --allow-launcher-network \
  --toolchain stable --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --proton "/path/to/Proton 10.0" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

The actual Proton argument selected the installed Steam `steamapps/common/Proton
10.0` directory; the personal directory prefix is replaced above. The first
attempt omitted `--vulkaninfo`, otherwise using these inputs. Raw attempts remain
under ignored `target/verification`; the repository retains compact observations,
not verbose loader output or runtime pointers. A new machine must prepare the
same explicit external inputs and provide a working host driver.

## Observations

| Attempt suffix | Exit | Observation |
|---|---:|---|
| `232251Z-proton-457a9ad6bcb8` | 1 | Nix `vulkaninfo` could not resolve host driver libdrm/libz dependencies. Equipment failed; AMD/GPU cases were blocked. |
| `232326Z-proton-1ab85c72521e` | 1 | Host `vulkaninfo`, both cross-builds and launch passed. Queries completed with exit 0, but the runner expected an older activation string and rejected the actual `CURRENT-VALIDATION-ENABLED` report. Dispatch was blocked. This failed attempt stays failed. |
| `232603Z-proton-b60aad251f28` | 0 | All eight required cases passed; inputs remained unchanged. Integrity verification also passed. |

The final child loader logs show the selected Nix validation library inserted at
both instance and device level. Queries and dispatch each emitted three
`CURRENT-VALIDATION-ENABLED` reports, each including Core Checks and
Synchronization. Both children completed with exit 0, zero validation errors and
zero validation warnings. No message filtering was used.

Both AMD module paths matched the staged application directory and pinned byte
hashes. The shared workload produced 640x360 output: 230400 finite pixels and
230400 changed pixels, min/max red 0.10076904/0.84765625, left/right red
0.19362283/0.7558286. Device-removal status remained 0; explicit context destroy,
resource cleanup and final device checkpoint passed.

The infrastructure suite in the flake passed 128 tests, zero skips, exit 0 in
120.400 seconds. After adapting the activation format and capture, the three
validation controls were rerun and passed; existing Proton controls were also
rechecked. These controls reject missing/partial activation, mixed instances
without synchronization, reported errors, wrong/missing layer libraries and
inherited disabling/filtering settings. Warnings are recorded without being
mislabelled as errors. A Rust 1.88.0 offline workspace/all-targets check inside
the shell also passed. These host checks do not constitute Windows MSRV GPU
execution.

## Baseline delta and limits

**Verified:** this bounded Proton workload can additionally run with the selected
Vulkan core and synchronization checks active, without reported validation
errors or changed numerical acceptance criteria. The locked development shell
provides the common tools and all ordinary infrastructure-test dependencies.

The DirectX debug layer still returns `0x887a002d`; its info queue remains
unavailable (`UINT64_MAX`, not zero errors). Vulkan validation observes translated
Vulkan usage; it does not replace D3D12 debug/GBV or prove the SDK's full native
safety contract. Vulkan GPU-assisted validation, temporal/native regression
matrices, FSR 4, other providers/effects and performance were not tested here.
Linux signature observations remain separate from Windows trust policy.

The flake does not pin rustup's stable compiler, install Proton, freeze the full
umu runtime or make runtime bootstrap offline. Launcher network contact is still
explicitly acknowledged. The same profile can be run on Steam Machine, but no
Steam Machine GPU/driver execution occurred in this experiment. Broader hardware
and runtime compatibility require their own observed runs.
