# Callback and diagnostic implementation verification

- Date: 2026-10-25.
- Scope: accepted [D040](../adr/D040.md), R03/U04, wrapper 0.3.0 / sys 0.1.2,
  pinned SDK v2.3.0, Windows x64/MSVC + DX12, both public ownership families.
- Result: implemented and verified within the recorded configuration; R03/U04 complete
  for bounded SDK access.

## Implemented contract

Both public families expose unsafe `new_with_callbacks`, paired host/resource/heap
allocation tables, CB allocation and native UTF-16 messages through
`upscaler::dx12::interop`. Portable debug checking/visualization settings and
`DebugLevel` are separate from the independent frame debug-view request.
Memory estimates and construction use the same option flags. Modern global
upscaling diagnostics and deprecated null/live-context routes preserve exact
native codes without fallback. Live-context Configure failure follows existing
poison precedence; runtime-global errors do not manufacture per-context poison.

Pinned state retains the descriptor chain and optional host table. The identical
host table reaches create and destroy. Supplying any callback changes native
create-error cleanup to retention of the full state/device/runtime bundle,
without operating on the returned handle. Empty bundles preserve ordinary D005
cleanup. Pre-native rejection cleans up normally; post-create query failure uses
ordinary terminal cleanup. OK/null creation and failed destroy retain the bundle;
destroy is attempted once. External targets remain independently caller-owned
through all possible native uses, including future ordinary contexts and errors.

This implements the reviewed contract without a new dependency, captured closure,
trampoline registry, hidden lock, automatic unregister, OOM recovery service,
GPU submission/wait or image-quality policy. Native threading, allocator
compatibility and callback quiescence remain explicit unsafe obligations.

## Equipment and commands

SDK commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; four pinned public
header hashes. The loader/provider DLLs are hash-checked and Authenticode-checked
by `native.py`. Windows 11 `10.0.26300`, AMD RX 9060 XT adapter 0, vendor
`0x1002`, device `0x7590`, driver `32.0.32015.7029`; ordinary DX12 debug layer,
GBV disabled. MSVC `14.44.35207`, Windows SDK `10.0.22621.0`, Python `3.14.1`,
Rust/Cargo `1.99.0` and exact MSRV `1.88.0`. No network acquisition/publication.

```sh
python tools/verification/run.py run ci --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-callbacks --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed
```

