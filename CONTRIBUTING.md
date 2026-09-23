# Contributing

This document defines the contribution workflow for people and coding agents.
[Development](docs/DEVELOPMENT.md) covers practical checks and native safety;
[decisions](docs/DECISIONS.md) records consequential technical choices.

## Before changing code

Read the relevant [project scope](SCOPE.md),
[code heuristics](docs/CODE_HEURISTICS.md), and affected source. Scope defines goals and boundaries; manifests and code
establish implemented facts. Keep changes
engine-independent and within the requested scope.

Preserve unrelated work. Clarify unresolved choices that materially affect public
APIs, native safety, platform commitments, or distribution before implementing
those choices. Existing explicit authorization does not need to be repeated.
Small, reversible implementation details do not require a separate decision.

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
and decide version pinning, integrity checks, offline behavior, and distribution
requirements first.

### License notices in contributions

Mark project-maintained Rust/C++ source files with
`// SPDX-License-Identifier: MPL-2.0` at the top. Use
`# SPDX-License-Identifier: MPL-2.0` for scripts and Cargo manifests (after a
shebang when present). This also applies to tests and experiment code. Mozilla
accepts this [SPDX form](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q4-i-want-to-use-the-mozilla-public-license-for-software-that-i-have-written-what-do-i-have-to-do).
Ordinary Markdown documentation is covered by the repository license; it does
not need a boilerplate header. Do not edit generated files just to add notices.

An MPL marker does not replace upstream notices. For the current AMD-derived
declarations, preserve the source/version attribution and references to
`crates/fsr-sdk-sys/LICENSE-AMD`. Record additional source headers in that crate's
README when extending the bindings. Do not apply MPL markers to unmodified
third-party files, license texts, or downloaded SDK contents. Inspect their own
terms before incorporating new material.

Each crate keeps a byte-identical copy of the root `LICENSE` and a package README
explaining its licensing. Keep these copies in sync. Cargo's `license` field
remains `MPL-2.0` for the existing project licensing; it does not erase retained
AMD MIT notices or offer a package-wide MIT alternative. Changes to the licensing
model or SPDX expression require an explicit decision.

When changing package contents, use `cargo package --list` and inspect a local
archive to check that license texts and notices are included. These checks do
not publish anything. Full release validation, registry dependency versions and
any binary redistribution review belong to release preparation.

## Changes and review

Write code, comments, and documentation in English; preserve upstream API names.
Use the heuristics as guidance rather than mechanical rules. Keep raw ABI fidelity
in `fsr-sdk-sys` and ergonomic ownership and validation in `fsr-sdk`.

For review, explain the problem, resulting behavior, relevant checks, and material
limits. Include safety reasoning for native changes. Update affected contracts
and [decisions](docs/DECISIONS.md) when a consequential choice changes; routine
edits do not need research records or a documentation overhaul.

Before retaining logs or research artefacts, remove secrets, personal paths, and
unrelated machine details. Preserve versions, hardware details, and commands
needed to reproduce the finding. Prefer a concise sanitized result over raw logs.
Research follows the [research process](docs/research/PROCESS.md).

A change is complete when the requested behavior is covered, relevant success and
failure cases have been checked, documentation matches the change, and checks
pass or concrete limitations are reported. Documentation-only changes normally
need link and consistency checks; verify changed commands or executable examples.

Use concise imperative commit messages. Creating commits, pushing, and publishing
are separate actions and are not implied by a request to edit files.
