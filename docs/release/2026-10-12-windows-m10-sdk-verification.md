# Windows M10 and SDK acquisition verification — 2026-10-12

The M10 Windows checkpoint and the real `sdk.py` Windows strict-mode checkpoint
passed. M10 is verified in unreleased source; R02 remains Active for its broader
runtime dependency/deployment work. No public contract, dependency, SDK pin or
crate version changed, and nothing was published. D019 remains a deferred proposal.

This follows the earlier [M10 implementation checks](2026-10-12-upscaler-query-verification.md)
and [SDK acquisition checks](2026-10-12-sdk-acquisition-verification.md).

## Equipment and inputs

- Windows 11, build 10.0.26300; `x86_64-pc-windows-msvc`.
- Rust/Cargo 1.99.0; Rust 1.88.0 for the separate MSRV profile.
- Python 3.14.1, PowerShell 7.6.6, MSVC toolset 14.44.35207,
  Windows SDK 10.0.22621.0.
- Adapter explicitly selected as index 0: AMD Radeon RX 9060 XT,
  vendor `0x1002`, device `0x7590`, feature level 12_0;
  driver query succeeded with version `32.0.31041.1004`.
- AMD SDK v2.3.0; source commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- The first native attempt used the existing full SDK checkout. A second fresh
  native attempt used only the eight-file installation produced by `sdk.py` at
  `target/sdk-windows-verification/v2.3.0`.

The exact source/configuration inputs are retained as hashes in
[inputs.json](2026-10-12-windows-m10-sdk-verification/inputs.json).
[Attempt summaries](2026-10-12-windows-m10-sdk-verification/attempts.json)
retain successful, failed and interrupted runs. All completed accepted runs
passed coordinator integrity verification before evidence selection.

## Commands and results

Commands were executed from the repository root. Ordinary profile execution was
offline; only explicit SDK acquisition and strict Windows trust checks needed
network permission. The existing full SDK was preserved.

```powershell
python tools/verification/run.py sdk verify
# Expected rejection: the default root is an existing full source checkout,
# not this tool's exact installation inventory (unexpected .git directory).

python tools/verification/run.py sdk install --root target/sdk-windows-verification/v2.3.0
python tools/verification/run.py sdk verify --root target/sdk-windows-verification/v2.3.0
python tools/verification/run.py sdk install --root target/sdk-windows-verification/v2.3.0

# Acquire the installed Windows SDK tools, including SignTool, without printing
# the complete developer environment into the evidence.
$sdkVerificationEnvironment = & pwsh -NoProfile -NonInteractive -File tools/verification/windows.ps1 | ConvertFrom-Json -AsHashtable
foreach ($sdkVerificationName in $sdkVerificationEnvironment.Keys) {
    [Environment]::SetEnvironmentVariable($sdkVerificationName, $sdkVerificationEnvironment[$sdkVerificationName], 'Process')
}
python tools/verification/run.py sdk verify --root target/sdk-windows-verification/v2.3.0 --strict

python tools/verification/run.py run native-upscaler --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run native-upscaler --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run ci --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv
```

| Check | Observed result |
|---|---|
| Online SDK installation | Eight pinned payload files plus provenance installed; ordinary Windows Authenticode status Valid for both DLLs |
| Offline verification and existing-install reuse | Both pass; reuse performed no download or repair |
| Real strict verification | Both DLLs pass SignTool, publisher, timestamp and Windows online chain policy |
| Native profile, existing SDK | All 22 cases pass |
| Native profile, acquired subset | All 22 cases pass |
| Final Windows `ci` profile | All 104 cases pass |
| Windows MSRV profile | All 87 cases pass on Rust 1.88.0 |

The final infrastructure suite ran 115 tests: **86 passed, 29 skipped, no failures**.
Twenty-eight skips are optional temporal experiment oracles requiring NumPy,
which was absent in this interpreter; one is the portable symlink control that
explicitly skips Windows privilege requirements. These are not new temporal or
Windows symlink verification claims. The current PowerShell bridge parsed, the
archive's required PowerShell check passed, and the corrected unsafe ZIP path
fixture executed successfully on Windows. The SDK suite's other 35 tests passed.
See the [test log](2026-10-12-windows-m10-sdk-verification/infrastructure-tests.txt).

The final `ci` run executes all four feature configurations, including the new
public controlled DLL query fixture, denied-warning Clippy/Rustdoc, all four
independent MSVC ABI probes, consumers, dependency trees, Vulkan rejection and
complete examples. MSRV separately executes all four configurations and the
consumer/example boundaries. See the selected
[default](2026-10-12-windows-m10-sdk-verification/rust-default-tests.txt) and
[movable](2026-10-12-windows-m10-sdk-verification/rust-movable-tests.txt) test logs.

## Native M10 observations

The [public query child](2026-10-12-windows-m10-sdk-verification/native.queries.public.txt)
called all five accepted methods for local and movable families with maximum
render size 1280×720 and output size 1920×1080. Both provider enumerations were
nonempty. Both families reported:

