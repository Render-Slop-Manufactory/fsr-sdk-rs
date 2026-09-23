# Source investigation: Embark fsr-rs ownership and safety model

- **Investigation date:** 2026-09-20
- **Topic and scope:** Static source investigation of Embark Studios' `fsr-rs`, specifically its FSR2 ownership, context lifecycle, backend scratch storage, graphics-device lifetimes, unsafe boundaries, failure handling, and thread-safety model.
- **Baseline and evidence cut-off:** Repository revision `2ec24b8abae1d6013d444cd472fa9712c584a21b`, corresponding to published `fsr` 0.1.11. The inspected bindings identify the bundled API as FSR2 **2.2.0**. The repository was archived on 2026-06-01. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))
- **Method and exclusions:** Inspected Rust wrapper source, `fsr-sys` generated bindings/build integration, and the bundled FSR2 native implementation where necessary to understand retained pointers, backend lifetime, device ownership, and destruction. Targeted history/issues searches were used only for design rationale; no useful lifecycle/thread-safety rationale was found. No compilation, execution, GPU experiment, sanitizer run, or modification of any repository was performed.
- **Repository revision/commit inspected:** `2ec24b8abae1d6013d444cd472fa9712c584a21b`.
- **Access date:** 2026-09-20.
- **Evidence terminology:** **Verified** means directly visible in the inspected source at that revision; it does **not** mean that runtime behavior was experimentally verified. **Upstream claim** denotes a comment/documentation contract. **Inference** denotes a conclusion from code structure. **Unresolved** denotes insufficient evidence.

## Scope and repository revision

**Verified —** `fsr-rs` is a two-crate workspace: `fsr-sys` provides the low-level generated/native-facing layer and `fsr` provides the higher-level Rust wrapper. The crate describes itself as “Unsafe Rust bindings for FidelityFX Super Resolution 2.” The fixed `fsr/src/lib.rs` source identifies commit `2ec24b8…`; the repository is now read-only. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** The generated FSR2 bindings at this revision encode FSR2 version 2.2.0, so conclusions about native implementation details below are specifically about the old FSR2 API bundled by this project, not FidelityFX SDK v2.3.0.

Primary inspected Rust paths include `fsr/src/lib.rs`, `fsr/src/interface.rs`, `fsr/src/d3d12.rs`, `fsr/src/vk.rs`, `fsr-sys/src/lib.rs`, `fsr-sys/src/bindings.rs`, the backend-specific generated bindings, and the `fsr-sys` native build inputs. Fixed-revision examples: [fsr/src/lib.rs](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs?utm_source=chatgpt.com) [fsr/src/interface.rs](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs?utm_source=chatgpt.com) [fsr/src/d3d12.rs](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/d3d12.rs?utm_source=chatgpt.com) [fsr/src/vk.rs](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/vk.rs?utm_source=chatgpt.com).

## Architecture

**Verified —** The higher-level native context representation is:

`fsr::Context { context: Box<fsr_sys::Context>, _interface: Interface }`.

The opaque native context is boxed because of its size, while the backend `Interface` is deliberately retained as a field of the Rust `Context`. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** `fsr::interface::Interface` contains both a raw `fsr_sys::Interface` and an owned `ScratchBuffer`. Its source explicitly comments that field order matters because the raw interface uses the scratch buffer. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Inference —** This is the most substantive ownership pattern in the wrapper: storage whose address has escaped into the native API is aggregated into the Rust object that represents the native context. It is an ownership relationship rather than a Rust borrow relationship.

**Verified —** The raw/high-level boundary remains permeable. `Interface::interface` and `Interface::scratch_buffer` are public, while raw FSR structures such as `fsr_sys::Interface` are C-layout bindings containing callback and pointer fields. The high-level layer therefore reduces some lifetime hazards but does not make the raw layer intrinsically safe.

**Verified —** The raw FSR2 `Interface` and opaque `Context` bindings are `Copy`/`Clone` in the generated layer. The high-level `fsr::Context` itself is not `Copy` or `Clone`.

**Inference —** The two-layer structure is meaningful: ordinary high-level use prevents accidental byte-copying of a live native context, while direct `fsr-sys` use deliberately exposes the native API's aliasing and duplication hazards.

## Context lifecycle

