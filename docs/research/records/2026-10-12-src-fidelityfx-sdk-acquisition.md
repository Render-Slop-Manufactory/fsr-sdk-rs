# AMD FidelityFX SDK v2.3.0 — acquisition, licensing, integrity, and cross-platform verification

**Record date / evidence cutoff:** 2026-10-12 (Europe/Berlin)  
**Proposed archive path:** `docs/research/records/2026-10-12-src-fidelityfx-sdk-acquisition.md`  
**Evidence method:** public-source review only; all cited web sources accessed 2026-10-12.  
**Status:** source-research record, **not** an implementation, legal opinion, SDK acceptance, or change to an existing architectural decision.

## 1. Question and boundaries

Can `fsr-sdk-rs` offer an explicit, reproducible developer-side bootstrap for AMD FidelityFX SDK **v2.3.0**, staging its API headers and signed Windows x86-64 / DirectX 12 DLLs under `external/FidelityFX-SDK/v2.3.0/`, for verification on Windows and Linux (including Windows test executables run through Proton), without introducing automatic acquisition or DLL selection into published Rust crates?

Relevant project facts **supplied as context, not independently audited**: `fsr-sdk` 0.2.0, `fsr-sdk-sys` 0.1.1; project license MPL-2.0; SDK not bundled with crates; native baseline v2.3.0; runtime DLL loading and GPU verification explicitly application-controlled; ordinary Cargo builds independent of the native SDK. No local repository, verification manifests, CI configuration, binary files, AMD GPU, Windows host, or Proton environment were inspected.

Evidence labels used below: **Verified** = directly established by identified primary public material (not necessarily by executing or downloading it); **Upstream claim** = an upstream assertion not independently reproduced; **Inference** = engineering/legal interpretation beyond direct observation; **Unresolved** = evidence insufficient. Source identifiers resolve to direct URLs in §11.

## 2. Method and pinned upstream inputs

The investigation read the **v2.3.0 release page and its asset-list endpoint**, the linked full commit, version-pinned AMD license and third-party notice, and API documentation/source headers at `v2.3.0` or the full commit. GitHub, Microsoft, `osslsigncode`, Valve, and vkd3d-proton documentation were consulted for distribution and verification semantics. `main`-branch content, mirrors, search snippets, and pre-existing project assertions were not substituted for versioned primary evidence. The release ZIP was **not downloaded**, enumerated, hashed, extracted, or executed.

| Input | Fixed value / location | Evidence and status |
|---|---|---|
| Authoritative project/release | `GPUOpen-LibrariesAndSDKs/FidelityFX-SDK`, release tag `v2.3.0`: https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0 | **Verified** release page [S1]. Published 2026-06-24 according to the page. |
| Git commit | `60f4ea81909200d8542eca14dccb2628b763a9a3` | **Verified** resolution of release-linked commit [S2]. GitHub labels its commit signature verified; this is not an AMD signature over the ZIP. |
| Named release asset | `FidelityFX-Samples-v2.3.0-prebuilt.zip` | **Verified** asset listing [S3]. |
| Canonical download | https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/v2.3.0/FidelityFX-Samples-v2.3.0-prebuilt.zip | **Verified** from official release listing [S1][S3]; a release-hosted URL, not content-addressed storage. |
| GitHub-published archive digest | SHA-256 `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273` | **Verified as published metadata**, **not independently recomputed** [S3]. Asset-list timestamp `2026-06-22T13:40:03Z`, displayed size `122 MB` (rounded; not a byte-count pin). |
| Authoritative versioned license | https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/docs/license.md | **Verified** text [S4]; a second copy exists under `Kits/FidelityFX/docs/license.md` [S5]. |
| Versioned third-party notice | https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/3rdpartynotice.md | **Verified** text [S6]. |
| Versioned API documentation | https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md | **Verified** [S7]. |

