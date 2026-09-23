# Decisions

This is the index of architecture decision records (ADRs). Each record keeps
the original D-number, rationale, status, consequences, and evidence. The
[project scope](../SCOPE.md) defines goals, the [roadmap](../ROADMAP.md)
tracks progress, and source and manifests establish implemented behavior.

## When to record a decision

Record choices affecting public contracts, native ownership or safety, SDK
version/binding strategy, runtime acquisition or redistribution, and platform
commitments. Small, reversible implementation details do not need an ADR.

Use the next D-number in `docs/adr/DNNN.md`. Record the date, status, context,
choice, rationale, consequences, and evidence. Status is `proposed`, `accepted`,
`rejected`, or `superseded`. Accept proposals after maintainer review or explicit
authorization for the choice; research alone does not accept a proposal.
When replacing a decision, retain its rationale, mark it superseded, and link
the replacement. Update SCOPE.md if project boundaries change.

## D001 — Separate raw bindings and ergonomic wrappers

**Status:** accepted

[Read D001](adr/D001.md)

## D002 — Pin AMD FSR SDK v2.3.0 as the initial native baseline

**Status:** accepted

[Read D002](adr/D002.md)

## D003 — Handwrite the first common ABI slice

**Status:** accepted

[Read D003](adr/D003.md)

## D004 — Load an explicit DLL with private entry points

**Status:** accepted

[Read D004](adr/D004.md)

## D005 — Own effect-independent context lifecycles with terminal teardown

**Status:** accepted

[Read D005](adr/D005.md)

## D006 — Curate ABI slices and permit reviewed generated bindings

**Status:** accepted

[Read D006](adr/D006.md)

## D007 — Keep runtime acquisition explicit and deploy the v2.3.0 DX12 upscaler DLL beside the executable

**Status:** accepted

[Read D007](adr/D007.md)

## D008 — Bound the first public DX12 upscaler construction contract

**Status:** accepted

[Read D008](adr/D008.md)

## Open choices

Concrete binding-slice selections under D006 and public APIs beyond D008 remain
undecided. D008 selects the first construction contract; implementation remains
pending. D007 resolves runtime discovery/acquisition policy for its bounded
baseline; its explicitly deferred runtime questions remain open. Record further
choices when supported by the relevant investigation; research recommendations
are not decisions.
