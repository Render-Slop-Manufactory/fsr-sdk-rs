# Project Scope

This document defines durable goals, boundaries, and engineering principles.
[ROADMAP.md](ROADMAP.md) owns status and sequencing;
[decisions](docs/DECISIONS.md) records consequential technical choices.
Source and Cargo manifests establish implemented behaviour. Goals here are not
claims of completed functionality or verified platform support.

## Purpose

Provide faithful Rust bindings and safe abstractions for AMD's official modern
FSR SDK and runtime. The long-term goal is complete SDK coverage: Upscaling,
Frame Generation (including presentation), Ray Regeneration, Radiance Caching,
and the associated runtime, Query and Configure interfaces. Extend backend
coverage where upstream provides a usable, verifiable integration. This is the
project's intended scope; the current implementation is a selected subset,
tracked in [ROADMAP.md](ROADMAP.md).

Make ordinary use straightforward:
most consumers should need only `fsr-sdk`, without resolving native symbols or
reproducing descriptor and lifetime boilerplate themselves.

The project wraps the official SDK; it does not reimplement FSR. It is independent
of any game engine or renderer. Name and version the Rust crates independently
of particular FSR algorithm generations.

### Verification priority

The priority through 1.0.0 is faithful, usable access to AMD's SDK. By explicit
maintainer direction on 2026-10-18, image-quality research is outside that scope
and deferred until after 1.0.0. This supersedes the 2026-10-14 hold and covers
temporal quality studies, sharpening-effect criteria, provider quality comparisons,
custom quality presets and policy calibration. Resumption after 1.0.0 requires
explicit maintainer selection; it is not automatic.

Keep these topics out of the active roadmap, release prerequisites and routine
status or remaining-work summaries. The [deferred backlog](docs/POST_1_0.md)
preserves their identifiers and evidence for a future explicit review.

Continue checking the wrapper's ABI, parameter and resource mapping, input
validation, ownership, error handling, GPU execution and cleanup. Feature and
release acceptance require evidence for those integration contracts; they do
not require proving AMD's reconstruction quality or resolving deferred image
criteria. Existing measurements, failures and limitations remain evidence.
Investigate concrete wrapper correctness or safety problems when they arise.

By maintainer direction on 2026-10-27, every release also requires fresh,
reproducible performance verification and review of regressions on its candidate.
[D043](docs/adr/D043.md) defines the evidence and release boundary. Coverage and
comparison criteria follow the affected paths; this adds no image-quality work
or automatic universal speed threshold.

## Crate responsibilities

**`fsr-sdk-sys`** represents AMD's public C ABI with minimal policy: layouts,
constants, opaque handles, descriptors, function pointers, platform definitions,
and runtime symbol loading. Preserve upstream contracts and naming where needed
for fidelity. Direct use is expected to involve unsafe operations.

**`fsr-sdk`** provides idiomatic ownership, RAII context cleanup, structured errors,
validated descriptor construction, queries, configuration, and typed dispatch
interfaces. Preserve native error information. Expose safe operations only where
the wrapper can uphold their contracts; document caller obligations at unsafe
native-resource boundaries.

The caller's renderer supplies GPU resources, image content, motion vectors,
camera and timing values. The first DX12 dispatch boundary also
leaves command-list submission and GPU synchronization to the caller. The
wrapper may validate metadata it can inspect and document the remaining
obligations, but it does not own the rendering pipeline.

Correctness and a clear correspondence with AMD's API take priority over an
elaborate facade. Implement only the API surface justified by real use.
The separation of supported direct provider controls and explicit SDK quality
policies is governed by [D016](docs/adr/D016.md).

### Core and optional utilities

[D033](docs/adr/D033.md) classifies functionality by who defines its behaviour.
Core exposes native SDK operations and provider-reported results, with checked
Rust representations, necessary unit/descriptor mappings, ownership and safety
contracts. Alternate constructors, accessors and dispatch preflight remain core
when they express the same native contract. AMD's quality/size/jitter queries,
automatic exposure and native tuning are core, regardless of helper-like names.
Documented defaults define the supported profile; they are not quality recommendations.

