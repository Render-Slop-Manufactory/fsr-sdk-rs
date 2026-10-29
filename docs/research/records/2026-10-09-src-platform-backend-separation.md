# Source investigation: platform and graphics-backend separation

- Investigation date: 2026-10-09, Europe/Berlin.
- Question: how should Windows/Linux and DX12/Vulkan dependencies be separated
  without confusing portable Rust code with native FSR support?
- Baseline: clean working tree at the start; current source includes D013's
  optional movable family and D015's portable common inputs. Both manifests
  still declare version `0.1.0`; current source is distinct from published
  archives. Supplementary revision: `16ebebf2d92e8d17bf3f2bf81d3c11c2be66c3a8`.
- Native SDK: AMD FSR SDK `v2.3.0`, pinned commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3` under D002.
- Method: source/manifests/CI inspection, primary-source browsing, offline
  dependency resolution and Rust compilation. No production changes, new
  dependencies, SDK downloads, native loading or GPU execution.
- Status: findings and recommendations; no new architecture decision or support
  commitment. D002, D008, D013 and D015 remain in force.

## Result

The existing two-crate architecture and common/DX12 module separation are a
sound starting point. The concrete unnecessary production dependency is
unconditional `libloading` in `fsr-sdk-sys`. Windows interface dependencies are
already isolated correctly. Native Linux/Vulkan support is blocked by the
selected upstream integration path, not merely by Rust module organization.

Recommend a small dependency cleanup now, retain concrete backend APIs, and
design a second native runtime only when a usable upstream backend is selected.
Splitting into several more crates or introducing a public backend trait has no
demonstrated benefit for the current implementation.

## 1. Four independent boundaries

| Boundary | What it selects | Appropriate mechanism |
|---|---|---|
| Target platform/ABI | Windows versus Linux; architecture and toolchain | Rust target and `cfg`; target-specific dependencies |
| Graphics backend | DX12 versus Vulkan resources and recording | Additive Cargo features and concrete backend modules |
| Native runtime/provider | Library exports, effect providers, deployment and lifetime | Explicit acquisition, retained ownership, compatibility contract |
| Effect/profile | Upscaling technology and accepted frame conventions | Effect API, validated inputs and recorded provider evidence |

The build host is separate again: building Windows Rust code from macOS does
not turn it into a macOS native runtime. Dependency resolution for a Linux target
does not execute Linux code.

**Inference:** `windows` must not mean DX12, and `linux` must not mean Vulkan.
Windows/Vulkan is a separate possible integration cell. A Windows binary run
through Wine/Proton would still use the Windows ABI; translating its graphics
calls underneath would not create native Linux bindings. This investigation
does not test or promise such compatibility.

Cargo features express requested capabilities, not operating systems. Adding
`windows`/`linux` features would duplicate the Rust target and allow contradictory
requests. Future `dx12` and `vulkan` implementations should coexist when their
native contracts permit it; features are unified across consumers. There should
be no implicit preference or exclusive selection merely because both features
are enabled. See [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html).

## 2. Actual dependency and API boundaries

**Verified from current source:**

| Area | Current boundary | Assessment |
|---|---|---|
| `upscaler::common` | Every host, no backend required | Correct portable value boundary |
| `error` | Every host; primitives and owned enums | Portable despite some DX12-specific meanings |
| Raw `api` / selected `upscale` declarations | Mostly ungated; Windows callback/create root gated separately | Reusable declarations, not a portable native ABI guarantee |
| `upscaler::dx12` and legacy owner reexports | `all(windows, feature = "dx12")` | Correct concrete resource boundary |
| `runtime` and raw `loader` | Same Windows/DX12 gate | Correct for the only implemented native path |
| Wrapper `windows` dependency | Optional and under `cfg(windows)` | Already correctly isolated |
| Raw `libloading` dependency | Unconditional | Unnecessary for production common-only consumers |
| `movable` | Additional optional ownership family within native gates | Thread policy, not another backend or platform |
| `vulkan` | Explicit compilation error in raw crate | Unsupported marker, not an implementation |

Evidence: [wrapper manifest](../../../crates/fsr-sdk/Cargo.toml),
[raw manifest](../../../crates/fsr-sdk-sys/Cargo.toml),
[wrapper root](../../../crates/fsr-sdk/src/lib.rs),
[raw root](../../../crates/fsr-sdk-sys/src/lib.rs),
[upscaler modules](../../../crates/fsr-sdk/src/upscaler.rs), and
[D015](../../adr/D015.md).

Offline production dependency trees show:

```text
Linux target, wrapper defaults:
  fsr-sdk -> fsr-sdk-sys -> libloading -> cfg-if

