# Explicit SDK acquisition — 2026-10-12

Follow-up: [real Windows verification](2026-10-12-windows-m10-sdk-verification.md)
subsequently passed on 2026-10-12: online installation, all eight pins, offline
reuse, strict Authenticode/SignTool/chain policy and native execution from the
acquired subset. The unperformed Windows checks below describe the earlier
macOS work. R02 remains Active for broader runtime dependency/deployment work.

Scope: the developer-only v2.3.0 bootstrap in `tools/verification/sdk.py` under
[D017](../adr/D017.md). Host: macOS arm64, Python 3.14.8. No AMD code was executed.
This checks acquisition and file identity, not Windows/GPU support or legal
clearance for redistribution. R02 remains Active pending real Windows strict-mode
verification and its broader runtime dependency/deployment work.

Changed files: new `tools/verification/sdk.py` and `tests/test_sdk.py`, small edits
to `run.py`, `native.py` and `windows.ps1`, setup documentation in `external/README.md`
and `tools/verification/README.md`, `docs/DEVELOPMENT.md`, `ROADMAP.md`, the decision
index and new D017, and this verification record. No Rust crate was changed.

## Actual upstream inputs

The [official release asset](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip)
was downloaded on 2026-10-12 with HTTPS-only redirects, a 200 MB byte limit and
bounded connection/transfer timeouts. It was subsequently downloaded again by
the implemented CLI, with matching results.

- Exact archive length: **127,451,548 bytes**.
- Computed full-file SHA-256:
  `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`.
- ZIP inventory: **231 entries**, including directories; **259,349,524 bytes**
  declared total expansion; maximum observed compression ratio approximately
  **12.56:1**. Names/types/expansion were accepted by the implemented validator.
- Top-level members are `Samples/`, `Tools/`, `ClearMediaCache.bat` and
  `UpdateMedia.bat`. No SDK API headers, standalone licence texts, third-party notice or
  archive EULA were present in the inspected member inventory. This does not
  establish the absence of every external service/contractual term.
- The selected DLLs are under
  `Samples/Upscalers/FidelityFX_FSR/dx12/x64/Release/`. Other sample copies and
  unrelated DLLs, shaders, executables and debugging files are not installed.

| Selected DLL | Bytes | Computed SHA-256; matches the existing project pin |
|---|---:|---|
| `amd_fidelityfx_loader_dx12.dll` | 26,376 | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` |
| `amd_fidelityfx_upscaler_dx12.dll` | 28,761,864 | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` |

The loader's embedded certificate metadata was inspected without executing the
DLL: subject organisation and common name are `Advanced Micro Devices`. This is
the strict publisher-name expectation; metadata inspection alone proves neither
signature mathematics nor trust, timestamp validity or current revocation.

## Sources, include closure and notices

The additional six files were downloaded from
`https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/`
and checked by the real CLI. The four header byte hashes equal the existing
[`tools/native-headers.json`](../../tools/native-headers.json) pins; each also
passes the unchanged normalised-text ABI hash contract.

| Additional source | Bytes | Identity |
|---|---:|---|
| `Kits/FidelityFX/api/include/ffx_api.h` | 10,726 | Existing header pin matched |
| `Kits/FidelityFX/api/include/ffx_api_types.h` | 13,636 | Existing header pin matched |
| `Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h` | 11,323 | Existing header pin matched |
| `Kits/FidelityFX/upscalers/include/ffx_upscale.h` | 19,313 | Existing header pin matched |
| `docs/license.md` | 60,891 | `6757f2fec461238da6f84f83536ec195e8c036b1773a06f6f34301b9ed8c0963` |
| `3rdpartynotice.md` | 2,441 | `94401a90486d1b53b528bb7f845a7b8daf7227b60eb7c0fc3b8c5571ba202a27` |

Source include inspection found only `stdint.h` / `stdbool.h`, the four selected
AMD headers, and Windows `d3d12.h`, `dxgi.h` / `dxgi1_5.h`. The actual C++ ABI
probes and native helper include these selected headers; no AMD C++ facade or
import library is needed. Windows standard/platform headers remain a toolchain
prerequisite, not part of acquisition.

The complete pinned AMD licence was read: all four canonical header paths and
both `Kits/FidelityFX/signedbin/` DLL names are explicitly in its MIT exception
list. That list is not a blanket licence for the ZIP. Full source notices and
embedded header copyright/permission text are retained unchanged; sample third-party
DLLs are omitted. No archive/binary distribution was performed or approved.

