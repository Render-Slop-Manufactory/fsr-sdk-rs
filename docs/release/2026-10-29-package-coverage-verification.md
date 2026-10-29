# Exact-package coverage verification — 2026-10-29

The missing callback/production-CB ABI and stable/MSRV example bindings are
implemented and pass against the existing P08 development archives. Standalone
feature-native and bounded Proton profiles can now select the same package pair.
No production Rust, package contents, SDK pin or public contract changed.

The [compact results](2026-10-29-package-coverage-verification.json) retain package
identities, case outcomes, compiler/runtime details, callback observations and
input identities. This is verification infrastructure work, not final release
qualification or publication authorization.

## Selected inputs and reproduction

Both Windows runs selected the unmodified archives from:

`target/release-performance/20261028T204807Z-release-performance-9ebf070a378f/candidate-build/package/`

| Package | Version | SHA-256 |
|---|---|---|
| fsr-sdk | 0.3.0 | `f55b3a2b46b96406fdbfeb82ef6ff179b3a4469b90c1198b318119b83adb204f` |
| fsr-sdk-sys | 0.1.2 | `0bb92f048d27b67d74a3876bc78b5016ac33452640ffa17cffdab77179230bb0` |

Equipment: Windows 11 x64/MSVC, Rust/Cargo 1.99.0 and Rust 1.88.0 for MSRV,
Python 3.14.1, PowerShell 7.6.9, Radeon RX 9060 XT at DXGI index 0, driver
32.0.32015.7029 and pinned SDK v2.3.0. The staged loader and upscaler DLLs pass
Authenticode checks; their hashes are retained in the JSON. The callback and
active-output checks explicitly select providers 3.1.5 and 4.1.1.

From the repository root, use the archive directory above for `<pair>`:

```sh
python tools/verification/run.py run release-coverage --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 \
  --wrapper-archive <pair>/fsr-sdk-0.3.0.crate \
  --sys-archive <pair>/fsr-sdk-sys-0.1.2.crate \
  --output-root target/package-coverage

python tools/verification/run.py run native-variable-output --artifacts --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 \
  --wrapper-archive <pair>/fsr-sdk-0.3.0.crate \
  --sys-archive <pair>/fsr-sdk-sys-0.1.2.crate \
  --output-root target/package-coverage
```

These commands are shell-neutral examples with placeholders. The actual Windows
invocations used PowerShell with `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`; the coordinator disables incremental compilation.
Cargo remained locked/offline. No benchmark or publication command was run.

## Results

| Check | Outcome |
|---|---|
| `release-coverage` | **70/70 passed**, 01:44:55–01:46:39 UTC |
| `native-variable-output --artifacts` | **66/66 passed**, 01:48:30–01:51:15 UTC |
| Targeted infrastructure unit tests | **65 passed, 3 skipped** out of 68; Windows lacks the Linux ownership/pidfd and mode/symlink test environments |
| Retained run inventories and required-case records | Both independently verified with `verify-run` |
| Selected archives and extracted contents after GPU execution | Unchanged in both runs |

Local attempts are `20261029T014454Z-release-coverage-def336992f7d` and
`20261029T014829Z-native-variable-output-b1f24c009ff5`, under
`target/package-coverage/`. Both recorded `inputs_unchanged=true`; the compact
JSON retains their summary hashes and all case outcomes without requiring those
ignored build directories to understand the result.

The focused run proves:

- Checksum-verified archive extraction and a fresh isolated local registry.
  Both downstream consumers have registry source IDs and lockfile checksums for
  the selected pair, with no path dependency fallback.
- Full packaged examples, README/Rustdoc snippets, four Windows feature
  combinations (`dx12`, `dx12,movable`, `dx12,utils`, `dx12,movable,utils`) and
  all eight separately invoked workflows on stable and Rust 1.88.0. This checks
  compilation and API availability, not renderer execution of the examples.
