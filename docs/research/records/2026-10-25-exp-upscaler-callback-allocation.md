# Experiment: upscaler callback ABI, routing and allocation ownership

- Investigation date: 2026-10-25.
- Scope: R03/U04, SDK v2.3.0, Windows x64/MSVC + DX12, selected signed
  providers 3.1.5 and 4.1.1. Direct SDK probe, not a production wrapper change.
- Question: can Rust/C exchange the constant-buffer result exactly, which
  allocation/message callbacks execute, and what survives callback replacement,
  omitted/incomplete registrations and destruction of the installing context?
- Inputs: the [SDK source investigation](2026-10-25-src-upscaler-callback-allocation-contracts.md),
  [Rust FFI practices](2026-10-25-src-rust-ffi-callback-allocation-practices.md)
  and [prior synthesis](../callbacks-and-allocation.md).
- Outcome: the final attempt passes **17/17 stages**, with **10 successful
  contexts and 16 submitted/fenced frames**. Resource/CB replacement and sticky
  registration are observed on both providers. Message routing differs between
  providers, and signed-loader behaviour differs from the inspected open loader.
  **Heap callbacks were not invoked**; their native execution remains unverified.

## Reasoning and fixed success boundary

The source record establishes a credible shared-state hazard, but cannot prove
the signed provider implements it. The smallest useful sequence therefore uses
two distinguishable, mutually allocation-compatible callback sets, two explicit
acquisitions of the same loader image, and surviving contexts after replacement.
A third message callback identifies the last installing context independently
of the surviving context. All callback code and userdata stay live; no retired
target is deliberately freed to test for a use-after-free.

The independent C++ consumer uses hash-checked SDK types; Rust reuses the existing
raw resource declaration and adds an experiment-only CB aggregate. This avoids
assuming Rust layout assertions alone prove MSVC return-by-value interoperability.
The fixture tests every meaningful field in both directions, including high
integer bits, with a corrupted-field negative control. It compares no padding.

Native success requires exact live provider identity, successful create/destroy,
the full frame sequence, completed fences, finite/written output texels, clean
ordinary D3D12 diagnostics/device status and balanced tracked allocations.
Callback absence is explicitly `not_observed`. An optional control's rejection,
or a diagnostic stimulus producing no message, is retained as an observation;
neither becomes a successful callback-delivery claim. There are no quality,
algorithm, output-equality or performance criteria, and no allocation-failure
injection, invalid context operation, concurrent-use stress or native reentry.

## Setup, source state and reproduction

The recorded baseline is `9ed7492989977d0d58b66c0e35d0bcfd77d77950`.
The worktree already contained the preceding research synthesis and incoming
records. This task adds only probe code, its protocol, a verification README
entry and this record/evidence; it changes no production crate source, manifest
or dependency. The [compact evidence](2026-10-25-exp-upscaler-callback-allocation/evidence.json)
contains the final source-file inventory and each attempt's source/input digest,
commands, evaluation, observations and failed-command diagnostics. A revision
alone is not needed to understand these results.

| Input | Recorded identity |
|---|---|
| SDK source/header baseline | v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; four header hashes match `tools/native-headers.json` |
| Loader DLL SHA-256 | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| Upscaler DLL SHA-256 | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |
| Signature check | Both staged DLLs report Valid Authenticode signatures from Advanced Micro Devices; cached/local inspection, no new strict online-chain claim |
| OS / GPU | Windows 11 `10.0.26300`; adapter 0, Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`, driver `32.0.32015.7029` |
| Native tools | x64 MSVC `14.44.35207`, Windows SDK `10.0.22621.0`, PowerShell `7.6.8`, Python `3.14.1` |
| Rust tools | `rustc 1.99.0` (`b940084d7`, LLVM 23.1.1) and exact `1.88.0` (`6b00bc388`, LLVM 20.1.5), Windows x64/MSVC |
| Final input identity | `4edad9bd27c391ad0b82cad3e4e2e5934cae04f9831777c4d0423ab2061e7ddb` |
| Final source digest | `3b44e111c268cce2d136a084b77dee35d15e7d3c789dcc3007ae1f4d10d0ecee` |

Run the [protocol](../../../tools/verification/callback-spike-protocol.md) from
the repository root with the trusted SDK already installed and both Rust
toolchains available:

```sh
python tools/verification/callback_spike.py --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The [Python coordinator](../../../tools/verification/callback_spike.py) uses the
existing subprocess watchdog, native staging, signature/header verification and
GPU helper. The [C++ probe](../../../tools/verification/callback_spike.cpp)
calls the SDK exports directly; the [Rust fixture](../../../tools/verification/callback_abi.rs)
is built as a separate DLL for each compiler. MSVC builds use `/W4 /WX` and the
Rust fixtures use `-Dwarnings`. No download, Cargo dependency, public callback
API, release-profile change or full CI regression is part of this experiment.

