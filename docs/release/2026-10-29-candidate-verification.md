# 0.3.0 candidate verification — 2026-10-29

**The same frozen rc1 pair now passes all three host qualifications: Windows
1,448/1,448, Deck 16/16 offline plus 17/17 timeout, and Machine 12/12.** These
are **1,493 required case checks across ten stages**, not 1,493 distinct tests.
Earlier source/archive results retain their original inputs and provider limits.
This is the consolidated review of those results and the closed performance review.
Release-facing package documentation and descriptive metadata are now ready.
The top-level README was subsequently updated and approved on 2026-10-30;
the [shared rc1 pair](#shared-candidate-pair) is qualified on the recorded hosts.
The [rc1 performance acceptance](2026-10-29-rc1-performance-acceptance.md)
now closes P08 input/impact reconciliation. Publication simulation now passes.
The [clean-source package reproduction](#rc1-package-byte-reproduction) now
regenerates both exact rc1 archives successfully; the subsequent
[publication dry-run](#rc1-publication-dry-run) verifies both packages, aborts
both uploads and regenerates matching bytes again. Nothing is published.

The subsequent [U13 resource inspection checkpoint](2026-10-29-resource-inspection-verification.md)
passes 1,419 source/native and seven focused archive checks, plus Cargo package
verification, after the private module reorganization.
[U14 common organization](2026-10-29-common-organization-verification.md) now
passes targeted Stable/MSRV source checks.
[U15 composition cleanup](2026-10-29-composition-cleanup-verification.md) also
passes targeted compiler, controlled-test and documentation checks. The internal
cleanup is complete and the [fresh functional batch](#post-cleanup-windows-functional-qualification)
now verifies its current source/native integration. Earlier results retain
their original source/archive identities; no source-based pass certifies the
eventual shipping archive pair.

The original [machine-readable results](2026-10-29-candidate-verification.json) retain
every original archive-batch case outcome, build/package identities, generator failures,
input stability, readback counts and cross-host archive reconciliation. Host
records linked below retain the detailed Proton equipment and diagnostics.
The combined `release` profile was not rerun: the original archive batch filled
its remaining package/feature coverage. The later functional batch below uses
the source-based composition and selected native profiles without duplicating
standalone `ci`/`msrv` runs or creating final archives.

## Shared rc1 cross-host qualification

All three incoming reports name the exact two archive hashes in the
[shared-pair table](#shared-candidate-pair). Their packaged source revision is
`6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`, SDK v2.3.0; both VCS members
identify that clean revision. Later report/distribution commits do not alter
the frozen package bytes. The earlier Windows/Linux archive mismatch below
does **not** apply to rc1.

| Host/report | Required case checks | Scope established |
|---|---:|---|
| [Windows](2026-10-29-rc1-windows-verification.md) | **1,448/1,448** | Extracted packages and isolated-registry consumers; supported feature combinations, Stable/MSRV, examples/utils, independent ABI, public callbacks/production CB and selected native regressions |
| [Deck](2026-10-29-rc1-deck-verification.md) | **16/16 + 17/17** | Exact-package Windows cross-build under Proton; both query families, fenced single-frame readback, enforced offline execution and forced-timeout containment/cleanup |
| [Machine](2026-10-29-rc1-steam-machine-verification.md), [compact evidence](2026-10-29-rc1-steam-machine-verification.json) | **12/12** | Exact-package Windows cross-build under Proton; both query families, fenced single-frame readback, destruction and PID containment; launcher networking acknowledged |
| **Total** | **1,493/1,493** | Three recorded host configurations, ten stages; no pooling with historical batches |

All required cases pass with unchanged inputs, archives and extracted bytes.
The Windows report retains every case outcome, 1,627 command records and
independent inventory-verification receipts. Its seven stages are core **630**,
motion **106**, formats **170**, depth **86**, BGRA/sRGB **106**, resource
formats **284** and variable output **66**. Core combines the existing package
and coverage protocols, counting shared case IDs once; their actions and oracles
are unchanged. Standalone overlapping source CI/MSRV profiles were not repeated.
The Windows source/SDK/archive content identity is
`e2e0447ffde0ca00b6152a2923d8636b10e75a98428334d4742afd4a91f18a41`.

The Deck report retains its required cases, commands, identities, equipment,
observations and inventory digests; Machine additionally provides structured
case outcomes in the linked JSON. Both hosts report successful `verify-run`
checks. This synthesis compares those retained records; it does not independently
re-read their remote ignored raw inventories on Windows or reconstruct absent
payloads. No substantive conflict between the three results was found.

Intake screening of all three reports and Machine's JSON finds no matches for
the documented private-key, token, email or private-workspace patterns. This
bounded text screen does not certify all credential formats or Git history.

Windows uses Radeon RX 9060 XT (`1002:7590`), driver `32.0.32015.7029`,
Rust/Cargo 1.99.0 and exact MSRV 1.88.0. Selected native comparisons observe
3.1.5/4.1.1; default observations also include 4.1.2 *. Deck uses RADV VANGOGH
(`1002:163f`), provider 3.1.5, Proton 10.0-4b and steamrt3. Machine uses RADV
NAVI33 (`1002:7481`), provider 4.1.1, Proton 11.0-2e and steamrt4. Both Linux
hosts record SteamOS 3.9.5, Mesa 26.2 development and VKD3D-Proton 3.0.0.
Their cross-builds use Rust/Cargo 1.99.0 and cargo-xwin 0.23.1, not executed
Linux-native Rust upscalers or a fresh Linux-hosted MSRV matrix.

Proton GPU children report active Vulkan core/synchronization validation with
zero errors/warnings and all 230,400 output pixels finite and written. DirectX
debug-layer/info-queue checks are unavailable: `UINT64_MAX` is an unavailable
count, not zero errors. GPU-based/assisted validation is not established. Linux
signature diagnostics cannot establish Windows Authenticode policy. Deck's
timeout proof concerns bounded process cleanup, not GPU/kernel-hang recovery;
Machine establishes neither forced-offline nor forced-timeout coverage.

These results close the exact-pair host/package gate within those configurations.
They add no performance measurements, deferred temporal/image-quality study,
universal provider or native Linux/Vulkan support. Historical generator no-write
failures remain visible and were not promoted by this batch. The later
[performance acceptance](2026-10-29-rc1-performance-acceptance.md) integrates the
separate short timing follow-up. The later publication simulation passes below;
shipping authorisation remains separate.

## Accepted rc1 performance reconciliation

The [candidate-bound follow-up](../research/records/2026-10-29-exp-rc1-performance-check.md)
passes 122/122 execution checks in 469.912 seconds, with ten complete pairs per
workload, stable inputs and independently verified inventories. All ordinary rc1
package source members match the captured measurement inputs. It integrates
U13–U15's recorded source impact with the accepted P08 basis.

The maintainer [accepts the combined assessment](2026-10-29-rc1-performance-acceptance.md)
under D043: average additional CPU recording cost below 5 us on the recorded
workloads/deployment, no detected wrapper-specific GPU overhead and no detected
regression from the final cleanup. P08's five pointwise nominal one-sided 95%
CPU upper bounds are **4.046–4.618 us** in its accepted 222-block temporal subset.
The post hoc selection and unestablished post-selection coverage remain explicit;
these are bounds on average added cost, not frame percentiles. The rc1 pilot
has means **3.009–4.841 us**, with no fresh formal budget verdict. All five
descriptive GPU difference intervals include zero; exact GPU equivalence is not
claimed. Original measurements and verdicts remain unchanged.

Final P08 source/package reconciliation is complete. No further timing run is
selected for rc1; the subsequent publication simulation passes below.

## Rc1 package-byte reproduction

On 2026-10-29, **22:24:43–22:25:03 UTC**, a fresh local checkout was prepared at the
frozen source revision `6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`. Its local Git
configuration selected LF checkout, and its working
tree was clean before and after packaging. Cargo 1.99.0 and Rust 1.99.0 match
the complete producer version records in the frozen manifest.

The existing packaging command ran with a separate fresh build directory:

```sh
cargo package --workspace --locked --offline --target-dir <fresh-build-directory>
```

The manifest's development/test debug settings of `0` and `CARGO_INCREMENTAL=0`
were preserved. Temporary compiler files were routed into the new workspace-local
attempt; existing build/evidence/SDK directories were not cleaned or modified.
Cargo's built-in extracted-package build verification passes for both crates
with the default Windows/DX12 configuration. No `--no-verify` or dirty-source
override was used.

| Regenerated archive | Bytes | SHA-256 | Compared with frozen rc1 |
|---|---:|---|---|
| fsr-sdk 0.3.0 | 318,967 | `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766` | Exact match |
| fsr-sdk-sys 0.1.2 | 43,576 | `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042` | Exact match |

Both generated VCS members identify the clean frozen revision and correct crate
directory. Their 171/33 members include 168/30 ordinary source members matching
the checkout byte-for-byte. The main checkout, package sources, root README and
original frozen archives remained unchanged throughout reproduction.

The [compact publication-verification record](2026-10-29-rc1-publication-verification.json)
retains both attempts, commands, environments, tools, log digests/sanitized text,
per-member hashes, VCS data and comparison outcomes. The first restricted attempt
failed during local Git cloning with `NtCreateDirectoryObject` access denied
(`0xC0000022`), before packaging. It remains failed; the fresh attempt with the
required process/filesystem rights establishes the successful reproduction.

This completes **step 1**, package-byte reproduction. The clean checkout and
build products remain in the new ignored `target/rc1-reproduction/` attempt for
the next step. No Cargo publish command or upload ran in step 1; the separate
dry-run below now completes simulation. No native/GPU matrix was repeated.

## Rc1 publication dry-run

On 2026-10-29, **22:53:18–22:53:39 UTC**, the same clean frozen-source checkout
ran the workspace publication simulation with Cargo/Rust 1.99.0:

```sh
cargo publish --workspace --dry-run --locked --target-dir <fresh-dry-run-build-directory>
```

Registry index reads were enabled. Cargo packaged and verified raw **0.1.2**
before wrapper **0.3.0**, including the wrapper's dependency on the prepared raw
package through Cargo's temporary registry. The command exited **0** and emitted
**two** explicit `aborting upload due to dry run` warnings, one after each
package's upload-stage label. The `--dry-run` flag prevented uploads; no actual
publication or publication authorisation follows.

Both retained dry-run archives have the exact sizes and SHA-256 values in the
rc1 table above: wrapper **318,967 bytes**, raw **43,576 bytes**. Both generated
VCS members still identify the clean `6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`
source revision and correct crate paths. The frozen checkout remains clean
afterward; the main working-tree state, crate sources, root README and original
candidate archive bytes are unchanged.

The [same compact verification record](2026-10-29-rc1-publication-verification.json)
now retains the successful simulation's exact commands, tools/environment,
timestamps, original log/run digests, sanitized output, abort-marker count,
archive hashes and cleanliness/integrity checks alongside unchanged step-1 data.
The Cargo publication command itself took **19.745 seconds**. This adds package
publication simulation, not another consumer/native/performance matrix or a
claim about actual registry uploads/hosted docs. Final archive cleanup, the
maintainer-controlled root README review and separately authorised publication
remain.

## Post-cleanup Windows functional qualification

The fresh batch ran on **2026-10-29, 14:56:50–15:35:08 UTC** after U13–U15.
Every required case passed, every run recorded unchanged inputs, and all eight
complete retained inventories and case outcomes were independently verified
with `verify_run`. No production or test-code correction was needed.

| Source-based profile | Required case checks | Coverage |
|---|---:|---|
| `windows-upscaler-followup` | **625/625** | Infrastructure, historical evidence integrity, Stable/MSRV feature and consumer/example checks, Clippy/Rustdoc, independent MSVC ABI and native core/Configure/sharpening regressions |
| `native-callbacks` | **15/15** | Production callback ABI, both public owners, diagnostics, fenced debug-view dispatch and cleanup |
| `native-motion-scale` | **102/102** | Both owners/providers, signed scales, render/display motion, jitter-inclusive controls, UAV inputs and active output |
| `native-formats` | **166/166** | Typed colour/output, R16F depth and selected generator inputs, with independent C++ comparisons |
| `native-depth-formats` | **82/82** | Typed/typeless D32/D32S8, reversed depth, dynamic extents, depth/stencil/shader readback and C++ comparisons |
| `native-bgra-srgb` | **102/102** | BGRA/sRGB main and generator bindings, UAV-capable inputs, fenced calls and resource integrity |
| `native-resource-formats` | **280/280** | Remaining colour/depth/motion/exposure/mask/output roles and supported provider-conditioned generation |
| `native-variable-output` | **62/62** | Active output within fixed maxima, both owners/providers, motion encodings, sharpening, independent inputs and readback comparisons |
| **Total** | **1,434/1,434** | Eight complete profiles; source/native qualification, not final shipping archives |

The [new compact record](2026-10-29-functional-release-verification.json)
retains all case outcomes, all **1,632** commands in losslessly checked argument/
environment tables, source/SDK hashes, equipment, timestamps and inventory
verification receipts. Its common source/SDK content identity is
`1f0781527459d7c0eccf128d84047f942ea0b497c1ef8dda65311b0d2ba23d1b`;
the first run records 352 source/external input files. This is a content identity,
not an archive hash. Original attempts remain local under ignored
`target/functional-release-verification-20261029T145648Z/`. Payload retention
follows each profile's existing policy; successful resource-format child payloads
are discarded after evaluation and cannot be recovered from their hashes. No
full export or GPU image corpus is added to the repository.

Equipment: Windows 11 x64/MSVC, Rust/Cargo 1.99.0, exact MSRV 1.88.0,
Python 3.14.1, adapter 0, AMD Radeon RX 9060 XT (`1002:7590`), driver
`32.0.32015.7029`. Feature-native comparisons use selected providers **4.1.1**
(`17700776140664213505`) and **3.1.5** (`17700776140660019205`) as defined by
each protocol; passing generator cases do not establish other-provider support.
SDK v2.3.0 headers remain pinned to commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Both DLLs report Windows
Authenticode status `Valid`; exact DLL/header identities are in the record.

The infrastructure case ran 312 Python tests: **308 passed, four platform-specific
checks skipped**. Those nested tests are part of one required case, not 312
additional cases in the table. No deferred GPU image-quality experiment ran.

Reproduce each listed profile through the existing coordinator, using a fresh
output directory and the recorded development/test debug settings of `0` and
`CARGO_INCREMENTAL=0`. The first profile already contains Stable/MSRV/consumer
coverage; do not precede it with separate overlapping profiles. For example:

```sh
python tools/verification/run.py run windows-upscaler-followup --require-passed --native \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 \
  --output-root target/functional-release-verification-new
```

The earlier attempt was stopped at the maintainer's request pending incoming
research, with 315 recorded passing cases and no completed profile. It remains
interrupted and is not counted as qualification. The fresh batch re-executed all
cases, reusing compiled Cargo caches only. Since interruption, the only changed
source-identity input was `.gitattributes` for candidate archive preservation;
no Rust or test source changed.

This source-based batch closed the Windows functional step. The subsequent
rc1 qualification above closes exact-package host checks, and the separate
accepted timing follow-up closes P08 reconciliation. Publication simulation
passes separately above. No timing benchmark, SDK upgrade,
Proton execution or publication occurred in this source-based batch. The known
generator failures and native trust/lifetime limitations below remain unchanged.

## Release documentation and metadata

The package READMEs, crate-level Rustdoc, migration/recovery guidance, ABI
inventory and release notes now describe `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`.
Both package manifests explicitly select their local README and versioned docs.rs
URL. The changelog retains its Unreleased marker until publication. No release
date, registry availability or final archive acceptance is asserted.

The [documentation verification record](2026-10-29-release-documentation-verification.json)
retains six passing commands: formatting, warning-denied Windows workspace
Rustdoc with movable/utils, portable utils Rustdoc, Cargo metadata and both
package file lists. Lists contain 171 wrapper and 33 raw-crate entries, including
the package README and both applicable licence files, with no candidate archives
or native build products. Listing is not final tarball verification.

Non-documentation Rust lines, existing Rust examples, dependencies, feature
definitions, MSRV, lockfile and workspace settings were compared and remain
unchanged. The top-level README remains byte-identical, SHA-256
`8440ecc1843cba91fbd153a31b7394fa949d2edbd4758c79f055711588988208`,
under the maintainer's explicit instruction to leave it until the final pass.
The functional/native matrix was not repeated for these text/metadata edits;
new archive identities and affected package checks remain necessary.

The supplied external review's Vulkan assessment agrees with the existing
source-based D042 conclusion and does not select another pre-0.3.0 redesign.
Its two concrete retained-data findings are corrected with
[dated provenance](2026-10-29-evidence-privacy-correction.md). Repeatable
[content screening](../release-content-review.md) ran on current non-ignored
worktree text: no matches for the selected key/token patterns; the email match
is a deliberate URL-userinfo rejection fixture, and workspace-pattern
matches are normalization prose, screening metadata or the documented expression. These bounded
searches certify neither all possible secrets nor Git history. Review exact
extracted candidate contents before their distribution commit.

The follow-up content review also identifies eight unnecessary external-volume
mount roots in four historical Deck JSON files. These are now
[sanitized with byte-identity provenance](2026-10-29-deck-volume-privacy-correction.md),
preserving every other JSON value and all historical outcomes. The new bounded
volume-path screen finds no remaining matches in current non-ignored text.
Neither shared rc1 archive changes, and no functional/native repeat is required.

## Shared candidate pair

The temporary distribution directory was removed after qualification and
publication simulation. Archive hashes and full member inventories remain in the
preparation record below. Subsequent documentation edits do not change what
those frozen hashes identify.

The [retained 0.3.0-rc1 preparation inventory](2026-10-29-candidate-package-preparation.json) now
freezes one pair for every host, generated from clean source revision
`6e1c93ed2ba85561faafa7f18f5d175d2caf0ffa`. Both `.cargo_vcs_info.json` members
identify that revision and their correct crate directory, without a dirty flag.
All three host reports above now qualify these distributed bytes. The manifest
and preparation record retain their creation-time status; current qualification
is recorded here, without overwriting either frozen input record. No publication
is claimed.

| Archive | Bytes | SHA-256 |
|---|---:|---|
| `fsr-sdk 0.3.0` | 318,967 | `e198d6f87938bfe3f321fdd61355ba0ec8cb4b133633f72ad83e3713fbfed766` |
| `fsr-sdk-sys 0.1.2` | 43,576 | `2137546b831d71b1b5e779273f8389ad51bcb28f16a92141746db7c5e57ce042` |

The [self-contained preparation record](2026-10-29-candidate-package-preparation.json)
retains commands, toolchain/environment, both packaging outcomes, every member's
size/hash, source-blob comparison, Git provenance, content screens and independent
inspection receipts. Cargo 1.99.0 packages **171 wrapper / 33 raw members** and
successfully verifies both extracted packages with the default Windows/DX12
configuration. Four focused existing coordinator cases pass: toolchain, selected
archives, isolated registry/member validation and final byte integrity. This
also verifies normalized manifests, dependency versions, README/licence inclusion,
safe/unique portable paths and absence of SDK binaries/build products.

Every non-generated member, including `Cargo.toml.orig`, matches the source
commit's Git blob byte-for-byte (168 wrapper / 30 raw members). All archive text
uses UTF-8/LF. Before packaging, 17 old CRLF crate files plus the root Cargo
manifest/licence were converted to their already-committed LF bytes. Git's index
was refreshed without a staged diff; no source-content change or source commit
was created. Both package MPL copies still match the root licence. The top-level
README remains byte-identical to its held-back version.

Eight bounded ripgrep screens across the exact extracted packages find no
matches for the documented key/token, email or private-workspace patterns.
This is scoped content review, not a guarantee about every credential format or
Git history. The final copied archive hashes match the inspected bytes; binary
Git attributes disable text conversion, text diffs and automatic merging.

The first restricted `cargo package --workspace --locked --offline` attempt
failed during wrapper verification with Cargo's internal `no hash listed for
fsr-sdk-sys v0.1.2` error. Its archive bytes were not retained. A fresh output
directory with the required access passes both package verifications. The failed
attempt remains failed; this retry establishes the successful outcome, not the
root cause or a general Cargo fix.

Windows and Proton checks used both explicit rc1 paths, without independent
host repackaging. Preparation alone established no host qualification; the later
reports above supply that evidence. The later accepted timing follow-up closes
P08 reconciliation; the later publication dry-run also passes. Report commits may advance
the distribution checkout while the
pair remains immutable. Publish/dry-run from the recorded clean source
revision and require regenerated hashes to match, as described in the
[candidate workflow](#rc1-publication-dry-run).

## Windows results

The following archive-bound rows predate U13–U15 and retain their original
evidence. All rows use the same Windows archive pair and passed their required
checks. Every attempt recorded stable inputs, unchanged selected archives and
unchanged extracted files; retained evidence inventories were independently
verified.

| Profile | Required case checks | Coverage |
|---|---:|---|
| `release-artifacts` | **594/594** | Extracted packages, seven feature configurations, Stable/MSRV tests, Clippy/docs, registry-only consumers, negative API/feature controls, full examples and eight workflow calls per compiler |
| `native-motion-scale --artifacts` | **106/106** | Both owners/providers; render/display motion, signed/asymmetric scales, jitter, dynamic output and UAV inputs |
| `native-formats --artifacts` | **170/170** | Typed colour/output/depth and generator inputs; independent C++ reference comparisons |
| `native-depth-formats --artifacts` | **86/86** | Typed/typeless D32 and D32/S8, reversed depth, active extents, plane/readback checks and C++ comparisons |
| `native-bgra-srgb --artifacts` | **106/106** | Both owners; BGRA/sRGB views, UAV inputs, output writes and supported generator paths |
| `native-resource-formats --artifacts` | **284/284** | Remaining colour/depth/motion/exposure/output/mask roles and provider-conditioned generation |
| **Fresh main batch** | **1,346/1,346** | Package and advanced native gaps |

The five advanced GPU profiles retain 2,646 validated frame/readback records,
including matched C++ reference frames and no-dispatch controls. These are
integration checks of encoding, mapping, native execution, writes and cleanup;
they add no image-quality or performance claim.

Equipment: Windows 11 x64/MSVC, Rust/Cargo 1.99.0 and exact Rust 1.88.0 for MSRV,
Python 3.14.1, Radeon RX 9060 XT at adapter 0, driver 32.0.32015.7029 and pinned
AMD SDK v2.3.0. The SDK DLL hashes match the P08/callback inputs and pass Windows
Authenticode checks. Advanced feature cases explicitly select 3.1.5 and 4.1.1;
the default generator probe observes 4.1.2 *. No general capability follows
from a provider's name or enumeration.

## Generator outcomes retained as failures

The additional generator profiles are **not passing profiles**:

| Profile | Passed | Failed | Blocked | Observation |
|---|---:|---:|---:|---|
| `experiment-public-reactive-mask --artifacts` | 16 | 34 | 1 | Default 4.1.2 * acknowledges generation but leaves the mask unchanged |
| `experiment-selected-reactive-mask --artifacts` | 52 | 34 | 1 | Explicit 3.1.5 succeeds; explicit 2.3.4 acknowledges without writes |

For each owner, 3.1.5 passes the no-generator/identical-colour controls and all
16 flag combinations. All 68 failing generation cases on 4.1.2 */2.3.4 completed
their exact child process successfully, reported native code 0 and cleanup
success, and produced three fenced readbacks. Every mask stayed at seed value
227 with zero changed pixels and zero outside-region changes. The write criteria
were preserved; acknowledgement was not relabelled as support.

Each profile's dependent final-integrity case is blocked by its failed generation
requirements. An additional read-only audit independently confirms unchanged
archives and extracted contents for both attempts, without changing either
profile's failed/blocked outcomes. Historical 4.1.1 no-write evidence remains
bounded prior evidence; this default-provider repetition observes 4.1.2 *.
The supported generator proof remains 3.1.5, not universal provider support.

The first default-provider attempt, `20261029T024726Z-experiment-public-reactive-mask-bab1bf6d91f3`,
also recorded `inputs_unchanged=false`: the concurrent Deck documentation update
changed the captured `tools/verification/README.md`. It is retained separately.
Only this short probe was repeated; the accepted observation attempt,
`20261029T025321Z-experiment-public-reactive-mask-ffb32978f26e`, has stable inputs
and reproduces the same failed write criteria. The Machine commit added only
release records and did not invalidate that repetition or the completed matrix.

## Fresh Proton evidence

These earlier runs predate the common rc1 pair. Current coverage is recorded in
the [rc1 cross-host qualification](#shared-rc1-cross-host-qualification).

| Host/profile | Required case checks | Bounded result |
|---|---:|---|
| Deck, `proton-offline --artifacts` | **16/16** | Package cross-build/helper, both query families, fenced readback, offline missing-input controls, PID/network containment and unchanged runtime/package inventories |
| Deck, `proton-timeout` with the same archives | **17/17** | Forced parent/detached-child timeout, independent cleanup, unrelated-prefix survival and subsequent query/GPU controls |
| Machine, `proton --artifacts` | **12/12** | Package cross-build/helper, both query families, fenced readback, explicit destruction, PID containment and package audit |

The [Deck record](2026-10-29-deck-proton-verification.md) observes RADV VANGOGH,
provider 3.1.5, Proton 10.0-4b/VKD3D-Proton 3.0.0 and the prepared steamrt3
runtime. The [Machine record](2026-10-29-steam-machine-release-verification.md)
observes RADV NAVI33, provider 4.1.1 and Proton 11.0-2e/VKD3D-Proton 3.0.0 on
steamrt4. Both use SteamOS 3.9.5 and the recorded Mesa 26.2 development stack.
The retained JSON summaries agree with their required case inventories and
passing outcomes; the host reports record successful `verify-run` checks.
Their original local artefacts were not independently re-read on this Windows host.

Vulkan core/synchronization validation is active with zero errors/warnings in
the recorded GPU children. DirectX debug-layer/info-queue checks are unavailable
under Proton; the unavailable error count remains `UINT64_MAX`, not zero errors.
Deck verifies enforced offline execution and forced-timeout cleanup; Machine's
ordinary run acknowledges launcher networking and proves neither of those extra
profiles. Linux signature diagnostics do not establish Windows Authenticode policy.
These proofs add no native Linux/Vulkan support, broad temporal/image-quality
result or GPU/kernel-hang recovery guarantee.

## Archive reconciliation and hash differences

This mismatch describes the earlier pairs, not rc1. All three current rc1 host
reports match the [frozen pair](#shared-candidate-pair) byte-for-byte.

| Host evidence | fsr-sdk 0.3.0 SHA-256 | fsr-sdk-sys 0.1.2 SHA-256 |
|---|---|---|
| Windows/P08 pair | `f55b3a2b46b96406fdbfeb82ef6ff179b3a4469b90c1198b318119b83adb204f` | `0bb92f048d27b67d74a3876bc78b5016ac33452640ffa17cffdab77179230bb0` |
| Deck **and** Machine pair | `56ca4a23e897063e7a0c975d6233919fa39f1db6e498c9c2b6977a709eefbdb5` | `af98d189914fbab70360056a40cdb1b044bae52b5757695d7b9fb8fade1f22d5` |

Comparing the retained host records now establishes that Deck and Machine tested
the same exact pair. It does not establish byte equality with Windows/P08 or
Windows execution of the Linux pair. Archive creation precedes Rust/C++
compilation; cross-compiling the executable itself does not explain a source
archive hash difference.

All 187 Windows crate files at the original archive checkpoint match the P08 file
hashes. Machine's
comparison reports 170 exact matches and 17 differences that match after LF/CRLF
conversion, with no further content differences in that inventory. Windows
inspection confirms CRLF in those 17 files. This explains a concrete byte
difference without an implementation-content change. It is not a complete
member-by-member comparison of the two archive pairs.

The Windows archives' `.cargo_vcs_info.json` identifies starting commit
`24daf1fbc1775a17dd10160a7b212ad6054a8cc4`. Such packaged Git metadata can also
change when packages are generated from another commit. The retained Linux
records do not include their VCS member contents, so their exact packaged Git
revision and any additional generated-member differences are not established here.

## Carried-forward checks and reproduction

- The [U12 functional checkpoint](2026-10-27-api-transition-verification.md)
  supplies 625 source/consumer/MSRV/ABI/native checks. Comparing its captured
  crate inventory finds 186 unchanged files; the sole later change is the
  benchmark fixture `src/upscaler/dx12/tests/performance_comparison.rs`, exercised
  by P08. The new archive matrix compiles/tests the complete current crate sources.
- [Package coverage](2026-10-29-package-coverage-verification.md) supplies the
  earlier 70-case callback/production-CB ABI/examples/core-native checkpoint and
  66-case active-output archive check against the same Windows pair.
- [Infrastructure correction](2026-10-29-ci-infrastructure-verification.md)
  supplies the complete 312-test suite under ordinary and actual Windows
  short-path roots: 308 pass and four platform checks skip in each run.
- [P08](2026-10-29-performance-review-closure.md) remains closed with its accepted
  temporal/deployment limits. This task changes no runtime inputs and selects no
  new benchmark; the Linux line-ending assessment remains distinct from archive
  byte identity. No universal speed-parity claim follows.

The historical Windows attempts below are under ignored `target/qualification/`; their IDs,
times, summary/inventory hashes and exact case outcomes are in the companion JSON.
Use the Windows pair from
`target/release-performance/20261028T204807Z-release-performance-9ebf070a378f/candidate-build/package/`:

```sh
python tools/verification/run.py run release-artifacts --require-passed \
  --wrapper-archive <pair>/fsr-sdk-0.3.0.crate --sys-archive <pair>/fsr-sdk-sys-0.1.2.crate

python tools/verification/run.py run <native-profile> --artifacts --native --require-passed \
  --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 \
  --wrapper-archive <pair>/fsr-sdk-0.3.0.crate --sys-archive <pair>/fsr-sdk-sys-0.1.2.crate
```

Substitute the five native profiles from the table. The generator probe commands
use the same native options and preserve their failed outcomes. Actual Windows
runs set `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and
`CARGO_INCREMENTAL=0`, remained locked/offline and used exclusive attempt directories.
Completed motion/typed-format readbacks received lossless NTFS compression to
control disk use; their full inventories were reverified afterwards. The existing
resource-format profile discards successful child payloads after evaluation;
failed payloads and all required compact observations remain retained.

## Remaining release work

Functional verification, exact rc1 package/consumer/host qualification and the
release-facing documentation pass are complete. No new functional blocker is
reported by the three hosts. The
[accepted rc1 performance reconciliation](2026-10-29-rc1-performance-acceptance.md)
also closes the final P08 gate. Clean-source package reproduction and the
uploads-aborted publication simulation now pass against the same rc1 hashes.
Remaining steps, in order:

1. Remove temporary candidate `.crate` files
   from the release tree, retaining manifests and reports. Complete the
   maintainer-controlled top-level README review at the end; the released
   baseline is activated only after publication.
2. Obtain separate publication authorisation, then verify registry bytes and
   hosted documentation and close shipped roadmap items.

Prose alone does not require another native matrix. Dependency, feature, build
or runtime changes require the relevant impact review and affected checks.
Publication simulation passes; nothing has been published by these checks.
The failed provider-specific generator probes
remain visible, as do callback heap-invocation/quiescence, threading and native
failure-recovery limits from their existing records. Unproven numerical Configure
application and conditional active-output/resource-access bounds are unchanged.
Nothing was published.
