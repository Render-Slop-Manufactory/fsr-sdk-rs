---
name: research-record
description: Write a self-contained source or experiment record from gathered evidence when asked to write a research record or finish a research investigation or experiment. Do not conduct new research, run probes, integrate topic syntheses, or turn routine checks into research.
---

# Research Record

Read [shared research guidance](../RESEARCH.md). Use the existing findings and
artefacts from the current investigation or supplied inputs. Read relevant
baseline material and referenced evidence only as needed to assemble an accurate
record; do not silently expand the task into fresh research or execution.

Choose `src` for code, documentation, web, or architecture source analysis, and
`exp` for a probe investigation, including failed or blocked attempts. Use the
[source](../../../docs/research/templates/source.md) or
[experiment](../../../docs/research/templates/experiment.md) template, omitting
irrelevant fields. An experiment may include source analysis; identify which
findings came from sources and which from execution. Split records only when
distinct questions or evidence boundaries make separate records useful.

Assemble a proportional, self-contained report under the conventions in the
[research process](../../../docs/research/PROCESS.md). Include the question,
scope, dates, method, baseline, identified inputs, findings, baseline delta,
limitations, and remaining questions. Experiment reports additionally use the
captured source state, environment, commands, observations, statuses, and
reproduction details. Preserve failed attempts and inconclusive or blocked
outcomes; a blocked probe must not be described as executed. State missing
provenance rather than inventing commands, dates, results, or guarantees.

Keep supported facts, upstream claims, inference, and recommendations distinct.
Preserve substantive disagreements and qualifications. Apply the process's
sanitization rules without destroying evidence needed to understand or reproduce
the findings. Use only useful supporting artefacts and reference them from the
report.

Respect the requested destination. For read-only or response-only work, return
the report without creating files. Otherwise save a uniquely named record under
`docs/research/records/` within the authorized scope, with necessary artefacts
in its adjacent directory. Preserve completed historical records; substantive
corrections belong in a separate linked record, not an overwritten result.

Check references, dates, provenance, and coverage of the requested inputs. Report
the destination and material evidence gaps. Leave topic synthesis, research-index
updates, roadmap status, and design acceptance to their separately authorized
workflows. Use [research-synthesis](../research-synthesis/SKILL.md) only when
integration is also requested.
