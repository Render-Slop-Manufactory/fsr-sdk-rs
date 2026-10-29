# Experiment: D013 serialized native Windows/DX12 transfers

- Investigation: 2026-09-30, Europe/Berlin (CEST, UTC+02:00).
- Question: Does the actual AMD runtime tolerate serialized load/create,
  dispatch, context cleanup and final wrapper-owned loader release on different
  threads, with GPU completion before each transfer?
- Success oracle: every scenario and equivalent same-thread control completes
  in its isolated child; native create/query/dispatch and explicit destroy return
  OK; all output pixels are finite, more than half differ from the sentinel;
  zero D3D12 error/corruption messages and `S_OK` device status; retention survives
  early runtime/first-sibling cleanup, expires at final cleanup, and the loader
  is no longer observed loaded. Native failure, oracle failure or timeout fails
  the case. Provider-DLL residency is reported separately from loader release.
- Exclusions: public movable API, ADR acceptance, parallel native calls, transfer
  with outstanding GPU work, GBV, callback/custom-allocator profiles, deliberate
  concurrency violations, image-quality certification, downloads/installations,
  commits, pushes and publication.

## Source state and baseline

Post-publication `0.1.0` work. Initial worktree inspection was clean at HEAD
`0495f347cf06c81e1082fed83e08350245c2c5f9`, base tree
`69bd4a0997dc7988375953bf116ac8c303eb207f`. The executed state includes the
six uncommitted executable-source changes below. The retained
[source manifest](2026-09-30-exp-d013-native-thread-transfer/source-inputs.json)
contains all 76 crate/workspace source inputs, including the new probe; its
SHA-256 is `419175CB2F8457600E4F577AA5FE1D4862E330C4AF72FD0F7E5BBD4F03D26F96`.
All fingerprints matched after the spike and regressions; the three runners'
source manifests were identical
([verification](2026-09-30-exp-d013-native-thread-transfer/source-verification.txt)).
The Git revision alone is not the executed source state.

During this task, independent documentation changes appeared in `ROADMAP.md`,
`docs/DECISIONS.md` and a new `docs/adr/D014.md`. They were preserved and excluded
from this experiment's executable delta. No D013 ADR edit was made.

| Changed file | Executed raw-byte SHA-256 |
|---|---|
| `crates/fsr-sdk/src/upscaler/dispatch.rs` | `A9D85432B54980B337C3EE1EE0D893877BA2AF4030F96205C0578C5C7FBD0E49` |
| `crates/fsr-sdk/src/upscaler/dispatch/resources.rs` | `A0AA3A75E4A767071D4CA35EDFB0CF0AD1B0F3CEE4DB8C4D2CEA1C4A77D51E87` |
| `crates/fsr-sdk/tests/native/d013.rs` | `01A1EE498613926B63BAEC31CE525C642C9B447B95EA3415E37F3C8C4599C512` |
| `crates/fsr-sdk/tests/native/dispatch_readback.rs` | `9788BA4215D843E57E8D84249AE91E66F86B5BDE4AAA9DA961FF8694854DFC3F` |
| `crates/fsr-sdk/tests/native/support.rs` | `2211A97EB7CF259256B6BD8D1DE2C477E463E242D4025DD6E6D02C5E0077C5EC` |
| `crates/fsr-sdk/tests/native/run.ps1` | `CDA246B95573D1DBAE131B2485009F4DCEB61F9E6CA3B670A1BCC19288004395` |

Earlier evidence read: [D013](../../adr/D013.md),
[compile/host probes](2026-09-29-exp-d013-movable-compile-host.md),
[Windows fixtures](2026-09-29-exp-d013-windows-fixtures.md), and
[final Windows release verification](2026-09-28-exp-final-windows-release-verification.md).
The old `src/upscaler/m6.rs` / `tests/m6_gpu.cpp` entry points have moved to
`tests/native/dispatch_readback.rs` / `tests/native/gpu.cpp`; `run-m6.ps1`
is the compatibility entry point. Current analytic-image code also incorporates
the later [dispatch-semantics work](2026-09-30-exp-dispatch-semantics.md).