- Native core queries, lifecycle, dispatch/Configure/selected-provider paths,
  both ownership families and the existing fenced readback/cleanup regressions,
  with the Rust executable built from the extracted wrapper.
- Production CB aggregate exchange with independent MSVC callers and stable/MSRV
  Rust DLLs importing the extracted sys `src/api.rs`: size 56, alignment 8,
  handle offset 48, four exchanges in each direction and four negative controls
  per compiler. The raw declaration SHA-256 is
  `5984e723c7ec13e821296de1518e35add27488a41226ae90f7a3c432f16acafb`.
- Public callbacks through the extracted wrapper: 12 created/destroyed contexts
  across local/movable owners and providers 3.1.5/4.1.1, 20 fenced GPU frames,
  diagnostics, allocator forwarding and balanced host/resource/heap allocations.
  Observed counts are 24 host allocations/frees, 240 resource allocations/frees,
  368 CB allocations, 16 messages and 80 cross-set resource frees. Native heap
  callback invocation remains unobserved; controlled forwarding is checked.

The separate variable-output run exercises the generic archive-mode gate and
builder, rather than only the explicit registry builders in `release-coverage`.
It passes the existing selected-provider, both-family maximum-allocation,
active-output/motion/sharpening and independent readback checks. It adds no wider
allocation-access safety or image-quality guarantee.

## Infrastructure changes and checks

`release` now includes archive-bound public callbacks and both production CB
exchanges. `release-artifacts` uses seven supported feature configurations and
the complete stable/MSRV registry example inventory. The focused profile omits
the broader source/consumer-contract suite; passing it does not imply a full
release-profile pass.

Supported standalone native profiles accept `--artifacts`; an explicit complete
archive pair also enables this mode. Omitting the pair generates fresh packages.
Partial selections and unsupported profiles fail instead of silently building
the checkout. The final audit checks hashes, extracted file bytes and inventory,
including extra files and symlinks. The
[verification guide](../../tools/verification/README.md#exact-package-native-checks)
lists the profiles and commands, including the public generator probes whose
original observed/no-write criteria are preserved.

Unit tests cover dependency ordering and mandatory archive gates, stable/MSRV
workflow selection, native manifest/environment/binary selection, production
raw-declaration selection, partial pairs, changed archives/extractions, injected
files and Proton package/helper paths. Run from `tools/verification/tests`:

```sh
python -m unittest test_package_coverage test_infrastructure test_proton test_proton_runtime test_proton_timeout
```

Later finalization affected only documentation, the unit-test portability/partial-pair
checks and the public generator profiles' archive-mode allowlist;
the exercised package builders and native workloads were unchanged. The final
unit run checks the complete allowlist. Both real runs retain their original
tooling/input identities.

## Remaining release boundaries

Actual Linux cross-build and Proton/GPU execution against this pair were **not
run** on this Windows host. Unit tests verify that `proton`, `proton-offline`
and `proton-timeout` use the extracted workspace, isolated Cargo registry and
packaged C++ helper inputs. Fresh bounded runs on available recorded Linux/GPU
hosts remain open; historical October 13/14 results are unchanged. This adds no
native Linux/Vulkan support or new temporal-quality study.

Final release presentation/metadata must still be frozen, the resulting exact
pair fully qualified (including selected advanced feature-native checks), and
publication simulated against the same bytes. The expanded seven-configuration
extracted-package suite and full consumer-contract inventory were not executed
by these focused runs. Other advanced profiles and generator provider limitations
retain their prior evidence until the final selected-pair checks run.

All **187** crate files recorded by the
[P08 closure](2026-10-29-performance-review-closure.md) still match their recorded
hashes. This work changes verification tooling/tests and repository documentation,
not measured runtime inputs; it selects no new performance measurement. Later
production/package edits still require the documented impact assessment.
General callback shutdown/threading and native failure-recovery guarantees remain
outside the bounded evidence. Nothing was published or uploaded.
