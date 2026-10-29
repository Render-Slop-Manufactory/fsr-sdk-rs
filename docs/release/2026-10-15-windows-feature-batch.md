# Windows feature and helper regression batch — 2026-10-15

Windows execution confirms the implemented linear HDR slice, colour-control
mapping/validation and recent native-helper/dispatch refactoring. Supplied-mask
GPU acceptance remains blocked by the wrapper's interpretation of the tested
provider's resource query.
The complete native profiles therefore **fail**; their failures are retained.
No production code, test assertion, public contract or standard profile changed.
The worktree was clean before testing.

## Correction: wrapper rejection is not an AMD dispatch denial

Follow-up source inspection on 2026-10-15 found that the
[pinned AMD 4.1.1 guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/techniques/super-resolution-ml.md#optional-resources)
explicitly permits supplying reactive and transparency/composition masks to FSR4.
The query header describes inputs required for correctness or consumed if supplied;
it does not explicitly prohibit supplying unlisted masks. D025 deliberately adds
that conservative rejection as wrapper policy. Consequently `optional=0` and the
observed Rust `UnsupportedResource` do not establish that AMD would reject these
inputs or that this GPU/driver lacks support. No native mask dispatch occurred.
The initial hardware/provider-blocker interpretation was too strong. Review the
query/documentation discrepancy and wrapper policy before choosing a fix or
claiming successful native mask consumption; the recorded failed tests are unchanged.

The maintainer subsequently accepted the
[D025 amendment](../adr/D025.md#amendment-rationale-and-implementation-status):
require mandatory masks, validate and forward supplied masks irrespective of
optional bits, and add no bypass mode. Implementation/tests and native
confirmation of that amended decision remain pending.

Subsequent [implementation and verification](2026-10-15-mask-forwarding-verification.md)
complete that amendment: all 18 supplied-mask GPU frames and all 29 standard
native-upscaler cases pass on the same provider. The failures and counts below
continue to describe this earlier attempt under the original wrapper policy.

## Scope and equipment

The audit found pending Windows checks for M15 supplied masks, M16 HDR/colour
controls and the responsibility-based helper/dispatch inspection cleanup.
M11–M14/U03 already had Windows evidence and were rerun as regressions.
P03/P04 were rerun to check independently compiled C++ consumers after the helper
rename. Planned effects, variable output, P05/P02 extensions, held image-quality
studies and final release/archive acceptance are outside this development batch.

- Windows 11 build 10.0.26300; x86_64-pc-windows-msvc.
- Rust/Cargo 1.99.0; separate Rust 1.88.0 MSRV profile.
- Python 3.14.1 with NumPy 2.5.4; PowerShell 7.6.7.
- MSVC 14.44.35207; Windows SDK 10.0.22621.0.
- Explicit adapter 0: AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
  feature level 12_0; driver `32.0.31041.1004`.
- SDK v2.3.0, pinned source commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`; local deployment under
  `target/sdk-windows-verification/v2.3.0`.
- Provider `4.1.1`, ID `0xf5a5ca1e01001001`.

Offline SDK verification passes for all eight installed files. Native cases
separately verify both DLL hashes and Valid AMD signatures. No SDK download or
new strict online revocation check was performed. Individual commands, source,
SDK and executable identities are retained in each evidence directory's
`cases.json` and `inputs.json`.

## Commands and outcomes

From the repository root, with cached Cargo inputs:

```powershell
python tools/verification/run.py sdk verify --root target/sdk-windows-verification/v2.3.0
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-upscaler --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-transfer-cleanup --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-recording --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-cpu-comparison --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run experiment-gpu-timing --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

| Check | Passed | Failed | Blocked | Evidence |
|---|---:|---:|---:|---|
| Current source CI | 115 | 0 | 0 | [Results](2026-10-15-windows-feature-batch/ci/run.json) |
| Rust 1.88 MSRV | 98 | 0 | 0 | [Results](2026-10-15-windows-feature-batch/msrv/run.json) |
| Full native-upscaler | 22 | 1 | 6 | [Failed result](2026-10-15-windows-feature-batch/native/run.json) |
| Full transfer-cleanup | 28 | 1 | 31 | [Failed result](2026-10-15-windows-feature-batch/transfer/run.json) |
| Recording profile | 28 | 1 | 0 | [Failed result](2026-10-15-windows-feature-batch/recording/run.json) |
| Supplemental independent regressions | 79 | 0 | 0 | [Results](2026-10-15-windows-feature-batch/independent/run.json) |
| Independent C++ CPU comparison | 45 | 0 | 0 | [Results](2026-10-15-windows-feature-batch/cpp-cpu/run.json) |
| Independent C++ GPU timing | 22 | 0 | 0 | [Results](2026-10-15-windows-feature-batch/cpp-gpu/run.json) |

These counts overlap; they are profile cases, not unique Rust tests. Every listed
finalized attempt passes `verify-run`, including the failed attempts, and records
unchanged inputs. Integrity verification does not turn failure into acceptance.
[Summary and attempt names](2026-10-15-windows-feature-batch/summary.json).

CI includes four independent C++ ABI probes; default, movable, no-default and
no-default+movable checks/tests/Clippy/docs; public consumers, dependency boundaries
and complete examples. The [infrastructure log](2026-10-15-windows-feature-batch/ci/infrastructure.tests.txt)
records 184 tests: 180 pass, four skip. Three Linux/Proton identity/process checks
and the privileged-symlink control are skipped. NumPy evaluator and real PowerShell
parser tests execute. The [movable test log](2026-10-15-windows-feature-batch/ci/rust.movable.test.txt)
includes the actual Windows colour, HDR, masks and inspection fixtures.

## Feature observations

**M15 supplied masks:** controlled Windows fixtures pass for both ownership
families, including exact mapping, requirements/presence policy, COM inspection,
active extents, retry, poison and reference balance. The actual provider reports
`required=15`, `optional=0` for both families in the
[public queries](2026-10-15-windows-feature-batch/native/native.queries.public.txt).
The prepared native mask protocol's first explicit-exposure/reactive-mask frame
returns `InvalidDispatch { reason: UnsupportedResource(ReactiveMask) }` before
native dispatch. The [failure log](2026-10-15-windows-feature-batch/native/native.dispatch.supplied-masks.txt)
is retained. This is consistent with D025's conservative rejection policy; it is
not evidence of mask GPU consumption or of AMD rejecting the supplied resource. The attempted
protocol stops there: automatic exposure, the second mask and both-mask native
combinations remain unexecuted. Acceptance needs review of the query interpretation
and wrapper policy, followed by native confirmation. No provider override
or bypass was introduced.

**M16 linear HDR:** two contexts, explicit and automatic exposure, each execute
three continuous fixed-size frames with scene RGB `(8,4,2)` and pre-exposure
`0.25 → 1 → 4`. Full input readbacks/guards match, all 230400 output pixels per
frame are finite and written, output red stays above 1 and below 64, fences and
explicit teardown pass, and device removal/debug errors stay zero. The final
automatic frame has red range `31.09375..31.984375`.
[Native log](2026-10-15-windows-feature-batch/native/native.dispatch.linear-hdr.txt).
This verifies the bounded linear HDR integration, not reconstruction quality or
general exposure accuracy.

**M16 colour controls:** the controlled Windows test
`colour_flags_checked_pairing_and_per_frame_override_share_both_families` passes
on current and MSRV toolchains. It checks independent creation/estimate bits,
exact sRGB/PQ forwarding, default pairing, frame-local override/restored checks,
exposure/pre-exposure preservation, other validation and poison paths. C++ ABI
assertions pass. No non-linear GPU protocol was prepared; provider PQ normalization,
transfer/exposure ordering, output interpretation and temporal semantics remain
unverified. M16 as a whole stays open.

## Helper and dispatch refactoring

The current renamed C++ helper builds, exports load, public controlled COM/error
precedence tests execute, and both independent C++ CPU/GPU executables build/link
and complete their existing protocols. Their reruns are regression evidence;
concurrent checks and the absence of a new performance question mean no new speed
comparison or parity conclusion is claimed.

All six recording-ahead arms pass: same-context counts 1/2/5/8 and sibling-context
counts 2/5. The recording profile remains failed solely for its mask baseline.
Original full native/transfer-cleanup prerequisite graphs block later unrelated
cases after the mask failure, so a separate explicit selection was executed.

The retained [selection script](2026-10-15-windows-feature-batch/independent-selection.py.txt)
composes the existing native-upscaler and transfer-cleanup cases, deduplicates
setup, omits their two supplied-mask cases and removes only references to those
cases from prerequisites. Actions, selectors, timeouts, expected markers, payload
counts and audits remain unchanged. `selection.scope` records every removed edge.
To reproduce, copy that text file to
`target/run-independent-windows-regressions.py` and run:

```powershell
python target/run-independent-windows-regressions.py
```

All 79 selected cases pass, including both public movable modes, three sharpening
integration repetitions and the independent 144-readback/78-comparison audit,
six transfer baselines, and all 30 cleanup matrix children (five scenarios ×
control/transfer × three repetitions). The
[cleanup audit](2026-10-15-windows-feature-batch/independent/transfer-cleanup.json)
passes log/lifetime/readback checks and retains diagnostic control/transfer
comparisons. These bounded observations do not establish provider termination or
universal cross-thread compatibility. The successful supplemental selection is
not full native-upscaler or M15 acceptance; all original failures remain failed.

## Sandbox attempts and retained evidence

Initial restricted-sandbox CI/MSRV attempts encountered access-denied temporary
metadata writes (`libpositive.rmeta`, OS error 5). The incomplete attempt names
are `20261015T180514Z-ci-0de7816e17b0` and
`20261015T180525Z-msrv-c56aaa856ea9`; the latter also hit a denied result-file
replacement and the former was stopped. Neither is a pass. Fresh authorized
outside-sandbox CI/MSRV attempts pass without changing source or expectations.
The native runs did not need this retry.

Permanent evidence retains normalized UTF-8/LF inputs, plans, case outcomes and
commands, selected logs and compact audits. Build products, AMD binaries and raw
readbacks remain under ignored `target/`. The
[inventory](2026-10-15-windows-feature-batch/artifacts.json) hashes the retained
bytes; no private environment dump is included. Full binary reanalysis needs
the local attempt or a fresh run. All results are limited to the recorded
Windows/GPU/driver/provider configuration. No publication, new SDK/platform
support, held quality study or final archive/release acceptance occurred.
