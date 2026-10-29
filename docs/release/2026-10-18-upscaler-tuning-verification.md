# Typed upscaler Configure implementation checks

- Date: 2026-10-18.
- Scope: accepted [D029](../adr/D029.md), U02's five typed context controls,
  development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.1.2`. The maintainer explicitly
  authorized implementation after initially deferring it.
- SDK: v2.3.0 at `60f4ea81909200d8542eca14dccb2628b763a9a3`; upscaler API
  version remains 4.1.1. No dependency, SDK pin or native target change.
- Host: `aarch64-apple-darwin`; current Rust 1.98.1 (`48a229cea`, LLVM 22.1.8),
  Cargo 1.98.1 (`797e8a9bc`); MSRV Rust 1.88.0 (`6b00bc388`, LLVM 20.1.5),
  Cargo 1.88.0 (`873a06493`); Python 3.14.8. Both compilers have the
  `x86_64-pc-windows-msvc` standard library installed.
- Result: implementation and local checks pass, including 106 required CI cases,
  95 required MSRV cases, Windows-target compilation on both compilers and
  downstream API/example compilation. Windows fixture execution, independent
  MSVC C++ ABI and AMD/native GPU confirmation remain deferred by the maintainer
  because no Windows host is available here. No final release archives were
  verified or published.

## Implemented behaviour and boundaries

Raw bindings add `ffxConfigureDescUpscaleKeyValue`, tag `0x10007` and all five
`FFX_API_CONFIGURE_UPSCALE_KEY_*` constants from the pinned `ffx_upscale.h`.
Paired [Rust](../../crates/fsr-sdk-sys/tests/query_abi.rs) and
[independent C++](../../crates/fsr-sdk-sys/tests/query_native_abi.cpp) assertions
cover tag/keys, exact field types and x64 size 40, alignment 8 and offsets
0/16/24/32. Rust assertions execute on the host and compile for Windows; the
MSVC probe has not executed for this addition.

Portable non-exhaustive `TuningParameter` and `TuningValue` expose all five keys,
one value or explicit provider-default request at a time. Both runtime ownership
families share `configure_tuning`, descriptor mapping and eligibility policy.
Construction makes no implicit Configure call. Every finite float is passed
unchanged, including negative zero and out-of-range values for native clamping;
NaN/infinity is rejected before native entry. Provider-default reset passes null,
without substituting rounded header defaults. Temporary descriptors/float storage
remain live through synchronous invocation, without a provider/backend chain.

Success is acknowledgement, not support detection or effective-value readback.
There is no cached state, automatic history reset, reapplication, provider-name
heuristic or batch rollback. Actual-provider acquisition trust now explicitly
covers all finite/reset processing, synchronous non-retention on both outcomes,
no deferred GPU work, profile/resource compatibility and terminal destroyability
after errors. Unsafe dispatch's lasting contract requires completion of earlier
submitted work and no executable unsubmitted recording before safe Configure.
Neither a CPU borrow nor a thread handoff establishes completion. These are
caller-established contracts, not newly proven AMD guarantees.

The shared private context eligibility enum replaces the dispatch-only boolean.
Native errors preserve `Operation::Configure` and the exact returned code;
subsequent dispatch, all seven live queries and Configure reject with
`ConfigurePoisoned` before inspection, validation or FFI. Existing dispatch
failure still gives `DispatchPoisoned`, including from Configure. Validation
and ordinary query errors do not poison a usable context; neither reset nor a
query clears poison. Terminal ownership/cleanup and failed-destroy dependency
retention are preserved. Configure receives no list to discard.

## Commands and outcomes

Run from the repository root using cached dependencies. Offline is the
coordinator default. No intentionally unsupported `--all-features` check ran.

| Command | Outcome and scope |
|---|---|
| `python3.14 tools/verification/run.py run ci --require-passed` | Final attempt `20261018T172212Z-ci-e9c5a49be3db`: **106/106 required cases pass**, inputs unchanged during execution. Includes formatting, all-target checks/tests/Clippy/docs in all four feature configurations, consumer boundaries, dependency trees, complete examples and infrastructure/archive integrity |
| `python3.14 tools/verification/run.py run msrv --require-passed` | Attempt `20261018T172048Z-msrv-63bb51ca6e72`: **95/95 required cases pass**, inputs unchanged during execution. Exact Rust 1.88.0 all-target checks/full host tests in all four configurations, consumer diagnostics and complete portable/movable examples |
| `cargo test --workspace --locked --offline` | **71 tests including doctests pass**: 51 wrapper unit, 11 portable options/dimensions, one portable query-value, six raw ABI/tag tests and two doctests. CI/MSRV also repeat full tests for movable, no-default and no-default+movable |
| `RUSTFLAGS='-D warnings' cargo +stable check --workspace --all-targets --target x86_64-pc-windows-msvc --locked --offline` and `+1.88.0` counterpart | Each repeated with default, `--features movable`, `--no-default-features` and `--no-default-features --features movable`: all eight compile without warnings, including platform-gated tests/examples. No Windows linking/execution |
| `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --features movable --locked --offline -- -D warnings` | Windows public APIs, controlled fixtures and existing native-test code lint clean |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --target x86_64-pc-windows-msvc --features movable --locked --offline` | Windows-target public documentation builds without warnings |
| Separate downstream consumer checks described below | Both public methods and all complete examples compile for the Windows target on stable and Rust 1.88.0; no native execution |
| `cargo package --list -p fsr-sdk-sys --allow-dirty --locked --offline` and wrapper counterpart | 31/102 inventory entries, including both licence notices and new policy/tuning/test files; no ignored SDK/build material. Inventory only, not final archive verification |

