# Source investigation: language migration audit

- Investigation date and evidence cut-off: 2026-10-15.
- Question: where do current language conventions differ from the updated policy,
  and how should existing material migrate?
- Baseline: the local checkout was clean before this investigation. The audit
  inventoried 1,314 tracked files and inspected current source and guidance.
- Method: tracked-file inventory, vocabulary searches, Python token inspection,
  approximate Rust/C++ identifier scanning, and manual inspection of matches and
  their producers and consumers. This is source analysis, not a complete
  dictionary-based spelling check or an execution result.
- Result: recommendations only. Existing files, contracts, decisions and roadmap
  status were not changed. This record is the only retained addition.

## Inputs and policy boundary

Sources were accessed locally on the investigation date:

- [Language policy](../../language_policy.md) and [Oxford guide](../../OXFORD_ENG.md).
- [Agent introduction](../../../.agents/INTRO.md), [scope](../../../SCOPE.md),
  [contribution rules](../../../CONTRIBUTING.md),
  [development guidance](../../DEVELOPMENT.md),
  [decisions](../../DECISIONS.md), [roadmap](../../../ROADMAP.md), and
  [research process](../PROCESS.md).
- Current [wrapper source](../../../crates/fsr-sdk/src/lib.rs),
  [raw bindings](../../../crates/fsr-sdk-sys/src/lib.rs), and
  [verification architecture](../../../tools/verification/README.md).

**Verified policy:** new files use Oxford English (`en-GB-oxendict`) for prose,
comments, Rustdoc, docstrings and human-readable messages. Identifiers, filenames,
directories, configuration keys and command/option names use American English
(`en-US`). The distinction follows the content, including inside mixed files.
Exact upstream names and established interfaces are preserved.

The policy explicitly leaves language changes in existing files to the
maintainer. Findings below are migration candidates, not claims that every
existing spelling violates a retroactive rule. This audit does not expand that
scope. Contribution rules separately preserve historical evidence and external
licence texts.

## Where languages are used

**Verified inventory:** the two crates contain 94 Rust files, eight C++ files and
11 headers. Active tooling outside its archive contains 49 Python files, one
PowerShell file, one Nix file and one additional C++ fixture. JSON, TOML and YAML
carry configuration, protocols and workflow definitions. These implementation
languages do not need migration under this spelling policy.

| Content | Observed convention | Recommended treatment |
|---|---|---|
| Current Markdown and crate READMEs | Predominantly British/Oxford prose | Keep British vocabulary; correct selected Oxford spelling and American prose leaks |
| Rust public API and native ABI | Examined names use American English, including `InputColorSpace`, `ColorEncodingHint`, `ResourceRole::Color` and native `color` fields | Retain names and ABI spelling |
| Rustdoc, comments, Python docstrings and messages | Predominantly British prose, with some non-Oxford `-ise` forms | Use Oxford prose independently of surrounding identifiers |
| Internal tests and verification helpers | Mixed American and British identifiers | Rename internal candidates with their imports and selectors |
| Colour-signal tooling | British paths, commands, environment names, helper export and parser labels | Handle as one connected migration with compatibility where needed |
| JSON and metadata | Mostly American keys, with an installed-SDK key in British English | Treat keys as contracts, descriptions as prose |
| Historical records, retained logs and archived scripts | Mixed spellings and verbatim machine output | Preserve completed evidence, paths, inventories and hashes |
| Agent guidance | English instructions; two German quoted examples in the task-prompt-review skill | Treat separately from English spelling; preserve useful quoted user wording |

The German examples occur in
[task-prompt-review](../../../.agents/skills/task-prompt-review/SKILL.md), around
lines 12 and 49. The skill also explicitly asks for feedback in the user's
language. The repository spelling policy does not establish an English-only
conversation rule. No broad translation requirement follows from these examples.

## Prose migration candidates

**Verified:** a bounded whole-word search found 23 lines across 16 current files
using the selected `-ise`/`-isation` families below. These are not all spelling
differences in the repository. The search excluded `docs/research/records/`,
`docs/release/` and `tools/verification/archive/`.

| File, with pre-migration line numbers | Candidate conversion |
|---|---|
| `CONTRIBUTING.md:40,106` | `authorise` → `authorize`; `recognise` → `recognize` |
| `ROADMAP.md:114` | `optimised` → `optimized` |
| `crates/fsr-sdk/src/query/tests.rs:179,378,490` | `initialised` → `initialized` |
| `crates/fsr-sdk/src/upscaler/common.rs:230` | `normalised` → `normalized` |
| `crates/fsr-sdk/src/upscaler/dx12/dispatch.rs:58` | `normalised` → `normalized` |
| `crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs:171` | `normalised` → `normalized` |
| `crates/fsr-sdk/src/upscaler/dx12/tests/active_render.rs:222` | `serialised` → `serialized` |
| `crates/fsr-sdk/src/upscaler/dx12/tests/depth_motion.rs:216` | `serialised` → `serialized` |
| `crates/fsr-sdk/src/upscaler/dx12/tests/utility_queries.rs:285` | `serialises` → `serializes` |
| `crates/fsr-sdk/tests/native/temporal_diagnosis.h:55` | `optimisation` → `optimization` |
| `docs/adr/D017.md:33` | `serialises` → `serializes` |
| `docs/adr/D024.md:32` | `normalised` → `normalized` |
| `tools/verification/README.md:90,464,610,621,624` | `recognise`, `initialises`, `normalised`, `serialises` → Oxford `-ize` forms |
| `tools/verification/native.py:22` | `normalises` → `normalizes` |
| `tools/verification/sdk.py:340` | `serialises` → `serializes` |
| `tools/verification/tests/test_sdk.py:205` | `normalises` → `normalizes` |

