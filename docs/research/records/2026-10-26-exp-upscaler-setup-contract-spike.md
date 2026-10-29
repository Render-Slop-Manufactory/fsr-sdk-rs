# Experiment: upscaler setup contract and early frame composition

- Investigation and execution date: **2026-10-26**.
- Question: can D042's loader/effect trust split, independently owned contexts
  and portable recording assembly be expressed with real Rust/Windows types,
  while preserving production ownership, routing and validation?
- Status: **bounded spike passed; contract draft for review**. No production
  API change, new accepted safety contract or U07/U08 completion is claimed.
- Hypothesis: an unsafe effect acquisition can produce a two-lifetime setup
  whose safe planning/creation operations cannot leak from a loader-only type;
  returned contexts need no setup borrow, and portable frame data plus named
  resources can feed existing mandatory inspection and descriptor translation.
- Exclusions: no AMD SDK/provider loading, actual GPU work, independent ABI
  experiment, image comparison, benchmark, safe callback framework or full
  U08–U12 implementation. Fake COM/native success is no provider-safety proof.

## Setup and provenance

Baseline is development wrapper **0.3.0**, sys **0.1.2**, edition 2024,
minimum Rust **1.88**, pinned AMD SDK **v2.3.0**. The worktree was clean at intake.
Only experiment equipment, the draft, this record/supporting metadata and the
roadmap progress note are outputs; production crates, manifests, lockfile and
accepted D042 are unchanged.

The [contract draft](../../design/upscaler-setup.md) records exact candidate
signatures, ownership/trait choices, the loader/setup/recording/installation
obligation allocation and the per-operation failure audit. It is a proposed
realization of [D042](../../adr/D042.md), not automatic acceptance of a new
contract. Both source reviews remain inputs with their original limits.

Execution environment:

