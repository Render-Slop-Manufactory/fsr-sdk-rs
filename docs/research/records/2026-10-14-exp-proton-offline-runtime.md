# Experiment: prepared Proton runtime with networking disabled

- Date: 2026-10-14 Europe/Paris; native attempts ran 2026-10-13 23:46–23:51 UTC.
- Question: can explicitly prepared, checksum-selected local Proton/sniper inputs
  execute the existing query/single-frame proof without external IP connectivity,
  and reject missing inputs without acquiring replacements?
- Result: **verified for the selected local-import policy on this Steam Deck**.
  Both fresh attempts passed all 12 required cases, with distribution integrity
  unchanged. This completes the bounded V08 checkpoint, not a fresh-OS tool
  bootstrap, network downloader, portable runtime distribution or OS closure.
- [Protocol declared before execution](2026-10-14-exp-proton-offline-runtime/protocol-before-run.md)
  and [compact execution evidence](2026-10-14-exp-proton-offline-runtime/results.json).
  V07d's failed criteria/launch and V09's forced-timeout cleanup remain separate.

## Tooling and trust policy

The starting worktree was clean at `7a113d3917955a5e8f90ae761f6d198ca7a176d1`.
Changes add `proton_runtime.py`, explicit `runtime prepare/verify` commands and
`proton-offline`, sharing the existing build, launcher, query and GPU cases.
No Rust API, crate dependency, SDK pin or native support target changes.

Preparation copies installed Proton and steamrt3 into a new local directory,
preserves notices/symlinks/modes, pins the installed umu package and copies its
prepared shim. The local-source policy has a combined 4 GB/100,000-entry bound
and 300-second checked import deadline. It rejects overlapping/existing outputs
and changed source/import identities. It performs no network acquisition. A
partial failed import is retained and lacks a completed manifest. Checksums
represent explicit trust in existing local installations; they do not establish
upstream authenticity or Windows Authenticode policy. No third-party bundle is
added to published repository contents.

The manifest includes complete regular-file hashes, modes, directory nodes and
literal symlink targets for Proton, sniper and the declared launcher package.
Only sniper's generated `var/` cache is excluded; umu-shim has a separate hash.
The expected manifest checksum must be supplied explicitly. Missing, changed or
additional distribution nodes reject execution. The launcher stays at its
declared installed path, including its external dependencies; this is a local
prepared stack rather than a relocatable self-contained distribution.

Each umu call runs through bubblewrap with new user/network namespaces. Before
exec, the guard requires a distinct network namespace, only loopback, and
ENETUNREACH from IPv4/IPv6 probes to reserved documentation addresses. It never
falls back to networked execution. Proxy controls are removed. Updates/retries
are disabled, but installed umu's `setup_umu` still attempts an upstream lookup
before testing UMU_RUNTIME_UPDATE; network isolation, not that variable, enforces
the offline boundary. This source observation came from the installed umu 1.4.4
`umu_runtime.py`. Ordinary/release profiles retain their execution contracts.

## Inputs and equipment

| Input | Configuration |
|---|---|
| Host | SteamOS 3.9.2 build 20260925.101; kernel 7.2.7-valve1-1-neptune-72-gc8730d37f9c6; x86_64 |
| GPU/driver | AMD Custom GPU 0405, `0x1002:0x163f`, RADV VANGOGH; Mesa 26.2.0-devel git-6c05e09fac; feature level 12_0 |
| AMD | D017 SDK 2.3.0; observed provider 3.1.5, ID `0xf5a5ca1e00c01005` |
| Compatibility stack | Steam-installed Proton 10.0-4b, `1785139158 proton-10.0-4b`; VKD3D-Proton 3.0.0; umu 1.4.4 |
| Container | steamrt3/sniper 3.0.20260928.262393; pressure-vessel/scripts 0.20260925.0 |
| Build tools | Rust/Cargo 1.99.0, cargo-xwin 0.23.1, Clang/LLD 21.1.8; prepared Microsoft SDK 10.0.22621/CRT 14.44.17.14 |
| Coordinator | Locked verification Nix shell; Python 3.14.7/NumPy 2.5.2; bubblewrap 0.12.0 |
| Diagnostics | Khronos validation 1.4.357.0, core/synchronization required; GPU-assisted validation disabled |

Manifest SHA-256: `ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1`.
Component identities hash sorted inventory JSON, including bytes/modes/link targets:

