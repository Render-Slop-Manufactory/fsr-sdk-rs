# Changelog

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
[release verification](docs/release/2026-10-10-0.2.0-release-verification.md).

The supported native target remains `x86_64-pc-windows-msvc`. Evidence is bounded
to the recorded provider/GPU/driver and workloads. Vulkan and other SDK effects
remain unimplemented; GBV timeout and the original failed sharpening-effect
experiment remain unresolved historical results.

## 0.1.0 — 2026-09-28

Initial two-crate release with explicit trusted runtime loading, checked DX12
upscaler construction, terminal ownership and a narrow unsafe dispatch route.
See the [publication record](docs/release/2026-09-29-publication-check.md).
