# DX12 setup, failure and recovery

Use optional early checks to catch integration mistakes before recording, and
keep renderer ownership through native failures and abandoned frames. These
workflows retain [D011](adr/D011.md), [D029](adr/D029.md) and
[D032](adr/D032.md)'s native and GPU boundaries. Explicit callback installation
and diagnostic configuration additionally follow [D040](adr/D040.md).

For the complete source migration, see the [0.3 guide](MIGRATING_0_3.md).
The [integrated consumer example](../crates/fsr-sdk/examples/upscaler_workflows.rs)
provides matching local/movable first-frame, explicit-provider and abandonment
flows, plus opt-in utility composition. It starts no GPU work from main.

## Acquire one setup for planning and creation

`unsafe Runtime::load` covers loader/ABI trust and common coordination;
`unsafe runtime.upscaler_dx12(&device)` separately acquires lasting effect trust.
Use the returned setup's single `ProviderSelection` for queries, estimates and
`create(&options)`. Bind with `with_provider`; options contain no provider.
Created owners retain their dependencies independently. Ending a setup borrow
establishes neither native isolation nor GPU completion. See the
[accepted contract](design/upscaler-setup.md) for safely reachable providers,
ordinary error release, exceptional retention and movable duties.

## Prepare and bind recording inputs

Portable `FrameParameters` stores checked jitter/time/camera and named
`HistoryAction`, plus optional native controls. Update required values for each
frame and explicitly change Reset to Continue when continuing history. A clone
repeats its intent. One `MotionVectorScale` and one `ColorEncodingRequest` store
the effective choices; restoring a temporary request is the caller's explicit
update, with no dormant fallback. Sharpening remains a direct native request.

DX12 `UpscaleResources::builder()` requires named colour/depth/motion/output and
an explicit `ExposureInput::Texture` or `Automatic` choice. Generator resources
require opaque-only/pre-upscale colour and mask output; `ReactiveMaskParameters`
holds its explicit region and controls. Missing roles return pure
`Error::Input(InputError::MissingBinding { .. })` without inspecting objects. Construction and cloning
neither retain COM objects independently nor extend GPU lifetime.

Call unsafe `record(list, &resources, &parameters)` or `record_reactive_mask`.
Matching `validate_upscale_inputs` / `validate_reactive_mask_inputs` are optional
metadata preflight only; recording always inspects again. All earlier provider,
state, access, ordering, abandonment and completion duties remain. See the
[U08 verification](release/2026-10-27-frame-composition-verification.md).

## Check inputs during setup

`TuningRequest::new(parameter, value)` checks each complete pair before any
native mutation, rejecting non-finite or out-of-domain values even when the
`TuningValue` is directly constructed or reused for a different parameter.
Private fields preserve the checked pair; `parameter()` and `value()` inspect it.
Both families accept only `request_tuning(request)`. Accepted bits are forwarded unchanged. The parameter domains are
explicit wrapper policy based on the shared SDK v2.3.0 header, independently of
provider selection. `ProviderDefault` requests the native default. Success means
native acknowledgement; open FSR2 can ignore the request, and no effective-value
query or generic tuning-capability query establishes application.

Both upscaler families expose `validate_upscale_inputs(list, &resources, &parameters)`. It uses the same
metadata inspection as dispatch, but calls no FidelityFX entry point and records
no commands. Supply live DX12 objects; the command list need not be open and
resources need not already be transitioned for this inspection. It checks
configuration, required input presence, shape/format/extent/flags and COM/device
identity. Existing poison rejects first; a usable context remains usable after
validation errors. Temporary COM references are released before return.

Validation cannot inspect image content, actual GPU states, heap overlap,
residency, provider capability or queue completion. It does not cache or reserve
anything. Dispatch still validates independently, so omitting setup validation
does not bypass checks and successful setup validation cannot authorize later
stale inputs. Keep preflight out of the per-frame path unless early diagnosis is
needed there; calling both methods adds an extra inspection.

## Inspect structured causes

Match `error::Error` by category and inspect its payload. Input reasons retain
resource roles and parameter identities; backend errors retain exact HRESULTs
and the original Windows source. Native errors retain `operation()` / `code()`.
After a failed native upscale, generation, tuning or live legacy diagnostic call,
later live operations return `Error::ContextUnusable` with the same first native
operation/code before inspection or FFI. Ordinary query/input/inspection errors
leave a usable context usable. This is diagnostic state, not GPU readiness.
See the [U10 contract](design/errors-and-provider-metadata.md).

