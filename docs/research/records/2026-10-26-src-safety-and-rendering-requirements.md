# Source investigation: safety and rendering requirements in public Rustdoc

- Investigation date and evidence cut-off: 2026-10-26.
- Question: should public contracts distinguish memory-safety obligations from
  rendering/SDK correctness requirements, and is another ADR justified?
- Baseline: development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`, SDK v2.3.0,
  Windows x64/MSVC + DX12, optional movable family. The inspected worktree was
  clean; supplemental revision identity is
  `98d4cc69d8b72af298f4ba14688ac452dd638985`.
- Method: inspect current Rustdoc and the corresponding loader, query,
  validation, descriptor and dispatch paths; compare accepted decisions with
  primary Rust and Microsoft documentation. No compilation, native execution,
  invalid-use experiment or image-quality study was performed.
- Output boundary: source findings and a proposed documentation convention.
  No Rust source, signature, accepted safety contract or release gate changes.

## Inputs

Repository paths below identify the inspected implementation. Line references
describe this baseline; the findings remain understandable without retrieving
the revision above.

| Input | Relevant locations and observations |
|---|---|
| [Local dispatch](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs) | `Upscaler::dispatch`, documentation/signature at lines 43–148: one long `# Safety` mixes resource/lifetime rules, native trust and signal interpretation. `validate_dispatch_inputs` explains its inspection limits. |
| [Local acquisition](../../../crates/fsr-sdk/src/runtime.rs) | `Runtime::load_unchecked`, lines 342–444: initialization/termination, ABI, shared native state and lasting Query/Configure/provider assumptions. |
| [Movable acquisition](../../../crates/fsr-sdk/src/runtime/movable.rs) | `MovableRuntime::load_unchecked`, lines 240–368: complete lasting contract for arbitrary destination threads, safe calls, errors, unwinding and cleanup. |
| [Movable upscaler](../../../crates/fsr-sdk/src/upscaler/dx12/movable.rs) | `dispatch`, documentation/signature at lines 291–339, inherits the local contract and restates GPU/ordering/cleanup duties. Its `Send` field audit and safe `destroy` connect acquisition to subsequent operations. |
| [Raw loader](../../../crates/fsr-sdk-sys/src/loader/windows.rs), [runtime state](../../../crates/fsr-sdk/src/runtime/state.rs) | Dynamic loading, typed symbol resolution, private entry points and forwarding of borrowed native descriptors. |
| [Query implementation](../../../crates/fsr-sdk/src/query/native.rs) | `copy_provider` uses `CStr::from_ptr`; enumeration passes writable capacity-bounded vectors. Checking the returned count afterwards cannot undo an out-of-bounds native write. |
| [Resource inspection](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs), [descriptor mapping](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs), [frame inputs](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs) | Pre-native scalar/profile/device/format/extent checks; borrowed COM inputs and temporary CPU descriptors. Inspection does not prove GPU states, storage aliasing, sampling bounds or completion. |
| [Construction and destruction](../../../crates/fsr-sdk/src/upscaler/dx12.rs), [context ownership](../../../crates/fsr-sdk/src/context.rs) | Safe ordinary creation and terminal destruction coexist with earlier unsafe acquisition/recording obligations. No automatic GPU wait. |

Reviewed decisions: [D008](../../adr/D008.md),
[D011](../../adr/D011.md), [D013](../../adr/D013.md) and
[D014](../../adr/D014.md), with the later [D024](../../adr/D024.md),
[D027](../../adr/D027.md), [D034](../../adr/D034.md) and
[D040](../../adr/D040.md) qualifications. The current
[API/safety synthesis](../api-and-safety.md) and the historical
[DX12 state/lifetime investigation](2026-09-24-src-m6-dx12-state-and-synchronization.md)
provide context; historical native results were not rerun.

External primary documentation, accessed 2026-10-26:

