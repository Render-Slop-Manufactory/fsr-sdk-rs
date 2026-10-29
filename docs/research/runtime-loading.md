# runtime loading

Updated: 2026-10-27 (U07 accepted/implemented; prior evidence: D042 loader-only candidate and persistent trust allocation integrated; deployment/native observations unchanged).

Extracted from the [API and safety synthesis](api-and-safety.md).
These findings retain their original evidence and limitations.

The [2026-10-09 maintenance recheck](../ROADMAP_HISTORY.md#october-9-native-diagnostic-refresh)
repeated trusted AMD loader acquisition, all four isolated provider layouts,
shared-runtime queries and D013 residency observations on its captured baseline.
Executable-adjacent providers again succeeded, private-directory-only/absent
providers returned code 4 and count 0, and provider residency survived final
loader release. D007 and the provider-termination uncertainty remain unchanged.

## Accepted loader-only acquisition boundary

The [D042 setup contract](../design/upscaler-setup.md) narrows loading to trusted
ABI/code, initialization/partial-failure/unload and family-wide coordination.
Effect assumptions are acquired separately through unsafe setup creation and
persist in derived owners. The [bounded spike](records/2026-10-26-exp-upscaler-setup-contract-spike.md)
observes zero FFX create/query/destroy calls on loader-only Drop and rejects safe
planning or use of the loaded candidate with the old safe constructor. Retained
runtime-state witnesses are distinct from observed final DLL unload, since the
fixture observer owns another library reference.

U07 was approved and implemented on 2026-10-27. Production now exposes
unsafe `Runtime::load` / `MovableRuntime::load` and separate unsafe setup
acquisition; old flat effect routes are removed. The
[implementation record](../release/2026-10-27-upscaler-setup-verification.md)
documents the production checks. No new DLL acquisition/discovery, provider isolation or termination
guarantee follows; the [API synthesis](api-and-safety.md#accepted-api-transition-and-setup-draft)
records review and implementation limits.

## Proton staging observations

The [Steam Deck proof](records/2026-10-13-exp-proton-linux-cross-build.md) and
[Steam Machine comparison](records/2026-10-13-exp-steam-machine-proton-fsr4-int8.md)
observe both pinned AMD DLLs loaded from the fresh executable stage. The runner
checks their hashes and resolves Wine paths through the isolated prefix's actual
DOS drive mappings; the Machine used `S:` rather than the Deck's `Z:` mapping.
This is observed staging success, not an assumed extension of Windows discovery
behaviour or a test of alternative Proton deployment layouts.

The Machine reports provider 4.1.1 with both Experimental and Proton 11, including
Experimental with automatic upgrading disabled. Optional Proton AMD components
are frozen as inputs when present. This does not attribute all internal provider
activity to a particular module. Linux signature-chain checks failed for lack of
a local issuer certificate; execution explicitly trusted the approved SDK hash
pins. The later bounded runtime and cleanup results below are separate from the
existing Windows provider-residency and termination questions.

## Prepared Proton runtime and launcher cleanup

The [V08 experiment](records/2026-10-14-exp-proton-offline-runtime.md) verifies
two fresh query/single-frame attempts on Deck with checksum-selected local
Proton/steamrt3 distributions and the installed launcher package. Complete
distribution inventories check bytes, modes, directory nodes and literal symlink
targets; only sniper's generated `var/` cache is excluded, and the prepared shim
has a separate hash. Missing/changed/extra distribution inputs reject execution.
Each launcher call proves network isolation before exec; update variables alone
are insufficient because the inspected umu still attempts an upstream lookup.
This local trust policy leaves external launcher dependencies and host libraries/
drivers outside the inventory. It establishes no Windows signature equivalence,
remote bootstrap or self-contained runtime distribution.

The [V09 experiment](records/2026-10-14-exp-proton-timeout-cleanup.md) retains
the failed group-only watchdog baseline: 12 Wine/container processes survived
across detached sessions. Shared prepared launch now uses bubblewrap PID
containment with `--die-with-parent`; offline calls also isolate networking.
Two fresh forced-expiry confirmations require all tracked owned processes to
exit, target heartbeats to stop and unrelated-prefix activity to continue.
Same-prefix public queries and GPU/fence/readback/teardown controls pass before
and afterwards, without prefix restoration. Missing isolation blocks launch.
This closes the bounded process-cleanup checkpoint, not AMD provider unloading,
GPU/kernel-hang recovery or arbitrary Proton-stack cleanup.

The separate [exit-247 diagnosis](records/2026-10-14-exp-proton-exit-status.md)
does not reproduce Deck's completed jitter launch failure in six exact-payload
replays. Wine error 1726 also appears in passing replays. A killed POSIX control
verifies SIGKILL-to-247 conversion as a possible mechanism; no original trace
identifies the failed layer or signal. V09's correction supplies no causal
explanation for that earlier non-timeout failure, and its verdict stays failed.

## DX12 provider discovery and deployment

The [provider-resolution investigation](records/2026-09-21-src-dx12-provider-dll-resolution.md)
examines SDK v2.3.0 at commit `60f4ea81909200d8542eca14dccb2628b763a9a3`.
The [signed-runtime experiment](records/2026-09-22-exp-dx12-provider-dll-resolution.md)
now resolves the bounded private-directory question for the tested v2.3.0 DLLs.
It agrees with the earlier deployment observation; the source record's unresolved
status reflected missing implementation evidence, not a contradictory result.

**Reported source facts:** creation enters `GetProvider`; a null-context version
query enters `GetProviderCount` or `GetProviderVersions`. Existing contexts retain
their selected provider. Selection uses effect/descriptor and provider-version
IDs and can consider an external/driver provider. `ffxLoadFunctions` only resolves
the five exports from an already-loaded module; it does not load effect DLLs.
Neither the inspected version-query descriptor nor `ffxOverrideVersion` supplies
a provider path. Version selection does not establish filesystem isolation.

**Upstream deployment guidance:** the tagged DX12 sample copies loader and effect
DLLs to its executable output directory; the denoiser guide explicitly requires
executable-adjacent placement. This agrees with the successful staged query.
It does not establish that colocating loader and provider in a private directory
away from the executable is sufficient.

**Unresolved implementation:** the public source omits the `amdinternal` layer
needed to identify the physical provider-DLL opening call, its path/flags and
eager versus lazy loading. The record's supplementary third-party binary report
is not tied to the pinned signed artefact and supplies no call arguments; its
imports/strings do not verify a v2.3.0 bare-name `LoadLibraryA` mechanism.

**Conditional Windows inference:** explicitly loading the API DLL by absolute
path does not itself add its directory to the search path for a later independent
bare-name load. If the private provider loader uses such a call, executable
placement and already-loaded same-basename modules can affect resolution.
Neither that call form nor deterministic coexistence of multiple FSR runtime
copies is established. No supported provider-directory control was identified
in the inspected public surface; this is not proof against private mechanisms.

**Verified runtime behaviour:** the signed v2.3.0 loader and upscaler DLLs were
tested on Windows build 26200.9457 with Rust x64/MSVC, using the existing null-
context/device count-only query. Each layout ran in a fresh process with an empty,
separate working directory and an absolute loader path. PATH was checked for
provider copies and inherited unchanged; the probe/runner made no DLL-directory
API calls. Both source DLLs had valid AMD signatures; fingerprints and logs are
preserved in the experiment record.

| Layout | Loader acquisition | Query return / count | Child exit |
|---|---|---|---:|
| A: loader and provider beside executable | succeeded | 0 (OK) / 2 | 0 |
| B: loader and provider together in private `runtime/` | succeeded | 4 (NO_PROVIDER) / 0 | 101 |
| C: private loader, provider beside executable | succeeded | 0 (OK) / 2 | 0 |
| D: private loader, no staged provider | succeeded | 4 (NO_PROVIDER) / 0 | 101 |

The provider was absent before and immediately after loader acquisition in every
case. After successful queries, module diagnostics identified exactly the staged
executable-adjacent provider. It remained absent in B and D. Negative exits were
the existing test assertion reacting to the observed return code, not loader
failures or native crashes. D and the successful module paths confirm dependence
on the staged provider under this environment.

**Verified conclusion:** colocation in the tested private directory alone was
insufficient; placing the provider beside the executable succeeded even with a
private loader. This establishes version enumeration for these binaries on this
configuration, not provider identity, GPU compatibility or dispatch.

**Inference and limits:** the results are consistent with executable-directory
resolution, but do not reveal AMD's exact loading API, argument, flags or full
search/timing algorithm. One run per layout on one Windows configuration does
not establish a contract for future releases or arbitrary process environments.
Same-basename coexistence, undocumented controls and transitive dependencies
remain unresolved; no clean-machine deployment was tested.

**Accepted policy and remaining research:**
[D007](../DECISIONS.md#d007--keep-runtime-acquisition-explicit-and-deploy-the-v230-dx12-upscaler-dll-beside-the-executable)
selects explicit, application-controlled acquisition and executable-adjacent
upscaler deployment for this bounded baseline. The loader may reside in a private
directory and is loaded through an explicit path. No additional experiment is
needed to answer private-directory colocation sufficiency for these inputs.
Alternate loading mechanisms, runtime coexistence, other systems and dependency
closure remain unresolved and would need their own evidence. The decision does
not establish AMD's internal search algorithm or authorize binary redistribution.

## D013: acquisition failures and release observations

The [2026-10-07 fixture experiment](records/2026-10-07-exp-d013-transfer-failure-cleanup.md)
adds six failure observations on named acquiring workers using the actual Windows
loader: missing-file Path/io/NotFound and each of the five missing required
exports as an exact Symbol error with its owned libloading source. No runtime
is published. Each missing-export fixture module is absent before loading and
after failure; temporary observation handles do not span cleanup. This is
acquisition-local release, not transfer of partially initialized runtime state
or evidence of AMD initialization rollback.

The [source intake](records/2026-10-07-src-d013-transfer-cleanup-contract.md)
reports Windows module-reference and loader-lock rules, with no successful
inspection of the requested pinned AMD implementation. **Inference:** observer
load references/pinning can alter residency; module snapshots and errors need
explicit semantics. Natural absence does not independently identify a detach
callback's thread or full provider/dependency termination. Process exit is not
ordinary release evidence. The earlier native loader disappearance and persistent
upscaler residency remain bounded observations described in
[runtime threading](runtime-threading.md#d013-fenced-native-transfers-and-final-release).

**Verified staged native residency:** the
[2026-10-08 follow-up](records/2026-10-08-exp-d013-native-transfer-cleanup.md)
observes all 30 control/transfer children after final contexts/runtime, frame
equipment, caller-device and helper references are naturally released, then
after ordinary worker exit. The loader and helper are absent; the upscaler
provider remains present. `GetModuleHandleW` supplies only an instantaneous
boolean and adds no module reference; no forced unload or reference-count
manipulation is used. All debug/device checks occur before device release.

**Inference and limit:** retaining the previously held application equipment/
device references is not necessary for provider persistence on these inputs.
Native/platform references and the provider's retention cause remain opaque;
actual COM device destruction, leak freedom, provider/dependency termination
and detach-thread attribution are not established. Process exit adds no normal
unload evidence. The [extended threading findings](runtime-threading.md#d013-extended-native-transfers-unwind-cleanup-and-staged-residency)
retain the tested synchronization premises; later D013 acceptance adds no unload evidence.

**Later review recommendation:** the
[soundness/architecture review imported on 2026-10-09](records/2026-10-09-src-d013-soundness-architecture-review.md)
distinguishes acquisition-local failure cleanup, final wrapper-owned loader
release and actual provider termination. Intake source comparison agrees that
failed symbol resolution releases its local library before publishing a runtime;
no partly constructed movable owner is transferred. Native rollback and
termination safety remain caller-established trust obligations.

The review considers additional provider-unload observation unnecessary before
implementation if the conditional unsafe contract covers any termination on its
actual executing thread. This is a recommendation, not an unload guarantee or
fresh execution result. The wrapper must not promise provider unload or its
thread, and the observed retention cause remains unresolved. No investigation
date or reviewed revision was supplied; the intake date is used. See
[the contract review](runtime-threading.md#d013-soundness-and-architecture-review)
for the reviewed requirements. [D013 was separately accepted on 2026-10-09](../adr/D013.md#acceptance-rationale-and-remaining-assumptions);
its [unsafe acquisition contract](../adr/D013.md#conditional-safety-contract)
adopts these assumptions without resolving provider termination. The subsequent
[public implementation check](../ROADMAP_HISTORY.md#movable-implementation-checkpoint)
verifies early runtime release, sibling retention and destination-thread final
runtime release; it establishes no provider termination guarantee.
