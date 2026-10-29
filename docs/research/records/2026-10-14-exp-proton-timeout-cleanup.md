# Experiment: Proton watchdog cleanup across detached sessions

- Date: 2026-10-14 Europe/Paris; native attempts 00:06–00:25 UTC.
- Question: does watchdog expiry terminate its launcher/Wine descendants while
  preserving an independent Wine application, and can the same prefix run normal
  public-query/GPU controls afterwards?
- Result: **the old process-group mechanism failed; PID-contained launches pass
  two fresh confirmations.** The normal network-acknowledged Proton profile also
  passes. V09 is verified for this bounded Steam Deck/Proton stack.
- [Predeclared protocol](2026-10-14-exp-proton-timeout-cleanup/protocol-before-run.md)
  and [compact execution evidence](2026-10-14-exp-proton-timeout-cleanup/results.json)
  retain the failed baseline, successful confirmations and source identities.

## Source and method

Tests used the uncommitted V08 tooling plus the new `proton-timeout` profile,
`proton_timeout.py` observer and `fixtures/proton_timeout.cpp` Windows fixture.
Each attempt freezes source/native identities and checks source stability at
completion; the evidence identifies the actual scripts and compiled bytes.
During this work a concurrent commit added the Steam Machine record, advancing
HEAD from the previously observed `7a113d3917955a5e8f90ae761f6d198ca7a176d1`
to `ecbd02054100bc77a7d6567cc3320398f19da1df`. That record remains unchanged;
source fingerprints, not that documentation commit, identify this experiment.
No Rust API, ownership contract, crate dependency, SDK pin or native target changes.

The shared Runner still starts a new session and uses SIGKILL on its subprocess
process group at expiry. Source inspection of installed umu 1.4.4 found its
`umu_run.py` starts its runtime child with `start_new_session=True`. The Windows
fixture additionally creates a child with DETACHED_PROCESS and
CREATE_NEW_PROCESS_GROUP. These provide real sessions beyond the original
watchdog group, not a mocked process tree.

Each fixture process writes a heartbeat every 100 ms, supports a cooperative stop
file and self-expires after 120 seconds. A second fresh prefix runs an independent
sentinel. The fixture wraps only Popen's startup boundary to await advancing
parent and child heartbeats before returning to Runner's unchanged one-second
`wait(timeout)`. Startup readiness is separately bounded and included in elapsed
time. The five-second cleanup oracle requires all observed pidfds to signal exit,
no live non-zombie processes matching the exclusively created target prefix, and
target heartbeats stable for a further 300 ms. It also requires the sentinel's
heartbeat to advance, its launcher to remain live and its Windows actor's PID/start
identity to remain unchanged. The observer emits ownership fields only, never
complete process environments.

Independent cleanup requests cooperative stops first; if needed it signals only
owned-prefix processes through pidfds after rechecking PID/start identity. It
never uses a global wineserver kill. Successful fallback cleanup cannot turn a
watchdog failure into a pass. Post-timeout controls require completed independent
fixture cleanup; startup/cleanup failure blocks them. They reuse the same test
prefix without deletion, restoration or reinitialization.

## Reproduced failure and correction

In the original mechanism the watchdog root was PID 90438. After its SIGKILL,
12 prefix-owned processes remained, including the parent actor PID 90627 in
group/session 90541, detached child PID 90629 in group/session 90629, wineserver,
services.exe, winedevice.exe, svchost.exe and the container/Proton helpers.
The sentinel continued running with its original actor identity. Cooperative
stop files then recovered both fixtures without emergency PID signals; the
ordinary query/GPU controls passed, but the watchdog verdict remained failed.

`proton_runtime.contained` now places each prepared Proton invocation in a new
user/PID namespace with bubblewrap `--unshare-pid --die-with-parent`. Offline
invocations additionally retain their V08 network namespace. Before exec, the
guard verifies that its PID namespace differs from its caller's. The namespace
init's lifetime contains detached sessions, so they cannot escape into the host
session when the watchdog kills its root. Prepared ordinary and temporal Proton
profiles use the same containment; ordinary profiles retain their existing
network acknowledgement and network access. Missing bubblewrap or failed isolation
blocks launch, without an uncontained fallback. The generic Runner's Windows
taskkill and ordinary non-Proton behaviour remain unchanged.

## Inputs and reproduction

SteamOS 3.9.2 build 20260925.101; kernel
7.2.7-valve1-1-neptune-72-gc8730d37f9c6; AMD Custom GPU 0405/RADV VANGOGH
(`0x1002:0x163f`), Mesa 26.2.0-devel git-6c05e09fac, feature level 12_0.
Installed Proton 10.0-4b (`1785139158 proton-10.0-4b`), VKD3D-Proton 3.0.0,
umu 1.4.4, sniper 3.0.20260928.262393 and bubblewrap 0.12.0. The prepared
V08 manifest SHA-256 remains
`ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1`.
It pins local distribution identities, not the complete host OS/dependency closure.

