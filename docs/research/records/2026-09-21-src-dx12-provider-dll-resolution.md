# Source investigation: FidelityFX SDK 2.3.0 DX12 provider-DLL resolution

- Investigation date: 2026-09-21
- Topic and scope: FidelityFX SDK v2.3.0, Windows x64/MSVC, DX12, specifically how `amd_fidelityfx_loader_dx12.dll` discovers and loads effect-provider DLLs after the API loader DLL itself is loaded.
- Baseline and evidence cut-off: The investigation starts from the existing `fsr-sdk-rs` runtime observations stated in the project baseline. Those observations are not repeated as new source verification. FidelityFX evidence is pinned to SDK tag `v2.3.0`, commit `60f4ea81909200d8542eca14dccb2628b763a9a3`; current AMD `main` is not used as substitute evidence. Microsoft documentation was consulted for Windows loader semantics. Access date for all web sources: 2026-09-21.
- Method and exclusions: Inspected the tagged public API source, headers, documentation, sample build files, release/package inventory, and Microsoft loader documentation. A third-party static-analysis report was consulted only because the public AMD source omits the implementation needed to answer the exact `LoadLibrary` question; its provenance is explicitly bounded below. No runtime experiment, DLL disassembly/decompilation, wrapper design, code modification, Vulkan investigation, or general licensing audit was performed.

## Inputs

