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

## D009 — Leave public error categories open for extension

**Status:** accepted

[Read D009](adr/D009.md)

## D010 — Own the public runtime load error in fsr-sdk

**Status:** accepted

[Read D010](adr/D010.md)

## D011 — Bound the first public DX12 upscaler dispatch contract

**Status:** accepted; M6b implemented and verified on one tested configuration;
public input representation and fixed sharpening partially superseded by D014
for 0.2.0

[Read D011](adr/D011.md)

## D012 — Keep upscaler options extensible

**Status:** accepted

[Read D012](adr/D012.md)

## D013 — Optional movable runtime/upscaler family

**Status:** accepted 2026-10-09 after soundness review and contract revisions;
caller-established native transfer/coordination assumptions explicitly accepted.
Optional API implemented; [bounded public-path verification](release/2026-10-09-d013-public-api-verification.md)
completed separately. Included in `0.2.0` by explicit 2026-10-10 maintainer
selection; combined release evidence is recorded
[here](release/2026-10-10-0.2.0-release-verification.md)

[Read D013](adr/D013.md)

## D014 — Extensible DX12 dispatch inputs and sharpening

**Status:** accepted 2026-10-07; 0.2.0 API, migration and bounded native-sharpening
trust selected. M8 implemented and checked on 2026-10-09; the direct-native M9
integration protocol and final local candidate checks passed on 2026-10-10
([M9 verification](release/2026-10-10-m9-verification.md)). The
[combined 0.2.0 release check](release/2026-10-10-0.2.0-release-verification.md)
includes the optional D013 family. Publication remains a maintainer action.
Original native effect criteria failed and stay failed.

[Read D014](adr/D014.md)

## D015 — Separate common upscaler inputs and the DX12 backend

**Status:** accepted 2026-10-09, including the follow-up that makes common inputs
available without platform/backend gates. Existing imports, native gates and
safety contracts are preserved. The later authorized loader follow-up isolates
Windows implementation and makes raw `libloading` optional and target-specific.
Vulkan remains unsupported.

[Read D015](adr/D015.md)

## D016 — Separation of Native Provider Controls and SDK-Level Policies

**Status:** accepted 2026-10-09 after consistency review and explicit maintainer
authorization. Owns the cross-feature direct-control and SDK quality-policy
invariant; existing feature contracts and release scope are unchanged.

[Read D016](adr/D016.md)

## Open choices

The dispatch-input and sharpening design in [D014](adr/D014.md) is accepted;
M7's contract decision and M8 implementation are complete. Its separate
[native integration protocol](release/2026-10-09-m9-native-sharpening-protocol.md)
passed in [M9 verification](release/2026-10-10-m9-verification.md), including
same-context mode switching and final local candidate checks. Publication remains
separate. The historical M9 projection excluded D013; the selected combined
release includes it and uses sys `0.1.1` for existing loader maintenance.
The original sharpening effect gate failed in the
[native spike](research/records/2026-09-30-exp-d014-native-sharpening.md)
and remains failed. D011 remains the published 0.1.0 dispatch baseline;
D014 selects its limited 0.2.0 replacement and provider-trust extension.

The optional movable family in [D013](adr/D013.md) is accepted after the
[soundness review](research/records/2026-10-09-src-d013-soundness-architecture-review.md).
Renderer-owned state migration motivates the separate `Send + !Sync` types.
The lasting unsafe acquisition contract carries native compatibility and
coordination assumptions; missing AMD guarantees and unobserved provider
termination remain unverified facts, not design-acceptance blockers. The optional
public family, concrete field audit and bounded public-path verification are
complete in current source; the existing D008 types stay confined to their
acquiring thread. Explicit 2026-10-10 selection includes movable in `0.2.0`;
the combined release record verifies its actual artifacts.

Concrete binding-slice selections under D006 and public APIs beyond D008/D011/D012/D013/D014
remain undecided. [D011](adr/D011.md) accepts the first public DX12 dispatch
contract; M6b implementation and bounded public-route GPU verification are
complete. D008 selects the first construction
contract; implementation and
bounded [Windows native verification](research/records/2026-09-24-exp-m5-windows-native-verification.md)
are complete. D007 resolves runtime discovery/acquisition policy for its bounded
baseline; its explicitly deferred runtime questions remain open. Record further
choices when supported by the relevant investigation; research recommendations
are not decisions.
