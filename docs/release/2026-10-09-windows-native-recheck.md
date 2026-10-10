# Windows native regression recheck — 2026-10-09

## Result and scope

All 30 existing ignored Windows native test declarations were exercised, including
their runner-selected variants and repetitions. The ordinary lifecycle, public
dispatch, temporal/semantic, recording-ahead, private transfer/cleanup and public
movable regressions passed. The original sharpening effect gate remains failed;
the GBV dispatch diagnostic again timed out. Existing raw allocation-failure and
zero-dimension probes again terminated their isolated processes. This is not an
all-green suite, release approval, new research protocol or broader support claim.

Runs completed on 2026-10-09, approximately 19:45–19:51 CEST (UTC+02:00), against
the captured baseline. Unrelated sharpening implementation edits appeared in the
shared worktree afterward, around 19:53. **These results do not verify those
later edits.** The input fingerprints below identify the executed baseline
without requiring Git history. No production code, public API, architecture
contract, SDK input or acceptance criterion was changed for this recheck.

One test-runner repair was necessary: the InputLifetime prerequisite still
expected `Production owner: destroy OK`, whereas the existing two-context public
control emits `Production owner: destroy A/B OK; B via explicit destroy`. The
runner now requires that exact complete marker as well as exit 0 and no timeout.
Its first invocation stopped before the lifetime variants; the repaired runner
then completed control, retain, protect and cleanup. No probe behavior changed.

## Configuration and retained evidence

- Windows 11 Pro 10.0.26300, `x86_64-pc-windows-msvc`.
- Rust 1.99.0 (`b940084d7`, LLVM 23.1.1); PowerShell 7.6.6.
- Visual Studio 2022 Build Tools 17.14.28, MSVC tools 14.44.35207,
  compiler 19.44.35224.0, Windows SDK 10.0.22621.0.
- AMD Radeon RX 9060 XT, vendor/device `0x1002/0x7590`, feature level 12_0,
  driver 32.0.31041.1004. Successful context queries selected provider 4.1.1,
  ID `0xf5a5ca1e01001001`.
- Locally acquired FidelityFX SDK v2.3.0, pinned headers at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`; no downloads or installations.
- Loader SHA-256:
  `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`.
- Upscaler DLL SHA-256:
  `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
  Both staged AMD Authenticode signatures were valid.
- Ordinary DX12 debug layer for dispatch regressions; GBV disabled except in
  its explicit diagnostic. Renderer equipment and DLLs were staged under
  `target`; native cases used separate child processes and existing watchdogs.

The first ordinary workspace test attempt was blocked by sandbox permissions
when a compiler fixture wrote temporary metadata. It stopped before native
execution. The successful retry and remaining fixture/native runners executed
outside that sandbox, with `CARGO_INCREMENTAL=0`. Rustdoc denied warnings.

Retained, sanitized evidence:

| Artifact | Contents |
|---|---|
| [Checks and environment](2026-10-09-windows-native-recheck/checks.txt) | Versions, runtime hashes, ordinary checks, runner exit statuses and movable workspace test summaries |
| [Source fingerprints](2026-10-09-windows-native-recheck/source-fingerprints.json) | 93 captured crate/manifest inputs, raw and UTF-8/LF SHA-256; captured before the runner repair and later unrelated edits |
| [Native inputs](2026-10-09-windows-native-recheck/native-inputs.json) | Header and staged executable/helper/runtime hashes; repaired InputLifetime runner hash recorded separately |
| [Case results](2026-10-09-windows-native-recheck/case-results.json) | 85 primary child results covering all 30 ignored declarations; compatibility-runner baseline repetitions and the superseded lifetime control are not duplicated |
| [Diagnostic observations](2026-10-09-windows-native-recheck/diagnostics.txt) | GBV, dimensions, allocation-failure and returned-error lifetime logs |
| [Transfer audit](2026-10-09-windows-native-recheck/audit-summary.json), [case outcomes](2026-10-09-windows-native-recheck/case-outcomes.json) | Independently checked statuses, frames, successful native destroys, unwind and residency observations |
| [Payload hashes](2026-10-09-windows-native-recheck/payload-hashes.json), [comparisons](2026-10-09-windows-native-recheck/payload-comparison.json) | All 96 selected D013 payload identities and control/transfer repeatability results |
| [Sharpening audit](2026-10-09-windows-native-recheck/sharpening-audit.csv) | Independent half-float decoding, final red-plane hashes, metrics and original criteria for all three repeats |

Raw image payloads remain local under `target`; the report retains their hashes
and decoded comparisons rather than binaries. The existing independent audit
scripts were copied to new target directories before execution, so their output
could not overwrite historical records. All 379 pre-existing files under
`docs/research/records/` and `docs/release/` remained byte-identical.

## Commands and coverage

Run from the repository root in an x64 Visual Studio developer shell, with the
trusted local SDK and cached dependencies. All Cargo operations were offline and
locked where supported. Let `$stage` be a fresh directory such as
`target/windows-native-recheck-2026-10-09`; do not reuse populated payload stages.

