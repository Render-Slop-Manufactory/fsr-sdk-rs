# Common type organization — 2026-10-29

**U14 is implemented; targeted checks pass.** Full source/package/native release
qualification remains deferred until U15 and documentation integration finish,
as selected in the [verification guide](../verification-selection.md).

`upscaler/common.rs` is reduced from 1,050 to 67 lines. It retains canonical
re-exports and the shared scalar-error helper. Private modules now own camera
parameters, diagnostic filtering, dimensions, creation options and requirements,
profile selections, frame scalars and tuning requests. Existing frame, sharpening,
reactive-mask and usability modules retain their implementations. Common profile
tests are separated; the test constructing invalid private motion fields stays
inside the scalar module, without widening field visibility.

The source comparison finds identical token blocks for all 65 functions, 12
complete implementations, eight structs and nine enums formerly in `common.rs`.
Imports/module placement and comments are excluded. Public paths, fields,
defaults, input checks and native mappings are preserved. Two moved intradoc
links have explicit destinations; the final Rustdoc contract pass remains U15.
Explicit `#[path]` declarations preserve standalone source-fixture loading.

The [machine-readable record](2026-10-29-common-organization-verification.json)
retains every command/exit code, test summaries, log hashes, build settings and
before/after source hashes. Windows x64/MSVC, Rust 1.99.0 and exact MSRV 1.88.0;
all commands use cached locked dependencies. No native runtime is required.

| Selected check | Result |
|---|---|
| Repository Rust formatting | Passed |
| Affected-crate all-targets Clippy, default + movable + utils and no-default + utils | Passed with warnings denied |
| MSRV all-targets compilation of those two configurations | Passed |
| Common unit tests, Stable and MSRV, no-default + utils | Eight passed on each compiler |
| Portable `dimensions`, `frame_parameters`, `tuning_requests` integration tests | 11 + 3 + 2 passed, including public re-export identity and accepted scalar bits |
| Isolated `context::tests::consumed_owner_and_private_handle_are_compile_errors` fixture | Passed, including its positive control against the moved source modules |
| Portable Rustdoc, no-default + utils | Passed with warnings denied |
| Cargo package file listing | Every moved module and test file included |

The eleven selected commands pass. Logs remain under ignored
`target/common-organization-checks`; reproduction uses the complete argument
lists and environment in the JSON. The tests are existing cases, relocated
without changing their assertions or inputs. Feature-sensitive imports retain
the original `test` / Windows+DX12 gates.

No GPU profiles, performance measurements, full CI/infrastructure matrix or
archive-native qualification were run. This move adds no runtime operation;
it does not prove identical generated code or certify different package bytes.
U15 and final candidate reconciliation own the remaining documentation,
performance-impact review and complete release checks. The
[consolidated verification report](2026-10-29-candidate-verification.md) remains
the release evidence hub.
