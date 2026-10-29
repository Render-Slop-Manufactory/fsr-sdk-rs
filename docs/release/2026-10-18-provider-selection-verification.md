# Provider selection and shared routing implementation checks

- Date: 2026-10-18.
- Scope: accepted [D028](../adr/D028.md), R04 and the U02 foundation, development
  `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`. SDK pin stays v2.3.0 at
  `60f4ea81909200d8542eca14dccb2628b763a9a3`; upscaler API version stays 4.1.1.
- Host: `aarch64-apple-darwin`; Rust 1.98.1 (`48a229cea`, LLVM 22.1.8),
  Cargo 1.98.1 (`797e8a9bc`). Windows Rust target installed; no Windows/MSVC
  runtime, SDK DLL or GPU execution was performed here.
- Result: implementation, host tests, Windows-target Rust compilation/Clippy and
  target Rustdoc pass. Rust 1.88.0 host/MSRV and Windows-target checks also pass
  (see follow-up below). Windows controlled execution, independent MSVC C++ ABI
  and native GPU confirmation remain pending. No release archives
  were created or verified, and nothing was published.

## Implemented behaviour

Raw `ffxOverrideVersion` and tag 5 have [paired Rust](../../crates/fsr-sdk-sys/tests/query_abi.rs) /
[C++](../../crates/fsr-sdk-sys/tests/query_native_abi.cpp) assertions for size 24,
alignment 8, header offset 0 and `u64` ID offset 16 on x64. The Rust assertions
compile for the Windows target and run on the 64-bit host. The C++ assertions
were added to the existing pinned-header query probe but have not run here.

Portable options expose an optional opaque provider ID. Explicit creation and
pre-create operations require membership in fresh enumeration on their actual
runtime/device, without a default fallback. Existing default calls preserve
shape and selection behaviour. Four additive selected-provider utilities and a
default/explicit pre-create resource-requirements method exist on both runtime
families; memory estimation reads selection from options. Pure numerical checks
run before selection enumeration. Informational resource masks preserve unknown
and overlapping bits without applying the supported-context profile filter.

Creation retains its optional override node alongside the API-version/backend
chain. Explicit creation validates effective live provider identity before
requirements; mismatch/query failure invokes normal terminal cleanup, including
exceptional dependency retention on failed destruction. Default creation adds
no identity query. Unsafe acquisition explicitly covers every actual default or
selected provider, override handling, synchronous temporary-query consumption and
coordination. Public docs and the query example require matching selection across
related pre-create queries and creation; default helpers do not inherit options.

Configure forwarding is private and unsafe at runtime-state and owner layers,
preserving the const descriptor, null/live context route and unmodified code.
It introduces no public control, callback, global mutation or new eligibility
policy. The intentionally unused production seam is annotated until the separate
typed Configure package supplies callers; controlled tests exercise it now.

## Commands and outcomes

Run from the repository root with cached dependencies; every listed Cargo command
completed successfully. No `--all-features` combination was used because Vulkan
is intentionally unsupported.

| Command | Outcome and scope |
|---|---|
| `cargo fmt --all` and final `cargo fmt --all -- --check` | Formatting; no native execution |
| `cargo test --workspace --locked --offline` | 66 tests including doctests pass on the host: 46 wrapper unit, 11 portable dimensions/options, one portable query-value, six raw ABI/tag tests and two doctests |
| `cargo test --workspace --features movable --no-default-features --locked --offline` | Host suite passes, including portable APIs and controlled Rc/Arc ownership; Windows public types are not executed on this host |
| `cargo clippy --workspace --all-targets --features movable --locked --offline -- -D warnings` | Host source/tests/examples lint clean |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline` | Default Windows APIs, tests and examples compile; no linking/execution claim |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline` | Both-family Windows compilation passes |
| `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings` | Both-family Windows source/fixture/native-test code lint clean; no Windows tests ran |
| `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc --no-default-features --features movable --locked --offline` | Windows no-backend feature boundary compiles |
| `cargo check --workspace --examples --target x86_64-pc-windows-msvc --features movable --locked --offline` | Updated query example compiles with matching explicit/default routing |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features movable --locked --offline` | Public target documentation builds without warnings |
| `cargo package --list -p fsr-sdk-sys --allow-dirty --locked --offline` and wrapper counterpart | Inventories include source/tests/examples and existing licence notices; not archive or registry-consumer verification |

The host selection test checks fresh bounded membership, default bypass, absent-ID
errors, exact enumeration errors, opaque override construction and portable option
cloning/reset. A real host-loaded controlled fixture checks const Configure
forwarding for null/global and live contexts, exact returned errors and subsequent
ordinary terminal destruction. This is not AMD runtime evidence.

The added [Windows public fixture](../../crates/fsr-sdk/src/upscaler/dx12/tests/provider_selection.rs)
exercises both ownership families: all selected
routes, every backend/device/override chain, preserved default helpers, numerical
preflight before enumeration, missing IDs without creation/fallback, unknown input
masks, native errors, identity mismatch/query failure, outer moves, retained nodes
through destroy and failed-destroy retention. It compiles but has not executed.
The existing ignored native public-query case now also exercises matching explicit
selection, utilities/memory/resources, creation and live identity for both families.
It remains selected by the existing `native-upscaler` profile and has not run here.

## Exact Rust 1.88.0 follow-up

The initial checks used current stable because Rust 1.88 was absent. On the same
host, `rustup toolchain install 1.88.0 --profile minimal --target
x86_64-pc-windows-msvc` then installed Rust 1.88.0 (`6b00bc388`, 2025-06-23,
LLVM 20.1.5), Cargo 1.88.0 (`873a06493`) and the Windows target. Python 3.14.8
ran the coordinator; the system `python3` lacks `tomllib` and does not meet its
Python 3.11+ prerequisite.

| Command | Outcome and scope |
|---|---|
| `python3.14 tools/verification/run.py run msrv --require-passed` | All 91 required cases pass: all-target checks and full host tests for default, movable, no-default and no-default+movable, downstream API/diagnostic boundaries, dependency trees and complete portable/movable examples |
| `RUSTFLAGS='-D warnings' cargo +1.88.0 check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline`, repeated with `--features movable`, `--no-default-features` and `--no-default-features --features movable` | All four configurations pass, including Windows API/tests/examples compilation; no Windows linking or execution |

The first Windows-target check compiled successfully but warned about an
unfulfilled `dead_code` expectation on the private runtime Configure forwarder.
Rust 1.88 counts its use by the owner forwarder differently from current stable.
That single runtime annotation now uses `allow(dead_code, reason = ...)`; the
owner retains its narrower expectation. The four Windows checks above pass with
warnings denied after this correction. No public API or runtime behaviour changed.

The 91-case host profile is separate from Windows MSRV execution: platform-gated
public fixtures and native consumers still need the Windows profiles below.

## Remaining confirmation

Run Windows `ci`, `msrv`, `abi` and `native-upscaler` through the existing
[verification coordinator](../../tools/verification/README.md), with trusted
pinned headers/runtime and the chosen GPU adapter. The native profile must observe
the actual provider selection and return behaviour; unsupported capabilities are
not universal support claims. Existing dispatch/cleanup regressions and final
extracted-package checks remain required at their appropriate checkpoints.

Neither host-layout assertions nor Windows Rust compilation substitutes for the
independent MSVC probe or runtime execution. R04 remains active pending that
confirmation; U02's five public Configure controls are not implemented here.
