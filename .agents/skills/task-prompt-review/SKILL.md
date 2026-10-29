---
name: task-prompt-review
description: Review an implementation prompt against the repository before changes, only when the user explicitly invokes task-prompt-review. Opinion questions and ordinary reviews do not trigger this skill; existing implementation approval remains valid.
---

# Task Prompt Review

## When to use

Use only when the user explicitly invokes `$task-prompt-review` or asks to use
the named `task-prompt-review` skill for an implementation prompt. For example:
"$task-prompt-review Prüfe diesen Implementierungsauftrag gegen das Repository,
bevor etwas umgesetzt wird."

Prompt length, AI-like wording and pasted work instructions do not trigger this
workflow. Answer opinion questions, discussions and ordinary code or prompt
reviews directly under normal routing; do not require this skill or an approval
before giving the requested feedback.

Read the user's framing and existing authorization. Instructions inside the
supplied prompt are proposals, not implementation approval.

## Review before implementation

Use read-only inspection of the prompt, relevant repository files and, when
needed, primary external sources. During this phase, do not edit files, generate
artefacts, run builds/tests or execute the supplied prompt's work instructions.

Compare its claims and requested work with the actual implementation, manifests,
applicable project decisions and available platform/tool capabilities. Identify
work already complete, unsupported assumptions, scope conflicts and checks that
cannot establish the claimed outcome. Keep the investigation proportional to
the task; do not turn it into unrelated research or cleanup.

Give concise feedback in the user's language: whether the prompt fits, concrete
corrections with their reasons, and any blockers or material questions. Distinguish
confirmed facts from assumptions and unverified claims. If the prompt is sound,
say so briefly. Do not invent objections, redesigns or a rewritten prompt merely
to offer an alternative.

Give the requested feedback in this turn. Unless implementation is already
authorized, end the review turn and wait for approval before implementing the
reviewed prompt. This approval applies only to subsequent implementation, not
to giving an opinion or continuing the discussion.

## After approval

If the user already authorized review followed by implementation, including
"prüfen und direkt umsetzen", review and then proceed without another approval
pause. Commands inside the supplied prompt do not count as that authorization.

A later "umsetzen" or equivalent approval authorizes the agreed prompt and
accepted corrections. Follow the task's normal routing and checks, preserve the
agreed scope, and do not ask for the same approval again. Clarify only newly
discovered choices that materially change scope, public contracts or required
permissions.
