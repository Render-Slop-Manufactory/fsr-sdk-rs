# Milestone 5 plan: public DX12 construction

- Recorded: 2026-09-23
- Status: implementation pending; D008 accepted, no implementation performed
- Basis: [accepted D008](adr/D008.md)

## Verification of the supplied proposal

The repository supports most of the supplied premises, with these qualifications:

| Premise | Repository finding |
|---|---|
| M4 is a usable lifecycle foundation | Confirmed in production source and recorded Windows execution; public construction remains absent. |
| One runtime can serve two contexts | Confirmed sequentially by the raw M5 probe, not yet by a shared production owner; no concurrency guarantee follows. |
| The provider survives destruction of A while B lives | The staged DLL remained resident and B remained queryable. Its presence does not identify all participating driver-provider code or prove module ownership. |
| Zero dimensions must be rejected | Confirmed: all four zero-component children aborted before native return. The current private constructor already takes `NonZeroU32`. |
| Further numerical limits can be delegated to the runtime | Selected by D008's bounded trust assumption, not proved as an ordinary-error guarantee. The three boundary cases returned native errors, but establish no universal numerical domain. |
| `!Send + !Sync` is a sufficient concurrency boundary | Only within one ownership family. Independently acquired objects can share native global state; D008 explicitly addresses external/native integration at unsafe acquisition. |
| Query/Configure must be added in M5 | No unconditional prerequisite was found; defer those public APIs. |
| D005/D007 need revision | No contradiction found. Preserve their cleanup, trust and deployment policies. |

This verification reconciles repository source, manifests and existing records;
it does not rerun native experiments or independently re-audit every upstream
citation. No new source research or research-record import was performed.

## Implementation sequence

1. **Apply the accepted numerical contract.** Reject zero before FFI and
   delegate further numerical limits under D008's explicit runtime/provider
   trust assumption. Introduce no arbitrary maximum or promise of ordinary
   errors for all unsupported sizes. Document the actual provider scope and
   the requirement to revisit the contract on contradictory safety evidence.
   No further numerical research or spike is a prerequisite. During
   implementation, align the private constructor's safety documentation with
   this policy rather than silently retaining its old supported-size obligation.

2. **Runtime ownership.** Replace the inline runtime alias in
   `crates/fsr-sdk/src/lib.rs` with a small Windows/DX12 runtime module. Retain
   `FfxLibrary` in private `Rc` state and adapt `context.rs` to retain a strong
   reference per context. Keep the host fixture adapter viable. Preserve cleanup
   order and exceptional retention; expose no raw entry points or safe adoption
   of a raw loader. Document the acquisition-thread and external-access contract.

3. **Construction and device retention.** In `upscaler.rs`, add checked dimension
   inputs and the concrete safe constructor selected by D008.
   Prefer a small options value over a builder unless implementation reveals a
   concrete need. Keep flags zero, mandatory API-version construction and pinned
   descriptor storage internal. Borrow the public device and clone/AddRef one
   independent reference per attempt; review/reexport the exact `windows` type.
   No new dependency or sys ABI slice is expected.

4. **Errors and documentation.** Extend `error.rs` only as needed for explicit
   Rust validation failures; preserve native `u32` codes, operation identity,
   invariant failures and separate load errors. Document the public dependency
   version, runtime trust, thread confinement, retention and lack of dispatch.
   Update README/roadmap status only when implementation actually exists.

5. **Compile-time and fixture checks.** Assert both public owners are neither
   Send nor Sync and contexts are neither Copy nor Clone; include positive
   compilation controls. Test that zero and any subsequently established invalid
   domain inputs never enter native create. Inspect the generated chain, tags,
   version and fixed flags through fixtures. Verify two contexts retaining one
   loader, dropping the public runtime first, destroying A before B, and final
   release after dependent cleanup. Preserve all D005 error/retention tests,
   including a failed sibling create and a sibling destroy failure. Verify the
   borrowed-device path retains an independent reference and keeps it on
   exceptional paths. Fixtures prove wrapper behavior, not AMD concurrency or
   safe numerical rejection.

6. **Windows-native verification.** Adapt the M4 runner or add a bounded M5
   runner using the public production path, one trusted staged runtime and a
   real device. Create A/B sequentially, drop the public runtime and caller's
   device reference, destroy A, identify B using test-only instrumentation, then
   destroy B. Check loader retention and record provider identity separately
   from module residency. Cover explicit destroy and ordinary Drop. Run valid
   representative dimensions from the successful baseline; reject zero in Rust
   without forwarding it. Keep each native scenario isolated and time-bounded,
   and record SDK/DLL fingerprints, OS, GPU, driver, returns and limits. No
   malformed native inputs, concurrency stress or Dispatch is part of this plan.

Use the applicable locked workspace checks and Windows fixture execution from
[development guidance](DEVELOPMENT.md). Reuse paired native ABI checks for the
existing descriptors; extend them only if ABI changes actually become necessary.
Keep host compilation, Windows compilation, fixture policy and real native
lifecycle evidence distinct. Passing checks do not prove D008's numerical trust
assumption or a universal provider support domain.

## Review boundary

D008's ownership, device API, fixed flags, unsafe integration obligations and
bounded numerical trust assumption are accepted. No substantive maintainer
decision remains open for this construction scope. Acceptance removes the
numerical research gate; it does not implement M5 or authorize starting it in
this documentation task. M6 remains separate.
