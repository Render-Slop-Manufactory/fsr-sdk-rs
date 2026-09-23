# Experiment: M4 production ownership on Windows

- Investigation date: 2026-09-22.
- Question: Does the implemented private M4 owner pass Windows fixture execution,
  paired native ABI checks and a real AMD create/query/destroy lifecycle?
- Success: every runner phase succeeds, the isolated native test identifies a
  provider and reports successful production-owner destruction, without timeout.
- Failure: a failed check, native error, crash, missing provider identity or timeout.
- Exclusions: dispatch, reconstructed images, performance, public construction,
  new failure injection, threading guarantees and additional hardware support.

## Setup

Repository revision: `abe1d66857cb258d4a4830c6baf863766c67ebae`, with a clean
worktree before execution. No production code, test code, manifests or lockfile
were changed for this experiment. The existing
[M4 runner](../../../crates/fsr-sdk/tests/run-m4.ps1),
[owner fixtures](../../../crates/fsr-sdk/src/context/tests.rs),
[native test](../../../crates/fsr-sdk/src/upscaler/tests.rs) and
[C++ ABI probe](../../../crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp)
are the executable evidence. Documentation was written after the run.

Environment: Windows build `26200.9457`, display version `25H2`; Rust
`1.98.1 (48a229cea 2026-09-01)`, host `x86_64-pc-windows-msvc`, LLVM 22.1.8;
VS 2022 Build Tools developer shell `17.14.28`, installed default MSVC tools
`14.44.35207`. The native helper selected AMD Radeon RX 9060 XT, vendor `0x1002`,
device `0x7590`, hardware feature level `12_0`. DXGI driver query returned
HRESULT `0x00000000`, version `32.0.31041.1004`. A preliminary CIM hardware
inventory was denied in the sandbox; hardware identity above comes from the
actual successful native helper, not that inventory.

Inputs are the existing local SDK `v2.3.0` under
`external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/`. Both DLLs reported Valid
Authenticode signatures with Advanced Micro Devices as signer. SHA-256 values
were remeasured and match the earlier raw lifecycle inputs:

| Input relative to that SDK directory | SHA-256 |
|---|---|
| `api/include/ffx_api.h` | `91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4` |
| `api/include/ffx_api_types.h` | `B54FBD96A0EED82662A49C00E28E5368AB69959F9856DAE5C12FED109D123D66` |
| `api/include/dx12/ffx_api_dx12.h` | `2082D6C2914E9C2FA9FAE6247D35F64C643CCB703311C83C8DBA9A35356BC711` |
| `upscalers/include/ffx_upscale.h` | `F13ABCDD4389E22AA50562A90255BEC9511E004722C7D2BF6F959D92FD78355C` |
| `signedbin/amd_fidelityfx_loader_dx12.dll` | `E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA` |
| `signedbin/amd_fidelityfx_upscaler_dx12.dll` | `D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46` |

These fingerprints identify local inputs; they are not independent verification
of an upstream release archive. Cargo fetched the already-locked
`unicode-ident 1.0.26`; this was not a fully offline run.

## Reproduction

From the repository root in an x64 VS developer shell:

```powershell
powershell -NoProfile -File crates/fsr-sdk/tests/run-m4.ps1 -Native
```

