# Experimental Windows-to-Proton handoff

This is a preliminary execution spike for the proposed [D019](../../docs/adr/D019.md).
Develop and prepare it on Windows, then execute the selected Windows binaries
on a real Linux host such as the Steam Deck. WSL is not used. No `proton` profile
has been registered in `run.py`, and this does not complete V07a's Linux cross-build.

## Windows preparation

Use an existing **complete passed** `native-upscaler` attempt and the exact
eight-file D017 SDK installation that supplies the retained notices:

```powershell
python tools/verification/proton_spike.py prepare --run target/verification/<passed-attempt> --sdk-root target/sdk-windows-verification/v2.3.0 --output target/proton-spike/<new-bundle>
python tools/verification/proton_spike.py inspect --bundle target/proton-spike/<new-bundle>
```

`--sdk-root` here is the version directory, parent of `Kits`, unlike the native
coordinator's `--sdk-root`. Preparation checks the complete attempt integrity,
successful status, current crate/manifests against its input hashes, the selected
SDK inventory and matching AMD bytes. A changed crate input requires a new native
attempt. Existing bundle destinations are rejected.

The bundle contains the exact Windows test executable, GPU helper, loader and
upscaler DLLs, complete notices, source/build/signature provenance, and copies of
the small experimental entry point plus shared execution/record modules. The
complete bundle file inventory is hashed. This is a private local experiment
input containing AMD binaries, not a public repository/release artefact.

Copy the entire bundle to the Linux host. Its `inspect` command requires only
Python 3.11+ and neither launches Proton nor downloads anything. There is no need
to clone or compile the Rust project on that host for this preliminary spike.

## Linux execution prerequisites

The initial launcher candidate is an existing local `umu-run`, a concrete local
Proton directory and `vulkaninfo`. Choose their actual versions on the Linux host;
no launcher, Proton distribution or Linux runtime is installed by this script.
Use desktop mode for the initial CLI run. Do not guess the Deck's installed paths.

The [umu documentation](https://github.com/Open-Wine-Components/umu-launcher/blob/main/docs/umu.1.scd)
describes explicit `PROTONPATH`, `WINEPREFIX`, argument forwarding and
`UMU_RUNTIME_UPDATE=0`. Its [runtime implementation](https://github.com/Open-Wine-Components/umu-launcher/blob/main/umu/umu_runtime.py)
still contacts upstream and may install a missing runtime, even with updates
disabled. Therefore this experimental entry point requires the explicit
`--allow-launcher-network` acknowledgement as well as `--native`; it makes no
offline-execution or pinned full-runtime acquisition claim. Ordinary Cargo and
coordinator profiles do not invoke it. A production D019 bootstrap must resolve
that version/integrity policy separately.

From inside the copied bundle:

```sh
python3 tools/verification/proton_spike.py inspect --bundle .
python3 tools/verification/proton_spike.py run \
  --bundle . --native --allow-launcher-network \
  --umu /absolute/path/to/umu-run --umu-version <actual-version> \
  --proton /absolute/path/to/installed-proton --proton-version <actual-version>
```

Those paths and version arguments are placeholders, not a tested Linux command.
Version arguments are operator labels, supplemented by actual launcher hashes
and captured launcher logs; they are not verified version attestations. Specify
`--adapter-index <nonnegative-index>` only after identifying the Linux adapter;
otherwise the existing helper chooses the first suitable hardware adapter.

Every invocation creates a fresh attempt under the bundle's sibling directory
`proton-spike-results/verification` and a fresh dedicated prefix under
`proton-spike-results/prefixes`. Output locations overlapping the immutable bundle
are rejected, so later inspections and repeated runs retain its identity. Prefixes remain separate
from evidence inventories and are retained for diagnosis. No existing prefix is
reused, deleted or altered. Proton may still use its shared launcher/runtime
cache. Do not run concurrent setup against that cache during the spike.

## Frozen cases and outcomes

The script uses the existing `Runner`, subprocess watchdogs and structured
execution/evaluation records. It stages a private executable-adjacent DLL copy
for the attempt and sets test DLL paths relative to that application directory.
The actual Proton working-directory/path behaviour needs the Linux run; Windows
preparation does not establish it.

1. Record Linux/Python identity, explicit launcher identities and
   `vulkaninfo --summary` (30-second watchdog). Missing `vulkaninfo` blocks AMD work.
2. Run the exact existing public five-query test (180 seconds), requiring exit 0,
   `M10 public queries: PASSED` and `1 passed; 0 failed`.
3. Run unchanged Gate A (180 seconds), requiring exit 0, `Gate A: PASSED` and
   `1 passed; 0 failed`. It checks the established 640×360 workload's dispatch,
   submission/fence/readback metrics, output properties, teardown, debug errors
   and device removal.

Inherited FSR test controls and common compatibility bypasses are cleared.
The query case must pass before dispatch. The Windows debug-layer requirement
remains enabled: a missing interface is an explicit failed environmental control,
not a reason to bypass an assertion or report a GPU pass. No helper or public
Rust API was changed. If that blocks the Deck run, a separately scoped diagnostic
mode can be designed from the actual observation, preserving the Windows case.

Result JSON and logs are retained for failed and successful attempts. Shared
watchdogs stop the launched process tree/group; cleanup of Proton services that
escape that group has not been verified. Do not infer full provider termination
or runtime isolation. Linux signature-policy equivalence, loaded module-path
auditing, full runtime component provenance, temporal quality, another GPU and
Linux cross-compilation remain outside this preparation result. A successful
spike is still narrower than D019's complete initial checkpoint.

## Host checks

```sh
python -m unittest discover -s tools/verification/tests -p test_proton_spike.py -v
```

These controls reject changed/missing/extra bundle files, stale crate source or
manifest inputs, unsupported schema, Windows execution and absent native/network
opt-ins. They execute no Proton or AMD code and cannot establish Linux behaviour.
