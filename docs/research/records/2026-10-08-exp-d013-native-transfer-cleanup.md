# Experiment: serialized native transfers, unwind cleanup and final residency

- Investigation date: **2026-10-08**, Europe/Berlin (CEST, UTC+02:00).
- Question: does the signed AMD v2.3.0 Windows/DX12 configuration tolerate
  creation after runtime transfer, repeated fenced transfers with continuous
  history, independently held siblings and destination-thread Rust unwind cleanup?
- **Verified execution:** all 30 new isolated children passed (five cases, two
  arms, three process repetitions), after all 11 native prerequisites passed on
  the same executed source. The new matrix submitted/completed 516 frames and
  returned OK from all 42 native context destruction calls, including 12 unwind
  cleanups. No unexpected panic, timeout, D3D12 error/corruption or device removal.
- **Verified selected payload equivalence:** all 96 selected packed RGBA16F
  readbacks match their corresponding controls and repetitions by SHA-256.
  Controls and transfers are repeatable for every selected frame.
- **Unresolved residency:** the upscaler provider remains loaded in all arms
  after every audited application-owned frame, helper and caller-device reference
  is released, and after ordinary worker exit. The loader and helper disappear.
  This does not establish provider termination or its executing thread.
- Exclusions: D013 acceptance, public movable types, unrestricted Send/Sync,
  outstanding-work transfers, overlapping native calls, native error injection,
  callbacks/allocators, D014 implementation, new effects or image-quality claims.
  No dependencies, downloads, installations, commits, pushes or publication.

## Preregistered protocol and prerequisite assessment

The complete [protocol saved before execution](2026-10-08-exp-d013-native-transfer-cleanup/protocol-before-run.md)
specifies operation placement, scene, resource policy, synchronization, cleanup,
repetitions, watchdogs, failure containment, selected frames and comparison rules.
Its pending-execution footer is preserved. There were no protocol amendments.

The [2026-09-30 native experiment](2026-09-30-exp-d013-native-thread-transfer.md)
provided successful short, joined-worker transfers and loader release, with
provider residency observed while the caller device remained retained. The
[2026-10-07 fixture spike](2026-10-07-exp-d013-transfer-failure-cleanup.md)
found no wrapper defect in transferred error state or coordinated cleanup. The
[source investigation](2026-10-07-src-d013-transfer-cleanup-contract.md) supplies
platform/coordination constraints but did not complete the requested pinned
implementation audit or establish an affirmative AMD transfer guarantee. None
identifies a concrete defect in the valid, fully fenced paths required here.
The create-error policy recommendation in that source record does not supersede
accepted D005, and no ambiguous-create or manufactured native failure is probed.

The first fixture spike extracted production dispatch orchestration but had not
rerun AMD regressions afterward. This experiment closes that bounded regression
gap before its new cases: public local-family explicit/Drop lifecycle, single
frame, three moving frames, motion/jitter/reset comparisons, and all six earlier
native transfer/control cases passed on the final source used for the matrix.

## Actual setup and source state

The initial worktree was clean. Changes listed below remained uncommitted.
The [source-state observation](2026-10-08-exp-d013-native-transfer-cleanup/source-state.txt)
records the supplementary Git identity and worktree delta. All **83** executed
crate/workspace inputs are fingerprinted in the
[raw-byte source manifest](2026-10-08-exp-d013-native-transfer-cleanup/source-inputs.json)
and [raw plus UTF-8/LF fingerprints](2026-10-08-exp-d013-native-transfer-cleanup/source-fingerprints.json).
Their bytes still matched after native execution and additional checks. The
source files included in this repository, not an external revision or CI page,
define the executed implementation.

