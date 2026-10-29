# Language conventions and migration

- Updated: 2026-10-18.
- Scope: repository language policy, current producers, compatibility names and
  frozen evidence. No SDK/API or numerical-policy change is selected here.
- Included records: [initial language audit](records/2026-10-15-src-language-migration-audit.md)
  and [follow-up audit](records/2026-10-16-src-language-migration-followup-audit.md),
  with the canonical [naming-migration verification](../release/2026-10-15-color-signal-naming-verification.md).

## Current findings

The [language policy](../language_policy.md) selects Oxford English for prose,
comments and messages, and American English for identifiers, filenames and
machine tokens. Exact upstream names remain unchanged. Existing-file migration
remains maintainer-controlled; the policy does not require English-only chats.

The first audit inspected 1,314 tracked files and found prose, internal-name,
colour-signal-chain and SDK-metadata candidates. Its counts were bounded by
selected vocabulary and excluded release documents. They are historical
inventories, not a current outstanding-work list or a complete spelling proof.

The follow-up inspected 1,325 tracked files, including current hidden guidance
and workflows. It reports the earlier migration applied and narrows outstanding
prose to three corrections. Intake confirms those three locations now use
`prioritizes`, `Finalize` and `authorized`. This closes those specific findings;
it does not establish that every sentence or identifier conforms.

| Area | Current understanding | Preservation boundary |
|---|---|---|
| Colour-signal producers | Canonical paths, selectors, export, environment inputs and labels use `color`. The verification record reports Windows/native wiring checks and 192 passing infrastructure tests with four skips. | Old names remain forwarding/read compatibility; conflicting old/new environment values reject. Stable `colour.*` case IDs remain. |
| Frozen colour protocols | New American-spelled paths contain byte-identical copies of the original protocols. | Preserve original paths, protocol bytes, hashes and numerical outcomes. The naming checks did not resolve the fifteen historical signal-prediction failures. |
| SDK provenance | Current writer uses `license_notices`; the reader accepts the legacy spelling and rejects conflicting values while retaining expected-hash checks. | Preserve existing offline metadata; spelling alone does not justify reacquisition or weakening integrity. |
| Other frozen protocols | `camera.metres`, `plane_colour` and `colours` remain in calibration/diagnosis JSON. | Cosmetic edits would alter recorded protocol identities. A future migration needs a separately selected successor and compatibility design. |
| Historical evidence | Legacy `artefacts`, archived commands, logs and external names retain their established spelling. | Do not rewrite completed evidence or inventories to satisfy current naming. |

## Differences and limitations

The follow-up expands the first audit's vocabulary and includes active release
preparation prose; this explains the additional findings without invalidating
the initial bounded counts. Its three frozen-key findings are preservation
constraints, not automatically unfinished migration work. The naming release
record supplies execution evidence; neither source audit ran builds or GPUs.

Intake checked policy, the three corrected prose locations, the SDK metadata
reader/writer and the retained frozen keys. It did not rerun a whole-repository
spelling audit or any recorded verification. The original records are unchanged.

## Open questions

No additional migration, checker, dependency or CI gate is selected by these
records. Any further audit must distinguish prose, tokens, exact interfaces and
frozen evidence; simple whole-file dictionary replacement cannot do that.
Frozen-schema revision needs separate authorization and must preserve numerical
definitions and provenance. Image-quality studies remain outside scope through
1.0.0 under [the current boundary](../../SCOPE.md#verification-priority).