GitHub states it computes SHA-256 digests at upload time and exposes them via its release UI/API [S8]. It also offers **optional release immutability**: only releases using that feature lock tags and assets, with a verifiable attestation [S9][S10]. **Unresolved:** no immutability attestation or affirmative `Immutable` designation for this particular v2.3.0 release was established. The digest in [S3] is authoritative *publication evidence*, not a downloaded-file measurement, a separately signed AMD checksum, or proof that an asset can never be removed/replaced. A Git commit pin does not itself authenticate a separately uploaded ZIP.

## 3. Acquisition and minimum verification payload

**Upstream claim:** AMD describes v2.3.0 as one minimal prebuilt package containing **signed DLLs and a sample library**, sufficient to get started but **without sample media assets**; media delivery is separately required to run AMD's samples [S1]. Its SDK structure documentation describes host-side headers and prebuilt libraries [S11]. This supports choosing the release asset rather than reconstructing a runnable SDK from source snapshots. **Unresolved:** the actual ZIP member inventory, top-level directory layout, exact header versions, DLL machine architecture, embedded signatures, and embedded notices were not inspected.

**Verified source-level API structure:** the SDK has five C entry points (`ffxCreateContext`, `ffxDestroyContext`, `ffxDispatch`, `ffxQuery`, `ffxConfigure`). AMD recommends explicit `LoadLibrary`/`GetProcAddress`; `.lib` startup linking is an alternative. Since SDK 2.0 the loader DLL manages separate effect DLLs, and the upscaler DLL houses FSR 2/3/4 providers [S7]. The API documentation identifies the DX12 backend as the supported backend in that release [S7]; the source README separately records Vulkan as not currently supported [S12]. Neither assertion establishes Proton compatibility.

For the **stated C ABI and DX12 upscaler scope**, the following is a **proposed smallest *logical* payload**, subject to a mechanical include-closure audit and actual test-manifest review:

- C-facing API declarations: `Kits/FidelityFX/api/include/ffx_api.h`, `ffx_api_types.h`, `dx12/ffx_api_dx12.h`, and `Kits/FidelityFX/upscalers/include/ffx_upscale.h`. Their v2.3.0 source files and include relationships are directly visible [S13][S14][S15][S16]. The DX12 header additionally includes Microsoft D3D12/DXGI headers, which belong to an appropriate Windows SDK/cross-toolchain, not to this AMD subset [S15].
- Native runtime: `Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_upscaler_dx12.dll`, as named in AMD's licensed-file list and DLL architecture documentation [S4][S7]. Preserve their original bytes and predictable relative placement. The actual loader-to-provider lookup convention and transitive dependencies need tests; no global DLL search should be presumed.
- Conditional only: `ffx_api_loader.h` for AMD's **C helper** (it itself uses Windows loader APIs), `amd_fidelityfx_loader_dx12.lib` for import-library linking, and C++ `.hpp` files for C++ test clients [S4][S17]. They are not intrinsic requirements of a Rust integration already resolving the five exports dynamically. Denoiser, frame-generation, radiance-cache DLLs, sample executables, media-delivery program, PDBs and sample media are not justified by the stated upscaler scope [S1][S7].

**Unresolved:** exact compile-time include closure and SDK file list used by `tools/verification/` cannot be established from a project summary alone. Source snapshots and ZIP contents must be compared before claiming archive sufficiency for this project's checks. A source archive at a pinned commit is useful for auditing headers/notices, but replacing the vendor's signed binaries by compiling from source would not reproduce the signed release, and GitHub-generated source ZIPs are not equivalent to the named prebuilt release asset [S1][S18].

## 4. License findings

**Verified, with an important qualification:** the v2.3.0 `docs/license.md` is **not a blanket MIT license for everything**. It first applies a default AMD **binary-form-only** installation/reproduction/distribution permission to files not excepted; it prohibits reverse engineering, decompilation and disassembly and requires its copyright, permission, disclaimers and notices on redistribution. It then explicitly enumerates numerous paths (including SDK C headers, four effect DLLs, the loader DLL and import library) under a separate **MIT-style grant**, requiring retention of its AMD copyright/permission notice [S4]. The SDK README's broad statement that the *samples* are MIT-licensed does not override this path-based distinction [S12]. Individual file headers also carry AMD copyright and the MIT-style grant [S13–S16].

