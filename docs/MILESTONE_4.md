# Milestone 4: context ownership implementation plan

Status: implemented and verified on the tested Windows x64/MSVC configuration, 2026-09-22. This plan accompanies
[D005](DECISIONS.md#d005--own-effect-independent-context-lifecycles-with-terminal-teardown).
The implementation follows this design; the verification status below distinguishes
host tests, Windows compilation and native execution.
The [roadmap](../ROADMAP.md) remains authoritative for milestone status.

## Preconditions and scope

D005 is accepted. Apply its evidence-backed runtime trust assumption for
caller-input cleanup after returned create failure; it is not a universal header
guarantee or an unresolved M4 blocker. The private unsafe boundary must document
actual input, lifetime and threading obligations, not claim to prove the runtime
assumption. Fixture coverage verifies wrapper policy, not that native property.

The first concrete effect remains the upscaler on Windows/DX12. Keep the common
ownership mechanism independent of its descriptor types. Do not implement other
effects, dispatch, public query/configure APIs, runtime discovery, provider
selection, custom allocators, callbacks or general backend traits.

## Private representation and construction

The implementation uses a private `NativeContextOwner<S, D>` containing
`Option<LiveContext<S, D>>` and an explicit `PhantomData<Rc<()>>` marker. The marker
requires no actual Rc allocation. Live state contains the private handle, an
audited retained bundle S, an owned device reference and an exclusively owned
`FfxLibrary`. Live state has no separate native-destroy destructor. These are
implementation choices, not permanent public API commitments. `D` is an owned
dependency instantiated with ID3D12Device in production and drop counters in
fixtures; it has no backend trait or public extension API.

S is not an arbitrary user-supplied value. Each concrete instantiation must
preserve the lifetimes and addresses of every natively referenced dependency,
even when intentionally retained indefinitely. It must have non-panicking
destructors. Borrowing externally owned memory does not meet this requirement
merely because the borrowing value can be forgotten. A `'static` bound alone
would not validate raw pointers either.

For the upscaler, start with `Pin<Box<UpscaleCreateState>>` plus `PhantomPinned`
containing root, version and DX12 backend descriptors. Allocate once, then link
`Upscale -> UpscaleVersion -> BackendDX12 -> null` at final addresses. Keep fields
private; permit no pointee extraction, replacement or external mutation. Audit
raw-pointer formation and Rust aliasing separately from address stability. Pinning
alone does not establish native pointer validity. Moves of the outer owner must
not change descriptor addresses.

The effect-specific code prepares this state and the matching root pointer.
A shared internal create operation already owns all dependencies while invoking
`ffxCreateContext` with a null-initialized output and null allocator. Its private
unsafe boundary must require the root to refer to the retained valid chain,
the matching live device/runtime, and justified native lifetime/threading
preconditions. Keep ownership establishment after successful create free of
fallible or panicking intermediate work. Centralize all result classification
from D005, including non-OK/non-null and OK/null; do not duplicate it per effect.

The private device owner is
`windows::Win32::Graphics::Direct3D12::ID3D12Device`, consumed by value without a
hidden clone. The crate does not appear in public signatures. The selected
`windows` version is `0.62.2` (Cargo's compatible 0.62 range, exact resolved version
in Cargo.lock), with default features disabled and only
`Win32_Graphics_Direct3D12` requested. It is an optional Windows-target dependency
activated by `dx12`; `libloading` remains global in sys.

This Microsoft-maintained binding supplies typed COM ownership and documented
`Interface::as_raw` borrowing / `from_raw` reference adoption. It avoids maintaining
a handwritten COM vtable. Using only windows-core/IUnknown would be narrower but
lose the concrete device type at this boundary. The cost is its generated binding
and support crates (14 newly resolved packages including procedural-macro
support); these are recorded in Cargo.lock and are not an SDK download.
`windows` is MIT OR Apache-2.0; the resolved support dependencies use compatible
MIT/Apache-2.0 licensing (`unicode-ident` additionally requires Unicode-3.0). Project and AMD
licensing remain unchanged. See the
[upstream package](https://docs.rs/crate/windows/0.62.2) and
[Interface ownership API](https://docs.rs/windows-core/0.62.2/windows_core/trait.Interface.html).

Consume an already acquired FfxLibrary; do not load a path inside creation.
Initially no Rc/Arc runtime sharing is needed. Future multiple-context designs
can revisit that representation while preserving retention on failure.

Use one internal teardown function for consuming destroy and Drop. Take live
ownership before calling native destroy with a writable handle slot and null
allocator. On success, release retained state, then the owned device reference,
then runtime ownership; audit actual field/explicit drop order. On failure,
intentionally retain effect-specific state, the owned device reference and
runtime/library ownership, and never revisit its handle. Retain the same three
dependency classes for OK/null creation. The retention path needs no allocation,
lock, global registry or user callback.
Noncritical error metadata can be released normally. Retention must preserve
heap/native resources, not rely on addresses of moved inline wrapper values.

## Files and minimal ABI promotion

The following production files implement this slice.

| File | Intended responsibility |
|---|---|
| `crates/fsr-sdk/src/context.rs` | Private effect-independent owner, shared create outcome handling, terminal teardown and dependency retention. |
| `crates/fsr-sdk/src/upscaler.rs` | Private concrete retained create state and bounded constructor inputs; no public builder. |
| `crates/fsr-sdk/src/error.rs` | Create/Destroy operation enum; Native with unchanged u32 code; NativeInvariantViolation with a small reason representation; Display/Error implementations. |
| `crates/fsr-sdk/src/lib.rs` | Platform gates; expose no raw handle or unnecessary public generic API. |
| `crates/fsr-sdk/Cargo.toml`, `Cargo.lock` | Windows/DX12-specific device ownership using windows 0.62.2. |
| `crates/fsr-sdk-sys/src/api.rs` | Common `ffxApiMessage` from `api/include/ffx_api.h` and `FfxApiDimensions2D` from `api/include/ffx_api_types.h`, with exact provenance. |
| `crates/fsr-sdk-sys/src/upscale.rs` | `ffxCreateContextDescUpscale`, `ffxCreateContextDescUpscaleVersion`, version tag and `FFX_UPSCALER_VERSION` from `upscalers/include/ffx_upscale.h`; reuse existing root tag. |
| `crates/fsr-sdk-sys/src/dx12.rs` | Backend descriptor, tag and opaque raw device pointee from `api/include/dx12/ffx_api_dx12.h`; no COM ownership in sys. |
| `crates/fsr-sdk-sys/src/lib.rs` | New module and appropriate platform/feature gates. |

Do not promote dispatch, resource, allocator-extension or future-effect types.
Do not remove or demote the existing M3 `ffxQueryDescGetVersions` ABI in
`fsr-sdk-sys`. Keep only additional provider-identification/instrumentation-only
declarations test-local unless production code independently requires them.
Windows callback `wchar_t` is u16; gate the affected
declarations appropriately rather than asserting this as a universal C layout.
Preserve AMD notices and extend the sys README's source-header provenance.

## Verification plan

Prefer fixture DLL calls through the existing FfxLibrary for production-path
coverage, with small private drop-counter helpers where needed. No general
production trait hierarchy is needed for tests. Synthetic fixtures establish
wrapper behavior, not AMD failure postconditions.

- Create OK/non-null establishes one owner.
- Create non-OK with null or non-null output preserves even unknown u32 codes,
  establishes no owner, makes no destroy call and releases wrapper inputs.
- Create OK/null reports an invariant violation, never destroys and retains
  effect-specific retained state, the owned device reference and runtime/library
  ownership.
- Explicit destroy and ordinary Drop each attempt destroy once; dropping the
  consumed shell performs no second call.
- Destroy failure never retries, whether native leaves the handle unchanged or
  clears it; the exact error code is preserved.
- Dependencies remain alive during both native calls. Counters verify normal
  cleanup, ordering and selective absence of cleanup on exceptional paths.
- Moves of the concrete outer owner preserve the upscaler chain addresses.
- Compile/trait checks reject Copy, Clone, Send, Sync, public raw-handle access
  and use after consuming destroy. Private-type tests should verify these
  properties directly, not pass merely because importing the type is forbidden.
- Pair Rust ABI type/layout/tag checks with the existing optional MSVC C++
  checks against local v2.3.0 headers. Keep expectations synchronized.

Move the ordinary opt-in successful AMD lifecycle verification to the wrapper's
production owner, using internal test access for provider identification rather
than exposing the handle. Adapt the existing runner and support files as needed;
do not preserve a second ordinary experiment-only ownership implementation.
The custom-allocator crash experiment remains a separate raw research probe
because that capability is deliberately outside M4.

After implementation, select ordinary Windows/DX12 and no-default-feature checks
from [development guidance](DEVELOPMENT.md), plus paired native ABI compilation.
Run opt-in native verification only with justified safety preconditions and
trusted local runtime inputs. Record SDK, provider, GPU and driver separately
from fixture results. Do not manufacture corrupt native state or retry a failed
destroy to test the policy. A successful lifecycle proves no dispatch support.

## Boundaries for future effects

Reuse ownership and terminal destroy mechanics where the effect's contract
permits it. Each new effect must establish its additional dependencies and any
shutdown, callback or GPU-completion requirements before native destroy.
Retention must include any relevant state introduced after creation as well.
Frame-generation swapchain integration, for example, involves queues and other
objects beyond the upscaler chain; storing S alone does not prove safe shutdown.
No refactor-free guarantee or new backend support follows from this design.

## Current verification and remaining work

On macOS, the actual generic lifecycle owner is tested through a test-only
runtime adapter loading a locally compiled fixture library. Drop counters verify
state/device cleanup and retention, the adapter reports runtime-owner release,
and compile checks cover consuming destroy and private handle access with a
positive control. This is wrapper-policy evidence, not an AMD macOS backend.
macOS may keep the fixture image resident after handle release; no host unload
claim is made. Windows/DX12 fixtures instead use the production FfxLibrary and
check DLL residency. Trait assertions inspect the real private types.

On 2026-09-22, all ten executable host tests, formatting, Clippy,
no-default-feature checks and host documentation builds passed. Windows x64/MSVC cross-checks include all Rust test targets
and Clippy; they do not link or execute the Windows tests. Descriptor-address
checks and actual COM ownership are Windows-specific. The real opt-in test
creates and destroys through Upscaler/NativeContextOwner, querying the provider
only through test-local access. The old ordinary raw lifecycle test was removed;
the separate allocator/input-lifetime research probes remain raw.

Run `powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1` on Windows for
ordinary checks. Add `-Native` from an x64 VS developer shell with the local SDK
and hardware device for paired C++ ABI verification and isolated native lifecycle
execution. Results go under `target/m4-lifecycle/`. On 2026-09-22 these checks
passed: 17 ordinary Windows executable tests, two compile-fail doctests,
formatting, Clippy, no-default-feature checking, paired C++ ABI compilation and
the production owner's actual AMD lifecycle. The native child exited 0 without
timeout, identifying provider 4.1.1 on RX 9060 XT / driver 32.0.31041.1004.
The [Windows experiment](research/records/2026-09-22-exp-m4-windows-verification.md)
records reproduction, fingerprints and limits. M4 verification is complete for
this bounded configuration. No GPU dispatch is included; native failure
postconditions and global threading guarantees remain outside this evidence.

Package file lists include the new source/tests and notices. The local sys Cargo
archive contains LICENSE and LICENSE-AMD. Wrapper Cargo archive creation remains
blocked by the pre-existing path-only sys dependency lacking a version requirement;
release packaging is not part of M4 and that manifest policy was not changed.
