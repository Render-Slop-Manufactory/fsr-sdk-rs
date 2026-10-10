# Platform and graphics backends

- Updated: 2026-10-09.
- Evidence: [platform/backend investigation](records/2026-10-09-src-platform-backend-separation.md),
  [D015](../adr/D015.md), and
  [portable-common implementation verification](../release/2026-10-09-portable-common-verification.md),
  followed by [loader refactor verification](../release/2026-10-09-loader-platform-verification.md).
- Scope: current Rust dependencies, module/API gates and the pinned AMD FSR SDK
  v2.3.0 path. Recommendations do not accept a new platform or backend.

## Current findings

**Verified:** common checked upscaler inputs and owned errors are available
without native backend gates. DX12 resources, runtime and raw loader require
Windows plus `dx12`. Optional `movable` adds a conditional thread-transfer family
within those gates. Windows interface dependencies are optional and restricted
to Windows already.

**Resolved dependency issue:** the investigation found unconditional raw
`libloading` in Linux and Windows common-only production trees. The authorized
D015 loader follow-up now makes it optional under `cfg(windows)`, activated by
`dx12`. Verified production trees omit it for common-only use and Linux, including
Linux with DX12 enabled; Windows/DX12 retains it. Fixture dev-dependencies remain.
These trees establish dependency selection, not native execution or measured
overhead.

**Implemented organization:** `loader/mod.rs` reexports the unchanged public
types from private `loader/windows.rs`. The existing Windows/DX12 gate and all
native contracts remain. This adds no Linux module or common loading framework.

**Verified upstream statements:** pinned v2.3.0 documentation explicitly excludes
Vulkan and describes a signed Windows/DX12 DLL integration. Older SDK Vulkan
paths and retained Vulkan ABI identifiers do not establish a compatible native
Linux/Vulkan implementation for the selected path.

**Verified porting hazard:** the raw loader relies on Windows symbol resolution
to establish non-null functions. The inspected `libloading 0.9.0` Unix code can
successfully resolve a null symbol on Linux. Removing the gate alone would not
preserve acquisition invariants. Callback character ABI, native artefacts,
provider resolution and cleanup also need separate review.

## Recommendations

Keep the two crates, portable common types and concrete backend modules.
The scoped loader/dependency cleanup is implemented under D015. Making the
wrapper's raw-crate dependency optional remains a separate, lower-priority
choice if completely dependency-free common usage is required.

Use the Rust target for OS selection and features for backend capabilities.
Do not couple Linux to Vulkan or add OS features. Preserve current DX12 defaults,
legacy imports and unsupported Vulkan diagnostics. Share private lifecycle
mechanisms where contracts match; defer public backend traits and additional
runtime modules until a second usable native path exists.

A future Vulkan backend needs device-lifetime ownership, trusted image metadata
and provenance, native resource mapping, command-buffer and queue-family rules,
layout/access synchronization, and its own native/threading/cleanup evidence.
Cloned handle storage does not provide COM-style device retention. Current
DX12 trust and `Send` audits do not transfer automatically.

## Evidence limits and open choices

The implementation follow-up passes macOS workspace tests, Clippy, Rustdoc,
Windows compilation with and without DX12/movable, two Windows-target loader
compile-fail doctests, and 49 downstream import controls. Linux/Windows production
trees and unsupported Vulkan diagnostics also pass their controls. Linux Rust
compilation, Windows executable tests, MSVC header ABI checks and native AMD/GPU
execution were not performed. The PowerShell CI script was extended but could
not run locally; remote success is not claimed.

Before native Linux implementation, choose whether the requirement is portable
configuration, Windows compatibility-layer use, or native Vulkan execution.
Select and pin actual upstream artefacts and technology coverage for the last
case. An older integration must not silently change D002 or claim modern-provider
equivalence. Preserve applicable third-party notices and review any new native
distribution separately.
