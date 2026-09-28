# Experiment: Caller input lifetime after a returned DX12 create error

- Investigation date: 2026-09-22.
- Question: Can a source-supported, non-UB request return a create error from
  the actual signed v2.3.0 runtime, and can the caller then withdraw its
  descriptor storage, device reference and FfxLibrary without an observed fault?
- Hypothesis under test: returned failure ends native use of these caller inputs
  on this specific path. Survival is a bounded observation, not a lifetime contract.
- Success condition: a separate successful create/destroy control, followed by a
  returned non-OK result and completed isolated observation variants.
- Negative conditions: no legitimate returned failure, crash/hang before return,
  failed protection/cleanup, or a fault after return. Preserve these separately.
- Exclusions: M4 implementation, NativeContextOwner, public API or dependency
  changes, accepting/rejecting D005, changing D006 or synthesis, dispatch,
  allocator failure injection, invalid pointers, mismatched descriptor tags,
  failed-handle operations, private-state corruption and double destruction.

## Setup

Repository revision: `cfc0ce8790e2619865e4931962df60f7ad94330d`.
`git status --short` was empty at start. There were no unrelated changes.
The [prior lifecycle](2026-09-20-exp-context-lifecycle.md),
[allocation failure](2026-09-20-exp-create-allocation-failure.md) and
[source lifecycle](2026-09-20-src-context-lifecycle.md) records supplied context;
candidate selection was checked against the actual local SDK files.

Environment observed here: Windows `10.0.26200.0`; Rust `1.98.1`
(`48a229ceaefd4985c50990b14116b6d856af0985`, 2026-09-01), LLVM `22.1.8`,
host `x86_64-pc-windows-msvc`; VS 2022 Build Tools developer shell `17.14.28`.
The runner uses that shell's x64 MSVC compiler, installed Windows headers,
`d3d12.lib` and `dxgi.lib`. No new installation or download was needed.

Every child created a hardware DX12 device through the existing native helper:
AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, feature level `12_0`,
driver `32.0.31041.1004`, device and driver-query HRESULTs `0`.
The helper chooses the first qualifying hardware adapter, without WARP.
No D3D12 debug layer or information queue was enabled in this setup. No debug
layer diagnostics were collected; absence of stderr diagnostics is not a clean
debug-layer report.

The trusted local SDK root was `external/FidelityFX-SDK/v2.3.0/`. The two
`signedbin` DLLs were staged beside the child executable. Both original DLLs
returned Authenticode status `Valid`, signer `CN=Advanced Micro Devices,
O=Advanced Micro Devices, S=California, C=US`. Hashes below fingerprint the actual
local inputs; this does not independently authenticate the source tree against
an upstream commit or establish binary/source equivalence.

Paths below are relative to `Kits/FidelityFX/` in that SDK root. SHA-256:

| Input | SHA-256 |
|---|---|
| `signedbin/amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `signedbin/amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| `api/include/ffx_api.h` | `91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4` |
| `api/include/ffx_api_types.h` | `B54FBD96A0EED82662A49C00E28E5368AB69959F9856DAE5C12FED109D123D66` |
| `api/include/dx12/ffx_api_dx12.h` | `2082D6C2914E9C2FA9FAE6247D35F64C643CCB703311C83C8DBA9A35356BC711` |
| `upscalers/include/ffx_upscale.h` | `F13ABCDD4389E22AA50562A90255BEC9511E004722C7D2BF6F959D92FD78355C` |
| `api/internal/ffx_api.cpp` | `41B91C3F43975D8B8411E9CC1AC81CC5E303EA4AEBF35D1255EC26167550CFCF` |
| `api/internal/ffx_backends.h` | `CF6815D064EA0D5755D764B52A2C304E65314BD8F4982414F88A65E38F2F67EF` |
| `backend/dx12/ffx_backends_dx12.cpp` | `2A3FF31DF749CC4C78F9BFACDF72412BFFFF9B3584D1302A7845F803DF22D096` |
| `backend/dx12/ffx_dx12.cpp` | `DCD6DF270B04CE153B5F6EB2AF584089E374B1E1D274DA528160D9E6D336C440` |
| `upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp` | `1D8A8B58BAD36DB252323C22789B50EAD725AEFA5DF613148E99624121CA714D` |

