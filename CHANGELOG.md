# Changelog

## Unreleased — fsr-sdk 0.3.0 / fsr-sdk-sys 0.1.2

Complete public upscaling access to the selected AMD SDK v2.3.0 interface on
Windows x64/MSVC + DX12, with a coherent setup/frame/resource API and optional
provider utilities. See the [migration guide](docs/MIGRATING_0_3.md) for source
changes and [release preparation](docs/release/0.3.0-preparation.md) for the
remaining package and publication checks.

### Breaking changes

- Split native acquisition into unsafe `Runtime::load` / `MovableRuntime::load`
  for loader trust and unsafe `upscaler_dx12(&device)` for lasting effect/provider
  trust. Replace flat runtime effect methods and public context constructors
  with concrete `UpscalerSetup` / `MovableUpscalerSetup`. Safe planning, memory
  estimates and `create(&options)` use one setup-bound `ProviderSelection`.
  Created contexts retain dependencies independently of setup/runtime borrows.
- Replace `UpscaleDispatch`, positional resource constructors and `dispatch(frame)`
  with portable `FrameParameters`, named `UpscaleResources::builder()` bindings
  and unsafe `record(list, &resources, &parameters)`. Reactive-mask recording uses
  the same separation through `ReactiveMaskParameters` and `ReactiveMaskResources`.
  Exposure is an explicit texture/automatic choice. Recording always reinspects
  current metadata, including after optional preflight.
- Require checked jitter/time/camera and `HistoryAction::{Reset, Continue}` when
  constructing frame parameters. Reusing/cloning parameters advances nothing.
  Motion is one `MotionVectorScale::{Uv, Pixels, Native(scale)}` request; colour
  hints and mismatch permission are one `ColorEncodingRequest`. Native sharpening
  is portable and remains disabled by default; enabled zero is distinct.
- Replace the flat, copyable main error with owned Input, Native, Backend,
  Invariant, ContextUnusable and QueryAllocation categories. Preserve exact SDK
  codes, HRESULTs, resource roles and source chains. `Error` is Send + Sync and
  no longer promises Clone/Copy/equality. Loading retains its separate LoadError.
- Extend unsafe acquisition duties to every reachable default/explicit provider,
  synchronous temporary-descriptor/name handling, query numerical domains,
  provider compatibility and error continuation. Those caller-established native
  assumptions persist through retained owners and exceptional cleanup; successful
  loading does not verify them. Local/movable threading and renderer GPU duties
  remain explicit.
- Query resource requirements after every context creation. Unsupported required
  inputs, unknown requirement bits and query errors reject construction with
  terminal cleanup. Live requirements remain queryable; no per-frame query or
  automatic provider fallback is added.

### Added and changed

- Expose provider enumeration, explicit override and live provider identity,
  pre-create/live resource requirements and memory accounting, plus native
  quality-mode ratio/render-size and jitter phase/offset queries. Portable owned
  results retain opaque IDs, byte counts and unknown resource-mask bits.
  `with_provider` rebinds a request; operations freshly validate membership and
  selected live identity without caching, reservation or fallback.
- Add portable checked `TuningRequest::new(parameter, value)` and one safe
  `request_tuning(request)` operation on both owner families. Five direct tuning
  parameters support explicit values or provider-default reset. Prepare complete
  request lists before mutation; requests are ordered and nontransactional.
  Success acknowledges a request, not support or observed application. Subsequent
  Configure requires an established idle interval, completed submitted work and
  no pending executable recordings. No implicit history reset is added.
- Add pure `usability()` and safe local `mark_recording_abandoned()` to both
  owners. Native mutating/live-diagnostic failures retain the first terminal
  cause and block later live operations/preflight before COM/FFI. Abandonment is
  idempotent and performs no native work, wait or release. Renderer invalidation,
  GPU completion and terminal cleanup remain required for every affected context.
- Add depth convention, motion resolution/jitter inclusion, checked native motion
  scales, explicit/automatic exposure and positive finite pre-exposure. Add opt-in
  linear HDR, non-linear creation and per-frame sRGB/PQ encoding hints with checked
  pairing or an explicit mismatch override. Preserve checked native bits and
  defaults; provide no pixel conversion, swizzle, PQ normalization or exposure
  smoothing.
- Add dynamic-resolution creation and active render/output regions within fixed
  maximum allocations. Reject invalid extents before COM/native inspection;
  adjust UV motion to the active region and retain unit pixel motion. Size changes
  do not request history reset or resize textures. Actual provider support and
  safe access remain caller-established; the full output allocation may be written.
- Accept supplied reactive and transparency/composition masks with shared
  role/device/identity checks and exact null/present mapping. Reported consumption
  bits are not a forwarding allow-list. Add unsafe `record_reactive_mask` and
  matching optional preflight, four native generation flags and explicit active
  regions. Renderer-owned UAV ordering, transitions and completion remain required.