| Input | Observed value |
|---|---|
| OS / native target | Windows 11 Pro 10.0.26300, build 26300; x86_64-pc-windows-msvc |
| GPU / driver | AMD Radeon RX 9060 XT, vendor 0x1002, device 0x7590; 32.0.31041.1004 |
| SDK | v2.3.0, pinned commit 60f4ea81909200d8542eca14dccb2628b763a9a3 |
| Observed provider | Every created context: 4.1.1, ID 0xf5a5ca1e01001001; query returned OK |
| Rust / Cargo | Rust 1.99.0, b940084d7eb6a299eb4bfeb8e34901bc051e7ac4, 2026-09-28; Cargo 1.99.0 |
| Minimum-version verification | Rust/Cargo 1.88.0 |
| MSVC / compiler / Windows SDK | Tools 14.44.35207; compiler 19.44.35224.0; SDK 10.0.22621.0 |
| Shell | PowerShell 7.6.6 |
| Locked native dependencies | libloading 0.9.0; windows 0.62.2; unchanged |
| Profile | Zero creation flags; no callbacks/allocators; sharpening/masks disabled; ordinary debug layer; GBV excluded |

The deployment/GPU/driver/provider match the September configuration. Rust
changed from that record's 1.98.1 to 1.99.0, and helper/test executable and source
changed; these are new execution evidence, not a reuse of the September result.

| Executed binary | SHA-256 |
|---|---|
| Signed AMD loader | E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA |
| Signed AMD upscaler | D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46 |
| Locally compiled m6_gpu.dll | 45CFB181C56B21926CD8B4832652A86CDD1CA2821BE6BB6E1F8660AB3E62B1DC |
| Executed wrapper test binary m6.exe | F35AB55AE842D74D17A27C84AFE3FFAE45A6DF7CF4CA8502E2C9C26FB5375F13 |

Both AMD Authenticode signatures were Valid (enum value 0 in the preserved
[environment observation](2026-10-08-exp-d013-native-transfer-cleanup/native.environment.txt)).
[Native inputs](2026-10-08-exp-d013-native-transfer-cleanup/native-inputs.json)
also fingerprint headers and executables. The three independent C++ ABI probes
compiled against the four LF-normalized hash-verified pinned headers. This does
not close the source investigation's missing implementation audit.

## Changes and safety audit

**Production behaviour:** unchanged. The only edit in
[context.rs](../../../crates/fsr-sdk/src/context.rs) adds `cfg(test)`-only optional
destruction tracing after the real native call returns. Ignored stderr-write
errors keep this tracing non-panicking. It is compiled out of the production
library; terminal ownership, exceptional retention and public traits are intact.

**Test-only changes:**

- [thread_transfers.rs](../../../crates/fsr-sdk/tests/native/thread_transfers.rs)
  implements the new matrix through the actual pinned creation state,
  NativeContextOwner, shared dispatch policy and shared COM/resource inspection.
  It adds no parallel copy of poison or terminal policy.
- [gpu.cpp](../../../crates/fsr-sdk/tests/native/gpu.cpp) adds two observations:
  completed fence value and exact packed four-channel RGBA16F payload copying.
  Neither calls AMD, changes existing frame generation or enables sharpening.
- [run.ps1](../../../tools/verification/archive/2026-10-10/files/crates/fsr-sdk/tests/native/run.ps1.txt) adds `-Mode d013-cleanup`,
  prerequisite ordering, 30 isolated repetitions, fresh output directories and
  payload hashing/comparison under the existing watchdog. GBV remains excluded.
- At the user's request, new test names describe behaviour. The earlier six
  native tests moved from `d013.rs` to
  [thread_transfer_baseline.rs](../../../crates/fsr-sdk/tests/native/thread_transfer_baseline.rs)
  with functional names and updated selectors. Their scenarios are unchanged;
  the September record preserves its actual historical names/fingerprints.

There are two new narrow `unsafe impl Send` declarations: the exact owned
fenced Frame and the exact owned context Bundle. Neither the generic owner nor
public Runtime/Upscaler receives Send or Sync. RuntimeBundle derives Send from
its actual Arc-retained fields and explicitly excludes Sync; both new context
and runtime bundles have positive Send/negative Sync compilation assertions.

The audit covers Arc-retained RuntimeState/FfxLibrary, original pinned descriptor
boxes, independently owned device/list/resource COM references and stable C++
harness allocations. Each harness owns queue, allocator, list, fence, readback,
textures/uploads and device references. All frames remain in an Equipment-owned
mutex vector through context cleanup. Main independently retains Equipment
until every native owner/runtime reference is gone and workers are idle. Frames
are then released under the common domain, the live device is checked, and the
last caller device/helper are released under the domain. Helper function pointers
are never invoked afterward. There is no retained Rc beyond the owner's marker,
borrowed native dependency, observer module reference, custom callback or allocator.
D005 destroy-failure retention still keeps its real critical dependency bundle.