## Source justification and candidate boundary

**Verified local source:** `backend/dx12/ffx_backends_dx12.cpp:42-87`,
`CreateBackend`, explicitly checks `backendFound` when it encounters a DX12
backend node. A second such node returns `FFX_API_RETURN_ERROR` (1) at lines
50-53, before reading that second node's device. The first backend sets the
flag, checks a non-null device, converts the device, allocates scratch storage
and calls `ffxGetInterfaceDX12` at lines 55-62. The first interface call must
succeed to reach the duplicate check in this open implementation.

`api/internal/ffx_backends.h:34-40`, `MustCreateBackend`, propagates that result.
The open `ffxProvider_FSR3Upscale::CreateContext` at
`upscalers/fsr3/internal/ffx_provider_fsr3upscale.cpp:89-106` allocates its internal
context before calling this helper, and propagates failure before creating the
core upscaler or publishing the output handle. `api/internal/ffx_api.cpp:43-69`
clears the output, obtains the device/provider and delegates creation.
`ffxGetInterfaceDX12` at `backend/dx12/ffx_dx12.cpp:266-343` initializes function
pointers, scratch metadata and the device field. This is backend-interface setup,
not proof of GPU dispatch or of core backend-context creation/AddRef.

Selected request:

```text
Upscale -> UpscaleVersion -> BackendDX12 -> BackendDX12 -> null
```

Both backend payloads are genuine correctly tagged descriptors containing the
same live helper-created device pointer. The chain is finite and acyclic;
neither output pointer nor root is null. Flags remain 0, render size 1280x720,
upscale size 1920x1080, API version `FFX_UPSCALER_VERSION = 0x01001001`.
Host allocator, effect message callback and backend callback extension are absent.
This is a source-supported rejected configuration, not a valid successful
configuration or a tag/payload mismatch. No deliberate invalid native input
pointer is used to induce the create error.

Missing backend was considered: `MustCreateBackend` explicitly rejects its
absence, but then no device is supplied to provider selection or backend setup.
The duplicate check offers a more useful dependency path with one additional
existing descriptor and no new ABI. It was therefore selected instead of testing
both. Missing/unsupported API versions were not guessed at; custom allocator
failure was excluded by its prior pre-return crashes. No other candidate sweep
was performed after this one returned a useful error.

**Unresolved binary depth:** these open source facts justify the trigger, but
are not a trace of the signed 4.1.1 implementation. A return of 1 does not prove
that the signed provider executed `MustCreateBackend` or initialized its first
backend identically. It may reject duplicates earlier. Successful controls
identify 4.1.1; failed outputs are never queried, so failure-provider identity and
the exact supplying module (bundled DLL versus driver) remain unverified.

## Probe changes and isolation

Changes are limited to the existing
[Rust probe](../../../crates/fsr-sdk-sys/tests/context_lifecycle.rs),
[runner](../../../crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1),
[native helper](../../../crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp),
new [descriptor storage helper](../../../crates/fsr-sdk-sys/tests/context_lifecycle/input_lifetime.rs),
this record and its [event log](2026-09-22-exp-create-error-input-lifetime/events.txt).
No production source, public API, manifest, dependency, decision or synthesis changed.

First, before lifetime instrumentation, the stack-backed duplicate chain was
run with all dependencies retained until process exit. Only after it returned
non-OK was page-backed storage added. The existing three-node success control
ran separately before discovery and again before the lifetime sweep.

For A/B/C, the four descriptor objects occupy 120 bytes in one dedicated
VirtualAlloc reservation/commit, rounded by Windows to pages. The probe writes
the objects then rebuilds all links at their final addresses. No Rust reference
into the allocation survives, and no unrelated heap/stack data shares its pages.
The aggregate is not a new AMD ABI: each member uses the unchanged paired Rust
and C++ descriptor checks. C++ helper exports use installed Windows declarations;
their signatures have compile-time checks.