- [Rust Reference: undefined behaviour](https://doc.rust-lang.org/reference/behavior-considered-undefined.html),
  especially `undefined.soundness`, `undefined.pointer-access`, `undefined.race`,
  `undefined.call` and the FFI note. It identifies concrete UB mechanisms and
  warns that Rust's full unsafe semantics are not completely specified.
- [Rust Reference: the unsafe keyword](https://doc.rust-lang.org/reference/unsafe-keyword.html#unsafe-functions-unsafe-fn):
  an unsafe function exposes additional caller-established safety conditions.
- [Rust Reference: behaviour not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#logic-errors):
  logical constraints can have incorrect results, panics or aborts without UB.
- [Rust API Guidelines, C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure):
  recommend `Safety`, `Errors` and `Panics` sections. They do not prescribe a
  `Requirements` section or a graphics-specific classification.
- [Rustdoc book: writing documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#markdown):
  documents CommonMark support; a `Requirements` heading is ordinary Markdown.
- [Rustonomicon: safe/unsafe interaction](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html):
  safe abstractions must defend against ordinary safe-client mistakes; calling
  native code internally does not preclude a justified safe wrapper.
- [Rustonomicon: FFI and unwinding](https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-unwinding):
  distinguishes a Rust panic aborting at a non-unwinding ABI boundary from a
  foreign exception entering Rust through such a boundary, which is UB.
- [Standard library: `Send`](https://doc.rust-lang.org/std/marker/trait.Send.html)
  and [`CStr::from_ptr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.from_ptr):
  transferability and concrete readable-string/lifetime requirements. These
  pages identified themselves as Rust 1.99.1; the repository MSRV remains 1.88.
- [Microsoft: changes from D3D11 to D3D12](https://learn.microsoft.com/en-us/windows/win32/direct3d12/important-changes-from-directx-11-to-directx-12#explicit-synchronization):
  application responsibility for memory lifetime, residency and synchronization.
- [Microsoft: command allocator reset](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12commandallocator-reset#remarks):
  explicitly describes resetting an executing allocator as undefined behaviour
  in its D3D12 contract; command-list reset has different rules.
- [Microsoft: resource barriers](https://learn.microsoft.com/en-us/windows/win32/direct3d12/using-resource-barriers-to-synchronize-resource-states-in-direct3d-12#using-the-resourcebarrier-api-to-manage-per-resource-state):
  distinguishes transitions, aliasing and UAV ordering, with incomplete debug
  validation. It supplies no general mapping from violations to Rust-language UB.

The Rust Reference, Rustdoc, Guidelines and Microsoft pages are rolling
documentation, identified here by access date rather than a frozen revision.
The locally installed `libloading 0.9.0`, `src/safe.rs`, was also inspected:
`Library::new` requires safe initialization and termination, and `get` requires
the correct symbol type. The package is pinned in [Cargo.lock](../../../Cargo.lock)
with checksum
`754ca22de805bb5744484a5b151a9e1a8e837d5dc232c2d7d8c2e3492edc8b60`.
Attempts to retrieve its versioned web documentation and tagged source failed;
the local package supplied this evidence. No downloaded source snapshot is
needed for the conclusions below.

## Existing policy and the remaining gap

**Verified source facts:** D008 separates wrapper safety, API validity and a
bounded numerical runtime trust assumption. It expressly rejects treating a
controlled abort as proof of memory unsafety. D011's “Why the call is unsafe”
section distinguishes jitter/motion/camera semantics from memory-safety duties,
while retaining the asynchronous resource and terminal-error contract.
D014's “Safety, validation and failure behaviour” explicitly asks documentation
to keep image semantics distinct from unsafe lifetime/state obligations.

D013 requires the movable acquisition's full lasting obligations in public
`# Safety`, not just an ADR link or warning. Its `Send` promise includes
arbitrary-thread cleanup and unwinding, common native coordination and hidden
activity assumptions. Those are not reducible to correct image production.

**Verified implementation facts:** the present dispatch Rustdoc nevertheless
places pre-exposure, colour interpretation, mask contents and motion conventions
alongside GPU retention and native non-retention under `# Safety`. Movable
dispatch inherits all of these. Acquisition similarly combines native memory
assumptions with feature application/interpretation. This observation is a
documentation classification issue, not a demonstrated soundness defect.

**Inference:** another ADR is justified only for the missing common convention:
where to put each kind of requirement, how to mark uncertain native risks, and
how to apply that distinction without weakening the earlier decisions. Repeating
the dispatch design or reopening D013 would be redundant. Proposed
[D041](../../adr/D041.md) supplies this narrow extension.

## Classification of current obligations

The table classifies consequences, not keywords. **Safety** identifies a
concrete memory-safety/UB mechanism. **Correctness** identifies signal or SDK
behaviour within an otherwise supported, memory-safe access domain.
**Uncertain** means the available evidence cannot settle the safety consequence;
it does not mean optional or harmless. Mixed sentences must be split before any
future reorganization. Every existing obligation remains in force meanwhile.

| Existing requirement | Classification and justification | Documentation consequence if D041 is accepted |
|---|---|---|
| Compatible DLL exports, function signatures and retained executable code; safe native initialization/termination, including partial acquisition | **Safety.** A wrong calling ABI, access through unloaded code or memory-unsafe initializer can compromise the whole process. Raw loader and `libloading` supply the concrete boundary. Authentication is one input to trust, not proof of ABI or behaviour. | Keep the concrete ABI, code lifetime and initialization/cleanup assumptions in `Safety`. |
| Valid CPU descriptors/pointees; synchronous consumption of temporary storage; bounded query writes and readable terminated provider names | **Safety.** Retention can produce dangling stack pointers, native writes can overrun Rust allocations, and `CStr::from_ptr` requires a live readable string. Null checks and post-call count checks do not establish these properties. | Keep in acquisition/operation `Safety`; distinguish wrapper-owned retention from caller-established provider assumptions. |
| Native CPU-state serialization, non-reentrancy and compatible shared configuration | **Safety** where they prevent data races, invalidated pointers or use-after-free; **Uncertain** whether every forbidden overlap/configuration change has such effects in each closed provider. D008/D013 establish a conservative common-domain obligation, not a universal observed race. | Retain the full lasting obligation in `Safety`, explaining both the concrete risk and provider uncertainty. |
| Movable use and cleanup on every permitted destination thread, including Drop/unwinding and hidden native activity | **Safety** of the conditional `Send` abstraction depends on compatible operation/cleanup; **Uncertain** which actual providers have affinity or hidden termination constraints. D013 does not establish universal compatibility. | Keep the complete acquisition contract and connections on later operations. A fenced handoff is not proof for every permitted safe use. |
| Live COM objects during CPU calls, and no CPU/FFI access through released objects | **Safety.** Calling through a dead COM object or dangling native context has a concrete invalid-pointer mechanism. Borrowed inputs protect only the CPU borrow; external native misuse is not repaired by that borrow. | Keep concrete pointer/object validity obligations in `Safety`. |
| Resource/context/backing-allocation and allocator lifetime through GPU completion; no premature reuse/reset | **Safety** where native/GPU execution can access reclaimed or repurposed backing: this is a memory-lifetime hazard, not just wrong pixels. Its general propagation to Rust/host-memory UB is **Uncertain**. Microsoft's allocator rule establishes D3D12 undefined behaviour, not by itself compiler-level Rust UB. Dangling CPU pointers or conflicting mapped-host access supply independent concrete safety mechanisms. | Retain D011's full lifetime/completion rules in `Safety`, identifying native lifetime hazards and unresolved language-level consequences. Do not relax its command-list retention rule from the allocator/list distinction. |
| Truthful states, residency, non-overlapping active GPU backing and no conflicting GPU accesses | **Uncertain** for the general Rust-memory-safety consequence. Microsoft establishes native state/aliasing/residency rules, not that every breach corrupts host memory or that GPU isolation always contains it. Distinct COM identities do not prove separate backing. | Retain in `Safety` with the native-contract basis and unresolved consequence. Host-memory races/aliasing violations remain concrete safety cases. |
| Open recording DIRECT list, valid allocator and queue dependencies; one execution per successful recording and temporal ordering | **Uncertain** as a combined obligation: temporal history ordering also affects **Correctness**, but commands share state and allocations, and no harmless repeated/out-of-order execution contract is established. Metadata checks cover list type/device, not open state or execution. | Keep the accepted execution obligation in `Safety`; explain temporal semantics separately without deleting the native ordering rule. |
| Never submit failed recordings; abandoned OK recordings require terminal handling; complete work before destroy/Drop or later safe Configure | **Uncertain** native continuation/rollback safety, with concrete lifetime risks if pending commands later use destroyed objects. D011/D014 and acquisition/Configure trust deliberately preserve conservative terminal behaviour. | Keep in `Safety`, including lasting post-call duties and error paths. Safe `destroy`/Configure cannot acquire a new hidden caller safety precondition. |
| Jitter matching projection, motion direction/units and depth/camera interpretation | **Correctness** for semantic mismatches within a supported numerical/access domain, as D011 states. Extreme literal scales or content-dependent indexing outside an established provider domain are a separate **Uncertain** safety issue under D034. | Put signal meaning in `Requirements`; retain actual-provider numerical/access-domain assumptions in `Safety`. No blanket “all pixel errors are harmless” claim. |
| Pre-exposure multiplication, colour/HDR encoding, normalization, exposure ordering, mask signal alignment and interpretation of output | **Correctness** for signal meaning within the supported access domain. Whether an arbitrary encoding/value/profile can affect native arithmetic or resource access is **Uncertain**; D027 requires actual-provider compatibility. | Separate signal semantics from native profile/domain assumptions. Existing checks, mismatch rules and obligations remain. |
| Actual-provider support for active output, literal scales, format/view combinations, generation and optional controls | Mixed: safe access, numerical processing, non-retention and teardown are **Safety/Uncertain**; whether a request is honoured or yields the intended signal is **Correctness**. D024/D034 explicitly reject deriving access safety from bounds, finiteness or native OK. | Document both aspects and retain safety trust at its existing boundary. An ignored geometry request is not automatically a memory-safety failure. |
| Query success must populate outputs; selected requests must not silently use a default provider | **Correctness** for a wrong initialized numeric result or ignored selection alone. The inspected query buffers and numeric structures are initialized; leaving those values unchanged is not automatically an uninitialized Rust read. Invalid returned name pointers, overflowing writes or selecting an implementation outside the established safety domain remain separate **Safety** cases. | Separate trustworthy result semantics from bounds, pointer validity and actual-provider safety coverage. Do not remove the current acquisition requirement. |
| Restore renderer descriptor heaps/compute bindings after FSR | **Correctness** for subsequent renderer operation. A later draw using invalid addresses would carry its own memory-access obligations; this does not make every stale binding a demonstrated Rust-UB event. | Explain changed state and restoration in ordinary behaviour/`Requirements`, cross-referencing applicable safety invariants. |
| Checked scalars, device membership, formats, dimensions and resource-role metadata | Enforced wrapper invariants, potentially serving both safety and correctness. A rejected input is not an unfulfilled caller safety duty. Safe construction/preflight of a frame does not prove native dispatch safety. | Document validation and errors; keep only the unestablished residual obligations in `Safety`. Preserve existing validation. |
| Callback targets/userdata/code lifetime, compatible deallocation, writable host storage and shared callback state | **Safety** for dangling pointers, wrong allocator/alignment, invalid COM ownership and CPU races; **Uncertain** native shutdown/threading properties under D040 remain conditional. GPU CB-range reuse additionally retains the GPU/host distinction above. | Apply the same convention to callback-bearing and future APIs; preserve D040's lasting installation/error/cleanup contract. |
| No unwinding across the callback/native ABI; native faults need not return a Rust error | **Safety** for foreign exceptions entering Rust across a non-unwinding boundary. A Rust panic reaching `extern "C"` instead safely aborts under the documented Rust rule; panic, abort or a missing `Result` alone is not proof of UB. D040 already permits normal return or process termination and provides no exception-catching trampoline. | Retain the ABI/no-cross-boundary-unwinding contract in `Safety`; describe abort/error behaviour accurately rather than declaring every panic memory-unsafe. |

This is not a claim that any currently supported provider exhibits those
failures. Nor does classifying GPU-only effects as uncertain establish that
dispatch could be safe. Known CPU/FFI assumptions and unresolved asynchronous
native hazards remain at the existing unsafe boundary.

## Rustdoc structure and unsafe boundary

**Recommendation:** retain the conventional `# Safety` heading, with a clearly
labelled subsection for native assumptions and unresolved safety consequences.
Use optional `# Requirements` for grouped rendering/SDK correctness conditions;
short semantic explanations can stay in ordinary prose or parameter/type docs.
Preserve conventional `# Errors` and `# Panics` where applicable.

`Requirements` is a project convention, not a Rustdoc safety mechanism or a
heading required by C-FAILURE. `Rendering requirements` is more specific but
does not cover loader/SDK semantics; `Semantics` works for explanations but is
less explicit about obligations. `Preconditions` alone does not distinguish the
two classes. `Notes` is too vague for mandatory input conventions. None can
substitute for an unsafe function's complete safety contract.

An API needs `unsafe` when relevant safety conditions remain for the caller to
establish rather than being guaranteed by types, validation, ownership or an
already established unsafe acquisition contract. Internal FFI alone and incorrect
pixels alone do not justify making an operation unsafe. Conversely, absence of
a proven Rust-UB mechanism does not justify making an existing native boundary
safe. For a new uncertain boundary, `unsafe` is necessary where safety depends
on caller-established native assumptions, but is not itself a sufficient
soundness argument: the assumptions must be concrete and reviewable.

## Baseline delta, limits and remaining questions

- Confirmed: D011/D014 already contain the substantive distinction; D008/D013
  establish lasting trust and cleanup duties. No accepted design is superseded.
- Added: a documented classification and proposed common presentation rule.
  No Rustdoc migration or design acceptance has occurred.
- Unresolved: exact Rust/host-memory consequences of each GPU-only state,
  residency, aliasing, ordering or lifetime violation; complete numerical and
  sampling domains for actual providers; hidden-worker, non-retention and
  termination properties. Published native rules and bounded successful runs
  cannot settle all of them.
- A future reclassification needs an applicable native guarantee or analysis of
  the actual access path. A successful invalid-use probe would not prove general
  safety. No unsafe probe campaign or new release prerequisite is selected here.
- Any later documentation migration must review complete local and movable
  contracts together, including safe operations depending on earlier unsafe
  calls. Weakening a safety precondition or changing `unsafe` requires separate
  maintainer review and compatibility/soundness analysis.
- No SDK/dependency, licence, redistribution or platform-support change follows.
  Topic synthesis, research index and roadmap integration were not requested.
