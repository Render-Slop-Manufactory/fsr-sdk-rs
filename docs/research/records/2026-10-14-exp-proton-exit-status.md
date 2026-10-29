# Experiment: Proton jitter launcher exit 247

2026-10-14 Europe/Paris (2026-10-13 UTC). Can the jitter launch's exit 247
reproduce with tracing, and does prefix reuse explain it? **Unresolved:** no
reproduction. V07d's failed launch and image-budget failures remain in the
[baseline record](2026-10-13-exp-proton-temporal-regression.md).

## Declaration and inputs

Before execution, declare POSIX zero/SIGKILL controls, three Windows zero-exit
controls, three full seed-419 replays on the historical prefix, then one each
on two newly prepared prefixes. If tracing does not reproduce failure, run one
untraced replay. Require launcher zero, both completion markers, independent
raw-data oracle and active Vulkan core/synchronization validation without errors.
Bound each launch to 360 seconds. Reduce workload only after reproduction.
Exclude image-criterion changes, V08/offline bootstrap, forced V09 expiry, public
API changes and automatic acceptance of exit 247.

Clean source at `fd477f4b45448af8699fe51e7af86bc4c454c648`; no production changes.
Reuse the baseline's exact Rust/helper/AMD bytes, seed-419 arm order, jitter
table and protocol: stationary/moving phase 0, six controls each, 128 frames per
arm, 1,536 frames per process. Payload SHA-256:

| File | SHA-256 |
|---|---|
| proton-fsr_sdk.exe | `7b88f69a19ce236e84ecdcd14ae6d927ba686e421f0059cb065d6fcfc6161098` |
| m6_gpu.dll | `5df2adc3241e92e4e9bf3178c8ab37fe8979f34aa3a1d835ccb2ec7b3f2a4fb7` |
| amd_fidelityfx_loader_dx12.dll | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| amd_fidelityfx_upscaler_dx12.dll | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

Arm-table hash: `1aebaf7741ca2aa427f775aa6224328164d299496425d43d07eaed34fed2f9b4`.
Equipment remained SteamOS 3.9.2 build 20260925.101, kernel
7.2.7-valve1-1-neptune-72-gc8730d37f9c6, AMD Custom GPU 0405/RADV VANGOGH
(`0x1002:0x163f`), Mesa 26.2.0-devel git-6c05e09fac. Provider 3.1.5,
ID `0xf5a5ca1e00c01005`, SDK 2.3.0; Proton 10.0-4b
(`1785139158 proton-10.0-4b`), VKD3D-Proton 3.0.0, umu 1.4.4/steamrt3,
Khronos validation 1.4.357.0. Existing strace 7.0, Python 3.14.7/NumPy 2.5.2;
Windows control compiled with Clang/LLD 21.1.8 and prepared Microsoft caches.
No package installation or new dependency.

## Method and reproduction

Command from the repository root, outside the GPU sandbox:

```sh
nix develop path:./tools/verification/nix --command \
  python3 target/proton-exit-diagnosis/probe.py
```

The disposable coordinator uses the shared `execution.Runner`, `records`,
`proton` and `vulkan_validation` helpers. Corrected coordinator hash:
`83da4da27d1403fe8e514682ff15daf30d2d7d85de8ba112e76b3122804d7e92`;
predeclared protocol hash:
`fea1044578b7b557b071fce8a0b39fa07d20a9a550d2219963fb6f1368ecff39`.
Use the baseline's `proton-temporal-jitter` build/staging command to prepare
payloads; copy its seed-419 `r1/arms.txt`, protocol `jitter.txt` and
`protocol.json`. Rebuilds need not reproduce historical binaries.
Each GPU command, in the staged native directory, was:

```sh
/usr/bin/strace -f -ttt -s 128 -e trace=process,signal -o trace.txt -- \
  "$VERIFICATION_UMU" ./proton-fsr_sdk.exe --ignored --exact \
  upscaler::dx12::m6::proton_temporal_diagnosis --nocapture --test-threads=1
```