Four persistent worker slots hold distinct owned bundles even while idle.
Zero-capacity owning request/reply channels establish handoff; a shared Domain
mutex covers all AMD calls, resource inspection, GPU work and cleanup. Bookkeeping
identifies a guard already held by the current thread, so nested owner Drop reuses
it without recursive locking. Bookkeeping is unlocked before waiting for the
native gate. Only the owning guard clears it. Guards are never transferred.
This serializes application operations, not unknown internal provider workers.

For each deliberate unwind, the guard is declared before the context local
inside the catch closure. The real NativeContextOwner drops before the guard;
the outer catch verifies the exact Rust panic payload. Gate poison is observed
and recovered with into_inner, without clearing it or issuing a second panic.
All relevant automatic Drop logs show domain_held=true. Native destroy codes
are observed through the real owner trace, not inferred only from weak expiry.

CPU synchronization and GPU completion are checked independently. Every new
submitted recording finishes the existing bounded fence wait and records completed
value 1 before handoff, next frame or cleanup. UINT64_MAX/insufficient completion
or finish failure would contain the isolated child without ordinary unwind. A
native dispatch error would use the shared poison policy and exit without
submission. Those unexpected-error containment paths were not exercised here;
successful normal cleanup is not claimed for them.

## Execution and comparisons

Inputs use the established sinusoidal scene: 320x180 to 640x360, translation
two render pixels per frame, UV motion -2/320 excluding jitter, eight-position
jitter sequence, reset only at each context's frame 0, explicit exposure 1.0,
16.667 ms and camera values 0.1/100/pi-over-3/1.0. Fresh per-frame recording
equipment is retained as above. There is no transfer-induced history reset.

| Case | Per child | Transfer observation | Three controls + three transfers |
|---|---|---|---|
| A: runtime before creation | 1 frame, 1 destroy | A load, B create, C dispatch/fence, D explicit destroy/final external runtime release | All pass; selected frame 0 exact |
| B: continuous history | 64 frames, 1 destroy | 63 context handoffs across four live workers; no reset after frame 0 | All pass; selected 0/1/7/15/31/63 exact |
| C: independent siblings | 8 first-sibling frames, 9 survivor frames, 2 destroys | Distinct B/C slots; first sibling B->D after frame 3, destroy on D; survivor works on C, final automatic Drop on B | All pass; both 0/7 and survivor 8 exact |
| D1: transferred context unwind | 1 frame, 1 unwind destroy | Fence on B, owner B->D; actual automatic destruction during Rust unwind on D | All pass; selected frame 0 exact |
| D2: sibling unwind/survival | 1 first-sibling frame, 2 survivor frames, 2 destroys | First sibling B->D unwinds on D; sibling stays on C, recovers poisoned gate, executes frame 1, final Drop on B | All pass; first 0 and survivor 0/1 exact |

All 516 new readbacks contain 230,400 finite and changed pixels; every debug
error/corruption count is zero and device status S_OK. All create/provider
query/dispatch and all 42 instrumented explicit/automatic destroy results are
OK. Twelve expected panic hooks occurred; each matched its declared payload and
completed successful cleanup without a second panic. Native child runtimes were
bounded by 180 seconds; the slowest observed child took approximately 2.72 seconds.

Actual Rust ThreadIds and worker names are retained per child. In the new
children, observer/test thread is ThreadId(2), and A/B/C/D are distinct
ThreadId(3/4/5/6). A's four-stage transfer therefore records load on 3, creation
on 4, dispatch/fence on 5 and destruction/final wrapper runtime release on 6.
Controls place all corresponding operations on A. The
[per-child outcomes](2026-10-08-exp-d013-native-transfer-cleanup/case-outcomes.json)
and [compact table](2026-10-08-exp-d013-native-transfer-cleanup/case-outcomes.csv)
preserve frame/destruction/unwind counts and observed handoffs. The raw native
recording equipment exposes no handle addresses; logical context labels preserve
identity without publishing transient pointer values.