## Executed checks

From the repository root, `python3` was the existing Python 3.14.8 interpreter:

```sh
python3 tools/verification/run.py sdk install --archive <verified-local-zip> --source-root <pinned-local-source-tree>
python3 tools/verification/run.py sdk install --root target/sdk-acquisition-check/v2.3.0
python3 tools/verification/run.py sdk verify
python3 tools/verification/run.py sdk install
python3 tools/verification/run.py sdk verify --strict
python3 -m unittest discover -s tools/verification/tests -v
python3 -m unittest discover -s tools/verification/tests -p test_sdk.py -v
python3 tools/verification/archive/check.py
cargo package --list --locked --offline --allow-dirty --manifest-path crates/fsr-sdk/Cargo.toml
cargo package --list --locked --offline --allow-dirty --manifest-path crates/fsr-sdk-sys/Cargo.toml
python3 tools/verification/repository_size.py
git diff --check
```

- Both real local/offline and real online acquisition paths installed exactly
  eight pinned payload files plus provenance. Online access required environment
  approval. The default SDK is present under ignored `external/`.
- Offline verification passed; repeated installation reused the checked tree
  without downloads or modifications. Strict verification exited **2**, explicitly
  reporting unavailable Windows/PowerShell verification.
- Full infrastructure suite: **114 tests**, **111 passed, 3 skipped**, no failures.
  Three skips are unavailable PowerShell parsing checks. An initial sandbox run
  failed six existing loopback-socket process tests with `PermissionError`; the
  permitted rerun outside that sandbox passed. This was an environment restriction.
- Final focused SDK suite after HTTP error-handling refinements: **35 tests**,
  **34 passed, 1 skipped** (current PowerShell source parsing unavailable).
  Synthetic cases cover wrong/truncated downloads and hashes, stream bounds,
  ZIP paths/types/duplicates/collisions/expansion, missing files, modified notices,
  forged provenance, links, hardlinks, interrupted installation, stale/concurrent
  locks, destination appearance, offline reuse and strict missing/failed tooling.
- Existing archive inventory/hashes and portable Python/JSON syntax passed;
  archived PowerShell parsing remained explicitly `not_run` / `not_evaluated`.
- Both Cargo package lists excluded the SDK, DLLs, acquisition code and notices.
  Git ignores the installed SDK; Rust crates/manifests and verification profiles
  were not changed. Repository budget and diff whitespace checks passed.

## Windows CI path regression — 2026-10-12

The initial Windows stable `ci` attempt failed only
`test_unsafe_zip_paths_including_ignored_members_are_rejected`, for `a\b`:
the expected `ValueError` was not raised. `ZipInfo` had converted that test
member to `a/b` while writing the fixture on Windows. Stable runs the
infrastructure suite; the MSRV profile does not include it.

The fixture now preserves the original member name. Archive path validation,
ancestor checks and pinned inventory parents use `PurePosixPath`; local
filesystem access retains `Path`. Both safe and unsafe archive fixtures are
checked with `/` and `\` simulated as `zipfile.os.sep`, including rejection of
empty, dot, repeated-separator and traversal names. After the fix,
`python3 -m unittest discover -s tools/verification/tests -v` ran **115 tests**:
**112 passed, 3 skipped**, with no failures on macOS. The original official
ZIP's **231 entries** still passed `sdk.zip_members`, and
`python3 tools/verification/run.py sdk verify` passed for the existing pinned
installation. This is local regression evidence; the corrected Windows CI run
has not been executed here.

## Unperformed checks and limitations

PowerShell 7, Windows SDK SignTool and a supported Windows host were unavailable.
The modified current PowerShell bridge has **not** been parsed or executed on
this host; Windows infrastructure tests parse it when PowerShell is available.
Real Authenticode mathematics, trusted publisher/chain, timestamp and online
revocation checks remain **not performed**. Mocked subprocess success/failure
tests do not establish those outcomes for AMD's DLLs. There is no Linux trust
implementation or equivalence claim.

No Windows ABI compilation, loader/provider execution, GPU dispatch, clean-machine
dependency audit or Proton execution was attempted. Same-filesystem rename and
sibling locks assume caller-controlled parents; power-loss durability or protection
against an adversarial same-user process changing those parents is not claimed.
Current-clock strict chain checks can reject expired certificates despite a
historically valid Authenticode timestamp. OS-valid cached revocation data is
accepted by OS policy; a fresh external response on every run is not guaranteed.
