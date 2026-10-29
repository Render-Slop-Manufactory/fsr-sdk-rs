---
name: research-investigation
description: Conduct a requested source investigation or research audit using repository code, documentation, headers, or web sources, including architectural reasoning. Do not use for ordinary code review, implementation lookups, executable experiments, external research briefs, or integration of completed records.
---

# Research Investigation

Read [shared research guidance](../RESEARCH.md), relevant project context,
current topic findings, and the source needed to answer the question.

Define the question, baseline, scope, and evidence cut-off. Inspect the actual
implementation before treating plans or documentation as implemented behaviour.
Use primary sources for external facts, pin material versions, and capture
source locations and access dates as evidence is gathered. State unavailable
inputs and their effect on the conclusions.

Reason through the findings and relevant alternatives. Distinguish directly
supported source facts, upstream claims, inference, and unresolved questions.
Keep recommendations separate from accepted decisions and implementation status.
Code analysis and architectural reasoning can support a source record, but do
not establish native execution, GPU correctness, or performance.

Identify questions that need experiments. Use
[research-experiment](../research-experiment/SKILL.md) when a bounded probe is
within the authorized scope; otherwise report the remaining question without
running it. Preserve the evidence boundary if sources and experiments both
inform the investigation. A source investigation alone does not authorize
implementation changes or new execution requirements.

Finish through [research-record](../research-record/SKILL.md), supplying the
question, baseline, source references, findings, reasoning, disagreements,
recommendations, and limitations gathered here. Respect read-only and other
output constraints; do not automatically integrate the resulting record.