The [profile contract](../../tools/verification/README.md#public-callback-verification)
and inspectable producer files define the bounded public workload. Final CI/MSRV
use the approved filesystem access required by existing temporary compiler tests.

## Results

| Attempt | Status | Stages |
|---|---|---|
| `20261025T204617Z-ci-aae263a41a79` | Passed; inputs unchanged | 192 |
| `20261025T204615Z-msrv-eba92c0d6d16` | Passed; inputs unchanged | 169 |
| `20261025T204556Z-native-callbacks-aa51cf078ec0` | Passed; inputs unchanged | 15 |

The [compact evidence](2026-10-25-callbacks-diagnostics-verification/summary.json)
retains all required case outcomes, source/equipment identities, callback events,
native measurements and earlier attempt outcomes. All fourteen finalized artifact
inventories were independently revalidated. The evidence is 924,156 bytes, SHA-256
`95d3d34a576b36d3621c7d6a175c1044255751fe6025a469b434c011ecd58ea6`.
All three final runs contain identical repository-source hashes. CI/native
additionally include supplied SDK hashes; offline MSRV has no SDK input.
Implementation, tests and orchestration are unchanged after these checks.

The independent Rust/MSVC assertions cover all callback signatures, diagnostic
tags/levels/message values, debug flags, descriptor/payload layouts and Microsoft
pointer-only metadata. The creation ABI step includes the callback assertions;
the shared profile inventory keeps its original number of setup steps.
Production raw CB aggregate exchange passes in both directions on Rust 1.99 and
1.88: size 56, alignment 8, handle offset 48, four exchanges each way and four
corrupted-field negative controls per compiler. Buffer type 0, generic-read state
20, the callback metadata constants and message severities are available through
`interop::abi`; an isolated consumer constructs the CB payload without a direct
sys dependency or hand-written SDK numeric constants.

Controlled callers exercise every optional group, complete paired callbacks,
64-bit sizes/offsets/statuses, omitted extension nodes, stable chain/host addresses,
owner moves, ordinary Drop and destination-thread movable cleanup. Both families
check callback-create error/OK-null/destroy-error retention, retained runtime and
device references, post-create query cleanup, native error preservation and
poison precedence. Existing ordinary-path tests continue to verify D005 cleanup.
Both families check diagnostic routing, null versus live context, callback
presence, all four levels, no fallback and no runtime-global fabricated poison.
Public consumer checks cover private fields, unsafe signatures, portable options,
feature/platform boundaries and `CreationCallbacks: !Copy + !Clone + !Send + !Sync`.
Creation/memory flag parity and independent debug-view/colour-hint merging share
the production mapping in controlled public dispatches.

## Public native observations

For each family and selected provider 3.1.5/4.1.1, create A with callback set 1,
create B with compatible set 2, dispatch through the surviving A and B, then
create an ordinary context with omitted allocation groups. Query each effective
provider identity. Match pre-create estimate flags. Configure modern global
diagnostics explicitly, exercise legacy null/live routes, then destroy each
context after completing all GPU work. The public workload totals 12 contexts,
20 submitted/fenced frames and 16 delivered message callbacks. A 0.5 ms frame
time is the already-spiked warning stimulus, not an invalid pointer/resource test.

All 20 frames report 230,400 finite and written pixels at 640×360, zero debug
errors and no device removal. Debug view is requested on eight frames and omitted
on twelve. No frame is compared with another provider, implementation or quality
reference. Internal readback is consumed for execution/write checks only.

Native callbacks observe 24 host allocations/frees, 240 resource allocations/frees
and 368 CB allocations. Eighty set-1 resources are freed through compatible set-2
callbacks. Final instrumented host/resource/heap live counts are all zero.
The 368 CB upload resources, callback targets/code and independent device
reference deliberately remain process-live; this is not automatic reclamation
or proof of quiescence. Actual messages use the exact callback pointers forwarded
through the public API. Earlier provider-specific creation/global routing and
post-context message persistence remain bounded observations in the unchanged
[spike record](../research/records/2026-10-25-exp-upscaler-callback-allocation.md).

Both public runtime families accept modern upscaling-global Configure. Deprecated
null-context Configure returns native code 2 for both selected providers; the
wrapper preserves it. Deprecated live-context Configure returns 0 on both
providers and both owners. A bounded subsequent public frame after the runtime
null-context error succeeds; this does not establish universal recovery.
Acknowledgement alone proves no effective level or message-delivery guarantee.

Heap ABI, forwarding and controlled calls pass, including a separately labelled
65,536-byte real equipment-owned heap allocation/free. Neither selected SDK
provider invokes the heap callbacks during this workload. Their public route is
implemented; SDK invocation remains an explicit native evidence limit.

## Earlier attempts and corrections

The first focused compile used the wrong debug-view constant name; it was fixed
to the pinned SDK name. Clippy caught explicit ABI identity signatures requiring
a local complexity allowance and a missing observation alias, then an unused
debug-level conversion in no-default test builds. A portable level-mapping test
now checks that conversion in those builds. No public contract changed.

Native attempt `20261025T201515Z-native-callbacks-9a65635c18fb` completed the
public workload but failed its evaluator because Rust stderr markers were not
included. `20261025T201901Z-native-callbacks-eba44a0007a5` also completed the
workload but lacked the buffered C++ final-balance line; explicit output flushing
fixed it. Both remain failed. Attempt
`20261025T201943Z-native-callbacks-9dbe04931d06` passed 16 stages before the
warning stimulus and ABI-step consolidation; it observed no message delivery.

CI/MSRV planning attempts `20261025T201916Z-ci-75e4e978bcab` and
`20261025T201916Z-msrv-b48470304b54` stopped before execution because two new
positive consumer definitions omitted the required `diagnostic: null` field.
Restricted runs `20261025T201953Z-ci-401a600fc9ca` and
`20261025T201953Z-msrv-cfc908bed593` retain temporary-file access failures.
Approved MSRV attempt `20261025T202147Z-msrv-d3cf5ac7c8d1` passed all 169
stages. Approved CI attempt `20261025T202220Z-ci-1f2b051cd454` retained the
no-default Clippy issue and six infrastructure plan-count failures from adding
a separate shared ABI step. Including the callback assertions in the existing
creation step preserves those inventories without weakening their checks.

The message-bearing native attempt `20261025T202907Z-native-callbacks-25bde078df18`
passed all 15 stages. Final checkout review found its linker import-library and
export files in the repository root. The producer now passes `/IMPLIB` with the
exclusive native-stage path. The misplaced files are preserved under ignored
`target/verification-build/callback-linker-products-20261025`; native attempt `20261025T204010Z-native-callbacks-9bf6d2c578c4`
passed all 15 stages with those products included in its artifact inventory.
That correction changed only the link-output argument. The subsequent final
source/MSRV/native runs below share the corrected producer.

A final public-consumer review added the two named CB metadata constants
(`BUFFER` and `GENERIC_READ`) and reexported the existing resource/message
constants through `interop::abi`. This completes the wrapper-only callback
payload boundary; it does not widen ordinary resource acceptance. Independent
Rust/MSVC assertions and the existing positive public consumer check these values.
All three source/MSRV/native profiles were repeated afterwards. Before that
addition, CI `20261025T202920Z-ci-23d5eb6e5514` and MSRV
`20261025T202919Z-msrv-7b725af0c444` also passed 192/169 stages; their outcomes
remain in the compact evidence.

Draft file/licence inspection also passes `cargo package -p <crate> --list
--allow-dirty --offline` for both crates and `cargo package --workspace
--allow-dirty --no-verify --offline`. The actual archives contain the callback
modules/tests, plus byte-identical package `LICENSE` and `LICENSE-AMD` files.
The compact evidence records the two draft hashes and file counts (33 raw,
144 wrapper). This is package-content inspection with verification disabled,
not final archive build/native acceptance or publication.

## Completion boundary

General native callback affinity/concurrency, reentry, shutdown/quiescence,
safe callback reclamation, recoverable native allocation failure and native
failed-create/destroy behaviour are not established. The native callback workload
is serial on one thread; destination-thread behaviour is controlled-fixture
evidence only. Heap SDK invocation remains unobserved. These limits are retained
in D040's explicit unsafe contract.

The checkpoint covers SDK access, ABI, mapping, lifetime/error handling, fenced
execution and cleanup on this recorded deployment. Final release reconciliation,
exact package archives, extracted-package native checks, publication simulation
and publication remain separate. No commit, push or publication was performed.
