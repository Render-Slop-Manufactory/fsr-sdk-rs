# Declared offline-runtime confirmation

2026-10-14 Europe/Paris, before local import or GPU execution. Starting source
`7a113d3917955a5e8f90ae761f6d198ca7a176d1`; current changes add local preparation,
checksum verification and `proton-offline`, without Rust API or SDK-pin changes.

Question: can a prepared checksum-selected Proton/sniper stack execute the shared
query/single-frame proof with external IPv4/IPv6 routes disabled? Required outcomes:
explicit bounded local import (4 GB/100,000 entries/300-second checked deadline),
full distribution integrity before/after execution, three missing-input failures
(manifest, Proton, steamrt3) without downloads or repaired files, independent
network-isolation proof before every launcher exec, both public-query families,
GPU dispatch/fence/readback/teardown and mandatory Vulkan core/synchronization
validation without errors. Nonzero native exit, missing marker, input drift or
unavailable isolation remains failed/blocked. Preserve all attempts.

Use installed Steam Proton 10.0-4b, existing umu 1.4.4/steamrt3 sniper
3.0.20260928.262393, locked verification shell, prepared Microsoft cache and
D017 SDK v2.3.0 on the available Steam Deck. Copy Proton/runtime to ignored
target; keep the Nix launcher at its declared package path. Runtime `var/` is a
generated cache and excluded from distribution identity; host OS/driver and
external launcher dependencies are not a fully pinned closure. Local preparation
uses explicit trust in existing inputs, without claiming upstream authenticity.

Exclude network acquisition, arbitrary Proton/runtime variants, offline Rust
bootstrap, temporal acceptance, V09 forced cleanup, native Linux/Vulkan support,
redistribution and commits/publication. Record versions, exact commands, hashes,
outcomes and these limits in a compact new record. Ordinary/release profiles keep
their existing execution contracts.