| Command | Observed outcome |
|---|---|
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode movable -Native -Stage "$stage/movable-retry"` | Exit 0: six native children, including explicit/Drop lifecycle, gates A/B/C and public movable siblings/unwind/failed handoff |
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode d013-cleanup -Native -Stage "$stage/d013-cleanup"` | Exit 0: all 11 prerequisites and 30 repeated control/transfer children |
| `./crates/fsr-sdk/tests/native/run.ps1 -Mode sharpening -Native -Stage "$stage/sharpening"` | Runner exit 1: all three children exit 101 at the original effect assertion; no timeout |
| `./crates/fsr-sdk/tests/run-m6-recording-spike.ps1` | Exit 0: same-context counts 1, 2, 5, 8 and sibling counts 2, 5; each child exit 0 without timeout |
| `./crates/fsr-sdk/tests/run-m6-gbv-spike.ps1` | Diagnostic runner exit 0; control child passes, dispatch child times out after 90 seconds and is terminated |
| `./crates/fsr-sdk-sys/tests/provider_resolution/run.ps1` | All four layouts observed; positive executable-adjacent cases A/C exit 0; negative B/D exit 101 with return code 4 and count 0 |
| `./crates/fsr-sdk-sys/tests/shared_runtime/run.ps1` | Child exit 0 without timeout; sequential sibling identities/queries and destroy pass |
| `./crates/fsr-sdk-sys/tests/context_dimensions/run.ps1 -Cases 'C0,Z1,Z2,Z3,Z4,T1,N1,E1,D1,B1,B2,B3'` | All 12 cases observed without timeout; successful, returned-error and process-abort outcomes distinguished below |
| `./crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -FailureInjection` | Successful controls 0/4 and three access-violation outcomes 1/2/3, each in a separate process |
| `./crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1 -InputLifetime` | After marker repair: control and all three lifetime variants exit 0 without timeout |
| `cargo test -p fsr-sdk-sys --features dx12 --test loader --locked --offline -- --ignored --exact loads_amd_runtime_without_api_calls --nocapture --test-threads=1` | Pass; `FSR_SDK_TEST_DLL` set to the absolute trusted local AMD loader path; no AMD API calls |
| `cargo test --workspace --features fsr-sdk/movable --locked --offline` | Pass: 62 ordinary tests and six doctests; all 30 native declarations ignored here and exercised separately above |
| `./tools/check-movable-api.ps1 -Stage "$stage/consumer-controls"` | Five positive and eighteen diagnostic-specific negative Windows consumer controls pass, alongside production dependency-tree and unsupported-Vulkan controls |
| `cargo doc --workspace --features fsr-sdk/movable --no-deps --locked --offline --target x86_64-pc-windows-msvc` | Pass with `RUSTDOCFLAGS=-D warnings` |

Each shared native runner also passed formatting, default workspace tests,
warning-denying Clippy, no-default-feature compilation, three independent native
C++ ABI probes against hash-verified local headers, and native helper builds.
`run-m4.ps1`/`run-m5.ps1` and `-Mode lifecycle` select the same two lifecycle tests
already covered above; `-Mode d013` is covered by the six transfer prerequisites.
The recording and GBV runners each additionally reran dispatch gates A/B/C.
No unbounded combined `cargo test -- --ignored` invocation was used.

## Positive regressions

Public explicit destroy and Drop both completed two-context cleanup. Loader
residency survived early public-runtime release and context A destruction, then
ended after B cleanup. Gate A and the three Gate B frames produced 230,400 finite,
changed pixels at 640x360, with zero reported debug errors and no device removal.
Gate C retained motion delta 0.24122791, jitter delta 0.03519135, reset delta 0,
and history delta 0.01101808. Analytic red MAE was 0.01334815 for correct inputs,
0.25143567 for reversed motion and 0.04247920 for reversed jitter.

The public movable regression passed two fenced frames and coordinated cleanup
of independent siblings during deliberate caught unwind and failed handoff.
The D013 cleanup audit checked all 41 children, 516 logged fenced-frame
observations, 42 successful traced native destroys and 12 actual unwind Drops.
All 96 selected RGBA16F payloads were 1,843,200 bytes; repeated control and
transfer outputs matched exactly for all five cases. Loader/helper residency
ended after application references were released, but the staged provider
remained resident before normal child exit. This does not attribute provider
termination or establish arbitrary threading/concurrency guarantees.

Recording-ahead counts 1/2/5/8 on one context and 2/5 with siblings all completed
readback. The raw shared-runtime test retained B's provider identity after A was
destroyed; both destroys returned 0. Provider residency persisted after loader
drop. Provider-resolution cases A/C returned a nonzero version count of 2;
placing both DLLs only in the private runtime directory (B), or omitting the
provider (D), returned `FFX_API_RETURN_NO_PROVIDER` (4), count 0. These bounded
results preserve D007's deployment policy and do not identify every counted
provider or its dependency closure.

## Reproduced diagnostic limits

