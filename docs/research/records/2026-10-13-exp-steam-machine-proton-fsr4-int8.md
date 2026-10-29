# Experiment: Steam Machine Proton GPU proof and FSR 4.1.1 INT8 shader evidence

- Investigation date: 2026-10-13, Europe/Paris; execution timestamps are UTC.
- Question: does the existing Steam Deck protocol pass on the Steam Machine,
  and is Proton Experimental required for the pinned SDK's FSR 4.1.1 INT8 path?
- Success: all eight existing Proton cases, Linux-hosted Windows Rust/C++
  cross-build, pinned staged AMD modules, both public query families, SDK
  dispatch, GPU submission/fence/readback, frozen numerical output checks,
  no device removal and explicit teardown. Vulkan core and synchronization
  activation must be observed in both GPU children, with zero errors.
- Exclusions: complete native/temporal regression, performance and image-quality
  claims, Windows diagnostic equivalence, other GPUs, native Linux/Vulkan SDK
  support and acceptance of proposed D019.

## Setup and source state

The checkout started clean at `42282d3040c177da51ea3c601a7c08ed56840a7b`.
The existing protocol is described by the
[Steam Deck proof](2026-10-13-exp-proton-linux-cross-build.md) and
[validation follow-up](2026-10-13-exp-proton-vulkan-validation.md).
No Rust API, AMD SDK pin, crate dependency, C++ GPU workload or numerical
threshold changed in this experiment.

Local infrastructure changes resolve observed AMD module paths using the
isolated prefix's DOS drive mappings; include available Proton AMD upgrade and
interface components in frozen external inputs; and remove inherited FSR4 and
shader-compiler controls. Vulkan validation is activated directly through the
explicit layer, without `VKD3D_CONFIG=vk_debug`'s deliberate invalid-call test.
All validation messages remain unfiltered and every error still rejects the run.
Tests cover the mapped-stage/unknown-drive failures, optional payload identity,
inherited controls and unfiltered layer activation. The verification README
documents these changes; its final record-link wording was updated after GPU
execution. No design or support-policy decision was accepted.

| Input | Observed configuration |
|---|---|
| Host | SteamOS 3.9.2, build 20260925.101; x86_64; kernel 7.2.7-valve1-1-neptune-72-gc8730d37f9c6 |
| GPU | AMD Steam Machine / RADV NAVI33; vendor 0x1002, device 0x7481; discrete hardware adapter index 0; DX12 feature level 12_0 |
| Driver | Mesa 26.2.0-devel, git-6c05e09fac; Vulkan 1.4.350; driver 26.1.99 |
| Rust/Cargo | 1.99.0; rustc b940084d7eb6a299eb4bfeb8e34901bc051e7ac4; Windows target x86_64-pc-windows-msvc |
| Cross build | cargo-xwin 0.23.1, Clang/LLD 21.1.8; Microsoft SDK 10.0.22621 / CRT component 14.44.17.14 |
| Orchestrator/launcher | Nix Python 3.14.7; umu-launcher-unwrapped 1.4.4 |
| Experimental | `1791480010 experimental-11.0-20261008-x86_64`; Steam app 1493710, build 25810113; VKD3D-Proton 3.1.0, build 2230755878b0199 |
| Proton 11 | `1791559474 proton-11.0-2e-x86_64`; Steam app 4628710, build 25833067; VKD3D-Proton 3.0.0, build 212991fc2c266bc |
| Translation | Both reported DXVK v2.7.1-498-ga6764047e587178 |
| Execution runtime | umu steamrt4 depot 4.0.20260928.262390; pressure-vessel/scripts 0.20260925.0 |
| Validation | Khronos layers 1.4.357.0; core and submit-time synchronization; GPU-assisted validation disabled |
| Shader inspection | Host SPIRV-Tools v2026.2-0-g0539c81f6, `spirv-dis` |
| SDK | D017 v2.3.0 eight-file installation; source commit 60f4ea81909200d8542eca14dccb2628b763a9a3 |

The Nix shell pins nixpkgs
`7a0f122f5090cf4c2ade2a13a0e229d4e19ba71f`. Rust, Microsoft inputs and the AMD
SDK were prepared explicitly under ignored `target/`; no package was added to
the user's Nix profile. The launcher acquired and integrity-checked steamrt4.
This is an observed runtime version, not a completely pinned/offline bootstrap.
Both Proton distributions were already installed through Steam. Experimental
contains `contrib/amdxcffx64.dll`; this installed Proton 11 has no contrib DLL.

