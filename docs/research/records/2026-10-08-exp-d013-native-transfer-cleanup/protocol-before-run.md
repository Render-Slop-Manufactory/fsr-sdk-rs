# D013 spike 2: preregistered native protocol

Declared 2026-10-08, Europe/Berlin, before new native execution.

Question: does the signed v2.3.0 Windows/DX12 configuration tolerate serialized
runtime/context transfers, continuous temporal history, separately held siblings
and destination-thread Rust unwind cleanup after completed GPU work?

Known baseline: the 2026-09-30 three transfer cases and controls passed; the
loader disappeared, while the provider remained with the caller device retained.
The 2026-10-07 fixture spike passed seven paired scenarios and found no wrapper
defect. The source investigation is available but did not complete the requested
pinned implementation audit or establish an affirmative AMD transfer guarantee.
Neither input identifies a concrete defect in a required path. This bounded
experiment keeps those provenance/affinity limits; it accepts no public contract.

## Inputs and resource policy

SDK v2.3.0 / commit 60f4ea81909200d8542eca14dccb2628b763a9a3, signed local loader
and upscaler, ordinary debug layer, no GBV. Expected historical provider 4.1.1 /
0xf5a5ca1e01001001; observe it rather than force a version override. Fingerprint
actual inputs and report any configuration change. No downloads, installations,
new dependencies, callbacks, allocators, flags, effects or D014 changes.

320x180 render / 640x360 output, existing fixed formats, exposure 1.0,
16.667 ms, camera near/far 0.1/100, FOV pi/3, pre-exposure 1.0, sharpening off.
Use the established sinusoidal semantic scene: two render pixels of positive
x translation per frame, UV current-to-previous motion (-2/320, 0), zero motion
on frame 0. Analytic samples subtract the supplied projection jitter. Repeat
the eight-position semantic jitter sequence continuously. Reset only frame 0
of each context. No discontinuities, context recreation or thread-transfer resets.

Fresh recording equipment per frame; submit each successful recording once,
wait at most 60,000 ms and verify completed fence value >= 1 (UINT64_MAX fails).
Retain every frame's harness, list, allocator, resources and COM references through
all context cleanup; retain caller device and helper until all frames are released.
Creation descriptors remain in pinned owned boxes. No observation retains an AMD
module reference. Record packed little-endian RGBA16F bytes, row padding excluded.

## Matrix and exact placement

Four persistent named workers A/B/C/D stay alive through cleanup. An owning
channel message and reply acknowledge every transfer. Worker slots own contexts
while idle; siblings are not collected into one transferred family. A single
shared mutex covers AMD calls, helper GPU work, COM inspection and cleanup,
including Drop and final runtime release. No guard is transferred. Native
operations never overlap. The same-thread arm uses A for every operation and
the same ownership round trips; the remaining workers stay live but idle.

| Case | Transfer arm | Matched control | Selected payloads |
|---|---|---|---|
| A: create after runtime transfer | Load on A; move runtime to B, create there; move context/runtime to C, dispatch/fence frame 0; move to D, explicit destroy and final runtime release | All corresponding operations and ownership round trips on A | context 0: frame 0 |
| B: continuous history | Load/create on A, early external runtime drop; 64 frames, one per A/B/C/D in cyclic order, with 63 fenced ownership handoffs; explicit destroy on final worker | Same 64 inputs/order and handoffs on A | context 0: 0, 1, 7, 15, 31, 63 |
| C: independent siblings | Create on A, drop external runtime; context 0 held on B, context 1 on C; alternate frames 0..7 on each; after frame 3 transfer context 0 B->D while context 1 stays on C; explicit destroy context 0 on D; frame 8 succeeds on context 1 on C; transfer final sibling C->B and automatic Drop | Same distinct bundles, alternating schedule, ownership round trips and explicit/automatic cleanup on A | both: 0, 7; survivor: 8 |
| D1: unwind one context | Load/create A; dispatch/fence frame 0 on B; transfer B->D; deliberate Rust panic outside FFI with the owner inside the unwound scope | Dispatch/fence and owner unwind on A | context 0: 0 |
| D2: unwind sibling | Create A, early runtime drop; context 0 on B and context 1 on C; each dispatch/fence frame 0; context 0 B->D and unwind there; context 1 stays on C, recovers the poisoned common gate and dispatches/fences frame 1; final sibling C->B and automatic Drop | Same schedule and poisoned gate on A | first: 0; survivor: 0, 1 |

For unwind, enter the common gate before declaring the owned local inside the
catch-unwind closure. Reverse destruction drops the context before the guard.
The catch boundary is outside both owning locals, asserts the exact deliberate
panic payload, and leaves worker execution alive. Drop reuses a gate already
held by the current thread rather than recursively locking it; later entries
recover mutex poison with into_inner. Record gate coverage and native destroy
return codes through test-only instrumentation of the real terminal owner.
No foreign unwind or deliberately manufactured native error is introduced.

## Residency and dependency checkpoints

Record loader, upscaler and helper presence separately using GetModuleHandleW
as an instantaneous boolean only: it adds no reference; never dereference or
FreeLibrary the observation handle. The isolated child has one staged deployment.
Observe (1) live contexts, (2) early external runtime drop where applicable,
(3) first sibling cleanup, (4) final contexts/runtime gone, (4a) frames released
while caller device remains, (5) all helper/device/caller references released,
and (6) immediately before normal child exit after ordinary worker joins.
Weak strong-count zero measures Rust retention only. Debug/device checks happen
while the device is live, including after frame release; none follows device drop.
No forced unload, COM reference manipulation, observation pins or exit-based
claim of earlier unload. Provider persistence is diagnostic and unresolved;
disappearance does not attribute every provider termination callback to a thread.

## Repetition, comparison, failure and execution oracle

Run three independent process repetitions per case/arm (30 new children).
First run both local-family lifecycle gates, all three existing public dispatch
gates and all six earlier D013 native cases on this exact final source. A failing
prerequisite stops the new matrix. Run ordinary checks, D013 fixtures and pinned
header ABI compilation. Use the existing 180,000-ms watchdog for every child.
Never relax timeouts. Preserve all failures and any execution/protocol amendment;
a protocol change requires a new complete matrix, without replacing prior results.

Execution pass: create/query/dispatch/explicit and automatic destroy return OK;
actual submission and completed fence logged; 230,400 finite output pixels and
more than half changed from sentinel; zero D3D12 error/corruption messages,
S_OK device status, expected panic only, no timeout/deadlock, survivor works,
normal wrapper release with weak count zero and loader absent. Uncertain fence
completion or native dispatch failure exits the child immediately without
submitting the failed recording or unwinding through ordinary cleanup. Such
containment is a failure and never successful cleanup evidence.

Hash all selected packed payloads with SHA-256. Compare every matching frame
across control repetitions, then transfer repetitions and control-vs-transfer.
If controls are deterministic, exact equality is the image-equivalence criterion.
If controls differ, label exact image equivalence inconclusive and report all
hashes/differences; do not waive differences or attribute them to transfer.
Keep execution, payload equivalence and residency as separate outcomes.
No general image quality, Send guarantee, future public API verification or D013
acceptance follows from any result. Limit final unload investigation to these
checkpoints and do not expand into binary reverse engineering.

Execution status at declaration: pending.
