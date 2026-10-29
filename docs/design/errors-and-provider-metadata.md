# Composable errors and provider metadata

- Drafted and accepted for U10 implementation: 2026-10-27.
- Status: **accepted, implemented and verified within the recorded bounds**,
  following maintainer approval of the reviewed contract. See
  [production verification](../release/2026-10-27-error-metadata-verification.md).
- Scope: [U10](../../ROADMAP.md#next-actions), refining the accepted boundaries in
  [D042 §§5–6](../adr/D042.md). U11's public status and abandonment operations
  were implemented separately; [the current migration guide](../MIGRATING_0_3.md)
  describes the completed API.
- Baseline: wrapper 0.3.0 / sys 0.1.2, Rust 1.88, SDK v2.3.0,
  Windows x64/MSVC + DX12. The existing native safety/retention contracts remain.

## 1. One error entry point, structured causes

Keep `error::Error` as the return type. Group failures by responsibility rather
than adding more unrelated flat variants. Accepted signature notation:

```text
#[non_exhaustive]
enum Error {
    Input(InputError),
    Native(NativeError),
    Backend(BackendError),
    Invariant { operation: Operation, reason: InvariantViolation },
    ContextUnusable(TerminalCause),
    QueryAllocation(QueryAllocationError),
}
```

`Error` guarantees `Debug + Display + std::error::Error + Send + Sync + 'static`;
remove its `Clone`, `Copy`, `PartialEq` and `Eq` commitments. Owned causes can then
be retained without making future error additions depend on those traits.
Small reason/code types retain `Debug + Clone + Copy + PartialEq + Eq` where
their contents support it. Extensible public enums remain non-exhaustive.
Loading retains the separate owned `runtime::LoadError` boundary.

| Payload | Contents and migration from the preceding Error |
|---|---|
| `InputError` | `MissingBinding { role }`, `InvalidDimensions { width, height }`, `InvalidTuningValue { parameter }`, `Query(QueryValidation)`, `Dispatch(DispatchValidation)`, `ProviderNotEnumerated { requested }`, `UnsupportedResourceRequirements { required }`. Preserve existing fields/reasons. |
| `NativeError` | Private `Operation` and exact `u32` code, exposed by `operation()` and `code()`. Replaces `Native { operation, code }`; unknown codes remain lossless. Library-emitted failures have nonzero codes. |
| `BackendError` | Initially `Dx12Inspection(Dx12InspectionError)`. Private payload exposes `object()`, `operation()` and signed `hresult()`, retaining the original error as an owned source. Replaces `Dx12Inspection`; resource-role attribution survives. |
| `InvariantViolation` | Retain existing reasons; add operation attribution at the outer variant. Replace `NativeInvariantViolation`. Display must identify the actual operation, including creation/provider mismatch, rather than describing every case as a query. |
| `TerminalCause` | Initially `Native(NativeError)`. Replaces the information-free `DispatchPoisoned` / `ConfigurePoisoned` errors with the first operation and code. See §2. |
| `QueryAllocationError` | Private count exposed by `count() -> u64`, inspectable reason `CountNotRepresentable` or `ReservationFailed` exposed by `reason()`, and the original `TryFromIntError` or `TryReserveError` source. Replaces `QueryAllocationFailed`. This covers existing fallible query buffers, not universal allocation-failure recovery. |

`Input` identifies rejected input, not a promise that the entire call performed
no inspection or enumeration. Checked-value construction remains pure; provider
membership checks still enumerate, and resource checks may inspect COM objects.
Remove only the dead `DispatchValidation::UnsupportedResource` reason.
The live `UnsupportedResourceRequirements` rejection remains.

`source()` exposes each wrapped error payload; invariant errors expose their
reason. Native/input/invariant leaves have no invented lower-level source.
Backend chains retain the original Windows error, and allocation chains retain
the conversion/reservation error. Terminal chains are `Error → TerminalCause → NativeError`.
Payloads implement `Display` and `std::error::Error` as needed for those chains.

The pre-U10 Windows dependency disabled default features without opting into
`std`, so its error did not implement `std::error::Error`. The
[current dependency](../../crates/fsr-sdk/Cargo.toml) now explicitly enables
`windows/std` within the existing Windows/DX12 gate, retaining version 0.62.2. Store the cause
as a private `Box<dyn std::error::Error + Send + Sync>`; public error signatures
stay portable and contain no Windows types. No new package or unsafe Send/Sync
implementation is added. The feature graph and MSRV are covered by verification.

## 2. Preserve the first terminal failure; prepare U11

Refine existing `Operation`: retain `Create`, `Destroy` and `Query`; replace broad
`Dispatch` / `Configure` cases with `Upscale`, `GenerateReactiveMask`, `Tuning`,
`ConfigureDebug`, `ConfigureGlobalDebugLegacy` and `ConfigureContextDebugLegacy`.
The three debug variants distinguish the setup's two null-context routes from the
live-context legacy route. All three may affect shared global diagnostic state;
the receiver does not imply context-local effects. These are wrapper operation
names, not new native entry points; the exact SDK code remains available.
Individual queries continue to use `Query`.

The shared eligibility state stores the first `NativeError` from `Upscale`,
`GenerateReactiveMask`, `Tuning` or `ConfigureContextDebugLegacy`. The first call
returns `Error::Native(failure)`; subsequent live calls return
`Error::ContextUnusable(TerminalCause::Native(the_same_failure))` before input
inspection or native entry. No later failure/default request overwrites it.
Ordinary query/input/inspection failures retain today's non-poisoning policy.
Errors from the setup's null-context `ConfigureDebug` and
`ConfigureGlobalDebugLegacy` routes do not poison individual contexts. A
`ConfigureContextDebugLegacy` error remains terminal for the receiving context,
even though the call may have global effects. Creation/destruction and exceptional
dependency retention keep their existing policies.

U10 supplies this cause representation and internal retention. The subsequent
U11 implementation extends the non-exhaustive `TerminalCause` with
`RecordingAbandoned` and exposes `usability()` / `mark_recording_abandoned()`;
see [D042 §5](../adr/D042.md#5-expose-known-terminal-usability-without-claiming-gpu-readiness).
U10 itself did not expose those operations. Neither slice adds a recovery
operation, submission tracker or GPU-readiness assertion.
Checked request construction remains independent of context eligibility.

## 3. Metadata is data, not a capability

Accepted portable additions, retaining private fields and existing getters:

```text
ProviderId::from_u64(value: u64) -> ProviderId
ProviderId::as_u64(self) -> u64
impl From<u64> for ProviderId; impl From<ProviderId> for u64;
ProviderInfo::new(id: ProviderId, name: impl Into<String>) -> ProviderInfo

ResourceFlags::empty() -> ResourceFlags
ResourceFlags::from_bits_retain(bits: u64) -> ResourceFlags
impl BitOr / BitOrAssign / BitAnd / BitAndAssign for ResourceFlags;
```

All ID bits, including zero and `u64::MAX`, round-trip without validation or
normalization. `ProviderSelection::Explicit(ProviderId::from_u64(0))` remains
explicit; only `ProviderSelection::Default` requests default selection. Native
membership checks, no-fallback behaviour and post-create identity checking stay
in place. The existing live-query `ZeroProviderIdOnSuccess` invariant also stays:
arbitrary stored metadata is distinct from a valid live native result.

`ProviderInfo::new` owns the supplied string verbatim, allowing synthetic names,
duplicates and empty names. It establishes no runtime provenance, capability or
persistent identity. Native names still use immediate lossy UTF-8 copying.
Change output-only getter descriptions to describe stored metadata. Name-based
generation policy remains in optional `utils`; the maintainer selected its
0.3.0 inclusion on 2026-10-27, retaining opt-in activation.

Mask operations act on all 64 bits, including unknown bits; retain existing
`bits()`, `contains()` and `unknown_bits()`. Add no complement operator or known-bit
truncation. `ResourceRequirements` and `GpuMemoryUsage` keep their existing output
construction boundary. Overlapping required/optional bits remain representable;
absence from both masks does not forbid supplying an otherwise valid mask.

## 4. Migration and acceptance checks

This replaces the unreleased error shape without compatibility variants. Callers
match a category and inspect its payload; tests compare relevant reason/code
fields rather than requiring equality of the entire Error. Migrate production,
both owner families, fixtures, examples, consumer gates and current documentation
together; retain frozen records/producers as historical evidence. Raw ABI and
native validation/cleanup order do not change.

Verification checkpoints:

- Complete mapping of every current live error; exact unknown SDK/HRESULT bits,
  roles and source chains; public allocation count/reason accessors; portable
  error traits without Copy/Eq requirements.
- Both-family first-cause retention for upscaling, generation, tuning and live
  diagnostics; no later COM/SDK entry, ordinary query-error continuation and
  terminal cleanup. Distinguish global diagnostics and consuming destruction.
- Pure synthetic provider/resolver tests; zero/max ID round-trips; unenumerated
  rejection, fresh membership and selected-provider mismatch cleanup unchanged.
- Full-width resource-mask union/intersection and existing optional-mask policy.
- Supported stable/MSRV/consumer/example checks, backend dependency isolation and
  bounded native query/selection/recording/Configure regressions. U11 operations
  and exact-package checks were later completed in their
  [separate records](../release/2026-10-29-candidate-verification.md#shared-rc1-cross-host-qualification).

The reviewed contract fixes the six error categories and source ownership; operation precision
and first-cause retention; and whether the metadata additions are sufficient
without expanding output construction or native trust.

Current implementation references: [errors](../../crates/fsr-sdk/src/error.rs),
[eligibility](../../crates/fsr-sdk/src/context_policy.rs),
[query values](../../crates/fsr-sdk/src/query.rs),
[query mapping](../../crates/fsr-sdk/src/query/native.rs),
[DX12 inspection](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs).