Additional manually confirmed prose candidates:

- `docs/adr/D019.md:238`: prose `GPU behavior` → `GPU behaviour`.
- `docs/research/performance.md:71`: prose `translating color` → `translating colour`.
- `crates/fsr-sdk/tests/native/recording_ahead.rs:130`: assertion message
  `color shift` → `colour shift`.

Retain `colour`, `behaviour`, `centre`, `artefact`, noun `licence`, and prose
`analyse`. Retain Oxford `organize`, `initialize`, `serialize`, `normalization`
and `synchronization`. A blanket American-to-British replacement, or a blanket
`-ize`-to-`-ise` replacement, would introduce errors. American words embedded in
API references, paths, commands, examples and the spelling guide are intentional.

## Identifier and path migration candidates

The identifier scan found candidates in 14 current source files. Python used
`tokenize.NAME`; Rust/C++ used an approximation that removed comments and double
quoted strings before scanning identifiers. The file list is a bounded inventory,
not a language parser's proof of completeness.

| Area | Existing names to consider moving to American English |
|---|---|
| `src/query/tests.rs`, `src/upscaler/dx12/tests/color_encoding.rs`, `tests/dimensions.rs` in `fsr-sdk` | Three test functions beginning `colour_` → `color_` |
| `crates/fsr-sdk/tests/native/active_render.h` | Local `colour` → `color` |
| `crates/fsr-sdk/tests/native/colour_signal.h` | `gpu_set_colour_signal` → `gpu_set_color_signal` |
| `crates/fsr-sdk/tests/native/dispatch_readback.rs` | Module `colour_signal` and test `native_colour_signal` → `color_signal`, `native_color_signal` |
| `tools/verification/colour_signal.py` | Seven `FSR_SDK_COLOUR_*` environment keyword names → `FSR_SDK_COLOR_*` |
| `tools/verification/profiles.py` | Import `colour_signal` and alias `colour_signal_cases` → `color_signal`, `color_signal_cases` |
| `tools/verification/temporal_calibration.py` | `analyse_arm`, imported `colour` → `analyze_arm`, `color` |
| `tools/verification/temporal_diagnosis.py` | `analyse_arm`, `colour`, `integral_colour` → `analyze_arm`, `color`, `integral_color` |
| `tools/verification/temporal_quality.py` | Imported/called `colour` → `color` |
| `tools/verification/tests/test_colour_signal.py` | Imported `colour_signal` → `color_signal` |
| `tools/verification/tests/test_sdk.py` | `test_header_normalisation_contract_is_unchanged` → `test_header_normalization_contract_is_unchanged` |
| `tools/verification/tests/test_temporal_diagnosis.py` | `integral_colour` and the test name containing `colour_depth` → American forms |

Six current filenames have British spelling candidates:

| Current path | Proposed basename |
|---|---|
| `crates/fsr-sdk/tests/native/colour_signal.h` | `color_signal.h` |
| `crates/fsr-sdk/tests/native/colour_signal.rs` | `color_signal.rs` |
| `tools/verification/colour_signal.py` | `color_signal.py` |
| `tools/verification/tests/test_colour_signal.py` | `test_color_signal.py` |
| `tools/verification/colour-signal-protocol.json` | `color-signal-protocol.json` |
| `tools/verification/colour-signal-controls-protocol.json` | `color-signal-controls-protocol.json` |

`analysis.py` is already correct: the noun `analysis` has the same spelling in
both conventions. The examined public wrapper names do not need renaming to
address these findings; no public Rust API break is indicated by this inventory.

## Contracts requiring coordinated treatment

**Colour-signal chain.**
[profiles.py](../../../tools/verification/profiles.py) selects
`experiment-colour-signal` and `experiment-colour-signal-controls`.
[colour_signal.py](../../../tools/verification/colour_signal.py) names the Rust
selector, protocols, environment variables, cases and parsed output markers.
[The Rust workload](../../../crates/fsr-sdk/tests/native/colour_signal.rs)
reads those environment variables and resolves the C++ export by its exact
name. [gpu.cpp](../../../crates/fsr-sdk/tests/native/gpu.cpp) includes the header.
Changing one layer alone can break execution without changing the algorithm.