| Mode | After returned non-OK | End of observation |
|---|---|---|
| A: `retain` | Keep descriptor allocation, caller device reference and FfxLibrary alive; no destroy | Exit process with retained owners |
| B: `protect` | VirtualProtect dedicated descriptor pages to PAGE_NOACCESS; verify via VirtualQuery; retain device/runtime | Exit with protection and dependencies retained |
| C: `cleanup` | Drop descriptor allocation (VirtualFree), release the helper-created device reference once, drop FfxLibrary; no destroy | Remain alive, then exit |

The device release uses the existing native helper's ordinary `Release` call,
not fake COM or manual invalidation. The helper library itself remains loaded
through the observation window; it does not link to AMD DLLs. In C, all supplied
effect-specific descriptor storage is actually released before the device and
FfxLibrary. No native context owner is constructed. No failed output handle is
destroyed, dereferenced or used for another operation, regardless of its bits.

Observation is a requested 2000 ms sleep measured with Instant after the selected
action completes. Two seconds targets obvious delayed activity; it has no
contractual significance. The inspected rejection path supplies no reason to
expect asynchronous work. No background GPU work was manufactured.

Each meaningful case uses a fresh child, inherited environment, redirected stdout
and stderr, no visible window, and a 15-second timeout in the final runner. It
requires successful control and retained-error baseline before B/C. It continues
from a crashing B to C; runner success means orchestration completed, not child
or native success. Signed, unsigned and hexadecimal exits are saved. Environment
inputs are restored in finally; no DLL-search API or permanent setting changes.
Discovery used a 60-second timeout and the same process isolation mechanism.

## Reproduction and checks

From the repository root with the trusted inputs, cached Cargo dependencies and
an x64 VS developer shell:

```powershell
powershell -NoProfile -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -InputLifetime
```

