# Experiment preparation: provider 4.1.1 versus current default

- Date: 2026-10-29.
- Question: on the current unchanged Windows/DX12 driver, do explicit provider
  4.1.1 and default 4.1.2 * differ in GPU and context costs under one C++ workload?
- Outcome: **equipment prepared and compilation verified; native timing not
  started**, under the maintainer's explicit request to run it manually later.
- Observable success: both providers available, actual initial context IDs match,
  debug controls pass, identical descriptors/scene inputs/clocks/frame history,
  all eight balanced pairs complete and produce valid fenced readbacks/telemetry.
- Failure: missing provider or unexpected default ID, selection/query/create/
  dispatch failure, invalid/mismatched inputs, incomplete samples, device removal,
  watchdog, changed source, or missing outcomes. A speed difference is a separate
  descriptive finding; there is no numerical acceptance gate.

## Baseline and scope

The [P08 supplemental review](../../release/2026-10-29-performance-supplement-review.md)
observes shared historical-to-current GPU/context cost increases in C++ and Rust.
Its old/new driver versions are 32.0.31041.1004 and 32.0.32015.7029; provider names
also differ, 4.1.1 versus 4.1.2 *. The staged signed SDK DLL hashes are unchanged.
Existing [October 27 verification](../../release/2026-10-27-upscaler-setup-verification.md)
observed both explicit 4.1.1 selection and the new default. Those observations
justify trying this comparison; preparation does not re-establish availability.

This probe holds the currently installed driver and C++ consumer fixed while
changing provider selection. It does not roll back or install a driver, change
the production API, repeat the large P08 CPU confirmation, or claim image quality,
FPS, GPU equivalence or universal hardware support. No research synthesis,
roadmap completion or release acceptance is inferred from preparing it.

## Protocol and probe implementation

[provider-performance-protocol.json](../../../tools/verification/provider-performance-protocol.json)
fixes eight paired fresh processes, seed 6102901, adjacent two-block arm-order
balancing, a single maximum-render SDR workload and 1920x1080 output. Each timed
child retains the existing 160 frames and excludes the first 32 from the primary
GPU/CPU estimate. Lifecycle mode records load, pre-create provider/render query,
first creation, first dispatch and used destruction, plus eight subsequent unused
create/destroy pairs per child. Warm contexts reuse the same immutable override
chain; the initial context identity is explicitly verified per process.

The [generator/coordinator](../../../tools/verification/provider_performance.py)
copies the hash-pinned independent
[C++ reference](../../../crates/fsr-sdk/tests/native/performance_reference.h)
into the fresh ignored run directory. It adds enumeration, selection and identity
checks; the original consumer and timing boundaries remain unchanged on disk.
The explicit arm uses enumerated ID 17700776140664213505 in both pre-create query
and creation chains. The default arm supplies no override and must report ID
17700776142811697154. Missing/unexpected providers fail without fallback.
Enumeration occurs outside the recorded load/query/create intervals.

Both arms use the same optimized binary, resources, queried sizes, jitter,
scene inputs, history, helper, GPU queue and driver within the run. Independent
input/output/fence audits and two ordinary debug controls precede the 16 timed
children. Output hashes are retained but need not match across different provider
algorithms; descriptors, input samples/full input fingerprints and clocks must
match. Retained raw data supports further diagnosis without deleting outliers.

Estimates are equal-weight means of eight paired process differences, **default
minus explicit 4.1.1**, with descriptive two-sided 95% Student t intervals. Frames
within each process are not treated as independent. No pilot, trimming, adaptive
sample count, early success or optional extension. The primary phase is limited
to 600 seconds, checked before starting another child; each child has a 120-second
watchdog. Build/debug work and a bounded final child can extend whole-batch time.
Provider-specific output quality and the causal effect of a driver change remain
outside this experiment even after successful execution.

## Preparation checks and provenance

The [retained checks](2026-10-29-exp-provider-performance-preparation/checks.json)
identify both local compilation attempts, source/input/run/inventory hashes,
probe source/binary hashes, compiler command/outcome and the frozen protocol.
Paths are normalized; no environment dump, secret, runtime address, SDK binary
or GPU image is exported. Provider IDs, hashes and native status codes retain
their values. Supporting JSON uses UTF-8/LF.

The first compilation attempt completed preparation but failed the probe build:
MSVC /W4 /WX detected C4456 because the added enumeration count shadowed an
existing inner count. The probe-only variable was renamed; the original C++
reference and all production sources remain unchanged. The second fresh
compile-only attempt passed **11/11 cases**:
20261029T005235Z-experiment-provider-performance-build-4f1d2c85d4da.
The failed attempt is preserved as
20261029T005150Z-experiment-provider-performance-build-065771824a3e.
Both remain under target/provider-build-check. Retained inventories were verified.

These checks acquire the local x64 MSVC environment, verify pinned headers and
AMD signatures, compile four ABI probes, the existing helper and the generated
C++ program. The successful build uses /O2 /W4 /WX /MT /EHsc. Neither profile
creates a device or runs the produced executable. **Zero GPU child processes were
started.** CPU/provider timings, hardware state and runtime override behaviour
therefore remain unverified until the manual run.

**65/65 focused unit tests passed**. Tests cover fixed balanced order, precise query/creation adaptation, reference
drift rejection, missing pairs, eight-process inference, input/equipment matching
with allowed cross-provider output differences, provider mismatch rejection,
watchdog and the separate build-only graph. Native launches are mocked in those
tests. The test suite also exercises the shared profile registry and existing
performance orchestration. Build and unit-test provenance concern the new probe,
not a new P08 qualification of the release candidate. Documentation added after
compilation does not alter the tested generator, protocol or native inputs.

## Manual execution and reproduction

From the repository root, with the game and competing heavy work closed:

```powershell
python tools/verification/run.py run experiment-provider-performance --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/provider-performance
```

Plan roughly **3–5 minutes**, based on earlier child-process durations; this is
not measured runtime for this new probe. Keep the terminal/machine running and
the checkout unchanged. About 300 MB of readbacks are expected; reserve 1 GB.
The chat need not stay active. Send the printed run.json path afterwards,
including on failure. analysis/provider-performance.json stores the estimates;
logs, samples, frozen protocol/order and source identity remain with that fresh
run. A new invocation starts a new attempt rather than resuming or overwriting.

Preview without native work:

```powershell
python tools/verification/run.py plan experiment-provider-performance
```

Reproduce preparation only, without launching the GPU consumer:

```powershell
python tools/verification/run.py run experiment-provider-performance-build --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/provider-build-check
```

Unit checks from tools/verification/tests:

```powershell
python -m unittest test_provider_performance test_release_performance test_performance_characterization test_performance_reconfirmation test_infrastructure
```

The initial test invocation could not write sandbox TEMP files; repeating with
normal temporary-file access succeeded. All process launches in these unit tests
are simulated. No dependencies were added or downloaded. Driver state and the
production library remain unchanged; actual provider/GPU findings are pending.
