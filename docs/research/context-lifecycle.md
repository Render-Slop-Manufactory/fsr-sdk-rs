# context lifecycle

Updated: 2026-10-09 (D013 lifecycle review, acceptance and separate public implementation verification recorded).

Extracted from the [API and safety synthesis](api-and-safety.md).
These findings retain their original evidence and limitations.

The [2026-10-09 maintenance recheck](../release/2026-10-09-windows-native-recheck.md)
repeated explicit/Drop lifecycle, shared contexts, public movable cleanup and
the existing isolated failure probes on its captured baseline. Normal cleanup
passed; injected allocation failures again caused access violations before create
returned. Returned-error retain/protect/cleanup variants completed the existing
two-second observation after a stale prerequisite marker was repaired in the
runner. D005's bounded assumptions and universal native failure gaps remain.

## Context lifecycle: v2.3.0 source findings

The [lifecycle report](records/2026-09-20-src-context-lifecycle.md) examines tag
`v2.3.0`, commit `60f4ea8`. These are reported source findings, not new runtime
results; the signed FSR4 and external/driver providers are not characterized by
the open FSR2/FSR3 implementations.

**Upstream contract:** ordinary DX12 upscaler creation needs an upscaler root,
`ffxCreateContextDescUpscaleVersion` with `FFX_UPSCALER_VERSION`, and a DX12
backend descriptor containing a live device. The effect-specific documentation
requires the version node even though the generic example omits it. Optional
`ffxOverrideVersion` instead selects an enumerated provider ID; it does not
replace the API-version node. The existing count-only query supplies no IDs.
Descriptor tags must match their actual payload types; mismatch is not merely a
recoverable validation error.

**Reported source behavior:** creation clears a valid output slot after checking
both input pointers, then delegates to a provider. Open FSR2/FSR3 providers publish
the handle only on success; there is no provider-independent failed-create
postcondition. Destruction in these open paths frees the context without clearing
the caller's handle, contradicting the generic documentation. Provider lookup
assumes a live handle: destroying a null handle or destroying twice is hazardous.
A null pointer-to-handle has a distinct error path and is not equivalent to a
pointer containing a null handle.

**Unresolved cleanup:** a create error does not establish that destruction is
valid or required. A destroy error does not establish that the context is still
usable, that teardown did nothing, or that retrying is safe. Native return codes
must remain representable beyond today's known constants. Exact-once teardown
needs independent Rust state; testing the raw handle for null is insufficient.

**Lifetime constraints:** keep the device and retained callback code live through
native use. Open providers retain the message callback. The destroy-path report
identifies device reference retention/release in the open v2.3.0 DX12 backend;
equivalent ownership in the signed/external 4.1.1 provider remains unresolved.
The header's generic pointer-lifetime wording does not establish a uniform
chain-node lifetime: the override node is explicitly call-scoped, while the
concrete upscaler/version/backend chain remains unresolved for signed providers.
Host allocation callbacks passed separately to create/destroy need compatible allocation state,
not the same callback-struct address. They are distinct from DX12 backend
allocation callbacks carried in the chain.

**Inference:** live contexts depend on callable provider code, so library ownership
must extend beyond individual forwarding calls. Provider selection can vary with
the device and driver; record the selected provider with lifecycle observations.
No usable upscaler concurrency guarantee was found. Neither Rust auto traits nor
successful stress tests establish that contract.

## Descriptor, callback, device and threading contracts

The [2026-09-22 lifecycle contract investigation](records/2026-09-22-src-lifecycle-contracts.md)
refines the earlier report against the same v2.3.0 baseline. These are reported
source findings, not additional runtime verification.

- **Descriptor storage:** tagged documentation explicitly permits the
  `ffxOverrideVersion` node to expire after creation. This contradicts a blanket
  contractual requirement to retain every chain node until destruction. Open
  FSR2/FSR3+DX12 paths consume/copy the inspected descriptors without retaining
  their addresses. Neither fact establishes call-only lifetime for the concrete
  `UpscaleVersion` or `BackendDX12` nodes in a signed provider. Conservative
  retention remains a wrapper policy, not a demonstrated universal AMD rule.
