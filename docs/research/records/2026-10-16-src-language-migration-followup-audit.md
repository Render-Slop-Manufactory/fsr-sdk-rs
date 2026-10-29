# Source investigation: language migration follow-up audit

- Investigation date and evidence cut-off: 2026-10-16.
- Question: what language migration work remains after the prose, identifier,
  colour-signal and SDK metadata changes, including hidden project files?
- Baseline: clean checkout at the start of inspection; 1,325 tracked files.
- Method: hidden-file filesystem inventory, tracked-file text scanning, expanded
  spelling vocabulary, Python token inspection, approximate Rust/C++ identifier
  inspection, recursive JSON-key inspection and manual classification of matches.
- Result: three current prose corrections remain. Three additional British-spelled
  keys belong to frozen protocols and need preservation or a separately versioned
  successor. Existing files were not changed by this audit.

## Inputs and coverage

The [language policy](../../language_policy.md) and
[Oxford guide](../../OXFORD_ENG.md) remain the spelling authorities. This audit
follows the [agent introduction](../../../.agents/INTRO.md) and
[research process](../PROCESS.md), and revisits the
[first audit](2026-10-15-src-language-migration-audit.md).

All 1,325 tracked files were inventoried; 1,323 decoded as UTF-8 text and two were
PNG images. Twenty tracked paths contain a hidden component: thirteen current
project files and seven historical evidence/retention files. The current hidden
set comprises:

- Root `.gitignore` and `.gitattributes`.
- Nine files under `.agents/`: `INTRO.md`, shared research guidance, the six
  skill files, and the task-prompt-review `agents/openai.yaml`.
- `.github/workflows/ci.yml` and `.github/actions/setup-rust/action.yml`.

The filesystem walk included hidden and ignored entries outside the explicitly
excluded subtrees and found no additional untracked source files. The tracked
[external README](../../../external/README.md) was inspected explicitly.
`.claude/` and `.codex/` are absent from this checkout. A nested
`fsr-sdk-rs/docs/research/records/` directory tree contains no files and supplies
no language content to migrate.

Git internals, ignored build output under `target/`, downloaded third-party
contents under `external/`, and Python bytecode caches were excluded from content
migration. Those exclusions do not remove the project's tracked external README
or any tracked hidden guidance from the audit. External names, licence texts,
completed records and archived evidence were classified for preservation.

## Confirmed current prose corrections

These are migration candidates under the policy's maintainer-controlled treatment
of existing files. Locations refer to the source inspected on the audit date.

| Location | Existing prose | Oxford spelling |
|---|---|---|
| `docs/research/README.md:36` | `prioritises` | `prioritizes` |
| `docs/release/0.3.0-preparation.md:119` | `Finalise` | `Finalize` |
| `docs/release/0.3.0-preparation.md:127` | `authorised` | `authorized` |

The [research index](../README.md) is current guidance, and the
[0.3.0 preparation document](../../release/0.3.0-preparation.md) explicitly tracks
an open, unpublished release. Its location under `docs/release/` does not make
it a completed historical record.

The earlier audit's spelling families omitted `prioritise` and `finalise`, and
its blanket `docs/release/` exclusion concealed the active preparation checklist.
The earlier implementation accurately applied the listed corrections but did
not establish a complete spelling audit.

## Previously missed frozen protocol keys

Recursive JSON inspection found three additional machine-facing names:

| Protocol and location | Existing key | Name for a future schema |
|---|---|---|
| `tools/verification/temporal-calibration-protocol.json:13` | `camera.metres` | `camera.meters` |
| `tools/verification/temporal-calibration-protocol.json:16` | `plane_colour` | `plane_color` |
| `tools/verification/temporal-diagnosis-protocol.json:13` | `colours` | `colors` |

These keys were not identified in the first audit. They differ from American
machine-token spelling, but the [calibration](../../../tools/verification/temporal-calibration-protocol.json)
and [diagnosis](../../../tools/verification/temporal-diagnosis-protocol.json)
protocols are existing frozen inputs. The
[verification documentation](../../../tools/verification/README.md) identifies
them as such. The [calibration coordinator](../../../tools/verification/temporal_calibration.py)
copies and hashes its protocol, and the
[diagnosis coordinator](../../../tools/verification/temporal_diagnosis.py) records
protocol hashes for its runs and analysis.

Recommendation: preserve those files and recorded identities. If a protocol
naming migration is selected later, introduce a separately named/versioned
successor, keep the numerical definitions intact, and define compatibility
explicitly. A cosmetic key replacement in the existing files would change their
hashes. Discovering these names does not authorize resuming held quality studies.

## Remaining names that should stay

- The old `colour_signal` source paths, command names, native selector/export,
  `FSR_SDK_COLOUR_*` inputs and recorded log labels are intentional compatibility
  interfaces. Current producers have canonical American-spelled names; stable
  `colour.*` case IDs remain evidence identifiers. See the
  [chain migration verification](../../release/2026-10-15-color-signal-naming-verification.md).
- [SDK metadata](../../../tools/verification/sdk.py) now writes `license_notices`.
  `licence_notices` remains in the compatibility reader and tests by design.
- The legacy `artefacts` evidence key remains in a historical inventory reader.
  Upstream API names, Cargo/SPDX `license` tokens, `LICENSE` filenames and
  commands such as `analyze` retain their exact spelling.
- British prose words such as `colour`, `behaviour`, `licence`, `artefact`,
  `metres` and `analysers` remain appropriate. American examples in the Oxford
  guide and identifiers inside code examples are intentional.
- German quoted user phrases in the
  [task-prompt-review skill](../../../.agents/skills/task-prompt-review/SKILL.md)
  are examples and invocation/approval wording. The policy does not impose
  English-only conversations or require translating quoted user language.

No additional required migration was identified in the inspected current hidden
project guidance, workflow metadata, ignore rules or attributes. Thirty-five
historical lines matched the expanded selected `-ise` vocabulary; this is a
candidate count, not a recommendation to rewrite completed evidence.

## Method, limits and follow-up

Inventory combined `git ls-files -z` with a filesystem walk that did not suppress
hidden files. Spelling searches included the original vocabulary plus families
such as `finalise`, `prioritise`, `minimise`, `sanitise`, `summarise`, `customise`,
`characterise`, `standardise` and their inflections. American-prose candidates
were reviewed outside Markdown code fences and inline identifiers. Source
inspection separated Python names, comments and strings, with approximate
equivalent treatment for Rust/C++. JSON objects were traversed recursively,
including nested keys such as `camera.metres`.

This is vocabulary-assisted source inspection, not a complete dictionary,
grammar or natural-language proof. No builds, native/GPU checks, downloads or
external spelling research were performed. The guide's supplied Oxford
conventions govern the recommendations.

Recommended next change: apply the three prose corrections in the table, then
check their diff. Keep compatibility names and frozen protocol schemas intact.
This audit adds only this record; it does not accept a protocol revision, update
the roadmap or alter historical records.