1. AMD FSR SDK v2.3.0 release. URL: [AMD FidelityFX-SDK releases — FSR SDK v2.3.0](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases). Version/tag: `v2.3.0`; commit: `60f4ea8` / `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21. The release identifies Upscaling 4.1.1 and reiterates the split-DLL model.

2. AMD `Kits/FidelityFX/api/internal/ffx_api.cpp`. URL: [ffx_api.cpp at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/api/internal/ffx_api.cpp). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21. This is the public dispatch layer for the five ABI functions.

3. AMD `Kits/FidelityFX/api/internal/ffx_provider.h`. URL: [ffx_provider.h at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/api/internal/ffx_provider.h). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21. This defines provider selection behavior exposed by the public source and references the omitted DX12 `amdinternal` provider header.

4. AMD `Kits/FidelityFX/api/include/ffx_api.h`. URL: [ffx_api.h at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/api/include/ffx_api.h). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21. Relevant public descriptors include provider-version enumeration and version override.

5. AMD `Kits/FidelityFX/api/include/ffx_api_loader.h`. URL: [ffx_api_loader.h at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/api/include/ffx_api_loader.h). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21.

6. AMD `Kits/FidelityFX/docs/getting-started/ffx-api.md`. URL: [Introduction to the AMD FSR API at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/docs/getting-started/ffx-api.md). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21.

7. AMD `Kits/FidelityFX/docs/techniques/denoising.md`. URL: [Denoising setup documentation at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Kits/FidelityFX/docs/techniques/denoising.md). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21.

8. AMD FSR DX12 sample project, `Samples/Upscalers/FidelityFX_FSR/dx12/FidelityFX_FSR_2022.vcxproj`. URL: [FSR DX12 sample project at v2.3.0 commit](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/Samples/Upscalers/FidelityFX_FSR/dx12/FidelityFX_FSR_2022.vcxproj). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21.

9. AMD tagged distribution/source inventory, `docs/license.md`. URL: [v2.3.0 distribution inventory](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/blob/60f4ea8/docs/license.md). Commit: `60f4ea81909200d8542eca14dccb2628b763a9a3`; access date: 2026-09-21. It lists the public API source and the signed FidelityFX DLLs; no `amdinternal` path occurs in the inventory.

10. Microsoft `LoadLibraryA` documentation. URL: [LoadLibraryA — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibrarya). Version: current Microsoft Win32 documentation as accessed 2026-09-21.

11. Microsoft DLL search-order documentation. URL: [Dynamic-link library search order — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order?trk=article-ssr-frontend-pulse_little-text-block). Version: current Microsoft Win32 documentation as accessed 2026-09-21.

12. Supplementary third-party static-analysis report for a file named `amd_fidelityfx_loader_dx12.dll`, SHA-256 `2f36843c3bb8c059621c10574e586a883ef337f2a549c67ecf3a82b3959ac238`. URL: [Hybrid Analysis report](https://hybrid-analysis.com/sample/2f36843c3bb8c059621c10574e586a883ef337f2a549c67ecf3a82b3959ac238/69fbb9ee24dc8b87d00653ae). SDK version/tag/commit: **Unresolved**; the report was not independently tied to AMD's v2.3.0 signed artefact. Access date: 2026-09-21. It is therefore supplementary evidence only.

## Findings

### Provider discovery and loading path

**Verified — public v2.3.0 source.** Provider selection is entered on demand from API operations rather than being performed by the Rust-side API-function loader. `ffxCreateContext` calls `GetProvider(desc->type, GetVersionOverride(desc), GetDevice(desc), ...)` before invoking that provider's `CreateContext`.

A null-context `ffxQuery` with `ffxQueryDescGetVersions` calls `GetProviderCount` for a count-only query and `GetProviderVersions` when actual IDs/names are requested. Other null-context queries call `GetProvider`. This directly establishes that the existing count-only upscaler-version experiment traverses the provider-enumeration path.

For an already-created context, the public layer stores the selected `ffxProvider*` in `InternalContextHeader`; subsequent context-bound operations retrieve that associated provider. Provider capability matching in the published base class is based on the effect portion of the descriptor type: `CanProvide` compares `descType & FFX_API_EFFECT_MASK` to the provider's `EffectId`. Providers also carry an ID and version name.

**Verified — public v2.3.0 source.** The DX12 provider-selection layer can additionally consider a driver-side/external provider. The public `ffx_provider.h` constructs an `ffxProviderExternal` and then compares it with providers supplied through a `std::span<ffxProvider* const>`. This is provider selection, not evidence about local effect-DLL path resolution.

**Upstream claim.** AMD describes `amd_fidelityfx_loader_dx12.dll` as a small DLL containing no effect code whose function is to manage loading effect-type DLLs. The documented v2.3.0 effect binaries include `amd_fidelityfx_upscaler_dx12.dll`, `amd_fidelityfx_framegeneration_dx12.dll`, `amd_fidelityfx_denoiser_dx12.dll`, and `amd_fidelityfx_radiancecache_dx12.dll`.

**Unresolved — exact physical DLL-loading implementation.** The public v2.3.0 source stops before the relevant implementation. `ffx_provider.h` declares the four-argument `GetProvider(...)` used by `ffx_api.cpp`, but its visible implementation overload requires an already-supplied provider span. For DX12 the same header includes `../../amdinternal/api/internal/dx12/ffx_provider_external.h`; that `amdinternal` source is not present in the tagged published source inventory.

Consequently, the public source establishes where provider enumeration/selection is requested, but not where the internal provider list is populated or which function physically opens the effect DLLs.

**Unresolved — eager versus lazy DLL opening.** `ffxCreateContext` and null-context `ffxQuery` demonstrably request providers at call time, but the omitted implementation could have loaded provider DLLs earlier or could load them from `GetProvider*`. The published source is insufficient to classify the physical `LoadLibrary` operation itself as lazy or eager.

**Verified / unresolved distinction on naming.** AMD publishes fixed effect-type DLL names, and provider selection after registration is descriptor/effect-ID and version-ID based. The exact mapping implementation from an effect descriptor to a DLL filename—hard-coded table, generated table, another private registry, etc.—is not published. No manifest-based mapping was identified in the inspected v2.3.0 public API source or documentation.

**Third-party corroboration, not v2.3.0 verification.** The supplementary static-analysis report contains the five FidelityFX basenames, including `amd_fidelityfx_upscaler_dx12.dll`, as strings in the analyzed loader and reports a direct import of `LoadLibraryA`, `GetModuleHandleA`, and `GetProcAddress`. This is consistent with a hard-coded-basename/`LoadLibraryA` implementation, but the report does not show the relevant call site or `LoadLibraryA` argument, and its analyzed file has not been tied by primary evidence to SDK v2.3.0. It therefore cannot establish the requested mechanism.

### Windows loading call and search semantics

**Verified — public v2.3.0 source.** `ffx_api_loader.h` is not the effect-provider loader. Its `ffxLoadFunctions(ffxFunctions*, void* module)` accepts an already-loaded module handle and calls `GetProcAddress` for `ffxCreateContext`, `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, and `ffxDispatch`. It performs no `LoadLibrary`, DLL-directory manipulation, or provider discovery.

