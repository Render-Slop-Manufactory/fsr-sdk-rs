# Code Heuristics

These repository-local heuristics apply to the Rust bindings and wrappers for
AMD's FSR SDK.
Readability and correct domain contracts matter more than mechanical rules.

## Functions and Data Flow

- A function should represent one coherent, nameable operation.
- Prefer small, idiomatic Rust public APIs without engine or renderer coupling.
- Design from the caller's perspective so high-level flow remains readable.
- Do not mix orchestration and low-level mechanics without good reason.
- Make dependencies, inputs, outputs, units, side effects, ownership, and
  failures visible.
- Pass only the dependencies an operation needs.
- Separate descriptor construction and validation from runtime loading, native
  calls, and I/O where that makes the code easier to reason about.
- Prefer return values over surprising mutation.

## Types and Contracts

- Use types for units, states, and invariants that can actually be guaranteed.
- Verify graphics assumptions, especially coordinate systems, jitter, motion
  vectors, depth, and scaling.
- Names should describe meaning, effects, and expectations.
- Prefer free functions when privileged access is unnecessary.
- Use iterators or named algorithms when they clarify intent. Direct loops are
  valid when they make the operation clearer or avoid unnecessary work.

## Native Boundaries

- Keep `fsr-sdk-sys` faithful to the pinned AMD API: preserve names, layouts,
  integer widths, calling conventions, and pointer contracts. Rust API design
  preferences must not change the ABI.
- Put ergonomic types, validation, ownership, and error conversion in
  `fsr-sdk`. Preserve native error codes when translating failures.
- Make an operation safe only when the wrapper can uphold its safety contract.
  Document caller obligations for unsafe APIs and justify unsafe blocks with
  the invariants that make them valid.
- Make runtime, context, descriptor, and resource lifetimes explicit. Keep the
  library loaded while its functions or objects are in use, and use RAII for
  owned native objects.
- Do not equate Rust lifetimes with GPU completion. Establish resource states,
  synchronization, and thread-safety requirements from the SDK contract before
  exposing safe dispatch or implementing `Send` and `Sync`.
- Check ABI assumptions against pinned headers; a successful Rust build alone
  does not verify native compatibility.

## Direct Provider Controls and Wrapper Policy

Make it clear who chooses a value or behavior: the caller, the native provider,
or an explicit wrapper policy. Use this distinction when it clarifies a real
choice; do not add speculative modes or prefix every SDK-facing type with Native.

- Use `Native` for direct provider controls when that distinction matters, as in
  the sharpening API discussion. Validate the documented input domain, then
  preserve the parameter's meaning and value without hidden clamping, tuning or
  output filtering. Native does not mean raw ABI access, unchecked inputs,
  experimental maturity, or a quality level.
- Keep a direct mode's meaning stable if wrapper-selected profiles are added.
  Such profiles may still use the native implementation internally; distinguish
  the selection policy rather than implying a different execution backend.
- Give wrapper policies explicit names, documented transformations and bounded
  evidence for their intended outcomes. State the tested provider/configuration
  and workload limits. Correct mapping, successful execution, observed effects
  and image-quality claims require distinct evidence, including for native modes.
- Prefer more precise provenance names where appropriate: caller-supplied inputs,
  provider recommendations and wrapper-selected settings are different contracts.
  Units, camera conventions and resource roles usually need domain names, not a
  Native label. There is no universal Native-versus-Validated split: direct
  provider inputs are validated too.
- Introduce an extensible mode selection only for a justified API distinction.
  Future variants still need their own resource, lifetime, synchronization and
  compatibility review; an open enum alone does not make new capabilities
  additive. Review public traits as part of that compatibility contract.

This is design guidance, not acceptance of a feature or release gate. Concrete
contracts remain in the relevant [decisions](DECISIONS.md); the current
[D014 proposal](adr/D014.md) remains subject to its own review.

## Performance

- Prefer static dispatch in hot paths when it improves performance without
  unreasonable build-time or code-size cost.
- Avoid hidden allocation, cloning, dynamic dispatch, copying, locking,
  conversion, and GPU/CPU synchronization.
- Measure consequential choices. Generics are not automatically zero-cost.

Consequential zero-cost, performance, or allocation claims require generated
code, profiling, or benchmark evidence as appropriate.

## Judgment

Small functions, purity, strong types, and abstraction are tools, not goals.
Keep an abstraction only when it improves the contract, testability, or caller
experience without unacceptable runtime or maintenance cost.
