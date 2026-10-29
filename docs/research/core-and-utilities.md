# Core and optional utilities

- Updated: 2026-10-29 (selected utility workflows and feature-on/off archive checks integrated; D033/D036 boundaries unchanged).
- Scope: responsibility boundaries in development `fsr-sdk 0.3.0` /
  `fsr-sdk-sys 0.1.2` under [amended D031](../adr/D031.md), pinned AMD SDK v2.3.0;
  the original studies are source/architecture evidence and used the then-selected
  sys `0.2.0` development baseline. The later D042 candidate adds bounded
  stable/MSRV compilation and controlled execution, not native utility evidence.
- Included records: [core/utils boundary audit](records/2026-10-19-src-core-utils-boundary-audit.md)
  and [composability/future-extension study](records/2026-10-19-src-core-utility-composability-and-future-extension.md).

## Current findings

**Selected utility package coverage, 2026-10-29:** the
[package checkpoint](../release/2026-10-29-package-coverage-verification.md) and
[consolidated review](../release/2026-10-29-candidate-verification.md) now compile
the packaged `utils` compositions for both owners on Stable and Rust 1.88.0,
including all eight named workflow calls, four Windows example feature
combinations and the seven-configuration extracted-package matrix. These are
compilation/API-isolation results, not renderer execution of the examples or a
provider capability guarantee. Final publication bytes remain separate; no new
utility policy, algorithm or default activation is selected.