**Unresolved — FidelityFX provider open call.** Primary v2.3.0 material inspected here does not establish whether the private provider loader calls `LoadLibraryA`, `LoadLibraryW`, `LoadLibraryExA/W`, or another wrapper; nor does it establish the flags or whether its argument is a basename, relative path, absolute path, loader-derived path, executable-derived path, or caller-provided path. The third-party report's `LoadLibraryA` import is insufficient to elevate any one of those possibilities to a v2.3.0 source fact.

The relevant Microsoft semantics are nevertheless determinate once an argument form is known. **Verified — Windows semantics:** for `LoadLibraryA`, a full path restricts the top-level module lookup to that path, whereas a relative path or module name without a path invokes Windows' DLL search strategy.

For an unpackaged application using the standard search order with Safe DLL Search Mode enabled, the relevant sequence includes the already-loaded-module list before filesystem application-directory lookup, followed later by the executable's directory, system directories, Windows directory, current directory, and `PATH`. The "application" directory here is the directory from which the executable was loaded, not the directory containing whichever DLL happens to call `LoadLibrary`.

Microsoft also documents altered search modes. `LoadLibraryEx` with `LOAD_WITH_ALTERED_SEARCH_PATH` and an absolute input path substitutes the loaded module's directory for the executable directory while resolving that load's dependency chain; `LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR` likewise concerns the dependencies of the DLL being loaded. `AddDllDirectory`, `SetDllDirectory`, and `SetDefaultDllDirectories` can modify the process search configuration. There is no primary v2.3.0 evidence found here that FidelityFX invokes any of these mechanisms.

### Relationship between loader and provider locations

For the specific question:

