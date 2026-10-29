# Checked tuning request verification

- Date: 2026-10-27.
- Scope: U09 under [D042](../adr/D042.md), wrapper 0.3.0 / sys 0.1.2,
  AMD SDK v2.3.0, Windows x64/MSVC + DX12, Rust 1.88+.
- Result: **U09 implemented and verified within the recorded configuration**.
  A fresh 557/557-case confirmation passes with unchanged captured inputs and
  independently verified artefact integrity.

## Implemented contract

Portable `TuningRequest::new(parameter, value)` is the sole parameter-domain
validator. It returns the existing `Error::InvalidTuningValue { parameter }` for
non-finite/out-of-domain explicit values, preserving every accepted bit including
negative zero and subnormals. The five domains are unchanged: velocity and
accumulation-added `[0,1]`, reactiveness and shading-change finite non-negative,
minimum-disocclusion `[-1,1]`. Explicit `ProviderDefault` remains a null native
value pointer, without substituting numerical defaults.

The request has private fields, `parameter()`/`value()` accessors and
Debug/Clone/Copy/PartialEq, without Default, mutation or unchecked construction.
It is available through `upscaler` and `upscaler::common` without backend features;
worker preparation requires no runtime, device or Windows object. Reusing a value
for a different parameter requires another constructor check.

Both owner families expose only `request_tuning(request)`. The old raw-pair
signature is removed. Native mapping consumes the checked request and checks
entry eligibility before descriptor construction/FFI; it does not duplicate the
domain rules. Complete desired lists can be collected before any native mutation.
No wrapper-side list storage, batch operation, sorting, coalescing, rollback or
transaction is introduced. A later native error can follow earlier
acknowledgements and remains terminal with the exact Configure code.

Pure construction errors are independent of context eligibility, including when
preparing data for an already terminal context. At operation entry every checked
request still rejects with the existing first-poison policy, without FFI. A checked
request neither probes support/application/effective values nor establishes GPU
idleness. Acquisition trust, synchronous descriptor/scalar consumption, common
native coordination, completion/no pending executable recording before Configure
and terminal destruction/exceptional dependency retention remain unchanged.
U10/U11 error and terminal-status changes remain separate.

The renderer recovery example now collects the complete list before applying it;
the fenced frame and movable examples accept checked requests. Development package
README and Rustdoc describe the same route. Root README remains published 0.2.0.
The setup/recovery guide and current topic summaries describe the new boundary;
historical research/release records and frozen spike producers are unchanged.

## Controlled and consumer coverage

Portable integration tests cover all five domains, immediately adjacent rejected
boundaries, NaNs/infinities, exact accepted bits, explicit defaults, cross-parameter
reuse and worker preparation. Shared mapper tests cover every key/tag/zero field,
literal scalar/null pointers and synchronous acknowledgement. Both-family public
fixtures prove that an invalid final desired entry prevents every earlier native
change, duplicates and caller order are retained, and a later native error stops
iteration without rollback or reset-based recovery. Existing tests retain query
failure continuation, both poison origins, stable owner storage, ordinary cleanup
and failed-destroy retention.

External consumers compile complete-list staging and canonical calls on both
families. Negative controls require private fields/construction, no Default,
rejection of unchecked values and rejection of the removed two-argument methods.
The native tuning workload uses checked public requests for all five explicit
values/default resets, followed by recording, submission, fenced completion and
cleanup on both families. Acknowledgement remains distinct from application.

## Commands and results

