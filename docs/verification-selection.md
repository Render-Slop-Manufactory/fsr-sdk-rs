# Selecting verification checks

Maintainer direction, 2026-10-29: use targeted checks during development and
batch full qualification on the finished release candidate. U13's 1,426-case
checkpoint is evidence for that change, not a minimum for subsequent work.

Select checks from the contracts actually changed, rather than every caller of
a moved function. Briefly record the selection and any deferred release checks.

| Change | Development checks |
|---|---|
| Prose or descriptive metadata | Links, consistency and text formatting. Compile examples only when their code or build claims change |
| Mechanical Rust module/type moves | Formatting, affected-crate compilation/Clippy and focused existing tests. Include relevant feature gates, public re-exports and MSRV compilation where affected. Unchanged validation/mapping bodies do not require every native format profile or package matrix |
| Runtime behaviour, mapping, ownership or ABI | Targeted regression tests and the relevant native/ABI checks for the changed contract. Widen coverage when failures or unresolved uncertainty justify it |
| Python verification tests or producers | Follow the focused [test-authoring guide](../tools/verification/tests/README.md), including its complete infrastructure-suite requirement for those changes |
| Finished release candidate | Run the complete selected source/package/native/platform qualification against the final inputs, including performance verification/review under [D043](adr/D043.md) |

Do not run overlapping broad profiles as mandatory prerequisites to another
profile that already includes those checks. After selected checks pass, stop
unless another change, failure or concrete unresolved concern warrants more.
Ordinary CI continues independently.

For 0.3.0, complete U14/U15 and documentation integration before the final
qualification batch. Later edits require an impact assessment and repetition
of affected checks, not an automatic restart of the entire batch. Earlier
results retain their original identities; deferred checks remain pending.
Performance verification remains a release requirement, not a benchmark per
internal refactor. Profile definitions and commands remain in
[Verification](../tools/verification/README.md).
