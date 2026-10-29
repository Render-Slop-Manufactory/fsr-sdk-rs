# Experiment: default and selected-provider setup-query cost

- Date: 2026-10-19.
- Question: what does the prepared six-query setup path cost before selecting
  any caching/session optimization?
- Outcome: three native optimized processes pass all 11 cases each and retain
  192 timings. Default medians are 229.55–230.74 ms; selected medians are
  696.98–707.10 ms per six-query sequence.
- No GPU dispatch, speed gate, isolated enumeration-cost claim or caching
  implementation is included.

## Setup and reproduction

Use the [Windows batch environment and exact signed runtime](../../release/2026-10-19-windows-upscaler-verification.md#equipment-and-tested-source):
Windows x64/MSVC, Rust 1.99.0 optimized builds, SDK v2.3.0, adapter 0 RX 9060 XT,
driver 32.0.31041.1004 and provider 4.1.1 (`17700776140664213505`).
The measured implementation is development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`.
The [retained checks](../../release/2026-10-19-windows-upscaler-verification/checks.json)
identify the complete commands, source identities, helper/executable hashes and
unchanged inputs. Only the test additions described in the Windows report
precede measurement; the prepared timing workload is unchanged.

Run the following three times sequentially, after other native work has finished:

```powershell
python tools/verification/run.py run experiment-provider-setup --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

Each fresh process observes a default context's provider ID, warms both routes
four times, then measures 32 alternating paired blocks. Every sample includes
ratio, quality render size, jitter phase count/offset, resource requirements and
memory estimate. Explicit selection includes fresh count/fill membership queries
and owned-name copying. Context lifetime is outside the timed sequences; the
context is destroyed explicitly. No query result cache is added.

## Measurements

| Fresh process | Default median / mean (ms) | Selected median / mean (ms) | Median paired difference (ms) |
|---|---|---|---|
| `191658Z-…-41d2ddabd41c` | 230.743 / 232.143 | 707.104 / 707.672 | 474.440 |
| `191753Z-…-3c20085df4ba` | 229.554 / 229.591 | 696.983 / 700.025 | 469.387 |
| `191856Z-…-326ec1e5d136` | 230.207 / 229.567 | 699.049 / 703.422 | 471.453 |

Every process retains exactly 32 samples per arm, native identity, completion
marker and successful terminal cleanup. All three commands exit 0. Individual
default samples range from 220.878 to 246.260 ms, selected from 677.337 to
762.838 ms. [Timings](2026-10-19-exp-provider-setup-cost/timings.csv),
[summary](2026-10-19-exp-provider-setup-cost/summary.json) and
[byte inventory](2026-10-19-exp-provider-setup-cost/inventory.json) preserve the
measurements without local paths. Means/medians describe six-query sequences,
not per-query costs or frame dispatch.

## Baseline delta and limits

P07 now has native measurements rather than compilation-only preparation.
The selected whole path is observably more expensive on this deployment.
These timings do not identify which native/backend/enumeration operations cause
the difference. An observed default ID does not reserve or freeze default routing.
The experiment defines no practical equivalence margin or statistical speed gate.
Background activity, clocks, temperature and power policy were not controlled or
recorded. No GPU work was launched by this profile, and other verification work
was completed before the timing processes. Broader machines, providers and
optimizations require separate measurement and a contract decision.
