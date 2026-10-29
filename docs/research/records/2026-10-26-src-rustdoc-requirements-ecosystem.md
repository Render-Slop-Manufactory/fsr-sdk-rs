# Source investigation: Requirements headings in the Rust ecosystem

- Investigation date and evidence cut-off: 2026-10-26.
- Question: is `Requirements` an established Rustdoc category for correctness
  obligations distinct from `Safety`, and which alternatives suit this project?
- Baseline: proposed [D041](../../adr/D041.md), following the
  [safety/correctness investigation](2026-10-26-src-safety-and-rendering-requirements.md).
  No existing public contract has been migrated or weakened.
- Method: online search followed by inspection of primary guidelines and
  library-maintained API documentation. This is a qualitative sample, not a
  census of crates.io, a popularity ranking or a numerical prevalence estimate.
- Scope: section names, their actual uses and a recommendation. No Rust source,
  native experiment, ADR acceptance or topic-synthesis integration.

## Sources and observed usage

All sources were accessed on 2026-10-26. Versions below are the versions shown
by the inspected pages; `latest` URLs and guidelines may subsequently change.
Some version-specific docs.rs URLs were unavailable to the browsing service;
the corresponding working `latest` pages identify the inspected releases.

| Primary source | Verified documentation observation |
|---|---|
| [Rust API Guidelines, C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure) | Recommend `Errors`, `Panics` and `Safety` for their respective concerns. No corresponding `Requirements`, `Correctness` or `Preconditions` category is prescribed there. |
| [Microsoft Pragmatic Rust Guidelines, M-CANONICAL-DOCS](https://microsoft.github.io/rust-guidelines/guidelines/docs/#M-CANONICAL-DOCS) | The canonical template includes free-form extended documentation, `Examples`, `Errors`, `Panics`, `Safety` and `Abort`. It does not establish a separate correctness-requirements heading. These are Microsoft's guidelines, not a Rust language requirement. |
| [Rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#markdown) | Documents a flexible prose/example structure and CommonMark support. Custom descriptive headings are possible; rendering a heading does not give it safety semantics. |
| [`egui-selectable-table` 0.7.0, `ColumnOperations`](https://docs.rs/egui-selectable-table/latest/egui_selectable_table/trait.ColumnOperations.html#requirements) | Has a `Requirements:` heading about implementing the trait to define column behaviour. This proves use of the label, not adoption of a safety/correctness taxonomy. |
| [`cargo-audit` 0.22.2 crate documentation](https://docs.rs/crate/cargo-audit/latest) | Uses `Requirements` for the minimum Rust version. This is setup documentation, not a method's rendering-correctness contract. |
| [`lilos` 1.3.0, `sleep_until`](https://docs.rs/lilos/latest/lilos/time/fn.sleep_until.html#preconditions) | This safe async function has `Preconditions` for the required task context and a separate `Cancellation` heading. An example of the alternative exists, without implying a universal meaning for that heading. |
| [Standard library, `Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html) | States ordering requirements and their logic-error status in ordinary descriptive prose; topic-specific headings include `Corollaries` and `Lexicographical comparison`. A generic correctness heading is not required to express a mandatory semantic contract. |
| [`rayon` 1.12.0, `ParallelIterator::reduce`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.reduce) | Describes associativity/identity and the consequences of unspecified reduction order in ordinary documentation, including a note. There is no `Associativity` heading on the inspected page. |
| [`tokio` 1.54.1, `spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html) | Combines `Examples`, `Panics` and a descriptive section about using non-Send values from a task. This illustrates domain-specific sections alongside conventional ones. |
| [`wgpu-hal` 30.0.1, `CommandEncoder`](https://docs.rs/wgpu-hal/30.0.1/wgpu_hal/trait.CommandEncoder.html) | Has a trait-level `Safety` section and method-specific safety sections, including state and object-liveness conditions. It does not establish a `Requirements` counterpart. |
| [`vulkano` 0.35.2, shader module](https://docs.rs/vulkano/latest/vulkano/shader/) | Separates `Shader interface` topics such as layout/alignment from `Safety`, whose subsections cover descriptors and memory accesses. Its graphics safety classification belongs to its own API; it is not proof about this wrapper's DX12 safety consequences. |

Searches included combinations of Rust/Rustdoc with `Requirements`,
`Preconditions`, `Correctness`, exact documentation-heading forms, and targeted
queries for graphics libraries and standard-library logic errors. Many results
were build prerequisites, type names or ordinary uses of the word requirements.
These were not counted as evidence of a public API section convention. The
table preserves both a positive API-heading example and contrary/common
structural alternatives rather than treating failed searches as proof of absence.

## Findings

**Verified:** `Requirements` occurs in real Rust documentation. The inspected
guidelines do not give it the standardized role of “all non-safety obligations”,
and the examples do not show agreement on that meaning. Its presence can refer
to setup prerequisites, implementation duties or application semantics.

**Inference:** calling it an established ecosystem counterpart to `Safety` would
overstate the evidence. It is a permissible project heading, but its intended
meaning needs to be stated by the project. The sample does not establish how
frequent it is across the ecosystem, nor does it show that Rust users reject it.

**Verified:** normal prose and descriptive topic sections already carry important
non-safety contracts in established APIs. `Preconditions` is also in actual use.
Neither fact makes those names a universal standard or prevents their use for
safety-related requirements elsewhere.

## Alternatives for this project

| Choice | Assessment and recommendation |
|---|---|
| Ordinary prose and input/type documentation | Prefer for a few short conditions. Directly supported by `Ord`/Rayon examples and the canonical free-form documentation slot; avoid an unnecessary second contract section. |
| `Rendering requirements` or `Input conventions` | Prefer a descriptive heading for a substantial dispatch-signal section. State mandatory matching semantics clearly. These exact names are a project recommendation, not a demonstrated ecosystem standard. Acquisition/query APIs should use a heading suited to their own content. |
| `Semantics` | Useful for units, interpretation and effects. Explain that applicable input conditions are required, since the heading alone sounds descriptive. Also a local naming choice. |
| `Requirements` | Acceptable optional grouping when the scope is clear. Keep the term explicitly about correctness in the project's convention; do not imply that the heading itself proves violations harmless. The search does not justify prescribing this spelling repository-wide. |
| `Correctness` or `Correctness requirements` | Makes the intended distinction explicit, but remains a project convention. No special ecosystem status was established by the inspected guidelines/sample. It can also blur the fact that memory safety is itself a correctness concern. |
| `Preconditions` | Established as a general contract term and found as a heading, but it can cover both safety and non-safety conditions. It does not itself communicate this project's proposed distinction and is a poor replacement for `Safety`. |
| `Errors` / `Panics` | Keep for the documented failure mechanisms. They do not cover a semantic mismatch that merely produces an unintended image or an ignored native request. |
| `Notes` / warnings | Appropriate supplementary explanations, but too weak as the sole location for mandatory conditions. |

## Recommendation for proposed D041

Retain the substantive safety/correctness distinction and the complete `Safety`
contract, including conservative native uncertainties. Prefer ordinary prose or
a descriptive section appropriate to the operation for correctness conditions.
Allow `Requirements` as one optional name rather than prescribing it as the
repository's preferred universal spelling. This is a refinement of presentation,
not a reason to move unresolved GPU/native obligations out of `Safety`.

D041 already says `Requirements` is optional and project-specific, so the search
does not contradict its stated ecosystem claim. It does weaken the rationale
for choosing that generic name ahead of more descriptive alternatives. A later
proposal revision could make the content distinction normative while leaving
non-safety heading names flexible. This follow-up records the recommendation;
it does not revise or accept D041 automatically.

## Limits and baseline delta

This adds concrete ecosystem examples and an additional independent style guide
to the earlier investigation. No whole-ecosystem prevalence claim follows from
search-engine rankings or from this selected sample. Library documentation is
evidence of naming practice, not a guarantee that every sampled safety contract
is complete or directly transferable to FSR. Native safety unknowns from the
earlier record remain unchanged. Only this new source record was added in this
follow-up; existing proposal, decision index, historical record and Rust code
were left unchanged.