- Owned provider name `4.1.1`, opaque diagnostic ID `17700776140664213505`.
  This is neither a public numeric ID API nor a version-order guarantee.
- Pre-create estimate and live total memory: **80,281,600 bytes**;
  aliasable memory: **0 bytes**.
- Required resource bits `15` (colour, depth, motion vectors and output);
  optional bits `0`.

The live memory total is a provider observation for these options, not a
reservation or universal accounting guarantee. The independent query C++ ABI
probe passed under MSVC alongside common, creation and dispatch probes.

Existing lifecycle, single-frame, three-frame and semantic dispatch regressions
passed after the query gate. The [single-frame log](2026-10-12-windows-m10-sdk-verification/native.dispatch.single.txt)
records queries before dispatch, successful GPU submission/fence/readback,
230,400 finite and changed pixels at 640×360, zero debug errors, no device removal
and successful explicit destruction. Both movable modes also passed with live
queries on transferred owners; the retained logs cover
[disabled sharpening](2026-10-12-windows-m10-sdk-verification/native.movable.disabled.txt)
and [native sharpening](2026-10-12-windows-m10-sdk-verification/native.movable.sharpening.txt).
Their declared/caught unwind control is expected, not an uncaught child failure.

Three M9 integration repetitions produced all 144 expected readbacks; the
[independent audit](2026-10-12-windows-m10-sdk-verification/native-sharpening-audit.json)
passed its 78 diagnostic comparisons. This does not revise the earlier failed
sharpening image-effect criteria. Full native selectors, watchdogs, commands and
evaluations remain in the selected
[plan](2026-10-12-windows-m10-sdk-verification/native-plan.json) and
[case records](2026-10-12-windows-m10-sdk-verification/native-cases.json).

## SDK identity and real Windows trust

The successful explicit acquisition downloaded the 127,451,548-byte official
prebuilt ZIP and six pinned source files. The archive SHA-256 matched
`f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`.
All eight file pins and both full notice hashes matched. Measured identities and
acquisition metadata are retained in
[provenance](2026-10-12-windows-m10-sdk-verification/sdk-provenance.json),
[offline verification](2026-10-12-windows-m10-sdk-verification/offline-result.json)
and [reuse](2026-10-12-windows-m10-sdk-verification/reuse-result.json).

The [strict result](2026-10-12-windows-m10-sdk-verification/strict-result.json)
records fresh checks of both installed DLLs:

- Authenticode status `Valid`, embedded signature type `Authenticode`, exact
  publisher `Advanced Micro Devices`.
- SignTool `verify /pa /all /v /tw`: exit 0, one signature verified per DLL,
  **zero warnings and zero errors**.
- Signer thumbprint `33D35682079E201671B738B7209B4586103BC271`;
  timestamp thumbprint `DD6230AC860A2D306BDA38B16879523007FB417E`.
- Signer and timestamp chains both `valid: true`, with empty error-status lists
  under the Windows online revocation policy.

This is the OS policy result at the recorded time. Cached valid revocation data
may satisfy that policy; the check does not prove a fresh remote OCSP response.
Strict verification did not rewrite installation provenance. The separately
retained fresh result, rather than historical provenance, establishes this check.

## Failed attempts and remaining boundaries

The first SDK download was blocked by sandbox DNS access; the first permitted
retry failed with `SSL: DECRYPTION_FAILED_OR_BAD_RECORD_MAC`. A subsequent fresh
retry succeeded. No failed download was accepted or installed.

The first Windows `ci` attempt finalized as failed: infrastructure temporary-file
cleanup and compile-test output creation encountered sandbox `WinError 5`.
A fresh run outside the sandbox passed those tests but was interrupted at
`consumer.queries-local` by `WinError 5` while replacing `result.json.pending`.
It completed only 50 of 104 cases and is not a complete verified run.
The next fresh run outside the sandbox completed all 104 cases successfully.
No source fix was needed; the earlier failed/incomplete attempts remain recorded
and are not relabelled as passes.

Evidence selection is compact and explicitly excludes AMD files, executables,
developer environment and raw readbacks. The retained audit cannot reconstruct
omitted readbacks; a fresh GPU run is needed for full image reanalysis. The
[selection description](2026-10-12-windows-m10-sdk-verification/selection.json)
and [file inventory](2026-10-12-windows-m10-sdk-verification/artifacts.json)
describe this boundary. The report and selected files are sufficient to read the
commands, inputs and observed outcomes without Git history or an external CI page.

These results cover one Windows/GPU/driver/provider configuration under D018's
caller-established trust. They do not prove arbitrary-provider safety,
in-flight query compatibility for every provider, a native Vulkan backend,
Proton execution, clean-machine runtime dependency closure or redistribution
clearance. Exact package/release checks and release/version selection remain
separate. R02's Windows strict checkpoint is complete; its broader deployment
work remains open. D019 has not been reviewed or accepted by this verification.
