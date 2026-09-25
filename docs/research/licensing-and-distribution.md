# Licensing and distribution

- Updated: 2026-09-25.
- Evidence: [license and package audit](records/2026-09-21-src-license-and-package-audit.md)
  and [DX12 provider-DLL resolution](records/2026-09-21-src-dx12-provider-dll-resolution.md),
  [distribution and artifact provenance](records/2026-09-22-src-distribution-artifact-provenance.md),
  plus the [signed-runtime directory experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md).
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
The [current wrapper manifest](../../crates/fsr-sdk/Cargo.toml) supplies
`version = "0.1.0"`; a wrapper archive still has not been inspected. The
historical missing text is not by itself labeled a proven legal violation.

**Setup verification:** both updated Cargo file lists include LICENSE and
README.md. A newly generated raw-crate archive includes those files plus
LICENSE-AMD. Both crate LICENSE copies match the root byte-for-byte; source
markers, local documentation links and formatting were checked. This verifies
notice packaging, not release readiness or a newly built wrapper archive.

**Verified scope:** no SDK headers/DLLs or dependency sources are bundled in the
raw archive. Runtime DLLs are only staged locally. AMD's pinned license inventory
explicitly includes the two staged DLLs in its MIT exception list; do not infer
restricted terms for those named files solely from the document's default clause.
This is not clearance of a future bundle or a claim about arbitrary AMD binaries.

**Deployment evidence:** AMD's tagged sample and documentation use executable-adjacent
loader/effect DLLs, consistent with the recorded query experiment discussed in the
[API and safety synthesis](api-and-safety.md#dx12-provider-discovery-and-deployment).
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

**Licensing scope:** the report agrees with the existing audit that AMD's license
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

## Open decisions and checks

- Keep package-local MPL text synchronized with the root and preserve AMD
  attribution. The setup retains existing `MPL-2.0` metadata and documents the
  MIT notice separately; a future expression change requires an explicit decision.
  `OR` would incorrectly suggest an already-authorized package-wide license choice.
- Inspect a local wrapper archive, including its notices, before claiming
  package readiness. The path-dependency version is now present.
- For any future binary distribution, inspect the exact contents and applicable
  dependency notices, rather than treating local test staging as a release bundle.
- Copyrightability of individual ABI declarations, all-contributor title and
  patent clearance were not adjudicated. Retaining AMD's notice does not require
  assuming declarations are unprotected.