**Verified —** `Context::new` is `unsafe`. It allocates a default `Box<fsr_sys::Context>`, calls `fsr_sys::ContextCreate`, and returns an error immediately on a non-`FFX_OK` result. On success it moves `desc.interface` into the returned `Context`, thereby retaining the scratch allocation. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** Destruction is an explicit `pub unsafe fn destroy(&mut self) -> Result<(), Error>`. It calls `ContextDestroy` and maps a non-success error code into Rust's error type. It does not consume `self`, replace the native context with an invalid state, set a destroyed flag, or otherwise make the object unusable. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** There is no `Drop` implementation for `fsr::Context` in the inspected wrapper. Dropping a still-live `Context` therefore drops its boxed storage and owned `Interface`/scratch allocation without first invoking `ffxFsr2ContextDestroy`.

**Inference —** “Destroy exactly once” is not encoded. Safe ownership prevents cloning the high-level `Context`, but repeated calls to the `unsafe` `destroy(&mut self)` are type-correct, as are calls to `dispatch` after a successful `destroy`.

**Verified —** The old bundled native `ffxFsr2ContextDestroy` performs native release but does not overwrite or zero the entire opaque context afterward. Its release path nulls selected state, including the backend device after backend destruction.

**Inference —** That partial mutation reinforces rather than removes the post-destroy-state concern: the Rust object remains present and contains native bytes that are neither represented as a distinct Rust state nor contractually reusable.

**Verified —** In the old native implementation, dispatch checks for a null context device and can therefore reject a context after the old release path has nulled that field. This is an old FSR2 2.2 implementation detail, not a Rust-side use-after-destroy prevention mechanism and not evidence for FidelityFX SDK v2.3.0.

**Upstream claim —** The generated FSR2 comments describe the context as holding persistent data/resources and state that GPU work using those resources should no longer be active when destruction occurs.

**Inference —** Omitting explicit `destroy` is more than an ordinary Rust resource leak in the old backends: expected backend teardown, GPU-resource cleanup, and—on D3D12—the matching device `Release` are skipped before the scratch allocation is freed.

## Lifetime and ownership model

**Verified —** `ContextDescription<'a>` contains `device: &'a Device`, but conversion to the native description copies `*val.device` into a raw `FfxDevice`; the returned `Context` has no lifetime parameter related to `'a`. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Inference —** This lifetime only keeps the Rust variable holding the raw `Device` value borrowed during construction. It does not express that the underlying D3D12/Vulkan device must remain valid for the native context's lifetime.

**Verified —** Scratch storage receives stronger treatment. `ScratchBuffer` owns an allocation; `Interface` owns the `ScratchBuffer`; and `Context` owns the `Interface`. The raw interface receives the scratch pointer. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Upstream claim —** `ScratchBuffer::ptr` documents that nothing may still use the pointer when the buffer is dropped and that normal Rust aliasing obligations remain with code using the pointer. Its `Drop` comment repeats this obligation. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Verified —** The old FSR2 native creation path copies the supplied context description, including its backend callback interface, into persistent context state. The Vulkan backend additionally places backend state in the supplied scratch allocation and records Vulkan instance/physical-device information and proc-address functions there.

**Inference —** For this old API, the “create-time pointer may actually be context-lifetime state” problem is real for the scratch pointer: it is not merely transient call input. `fsr-rs` handles that particular dependency by ownership aggregation.

**Verified —** That protection is not encoded using Rust lifetimes. The raw `fsr_sys::Interface` itself is copyable and contains the raw scratch pointer, and the high-level `Interface` exposes that raw member publicly.

**Inference —** A user crossing into `fsr-sys` can detach a raw interface value from the `ScratchBuffer` owner and use it after the allocation has gone away. The high-level ownership pattern therefore protects its intended construction path, not every reachable raw-FFI path.

### D3D12 device

**Verified —** `d3d12::get_device(&mut ID3D12Device)` converts the COM object pointer to a raw FSR `Device`; no Rust lifetime is attached to the result. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/d3d12.rs))