- **Message callback:** open FSR2/FSR3 providers retain `fpMessage` per context
  and install it in unsynchronized module-global state. Destroy does not clear
  that global pointer, so the dependency can outlive its installing context.
  Inspected debug dispatch calls it synchronously, but no public lifetime,
  calling-thread, reentrancy or serialization guarantee was found for signed
  providers. Null is explicitly permitted and introduces no callback dependency
  of its own; it does not characterize separately configured global callbacks.
- **Device ownership:** the open DX12 backend takes one COM reference when its
  scratch/backend instance gains its first effect context and releases it after
  the last. This is not necessarily one reference per API context. The public
  descriptor promises no ownership transfer or permission to release the caller's
  reference immediately; signed-provider COM retention remains unresolved.
- **Concurrency:** no upscaler contract guaranteeing or prohibiting same-context,
  cross-context, creation/destruction or provider-enumeration concurrency was
  found. Mutable context state, global callbacks and shared version-name storage
  prevent inferring isolation from separate Rust owners. Frame Generation has
  explicit synchronization rules, but those do not transfer to Upscaling.

**Design implication:** D005's null callback and retained dependencies avoid
some unresolved dependencies, but excluding Send/Sync alone supplies no global
native threading contract. Non-null callback support needs a broader lifetime
model than context-local ownership. The failed-create spike does not resolve
successful-context descriptor lifetimes, callback behavior or threading. D005
is now accepted; suggested instrumentation in the source report is future
research, not executed evidence or authorization to run it.

## Teardown after submitted GPU work

**Verified bounded observation:** the
[final Windows verification](records/2026-09-28-exp-final-windows-release-verification.md)
repeated public construction/dispatch on the pinned v2.3.0 Windows x64/MSVC
baseline, with no production-code change. After Gate A's submitted frame
completed its fence/readback, consuming `upscaler.destroy()` returned success.
Gate B used one runtime/device/upscaler for three ordered moving frames, each
submitted and fenced before the next. Normal Drop completed after frame 2
while the runtime, caller device and all frame resource sets remained alive.
Checks after context teardown and frame-resource cleanup reported zero D3D12
errors/corruption and `S_OK` device status on the recorded RX 9060 XT/driver
32.0.31041.1004/provider 4.1.1 configuration. Dispatch details and temporal
coverage remain in the [dispatch synthesis](upscaling-dispatch.md).

**Safety limit:** these observations add successful teardown after GPU work to
the earlier lifecycle-only evidence. They do not establish minimum dependency
lifetimes, provider COM ownership, native failure cleanup or general concurrency.
The public API exposes no native Drop result; clean subsequent observations
cannot substitute for that unavailable return code. D005's terminal no-retry/
retention policy and D011's caller-managed GPU completion obligations remain
unchanged. No failed or abandoned native recording was exercised.

## Destroy failures: source paths and limits

