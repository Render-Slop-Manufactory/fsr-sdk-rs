# Roadmap

This document owns current project status, priorities, and sequencing.
[SCOPE.md](SCOPE.md) defines durable goals and boundaries. Source and
manifests remain authoritative for implemented behavior. Technical choices belong
in [decisions](docs/DECISIONS.md), with supporting [research](docs/research/README.md).

## Current state

The project supports Windows/DX12 upscaler construction and a narrow public
unsafe dispatch method. M6b recorded one public-route native dispatch and
plausible GPU readback on a tested configuration; no image-quality or broad
hardware-support claim is available.

- [x] Two-crate Cargo workspace: `fsr-sdk-sys` and `fsr-sdk`.
- [x] Initial module scaffolding for ABI, loading, context, errors, queries, and upscaling.
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
ownership are implemented. M5 adds explicit public runtime acquisition and safe
checked upscaler construction, with shared loader retention and independent COM
device ownership. M6b adds the bounded unsafe public dispatch contract. M5 has
Windows-target Rust compilation, host ownership fixtures, Windows CI fixture execution, and both
opt-in native lifecycle cases passing on the tested Windows x64/MSVC machine
([M5 verification](docs/research/records/2026-09-24-exp-m5-windows-native-verification.md)).
Windows M4 fixture execution, paired
native ABI checks and the real production-owner lifecycle now pass on the tested
Windows x64/MSVC configuration
([M4 verification](docs/research/records/2026-09-22-exp-m4-windows-verification.md)).
An opt-in, experiment-only Windows/DX12
[context lifecycle probe](docs/research/records/2026-09-20-exp-context-lifecycle.md)
has successfully created, identified and destroyed a provider 4.1.1 context.
Its creation ABI is now promoted into sys, while provider-identification ABI
remains test-local. The ordinary lifecycle runner now targets the production
owner; its Windows execution succeeded with provider 4.1.1. The later
[M6a proof](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md)
established one GPU dispatch and readback through the public M5 constructor.
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
is complete for its bounded private Windows/DX12 ownership scope; M5's public
construction path has Windows fixtures and both opt-in native cases passing on
the tested configuration; [M6a](docs/MILESTONE_6.md) has a private dispatch
implementation and one submitted/fenced GPU readback from the
[recorded run](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md).
The public dispatch contract is accepted in [D011](docs/adr/D011.md); M6b
implementation and [public-route verification](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md)
are complete on one tested Windows/DX12 configuration. These milestones
are not release dates or support promises.

| Milestone | Work | Completion evidence |
|---|---|---|
| 1. Establish the native baseline | **Done:** pin SDK `v2.3.0` (D002), verify the common ABI (D003), identify the API DLL and select explicit-path loading (D004), select the handwritten-by-default policy with reviewed generated exceptions ([D006](docs/DECISIONS.md#d006--curate-abi-slices-and-permit-reviewed-generated-bindings)), and select explicit acquisition and executable-adjacent upscaler deployment for the bounded DX12 baseline (D007). **Pending:** broader safety contracts, effect runtime requirements beyond that baseline, applicable distribution terms and concrete additional binding slices. | Version-pinned research and decisions in the [API/safety synthesis](docs/research/api-and-safety.md). |
| 2. Minimal ABI and loader | **Done for Windows x64/MSVC:** common ABI and explicit-path loader with five private entry points and retained library ownership. | Paired ABI checks, Windows build, successful local AMD DLL symbol resolution, fixture-based failure/forwarding/unload checks and ownership compile-fail tests. |
| 3. Native query | **Done for Windows x64/MSVC:** one global count-only upscaler version query with its descriptor and two tags. | Opt-in local runtime test returned OK and count 2; paired ABI checks and both failed/successful deployment observations are [recorded](docs/research/records/2026-09-20-exp-version-count-query.md). |
| 4. Context ownership | **Done for the tested Windows x64/MSVC configuration:** private creation, terminal destruction, dependency retention and error conversion. | Windows fixtures, paired native ABI compilation and production-owner AMD create/query/destroy passed; [recorded inputs and limits](docs/research/records/2026-09-22-exp-m4-windows-verification.md). No dispatch or public constructor. |
| 5. Public DX12 construction | **Done for the tested Windows x64/MSVC configuration:** shared explicit runtime owner, checked dimensions, borrowed device with independent COM retention, safe upscaler constructor and terminal teardown. Query/Configure remain outside the accepted D008 construction scope. | Host ownership fixtures and Windows-target compilation passed; ordinary Windows CI fixtures passed in the [recorded run](docs/research/records/2026-09-24-exp-m5-windows-ci-verification.md). Both opt-in two-context native cases passed with [recorded inputs and limits](docs/research/records/2026-09-24-exp-m5-windows-native-verification.md). No dispatch. |
| 6. Minimal native DX12 dispatch proof | **M6a and M6b done for the tested Windows x64/MSVC configuration:** narrow dispatch ABI, private proof followed by D011's borrowed public unsafe dispatch input, validation, and poison/error contract ([M6 plan](docs/MILESTONE_6.md)). | Paired ABI checks and AMD helper parity passed. The [M6a record](docs/research/records/2026-09-24-exp-m6a-native-dispatch-proof.md) and [M6b public-route record](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md) each show OK dispatch, submission/fence and a 230,400-pixel finite changed readback with zero debug errors and no device removal on one setup. GPU-based validation remains inconclusive. |

The first native milestone is loading, resolving symbols, and querying the
runtime. It does not require image reconstruction or a complete public API.

## Deferred work

- Broader dispatch behavior: temporal reconstruction, varying exposure, other
  formats and providers need separate evidence before broadening D011's profile.
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

1. Pursue bounded temporal and broader-profile checks only when a concrete
   follow-on API requirement calls for them. The
   [GBV spike](docs/research/records/2026-09-24-exp-m6-gbv-dispatch-stall.md)
   localized the timeout to native `ffxDispatch` on the tested setup; its
   provider/driver/validation cause remains unresolved.
2. Pursue other SDK `v2.3.0` baseline questions in the research index when a
   concrete follow-on requirement calls for them.
