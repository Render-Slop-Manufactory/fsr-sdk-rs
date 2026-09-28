# Experiment: M6 dispatch recording ahead of GPU completion

- Investigation date: 2026-09-24.
- Question: Can the private DX12 upscaler path record several dispatches before
  submitting any list, then execute them in order and obtain distinct valid
  outputs? Does the same observation hold when the dispatches belong to sibling
  upscaler contexts sharing one runtime and device?
- Success criterion: every native dispatch returns OK; all lists close, submit
  on one queue and reach a fence; each output has 230,400 finite pixels changed
  from its sentinel, a spatial gradient, and an increasing red level matching
  its distinct input; no D3D12 debug error or device removal.
- Failure criterion: native error, crash, timeout, missing or swapped output
  signature, debug error, or device removal.
- Exclusions: public API design, general GPU lifetime safety, image quality,
  arbitrary frame counts, other providers/drivers, and proof of transient
  allocator ownership or isolation.

## Setup and reproduction

The baseline was repository commit `76a604167f00679fc81c9247446fa65770ecad37`
plus the uncommitted, opt-in probe in [the Rust test](../../../crates/fsr-sdk/src/upscaler/m6_recording.rs),
[DX12 helper](../../../crates/fsr-sdk/tests/m6_gpu.cpp), and
[runner](../../../crates/fsr-sdk/tests/run-m6-recording-spike.ps1). The helper's
existing M6a path was factored to share readback logic and rerun as a control.
No public dispatch method, constructor option, or production native ABI was
changed. The SDK source is `v2.3.0` at
`60f4ea81909200d8542eca14dccb2628b763a9a3`. The signed staged loader and
upscaler binaries had SHA-256 hashes
`E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA`
and `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46`
respectively. The rebuilt local test helper hash was
`5B5C54E5B13DCF560EE21EFF01AB2CB8E7E8EEA0FEF8589D9AD80BC5900EC79F`.
The M6a control queried provider ID `0xf5a5ca1e01001001`, version `4.1.1`.

The machine was Windows 11 Pro build `26200`, Rust `1.98.1` on Windows
x64/MSVC, Visual Studio 2022 Build Tools, and AMD Radeon RX 9060 XT
(`vendor=0x1002`, `device=0x7590`, feature level `12_0`) with driver
`32.0.31041.1004`. The D3D12 debug layer was enabled; GPU-based validation
was disabled. The successful command, from the repository root in PowerShell,
was:

```powershell
cmd /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && pwsh -NoProfile -File crates\fsr-sdk\tests\run-m6-recording-spike.ps1'
```

The runner first performs the normal M6a formatting, Clippy, workspace tests,
no-default-features check, paired header ABI compile, helper build, and fenced
single-dispatch GPU control. It then runs each spike case in a separate child
process with a 180-second timeout. It writes environment, status, stdout and
stderr files under ignored `target/m6a/`. Ordinary `cargo test` ignores this
GPU case.

Each dispatch used the tested M6a profile: the public M5 constructor with
zero flags; 320×180 RGBA16F color, R32F depth, RG16F motion, and explicit 1×1
R32F exposure; 640×360 RGBA16F UAV output; fixed frame values and jitter.
Each recording had its own DIRECT list, allocator, five textures, upload
buffers and output readback. Before submission, the CPU shifted that
recording's red input by `0.015 × index`, keeping it below `1.0`; all other
scene fields remained the same. In `same` mode, one upscaler recorded all
dispatches, with `reset=true` only on the first. In `sibling` mode, each
dispatch was the first, reset dispatch of its own upscaler; the contexts shared
one runtime and device. All recording completed before the first submission.
The helper then closed each list, submitted them through separate ordered
`ExecuteCommandLists` calls on one queue, signalled one fence, waited, and
read back every output before destroying resources or contexts.

## Observations

The final runner exited `0`. Every child reported `timeout=False exit=0`:

| Mode | Dispatches recorded before submission | First → last left red mean | GPU result |
|---|---:|---:|---|
| Same context | 1 | `0.19475134` | passed |
| Same context | 2 | `0.19475134` → `0.20395952` | passed |
| Same context | 5 | `0.19475134` → `0.24901712` | passed |
| Same context | 8 | `0.19475134` → `0.29507476` | passed |
| Sibling contexts | 2 | `0.19475134` → `0.20972249` | passed |
| Sibling contexts | 5 | `0.19475134` → `0.25466114` | passed |

For every output in every case, the 640×360 readback contained `230400`
finite pixels and `230400` pixels changed from the half-float sentinel. Red
increased spatially from left to right and its left-region mean increased
between the distinct inputs by more than the probe's `0.005` threshold.
All native dispatches returned `0x0`; all readbacks reported `debug_errors=0`
and `device_removed=0`. For example, the eighth same-context output reported
`left_red=0.29507476`, `right_red=0.84656924`, while the first reported
`left_red=0.19475134`, `right_red=0.7568281`.

An initial restricted-shell invocation passed ordinary compilation and tests
but was denied access to OS/GPU inventory before native execution. The run
above used the approved unrestricted environment. A preliminary version with
identical inputs also passed, but could not distinguish descriptor reuse;
the distinct-input run above is the relevant result.

## Interpretation and limits

**Verified for this configuration:** Recording ahead by 2, 5 and 8 calls on
one context, and by 2 and 5 sibling contexts, produced distinct plausible
fenced outputs. The count of eight numerically exceeds the
[`FFX_MAX_QUEUED_FRAMES=4`](../../../external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/internal/ffx_internal_types.h)
constant in the pinned open SDK, but this does not locate a ring boundary: its
allocation formulas also account for passes and effect contexts, and the
selected signed provider need not use that open backend. The observation rules
out a simple claim that the fifth CPU-recorded dispatch must fail on this
provider and fixture.

**Unresolved:** The selected signed provider is closed, so this black-box
result does not identify its transient descriptor or constant-buffer allocator,
whether that allocator is shared across contexts, or whether it retains data
correctly for every input and scheduling pattern. The coarse image signatures
can expose gross resource aliasing but cannot validate each native constant,
descriptor, or temporal history value. The lists were submitted sequentially
on one queue and all resources were retained until one final fence; this does
not test concurrent queues, overlapping GPU execution, early destruction,
residency changes, or arbitrary numbers of in-flight frames. It is one run on
one GPU/driver/provider, without GPU-based validation or image-quality checks.

Relative to [M6a](2026-09-24-exp-m6a-native-dispatch-proof.md), this adds
direct evidence that multiple private calls can be CPU-recorded before GPU
completion in the tested profile. It does **not** establish a safe public
multi-dispatch contract or justify relaxing D011's conservative fence rule.
That decision still needs a provider-backed lifetime/allocator guarantee or a
contract that bounds recording and submission independently of this black-box
observation.
