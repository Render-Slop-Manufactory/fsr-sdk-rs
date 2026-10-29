# D32/D32S8 depth-buffer verification

- Date: 2026-10-22, Europe/Berlin; coordinator names use UTC `20261021`.
- Contract: accepted [D037](../adr/D037.md), development `fsr-sdk 0.3.0` /
  `fsr-sdk-sys 0.2.0`, pinned AMD SDK v2.3.0, Windows x64/MSVC + DX12.
- Status: implementation and bounded verification complete. All 178 stable CI,
  155 Rust 1.88.0/MSRV and 82 native verification cases pass with unchanged inputs.

## Behaviour and native safety

Both public ownership families accept `R32_TYPELESS`, `D32_FLOAT`,
`R32G8X24_TYPELESS` and `D32_FLOAT_S8X24_UINT` as depth-buffer allocations.
The exact DSV/depth SRV pair is fixed by the real allocation. D32S8 reads depth
plane 0 as `R32_FLOAT_X8X24_TYPELESS`; stencil is ignored. Typed allocations
require OPTIONS3 fully typed casting support. DSV texture/depth-stencil and
SRV texture/load/sample checks occur after canonical device membership, shared
by optional preflight and compulsory recording. Native format/usage match AMD's
public helper, including typeless READ_ONLY usage and typed depth/stencil bits.

Missing depth capability, shader-read denial, UAV-capable depth, missing device
bits and query failures reject before FidelityFX entry and preserve usable
eligibility. Diagnostics retain `Depth`, exact HRESULT and operation, including
the added `CheckFormatCasting` operation. Existing native recording failures
retain the terminal context/list policy. There is no format selector, implicit
copy, resource retention, conversion or barrier in production.

The caller owns DSV production, depth conventions/content, actual provider
support, lifetime, residency, submission and synchronization. Both D32S8 planes
must enter shader-read together; concurrent writable stencil is outside this
contract. Other roles and baseline formats retain their validation behaviour.

## Equipment and reproduction

Windows 11 build 10.0.26300, adapter 0 AMD Radeon RX 9060 XT, vendor/device
`0x1002/0x7590`, feature level 12_0, driver `32.0.32015.7029`. Native child logs
report the adapter and driver, independently of earlier records. Rust/Cargo
1.99.0 and separately selected Rust 1.88.0; MSVC 14.44.35207, Windows SDK
10.0.22621.0, Python 3.14.1 and PowerShell 7.6.7.

SDK v2.3.0 is commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. Four staged
headers match [the pins](../../tools/native-headers.json). Staged loader/provider
SHA-256 values are `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa`
and `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
AMD Authenticode signatures are Valid. The runs use the local SDK without a
download or new strict online revocation check. Selected providers are 4.1.1 /
`17700776140664213505` and 3.1.5 / `17700776140660019205`; live identity must match.

```powershell
python tools/verification/run.py run ci --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --require-passed
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-depth-formats --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

CI/MSRV also cover concurrent development of the separately accepted D036
utility; the depth feature introduces no utility dependency or release policy.
The profile/equipment is in [the depth protocol](../../tools/verification/depth-formats-protocol.md).
Every result must retain unchanged source inputs and pass artefact inventory
verification; individual case success alone cannot pass an altered-input run.

## Outcomes and retained failures

| Profile | Final attempt | Result |
|---|---|---|
| Stable CI | `20261021T235901Z-ci-054a0ced2603` | 178/178 passed |
| Rust 1.88.0/MSRV | `20261021T235656Z-msrv-2485d8b11839` | 155/155 passed |
| Native depth formats | `20261021T235845Z-native-depth-formats-21fc088b276b` | 82/82 passed |

All three finalized runs have `inputs_unchanged: true` and pass `verify-run`'s
complete artefact inventory checks. Their 273 shared source files have identical
hashes. CI/native additionally inventory the six pinned SDK inputs. Stable CI
includes independent MSVC ABI, seven feature configurations, Windows controlled
fixtures, public consumer boundaries, formatting, Clippy, documentation and full
examples. Its infrastructure suite runs 263 tests with four explicit optional
skips; the depth evaluator's negative controls pass. MSRV repeats tests/checks,
consumers and examples on exactly Rust 1.88.0.