The v2.3.0 third-party notice identifies **vcpkg, DX12 Agility SDK, Microsoft DirectX-Headers, DirectXShaderCompiler, imgui, nlohmann/json, Microsoft PIX, stb and vectormath** with license URLs or comments [S6]. It is an inventory, **not** a substitute for those component licenses. The default AMD text, path exceptions, separately licensed bundled files, and embedded archive notices must therefore be assessed together. **Unresolved:** there has been no full ZIP-level reconciliation of actual members against every applicable license/notice file.

### Acquisition/licensing matrix

| Artefact or file class | Public-source evidence | License / handling consequence |
|---|---|---|
| Prebuilt release ZIP, as a whole | GitHub hosts named asset [S1][S3]; ZIP member list not inspected. | **Unresolved** as one uniformly licensed redistributable object. Direct user-initiated acquisition from upstream is preferable to project rehosting. Retain original archive notices when caching it; do not assume a single license covers every member. |
| `ffx_api.h`, `ffx_api_types.h`, `dx12/ffx_api_dx12.h`, `ffx_api_loader.h` | Individually enumerated in v2.3.0 MIT-style exception [S4]; example source headers [S13–S15][S17]. | **Verified** AMD MIT-style terms for these paths; preserve AMD copyright, permission and warranty disclaimer on copies/substantial portions. Microsoft SDK header dependencies are separately sourced/licensed. |
| `upscalers/include/ffx_upscale.h` | Enumerated in exception and inspected in versioned source [S4][S16]. | **Verified** AMD MIT-style terms; same notice retention. Additional headers only as dictated by actual include closure. |
| `signedbin/amd_fidelityfx_loader_dx12.dll` | Explicit license-list entry; described as loader [S4][S7]. | **Verified** inclusion in AMD MIT-style exception; permitted copying/distribution **under that license** with notices, subject to other applicable terms. Signature/publisher not independently checked. |
| `signedbin/amd_fidelityfx_upscaler_dx12.dll` | Explicit license-list entry; described as upscaler provider container [S4][S7]. | **Verified** same AMD MIT-style exception and notice condition; exact released bytes and signature unverified. |
| Other named signedbin effect DLLs and `amd_fidelityfx_loader_dx12.lib` | Denoiser, framegeneration, radiancecache DLLs and loader `.lib` individually listed [S4][S7]. | **Verified** exception for the named files, but omit from minimum installation unless test scope requires them. No inference about unknown files in the folder. |
| Files **not** listed under the exception | Default restrictive permission in v2.3.0 license [S4]. | **Verified** default **binary-only**, no-reverse-engineering wording. Do **not** relicense under MIT; redistribution of source-form material without an applicable exception or separate grant needs specific analysis. |
| Third-party libraries, fonts, programs and ancillary files | Third-party inventory and various vendor-specific license files referenced in the exception listing [S4][S6]. | **Unresolved** per-member obligations until ZIP inventory and individual licenses audited. Avoid extracting or republishing them incidentally. |
| Versioned `docs/license.md`, `3rdpartynotice.md`, any selected component licenses | AMD's own exception includes the documentation/notice paths [S4]; notice content [S6]. | Preserve **unmodified full texts** in local provenance/notices; maintain per-file attribution mapping. A machine-generated summary alone is insufficient. |

**Permission analysis (Inference, not legal clearance):** a publicly published **script containing only a pinned URL and verifier**, with no AMD payload bundled, is materially different from redistributing AMD binaries; local user-initiated download/caching/extraction of the identified MIT-exception files is consistent with their broad copyright grants [S4]. The default grant also expressly refers to installing and copying binary-form software. But a ZIP cache may contain files with other grants, and no review established *all* conditions for that cache. A project-hosted mirror, public CI cache/artifact, downloadable pre-extracted DLL collection, or crate package containing the SDK constitutes additional copying/distribution and requires a complete per-member license check and notices. Keeping binary files outside Git/Cargo mitigates that risk; it does not automatically make CI transfers or external caches private/non-distributive.