The initial invocation, after `vcvars64.bat` initialized x64 tools, exited 1:
Windows PowerShell refused the script because script execution was disabled.
No runner phase had executed. The authorized retry used this exact command
from PowerShell, outside the sandbox with a process-local execution-policy override:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && powershell -NoProfile -ExecutionPolicy Bypass -File crates/fsr-sdk/tests/run-m4.ps1 -Native'
```

This does not persistently change the machine's execution policy. The unchanged
runner performs formatting, Clippy, workspace tests and no-default-feature
checking with locked dependencies, then compiles the paired C++ ABI probe and
DX12 helper with `/std:c++17 /W4 /WX`. It stages the wrapper test executable,
helper, loader and upscaler DLL in `target/m4-lifecycle/` and runs:

```text
m4.exe --ignored --exact upscaler::tests::native_lifecycle --nocapture --test-threads=1
```

The runner supplies absolute staged paths through `FSR_SDK_TEST_DLL` and
`FSR_SDK_TEST_DEVICE_DLL`, restores their previous values, and limits the child
to 60 seconds. It makes no DLL-search-directory API changes. Native inputs are
render size 1280x720, upscale size 1920x1080, flags 0, null message callback,
null host allocators and no provider override. The retained chain is
upscale -> version (`0x01001001`) -> DX12 backend -> null. The helper transfers
one owned device reference to the wrapper; no GPU work is submitted.

## Observations

**Verified:** the complete retry exited 0.

| Phase | Observed result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 17 executable tests and 2 compile-fail doctests passed; 5 opt-in tests ignored |
| `cargo check --workspace --no-default-features --locked` | Passed |
| Paired C++ ABI compilation | Passed, including common ABI and creation/provider-query ABI |
| Native device helper and wrapper test build | Passed |
| Isolated production-owner lifecycle | 1 passed, 0 failed; `timeout=False exit=0` |

The 17 ordinary tests comprise 7 wrapper tests, 6 ABI tests and 4 loader tests.
Wrapper fixtures execute through production `FfxLibrary` on Windows and verify:
successful explicit destruction and Drop each call destroy once; cleanup order
is state then device then runtime; non-OK create with null or non-null output
never destroys and releases inputs; OK/null retains all dependencies; destroy
errors retain dependencies and never retry, whether the fixture clears the handle
or leaves it unchanged. Unknown codes remain unchanged. Windows DLL-residency
checks pass. Positive-control compile checks precede expected moved-owner and
private-handle errors. Concrete descriptor-address checks and compile-time trait
exclusions also pass. These synthetic errors establish wrapper policy only.

Five ignored tests in the ordinary run were the new native lifecycle, two raw
failure probes, standalone AMD loader smoke test and count-only query. The native
phase then explicitly executed the production-owner lifecycle; the other four
were not rerun. The provider query inside that lifecycle is distinct from the
count-only query.

The runner saved `native.stdout.txt`, `native.stderr.txt` and `native.status.txt`
under `target/m4-lifecycle/`. Their relevant output is reproduced here; it contains
no personal paths or transient pointers:

```text
running 1 test
test upscaler::tests::native_lifecycle ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.61s

Adapter: AMD Radeon RX 9060 XT; vendor=0x1002 device=0x7590; feature_level=12_0
Driver query: HRESULT=0x00000000; version=32.0.31041.1004
Production owner: create OK
Provider: id=0xf5a5ca1e01001001; name=4.1.1
Production owner: destroy OK

timeout=False exit=0
```

## Baseline delta and limits

**Verified delta:** M4 now has actual Windows fixture execution, paired native
ABI compilation and a successful AMD lifecycle through `Upscaler` /
`NativeContextOwner`, including moving the outer owner before querying and
consuming destruction. The former macOS/cross-compilation-only verification gap
is closed for this configuration. No implementation fix was needed.

The provider identity agrees with the
[historical raw lifecycle](2026-09-20-exp-context-lifecycle.md), but this run adds
production-owner evidence. It does not remeasure post-destroy native handle bits,
prove provider-internal allocation cleanup or measure COM reference counts.
Actual AMD Drop teardown was not separately invoked; Drop policy is fixture-tested.

One successful native run on one GPU/driver is not a portability or stress test.
No valid-context AMD destroy failure or AMD OK/null create was induced. D005's
returned-create-error trust assumption is unchanged, not newly proved by these
fixtures. Provider identity does not identify whether the implementation came
from the bundled DLL or driver. No dispatch, image correctness, performance,
public API or global threading guarantee is established. Those remain later work;
there is no remaining failure from this bounded M4 verification run.
