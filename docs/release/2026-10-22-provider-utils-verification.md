# Provider resolver implementation verification — 2026-10-22

The maintainer accepted D036 and authorised implementation after review. The
development `fsr-sdk 0.3.0` implementation adds the off-by-default `utils` feature,
portable generation-name resolver, standard resolver error traits and examples.
No dependency, raw ABI, native default, loading or safety contract changes.
Release inclusion remains separately unselected.

## Implementation and inputs

- Pinned AMD SDK v2.3.0; the wrapper and raw development versions are `0.3.0`
  and `0.2.0`. The existing local SDK supplies headers to independent ABI checks.
- Resolver: `crates/fsr-sdk/src/utils/provider.rs`; metadata remains private in
  `query`, with tests in `query/provider_selection_tests.rs`.
- The parser accepts exactly three ASCII-decimal `u32` components and major
  2/3/4, including leading zeros. Pure tests cover malformed names, numeric
  boundaries, all explicit choices, no fallback, order-independent unique results,
  repeated/distinct IDs, exact ambiguity counts and snapshot-only diagnostics.
- Windows fixtures keep `selected-fixture` unrecognised, add a separate
  version-shaped enumeration arm and apply the resolved ID through public options
  and context creation for local and movable owners. Existing selected-route
  error, mismatch and terminal-cleanup tests remain enabled.
- CI/MSRV now cover seven feature configurations, including utils-only and both
  DX12 ownership combinations. Downstream consumers check feature-off imports,
  public errors/propagation, Linux/Windows production dependency graphs and the
  complete provider-choice example. docs.rs enables both `movable` and `utils`.

## Commands and results

Equipment: Windows 11 build 10.0.26300, `x86_64-pc-windows-msvc`, Rust/Cargo
1.99.0, exact Rust/Cargo 1.88.0 and Python 3.14.1. Run from the repository root
with cached inputs; profile execution supplies `--locked --offline` and
`RUSTDOCFLAGS=-D warnings`. No downloads or native GPU execution were requested.

```powershell
python tools/verification/run.py run ci --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run consumers --require-passed
python tools/verification/run.py run consumers --toolchain 1.88.0 --require-passed
cargo fmt --all -- --check
python -m unittest discover -s tools/verification/tests -p test_native_contract.py -v
python -m unittest discover -s tools/verification/tests -p test_infrastructure.py -v
```

| Check | Observed result |
|---|---|
| Stable Rust matrix | All 28 check/test/Clippy/doc cases pass across default, movable, no-default, no-default+movable, utils, movable+utils and no-default+utils |
| Rust 1.88 matrix | All 14 all-target check/full-test cases pass across the same seven configurations, including Rustdoc examples |
| Resolver and public paths | All six new pure tests pass in each utils configuration; the new Windows enumeration → resolver → options → creation fixture passes locally and with both ownership families under movable+utils. Existing D028 absent-ID/native-error/mismatch/cleanup regressions pass |
| Fresh stable consumers after harness correction | **141/141 passed**, exit 0; inputs unchanged |
| Fresh Rust 1.88 consumers after harness correction | **141/141 passed**, exit 0; inputs unchanged |
| Focused formatting and helper-export recheck | Formatting passes; both native-equipment contract tests pass |
| Verification infrastructure after harness correction | **29/29 passed** with temporary-directory access |

The consumer profiles cover public resolver/error imports and boxed-error
propagation, feature-off compile errors with and without native features, absence
of native APIs in portable configurations, normal Linux/Windows dependency graphs,
all-declared-feature compilation, and complete examples under `dx12,utils` and
`dx12,movable,utils`. The resolver's portable Rustdoc example also passes with
no default features. docs.rs feature metadata includes `movable` and `utils`.

### Initial failures and corrections

The first broad CI attempt (`20261021T234529Z-ci-f23846bac0cf`) remains **failed**:
175/177 cases pass. Parallel depth-format work changed shared inputs, and the
infrastructure helper-export check observed an export before its Rust consumer
was present; formatting also observed an unfinished depth-format test edit.
Both focused checks pass after those edits progressed. All four independent ABI
probes and the stable Rust matrix pass, but the changing-input attempt is not
reclassified as a clean whole-source verification.

The initial MSRV attempt (`20261021T235141Z-msrv-22ffb0003dbf`) likewise remains
**failed**, with 149/155 cases passing and changed inputs. Six unrelated negative
consumer controls returned exit 0. Their logs reused earlier snippets' diagnostics
without compiling the new source: rewriting one `src/main.rs` rapidly allowed
Cargo's timestamp freshness checks to retain an earlier successful result.

The consumer harness now writes each snippet once to its own source file and
compiles a distinct binary target. Existing expected diagnostic codes and
positive/negative criteria are unchanged. The fresh stable attempt
`20261021T235636Z-consumers-4d270498629b` and exact-1.88 attempt
`20261021T235654Z-consumers-2d8976dedcd1` both pass every case with unchanged inputs,
including all six previously stale controls. This fixes verification equipment;
no core public type or error contract was changed to satisfy a negative test.

An initial standalone utils-only test run passed all six new pure tests but an
existing compiler-boundary fixture could not write its temporary `.rmeta` under
the sandbox. The identical unrestricted rerun passed: 62 unit tests, 15 integration
tests and three Rustdoc examples. This environmental failure is not a resolver
failure. The focused infrastructure command also encountered sandbox temporary
directory setup/cleanup denial; its identical unrestricted rerun passed all 29
tests. Profile checks use access to the compiler's temporary files.

## Limits

These checks distinguish pure resolver behaviour, Windows compilation and
controlled fixture execution from native AMD runtime/GPU execution. Actual
non-Windows compilation/test execution was not performed: portable utils-only
tests ran on Windows, with Linux dependency isolation checked through Cargo's
normal target dependency graph. No new GPU
workload or compatibility campaign is selected or run for this CPU-only utility.
The [existing Windows provider evidence](2026-10-19-windows-upscaler-verification.md)
remains bounded historical evidence. Names and IDs establish no generation,
capability or persistent-identity guarantee. Release archives, publication and
utility release inclusion are outside this implementation checkpoint.