[Payload hashes](2026-10-08-exp-d013-native-transfer-cleanup/payload-hashes.json)
cover 96 selected 1,843,200-byte little-endian RGBA16F payloads, excluding row
padding and retaining all four channels without float conversion.
[Comparisons](2026-10-08-exp-d013-native-transfer-cleanup/payload-comparison.json)
show deterministic controls, deterministic transfers and exact matching for
every selected corresponding frame. No nondeterminism waiver was needed.
Unselected frames were checked for finite/changed output and device/debug status,
not full image equivalence. Raw payload files remain under the ignored target
stage; the sanitized record retains exact hashes rather than duplicating them.
These comparisons do not establish general reconstruction accuracy or quality.

## Natural cleanup and residency

GetModuleHandleW is used only for an instantaneous presence boolean. It adds no
reference; its returned address is never dereferenced, freed or called through.
No observer spans teardown with a module ref. The helper imports D3D12/DXGI and
contains no AMD SDK calls; its last reference is released before checkpoint 5.
Device creation releases temporary factory/adapter/adapter3 references before
returning its single owned device ref. Debug enablement and checks release their
temporary debug/info-queue interfaces on return; no helper global retains them.
All native owners/runtime references and frames are accounted for before final
caller-device/helper release. The actual COM device's destruction is not inferred
from releasing the application's refs; native/platform ownership remains opaque.

| Checkpoint | Loader | Upscaler provider | Helper | Rust runtime ownership |
|---|---|---|---|---|
| Live contexts | Present | Present | Present | Contexts plus external runtime |
| Early external runtime drop (B/C/D cases) | Present | Present | Present | Only one/two contexts |
| First sibling destroy/unwind | Present | Present | Present | Surviving context only |
| Final contexts and external runtime released | Absent | Present | Present | Strong count zero |
| Frames released, live caller device checked | Absent | Present | Present | Strong count zero |
| All audited frame/device/helper refs released | Absent | Present | Absent | Strong count zero |
| Workers joined, immediately before normal child exit | Absent | Present | Absent | Strong count zero |

Every applicable checkpoint has the same residency outcome in all repetitions
and both arms. There were no code/device inspections after its release. All
remaining debug/device checks passed before that release. Natural loader/helper
absence is separate from upscaler residency. The provider remains after releasing
the formerly retained caller equipment/device and after ordinary worker exit;
retaining those application refs is therefore not necessary for its persistence
in this experiment. Attribution to a cache, driver, private interface or other
native owner remains unresolved. No FreeLibrary/ref-count manipulation or
additional unbounded unload investigation was attempted. Process exit supplies
no evidence of earlier normal provider unload or destination-thread termination.

## Reproduction and ordinary verification

From the complete repository, with the already available trusted local SDK:

```powershell
./crates/fsr-sdk/tests/native/run.ps1 -Mode d013-cleanup -Native -Stage target/d013-native-cleanup/attempt-1
./docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/run-ordinary-checks.ps1
./docs/research/records/2026-10-08-exp-d013-native-transfer-cleanup/audit-results.ps1 -Stage target/d013-native-cleanup/attempt-1
```

Use a fresh stage for another complete run. The ABI script initializes an installed
x64 MSVC developer environment when needed. Native children use the staged test
executable with `--ignored --exact <functional-test-name> --nocapture
--test-threads=1`; their actual test names are retained in every stdout file.
The runner supplies trusted loader/helper paths and, for new children only,
FSR_SDK_TEST_D013_TRACE=1 and a fresh FSR_SDK_TEST_D013_OUTPUT directory. GBV and
the diagnostic M6 control selector are unset. Timeouts remain 60,000-ms fences
and 180,000-ms children. No unsupported --all-features combination was used.

