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
