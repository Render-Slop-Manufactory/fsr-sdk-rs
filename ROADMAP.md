# Roadmap

This document owns current project status, priorities, and sequencing.
[SCOPE.md](SCOPE.md) defines durable goals and boundaries. Source and
manifests remain authoritative for implemented behavior. Technical choices belong
in [decisions](docs/DECISIONS.md), with supporting [research](docs/research/README.md).

## Current state

The project is scaffolding, not yet a usable FSR wrapper.

- [x] Two-crate Cargo workspace: `fsr-sdk-sys` and `fsr-sdk`.
- [x] Module placeholders for ABI, loading, context, errors, queries, and upscaling.
- [x] DX12 feature wiring and Windows loader gating; Vulkan explicitly rejects
  compilation and is not an implemented backend.
- [x] Contribution, development, decision, research, and agent-routing setup.
- [x] Consolidate durable goals in SCOPE.md and finish documentation routing.
- [x] Pin AMD FSR SDK `v2.3.0` as the initial native baseline
  ([D002](docs/DECISIONS.md#d002--pin-amd-fsr-sdk-v230-as-the-initial-native-baseline)).

The first common ABI slice is implemented: context and scalar aliases, return
constants, base descriptor headers, allocation callbacks, and the five native
function-pointer types ([D003](docs/DECISIONS.md#d003--handwrite-the-first-common-abi-slice)).
The explicit-path Windows/DX12 loader now owns the library and exposes five
unsafe forwarding methods with private function pointers
([D004](docs/DECISIONS.md#d004--load-an-explicit-dll-with-private-entry-points)).
The private M4 lifecycle owner, retained upscaler creation state and DX12 device
ownership are implemented. Public construction and upscaling dispatch remain
unavailable. Windows fixture execution, paired native ABI checks and the real
production-owner lifecycle now pass on the tested Windows x64/MSVC configuration
([M4 verification](docs/research/records/2026-09-22-exp-m4-windows-verification.md)).
An opt-in, experiment-only Windows/DX12
[context lifecycle probe](docs/research/records/2026-09-20-exp-context-lifecycle.md)
has successfully created, identified and destroyed a provider 4.1.1 context.
Its creation ABI is now promoted into sys, while provider-identification ABI
remains test-local. The ordinary lifecycle runner now targets the production
owner; its Windows execution succeeded with provider 4.1.1 and no GPU dispatch is established.
The first ABI verification is preserved in an
[experiment record and API/safety synthesis](docs/research/api-and-safety.md).
The local AMD loader DLL was loaded and all five symbols resolved.
[Loader evidence](docs/research/records/2026-09-20-exp-explicit-dll-loader.md)
covers fixture-based errors, forwarding and library release. A separate
[count-only version query](docs/research/records/2026-09-20-exp-version-count-query.md)
now succeeds without a context/device, using a locally staged loader and upscaler
DLL beside the test executable. This does not establish GPU compatibility.
The common ABI has Rust tests and a
compile-only local header check. Checks and their limits
are documented in [development](docs/DEVELOPMENT.md); passing builds do not imply
runtime support.

## Implementation sequence

Milestone 1 has established the baseline needed for the common ABI and explicit
loader; broader investigation remains open. Milestone 2 is complete for that
bounded scope. Milestone 3 has its count-only smoke test completed. Milestone 4
is complete for its bounded private Windows/DX12 ownership scope; Milestones 5–6 remain pending.
Complete a small, verifiable slice before widening the API; these milestones are not release dates or support promises.

| Milestone | Work | Completion evidence |
|---|---|---|
| 1. Establish the native baseline | **Done:** pin SDK `v2.3.0` (D002), verify the common ABI (D003), identify the API DLL and select explicit-path loading (D004), select the handwritten-by-default policy with reviewed generated exceptions ([D006](docs/DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)), and select explicit acquisition and executable-adjacent upscaler deployment for the bounded DX12 baseline (D007). **Pending:** broader safety contracts, effect runtime requirements beyond that baseline, applicable distribution terms and concrete additional binding slices. | Version-pinned research and decisions in the [API/safety synthesis](docs/research/api-and-safety.md). |
| 2. Minimal ABI and loader | **Done for Windows x64/MSVC:** common ABI and explicit-path loader with five private entry points and retained library ownership. | Paired ABI checks, Windows build, successful local AMD DLL symbol resolution, fixture-based failure/forwarding/unload checks and ownership compile-fail tests. |
| 3. Native query | **Done for Windows x64/MSVC:** one global count-only upscaler version query with its descriptor and two tags. | Opt-in local runtime test returned OK and count 2; paired ABI checks and both failed/successful deployment observations are [recorded](docs/research/records/2026-09-20-exp-version-count-query.md). |
| 4. Context ownership | **Done for the tested Windows x64/MSVC configuration:** private creation, terminal destruction, dependency retention and error conversion. | Windows fixtures, paired native ABI compilation and production-owner AMD create/query/destroy passed; [recorded inputs and limits](docs/research/records/2026-09-22-exp-m4-windows-verification.md). No dispatch or public constructor. |
| 5. Typed wrapper | Add validated descriptors, queries, and configuration as required by real use. | Documented safety contracts and meaningful success/failure tests for the exposed operations. |
| 6. Upscaling dispatch | Introduce DX12 resources and frame inputs with explicit synchronization obligations. | An end-to-end Windows GPU example with inspected output, relevant failure cases, and recorded runtime/hardware details. |

The first native milestone is loading, resolving symbols, and querying the
runtime. It does not require image reconstruction or a complete public API.

## Deferred work

- Automated SDK acquisition: decide download permission, pinned versions,
  checksums, cache behavior, offline use, and redistribution before implementing.
  This is separate from dynamic runtime loading.
- CI: keep the existing host and Windows compilation checks useful as implementation grows;
  keep hardware-dependent evidence separate from ordinary build jobs.
- Release readiness: establish compatibility documentation, package checks,
  examples, and any required third-party notices before publishing crates or
  distributable releases.
- Additional backends and broader SDK coverage: revisit only when upstream
  capabilities and a concrete use case justify them.

## Next actions

1. Investigate the accepted SDK `v2.3.0` baseline using the remaining questions
   in the research index.
2. Proceed to Milestone 5's typed-wrapper work from the verified private M4
   foundation. D005's explicit runtime trust assumption remains part of the
   failed-create cleanup contract; successful M4 verification does not settle
   the remaining native-contract research questions or establish dispatch.
