# Context usability and recording abandonment verification

- Date: 2026-10-27.
- Scope: authorised U11 under [D042 §5](../adr/D042.md#5-expose-known-terminal-usability-without-claiming-gpu-readiness),
  wrapper 0.3.0 / sys 0.1.2, pinned SDK v2.3.0, Windows x64/MSVC + DX12,
  Rust 1.88+.
- Result: **U11 implemented and verified within the recorded configuration**.
  All 593/593 required source/MSRV/consumer/ABI/native cases pass, including the
  two-family native abandonment extension. Artefact integrity passes and captured
  inputs remain unchanged.

## Implemented contract

Both `Upscaler` and `MovableUpscaler` expose `usability() -> ContextUsability`
and safe `mark_recording_abandoned()`. The portable snapshot is also reexported
from `upscaler::common`: non-exhaustive, Debug/Clone/Copy/PartialEq/Eq, with no
Default. Its variants are `Usable` and `Unusable(TerminalCause)`.
`TerminalCause::RecordingAbandoned` has a descriptive Display and no native source;
`Error::ContextUnusable` retains the cause in its standard-error source chain.
No package/version, dependency, SDK, raw ABI or Send/Sync change is introduced.

| Earlier state | Event | Later state |
|---|---|---|
| Usable | Local validation, inspection or ordinary query error | Usable |
| Usable | Native upscale, generation, tuning or live legacy diagnostic failure | Unusable, exact first native operation/code |
| Usable | Renderer marks abandonment | Unusable, RecordingAbandoned |
| Unusable | Further mark or attempted live use | Same first cause; no revival or replacement |

Status and marking inspect no COM objects, change no reference counts, make no
native call, wait for nothing and release nothing. The existing eligibility
store remains authoritative; the public snapshot is derived, not a second state
machine. Marking changes only that store. All live record/query/tuning/context
diagnostic and context-dependent preflight routes share the terminal guard before
inspection/FFI. Status and consuming destruction remain available; Drop and
explicit destruction retain their single-attempt and failed-destroy retention
policies. Global setup operations do not gain per-context poison.

The renderer must mark every context affected by an abandoned recording,
including generator use. Marking does not invalidate an executable list or
complete submitted GPU work. Retain contexts, resources and allocator dependencies
through their respective invalidation/completion duties before terminal cleanup.
A copied Usable snapshot may become stale and certifies no readiness, native
capability, recovery or callback quiescence. The recovery example marks before
fallible invalidation/completion callbacks, leaving the old owner terminal if a
callback fails. Root README remains the published 0.2.0 entry point.

## Checks and reproduction

Controlled tests exercise both families: successful recordings by two contexts on
one list, explicit marking of each, stale snapshot semantics, repeated marking,
local inspection-error continuation and every blocked live route. Deliberately
failing COM inputs and invalid query inputs cannot bypass the guard. Fixture
create/destroy/dispatch/query/Configure counts, format-inspection counts and COM
reference balances stay unchanged across status/marking and rejected live use.
Explicit and Drop cleanup each attempt destruction once after abandonment;
success releases dependencies while a failed destroy retains the device bundle.

The portable first-cause matrix covers all four mutating native origins and
known/unknown/max codes, marking after native failure and attempted mutation after
abandonment. Error source tests distinguish native failures from local abandonment.
Five additional downstream compile cases cover portable traits/reexports,
non-exhaustive matching, absence of Default and both-family status/mark/cleanup.
Existing source, mapping, ownership, error, callback and feature checks remain.

The existing native Configure integration case additionally runs one completed
frame and one successfully recorded but unsubmitted frame per family. It marks
the latter twice, captures status and rejected recording/query results, invalidates
the pending recording **before** assertions/destruction and retains textures and
allocator through SDK teardown. Only the earlier frame is submitted/fenced; the
abandoned frame is never submitted. Ordinary DX12 validation remains enabled.
This is bounded successful-native-recording/renderer-abandonment integration,
not an induced native/device failure or a context-reuse experiment.

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/context-usability-verification
python tools/verification/repository_size.py
```

Package listing with `cargo package --list -p fsr-sdk --allow-dirty --offline --locked`
includes the new source module, tests, recovery example and licence texts. This
is a content-list check, not final archive acceptance.

Preparation also ran Cargo tests with `movable,utils`, Clippy for all supported
features and no-default features, and warning-free workspace Rustdoc. The first
restricted full test run hit a temporary-compiler write denial. An initial new
test injected QueryInterface failure on an object that this inspection path did
not query; its assertion correctly failed. The test now injects GetDevice failure
on the command list, verifies that ordinary preflight rejects it, then proves the
terminal guard takes precedence. Targeted final tests pass. These preparation
failures are not accepted passes.

## Captured confirmation

Accepted attempt:
`20261027T144330Z-windows-upscaler-followup-09a14640c965` under ignored
`target/context-usability-verification/`. It ran 2026-10-27 14:43:31–14:52:48 UTC,
completed 593/593 required cases with status `passed`, and reported
`inputs_unchanged: true`. `verify_run` separately verified the complete required
inventory, result agreement and artefact hashes. Reproduce the integrity check:

```powershell
python tools/verification/run.py verify-run --run target/context-usability-verification/20261027T144330Z-windows-upscaler-followup-09a14640c965
```

| Identity | SHA-256 |
|---|---|
| Repository-only source | `948404973f4c312ccf394ccc6392d9c0241a103db91376646a3ba61eb9f5d086` |
| Captured repository plus SDK source | `4105408b89539b92489654c9471acc641fafcf538ba8a82cf7bcb28a83a0e4ef` |
| Run inputs, including configuration/environment | `0d757f9f7be6019900966fcdc8ec6b30d656078c37e27a7dd694843f7a1ea5d2` |

The matrix covers supported default/movable/no-default/utils combinations,
warning-free formatting/Clippy/Rustdoc, Rust 1.88.0, positive/negative external
consumers and examples, verification infrastructure/archive integrity, independent
pinned-header ABI checks and existing query/lifecycle/dispatch/sharpening native
regressions. The new five consumer cases pass on both stable and MSRV. Native
`native.dispatch.configure` exits 0 and emits, for each of `local` and `movable`,
`Recording abandonment: PASSED`, `completed_frames=1`, `abandoned_frames=1` and
`invalidated_before_destroy=true`. Its original five-parameter/two-phase Configure
integration also passes. No failed full matrix attempt is accepted or relabelled.

Repository text/link checks and `git diff --check` pass. The repository-size check
reports 41.89 MB against the existing 50 MB budget; generated native artefacts
remain under ignored `target/`.

## Equipment and boundaries

Rust/Cargo 1.99.0, exact MSRV 1.88.0, Python 3.14.1, Windows 11 build 26300,
MSVC 14.44.35207 and Windows SDK 10.0.22621.0. The local SDK v2.3.0 headers are
pinned at `60f4ea81909200d8542eca14dccb2628b763a9a3`. Both staged AMD DLLs have
valid AMD Authenticode signatures; SHA-256:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

Native query and Configure cases observe adapter 0 AMD Radeon RX 9060 XT
(`1002:7590`), feature level 12_0 and driver 32.0.32015.7029, with GBV disabled.
Both abandonment paths observe provider display name `4.1.2 *`, opaque ID
`0xf5a5ca1e81001002`. A display name is neither general capability detection nor
provenance for every internally loaded module.
No new format, signal-domain, generator-provider, allocator or callback behaviour
is introduced. Earlier [U10 format/callback evidence](2026-10-27-error-metadata-verification.md)
keeps its original source identities and limits; those supplementary profiles are
not rerun or attributed to this source. U11 neither changes their native mapping
nor establishes rollback, abandoned-context reuse, recovery from device loss,
callback-target reclamation or GPU readiness. Final coherent migration and exact
candidate-archive verification remain U12/release work; nothing is published.
