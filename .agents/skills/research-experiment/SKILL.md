---
name: research-experiment
description: Run a bounded code experiment, spike, or native SDK proof and preserve reproducible evidence. Use for requested investigations, not ordinary implementation or routine tests.
---

# Research Experiment

Read [shared research guidance](../RESEARCH.md), relevant topic findings, affected
source, and [code heuristics](../../../docs/CODE_HEURISTICS.md).

State the question, observable success and failure conditions, and exclusions.
Build the smallest probe that answers it. Preserve unrelated work; an experiment
does not grant extra permissions or justify unrequested dependency changes.

Check what the available platform can establish. If native execution needs an
unavailable Windows runtime or GPU, complete useful independent work and report
the missing verification rather than treating compilation or mocks as proof.

Record the actual setup, commands, inputs, results, and limitations using the
shared evidence convention. Preserve unsuccessful results and clearly identify
probe code versus production changes. Keep only useful sanitized artefacts.

Write the experiment record when recording is within the request. Integrate it
into a topic synthesis only when the task also includes integration; otherwise
report the record and remaining question. Do not accept architecture implicitly.