Recommendation: migrate current imports, includes, selector and helper export
together. New command names, environment names, output basenames and machine
labels can use `color`; retain `colour` in surrounding descriptions. Add old
command/environment aliases where existing scripts need compatibility, resolving
them to one canonical implementation and rejecting contradictory old/new values.
Rebuild the helper so a stale DLL cannot conceal an export mismatch.

Case IDs are documented as stable. Preserve completed attempts' IDs and logs.
Either retain existing stable IDs as established interfaces or introduce new
American IDs with an explicit legacy mapping; do not silently relabel old runs.
The repository already uses `legacy_name` metadata for native case history.
Historical marker readers should continue accepting the recorded `COLOUR_*`
forms if current producers move to `COLOR_*`.

**Installed SDK provenance.**
[sdk.py](../../../tools/verification/sdk.py), around lines 254 and 333, both
validates and writes `licence_notices`. This is a machine key, so a future writer
should use `license_notices`, but a direct replacement would reject existing
installations. Recommendation: read both forms during migration, preserve all
existing expected-hash checks, reject conflicting values, and write only the new
form for new metadata. Test old, new, missing, altered and conflicting fields.
Do not weaken provenance checks or automatically rewrite historical metadata.

**Frozen evidence readers.**
[test_line_endings.py](../../../tools/verification/tests/test_line_endings.py),
line 54, reads the historical `evidence.json` key `artefacts`. Keep that exact key in the
legacy reader. Current inventories already use `artifacts.json` and
`verify_artifacts`; converting their prose descriptions to `artifacts` would be
incorrect. The same distinction applies to CLI `analyze` and prose “analyse”.

## Recommended migration order and acceptance

These batches are recommendations for maintainer selection, not implemented work.

1. **Prose spelling.** Apply the listed Oxford corrections to selected existing
   documents, Rustdoc, comments and the assertion message. Preserve code spans,
   exact names, links, quoted external material and README voice/structure.
   Check the diff and links; documentation-only corrections need no Rust test
   suite. A message change also needs a search for consumers of that exact text.
2. **Internal identifiers.** Rename isolated tests, local variables and Python
   helpers with their call sites. Check Rust test discovery and affected Python
   oracle tests. Renaming held temporal-quality helpers does not authorize
   resuming GPU quality research or changing its budgets.
3. **Connected colour-signal tooling.** Move the six paths and all current
   imports, includes, helper symbol, selector and protocols together. Select
   compatibility for CLI/environment/marker names and stable IDs explicitly.
   Verify Python oracle tests, profile plans, non-empty Rust selector discovery,
   C++/Rust builds and rebuilt-helper export resolution. A bounded native
   integration check can then confirm the cross-language wiring; it must retain
   existing numerical failures and must not imply new image-quality acceptance.
4. **Metadata compatibility.** Migrate the SDK key through compatible reading and
   canonical writing, with old/new/conflict and integrity controls. Confirm
   existing offline installations still verify. Do not redownload SDK material
   merely to change spelling.
5. **Prevent new drift.** Use a review checklist first: Oxford prose, American
   tokens, exact external names. If automation becomes useful, start with a
   small advisory checker on newly added files and explicit reviewed migration
   paths. Separate prose and identifier checks, and exclude licences, quoted
   names and historical evidence. A single dictionary over whole files cannot
   enforce this policy correctly. No new dependency or CI gate is selected here.

Keep completed research/release evidence, archived scripts, licence texts,
upstream paths and recorded hashes unchanged. Historical record filenames with
`colour` are established references; new record filenames should use American
English. Current documentation may need links updated if a live protocol moves;
historical links need a preserved compatibility path where they would otherwise
break. Frozen protocol content and numerical budgets must survive any move
unchanged, with recorded content identity checked.

## Baseline delta, reproduction and limits

The updated policy was confirmed. The audit identifies migration work but does
not accept a compatibility design or convert existing files. No SDK/native/GPU
behaviour was tested or inferred, and no external language authority was fetched;
the repository's supplied guide is the spelling authority for this audit.

To reproduce the bounded inventory, start with `git ls-files -z`, then exclude
the three historical prefixes listed above. Scan current Markdown, Rust,
Python, C++, headers, PowerShell, Nix, JSON, TOML and YAML as UTF-8. The 23-line
count uses case-insensitive whole words matching:

```text
\b(?:initial|serial|normal|optim|synchron|author|organ|recogn)is(?:e|ed|es|ing|ation)s?\b
```

For identifier/path candidates, search case-insensitively for `colour`,
`behaviour`, `artefact`, `centre`, `licence`, the verb forms
`analyse/analysed/analyses/analysing`, and the selected `-ise` families, then
inspect context. Inflections and homographs produce false positives; Python
tokens, comments, strings and exact interface references need different
treatment. The six-path count checks basenames and excludes historical paths.

The inventory deliberately separates historical evidence from current work.
The whole `docs/release/` prefix was excluded from candidate counts, so active
release-preparation prose there still needs a separate editorial pass if selected.
Searches do not establish that every remaining sentence is Oxford English, nor
that no further identifier spelling exists outside the selected vocabulary.
The report's file/line references identify the pre-migration source and will
shift as the migration proceeds.
