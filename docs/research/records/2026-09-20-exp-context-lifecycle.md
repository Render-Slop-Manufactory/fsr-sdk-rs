# Experiment: Successful DX12 upscaler context creation and destruction

- Date: 2026-09-20.
- Question: Can the existing Rust loader create a real v2.3.0 DX12 upscaler
  context, identify its provider, and destroy it once? Does destroy null the handle?
- Success: hardware device creation succeeds, create returns OK with a non-null
  handle, provider query returns a valid identity, and destroy returns OK.
  Nulling after destroy is an observation, not a success requirement.
- Exclusions: public ABI expansion, safe context/RAII/Drop, dispatch, application
  resources, custom allocators, negative/crash tests, device AddRef/lifetime
  experiments, threading guarantees and general cleanup policy.

## Setup and implementation boundary

Base revision: `65a7d52d078b00653a8b7e2b0e5c7fb31d8141c0`, plus the new
[Rust probe](../../../crates/fsr-sdk-sys/tests/context_lifecycle.rs) and
[support files](../../../crates/fsr-sdk-sys/tests/context_lifecycle/).
No production Rust source, manifests, dependencies or lockfile changed.
The maintainer explicitly chose experiment-local declarations over deciding
the public representation of `ID3D12Device` at this stage.

Environment: Windows 10.0.26200.0, Rust 1.98.1, x86_64-pc-windows-msvc,
VS 2022 Build Tools 17.14.28. The helper uses installed Windows SDK headers
and links `d3d12.lib` and `dxgi.lib`, without a Cargo build script or dependency.
Adapter: AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`.
Driver version reported by DXGI: `32.0.31041.1004` (query HRESULT 0).

The C++ helper selects the first enumerated non-software adapter on which
`D3D12CreateDevice` succeeds at feature level 12_0. There is no WARP fallback.
It retains one device reference until Rust explicitly requests release after
successful context destruction. No device reference-count experiment is performed.

## Independent local source checks

Authoritative ABI inputs are the locally supplied SDK v2.3.0 files under
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/`:

- `upscalers/include/ffx_upscale.h`: root descriptor, nullable message callback,
  dimensions, flags, version descriptor and `FFX_UPSCALER_VERSION` (4.1.1,
  encoded as `0x01001001`).
- `api/include/ffx_api.h`: header, common entry points, callback signature,
  provider query descriptor/tag, null system allocator and descriptor-pointer
  lifetime contract.
- `api/include/ffx_api_types.h`: two unsigned 32-bit dimension fields.
- `api/include/dx12/ffx_api_dx12.h`: backend tag 2 and `ID3D12Device*` field.
- `docs/techniques/super-resolution-ml.md`, New APIs: the upscaler version
  descriptor is required and linked from the root's `header.pNext`. This refines
  the older generic creation example that omits it.
- `backend/dx12/ffx_backends_dx12.cpp`, `GetDevice`: scans the chain for the
  backend/device. The probe uses root -> version -> backend -> null, following
  the effect-specific placement and providing the backend in the same chain.
  This run does not claim every possible node ordering is valid.
- `api/internal/ffx_api.cpp`: create/destroy provider delegation.
  `api/internal/ffx_query_fallback.cpp`: existing-context provider identity query.
  These explain why identifying the actual provider matters; open source is not
  assumed to characterize every signed/driver implementation.

The complete descriptor chain stays in stable local storage until after destroy;
the device, helper and loader likewise remain alive. Message callback and both
host allocator arguments are null. No provider override is used. Keeping all
dependencies alive avoids deciding the unresolved minimum lifetimes.

The experiment-local ABI adds `FfxApiDimensions2D`, `ffxApiMessage`, an opaque
device pointee, `ffxCreateContextDescUpscale`,
`ffxCreateContextDescUpscaleVersion`, `ffxCreateBackendDX12Desc`, and
`ffxQueryGetProviderVersion`. The latter is necessary to identify the observed
provider rather than assume it from the SDK release number.
Rust compile-time assertions and the paired C++ probe check every descriptor
field type and offset, size/alignment, callback signature, tags and version.
Windows `wchar_t` is verified as unsigned 16-bit; the Rust callback uses `u16`.
The opaque device is only passed as a pointer, never instantiated/dereferenced
by Rust. The existing paired common ABI and five entry-point signatures are
also rechecked by the native probe.

Local input SHA-256 fingerprints (not independent upstream authentication):

