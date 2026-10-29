# Compact experiment evidence

The [record](../2026-10-11-exp-temporal-quality-validation.md) explains the sources,
frozen protocol, execution, outcomes and evidence limits.

- `protocol.json`, `arms.txt`, `jitter.txt`: frozen workload and evaluation inputs.
- `inputs.json`: tested source, equipment and SDK file identities.
- `measurements.json`: all evaluator controls, three native property vectors,
  criteria and factorial contrasts, including favourable wrong-input rankings.
- `outcomes.json`, `run.json`, `native-observations.json`: sanitised execution
  commands/statuses, shuffled orders, full input audits, fence/readback counts
  and actually loaded module identities.
- `checks.json`, `sources.json`: infrastructure results and primary online sources.
- `retention.json`: removed bulk-file inventory identity and reanalysis boundary.
- `artifacts.json`: hashes and sizes for these selected supporting files.

Raw historical GPU images are not retained. Rerunning the frozen generator is
possible; byte-identical future GPU outputs are not guaranteed.
