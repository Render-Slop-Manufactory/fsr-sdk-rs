# D034 implementation and public native motion verification

- Date: 2026-10-21, Europe/Berlin. Run-directory timestamps use UTC and therefore
  start with 20261020 for these checks after local midnight.
- Contract: [accepted D034](../adr/D034.md), including the reviewed mixed-sign
  scale / changing-jitter addition. Development wrapper `0.3.0`, sys `0.2.0`.
- Outcome: **129 stable CI cases, 112 Rust 1.88.0/MSRV cases and 102 public
  native cases pass**. Native execution comprises 252 dispatched and 12 control
  frames; all 56 encoding/sensitivity criteria pass. A separate five-case bounded
  package audit passes after retaining the workspace-verifier failure below.
- Limits: first U05 slice only. Broader formats, general numerical/access bounds,
  arbitrary-thread native claims and final release verification remain separate.

## Implemented behaviour and source identity

`NativeMotionVectorScale` is portable, checked and reexported from `upscaler`.
It accepts every finite pair, including zeros, negative zero, subnormals and
extreme finite values, preserving component bits. NaN/infinities reject X first,
then Y. No default/unchecked public constructor or new native ABI layout exists.
The frame's optional override is literal and takes precedence over retained
UV/pixel units; clearing it restores their current active-region mapping.

Both ownership families share compulsory inspection and descriptor construction.
Only main motion adds typed RG32F to RG16F; generation and other role formats
stay unchanged. Actual allocation metadata/usage is preserved. RG32F alone checks
device texture/load/sample support and typed UAV-view support when its actual
allocation has UAV capability. Missing bits or query HRESULTs reject before
FidelityFX entry without poisoning. View support does not certify provider
support or require typed UAV shader loads/stores for a read-only role.

Source is identified by the retained per-file [native inputs and commands](2026-10-21-native-motion-verification/native-checks.json),
with source digest `df61d25ef14cfe11ee4a1088745c3760448780c7414291ebef58a43e649628e1`
and input/configuration identity
`e5abc087d4247d4bc0c011c3b2c821552da5aa8b09349f0a51f4a8d39e62a317`.
The initial worktree held the existing D034 proposal, its review addition and
roadmap/decision links; this task accepted and implemented those choices.
[Source changes](2026-10-21-native-motion-verification/source-changes.json)
identify modified baseline files. Full source-input lists also identify new
modules, helpers and tests. Research records and the released-version root
README remain byte-identical. Every finalized attempt reports unchanged inputs.

Production changes are in [common values](../../crates/fsr-sdk/src/upscaler/common.rs),
[frame inputs](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs),
[inspection](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/resources.rs),
[descriptor mapping](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs),
[diagnostics](../../crates/fsr-sdk/src/error.rs) and the selected raw constant.
The [controlled public fixtures](../../crates/fsr-sdk/src/upscaler/dx12/tests/native_motion.rs),
[native child](../../crates/fsr-sdk/tests/native/public_motion.rs),
[C++ inputs](../../crates/fsr-sdk/tests/native/public_motion.h),
[independent auditor](../../tools/verification/public_motion.py) and
[frozen protocol](../../tools/verification/public-motion-protocol.md) are test
equipment. Existing helper entry points and default RG16F behaviour are retained.
No dependency/version, SDK acquisition, licence or publication change is made.

## Actual native setup

| Equipment | Observed configuration |
|---|---|
| Platform | Windows 11 build 10.0.26300; x86_64-pc-windows-msvc; ordinary DX12 debug layer |
| GPU | Adapter 0, AMD Radeon RX 9060 XT; vendor `0x1002`, device `0x7590`; feature level 12_0 |
| Driver | `32.0.32015.7029` |
| Compilers | Rust/Cargo 1.99.0; separate Rust 1.88.0 checks; MSVC 14.44.35207; Windows SDK 10.0.22621.0 |
| Coordinator | Python 3.14.1; PowerShell 7.6.7 |
| SDK pin | v2.3.0, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; upscaler API 4.1.1 |
| Selected providers | 4.1.1 ID `17700776140664213505`; 3.1.5 ID `17700776140660019205` |

