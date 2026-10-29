# Steam Deck package-based Proton verification — 2026-10-29

The current `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2` checkout passes the bounded
Proton compatibility recheck on Steam Deck through freshly generated packages:
**16/16 offline cases and 17/17 timeout-cleanup cases passed**. Both runs use
the same exact archive pair. The [compact evidence](2026-10-29-deck-proton-verification.json)
retains every required outcome, command, package identity, compiler/runtime
observation, validation result and independent cleanup verdict.

This supplies fresh Deck evidence for the replacement API and package-mode
execution. It does not qualify the complete release or identify these packages
as the final publication pair. The Steam Machine was not available in this local
Deck session and has no fresh result here. Its October 13/14 results remain
historical evidence.

## Inputs

The worktree was clean before execution. The offline run generated packages from
the checkout; the timeout run explicitly selected those unchanged archives:

| Package | SHA-256 |
|---|---|
| `fsr-sdk-0.3.0.crate` | `56ca4a23e897063e7a0c975d6233919fa39f1db6e498c9c2b6977a709eefbdb5` |
| `fsr-sdk-sys-0.1.2.crate` | `af98d189914fbab70360056a40cdb1b044bae52b5757695d7b9fb8fade1f22d5` |

These are distinct from the earlier Windows/P08 development archive identities.
No equivalence to that pair or final performance/package reconciliation is
claimed. Each run audited the selected packages against the checkout, built the
Rust executable from the extracted workspace with an isolated registry and
compiled the C++ helper against the extracted wrapper/sys inputs. Final audits
found no changed archives or extracted inputs. Runtime distribution inventories
also remained unchanged. Only documentation/evidence was added after execution.

| Equipment | Observed configuration |
|---|---|
| Host | SteamOS 3.9.5, build 20261023.100; x86_64; kernel 7.2.7-valve2-2-neptune-72-g31504582a073 |
| GPU | AMD Custom GPU 0405 (RADV VANGOGH), `0x1002:0x163f`, feature level 12_0 |
| Driver | Mesa 26.2.0-devel, git-5c65d4fc31; Vulkan 1.4.350 |
| AMD SDK/provider | Pinned SDK 2.3.0; provider 3.1.5, ID `0xf5a5ca1e00c01005` |
| Compatibility stack | Prepared Proton 10.0-4b (`1785139158`); VKD3D-Proton 3.0.0, build 21a49c975df83a7; DXVK v2.6.2-23-g3cb664e1260926e |
| Runtime/launcher | steamrt3/sniper 3.0.20260928.262393; pressure-vessel 0.20260925.0; umu 1.4.4 |
| Cross-build | Rust/Cargo 1.99.0; cargo-xwin 0.23.1; Clang/LLD 21.1.8; prepared Microsoft SDK 10.0.22621 / CRT 14.44.17.14 |
| Coordinator/validation | Locked Nix shell; Python 3.14.7; bubblewrap 0.12.0; Khronos validation 1.4.357.0 |

The existing local runtime manifest SHA-256 is
`ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1`.
It selects the previously imported local stack, not a new runtime download or
independent authenticity assertion. Every launcher invocation was contained in a
distinct PID/network namespace with only loopback and IPv4/IPv6 errno 101.

## Results

| Profile / local attempt under `target/verification/` | UTC interval | Result |
|---|---|---|
| `20261029T022510Z-proton-offline-4001f29aefa0` | 02:25:14–02:26:45 | 16/16 passed |
| `20261029T022709Z-proton-timeout-c01a9616ddf3` | 02:27:12–02:29:09 | 17/17 passed |

Both query families enumerate providers and exercise public metadata, memory,
resource requirements and explicit provider selection, then destroy contexts.
All six query/dispatch children across the two runs, including post-timeout
controls, observed Vulkan core and synchronization validation activation with
zero errors and zero warnings. Each child reported three validated instances.

All three single-frame dispatches passed the existing fence/readback, finite/write,
gradient and explicit teardown checks. The 640×360 output had all 230,400 pixels
finite and written, red range 0.10076904–0.84765625 and left/right means
0.19362283/0.7558286. Device removal remained zero.

The timeout fixture first proved its Windows parent and detached child live,
then triggered the real one-second watchdog. Independent pidfd/prefix inventories
found no surviving target descendants within the five-second bound; both
heartbeats stopped. The unrelated prefix retained process identity and heartbeat
activity. No emergency termination was needed. Subsequent queries and GPU
dispatch passed on the verification prefix. This verifies process cleanup,
not recovery from a GPU/kernel hang.

Both finalized runs passed `verify-run`; sanitized full exports remain under
`target/verification-exports/2026-10-29-deck-proton-{offline,timeout}`. The compact
JSON omits unrelated drive mappings and verbose signature diagnostics while
retaining hashes of original summaries, case records and inventories. Full logs,
archives, prefixes and build outputs remain local under ignored `target/`.

## Reproduction and limits

From the repository root, enter the locked environment with
`nix develop path:./tools/verification/nix --offline`. The actual calls used its
`--command bash -c` form. Run on the host with GPU/namespace access:

```sh
python3 tools/verification/run.py run proton-offline --artifacts --require-passed \
  --native --toolchain stable \
  --runtime-manifest target/proton-runtime/v08-deck/runtime.json \
  --runtime-sha256 ff4406dce8d0b43b2360f11c5d533ad74e0c6c1ea6e579fff45291e2a7db84d1 \
  --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --umu "$VERIFICATION_UMU" \
  --vulkaninfo /usr/bin/vulkaninfo \
  --vulkan-validation-manifest "$VERIFICATION_VULKAN_MANIFEST"
```

For the second run, change the profile to `proton-timeout` and replace
`--artifacts` with both archive options pointing to the first attempt's
`package-build/package/fsr-sdk-0.3.0.crate` and `fsr-sdk-sys-0.1.2.crate`.
Use those archive options for future checks of these exact bytes; generating
packages again may produce different identities. The initial sandbox could not
access the Nix daemon; authorized host execution supplied GPU and namespace access.

DirectX debug-layer HRESULT was `0x887a002d`; the unavailable info-queue count
remained `18446744073709551615`, not zero errors. D3D12 GBV, Vulkan GPU-assisted
validation and Windows Authenticode policy were not verified. Linux signature
diagnostic exit 1 remains an observation; execution trusts the pinned AMD bytes.
No MSRV, full Windows suite, performance benchmark, temporal/image-quality suite,
native Linux/Vulkan support or publication is claimed. Final release acceptance
must reconcile its selected package hashes and all remaining qualification gates.