Windows target, wrapper defaults:
  fsr-sdk -> fsr-sdk-sys -> libloading -> windows-link
          -> windows and its transitive dependencies

Windows target, no defaults:
  fsr-sdk -> fsr-sdk-sys -> libloading -> windows-link
```

Thus Linux does not inherit the `windows` interface crate. Windows without DX12
still gets `windows-link` through the unused loader dependency. An entry in
`Cargo.lock` alone would not prove that a package is built for a target; the
normal dependency trees are the relevant evidence. No size or build-time
benchmark was run, so the practical saving is not quantified.

`error::Dx12Inspection` stores an `i32` HRESULT rather than a Windows crate
type. It causes semantic coupling but no Windows dependency. Keep existing
variants for compatibility. If Vulkan is introduced, preserve Vulkan results in
their own category rather than interpreting them as HRESULTs. The COM-specific
documentation in common error categories can be refined then.

`UpscalerOptions` contains portable dimensions but currently documents the
fixed DX12 profile. Portable representation does not prove that another backend
supports the same profile. Share checked units only where their meaning and
valid domain remain equivalent.

## 3. Upstream is the native blocker

**Verified source statement:** the pinned
[v2.3.0 README](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/readme.md)
lists Vulkan as unsupported. Its opening description mentions Vulkan, but the
explicit support limitation and concrete integration documentation qualify that
general description.

The pinned
[FSR API guide](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/getting-started/ffx-api.md)
describes signed DLL acquisition, currently DX12-only backend functionality,
and separate loader/effect DLLs. These are upstream requirements, not new runtime
observations. The wrapper retains a library exporting five common functions;
AMD's loader then resolves effect providers. Those are two separate loading
layers. Porting the Rust layer does not port AMD's provider resolution.

The pinned
[common header](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h)
contains Vulkan backend IDs and old Vulkan swapchain identifiers. It also uses
`__declspec(dllexport)` and a `wchar_t` message callback. Identifiers and opaque
`void*` payloads do not establish a working Linux/Vulkan runtime.

Historical
[SDK v1.1.4](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v1.1.4/readme.md)
does describe Vulkan integrations and Vulkan sample issues. That is a different
SDK baseline. It establishes neither native Linux support nor an equivalent
modern FSR 4.1.1 provider. The legacy tag was inspected as context only; its full
commit, buildability and runtime were not independently verified here.

**Recommendation:** if native Linux is required now, first select an actual
upstream runtime/source path and its technology coverage. Treat an older Vulkan
implementation as a separately versioned integration, with its own ABI,
deployment and safety review. Do not disguise it as equivalent `vulkan` support
for the current signed v2.3.0 path.

## 4. Runtime separation requires more than moving files

`RuntimeState` owns only `FfxLibrary`; it contains no DX12 device. Keeping
`runtime::Runtime` in its current location is therefore reasonable under D015.
Its contract nevertheless explicitly trusts the Windows/DX12 runtime/provider
family. The private lifecycle owner also directly depends on this runtime state;
it is not an independent runtime-provider interface.

The private generic `NativeContextOwner<S, D, R>` already separates retained
effect descriptors, device dependencies and Rc/Arc runtime ownership. Reuse that
small mechanism where native contracts match. Its generic parameters do not
prove that the DX12 assumptions are valid for another implementation.

**Verified portability hazard:**
[loader.rs](../../../crates/fsr-sdk-sys/src/loader/windows.rs) stores nullable ABI
function pointers, relies on Windows symbol resolution rejecting null, then
uses `expect("resolved Windows export")` when calling them. In the locally cached
`libloading 0.9.0` source, `src/os/unix/mod.rs`, `Library::get` on Linux routes
through a path that can return an `Ok(Symbol)` containing a null address when
`dlsym` returns null without a loader error. Lifting the Windows gate alone
would therefore not preserve the acquisition invariant. A future Unix loader
must reject a missing/non-callable required export during acquisition. This is
not a defect in the currently gated Windows implementation.

A Linux port would also need concrete native calling-convention/layout checks,
callback character representation, native dependency search rules, symbol
visibility and partial-load/unload behaviour. The existing Windows callback's
`*const u16` must not simply become the Linux callback. Ungated raw Rust
declarations and a passing Rust build establish none of these contracts.

If a second runtime really exposes the same ABI, share private library retention
and forwarding, with small OS-specific loading operations where necessary. Keep
typed acquisition boundaries so a Vulkan owner cannot accept a DX12 runtime
merely because both expose the same five symbol names. Concrete
`runtime::dx12::Runtime` / `runtime::vulkan::Runtime` types with shared private
implementation are an option then; retain existing paths as DX12 reexports.
Do not introduce those modules before the second path exists.

## 5. Vulkan needs its own resource contract

The concrete DX12 adapter currently owns an AddRef'd device, checks canonical
COM identity, obtains resource descriptions and device identity, maps DXGI
formats, and validates DIRECT command-list type. These belong in the DX12 module:
[resource inspection](../../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs).

The following is a future design checklist, not selected Vulkan API:

| Concern | DX12 implementation | Required Vulkan investigation |
|---|---|---|
| Device lifetime | Owned COM reference | Explicit owner/borrow contract preventing premature native destruction |
| Image metadata | `GetDesc` on the COM object | Trusted creation metadata or a resource owner; no assumed COM-like introspection |
| Device association | `GetDevice` and canonical identity | Device provenance for opaque handles |
| Recording | DIRECT command list | Command-buffer state, pool and applicable queue capabilities |
| Synchronization | Documented D3D12 resource states and caller fences | Image layouts, stage/access scopes, subresources and queue-family ownership |
| Memory | Caller establishes non-overlapping backing storage | Memory binding/aliasing, residency and pending-use contract |

Khronos documents device-bound child lifetimes and explicit cleanup in
[vkDestroyDevice](https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyDevice.html).
Image creation metadata includes format, extent, usage and sharing mode in
[VkImageCreateInfo](https://docs.vulkan.org/refpages/latest/refpages/source/VkImageCreateInfo.html).
Layout transitions and queue-family transfer fields appear in
[VkImageMemoryBarrier2](https://docs.vulkan.org/refpages/latest/refpages/source/VkImageMemoryBarrier2.html).
These references describe Vulkan itself, not an available AMD FSR backend; they
were accessed on 2026-10-09 and were not pinned to a future FSR integration.

**Inference:** cloning a Rust Vulkan handle wrapper cannot substitute for the
current independent COM device retention. A future backend must explicitly own
or borrow the actual native lifetime. `Arc` retains Rust storage, not an
externally destroyed `VkDevice`. Likewise the movable family's manual `Send`
audit, native cleanup trust and dispatch failure policy require backend-specific
review. Do not copy D008/D013 trust to Vulkan automatically.

Keep common scalars separate from concrete borrowed GPU inputs. Prefer separate
`dx12::UpscaleDispatch` and `vulkan::UpscaleDispatch` over an enum of raw pointers
or a public trait erasing their safety differences. Introduce shared internals
only after real duplication is visible.

## 6. Small recommended next step

For the current raw crate, a proposed manifest boundary is:

```toml
[features]
default = []
dx12 = ["dep:libloading"]
vulkan = [] # Existing explicit unsupported marker remains.

