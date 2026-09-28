# Final Windows release verification — 2026-09-28

## Scope and candidate

The authorized task verifies the current `0.1.0` candidate through two separate
Windows/DX12 child processes. It does not publish, tag, change versions, or
change D005/D007/D008/D011/D012.

Pre-harness commit: `54e2c3f102ca9099d2bcc09fdddbbf75f716f5c4`.
Pre-harness Git tree: `d9f43893a5cea9b50dd79549c2ce8dc87fc00d8c`.
The worktree was clean before this task; no unrelated changes were present.

Executed state: that baseline plus the uncommitted, three-file
[candidate patch](2026-09-28-exp-final-windows-release-verification/candidate.patch).
Patch SHA-256: `054D7AEDF8A5CDC99B00417C838CEF5849F0DF7E1B6E3DCFDEC1FB6474164CE7`.
The [source manifest](2026-09-28-exp-final-windows-release-verification/source-inputs.json)
fingerprints 57 tracked files under the crates and the workspace manifest/lockfile;
its SHA-256 is `09C8A2D264BDF0C427385A0BC6F99998EA409CB5957841FBA3999B4D20DB7B1C`.
All 57 fingerprints still matched after execution. The
[source-state capture](2026-09-28-exp-final-windows-release-verification/source-state.txt)
was taken at 18:51:31 CEST. The only untracked file then was this in-progress
record. Documentation and sanitized evidence were finalized afterward; they
are not executable changes. No new commit/tree was created, and the baseline
Git tree by itself is not the executed harness state.

Before Gate B implementation, at 2026-09-28 18:43 CEST (UTC+02:00,
Europe/Berlin), Gate B was classified **required — bounded harness extension**.
Existing per-frame resources, upload rewriting, submission, fences and readback
can support the exact three-frame scenario without a renderer or public API
change. Gate B is not deferred. A later failure cannot become a scope deferral.

## Test-harness changes

Production-code changes: **none**. Public APIs, descriptors, dispatch semantics,
runtime acquisition and ownership are unchanged. All Rust additions are in an
existing `#[cfg(test)]` module.

Test/verification-harness changes:

- `crates/fsr-sdk/src/upscaler/m6.rs`: keep Gate A's public route, resource parity,
  sentinel/gradient assertions and explicit destruction. Share provider/error
  reporting; verify the actual selected AMD adapter and post-destroy status.
  Add `native_temporal_smoke`: one runtime/device/upscaler, three fresh resource
  sets, sequential dispatch/submission/fence/readback, and normal Drop with all
  external dependencies retained. Check device/debug state after Drop and after
  frame-resource cleanup.
- `crates/fsr-sdk/tests/m6_gpu.cpp`: rewrite only unsubmitted color/motion uploads
  for the specified analytic frames. Preserve Gate A's original image. Add an
  adapter-LUID AMD check, error/corruption message ID/text reporting, post-teardown
  device checks and stage HRESULT logging. Preserve 60-second fence waits and
  record output transition/copy before Close/submission.
- `crates/fsr-sdk/tests/run-m6.ps1`: record checks and source/native fingerprints,
  tool versions and AMD DLL signatures; clear/restore the inherited control
  variable; run both gates in separate 180-second children with independent
  stdout/stderr/status. An optional `-Stage` selects a fresh run directory while
  the existing default `target/m6a` layout remains available to the other runners.

Executed working-copy file SHA-256 values:

| File | SHA-256 |
|---|---|
| `crates/fsr-sdk/src/upscaler/m6.rs` | `88A5E062992364B974F912EF7FCD059263AF9138468DB0D8D227582EB6EAF647` |
| `crates/fsr-sdk/tests/m6_gpu.cpp` | `278776AB3B4930E1ABE6C76679F7450F392BE34ECF211A3ACFA4815E50FC030B` |
| `crates/fsr-sdk/tests/run-m6.ps1` | `59152E5716571300ADF0C0F14B03691F77C305592676936CB427E577CBAFE4E4` |

## Environment and pinned inputs