**Sharpening:** all 240 frame observations were finite, changed and fenced, with
zero reported debug errors and no device removal. All three repeats completed
the same existing two scenes and five settings, then failed the unchanged effect
assertion. Independent decoding of all 30 final red-plane payloads reproduced
the metrics. Each scene/setting had identical final hashes across repetitions;
the two disabled controls matched exactly, while enabled zero differed.

The existing metric criteria require ROI mean absolute difference `D >= 0.001`, changed
fraction `C >= 0.01`, horizontal gradient ratio `G/G_off >= 1.01`, and absolute
mean bias `B <= 0.005`. Only scene 0 at 0.35 and 1 contributes effect criteria
to the original aggregate assertion, alongside repeatability checks. Enabled zero
and scene 1 are diagnostic only. The aggregate assertion still fails:

| Scene | Enabled setting | D | C | G/G_off | B | Metric criteria |
|---|---|---:|---:|---:|---:|---|
| 0 | 0 | 0.0022058795 | 0.8322365334 | 1.0068531333 | 0.0002401376 | Fail |
| 0 | 0.35 | 0.0034433764 | 0.8488762200 | 1.0058436764 | 0.0013968057 | Fail |
| 0 | 1 | 0.0305979964 | 0.9914191535 | 1.0653898496 | 0.0195217544 | Fail |
| 1 | 0 | 0.0010861201 | 0.2518064940 | 1.8781280247 | 0.0007913059 | Pass |
| 1 | 0.35 | 0.0029274483 | 0.4985806119 | 0.9795018289 | 0.0025952632 | Fail |
| 1 | 1 | 0.0042345393 | 0.4987272429 | 4.4716495103 | 0.0010167820 | Pass |

This repeats the [original failed gate](../research/records/2026-09-30-exp-d014-native-sharpening.md).
It verifies execution of the test-only native pass, not a later public D014 API,
its separate release protocol, or general image quality. No scene, threshold or
contract was amended.

**GBV:** the control omitting FSR dispatch returned the unchanged 65504 sentinel,
230,400 finite pixels, zero changed pixels/debug errors and no device removal.
The dispatch case reached the resource-parity marker but produced no dispatch
completion/readback before the existing 90-second watchdog; sampled CPU time
reached 86 seconds. It was terminated with exit -1. The runner's exit 0 means
the diagnostic completed, not that GBV dispatch passed. Fault ownership and the
exact blocking instruction remain unestablished.

**Raw dimensions:** C0, T1, N1, E1 and D1 each created, queried and destroyed a
context successfully. Z1–Z4, each setting one dimension to zero, entered native
create and aborted with `0xC0000409` and “Rust cannot catch foreign exceptions”.
No native return or cleanup postcondition was observed for those calls. B1/B2/B3
(width 16383/16384/16385, height 720) each returned generic error 1 and a null
handle, with no destroy attempt; the rejection cause remains unestablished.
Raw acceptance of unusual positive sizes is not a dispatch/support guarantee.

**Allocation failure:** the successful baseline made three callback allocations;
destroy freed each once. Injecting null at indices 1/2/3 again caused process
access violation `0xC0000005` before create returned, with no observed frees.
The unreached index-4 control completed create/destroy normally. These process
codes are not FFX error returns or successful failure cleanup.

**Returned-error input lifetime:** the repaired runner's control passed before
each fresh retain/protect/cleanup variant. The duplicate-backend chain returned
error 1, null context; all three variants then completed the existing two-second
observation and exited 0, without destroy. Protect verified `PAGE_NOACCESS`;
cleanup freed descriptor storage and released the caller device/library. This
repeats only the bounded trigger, not a universal failed-create safety guarantee.

## Remaining boundaries

The results cover the recorded machine, DLLs, provider observations and existing
workloads. Other GPUs/drivers/providers, Vulkan, general temporal image quality,
outstanding-work teardown, universal native failure cleanup and provider
termination remain unverified. The original failed sharpening gate and GBV
dispatch timeout remain open. Later concurrent production changes need their own
verification. No historical research record or earlier verification report was
updated, and no commit, publication or release was performed.

## Documentation integration and checks

This report and its evidence were added, and current references were updated in
[ROADMAP](../../ROADMAP.md), [DEVELOPMENT](../DEVELOPMENT.md), the
[research overview](../research/README.md), [API/safety](../research/api-and-safety.md),
[lifecycle](../research/context-lifecycle.md), [loading](../research/runtime-loading.md),
[threading](../research/runtime-threading.md), [construction](../research/upscaler-construction.md)
and [dispatch](../research/upscaling-dispatch.md). The sole executable change for
this task is the prerequisite-marker repair in the
[InputLifetime runner](../../crates/fsr-sdk-sys/tests/context_lifecycle/run.ps1).

Current-document local/repository links and heading anchors passed, as did
whitespace checks on this task's changes. The hash comparison of all 379
pre-existing historical files passed. The existing historical source-link
limitations and their [navigation map](../DEVELOPMENT.md#historical-command-index)
remain unchanged; this maintenance run did not rewrite their evidence.
