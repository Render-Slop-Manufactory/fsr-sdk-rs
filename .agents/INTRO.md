# Agent Introduction

This is the provider-neutral entry point shared by AGENTS.md and CLAUDE.md.
Keep communication concise. Ask when missing information materially affects the
result; otherwise proceed within the user's request.
Follow the [language policy](../docs/language_policy.md) for repository language conventions.

## Context and authority

Read [SCOPE.md](../SCOPE.md) for project purpose and boundaries.
[ROADMAP.md](../ROADMAP.md) owns current status and sequencing;
[README.md](../README.md) is the human-facing entry point. Source code and
Cargo manifests establish implemented behaviour. Goals and plans are not evidence
of completed functionality.
Before editing the root README, read and follow the
[README maintenance rules](../CONTRIBUTING.md#readme-maintenance).

This project wraps AMD's official FSR SDK in two layers: faithful native bindings
in `fsr-sdk-sys`, and engine-independent Rust abstractions in `fsr-sdk`.
Research supplies evidence; recommendations are not automatically decisions or
implementation. Keep project facts and policy in human-facing documents, not
only in agent instructions.
Verification claims must remain understandable from files included in the
published repository. Put dated commands, inputs, observations, and limits in
self-contained records; link current documentation to those records. Do not
make a Git revision, external CI page, or unpublished history necessary to
understand a result.

## Task routing

Read only the routes relevant to the task, and reconsider them if scope changes.

- Use [task-prompt-review](skills/task-prompt-review/SKILL.md) only when the user
  explicitly invokes `$task-prompt-review` or asks to use that named skill.
  Prompt length, AI-like wording, pasted instructions, opinion questions and
  ordinary reviews do not trigger it. Other requests follow normal routing;
  existing implementation approval remains valid.
- Repository changes or SDK, dependency, packaging, or licensing work: read
  [CONTRIBUTING.md](../CONTRIBUTING.md).
- Code changes or reviews: read [development guidance](../docs/DEVELOPMENT.md),
  relevant [decisions](../docs/DECISIONS.md), and
  [code heuristics](../docs/CODE_HEURISTICS.md),
  affected source and manifests, and relevant safety or SDK evidence.
- Planning, status, or documentation: inspect the actual implementation and
  relevant [decisions](../docs/DECISIONS.md). Consult
  [ROADMAP.md](../ROADMAP.md) for status or sequencing and
  [README.md](../README.md) for user-facing documentation or usage claims.
- Tests, build commands, or support claims outside code work: read
  [development guidance](../docs/DEVELOPMENT.md).
- Changes to test infrastructure, verification profiles or scripts, CI, or
  platform helpers: follow the
  [verification contribution guidance](../CONTRIBUTING.md#verification-infrastructure).
- Writing or changing Python tests in `tools/verification/tests/`, or the Python
  verification code they exercise: read the focused
  [test-authoring guide](../tools/verification/tests/README.md).
- SDK, dependencies, packaging, or licensing: read relevant
  [decisions](../docs/DECISIONS.md); inspect affected manifests,
  relevant lockfile entries, [LICENSE](../LICENSE), and pinned upstream evidence.
  Distinguish the project's licence from AMD's terms.
- Research: use the matching skill below for commissioning, investigating,
  experimenting, recording, or integrating findings. Read the
  [shared research guidance](skills/RESEARCH.md) for the workflow boundaries.

## Research skills

Read the selected SKILL.md before using it. These workflows do not require a
particular agent provider or automatically delegate work.

| Task | Skill |
|---|---|
| Prepare a prompt for an external source investigation | [research-prompt](skills/research-prompt/SKILL.md) |
| Conduct a code, documentation, web, or architecture investigation | [research-investigation](skills/research-investigation/SKILL.md) |
| Run a bounded spike or native experiment and capture evidence | [research-experiment](skills/research-experiment/SKILL.md) |
| Write a source or experiment record from gathered evidence | [research-record](skills/research-record/SKILL.md) |
| Import and integrate completed research | [research-synthesis](skills/research-synthesis/SKILL.md) |

Investigations and experiments finish through `research-record`; integration is
separate and runs only when requested. Read-only or response-only requests keep
the report in the response without writing files. An ordinary implementation,
code review, lookup, or test run does not automatically need a research record.

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