**Verified —** In the bundled FSR2 D3D12 backend, backend-context creation obtains the `ID3D12Device*`, calls `AddRef`, and stores it. Backend destruction later calls `Release` and clears its stored device pointer. The separate `ffxGetDeviceDX12` helper itself is only a pointer conversion; the ownership increment happens when the backend context is created. ([raw.githubusercontent.com](https://raw.githubusercontent.com/EmbarkStudios/fsr-rs/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr-sys/FidelityFX-FSR2/src/ffx-fsr2-api/dx12/ffx_fsr2_dx12.cpp))

**Inference —** For this old D3D12 implementation, successful native context creation establishes an internal COM reference that can keep the device alive independently of the caller's original reference. Correct balancing nevertheless depends on native context destruction actually occurring.

### Vulkan device and loader objects

**Verified —** `vk::get_interface` borrows `&ash::Entry` and `&ash::Instance` only during the call, allocates scratch storage, and passes instance/physical-device handles and Vulkan proc-address functions into `GetInterfaceVK`. It returns an `Interface` with no lifetime tied to either borrowed Rust object. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/vk.rs))

**Verified —** The bundled Vulkan backend stores this backend state in the scratch allocation. Backend-context creation records the Vulkan device handle and uses it for later resource operations; backend destruction uses that stored device to destroy resources. Vulkan has no COM-style `AddRef` in this path. ([raw.githubusercontent.com](https://raw.githubusercontent.com/EmbarkStudios/fsr-rs/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr-sys/FidelityFX-FSR2/src/ffx-fsr2-api/vk/ffx_fsr2_vk.cpp))

**Verified —** `vk::get_device(device: ash::Device) -> Device` returns only the raw FSR device value. The resulting `Context` carries neither an `ash::Device` field nor a lifetime connecting it to the supplied device. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/vk.rs))

**Inference —** Validity of the Vulkan device and any loader/instance state required by retained function pointers is a caller obligation rather than a relationship enforced by the wrapper's types.

**Unresolved —** Static inspection of `fsr-rs` alone does not establish the complete lifetime requirements of the particular `ash` version's `Entry`, `Instance`, and `Device` wrappers, nor whether all retained proc pointers remain callable after corresponding Rust wrapper values are dropped. That requires the relevant `ash` contracts and/or an execution experiment.

## Unsafe boundaries

**Verified —** The principal lifecycle methods—`Context::new`, `Context::dispatch`, and `Context::destroy`—are all `unsafe`, as are backend helpers such as `get_interface`/`get_device`. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** The inspected `Context` methods do not have detailed `# Safety` documentation stating all native lifetime invariants. In contrast, `ScratchBuffer::ptr` does document its escaped-pointer/drop obligation. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Inference —** Important requirements are therefore communicated mainly by the fact that operations are `unsafe`, by isolated comments, and by native API structure, rather than by a comprehensive Rust safety contract.

**Verified —** Message callbacks are represented as `Option<unsafe extern "C" fn(MsgType, *const WideChar)>`. The old context description has no Rust closure, owned callback object, or callback-userdata field; the function pointer is copied into the raw context description. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Inference —** The old message callback avoids one common userdata-lifetime problem because there is no arbitrary userdata pointer in this API shape. Any state accessed by the callback through global/external means remains outside the wrapper's ownership model.

**Verified —** Backend callback function pointers and scratch/native pointers live in `fsr_sys::Interface`; the Rust wrapper owns the scratch allocation but not any typed object corresponding to those pointers.

**Verified —** `ScratchBuffer::new` checks layout construction but directly accepts the pointer returned by `std::alloc::alloc`; it does not represent allocator-null failure as an error. The `Result` covers `LayoutError`, not allocation exhaustion. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Inference —** This is a low-level memory-safety robustness concern independent of FSR2 lifetime design: the allocation abstraction assumes successful allocation while subsequently exposing/deallocating the returned pointer.

**Verified —** The crate-level Vulkan pseudo-code still says that a context must not outlive an externally supplied scratch buffer, but the inspected `vk::get_interface` implementation now allocates/owns the scratch buffer internally. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Inference —** The documentation was not fully updated after the ownership design changed, so comments/examples should not be treated as a complete description of the actual lifetime model.

## Error and failure handling

**Verified —** `Context::new`, `dispatch`, and `destroy` translate a returned FSR error code into `Result`. `destroy` leaves the Rust object structurally unchanged on either success or error. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Inference —** After a destruction error, the Rust type provides no indication whether native teardown was absent, partial, or complete. Retrying, dropping, or using the context all remain representable.

**Verified —** On `ContextCreate` failure, `Context::new` immediately returns `Err`. It does not call `ContextDestroy` or another rollback API before the local `Box` and input `Interface` are dropped. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