[target.'cfg(windows)'.dependencies]
libloading = { version = "0.9.0", optional = true }
```

Keep its loader's existing Windows/DX12 gate. The wrapper already forwards
`dx12` and restricts `windows` correctly. Its direct `libloading` dev-dependency
is used by portable fixtures and should remain. Production and test dependency
graphs have different purposes.

This eliminates the loader from common-only production trees, including Linux
with wrapper defaults and Windows without DX12. The snippet is a recommendation,
not an applied or tested patch. Cargo's
[dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies)
requires optional dependencies plus feature wiring; a feature expression inside
the target dependency table would not work as intended.

Making the wrapper's `fsr-sdk-sys` dependency optional could also remove the
small raw declaration crate from common-only builds. That needs separate test
wiring because portable ownership fixtures use the raw ABI. Defer it unless
consumers need a completely dependency-free common surface. No extra common
crate is needed merely to isolate dynamic loading.

Retain the DX12 default and existing import aliases. Changing defaults would
alter ordinary consumer behaviour; changing the public `windows` crate version
also changes resource type identity under D008. Neither follows from this
analysis. The current unsupported Vulkan marker intentionally makes
`--all-features` fail; add a negative control for its exact diagnostic instead
of turning it into an empty successful backend.

## 7. Verification and implementation criteria

Checks below ran on macOS ARM64, Rust `1.98.1`, using cached dependencies.
Installed Rust targets: `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`.

| Executed command | Result and limit |
|---|---|
| `cargo tree -p fsr-sdk --target x86_64-unknown-linux-gnu --edges normal --locked --offline` | Resolved Linux production graph above; no Linux compilation |
| `cargo tree -p fsr-sdk --target x86_64-pc-windows-msvc --edges normal --locked --offline` | Resolved Windows default graph above |
| Same Windows tree with `--no-default-features` | Loader and `windows-link` remain; no `windows` interface crate |
| `cargo check --workspace --all-targets --no-default-features --locked --offline` | Pass on macOS; portable code and fixture compilation |
| `cargo check --workspace --all-targets --features fsr-sdk/movable --locked --offline` | Pass on macOS; native API remains gated out |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --locked --offline` | Pass; Windows compilation with native API disabled |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features fsr-sdk/movable --locked --offline` | Pass; Windows DX12/movable Rust compilation |
| `cargo check -p fsr-sdk --no-default-features --features vulkan --locked --offline` | Expected exit 101, exact intentional unsupported-Vulkan diagnostic |

The current [CI workflow](../../../.github/workflows/ci.yml) defines Ubuntu and
Windows portable/default/movable checks, consumer import boundaries and pinned
Windows ABI probes. This inspection does not establish that remote CI has run
successfully. Existing [D015 verification](../../ROADMAP_HISTORY.md#portable-inputs-and-loader-checkpoint)
supplies earlier bounded Windows native evidence; it was not rerun here.

For the proposed cleanup, acceptance should cover downstream production trees
on Linux and Windows, common/native positive and negative import controls,
portable fixtures, Windows loader/lifecycle failures, and both ownership families.
Use downstream trees rather than only workspace tests: test dependencies can
otherwise hide an unwanted production dependency. Preserve raw ABI checks and
selected existing native regressions when implementing the change.

Future native Linux/Vulkan support needs a separate progression: pinned usable
upstream artefacts, native ABI comparison, acquisition/failure/unload checks,
query, lifecycle, submitted/fenced dispatch, readback and temporal regressions.
A headless upscaler proof does not need to introduce window-system integration.

## Baseline delta and remaining choices

This investigation confirms D015's boundary and identifies dependency cleanup
and the null-symbol invariant as concrete follow-up points. It changes no SDK,
release sequence, support claim, safety decision or API. Original work remains
under the project's MPL-2.0 terms; retained AMD materials keep their own notices.
No new native bundle was inspected or cleared for redistribution. A legacy or
future runtime selection requires its own artefact and licence review under
[existing distribution findings](../licensing-and-distribution.md).

The material unanswered product choice is whether Linux means portable common
configuration, a Windows application under a compatibility layer, or a native
Vulkan FSR integration. The first is already available. The second is untested.
The third requires a different usable upstream path before implementation can
be scoped. Clarify that choice if implementation is requested; it does not block
the analysis above.