## Private production extraction (separate from probe code)

`dispatch/resources.rs` extracts the existing inspection body into private
`inspect_dispatch(device, options, dispatch)`. The existing `Upscaler` method
forwards its retained device/options to it. Device/COM identity checks, resource
conversion, extents, descriptor values and error behaviour are unchanged; most
diff lines are indentation. `dispatch.rs` re-exports this function only under
`cfg(test)` for the probe. No new production retention choice, public function,
type, feature, trait implementation or dispatch policy is added.

The real runtime, pinned `UpscaleCreateState` builder and terminal
`NativeContextOwner` are reused unchanged. There is no separate crate or copied
runtime/context implementation. Production still uses Rc and remains
`!Send + !Sync`.

## Test-only equipment and audit

`tests/native/d013.rs` adds six ignored tests and one concrete `SerialFamily`
wrapper around the real Arc-retained owners and their entire native equipment.
Its sole manual Send override applies to this exact test bundle. A compile
assertion requires `Send` and rejects `Sync`; neither an Rc-owned type nor the
generic owner receives Send. The earlier local-type/owner trait checks also
pass on the real Windows target.

The audit is next to the override: creation descriptors stay in their original
pinned boxes; runtime retention is Arc; each device/list/resource owns COM
references; raw harness pointers refer to stable C++ heap allocations. The
helper library outlives its function pointers and every harness. There are no
callbacks, custom allocators or borrowed creation dependencies. All frame
equipment is retained through final context cleanup and loader release.
Returned native dispatch or finish failure exits the child without submitting
a failed recording or performing cleanup with uncertain GPU completion. The
probe intentionally does not implement a second public-style poison policy.

Each case creates fresh named workers, immediately joins them, then transfers
the whole exclusive bundle to the next worker. The coordinator performs no
native operation between workers. Join establishes happens-before; it is not
the GPU fence. Every submitted frame calls the existing `m6_finish`, which
closes/submits/signals, waits up to 60,000 ms for completion and maps readback,
before any further ownership transfer or cleanup. Workers A/B/C have distinct
Rust thread IDs in the retained logs. Their lifetimes do not overlap.

The runner gains `-Mode d013`, rejects `-GpuValidation` in that mode and uses
its established one-test-per-child, 180,000 ms watchdog. It captures fingerprints
for all crate inputs, and its tracked-source patch scope now includes the shared
private extraction. Test-only provider reporting is factored to accept either
real owner retention family. The C++ helper and its fence/readback behaviour are
unchanged by this task. Existing lifecycle and dispatch modes remain available.

## Actual environment and inputs

- Windows 11 Pro 10.0.26300, build 26300; native host/target
  `x86_64-pc-windows-msvc`.
- Rust 1.98.1 (`48a229cea`, 2026-09-01), LLVM 22.1.8; locked
  `libloading` 0.9.0 and `windows` 0.62.2.
- MSVC tools 14.44.35207, compiler 19.44.35224.0, Windows SDK 10.0.22621.0;
  PowerShell 7.6.6; x64 VS developer environment.
- Actual selected device: AMD Radeon RX 9060 XT, hardware adapter vendor
  `0x1002`, device `0x7590`, feature level 12_0. Driver interface query returned
  `S_OK` and version `32.0.31041.1004`, matching Windows inventory.
