# Experiment: explicit 4.1.1 versus default 4.1.2 on the current driver

- Experiment and offline audit date: 2026-10-29.
- Question: does provider selection explain the shared historical-to-current
  GPU/context cost increase observed in P08?
- Outcome: **30/30 cases passed, eight complete provider pairs**. The GPU mean
  difference is +11.503 us (about +1.10%), with a descriptive two-sided 95%
  interval spanning zero. Default 4.1.2 * has longer measured context creation
  and destruction intervals than explicit 4.1.1 in this workload.
- Scope: fixed current-driver C++ provider/selection-route comparison, not a
  controlled old/new driver experiment, Rust comparison or release qualification.

## Setup and execution

The maintainer started the [prepared experiment](2026-10-29-exp-provider-performance-preparation.md)
in a separate terminal. The assistant found it already running and did not
launch a duplicate. It ran from **01:58:41 to 02:00:16 CET**, 95.477 seconds total;
the primary phase consumed 79.856 seconds. The run is
20261029T005841Z-experiment-provider-performance-03dce9970361 under ignored
target/provider-performance. The original run, logs, raw samples, generated probe,
readbacks, build artifacts and inventories remain unchanged there.

| Input | Recorded value |
|---|---|
| OS / architecture | Windows 11 build 10.0.26300, x64 |
| C++ build | MSVC 14.44.35207; Windows SDK 10.0.22621.0; /O2 /W4 /WX /MT /EHsc |
| GPU / driver | Radeon RX 9060 XT, adapter 0, 1002:7590; driver 32.0.32015.7029 |
| SDK | v2.3.0; pinned commit 60f4ea81909200d8542eca14dccb2628b763a9a3 |
| Explicit provider | 4.1.1, ID 17700776140664213505 |
| Default provider | 4.1.2 *, ID 17700776142811697154 |
| Render / output | 1280x720 maximum render; 1920x1080 output; SDR, dynamic-resolution flag retained |
| Clock frequencies | CPU QPC 10 MHz; GPU direct-queue timestamps 100 MHz |
| Repository source identity | e0bf8741d1e538d48e8ca15259faf28637c323381de4884368068b1fc65494e8 |
| Source plus SDK identity | c5b0246a772c8b62f6709dd530c400cd358b8b53f1d3dc951116aa09f21cdbcb |

The [compact evidence](2026-10-29-exp-provider-performance-comparison/measurements.json)
retains input/run/inventory/protocol/source/binary hashes, configuration, equipment,
all paired metric vectors, process means, availability/actual provider identities,
per-case times and readback hashes. Its adjacent
[inventory](2026-10-29-exp-provider-performance-comparison/artifacts.json) hashes
the retained normalized bytes. Personal paths/environment dumps and runtime
pointers are not exported; provider IDs, native hardware identifiers, return codes
and hashes retain their meanings. Permanent evidence contains no GPU images or
SDK binaries. Full raw evidence remains local, so exact future GPU outputs need
not be reproducible from the compact record alone.

The maintainer reported disabling the animated wallpaper before this run. This
is a reported condition, not an independently verified process/utilization
observation. Other workloads, CPU/GPU frequency state, temperatures and power
were not monitored. Game closure was not separately recorded. There was no
paired wallpaper-on/off test, and the driver was not changed for this experiment.

## Method and validity

The frozen protocol uses eight balanced pairs, seed 6102901, with opposite
provider order in each adjacent two-block group. One optimized independent C++
binary runs both arms in fresh child processes. Each arm enumerates providers;
explicit 4.1.1 selection is attached to both pre-create query and creation chains.
Every initial context must report the expected ID, with no fallback. Warm
creations reuse that same immutable chain; individual warm-context provider IDs
are not queried again.

Two four-frame ordinary debug controls precede 16 timed children. Each timed
child submits/fences 160 frames, excluding frames 0–31 from warmed CPU/GPU
means. This gives 128 warmed frames per process, **2,048 timed frame observations**
across the two arms, with **eight paired process differences** as inferential
units. Descriptor/scene inputs, active sizes, jitter/history, full input
fingerprints, adapter/driver and clock frequencies match across providers.
Both outputs independently satisfy the finite/write/fence/device-removal checks;
bitwise output equality is not required between provider algorithms.

Each timed child also records first-use/lifecycle intervals and eight unused
create/destroy pairs. Lifecycle estimates use eight paired process means, not
64 independent warm-context pairs. No pilot, trimming, early success, adaptive
extension or pooling with other runs. The 600-second primary allowance was not
reached. Passed status means valid complete execution; this probe selects no
numerical speed gate or release verdict.

Enumeration occurs before the pre-create render query, outside its timer. This
can warm startup/provider state compared with P08, which did not add the same
membership-query preparation to the independent C++ path. Query/first-use
absolute times therefore cannot be compared to P08 as if the startup sequences
were identical. This holds for both arms of the new matched experiment.

