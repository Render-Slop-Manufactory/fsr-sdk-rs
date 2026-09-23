# Research Process

This document defines how research is recorded and integrated. Research supplies
evidence; it does not establish accepted architecture or implementation status.

## Baseline and evidence

Use [SCOPE.md](../../SCOPE.md) for goals, [ROADMAP.md](../../ROADMAP.md) for
status, and actual code for implemented behavior as the project baseline.
Read the research index and relevant topic synthesis when they exist. Do not
assume an SDK release is pinned until the repository establishes one.

Prefer primary documentation, headers, source, and release artefacts. Record
exact SDK versions or commits and access dates when they affect findings.
Distinguish directly supported findings, upstream claims, inferences, and
unresolved questions. Source inspection cannot establish runtime correctness.
Compilation, loading, successful queries, and correct upscaling are different
results. Include licensing and redistribution findings when relevant.

## Records and current understanding

Use `docs/research/records/YYYY-MM-DD-kind-description.md`, where kind is `src`
or `exp`; choose a unique descriptive suffix if needed. Use the investigation
date when supplied, otherwise use the intake date without inventing a research
date. Paths here are relative to the repository root.

Keep each record self-contained and proportional to the investigation:

- question, scope, date, method, and pinned inputs;
- findings with direct evidence and explicit limitations;
- what changed relative to the supplied baseline;
- unresolved questions and follow-up work.

An experiment additionally records revision and relevant uncommitted changes,
environment, hardware when relevant, exact commands and inputs, observed results,
and reproduction limits. Preserve negative and inconclusive results. Retain only
useful, sanitized artefacts; exclude secrets and personal machine details.

Before retaining records or logs, normalize transient runtime addresses to
semantic labels such as `<non-null-device>` or `<allocation-1>`. Use consistent
labels within each process to preserve pointer identity, including unchanged
handle bits after destruction, and state that addresses were normalized. Preserve
null values and any address relationships needed as evidence; retain exact
addresses only when the numeric values themselves matter to the finding.
Do not normalize meaningful numeric values such as provider IDs, error codes,
hashes, hardware IDs, or versions.

Preserve completed records as historical evidence. Correct clerical or privacy
issues transparently; record substantive corrections separately and link them.
Maintain current understanding in topic syntheses at `docs/research/<topic>.md`,
with an update date and links to the exact records used. Preserve disagreements
and version boundaries. No claim-ID system or separate claim ledger is required.

Start with API/safety and SDK acquisition/distribution topics when appropriate;
create only documents backed by actual work. The [research index](README.md) lists existing topics. Research recommendations do not silently change project scope,
accepted decisions, or implementation status.

## Workflow

1. Define a bounded question against the actual project baseline.
2. Gather source evidence or run a reproducible experiment. Use the
   [source](templates/source.md) or [experiment](templates/experiment.md)
   template, omitting fields that do not apply.
3. Preserve the completed record under `records/`. Store necessary supporting
   artefacts in an adjacent directory with the record's basename.
4. When integrating, compare the record with the relevant current findings.
   Update or create a [topic synthesis](templates/topic.md), linking all records
   used. Do not rewrite historical observations to match newer conclusions.
5. Update the index when topics are added or their coverage changes. Check links,
   dates, version qualifications, and that every requested input is accounted for.

Use these evidence labels where conclusions might otherwise be ambiguous:
**Verified** (directly supported by identified primary evidence), **Upstream
claim** (reported but not reproduced here), **Inference**, and **Unresolved**.
State separately when a question needs an experiment. A verified source fact
is not a verified runtime result.

Self-contained external briefs must include the relevant baseline, precise
questions, exclusions, and this process's evidence requirements; local file
links alone are insufficient for a researcher without repository access.

Ordinary implementation and routine checks do not require research records.
Templates are aids, not evidence. Keep records proportional to the question.
