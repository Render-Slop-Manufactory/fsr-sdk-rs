# P08 independent confirmation plan — 2026-10-28

Status: **initial execution complete, CPU inconclusive; new fixed confirmation
prepared, not started**. P08 and release approval remain open. This ordinary
release-preparation record preserves the planning inputs; it does not replace
historical P06/P07 records or claim final archive qualification.

## Initial result and distance from the criterion

The maintainer-run batch started on 2026-10-27 at 21:36:39 UTC and finished at
22:10:12 UTC: 33.55 minutes total. All 515 execution checks passed with unchanged
inputs: three P07 setup runs of 11 cases each, P06 pilot 122 and confirmation 360.
The pilot selected 26 paired process blocks per workload. Initial evidence remains
under target/release-performance/20261027T213639Z-release-performance-6e3adfcf6698;
its linked dispatch batch is
20261027T213706Z-native-performance-84ab089826ae. The final confirmation is
20261027T214449Z-experiment-performance-confirm-c6b494a1e0b0.

The [retained compact evidence](2026-10-28-performance-confirmation-metrics.json)
contains source/package hashes, run identities, stage outcomes, equipment, all
130 paired CPU block differences and planning inputs. CPU estimates were
recomputed from those differences and agree with the saved analysis. Complete
native logs/readbacks remain in the local batch; this compact record supports
CPU planning, not an independent audit of every native execution or a completed
review of GPU/setup/lifecycle results.

Values below are microseconds of additional CPU recording cost, Rust minus C++.
Passing requires every one-sided 95% upper bound strictly below **5 us**.

| Workload | Measured mean | Upper 95% bound | Upper bound minus 5 us | Initial verdict |
|---|---:|---:|---:|---|
| Maximum render | 4.511 | 5.189 | +0.189 | Inconclusive |
| Small render | 3.555 | 5.083 | +0.083 | Inconclusive |
| Shrink/grow | 3.835 | 4.886 | -0.114 | Within budget |
| Supplied masks | 4.432 | 5.786 | +0.786 | Inconclusive |
| Linear HDR | 3.093 | 4.249 | -0.751 | Within budget |

All means are below the criterion. None of these findings establishes an
above-budget mean. Three uncertainty bounds are insufficient to establish the
selected within-budget outcome. Historical P06 used a different driver/provider;
a numerical change alone cannot be attributed to the Rust API transition.

## Input identity and limits

- Producer source identity:
  590b73027f16ffb12f573268832f3e77c4639cfba38b73bc3729e827e3c4b474.
- fsr-sdk 0.3.0 candidate SHA-256:
  326e163efa02292cf2390767a2f8019818b958064cbac331cb51192c0c0febf9;
  157 non-generated packaged source files matched the checkout.
- fsr-sdk-sys 0.1.2 candidate SHA-256:
  05fba7838b4c947b5ee9950e97ac1b2aa0091150057959e2072b29a682e549a5;
  30 non-generated source files matched.
- Windows x64/MSVC, DX12, adapter 0 AMD Radeon RX 9060 XT, 1002:7590;
  driver 32.0.32015.7029; provider 4.1.2 *, opaque ID 0xf5a5ca1e81001002.
- SDK v2.3.0 at 60f4ea81909200d8542eca14dccb2628b763a9a3;
  optimized Rust/C++ consumers, fixed 1920x1080 output. Primary child processes
  retain 160 frames and 32 warm-up frames; paired process means are the
  inferential units. Caller input assembly stays outside the timed public call.

Candidate source matching excludes Cargo-generated normalized manifest/lock/VCS
metadata. Measurement builds use the matching checkout, not extracted packages.
Final exact-package verification and release-impact assessment remain separate.

## Fixed length and duration

The maintainer requested a new length after discussing an additional measurement.
Keep the old result as planning evidence only and collect **256 fresh paired
blocks for every workload**, using the separate
[fixed protocol](../../tools/verification/performance-reconfirmation-protocol.json).
The existing P06 protocol and completed data remain unchanged. No new adaptive
pilot, old-sample pooling, trimming, early success or optional extension.

Planning uses the largest observed block SD, 4,562.510629 ns (small render).
The one-sided margin is t(0.95, n-1) * SD / sqrt(n). A target of 500 ns requires
at least 228 even blocks; 256 gives 470.751604 ns under the same dispersion.
The initial largest margin was 1,528.413 ns. This is a precision target, not a
power guarantee or a promise that future means/noise will remain the same.

Primary confirmation previously consumed 1,123.422661 seconds for 26 blocks.
Scaling to 256 yields 184.36 minutes; allow about **3.25 hours total** at that
pace after setup/build/supplements. The existing 1.5 planning multiplier yields
4.61 hours of primary work; the selected primary watchdog is **18,000 seconds
(five hours)**. It stops before another child starts; bounded child completion
and non-primary work can take the whole batch beyond five hours. Readbacks alone
project about 44 GB; reserve at least 50 GB free disk space.

The fixed count, new seed/order, source identity and analysis are frozen before
fresh setup/timing. Planning verifies the old confirmation inventory, candidate
source bytes and SDK identity. Fresh stages require matching SDK and native
equipment, including provider/driver/adapter. Formal estimates use only new
blocks. Both attempts remain visible; pointwise confidence on a new run does
not imply confidence control over unlimited retries. An inconclusive outcome
requires review rather than another automatic extension/retry.

## Manual start and result review

Run from the repository root in your own PowerShell terminal:

```powershell
python tools/verification/release_performance.py reconfirm --planning-batch target/release-performance/20261027T213639Z-release-performance-6e3adfcf6698 --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Keep the terminal open, machine awake and checkout/SDK unchanged. Avoid competing
builds or GPU-heavy work. The coordinator prints a fresh P08 folder immediately;
it freezes selection.json, runs three fresh P07 setup processes, then the fixed
confirmation with all supplements/audits. The dispatch batch is a linked sibling
to keep Windows compiler paths short. Send the printed status.json path after
completion or failure; the chat need not remain active. No old data is removed.

Preview without native work:

```powershell
python tools/verification/release_performance.py plan --reconfirmation
```

## Preparation verification

From tools/verification/tests:

```powershell
python -m unittest test_release_performance test_performance_characterization test_performance_reconfirmation test_infrastructure
```

**58/58 tests passed**. Focused tests cover source/archive/SDK mutation, complete paired samples, fixed
count/order/source/budget validation, missing/invalid planning evidence, precision
and runtime feasibility, independent profile composition, primary watchdog,
skipping pilot/selection, CLI routing and completed-but-inconclusive outcomes.
Every native launch in these orchestration tests is mocked. Ordinary profile
registry tests are included because a new opt-in profile was registered. Initial
sandbox TEMP access failed; rerunning the tests with normal temporary-file access
succeeded. No new native timing batch is launched by these checks.

A separate non-measuring preparation check against the actual retained P08 data
verified the old confirmation inventory and both archived source inventories,
froze and reloaded a 256-block plan, and composed the complete new profile:
2,660 cases, including 2,560 primary children. No case action was executed. The
manual command creates its own fresh plan at launch; it does not reuse this
preparation-check selection. Python syntax, JSON, local documentation links,
whitespace and the repository size budget also passed.

After manual completion, review both CPU results plus GPU/setup/allocations/
lifecycle observations, source/package identities and comparability under
[D043](../adr/D043.md). Successful execution is not release approval.