All contexts use API version 4.1.1 independently of selected provider ID,
320×180 input and 640×360 output, explicit exposure, creation flags **640**
(checking + visualization), and reset on each probe frame. Memory V2 uses the
same flags, dimensions, device and provider override. The helper supplies its
existing deterministic RGBA16F/R32F/RG16F profile and output sentinel. Frames
alternate ordinary/debug-view requests as specified by the protocol. Each new
resource/list harness is submitted and CPU-fenced before destruction or reuse.
Memory-query success is not an estimate-accuracy or flag-application proof.

## Observed ABI results

**Verified on both Rust compilers:** CB result size **56**, alignment **8**,
`handle` offset **48**; resource size **48**, alignment **8**, description offset
**8**, state offset **40**. Four calls return the complete Rust aggregate to
MSVC, and four return the complete MSVC aggregate to Rust, per compiler. All
fields and argument values agree. Four deliberately corrupted return values
are rejected per compiler. This establishes the tested `extern "C"`/MSVC exchange,
not native provider acceptance or GPU lifetime by itself.

The C++ declaration assignments compile all five allocation callback signatures
against the pinned headers. Only resource/free and CB actually execute through
the SDK in the GPU portion; C++ compilation does not count as Rust ABI exchange
or native execution of the heap pair.

## Observed native allocation and routing results

Each provider has a one-context/two-frame default-DX12-allocation control and a
four-context/six-frame shared-registration child. Every frame writes all
**230,400** output texels, all are finite, ordinary debug errors are zero and
device removal is zero. These are integration checks only.

| Final native observations | Provider 3.1.5 | Provider 4.1.1 |
|---|---:|---:|
| Successful contexts / fenced frames | 5 / 8 | 5 / 8 |
| Host alloc / free calls | 15 / 15 | 15 / 15 |
| Host allocation sizes per context, bytes | 524,600; 1,634,664; 16 | 1,099,864; 1,634,664; 16 |
| Custom resource alloc / free calls, shared child | 104 / 104 | 56 / 56 |
| Custom CB allocations, shared child | 56 | 164 |
| Resource allocations per custom context | 26 | 14 |
| Set-1 resources later freed by set 2 | 26 | 14 |
| Heap alloc / free calls | 0 / 0 | 0 / 0 |
| Message callbacks, default / shared child | 2 / 6 | 1 / 6 |

**Verified routing sequence on both providers:** context A installs set 1 and
initially uses CB set 1. Creating B through the second acquisition installs set 2;
A's later CB calls use set 2. C omits the allocation extension, while D supplies
only resource/heap allocation functions from set 1, without their frees or CB.
Both still allocate resources and CBs through set 2. Destroying D and B does not
restore set 1: surviving A continues using CB set 2, and its original set-1
resources are freed by set 2. All instrumented blocks are matched once, leaving
zero tracked host/resource/heap allocations after successful teardown.

The two application loader acquisitions return the same module handle. This
establishes sharing for this deployment, not a universal process-wide namespace
across distinct provider/DLL images. Both callback sets intentionally use the
same allocation/free protocol and device, so these cross-set frees are valid
probe observations, not a mismatched-allocator crash test.

CB uploads copy the supplied bytes into separate 256-byte-aligned upload
resources; all **220** are retained through GPU completion and context teardown,
then released after the application loader handles. No ordinary callback free
exists for this backing. All recorded callbacks execute on the main thread at
callback depth 1; the probe makes no reentrant or concurrent SDK call. These
observations cannot prove callback affinity or absence of future native work.

## Diagnostics: source and signed runtime differ

The stimulus uses positive **0.5 ms** frame time. Its observed message has
87 UTF-16 code units and FNV-1a hash `11733137168520346870`, matching the inspected
checker's frame-time warning. Text is measured during invocation; no borrowed
native pointer or private address is retained. Message event fields named
`allocation` and `bytes` represent this hash and code-unit length, respectively.

**3.1.5:** creation message callbacks replace the observed global route. After B
is created, A's warning reaches callback 2; C then installs callback 1, D installs
distinct callback 3. After **D and B are destroyed**, A's warning still reaches
callback **3**, despite A having been created with callback 1. This directly
observes a callback dependency surviving its installing context.

**4.1.1:** the baseline's first frame produces no callback. Installing the modern
upscaling-global verbose callback 2 then produces the warning on the second
frame. In the shared child, global warning callback 1 is installed before
creation; all six warnings reach callback 1 despite creation callbacks 2/3 on
later contexts. Thus this stimulus does not show creation callbacks controlling
the signed 4.1.1 global warning route. It does not prove those pointers are never
used for other messages.

**Both providers:** changing the modern loader-global upscaling callback to set 2
between the baseline frames changes observed warning delivery. The signed loader
therefore reaches the selected provider's diagnostic route in this bounded test.

| Standalone signed-loader request, null context | Observed return |
|---|---:|
| Modern global debug, upscaling effect, callback 1 | `0` (OK) |
| Modern global debug, unknown effect `0x00fe0000`, callback 2 | `2` (unknown descriptor) |
| Deprecated global-debug descriptor, callback 1 | `2` (unknown descriptor) |
| Modern global debug, upscaling effect, null callback | `0` (OK) |
| Unknown query header `0x00fefffe`, after each setter | `2`, no message, all four requests |