Remove the strace prefix for the untraced control. Clean inherited environment
with `proton.clean_environment`; set GAMEID=umu-default, STORE=none,
PROTONPATH to installed Proton, PROTONFIXES_DISABLE=1, UMU_LOG=info,
UMU_RUNTIME_UPDATE=0, PROTON_VERB=runinprefix, PROTON_LOG=1, WINEDEBUG=+loaddll,
WINEDLLOVERRIDES=`d3d12=n;d3d12core=n;dxgi=n`, WINEPREFIX and separate
PROTON_LOG_DIR. Use `vulkan_validation.environment` with the locked shell's
`VERIFICATION_VULKAN_MANIFEST`. Through `proton.windows_path`/`wine_drives`, set
the following FSR_SDK_TEST_ variables to staged DLLs or fresh per-case paths:
DLL=loader, PROVIDER_DLL=upscaler, M6_DLL=DEVICE_DLL=helper,
TEMPORAL_DIAGNOSIS_OUTPUT=frames directory, TEMPORAL_DIAGNOSIS_ARMS=arm table,
TEMPORAL_CALIBRATION_JITTER=jitter table. Set TEMPORAL_DIAGNOSIS_FRAMES=128 and
TEMPORAL_DIAGNOSIS_JITTER_STUDY=1.

Historical prefix: `target/proton-prefixes/20261013T221309Z-proton-4940f550f990`.
Fresh prefixes initialize the tiny Windows control with `waitforexitandrun`,
then check translator hashes against installed Proton. Control source:
`#include <cstdio>` followed by
`int main(){std::puts("ZERO-CONTROL: completed");return 0;}`; console/MSVC `/MT`.
Trace exec/wait/exit/signal events; record module identities and cgroup
`memory.events` before/after. Audit raw data with the existing
`temporal_diagnosis.validate` and unchanged protocol, adapting only its filename
resolver from compressed to raw paths. Hash ordered raw filename/SHA pairs and
remove only those newly generated, audited files.

## Observations

Attempt `20261013T230607Z-trace-3c7bdc9a3afc` under `target/proton-exit-diagnosis`
ran 23:06:07–23:28:30 UTC; all 12 cases passed, inputs unchanged.

| Launch | Exit | Native launch seconds |
|---|---:|---:|
| Traced historical prefix 1 | 0 | 170.587 |
| Traced historical prefix 2 | 0 | 185.434 |
| Traced historical prefix 3 | 0 | 179.109 |
| Traced fresh prefix 1 | 0 | 181.472 |
| Traced fresh prefix 2 | 0 | 175.405 |
| Untraced historical prefix | 0 | 97.353 |

Each completed 1,536 frames and the Rust test, with active core/sync
validation (three instances, zero errors/warnings). Each oracle checked
1,415,583,744 input bytes; input digest
`64cf9f37a91946a3d0d8d28768fc2a76635c4af472b2a43d9c51e502ee90cae6`.
Each 3,072-file, 4,246,738,944-byte input/output inventory matched the original
failed launch exactly:
`2c398bd27afadb92161d09ed78a2a8dc97110eb5f2b4bd2322bb70e7ce6bd7b0`.
No image acceptance criteria were evaluated here. Error 1726 occurred in all
six successful full replays; it is not a distinguishing failure marker.

All five full matrix traces had no SIGKILL event. Example first replay:
Proton Python PID 60021 waited for Wine PID 60022 with WEXITSTATUS=0;
umu PID 59876 waited for container PID 59909 with WEXITSTATUS=0; both then
exited 0. Container startup also had nonzero optional helper statuses, so not
every child status is a native failure. Three Windows zero controls exited 0.
The POSIX zero fixture returned 0. The deliberately killed POSIX child yielded
Python subprocess status -9 and wrapper exit 247, with SIGKILL visible in trace.
Installed Proton's `runinprefix` uses `subprocess.call` and then `sys.exit(rc)`;
umu forwards its child wait status. Thus SIGKILL-to-247 conversion is verified
as a possible mechanism, **not the original launch's established cause**.

Cgroup oom/oom_kill/oom_group_kill counters were zero before/after every replay.
`journalctl -k --since '2026-10-13 22:52:00 UTC' --until '2026-10-13 23:31:00 UTC'
--grep 'oom|Out of memory|Killed process' --no-pager` returned no entries.

Pilot `20261013T225247Z-trace-ea265626bef9` remains failed: its first full
replay exited 0 but the reader incorrectly expected `.inputs.gz`. Its second
GPU child also completed normally; only the coordinator was stopped after
child exit, preserving an interrupted/failed case and four blocked cases.
Corrected supplementary raw audits passed both datasets and matched the same
inventory; original case outcomes were not overwritten. Both finalized attempt
inventories passed integrity verification.

## Limits and remaining question

Tracing slowed execution. The untraced pass does not eliminate timing-sensitive
failure. Global caches and OS/service history were not reset; staging copied
the historical VKD3D cache and reused it.
No original-failure trace exists. Keep the nonzero verdict; no workload reduction
without reproduction. Capture process/wait/signal events on recurrence to
identify which layer first reports failure.
Raw images were removed after audits; traces/outcomes remain under ignored
target. No raw addresses or personal paths are published.