## GPU and warmed CPU recording

Values are us per recorded FSR interval/call, not complete frame time. Differences
are **default minus explicit 4.1.1**, with descriptive two-sided 95% Student t
intervals over the eight pairs. Bounds are pointwise and model-based; this is
not a prespecified equivalence or non-inferiority test.

| Metric | Explicit 4.1.1 | Default 4.1.2 * | Difference [95% interval] |
|---|---:|---:|---:|
| Warmed FSR GPU interval | 1047.245 | 1058.747 | +11.503 [-5.392, 28.398] |
| Warmed C++ CPU call | 52.295 | 52.849 | +0.554 [-4.454, 5.562] |

**Verified observation:** both intervals span zero; no clear GPU/CPU difference
is established by this small sample. The GPU mean difference is about 1.10%,
substantially smaller than the historical P08-versus-P06 change. The GPU paired
differences in us are 11.092, 48.902, 16.525, 18.820, 16.562, -20.576, -5.883
and 6.581. Every value remains in the analysis.

Historical P06's maximum-render C++ GPU mean was 938.530 us under the older
driver/provider environment. **Even explicit 4.1.1 on the current driver measures
1047.245 us, about 11.58% above that historical value.** This is a cross-run
observation, not a controlled driver estimate: helper inputs, provider-selection
route, background/clock/thermal state and run dates also differ.

**Inference:** the default provider change alone does not explain the historical
roughly 10% GPU increase in this workload. A driver/environment/shared-helper
factor remains plausible; this run does not identify which one. An interval
spanning zero does not prove exact provider equivalence or generalize to other
resolutions, flags, HDR or content.

## Context and first-use costs

Values are ms; boundaries and differences follow the same eight-pair method.

| Metric | Explicit 4.1.1 | Default 4.1.2 * | Difference [95% interval] |
|---|---:|---:|---:|
| Loader acquisition | 0.768 | 0.713 | -0.056 [-0.145, 0.034] |
| Pre-create render/provider query | 39.242 | 41.875 | +2.634 [1.240, 4.027] |
| First create after query | 53.555 | 58.127 | +4.571 [3.728, 5.415] |
| Warm unused create | 54.471 | 57.596 | +3.125 [0.985, 5.265] |
| First dispatch CPU recording | 10.024 | 10.286 | +0.262 [-0.278, 0.801] |
| Used destroy after fence | 5.107 | 9.442 | +4.335 [4.003, 4.667] |
| Unused destroy | 3.586 | 7.679 | +4.093 [4.002, 4.185] |

**Verified observation:** query/create/destroy difference intervals are positive
in this tested path; the first-dispatch and loader intervals span zero. The
provider/selection route changes context costs much more clearly than warmed
GPU time here. Unused destruction is about 2.14 times as long for the default
provider. These are whole native-call/context boundaries, not isolated internal
allocator or driver operations.

The earlier P08 default SDR observations were around 65.65 ms warm creation and
13.57 ms unused destruction, compared with 57.60/7.68 ms for the current default.
Consequently, provider identity alone cannot explain all absolute cross-run
variation. The reported wallpaper change is a possible environmental contributor,
but enumeration/startup preparation and uncontrolled state also differ. This
experiment cannot quantify a wallpaper effect or assign all of that variation
to the game, wallpaper or driver.

## Offline audit and reproduction

The complete native run inventory was rehashed and verified, including all raw
readbacks. All 18 native logs were independently reparsed through the existing
input/clock/output validators; sample matrices and all nine paired metric sets
recompute exactly. All actual context IDs and the explicit-provider availability
checks match the frozen protocol. Current repository and SDK file hashes still
match captured inputs, including the SDK-labelled external inputs. No code or
protocol changed during execution; no new producer changes were made during
this audit/record assembly.

From the repository root, the documented maintainer command was:

```powershell
python tools/verification/run.py run experiment-provider-performance --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --output-root target/provider-performance
```

Running it again starts a fresh native attempt; no additional run was selected
by this analysis. The retained configuration identifies the actual runner inputs;
terminal keystrokes/shell history were not separately captured. Recheck the full
local run while its original files remain available:

```powershell
python tools/verification/run.py verify-run --run target/provider-performance/20261029T005841Z-experiment-provider-performance-03dce9970361
```

The [preparation record](2026-10-29-exp-provider-performance-preparation.md)
retains the compile-only failure/correction, 11-case successful build and 65
mocked unit checks. This record adds actual device/provider selection, valid
fenced GPU dispatch and complete paired measurements. It changes no library or
probe code, driver state, accepted API or release decision. Topic synthesis,
P08 disposition and final archive qualification remain separate follow-ups.