**Separate service terms:** the release page visibly exposes the asset without an archive-specific agreement or click-through in the material reviewed [S1]; this is **not proof none exists** inside the ZIP or in relevant download-service terms. GitHub's public-repository terms do not generally bar lawful access to public content, while its API terms impose rate/abuse constraints [S19]. Download automation should use ordinary public endpoints, respect service limits, avoid shared tokens, and never equate URL reachability with a copyright grant. No additional AMD/GPUOpen v2.3.0 archive-specific EULA was located **in the examined public release page and versioned notices**; full asset-level contractual review remains unresolved.

## 5. Integrity, authenticity and trust

**Expected values, separated by provenance:**

| Object | SHA-256 | Evidentiary status |
|---|---|---|
| Official named ZIP | `f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273` | **Verified as GitHub-published release metadata**, not locally measured [S3]. |
| `amd_fidelityfx_loader_dx12.dll` | `e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa` | **Supplied project baseline only; unresolved** independently. |
| `amd_fidelityfx_upscaler_dx12.dll` | `d0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46` | **Supplied project baseline only; unresolved** independently. |

No authoritative per-DLL digest manifest was found in the reviewed release metadata. The two DLL digests **cannot currently be corroborated against a directly observed official release ZIP**. Do not treat repetition in an existing local project as independent evidence. A reviewed, immutable project-side pin of the archive SHA-256 is independent **of the bytes received during a future download**, but its original publication shares GitHub's supply-chain trust domain; it is not a vendor-signed manifest.

A defensible **proposed** verification sequence:

1. Fix expected owner/repository, tag, release asset filename, full source commit, archive SHA-256, and after initial confirmation a per-file SHA-256 manifest **in reviewed project-controlled metadata**. Read live GitHub metadata only as corroboration/diagnostics, never as the sole source for expected hashes. Do not follow `latest` or arbitrary mirrors [S1–S3][S8].
2. Require HTTPS and the canonical release link (permit only expected GitHub asset-CDN redirects); bound downloads by size and time, validate HTTP completion, and write to a temporary file. Calculate **flat SHA-256 over the entire ZIP**, compare to the independently pinned expected value **before** extraction, and fail closed. A rounded UI size is not an exact-length integrity check.
3. Validate ZIP central directory, bounded entry count/uncompressed sizes/compression ratios, member type, exact allowlisted paths, duplicates, Windows case-insensitive collisions, absolute/drive/UNC paths, `..`, Unicode/name ambiguities, symlink/reparse-point escapes and overwrite attempts. Extract into a fresh staging directory without following links; verify required files and per-file SHA-256; compare notices; publish atomically by same-filesystem rename where possible. No partially accepted installation.
4. On **Windows**, verify the **embedded Authenticode signature** of each staged DLL with the appropriate Windows Authenticode policy (e.g. `signtool verify /pa /all /v`, plus programmatic `WinVerifyTrust`/chain-policy checks where needed). Require a trusted chain, intended code-signing usage, explicit expected AMD legal-entity/publisher identity, acceptable signing time/timestamp chain and revocation policy; record the observed subject, issuer, thumbprint, timestamp, policy, results and tool versions. `signtool /tw` is only a **warning** if not timestamped, so strict timestamp policy requires interpreting/enforcing it [S20][S21][S22]. Do not invent an AMD certificate thumbprint before inspecting the binaries.
5. On **Linux**, `osslsigncode verify` can inspect/cryptographically verify Authenticode PE signatures and evaluate chains with configured signer and TSA CA bundles/CRLs; its `-CAfile`, `-TSA-CAfile`, `-time`, revocation options, and `-require-leaf-hash` have documented semantics [S23]. Compare actual signer identity with an independently approved AMD identity, not merely a self-described name. **Do not** call `osslsigncode` success exact equivalence to Windows trust-policy success; store a separate Linux verification status. Without trusted roots, timestamp validation, or revocation information, report **indeterminate** rather than authenticated.
6. Reverify existing installations and offline imported archives against project pins before reuse. Make offline verification distinguish `hash/identity OK` from `signer trust validated` and `revocation status unavailable`. Never repair a modified installation silently; fail or require a separate explicit replacement action. Log provenance but no executable code execution is needed for acquisition.

