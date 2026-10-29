# fsr-sdk

Engine-independent Rust ownership and checked inputs for AMD's FidelityFX SDK
v2.3.0. Version **0.3.0** uses `fsr-sdk-sys 0.1.2` for its curated native ABI.
Create an upscaler through a device-bound setup, prepare portable frame values,
then record against borrowed DX12 resources. Your renderer owns GPU submission
and completion.

## Installation and API documentation

Requires Rust **1.88+**. DX12 is enabled by default:

```toml
[dependencies]
fsr-sdk = "0.3.0"
```

Enable optional thread-transfer owners and provider utilities as needed:

```toml
[dependencies]
fsr-sdk = { version = "0.3.0", features = ["movable", "utils"] }
```

See the [0.3.0 API documentation](https://docs.rs/fsr-sdk/0.3.0/fsr_sdk/)
and [migration guide](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/MIGRATING_0_3.md).
Acquire the trusted AMD SDK runtime separately; Cargo neither downloads nor
bundles AMD binaries.

## Features and platform

| Feature | Behaviour |
|---|---|
| `dx12` (default) | Windows/DX12 loading, setup, native contexts and borrowed resources |
| `movable` (optional) | Separate Send + !Sync owners, under lasting caller-established native compatibility and common coordination; no internal mutex |
| `utils` (optional) | Portable provider-generation resolver; off by default, no extra dependency or implicit backend/movable activation |
| `vulkan` (reserved) | Deliberately fails compilation; usable Vulkan support is not implemented |

The supported native target is **Windows x64/MSVC + DX12**. The API uses
`windows` 0.62 interfaces. Other Windows targets are unverified/unsupported.
Portable checked inputs, frame parameters, errors and query metadata need no
backend: use `--no-default-features`, optionally adding `--features utils`.
Frame Generation, Ray Regeneration and Radiance Caching are not implemented.
Native observations apply only to the recorded SDK/provider/GPU/driver/workload;
accepted metadata is not a universal compatibility or capability certificate.

## Create once, then retain the owner

Load the trusted SDK DX12 runtime explicitly. Deploy
`amd_fidelityfx_upscaler_dx12.dll` beside the application executable; acquiring
AMD binaries is separate from Cargo builds. The setup binds one device and one
`ProviderSelection` for planning and creation. Options contain no provider field.

```rust
use fsr_sdk::{
    runtime::Runtime,
    upscaler::{Dimensions, UpscalerOptions, dx12::{ID3D12Device, Upscaler}},
};

/// Create once before recording; keep the returned owner for subsequent frames.
///
/// # Safety
/// Establish Runtime::load's loader/ABI contract and upscaler_dx12's complete
/// lasting effect/provider/common-coordination contract through later cleanup.
unsafe fn create(
    loader: &std::path::Path,
    device: &ID3D12Device,
) -> Result<Upscaler, Box<dyn std::error::Error>> {
    // SAFETY: The integrating renderer establishes both acquisition contracts.
    let mut runtime = unsafe { Runtime::load(loader)? };
    let mut setup = unsafe { runtime.upscaler_dx12(device) };
    let options = UpscalerOptions::new(Dimensions::new(320, 180)?, Dimensions::new(640, 360)?);
    Ok(setup.create(&options)?)
    // The owner retains runtime/device/pinned storage independently of these borrows.
}
```

For an explicit provider, rebind with `let mut setup = setup.with_provider(selection)`.
Planning and `create` use that request, with fresh membership checks and no
fallback/cache/reservation. Default selection may resolve differently per call.
The optional `utils::provider::{FsrChoice, select_provider}` resolves strict
numeric names to the same core selection. Auto retains SDK default; explicit
choices reject missing or ambiguous matches. Names and IDs prove no hardware
support, generator support or executed algorithm generation. Switching a live
context's provider requires recreation.

## Prepare portable parameters and bind DX12 resources

`FrameParameters::new(jitter, time, camera, HistoryAction::Reset)` requires checked
renderer values. Update jitter, elapsed time, camera and history intent for each
frame; use Continue when continuing existing history. Reusing/cloning values
advances nothing and preserves Reset. Parameters are Clone, without Copy or
Default. One `MotionVectorScale::{Uv, Pixels, Native(scale)}` and one complete
`ColorEncodingRequest` store the effective choices. Native sharpening remains
explicit; disabled differs from enabled zero. No image conversion is performed.

Creation sizes are allocation maxima. With dynamic resolution enabled,
`set_active_render_size` and `set_active_output_size` select smaller active
regions while the allocations stay at their maxima. Output padding may be
written; provider support and access remain part of the recording contract.

Use `UpscaleResources::builder()` with named colour, depth, motion, output and
explicit `ExposureInput::Texture` or `Automatic` bindings. Missing roles return
pure `Error::Input` diagnostics. Construction/cloning retain borrows, not
independent COM references or GPU lifetime. Builder and `with_provider` results
must be used; mutating `set_*` methods instead update their receiver.

`unsafe context.record(list, &resources, &parameters)` always inspects current
metadata. Optional `validate_upscale_inputs` performs the same inspection without
recording. Generator recording follows the same shape with
`ReactiveMaskParameters`, `ReactiveMaskResources`, `validate_reactive_mask_inputs`
and unsafe `record_reactive_mask`. Generator support is provider-dependent:
recorded 3.1.5 writes succeed, whereas the tested default `4.1.2 *` and earlier
4.1.1/2.3.4 acknowledge the tested calls without writing masks. These are bounded
observations, with no automatic probe or fallback.

The renderer supplies compatible signals, formats/views, states, residency,
non-overlapping backing, queue ordering and genuine GPU completion. Keep context,
textures, command list and allocator dependencies through their respective use.
The wrapper neither submits nor waits. Rust borrows cover only the CPU call;
`&mut`, ownership transfer and a successful native return are not fences.

## Tuning, errors and abandonment

Prepare all `TuningRequest::new(parameter, value)` pairs before applying them
with `request_tuning(request)` during an established idle interval. Value and
ProviderDefault are explicit. Success acknowledges the native request without
proving application, support or effective settings. Requests are ordered and
nontransactional; no automatic history reset or recovery occurs.

Match owned `Error` by category: Input, Native, Backend, Invariant,
ContextUnusable and QueryAllocation. Exact codes, roles and original sources
remain available. Error is Send + Sync, without Clone/Copy/equality commitments;
loading keeps its separate LoadError boundary. Input/inspection/ordinary query
errors preserve usability. Native recording/tuning/live diagnostic failures retain
the first terminal cause and block later live operations before inspection/FFI.

Both owners expose pure `usability()` and safe local `mark_recording_abandoned()`.
On discarded recordings or failed Close, mark **every affected context**,
invalidate executable recordings and finish earlier submitted work before
terminal cleanup. Marking is idempotent, retains an earlier native cause and
performs no native/COM work, wait or release. A Usable snapshot means only that
no terminal cause is recorded; copied snapshots can become stale. Destroy is a
single terminal attempt; Drop discards its error. Neither waits, and failed
destruction retains critical dependencies.

Unsafe callback installation and global diagnostics have additional lasting
contracts. External callback targets/allocator state may remain globally reachable
after errors or destruction; neither successful cleanup nor nulling a callback
establishes quiescence or authorises reclaiming targets.

## Complete integration and migration

The [complete consumer workflows](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/upscaler_workflows.rs)
cover simple creation/first frame, explicit-provider planning/create and portable
preparation followed by recording/abandonment for **both** ownership families,
plus optional utility composition. They retain the owner on fallible renderer
cleanup paths and start no GPU work from main. The
[movable example](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/movable_upscale.rs)
separately demonstrates coordinated thread handoff and Drop.

Read the [0.3 migration guide](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/MIGRATING_0_3.md)
and [DX12 recovery guide](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/DX12_RECOVERY.md)
for source breaks, failure classification and renderer obligations. Build
local Rustdoc on Windows with
`cargo doc -p fsr-sdk --features movable,utils --no-deps --locked`.
The [coverage summary](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/upscaling-coverage.md)
records bounded native observations and provider limitations.

## Licensing and provenance

Original code is licensed under [MPL-2.0](LICENSE).
The `fsr-sdk-sys` dependency retains AMD's MIT notice for its header-derived
declarations. The constructor fixture in `tests/fixtures/upscaler.rs` follows SDK v2.3.0
`Kits/FidelityFX/api/include/ffx_api.h` at commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`; its AMD notice is retained in
[LICENSE-AMD](LICENSE-AMD). The optional Windows device binding uses the
MIT OR Apache-2.0 licensed `windows` crate. AMD SDK components and runtime binaries
retain their applicable terms and are not included in this package.
This project is independent of and not endorsed by AMD.
