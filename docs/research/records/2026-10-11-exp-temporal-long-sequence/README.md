# Compact continuous-history evidence

These files support the adjacent [experiment record](../2026-10-11-exp-temporal-long-sequence.md).

- `protocol.json`, `arms.txt`, `jitter.txt`: frozen duration and treatment inputs.
- `inputs.json`: source/configuration file identities.
- `measurements.json`: all seven windows in three blocks, both arms' per-frame
  trajectories, known-defect evaluator outcomes and explicit pass/failure fields.
- `native-observations.json`: complete input audit totals, dispatch/fence/readback
  counts, reset frames, process orders, loaded module identities and retained
  GPU/host memory observations.
- `outcomes.json`, `run.json`: all 21 cases, commands and source-freeze result.
- `checks.json`: implementation checks and unsuccessful fixture/permission attempts.
- `retention.json`: verified removal of temporary full diagnostics.
- `artifacts.json`: byte counts and SHA256 identities for the retained files.

Personal paths and transient native addresses were normalized to semantic labels.
No full input/output images or source snapshots are retained. Exact past GPU
images cannot be recovered from these hashes. A new run requires the pinned SDK
and native equipment; reproducible input generation does not promise identical
GPU output on another configuration.