Native verification comprises ten preparation/build cases, 64 paired public/C++
workloads and eight independent sensitivity/equivalence analyses. Each paired
workload submits/fences three frames per route: 384 sequences, including 24
no-dispatch controls and 360 actual upscaler dispatches. All four allocation
formats pass standard/reversed depth and active-render/output shrink/grow through
both owners on both selected providers. DSV-produced depth passes 288 exact
shader-sample readbacks; 144 separate stencil-plane readbacks preserve `0x5a`.
Post-dispatch depth copies also match the CPU oracle exactly. Typed casting is
reported supported on this adapter. Every active output is finite/written,
ordinary diagnostics/device-removal counts are zero, and terminal cleanup passes.

All 192 per-frame public/C++ full-output comparisons are byte-identical. The
96 fixed-geometry depth-family/R32F baseline comparisons also match exactly;
changed-colour sensitivity and unchanged no-dispatch controls pass for each
owner/provider group. This certifies the recorded mappings and integration,
not a general numerical equivalence contract for arbitrary scenes/providers.

The [compact result](2026-10-22-depth-format-results.json) retains run/input
identities, the complete shared source hash inventory, SDK pins, per-workload
measurements, native view/casting records, depth/stencil readback hashes and
earlier failures. Its UTF-8/LF bytes are 804,971 bytes, SHA-256
`060e3f9c79776a4d03cb5309387a405b0355389d722f1d22ce1a9488b40ba5b1`.
Full raw uploads/readbacks and logs remain ignored under `target/verification`.

An initial helper build (`20261021T234615Z-native-depth-formats-6118b724bf99`)
failed `/WX` on a signed/unsigned byte-count comparison. Correcting the unsigned
literals fixed compilation; no GPU workload ran in that attempt.
The next attempt (`20261021T234651Z-native-depth-formats-1d920f4df8eb`) passed
all 82 case evaluations but failed overall acceptance because verification
source inputs changed while it ran. This failure remains failed. Final dynamic
geometry includes active-render and output shrink/grow.

The first frozen MSRV attempt (`20261021T235235Z-msrv-6d2f63961659`) passed
154/155 evaluations with unchanged inputs. `consumer-msrv.feature-disabled`
incorrectly returned success and compiler output naming a different source
(`UpscaleResources` instead of the supplied `MovableRuntime` import). Another
MSRV attempt overlapped it using the shared compilation cache. This is consistent
with cache interference, not evidence that movable leaked through feature gates;
the failure remains recorded and required a separate complete rerun.

The corrected-verifier MSRV rerun above passes all 155 cases, including that
negative feature-gating check, with unchanged inputs.

Concurrent development subsequently changed the shared verifier to give every
compiler snippet an immutable source file and distinct binary target. Fresh
CI/MSRV/native runs use that corrected verifier and identical shared source
inputs. The preceding native attempt
`20261021T235200Z-native-depth-formats-5c95fec98125` passed all 82 evaluations but
is rejected for that input change, as is CI attempt
`20261021T235219Z-ci-9a8c9fd60fe4`. These integrity failures are not GPU failures
and do not count as accepted runs.

Standalone evaluator tests initially could not write temporary files inside the
sandbox; the identical unrestricted rerun passed all four tests. An earlier
invocation from the repository root lacked the required module search path;
running from `tools/verification` resolves the test imports. Neither is a GPU
or provider failure.

## Limits

This is bounded native integration on one GPU/driver configuration and two
selected providers. No universal depth/provider certification, native access
bound, untouched output padding, stencil consumption, image-quality,
performance, GBV or final-package claim follows. D16/D24, BGRA/sRGB and other
remaining U05 families remain separate work. Final extracted release archives
and publication are not part of these checks.
