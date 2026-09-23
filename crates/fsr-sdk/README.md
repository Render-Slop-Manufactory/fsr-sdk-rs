# fsr-sdk

Engine-independent Rust abstractions for AMD's FSR SDK. This crate is currently
a private Windows/DX12 context-ownership foundation with native error types.
Public construction and upscaling dispatch are not exposed yet. Windows fixtures,
paired native ABI checks and an AMD create/query/destroy lifecycle through the
production owner passed on the tested Windows x64/MSVC configuration. See the
[M4 verification record](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/research/records/2026-09-22-exp-m4-windows-verification.md)
for inputs and limits. This does not establish GPU dispatch or broader hardware
support.

Original code is licensed under [MPL-2.0](LICENSE).
The `fsr-sdk-sys` dependency retains AMD's MIT notice for its header-derived
declarations. The test-only provider-query declaration in `src/upscaler/tests.rs`
is derived from SDK v2.3.0 `Kits/FidelityFX/api/include/ffx_api.h` at commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`; its AMD notice is retained in
[LICENSE-AMD](LICENSE-AMD). The optional Windows device binding uses the
MIT OR Apache-2.0 licensed `windows` crate. AMD SDK components and runtime binaries retain their applicable
terms and are not included in this package.

Project documentation: <https://github.com/Render-Slop-Manufactory/fsr-sdk-rs>.
