# Source and package audit: project licensing and AMD notices

- Date: 2026-09-21.
- Repository revision: `741d80fb4592d055bb7daa70b18658cfcc78fdff`; clean worktree
  before this audit. Cargo 1.98.1 on Windows x64/MSVC.
- Scope: currently identifiable AMD-derived declarations, retained notices,
  Cargo metadata and actual package contents, local SDK binary distribution,
  and locked Rust dependencies. No publication or license change performed.
- Method: inspect project sources/manifests, compare all four relevant header
  notices, inspect pinned upstream license documents, list both Cargo packages,
  create and inspect the raw crate archive, attempt wrapper packaging.
- Limit: technical provenance/compliance review, not a legal opinion about
  copyrightability, title to all contributions, patents or jurisdiction-specific law.

## Primary sources

AMD baseline: SDK v2.3.0, commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`. Accessed 2026-09-21:

- [Common API header](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h).
- [API types](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h).
- [DX12 API header](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h).
- [Upscaler header](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/upscalers/include/ffx_upscale.h).
- [SDK license inventory](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/docs/license.md)
  and [root license inventory](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/docs/license.md).
- [MIT terms](https://opensource.org/license/mit),
  [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/) and
  [Mozilla's FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).
- [Cargo license metadata and package inclusion](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields).

Pinned upstream pages confirm the notices and inventory entries. No full
byte-for-byte authentication of all local SDK files or signed DLLs was performed.
The raw kit-license URL initially returned a web cache miss; the pinned GitHub
page and root raw document were readable.

## Declaration provenance and retained notices

Paths below are relative to `crates/fsr-sdk-sys/` and SDK `Kits/FidelityFX/`.

| Project material | AMD source |
|---|---|
| `src/api.rs`: common ABI, callbacks, entry-point signatures, global version query | `api/include/ffx_api.h` |
| `src/upscale.rs`: upscaler selector tag | `upscalers/include/ffx_upscale.h`, using common API effect/tag definitions |
| `tests/context_lifecycle/abi.rs`: callback, provider query, dimensions, creation/version/backend descriptors | All four headers listed above |
| ABI assertions and fixture/native function signatures in tests | Checks/use of those same declarations; native checks include local headers rather than bundling them |

**Verified:** all four local headers carry the same AMD 2026 copyright and MIT
permission/disclaimer block. Each matches `LICENSE-AMD` after whitespace removal.
The production declaration modules and lifecycle ABI module refer to that notice.
The native device helper includes installed Windows SDK headers and uses an
opaque device pointer on the Rust side; no Windows SDK header is packaged.
No additional copied SDK implementation was identified in the current crate
inventory. This is not a forensic guarantee of originality of every contribution.

Local SHA-256 fingerprints:

| Header | SHA-256 |
|---|---|
| `ffx_api.h` | `91F7F4A9111D18996E3BAE083BF82D47AC6497DE144B352ABFCA44F07D2871C4` |
| `ffx_api_types.h` | `B54FBD96A0EED82662A49C00E28E5368AB69959F9856DAE5C12FED109D123D66` |
| `dx12/ffx_api_dx12.h` | `2082D6C2914E9C2FA9FAE6247D35F64C643CCB703311C83C8DBA9A35356BC711` |
| `ffx_upscale.h` | `F13ABCDD4389E22AA50562A90255BEC9511E004722C7D2BF6F959D92FD78355C` |

**Assessment:** preserving the full AMD notice is supported by the inspected MIT
conditions. Whether these particular declarations alone constitute copyrightable
or substantial material need not be resolved to retain the notice conservatively.
MIT permits modification and sublicensing; original project contributions under
MPL are not inherently incompatible with retaining the AMD MIT notice. This does
not mean every mixed Rust file is independently available under MIT.

## Actual Cargo package findings

| Check | Result |
|---|---|
| `cargo package -p fsr-sdk-sys --list --locked --offline` | Success; lists `LICENSE-AMD`, but no MPL `LICENSE` or README |
| `cargo package -p fsr-sdk --list --locked --offline` | Success; no license-text file or README listed |
| `cargo package -p fsr-sdk-sys --locked --offline --no-verify` | Success; 20 files, about 62 KiB unpacked / 17 KiB compressed |
| `cargo package -p fsr-sdk --locked --offline --no-verify` | Fails: path dependency `fsr-sdk-sys` lacks a version requirement |

The raw archive was independently inspected with:

```powershell
tar -tf target/package/fsr-sdk-sys-0.1.0.crate
tar -xOf target/package/fsr-sdk-sys-0.1.0.crate fsr-sdk-sys-0.1.0/Cargo.toml
```

**Verified:** its normalized manifest declares `license = "MPL-2.0"` and
`readme = false`. The AMD notice is present; the root MPL text is absent.
There are no AMD DLLs, SDK headers, generated helper binaries or bundled Rust
dependency sources in this archive. Lifecycle experiment source files are included.
For `fsr-sdk`, only the file list is verified: no wrapper archive was produced.
No compilation verification was requested by these `--no-verify` commands.

**Packaging gap:** the repository's root LICENSE does not automatically accompany
each workspace member. Include the MPL text and a clear license reference in
each distributable crate before release. Absence of a bundled text alone is not
declared a proven MPL violation here: MPL section 3.1 also concerns informing
recipients where to obtain the license, and the manifest identifies its SPDX ID.
The practical gap is an incomplete standalone package licensing presentation.

**Metadata assessment:** both crates inherit `MPL-2.0`; this communicates original
project licensing but does not expose the retained AMD terms to metadata-only
consumers. For an explicit aggregate description of MPL original work plus MIT
material, `MPL-2.0 AND MIT` on `fsr-sdk-sys` is a candidate, paired with a scope
notice. It is not an automatic legal requirement established by this audit.
`MPL-2.0 OR MIT` would instead advertise a choice for the package and is not
supported by current project policy. The wrapper need not inherit dependency
licenses as its own package license. No metadata or license policy was changed.

## SDK binaries and other dependencies

**Verified distribution boundary:** `git ls-files '*.dll' '*.exe' '*.lib' '*.h'
external` finds only `external/README.md`. SDK contents and `target/` are ignored.
The lifecycle runner copies two AMD DLLs into local `target/context-lifecycle`;
it does not publish them. Current Cargo packages do not redistribute these DLLs.
The staging directory is not a prepared redistribution bundle and carries no
complete third-party notice bundle. Review it separately before sharing it.

**Important refinement:** both local AMD license inventories are byte-identical
(SHA-256 `F0DA09D71AD5C82759A179E774535D4A829E5C96C49294167C1152402B2CB400`).
They start with restricted binary terms but explicitly list the four headers and
both `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll` in
the exception list followed by MIT terms. The pinned upstream inventory agrees.
Therefore a blanket claim that these two named v2.3.0 DLLs necessarily fall under
the default restricted license is not supported. This observation is specific
to the listed files/version, not all AMD SDKs, driver components or dependencies.
The inventory calls the list source files even though it includes binaries;
retain its actual entries rather than silently removing the exceptions.

The SDK's `3rdpartynotice.md` lists additional sample/tool dependencies. The
current Rust packages do not bundle that SDK/sample tree. This audit does not
establish the complete embedded-component or patent clearance of a future
binary bundle, nor does it authorize its distribution.

`cargo metadata --format-version 1 --locked --offline` and cached license files
identify the locked Rust dependencies:

| Dependency | Version | Declared license |
|---|---|---|
| `libloading` | 0.9.0 | ISC |
| `cfg-if` | 1.0.4 | MIT OR Apache-2.0 |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 |

Their cached packages contain their respective license texts. They are registry
dependencies, not source vendored inside our crate archive. Distribution of a
compiled application/helper is a separate notice assessment from publication
of this source crate. No incompatible dependency license was identified for the
current arrangement.

## Result and recommended follow-up

The proposed README distinction between original MPL code and preserved AMD MIT
notices is supported for the inspected header-derived material. Current facts
support that wording more strongly than a blanket MIT label on the declarations.
No license incompatibility was identified, but the project is not release-ready:

1. Include MPL text and clear provenance/licensing information in each crate.
2. Choose and document the intended aggregate metadata treatment of AMD-derived
   material; preserve `LICENSE-AMD` regardless of that choice.
3. Add a version requirement to the wrapper's path dependency as part of release
   preparation, then inspect the resulting wrapper archive too.
4. If binaries are later shipped, enumerate that exact bundle and preserve the
   applicable AMD and third-party notices; local staging is not that check.

No source, manifest, LICENSE, README policy or accepted decision was modified.
The audit does not settle copyrightability, ownership of all contributions,
patent rights or every possible downstream distribution; those are limits of
the evidence, not reasons to leave the concrete package gaps unreported.