From the repository root, using already installed SDK/toolchains:

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
python tools/verification/run.py run windows-upscaler-followup --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0 --require-passed --output-root target/tuning-request-verification
python tools/verification/run.py verify-run --run target/tuning-request-verification/20261027T130451Z-windows-upscaler-followup-9835a6bd6997
python tools/verification/repository_size.py
```

## Equipment and input identity

Current Rust/Cargo 1.99.0, exact MSRV 1.88.0, Python 3.14.1, Windows 11 build
26300, MSVC 14.44.35207 and Windows SDK 10.0.22621.0. Native execution uses AMD Radeon RX 9060 XT, adapter 0 (`1002:7590`), driver
32.0.32015.7029, ordinary DX12 debug validation with GBV disabled. Native preparation verifies
the pinned v2.3.0 headers and valid AMD Authenticode signatures. Staged SHA-256:

| DLL | SHA-256 |
|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The final confirmation's combined repository/SDK identity is
`8796577fa55ab965b7c09d1a6a93d2eedbbb8e788f290cf7e65454996665aa22`;
repository-only identity is
`a77e21c57eaa2f4bff98d27d798eeef38071350a2939d5800466783ec760b4f4`.
The coordinator inventories the repository inputs plus four SDK headers and both
DLLs. Results refer to captured bytes rather than a required Git revision.

## Preparation failures

Earlier attempts remain failed, with complete recorded outcomes under
`target/tuning-request-verification`:

- `20261027T122642Z-windows-upscaler-followup-d349e5109fd6`: restricted Windows
  temporary-file access causes infrastructure cleanup and nested compiler writes
  to fail with OS error 5. Complete README consumers also expose the code-block
  issue below. Native regression prerequisites do not pass; no GPU acceptance.
- `20261027T122920Z-windows-upscaler-followup-65542d25f860`: repeated with the
  necessary local permissions. Infrastructure, all seven current/MSRV test
  configurations and individual external consumers pass, but eight complete
  example configurations fail. A newly added README code block was accidentally
  selected as the full integration snippet and included a Rustdoc-only hidden
  `# Ok` line. This was a documentation/example failure, not a native rejection.
  Native regressions remain blocked by their source gate.

The README now describes staging and links the complete renderer example;
its original full integration block is again selected. No production, test,
example or consumer source changed between the second and final confirmation;
within the captured repository inventory, only package README text changed.
Both earlier attempts retain their original
input identity `6ffa0e4e7fca7b74bf92e0fdaf42bb382a5910520bd90d0163eb09a68f3a3151`
and unchanged captured inputs. The fresh confirmation passes independently below.

## Final confirmation

`20261027T130451Z-windows-upscaler-followup-9835a6bd6997` passes **557/557**
required cases with `inputs_unchanged: true`. `run.py verify-run` independently
passes the complete outcome/artefact inventory. The final repository source
identity is unchanged from the captured inputs.

| Check | Result and boundary |
|---|---|
| Targeted Tuning tests before the matrix | 8/8 passed: mapper, both-family public staging/lifecycle and loaded-owner retention |
| Current Rust | Seven supported feature configurations pass check/test/Clippy/docs |
| Exact Rust 1.88.0 | The same seven configurations pass all-target checks and complete tests |
| External consumers/examples | Both compiler versions pass portable values/traits, private construction, canonical both-family full-list calls, removed API rejections, dependency/backend gates, expected Vulkan errors and full integration examples |
| Independent MSVC ABI | Pinned common, creation, dispatch and query assertions pass; no raw declarations changed |
| Native Configure | Both families, all five parameters, 20 explicit-value/default acknowledgements followed by 20 fenced upscale frames and ordinary teardown |
| Native regression | Queries, lifecycle, selected provider, scalar/geometry/mask/HDR dispatch, temporal/semantic controls, both movable modes and three sharpening integration repetitions plus independent readback audit pass |
| Final maintenance | Rust formatting, UTF-8/LF, changed Markdown file links, diff whitespace and repository size checks pass |

Native Configure creates five contexts per family at 320×180 render / 640×360
output. Explicit values are velocity/reactiveness/shading-change `0.5`,
accumulation-added `0.25` and minimum-disocclusion `0.0`. Each explicit request
is followed by record/submit/fenced completion, then a default request and another
completed frame before teardown. The live provider reports `4.1.2 *`, opaque ID
`0xf5a5ca1e81001002`, for all ten contexts. The name is an observation, not an
attribution of all internally loaded provider code or a capability guarantee.
These results establish acknowledgement and continued bounded integration;
numerical application, persistence and effective defaults remain unproven.

U10–U12 remain open. No final package/archive acceptance,
universal provider support, numerical application, GPU-readiness, performance or
image-quality claim follows from this work.
