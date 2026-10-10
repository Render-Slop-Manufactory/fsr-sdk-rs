# M8 implementation verification — 2026-10-09

## Result and scope

M8 is implemented in current Windows/DX12 source under [D014](../adr/D014.md).
The frame/resource API, checked native sharpening, migration and deterministic
implementation checks passed. This record covers Rust compilation, controlled
local DLL calls and public consumers; no AMD runtime or GPU was executed for
M8 verification. M9's [selected native integration protocol](2026-10-09-m9-native-sharpening-protocol.md)
and final release artifacts remain pending. Published `0.1.0` is unaffected;
the wrapper's manifest version stays `0.1.0` until release preparation selects
the `0.2.0` artifacts. No raw ABI or dependency version changed in this work.

## Checked implementation

[Frame inputs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs) now have
private fields. `UpscaleResources::new` requires all five borrowed textures,
including exposure; accessors expose those borrows, with exposure returning
`Some` for all current constructors. Cloning duplicates borrows without COM
retention or native calls. `UpscaleDispatch::new` explicitly takes list,
resources, jitter, frame time, camera and reset, and defaults to Disabled. Its
selection getter returns `&Sharpening`; only the owned sharpening setter is
provided. Dispatch's unsafe signature and renderer obligations are retained.

[Sharpening types](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/sharpening.rs)
accept finite `[0,1]` values including negative zero. NaN, either infinity,
out-of-range values and the immediate representable values outside the interval
return `Error::InvalidDispatch` with
`DispatchValidation::Scalar(DispatchScalar::NativeSharpness)` before FFI, without
clamping.
The descriptor sets Disabled to `(false, +0.0)` and Native to `(true, v)` with
the input's `f32` bits unchanged. Enabled zero remains distinct from Disabled.
No filter, quality profile, extra quality parameter or visual guarantee is added.
The new types retain D014's Windows/DX12 gate through both `upscaler` reexports
and `upscaler::dx12`; D015's existing common inputs remain portable.

The [public consumer tests](../../crates/fsr-sdk/tests/dispatch_inputs.rs) check
scalar boundaries, bit preservation, both public paths and external
non-exhaustive matching. Compile-time assertions confirm Send/Sync for all four
input types, resources/selection Debug+Clone, scalar Debug+Clone+Copy+PartialEq,
non-Copy resources/selection and non-Copy/non-Clone frame. Existing local
Runtime/Upscaler negative trait assertions remain.

The [controlled-FFI checks](../../crates/fsr-sdk/src/upscaler/dx12/tests/sharpening.rs)
register an opt-in C-ABI callback in the existing local runtime fixture to
inspect the actual complete descriptor reaching its dispatch export. One
context receives Disabled, Native(+0), Native(-0), Native(0.35), Native(1), then
Disabled, with reset only initially. The checks cover selection replacement,
borrowed getter/cloning, exact enablement/scalar bits, supplied resource pointers,
unchanged flags/pre-exposure, no calls on invalid scalar construction, unknown
native error preservation, poison-first rejection and terminal destruction.
Existing full disabled-descriptor and COM/runtime failure tests also pass.

Resource-role tests reject format-distinct swaps, depth/exposure at unequal
extents and color/output at unequal extents before native dispatch, retaining a
usable context. An equal-extent pair with both RGBA16F textures UAV-capable
passes metadata inspection and follows supplied pointer roles, demonstrating the
existing semantic-validation limit without executing the swapped case on a GPU.
Constructor/getter/bundle-clone checks preserve COM reference counts.

The checkout DX12 example, crate Rustdoc, COM inputs and all native harness
inputs use the new constructors. The historical sharpening spike retains its
private override and original effect assertions; its input construction alone
was migrated. Its saved protocol/results were not changed or rerun. Root README
keeps explicitly labeled published-`0.1.0` examples alongside current migration
guidance; checkout examples target current source.

## Commands and observations

Run from the repository root on Windows `x86_64-pc-windows-msvc`; dependencies
and SDK headers were already present locally. `CARGO_INCREMENTAL=0` was used for
the check/lint/doc/consumer commands to avoid filesystem hard-link-cache noise.
Test runs retained harmless cache-copy warnings and passed. All commands below
completed with exit 0 after the implementation was corrected.

| Command | Observation |
|---|---|
| `cargo fmt --all -- --check` | Formatting passed. |
| `cargo clippy --workspace --all-targets --features fsr-sdk/movable --locked --offline -- -D warnings` | All targets and examples passed with warnings denied. |
| `cargo check --workspace --all-targets --no-default-features --features fsr-sdk/movable --locked --offline` | Feature-disabled configuration passed. |
| `cargo +1.88.0 check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Minimum-version target/example compilation passed. |
| `cargo test --workspace --locked --offline --quiet` | Standard-feature workspace tests and doctests passed; wrapper library: 39 passed, 23 native tests ignored. |
| `cargo test --workspace --features fsr-sdk/movable --locked --offline --quiet` | Workspace tests/doctests passed; wrapper library: 42 passed, 24 native tests ignored; both new public tests and all six movable consumer tests passed. |
| `cargo +1.88.0 test --workspace --features fsr-sdk/movable --locked --offline --quiet` | Minimum-version workspace tests/doctests passed, including the same 42 library and two new public tests. |
| `cargo test --workspace --no-default-features --features fsr-sdk/movable --locked --offline --quiet` | Feature-disabled tests/doctests passed; wrapper library: 21 passed, common-input tests: 3 passed. |
| `cargo doc --workspace --features fsr-sdk/movable --no-deps --locked --offline` with `RUSTDOCFLAGS=-D warnings` | Current Windows API documentation passed with warnings denied. |
| `./tools/check-movable-api.ps1 -Stage target/m8-api-controls` | Existing consumer/dependency checks and added M8 controls passed. Private literals/fields and unchecked scalar construction produced the expected E0451/E0616/E0423 diagnostics; disabled native imports produced E0432. |
| `./tools/check-native-abi.ps1 -SdkRoot external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX -Stage target/m8-native-abi` | Four pinned header hashes and all three MSVC `/W4 /WX` ABI compilation probes passed; no runtime linked or loaded. |

Initial validation found a missing Rustdoc import and a role-swap fixture whose
output lacked UAV capability, causing earlier capability rejection than the
intended extent test. The import and fixture were corrected; final test runs
passed. An existing context compile-test initially failed in the sandbox with
access denied writing its temporary metadata; the authorized test reruns outside
the sandbox passed. These initial attempts are not reported as successful runs.

Final diff/consistency checks found no whitespace errors. All 396 local links in
the affected documentation resolved, including their Markdown anchors. The
original D014 comparison-protocol section and 2026-09-30 result record were
compared against their prior contents and are unchanged.

## Remaining verification

Implement the separate M9 public-API GPU runner and execute its predeclared
same-context controls, disabled native regressions, fences/readbacks and
debug/device/cleanup checks. A prior historical spike or concurrent baseline
native recheck does not verify this new public frame path. Final versions,
packaged Rustdoc/examples, archives and downstream package consumers still need
release validation. M9 is not complete, and publication is not authorized by
these implementation checks.
