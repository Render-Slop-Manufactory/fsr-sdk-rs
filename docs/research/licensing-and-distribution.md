# Licensing and distribution

- Updated: 2026-10-29 (SDK 2.4 source/provenance limits integrated;
  existing v2.3.0 and bounded Proton findings retain their scope).
- Evidence: [licence and package audit](records/2026-09-21-src-license-and-package-audit.md)
  and [DX12 provider-DLL resolution](records/2026-09-21-src-dx12-provider-dll-resolution.md),
  [distribution and artefact provenance](records/2026-09-22-src-distribution-artifact-provenance.md),
  the [signed-runtime directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md),
  [M9 candidate verification](../release/2026-10-10-m9-verification.md),
  [SDK acquisition, integrity and platform review](records/2026-10-12-src-fidelityfx-sdk-acquisition.md),
  [Steam Deck Proton proof](records/2026-10-13-exp-proton-linux-cross-build.md),
  [Steam Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md),
  [prepared offline Proton runtime](records/2026-10-14-exp-proton-offline-runtime.md),
  [AMD Vulkan announcement research](records/2026-10-23-src-amd-fsr-vulkan-announcement.md),
  and [SDK 2.4 Vulkan/API compatibility research](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md).
- Scope: existing header-derived ABI and present Cargo packages against AMD SDK
  v2.3.0, acquisition provenance and bounded runtime deployment evidence.
  Research findings, not a new licensing or distribution decision.

## Current findings

**Verified:** the four headers underlying current production and experiment ABI
carry matching AMD MIT notices. `fsr-sdk-sys/LICENSE-AMD` retains their full text
and is present in the generated raw-crate archive. Original project work is
declared MPL-2.0. No inherent incompatibility was identified; preserved MIT
notices do not mean all Rust additions or complete mixed files are MIT-licensed.

