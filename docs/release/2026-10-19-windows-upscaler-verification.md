# Windows upscaler verification — 2026-10-19

The prepared Windows follow-up passes **256/256 cases**. A subsequent native
batch, adding explicit-provider GPU dispatch for both ownership families,
passes **31/31 cases**. These development-source checks confirm the pending
D028/D029/D031/D032 Windows fixtures, independent MSVC ABI and bounded native
integration. They do not identify final release archives or publish packages.

Reactive-mask results are provider-specific. Both public families pass all
generation checks on **3.1.5**. Providers **4.1.1** and **2.3.4** return OK but leave
the seeded masks unchanged; those write criteria and their complete profiles
remain **failed**. See the [generator experiment](../research/records/2026-10-19-exp-reactive-mask-providers.md).

## Equipment and tested source

- Windows 11 build 10.0.26300, `x86_64-pc-windows-msvc`, DX12 debug layer.
- Adapter 0: AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
  feature level 12_0; driver `32.0.31041.1004`.
- Rust/Cargo 1.99.0, plus exact Rust 1.88.0; Python 3.14.1, NumPy 2.5.4,
  PowerShell 7.6.7, MSVC 14.44.35207, Windows SDK 10.0.22621.0.
- Development `fsr-sdk 0.3.0` / `fsr-sdk-sys 0.2.0`; SDK v2.3.0 at source pin
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Native preparation verifies four pinned header hashes, the two exact DLL
  hashes below and Valid AMD signatures. No downloads or strict online
  revocation verification were performed.

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The initial worktree was clean. The first complete batch tests the implemented
provider selection, tuning domains/default reset, dispatch preflight, public
generator fixtures and composition example. Subsequent changes only extend
test equipment and profile registration: explicit-provider dispatch, raw/public
generator route selection and exact-provider generator cases. No production
library implementation, native contract, write oracle or acceptance threshold
changed. Baseline revision `99954b20ca23fe358123f1a3da48b5eca759fc66` is supplemental;
the [retained checks](2026-10-19-windows-upscaler-verification/checks.json) include
source hashes, commands, execution results, native identities and original
run/inventory hashes. All eight finalized attempts report unchanged inputs.

## Commands and outcomes

All commands use cached Cargo inputs. For each profile below, run from the root:

```powershell
python tools/verification/run.py run PROFILE --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The two acceptance batches and all three timing runs additionally use
`--require-passed`. Generator experiments retain their failed exit status
without changing criteria.

| Profile / attempt suffix | Outcome | Scope |
|---|---|---|
| `windows-upscaler-followup` / `190550Z-…-96eeb6cb907a` | 256 passed; exit 0 | ABI, infrastructure, stable/MSRV, consumers/examples, native regressions and Configure |
| `native-upscaler` / `191248Z-…-d830f18ff224` | 31 passed; exit 0 | Rebuilt native equipment; explicit-provider dispatch added |
| `experiment-reactive-mask` / `190616Z-…-dc6d38abae54` | 12 passed, 17 failed; exit 1 | Raw/default 4.1.1; unchanged masks |
| `experiment-public-reactive-mask` / `191405Z-…-10a5c42341f6` | 13 passed, 34 failed; exit 1 | Both public families/default 4.1.1; unchanged masks |
| `experiment-selected-reactive-mask` / `191537Z-…-fa3b0033962a` | 49 passed, 34 failed; exit 1 | All 36 children on 3.1.5 pass; 2.3.4's 34 generation children fail |
| `experiment-provider-setup`, three fresh attempts | 11 passed each; exit 0 each | 192 whole-path CPU timings; no GPU dispatch |

The infrastructure suite ran 241 tests: 237 pass, with three Proton checks and
one portable symlink check skipped on Windows. Stable and Rust 1.88.0 execute
all four feature configurations, public boundary consumers and complete examples.
Controlled fixtures establish generator mapping, validation, poisoning and cleanup;
they do not establish actual-provider writes.

The first attempted follow-up (`190424Z-…-b3e24723b93a`) encountered sandbox
`PermissionError` failures in Python temporary directories and was interrupted.
Its local partial record is not finalized and supplies no acceptance result.
The complete unrestricted rerun above replaces no historical evidence.

## Native integration and remaining limits

Provider 4.1.1 (`17700776140664213505`) passes selected/default setup and live
queries, lifecycle, dispatch, exposure/depth/motion/active-region/supplied-mask/HDR
regressions, movable handoffs and three sharpening integration repetitions. The
independent audit accepts all 144 retained readbacks and 78 comparisons.

All five tuning parameters pass value → dispatch/fence → provider-default →
dispatch/fence for both families: 20 completed frames. This proves native
acknowledgement and integration, not numerical application or default restoration.
Explicitly selecting the observed default ID also passes two submitted/fenced
frames per family, with live identity checked before and after. The generator
experiment adds explicit 3.1.5/2.3.4 selection and GPU consumers, retaining their
different write outcomes. It establishes no general provider-support matrix.

The [setup measurement](../research/records/2026-10-19-exp-provider-setup-cost.md)
records per-process median six-query costs of 229.55–230.74 ms for default and
696.98–707.10 ms for selected routing. No cache or session design is selected.

After adding the probes, stable Clippy with movable/all targets and Rust 1.88.0
all-target compilation pass. Eight generator-evidence tests, complete ordered
profile inventory, source-failure GPU blocking and source/package native-coverage
controls pass. The compilers report an incremental-cache hard-link warning and
copy files instead; compilation succeeds. Final documentation checks are separate.

Full runs, logs, executables and existing readbacks remain under ignored
`target/verification`; sanitized exports remain under `target/verification-evidence`.
The retained compact reports contain the observations needed to understand these
results. Runtime addresses and local paths are normalized. No image-quality
study, new native platform, final archive verification or publication is claimed.