**Trust distinctions:** flat ZIP/per-file SHA-256 establishes bit-for-bit correspondence to **a precommitted expected value**; it does not prove who authored it. Authenticode links the signed *PE image portions* to a signing certificate and a configured trust policy, not to the release tag or archive, and does not sign SDK text headers. Microsoft's PE documentation identifies fields excluded from the Authenticode image hash, making full-file SHA-256 additionally valuable [S24]. A trusted code-signing signature and timestamp do **not** prove security, functional correctness, AMD GPU support, or Proton compatibility. GitHub's verified commit signature likewise does not sign the separately uploaded DLLs.

## 6. Windows versus Linux/Proton

**Inference, strongly supported by file-format and tooling documentation:** Linux can obtain an ordinary GitHub-hosted ZIP, calculate SHA-256 and extract Windows PE DLLs **without executing them or installing Proton**; an extraction that writes the original bytes unchanged should preserve embedded Authenticode signatures. This is not a report of a performed Linux download or byte-identity comparison. ZIP case/path behavior still differs across filesystems, making a shared cross-platform safe extractor valuable [S3][S23][S24].

On Windows, native trust assessment can use Microsoft SignTool/WinVerifyTrust [S20][S21]. On Linux, `osslsigncode` can supply a meaningful but policy-distinct assessment, provided trusted roots, timestamp verification and revocation behavior are explicitly specified [S23]. Strictly air-gapped verification may validate pinned hashes but cannot necessarily establish *fresh* revocation status; cached CRLs/trust material and their validity periods must be recorded. No Windows installation, global DLL registration, administrator rights, or Proton prefix is needed for **acquisition**.

**Separate execution boundary:** Proton is Wine-based and its Direct3D 12 path uses **vkd3d-proton**, which implements D3D12 over Vulkan and has GPU/driver requirements [S25][S26]. AMD's documented DLL loader and provider discovery are Windows-process behavior [S7]. A verification harness must explicitly stage the chosen native Windows x86-64 DLLs adjacent to, or otherwise resolve them for, its Windows test executable and confirm module resolution under the chosen Proton version/prefix. Test the loader's provider search path, load order, PE imports and dependent Windows/D3D12 DLLs, architecture match, vkd3d-proton capabilities, GPU/driver feature levels and dispatch return codes. Do not infer discovery from success at downloading, extracting, cross-compiling, checking signatures, enumerating exports, or loading just one DLL. **No Proton GPU dispatch compatibility is established by this research.**

## 7. Proposed bootstrap policy: assessment and alternatives

The proposed **explicit opt-in** CLI, with no `cargo build` downloads, no hidden library DLL search/installation, default `external/FidelityFX-SDK/v2.3.0/` prefix, pinned hashes, offline reuse, fail-closed verification, protected extraction, atomic installation and retained notices is **technically coherent** and respects the project's stated architecture (**recommendation, not acceptance**). Versioned SDK files should remain absent from Git, release archives and Cargo packages. Provenance should record upstream URL/release/commit, expected and observed archive hash, per-file hashes, extraction allowlist, preserved license paths, signature policy/results and tool/trust-store versions; this metadata should not be treated as a cryptographic attestation in itself.

Conditions requiring explicit design decisions:

- **Legal:** initially restrict extraction to reviewed MIT-exception headers/DLLs plus complete relevant notices. Avoid publishing the entire downloaded ZIP or a public CI cache until full member-by-member licensing is audited; preserve permissions/disclaimers for every redistributed item [S4][S6].
- **Offline:** accept a user-supplied previously downloaded ZIP or verified staging tree, **never** downgrade to an unverified artifact; permit hash verification offline and clearly mark online PKI/revocation checks unavailable rather than assuming success. Decide whether an unavailable trust check blocks installation or only blocks a stricter verification tier.
- **Security:** untrusted existing destination directories, reparse points/symlinks, hardlinks, same-name races, case-folding conflicts, decompression bombs, world-writable parent paths and interrupted replacements need explicit controls. Atomic directory rename must be within the same volume/filesystem and must not replace unreviewed user content. Retain a machine-checkable installation manifest plus a human-readable license/provenance record.
- **Reproducibility:** freeze the minimal file manifest only **after** inspecting the ZIP and the actual ABI test include graph. Recheck on each use or from an appropriately authenticated immutable cache; archive hash plus approved file hashes are stronger than filenames/version strings alone. Host trust policies will still vary by OS, clock and revocation connectivity.

| Alternative | Assessment |
|---|---|
| Official prebuilt release ZIP, locally downloaded and allowlist-extracted | **Preferred for investigation**: exact versioned signed-binary target and one vendor-associated digest; larger download, as-yet-uninspected archive/notice surface [S1][S3]. |
| Fetch individual source headers from the **full pinned Git commit**, plus official signed DLLs from ZIP | Useful as a *cross-check* of header provenance. Not a single-artifact bootstrap; requires independent hashes/relationship proof and increases divergence risk [S2][S13–S16]. |
| GitHub automatically generated source ZIP/tarball alone | **Insufficient** for signed prebuilt DLL requirements; different artifact with separately generated bytes [S1][S18]. |
| Rebuild signed DLLs from source | Does not reproduce vendor-signed binary identity and changes the verification target; not recommended. |
| Project mirror, checked-in binaries, public CI download artifact | Operationally convenient but materially increases distribution, notice and provenance duties; defer pending ZIP-wide license audit. |
| User-provided SDK folder only | Minimizes downloader/legal surface but weakens standardized provenance and repeatability; can remain a supported explicit verification path. |

## 8. Delta against supplied project baseline

| Baseline assertion | Result as of cutoff |
|---|---|
| Release/tag `v2.3.0`, named prebuilt ZIP, full commit `60f4ea8…` | **Confirmed by primary release/commit evidence** [S1–S3]. |
| Archive SHA-256 `f90890…42273` | **Confirmed as GitHub-published digest** [S3]; **not independently recomputed** from bytes. |
| Per-file loader/upscaler SHA-256 values `e2d85…08aa` / `d0dccc…fb46` | **Still unresolved** outside the supplied project baseline; no official per-DLL digest or measured archive bytes established. |
| Named loader/upscaler DLLs covered by MIT-style exceptions | **Confirmed** at exact v2.3.0 paths [S4]. Also loader `.lib` and other named effect DLLs are excepted. |
| Entire SDK uniformly MIT | **Contradicted as a generalization** by default binary-only/restricted grant plus explicit exceptions and third-party notices [S4][S6]. This was **not** asserted by the supplied baseline. |
| Prebuilt package provides requisite headers/signed DLLs | **Upstream claim strengthened**, but **archive membership and sufficiency for this project's tests not measured** [S1][S11]. |
| Complete archive licensing audit completed | **Not completed**. |
| Windows Authenticode verification currently implemented | Remains **project-supplied context only**; implementation and historical outcomes not inspected. Windows verification policy is documented independently [S20][S21]. |
| Linux/Proton feasibility or GPU success | Source-level acquisition and PE signature tooling are feasible **in principle**; runtime/GPU behavior **unresolved** [S23][S25][S26]. |

## 9. Recommendations (not established project decisions)