The driver differs from the earlier raw spike's `32.0.31041.1004`; comparisons
here are matched within this batch, not a controlled comparison between drivers.
Four headers match their normalized-text pin hashes. Both staged DLLs retain
Valid AMD Authenticode signatures; loader/provider SHA-256 values are respectively
`e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` and
`d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46`.
No SDK download or strict online revocation check was performed.

## Commands and required checks

```powershell
python tools/verification/run.py run ci --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
python tools/verification/run.py run msrv --require-passed
python tools/verification/run.py run native-motion-scale --native --require-passed --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX --adapter-index 0
```

| Finalized attempt | Exit and result |
|---|---|
| `20261020T225445Z-native-motion-scale-8162e657537e` | Exit 0; 102/102 passed, 22:54:45–22:58:44 UTC |
| `20261020T225445Z-ci-de0ec6223b9c` | Exit 1; 124 passed / five failed. Windows sandbox denied temporary-file writes in infrastructure and four Rust test configurations |
| `20261020T225744Z-ci-faec21e236ca` | Exit 0 outside the sandbox; 129/129 passed |
| `20261020T225918Z-msrv-d5510b549489` | Exit 0 outside the sandbox; 112/112 passed |

The failed sandbox attempt is preserved, not counted as acceptance. Stable CI
passes formatting, independent MSVC ABI, infrastructure/archive checks,
check/test/Clippy/docs for all four feature configurations, downstream diagnostics,
production dependency isolation and complete examples. Infrastructure runs 259
tests with four intentional skips. MSRV repeats all four check/test configurations,
consumers and examples using Rust 1.88.0. Controlled fixtures cover exact finite
bits, scalar rejection, override defaults/precedence/clearing, active geometry,
role rejection, metadata/device/usage checks, each missing format bit, HRESULT
preservation, no-new-query baseline routes, retry and poison precedence.

Before the finalized batch, targeted checks corrected a fixture call that
incorrectly assumed `ReactiveMaskOptions::default()` and one Clippy nested-if
finding. A targeted Python audit initially hit the same sandbox temporary-file
restriction and passed outside it. Compiler incremental-cache hard-link warnings
fall back to copying; they did not prevent the passed checks.

## Public GPU observations

Every group (two providers × two owner families) runs 21 dispatch workloads and
one equipment control, each with a fresh context over three serial frames.
There is no movable handoff or overlapping GPU execution. Render allocation is
320×180, output allocation 640×360; motion allocation follows render/display
resolution and retains maxima during active-output transitions.

The matrix includes RG32F pixel reference, positive `(2,4)` and mixed `(-2,4)`
scales, independent wrong-X/wrong-Y controls, RG16F constructor UV defaults,
jitter-inclusive reference/mixed/misencoded-jitter controls and UAV-capable
display motion with output 640×360 → 512×288 → 640×360. A clearing arm restores
pixel fallback. The included-jitter sequence is `(0.25,-0.25)`, `(-0.25,0.25)`,
`(0.125,-0.375)`. Both matched arms share actual jitter, context flags and history.
The whole declared pixel contribution, including previous-minus-current jitter,
is divided by the explicit scale before texture packing. The wrong jitter arm
deliberately omits that division for the jitter term. See the protocol for exact
binary32/half arithmetic, colour pattern and reset conventions.

All 252 dispatched frames replace the complete active output with finite values.
All 12 control frames preserve every seed. Input readbacks match independent
Python-generated bytes; actual resource descriptions match the pinned C++ helper.
Native scale/jitter bits, family/provider identity, format, flags and clear/default
choices match the independent descriptor oracle. GPU fences complete; device
removal, debug errors and input mismatches are zero. All 88 contexts complete
terminal destruction, including the four controls; no native error branch occurs.

[Measurements](2026-10-21-native-motion-verification/measurements.json) retain
all 264 frame descriptors, device capabilities, metrics and input/output hashes.
[Comparisons](2026-10-21-native-motion-verification/comparisons.json) retain all
168 paired frame results. All 56 group criteria pass: equivalent encodings match
in every compared frame; every sensitive wrong-axis/misencoded-jitter control
matches reset and changes at least one later output. RG16F UV defaults are
integration cells, not a precision-equivalence prediction against RG32F.

