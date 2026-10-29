# P08 performance review closure — 2026-10-29

Status: **complete as a bounded performance review with maintainer-accepted
exceptions**. This closes P08 under [D043](../adr/D043.md); it does not qualify
the final archives or authorize publication of 0.3.0.

## Maintainer disposition

After the [provider comparison](../research/records/2026-10-29-exp-provider-performance-comparison.md),
the maintainer considered the remaining provider/deployment differences immaterial
for this release and requested the documentation/readiness update. This accepts
them as non-blocking for fsr-sdk 0.3.0 / fsr-sdk-sys 0.1.2 on the recorded
Windows x64/MSVC + DX12 deployment. It does not establish driver or wallpaper
causation, require a provider default change, accept a new universal threshold,
or authorize another benchmark or release publication.

The earlier authorization to exclude the disturbed common time window remains
explicit. Full-run CPU results remain inconclusive. All five recalculated
nominal CPU upper bounds are below 5 us in the retained subset, but this is a
post hoc selection with unestablished post-selection 95% coverage, not a fresh
independent confirmation. Closure records that qualified review and acceptance;
it does not rewrite the original protocol, samples, verdicts or historical records.

## Evidence reviewed

| Checkpoint | Result and remaining limit |
|---|---|
| [Initial P08](2026-10-28-performance-confirmation-plan.md) | 515/515 execution checks; 26-block CPU outcome inconclusive; used for planning only |
| [Fixed confirmation and temporal review](2026-10-29-performance-time-window-review.md) | 2,660/2,660 confirmation cases plus three 11-case setup runs, unchanged inputs; common exclusion leaves 222 balanced blocks/workload, nominal CPU upper bounds 4.046–4.618 us |
| [Supplemental review](2026-10-29-performance-supplement-review.md) | 77 supplemental and three setup logs audited; zero Rust allocator events across 1,920 warmed calls; matched GPU/lifecycle differences small or uncertain, absolute historical shifts recorded |
| [Provider probe preparation](../research/records/2026-10-29-exp-provider-performance-preparation.md) | Compile-only failure/correction preserved, 11/11 successful build cases and 65 mocked checks; no timing claims from compilation |
| [Provider comparison](../research/records/2026-10-29-exp-provider-performance-comparison.md) | 30/30 cases, eight complete pairs in 95.477 seconds; actual provider IDs verified on one current driver |

The provider comparison measured 1.047 ms FSR GPU intervals for explicit 4.1.1
and 1.059 ms for default 4.1.2 *, a +11.503 us difference with a two-sided 95%
interval of [-5.392, 28.398] us. The difference is not clearly established by
this small sample. First creation was 53.555/58.127 ms; unused destruction was
3.586/7.679 ms. The provider/selection route affects context costs more clearly
than warmed GPU time. These are C++ observations, not added Rust overhead.

Explicit 4.1.1 on the current driver still measured about 11.58% above historical
P06's maximum-render C++ GPU mean. The historical/current driver versions,
helper inputs, background/clock state and startup sequences differ. The animated
wallpaper was reported disabled for the provider run, without an isolated on/off
test. No single cause of the historical GPU/context change is established.

## Source and package binding

Primary P08 producer identity:
9e2409a86f0604fb4f7e01ca22641a72059a43a6bafb33e38d32d6dccd7f682a.
The primary matched candidate archives were:

- fsr-sdk 0.3.0:
  f55b3a2b46b96406fdbfeb82ef6ff179b3a4469b90c1198b318119b83adb204f.
- fsr-sdk-sys 0.1.2:
  0bb92f048d27b67d74a3876bc78b5016ac33452640ffa17cffdab77179230bb0.

The [closure snapshot](2026-10-29-performance-review-closure.json) records hashes
of every included compact evidence file and all 187 measured crate files.
Those crate files, including production code, package READMEs and the native
consumer, still match the fixed P08 inputs. Added provider-probe tooling and
release/research documentation change the full producer identity but do not
change measured library behaviour. No further timing is selected by this update.

P08 matched package source bytes against the checkout; it did not execute the
exact final extracted release archives. Newly generated archives still need
source/metadata reconciliation and full archive qualification. Later changes to
production code or measured consumers require an impact assessment and repetition
of affected checks/measurements. These numbers cannot silently certify such changes.

## Remaining release work

Feature/API implementation and the selected performance review are complete.
The [release checklist](0.3.0-preparation.md#before-release-closure) still requires:

1. Bind public callback execution and the production constant-buffer ABI exchange
   to the exact package pair; include all new consumer workflows and opt-in utils
   in stable/MSRV archive coverage.
2. Finish user-facing release presentation and freeze candidate metadata/package
   inputs, then run final current/MSRV and applicable Windows ABI/native checks
   through those exact archives.
3. Run publication simulation against the verified bytes. Publication and the
   subsequent root README baseline update require the separate release step.

This update integrates completed evidence and records the maintainer disposition.
It starts no native execution and changes no public API or library code. JSON,
evidence inventories/digests, source matching, local links, whitespace and the
repository size budget are checked; documentation/data changes need no Rust run.
