# fsr-sdk

Engine-independent Rust abstractions for AMD's FSR SDK. On Windows/DX12,
`runtime::Runtime::load` acquires an explicit trusted v2.3.0 loader under an
unsafe integration contract. `upscaler::Upscaler::new` safely constructs a
context from that runtime, a borrowed `upscaler::ID3D12Device` (`windows` 0.62),
and checked dimensions in `UpscalerOptions`. It retains the runtime and an
independent COM reference through teardown. Zero sizes are rejected; further
numerical limits are delegated to the trusted runtime and may fail or abort.
The public `unsafe Upscaler::dispatch` records one narrow, fixed-profile
DX12 frame using borrowed list and textures; the caller retains them and the
upscaler through GPU completion, controls states and submission, and discards
failed recordings. See its `# Safety` documentation and
[D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md).
The M4 Windows fixtures,
paired native ABI checks and an AMD create/query/destroy lifecycle passed on
the tested Windows x64/MSVC configuration. See the
[M4 verification record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-22-exp-m4-windows-verification.md)
for inputs and limits. M5's public path has passed Windows-target compilation and ordinary
Windows CI fixture tests in the
[recorded run](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-24-exp-m5-windows-ci-verification.md);
both opt-in native lifecycle cases
(explicit destruction and `Drop`) passed on the tested Windows x64/MSVC
configuration. See the
[M5 verification record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-24-exp-m5-windows-native-verification.md)
for inputs and limits, and [D008](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D008.md)
for the runtime, provider and external/native integration obligations.
M6a's private path then completed one synthetic DX12 dispatch, fence and
readback through that public constructor on the tested configuration. The
[M6a verification record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
gives the earlier private-path proof. The public route passed a separate
[M6b verification](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
on the tested Windows x64/MSVC configuration. Neither single-frame proof
establishes temporal quality or broader device support.

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