| Input | Observed value |
|---|---|
| Host | Windows 11, build 10.0.26300, `x86_64-pc-windows-msvc` |
| Current Rust | `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 23.1.1 |
| Current Cargo | `cargo 1.99.0 (5f94df478 2026-08-27)` |
| MSRV Rust | `rustc 1.88.0 (6b00bc388 2025-06-23)`, LLVM 20.1.5 |
| MSRV Cargo | `cargo 1.88.0 (873a06493 2025-05-10)` |
| Dependencies | Existing cached workspace dependencies, notably `windows 0.62.2` and `libloading 0.9.0`; no additions/downloads |
| Native equipment | Existing Rust-built controlled fixture DLL and fake COM device/list/resource implementations; no GPU/driver observation |

The retained [input inventory](2026-10-26-exp-upscaler-setup-contract-spike/inputs.json)
identifies copied crate inputs, root manifests/lockfile, D042 and all four probe
producer files by path, byte length and SHA-256. The
[process results](2026-10-26-exp-upscaler-setup-contract-spike/results.json)
preserve commands, actual exits, expected compiler diagnostics and evaluation;
[summary](2026-10-26-exp-upscaler-setup-contract-spike/summary.json) records counts
and toolchain identities. Neither Git history nor a retained full source copy is
needed to understand the results. Reproduction requires the recorded baseline
inputs; the generator rejects mismatched patch anchors rather than silently
adapting to a different layout.

## Probe construction and source reasoning

Retained producers live in
[`tools/experiments/upscaler_setup`](../../../tools/experiments/upscaler_setup/README.md).
The coordinator materializes an isolated workspace under ignored `target/`,
copies the existing crates and applies small explicit experiment-only injections.
No other checkout is changed. Generated source snapshots, compiled binaries and
verbose logs remain temporary there; retained evidence contains identities and
compact results. No transient runtime address is needed or retained. Personal
checkout paths in logs are replaced with `<repo>`; commands in retained results
contain no personal path.

The experimental Runtime holds the actual `RuntimeState` but exposes only
unsafe loading and unsafe setup acquisition. It cannot be passed to the old
safe constructor. Setup privately creates short-lived baseline-runtime bridges
only after acquisition establishes effect trust. Those bridges preserve the
existing query routing, fresh membership and exact creation path. They are
experimental reuse scaffolding, not the final optimized implementation or a
second production facade.

Created experimental contexts own actual local/movable production upscalers.
Consequently, the real pinned descriptor/dependency bundle, independent COM
device reference, retained runtime and terminal destruction mechanisms are
exercised. The portable sharpening definition is relocated inside the isolated
candidate, retaining type identity through the existing DX12 reexport. Frame
and binding adapters assemble a private baseline dispatch for every call so the
production inspection and native descriptor mapping still execute.

**Source finding:** if `with_provider` remains safe after acquisition, the setup
trust cannot cover only its initial default or one preferred ID. It must cover
providers and input domains reachable through later safe operations, including
default changes and safe rebinding. Otherwise the new API would merely move an
unexpressed safety precondition onto a safe setter. The draft makes this cost
explicit; compiler tests do not establish native compliance with that assumption.

**Source finding:** movable loader release compatibility must be established
before the loaded runtime is available to safe `Send` code. It cannot be deferred
to a later setup. Effect operation compatibility can be acquired with the setup
and persists in the derived owners. Both stages retain common-domain coordination.

The source audit covers ordinary and callback-bearing creation, post-create
query failure cleanup, all seven live queries, both recording routes, preflight,
Configure, global/live diagnostics, consuming destroy, Drop/unwind and exceptional
retention. It uses existing D005/D008/D013/D018/D022/D028/D029/D030/D040/D041 and
the actual owner/routing code; it is not an independent proof of their native
assumptions. The draft's table distinguishes enforced wrapper facts and the
specific earlier/explicit unsafe boundary discharging each obligation.

## Reproduction and observable criteria

From the recorded checkout, with the tools and cached dependencies above:

```sh
python tools/experiments/upscaler_setup/run.py
```

The default runs both toolchains. `--toolchain stable` and
`--toolchain 1.88.0` run one; `--prepare-only` creates a candidate without checking
it. No tool is installed and no network is used. The coordinator invokes:

```sh
cargo generate-lockfile --offline
rustc +stable -Vv
cargo +stable check -p fsr-sdk --no-default-features --offline --locked
cargo +stable check -p fsr-sdk --no-default-features --offline --locked --features dx12
cargo +stable check -p fsr-sdk --no-default-features --offline --locked --features dx12,movable
cargo +stable test -p fsr-sdk --lib --features movable --offline --locked setup_spike -- --nocapture
cargo +stable test -p fsr-sdk --lib --no-default-features --offline --locked setup_spike
cargo +stable clippy -p fsr-sdk --lib --features movable --offline --locked -- -D warnings
```

It repeats the build/test commands with `+1.88.0`; strict Clippy runs on stable.
Generated dev/test profiles use `debug = 0` and `incremental = false` to bound
temporary disk use, without altering production profiles. For each immutable
consumer target it executes `cargo +TOOLCHAIN check -p setup-spike-consumer
--bin NAME --offline --locked --message-format=json`, adding
`--features dx12,movable` for native-surface consumers and no feature for the
portable one. The complete exact command list is in retained results.

Success requires all positive consumers and selected Rust tests to pass, and
every negative consumer to fail with its specific expected diagnostic attributed
to that consumer target. A dependency error, unrelated failure or arbitrary
nonzero exit cannot satisfy a negative control. Native counters, copied
descriptor bits, COM reference counts and weak runtime-state witnesses supply
observable assertions beyond compilation.

## Observations

The final frozen confirmation produced **64/64 passing process cases**, coordinator
exit **0**. All positive Cargo checks/tests exited 0; all forty expected negative
consumer compilations exited **101** with the matching diagnostic. The two
toolchain identity cases and lock generation also exited 0; they identify inputs
rather than establishing behavioural guarantees. Strict stable Clippy also
passes with `-D warnings`.

| Check, on each of Rust 1.99 and 1.88 | Observed result |
|---|---|
| Library configurations: no-default, DX12, DX12 + movable | 3/3 compiled |
| Selected DX12 + movable Rust tests | 17 passed, 0 failed, 0 ignored; 178 unrelated tests filtered out |
| Separate no-default portable parameter test | 1 passed, 0 failed; 58 unrelated tests filtered out |
| External successful consumers | 5/5 compiled: portable values, simple creation, selected planning/create, frame recording/abandonment, movable trait assertions |
| External expected compiler failures | 20/20 rejected as specified |

The simple, planning and recording consumers each contain local and movable
workflows. Their bodies compile against actual Windows resource/list/device
types and the candidate. They are not executed with an AMD DLL or GPU resources.

Controlled tests verify:

- **Lifetime and ownership:** two independently created contexts survive setup,
  public-runtime and original-device loss. Weak `RuntimeState` witnesses stay
  live through the first context's destruction and expire only after the last;
  COM counts match. The observer also holds its own DLL reference, so this
  establishes wrapper state retention rather than independently observing final
  operating-system DLL unload.
- **Loading without effect entry and unwinding:** loading/dropping the candidate
  Runtime makes zero FFX create/query/destroy calls. Dropping a context during an
  intentionally caught Rust panic attempts native destruction once and releases
  its retained state under the controlled no-GPU conditions.
- **Selection and ordering:** ratio, estimate and creation retain the requested
  ID; fresh count/fill enumeration occurs. A later unavailable ID is rejected,
  invalid jitter index is rejected before enumeration and a live identity
  mismatch triggers ordinary cleanup. There is no default fallback.
- **Recording assembly:** metadata changes after successful preflight are still
  rejected by canonical recording without poisoning or leaking COM references.
  Captured descriptors preserve negative-zero/subnormal native scale bits, then
  correctly replace them with pixel `(1, 1)` and active UV dimensions. The initial
  reset request can be cleared for the next frame.
- **Resource binding:** missing colour and missing exposure are pure assembly
  errors; explicit automatic exposure assembles successfully. This is not a
  claim that automatic mode matches every receiving context or provider.
- **Terminal state:** native recording error preserves its operation/code as
  the first cause; later abandonment cannot overwrite it. Reported abandonment
  blocks live queries, recording and context-dependent preflight. Configure
  failure is distinguished from recording failure. No operation waits or revives
  a context.
- **Failure retention:** ordinary create error, OK/null creation and destroy
  failure preserve the production device/cleanup policy. A message-bearing create
  failure retains the wrapper bundle and performs no destroy. The static no-op
  message target is not evidence of general callback shutdown or heap invocation.
- **Movable cleanup:** a real owned candidate moves into a joined Rust thread
  and destroys there under serialized fixture/COM access. This is one controlled
  transfer, not arbitrary-thread AMD compatibility.

Expected compilation rejections, on both toolchains:

| Cases | Required Rust diagnostic |
|---|---|
| Unsafe load omitted; local/movable unsafe setup omitted; unsafe callback create omitted | `E0133` |
| Planning on a merely loaded runtime; provider setter on provider-free config | `E0599` |
| Passing the loaded candidate Runtime to the old constructor | `E0308` |
| Accessing private retained runtime state | `E0616` |
| Two overlapping live setup views from one runtime | `E0499` |
| Escaping resource borrows or a setup's local runtime | `E0515` |
| Local runtime/context/setup as Send or Sync; movable runtime/context/setup as Sync | `E0277` |

## Unsuccessful preparation and preliminary runs

The first materialization stopped before compilation because its strict
workspace patch expected four-space indentation while the baseline manifest
uses two. It exited 1 with `Baseline anchor changed`; no runtime test or native
behaviour failed. The anchor was corrected to the actual baseline.

A preliminary stable-only candidate passed twelve selected tests and its
then-current consumers. A subsequent two-toolchain run passed fifteen selected
tests and the expanded consumer inventory; a redundant test import emitted one
warning and was removed. Lifetime assertions were then strengthened with weak
runtime-state witnesses and loader-only/unwind checks were added. The final
recorded confirmation passes seventeen selected tests without that warning.
Earlier temporary directories/logs remain under ignored `target/`; their bytes
are not substituted for the final input identities/results retained here.

An additional strict Clippy check then rejected one redundant movable bridge
closure. It exited 1; the closure was replaced with the associated function
item. The coordinator now includes this strict check, and the full final
confirmation was repeated against the corrected producer identities.

That repeat initially stopped during stable test compilation with LLVM's
`no space on device` and Windows error 112. The coordinator exited 1; the
controlled/portable tests did not execute in that attempt, and its subsequent
Clippy attempt also lacked output space. Its
[interrupted process results](2026-10-26-exp-upscaler-setup-contract-spike/interrupted-results.json)
are retained separately from the passing final results. No AMD operation was involved. Only
verified build directories generated by this spike were removed, preserving
source identities, logs and results. Candidate debug/incremental output was
disabled before the final full repeat. This environment failure is distinct
from the earlier passing functional checks and the final confirmation.

## Baseline delta, limits and recommendation

**Verified within this experiment:** the proposed boundaries are expressible
with actual types and independent setup lifetimes on stable and MSRV; the
experimental loaded Runtime has no safe effect escape; contexts retain real
production ownership; selected routing, assembly, validation and terminal
behaviour pass controlled execution. The three ordinary consumer workflows
compile, including both ownership families.

**Inference/recommendation:** the candidate is a viable concrete U07 draft and
early U08 starting point. Retain it for review and migrate through shared private
mechanics when implementing the accepted target. Native trust must remain as
explicit as the draft states; a shorter load contract cannot leave later safe
calls without established obligations.

**Unresolved:** native capacity/name validity, non-retention, numerical/access
domains, common-state/thread compatibility, callback quiescence, returned-error
cleanup and termination are not newly established for AMD providers. Compiler
and fixture success do not certify any of them. No independent ABI, GPU result,
performance/allocation, native Linux/Vulkan, final package or release proof is
claimed. The experimental bridges add temporary retention clones; their cost
is not measured. The feature-selected library unit-test harness was compiled;
only the reported filtered tests were executed. Existing integration and native
GPU suites were not run.

U09 checked tuning, U10 error/metadata restructuring and the complete U08
generator builder are outside this probe. Its small first-cause/abandonment guard
tests the third workflow without completing U11. Public diagnostics compile and
their safety edges are audited, but global registration/heap callbacks are not
re-exercised as a native experiment. Production Rustdoc, removal of old flat
routes, current docs/examples and appropriate existing native/package regressions
remain implementation work. U07 and U08 must not be marked Verified from this
bounded record alone.