| Check | Result / retained evidence |
|---|---|
| Formatting, denied-warning Clippy, stable workspace tests, feature-disabled check | Exit 0; [commands](2026-10-08-exp-d013-native-transfer-cleanup/checks.status.txt), [ordinary output](2026-10-08-exp-d013-native-transfer-cleanup/ordinary-checks.txt) |
| Hash-verified common/create/dispatch ABI probes and helper /W4 /WX compile | Exit 0; ordinary output and native input fingerprints |
| Real Windows/DX12 D013 fixture probes and all 14 child scenarios | Exit 0; [events](2026-10-08-exp-d013-native-transfer-cleanup/additional-fixture-transfer-events.txt) |
| Feature-disabled fixture host adapter on Windows | Exit 0; [events](2026-10-08-exp-d013-native-transfer-cleanup/additional-fixture-host-adapter.txt); not Unix execution |
| Feature-disabled all-targets compilation | Exit 0; [log](2026-10-08-exp-d013-native-transfer-cleanup/additional-feature-disabled-all-targets.txt) |
| Rust 1.88 all-targets check, full tests, feature-disabled all-targets check | All exit 0; [command statuses](2026-10-08-exp-d013-native-transfer-cleanup/additional-status.txt) |
| Rustdoc with RUSTDOCFLAGS=-D warnings | Exit 0; [log](2026-10-08-exp-d013-native-transfer-cleanup/additional-rustdoc.txt) |
| Local package list/archive and licence inspection | Exit 0; both LICENSE texts and new functional test files included; [archive log](2026-10-08-exp-d013-native-transfer-cleanup/additional-package-archive.txt). --no-verify does not verify package builds; no publication. |
| Executed-source and native-result audit | All 83 raw hashes unchanged, 41 successful native children; [audit](2026-10-08-exp-d013-native-transfer-cleanup/audit-summary.json) |

Stable/MSRV full workspace runs each pass 36 wrapper unit tests, one dimensions
integration test, 13 sys tests and four doctests. Twenty-nine unrelated or opt-in
native/research tests stay ignored in ordinary runs; the explicit native matrix
separately executes its 41 children. New Bundle/RuntimeBundle Send and negative
Sync assertions and the existing real Windows local-family negative traits
compile. Package manifests, dependencies and public API are unchanged; test
source additions/renames are included in local archives.

There were **no unsuccessful native attempts or failed check commands** during
this spike. Initial preparation compiled/checked the harness before execution;
functional naming was adjusted before the first native run at the user's request.
Cached incremental-build hard-link warnings appeared and Cargo copied cache
files instead; denied-warning Clippy still exited 0. Test/native execution used
approved execution outside the filesystem sandbox because its CIM inventory and
fixture compiler metadata access were known restrictions. Nothing was weakened
to bypass those boundaries. Personal paths use <repo>/<user>; panic-hook OS
thread numbers use <os-thread>. Actual Rust ThreadIds, versions, provider IDs,
error codes, hashes and semantic context relationships are preserved. Adjacent
per-child stdout/stderr/status files include prerequisites and every repetition.

## Baseline delta and remaining contract questions

**New bounded evidence:** create after runtime transfer; 64-frame continuous
history across 63 handoffs between simultaneously live workers; independently
held coordinated siblings; destination-thread native destruction during real
Rust unwinding; poisoned-gate recovery and continued sibling work; exact selected
control equivalence; natural residency after releasing all audited application
equipment/device/helper references. The native local-family regression gap after
the first spike's dispatch-policy extraction is closed for this executed source.

**Unresolved:** signed-provider transfer guarantees, hidden/TLS work, native
returned-error cleanup and arbitrary failures, partial-acquisition transfer,
provider/dependency termination and its thread, external native-user coordination,
outstanding GPU work, other providers/drivers/hardware and future public APIs.
No Miri/ThreadSanitizer or binary reverse engineering was performed.

No observed result requires changing D005/D011's tested ownership/dispatch policy.
D013's review must still distinguish wrapper-owned call/reference release from
complete provider termination; stronger destination-thread termination wording
is unsupported. Common coordination including implicit Drop remains an actual
premise, not something Send + !Sync or Arc alone enforces. This record supplies
bounded evidence for that review and **does not accept D013** or verify a public
movable family. Public Runtime/Upscaler remain !Send + !Sync.

> Archive navigation added 2026-10-10: retired verification scripts are preserved in the
> [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Script links now point to exact archived source bytes;
> recorded commands, observations and fingerprints are unchanged. Historical commands require
> their original layout and compatible inputs; archived sources are not current entry points.

> Orchestration migration (2026-10-10): the shared native, ABI and consumer entry points are now frozen in the [indexed archive](../../../tools/verification/archive/2026-10-10/README.md). Current commands live in [verification](../../../tools/verification/README.md); command text and observations below remain historical.
