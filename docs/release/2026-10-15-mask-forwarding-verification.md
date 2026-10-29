# Supplied-mask forwarding correction — 2026-10-15

The amended D025 validation is implemented. On the same RX 9060 XT / FSR 4.1.1
configuration that previously failed before AMD entry, supplied-mask dispatch
now passes all six native combinations and 18 GPU frames. No provider override,
bypass mode, native SDK modification or native acceptance criterion change was
needed. This is development-source integration evidence, not proof of mask
consumption, image-quality benefit or final release-archive acceptance.

## Change and regression coverage

`check_mask_requirements` now rejects missing required masks without treating
required/optional query membership as permission to supply SDK-defined masks.
All supplied resources still undergo ordinary metadata/device/identity checks;
resource states, backing storage, residency and completion remain caller
obligations under unsafe dispatch. Unknown required-input rejection, native
error preservation, poison policy and terminal cleanup are unchanged. The public
`UnsupportedResource` variant is retained for compatibility and no longer emitted
solely because a mask is unlisted. No raw ABI, manifest or dependency changed.

The portable requirement test covers four required masks × five optional masks
(including zero, overlap and unknown bits) × four supplied combinations. Windows
controlled fixtures execute the same matrix under explicit/automatic exposure
for both local and movable owners. They inspect exact native pointers, R8_UNORM
metadata, maximum allocations, active extents, null descriptors and reset values,
with no per-frame query. Missing-required failures still prevent native entry.

With `optional=0`, injected COM identity failure still produces the precise
inspection error, balances references and permits a valid retry. Wrong format,
extent, shape, shader access, device and duplicate identity still reject supplied
masks. Native-error poisoning and poisoned live-query behaviour also execute
with unlisted supplied masks. This establishes forwarding without bypassing
ordinary inspection.

Before editing production validation, the updated focused tests failed twice
with `UnsupportedResource(ReactiveMask)` instead of the expected forwarding.
After the fix, both pass with the same command; the native test remains ignored
in this ordinary Cargo invocation and runs separately through the coordinator:

```powershell
cargo test -p fsr-sdk --lib --features movable --locked --offline mask -- --nocapture
```

The compiler reported that incremental-cache hard linking fell back to copying;
the post-fix compilation and tests completed successfully. This warning is not
a GPU or test failure. The full coordinator uses its existing non-incremental
build configuration.

## Equipment and reproducible checks

- Windows 11 build 10.0.26300; x86_64-pc-windows-msvc.
- Rust/Cargo 1.99.0, with Rust 1.88.0 for MSRV checks.
- Python 3.14.1, NumPy 2.5.4, PowerShell 7.6.7.
- MSVC 14.44.35207; Windows SDK 10.0.22621.0.
- Explicit adapter 0: AMD Radeon RX 9060 XT, vendor `0x1002`, device `0x7590`,
  feature level 12_0, dedicated memory 16974905344 bytes;
  driver `32.0.31041.1004`.
- Pinned SDK v2.3.0, source commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- Provider `4.1.1`, ID `0xf5a5ca1e01001001`.

Commands run from the repository root, with cached Cargo inputs and the existing
local SDK. The Windows coordinator verifies four pinned headers, independent C++
ABI assertions, both DLL hashes and Valid AMD signatures. There is no SDK download
or new strict online revocation check. CI/MSRV run outside the restricted sandbox
under the existing authorization because earlier compilation fixtures encountered
temporary-file access denial there. This correction required no sandbox retry.

```powershell
cargo fmt --all
python tools/verification/run.py run native-upscaler --require-passed --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
```

## Native observations and limits

| Profile | Result | Evidence |
|---|---|---|
| Windows CI | All 115 cases pass | [Run](2026-10-15-mask-forwarding-verification/ci/run.json) |
| Rust 1.88 MSRV | All 98 cases pass | [Run](2026-10-15-mask-forwarding-verification/msrv/run.json) |
| Standard native-upscaler | All 29 cases pass; none blocked | [Run](2026-10-15-mask-forwarding-verification/native/run.json) |

CI includes format, four feature configurations of check/test/Clippy/docs,
public consumer/example/dependency checks and independent Windows C++ ABI probes.
The infrastructure suite runs 184 tests: 180 pass and four Linux/Proton or
privileged-symlink controls skip on Windows. NumPy and real PowerShell parser
checks execute. The [movable test log](2026-10-15-mask-forwarding-verification/ci/rust.movable.test.txt)
retains the corrected portable matrix and Windows public fixture results.

All three complete attempts pass `verify-run` and record unchanged inputs.
[Summary and attempt names](2026-10-15-mask-forwarding-verification/summary.json).
Source/configuration, toolchain, SDK and executable identities and exact commands
are in each directory's `inputs.json` and `cases.json`. After these attempts,
only status documentation and package README prose were updated; production code,
test assertions, selectors and protocols remain the verified bytes.

The resource query still reports `required=15`, `optional=0` for both ownership
families. Six fresh local-owner contexts supply reactive-only, composition-only
or both masks, each under explicit and automatic exposure. Each runs three
continuous frames with active render sizes `320×180 → 256×144 → 288×162`, fixed
`320×180` input allocations and `640×360` output. Reset is set only on the first
frame of each context.
[Public-query log](2026-10-15-mask-forwarding-verification/native/native.queries.public.txt),
[mask execution log](2026-10-15-mask-forwarding-verification/native/native.dispatch.supplied-masks.txt).

All 18 dispatches return native success, submit and fence successfully. Both
fixture mask textures pass exact uploaded-byte readback on every frame, ordinary
input readbacks/guards pass, and all 230400 output pixels per frame are finite
and replace the sentinel. Device removal/debug-error counts remain zero through
explicit context destruction and frame-resource cleanup. The fixtures allocate
both mask textures even when only one is supplied; their readbacks establish
fixture integrity, not which resources or texels AMD reads.

The complete standard native profile also executes public queries, utility
queries, lifecycle, prior dispatch regressions, both movable modes, three
sharpening integration repetitions and the independent 144-readback/78-comparison
audit. The supplied-mask GPU matrix itself uses the local owner; both owners'
exact mask forwarding and validation are covered by controlled Windows fixtures.
There is no new cross-thread provider guarantee, mask-consumption or quality
claim, non-linear colour validation, support expansion or publication.

The [earlier failed batch](2026-10-15-windows-feature-batch.md) remains evidence
of the original wrapper policy. The native workloads, selectors, markers, input
readbacks, output checks and watchdogs are unchanged for this rerun.

The [retained evidence inventory](2026-10-15-mask-forwarding-verification/artifacts.json)
hashes normalized UTF-8/LF summaries, plans, inputs, case records and selected
logs. Private paths are sanitized; DLLs, executables and raw diagnostics remain
under ignored `target/`. Full native reanalysis needs that local attempt or a
fresh run. These checks cover one GPU/driver/provider configuration and do not
establish universal native safety or mask consumption.
