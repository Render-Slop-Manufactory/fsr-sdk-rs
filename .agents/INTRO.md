# Agent Introduction

This is the provider-neutral entry point shared by AGENTS.md and CLAUDE.md.
Keep communication concise. Ask when missing information materially affects the
result; otherwise proceed within the user's request.

## Context and authority

Read [SCOPE.md](../SCOPE.md) for project purpose and boundaries, and
[ROADMAP.md](../ROADMAP.md) for current status and sequencing. Source code and
Cargo manifests establish implemented behavior. Goals and plans are not evidence
of completed functionality.
Use [README.md](../README.md) as the human-facing entry point.

This project wraps AMD's official FSR SDK in two layers: faithful native bindings
in `fsr-sdk-sys`, and engine-independent Rust abstractions in `fsr-sdk`.
Research supplies evidence; recommendations are not automatically decisions or
implementation. Keep project facts and policy in human-facing documents, not
only in agent instructions.

## Task routing

Read only the routes relevant to the task, and reconsider them if scope changes.

- Every repository change: read [CONTRIBUTING.md](../CONTRIBUTING.md).
- Code changes or reviews: read [development guidance](../docs/DEVELOPMENT.md),
  relevant [decisions](../docs/DECISIONS.md), and
  [code heuristics](../docs/CODE_HEURISTICS.md),
  affected source and manifests, and relevant safety or SDK evidence.
- Planning or documentation: inspect the actual implementation and relevant
  scope sections, [ROADMAP.md](../ROADMAP.md), and relevant
  [decisions](../docs/DECISIONS.md).
- Tests, build commands, or support claims: read
  [development guidance](../docs/DEVELOPMENT.md).
- SDK, dependencies, packaging, or licensing: read
  [CONTRIBUTING.md](../CONTRIBUTING.md) and relevant
  [decisions](../docs/DECISIONS.md); inspect affected manifests,
  relevant lockfile entries, [LICENSE](../LICENSE), and pinned upstream evidence.
  Distinguish the project's licence from AMD's terms.
- Research: select the skill below and read the relevant existing topic under
  `docs/research/`. Read historical records only to resolve provenance or gaps.

## Research skills

Read the selected SKILL.md before using it. These workflows do not require a
particular agent provider or automatically delegate work.

| Task | Skill |
|---|---|
| Prepare a prompt for an external source investigation | [research-prompt](skills/research-prompt/SKILL.md) |
| Run a bounded spike or native experiment and record evidence | [research-experiment](skills/research-experiment/SKILL.md) |
| Import and integrate completed research | [research-synthesis](skills/research-synthesis/SKILL.md) |

An ordinary implementation or test run does not need a research record. A direct
request to perform source research is not merely a request to write a prompt;
conduct it using the [shared research guidance](skills/RESEARCH.md).

## Execution and authorization

- Inspect the worktree before editing and preserve unrelated changes.
- Complete the smallest change that satisfies the request. Ask about unresolved
  choices that materially change scope, public contracts, native safety, or
  distribution terms; do not repeatedly seek already-given authorization.
- Do not commit, push, publish, or change another checkout unless requested.
  Repository instructions do not grant permission to contact external parties.
- Follow the environment's permission boundaries for downloads, installations,
  and filesystem access. Do not introduce dependencies just for convenience.
- Check the paths and failure cases affected by a code change. Select Cargo
  checks from the actual manifests; do not assume every feature combination is
  supported or use `--all-features` blindly.
- Separate host compilation, Windows compilation, runtime loading, and actual
  GPU execution in reports. Never infer native or hardware support from a host
  build. Report relevant checks that could not be run.
- Documentation-only edits need link and consistency checks, not Rust tests,
  unless executable examples or build claims change.
- Review the final diff and update affected documentation without expanding the
  task into unrelated documentation maintenance.
