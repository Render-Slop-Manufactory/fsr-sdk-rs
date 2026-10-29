# rc1 performance acceptance — 2026-10-29

Status: **accepted by the maintainer; final P08 source/package reconciliation
is complete under [D043](../adr/D043.md).** Publication simulation remains the
next release gate. This acceptance does not authorise publication.

## Accepted assessment and wording

After reviewing the [rc1 follow-up](../research/records/2026-10-29-exp-rc1-performance-check.md),
the maintainer explicitly accepts the source impact assessment and combined
performance evidence for `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`:

> On the recorded deployment, the accepted P08 analysis places average additional
> public Rust CPU recording cost below 5 microseconds relative to the independent
> C++ call for all five workloads. Their pointwise, nominal one-sided 95% upper
> confidence bounds are 4.046–4.618 microseconds. The rc1 follow-up is consistent
> with that assessment. No additional GPU cost attributable to the Rust wrapper,
> or performance regression from the final internal cleanup, was detected.

These claims describe the five measured workloads on the recorded Windows
x64/MSVC + DX12 deployment: RX 9060 XT, driver `32.0.32015.7029`, provider
4.1.2 * and SDK v2.3.0. They concern average CPU recording cost and measured
GPU command intervals, not every-frame limits or an end-to-end FPS guarantee.
The maintainer accepts this bounded engineering conclusion as sufficient for
the release; another long confirmation is not required.

The CPU bounds use Student t across paired process-mean differences, with
one-sided alpha 0.05. They are confidence bounds on the **average added cost**,
not per-frame 95th percentiles, a two/three-sigma frame limit or reported p-values.
The P08 subset was selected after observing temporal disturbance; nominal 95%
coverage after that selection is unestablished. The claim must retain that
qualification when communicating confidence, as in the linked evidence.

| Workload | Accepted P08 mean, us | P08 nominal one-sided 95% upper bound, us | rc1 follow-up mean, us |
|---|---:|---:|---:|
| Maximum render | 4.109 | 4.558 | 3.361 |
| Small render | 3.875 | 4.311 | 3.414 |
| Shrink/grow | 3.590 | 4.046 | 3.009 |
| Supplied masks | 4.139 | 4.470 | 4.841 |
| Linear HDR | 4.256 | 4.618 | 4.256 |

P08 columns use 222 retained paired blocks per workload; rc1 uses ten fresh
pairs. The short follow-up is not pooled into those confidence bounds. Its own
upper bounds remain in the experiment record, including the three above 5 us.

## Evidence and final candidate binding

- The [accepted P08 review](2026-10-29-performance-review-closure.md) retains its
  original full-run inconclusive CPU results, 222-block temporal subset and
  accepted deployment/provider limitations. Its supplemental and provider
  evidence is not replaced by the short follow-up.
- The fixed rc1 follow-up passes **122/122** execution checks in **469.912 s**,
  with ten complete pairs per workload, unchanged inputs and verified retained
  inventories. All five measured mean CPU differences are below 5 us:
  **3.361, 3.414, 3.009, 4.841 and 4.256 us**.
- GPU means move about **1.66–2.33%** relative to the historical retained P08
  period across both C++ and Rust. The within-run Rust-minus-C++ intervals include
  zero for every workload; the descriptive two-sided 95% intervals are:

  | Workload | rc1 Rust-minus-C++ GPU mean, us | Two-sided 95% interval, us |
  |---|---:|---|
  | Maximum render | -2.284 | [-10.946, 6.378] |
  | Small render | 0.460 | [-7.292, 8.211] |
  | Shrink/grow | 1.165 | [-6.893, 9.223] |
  | Supplied masks | 3.914 | [-9.507, 17.335] |
  | Linear HDR | -0.497 | [-9.333, 8.339] |

  These are pointwise intervals without a selected equivalence margin. The
  shared absolute shift does not isolate a wrapper cost;
  its physical cause is not established.
- The follow-up's captured sources match **168 wrapper / 30 raw** ordinary
  members of the exact shared rc1 pair. Archive SHA-256 values are
  `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766`
  and `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042`.
  Their packaged source revision is `6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`.
- U13/U14 source-preservation evidence, U15's recorded descriptor/timing-boundary
  assessment and the fresh candidate-bound measurement reconcile the changes
  since P08. SDK inputs, lockfile, timing producer/protocol and independent C++
  reference/consumer match the comparison baseline. Non-runtime package text,
  line endings and Git metadata explain additional byte-identity changes without
  making different archive hashes equivalent.
- [Exact-package host qualification](2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification)
  separately passes Windows, Deck and Machine checks against this same pair.

## Preserved statistical and release boundaries

The acceptance records the maintainer's release judgement; it does not rewrite
samples, remove noisy blocks, pool runs or change the pilot's null formal verdict.
The short sample is not a new confirmatory 95% budget pass or a proof of exact
GPU equivalence. The numerical evidence and its uncertainty remain in the
unchanged experiment record and linked JSON.

No further performance run is selected for rc1. Changes to measured runtime,
build/dependency or SDK inputs still require the relevant impact review under
D043. Publication simulation with matching regenerated hashes, final candidate
archive cleanup, the controlled root README pass and separately authorised
publication/registry verification remain release work.