This demonstrates bounded public mapping/encoding consistency, including the
reviewed scale/jitter interaction. It does not prove native jitter cancellation
consumption, exact closed-provider arithmetic, reconstruction quality, RG32F
precision benefit, a numerical/access bound or untouched padding.

## Bounded package audit and retained failures

The [package audit driver](2026-10-21-native-motion-verification/package-audit.py)
uses the existing Cargo inventory and isolated-registry inspection. Copy it to
`target/native-motion-package-audit.py` before reproduction. It does not publish.

1. `python target/native-motion-package-audit.py` created
   `20261020T230135Z-native-motion-package-audit-b92c3a47a33c`. Inventories passed
   and both archives were emitted. Cargo 1.99's automatic workspace verification
   then reported `no hash listed for fsr-sdk-sys v0.2.0` from its temporary registry;
   the profile exited 1, registry audit blocked. This failure is not reclassified.
2. An explicit-archive retry, `20261020T230303Z-native-motion-package-audit-53eee1e2cca9`,
   exited 1 because the supplied paths omitted Cargo's `tmp-crate` directory.
   No archive was selected in that attempt. Corrected paths use the emitted bytes.
3. The final command below creates
   `20261020T230335Z-native-motion-package-audit-3b0965455339`: exit 0, five cases
   passed. The existing isolated registry audits every package/source/licence
   byte and generates its own verified index/lock. All 150 extracted source/licence
   files match the checkout; all targets compile with movable using that registry.

```powershell
python target/native-motion-package-audit.py --wrapper-archive target/verification/20261020T230135Z-native-motion-package-audit-b92c3a47a33c/package-build/package/tmp-crate/fsr-sdk-0.3.0.crate --sys-archive target/verification/20261020T230135Z-native-motion-package-audit-b92c3a47a33c/package-build/package/tmp-crate/fsr-sdk-sys-0.2.0.crate
```

Selected wrapper/raw archive SHA-256 values are respectively
`b731c4d53df1cc04385900fb182fa970dfbc7e85c54053d59c4f3b6dce113934` and
`28bbed6987ece6d80bbf8a8f670ba5e8d93536ec461368ec7dfeb7befff350f0`.
Project/AMD notices are unchanged and included; there are no packaged SDK
binaries. This is a bounded source-package audit, not a passed automatic Cargo
workspace verifier or complete final release-archive/consumer/native verification.

## Integrity, retention and remaining work

All seven finalized attempts pass evidence-integrity verification, including
failed attempts. Compact [attempts](2026-10-21-native-motion-verification/attempts.json),
per-attempt checks, original run/inventory hashes and the
[compact inventory](2026-10-21-native-motion-verification/inventory.json) identify
the executed commands, inputs and outcomes. Verify local full attempts with
`python tools/verification/run.py verify-run --run <attempt-directory>`.

Retained evidence is UTF-8/LF with a scoped byte-preservation rule. Personal paths
and transient pointers are sanitized; adapter LUID is labelled `<adapter-luid>`.
Provider IDs, hardware IDs, HRESULTs, versions and hashes remain meaningful.
Full local attempts, raw payloads, SDK binaries and archives remain under ignored
`target/`. The compact record supports outcome/provenance inspection but cannot
recover original GPU bytes after local diagnostics are removed. Rerunning the
generator does not guarantee identical output on another configuration.

D034's bounded implementation/verification checkpoint is complete. Finite zeros,
negative zero, subnormals and extreme values have controlled bit-forwarding
coverage, not a native-domain campaign. Actual-provider numerical, format/view,
resource-access and GPU-ownership obligations remain at unsafe dispatch.
Broader U05 formats, provider 2.3.4 extensions, GBV's existing stall, native error
cleanup guarantees, concurrency and final selected release archives remain open
or outside this slice. No quality/performance gate or publication is introduced.
