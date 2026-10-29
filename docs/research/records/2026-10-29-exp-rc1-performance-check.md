# Experiment: rc1 performance plausibility check

- Date: 2026-10-29. Maintainer-run measurement; subsequent read-only result review.
- Question: does the post-cleanup rc1 show a clear new wrapper-specific slowdown
  relative to the accepted P08 evidence on the same recorded deployment?
- Result: **122/122 execution checks pass; no clear new wrapper-specific
  regression is apparent in this small descriptive comparison.** This is a
  ten-block pilot, with no formal CPU-budget or equivalence verdict.
- Evidence: [compact measurements, provenance and commands](2026-10-29-exp-rc1-performance-check.json).
  The historical [P08 closure](../../release/2026-10-29-performance-review-closure.md)
  and its accepted limitations remain unchanged.

## Selection and execution

Before measurement, the maintainer selected a short fixed follow-up rather than
another long confirmation. The existing `experiment-performance-pilot` profile
provides exactly ten randomised paired process blocks for each of maximum render,
small render, shrink/grow, supplied masks and linear HDR. It retains the original
160 frames per timed child, 32 warm-up frames, independent C++ arm, complete
public Rust recording call, debug controls and native execution/output checks.
No producer or frozen protocol was edited for this run.

The comparison was selected as descriptive: assess Rust-minus-C++ CPU costs,
their dispersion and GPU intervals against P08, without pooling samples,
automatically extending the run or claiming a fresh precise 5-us pass. No new
percentage threshold was selected. An execution/integrity failure would invalidate
the observation; a conspicuous timing shift would require review of its cause.

Attempt: `20261029T211123Z-experiment-performance-pilot-778d3226a671`.
The run completed at **21:11:23.797–21:19:13.709 UTC**
(22:11–22:19 CET): **469.912 seconds total**, including **433.318 seconds**
of primary measurement. All 122 required cases passed, comprising preparation,
ABI/native setup, ten diagnostic children, 100 timed children and analysis.
Inputs remained unchanged. All ten blocks are retained; none was excluded.

The review ran the existing `verify_run` integrity check over the complete
**602-entry** retained inventory and passed. The compact JSON preserves every
case outcome, **121 command records**, 100 primary process summaries, all five
paired CPU/GPU vectors and estimates, input hashes, original metadata digests
and baseline comparison statistics. Original readbacks/logs remain under ignored
`target/rc1-performance-check/<attempt>/`; their hashes cannot reconstruct them
if later removed. No new compilation or native timing was started during review.

## Setup and candidate binding

Windows 11 x64/MSVC; Rust/Cargo 1.99.0, Python 3.14.1, MSVC 14.44.35207,
Windows SDK 10.0.22621.0. Rust builds use Cargo `--release --locked --offline`;
the independent C++ build uses `/O2`. The GPU is Radeon RX 9060 XT
(`1002:7590`), adapter 0, driver `32.0.32015.7029`. Observed provider:
**4.1.2 *** (`17700776142811697154`). SDK v2.3.0 headers and DLLs match P08;
the DLL trust check reports Windows Authenticode `Valid`.

Provider, adapter/driver, timer frequencies and render/jitter query results
match the recorded P08 comparison. Cargo.lock, the characterization producer
and protocol, C++ performance reference and C++ consumer are byte-identical to
P08. That does not prove identical background activity or thermal/clock state.
The instruction to pause games and the animated wallpaper is not an independent
observation that the machine was idle.

| Frozen archive | Bytes | SHA-256 |
|---|---:|---|
| fsr-sdk 0.3.0 | 318,967 | `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766` |
| fsr-sdk-sys 0.1.2 | 43,576 | `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042` |

These are the shared rc1 archives from clean packaged source revision
`6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`. The readout was performed from clean
checkout `8578d8ca1a4793b6da50cbf903ee99f2e442a8c5` before adding this record;
the measurement itself did not capture a Git revision. Its authoritative input
identity is `b44b1a6019a6c9c5ba4bd47a5f8db6047d9cc78bc179a08f1188abd3c6c01488`.

Preparation verified the selected package sources before launch. This review
also hashes every ordinary member of the frozen archives against the run's
captured source inventory: **168 wrapper / 30 raw files match**, including
`Cargo.toml.orig` mapped to each source manifest. Generated Cargo.toml, Cargo.lock
and VCS members are excluded from that source-member comparison. The preparation
file map equals the measurement's non-SDK file map; the run captures 352 total
source/external inputs and independently checks stability. Opaque preparation
and run identities need not match because the latter includes SDK/configuration.

