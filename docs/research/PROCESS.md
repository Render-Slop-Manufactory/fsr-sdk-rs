# Research Process

This document defines how research is recorded and integrated. Research supplies
evidence; it does not establish accepted architecture or implementation status.

## Baseline and evidence

Use [SCOPE.md](../../SCOPE.md) for goals, [ROADMAP.md](../../ROADMAP.md) for
status, and actual code for implemented behaviour as the project baseline.
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

An experiment additionally records the tested source state and relevant local
changes, environment, hardware when relevant, exact commands and inputs,
observed results, and reproduction limits. Preserve negative and inconclusive
results. Retain only useful, sanitized artefacts; exclude secrets and personal
machine details. Records must state the relevant observations and limits in the
files themselves. A Git revision or external CI link may supplement a record,
but must not be needed to understand its result. Link current plans, package
READMEs, and topic syntheses to these self-contained records.

Generated raw images and inputs, verbose intermediate analyses, full exports and
repeated source snapshots are temporary local diagnostics under ignored `target/`.
Permanent records normally keep the protocol/generator, relevant source/native
identities, equipment, compact measurements, outcomes and a few useful examples.
Preserve failed outcomes without automatically preserving every failed run's
entire image corpus. Any larger retained dataset needs a concrete purpose and
explicit scope. State when original raw bytes have been removed: rerunning a
synthetic experiment is possible, but exact past GPU outputs are not guaranteed.
The repository size check in [Contributing](../../CONTRIBUTING.md) enforces the
checkout budget. Temporary calibration retention is described in
[Verification](../../tools/verification/README.md#evidence-and-reanalysis).

Before retaining records or logs, normalize transient runtime addresses to
semantic labels such as `<non-null-device>` or `<allocation-1>`. Use consistent
labels within each process to preserve pointer identity, including unchanged
handle bits after destruction, and state that addresses were normalized. Preserve
null values and any address relationships needed as evidence; retain exact
addresses only when the numeric values themselves matter to the finding.
Do not normalize meaningful numeric values such as provider IDs, error codes,
hashes, hardware IDs, or versions.

Follow the [text and byte-identity rules](../../CONTRIBUTING.md#text-files-and-byte-identity):
new normalized evidence is UTF-8/LF before hashing, and byte-exact retained inputs
need scoped Git preservation. Keep original inventory hashes during historical
repairs; verify any reconstructed bytes against their recorded size and hash
before replacing a file. Do not treat this as recovery of removed raw outputs.

Preserve completed records as historical evidence. Correct clerical or privacy
issues transparently; record substantive corrections separately and link them.
Maintain current understanding in topic syntheses at `docs/research/<topic>.md`,
with an update date and links to the exact records used. Preserve disagreements
and version boundaries. No claim-ID system or separate claim ledger is required.

Start with API/safety and SDK acquisition/distribution topics when appropriate;
create only documents backed by actual work. The [research index](README.md) lists existing topics. Research recommendations do not silently change project scope,
accepted decisions, or implementation status.

## Workflow

Separate evidence gathering, record writing, and synthesis. Source investigations
analyse code, documentation, headers, web sources, or architecture; experiments
run bounded probes. Capture references and provenance during investigation, and
commands, inputs, observations, and statuses during execution. Final report
assembly uses that evidence rather than reconstructing it afterward.

Both investigation routes normally finish with a source or experiment record.
For read-only or response-only requests, deliver the report in the response
without writing files. Otherwise preserve it within the authorized repository
scope. Ordinary implementation, code review, lookups, and routine checks do not
automatically require this research workflow. Recording does not authorize new
execution, implementation, synthesis, or acceptance of a design.

1. Define a bounded question against the actual project baseline.
2. Gather source evidence or run a reproducible experiment, capturing its inputs
   and outcomes as work proceeds.
3. Assemble a self-contained record from the gathered evidence. Use the
   [source](templates/source.md) or [experiment](templates/experiment.md)
   template, omitting fields that do not apply. Disclose missing provenance and
   retain negative, inconclusive, and blocked outcomes without inventing results.
4. When saving, preserve the completed record under `records/`. Store necessary
   supporting artefacts in an adjacent directory with the record's basename.
5. When integration is requested, compare the completed record with the relevant
   current findings. Imported completed records can enter at this step, with
   intake naming and provenance checks that preserve their substantive content.
   Update or create a [topic synthesis](templates/topic.md), linking all records
   used. Do not rewrite historical observations to match newer conclusions.
6. As part of integration, update the index when topics are added or their coverage
   changes. Check links, dates, version qualifications, and that every requested
   input is accounted for.

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