- Extend resource support with RG32F motion; typed RGBA32F, R11G11B10F, RGB10A2
  UNORM and RGBA8 UNORM; BGRA8 and RGBA8/BGRA8 sRGB views; fixed typeless
  colour/motion/depth/exposure/mask views; SNORM and shared-exponent formats;
  load-only RGB32F input; R16F device depth; and D16/D24/D32/D32S8 depth routes.
  Check exact role/view/device capabilities and preserve HRESULT provenance.
  RGB32F UAV/output remains rejected; no format conversion is performed.
- Add explicit unsafe host/resource/heap and constant-buffer callback creation,
  UTF-16 messages, global modern/legacy diagnostics and independent debug checking,
  visualization and per-frame debug view. Preserve by-value CB ABI and externally
  owned targets. Failed callback-bearing creation retains pinned dependencies;
  successful destruction or callback replacement does not establish quiescence.
- Ship the off-by-default `utils::provider::{FsrChoice, SelectionError,
  select_provider}` resolver. Strict numeric names map to core ProviderSelection;
  Auto retains SDK default, explicit choices reject missing/ambiguous matches.
  Names and IDs certify neither hardware support nor executed generation. The
  feature adds no dependencies and enables neither DX12 nor movable ownership.
- Add complete local/movable consumer workflows, provider selection, active-output,
  reactive-mask composition, tuning and renderer-owned recovery examples. Warn
  when consuming builders, provider replacements or usability snapshots are
  accidentally discarded. Preserve idiomatic checked constructor names and keep
  extensible enums open where required.
- Extend the curated `fsr-sdk-sys 0.1.2` ABI with the selected provider, resource,
  memory, quality/jitter, override, tuning, reactive-generation and diagnostic
  descriptors/flags and callback payloads. Declare all 40 pinned surface formats
  and selected depth/stencil usage bits. Existing descriptor layouts, loader
  contracts and the SDK v2.3.0 pin are retained; no runtime binary is bundled.

### Compatibility and verification limits

- Require Rust 1.88+. DX12 remains the wrapper default; raw DX12 and optional
  `movable` / `utils` remain explicit. The reserved `vulkan` feature is retained
  and deliberately fails compilation, including with `--all-features`. Vulkan,
  Frame Generation, Ray Regeneration and Radiance Caching are not implemented.
- Generator support is provider-dependent: bounded 3.1.5 calls write masks;
  tested default `4.1.2 *` and earlier 4.1.1/2.3.4 calls acknowledge without writes.
  No automatic capability probe or fallback is added. Non-linear/pre-exposure
  semantics, numerical tuning application, callback quiescence and native
  failure recovery retain their documented limits.
- Native support remains Windows x64/MSVC + DX12. Experimental Deck/Machine
  results use Windows DLLs through Proton and establish no native Linux/Vulkan
  support. Rust borrows, successful returns and ownership transfer are not GPU
  fences; the wrapper neither submits nor waits.
- The [bounded performance review](docs/research/performance.md) retains the
  maintainer-accepted temporal exclusion and deployment limits. No Rust allocator
  events were observed in 1,920 warmed calls on the instrumented calling thread;
  this promises neither universal latency nor native/driver allocation freedom.

## fsr-sdk 0.2.0 / fsr-sdk-sys 0.1.1

- **Breaking:** construct frames with `UpscaleResources::new` and
  `UpscaleDispatch::new` instead of `0.1.0` public struct literals. Read inputs
  through accessors; clone the resource bundle to reuse its borrows.
- Add explicit native sharpening, disabled by default. Select
  `Sharpening::Native(NativeSharpness::new(value)?)` with a finite `[0,1]`
  parameter. Enabled zero remains distinct from disabled. This forwards the
  provider control and supplies no image-quality guarantee.
- Add optional `movable` ownership: `MovableRuntime` and `MovableUpscaler` are
  `Send + !Sync`. Unsafe acquisition requires lasting native compatibility and
  coordination across all calls, handoffs and cleanup, including unwinding.
  The local family remains `!Send + !Sync`; dispatch remains unsafe.
- Expose checked common inputs on every host without backend features through
  `upscaler::common`; retain existing imports as reexports. Native APIs require
  Windows and `dx12`.
- In `fsr-sdk-sys 0.1.1`, isolate the Windows loader and make its `libloading`
  production dependency optional and Windows/DX12-only. The selected ABI,
  pinned SDK v2.3.0 and third-party dependency versions are unchanged.
- Include movable APIs in Windows-target docs.rs builds and compile both
  integration examples against packaged consumers.

See [D014's migration example](docs/adr/D014.md#migration-example),
[movable coordination example](crates/fsr-sdk/examples/movable_upscale.rs) and
[release verification](docs/ROADMAP_HISTORY.md#release-qualification).

The supported native target remains `x86_64-pc-windows-msvc`. Evidence is bounded
to the recorded provider/GPU/driver and workloads. Vulkan and other SDK effects
remain unimplemented; GBV timeout and the original failed sharpening-effect
experiment remain unresolved historical results.

## 0.1.0 — 2026-09-28

Initial two-crate release with explicit trusted runtime loading, checked DX12
upscaler construction, terminal ownership and a narrow unsafe dispatch route.
See the [publication record](docs/ROADMAP_HISTORY.md#010--published-2026-09-28).
