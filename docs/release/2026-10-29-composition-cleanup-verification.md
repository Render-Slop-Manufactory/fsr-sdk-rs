# Composition and documentation cleanup — 2026-10-29

**U15 is implemented and targeted checks pass.** Final release qualification
remains a separate batch on the finished candidate, following the
[verification selection guide](../verification-selection.md).

## Changes

The private DX12 descriptor builder now borrows `FrameParameters` directly,
alongside converted resources and validated geometry. The positional tuple and
separately repeated sharpening, exposure, colour-hint and motion-scale arguments
are removed. History reset and debug visualization read their named settings.
Resource inspection, COM guard ownership and validation/error ordering before
descriptor construction are unchanged; the recorded source comparison checks
that prefix independently. The raw ABI and public operation signatures are unchanged.

The test-only methods on `Upscaler`/`MovableUpscaler` and fake inherent resource
constructors are removed. Across controlled and compiled native workloads,
98 recording, 54 upscale-preflight, 15 generator-recording and 18 generator-preflight
calls now use the production API directly. Twenty-nine resource constructions
use named builders. Pure `UpscaleTestInputs`/`ReactiveMaskTestInputs` remain as
fixture data holders with the existing parameter factories.

The migration audit covers 39 changed test files. Workload numeric/string/bool
literals, function names, test counts, ignore attributes and raw dispatch calls
are preserved, excluding the deliberately removed adapter declarations. Complex
input expressions are evaluated once; simple bindings are passed directly.
Existing assertions and provider/workload selections remain intact. Native test
files, including deferred experimental workloads, were migrated for compilation;
their experiments were not run or reclassified.

Rustdoc now distinguishes fixed maximum allocations from smaller active render
and output regions. The package README and migration guide describe the same
dynamic-resolution, padding-write and provider obligations. Generator limits
also include the already recorded default `4.1.2 *` no-write observation.
Current examples
already use named builders and `record`; the changed crate's all-targets checks
compile them for both ownership families. The released root README remains at
0.2.0 under its maintenance policy.

## Targeted verification

The [machine-readable record](2026-10-29-composition-cleanup-verification.json)
retains complete command arguments, exit codes, test summaries, log hashes,
source hashes, migration counts and the performance-impact assessment.
Execution used Windows x64/MSVC, current Rust 1.99.0 and MSRV 1.88.0, with cached
locked dependencies, incremental compilation disabled and debug information off
for dev/test builds. Rustdoc warnings are denied.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| Affected-crate all-targets Clippy, `movable,utils` | Passed with warnings denied |
| MSRV all-targets compilation, `movable,utils` | Passed, including updated native workload sources |
| Affected-crate library tests, `movable,utils` | 137 passed; 58 opt-in tests remain ignored |
| Portable dimensions/frame-parameter integration tests, no-default + utils | 11 + 3 passed |
| Rustdoc, `movable,utils` and no-default + utils | Both passed with warnings denied |

All seven selected commands pass. Controlled coverage includes descriptor field
mapping, motion/colour/sharpening/exposure, independent diagnostic flags,
preflight/recording inspection, terminal policy, both ownership families and
callback/cleanup fixtures. No new test expectations were substituted for failures.
Logs remain under ignored `target/composition-cleanup-checks`; the complete
argument lists and environment in the JSON reproduce the checks from this source.

## Performance impact and release boundary

The same checked values and mapping branches are read inside the builder instead
of staged tuple/scalar arguments. The change adds no allocation, clone, cache,
COM/SDK operation or synchronization. The P08 workload still calls public
`record` directly; its equivalent resource-builder expansion remains outside the
timed interval. Input generation, native selectors, repetition and criteria are unchanged.

This is a source-level impact assessment plus controlled descriptor evidence,
not proof of identical optimized machine code or timing. No timing experiment
is selected or run by U15. Final source/archive reconciliation against the closed
P08 evidence remains required under D043, repeating affected measurements if
warranted. The final candidate also needs its complete source/package/native/
platform qualification and publication simulation. No AMD GPU profiles, full
infrastructure/CI sweep, final archive qualification or publication ran here.

The [consolidated verification report](2026-10-29-candidate-verification.md)
remains the release evidence hub. Earlier records retain their original source
and archive identities; this checkpoint does not silently qualify new shipping bytes.