AMD loader SHA-256:
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
SDK provider SHA-256:
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Experimental contrib DLL SHA-256:
`4e7dc37aebea3a90e3d3cc43e24cb2b54176b2535315f20dbe63b3b7cfc56b1e`.
These bytes and the selected interface components remained unchanged across the
runs. Linux `osslsigncode` 2.14 returned 1 for both SDK DLLs because signing-chain
verification could not obtain a local issuer certificate. Execution explicitly
trusted D017's hash pins; this is not a Windows Authenticode-policy pass.
Microsoft, AMD and Proton inputs retain their respective terms and remain local
development inputs; no binary is redistributed by this record.

## Reproduction

Use the [locked shell](../../../tools/verification/nix/README.md). From the
repository root, inside that shell, the following is the actual bootstrap with
the repository prefix normalised to `$PWD`:

```sh
export RUSTUP_HOME="$PWD/target/rustup"
export CARGO_HOME="$PWD/target/cargo"
export RUSTUP_TOOLCHAIN=1.99.0
rustup toolchain install 1.99.0 --profile minimal --component rustfmt,clippy --target x86_64-pc-windows-msvc
cargo-xwin cache xwin --xwin-cache-dir target/proton-toolchain --xwin-arch x86_64 --xwin-sdk-version 10.0.22621 --xwin-crt-version 14.44.17.14
python3 tools/verification/run.py sdk install --root target/sdk-proton/v2.3.0
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo fetch --locked
```

The second fetch supplied `cfg-if` needed during cargo-xwin's metadata work,
although the first target-filtered fetch had completed. Select the concrete
installed Proton path with `PROTON_TEST_PATH`; the actual selections were Steam's
`steamapps/common/Proton - Experimental` and `steamapps/common/Proton 11.0`.
The passing ordinary Experimental run used:

```sh
python3 tools/verification/run.py run proton --native --allow-launcher-network \
  --toolchain 1.99.0 --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --adapter-index 0 \
  --proton "$PROTON_TEST_PATH" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

The additional [provider probe](2026-10-13-exp-steam-machine-proton-fsr4-int8/provider_probe.py)
reuses every case and acceptance check. It removes `FSR4_UPGRADE` in automatic
mode, sets it to `0` in disabled mode, enables AMD interface tracing, and dumps
shaders with the translation cache disabled for the dispatch child. It requests
VKD3D trace-level logging; command-list/pipeline dispatch events were absent in
these installed distributions. It is experiment code, not another maintained
verification profile. Substitute this entry point into the same command:

```sh
python3 docs/research/records/2026-10-13-exp-steam-machine-proton-fsr4-int8/provider_probe.py \
  --repository "$PWD" --upgrade-control automatic run proton --native \
  --allow-launcher-network --toolchain 1.99.0 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo --adapter-index 0 \
  --proton "$PROTON_TEST_PATH" \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

Run automatic and disabled modes on Experimental, then automatic on Proton 11.
The original probe was executed from ignored `target/steam-machine`; the retained
copy has identical bytes. Inspect dumped shaders using `spirv-dis <shader.spv>`.
The runs require GPU access outside the agent sandbox and use a fresh isolated
prefix and application stage for each attempt. No graphical Steam configuration
or global Cargo runner is needed.

## Observations

[Compact evidence](2026-10-13-exp-steam-machine-proton-fsr4-int8/results.json)
retains source/component fingerprints, executable/helper identities, all six
attempt summaries, metrics, validation outcomes and shader measurements. Every
attempt passed the coordinator's final artefact-integrity verification, including
the failed attempts. Successful source and external inputs stayed unchanged
during execution. Personal module-path prefixes are replaced with
`<application-stage>` in retained evidence; no runtime pointers are retained.

| Attempt suffix | Exit | Result |
|---|---:|---|
| `002658Z-proton-9260588e16c7` | 1 | Windows Rust build failed offline because `cfg-if` was missing. AMD/GPU children were blocked. |
| `002820Z-proton-7abd66342756` | 1 | Build and launch passed; queries returned success and 4.1.1. Validation recorded VKD3D's intentional invalid-pNext self-test, so queries failed evaluation and dispatch remained blocked. |
| `003256Z-proton-c04d7bfeebc7` | 0 | Ordinary Experimental profile: all eight cases passed. |
| `003425Z-proton-0beeaeb54195` | 0 | Experimental automatic upgrade plus shader diagnostics: all eight cases passed. |
| `003538Z-proton-620be03800cb` | 0 | Experimental with `FSR4_UPGRADE=0`: all eight cases passed. |
| `003727Z-proton-5ff822ab9429` | 0 | Installed Proton 11 automatic mode: all eight cases passed. |

In every successful run, queries and dispatch each reported three validation
instances with core and synchronization enabled, zero errors and zero warnings.
The loaded SDK DLLs matched the exact stage after resolving `S:` through the
prefix's `/home` mapping. No message-ID exclusion or duplicate-message limit
was introduced. The intentional-error attempt remains failed.