Both families expose pure `usability() -> ContextUsability`. `Usable` means only
that no terminal cause has been recorded; pending recordings and submitted GPU
work may still exist. On abandoned recordings, call safe
`mark_recording_abandoned()` on **every affected context**. It preserves an earlier
native cause, otherwise records `TerminalCause::RecordingAbandoned`. Repeated
marking is idempotent. Later live operations and context-dependent preflight
reject before inspection or FFI; status and terminal destruction remain available.
Neither status nor marking calls native code, inspects COM, changes reference
counts, releases objects, invalidates lists, waits or establishes callback
quiescence. A copied snapshot does not follow later state changes.

## Classify the failure before cleanup

| Event | State and renderer action |
|---|---|
| Tuning request construction fails | Correct the value; pure construction makes no native call and changes no context state. Preparing the complete list first prevents any native changes on invalid input; independently applied earlier requests are not rolled back. |
| Dispatch/preflight rejects metadata | This call added no FSR commands and did not poison a usable context. Correct the input; earlier successful recordings still need their original submission/lifetime handling. |
| Native dispatch fails, including an unsupported-operation code | The affected list may contain partial work: never submit it. Invalidate affected recordings, complete earlier submitted work, then destroy/recreate affected contexts. Preserve the exact operation/code for diagnosis. |
| Native context Configure fails, including live legacy diagnostics | The context becomes terminal. Configure has no failed list of its own; the required idle interval was already established. Destroy/recreate normally; earlier acknowledged settings are not rolled back. |
| Runtime-global diagnostic Configure fails | Preserve the exact code and account for possible shared-state effects. The wrapper does not poison individual contexts. The unsafe call requires native-safe continuation and coordination of every affected user; the lack of poison is not a recovery guarantee. |
| Callback-bearing native creation fails | No usable owner is returned and the failed output is never operated on. The wrapper retains its pinned state/device/runtime bundle. Keep external targets valid for all native users; an error does not prove registration rollback or authorize freeing them. |
| Native recording succeeds but the renderer discards the list, or `Close` later fails | Call `mark_recording_abandoned()` on every FSR context with an acknowledged recording on that abandoned list. Invalidate the recordings, finish earlier submitted work and destroy/recreate those contexts. The wrapper cannot observe this abandonment automatically. |
| Ordinary native query fails | Existing query policy does not poison or clear poison. This is not evidence that a dispatch/Configure error is equally harmless. |
| Submission/device completion fails | Do not report GPU idle from a CPU return, failed wait or removed-device sentinel. Follow renderer/device-loss recovery; no automatic context-reuse guarantee applies. |

For replacement, keep the old owner and resources until recording invalidation
and earlier GPU completion succeed. Then consume the owner with `destroy` once.
A destroy error is terminal and retains critical dependencies; do not retry that
native handle. A fresh context's first ordinary frame uses `HistoryAction::Reset`.
History reset does not repair an old context or roll back abandoned work.

The compilable [recovery example](../crates/fsr-sdk/examples/dx12_recovery.rs)
marks the old context before either fallible renderer callback and keeps it in
`Option<Upscaler>` until those callbacks complete.
A callback error leaves the owner present for terminal cleanup, never resumed
recording. A destroy/create error after taking the owner leaves `None`. Its
callbacks are explicit unsafe renderer obligations, not a safe fence token.
Movable callers use the same steps in their common native coordination domain;
moving an owner establishes neither GPU completion nor thread-safe native use.

## Prepare requests before an established idle interval

For context-local `request_tuning`, a new context has no earlier GPU work to
drain before its first dispatch. Global diagnostics instead require the idle
conditions for every affected user described below. For later tuning changes,
construct and validate the complete desired list first, then apply its requests
in caller order during one renderer-scheduled idle interval. All submitted work on that context must be complete and no
executable unsubmitted recording may execute afterward. The wrapper performs
no wait.

The recovery example's `request_desired_at_idle` uses fallible collection before
its first native call, so a later invalid input prevents every native change.
Native requests remain independent: a later native failure poisons the context
without rolling back earlier acknowledgements. Duplicate parameters and their
order are preserved. Checked requests do not establish provider support or GPU
idleness; operation entry still checks existing poison before mapping/FFI. No automatic history reset,
effective-value cache, transaction or in-flight Configure claim is introduced.

## Explicit callbacks and global diagnostics

