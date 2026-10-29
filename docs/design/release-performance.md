# Release performance check for 0.3.0

- Accepted adaptation: 2026-10-27, following the maintainer's instruction to reuse
  P06/P07 and run the long batch manually, with later result review.
- Status: **closed on 2026-10-29 as a bounded review with maintainer-accepted
  exceptions**; [closure](../release/2026-10-29-performance-review-closure.md).
  Original full CPU outcomes remain inconclusive. The subsequent
  [rc1 follow-up and acceptance](../release/2026-10-29-rc1-performance-acceptance.md)
  close final source/package reconciliation; [publication simulation now passes](../release/2026-10-29-candidate-verification.md#rc1-publication-dry-run).
  [D043](../adr/D043.md) requires this check before release.
- Machine-readable selection: [release-performance-protocol.json](../../tools/verification/release-performance-protocol.json).
  Its checksum pins the unchanged P06 protocol; historical records are preserved.

## Scope and acceptance

P08 reuses P06's five matched workloads: maximum render size, smaller render size,
shrink/grow, supplied masks and linear HDR. It retains the independent native C++
reference, complete public Rust CPU call, GPU intervals, warm-up, execution/output
checks, allocations, preparation and lifecycle supplements. The Rust timed arm
now calls the public record(list, resources, parameters) directly. Caller input
assembly stays outside the timing interval; no old convenience adapter is timed.

For 0.3.0 the maintainer adopts P06's existing **5 microsecond** budget explicitly:
for every workload, the one-sided 95% upper bound of average additional Rust CPU
recording cost relative to the matched C++ call must be strictly below 5 us.
These are pointwise, model-based bounds over paired process blocks, not independent
frame samples, a per-frame maximum or a simultaneous 95% confidence band.

The initial protocol uses a fresh ten-block pilot to select an even confirmation count from 24 to 120 blocks,
subject to the existing two-hour primary measurement allowance. It freezes the
schedule before confirmation and excludes pilot data from the final estimate.
Noise can make the preferred 1 us precision infeasible; that remains visible.
The initial confirmation cannot be extended after inspecting its verdict. The
2026-10-28 amendment below selects a separate independent confirmation; it does
not extend that run or pool its samples. No early success stop is allowed. The existing 160 frames/child and 32 warm-up frames remain.

P08 also runs P07's six-query default/explicit-provider sequence in three fresh
processes: four warm-ups per arm, 32 alternating paired blocks per process.
The current setup-based API already implements this path. The report preserves
all samples, per-process medians and paired differences. Setup, GPU and lifecycle
costs remain descriptive and require regression review; no unreviewed percentage
threshold, GPU equivalence or causal enumeration-cost claim is introduced.
Earlier numbers are historical references. Different driver/provider inputs
prevent a simple attribution of changes to the Rust API alone.

The final CPU outcome is within-budget, exceeds-budget or inconclusive.
Only within-budget demonstrates the selected CPU criterion. Inconclusive is not
a speed pass. Completed evidence still requires review of CPU, GPU, setup,
allocations/lifecycle, comparability and material regressions under D043. The
script never approves or publishes a release automatically.

## Independent confirmation amendment — 2026-10-28

The initial batch completed 515/515 execution checks with unchanged inputs, but
three CPU upper bounds straddle the selected criterion. All five measured mean
costs are below 5 us. See the [dated results and planning calculation](../release/2026-10-28-performance-confirmation-plan.md).

Use **256 fresh paired blocks per workload**, unchanged frames/warm-up/timing
boundary, all five workloads and the strict 5 us criterion. The new
[fixed protocol](../../tools/verification/performance-reconfirmation-protocol.json)
is separate from the unchanged P06 protocol. At the prior worst SD of
4,562.51 ns, the one-sided Student t margin is about 470.75 ns at n=256. The
minimum even count meeting a 500 ns planning target is 228; 256 adds headroom.
This is a precision plan conditional on similar noise, not a promised speed pass.

The old 26 blocks supply dispersion, duration and equipment provenance only.
Reconfirmation freezes its new source identity, fixed count, seed/order and
analysis before fresh setup/timing. It verifies the planning run and archived
source bytes, requires matching SDK/provider/driver/adapter, then computes every
formal estimate from new blocks only. No pooling, trimming, early success or
optional extension. Keep both results visible: pointwise 95% bounds on the new
run do not imply 95% coverage over an open-ended sequence of retries. Another
inconclusive result requires review, not another automatic retry.

## Completed confirmation and temporal review — 2026-10-29

The fixed confirmation completed 2,660 cases plus three 11-case setup runs with
unchanged inputs. All means were below 5 us; the full-run CPU verdict remained
inconclusive. After reporting concurrent gaming, the maintainer authorized
excluding disturbed measurements. The [dated review](../release/2026-10-29-performance-time-window-review.md)
excludes the common early period through complete balance groups, retaining
blocks 35–256 for every workload/arm. All five nominal upper CPU bounds are below
5 us in those 222 blocks. This is a disclosed post hoc subset, not a new
independent confirmation; full original evidence and later outliers remain.
The [supplemental review](../release/2026-10-29-performance-supplement-review.md) is
complete: no additional wrapper-specific GPU/lifecycle slowdown is established,
and all 1,920 instrumented warmed calls have zero Rust allocator events. Shared
absolute GPU/context costs are higher than historical P06. The
[provider comparison](../research/records/2026-10-29-exp-provider-performance-comparison.md)
adds a small GPU difference with an interval spanning zero and clearer context
cost differences on the current driver. The maintainer accepted the remaining
differences as non-blocking for 0.3.0; [P08 is closed](../release/2026-10-29-performance-review-closure.md)
with all temporal/causal/inference limits retained. Final package qualification
is separate. No additional timing batch is selected.

## Candidate identity and orchestration

The later [rc1 plausibility check](../research/records/2026-10-29-exp-rc1-performance-check.md)
uses only the existing ten-block pilot: 122/122 execution checks pass with
unchanged inputs and source hashes matching all ordinary frozen rc1 members.
The maintainer accepts the combined source/performance assessment without
selecting another confirmation. Communicated CPU evidence includes P08's
pointwise nominal one-sided 95% upper bounds of 4.046–4.618 us in its accepted
temporal subset, retaining post-selection limits. All rc1 descriptive GPU
difference intervals include zero; no added wrapper GPU cost or cleanup
regression is detected. Neither the original protocol nor its historical
verdicts are rewritten; the short pilot has no new formal budget verdict.

[release_performance.py](../../tools/verification/release_performance.py) is the
unattended coordinator. It runs serially:

1. Package both workspace crates offline, with no-verify, or check an explicitly
   supplied pair. Match Cargo's current package inventory and every packaged
   source file against the checkout, including Cargo.toml.orig. Record archive
   hashes and the complete producer source identity. Cargo-generated normalized
   manifest/lock/VCS metadata is not qualified by this content check.
2. Freeze a 256-block plan from the verified initial result when using reconfirm,
   before any fresh measurements. The original run mode retains P06
   pilot/selection/confirmation for initial measurements.
3. Run and verify three fresh P07 setup processes, then dispatch; retain logs and
   samples and all dispatch supplements/audits, without a new pilot in reconfirm
   mode. Require the planning SDK and provider/driver/adapter equipment.
4. Verify stage inventories, passed execution, unchanged inputs, matching source
   and SDK hashes, and unchanged candidate archive bytes. Record the CPU verdict
   and keep release_disposition set to review-required.

Measurements are compiled from the matching checkout. This source-to-archive
binding is **not** an extracted-package execution or final archive qualification.
Later candidate code changes require repetition of affected measurements; metadata
changes require a documented impact assessment and candidate reconciliation.

Failure stops subsequent stages and leaves status/logs in the fresh batch folder.
An interrupted/incomplete batch cannot pass; a new invocation starts a fresh batch.
Cargo build caches may be reused, measurement results may not.

## Recorded manual invocation for reproduction

Use PowerShell from the repository root on the existing Windows/MSVC + DX12
machine, with the trusted local SDK and adapter 0 used in recent verification.
The command needs no active chat or agent:

```powershell
python tools/verification/release_performance.py reconfirm --planning-batch target/release-performance/20261027T213639Z-release-performance-6e3adfcf6698 --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

A read-only preview starts no native work:

```powershell
python tools/verification/release_performance.py plan --reconfirmation
```

Default output is a fresh folder under target/release-performance. The console
prints its path immediately. status.json retains the current/completed/failed
state, package identities, setup stages, dispatch batch path and final verdict.
setup-1.log through setup-3.log and dispatch.log contain progress; the linked
dispatch batch stores its own confirmation log and analyses. The P08 folder
stores the new selection and the planning-only provenance. Initial run mode also
retains a pilot log. It is
a sibling folder under the same output root, linked by dispatch_batch in the
P08 status file; avoiding an extra nesting level keeps MSVC paths short.
When it finishes, send the printed status.json path in chat. On failure, send
that same path; the record retains the failed stage and reason.

Keep the terminal open and the machine awake. Keep the checkout/SDK unchanged
and avoid other builds, benchmarks or GPU-heavy applications while measuring.
Readbacks and diagnostics can occupy tens of gigabytes under ignored target/;
ensure sufficient free space before starting. No files are automatically removed.

The initial P08 batch took 33.55 minutes, including a 26-block confirmation.
The selected 256-block confirmation subsequently completed in about **3 hours
18 minutes total**. Planning projected 184 minutes primary at the earlier pace;
a 1.5 multiplier gave 4.61 hours and the frozen watchdog allowed **18,000 seconds
(five hours) primary**, plus other stages and bounded child completion. For an
explicit reproduction, readbacks alone project about 44 GB; reserve at least
50 GB free. The recorded P08 review is closed; no new timing run is selected.