These signed-loader observations qualify the source record's **open-loader**
unconditional handling of both descriptors and ignored `effectId`; they do not
contradict what that inspected source contains. The signed path must not be
described as universally ignoring the effect field. Rejection of one unknown
effect does not prove isolation between supported effects. The deprecated
descriptor was tested with **null context only**; no rejection claim is made for
an alternative live-context route. The synthetic query never reached the
source-described no-provider diagnostic path, so it establishes no loader
message-delivery or callback-unregistration proof.
The standalone case's historical suffix `global.4.1.1` is a case label, not an
observed provider identity: global Configure supplies an effect, without a
provider override or live context. Provider-specific claims above come from
the separate children that query and verify their actual context identity.

One exact-value correction to the incoming SDK record: pinned-header
`FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE` is **`0x0fffffff` (268,435,455)**,
not `UINT32_MAX`. This experiment asserts and uses the header value. The incoming
historical record remains unchanged.

## Attempts, failures and retained evidence

Every attempt used the command above and a fresh coordinator directory. All
four finalized input snapshots remained unchanged and their retained local
artefact inventories were checked after execution.

| Attempt under `target/verification/` | Outcome and limit |
|---|---|
| `20261025T190701Z-callback-spike-1bbd26db5205` | Failed: 9 passed, 1 failed, 7 blocked. The standalone Rust fixture omitted the production crate's allowance for upstream C-style field names; `-Dwarnings` stopped compilation. No ABI exchange or native experiment child ran. Add `non_snake_case` to the imported module's allowances. |
| `20261025T190743Z-callback-spike-5687f4ac968e` | Failed: 16 passed, 1 failed. ABI/current/MSRV and all four GPU children pass; the global child exits 11 because unknown query returned `2`, contradicting the probe's expected `4`. Preserve that failed prediction; accept either declared unsupported-query code in the subsequent observational protocol. |
| `20261025T190954Z-callback-spike-e0a264b489c4` | Failed: 16 passed, 1 failed. ABI and GPU children pass. The global child exits 11 when unknown-effect Configure returns non-OK, contradicting the expected open-source ignored-field outcome. Preserve the rejection; subsequent protocol records all optional-route codes rather than requiring open-source behaviour. This attempt adds live-global diagnostics and the distinct post-destroy message callback. |
| `20261025T191147Z-callback-spike-0756c72331e5` | Passed 17/17; 19:11:47–19:12:04 UTC. Records all four global requests, their code-2 rejections and absent synthetic diagnostics; enables shared warning delivery explicitly. The table and totals above refer only to this attempt. |

Earlier successful GPU children are preserved as observations inside failed
attempts; they do not turn those runs into passes. The final pass means the
bounded observational protocol completed, including its explicit limitations.
It does not relabel rejected deprecated/unknown-effect requests, absent heap
callbacks or missing synthetic diagnostics as supported routes.

The compact [evidence JSON](2026-10-25-exp-upscaler-callback-allocation/evidence.json)
retains full final callback events, all attempt outcomes/commands and failed
diagnostics; earlier successful event streams are compacted to phase/count/message
summaries and event digests. Original local attempts remain under ignored
`target/verification`. No AMD binaries, raw frames, environment dump or source
snapshots are copied into this record. Hashes identify retained measurements;
they cannot reconstruct a past GPU frame. Evidence JSON SHA-256:
`62e692b9d698e91e9e9fd313e2614eebea1a9ffd1ba16a47f86e87f02c81a3ff`.

## Consequence for the next decision

**Inference/recommendation, not accepted design:** proceed to the R03/U04 API
contract using explicit unsafe function-pointer integration and separate debug
controls. Resource/CB allocators need a compatible shared policy across the
affected native module's contexts, or an externally enforced prohibition on
incompatible replacement. A context-local callback owner cannot reclaim global
state on successful Drop or silently restore defaults by omitting an extension.
Keep creation messages and modern global diagnostics distinct; document the
observed provider differences and preserve native errors for unsupported routes.

Host userdata retention, compatible create/destroy allocation, COM transfer,
CB/GPU lifetimes, no-unwind policy, failure retention and local/movable audits
still need the explicit public contract and implementation. The by-value CB ABI
question is resolved for the tested target/compilers. Heap native invocation,
nonzero heap offsets, safe unregister/quiescence, failed-create/destroy paths,
OOM recovery and general native threading remain unverified. None of these
observations justifies safe captured closures, a registry framework or a universal
native guarantee. Heap/legacy-route evidence belongs in the later applicable
implementation checks; no broad online rerun is indicated by this spike.

This record accepts no API, implements no R03/U04 production feature, and verifies
no final 0.3.0 package. Integration into current topic/roadmap status remains
separate from recording the completed experiment.
