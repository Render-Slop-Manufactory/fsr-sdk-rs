# Deferred until after 1.0.0

By explicit maintainer direction, 2026-10-18, image-quality research and custom
quality policies are outside the scope through 1.0.0. SDK access and correct
wrapper integration take priority. These items are not release prerequisites
or active remaining work. Resumption requires explicit maintainer selection
after 1.0.0; reaching that version does not select them automatically.

| Item | Deferred work and preserved evidence |
|---|---|
| V02 — Sharpening image-effect follow-up | Original effect criteria failed; [baseline](research/records/2026-09-30-exp-d014-native-sharpening.md). Direct parameter mapping and native execution remain ordinary wrapper checks. |
| V07d — Proton temporal quality regression | Selected Deck/Machine protocols retain failed image criteria and a blocked Deck jitter verdict; [comparison and launcher diagnosis](research/upscaling-dispatch.md#selected-proton-temporal-regressions). Concrete launcher or wrapper defects remain eligible for normal integration work. |
| Custom quality presets and calibration | Deferred policy portion of the former U03 scope. The direct provider-routed SDK helpers remain implemented and verified under U03; [D022](adr/D022.md). |
| Broader reconstruction studies | Further temporal quality studies and provider image-quality comparisons, including extensions beyond the completed [V01 baseline](ROADMAP_HISTORY.md#v01--temporalimage-baseline--completed-2026-10-12). |

Historical results remain evidence; deferral does not turn failed or blocked
criteria into passes. The [project scope](../SCOPE.md#verification-priority)
retains ABI, parameter/resource mapping, validation, ownership, error handling,
native execution and cleanup verification.
