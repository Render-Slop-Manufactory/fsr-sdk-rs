# Locked Linux verification environment

From the repository root on x86_64 Linux:

```sh
nix develop path:./tools/verification/nix
python3 -m unittest discover -s tools/verification/tests -v
```

The explicit `path:` form also includes newly created, untracked flake files.
The committed `flake.lock` pins nixpkgs. The shell supplies Git, GCC, rustup,
cargo-xwin, Clang/LLD, Python with NumPy/Pillow, PowerShell, unwrapped umu,
osslsigncode, Vulkan tools and Khronos validation layers. It neither installs
packages into the user's profile nor downloads SDK/runtime payloads on entry.
The initial Nix build may fetch the locked packages.

Rust toolchains and targets remain explicit rustup inputs. For a new machine:

```sh
rustup toolchain install stable --profile minimal --component rustfmt,clippy
rustup toolchain install 1.88.0 --profile minimal
rustup target add --toolchain stable x86_64-pc-windows-msvc
cargo-xwin cache xwin --xwin-cache-dir target/proton-toolchain --xwin-arch x86_64 --xwin-sdk-version 10.0.22621 --xwin-crt-version 14.44.17.14
python3 tools/verification/run.py sdk install --root target/sdk-proton/v2.3.0
```

The shell includes the pinned cargo-xwin package; a separate `cargo install` is
unnecessary. Stable follows rustup's selected installation; the flake does not
freeze that compiler. The Microsoft SDK/CRT and AMD SDK keep their existing
explicit acquisition and licence boundaries. Supply an installed Proton
distribution separately. umu's runtime bootstrap/network acknowledgement remains
separate too; this is not a completely offline runtime setup.

After that preparation, inside the shell:

```sh
python3 tools/verification/run.py run proton --native --allow-launcher-network \
  --toolchain stable --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --proton /path/to/installed/Proton \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

`VERIFICATION_UMU` selects the unwrapped launcher, avoiding the FHS wrapper that
hid the SteamOS driver in the initial experiment. The manifest variable identifies
the exact validation package; no Vulkan layer is enabled globally on shell entry.
On the tested SteamOS host, add `--vulkaninfo /usr/bin/vulkaninfo`: the Nix tool
could not resolve the host driver's libdrm/libz dependencies. Driver setup remains
a host prerequisite, including on NixOS. The shell does not replace drivers or
apply nixGL implicitly. The same command can be used on another SteamOS device;
its GPU, driver and provider results require an actual run there.

The required mode checks Vulkan core and synchronization activation in each GPU
child and rejects any validation error. It records warnings and preserves failed
attempts. Vulkan GPU-assisted validation and Windows D3D12 debug/GBV are separate
checks and are not enabled by this mode. See the
[profile contract](../README.md#experimental-proton-verification).