**Verified —** In the bundled old FSR2 implementation, context creation zeroes initial context storage, then creates backend state and subsequently performs additional capability/resource/pipeline setup with error-return paths. There is no wrapper-level cleanup call when one of those later errors reaches Rust. ([raw.githubusercontent.com](https://raw.githubusercontent.com/EmbarkStudios/fsr-rs/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr-sys/FidelityFX-FSR2/src/ffx-fsr2-api/ffx_fsr2.cpp))

**Inference —** If a reachable native error occurs after backend/resource initialization and that native error path does not itself fully roll back, `fsr-rs` can discard the partially initialized context and scratch owner without invoking the normal destroy sequence. Static inspection identifies the control-flow risk; it does not prove an actual leak or dangling GPU resource for a particular failure.

**Verified —** D3D12 and Vulkan interface construction differ in error handling. `vk::get_interface` checks the `GetInterfaceVK` return code; `d3d12::get_interface` calls `GetInterfaceDX12` but discards its returned `ErrorCode` and returns `Ok(retval)` unconditionally after scratch allocation. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/d3d12.rs))

**Inference —** A D3D12 backend-interface initialization failure can therefore be hidden from the caller at this wrapper boundary.

**Verified —** The old core destruction implementation ultimately returns `FFX_OK` from its release path after performing its cleanup sequence; backend release results are not exposed as a rich transactional teardown state.

**Unresolved —** Whether any practically reachable old-FSR2 destruction path can partially mutate state and still return a failure cannot be established from the wrapper alone. FidelityFX SDK v2.3.0 must be investigated independently.

## Thread-safety

**Verified —** `ScratchBuffer` has explicit `unsafe impl Send` and `unsafe impl Sync`; `Interface` likewise has explicit unsafe `Send` and `Sync` implementations. No synchronization primitive accompanies those implementations. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs))

**Verified —** `Context` has no explicit `Send`/`Sync` implementation. Given its fields, the published type receives auto-trait `Send` and `Sync`; generated documentation for this crate also lists both traits for `Context`.

**Verified —** Mutating operations such as `dispatch` and `destroy` require `&mut self`, so ordinary safe Rust cannot invoke those methods concurrently through the same unsynchronized `Context` reference.

**Inference —** `Send` permits moving a context between threads, and `Sync` permits sharing `&Context` across threads. The wrapper therefore makes stronger Rust auto-trait statements than a conservative “thread-safety not assumed” policy would.

**Unresolved —** No inspected source comment or targeted issue/history result supplied a native FSR2 guarantee that justifies the explicit `Sync`/`Send` assertions for `Interface` or the resulting auto traits of `Context`. Static inspection cannot establish whether moving a live context between threads is supported by all old FSR2 backends.

## Patterns relevant to fsr-sdk-rs

**Inference —** A generally useful pattern is the aggregation of native backing storage with the Rust owner of the native context: `Context -> Interface -> ScratchBuffer`. This directly addresses a class of APIs where a pointer supplied at creation remains live until context teardown, without assuming that FFI input pointers are temporary.

**Inference —** The separation between `fsr-sys` and `fsr` is also generalizable as an architectural observation: raw bindings remain faithful and unsafe, while a separate wrapper can impose stronger ownership/non-copy semantics without hiding the underlying FFI surface.

**Inference —** Making the high-level `Context` non-`Copy`/non-`Clone` is a useful reduction of accidental duplicate-handle destruction relative to the raw generated `Context`, even though it does not solve repeated explicit destruction.

**Inference —** Explicitly retaining dependencies by value can be more robust than merely accepting borrowed values at creation. The scratch-buffer handling demonstrates this; the graphics-device handling demonstrates the converse, where a short-lived constructor borrow does not express a context-long native dependency.

**Inference —** Backend-specific ownership needs to be determined separately. In old FSR2, D3D12 establishes a native COM reference during backend creation, while Vulkan retains unowned handles. A single generic Rust `Device` pointer obscures that distinction.

These are observations for later synthesis with SDK v2.3.0 evidence, not an architecture recommendation.

## Patterns that should not be copied blindly

**Inference —** Explicit `destroy(&mut self)` with no state transition leaves double-destroy and use-after-destroy structurally possible. Marking the function `unsafe` transfers the invariant to callers rather than encoding it.