This is optimized **matching-checkout execution**, as in P08, rather than
execution built from extracted packages. The separate
[rc1 host qualification](../../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
supplies the exact-package functional/native coverage.

## CPU comparison

Values are microseconds of additional public Rust CPU recording time versus
the independent C++ call, averaged equally across paired process blocks.
The historical column uses the accepted **222-block** P08 temporal subset;
the original 256-block result remains inconclusive. No samples are pooled.

| Workload | P08 retained mean | rc1 mean, n=10 | rc1 nominal one-sided 95% upper bound |
|---|---:|---:|---:|
| Maximum render | 4.109 | 3.361 | 4.062 |
| Small render | 3.875 | 3.414 | 5.170 |
| Shrink/grow | 3.590 | 3.009 | 3.941 |
| Supplied masks | 4.139 | 4.841 | 6.690 |
| Linear HDR | 4.256 | 4.256 | 5.237 |

The means remain on the same microsecond scale. Three are lower, HDR is effectively
unchanged at this precision, and masks is about 0.701 us higher with substantial
block dispersion. These observations do not establish an improvement or a new
mask regression. In particular, **three nominal upper bounds exceed 5 us**;
the pilot correctly leaves its budget and overall verdict null. All means being
below 5 us is not a formal within-budget result. The short-run uncertainty and
P08's post hoc subset limitations remain explicit.

## GPU comparison

Values below are mean GPU command intervals in milliseconds, using the same
primary `all` segment and equal process-block weighting. Historical means use
the same 222 retained P08 blocks for both arms. The original local P08 analysis
was checked against its retained SHA-256 before this calculation.

| Workload | P08 C++ | rc1 C++ | P08 Rust | rc1 Rust |
|---|---:|---:|---:|---:|
| Maximum render | 1.044228 | 1.067441 | 1.044299 | 1.065157 |
| Small render | 1.022336 | 1.039419 | 1.022890 | 1.039879 |
| Shrink/grow | 1.041975 | 1.063532 | 1.043946 | 1.064697 |
| Supplied masks | 1.042696 | 1.063714 | 1.043373 | 1.067628 |
| Linear HDR | 1.042982 | 1.066414 | 1.043894 | 1.065916 |

Absolute GPU means are **1.66–2.33% higher** across the two arms/workloads.
Both arms move together; this does not isolate a wrapper-specific regression
or establish a driver, wallpaper, scheduling or thermal cause. Within rc1,
Rust-minus-C++ mean GPU differences range from **-2.284 to +3.914 us**;
all five descriptive two-sided 95% intervals include zero. That is not proof of
GPU equivalence, and GPU command intervals are not end-to-end frame time/FPS.

## Source reconciliation and limits

The captured crate inventory contains 198 files versus P08's 187: 120 unchanged,
67 with changed bytes, 11 added and none removed. The JSON retains the exact
path lists. Hash differences include the documented private-module moves,
composition/test changes, documentation/metadata and line-ending normalization;
a changed hash alone does not establish changed native work.

The existing [U13](../../release/2026-10-29-resource-inspection-verification.md),
[U14](../../release/2026-10-29-common-organization-verification.md) and
[U15](../../release/2026-10-29-composition-cleanup-verification.md) records provide
the source-level impact assessment. U13/U14 preserve moved operations and public
contracts. U15 reads the same checked values/branches through borrowed frame
parameters and adds no allocation, clone, COM/SDK operation or synchronization;
performance fixture resource assembly remains outside timing. Those are source
and controlled-test findings, not proof of identical optimized machine code.
The fresh pilot now observes the changed candidate's actual timed path.

**Assessment:** the source impact evidence plus this candidate-bound follow-up
supports retaining the accepted P08 review without selecting another long run.
It does not replace P08 with a fresh confirmatory pass or widen its acceptance.
No setup-query, allocation or lifecycle supplements were repeated in this pilot;
their earlier evidence remains bounded to its recorded inputs. There is no new
temporal/image-quality, other-provider/hardware, Linux/Proton or native safety
claim. Publication simulation and release authorisation remain separate.
This record does not update topic syntheses or declare a release accepted.

## Reproduction

From the repository root, with the recorded trusted SDK and Windows/MSVC tools:

```powershell
python tools/verification/run.py run experiment-performance-pilot --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/rc1-performance-check
```

The command creates a fresh attempt; it does not start confirmation automatically.
Keep the selected checkout/SDK unchanged during execution. This profile has no
archive-selection mode: repeat the source-to-rc1 member comparison when claiming
candidate binding for another run. Use `verify-run --run <attempt>` through the
same coordinator to recheck retained integrity. Reproduce the comparison by
averaging the ten saved paired differences, and by averaging each arm's saved
process GPU means; the baseline selection and summaries are in the linked JSON.
