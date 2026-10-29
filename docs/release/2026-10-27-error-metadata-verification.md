# Structured error and provider metadata verification

- Date: 2026-10-27.
- Scope: approved U10 [contract](../design/errors-and-provider-metadata.md)
  under [D042](../adr/D042.md), wrapper 0.3.0 / sys 0.1.2, AMD SDK v2.3.0,
  Windows x64/MSVC + DX12, Rust 1.88+.
- Result: **U10 implemented and verified within the recorded configuration**.
  Final confirmation passes 583/583 source/MSRV/consumer/ABI/native cases and
  15/15 callback cases; 166/166 format/generator cases retain the explicitly
  bounded callback-test-helper delta below. All accepted attempts pass artefact
  integrity checks with unchanged captured inputs.

## Implemented contract

Main `error::Error` exposes Input, Native, Backend, operation-attributed Invariant,
ContextUnusable and QueryAllocation. It remains an owned standard Error with
Send/Sync/'static, without Clone/Copy/PartialEq/Eq. Loading keeps its separate
owned boundary. InputError retains all live reasons/fields, including unsupported
required inputs; only dead DispatchValidation::UnsupportedResource is removed.
Input rejection does not imply that preceding enumeration/COM inspection was absent.

NativeError retains exact u32 code and operation. Native Upscale, GenerateReactiveMask,
Tuning and ConfigureContextDebugLegacy failures store the first NativeError in
shared eligibility. The first return is Native; subsequent live queries, recording,
Configure and context preflight reject as ContextUnusable with the identical
TerminalCause::Native before inspection/FFI. Ordinary query/input/inspection
failures do not poison; the two null-context diagnostic routes do not mark
individual owners terminal. All three debug routes may affect shared global state.
Creation/destruction and exceptional dependency retention retain existing policies.
U11 public status/abandonment operations are not implemented by this slice.

DX12 failures retain object/operation/role and exact signed HRESULT, with the
original windows::core::Error as their owned source. The existing Windows 0.62.2
dependency enables std, so the complete original source can be retained behind a
portable boxed standard-error boundary. No package/version/raw ABI change or
wrapper unsafe Send/Sync implementation is added. Query allocation exposes
count()/reason() and preserves TryFromIntError/TryReserveError; this covers the
existing fallible buffers, not all allocation failures. Invariant displays
attribute creation results to Create rather than describing every error as Query.

ProviderId from_u64/as_u64/From conversions retain every bit, including zero and
u64::MAX. ProviderInfo::new owns arbitrary supplied names without provenance or
capability claims. Numeric zero remains explicit, requiring fresh enumeration;
only ProviderSelection::Default requests default selection. Live zero-ID output
still violates its existing query contract. Membership, no fallback, immediate
native-name copying and post-create selected-provider matching remain unchanged.
ResourceFlags empty/from_bits_retain and union/intersection operate on every u64
bit; no truncation, complement, new resource rejection or requirements/memory
constructor is added. The optional resolver's policy/inclusion stays unchanged.

## Controlled and downstream coverage

All existing workloads/fixtures migrate to structured error metadata comparisons,
without adding Error equality even under cfg(test). Shared assertion equipment
checks categories and relevant reason/operation/code/object/HRESULT fields;
original-source preservation has separate production-path tests. There is no
release-facing compatibility error or competing validation/recording path.

New tests cover owned error traits and source downcasts; original Windows backend
causes; allocation count/reason/conversion/reservation sources; unknown codes;
first-cause retention across all four mutating origins and later attempted success,
failure, inspection and queries; and creation invariant attribution. Both-family
controlled diagnostics distinguish all three routes and preserve live terminality.
Existing generator/tuning/recording tests retain first codes, independent contexts,
ordinary query continuation, no post-terminal native work and cleanup/retention.

Portable external tests assemble synthetic provider metadata and resolver policies,
round-trip zero/high-bit/max IDs, and compose full-width unknown resource bits.
Controlled zero/max selection proves that an override node remains explicit and
an absent ID still requires enumeration/rejects. Consumers require portable
structured categories/getters and reject private NativeError fields, exhaustive
terminal matching, removed flat poison/resource variants and Error Clone/Copy/Eq.
Examples, package README and Rustdoc describe the canonical API. Root README and
frozen research/release records/producers retain their published/baseline meanings.

## Reproduction

