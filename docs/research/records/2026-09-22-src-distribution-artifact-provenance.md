# FidelityFX SDK v2.3.0 distribution and artifact provenance

- **Date:** 2026-09-22
- **Scope:** FidelityFX/FSR SDK **v2.3.0 only**, Windows x64/MSVC, DirectX 12, with emphasis on the production FSR API upscaling runtime and AMD FSR Upscaling 4.1.1. Later releases are not used as substitutes.
- **SDK baseline:** tag `v2.3.0`; release commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. The GitHub release identifies short commit `60f4ea8`; GitHub reports the commit as signed with a verified signature, GPG key `B5690EEEBB952194`.
- **Method:** primary AMD/GPUOpen/GitHub material was inspected at the exact v2.3.0 revision, including release metadata, FSR API documentation, committed signed binaries, licensing files, third-party notices, source/build documentation, and Microsoft licensing material where applicable. The release ZIP could not be fetched as raw binary in this research environment, so local PE import-table and Authenticode inspection of the actual DLL bytes remains explicitly unresolved.
- **Evidence classes:** **Verified** = directly observed in the exact source/artifact metadata; **Upstream claim** = explicitly stated by AMD/GPUOpen; **Inference** = consequence of documented relationships but not itself explicitly stated; **Unresolved** = evidence is insufficient.

## 1. Official acquisition channels

