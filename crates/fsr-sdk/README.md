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

## Consumer workflow

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
masks and sharpening are not exposed. The API uses `windows` 0.62 DX12 interfaces.

## API contract and example

Full field, error and safety contracts live in Rustdoc. In a repository checkout,
generate the Windows API with:

```sh
cargo doc -p fsr-sdk --no-deps --locked --target x86_64-pc-windows-msvc
```

The Windows target must be installed; a host-only macOS/Linux build omits the DX12
modules. docs.rs is configured to use Windows x64/MSVC as the default target.
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
for release work and future scope.

## Licensing and provenance

Original code is licensed under [MPL-2.0](LICENSE).
The `fsr-sdk-sys` dependency retains AMD's MIT notice for its header-derived
declarations. The test-only provider-query declaration in `src/upscaler/tests.rs`
and the constructor fixture in `tests/fixtures/upscaler.rs` follow SDK v2.3.0
`Kits/FidelityFX/api/include/ffx_api.h` at commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`; its AMD notice is retained in
[LICENSE-AMD](LICENSE-AMD). The optional Windows device binding uses the
MIT OR Apache-2.0 licensed `windows` crate. AMD SDK components and runtime binaries retain their applicable
terms and are not included in this package.

Project documentation: <https://github.com/Render-Slop-Manufactory/fsr-sdk-rs>.