The first CI attempt, `20261018T172047Z-ci-8f07d84c75df`, failed only its
infrastructure case: six watchdog tests could not bind their local loopback
socket under the sandbox (`PermissionError: [Errno 1] Operation not permitted`).
All other 105 required cases passed. The authorized repetition of the same
command outside the sandbox passed. Its infrastructure suite ran 232 tests,
with five explicit skips: three PowerShell-parser checks and two Linux-only
process-oracle checks. This is not a passed PowerShell or Linux-process verdict.

The new checked-in [consumer cases](../../tools/verification/consumer-cases.json)
cover portable aliases/traits, both enums' non-exhaustiveness, preserved error
traits and local/movable Configure signatures. The host profiles execute the
four portable cases. To additionally compile the two Windows-specific cases
without executing Windows, a separate disposable package was prepared at
`target/tuning-windows-consumer` with path dependency on `crates/fsr-sdk`, default
features disabled and these forwarding features:

```toml
[features]
dx12 = ["fsr-sdk/dx12"]
movable = ["fsr-sdk/movable"]
```

Its `src/bin` files copy the `tuning-local-positive` and `tuning-movable-positive`
sources verbatim from that consumer inventory; its `examples` copy all checked-in
wrapper examples. After `cargo generate-lockfile --offline` for that manifest,
the following ran with both `+stable` and `+1.88.0`:

```sh
cargo +1.88.0 check --manifest-path target/tuning-windows-consumer/Cargo.toml --target x86_64-pc-windows-msvc --features dx12 --examples --bin tuning-local-positive --locked --offline
cargo +1.88.0 check --manifest-path target/tuning-windows-consumer/Cargo.toml --target x86_64-pc-windows-msvc --features dx12,movable --examples --bin tuning-local-positive --bin tuning-movable-positive --locked --offline
```

Compile-only consumer functions emitted ordinary unused-function warnings;
the workspace Windows checks above separately deny warnings. The
[renderer example](../../crates/fsr-sdk/examples/dx12_upscale.rs) delegates actual
submit/fence waiting to a renderer-supplied closure before safe Configure and
describes the next recording. The [movable example](../../crates/fsr-sdk/examples/movable_upscale.rs)
keeps configuration inside its common native coordination domain.

## Controlled evidence and remaining native work

[Host tuning tests](../../crates/fsr-sdk/src/upscaler/tuning/tests.rs) inspect all
five descriptor mappings, 100 value/reset invocations, unchanged finite bits,
non-finite preflight, acknowledgement without application, arbitrary returned
codes, both first-failure origins, every live-query poison gate and preflight
priority. Ordinary query errors leave Configure/dispatch available.

A [loaded lifecycle fixture test](../../crates/fsr-sdk/src/context/tests.rs)
uses the production descriptor helper, runtime forwarding and real context owner
on this host. It observes scalar/null requests through the retained handle after
an outer move, exact Configure errors and no calls after poison, then explicit
or implicit terminal teardown. Both successful destruction and failed-destroy
retention are covered. Its private protocol's `f32::MAX` rejection is deliberate
test behaviour, not an AMD-domain rule.

[Windows public fixtures](../../crates/fsr-sdk/src/upscaler/dx12/tests/tuning.rs)
compile for both ownership families. They inspect live handles, all keys,
float/null requests, unchanged construction, query-error continuation, moves,
both poison origins, all public query gates, dispatch preflight suppression,
terminal destruction and failed-destroy dependency retention. They have not run
on Windows here.

Independent MSVC assertions, Windows fixture execution and bounded actual-provider
value/reset → dispatch → cleanup observations remain unrun. Actual provider
application, persistence, clamping and default semantics are not inferred from
the controlled fixture or an OK code. Final extracted-package checks and release
verification remain separate. Nothing was committed, uploaded or published by
this implementation task.