**Inference —** Omitting `Drop` while the type owns resources whose correct cleanup requires native destruction makes forgotten teardown possible and causes Rust field destruction to proceed independently of the native lifecycle.

**Inference —** Conversely, the old native behavior should not be used to conclude that adding an unconditional destructor would itself be correct for v2.3.0: the user's baseline already identifies uncertainty around failed creation, failed destruction, and post-destroy state. Those questions must first be resolved against the newer SDK.

**Inference —** Public exposure of a copyable raw `Interface` containing a scratch pointer weakens the lifetime protection obtained by embedding the scratch owner beside it.

**Inference —** The `ContextDescription<'a>::device: &'a Device` lifetime can give a misleading impression of long-lived device borrowing even though no such lifetime reaches the resulting context.

**Inference —** The explicit `unsafe impl Send + Sync` on scratch/interface storage should not be interpreted as evidence that the native FSR context or backend is thread-safe.

**Inference —** Silently discarding the D3D12 backend-interface initialization error is particularly unsuitable as a model for a wrapper intended to reason carefully about partially initialized native state.

**Verified —** No lifetime-specific TODO resolving these issues was found in the inspected wrapper. The visible TODO in the core error type concerns improving which module dependency failed, not ownership/lifetime safety. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs))

## Comparison with our FidelityFX v2.3.0 findings