| Channel | Exact v2.3.0 URL | Contents | Immutable? | Official status | Notes |
|---|---|---|---|---|---|
| GitHub v2.3.0 release | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0) | Release notes plus one AMD-described “minimal packaged download” containing pre-built signed DLLs and the sample library. | **Not proven immutable.** No `Immutable` marker was observed on the release page. | **Official AMD/GPUOpen release.** | AMD explicitly states that the package contains pre-built, signed DLLs and is ~125 MiB. GitHub documents that only releases explicitly marked immutable lock both tag and assets; normal release assets can otherwise be updated/deleted by authorized maintainers. |
| Exact prebuilt release ZIP | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip) | Prebuilt signed DLLs and sample library; sample media excluded. | URL is version-pinned but **not independently established as immutable**. | **Official release asset.** | **Verified metadata:** filename is `FidelityFX-Samples-v2.3.0-prebuilt.zip`; GitHub release metadata exposed size **122 MB** and SHA-256 `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`. GitHub's release-asset API defines the `digest` field as a SHA-256 digest. Raw download was inaccessible to this research environment, so the digest was not independently recomputed. |
| Git tag/tree | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/v2.3.0) | Source, documentation, third-party notices **and committed signed binaries** under `Kits/FidelityFX/signedbin`. | A tag name is not intrinsically immutable. | **Official repository.** | GitHub notes that tags may move unless protected by immutable-release semantics. |
| Commit-pinned repository tree | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/60f4ea81909200d8542eca14dccb2628b763a9a3) | Exact committed source tree plus the committed `signedbin` binaries. | **Content-addressed at Git object level**, subject to the normal limitations of Git SHA-1 identity and repository availability. | **Official repository.** | This is the strongest source-tree identity found. The commit is titled “AMD FSR SDK 2.3.0”; its tree includes the loader, upscaler and other signed DLLs. |
| GitHub-generated tag ZIP | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/refs/tags/v2.3.0.zip](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/refs/tags/v2.3.0.zip) | Snapshot of tagged repository; because `signedbin` is committed, this is not semantically “source only”. | **Extracted contents depend on tag target. Archive bytes are not stable.** | GitHub-generated official repository archive. | GitHub explicitly states that archives are regenerated and compression settings may change, so identical contents do not guarantee identical ZIP bytes. |
| GitHub-generated tag tarball | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/refs/tags/v2.3.0.tar.gz](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/refs/tags/v2.3.0.tar.gz) | Same repository snapshot in tar.gz form. | Same caveat as ZIP. | Official GitHub archive. | Not suitable for archive-byte reproducibility without an independently pinned checksum. |
| Commit-pinned generated archive | [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/60f4ea81909200d8542eca14dccb2628b763a9a3.zip](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/archive/60f4ea81909200d8542eca14dccb2628b763a9a3.zip) | Exact committed tree contents. | Extracted files are stable for that commit; outer ZIP bytes are not guaranteed stable. | Official GitHub archive mechanism. | GitHub specifically recommends commit-ID archives when reproducible extracted contents are required. |
| GPUOpen FSR SDK page | [https://gpuopen.com/amd-fsr-sdk/](https://gpuopen.com/amd-fsr-sdk/) | Product/download landing page; currently labels SDK **v2.3** as the latest. | **Mutable/current landing page.** | **Official AMD GPUOpen.** | The page links a “Download AMD FSR SDK v2.3 package” and GitHub repository. Suitable for discovery, not deterministic automated acquisition. |
| GPUOpen download endpoint | [https://gpuopen.com/download-fsr-sdk/](https://gpuopen.com/download-fsr-sdk/) | Download endpoint reached from the GPUOpen “latest version” link. | **Mutable/latest-oriented.** | Official AMD GPUOpen. | Browser restrictions prevented observing the complete redirect chain. Because the referring page explicitly presents it as the current/latest package, it should not be treated as a permanent v2.3.0 byte identity. |
| Separate AMD installer/MSI | — | None identified. | — | **Unresolved / no such official v2.3.0 channel found.** | The v2.3.0 release instead describes a single minimal packaged download. |
| AMD FidelityFX NuGet/package feed | — | None identified for the FidelityFX/FSR runtime itself. | — | **Unresolved / no official AMD runtime package found.** | NuGet is used for third-party SDK dependencies such as Microsoft's DirectX Agility SDK, not identified by AMD as the FSR SDK distribution channel. |
| Build from tagged source | Exact tree/commit URLs above | Public source and sample projects. | Commit identity can be pinned. | **Officially documented for building samples/open components; not an equivalent acquisition route for the production FSR API runtime.** | AMD's exact v2.3.0 FSR API documentation says an application **must use one of the provided signed DLLs**. Build documentation covers the samples and their dependencies, not reproduction of AMD's signed FSR 4 runtime. |

The important distinction is that both the GitHub repository snapshot and the release package can contain binary material. “GitHub source archive” does **not** mean “only rebuildable source” for this repository, because AMD commits its signed FSR DLLs into `Kits/FidelityFX/signedbin`.

## 2. Required Windows/DX12 runtime artifacts

For the documented **upscaling-only FSR API path**, AMD describes a loader DLL that manages effect DLL loading and a separate upscaler DLL; it explicitly states that applications may ship only the effect types they actually use. GPUOpen separately states that the upscaler DLL must be included alongside the loader for FSR Upscaling.

| Artifact | Package path | Role | Direct/transitive | Required at runtime? | Evidence |
|---|---|---|---|---|---|
| `amd_fidelityfx_loader_dx12.dll` | `Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll` | Public FSR API loader; contains no effect implementation and manages loading effect-type DLLs. | **Direct.** Application loads it using `LoadLibrary`/`GetProcAddress`, or the OS loader loads it when using the import library. | **Yes.** | **Verified/Upstream claim.** Exact v2.3.0 docs identify it as the loader and replacement for the former monolithic DLL. The exact tagged binary exists and GitHub displays size 25.8 KB. |
| `amd_fidelityfx_upscaler_dx12.dll` | `Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll` | Effect DLL containing providers for FSR Upscaling 4 plus legacy FSR 3/2 implementations. | **Transitive through the loader** for the documented loader path. | **Yes for upscaling.** | **Verified/Upstream claim.** v2.3.0 API docs state the function of this DLL. The exact tagged binary exists and GitHub displays size 27.4 MB. |
| `amd_fidelityfx_loader_dx12.lib` | `Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.lib` | MSVC import library for startup/dynamic-linker binding. | Direct link-time dependency if that integration form is chosen. | **No** when runtime loading is used; development/link-time only. | **Verified.** AMD recommends `LoadLibrary`/`GetProcAddress` but documents `.lib` linking as an alternative. Tagged file size displayed as 2.81 KB. |
| Loader PDB(s) | Supplied with release; exact member path not enumerated here | Symbols/debugging. | Development/debug support. | **No.** | **Upstream claim.** v2.3.0 release says PDBs are provided for the loader DLL. |
| `amd_fidelityfx_framegeneration_dx12.dll` | `Kits/FidelityFX/signedbin/...` | Frame generation provider DLL. | Separate effect DLL. | **No for an upscaling-only deployment.** | AMD explicitly says the effect split allows applications to ship only effect types they use. |
| `amd_fidelityfx_denoiser_dx12.dll` | `Kits/FidelityFX/signedbin/...` | Ray-regeneration/denoiser provider. | Separate effect DLL. | **No for upscaling-only.** | Same evidence. |
| `amd_fidelityfx_radiancecache_dx12.dll` | `Kits/FidelityFX/signedbin/...` | Radiance-caching provider. | Separate effect DLL. | **No for upscaling-only.** | Same evidence. |
| Separate FidelityFX backend DLL | None documented | DX12 backend implementation. | — | **No separate DLL identified.** | AMD states backend-specific functionality is supported through the corresponding DLL; no sibling backend DLL is listed in the FSR API DLL structure. |
| `fsrapiconfig.json` | `Samples/Upscalers/FidelityFX_FSR/dx12/config/fsrapiconfig.json` | Sample configuration. | Sample-only. | **No evidence that it is required by the production API runtime.** | **Verified location; runtime exclusion is an inference from AMD's documented loader/upscaler deployment model.** The file is explicitly under `Samples`. |
| Runtime shader/data files | None documented alongside the production DLL pair | Potential data/shader assets. | — | **No separate runtime assets established.** | **Unresolved in the strict clean-machine sense.** AMD's title-shipping instructions name the loader/effect DLLs; no separate FSR-upscaling shader/data deployment package is documented. |

**Architecture/configuration.** The intended platform is Windows x64/DX12 and the SDK samples have x64 configurations, but this web environment did not inspect the PE `Machine` field of the binaries. Therefore `AMD64` machine identity should be verified locally rather than inferred from project context. No separate `_debug`, `d`, Debug, or Release variants of the signed FSR API DLL names appear in `Kits/FidelityFX/signedbin`; Debug/Release are sample build configurations, while the documented production API uses the supplied signed DLLs.

AMD directly binds the v2.3.0 release to **FSR Upscaling 4.1.1** in its release notes. GPUOpen also says the v2.3 upscaler DLL provides FSR Upscaling 4.1.1 together with legacy FSR 2.3.4/3.1.5. This is official evidence connecting the v2.3.0 signed runtime distribution with FSR Upscaling 4.1.1 without relying on observed runtime behavior.

The proposed provider ID `0xf5a5ca1e01001001` was **not found in primary AMD/GPUOpen v2.3.0 material inspected here**. Its association with 4.1.1 therefore remains **Unresolved**. AMD's API documentation also explicitly says only version IDs returned by `ffxQuery` should be used and that IDs must not be hard-coded.

## 3. Native dependency closure

### PE-visible imports

**Unresolved.** The official v2.3.0 binary asset could not be fetched as bytes in this research environment, so neither `amd_fidelityfx_loader_dx12.dll` nor `amd_fidelityfx_upscaler_dx12.dll` was subjected to PE import/delay-import inspection. No claim such as "`VCRUNTIME140.dll` is required" is therefore promoted to Verified merely because VS2022 is AMD's build prerequisite.

The later local artifact-inspection pass should operate on DLLs extracted from the SHA-256-pinned official release ZIP and record:

- PE `Machine`/subsystem and image version;
- ordinary import descriptors;
- delay-import descriptors;
- embedded version resources;
- Authenticode `WIN_CERTIFICATE`;
- per-DLL SHA-256;
- any manifest/SxS requirements.

Suitable independent methods are Microsoft's `dumpbin /imports`/`dumpbin /headers`, LLVM PE/COFF inspection, or another PE parser; Authenticode can be inspected with `Get-AuthenticodeSignature` or Microsoft/Sysinternals signing tools. The precise tool output should become project evidence rather than assumptions.

### Dynamic/delay-loaded dependencies

One dynamic dependency is **Verified by upstream design**, independent of PE imports: `amd_fidelityfx_loader_dx12.dll` manages loading effect-type DLLs, and for upscaling the relevant effect DLL is `amd_fidelityfx_upscaler_dx12.dll`. Thus absence of an ordinary PE import from loader to upscaler would not negate the runtime dependency.

AMD also documents routing through a **driver/external provider** when a DX12 device is supplied to applicable provider/version queries. GPUOpen states that, beginning with this FSR “Redstone” model, future AMD Software: Adrenalin releases can update the ML technology selected in games without a title update. Therefore the bytes shipped beside an application do not necessarily prove which provider implementation ultimately executes on every AMD system.

The specific external-provider DLL/module names and their loader mechanism are **Unresolved** from the inspected documentation and must not be invented from driver observations.

### OS-provided dependencies

**Upstream requirements:** Windows 10 and Windows 11 are listed supported operating systems, with DirectX 12 as the supported API. Exact imported OS DLLs—e.g. kernel/user/runtime components—remain **Unresolved until PE inspection**.

For FSR Upscaling 4.1.1 specifically, AMD requires Shader Model 6.6 and states that Shader Model 6.6 requires **DirectX 12 Agility SDK 1.4.9 or later**. The v2.3.0 third-party manifest itself records DirectX Agility SDK **1.616.1**, pulled through vcpkg for the SDK environment. Whether the application already supplies a sufficient Agility runtime or depends on an OS/other deployment mechanism is application-level deployment state that cannot be inferred from the AMD DLL filenames alone.

### Redistributable dependencies

No Visual C++ Runtime DLL has been established as a dependency of the AMD binaries because the PE tables were not inspected. VS2022 being the required sample-development toolchain is **not evidence** of a particular dynamically linked VC runtime.

The DirectX Agility SDK is separately licensed Microsoft software. Microsoft's 1.616.1 license permits distribution only of object code identified as distributable, subject to its stated distribution requirements; it also prohibits distributing the software generally as a standalone offering except for permitted distributable code. Therefore, if `fsr-sdk-rs` or an application package itself carries Agility SDK files, Microsoft's package-specific distributables list and terms need to be applied separately from AMD's FidelityFX license.

### Remaining unknowns

The remaining dependency questions are: exact Win32/DirectX imports of both DLLs; VC runtime linkage; delay imports; any `LoadLibrary`/`LdrLoadDll` targets other than FidelityFX effect DLLs; exact relationship to app-local Agility SDK files; whether driver/external providers load additional modules; and whether any environment/registry/driver state becomes mandatory on a clean installation.

These are precisely the areas where static PE inspection plus clean-machine runtime tracing is still required.

## 4. Redistribution terms

### AMD FidelityFX license

The exact v2.3.0 `docs/license.md` begins with a **default license applying to all files except specifically noted exceptions**. The default license permits installation, reproduction, copying and distribution **in binary form only**, prohibits reverse engineering/decompilation/disassembly, and requires preservation of AMD's copyright, permission, disclaimer and notices in documentation or other distributed materials.

The file then enumerates a large exception set and places those listed files under an MIT-style grant permitting use, copy, modification, merge, publication, distribution, sublicensing and sale, provided the copyright and permission notice is retained.

### Binary redistribution

Crucially, the exception list explicitly names:

`Kits\FidelityFX\signedbin\amd_fidelityfx_loader_dx12.dll`

`Kits\FidelityFX\signedbin\amd_fidelityfx_loader_dx12.lib`

`Kits\FidelityFX\signedbin\amd_fidelityfx_upscaler_dx12.dll`

as well as the denoiser, framegeneration and radiancecache signed DLLs. Those listed files are followed by the MIT-style license text.

Accordingly, **the primary v2.3.0 legal text expressly places the relevant signed loader/upscaler binary files in the permissive exception set**. Practical distribution implication, without giving legal advice: the text supports redistribution of those specific AMD-provided binaries subject to preservation of the applicable copyright/permission notice.

Modification is likewise within the MIT-style grant for exception-listed files, but modifying an Authenticode-signed DLL would of course destroy the original signed-byte identity and should not be confused with redistribution of AMD's original signed artifact.

### Third-party obligations

`3rdpartynotice.md` identifies third-party components used by SDK 2.3.0, including vcpkg, DirectX Agility SDK 1.616.1, DirectX Headers 1.616.0, DXC 1.8.2505(.28), ImGui 1.87, nlohmann/json 3.11.2, Microsoft PIX, stb 2.25 and vectormath. AMD describes several of these specifically as components pulled in to build samples.

For the narrow loader+upscaler binary distribution, the notice does **not** establish that all listed third-party components are statically incorporated into those DLLs. It would therefore be incorrect to treat the entire third-party manifest as a proven runtime dependency list.

DirectX Agility SDK is the most directly runtime-relevant separate component because FSR Upscaling 4.1.1 requires Shader Model 6.6 and AMD connects SM6.6 with Agility SDK 1.4.9+. If its redistributable files are shipped, the Microsoft license and any Microsoft/third-party notice files accompanying that package apply separately.

### Ambiguities

There is an unusual drafting detail in AMD's license: the exception section refers to “source files”, while its explicit pathname list includes `.dll` and `.lib` binaries. The path enumeration unambiguously names the signed binary files, but the terminology is inconsistent. That should be recorded rather than silently normalized.

No separate AMD binary EULA tied specifically to the v2.3.0 GitHub release asset was located in the primary release/tag material inspected. Because the release ZIP itself could not be opened byte-for-byte here, the absence of an additional package-root EULA inside that ZIP is **Unresolved**, not proven.

Nothing located in the AMD exception license imposes a distinct rule specifically against internal caches, CI artifact storage, mirrors, package registries, or source-repository vendoring of the exception-listed loader/upscaler DLLs. However, that conclusion applies to those named files; it must **not** be generalized to republishing the entire SDK ZIP, whose contents include files under the default AMD license and separate third-party terms.

## 5. Third-party component matrix

| Component | Version | License | Runtime relevance | Required notice |
|---|---|---|---|---|
| DirectX 12 Agility SDK | 1.616.1 in AMD's v2.3.0 third-party manifest | Microsoft DirectX software license supplied by package | **Potentially/directly relevant to the intended FSR 4.1.1 environment.** AMD says FSR 4.1.1 requires SM6.6 and SM6.6 requires Agility SDK 1.4.9+. Whether 1.616.1 must be app-local depends on application/OS deployment. | Follow Microsoft's package license/distributables list and accompanying notices for any files actually redistributed. |
| DXC shader compiler | 1.8.2505(.28) | License linked by AMD to Microsoft's DirectXShaderCompiler repository | AMD manifest says pulled through vcpkg; **not established as a production loader/upscaler runtime dependency**. | Only relevant if corresponding DXC material is actually redistributed. |
| DirectX Headers | 1.616.0 | Microsoft DirectX-Headers license | Build/source dependency; no evidence of separate runtime artifact for the signed DLL pair. | Preserve applicable license if redistributed as source/package material. |
| vcpkg | 2026.05.25 | Microsoft vcpkg license | AMD says used to pull components for building samples; not runtime. | Relevant to redistributed tooling/source, not the two-DLL deployment closure. |
| ImGui | 1.87 | MIT | Sample/framework relevance; no evidence that it is a separate loader/upscaler runtime file. | MIT notice if redistributed/incorporated as applicable. |
| nlohmann/json | 3.11.2 | MIT | Sample/framework relevance; not established as runtime dependency. | MIT notice if applicable. |
| Microsoft PIX events | 1.0.2108180012 | Microsoft/PixEvents license linked by AMD | Development/sample instrumentation; no evidence that a separately shipped PIX runtime is required by loader/upscaler. | Applicable only to redistributed PIX material. |
| stb | 2.25 | MIT, per AMD manifest | Sample/framework source; not established as runtime dependency. | MIT notice where incorporated/redistributed. |
| vectormath | commit `ee960fad0a4bbbf0ee2e7d03fc749c49ebeefaef` | License linked in AMD manifest | Sample/framework source; not established as two-DLL runtime dependency. | Applicable license notice if incorporated/redistributed. |

The third-party manifest is therefore evidence of **SDK component licensing**, not evidence that nine third-party packages must accompany an upscaler deployment.

## 6. Artifact integrity and authenticity

The v2.3.0 source identity and binary identity must remain distinct. The release points to Git commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; GitHub reports the commit signature as verified. That establishes repository revision identity, not the bytes of an independently produced DLL.

| Artifact | Size | SHA-256 | Authenticode signer | Timestamp | Bound to v2.3.0 how? |
|---|---:|---|---|---|---|
| `FidelityFX-Samples-v2.3.0-prebuilt.zip` | GitHub metadata: **122 MB**; release prose: ~125 MiB | **`f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`** | N/A; ZIP itself was not established as Authenticode-signed | Asset date shown by indexed GitHub metadata: Jun 22, 2026; release published Jun 24, 2026 | Exact asset under `releases/download/v2.3.0/...`; GitHub's release-asset metadata supplies SHA-256. |
| `amd_fidelityfx_loader_dx12.dll` | GitHub display: **25.8 KB** | **Unresolved**; not calculated here | **Unresolved independently.** AMD calls the release DLLs “signed”, but signer/certificate was not inspected. | **Unresolved** | File is committed in `Kits/FidelityFX/signedbin` at tag v2.3.0/commit baseline and included in the documented signed-runtime set. |
| `amd_fidelityfx_upscaler_dx12.dll` | GitHub display: **27.4 MB** | **Unresolved**; not calculated here | **Unresolved independently.** AMD upstream claims signed distribution. | **Unresolved** | File is committed at exact v2.3.0; AMD documents this DLL as the upscaler provider container and v2.3.0 as containing FSR Upscaling 4.1.1. |
| `amd_fidelityfx_loader_dx12.lib` | GitHub display: **2.81 KB** | **Unresolved** | N/A/not established | N/A | Exact v2.3.0 committed file; development/link-time artifact. |

**Published hashes.** A SHA-256 is available in current GitHub release-asset metadata for the prebuilt ZIP. No AMD-published per-DLL SHA-256/SHA-1 manifest was found in the inspected v2.3.0 sources.

**Git signing.** GitHub reports the release's underlying commit as created on GitHub.com with a verified signature using key `B5690EEEBB952194`. This does not establish that the `v2.3.0` tag itself is an independently signed annotated Git tag; no such claim is made here.

**Authenticode.** AMD repeatedly calls the supplied DLLs **signed**, including both the release page and product page. The actual certificate subject, chain, signing algorithm and timestamp countersignature were not observable without downloading and inspecting the binaries. Those fields therefore remain **Unresolved**.

**Reproducible builds.** No AMD statement or build recipe was found claiming bit-reproducible generation of the signed v2.3.0 runtime DLLs from the public source. More importantly, the public source tree visibly contains FSR2/FSR3 upscaler implementation source while FSR Upscaling 4.1.1 is supplied through the signed upscaler DLL.

The strongest evidence-supported way to bind an AMD binary to this project baseline is therefore:

1. identify the official v2.3.0 release asset by its fixed release URL;
2. verify the release asset against SHA-256 `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`;
3. extract the DLLs and record **project-controlled per-DLL SHA-256 values**;
4. locally inspect and record their Authenticode certificate subject/chain/timestamp;
5. optionally byte-compare them with the same files from the commit-pinned `signedbin` tree.

This binds archive identity, extracted DLL identity, and publisher signing as separate facts rather than treating any one as a substitute for the others.

## 7. Reproducible acquisition assessment

### Stable official URLs

The GitHub release asset URL is explicitly versioned with `v2.3.0` and a versioned filename. The commit-pinned repository/archive URLs are stronger still for **repository content identity** because they use the full Git commit SHA.

A fixed official release URL plus expected size plus SHA-256 is technically sufficient to reject changed/corrupted archive bytes, regardless of whether the storage URL itself is intrinsically immutable. GitHub's release-asset metadata now provides a SHA-256 field specifically for this purpose.

### Mutable/unsuitable URLs

`https://gpuopen.com/amd-fsr-sdk/` and its `/download-fsr-sdk/` link are “latest/current” product endpoints and should be regarded as discovery locations, not deterministic version identities.

A Git tag is weaker than a full commit SHA because GitHub explicitly documents that tags may move. Generated GitHub source archive **bytes** are also not stable: GitHub may regenerate them with changed compression while preserving extracted contents.

The v2.3.0 release was not observed carrying GitHub's `Immutable` marker. GitHub states that an immutable release displays that marker and locks both assets and the tag. The conservative conclusion is therefore **not to rely on URL immutability alone**.

### Available integrity checks

- **Source revision:** full Git commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- **Commit authenticity signal:** GitHub “Verified” signature, GPG key ID `B5690EEEBB952194`.
- **Prebuilt ZIP bytes:** SHA-256 `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`.
- **DLL publisher authenticity:** AMD says the DLLs are signed; exact Authenticode signer/timestamp must still be recorded locally.
- **Individual DLL byte identity:** no primary-source per-DLL SHA-256 found; project-side hashes should be calculated after extracting a verified official archive.
- **Reproducible-build verification:** none found.

Therefore a mechanism based on **fixed official URL + expected cryptographic hash + expected size + optional/independent Authenticode verification** is supported by the available evidence. This statement establishes feasibility; it does not choose that mechanism as project policy.

### Offline/cache implications

An online-first acquisition can be made repeatable by validating the official archive before admitting it into a cache keyed by its cryptographic digest. Subsequent offline builds need only the validated cached bytes. CI likewise need not redownload an unchanged asset if its cache/artifact store preserves the hash-verified archive or extracted hash-verified DLLs.

An explicit user-supplied artifact path is compatible with the same provenance model if the supplied archive/DLL bytes are checked against the project's recorded expected identities rather than trusted by filename.

For the specifically exception-listed AMD loader/upscaler DLLs, the MIT-style grant contains no channel-specific prohibition on internal caching, CI storage, mirroring or redistribution. Replicating the **entire SDK archive**, however, would carry the package's mixed AMD/default and third-party licensing context and should not be treated as legally identical to caching or redistributing the two exception-listed runtime DLLs.

The DirectX Agility SDK has its own Microsoft distribution restrictions and cannot simply inherit AMD's redistribution terms.

## 8. AMD binary versus source-built runtime

| Property | AMD-distributed binary | Source-built v2.3.0 |
|---|---|---|
| Official production FSR API status | **Required by AMD.** Exact docs say applications using the FSR API must use one of the provided signed DLLs. | AMD documents building samples/open components, but does **not** document a source build as an equivalent replacement for the required signed production DLL. |
| Baseline identity | Official v2.3.0 release asset and/or exact committed `signedbin` objects. | Can pin source to commit `60f4ea81909200d8542eca14dccb2628b763a9a3`. |
| Compiler/toolchain | None required for consuming DLLs; import-library linking optional. | AMD's sample build minimum is VS2022 + Windows 10 SDK 10.0.18362.0, with vcpkg. |
| FSR Upscaling 4.1.1 | **Yes.** Explicitly part of the v2.3.0 release/upscaler distribution. | **No evidence that the public v2.3.0 tree contains source sufficient to reproduce the 4.1.1 provider.** The tree exposes FSR2/FSR3 upscaler implementation source while the FSR4-capable path is supplied in `signedbin`. |
| Legacy FSR2/FSR3 source | Runtime DLL contains legacy providers. | Public tree visibly contains `ffx_fsr2.cpp`, `ffx_fsr3upscaler.cpp`, `ffx_provider_fsr2.cpp`, etc. |
| Closed/internal providers | Signed DLL can contain implementation not represented by open source, demonstrably relevant to FSR 4.1.1. | Cannot reproduce source that is not present. |
| Expected behavioral equivalence | This is the upstream-supported production target. | **Not established.** A locally compiled set of public implementations must not be assumed equivalent to AMD's signed provider bundle. |
| Publisher signature | AMD states the release DLLs are signed; exact certificate still needs local verification. | A local build will not carry AMD's original Authenticode signature unless signed by AMD through some separate process not documented here. |
| Byte reproducibility | Official archive byte identity can be pinned by SHA-256; individual DLLs can be hashed after extraction. | No upstream reproducible-build claim found; output varies with compiler, toolchain and build configuration, and cannot reconstruct unavailable providers. |
| Redistribution | Exact signed DLL pathnames are in AMD's MIT-style exception list. | Public files have file-specific/default licensing as enumerated by AMD; source redistribution cannot be generalized from a single repository-wide “MIT” label. |
| Integrity strength | Release SHA-256 + per-DLL project hashes + Authenticode can bind exact AMD bytes. | Git commit strongly binds inputs; resulting binaries require separately recorded hashes and do not establish equivalence to AMD's binaries. |
| Automatic driver/external provider interaction | Supported API can route to external/driver providers; GPUOpen advertises driver-based ML upgrades. | Building an open implementation does not establish equivalence to this signed/driver-aware provider mechanism. |

Accordingly, **Model B is not evidence-equivalent to Model A for the stated FSR Upscaling 4.1.1 target**. That is not merely a signing difference: the public source tree inspected at the v2.3.0 baseline does not expose an obvious FSR4 provider source corresponding to the functionality AMD says is contained in the signed upscaler DLL.

The observed provider identifier `0xf5a5ca1e01001001` remains **Unresolved as an official v2.3.0 identifier**. The correct future verification is to query provider IDs/names from the exact hash-pinned AMD runtime on the target device/driver and record the result. AMD expressly warns against hard-coding those IDs.

## 9. Clean-machine deployment gaps

Static source/release analysis does **not** yet prove a complete clean-target deployment closure.

A later clean Windows VM or equivalent validation should use the exact hash-pinned v2.3.0 DLLs, with no globally installed FidelityFX SDK, and establish:

1. whether `amd_fidelityfx_loader_dx12.dll` plus `amd_fidelityfx_upscaler_dx12.dll` are sufficient FidelityFX-local files for creating the intended upscaler context;
2. the exact ordinary and delay-loaded PE dependency set;
3. whether a VC runtime must be installed or shipped;
4. which DirectX/Agility SDK files are actually loaded and whether they are app-local or OS-provided;
5. whether the loader searches for any FidelityFX/support files not documented in the title-shipping instructions;
6. which external/driver-provider modules are loaded on AMD hardware;
7. the provider names and IDs actually returned by `ffxQuery` for the tested device/driver, including whether `4.1.1` maps to the observed `0xf5a5ca1e01001001`;
8. whether behavior differs with driver/external provider routing disabled/unavailable where such a configuration is supported;
9. the module load paths, versions, hashes and signer identities for every non-OS DLL involved;
10. whether a clean supported Windows installation lacking app-local Agility components can satisfy the SM6.6/FSR 4.1.1 requirement.

A Process Monitor/ETW/module-load trace combined with static PE analysis would close the principal gap left by this report: “everything actually required on a clean target machine.”

## 10. Differences from current project assumptions

### Confirmed assumptions

- `v2.3.0` is an official release and is associated with commit `60f4ea8`; the full commit identity used here is `60f4ea81909200d8542eca14dccb2628b763a9a3`.
- AMD distributes pre-built, signed native DLLs for the v2.3.0 FSR SDK.
- The exact release explicitly includes **AMD FSR Upscaling 4.1.1**.
- The production DX12 FSR API is designed around supplied signed DLLs rather than a machine-global SDK installation.
- The upscaling runtime is split into a loader and an upscaler effect DLL rather than one monolithic DLL.

### Refined assumptions

- The minimum documented FidelityFX-local upscaling set is **two runtime DLLs**, not “one FidelityFX DLL”: `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`. The `.lib` is optional link-time material.
- “SDK source archive” and “binary distribution” overlap: the exact Git tree itself commits AMD signed binaries.
- The top-level legal situation is **not accurately represented by saying simply “the SDK is MIT.”** The exact license has a restrictive default plus an enormous explicit MIT-style exception set; the relevant signed DLLs are in that exception set.
- The release asset has a usable SHA-256 identity in GitHub metadata, so deterministic archive verification need not rely on filename/version strings alone.
- A version-pinned GitHub release URL is not by itself evidence of immutable content; the hash is still material. GitHub distinguishes ordinary and immutable releases explicitly.
- The deployed DLL bytes may not uniquely determine the provider executed on every machine because the API supports driver/external providers and AMD advertises automatic driver-delivered ML upgrades.

### Contradicted assumptions

- Any assumption that the open v2.3.0 source can straightforwardly reproduce the AMD-supplied **FSR Upscaling 4.1.1 production runtime** is unsupported and contradicted by the documented requirement to use AMD's supplied signed DLL plus the absence of corresponding FSR4 implementation source in the inspected public upscaler tree.
- Any assumption that the observed numeric provider ID `0xf5a5ca1e01001001` is an officially stable identifier for FSR Upscaling 4.1.1 is unsupported; AMD explicitly says not to hard-code version IDs.
- Any assumption that neighboring signed DLLs such as framegeneration/denoiser/radiancecache must all ship with upscaling is contradicted by AMD's explicit effect-splitting design.

### New unresolved questions

- Exact per-DLL SHA-256 values of the binaries extracted from `FidelityFX-Samples-v2.3.0-prebuilt.zip`.
- Exact Authenticode signer subject, chain, algorithm and timestamp for loader/upscaler.
- Exact PE imports and delay imports, including VC runtime requirements.
- Exact ZIP member layout versus the repository `Kits/FidelityFX/signedbin` layout; repository paths are verified, but the release ZIP directory listing was not independently enumerated in this environment.
- Exact PE machine type/file-version metadata.
- Whether an additional license/EULA file exists only inside the prebuilt ZIP.
- Exact runtime module/dependency closure on a clean supported Windows target.
- Official mapping, if any, between FSR Upscaling 4.1.1 and provider ID `0xf5a5ca1e01001001`.
- Which provider is selected on each relevant GPU/driver configuration when AMD's documented external/driver provider mechanism is active.

## 11. Evidence inventory

1. **AMD/GPUOpen GitHub v2.3.0 release** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0) — release date 2026-06-24; tag `v2.3.0`; short commit `60f4ea8`; release notes explicitly cover FSR Upscaling 4.1.1 and the prebuilt signed package.

2. **Exact release commit** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3) — full Git commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; tree shows signed binaries and public FSR2/FSR3 upscaler source.