| File | SHA-256 |
|---|---|
| `ffx_api.h` | `91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4` |
| `ffx_api_types.h` | `B54FBD96A0EED82662A49C00E28E5368AB69959F9856DAE5C12FED109D123D66` |
| `ffx_api_dx12.h` | `2082D6C2914E9C2FA9FAE6247D35F64C643CCB703311C83C8DBA9A35356BC711` |
| `ffx_upscale.h` | `F13ABCDD4389E22AA50562A90255BEC9511E004722C7D2BF6F959D92FD78355C` |
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

## Reproduction

Run at repository root from an x64 VS 2022 developer shell:

```powershell
powershell -NoProfile -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1
```

The tested invocation from an ordinary PowerShell session initialized that shell:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1'
```

The checked-in runner contains the exact commands: MSVC `/std:c++17 /W4 /WX`
compile-only ABI check; `/MT /LD` helper build; offline locked Cargo test build;
staging under `target/context-lifecycle`; then the dedicated test executable
with `--ignored --exact creates_and_destroys_upscaler_context --nocapture
--test-threads=1`. It temporarily sets `FSR_SDK_TEST_DLL` and
`FSR_SDK_TEST_DEVICE_DLL` to absolute staged paths and restores their prior values.
Only run with the trusted local inputs. No downloads, redistributions, permanent
installation or DLL search-path mutation are involved.

The only AMD DLLs staged were the loader and upscaler DLLs. They were sufficient
alongside the installed OS/driver on this machine. No absence test was performed;
this does not prove either DLL is universally necessary or identify whether
the selected implementation was supplied by the bundled DLL or driver.

## Observations

First native run passed without a failed create/destroy attempt:

| Observation | Value |
|---|---|
| Device creation HRESULT | `0x00000000` |
| Device pointer | `<non-null-device>` |
| Create chain | upscale -> upscale version -> DX12 -> null |
| Inputs | flags 0; render 1280x720; upscale 1920x1080 |
| Handle before create | `0x0` |
| Create return | `0` (`FFX_API_RETURN_OK`) |
| Handle after create | `<context-1>` |
| Provider query return | `0` |
| Provider ID / name | `0xf5a5ca1e01001001` / `4.1.1` |
| Handle immediately before destroy | `<context-1>` |
| Destroy return | `0` (`FFX_API_RETURN_OK`) |
| Handle after destroy | `<context-1>` |

**Verified runtime observation:** successful destroy did not null the handle.
The pointer was printed numerically only, never dereferenced or passed back
after destruction. Process-specific addresses are normalized here; repeated `<context-1>` values
preserve the observed unchanged handle bits.
The device's helper-owned reference was then released and library owners dropped.

Validation commands, all successful:

```powershell
cargo fmt --all -- --check
cargo build --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
cargo doc --workspace --no-deps --locked --offline
```

Ordinary tests passed five ABI tests, four loader fixture tests and two
compile-fail doctests; all three AMD runtime tests were ignored as intended.
The new experiment ABI assertions passed during compilation. The separate native
ABI compilation, helper build, and opt-in lifecycle test all exited successfully.
The toolchain emitted its existing home-path canonicalization warning; no
compilation, lint, ABI or runtime failure occurred. An earlier optional CIM GPU
inventory read was denied; the probe obtained adapter/driver metadata via DXGI.

## Baseline delta and limits

The project now has runtime evidence of one successful provider-identified
context lifecycle, beyond the prior count-only query. All new native declarations
remain experiment code, and this does not accept a public ownership architecture.
The observed non-nulling agrees with the earlier open-provider source findings
and contradicts relying on the generic documentation's null-after-destroy claim
for this tested path.

No dispatch was submitted or image output inspected. Context creation may
allocate SDK-internal GPU resources; that is distinct from application resource
management or executed upscaling. This pass does not establish general hardware
compatibility, allocation correctness, leak freedom, FSR image quality or speed.

Minimum chain-node lifetimes, device COM ownership, callback lifetimes,
failed-create/destroy postconditions, threading, general DLL unload contracts
and behavior of other providers remain unresolved. The probe exits its isolated
process on unexpected native creation/destruction failure without speculative
cleanup; those paths were not deliberately exercised and are not a wrapper policy.
A successful create is destroyed once even if provider identification fails.
No null/double-destroy, allocator injection or shortened-lifetime tests were run.