1. Use the canonical **v2.3.0 release ZIP** as the acquisition candidate; carry its **published** SHA-256 as a reviewed expected input, documenting that a local archive recomputation remains an acceptance gate [S1][S3]. Keep the two project-supplied DLL hashes provisional pending direct comparison.
2. Implement the developer bootstrap, **if separately approved**, entirely outside the published `fsr-sdk`/`fsr-sdk-sys` build/load paths; require explicit invocation and explicit test DLL paths. Do not silently substitute DLLs or install global components.
3. Gate any public distribution of a cached archive or selected binaries on an archive member/notice audit, even though AMD's named DLLs and headers are MIT-excepted. Include full upstream license, third-party notice, and applicable third-party component license texts; keep MPL-2.0 project licensing separate [S4][S6].
4. Define layered verification: **identity/flat hashes**, **signature mathematics**, **trusted publisher and timestamp**, **revocation freshness**, then **runtime/provider/GPU dispatch**. Record and report these independently; do not collapse all into `verified`.
5. Keep Linux bootstrap and signature checks separate from Proton execution; document supported offline guarantees and explicitly gate Proton GPU claims on actual AMD hardware and a known Proton/vkd3d-proton configuration [S23][S25][S26].

## 10. Unresolved questions and follow-up experiments — **proposed, not performed**

| Experiment / evidence request | Prerequisites and exact procedure | Establishes / pass condition |
|---|---|---|
| **Archive provenance and completeness** | On a networked Windows **and/or** Linux workstation obtain only the official v2.3.0 prebuilt release ZIP; record redirect/asset metadata and final byte length; independently calculate entire-file SHA-256 and compare to `f90890…42273`. Inspect ZIP filenames, sizes, Unix modes, special entries and contained notice files **without executing** anything. | Reproduces or falsifies published archive digest; establishes real inventory, layout, extraction threat surface and whether named required files/notices exist. |
| **Exact per-file identity** | From that verified archive, extract in isolated staging; SHA-256 both DLLs; compare to baseline `e2d85…08aa` / `d0dccc…fb46`; record all approved header hashes and full include closure against actual C ABI checks. Compare header bytes to Git commit `60f4…a3` where the same path exists. | Establishes/refutes DLL hash baselines, true minimum allowlist, archive-to-source relationship and repeatable manifest. No values are asserted here. |
| **License audit** | Enumerate **all** archive entries; match each to v2.3.0 `docs/license.md` exception/default rules, embedded file license headers, third-party notices and separate component licenses. Examine any archive installer/EULA/readme/website terms, and document CI cache recipient/visibility/retention. Obtain legal review if distribution is contemplated. | Determines whether entire-archive caching/mirroring and any selected-file redistribution meet actual per-member terms; resolves currently unverified archive-specific restrictions. |
| **Windows Authenticode policy** | On supported Windows, with current roots and known revocation network conditions, run SignTool Authenticode verification for **both** archived DLLs; capture embedded signers, issuer, EKU, certificate chain, timestamp authority, signing time, revocation result and relevant tool policy. Independently compare identity to approved AMD corporate signer. Repeat offline to characterize uncertainty. | Tests authentic vendor-signature/trust/timestamp assertions. A nonzero failure or unknown signer must block strict trust acceptance. |
| **Linux verification parity** | On Linux, with documented `osslsigncode` version, CA/TSA trust bundle and CRL/network configuration, verify the same byte-hashed DLLs. Compare signer/timestamp/chain outcomes with Windows and record any divergence/indeterminate revocation. | Determines which trust assurance can be obtained on Linux and what **cannot** be claimed equivalent to Windows. |
| **Extraction and cache fault injection** | Using **synthetic** benign malformed ZIPs and directories (not AMD binaries), test traversal, duplicate/case-collision entries, escaping links/reparse points, oversized entries, truncation, corrupt hashes, pre-existing destination, concurrent bootstrap, interruption and cross-volume placement; test offline import. | Demonstrates fail-closed, no-overwrite, deterministic verified cache behavior and atomicity limits without running AMD code. |
| **Windows native ABI/provider execution** | On a clean supported Windows x86-64 D3D12 machine, stage **only** the verified loader/upscaler and include paths; run the existing project's ABI probe and explicit DLL-path provider/version/context/dispatch checks; log module resolution and GPU result. | Tests SDK/ABI compatibility, provider discovery, transitive dependencies and actual native GPU execution; not covered by source review. |
| **Proton GPU execution** | On Linux with supported AMD GPU/Vulkan driver and version-pinned Proton + vkd3d-proton, run the **Windows** x86-64 test executable with explicitly staged verified DLLs. Log loaded module paths, D3D12 device/feature level, provider enumeration, dispatch return values and output-image correctness against established controls; compare to native Windows where feasible. | Establishes or refutes actual Proton GPU dispatch for a specific environment. Acquisition/signature success is **not** a substitute. |