- Execution: 2026-09-28, Europe/Berlin, CEST (UTC+02:00). Windows reports
  `W. Europe Standard Time`; its base offset is +01:00, with DST active here.
  Native setup metadata was captured at 18:51:46 CEST.
- OS: Microsoft Windows 11 Pro, version 10.0.26200, build 26200.
- Rust: `rustc 1.98.1 (48a229cea 2026-09-01)`, LLVM 22.1.8;
  native host/target `x86_64-pc-windows-msvc` with `dx12`.
- Visual Studio 2022 Build Tools, x64 developer environment; MSVC tools
  14.44.35207, compiler file version 19.44.35224.0; Windows SDK 10.0.22621.0;
  PowerShell 7.6.6.
- Actually selected in **both** children: AMD Radeon RX 9060 XT,
  feature level 12_0, vendor `0x1002`, device `0x7590`, hardware adapter.
  Each device's adapter LUID was independently resolved and checked as AMD.
  The selected adapter's successful driver query returned 32.0.31041.1004,
  matching Windows inventory.
- FidelityFX SDK v2.3.0; upstream baseline
  `60f4ea81909200d8542eca14dccb2628b763a9a3`. Locally pinned header fingerprints
  are in the [native input manifest](2026-09-28-exp-final-windows-release-verification/native-inputs.json).
- Observed provider in both contexts: name/version `4.1.1`, ID
  `0xf5a5ca1e01001001`; provider query returned `0x00000000`.
- Ordinary D3D12 debug layer enabled before device creation in both children;
  enabling returned `S_OK`. GBV disabled; no GBV execution/investigation performed.

Both AMD DLLs were copied from the pinned SDK's `signedbin` to the isolated
test executable's directory. Their Authenticode status was **Valid** (the raw
environment capture serializes that enum as `0`). No DLL search configuration
was changed. SHA-256 values:

| Staged file | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |
| Test-only `m6_gpu.dll` | `63D4D9E73FECAA092B04BC9F29D88A0A4DB106F59B179498CABFE29F53184AA3` |
| Test executable `m6.exe` | `27B3B5B0FBE9AC0FDB52A7E480A26D39FACDABB429F260B03D4AB9A2B78DCCCA` |

The [environment capture](2026-09-28-exp-final-windows-release-verification/native.environment.txt)
preserves tool/OS/GPU/DLL data. Relevant child environment variables were:

```text
FSR_SDK_TEST_DLL = <repo>/target/final-windows-20260928-attempt-1/amd_fidelityfx_loader_dx12.dll
FSR_SDK_TEST_M6_DLL = <repo>/target/final-windows-20260928-attempt-1/m6_gpu.dll
FSR_SDK_TEST_GBV = unset
FSR_SDK_TEST_M6_CONTROL = unset
VSCMD_ARG_TGT_ARCH = x64
VCToolsVersion = 14.44.35207
WindowsSDKVersion = 10.0.22621.0\
```

The first two paths were absolute at execution; `<repo>` substitutes the personal
checkout prefix. The runner restored all four FSR test variables afterward.

## Exact commands

From the repository root, the exact outer invocation was:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6.ps1 -Native -Stage target/final-windows-20260928-attempt-1'
```

This used permission to access required Windows metadata and the native GPU
outside the sandbox. No dependencies were downloaded or installed. Preparation
ran `cargo fmt --all` once to format the harness and parsed the runner without
syntax errors. The established runner then executed all of these checks, each
with exit code **0**:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo check --workspace --no-default-features --locked --offline
cl /nologo /std:c++17 /W4 /WX /c /I"external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/include" crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp /Fo"target/final-windows-20260928-attempt-1/dispatch_native_abi.obj"
cl /nologo /std:c++17 /W4 /WX /MT /LD /I"external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include/dx12" crates/fsr-sdk/tests/m6_gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo"target/final-windows-20260928-attempt-1/" /link /OUT:"target/final-windows-20260928-attempt-1/m6_gpu.dll" /IMPLIB:"target/final-windows-20260928-attempt-1/m6_gpu.lib" d3d12.lib dxgi.lib
cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json
```