- SDK v2.3.0, pinned upstream commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`. Existing local SDK used; four
  selected headers were hash-verified after CRLF-to-LF normalization by
  `tools/check-native-abi.ps1`. All eight local API/upscaler header raw-byte
  hashes are in the retained [native inputs](2026-09-30-exp-d013-native-thread-transfer/native-inputs.json).
- Observed actual provider query in every context: ID
  `0xf5a5ca1e01001001`, name/version `4.1.1`, query code `0x00000000`.
  This identifies the selected provider as reported by the actual runtime;
  it does not fingerprint all participating driver/provider internals.
- Ordinary D3D12 debug layer enabled before each child's device creation,
  with `S_OK` required. GBV and the no-dispatch control variable were unset.
  Both AMD DLLs were staged beside the child executable from local `signedbin`,
  with Authenticode **Valid** (serialized status `0`). No DLL search-path change.

| D013 attempt-2 binary | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| Locally built `m6_gpu.dll` | `F5F9F076CF266608DFED9C1F71CA22ABB326F683EEA2C7B24F6D0E699B06C214` |
| Rust `m6.exe` | `DAF33039073312C67053BBE41E527FDCF0ED3FC27EDF5D87AAFDF440162009B9` |

Each regression rebuild has its own retained `lifecycle-native-inputs.json`
or `dispatch-native-inputs.json`; source fingerprints match the D013 run.
[Environment capture](2026-09-30-exp-d013-native-thread-transfer/native.environment.txt)
retains tool, OS, GPU, signature and DLL data. No AMD binaries are retained in
the research record.

## Commands and initial blocked attempt

From the repository root, initial formatting and compilation:

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

The first runner invocation used the D013 command below with stage
`target/d013-native-20260930-attempt-1` inside the sandbox. Ordinary checks,
all three ABI compilations, helper build and Rust test-executable build passed.
It stopped at `Get-CimInstance Win32_OperatingSystem` in `native/run.ps1:73`
with **access denied**, outer exit 1. No native AMD/GPU child had started;
this is an environment failure, not a passed native attempt or AMD failure.
After required permission, attempt 2 and both native regression invocations
ran outside the sandbox. Exact outer commands:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\native\run.ps1 -Mode d013 -Native -Stage target/d013-native-20260930-attempt-2'
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\native\run.ps1 -Mode lifecycle -Native -Stage target/d013-regression-lifecycle-20260930'
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6.ps1 -Native -Stage target/d013-regression-dispatch-20260930'
cargo check --workspace --all-targets --no-default-features --locked --offline
```