3. **Official prebuilt release asset** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip) — filename `FidelityFX-Samples-v2.3.0-prebuilt.zip`; GitHub metadata observed SHA-256 `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`, size 122 MB. Raw bytes were not downloadable in the research environment.

4. **Exact v2.3.0 FSR API documentation** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md) — signed-DLL requirement, loader/effect split and DX12 backend model.

5. **Exact v2.3.0 loader DLL** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll) — GitHub display size 25.8 KB.

6. **Exact v2.3.0 upscaler DLL** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll) — GitHub display size 27.4 MB.

7. **Exact v2.3.0 loader import library** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.lib](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.lib) — GitHub display size 2.81 KB.

8. **Exact v2.3.0 AMD license** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/docs/license.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/docs/license.md) — default binary-only terms plus explicit MIT-style exception list containing the relevant signed DLLs.

9. **Exact v2.3.0 third-party notice** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/3rdpartynotice.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/3rdpartynotice.md) — component/version/license manifest.

10. **Exact v2.3.0 sample-build documentation** — [https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/docs/getting-started/building-samples.md](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/v2.3.0/docs/getting-started/building-samples.md) — VS2022, Windows SDK and vcpkg requirements.

11. **AMD GPUOpen FSR SDK page** — [https://gpuopen.com/amd-fsr-sdk/](https://gpuopen.com/amd-fsr-sdk/) — current v2.3 product page; signed DLL model, automatic driver updates, FSR 4.1.1 hardware/API requirements.

12. **GPUOpen current download endpoint** — [https://gpuopen.com/download-fsr-sdk/](https://gpuopen.com/download-fsr-sdk/) — official latest-package endpoint; unsuitable as a standalone immutable identity.

13. **Microsoft DirectX Agility SDK 1.616.1 package** — [https://www.nuget.org/packages/Microsoft.Direct3D.D3D12/1.616.1](https://www.nuget.org/packages/Microsoft.Direct3D.D3D12/1.616.1) — exact version listed by AMD's v2.3.0 third-party manifest. NuGet reports it as Microsoft's package.

14. **Microsoft DirectX 1.616.1 license** — [https://www.nuget.org/packages/Microsoft.Direct3D.D3D12/1.616.1/License](https://www.nuget.org/packages/Microsoft.Direct3D.D3D12/1.616.1/License) — separate installation/distribution terms and distributable-code conditions.

15. **GitHub source-archive stability documentation** — [https://docs.github.com/en/repositories/working-with-files/using-files/downloading-source-code-archives](https://docs.github.com/en/repositories/working-with-files/using-files/downloading-source-code-archives) — commit/tag behavior and explicit statement that archive compression/outer bytes may change.

16. **GitHub release-asset API documentation** — [https://docs.github.com/en/rest/releases/assets](https://docs.github.com/en/rest/releases/assets) — documents asset `size`, SHA-256 `digest`, and mutable release-asset operations.

17. **GitHub immutable-release documentation** — [https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases) — defines asset/tag locking and the visible `Immutable` release marker.

**Research boundary:** the principal missing evidence is not another documentation search; it is inspection of the exact official binary bytes. Until the SHA-256-pinned ZIP is downloaded in an environment permitting raw artifact access and its DLLs are PE/AuthentiCode-inspected, the VC/Windows import closure, individual DLL hashes, machine type, file-version resources, signer identity, certificate timestamp, and clean-machine runtime closure should remain explicitly **Unresolved**.