Windows ordinary tests: 25 wrapper tests passed, five native tests ignored;
six sys common-ABI tests, two dispatch-ABI tests and four loader fixture tests
passed. Four doctests passed. Other opt-in native probes stayed ignored. The
paired selected dispatch ABI compilation and native helper build passed with
`/W4 /WX`; Gate A also passed all five resource conversions against AMD's helper.
The [check status file](2026-09-28-exp-final-windows-release-verification/checks.status.txt)
and [sanitized runner transcript](2026-09-28-exp-final-windows-release-verification/runner.log)
preserve results. The outer runner exited **0**.

Separate exact child commands, with 180,000 ms timeouts:

```powershell
target/final-windows-20260928-attempt-1/m6.exe --ignored --exact upscaler::m6::native_dispatch_readback --nocapture --test-threads=1
target/final-windows-20260928-attempt-1/m6.exe --ignored --exact upscaler::m6::native_temporal_smoke --nocapture --test-threads=1
```

## Gate A — final native regression

**PASSED.** Child started 18:51:46.719 CEST and finished 18:51:47.543;
`timeout=False exit=0`. Runtime acquisition, public `UpscalerOptions::new`,
`Upscaler::new` and `unsafe Upscaler::dispatch` succeeded. Creation, provider
query, dispatch and explicit destroy each succeeded with native code
`0x00000000`. Output readback recording, list Close, queue Signal, fence
completion and mapped readback each returned HRESULT `0x00000000`.
`ExecuteCommandLists` was issued; it has no return value. GPU completion was
then established by the fence.

| Metric | Observed | Required |
|---|---:|---|
| Output size | 640 × 360 | 640 × 360 |
| Pixels finite in all RGBA channels | 230,400 | 230,400 |
| Pixels changed from sentinel | 230,400 | >115,200 |
| Red minimum / maximum | 0.101745605 / 0.8486328 | range >0.1 |
| Left-quarter / right-quarter mean red | 0.19475134 / 0.7568281 | right > left +0.05 |
| D3D12 errors/corruption | 0 | 0 |
| Device removed reason | `S_OK` (`0x00000000`) | `S_OK` |

The fence timeout remained 60,000 ms; no timeout occurred. Explicit
`upscaler.destroy()` succeeded. After context destruction and harness-resource
cleanup, the still-live caller device again reported zero errors/corruption
and `S_OK`. The child completed normally. Independent
[stdout](2026-09-28-exp-final-windows-release-verification/gate-a.stdout.txt),
[stderr](2026-09-28-exp-final-windows-release-verification/gate-a.stderr.txt) and
[status](2026-09-28-exp-final-windows-release-verification/gate-a.status.txt)
preserve the observations.

## Gate B — temporal multi-frame smoke

**PASSED — all three frames.** Child started 18:51:47.549 CEST and finished
18:51:48.032; `timeout=False exit=0`. One runtime, one device and the same
public `Upscaler` were retained throughout. Each frame used a fresh existing
M6 resource/list/allocator set and queue. Each dispatch was submitted and fenced
before the next frame started. All three sets remained alive through normal
upscaler Drop. This serialization is a test strategy, not a wrapper requirement
or a general cross-queue support claim.

Resource profile: render 320 × 180, output 640 × 360; linear non-HDR RGBA16F
color, R32F depth at 0.5, RG16F motion, 1 × 1 R32F exposure at 1.0, distinct
UAV-capable RGBA16F output. Every fresh output channel was initialized to
half-float sentinel `0x7bff`. Frame time 16.667 ms; near/far 0.1/100;
vertical FOV π/3; view-space-to-metres 1.0; pre-exposure 1.0; unchanged zero
creation/dispatch flags and disabled sharpening/masks.

| Frame | Translation | Jitter (pixels) | Reset | Motion (UV) |
|---|---|---|---|---|
| 0 | 0 px | (0.25, -0.25) | true | (0, 0) |
| 1 | +1 px right | (-0.25, 0.25) | false | (-1/320, 0) |
| 2 | +2 px from frame 0 | (0.125, -0.125) | false | (-1/320, 0) |

For each integer texel, CPU generation used exactly:

```text
sx = x + 0.5 + jitter_x[frame]
sy = y + 0.5 + jitter_y[frame]
u = sx - frame
RGBA = (0.1 + 0.75*u/320, 0.1 + 0.75*sy/180, 0.2, 1.0)
```

No wrapping/clamping was introduced. The same jitter entered image generation
and `JitterOffsetPixels`. Constant depth is invariant under these samples.
Motion was current-to-previous, top-down UV, excluding jitter; the native scale
was (320, 180). The existing half conversion quantizes -1/320 to approximately
-0.003124237, or -0.9997559 render pixels after scaling.

Every public dispatch succeeded with native code `0x00000000`; every harness
creation succeeded. For each frame, output transition/copy recording, Close,
Signal, the 60-second-bounded fence and mapped readback returned `S_OK`.
Submission was issued for each list. Required and optional metrics were:

| Frame | Finite pixels | Changed pixels | Debug errors | Device removed | Red min / max | Left / right mean red |
|---|---:|---:|---:|---|---|---|
| 0 | 230,400 | 230,400 | 0 | `S_OK` | 0.10168457 / 0.8486328 | 0.19475116 / 0.7568281 |
| 1 | 230,400 | 230,400 | 0 | `S_OK` | 0.08880615 / 0.84521484 | 0.19059116 / 0.7517236 |
| 2 | 230,400 | 230,400 | 0 | `S_OK` | 0.08831787 / 0.84277344 | 0.1885455 / 0.7491612 |

Every output was 640 × 360. No non-finite pixel, all-sentinel output or timeout
occurred. Gradient values are optional diagnostics; no monotonicity or exact
reconstruction assertion was added.

After frame 2's fence, normal `drop(upscaler)` completed while runtime, device
and all three frame resource sets remained alive. The public API exposes no
native Drop return code, so none is claimed. Device checks immediately after
Drop and after frame-resource cleanup both reported zero D3D12 errors/corruption
and `S_OK`. Independent
[stdout](2026-09-28-exp-final-windows-release-verification/gate-b.stdout.txt),
[stderr](2026-09-28-exp-final-windows-release-verification/gate-b.stderr.txt) and
[status](2026-09-28-exp-final-windows-release-verification/gate-b.status.txt)
preserve frame parameters and results.

## Failures and warnings

Sandboxed GPU metadata access during prompt review was denied. The authorized
read-only metadata query outside the sandbox succeeded before implementation.
No native GPU test ran during that review/setup inventory.

The first actual runner attempt passed all checks and both gates. There was no
native failure to fix or reclassify. Git emitted working-copy LF-to-CRLF notices
for the three changed harness files; these did not affect the checks. Non-error
D3D12 warnings were not enumerated; the required error/corruption counts were
checked during readback and after teardown. The historical GBV dispatch stall
remains unresolved and outside this gate.

Retained evidence excludes personal checkout prefixes, usernames, machine names
and transcript host identifiers. The transcript's host header was removed and
the repository prefix replaced with `<repo>`; meaningful GPU IDs, versions,
hashes and HRESULT/native codes were preserved. No transient pointers occurred
in the retained native output. No AMD binaries were copied into these records.

## Limitations

This bounded proof cannot establish image quality, temporal reconstruction
accuracy, general cross-queue support, arbitrary providers/hardware, or GBV
success. Serialization is the test strategy; the wrapper imposes no per-dispatch
fence cadence.

Three successful moving non-reset/reset dispatches establish a temporal
multi-frame **smoke** result, not correct history use, reset effectiveness or
an image-quality oracle. Provider identification and staged DLL signatures do
not identify every participating driver/provider implementation. This is the
recorded SDK/runtime/GPU/driver/profile only. Packaging, registry publication,
hosted documentation, public history curation and the final release/privacy
audit are not newly verified by this task.

## Release conclusion

```text
Gate A: PASSED
Gate B: PASSED (three frames; not deferred)
Overall: WINDOWS RELEASE VERIFICATION PASSED
```

Required provenance is present, the actual adapters were AMD hardware, standard
checks passed, and no relevant unresolved failure remains within these two
gates. This conclusion applies to the captured baseline-plus-harness state,
not an independently changed or future release tree.