| Current v2.3.0 concern | What `fsr-rs` / old FSR2 shows | Evidence |
|---|---|---|
| Context must be destroyed exactly once | High-level `Context` is non-copyable, but `destroy(&mut self)` neither consumes nor invalidates it; repeated destruction remains possible. No `Drop` guarantees one teardown. | **Verified —** wrapper API shape. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs)) |
| Native handle state after destroy may not be trustworthy | Old native teardown mutates selected internal state but does not comprehensively invalidate/zero the opaque object; Rust retains the same `Context` afterward. | **Verified —** old FSR2 source behavior; **Inference —** the Rust object cannot encode whether further operations are valid. |
| Failed creation may leave unclear output state | `fsr-rs` returns immediately on create failure with no wrapper cleanup. Old native creation has error-return points after backend initialization has begun. | **Verified —** control flow; **Unresolved —** actual resource state for each failure path requires runtime/native-path verification. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/lib.rs)) |
| Create-time pointer lifetimes may extend until destroy | The old interface's scratch pointer is persistent state; `fsr-rs` deliberately owns that storage through the `Context`. Vulkan interface creation also places persistent backend data in scratch. | **Verified —** old FSR2 source and wrapper ownership. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs)) |
| Device ownership/AddRef may be unclear | For the bundled old D3D12 backend, source resolves this: backend creation `AddRef`s and backend destruction `Release`s. Vulkan instead stores an unowned handle. | **Verified —** old FSR2 only. ([raw.githubusercontent.com](https://raw.githubusercontent.com/EmbarkStudios/fsr-rs/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr-sys/FidelityFX-FSR2/src/ffx-fsr2-api/dx12/ffx_fsr2_dx12.cpp)) |
| Thread-safety is not assumed | `fsr-rs` nevertheless marks backend storage `Send + Sync`, making `Context` inherit these auto traits. No inspected rationale establishes the native guarantee. | **Verified —** Rust trait behavior; **Unresolved —** native safety of cross-thread movement/use. ([github.com](https://github.com/EmbarkStudios/fsr-rs/blob/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr/src/interface.rs)) |

## Baseline delta

### Confirms

**Verified —** The investigation confirms that create-time native data can have context-long lifetime in an FSR-family API. In old FSR2 the backend scratch buffer is persistent storage, not temporary construction workspace; Embark explicitly arranges Rust ownership so that it survives with the context.

**Verified —** It also confirms that “exactly once” destruction cannot be assumed to fall out automatically from ordinary Rust ownership when the native destructor is exposed as a repeatable `&mut self` operation.

**Inference —** It reinforces the baseline caution about post-destroy state: even where the old implementation clears selected fields, that is not equivalent to a reliable reusable or safely inspectable native handle.

### Contradicts

**Verified —** For **old FSR2 D3D12 specifically**, the “does the backend hold a device reference?” question is not unclear after source inspection: backend-context creation calls `AddRef`, and destruction calls `Release`. This resolves that old implementation detail but does not contradict uncertainty about FidelityFX SDK v2.3.0. ([raw.githubusercontent.com](https://raw.githubusercontent.com/EmbarkStudios/fsr-rs/2ec24b8abae1d6013d444cd472fa9712c584a21b/fsr-sys/FidelityFX-FSR2/src/ffx-fsr2-api/dx12/ffx_fsr2_dx12.cpp))

**Verified —** The old 2.2 implementation also initializes/zeros context storage on creation and nulls selected state during destruction. Those facts narrow questions about this particular implementation but do not establish the newer SDK's failure or destruction contract.

### Refines

**Inference —** “Failed creation may leave unclear output state” can be sharpened for `fsr-rs`: the wrapper itself has no partial-initialization state machine or rollback policy. Once native creation reports failure, it drops the Rust-owned storage immediately. Whether that is sufficient depends entirely on guarantees of the native create failure path.

**Inference —** “Device lifetime” is backend-specific rather than merely a generic raw-pointer concern: old D3D12 uses COM ownership internally after successful backend creation; old Vulkan does not acquire analogous ownership and therefore depends on caller-managed handle validity.

**Inference —** “Create-time pointer lifetime” should distinguish owned backing memory from other borrowed native objects. `fsr-rs` handles scratch ownership comparatively well while leaving graphics-device/loader relationships largely as unsafe obligations.

### Leaves unresolved

**Unresolved —** Nothing in this investigation establishes FidelityFX SDK v2.3.0's creation-failure postconditions, destruction-failure semantics, device reference behavior, pointer-retention rules, or thread-safety guarantees.

**Unresolved —** Source inspection alone does not establish whether every partially initialized old FSR2 creation failure leaks, whether a second destruction actually faults on a given backend, or what GPU-visible consequences result from dropping `fsr::Context` without first calling native destruction.

## Open questions

**Unresolved —** Was explicit destruction instead of `Drop` a deliberate decision to avoid infallible-destructor semantics, or simply an incomplete wrapper? No inspected comment, issue, or history item supplied a rationale.

**Unresolved —** Were the explicit `unsafe impl Send/Sync` declarations audited against AMD's FSR2 concurrency contract, or added solely because the raw scratch pointer prevented desired auto traits? No rationale was found.

**Unresolved —** Does every old Vulkan function pointer retained in scratch remain valid for the entire FSR context lifetime if the originating Rust `ash::Entry`/`Instance` wrapper is no longer retained? This needs the exact `ash` and Vulkan loader contracts.

**Unresolved —** For each old FSR2 native creation error after backend creation starts, what resources have actually been acquired and which are rolled back before return? Static control-flow inspection shows the risk but not all runtime side effects.

**Unresolved —** The meaning of successful or failed repeated `ContextDestroy` calls is not specified by the Rust wrapper. Behavior observed in the old source should not be elevated into a stable API contract.

## Limits and follow-up

**Unresolved —** This record is a static source investigation. It does not prove runtime behavior, GPU synchronization behavior, allocator-failure behavior, COM reference counts in a running process, Vulkan object validity, or the effects of invoking APIs after destruction.

**Unresolved —** To determine old-FSR2 runtime consequences of the identified lifecycle gaps, useful experiments would include instrumented create-failure injection at each native initialization stage; D3D12 COM reference-count tracing around successful create, failed create, explicit destroy, and Rust-only drop; Vulkan validation-layer runs for premature device/instance destruction; repeated-destroy and dispatch-after-destroy tests; and ASan/Valgrind-equivalent host-memory checking around scratch-buffer lifetime.

**Unresolved —** To apply any pattern to FidelityFX SDK v2.3.0, the corresponding v2.3.0 headers, backend implementations, documentation, and—where contracts remain ambiguous—native experiments must be examined independently. In particular, the old FSR2 2.2 D3D12 `AddRef`, Vulkan scratch layout, context zeroing, and post-destroy checks must not be assumed to survive into the newer FidelityFX SDK architecture.

**Unresolved —** A separate v2.3.0 investigation should establish explicit answers for: whether creation failure permits or requires destruction; whether destruction can fail after partial teardown; whether a destroyed context's bytes have any contractual state; exactly which create-time pointers are retained; whether each backend retains/refcounts its device; and whether any native context/interface is guaranteed movable or concurrently usable across threads.
