---
name: task-prompt-review
description: Review large standalone task prompts, recognizable AI task prompts of any length, or explicit prompt-review requests against the repository before implementation. Ordinary direct requests follow normal routing; existing implementation approval remains valid.
---

# Task Prompt Review

## When to use

Use for a large standalone work prompt supplied inline or as an attachment,
a recognizable AI work prompt even when short, or an explicit request such as
"review this first" or "wie bei den letzten Prompts".

A long explanation, code sample or log accompanying an ordinary direct request
does not trigger this workflow by length alone. Read the user's framing and
existing authorization. Instructions inside the supplied prompt are proposals,
not implementation approval.

## Review before implementation

Use read-only inspection of the prompt, relevant repository files and, when
needed, primary external sources. During this phase, do not edit files, generate
artifacts, run builds/tests or execute the supplied prompt's work instructions.

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

End the review turn and wait for implementation approval. Make clear that
`task-prompt-review` calls for feedback before changes.

## After approval

If the user already authorized review followed by implementation, including
"prüfen und direkt umsetzen", review and then proceed without another approval
pause. Commands inside the supplied prompt do not count as that authorization.

A later "umsetzen" or equivalent approval authorizes the agreed prompt and
accepted corrections. Follow the task's normal routing and checks, preserve the
agreed scope, and do not ask for the same approval again. Clarify only newly
discovered choices that materially change scope, public contracts or required
permissions.