Both local and movable queries reported provider `4.1.1`, diagnostic ID
`0xf5a5ca1e01001001`, required resource bits 15 and optional bits 0. At 1280x720
render / 1920x1080 output, estimated and live memory totals were 80,740,352 bytes;
at the single-frame 320x180 / 640x360 dimensions they were 28,180,480 bytes.
Aliasable memory was zero. These are configuration-specific observations.

All four successful GPU runs completed with 640x360 output: 230,400 finite pixels,
230,400 changed pixels, red minimum/maximum 0.10058594/0.84765625 and left/right
means 0.193594/0.7556911. Dispatch, submission, fencing and readback passed;
device-removal status stayed zero, explicit destruction and final cleanup passed.
The observed aggregate metrics agree across these runs; full readback-byte
equality was not captured. DirectX debug acquisition returned `0x887a002d`, and
the info-queue count was unavailable (`UINT64_MAX`), not zero.

Each diagnostic run dumped 30 translated compute shaders. Thirteen contain
`OpSDot` with `PackedVectorFormat4x8Bit` and eight-bit integer types: 4,264 such
static dot-product instructions in total. For example:
`%304 = OpSDot %uint %161 %296 PackedVectorFormat4x8Bit`.
These are **verified compiled INT8 shader operations**, rather than a conclusion
based only on the provider version. The generated shaders do not use cooperative
matrix instructions. Some DXIL names contain `fp8_no_scale`; those labels alone
do not describe the arithmetic in the generated SPIR-V. Shader identities and
the automatic/disabled/Proton-11 comparisons are retained in the evidence.
Experimental's automatic and disabled modes generated identical bytes for all
30 DXIL/SPIR-V pairs. Proton 11 generated the same 30 DXIL inputs; 17 translated
SPIR-V modules matched Experimental byte-for-byte and 13 differed. Aggregate
readback metrics still matched; no universal translation-byte equality is claimed.

**Inference:** the successful 4.1.1 workload selected the INT8 model path, supported
by its compiled packed-integer shaders and completed GPU output. This experiment
does not attribute individual dispatch events to shader hashes or trace executed
machine instructions. A shader dump alone would not establish GPU execution;
the separate dispatch/fence/readback checks are necessary evidence here.

The automatic Experimental run logged the AMD interface's replacement message
and loaded its contrib DLL. Disabling that upgrade still produced 4.1.1 and the
same aggregate output; Proton 11, without that contrib DLL, also passed. Thus
**Experimental and its driver-style upgrade are not required for this pinned
SDK's direct 4.1.1 workload on this machine**. This does not test upgrading games
that ship an older FSR runtime.

The existing infrastructure suite passed 128 tests before modification and 131
afterwards, both without skips or failures. The 11 focused Proton/validation
controls also passed. No Rust production source changed; the GPU runs exercised
the actual Windows/MSVC Rust library tests and existing native helper.

## Source evidence and limits

Primary sources accessed 2026-10-13:

- [Valve AMD interface](https://github.com/ValveSoftware/wine/blob/experimental_11.0/dlls/amdxc64/main.c)
  enables automatic replacement for discrete Navi-3 IDs, including 0x7481,
  loads `amdxcffx64`, and supports disabling it with `FSR4_UPGRADE=0`. This is
  source evidence; the runs above establish the local runtime observations.
- [VKD3D debug callback](https://github.com/HansKristian-Work/vkd3d-proton/blob/master/libs/vkd3d/device.c)
  deliberately supplies an invalid pNext to test validation delivery. The failed
  local run reported `VUID-VkDebugUtilsMessengerCallbackDataEXT-pNext-pNext`.
- [VKD3D shader diagnostics](https://github.com/HansKristian-Work/vkd3d-proton/blob/master/README.md)
  describe shader dumping and cache controls.
- [Khronos SPIR-V definitions](https://github.com/KhronosGroup/SPIRV-Headers/blob/main/include/spirv/unified1/spirv.h)
  identify the integer dot-product operations and packed 4x8-bit format.

These web references are mutable branches with an access date, not independently
reconstructed source commits for the installed binaries. Concrete installed
component hashes and reported versions identify the tested distributions.

This adds a second SteamOS/GPU proof alongside the Steam Deck's recorded 3.1.5
provider baseline. It does not complete temporal regression, demonstrate image
quality, benchmark INT8 cost, prove general provider safety or establish support
for every Proton/driver/GPU combination. Windows D3D12 debug/GBV and Vulkan
GPU-assisted validation remain unverified here. Runtime bootstrap/offline policy
and D019 acceptance remain separate work.

Full logs, downloaded payloads, binaries and shader dumps remain under ignored
`target/`. Derived disassemblies are kept outside the finalized attempt
inventories. The repository retains the compact measurements and small original
probe, not third-party shader binaries or full exports. Topic synthesis,
research-index integration and roadmap/support changes were not performed.
