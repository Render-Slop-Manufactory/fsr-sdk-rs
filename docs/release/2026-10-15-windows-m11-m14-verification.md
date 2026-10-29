# Windows M11–M14 and U03 verification — 2026-10-15

The previously deferred Windows native batch passed for pre-exposure (M11),
depth/motion profiles (M12), direct SDK query helpers (U03), automatic exposure
(M13) and active render regions with fixed output (M14's implemented slice).
This is development-source verification for `fsr-sdk 0.3.0` /
`fsr-sdk-sys 0.1.2`, not final release-archive verification.

## Equipment and source

- Windows 11 build 10.0.26300, `x86_64-pc-windows-msvc`.
- Rust/Cargo 1.99.0; separate MSRV profile on Rust 1.88.0.
- Native coordinator: Python 3.14.1. CI/MSRV coordinator: existing bundled
  Python 3.12.14 with NumPy 2.3.5; PowerShell 7.6.6.
- MSVC 14.44.35207 and Windows SDK 10.0.22621.0.
- Automatically selected hardware adapter index 0: AMD Radeon RX 9060 XT,
  vendor `0x1002`, device `0x7590`, feature level 12_0;
  successful driver query reports `32.0.31041.1004`.
- SDK v2.3.0, pinned source commit
  `60f4ea81909200d8542eca14dccb2628b763a9a3`.
  Provider `4.1.1`, diagnostic ID `0xf5a5ca1e01001001`.

The existing eight-file SDK installation under
`target/sdk-windows-verification/v2.3.0` passed offline verification. The native
profile separately checked both DLL hashes and Valid AMD Authenticode signatures.
No SDK download, installation or new strict online revocation check was performed.

No crate, manifest, helper, fixture or verification implementation was changed.
The starting worktree contained only the preceding research-synthesis edits.
[Native input hashes](2026-10-15-windows-m11-m14-verification/inputs.json),
[CI inputs](2026-10-15-windows-m11-m14-verification/ci-inputs.json) and
[MSRV inputs](2026-10-15-windows-m11-m14-verification/msrv-inputs.json) identify
the tested source/configuration and applicable local SDK bytes. Every accepted
attempt checked unchanged inputs and passed `verify-run` before compact evidence
selection. Repository source hashes match across all three profiles; their
configuration identities differ by profile and SDK inputs.
Attempt timestamps are UTC on October 14; these runs occurred on October 15 in
Europe/Berlin.

## Commands and outcomes

From the repository root, using existing offline Cargo inputs:

```powershell
python tools/verification/run.py sdk verify --root target/sdk-windows-verification/v2.3.0
python tools/verification/run.py run native-upscaler --native --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX

# Use the existing NumPy-equipped bundled interpreter for these profiles.
& $verificationPython tools/verification/run.py run ci --sdk-root target/sdk-windows-verification/v2.3.0/Kits/FidelityFX
& $verificationPython tools/verification/run.py run msrv
```

Set `$verificationPython` to an existing Python 3.11+ executable with NumPy for
all applicable infrastructure checks. The interpreter version used is identified
above; its private installation path is omitted.
Native prerequisites and unchanged selectors are defined by
[the verification profiles](../../tools/verification/README.md).

| Profile | Outcome | Retained summary |
|---|---|---|
| `native-upscaler` | All 27 cases passed | [Native run](2026-10-15-windows-m11-m14-verification/native-run.json) |
| `msrv` | All 95 cases passed on Rust 1.88.0 | [MSRV run](2026-10-15-windows-m11-m14-verification/msrv-run.json) |
| `ci` | All 112 cases passed | [CI run](2026-10-15-windows-m11-m14-verification/ci-run.json) |

CI executes all four feature configurations, denied-warning Clippy/Rustdoc,
consumer boundaries, dependency trees, unsupported-Vulkan rejection and complete
examples. MSRV executes all four configurations and consumer/example checks on
Rust 1.88.0. Selected [default](2026-10-15-windows-m11-m14-verification/rust-default-tests.txt)
and [movable](2026-10-15-windows-m11-m14-verification/rust-movable-tests.txt)
logs retain the new public controlled fixtures. The earlier M13 ownership-harness
failure does not recur in either completed profile.

The [infrastructure suite](2026-10-15-windows-m11-m14-verification/infrastructure-tests.txt)
runs 154 tests: 150 pass, four skip. Skips cover Linux mode/symlink identity,
Linux pidfd descendants, Linux prefix ownership and the portable symlink control
that avoids Windows privilege requirements. All applicable NumPy and PowerShell
parser checks execute; no Linux-only or privileged-symlink result is inferred.

Native prerequisites include all four independent C++ common/creation/dispatch/
query ABI probes, signed pinned deployment and compilation of the current C++ GPU
helper and Rust test executable. The complete profile also passes prior public
queries, lifecycle, single/three-frame and semantic cases, both public movable
modes, three sharpening integration repetitions and the independent
144-readback/78-comparison audit. These are integration checks; historical
sharpening quality failures are not reclassified.
[Equipment, SDK and audit observations](2026-10-15-windows-m11-m14-verification/native-observations.json)
retain those identities and counts.

## Newly confirmed native paths

| Item | Executed coverage and observations |
|---|---|
| M11 | Three fresh contexts with matching input RGB/pre-exposure factors `0.25`, `1.0`, `4.0`. All dispatch, fence, output, debug/device and explicit cleanup checks pass. [Log](2026-10-15-windows-m11-m14-verification/native.dispatch.pre-exposure.txt). |
| M12 | Eight representative depth/motion profiles, each with three moving/jittered frames: standard/reversed finite/infinite depth, render/display motion, UV/pixel units and excluded/included jitter. All selected execution and cleanup checks pass. This is not every cross-product combination on the GPU. [Log](2026-10-15-windows-m11-m14-verification/native.dispatch.depth-motion.txt). |
| U03 | Both ownership families execute all five quality modes through pre-create/live ratio and render-size queries, plus phase-count and first/middle/last jitter-offset queries. Context teardown passes. [Log](2026-10-15-windows-m11-m14-verification/native.queries.utilities.txt). |
| M13 | Real provider requirements/memory queries and three moving/jittered automatic-exposure frames with omitted exposure input pass dispatch, fence, output, diagnostics and teardown checks. The native case uses the local family. [Log](2026-10-15-windows-m11-m14-verification/native.dispatch.automatic-exposure.txt). |
| M14 render slice | Four render/display × UV/pixel motion profiles, each shrinking/growing through `320×180`, `256×144`, `288×162` active regions inside fixed `320×180` allocations and `640×360` output. Input/guard preservation, full output replacement, finite values, fences, zero debug errors/device removal and teardown pass. [Log](2026-10-15-windows-m11-m14-verification/native.dispatch.active-render.txt). |

U03 reports matching pre-create/live ratios `1`, `1.5`, `1.7`, `2`, `3` and
render sizes `1920×1080`, `1280×720`, `1129×635`, `960×540`, `640×360` for the
five quality modes. Both routes/families report jitter phase count `18` for the
selected dimensions. These are direct observations for this provider, not new
universal numerical requirements.

The controlled Windows fixtures also execute exact mappings, preflight failures,
retry/poison behaviour and reference balance for both ownership families under
the workspace test configurations. This complements the bounded native matrix;
it does not add unexecuted native combinations or cross-thread provider guarantees.

## Failed sandbox attempts and evidence limits

Initial CI/MSRV attempts inside the restricted sandbox were interrupted after
access-denied temporary-file failures. The MSRV positive compilation fixture
could not write `libpositive.rmeta` (OS error 5). The initial infrastructure suite
also reported errors, but interruption prevented a completed verdict and full
traceback capture. These attempts were not passes. Fresh outside-sandbox
attempts use unchanged source; no assertion, criterion or fixture was weakened.
Their local directories remain under `target/verification`:
`20261014T221915Z-ci-1cfb005feada` and
`20261014T221959Z-msrv-142d58ef828d`.

Permanent evidence contains sanitized summaries, input hashes and the five
selected native logs. Transient addresses are normalized by the coordinator's
export; full local attempts, build products, AMD binaries and sharpening readbacks
remain ignored local diagnostics. No whole environment dump is retained.

These checks cover one Windows GPU/driver/provider configuration. Image-quality
validation remains on hold; no new reconstruction-quality, sharpening-effect or
exposure-accuracy claim follows. M14 variable output/allocation sizes and P01/P02
performance measurements remain open. Future release closure still requires
final scope selection and exact archive/registry-consumer/native-artefact checks;
this batch neither selects additional release contents nor publishes anything.