An independently specified algorithm, selection rule, smoothing strategy or
signal conversion is a candidate for future opt-in `fsr_sdk::utils` functionality
within `fsr-sdk`. It must use the public core API, preserve direct native controls
and core defaults, and make its assumptions and effects explicit. Prefer returning
existing checked core inputs for caller application; core stays independent of
policy provenance. Preserve creation, frame, query, Configure and GPU-operation
boundaries. Policy-specific core extensions require a demonstrated missing
capability; future effects do not justify a generic policy interface in advance. Add the
off-by-default feature/module only with a concrete, separately selected utility.
The [boundary/composability synthesis](docs/research/core-and-utilities.md) identifies
no required extraction or core refactor. [D036](docs/adr/D036.md) selects the first
concrete utility: an off-by-default, portable provider-name resolver returning
the existing core provider selection for caller application. The maintainer
selected its release inclusion in 0.3.0 on 2026-10-27, retaining opt-in activation.
It adds no compatibility
guarantee or native fallback. Other utilities require separate selection, and
quality policies remain deferred under the
[verification priority](#verification-priority).

The renderer/integration layer owns camera projection, renderer GPU allocations,
GPU texture conversions, render graphs, resource transitions, queue submission,
fences, temporal histories and the application's presentation loop. A generic
utility must not silently take over these responsibilities or erase unsafe
dispatch/coordination obligations. Core still owns its native SDK objects and
their contracts; wrapping native Frame Generation/presentation capabilities
remains part of the long-term SDK-coverage goal.

### Public API surface

`fsr-sdk` owns its public configuration and error types, translating native
failures while preserving codes where supplied and underlying causes where
available. Ordinary safe APIs must not expose `fsr-sdk-sys` ABI types, raw
pointers or handles, descriptor chains, or function pointers. Keep those details
behind the wrapper's validated types and ownership rules.

Platform resource types that callers already own may be deliberate public
inputs. The DX12 `ID3D12Device` is one such choice; its `windows` dependency
version is part of the compatibility review ([D008](docs/adr/D008.md)). A future
low-level or unsafe escape hatch may expose native types when its caller
obligations and public compatibility costs are explicit and recorded in a
[decision](docs/DECISIONS.md). The owned load-error boundary is recorded in
[D010](docs/adr/D010.md).

The current upscaler surface separates common checked values from the DX12
backend, with existing imports retained as reexports ([D015](docs/adr/D015.md)).
Common checked values are available independently of the host and backend
features; native runtime and DX12 APIs retain their Windows/DX12 gates.
This organization does not add native targets or another backend.

## Platform and compatibility boundaries

This section is authoritative for native target support. The supported native
target is `x86_64-pc-windows-msvc` with the `dx12` feature and the official DX12
runtime. Windows/DX12 APIs are exposed on other Windows targets, but those
targets are currently unverified and unsupported; compilation may succeed.
Native verification is limited to the SDK, runtime/provider, GPU, driver and
workload configurations recorded in the [verification records](docs/research/README.md),
including the [M6b public dispatch record](docs/research/records/2026-09-25-m6b-public-dispatch-verification.md).
Target support does not imply compatibility with every hardware/provider
configuration.

Separate [Proton experiments](docs/research/platform-and-backends.md#observed-proton-execution)
verify Linux-hosted Windows cross-build, public upscaler queries and one shared
GPU workload on Steam Deck and Steam Machine. These are bounded compatibility-layer
observations; they do not add native Linux/Vulkan support or complete temporal
regression coverage. [D019](docs/adr/D019.md) accepts reproducible Linux-hosted
Proton verification as a project objective; its initial bounded proof is complete.
Selected temporal regressions have run on both hosts, with failed or blocked
criteria preserved in historical evidence. Bounded local-runtime
import/offline execution and detached process timeout cleanup are verified on the
recorded Deck stack. These separate
[roadmap checkpoints](ROADMAP.md#verification-and-compatibility) do not establish
full regression portability, remote runtime acquisition or a pinned host closure.

Vulkan is an explicit required future support target, not an abandoned backend.
The pinned SDK v2.3.0 path does not provide a usable Vulkan integration.
The SDK 2.4 source interface is identified, but adoption, runtime artefact/ABI
review and native verification remain separate prerequisites for implementation.
The reserved `vulkan` Cargo feature deliberately fails to compile until a usable
backend is implemented; retaining the feature does not establish native support.

Windows and DX12 code, native types, loaders and dependencies **must** remain
encapsulated behind their platform/backend boundaries. Common APIs and future
Vulkan consumers must not require Windows/DX12 dependencies. Operating-system
selection and graphics-backend selection are separate concerns; Vulkan must not
be treated as synonymous with Linux. Keep portable APIs free of DX12 types and
verify target/feature dependency graphs and unavailable native APIs. Additional
backends require actual upstream capabilities, implementation and evidence;
feature names alone do not establish support. Avoid speculative cross-backend
abstractions. [D031](docs/adr/D031.md) records the maintainer's explicit requirement.

Pin a specific SDK release before establishing the native boundary. Keep Rust
crate versions, SDK versions, and individual technology/runtime versions distinct.
Document tested compatibility explicitly as coverage grows rather than implying
compatibility with every SDK release or hardware configuration.

Engine integration, shader reimplementations, and compatibility with prior Rust
FSR projects' APIs are not requirements. Prior art can inform the design without
becoming a contract this project must preserve.

## Native safety and runtime ownership

Dynamically load the AMD runtime and retain library handles for as long as any
resolved functions or SDK objects depend on them. Report missing runtimes,
symbols, and native failures with useful errors.

Establish invariants for ABI layout, alignment, integer widths, calling
conventions, descriptor tags and chains, pointer validity, resource ownership,
and thread safety. A Rust lifetime alone does not establish GPU completion or
correct resource states. Do not hide synchronization obligations behind an
unjustified safe interface.

Use explicit ownership and automatic cleanup where justified by the SDK contract.
Detailed coding and verification guidance belongs in
[code heuristics](docs/CODE_HEURISTICS.md) and
[development](docs/DEVELOPMENT.md).

## Setup and distribution goals

Minimize downstream setup while preserving reproducible and offline build paths.
Runtime loading and acquiring SDK artefacts are separate concerns. The acquisition
mechanism is a design choice, not a requirement to download during Cargo builds.
If downloads are introduced, use pinned versions, cryptographic verification,
and explicit permission behaviour. Support local artefacts without requiring
network access. Keep generated and downloaded build artefacts in Cargo-controlled
or explicitly configured locations rather than arbitrary source-tree paths.

The project's own work uses [MPL-2.0](LICENSE). AMD and other third-party materials
retain their applicable terms. Establish provenance and required redistribution
notices before shipping external artefacts; the project's licence does not
relicense them. Contribution rules live in [CONTRIBUTING.md](CONTRIBUTING.md).

## Evidence and design discipline

Prefer small, verifiable increments. Source research, host compilation, native
loading, successful queries, correct GPU output, and performance are distinct
forms of evidence. Do not substitute one for another or claim support without
appropriate verification.

Keep consequential decisions traceable to their rationale and evidence. Research
uses the [research process](docs/research/PROCESS.md); recommendations do not
silently change scope or accepted decisions. Update this document when the
project's durable boundaries change, not for routine implementation progress.
