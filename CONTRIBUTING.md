# Contributing

This document defines the contribution workflow for people and coding agents.
[Development](docs/DEVELOPMENT.md) covers practical checks and native safety;
[decisions](docs/DECISIONS.md) records consequential technical choices.

## Before changing code

Choose work from the [rolling roadmap](ROADMAP.md). Each open item has a stable
identifier, status, prerequisites and completion checkpoint; update that item
as work progresses. Follow the [roadmap maintenance rules](#roadmap-maintenance).

Read the relevant [project scope](SCOPE.md),
[code heuristics](docs/CODE_HEURISTICS.md), and affected source. Scope defines goals and boundaries; manifests and code
establish implemented facts. Keep changes
engine-independent and within the requested scope.

Preserve unrelated work. Clarify unresolved choices that materially affect public
APIs, native safety, platform commitments, or distribution before implementing
those choices. Existing explicit authorization does not need to be repeated.
Small, reversible implementation details do not require a separate decision.

When choosing checks for a change, follow the focused
[verification selection guide](docs/verification-selection.md): targeted
development checks, then full qualification on the finished release candidate.

### Verification infrastructure

Before changing test infrastructure, verification profiles or scripts, CI, or
platform helpers, read the [verification architecture and platform boundaries](tools/verification/README.md#architecture-and-platform-boundaries)
and the affected case definitions. This requirement applies to both people and
coding agents making those changes. The verification README owns the architecture,
profile inventory and evidence contracts; [Development](docs/DEVELOPMENT.md)
provides contributor commands. Keep changes and their documentation consistent
with those responsibilities.

When writing or changing Python verification tests, or the verification code
they exercise, follow the focused
[test-authoring guide](tools/verification/tests/README.md). It covers path
contracts, portable regression fixtures and the complete infrastructure test
command; keep those rules there.

## README maintenance

The root [README](README.md) is a short introduction for someone deciding whether
to use the library. It answers: what is this, what can I use today, what do I
need, and where do I start? It is not a project status report or evidence index.

1. Preserve the agreed structure, author-written warning, note, branding and
   voice. Routine maintenance does not authorize removing, rewriting or moving
   them. Ask before changing that structure or editorial content.
   Keep current capabilities, platform requirements, installation and integration
   easy to scan. Describe planned features as planned, never as available.
2. Describe the released version used by the installation example. Put
   unreleased changes in [CHANGELOG.md](CHANGELOG.md); link there rather than
   adding progress updates to the README.
3. Keep decision and milestone IDs, internal ownership-family terminology,
   acceptance status, verification commands, dated test reports and research
   narratives out of the README. Do not link individual ADRs or evidence records
   from it. Put design rationale in [decisions](docs/DECISIONS.md), progress in
   [ROADMAP.md](ROADMAP.md), and evidence in the relevant research or release
   record.
4. State limitations in plain language when they affect a user's choice or
   setup. Keep essential runtime, platform and GPU-lifetime requirements visible;
   put detailed API and safety contracts in Rustdoc and integration examples.
5. Prefer a short explanation and a useful link over exhaustive method, type,
   feature or test inventories. Use code only to help users install or integrate;
   preserve the existing integration sketch and section layout.
6. Update the README when released behaviour, requirements or the getting-started
   path changes. Completing an investigation or test run alone is not a reason
   to add a paragraph. Revise existing text rather than appending another update.
7. Before finishing, read it as a newcomer: remove anything that requires knowing
   the project's internal history, verify relative links and version claims, and
   preserve checks for any executable examples you change.

## Roadmap maintenance

The roadmap owns status, sequencing, prerequisites and short completion
checkpoints. Keep technical rationale, detailed measurements and release history
in their existing documents, linking them from the relevant row. A planned row
does not select a crate version or accept an API design. Track order does not
select which other SDK effect ships first.

1. Keep one row per item. Preserve IDs when moving rows; never renumber M10–M16
   or reuse an archived ID. New ideas go at the bottom of the relevant track.
2. Promote a row into **Next actions**, rather than copying it. Dependencies
   refer to stable IDs, including completed prerequisites in roadmap history;
   the Needs cell also names any external requirement.
3. Update status and checkpoint as work progresses. Accept consequential
   contracts through the decision process; a roadmap row alone accepts no API,
   SDK pin, safety or platform claim.
4. A feature checkpoint needs source, ABI/consumer checks, documentation and
   native evidence appropriate to the claimed behaviour. An investigation ends
   with its bounded findings and a recorded follow-up or limitation. Discovery
   must pin the upstream surface and usable hardware/runtime requirements before
   implementation scope or public contracts are accepted.
5. Mark passed checkpoints **Verified** (optionally `✓ Verified`) and retain
   their rows until release close. At release close, move shipped/finished rows
   to [roadmap history](docs/ROADMAP_HISTORY.md), recording the release/date and
   completion evidence. Update the baseline and README coverage table; keep
   unfinished rows and their order. Completed investigations may be archived
   independently, without inventing a release assignment.
6. Append new scoped work below the remaining queue. Routine releases change
   rows and baseline metadata; they do not replace the plan with a new before/
   after-release narrative. Repeat release checks as needed without making a
   permanent milestone for every routine test run.

## Release performance verification

Before every release, complete and review a reproducible performance check of
the candidate under [D043](docs/adr/D043.md). Select workload, baseline and
regression criteria before timing; record candidate/package identities and
equipment. Resolve or obtain explicit maintainer acceptance of material
regressions. Assess subsequent edits and repeat affected measurements before
publication. A missing check is incomplete release verification; an earlier
development benchmark or ordinary release-profile pass is not a substitute.
The [release preparation](docs/release/0.3.0-preparation.md#before-release-closure)
tracks the current run and remaining protocol work.

## Naming by responsibility

Name files, modules, symbols, tests, and verification profiles by what they do,
not by milestone or decision IDs: `temporal_calibration.py`, not `v01.py`.
Keep those IDs in documentation or metadata; historical records and upstream
names retain their identifiers.

This also applies to test-helper exports and binaries, environment controls,
thread names and emitted log labels. Readers may recognize frozen historical
log labels, and stable case IDs may retain historical metadata, without using
their spelling to select behaviour. Keep historical evidence unchanged when
renaming current producers.

## Dependencies and provenance

Justify dependency or SDK changes with their purpose, alternatives, build and
maintenance cost, and relevant licence terms. Keep native dependencies and
platform requirements isolated. Do not add tooling merely for convenience.
Fetching already-declared dependencies for normal checks is ordinary development;
new downloads or installations must respect the environment's permissions.

Original project work uses the root [MPL-2.0 licence](LICENSE). Preserve external
copyright and licence notices. AMD headers, generated bindings, runtime binaries,
and other third-party material retain their applicable terms; do not assume the
project licence covers them. Record upstream versions and provenance when adding
such material, and establish required redistribution notices before shipping it.
Do not introduce implicit SDK downloads as an incidental build change: document
and decide version pinning, integrity checks, offline behaviour, and distribution
requirements first.

### Licence notices in contributions

Mark project-maintained Rust/C++ source files with
`// SPDX-License-Identifier: MPL-2.0` at the top. Use
`# SPDX-License-Identifier: MPL-2.0` for scripts and Cargo manifests (after a
shebang when present). This also applies to tests and experiment code. Mozilla
accepts this [SPDX form](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q4-i-want-to-use-the-mozilla-public-license-for-software-that-i-have-written-what-do-i-have-to-do).
Ordinary Markdown documentation is covered by the repository licence; it does
not need a boilerplate header. Do not edit generated files just to add notices.

An MPL marker does not replace upstream notices. For the current AMD-derived
declarations, preserve the source/version attribution and references to
`crates/fsr-sdk-sys/LICENSE-AMD`. Record additional source headers in that crate's
README when extending the bindings. Do not apply MPL markers to unmodified
third-party files, licence texts, or downloaded SDK contents. Inspect their own
terms before incorporating new material.

Each crate keeps a byte-identical copy of the root `LICENSE` and a package README
explaining its licensing. Keep these copies in sync. Cargo's `license` field
remains `MPL-2.0` for the existing project licensing; it does not erase retained
AMD MIT notices or offer a package-wide MIT alternative. Changes to the licensing
model or SPDX expression require an explicit decision.

When changing package contents, use `cargo package --list` and inspect a local
archive to check that licence texts and notices are included. These checks do
not publish anything. Full release validation, registry dependency versions and
any binary redistribution review belong to release preparation.

## Text files and byte identity

Use UTF-8 and LF for new project text on every platform. The root
[`.gitattributes`](.gitattributes) sets `text=auto eol=lf`; existing scoped
`-text` rules preserve archived and historical evidence bytes. Do not renormalize
those exceptions or regenerate their original inventory hashes.

Write normalized evidence as UTF-8/LF **before** calculating sizes and hashes.
Hash the files actually written, after sanitization and final formatting; never
rely on Git to fix generated line endings afterwards. Preserve verbatim inputs
separately when required by their evidence contract. Exported inventories identify
exported bytes, not the original unsanitized input bytes.

The exporter includes a scoped `.gitattributes` with `* -text` and hashes that
file too, protecting all exported bytes during Git round trips. Preserved CRLF
text uses `whitespace=cr-at-eol` for whitespace checks. When selecting
compact evidence for retention, keep its byte-preservation rule or provide an
equivalent scoped exception. A historical line-ending repair must first match
the original recorded size and SHA-256, preserve the inventories, and have a
dated correction note. Unmatched files remain unchanged.

Infrastructure tests check UTF-8/LF output, strict historical hashes and Git
round trips with Windows-style conversion settings. These rules do not change
raw-data retention or authorize deleting original inputs.

## Changes and review

Follow the [language policy](docs/language_policy.md) for repository language
conventions.
Use the heuristics as guidance rather than mechanical rules. Keep raw ABI fidelity
in `fsr-sdk-sys` and ergonomic ownership and validation in `fsr-sdk`.

For review, explain the problem, resulting behaviour, relevant checks, and material
limits. Include safety reasoning for native changes. Update affected contracts
and [decisions](docs/DECISIONS.md) when a consequential choice changes; routine
edits do not need research records or a documentation overhaul.

Before retaining logs or research artefacts, remove secrets, personal paths, and
unrelated machine details. Preserve versions, hardware details, and commands
needed to reproduce the finding. Prefer a concise sanitized result over raw logs.
Research follows the [research process](docs/research/PROCESS.md).
Before distributing candidates or publishing, follow the focused
[content review](docs/release-content-review.md) for the repository and exact
extracted package contents, including generated JSON/TXT/log evidence.

Keep generated frames, full verification exports, verbose intermediate analyses
and repeated source snapshots under ignored `target/`. Commit compact reports,
protocols, measurements and a few justified examples. Full diagnostics require a
specific purpose and should be removed after that diagnosis; ordinary consumers
must not download them. A historical GPU output is a measurement, not a promise
that another machine can regenerate identical bytes.

Retire superseded operational release preparation, duplicate verification exports
and obsolete architecture checklists when they no longer supply unique evidence.
First retain relevant release identities, outcomes and limitations in the existing
history or topic summary, and update incoming links and archive navigation. Keep
original experimental evidence for still-used safety claims and unresolved
failures. Git history holds retired operational detail; a current claim must remain
understandable from the retained summary. Do not keep empty redirect documents
solely to satisfy obsolete test-data paths; adapt those dependencies instead.

Selected release-candidate archives are a deliberate temporary exception: keep
one small frozen crate pair outside `crates/` and `docs/`, with a hash manifest,
so verification hosts obtain identical inputs. Preserve binary Git attributes
and follow the existing size limits. Remove temporary archives after qualification
and publication simulation; retain their identities in self-contained reports.
The [current release preparation](docs/release/0.3.0-preparation.md#final-verification-commands)
describes pair selection and reproduction. The 0.3.0 rc1 distribution directory
has been removed; its [preparation inventory](docs/release/2026-10-29-candidate-package-preparation.json)
and verification reports retain both archive hashes and member inventories.

Run `python tools/verification/repository_size.py` before committing. CI checks a
50 MB total checkout budget and 1 MB per file (decimal units), and rejects raw
payloads and source snapshots under `docs/`. The local check includes new files
that Git does not ignore. Change a budget deliberately with its rationale;
thousands of small generated files must not bypass the total limit. The maintainer
raised the total budget to 50 MB on 2026-10-19 to allow room for source and compact
verification evidence as the project grows; runtime bundles remain ignored local
inputs.

A change is complete when the requested behaviour is covered, relevant success and
failure cases have been checked, documentation matches the change, and checks
pass or concrete limitations are reported. Documentation-only changes normally
need link and consistency checks; verify changed commands or executable examples.

Use concise imperative commit messages. Creating commits, pushing, and publishing
are separate actions and are not implied by a request to edit files.
