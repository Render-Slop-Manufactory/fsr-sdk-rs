# RC1 Steam Deck verification — 2026-10-29

**Passed: 16/16 offline cases and 17/17 timeout cases on the exact RC1 pair.**
Public queries, GPU dispatch/readback, forced process cleanup and subsequent
GPU execution pass on this Deck. Both finalized attempts pass `verify-run`.
This qualifies the selected Deck compatibility checks, not the entire release.

## Scope and exact inputs

This record covers the bounded Steam Deck qualification of the frozen
`0.3.0-rc1` source-package pair through the existing `proton-offline` and
`proton-timeout` profiles. It uses the distributed archives directly, without
local repackaging. The earlier Deck run used different archives and is not
substituted for this check.

The worktree was clean before execution at
`1b1c7a3fc1fa3877972aa1111eb79e7916c3732c`. The candidate's recorded clean
packaging revision is `6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`.
The [retained candidate inventory](2026-10-29-candidate-package-preparation.json)
and the independently hashed local files agree:

| Archive under `release-candidates/0.3.0/rc1/` | Bytes | SHA-256 |
|---|---:|---|
| `fsr-sdk-0.3.0.crate` | 318,967 | `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766` |
| `fsr-sdk-sys-0.1.2.crate` | 43,576 | `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042` |

Only this report is added to the repository. Existing documentation, source,
verification infrastructure and candidate files are unchanged. Generated test
outputs, prefixes and full evidence remain under ignored `target/`.

## Equipment and reproduction

| Component | Observed configuration |
|---|---|
| Device/OS | Steam Deck; SteamOS 3.9.5, build 20261023.100; x86_64 |
| Kernel | `7.2.7-valve2-2-neptune-72-g31504582a073` |
| GPU | AMD Custom GPU 0405 (RADV VANGOGH), `0x1002:0x163f`; adapter 0, D3D feature level 12_0 |
| Driver | Mesa 26.2.0-devel (`git-5c65d4fc31`); Vulkan 1.4.350 |
| AMD SDK/provider | SDK 2.3.0; provider 3.1.5, ID `0xf5a5ca1e00c01005` |
| Proton | Prepared local Proton 10.0-4b, version `1785139158` |
| Translators | VKD3D-Proton 3.0.0 (`21a49c975df83a7`); DXVK `v2.6.2-23-g3cb664e1260926e` |
| Runtime/launcher | steamrt3/sniper 3.0.20260928.262393; pressure-vessel 0.20260925.0; umu 1.4.4 |
| Cross-build | Rust/Cargo 1.99.0; cargo-xwin 0.23.1; Clang/LLD 21.1.8; prepared Microsoft SDK 10.0.22621 / CRT 14.44.17.14 |
| Coordinator/diagnostics | Locked Nix shell; Python 3.14.7; Khronos validation 1.4.357.0 |

The existing local runtime manifest is
`target/proton-runtime/v08-deck/runtime.json`, SHA-256
`ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1`.
Its imported distributions are the explicit local trust input. No new runtime
import or download is required by these calls. AMD loader/provider DLL hashes:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The initial sandboxed Nix plan command could not access the Nix daemon socket
(`Operation not permitted`). The actual runs used authorised host execution
for Nix, GPU access and namespace containment. This setup rejection did not run
the candidate and is separate from the test outcomes below.

From the repository root, enter
`nix develop path:./tools/verification/nix --offline`. The actual invocations
used its `--command bash -c` form with the following command, first for
`proton-offline`, then for `proton-timeout`:

```sh
python3 tools/verification/run.py run proton-offline --artifacts --require-passed \
  --native --toolchain stable \
  --runtime-manifest target/proton-runtime/v08-deck/runtime.json \
  --runtime-sha256 ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST" \
  --wrapper-archive release-candidates/0.3.0/rc1/fsr-sdk-0.3.0.crate \
  --sys-archive release-candidates/0.3.0/rc1/fsr-sdk-sys-0.1.2.crate
```

Both profiles use the same explicit archive paths. Rust tests build for
`x86_64-pc-windows-msvc` with `dx12,movable` from the extracted packages and an
isolated local Cargo registry. The C++ GPU helper also uses the extracted pair.
No checkout-only build is substituted for package execution.

## Results

All times below are UTC on 2026-10-29 (local Europe/Paris time is UTC+1).

| Profile | Attempt under `target/verification/` | Start–finish (UTC) | Required cases |
|---|---|---|---|
| `proton-offline` | `20261029T165712Z-proton-offline-d3b7e85f9e6a` | 16:57:14–16:58:29 | 16/16 passed |
| `proton-timeout` | `20261029T165920Z-proton-timeout-4b766564f60a` | 16:59:22–17:01:19 | 17/17 passed |

Every required case completed; there are no failed, blocked or skipped cases.
A dash means the case is not part of that profile.

| Case ID | Offline | Timeout |
|---|---|---|
| `equipment.toolchain` | Passed | Passed |
| `proton.equipment` | Passed | Passed |
| `release.package` | Passed | Passed |
| `release.registry` | Passed | Passed |
| `artifact.bind` | Passed | Passed |
| `proton.offline.missing-manifest` | Passed | — |
| `proton.offline.missing-proton` | Passed | — |
| `proton.offline.missing-runtime` | Passed | — |
| `proton.sdk` | Passed | Passed |
| `proton.helper` | Passed | Passed |
| `proton.build` | Passed | Passed |
| `proton.launch` | Passed | Passed |
| `proton.queries` | Passed | Passed |
| `proton.dispatch` | Passed | Passed |
| `proton.timeout.fixture` | — | Passed |
| `proton.timeout.expiry` | — | Passed |
| `proton.timeout.post-queries` | — | Passed |
| `proton.timeout.post-dispatch` | — | Passed |
| `proton.offline.integrity` | Passed | Passed |
| `release.integrity` | Passed | Passed |

