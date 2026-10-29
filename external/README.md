# External

This directory is an optional local SDK input for maintainer checks and native
experiments. Ordinary Cargo builds need no installed SDK and do not download one.

Install and check the pinned maintainer subset explicitly:

```sh
python tools/verification/run.py sdk install
python tools/verification/run.py sdk verify
```

Default layout:

```text
external/FidelityFX-SDK/v2.3.0/
  Kits/FidelityFX/api/include/
  Kits/FidelityFX/upscalers/include/
  Kits/FidelityFX/signedbin/
  docs/license.md
  3rdpartynotice.md
  provenance.json
```

The prebuilt ZIP supplies only the two selected DLLs. Four headers and two full
notices come from the fixed upstream source commit. All eight installed files
are checked against project-controlled SHA-256 pins. No samples or other binaries
are installed. The directory is ignored by Git and outside both Cargo packages.

Use `--archive <zip>` to supply the exact prebuilt release locally. For a fully
offline installation also supply `--source-root <upstream-tree>` containing the
four headers under `Kits/FidelityFX/`, `docs/license.md` and `3rdpartynotice.md`.
Without `--source-root`, even a local ZIP installation fetches those six files.
`--root <directory>` selects the version directory above, not `Kits/FidelityFX`.

`sdk verify` is completely offline. `sdk verify --strict` additionally requires
Windows, PowerShell 7, Windows SDK SignTool on PATH, and complete Windows trust,
timestamp and online revocation-policy checks. Missing checks fail strict mode;
the ordinary hash check makes no signature-trust claim.

Existing installations are rehashed, never silently repaired or overwritten.
Use a new `--root` or explicitly move an old installation before reinstalling.
See [acquisition commands and trust boundaries](../tools/verification/README.md#sdk-acquisition)
for interruption handling and exact limits. AMD's complete retained notices
apply to its files; the project's MPL-2.0 licence does not relicense them.

Use the pinned baseline from [D002](../docs/adr/D002.md) and the setup and
verification commands in [development](../docs/DEVELOPMENT.md#abi-and-native-verification).
Runtime loading and provider deployment follow [D007](../docs/adr/D007.md);
placing files here does not make them discoverable by an application or bundle
them in either Rust crate.
