# P08 preparation and manual handoff

- Date: 2026-10-27.
- Result: **unattended adaptation prepared; no native measurements executed**.
- Authority: maintainer selected reuse of P06/P07 and its proposed 5 us CPU
  criterion, then requested a manual start with later analysis in chat.
- Contract/start command: [release performance design](../design/release-performance.md).

## Implementation

The new release_performance.py coordinator reuses existing P06 pilot, selection,
confirmation and analysis without changing that protocol or historical records.
A separate checksum-pinned P08 protocol adopts the five-workload CPU criterion
for 0.3.0 and adds three existing P07 setup processes. The Rust timed arm calls
public record directly; input assembly remains outside the bracket. P07 already
uses the current setup API, preserving fresh membership and its whole-path limits.

The batch packages or selects both candidate archives, checks Cargo's source
inventory and exact packaged source bytes against the checkout, records archive
hashes, and verifies source/SDK agreement across passed stages. Measurements use
the matching checkout; normalized Cargo manifest/lock/VCS metadata and exact
extracted-package execution remain final qualification work. Candidate mutation,
stage failure, changed source/SDK or incomplete evidence cannot look completed.
A completed CPU result can still be inconclusive/exceeds-budget and always
requires review. The coordinator never publishes or approves a release.

## Checks performed without measurements

- `cargo clippy -p fsr-sdk --all-targets --features movable,utils --offline --locked -- -D warnings`:
  passed, including compilation of the adapted ignored timing producer.
- From tools/verification/tests:
  `python -m unittest test_release_performance test_performance_characterization`:
  **21 tests pass**. All native/profile launches are mocked. Coverage includes
  serial stage order, failure stopping, source/archive/SDK changes, exact setup
  sample completeness, candidate source/inventory mismatches, budget pinning and
  separation of execution completion from release approval.
- Read-only `python tools/verification/release_performance.py plan`: passed;
  writes the planned protocol under ignored target/ when redirected, no GPU work.
- Actual offline no-verify workspace packaging/source matching, through the new
  candidates function: passed without native execution. Ignored evidence folder:
  target/release-performance-preparation/20261027T212207Z-candidate-check-e33231ebad79.
  It checks 157 wrapper and 30 raw source files, excluding Cargo-generated metadata.
  Wrapper draft SHA-256 is
  326e163efa02292cf2390767a2f8019818b958064cbac331cb51192c0c0febf9;
  raw draft SHA-256 is
  05fba7838b4c947b5ee9950e97ac1b2aa0091150057959e2072b29a682e549a5.
  These are preparation previews, not qualified final archives.

The first unit-test invocation hit restricted temporary-directory writes; the
permitted repeat also exposed a test-fixture CRLF/LF mismatch. Fixture writes
now explicitly use UTF-8/LF, so exact-byte matching tests the intended invariant.
The subsequent 21-test run passes. A preparation-only status-print edit initially
used the wrong working directory and changed no source; it was applied from the
repository root. These are corrected preparation issues, not timing results.

## Deferred execution

The maintainer runs the documented single command in a separate terminal, leaves
the machine awake and checkout/SDK unchanged, and sends the printed status.json
path after completion or failure. No agent needs to poll during the batch.
The previous P06 run took about 93 minutes. P07, packaging/build and supplemental
work add time; the two-hour primary-confirmation cap is not a whole-batch limit.

Source/format/link checks and repository-size checks complete preparation only.
Native timing/execution, CPU outcome, descriptive regression review and final
package qualification remain pending. No historical samples or prior API/native
verification result is relabelled as a current P08 measurement.

## First manual start and path correction

The maintainer started the batch
20261027T213002Z-release-performance-25cd5afe953c. All three setup processes
pass; the P06 pilot then fails at characterization.build.cpp before timed work.
MSVC reports C1083/Invalid argument for its generated output. The nested object
path is 265 characters long. The failed batch and its setup records remain
unchanged; they are not a completed P08 result.

The coordinator now creates the P06 batch beside the P08 control folder under
the same output root, rather than nesting two timestamped batches. The existing
status/log references retain the association. A regression assertion requires
that shallow dispatch output root. All 21 orchestration/statistics unit tests
pass after the correction.

An actual compile-only check then passes all 11 selected toolchain/ABI/SDK/helper,
Rust and C++ build stages. Its object path is 218 characters even with a longer
preflight output-root name. Evidence is under ignored
 target/release-performance-build-check/20261027T213504Z-native-performance-a660390d4dda/runs/20261027T213504Z-experiment-performance-pilot-6dd0921abfc1.
No native measurement cases were selected or executed by that check. The
maintainer can restart the same documented command; it creates a fresh batch.