| Component | Entries | Regular-file bytes | Inventory SHA-256 |
|---|---:|---:|---|
| Proton | 8,138 | 1,404,192,639 | `ceeed77a92488627402161d2f255799d606c2dda777d50025d2b5911fd6bb3ef` |
| sniper | 9,593 | 810,303,158 | `ec3b47dbb379a66706fa35894dfbc29deba4f9a986355d63c6bf255782ba8a84` |
| umu package | 63 | 1,484,292 | `ecc31faf8fa5ac3347278ea24fc1d6da74a97ee71483614731f11f7ead565517` |

Both attempts independently cross-built identical native bytes. Rust executable
SHA-256: `7b88f69a19ce236e84ecdcd14ae6d927ba686e421f0059cb065d6fcfc6161098`;
C++ helper: `5df2adc3241e92e4e9bf3178c8ab37fe8979f34aa3a1d835ccb2ec7b3f2a4fb7`.
Observed staged AMD loader/provider hashes are respectively
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
Evidence retains selected source-file identities, shim hash and module paths.

## Reproduction

From the repository root, using the existing prepared shell and local inputs:

```sh
nix develop path:./tools/verification/nix
python3 tools/verification/run.py runtime prepare \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 10.0" \
  --runtime "$HOME/.local/share/umu/steamrt3" --umu "$VERIFICATION_UMU" \
  --launcher-root /nix/store/3dwzqgm2n8h634cgvyl9xfbnxwx9ih8q-python3.14-umu-launcher-unwrapped-1.4.4 \
  --output target/proton-runtime/v08-deck \
  --provenance 'Existing Steam installation Proton 10.0-4b and umu-prepared sniper 3.0.20260928.262393; explicit local trust, not independent upstream signature verification'
python3 tools/verification/run.py run proton-offline --native --toolchain stable \
  --runtime-manifest target/proton-runtime/v08-deck/runtime.json \
  --runtime-sha256 ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

Actual calls used `nix develop path:./tools/verification/nix --command` before
each Python invocation. The run command was executed twice; the original umu
source was the package above. `$VERIFICATION_UMU` resolved to that package's
`bin/umu-run`; validation came from the locked shell's 1.4.357.0 package. Local
GPU/namespace execution required running outside the agent sandbox. A new import
needs a new destination and its printed checksum: the manifest includes absolute
local paths, so another destination will have a different manifest hash.

## Results and limitations

| Attempt under target/verification | UTC interval | Result |
|---|---|---|
| 20261013T234657Z-proton-offline-ed73de9088c9 | 23:46:59–23:47:57 | 12/12 passed, inputs unchanged |
| 20261013T235017Z-proton-offline-21ceb6103b0a | 23:50:19–23:51:18 | 12/12 passed, inputs unchanged |

Each case inventory covers toolchain/equipment, missing manifest/Proton/sniper,
pinned AMD deployment, Linux-hosted C++/Rust Windows cross-builds, isolated launch,
both query families, single-frame dispatch and final runtime integrity. Missing
inputs returned exit 1 before umu invocation; empty runtime fixtures remained
empty. Four umu calls per attempt (two listing/setup calls, queries, dispatch)
each reported only `lo` and IPv4/IPv6 errno 101. The missing-input controls also
ran inside isolated namespaces. All launcher/native statuses were zero.

Both GPU children in each attempt activated core/synchronization validation for
three instances, with zero errors/warnings. Checked 640×360 readback had all
230,400 pixels finite and written, red range 0.10076904–0.84765625, left/right
means 0.19362283/0.7558286, no device removal, and successful explicit teardown.
DirectX debug-layer HRESULT remains `0x887a002d`; unavailable info-queue count
remains `18446744073709551615`, not zero errors. Windows GBV/signature-policy
equivalence and temporal/image-quality acceptance are not established.

The first confirmation preceded failure-path hardening: an unavailable isolation
tool now reports its original blocked reason without command-bookkeeping failure.
The second confirms that final code. Both finalized artifact inventories passed
`verify-run`; neither native attempt failed or was overwritten. Final complete
infrastructure suite: `nix develop path:./tools/verification/nix --command python3
-m unittest discover -s tools/verification/tests -v`, 147 tests passed with no
skips, including real PowerShell parsing. No Rust changes required another Rust
suite; both native attempts rebuilt the Windows tests/helper successfully.

Full manifests, distributions and logs remain ignored local diagnostics; this
record/evidence retain compact results without raw pointers or private home paths.
The checkout budget deliberately grows from 15 to 15.1 MB for tooling/tests and
compact records; the 1 MB individual-file limit and raw-payload exclusions remain.
Offline verification covers these prepared distribution inputs, not all host
libraries/drivers, external launcher dependencies, missing toolchain downloads,
arbitrary Proton versions, remote acquisition or another hardware configuration.
Detached Wine-service termination on watchdog expiry remains V09.
