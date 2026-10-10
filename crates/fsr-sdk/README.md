# fsr-sdk

Engine-independent Rust ownership and checked inputs for AMD's FidelityFX SDK
v2.3.0. The supported native target is `x86_64-pc-windows-msvc` with the default
`dx12` feature. Windows/DX12 APIs are exposed on other Windows targets, but those
targets are currently unverified and unsupported; compilation may succeed.
Native verification is limited to the recorded SDK, runtime/provider, GPU,
driver and workload configurations; target support does not imply compatibility
with every hardware/provider configuration. The
[project scope](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/SCOPE.md#platform-and-compatibility-boundaries)
owns this policy. Other SDK effects and Vulkan are not implemented.

## Extensible frames and native sharpening in 0.2.0

Version `0.2.0` replaces `0.1.0` public `UpscaleDispatch` literals with
`UpscaleResources::new(color, depth, motion_vectors, exposure, output)` and
`UpscaleDispatch::new(list, resources, jitter, frame_time, camera, reset)`.
Accessors expose the inputs; clone the resource bundle to reuse its borrows.
Construction performs no COM inspection or independent COM retention.

Frames default to `Sharpening::Disabled`. The only setter selects Disabled or
`Sharpening::Native(NativeSharpness::new(v)?)`, forwarding finite `[0,1]` values
unchanged, including negative zero. Enabled zero is distinct from Disabled.
There is no wrapper filter, clamping, quality profile or image-quality promise.
The frame and sharpening types require Windows plus `dx12`; common inputs keep
their portable availability. Native error, poison and renderer GPU obligations
continue to apply.

[D014](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D014.md)
contains migration and the original failed sharpening-effect evidence.
[M8 verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/release/2026-10-09-m8-implementation-verification.md)
covers compilation and controlled FFI. The
[M9 verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/release/2026-10-10-m9-verification.md)
records public-route GPU execution, same-context mode changes, candidate artifact
checks and their configuration limits. The
[0.2.0 release verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/release/2026-10-10-0.2.0-release-verification.md)
checks the release artifacts including movable ownership.

## Optional movable ownership

The `movable` feature exposes `runtime::MovableRuntime` and
`upscaler::MovableUpscaler` with Windows + `dx12`. Both are `Send + !Sync`;
the existing local types remain `!Send + !Sync`. Unsafe acquisition requires
lasting arbitrary-thread native compatibility and serialization across all
potentially shared state, including implicit Drop/unwinding. Dispatch retains
its independent unsafe GPU contract. There are no family conversions or internal locks.

The [consumer example](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/movable_upscale.rs)
coordinates independently transferred contexts and automatic cleanup.
Generate its current Rustdoc with `cargo doc -p fsr-sdk --features movable --no-deps --locked`
on Windows. [D013](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D013.md)
and the [verification record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/release/2026-10-09-d013-public-api-verification.md)
separate the contract from bounded native observations. Enable it with:

```toml
fsr-sdk = { version = "0.2.0", features = ["movable"] }
```

## Consumer workflow

The crate exposes `upscaler::common` for checked sizes, options
and frame scalars, and `upscaler::dx12` for GPU construction and dispatch.
Existing imports from `upscaler` remain reexports of the same types. Common inputs
are available on every host without backend features. `upscaler::dx12`, its
reexports and `runtime` require Windows plus `dx12`; Vulkan remains unsupported.

Install [`fsr-sdk 0.2.0`](https://crates.io/crates/fsr-sdk):

```toml
[dependencies]
fsr-sdk = "0.2.0"
```

1. Load a separately acquired, trusted runtime with `unsafe Runtime::load_unchecked`.
2. Create options with `UpscalerOptions::new` from checked dimensions, then create
   `Upscaler` with those options and a borrowed DX12 device.
3. Prepare the renderer's resources, states and open DIRECT command list.
4. Build `UpscaleDispatch` and call `unsafe Upscaler::dispatch`.
5. Submit/fence through the renderer and retain GPU objects and the context until
   completion. A successful Rust return only records commands.

The crate owns the context, native descriptors, runtime retention and an
independent device reference. It validates inspectable inputs and preserves native
error codes. The renderer owns states, backing-storage aliasing, residency,
submission, ordering, allocator/list reuse and GPU-visible lifetime. Native
dispatch failures poison further dispatch; teardown still requires correct
completion and recording cleanup.

The first profile uses fixed extents, linear non-HDR RGBA16F color, ordinary R32F
depth, jitter-free current-to-previous top-down UV RG16F motion, explicit 1×1 R32F
exposure and a UAV-capable RGBA16F output. HDR, auto exposure, dynamic resolution,
masks are not exposed. Native sharpening uses the
control described above. The API uses `windows` 0.62 DX12 interfaces.

## API contract and example

Full field, error and safety contracts are published in the
[`0.2.0` API documentation](https://docs.rs/fsr-sdk/0.2.0/fsr_sdk/).
In a repository checkout,
generate the Windows API with:

```sh
cargo doc -p fsr-sdk --features movable --no-deps --locked --target x86_64-pc-windows-msvc
```

The Windows target must be installed; a host-only macOS/Linux build omits the DX12
modules. docs.rs uses Windows x64/MSVC and includes the optional movable API.
The [DX12 integration example](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/crates/fsr-sdk/examples/dx12_upscale.rs)
accepts renderer-owned objects and uses only the public API. It compiles without
implementing a renderer and does not execute GPU work when run.

The accepted [D008](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D008.md)
and [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md)
record runtime trust and the bounded public contract. Zero dimensions are
rejected; further numerical limits are delegated to the trusted runtime and may
fail or abort. One public-route dispatch, fence and plausible readback passed in
[M6b](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-25-m6b-public-dispatch-verification.md).
This is evidence for one configuration, not image-quality, temporal or broad
GPU/driver/provider support. See the
[roadmap](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/ROADMAP.md)
for current status and future scope.

The [final 0.1.0 Windows verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-28-exp-final-windows-release-verification.md)
also passed a three-frame moving smoke. Version `0.2.0` includes
[controlled horizontal-motion, jitter and history-reset comparisons](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-30-exp-dispatch-semantics.md)
on that configuration. These are bounded regression evidence; they do not
establish general temporal image quality or revalidate the published archives.

## Licensing and provenance

Original code is licensed under [MPL-2.0](LICENSE).
The `fsr-sdk-sys` dependency retains AMD's MIT notice for its header-derived
declarations. The test-only provider-query declaration in `src/upscaler/dx12/tests/lifecycle.rs`
and the constructor fixture in `tests/fixtures/upscaler.rs` follow SDK v2.3.0
`Kits/FidelityFX/api/include/ffx_api.h` at commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`; its AMD notice is retained in
[LICENSE-AMD](LICENSE-AMD). The optional Windows device binding uses the
MIT OR Apache-2.0 licensed `windows` crate. AMD SDK components and runtime binaries retain their applicable
terms and are not included in this package.

Project documentation: <https://github.com/Render-Slop-Manufactory/fsr-sdk-rs>.
