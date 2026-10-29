# Experiment: Linux cross-build and one Proton DX12 GPU workload

- Investigation date: 2026-10-13, Europe/Paris. Retained attempt timestamps are
  UTC on 2026-10-12, 22:49–23:03.
- Question: Can the Steam Deck build the existing Windows/MSVC Rust tests and
  C++ helper locally, load the pinned AMD runtime through Proton, and complete
  one deterministic upscaler dispatch with checked output and teardown?
- Scope: The bounded proof in proposed [D019](../../adr/D019.md), one GPU and
  one Proton distribution. Complete native/temporal regression, Windows
  diagnostic equivalence, native Linux/Vulkan support and FSR 4 are excluded.
- Success: Actual Windows cross-build, observed staged AMD module paths, public
  queries/context teardown, dispatch, submission/fencing/readback, frozen
  numerical output checks, no device removal and explicit destruction.
- Failure: Missing tools/capabilities, a nonzero child exit, absent test markers,
  unexpected module paths, failed SDK operation or invalid output. Exit zero
  alone is insufficient. Unavailable diagnostics remain unavailable.

## Setup and source state

The starting revision was `a380f7975f7c020ee1d9f5346bf2aa5e20309de4`;
the worktree initially had no local changes.

Local implementation changes added `tools/verification/proton.py`, registered
the experimental profile and its CLI options, and added five infrastructure
tests. The existing Rust single-frame workload now has two explicit entry
points: Windows Gate A retains its required debug layer and info-queue checks;
the Proton selector shares its numerical and GPU work with separately declared
diagnostic limits. The C++ helper adds a device-removal-only checkpoint for that
Proton selector. No public Rust API, crate dependency or supported target changed.

The [compact evidence](2026-10-13-exp-proton-linux-cross-build/results.json)
retains selected source/component hashes, commands, exits, observations and all
eight attempts. It is not a complete verification export. The final run identity
is `1281cbfba62df1c0f442258c7f13dc6e84f683f57fee3e438357939032813b4c`;
the source and selected external inputs remained unchanged during that run.
Full local logs, binaries and cross-toolchain payloads stay under ignored
`target/`; no private paths, complete inherited environment or binary payloads
are required to understand this record.

| Input | Observed configuration |
|---|---|
| Host | SteamOS 3.9.2, build 20260925.101; x86_64; kernel 7.2.7-valve1-1-neptune-72-gc8730d37f9c6 |
| GPU | AMD Custom GPU 0405, RADV VANGOGH; vendor 0x1002, device 0x163f; hardware adapter index 0 |
| Driver | Mesa 26.2.0-devel, git-6c05e09fac; Vulkan 1.4.350; driver 26.1.99 |
| Rust/Cargo | 1.99.0 stable; rustc b940084d7eb6a299eb4bfeb8e34901bc051e7ac4; Windows target x86_64-pc-windows-msvc |
| Orchestrator | Python 3.13.11; standard library only |
| Cross tools | cargo-xwin 0.23.1, xwin 0.10.0; clang-cl/LLD 21.1.8 |
| Microsoft cache | SDK component 10.0.22621; CRT component 14.44.17.14, manifest package build 14.44.35220; x86_64 |
| Proton | Installed Steam Proton 10.0; version file `1785139158 proton-10.0-4b`; bundled Wine components identified by this distribution and hashes |
| Translation | VKD3D-Proton applicationVersion 3.0.0, build 21a49c975df83a7; DXVK v2.6.2-23-g3cb664e1260926e |
| Launcher | umu-launcher 1.4.4, Python 3.14.7; Nix unwrapped executable on SteamOS |
| Runtime | umu steamrt3/sniper depot 3.0.20260928.262393; pressure-vessel/scripts 0.20260925.0 |
| AMD SDK | D017 v2.3.0 install; source commit 60f4ea81909200d8542eca14dccb2628b763a9a3; all eight pinned files verified |
| Signature tool | osslsigncode 2.14; signing-chain verification exited 1 for both DLLs: unable to get local issuer certificate |

LLVM, LLD and umu came from Nixpkgs revision
`7a0f122f5090cf4c2ade2a13a0e229d4e19ba71f`. Their selected executable hashes
are retained in the compact evidence. Cargo-xwin was installed from Cargo with
its version and lockfile selected. It declares MIT; xwin declares Apache-2.0 OR
MIT. Downloaded Microsoft SDK/CRT files retain Microsoft's terms, independently
of the tooling licences. AMD's headers/binaries/notices retain the existing SDK
terms. These are local development inputs, not new published crate dependencies.