### Package, offline and query checks

The selected archives passed extraction/content checks against the checkout and
were bound to the isolated Cargo registry. Final audits in both attempts found
the archives and extracted bytes unchanged; both run summaries also record
`inputs_unchanged=true`. The prepared Proton, runtime and launcher inventories
were unchanged at the end of each run.

The offline negative controls correctly rejected a missing manifest, missing
Proton and missing runtime. Launcher observations confirmed distinct PID and
network namespaces, `--die-with-parent`, loopback-only interfaces and IPv4/IPv6
errno 101 (`ENETUNREACH`). No network-enabled launcher fallback was used.

All three public-query invocations (offline, timeout before expiry, timeout after
expiry) passed for local and movable ownership. Both families reported provider
3.1.5, memory estimate 71,172,096 bytes, live usage 78,970,880 bytes and aliasable
usage 11,010,048 bytes; resource requirements were required flags 15 and optional
flags 48. Enumeration, metadata, memory/resource queries, explicit provider
selection and context destruction passed. Observed AMD module paths resolved to
the staged DLLs with the pinned hashes recorded above.

### GPU execution and diagnostics

All three single-frame GPU invocations (offline, timeout before expiry, timeout
after expiry) passed dispatch, fence/readback, finite/write, gradient and explicit
destruction/resource-cleanup checks. Each reported the following measurements:

| Readback measurement | Result |
|---|---:|
| Output size | 640 × 360 |
| Finite pixels | 230,400 / 230,400 |
| Written/changed pixels | 230,400 / 230,400 |
| Red minimum / maximum | 0.10076904 / 0.84765625 |
| Left / right red mean | 0.19362283 / 0.7558286 |
| Device removal | 0 |

All six query/dispatch children observed Vulkan core and synchronization
validation active, with three validated instances per child, **zero errors and
zero warnings**. Native context creation and explicit destruction returned zero.
These are integration checks on the shared single-frame workload, not temporal
image-quality or performance acceptance.

### Forced timeout and isolation

The cross-built Windows fixture proved its parent and detached child ready with
advancing heartbeats before the real one-second watchdog began. The observed
command timed out with exit `-9`, as explicitly required by this diagnostic case.
Its recorded 6.602-second elapsed time includes readiness/startup, rather than
being a measurement of the watchdog interval alone.

Independent pidfd and prefix-process inventories found all observed target
descendants stopped within the five-second cleanup bound, no surviving target
processes and stopped parent/child heartbeats. A separate sentinel prefix
retained process identity and an advancing heartbeat. The fixture recorded
`detached_groups=true`, `descendants_stopped=true`, `heartbeats_stopped=true`,
`unrelated_processes_preserved=true` and `unrelated_heartbeat_advanced=true`.
Emergency cleanup was empty. Public queries and GPU dispatch then passed on
the original verification prefix. This is process-timeout cleanup evidence;
it does not demonstrate recovery from a GPU/kernel hang.

## Evidence validation and retention

`verify-run --run <attempt>` returned exit 0 for both finalized attempts,
checking their retained artefact inventories. The table identifies the original
local summary and inventory bytes, not a requirement for reproducing identical
future logs:

| Attempt/profile | File | SHA-256 |
|---|---|---|
| Offline | `run.json` | `b0e679e28bbb511a4c9b8a99d6c1063e83ed4516adf356d7380ec85c1cf37f62` |
| Offline | `artifacts.json` | `d9568896e022f1e3497ce5dede41ebb06903cb1025a6b6713b5b7270a67ae533` |
| Timeout | `run.json` | `8943f300e0f0168cdddf5a2e2c96d7da6ade8b4ee9c7c35a38408d07c7d461c8` |
| Timeout | `artifacts.json` | `bcf4fdae9f1d5b9586fac929caf0e1f6e9a94a8afec1748c71d39ca2aaf50b96` |

Sanitized full exports are local under
`target/verification-exports/2026-10-29-rc1-deck-proton-offline` and
`target/verification-exports/2026-10-29-rc1-deck-proton-timeout`.
This report retains the inputs, commands, all required outcomes and decisive
measurements without adding full exports, environment dumps or raw GPU payloads
to the repository.

## Limits and release boundary

The DirectX debug layer was unavailable (`HRESULT=0x887a002d`); the info-queue
error count was `18446744073709551615` (`UINT64_MAX`), meaning unavailable,
not zero errors. D3D12 GPU-based validation and Vulkan GPU-assisted validation
were not enabled. Linux `osslsigncode` returned exit 1 for both AMD DLLs because
the local certificate chain could not be completed; this remains a diagnostic
observation. Execution trusts the pinned SDK bytes and does not assert Windows
Authenticode-policy acceptance.

The tested runtime is a checksum-selected local import, not a complete pinned
host/driver dependency closure or fresh offline installation proof. This record
does not cover Steam Machine, native Windows/MSVC runtime execution, the full
Windows feature/native matrix, MSRV, `utils`, benchmarks or temporal quality.
The existing closed P08 review is not rerun or extended here. No general provider,
native Linux/Vulkan or complete temporal-regression support follows from this
bounded Proton result. Publication simulation, final release reconciliation and
publication remain separate; no publication was performed or authorised here.