## 11. Primary sources and access dates

All sources below were accessed **2026-10-12**; AMD repository materials are pinned to **v2.3.0** (or exact commit) unless stated otherwise.

- **[S1] AMD/GPUOpen upstream release** — https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0
- **[S2] Linked full commit** — https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/commit/60f4ea81909200d8542eca14dccb2628b763a9a3
- **[S3] GitHub release's official expanded asset list and SHA-256** — https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/expanded_assets/v2.3.0
- **[S4] Versioned root AMD license** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/docs/license.md
- **[S5] Versioned SDK-subtree AMD license** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/license.md
- **[S6] Versioned third-party notice** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/3rdpartynotice.md
- **[S7] AMD FSR API and DLL structure** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/docs/getting-started/ffx-api.md
- **[S8] GitHub release-asset digest announcement (2025-06-03)** — https://github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets/
- **[S9] GitHub immutable releases** — https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases
- **[S10] GitHub release immutability configuration and prospective-only scope** — https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/establish-provenance-and-integrity/prevent-release-changes
- **[S11] Versioned SDK structure** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/docs/getting-started/sdk-structure.md
- **[S12] Versioned repository README** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/readme.md
- **[S13] Pinned `ffx_api.h`** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api.h
- **[S14] Pinned `ffx_api_types.h`** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_types.h
- **[S15] Pinned `dx12/ffx_api_dx12.h`** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/dx12/ffx_api_dx12.h
- **[S16] Versioned `ffx_upscale.h`** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/upscalers/include/ffx_upscale.h
- **[S17] Pinned optional AMD `ffx_api_loader.h`** — https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/60f4ea81909200d8542eca14dccb2628b763a9a3/Kits/FidelityFX/api/include/ffx_api_loader.h
- **[S18] GitHub release and auto-generated source assets** — https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases
- **[S19] GitHub Terms of Service, public content/API terms** — https://docs.github.com/en/site-policy/github-terms/github-terms-of-service
- **[S20] Microsoft SignTool verify options, `/pa`, `/all`, `/tw`** — https://learn.microsoft.com/en-us/windows/desktop/seccrypto/signtool
- **[S21] Microsoft WinVerifyTrust** — https://learn.microsoft.com/en-us/windows/win32/api/wintrust/nf-wintrust-winverifytrust
- **[S22] Microsoft Authenticode timestamping** — https://learn.microsoft.com/en-us/windows/win32/seccrypto/time-stamping-authenticode-signatures
- **[S23] `osslsigncode` upstream verification manual** — https://github.com/mtrojnar/osslsigncode/blob/master/osslsigncode.md (tool's `master`, **not** a pinned tool version; pin tool version in experiments)
- **[S24] Microsoft on Authenticode PE hash exclusions/security** — https://learn.microsoft.com/en-us/windows/win32/secbp/understanding-pe-signatures
- **[S25] Valve Proton upstream project** — https://github.com/ValveSoftware/Proton (general architecture, not a pinned runtime version)
- **[S26] vkd3d-proton upstream documentation** — https://github.com/HansKristian-Work/vkd3d-proton (general architecture, not a pinned runtime version)

**Evidence limit:** This record can establish the upstream release pointer, GitHub's *published* digest and the versioned text of license/API materials. It cannot establish archive byte identity, per-DLL digest/signature validity, complete archive redistribution clearance, Windows DLL dependency success, or Proton GPU functionality without the enumerated experiments.