setup `unsafe create_with_callbacks` installs native requests; its callback-table values
are not registration guards. Empty/default assembly owns no target state and
invokes no callback. Omitted DX12 groups leave earlier native registrations in
place; creation-message null behaviour is provider-specific. Later ordinary
contexts may therefore still use callbacks installed by another context.

Keep callback code, userdata, bookkeeping and independent device references valid
for every possible native use, including replacement, failed creation and
successful/failed destruction. Resource/heap replacement must free outstanding
allocations compatibly. Keep custom CB upload backing, ranges and residency
through GPU completion; a live COM reference alone does not prevent suballocation
reuse. A successful destroy releases wrapper-owned storage without establishing
quiescence of copied global function pointers. No automatic unregister, target
reclamation or failed-create recovery service is provided.

Setup `configure_debug` and `configure_global_debug_legacy` act on
native global state. Establish CPU idleness, completed GPU work and absence of
executable pending recordings for **all affected users**, with common-domain
coordination and native-safe continuation even after errors. The live-context
legacy method additionally follows context poison. All three routes may affect
shared global state; a live receiver does not imply context-local effects. Neither route silently falls
back. On the [recorded deployment](release/2026-10-25-callbacks-diagnostics-verification.md),
modern global calls and legacy live calls succeed; legacy null-context calls
return code 2 unchanged. These outcomes are not universal capability detection.
Nulling a callback and elapsed time do not authorize reclaiming targets.

The complete lasting contract is in the
[interop module](../crates/fsr-sdk/src/upscaler/dx12/interop.rs) and
[callback synthesis](research/callbacks-and-allocation.md). Movable users must
satisfy it on every permitted destination thread, including Drop and unwinding.

## Measure provider setup before caching

Explicit-provider setup queries still enumerate freshly under D028. The
`experiment-provider-setup` profile prepares a bounded native measurement:

```sh
python tools/verification/run.py run experiment-provider-setup --native --require-passed --sdk-root path/to/Kits/FidelityFX --adapter-index 0
```

It requires the existing Windows/MSVC environment and pinned local signed SDK.
It builds optimized code, runs four warm-up sequences per arm, then 32 paired
blocks with alternating default/selected order. Each sample performs six setup
queries: ratio, render size, jitter phase/offset, resource requirements and memory
estimate. Explicit selection includes the existing count/fill membership checks
and name copying. Capture all 64 `PROVIDER_SETUP` timing rows and provider/device,
runtime, driver and source identities from the normal native run record.

The benchmark measures the entire setup path, not isolated enumeration cost.
An observed default context supplies the explicit ID, but does not reserve the
default route or prove it stays identical during queries. It submits no GPU work
and selects no speed gate, cache or session. Run independent processes before
drawing timing conclusions. The [2026-10-19 native measurement](research/records/2026-10-19-exp-provider-setup-cost.md)
passes three processes and retains 192 samples. Median six-query setup costs
are 229.55–230.74 ms for default and 696.98–707.10 ms for selected routing on
the recorded stack. This does not isolate enumeration or select a cache.

## Generator boundary

Both ownership families expose reactive-mask generation under the
[D030 contract](adr/D030.md).
`validate_reactive_mask_inputs` inspects metadata without FidelityFX calls;
`unsafe record_reactive_mask` repeats that inspection before recording.
Requirement bits do not certify generator capability. The
[renderer example](../crates/fsr-sdk/examples/dx12_reactive_mask.rs) records
generation, explicit UAV ordering/transition and supplied-mask upscaling on one
list. Submit successful work once and fence before Configure or teardown; the
idle interval must cover both GPU recording routes. Any generator native error,
including unsupported, poisons generation, upscaling, live queries and Configure.
Invalidate failed recordings and complete earlier submitted work before terminal
cleanup. A later preflight error does not roll back successful generation.
Do not probe unknown support on application history. D030 explains the limits
of a deliberate disposable-context probe; no automatic probe or fallback exists.
[Local implementation checks](release/2026-10-19-reactive-mask-generation-verification.md) do not
establish provider support, state restoration or GPU completion. Subsequent
[Windows GPU probes](research/records/2026-10-19-exp-reactive-mask-providers.md)
pass both public families/all flags on 3.1.5; 4.1.1 and 2.3.4 acknowledge but
leave masks unchanged. Those write criteria remain failed, with no automatic
fallback or provider-independent support guarantee.
Reusing a context after native errors/abandonment or configuring while work is
in flight still needs pinned provider evidence and an explicitly revised contract.