## Reproduction

Preparation and the generic command are documented in
[verification setup](../../../tools/verification/README.md#experimental-proton-verification).
The actual bootstrap actions were:

```sh
cargo +stable install cargo-xwin --version 0.23.1 --locked
rustup target add --toolchain stable x86_64-pc-windows-msvc
nix profile install nixpkgs#llvmPackages.clang-unwrapped nixpkgs#llvmPackages.lld nixpkgs#umu-launcher nixpkgs#osslsigncode
"$HOME/.cargo/bin/cargo-xwin" cache xwin --xwin-cache-dir target/proton-toolchain --xwin-arch x86_64 --xwin-sdk-version 10.0.22621 --xwin-crt-version 14.44.17.14
python3 tools/verification/run.py sdk install --root target/sdk-proton/v2.3.0
```

The first two Microsoft-cache attempts used CRT build numbers `14.44.35207`
and `14.44.35220`; both were rejected because the command selects component
versions, not package build numbers. The successful selection above downloaded
and prepared the cache. Cargo-xwin itself requires Rust 1.89 or newer to build;
using stable for tool installation does not raise the project's Rust 1.88 MSRV.

The passing command, with the user-home path normalised, was:

```sh
python3 tools/verification/run.py run proton --native --allow-launcher-network \
  --toolchain stable --sdk-root target/sdk-proton/v2.3.0/Kits/FidelityFX \
  --xwin-cache target/proton-toolchain --cargo-xwin "$HOME/.cargo/bin/cargo-xwin" \
  --umu /nix/store/3dwzqgm2n8h634cgvyl9xfbnxwx9ih8q-python3.14-umu-launcher-unwrapped-1.4.4/bin/umu-run \
  --proton "$HOME/.local/share/Steam/steamapps/common/Proton 10.0"
```

That store path identifies this installation; a different installation must
explicitly supply its own host-compatible launcher. GPU execution needed access
outside the agent sandbox. Native tools and Vulkan access in the sandbox alone
did not establish host capability.

The runner builds the helper with `clang-cl`, the pinned AMD header and prepared
Microsoft include/import-library paths, MSVC ABI, `/MT`, `/W4 /WX` and LLD. Cargo
builds `fsr-sdk` library tests with `dx12,movable`, `--target
x86_64-pc-windows-msvc --no-run --locked --offline`. Complete argument arrays and
the executable/helper hashes are in the compact evidence.

Every attempt gets a new application stage and prefix. Normal Proton launch
initialises the prefix; direct `runinprefix` captures test output, with explicit
`d3d12=n;d3d12core=n;dxgi=n` selection. Copied translator DLL hashes must match
the selected Proton payloads. The runner requires child exit zero **and** test
markers from Proton's redirected log. Wine module-load traces must show both
AMD DLLs from the exact application stage; their hashes are checked independently.

## Declared validation and trust

Before the first unmodified Gate A attempt, the matrix required its Windows
debug layer. That attempt failed; it was not relabelled successful. Before the
separate Proton selector ran, the matrix explicitly allowed missing Windows
debug-layer/info-queue coverage and retained all applicable numerical, dispatch,
completion and device-removal requirements. No threshold was fitted to output.

| Check | Outcome and claim |
|---|---|
| SDK identity | All eight D017 files and provenance checked; loader/provider hashes match fixed project pins |
| Windows Authenticode policy | Unavailable on Linux; never treated as passed |
| Linux signature inspection | Both osslsigncode calls exited 1 on certificate-chain trust; execution explicitly trusts D017 pinned bytes |
| D3D12 debug interface | `DXGI_ERROR_SDK_COMPONENT_MISSING`, HRESULT 0x887a002d |
| D3D12 info queue | Unavailable; `debug_errors=UINT64_MAX`, not zero validation errors |
| GPU-based validation | Not enabled; not claimed |
| Vulkan validation layer | Not enabled; not claimed |
| Numerical and GPU checks | Required and passed in the explicit Proton selector |

## Observations

Eight finalised attempts are retained in the compact evidence:

1. `20261012T224938Z-proton-6ef12c86444b`: Cross-build passed; launch returned
   zero but required test output was absent. Proton's Steam shim detached its
   console. Launch failed evaluation; GPU cases were blocked.
2. `20261012T225248Z-proton-27626a3c841f`: `runinprefix` produced test output,
   but skipped Proton's translator overrides and selected Wine builtins.
   The query case failed its required markers; GPU dispatch was blocked.
3. `20261012T225412Z-proton-723861ee7a8d`: Explicit native translators selected;
   query child exited 101 because DXVK could not see `VK_KHR_surface` through
   the Nix FHS wrapper. Host `vulkaninfo` separately showed a working RADV GPU.
4. `20261012T225655Z-proton-31cde53c7325`: Nix's unwrapped umu launcher restored
   SteamOS driver visibility. Queries passed; unchanged Windows Gate A exited
   101 at its required debug-interface check, before SDK dispatch.
5. `20261012T225827Z-proton-4b397712bf41`: Separately declared Proton GPU
   selector passed all eight profile cases.
6. `20261012T230042Z-proton-704bff96501d`: Passed after strengthening inherited
   environment controls and failed-command observation retention.
7. `20261012T230200Z-proton-a6936c76da3d`: Passed with exact translator payload
   identities and prefix-copy checks.
8. `20261012T230259Z-proton-65a307e04700`: Final profile/prerequisite state;
   eight completed/passed cases, unchanged inputs, profile exit 0.

The installed nixGL wrapper was found in the Nix store, although absent from
PATH/profile. It configured Mesa 25.0.5 OpenGL/EGL paths, not Vulkan ICD selection.
It was not used in the passing configuration. This does not test a nixVulkan
wrapper or establish what a NixOS host would need.

Both local and movable public-query families selected provider
`0xf5a5ca1e00c01005`, name `3.1.5`, returned resource/memory queries and destroyed
their contexts successfully. The single-frame dispatch selected the same
provider. That observation establishes its reported identity, not an ML path.

The dispatch child completed the SDK call, command-list close, queue signal,
fence completion and mapped readback with HRESULT/native code zero. It then
explicitly destroyed the context and released resources; the post-teardown
device-removal checkpoint returned zero.

| Frozen output property | Observed result |
|---|---|
| Render/output size | 320×180 → 640×360 |
| Finite output | 230,400 / 230,400 pixels |
| Output changed from sentinel | 230,400 pixels; requirement > half |
| Red variation | min 0.10076904, max 0.84765625; required range > 0.1 |
| Input gradient retained | left mean 0.19362283, right mean 0.7558286; required difference > 0.05 |
| Device removed | 0 before and after teardown |
| DirectX error count | Unavailable, 18,446,744,073,709,551,615 |

Additional verification: five Proton control tests passed; infrastructure suite
ran 125 tests, 94 passed and 31 explicitly skipped for missing NumPy/PowerShell.
Stable default workspace tests passed (38 tests and two doc tests); Rust 1.88
host all-target check passed; formatting and diff whitespace checks passed.
`verify-run --run target/verification/20261012T230259Z-proton-65a307e04700`
validated the retained final attempt. Windows/MSRV cross-execution and the
native Windows diagnostic path were not rerun on this Linux host.

## Baseline delta and limits

Linux-hosted MSVC Rust/C++ compilation, pinned AMD runtime loading, context
queries and one real GPU upscaler dispatch/readback/teardown are now observed on
this Steam Deck. Cargo can supply cargo-xwin and its explicit Microsoft cache
bootstrap; rustup supplies the target. LLVM, Python and the runtime launcher are
separate development tools. Nix is optional, and its default FHS launcher was
unsuitable for the tested SteamOS driver setup. GCC remains a host compiler,
not the Windows helper compiler.

The initial umu run acquired its own steamrt3 runtime despite an existing Steam
runtime installation. Later runs disabled updates and reused that installation.
Its observed version and post-run metadata hashes are retained, but no complete
pre-run runtime inventory, bounded acquisition policy or fully offline launcher
guarantee is established. The profile therefore still requires explicit launcher
network acknowledgement. Prefixes and unsuccessful attempts remain separate.

The shared 180-second watchdog bounds launcher invocations; termination of
umu's detached Wine services after a watchdog expiry was not independently
tested. Translation-layer logs contain FSR resource-retention workarounds; this
proof makes no broader lifetime or native-Windows-equivalence conclusion from
them. A first launcher debug run included inherited environment data in its raw
local logs; those logs are not exported or retained in this compact record.

Only the public queries and one deterministic single-frame workload were
selected. No temporal quality, complete regression, FSR 4, other GPU/Proton
version or general platform compatibility claim follows. D019 remains proposed;
this experiment does not accept it. Runtime bootstrap/offline and detached-child
cleanup policy remain explicit follow-ups before a maintained-path claim.
