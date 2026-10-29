# Migrating to fsr-sdk 0.3

Version 0.3.0 pairs with `fsr-sdk-sys 0.1.2`, Rust 1.88+ and AMD SDK v2.3.0.
Use `fsr-sdk = "0.3.0"` and the
[versioned API documentation](https://docs.rs/fsr-sdk/0.3.0/fsr_sdk/) alongside the
[package README](../crates/fsr-sdk/README.md) and integration examples below.
Source, trait and unsafe acquisition-contract changes from 0.2.0 are intentional;
no compatibility facade retains the old assembly/recording routes.

## From published 0.2.0

| Earlier API | 0.3 replacement |
|---|---|
| Unsafe runtime loading as the whole acquisition boundary | Unsafe load establishes loader trust; separately acquire lasting effect/provider trust with unsafe `runtime.upscaler_dx12(&device)` |
| `Upscaler::new(&runtime, &device, options)` | Acquire a concrete setup, then safe `setup.create(&options)`; the context owns its dependencies independently |
| Positional `UpscaleResources::new(...)` | Named `UpscaleResources::builder()` roles, including explicit texture/automatic exposure |
| Combined `UpscaleDispatch` and `context.dispatch(frame)` | Portable `FrameParameters` plus borrowed DX12 resources and unsafe `record(list, &resources, &parameters)` |
| Boolean reset argument | Explicit `HistoryAction::Reset` / `Continue` stored in parameters and updated by the renderer |
| Flat/copyable main errors | Match owned `Error` categories and inspect payloads/source chains; propagate or borrow errors instead of assuming Clone/Copy/equality |

Apply the same changes to MovableRuntime/MovableUpscaler. Local owners remain
!Send + !Sync; movable owners remain Send + !Sync and require lasting native
compatibility and common coordination on every destination thread, including
implicit Drop/unwinding. No conversion or internal synchronization is added.

## From earlier 0.3 development snapshots

These APIs were introduced after 0.2.0 and changed during development:

| Earlier development API | Canonical API |
|---|---|
| Flat Runtime effect queries and provider stored in UpscalerOptions | One setup-bound `ProviderSelection` through `with_provider`, planning and create |
| Optional-ID resolver output | `utils::provider::select_provider` returns core `ProviderSelection`; Default is distinct from Explicit(id), including explicit numeric zero |
| Combined generator dispatch/positional generator resources | `ReactiveMaskParameters`, named `ReactiveMaskResources` and unsafe `record_reactive_mask` |
| Motion units plus a separate native override | One `MotionVectorScale::{Uv, Pixels, Native(scale)}` value; switching removes the old request |
| Separate colour hint/mismatch setters | One complete `ColorEncodingRequest::checked(hint)` or `allow_context_mismatch(hint)` |
| Context tuning with raw parameter/value arguments | Checked portable `TuningRequest::new(parameter, value)` followed by `request_tuning(request)` |
| Flat poison/backend errors | Structured native operation/code, backend sources and `ContextUnusable(TerminalCause)` |
| Exhaustive development DebugLevel matches | Retain the four existing filters and add a wildcard for future SDK filter variants |
| Abandonment only represented in renderer bookkeeping | Also call `mark_recording_abandoned()` on each affected owner; inspect the pure `usability()` snapshot |

Use `upscaler` for portable effect values and `upscaler::dx12` for concrete DX12
owners/setups/resources. Existing type-identity reexports remain available; they
are aliases of the same implementation, not parallel APIs. No wholesale Config,
Extent or generic backend/ownership-trait migration is required.

## Three complete workflows, both families

The [integrated example](../crates/fsr-sdk/examples/upscaler_workflows.rs) is copied
into an external consumer and compiled on stable and MSRV. Both local and movable
modules expose the same three entry points:

1. `first_frame` creates once, records and invokes the renderer's submission/
   completion callback. The caller's Option receives the owner **before** recording;
   errors or panics leave it present for required cleanup. Success retains it for
   subsequent frames. Setup/runtime borrows do not escape into that owner.
2. `selected_provider` binds one explicit ID, performs planning and creates through
   the same setup. Each operation still checks membership freshly; the snapshot
   reserves nothing. `generation_choice` additionally demonstrates selected opt-in
   `utils` composition for both families, without a separate native facade.
3. `record_then_abandon` deliberately discards the entire list, marks the owner,
   invalidates executable recordings, completes earlier work, then destroys once.
   Either fallible renderer callback leaves the owner terminal and retained. A
   native recording error keeps its first cause and is returned after cleanup.

These are renderer integration functions, not a standalone renderer or safe
fence abstraction. Their main starts no native work. Uphold their complete unsafe
contracts; a callback error does not reveal whether submission occurred. Keep all
referenced renderer dependencies on every failure/unwind path. Mark other contexts
on a shared abandoned list independently. Device loss or uncertain completion
requires renderer recovery; CPU returns/status cannot establish GPU idle.

Portable `prepare_frame` and `update_frame` take checked current jitter, elapsed
time, camera and explicit history. Use Reset on the first ordinary frame and
Continue when continuing history. Reuse/Clone preserve values; neither advances
history or time. Build resource roles by name using `bind`, or the public builders
directly. Builders retain borrows without COM AddRef or GPU-lifetime extension.
Optional metadata preflight is not a reservation: recording always inspects again.

Creation sizes remain allocation maxima. Dynamic resolution permits smaller
active render and output regions through the frame parameters; it does not
resize textures. Output/display-motion allocations stay at their maxima and
the provider may write output padding. See
[`dx12_active_output.rs`](../crates/fsr-sdk/examples/dx12_active_output.rs) for
the renderer's matching image-content and lifetime obligations.

## Handle errors and terminal cleanup

Match Input, Native, Backend, Invariant, ContextUnusable and QueryAllocation with
a wildcard for future categories. NativeError exposes operation()/code(); backend
and allocation errors retain their original owned sources. Input reasons retain
roles and checked parameter identities. LoadError remains a separate boundary.

Input/inspection/ordinary query errors leave a usable owner usable. Native upscale,
generation, tuning and live legacy diagnostic errors retain the first native
cause. Later live operations and context-dependent preflight reject before COM/FFI.
Marking abandonment instead records RecordingAbandoned only if no earlier cause
exists. Repeated marking is idempotent. Global setup diagnostics do not poison
individual contexts and can affect shared native state.

Neither status nor marking inspects native objects, changes reference counts,
waits or releases resources. Before consuming destroy/Drop, invalidate failed or
abandoned executable recordings and genuinely complete earlier submitted work.
Destroy makes one terminal attempt; failure retains critical dependencies without
retry. Callback targets can outlive context cleanup; no quiescence/unregister or
automatic recovery guarantee follows. See [DX12 recovery](DX12_RECOVERY.md).

## Constructors, traits and compiler diagnostics

Checked `new(...) -> Result<_, _>` remains the naming convention; no blanket
try_new rename is needed. Mutating set_* methods update their receiver; consuming
with_provider and resource builders return the changed value and warn if it is
discarded. Pure usability() also warns on accidental discard. `let _ = ...` remains
available for deliberate discard; it does not establish safety or completion.

Small checked values/snapshots retain appropriate Copy/equality traits. Extensible
frame/options/resource aggregates remain Debug + Clone without a blanket Copy or
Default promise. Required frame inputs/history are explicit. Native owners are
neither Clone nor Copy; Error owns sources and is not Clone/Copy/Eq. Open error,
choice and status enums remain non-exhaustive. These are deliberate compatibility
boundaries, not missing derives to add mechanically.

## Features and compatibility limits

`utils` ships with 0.3.0 and remains off by default, with no extra dependencies or
implicit `dx12`/`movable` activation. Portable values need no backend features;
native APIs require Windows and `dx12`. The reserved `vulkan` feature deliberately
fails compilation, including all-features builds; it is not an available backend.

Generator support remains provider-dependent. Recorded 3.1.5 calls write masks;
tested `4.1.2 *`, 4.1.1 and 2.3.4 calls acknowledge without writing. There is no
automatic probe or fallback. No image conversion, image-quality guarantee or
submission/synchronization service is introduced. The
[integration coverage](research/upscaling-coverage.md) records the bounded
provider observations; the [changelog](../CHANGELOG.md) lists the release changes.