The [destroy-path investigation](records/2026-09-20-src-destroy-failure-paths.md)
supplies commit-pinned sources for v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`.
These are reported static findings about the common layer, open FSR2/FSR3
providers and their DX12 backend, not additional runtime observations or an
implementation description of the signed 4.1.1 provider.

**Reported source behavior:** the common destroy entry point checks the outer
pointer, reads the associated provider through the handle, and forwards the
provider's result unchanged. It then performs zero-refcount provider cleanup
even if that result is an error. The inspected static FSR2/FSR3 providers do not
reach zero on this path; external-provider reference accounting is unavailable.
An error alone therefore does not establish that provider code/state survives.

| Inspected path | Error handling and post-state | Reachability limit |
|---|---|---|
| Null pointer-to-handle argument | Parameter error 6 before teardown | No live context involved; does not test post-failure liveness |
| Pointer to a null, stale or invalid handle | Provider lookup dereferences the handle before inner provider checks | No controlled error or safe validation established |
| Open FSR2 valid context | Core release ignores backend cleanup results and returns success; provider then frees scratch/context | No legitimate ordinary-use destroy error identified |
| Open FSR3 shared-resource loop | Backend errors become public runtime error 3; earlier entries may already be released, while core/scratch/context teardown has not completed | Stock DX12 trigger found is an out-of-range private resource index; no valid-state supported-use trigger identified |
| Host deallocation callback | Returns void, with no error channel | Allocator incompatibility is a contract violation, not a recoverable destroy-error mechanism |

Both open core destroy functions have a null-core-pointer error, but valid
providers pass the address of an embedded object. That branch is not a supported
failure trigger for a live context. FSR2/FSR3 core safe-release helpers discard
backend destruction results; this differs from the FSR3 provider's earlier
shared-resource loop, which propagates them. The mere existence of an API error
constant does not show that a destroy path can emit it.

**Inference:** destruction is not established as transactional. The conditional
FSR3 error path can follow partial teardown; it does not justify treating an
error as an untouched, usable context. Neither inspected documentation nor the
source establishes a general retry contract. This conditional source path was
not exercised and must not be presented as an observed valid-context failure.

**Unresolved:** the signed loader's exact forwarding/validation and the
FSR4/ML/external provider's destruction implementation are unavailable in the
inspected public source. Open-provider behavior cannot classify a hypothetical
4.1.1 destroy error as live, dead or partially destroyed. No legitimate,
non-corrupting valid-context failure trigger suitable for a runtime experiment
was identified. The report's null-outer-pointer case concerns API conformance
only; it does not close that gap or authorize a new experiment.

The report also confirms that the C++ convenience wrapper merely forwards
destroy and does not clear the handle. This reinforces the existing
documentation conflict and successful lifecycle observation without proving
that the signed provider uses the same internal implementation. No Context,
Drop or failure policy is accepted by integrating these findings.

## Context lifecycle: observed runtime behavior

The [successful lifecycle experiment](records/2026-09-20-exp-context-lifecycle.md)
and [allocation-failure experiment](records/2026-09-20-exp-create-allocation-failure.md)
use the local SDK v2.3.0 loader/upscaler DLLs on Windows x64/MSVC with an AMD
Radeon RX 9060 XT and driver `32.0.31041.1004`. Both retain the complete
root -> version -> DX12 descriptor chain, device and libraries. No dispatch is
submitted. The creation descriptors have since been promoted into sys for the private M4
owner; provider-query instrumentation remains test-local. These historical
results do not themselves verify the new production owner or establish public construction.

**Verified M4 production-owner execution:** the
[Windows verification](records/2026-09-22-exp-m4-windows-verification.md)
now closes the implementation's Windows execution gap on build 26200.9457 with
the same RX 9060 XT, driver and fingerprinted SDK inputs. Formatting, Clippy,
17 ordinary executable tests, two compile-fail doctests, no-default-feature
checking and paired MSVC C++ ABI compilation passed. Windows fixtures exercise
the production loader and verify terminal teardown, ordered release, exceptional
retention and DLL residency. These are wrapper-policy results, not AMD failure
postconditions. The isolated native test created through the production
`Upscaler` / `NativeContextOwner`, moved the outer owner, identified provider
`0xf5a5ca1e01001001` / `4.1.1`, and successfully destroyed it; exit 0 without
timeout. No production change was needed. One successful lifecycle does not
establish dispatch, image correctness, native failure cleanup, measured COM
reference counts or general threading guarantees. D005's runtime trust assumption
and remaining provider-contract questions are unchanged.

**Verified successful lifecycle:** creation returned OK with a non-null context;
the provider query identified `0xf5a5ca1e01001001`, name `4.1.1`; destruction
returned OK without nulling the handle. This independently reproduces the
source-reported non-nulling behavior on the tested runtime path and contradicts
the generic documentation's null-after-destroy claim for that path. It does not
establish minimum descriptor/device lifetimes or general GPU compatibility.

**Documented allocation contract:** the local `ffx_api.h` explicitly permits
the host allocator to return null to indicate failure. It requires suitable
alignment, acceptance of null free arguments and compatible create/destroy
allocators. It supplies no universal failed-create handle or rollback guarantee.

**Verified allocation observations:** instrumented successful controls reached
three host allocations of 1,099,864, 1,634,664 and 16 bytes. Create and destroy
returned 0; destroy freed all three exactly once, in order 2, 1, 3. The same
callbacks were used for both calls; replacement-callback compatibility was not
tested. The following outcomes were reproduced in two process-isolated sweeps:

| Injected null at attempt | Successful allocations / observed frees | Outcome |
|---|---|---|
| 1 | 0 / 0 | Access violation before create returned |
| 2 | 1 / 0 | Access violation before create returned |
| 3 | 2 / 0 | Access violation before create returned |
| 4 (not reached) | 3 / 3 after destroy | Successful control |

All injected cases ended with process status `0xC0000005`, not an FFX return
code. Their handles were null before create; neither a returned error nor an
after-create handle state was observable. No free callback occurred before the
crashes, and no destroy-after-failure or speculative manual cleanup was attempted.
The successful controls identified provider 4.1.1; failed processes could not
independently query provider identity. Neither experiment identifies whether the
implementation came from the bundled provider DLL or the driver.

**Inference:** on this tested path, permitted host-allocation failures did not
become recoverable FFX errors, and indices 2 and 3 showed no callback-visible
partial cleanup before termination. This is not evidence of a usable failed
context or permission to destroy it. OS process reclamation is not SDK cleanup.

**Unresolved:** exact fault location/cause was not captured; the results do not
attribute a defect to a specific module. Accounting covers only these host
callbacks, not GPU allocations or other internal allocators. Those crashes supply
no returned-error postconditions; the separate experiment below covers one
returned rejection. Other providers/builds and failed destruction remain untested.
A successful control does not
establish general leak freedom, and a crashing injection does not establish a
universal failure contract.

## Returned create error: caller input lifetime

The later [D013 source intake](records/2026-10-07-src-d013-transfer-cleanup-contract.md)
recommends quarantine for ambiguous returned-create-error ownership, but supplies
no new contradictory execution or pinned provider postcondition. D005's accepted
bounded RAII trust assumption below remains in force; it never authorizes
destroying an ambiguous failed handle. Fixture postconditions and source
recommendations do not substitute for the selected provider's native contract.

The [D005 input-lifetime spike](records/2026-09-22-exp-create-error-input-lifetime.md)
adds a normally returned error from the signed v2.3.0 runtime on Windows
x64/MSVC, RX 9060 XT and driver `32.0.31041.1004`. It uses a correctly tagged
`upscale -> version -> DX12 -> DX12` chain with the same live device in both
backend nodes and null host, message and backend callbacks. Local source
explicitly rejects the duplicate backend; no invalid pointer or mismatched
descriptor payload induces the error.

**Verified observations:** discovery and all three isolated lifetime variants
returned 1 (`FFX_API_RETURN_ERROR`) with a null output. No failed output was
destroyed, dereferenced or queried. Each variant exited normally after a
two-second observation window, with no observed fault or timeout:

| Variant | Action after returned error | Result |
|---|---|---|
| Retain | Keep descriptor storage, device reference and runtime | Observation completed |
| Protect | Apply and verify PAGE_NOACCESS on dedicated descriptor pages; retain device/runtime | Observation completed |
| Cleanup | Free descriptor pages, release caller device reference, then drop FfxLibrary | Cleanup and observation completed |

Separate successful controls identified provider `4.1.1`, returned OK from create
and destroy, and again left the handle unchanged after destruction. The record
preserves input fingerprints, reproduction commands and the linked event log.

**Source/binary distinction:** the open FSR3 path allocates internal state and
initializes its first backend interface before rejecting the duplicate. The
signed runtime may reject it earlier; neither its failure depth nor the failed
call's provider identity or supplying module was established. Successful control
identity cannot resolve those unknowns.

**Inference and limits:** this is consistent with no continuing native use of
caller inputs on this tested rejection path. Unlike the allocation crashes, it
exercises the proposed caller cleanup order after an actual returned error.
Survival does not establish rollback, leak freedom, absence of internally handled
access faults, later asynchronous use, COM destruction or provider DLL unload.
No debug layer or exception debugger was active. Other failures, callbacks,
providers and threading remain outside the evidence.

**D005 implication:** no observation contradicts ordinary RAII for this request.
[D005](../DECISIONS.md#d005--own-effect-independent-context-lifecycles-with-terminal-teardown)
now accepts it for the supported runtime path as an explicit, revisable runtime
trust assumption, not a provider-independent guarantee. The evidence limits
above remain unchanged; this is no longer a general M4 blocker. This experiment
alone did not establish a public ownership contract; D005 and D008 later selected
bounded policies that M5 implements. Repeating this wait or adding a staged cleanup
matrix cannot establish the missing universal contract. Further investigation
would need provider-specific source/contract evidence or an independently
justified deeper returned-failure path.

## D013: terminal cleanup after transferred fixture failures

**Verified wrapper-policy result:** the
[2026-10-07 experiment](records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
tests seven control/transfer pairs on Windows x64/MSVC through the real owner
and shared dispatch policy. Known/unknown dispatch errors preserve poison across
handoffs and allow exactly one terminal destroy. A subsequent destroy failure
retains the complete descriptor/callback state, device surrogate, runtime and
fixture DLL with no retry, under both explicit destruction and Drop. Ready and
poisoned Rust unwinds clean up on the destination while holding the common
mutex; a waiting sibling recovers mutex poison, remains usable and completes
final release. Library liveness and pointer identity are checked through cleanup.

These controlled returned codes and injected inspection errors establish ownership
and cleanup behavior, not AMD failed-recording recovery, rollback/quiescence or
native unwind/device-release compatibility. Earlier create-error/OK-null controls
were reused; new creation-after-runtime-transfer and extra native-slot mutations
were not tested. Module absence does not attribute termination callbacks to a
thread. The [threading synthesis](runtime-threading.md#d013-transferred-failure-state-and-coordinated-fixture-cleanup)
records the complete coverage and limits; later D013 acceptance adds no execution evidence.

## D013: native Rust-unwind cleanup after fenced transfers

**Verified bounded native result:** the
[2026-10-08 experiment](records/2026-10-08-exp-d013-native-transfer-cleanup.md)
executes actual automatic owner destruction during Rust unwind outside FFI on
the recorded SDK v2.3.0/provider 4.1.1, RX 9060 XT/driver 32.0.31041.1004 inputs.
All 12 unwind cleanups return native OK in isolated control/transfer repetitions.
Guard/drop order keeps cleanup inside the common domain; the surviving sibling
recovers mutex poison, successfully dispatches another fenced frame and later
Drops on another worker. Across the full matrix, all 42 explicit/automatic native
destroy calls return OK. Test-only post-call tracing observes these return codes;
the public Drop API still exposes no result.

The matrix also covers creation after runtime transfer and independently held
siblings. After all contexts/runtime and audited application frame/device/helper
refs are released, the loader and helper are absent but the provider remains,
including after worker exit. This does not establish actual COM device destruction
or complete provider termination; see [loading observations](runtime-loading.md#d013-acquisition-failures-and-release-observations).

**Limit:** these successful fenced paths add native compatibility evidence for
Rust-unwind cleanup, not native error recovery, failed recording, outstanding
work, quiescence or arbitrary unwind states. The fixture's poisoned native
dispatch/destroy paths remain fixture evidence. D005's terminal no-retry and
exceptional retention policy and D011's GPU obligations remain intact. The
[threading synthesis](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency)
keeps source/coordination limits, distinct from later D013 design acceptance.

## D013: reviewed ownership and lasting cleanup obligations

The [soundness/architecture review imported on 2026-10-09](records/2026-10-09-src-d013-soundness-architecture-review.md)
finds no confirmed double-destruction, descriptor-movement or reference-counting
defect in the proposed ownership structure. Intake source comparison confirms
the existing sealed Rc/Arc seam, pinned allocation and ordered terminal cleanup;
the review does not independently rerun fixtures or native tests, and supplies
no reviewed source revision. The review required the real public movable bundle's
own complete field and cleanup audit; the subsequent
[implementation verification](../release/2026-10-09-d013-public-api-verification.md)
records that audit and actual public consumer/cleanup checks. It does not change
these historical findings or establish native returned-error recovery.

**Design recommendation:** retain D005's at-most-once destruction, ordered
state/device/runtime release on success, full exceptional retention and bounded
returned-create-error RAII trust. The earlier source report's quarantine proposal
remains a recommendation, without new contrary evidence overturning D005.
The strengthened acquisition contract must cover safe destruction and automatic
Drop on every possible destination thread, including unwind, while common-domain
coordination persists for siblings and external native users. Dropping the public
runtime or retaining dependencies after a failure does not discharge those
obligations or establish native quiescence. GPU completion remains independent.
These requirements were adopted in the
[accepted D013 safety contract](../adr/D013.md#conditional-safety-contract) on
2026-10-09. They are not proven AMD error/termination behavior; design acceptance
remains distinct from implementation and support verification.