**D042 composition selection:** the [API-review synthesis](api-and-safety.md#accepted-api-transition-and-setup-draft)
reconciles both 2026-10-26 audits with accepted [D042](../adr/D042.md) and the
[setup/composition spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md).
Setup, portable frame parameters, named resources and checked requests remain
core representations of native operations. They introduce no policy provenance,
generic policy hook or renderer service. The candidate compiles direct and
prepared frame inputs without a backend dependency for portable data; production
U08 now implements portable frame/generator parameters and named DX12 bindings. U07 setup was approved and implemented
on 2026-10-27, including resolver output `ProviderSelection`. See the
[implementation checks](../release/2026-10-27-upscaler-setup-verification.md).
No additional utility algorithm or quality work is selected. On 2026-10-27 the
maintainer selected D036 for release with 0.3.0, retaining opt-in activation.

**Later concrete selection:** the maintainer separately accepted and authorised
[D036](../adr/D036.md) on 2026-10-22. The off-by-default portable provider-name
resolver is now implemented using the public metadata getters and existing
setup provider request. Its [implementation verification](../release/2026-10-22-provider-utils-verification.md)
is separate from the two source studies. The intake observations and unselected
examples below describe their 2026-10-19 baseline; that D036 update imported no
new research or quality-policy work. The subsequent release selection is a
maintainer scope decision, not new research evidence.

The audit classified APIs by the source of their semantics. Its 2026-10-19
inventory found no production implementation of an independent project-authored utility policy
requiring extraction. Current native operations, checked representations and
shared ownership mechanics fit the core boundary; no structural migration is
justified by the audit.

| Responsibility | Current interpretation |
|---|---|
| `fsr-sdk-sys` | Curated native ABI declarations and symbol loading, preserving upstream contracts. |
| `fsr-sdk` core | Faithful Rust access to native capabilities, including validation, representation/unit mappings, ownership, error handling and explicit unsafe obligations. |
| Opt-in `fsr_sdk::utils` | Independently specified calculations or strategies; D036 now implements the first concrete utility, a provider-name resolver. Sequences and resolution controllers remain unselected examples. |
| Renderer/integration layer | Renderer resources, camera projection, render graphs, transitions, submission/fences, temporal histories and application presentation orchestration. |

**Verified at source intake (2026-10-19):** the manifest and exports then
contained no `utils` feature/module. The current
[manifest](../../crates/fsr-sdk/Cargo.toml) and [exports](../../crates/fsr-sdk/src/lib.rs)
include the separately accepted D036 resolver.
The [native utility queries](../../crates/fsr-sdk/src/query/native/utilities.rs)
invoke AMD descriptors and check provider results; they are core despite their
private filename. [Descriptor motion scaling](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/descriptor.rs)
maps declared UV/pixel units without converting texture contents.
[`validate_upscale_inputs`](../../crates/fsr-sdk/src/upscaler/dx12/dispatch.rs)
reuses compulsory recording inspection, while
[`ExposureInput::Automatic`](../../crates/fsr-sdk/src/upscaler/dx12/dispatch/frame.rs)
and [`CameraParameters::new_infinite`](../../crates/fsr-sdk/src/upscaler/common/camera.rs)
express native input profiles. These remain core, as do checked tuning and
provider selection; convenience alone does not create utility policy.

**Accepted decision, separately from evidence:** the maintainer's integration
request accepts [D033](../adr/D033.md) with the audit's two clarifications:
representational convenience stays core, and generic utilities must not absorb
renderer ownership or existing unsafe obligations. Core defaults describe a
supported profile rather than an endorsed quality preset. Native SDK object
ownership and future native presentation access remain core; managing the
application's renderer/presentation loop remains integration work.

Future utilities should initially live behind an off-by-default feature/module
in the existing crate, introduced only with a concrete reviewed need. They use
public core APIs, preserve core defaults and direct controls under
[D016](../adr/D016.md), and keep pure calculations independent of Windows/DX12
under [D031](../adr/D031.md). A feature name must not bypass validation or change
dispatch safety. The studies selected no new crate, feature, API, migration or milestone;
D036's later implementation is separately authorised above.
The [through-1.0.0 quality-policy deferral](../POST_1_0.md) remains binding.

## Composition and future extension

The 2026-10-19 follow-up study finds no required core refactor: a utility can
return an existing checked value and the caller can supply it through the ordinary API.
For example, a selection can return `Sharpening::Native` containing a checked
`NativeSharpness`, then feed `set_sharpening`; no policy-specific enum variant
or competing upscaler API is needed. This is source-level composability, not
compiled or calibrated utility evidence.

That finding concerns expressibility through the then-current public values.
The later API audits address repeated application glue and discoverability;
D042's chosen portable aggregates and effective settings improve that caller
experience without contradicting the earlier core/utility boundary. The table
below still describes the implemented baseline and its operation phases. The
spike supports the new assembly shape in isolation, without implementing
utility policies or the complete production transition.

| Phase | Composition boundary |
|---|---|
| Creation | Construct/modify `UpscalerOptions` before creating a context. Maxima are constructor inputs with getters, not mutable setters; changing options does not reconfigure a live context. |
| Per-frame | Return checked frame values and apply them through existing constructors/setters. Multiple values may form a utility-owned plan; core need not depend on its type or provenance. |
| Native queries | Keep calls explicit, coordinated and subject to eligibility/error rules. Caller-supplied owned results may suit a future policy, but provider identity is not a capability guarantee. |
| Live Configure | `request_tuning` retains GPU-idleness, acknowledgement-only success and terminal native-error rules. Combining requests with frame settings creates no transaction or rollback. |
| GPU operations | Parameter planning does not allocate textures, record/submit work or establish completion. Native reactive-mask generation is a separate core operation with its own provider and recording contract. |

The D033 clarification prefers composition through existing checked inputs and
requires a demonstrated public-API gap before policy-specific core extensions.
It prescribes no universal trait, builder, registry or second effect API.
Future-effect overlap remains an inference: use effect-specific contracts and
revisit shared abstractions only when implemented consumers demonstrate a need.
Feature-local policy errors and owned query snapshots are possible future designs,
not selected APIs; native error details must remain available.

M14 variable output and U05 signed/asymmetric motion scales remain native core
capabilities. Computing a size or coefficient does not implement them or resolve
provider colour/exposure limits. The earlier utility studies selected no release
milestone; D042 separately selects the U07–U12 API transition.

## Provenance and limitations

The two 2026-10-19 records use the same reported baseline ZIP, with D033 supplied separately
as a proposal patch/archive. Intake preserves both records unchanged; original
archive/proposal hashes and external citations were not independently revalidated.
The follow-up's hash for the earlier audit matches this checkout's record.
Local source checks corroborate composition signatures; they also narrow the
study's creation-profile wording: options have no setters for fixed maxima.
Both records' descriptions of D033 as proposed reflect their older input,
not its current accepted status. The follow-up adds a composition clarification
under the maintainer's synthesis/documentation request, not implementation approval.

There is no substantive conflict with D016 or D015's common/backend split.
Neither of those earlier records supplies compilation, native/GPU, quality or performance evidence
for hypothetical utilities; synthesis runs none. Test/research algorithms are
equipment, not accepted production utility APIs. Shared private mechanics and
method repetition across ownership families do not alone justify new public APIs.

## Open questions

Beyond the separately accepted D036 resolver, a future candidate needs
explicit semantics, domains, compatibility limits and proportionate verification;
before approving an API, a small downstream caller should demonstrate direct
and utility-selected inputs through the same public core API and identify any
actual missing capability. Future feature checks should cover portable utility-only
and applicable DX12/movable combinations without changing core behaviour.
These checks belong to a later selected utility, not this synthesis.
Image-quality or performance claims need separate evidence. Unresolved provider
colour/exposure behaviour does not establish a correct conversion algorithm.
These are conditions for later selection, not active release work.