> If an application explicitly loads `C:\Game\runtimes\fsr\amd_fidelityfx_loader_dx12.dll`, does FidelityFX SDK v2.3.0 thereby cause `amd_fidelityfx_upscaler_dx12.dll` to be searched for in `C:\Game\runtimes\fsr\`?

**Unresolved — FidelityFX v2.3.0 implementation.** The public source does not expose the effect-DLL opening code and therefore cannot prove that the loader derives a sibling path, requests a bare basename, changes the process search path, or uses a `LoadLibraryEx` mode that would make its own directory relevant.

**Verified — Windows semantics.** Loading the main FidelityFX loader from an absolute path does not, by itself, permanently add that DLL's directory to the standard DLL search order for an independent later bare-name `LoadLibrary` call. Under the documented standard search order, the application directory is the executable's directory.

**Inference, conditional on a bare-name provider call.** If the omitted implementation is equivalent to `LoadLibraryA("amd_fidelityfx_upscaler_dx12.dll")` with the normal process search policy, merely having previously loaded the API loader from `C:\Game\runtimes\fsr\` would not make `C:\Game\runtimes\fsr\` a search location. A provider in that directory would need to be reachable through some independently applicable search location or explicit FidelityFX path construction/search-path manipulation.

Conversely, if the private FidelityFX implementation obtains the loader's own path and constructs a fully qualified sibling path, then the proposed private directory could work. No primary v2.3.0 source inspected here establishes such path derivation.

**Upstream claim / deployment evidence.** AMD's documented deployment examples avoid this ambiguity by placing the effect DLLs in the executable directory. The denoiser guide explicitly instructs applications to copy both `amd_fidelityfx_loader_dx12.dll` and `amd_fidelityfx_denoiser_dx12.dll` into the project's executable directory. The FSR DX12 sample likewise copies loader, upscaler, and frame-generation DLLs to `$(OutDir)`, i.e. the sample executable output directory.

Thus the proposed `Game\runtimes\fsr\` layout is **not established by the available source evidence**, even when loader and provider are colocated there.

### Explicit provider-path controls

**Verified — bounded public API inspection.** The public general API exposes no provider-DLL pathname or provider-directory field in `ffxQueryDescGetVersions`; it takes the effect creation descriptor type, optional DX12 device, output count, provider IDs, and provider names. `ffxOverrideVersion` contains only a provider version ID. A version override therefore chooses among provider versions known to the runtime; it is not a DLL path selector.

Likewise, `ffxLoadFunctions` accepts a handle to the already-loaded API entry DLL solely for resolving the five exported ABI functions. It provides no provider directory.

**Upstream claim / negative documentation result.** In the inspected tagged v2.3.0 API documentation, sample deployment, and public API headers, no supported FidelityFX environment variable, registry discovery mechanism, provider-directory descriptor, loader configuration API, or documented arbitrary provider search path was identified. AMD instead documents copying the required effect-type DLLs into the executable directory. This is a bounded finding about the inspected public v2.3.0 surface, not proof that no private/undocumented mechanism exists.

Generic Windows mechanisms such as `SetDllDirectory`, `AddDllDirectory`, `SetDefaultDllDirectories`, altered `LoadLibraryEx` flags, or `PATH` changes are Windows process-loader controls, not FidelityFX provider-path APIs. Microsoft documents these as changes to DLL search behavior.

### Multiple versions and basename collisions

**Verified — Windows behavior.** When `LoadLibraryA` is called without a path and more than one module with the same base name and extension is already loaded, Microsoft states that the handle to the module loaded first is returned. The standard DLL search process also checks the already-loaded-module list before searching the executable directory and later filesystem locations.

Accordingly, **Inference, conditional on FidelityFX using a bare provider basename:** a previously loaded `amd_fidelityfx_upscaler_dx12.dll` can determine which module a subsequent bare-name request resolves to, regardless of another copy existing beside a different FidelityFX loader. If no matching module is already loaded, another same-named copy in an earlier searched directory can likewise be selected according to the active Windows search policy.

A full path changes this property for the top-level file lookup: Microsoft documents that `LoadLibraryA` searches only the supplied full path. Whether FidelityFX takes advantage of that property is **Unresolved**.

**Unresolved — FidelityFX collision behavior.** Primary v2.3.0 source does not prove that effect providers are requested solely by basename, so the Windows basename-collision behavior cannot be asserted as the actual FidelityFX behavior. The third-party report's embedded provider basenames make this concern plausible but do not establish the call arguments for the pinned release.

The public FidelityFX "version override" mechanism does not solve filesystem/module identity collisions. It operates after provider discovery and selects a provider by `versionId`; the public selection code compares that ID against available provider objects.

Therefore source evidence is insufficient to claim that two independently packaged FidelityFX runtimes containing the same provider filename can coexist deterministically in one process.

### Deployment guidance and implications

**Upstream claim.** AMD states that splitting effects into separate DLLs allows applications to ship only the effect types they use. For v2.3.0 the documented DX12 runtime set includes the loader plus effect-specific upscaler, frame-generation, denoiser, and radiance-cache DLLs. The tagged distribution inventory contains those signed binaries.

**Verified — tagged sample artefact / Upstream deployment guidance.** AMD's concrete sample and denoiser instructions use executable-adjacent deployment, not a loader-adjacent private subdirectory: loader and applicable effect DLLs are copied to the application output/executable directory. This is the strongest primary evidence found for AMD's intended v2.3.0 filesystem layout.

**Inference.** Executable-adjacent deployment is compatible with ordinary Windows bare-name DLL resolution because the executable directory participates in the documented standard search order. That compatibility does not prove AMD's private loader actually uses a standard bare-name call.

**Unresolved — private-subdirectory sufficiency.** No primary evidence found establishes that merely colocating

```text
Game\runtimes\fsr\amd_fidelityfx_loader_dx12.dll
Game\runtimes\fsr\amd_fidelityfx_upscaler_dx12.dll
```

is sufficient when `Game.exe` resides in `Game\` and the process has not otherwise added `runtimes\fsr` to its DLL search path.

**Unresolved — provider transitive dependency closure.** The inspected primary material does not provide a PE-import/dependency inventory proving that every provider DLL is self-contained apart from standard OS/driver runtime dependencies. The sample project separately copies its vcpkg runtime dependencies into the output directory, but those are sample/application dependencies and cannot be attributed wholesale to FidelityFX providers. The supplementary loader analysis reports `VCRUNTIME140.dll` and Universal CRT imports for the analyzed loader file, but that file is not independently pinned here to v2.3.0 and is the loader rather than an effect provider.

No additional FidelityFX sibling DLL beyond the selected effect-type DLLs is established as required by this investigation. If later dependency inspection establishes additional redistributable non-system binaries, those binaries would need their own redistribution/licensing check; the existing AMD DLL licensing result should not be generalized to them.

## Baseline delta

**Confirmed.** The source/documentation confirms the architectural fact underlying the existing runtime observation: `amd_fidelityfx_loader_dx12.dll` contains no effect implementation according to AMD and delegates effects to effect-type DLLs. The public API source confirms that a count-only `ffxQueryDescGetVersions` query enters provider enumeration through `GetProviderCount`, so provider availability is relevant even when only the count is requested. AMD's v2.3.0 release also identifies the upscaler update as 4.1.1, consistent with the previously observed selected-provider version, although this source agreement is not an independent runtime verification.

**Contradicted.** None of the stated baseline runtime facts is contradicted. In particular, the baseline deliberately did not assert loader-relative provider resolution, so the absence of evidence for that behavior is not a contradiction.

**Refined.** Provider selection/enumeration can now be located precisely in the public API layer: `ffxCreateContext` → `GetProvider`; count-only/version enumeration `ffxQuery` → `GetProviderCount`/`GetProviderVersions`; other null-context queries → `GetProvider`; established contexts retain their selected provider. Provider capability selection is effect-descriptor based, with optional version-ID override. The physical DLL-loading step lies behind a public/private source boundary not included in the tagged published source.

The deployment baseline is also refined: executable-adjacent loader/provider placement is explicitly demonstrated by AMD's tagged v2.3.0 documentation and DX12 sample. Private `runtimes\fsr\` placement is not documented by the evidence inspected here.

**Still unresolved.** The exact Windows API used by the pinned loader to open effect DLLs; its exact filename/path argument and flags; whether effect DLLs are physically loaded lazily or eagerly; whether the loader derives paths from its own module location; whether an undocumented private provider-directory control exists; whether same-basename runtime copies collide in the actual signed FidelityFX implementation; complete provider transitive dependencies; and signed-binary behavior when loader and provider are placed outside the executable directory all remain unresolved.

Most importantly, source inspection does **not** establish that loading `C:\Game\runtimes\fsr\amd_fidelityfx_loader_dx12.dll` by absolute path causes `amd_fidelityfx_upscaler_dx12.dll` to be searched in the same directory. Windows provides no such general consequence for a later ordinary bare-name DLL load, and the private FidelityFX code needed to show a different rule is absent from the published v2.3.0 source.

## Limits and follow-up

The decisive limitation is the missing implementation between the public `GetProvider*` entry points and the construction/loading of the internal provider list. AMD's published v2.3.0 source therefore documents provider selection but not the provider-DLL pathname calculation or Windows loader call. The supplementary static-analysis report is insufficient to close that gap because its exact SDK provenance and `LoadLibraryA` call arguments are not established.

The smallest signed-runtime experiment needed to resolve the deployment question is a two-layout test in fresh processes using the actual signed v2.3.0 binaries. Put the test executable in directory `E`, the explicitly loaded API loader in distinct directory `L`, keep the current directory neutral, ensure neither test directory is introduced through `PATH` or process DLL-directory APIs, and ensure no provider of that basename is already loaded. In layout A, place the upscaler provider only in `L`; load `L\amd_fidelityfx_loader_dx12.dll` by absolute path and issue the same count-only upscaler version query. In layout B, keep the loader in `L` but place the provider only in `E` and repeat in a fresh process. Record the actual loaded provider path using normal module enumeration or Windows loader tracing.

Layout A directly tests whether loader/provider colocation in a private directory works without search-path mutation; layout B provides the executable-directory control documented by AMD. Observing the loaded module path prevents a successful query from being misinterpreted if an unintended copy was selected elsewhere. This experiment would establish behavior of the signed v2.3.0 binaries for the two deployment layouts, but still should not be described as proof of the private implementation's exact source-level algorithm.

A separate collision experiment—preloading a same-basename provider from one absolute location before invoking a loader from another—would only be needed if deterministic coexistence of multiple FidelityFX runtime copies later becomes a concrete deployment requirement.
