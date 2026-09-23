# Project Scope

This document defines durable goals, boundaries, and engineering principles.
[ROADMAP.md](ROADMAP.md) owns status and sequencing;
[decisions](docs/DECISIONS.md) records consequential technical choices.
Source and Cargo manifests establish implemented behavior. Goals here are not
claims of completed functionality or verified platform support.

## Purpose

Provide faithful Rust bindings and safe abstractions for AMD's official modern
FidelityFX Super Resolution SDK and runtime. Make ordinary use straightforward:
most consumers should need only `fsr-sdk`, without resolving native symbols or
reproducing descriptor and lifetime boilerplate themselves.

The project wraps the official SDK; it does not reimplement FSR. It is independent
of any game engine or renderer. Name and version the Rust crates independently
of particular FSR algorithm generations.

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

Correctness and a clear correspondence with AMD's API take priority over an
elaborate facade. Implement only the API surface justified by real use.

## Platform and compatibility boundaries

The initial integration target is Windows with the official DX12 runtime. Isolate
platform-specific code and dependencies. Additional backends require actual
upstream capabilities, implementation, and evidence; feature names alone do not
establish support. Avoid speculative cross-backend abstractions.

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
and explicit permission behavior. Support local artefacts without requiring
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
