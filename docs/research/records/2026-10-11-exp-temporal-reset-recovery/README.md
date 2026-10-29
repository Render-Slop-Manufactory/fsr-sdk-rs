# Compact paired cut/reset evidence

These files support the adjacent [experiment record](../2026-10-11-exp-temporal-reset-recovery.md).

The `exploration-*` files preserve the single diagnostic/selection block;
`confirmation-*` preserve the three independently ordered confirmation blocks.
They contain protocols, source/input identities, arm/jitter tables, all case
outcomes and commands, per-window/per-frame measurements, native counts and
retention records. `criteria.json` retains the independently selected limits,
fixed recovery/sensitivity predictions, and known-defect discrimination margins.
`checks.json` records implementation checks, provenance and unsuccessful local
permission/collection attempts. `artifacts.json` hashes the retained files.

Native observations include exact reset frames, complete input audit totals,
actual loaded DLL identities, pre-cut output equality, cross-process output
equality and end-of-arm retained-resource memory. No full images, source
snapshots or AMD binaries are retained. Paths and transient addresses were
normalized; meaningful numeric IDs and hashes remain. Old GPU images cannot be
recovered from these hashes; a new run requires native equipment and the pinned
SDK and may produce different bytes.