Actual invocation from ordinary PowerShell (also used initially without the last
switch for the successful pre-discovery control):

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -InputLifetime'
```

The checked-in runner contains the exact native commands: paired ABI compilation
with `/std:c++17 /W4 /WX /c`, helper `/MT /LD` build, and
`cargo test -p fsr-sdk-sys --features dx12 --test context_lifecycle --locked --offline --no-run --message-format=json`.
It stages under `target/context-lifecycle`, supplies absolute `FSR_SDK_TEST_DLL`
and `FSR_SDK_TEST_DEVICE_DLL`, and launches
`--ignored --exact probes_create_error_input_lifetime --nocapture --test-threads=1`
with `FSR_SDK_INPUT_LIFETIME=retain`, `protect`, or `cleanup`.
The control uses `creates_and_destroys_upscaler_context` with that variable unset.

The initial discovery used the staged executable with the same test arguments and
`FSR_SDK_INPUT_LIFETIME=discover`, through ProcessStartInfo with redirected output,
`UseShellExecute=false`, `CreateNoWindow=true`, asynchronous stream reads and
`WaitForExit(60000)` followed by Kill on timeout. That initial stack-only version
is described above; the final discovery mode uses dedicated pages and exits
immediately after a returned failure. The retained variant reproduces the same
trigger and adds the bounded observation interval.

Additional commands executed successfully:

```powershell
rustc -Vv
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
git diff --check
```

Native ABI compilation and helper build passed. Ordinary tests passed five ABI
tests, four loader fixtures and two compile-fail doctests; five AMD runtime tests
were ignored as intended. Existing toolchain home-path canonicalization and Git
line-ending warnings appeared; no build, lint or test failure occurred. Native
execution here covers loading and creation/destruction, not GPU dispatch.

## Observations

**Verified runtime observations:** every loader acquisition succeeded. Both
successful controls returned create 0 and destroy 0; provider query returned 0,
ID `0xf5a5ca1e01001001`, name `4.1.1`. No provider ID was hardcoded or overridden.
Process-specific non-null addresses are normalized to semantic labels.
The sweep control handle was `<sweep-context>` both after create and destroy.
The pre-discovery control handle was `<pre-discovery-context>` and likewise remained unchanged.

| Child | Create result | Immediate output | Interval (ms) | Signed / unsigned / hex exit | Timeout |
|---|---:|---|---:|---|---|
| Initial discovery | 1 (ERROR) | `0x0` | None | 0 / 0 / `0x00000000` | No |
| Sweep control | 0 (OK), destroy 0 | `<sweep-context>` | Not applicable | 0 / 0 / `0x00000000` | No |
| A retain | 1 (ERROR) | `0x0` | 2000 | 0 / 0 / `0x00000000` | No |
| B protect | 1 (ERROR) | `0x0` | 2000 | 0 / 0 / `0x00000000` | No |
| C cleanup | 1 (ERROR) | `0x0` | 2000 | 0 / 0 / `0x00000000` | No |

All input handles started null. No callback was supplied, so there were no
callback observations or host-allocation accounting. In B, PAGE_NOACCESS was
successfully applied and verified; the process survived the window. In C,
VirtualFree succeeded, Release returned, FfxLibrary drop returned, and the
observation completed. No crash, timeout, loader error or stderr diagnostic was
observed. No exception debugger/first-chance handler was attached: internally
handled exceptions are outside the observation. Survival detects no observed
fault; it does not rule out internally handled access attempts.

The log preserves stdout, stderr and per-child status. Its early generic
`Create chain` line incorrectly describes the ordinary three-node chain in failure
children; the following `Candidate` line correctly identifies the actual four-node
chain. This logging-only issue was corrected after the sweep, without repeating
native execution. No runtime behavior changed. The discovery log preserves its
earlier retained-dependency exit text. There were no unsuccessful native outcomes
other than the intended returned errors, and no skipped crashing candidate.

## Baseline delta and limits

1. **Legitimate returned failure found:** yes. Correctly tagged duplicate DX12
   backend descriptors with a live device are explicitly rejected by the local
   source, and the actual signed runtime returned non-OK normally.
2. **Depth:** the open FSR3 path reaches internal allocation and first backend
   interface setup before rejecting the second node. Actual signed-provider
   internal depth is unresolved; the return code alone cannot establish it.
3. **Return/output:** 1 (`FFX_API_RETURN_ERROR`), null immediately after return
   in discovery and all three lifetime variants.
4. **Protected-state access:** no post-return fault was observed during B's
   verified PAGE_NOACCESS window. This is not proof of no access forever, nor
   coverage of internally handled exceptions or memory outside those pages.
5. **Cleanup:** releasing descriptor storage, the caller device reference and
   FfxLibrary caused no observable failure during C's cleanup or two-second wait.
6. **Adds to D005 evidence:** unlike allocation injection, a real create call now
   returns an error and the proposed caller cleanup order has actually executed.
   This reduces the runtime evidence gap for one rejected configuration only.
7. **Does not prove:** universal failure rollback, leak freedom, later/deeper
   failures, callback lifetime, other providers/devices/builds, threading, or a
   provider-independent safe constructor contract. Dropping the caller reference
   neither proves COM destruction nor proves absence/presence of a native-owned
   reference. FfxLibrary drop does not prove provider DLL unload: module reference
   counts/internal ownership may outlive it. Module state was not measured.
8. **Contradiction:** no observation contradicts proposed ordinary RAII for this
   tested request. Absence of contradiction does not accept D005 or resolve its
   native safety prerequisite.
9. **Next step:** this spike has reached the useful limit of this trigger. No
   staged cleanup matrix is justified because C did not fail. More identical
   waits cannot supply a missing contract. A later targeted experiment would be
   useful only with an independently justified deeper returned-failure path or
   provider-specific source/contract evidence; none is authorized or implemented
   by this record.

**Inference:** the outcomes are consistent with rejection leaving no continuing
use of caller dependencies on this run, but also with an earlier signed-runtime
validation path than the inspected open backend. This ambiguity is material.
The result is evidence for a later D005 review, not a decision about that policy.
