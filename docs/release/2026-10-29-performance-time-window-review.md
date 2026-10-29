# P08 temporal exclusion review — 2026-10-29

Status: **complete measurement; all five nominal CPU upper bounds below 5 us
in the retained period after a maintainer-authorized exclusion**. The original
256-block result remains inconclusive. This is a post hoc review of the same
run, not an independent confirmation. Full regression review of supplements and
final archive qualification remain open; this record grants no release approval.

## Completed run and reported interference

The fixed 256-block batch completed on 2026-10-29 at 01:06:31 CET after about
3 hours 18 minutes. All 2,660 confirmation cases and three setup runs of 11 cases
each passed; inputs were unchanged. Producer identity:
9e2409a86f0604fb4f7e01ca22641a72059a43a6bafb33e38d32d6dccd7f682a.
The [compact evidence](2026-10-29-performance-time-window-review.json) retains
candidate archive hashes, equipment, original CPU differences, original verdict,
block time bounds, excluded/retained block IDs and recalculated primary CPU/GPU
estimates. Historical P06 and the earlier 26-block P08 samples are not pooled.

Control batch: 20261028T204807Z-release-performance-9ebf070a378f.
Dispatch batch: 20261028T204914Z-native-performance-e614410b2f09.
Run: 20261028T204914Z-release-performance-reconfirm-75f83c242284.
Full original evidence remains under target/release-performance; no original
status, analysis, log, sample or artifact inventory was changed or deleted.

After completion, the maintainer reported playing Return of the Obra Dinn during
the measurement and explicitly instructed discarding disturbed measurements.
Exact game times were unavailable. An exploratory timing review had already
identified a pronounced common disturbance around 21:49–22:15 CET, weaker noise
until about 22:45 and scattered later excursions. Both Rust and C++ suffered
slowdowns; sign-changing paired differences do not isolate a wrapper slowdown.
The first 32 blocks contributed 57–70% of the total squared deviations, depending
on workload, despite comprising only 12.5% of the original blocks.

Timing data cannot prove which samples overlap the game, or prove that the
remaining period was idle. The exclusion below is a disclosed engineering review
choice based on the observed common time pattern and the maintainer's report.

## Common temporal rule

Choose **22:15 CET on 2026-10-28** (21:15 UTC), the end of the strongest previously
identified disturbance. Retain an original two-block balance group only if its
earliest case starts at or after this cutoff. Apply the same selection to every
workload, both arms and both clocks. This preserves complete paired blocks and
all original two-block arm-order balancing; it does not select individual values
by sign, magnitude or whether the resulting bound passes.

This excludes **blocks 1–34** for all five workloads. The last excluded block
finishes at 22:15:45.471 CET; block 35 starts at 22:15:45.500 CET. Retain
**blocks 35–256: 222 blocks per workload, 1,110 pairs and 2,220 primary child
processes**. Later outliers, including the large excursions around 00:38 and
00:48, remain. The cutoff was selected from the diagnosed time interval before
recalculating these subset bounds; alternative cutoffs were not searched for a
passing result. No data from other attempts is added.

The original frozen protocol prohibited post hoc exclusions. This review is an
explicit, case-specific amendment under the maintainer's subsequent instruction,
not a claim that the original protocol passed or a default filter for future runs.

## CPU estimates for the retained blocks

Each observation is the original paired process-mean difference, Rust minus C++.
Use the existing equal-weight Student t calculation at the unchanged 5,000 ns
budget. Values below are microseconds.

| Workload | Original upper bound, n=256 | Retained mean, n=222 | Retained nominal upper 95% bound |
|---|---:|---:|---:|
| Maximum render | 5.216 | 4.109 | 4.558 |
| Small render | 4.559 | 3.875 | 4.311 |
| Shrink/grow | 5.049 | 3.590 | 4.046 |
| Supplied masks | 4.747 | 4.139 | 4.470 |
| Linear HDR | 4.911 | 4.256 | 4.618 |

All five nominal upper bounds in the retained period are below 5 us. The saved
JSON contains the original 256 CPU differences per workload, the common retained
IDs and recalculated CPU/GPU summaries. Reproduce CPU numbers by selecting those
IDs and passing the resulting vectors to estimate(values, 5000) in
[performance_characterization.py](../../tools/verification/performance_characterization.py).
The original complete-run verdict is recorded alongside the new estimates.

These are nominal bounds after a selection informed by the observed data. Their
usual 95% coverage is not established after that selection, and the subset is
not a fresh confirmatory pass. CPU results concern the complete public record
call versus the matched C++ call on this recorded SDK/provider/driver; they do
not prove every-frame latency, universal performance or GPU equivalence.

## Review verification and remaining scope

The review checked SHA-256 and sizes against 2,564 retained inventory entries:
run/inputs/plan metadata, original characterization analysis and all 2,560 primary
case results. Every original CPU and GPU paired difference was cross-checked
against those process summaries before recalculation. Full readbacks were not
rehashed again during this review; the completed coordinator had already verified
the run. The original candidate/source binding and native execution results
remain unchanged. No native measurement or compilation was started.

Selection checks confirm that all 222 retained blocks are complete, belong to
whole original balance groups and use the same IDs across workloads/arms. The
common cutoff and every excluded ID are saved explicitly. JSON parsing,
recalculation, local documentation links, whitespace and repository size are
checked for this documentation/data change; no Rust tests are needed.

This closes the requested exclusion and CPU recalculation. It does not discard
or reinterpret setup/allocation/lifecycle evidence collected separately after
primary timing. Review GPU/setup/allocation/lifecycle regressions and final
source/package reconciliation under [D043](../adr/D043.md) before P08/release
closure. No further measurement is scheduled by this review.