Use the existing SDK/toolchains, without downloads, from the repository root:

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/error-metadata-verification
python tools/verification/run.py run native-formats --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/error-metadata-verification
python tools/verification/run.py run native-callbacks --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/error-metadata-verification
python tools/verification/repository_size.py
```

Initial local migration checks exposed standalone compiler-module path/feature
assumptions, remaining whole-Error comparisons, and old poison expectations which
had not retained the original codes/origins. These were corrected structurally;
no release Error equality was added. Restricted temporary compiler writes required
the permitted Windows repeat. A new pure resolver test initially used a decorated
name that the existing strict resolver intentionally ignores; the test now uses
two recognised numeric names without widening resolver policy. Those preparation
failures are not accepted passes. Captured attempts, results and limits follow below.

## Captured preparation attempt

`20261027T135545Z-windows-upscaler-followup-a2f821fd756e` is a failed preparation
attempt. Default/movable Rustdoc rejected one stale link to removed
`Error::Dx12Inspection`. The link was corrected to Backend and a full-feature
Rustdoc check passed. Other source/consumer/MSRV cases continued, but this attempt
is not accepted: its source gate blocks native regressions and the correction
also changes its captured input bytes. A fresh full confirmation uses the final
source below. No earlier failure is relabelled as a pass.

## Callback test-helper correction and final source

The initial passed `native-formats` attempt
`20261027T135946Z-native-formats-a2cface5db83` (166/166) and passed Windows attempt
`20261027T140119Z-windows-upscaler-followup-e3c4232e1545` (583/583) share captured
repository/SDK source identity
`073c6230a3c03bc5b5c58b130779c8e8a9f93bb53c1a527c937fb411929ac4d6`.
Their inputs remain unchanged within those attempts.

`20261027T141118Z-native-callbacks-3716527768b1` fails its native case: the test
helper accepted only ConfigureContextDebugLegacy when interpreting an ordinary
legacy refusal. The wrapper correctly returned ConfigureGlobalDebugLegacy/code 2
for the null-context request. This is an outdated test expectation, not a new
SDK failure or a production operation-label defect. The helper now selects the
expected global/live operation using its explicit `live` argument.

Only `crates/fsr-sdk/tests/native/callbacks.rs` changes between the passed format/
Windows inputs and the final source. Its SHA-256 changes from
`e099a7d7fb5c88bd3bd0c296dccff79230dbf399f1aabf843f12e5af35a6c4ba` to
`3ad4f4624f0007ec888afb4454ebcf2c34d7235db0c5aa6fccb50d42395dd4e0`.
Every captured production, format-workload, consumer, C++ and protocol file still
matches the final source. The format selectors do not invoke that callback test
helper; their 166-case execution remains evidence for the unchanged format paths.
The full Windows matrix and native callbacks are separately repeated after this
correction, rather than treating the failed callback attempt as accepted.

Final repository/SDK source identity:
`9c45a51f2edab8d469c1ef513cb1eb978f925b733a8db62e03e4007e81cb4eb1`.
Final repository-only source identity:
`1cf2ae3be57004d2ff4ae78704532fe2ebc046403466bcb727c9bb3c486ef4d2`.
Per-run input identities also include configuration; they are distinct from source
identity. The initial Rustdoc preparation's repository/SDK source identity was
`bb9ae0eaee1a0e2585eccb8d97000618afbf60940fda7287a50571b47030a087`.

## Equipment and native bounds

Rust/Cargo 1.99.0, exact MSRV 1.88.0, Python 3.14.1, Windows 11 build 26300,
MSVC 14.44.35207, Windows SDK 10.0.22621.0. AMD Radeon RX 9060 XT adapter 0
(`1002:7590`), driver 32.0.32015.7029, ordinary DX12 debug validation with GBV
disabled. SDK v2.3.0 headers are pinned at
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Both staged DLLs have valid AMD
Authenticode signatures; SHA-256:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

Format/callback supplements explicitly select 4.1.1 and 3.1.5; public generator
write/parity cases select 3.1.5. Names identify observations, not universal support
or attribution of all internally loaded code to the staged modules.

Final callbacks pass 15/15 cases: current/MSRV raw aggregate ABI exchange and
12 contexts / 20 completed frames across both families and selected providers.
Legacy null-context calls retain code 2, live calls code 0. Observed host and
resource allocations/frees balance at 24/24 and 240/240; 368 CB allocations and
16 messages are observed. External targets and CB backing remain process-lived.
Native heap callback invocation remains unobserved; controlled/ABI coverage does
not establish that native route. No general callback reclamation/quiescence claim
is added.

## Final confirmation and limits

| Accepted attempt | Result | Boundary |
|---|---|---|
| `20261027T141323Z-windows-upscaler-followup-9d16e36b3354` | 583/583 passed, inputs unchanged | Final source: seven current Rust check/test/Clippy/docs configurations, exact MSRV 1.88.0 all-target/tests, both compiler consumer/example matrices, dependency/Vulkan gates, independent MSVC ABI and existing native queries/lifecycle/recording/Configure/selected-provider/movable/sharpening regressions |
| `20261027T135946Z-native-formats-a2cface5db83` | 166/166 passed, inputs unchanged | Both public families, selected 4.1.1/3.1.5 and generator 3.1.5; independent C++ parity, inputs/readbacks, diagnostics/fencing/cleanup. Only the separately corrected, uninvoked callback-test helper differs from final source |
| `20261027T141301Z-native-callbacks-d9ca0d236b03` | 15/15 passed, inputs unchanged | Final source: current/MSRV callback ABI, both families/providers, modern/null-context/live diagnostics and compatible callback cleanup |
| Three accepted `run.py verify-run` checks | Passed | Complete outcome and retained artefact hash inventories |
| Formatting / documentation links / UTF-8/LF / repository size | Passed | Final maintenance checks and 50 MB checkout budget |
| `cargo package --list` | Passed | New error modules, portable tests, MPL/AMD notices and examples are included |

Integrity reproduction uses `python tools/verification/run.py verify-run --run
<attempt-directory>` for each accepted path under `target/error-metadata-verification`.
The coordinator checks original retained bytes, not merely status labels.

The default native query/Configure regressions report provider `4.1.2 *`; the
supplements above select 4.1.1/3.1.5 explicitly. Bounded native success does not
prove numerical tuning application, general provider capabilities, hidden native
thread/quiescence guarantees or arbitrary GPU states. There is no new image-quality
or performance claim.

An isolated `cargo package --allow-dirty --no-verify --offline --locked` attempt
with `--target-dir target/error-metadata-package-inspection` cannot resolve
`fsr-sdk-sys ^0.1.2` from the local registry cache, which offers 0.1.1. No exact
release archive is produced or accepted by that check. Final registry/package
closure and archive-native coverage remain U12/release-preparation work; no
network fetch or publication is implied. Package file-list inspection passed.

U11 public usability/abandonment and U12 integration/release archives remain open.
No final release, publication or broader native support is established here.