**Audit baseline:** neither Cargo file list included the root MPL text or a
README. The raw archive confirmed its absence. The subsequent repository setup
added package-local copies of LICENSE, package READMEs, source SPDX markers and
contribution rules preserving AMD notices. At audit time, wrapper packaging
failed because the `fsr-sdk-sys` path dependency lacked a version requirement.
The 0.2.0 wrapper release supplied `version = "0.1.1"`; the
[current wrapper manifest](../../crates/fsr-sdk/Cargo.toml) pairs wrapper 0.3.0
with raw 0.1.2. The later
[final artefact recheck](../ROADMAP_HISTORY.md#010--published-2026-09-28)
inspected both archives, including notices and normalized manifests; the
[publication check](../ROADMAP_HISTORY.md#010--published-2026-09-28) records the
published packages. The
historical missing text is not by itself labelled a proven legal violation.

**Historical setup verification:** the initial setup checks found LICENSE and
README.md in both Cargo file lists and inspected a raw-crate archive containing
those files plus LICENSE-AMD. Both crate LICENSE copies matched the root;
source markers, links and formatting were checked. Those setup checks alone
did not establish wrapper archive contents or release readiness. The later
artefact recheck and publication check linked above cover both `0.1.0` packages;
the later M9 record below separately identifies and verifies the local candidate.

**Verified local M9 candidate:** the
[2026-10-10 verification](../release/2026-10-10-m9-verification.md) identifies
Cargo-generated `fsr-sdk 0.2.0` and `fsr-sdk-sys 0.1.1` archives with full
[file inventories and hashes](../release/2026-10-10-m9-verification.md#observations-and-limits).
Both contain READMEs and MPL texts byte-identical to the root licence;
sys retains LICENSE-AMD.
Neither packages DLLs, SDK trees, executables or compiled artefacts. The sys patch
identifies already accepted loader/dependency maintenance, without a new ABI or
SDK pin. D013's public movable family is excluded from this candidate.

Unmodified extracted packages, registry-only downstream consumers, examples,
feature/private-field controls, stable/MSRV and Rustdoc checks passed. Public native
integration also passed through the extracted wrapper. The archives and every
extracted file remained unchanged, and the publication dry-run regenerated
byte-identical archives before aborting both uploads. The source projection and
inventories identify the exact tested bytes; changed or regenerated candidates
need separate verification. This is an unpublished local candidate, not registry
server acceptance, hosted-docs verification or a binary redistribution review.

**Combined release scope:** the maintainer superseded the movable exclusion on
2026-10-10. The [complete release verification](../ROADMAP_HISTORY.md#release-qualification)
checks the actual `0.2.0` / `0.1.1` archives with movable retained, including
licence/notice copies, registry-only consumers and native execution. The earlier
projection above remains historical evidence; no new licensing interpretation
or native binary redistribution is introduced.

**Verified scope:** no SDK headers/DLLs or dependency sources are bundled in the
raw archive. Runtime DLLs are only staged locally. AMD's pinned licence inventory
explicitly includes the two staged DLLs in its MIT exception list; do not infer
restricted terms for those named files solely from the document's default clause.
This is not clearance of a future bundle or a claim about arbitrary AMD binaries.

**Deployment evidence:** AMD's tagged sample and documentation use executable-adjacent
loader/effect DLLs, consistent with the recorded query experiment discussed in the
[API and safety synthesis](runtime-loading.md#dx12-provider-discovery-and-deployment).
**Verified runtime refinement:** the signed v2.3.0 directory experiment found that
colocating loader and provider in a private directory was insufficient on the
tested Windows configuration: acquisition succeeded but enumeration returned
NO_PROVIDER/count 0. Moving the provider beside the executable succeeded with
OK/count 2 while the loader stayed private; module paths and a failed no-provider
control confirmed the staged provider's role. This closes that bounded deployment
question without selecting an acquisition/distribution architecture.

**Unresolved:** neither investigation establishes a complete transitive dependency
inventory or any additional required FidelityFX sibling DLL. The runtime test
does not certify clean-machine deployment or change the licensing findings.
Any later identified non-system dependencies need their own terms checked;
the existing named-AMD-DLL licensing finding cannot be generalized to them.

## Acquisition, binary identity and distribution scope

The [distribution/provenance report](records/2026-09-22-src-distribution-artifact-provenance.md)
adds source and release-metadata findings for v2.3.0. It did not download or
inspect the release archive's bytes.

**Reported upstream requirements:** production FSR API use requires AMD's supplied
signed DLLs. The documented upscaling set is the loader plus upscaler DLL;
other effect DLLs are not required merely because they are adjacent in the SDK.
The import library is optional link-time material, and loader PDBs are debugging
material. No separate backend DLL or shader/data package was identified, but
that is not a verified clean-machine dependency closure. Public FSR2/FSR3 source
is not established as sufficient to reproduce the signed FSR 4.1.1 implementation.

**Acquisition identity:** both the repository's `signedbin` tree and official
`FidelityFX-Samples-v2.3.0-prebuilt.zip` contain binary material; a repository
archive is not source-only. The report records GitHub's release-asset SHA-256:

`f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273`

This is reported metadata, not a locally recomputed archive hash. Displayed
sizes (122 MB and approximately 125 MiB) are not an exact byte-length check.
The versioned asset URL was not established as immutable. A full Git commit
pins repository contents, while generated archive compression can change the
outer bytes. GPUOpen's current-download endpoint is discovery material, not a
fixed v2.3.0 identity. Hash-validated caches and explicit local inputs are feasible
options, not an accepted acquisition mechanism or permission to add downloads.

**Licensing scope:** the report agrees with the existing audit that AMD's licence
has restrictive default terms and an explicit MIT-style exception list naming
these loader/upscaler binaries and the loader import library. Its use of the
phrase “source files” for a list that includes binaries is a drafting ambiguity
retained in the record. The report found no channel-specific cache/mirror rule
for those named files; that does not extend their terms to the whole mixed SDK
archive. An additional archive-only EULA was not ruled out because the ZIP was
not opened. Applicable notices must follow the actual future bundle.

**Dependencies:** the SDK third-party manifest lists build/sample components,
not a demonstrated runtime dependency set. In particular, VS2022 build guidance
does not prove a dynamically linked VC runtime requirement. The report cites
AMD's FSR 4.1.1 SM6.6/Agility guidance and SDK manifest version 1.616.1; the
application's actual Agility/OS deployment remains unverified. Any redistributed
Microsoft components need their own package terms and distributable-file review.

**Reconciliation with local evidence:** the source report's unknown per-DLL
hashes and signer are limits of that investigation, not gaps in all project
observations. The [directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md)
and [input-lifetime experiment](records/2026-09-22-exp-create-error-input-lifetime.md)
already record hashes and valid AMD Authenticode results for their local loader
and upscaler files. They do not establish byte equality with this release ZIP,
full certificate-chain/timestamp details, or a complete PE/dynamic dependency
inventory. Likewise, provider ID `0xf5a5ca1e01001001` and name `4.1.1` were
observed together locally; the report's absent official mapping does not undo
that observation. It does prevent treating the ID as a stable cross-machine
constant. AMD requires queried IDs; driver/external routing means shipped DLL
identity alone does not determine the executed provider on every machine.

**Remaining verification:** exact archive contents and package-only terms,
archive-to-local DLL equality, PE imports/delay imports and metadata, dynamic
module paths and clean-machine deployment remain open. Additional experiments
or distribution choices are not completed by importing this report.

## Future SDK and Vulkan artefacts

The [2026-10-23 announcement record](records/2026-10-23-src-amd-fsr-vulkan-announcement.md)
identified a prospective delivery path. The
[2026-10-29 source report](records/2026-10-29-src-sdk-240-vulkan-api-compatibility.md)
now identifies the official SDK 2.4.0 release/tag and Vulkan headers, superseding
the earlier absence of a public release at the October 24 cut-off. Selected
header notices were inspected, but the downloadable release and individual DLL
bytes, signatures, complete package terms and dependency closure were not.
The [platform assessment](platform-and-backends.md#sdk-24-and-release-sequencing)
separates conditional SDK-only adoption from usable Vulkan implementation.

The inspected AMD MIT-style header notices do not establish redistribution terms
for every signed effect DLL, model, driver component or asset. SDK 2.4 also adds
the loader's directory to effect lookup after already-loaded modules and the
executable directory, according to upstream documentation. Review actual module
provenance and dependencies for DX12 adoption as well as Vulkan: successfully
loading five exports does not identify the effect binary eventually selected.

**Unresolved:** reports of a Windows driver component named `amdvkffx64.dll`
establish neither a public SDK application interface nor its redistribution
terms. The existing v2.3.0 findings concern specifically named SDK files;
their exception list cannot be extended to a new driver DLL, Vulkan provider
or Linux runtime by analogy. Existing Proton evidence still concerns the pinned
Windows/DX12 subset.

Before adopting or distributing candidate SDK material, R06 needs exact release
assets and measured archive/member identities, the actual package licence and
third-party notices, named binary exceptions, signature/trust results and the
runtime dependency inventory for each selected OS/backend. Review new Windows
and any native Linux artefacts separately. SDK acquisition remains explicit and
separate from Cargo builds under the existing policy; importing this research
adds no downloads, bundled binaries or distribution approval.

## Explicit developer SDK acquisition

The [2026-10-12 source record](records/2026-10-12-src-fidelityfx-sdk-acquisition.md)
assesses an opt-in maintainer bootstrap outside the published crates. It agrees
with the earlier v2.3.0 provenance findings and adds extraction, offline trust
and Linux/Proton considerations. Its project baseline was supplied context;
it inspected neither this repository nor release ZIP bytes and performed no
download, signature verification or GPU execution. Intake checks current
manifests, explicit loading and local SDK guidance; selected upstream licence
paths and the release-linked commit were spot-checked on 2026-10-12.

**Corroborated identity:** the record identifies source commit
`60f4ea81909200d8542eca14dccb2628b763a9a3`, the official
`FidelityFX-Samples-v2.3.0-prebuilt.zip` and the same GitHub-published archive
SHA-256 recorded above. This is publication metadata, not a measured ZIP hash.
Release immutability, real member/layout inventory, embedded notices, archive-to-
source header equality and archive-to-local DLL equality remain unverified.
The record's provisional per-DLL hashes are supplied baseline values; earlier
local experiments measured them and observed valid AMD signatures. That local
evidence remains valid within its scope and does not establish ZIP provenance.

**Licensing agreement:** the pinned
[AMD licence](https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/v2.3.0/docs/license.md)
lists the selected API/upscaler headers and loader/upscaler DLLs under its
MIT-style exception, alongside restrictive default terms and separate third-party
notices. The exception does not make the entire mixed archive uniformly MIT.
Actual future bundles or shared caches need their own member/notice assessment;
the source report supplies no completed archive-level clearance.

**Recommended scope, not accepted architecture:** use explicit invocation,
reviewed archive/per-file pins, allowlisted bounded extraction, a fresh staging
directory, verified offline reuse and complete relevant notices. The logical
upscaler subset is four C headers plus the loader/upscaler DLLs; actual archive
inventory and test include closure must establish the extraction list. Import
libraries, C helpers, other effects and sample media are conditional inputs.
Keep acquisition separate from ordinary Cargo builds and caller-selected runtime
paths, consistent with [D007](../adr/D007.md) and [local SDK inputs](../../external/README.md).

**Trust boundaries:** full-file hashes establish equality to approved expected
bytes; signature verification, trusted AMD signer/timestamp, revocation status
and runtime/provider/GPU execution are separate observations. Linux signature
tooling is policy-distinct from Windows Authenticode, and offline checks may
leave revocation unknown. The record proposes layered reporting rather than
one undifferentiated verification result. At source-review time, strict trust
policy, extraction/cache replacement behaviour and acquisition were still
proposals; D017 and the later observations below cover the selected mechanism.
Acquiring Windows DLL bytes on Linux establishes no native Linux or Proton
support; see [platform assessment](platform-and-backends.md#sdk-acquisition-and-proton-boundary).

## Proton acquisition and trust observations

The [Steam Deck proof](records/2026-10-13-exp-proton-linux-cross-build.md) and
[Steam Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
subsequently exercised [D017's accepted maintainer bootstrap](../adr/D017.md)
on Linux and verified its eight pinned SDK files. This supersedes the source
review's acquisition-feasibility uncertainty for that selected subset. Microsoft
cross-build inputs, AMD SDK materials and installed Proton components remain
local development inputs under their own terms; no binaries were redistributed.

Both records report Linux signature-chain verification failure for missing local
issuer certificates and explicitly trust the approved DLL hashes. They establish
neither a Windows Authenticode-policy pass nor equivalent Linux trust. umu
acquired steamrt3 on Deck and steamrt4 on Machine; recorded versions and selected
component hashes alone did not establish a fully pinned, bounded or offline
runtime bootstrap. The later local-import policy below resolves a bounded part
of that question; future redistribution audits remain separate work.

## Prepared local Proton runtime

The [2026-10-14 V08 record](records/2026-10-14-exp-proton-offline-runtime.md)
adds explicit bounded import of existing local Proton 10.0-4b and steamrt3/sniper,
plus inventory of the installed umu 1.4.4 package and a copied prepared shim.
Import preserves notices, modes and symlinks, rejects existing/overlapping output
paths and changed input identities, and has 4 GB/100,000-entry/300-second limits.
Partial imports lack a completed manifest. No remote download or third-party
bundle is added to the repository or published crates.

Complete distribution inventories and an explicitly supplied manifest checksum
select trusted local bytes, not independently authenticated upstream releases.
Two fresh Deck attempts pass all 12 cases with enforced launcher network
isolation and unchanged distributions; missing-input controls reject without
downloads. This completes V08 for its local-import policy. It does not audit
every licence in a future redistributable bundle, pin external launcher/OS/driver
dependencies, establish Windows Authenticode equivalence, relocate the installed
launcher, or bootstrap a fresh machine offline. Local preparation and successful
execution are evidence of that bounded workflow rather than redistribution
clearance. The [platform synthesis](platform-and-backends.md#prepared-offline-runtime-and-timeout-containment)
assesses the separate timeout-cleanup checkpoint.

## Open decisions and checks

- Keep package-local MPL text synchronized with the root and preserve AMD
  attribution. The setup retains existing `MPL-2.0` metadata and documents the
  MIT notice separately; a future expression change requires an explicit decision.
  `OR` would incorrectly suggest an already-authorized package-wide licence choice.
- Repeat the archive and notice inspection for future releases. Both published
  `0.1.0` archives and the exact local M9 candidate archives were inspected in the
  records linked above; the combined `0.2.0` release inspection is separate from
  that historical candidate projection.
- For any future binary distribution, inspect the exact contents and applicable
  dependency notices, rather than treating local test staging as a release bundle.
- The source review's bootstrap recommendations subsequently led to D017's
  accepted mechanism; the Proton records above verify Linux reuse of its selected
  subset. Audit wider SDK/dependency closure and any future bundles separately;
  V08's bounded local-import/offline checkpoint is verified. Remote runtime
  acquisition, relocatable deployment, wider host/launcher dependency closure
  and Windows/Linux trust equivalence remain open.
- Copyrightability of individual ABI declarations, all-contributor title and
  patent clearance were not adjudicated. Retaining AMD's notice does not require
  assuming declarations are unprotected.
