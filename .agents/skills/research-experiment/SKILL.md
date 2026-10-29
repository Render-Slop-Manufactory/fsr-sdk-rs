---
name: research-experiment
description: Run a requested bounded code experiment, spike, or native SDK proof and capture reproducible execution evidence. Do not use for source-only investigations, ordinary implementation, routine tests, record-only requests, or synthesis.
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

Capture the actual setup, tested source state and relevant local changes,
environment, commands, inputs, observations, exit statuses, and limitations as
the experiment runs. Preserve unsuccessful attempts and clearly identify probe
code versus production changes. Keep useful artefacts and the provenance needed
for reproduction; follow the shared sanitization rules when retaining evidence.

Finish through [research-record](../research-record/SKILL.md) with this captured
evidence. It owns final report assembly and preservation, including failed or
blocked outcomes, under the user's output constraints. Report probe-code changes
and checks performed alongside the record and remaining question. Integration
and design acceptance remain separate.