Builds used the locked verification shell: Rust/Cargo 1.99.0, cargo-xwin 0.23.1,
Clang/LLD 21.1.8, prepared Microsoft SDK 10.0.22621/CRT 14.44.17.14 and Python
3.14.7. GPU controls require Khronos validation 1.4.357.0 core/synchronization.
AMD SDK 2.3.0 selected provider 3.1.5, ID `0xf5a5ca1e00c01005`. Linux trusts
the project's pinned bytes; Windows strict signature-policy equivalence is absent.

The Windows fixture's SHA-256 is identical in all three timeout attempts:
`adbf02577bbaf2f1ac557404c66092401083013a9df4ad5d415eae0fd04bfa8c`.
The Rust executable and shared C++ helper remain respectively
`7b88f69a19ce236e84ecdcd14ae6d927ba686e421f0059cb065d6fcfc6161098` and
`5df2adc3241e92e4e9bf3178c8ab37fe8979f34aa3a1d835ccb2ec7b3f2a4fb7`.
Observed AMD loader/provider hashes remain
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.

From the repository root, with V08's prepared local inputs:

```sh
nix develop path:./tools/verification/nix --command \
  python3 tools/verification/run.py run proton-timeout --native --toolchain stable \
  --runtime-manifest target/proton-runtime/v08-deck/runtime.json \
  --runtime-sha256 ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain \
  --umu /nix/store/3dwzqgm2n8h634cgvyl9xfbnxwx9ih8q-python3.14-umu-launcher-unwrapped-1.4.4/bin/umu-run \
  --vulkaninfo /usr/bin/vulkaninfo \
  --vulkan-validation-manifest /nix/store/2p3ff6vbl8i2gyf2zvickc48k7hm9x0k-vulkan-validation-layers-1.4.357.0/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json
```

The normal-profile control used the same shell, tools, SDK and cache, replacing
`proton-timeout` with `proton`, omitting both runtime-manifest options, and adding
`--allow-launcher-network --proton "$HOME/.local/share/Steam/steamapps/common/Proton 10.0"`.
It used the already installed home runtime; no new tooling or distribution was
acquired. Native GPU/namespace execution required running outside the sandbox.

## Results and limits

| UTC attempt under target/verification | Outcome | Forced command / prefix survivors |
|---|---|---|
| 20261014T000636Z-proton-timeout-cc249cd0365b | Failed; 12/13 cases passed | timed_out, -9; 12 surviving processes |
| 20261014T001526Z-proton-timeout-2d2d8b85cb48 | Passed; 13/13 | timed_out, -9; zero survivors |
| 20261014T002049Z-proton-timeout-1f50360a32cb | Passed; 13/13 | timed_out, -9; zero survivors |
| 20261014T002434Z-proton-d8d4fbc93955 | Normal profile passed; 8/8 | no forced timeout |

Forced command elapsed times, including synchronized startup, were 7.459,
7.236 and 7.383 seconds; the actual watchdog wait was one second. Both corrected
attempts tracked 15 owned processes, required their exit before cooperative
fixture cleanup, stopped both target heartbeats, preserved the sentinel and
needed zero emergency cleanup signals. All four attempts retained unchanged
source/input identities and passed artifact integrity verification, including
the failed baseline. The second confirmation additionally hardens post-control
gating on completed cleanup; it does not change the containment mechanism.

Before and after every forced expiry, both public-query families and the shared
single-frame dispatch/fence/readback/teardown checks passed. All GPU children
activated core/synchronization validation for three instances with zero
errors/warnings. The 640×360 readback remained finite/written in all 230,400
pixels and reported no device removal. DirectX debug HRESULT `0x887a002d` and
unavailable info-queue count `18446744073709551615` retain their meaning;
Windows GBV/debug equivalence is not claimed. Prepared runtime distributions
also matched their V08 inventories after all timeout attempts.

Final infrastructure command: `nix develop path:./tools/verification/nix --command
python3 -m unittest discover -s tools/verification/tests -v`: 153 tests passed
without skips, including real PowerShell parsing, ownership/privacy controls,
live-descendant rejection, missing-containment rejection and post-cleanup gating.
No Rust source changes required another Rust suite; native binaries/helpers were
cross-built in each attempt. Full logs/heartbeats stay under ignored target;
published results normalize private paths and contain no raw memory addresses.
The 15.6 MB checkout budget accommodates the concurrently retained Machine
comparison and this tooling/evidence; the 1 MB per-file limit remains.

This proof covers real Windows application/service processes on the selected
stack, not deliberate GPU/kernel hangs, uninterruptible kernel tasks, provider
termination guarantees, arbitrary host applications or other Proton versions.
Namespace/pidfd availability remains an explicit prerequisite. Nothing here
identifies the cause of the earlier completed jitter launch's exit 247 or accepts
V07d's numerical failures. Broader runtime/provider comparisons remain separate.