All four commands exited **0**. The three native runners execute these checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
./tools/check-native-abi.ps1 -SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX -Stage <stage>/abi
cl /nologo /std:c++17 /W4 /WX /MT /LD /I"external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/dx12" crates/fsr-sdk/tests/native/gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo"<stage>/" /link /OUT:"<stage>/m6_gpu.dll" /IMPLIB:"<stage>/m6_gpu.lib" d3d12.lib dxgi.lib
cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json
```

`<stage>` is exactly the corresponding outer command's stage. ABI verification
compiles common, context-creation and dispatch probes with `/std:c++17 /W4 /WX`.
Every ordinary check passed: 33 wrapper unit tests (including all eight D013
fixtures), one consumer integration test, 12 sys tests and four doctests.
Twelve wrapper and six sys opt-in tests were ignored in ordinary workspace runs;
the required native tests were then executed explicitly in isolated children.
The feature-disabled all-targets check also passed. These counts distinguish
compilation/fixtures from the actual native results below.

Child command form, executed once for each listed exact test:

```text
<stage>/m6.exe --ignored --exact <test> --nocapture --test-threads=1
```

For D013, `<test>` is `upscaler::m6::d013::` plus each table suffix below.
Lifecycle regression tests are `upscaler::tests::native_lifecycle` and
`upscaler::tests::native_lifecycle_drop`. Dispatch regression tests are
`upscaler::m6::native_dispatch_readback`, `upscaler::m6::native_temporal_smoke`
and `upscaler::m6::native_semantic_comparisons`.

## Executed scenarios and equal controls

| Scenario | Same-thread suffix / result | Transfer suffix / result | Actual operations on A / B / C |
|---|---|---|---|
| 1 | `control_abc`: passed | `transfer_abc`: passed | A loads/creates; B dispatches and fences frame 0; C explicitly destroys and releases final runtime Arc/loader |
| 2 | `control_three_frames`: passed | `transfer_three_frames`: passed | A loads/creates and fences frame 0; B fences frame 1; C fences frame 2, destroys and releases |
| 3 | `control_siblings`: passed | `transfer_siblings`: passed | A loads/creates two siblings and drops external runtime; B fences sibling 0 and explicitly destroys it; C fences sibling 1, normally Drops it and performs final Arc/loader release |

Controls use the same Arc owner, resources, parameters, retention checks and
cleanup strategy, all on one `d013-control` worker. Only transfer placement
differs. Scenario 3 retains both siblings at first use, tests library retention
after the external runtime and first sibling are gone, and releases the final
sibling by ordinary Drop. Drop's native return code is not exposed; weak-reference
expiry and loader disappearance exclude D005's retained-runtime failure outcome
in this successful path, without claiming a directly observed Drop return code.

Attempt 2 ran six children from 01:14:49.949 to 01:14:53.041 CEST.
Every child had `timeout=False exit=0`; duration about 0.42–0.61 seconds.
All 12 D013 dispatches/readbacks passed. No fence or child timeout occurred.
Each create, query, dispatch and explicit destroy returned `0x00000000`.
Output recording, Close, Signal, fence completion and mapping reported `S_OK`.

Profile: render 320×180, output 640×360, linear RGBA16F colour/output, R32F depth
0.5, RG16F motion, explicit 1×1 R32F exposure 1.0, distinct UAV output initially
filled with half-float sentinel `0x7bff`. Flags zero, sharpening/masks disabled,
frame time 16.667 ms, near/far 0.1/100, vertical FOV π/3, metres scale and
pre-exposure 1.0. A fresh list/allocator/resource/queue set is used per frame,
then retained through cleanup; queues are ordered by completed CPU fence waits.

| Frame | Translation | Jitter pixels | Reset | Motion UV |
|---|---|---|---|---|
| 0 | 0 px | (0.25, -0.25) | true | (0, 0) |
| 1 | +1 px right | (-0.25, 0.25) | false | (-1/320, 0) |
| 2 | +2 px | (0.125, -0.125) | false | (-1/320, 0) |

Current helper image generation: `sx=x+0.5-jitter_x`,
`sy=y+0.5-jitter_y`, `u=sx-frame`, then
`RGBA=(0.1+0.75*u/320, 0.1+0.75*sy/180, 0.2, 1)` in half floats.
Motion is current-to-previous and excludes jitter; descriptor scale is (320,180).
Scenario 1 and each sibling use frame 0; scenario 2 uses frames 0/1/2 on the
same context. This records the current minus-jitter generator explicitly;
the older release record used a different generator.

| Frame | Finite / changed pixels in both control and transfer | Red min / max | Left / right red mean |
|---|---|---|---|
| 0 (all contexts) | 230,400 / 230,400 | 0.1005249 / 0.84765625 | 0.19358033 / 0.7556558 |
| 1 | 230,400 / 230,400 | 0.08795166 / 0.8461914 | 0.19121014 / 0.7527847 |
| 2 | 230,400 / 230,400 | 0.09069824 / 0.8432617 | 0.18860283 / 0.7495062 |

Every D013 readback was 640×360 with zero debug errors/corruption and
`device_removed=0` (`S_OK`). The printed metrics match between equal controls
and transfers; no additional pixel-by-pixel equivalence oracle is claimed.
Device/debug checks also passed after each readback, first-sibling destroy,
final context/runtime cleanup and frame-resource cleanup.

Independent stdout/stderr/status files are retained for every child, for example
[ABC transfer log](2026-09-30-exp-d013-native-thread-transfer/d013-transfer-abc.stderr.txt),
[three-frame transfer log](2026-09-30-exp-d013-native-thread-transfer/d013-transfer-three-frames.stderr.txt)
and [sibling transfer log](2026-09-30-exp-d013-native-thread-transfer/d013-transfer-siblings.stderr.txt).

## Final release observation and its limit

**Verified for all six cases:** the runtime weak reference expired, and
`open_already_loaded` no longer found the specific loader DLL after final release.
The check briefly acquires/releases an observation reference while a module is
loaded; none is held across cleanup. In transfers, this release and observation
occur on C, after fenced work and context cleanup. The child subsequently exits
normally with all helper/device/resource cleanup completed.

**Unresolved provider termination:** the staged upscaler DLL remains observed
loaded (`provider_loaded=true`) after final wrapper release in both controls and
transfers, while the caller device is still retained. Thus these runs establish
destination-thread final release/unload of the wrapper-owned **loader**, not
destination-thread DLL termination/unmapping of the entire provider/dependency
closure. The source of the remaining native module reference was not investigated;
no forced FreeLibrary, refcount manipulation or provider unload was attempted.
This is a material limit on D013's broader final-DLL-unload question.

## Native regression results on the changed source

Both lifecycle cases passed after the spike (01:15:44.542–01:15:45.472 CEST):
explicit and ordinary Drop sibling cleanup, actual provider query and loader
retention/release through the existing public Rc route.

All three dispatch gates then passed:

- Gate A: public construction/dispatch, all five AMD resource-conversion parity
  checks, one fenced finite/changed gradient readback, explicit destroy and
  post-cleanup debug/device checks.
- Gate B: three moving frames on the same public context, all 230,400 pixels
  finite and changed each frame, normal Drop and post-cleanup checks.
- Gate C: existing motion/jitter/reset semantic comparisons. Motion delta
  0.24122791, jitter delta 0.03519135, reset delta 0, history delta 0.01101808;
  analytic red MAE 0.01334815 (correct), 0.25143567 (reversed motion),
  0.04247920 (reversed jitter). Its native checkpoints passed.

Every regression child exited 0 without timeout; all three outer native runners
exited 0. Logs/status and check results are retained with `lifecycle-` and
`dispatch-` prefixes. Recording-ahead, returned-native-error/failure-injection
experiments and GBV were not rerun; they are separate research probes, not these
ordinary lifecycle/dispatch regression gates.

## Warnings, evidence hygiene and remaining bounds

Sandboxed launcher calls emitted the existing home-path canonicalization warning;
successful outside-sandbox runs did not. Git reported LF-to-CRLF notices. Neither
was a denied Rust lint or native failure. The initial metadata denial is the
only unsuccessful runner attempt. No dependency was downloaded or installed.

Retained evidence substitutes personal checkout prefixes with `<repo>` and
excludes transcript machine/user headers. No native raw pointer addresses were
printed; Rust thread IDs/names are retained to establish distinct workers.
Hashes, provider IDs, hardware IDs, HRESULTs and versions are unchanged.

**Bounded conclusion:** all three requested serialized transfer workloads and
their equal same-thread controls passed on this one identified configuration,
with full fencing and healthy debug/device checks. Shared-source native
regressions and all eight lifecycle fixtures pass. This does not establish
unrestricted AMD movability, arbitrary hardware/provider support, provider DLL
termination on the destination thread, safe concurrent use, image quality,
unfenced transfer, native error/unwind transfer paths, or partial-load cleanup
across threads. Miri/ThreadSanitizer were not run. D013 remains **proposed**;
there is no public movable family or accepted threading contract.

> Archive navigation added 2026-10-10: retired verification scripts are preserved in the
> [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Script links now point to exact archived source bytes;
> recorded commands, observations and fingerprints are unchanged. Historical commands require
> their original layout and compatible inputs; archived sources are not current entry points.

> Orchestration migration (2026-10-10): the shared native, ABI and consumer entry points are now frozen in the [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Current commands live in [verification](../../../tools/verification/README.md); command text and observations below remain historical.
