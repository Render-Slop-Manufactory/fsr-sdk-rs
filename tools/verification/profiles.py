# SPDX-License-Identifier: MPL-2.0
"""Explicit claim inventories. Effects retain their own workloads and oracles."""
import json
import os
import platform
import sys

from consumers import consumer_profile
from examples import example_cases
from execution import Case, Rejected
from native import abi_cases, build_case, child_case, experiment_cases, native_setup, regression_cases
from release import release_cases

EXPERIMENTS=('allocation','dimensions','gbv','input-lifetime','provider','recording','shared-runtime','sharpening','transfer','transfer-cleanup','temporal-calibration','temporal-diagnosis','temporal-quality','temporal-long','temporal-reset-explore','temporal-reset','temporal-jitter-explore','temporal-jitter')
ARTIFACT_NATIVE_PROFILES = {'native-upscaler', 'native-callbacks', 'native-motion-scale',
    'native-formats', 'native-depth-formats', 'native-bgra-srgb', 'native-variable-output',
    'native-resource-formats', 'experiment-public-reactive-mask',
    'experiment-selected-reactive-mask', 'proton', 'proton-offline', 'proton-timeout'}

PROFILES={
    'release-coverage':'Exact-package native regressions, callback/CB ABI and stable/MSRV full examples/utils workflows; no publication',
    'native-callbacks':'Both public owners: production callback ABI, allocator forwarding, diagnostics, fenced debug-view dispatch and cleanup',
    'native-resource-formats':'Remaining external format roles on both public owners; mapping, fenced calls, writes and cleanup only',
    'fast':'Formatting and ordinary default Rust tests',
    'ci':'Infrastructure, seven feature configurations, public consumers, examples, docs, Clippy and Windows ABI',
    'msrv':'All targets, seven feature configurations and public consumers on Rust 1.88.0',
    'abi':'Independent C++ assertions against four pinned SDK headers (Windows only)',
    'consumers':'Public API boundaries, traits, dependency graphs and complete public examples',
    'native-upscaler':'Pinned local DX12 deployment; public queries, lifecycle, dispatch, movable and sharpening integration with independent readback audit',
    'windows-upscaler-followup':'One Windows ABI/CI/MSRV/public-fixture and native upscaler/Configure batch; no experimental generator acceptance',
    'experiment-reactive-mask':'Isolated raw generator support/write/consumer/cleanup probe; rejection stays observational, no public generator API',
    'experiment-public-reactive-mask':'Both public generator families with mask-write/consumer/cleanup checks; acknowledgement alone cannot pass',
    'experiment-selected-reactive-mask':'Explicit pinned 3.1.5/2.3.4 providers, both public generator families and all flag combinations; no fallback',
    'experiment-performance':'Optimized current-feature CPU baseline, controlled counts and signed native dispatch; no GPU timing or parity verdict',
    'experiment-provider-performance':'Eight fixed C++ provider 4.1.1/default pairs: GPU and context costs on the current driver; no release gate',
    'experiment-provider-performance-build':'Compile provider A/B equipment only; no device creation or GPU child process',
    'experiment-provider-setup':'Optimized default/selected-provider startup-query timing; no cache, GPU dispatch or speed gate',
    'experiment-variable-output':'Raw variable-output geometry/allocation/motion/sharpening probe on three pinned providers; padding writes are observational',
    'experiment-motion-scale':'Raw RG16F/RG32F motion and signed/asymmetric scale equivalence with independent-axis wrong-sign controls on two pinned providers',
    'native-motion-scale':'Both public families: RG32F render/display, explicit mixed-sign scales, jitter-inclusive equivalence, UAV views and active output',
    'native-formats':'Both public families: typed colour/output, R16F depth and generator inputs versus independent C++ with exact input/readback audit',
    'native-depth-formats':'Both public families: D32/D32S8 typed and typeless depth, DSV production, shader/plane readback and independent C++ parity',
    'native-bgra-srgb':'Both public families: BGRA/sRGB input/output/generator views, UAV-capable inputs, resource integrity, fenced SDK calls and cleanup',
    'native-variable-output':'Both public active-output families on selected 4.1.1/3.1.5, maximum allocations and independent input/output audit',
    'experiment-cpu-comparison':'P03 independent C++ versus Rust wrapper/prepared raw CPU processes, balanced blocks and warm-state controls; no GPU timing or speed gate',
    'experiment-gpu-timing':'Bounded matched Rust/C++ 1080p same-queue timestamps, empty brackets and separate CPU recording; no speed gate',
    'experiment-active-timing':'Matched Rust/C++ active-render shrink/grow timing and stable controls, fixed allocations/output; no speed gate',
    'experiment-feature-comparison':'P02 matched Rust/C++ supplied-mask/linear-HDR CPU/GPU comparison with fixed/active-render controls; descriptive, no speed gate',
    'release-performance-reconfirm':'Exactly 256 fresh P08 paired blocks, prior results used only for planning, unchanged 5 us criterion',
    'experiment-performance-pilot':'Ten randomized matched native process blocks for CPU noise and confirmation selection; no speed verdict',
    'experiment-performance-confirm':'Frozen pilot-derived paired CPU overhead bounds, separate GPU observations, allocations, preparation and lifecycle; no release gate',
    'experiment-color-signal':'Declared linear/sRGB/PQ inputs, independent readback hypotheses and exposure/pre-exposure probes; no quality or universal semantic verdict',
    'experiment-color-signal-controls':'Static pre-exposure and per-frame-reset controls for the recorded colour-signal transition discrepancies; original hypotheses unchanged',
    'proton':'Experimental Linux MSVC cross-build, Proton launch, public queries and shared single-frame GPU proof; explicit diagnostic limits',
    'proton-offline':'Checksum-selected local Proton/steamrt3, missing-input controls and shared GPU proof with launcher networking disabled',
    'proton-timeout':'Readiness-synchronized forced Wine timeout, unrelated-prefix survival and subsequent query/GPU controls',
    **{'proton-temporal-'+kind:'Selected temporal '+kind+' regression through Proton; frozen numerical criteria and required Vulkan core/synchronization diagnostics'
       for kind in ('quality','long','reset','jitter')},
    'release-artifacts':'Exact archives, local registry, current/MSRV artefact checks and consumers; no GPU claim',
    'release':'Full source/MSRV/ABI, exact archive and native source/artefact verification; no publication',
    'release-dry-run':'Exact package verification followed by Cargo publication simulation; uploads are always aborted',
    **{'experiment-'+kind:'Bounded named '+kind+' protocol; observations remain distinct from acceptance' for kind in EXPERIMENTS},
}

PROFILE_ALIASES = {
    'experiment-colour-signal': 'experiment-color-signal',
    'experiment-colour-signal-controls': 'experiment-color-signal-controls',
}
PROFILES.update({alias: 'Compatibility alias for ' + name for alias, name in PROFILE_ALIASES.items()})


def equipment():
    def identify(r):
        prefix=['+'+r.options.toolchain] if r.options.toolchain else []
        rust,_,_=r.process(['rustc']+prefix+['-vV'])
        cargo,_,_=r.process(['cargo']+prefix+['-V'])
        r.note(rustc=rust,cargo=cargo,python=sys.version,host=platform.platform(),resume='disabled')
    return [Case('equipment.toolchain','Actual compiler, Cargo, Python and host identity',identify)]


def rust_cases(prefix='rust',quality=True,toolchain=None):
    cases=[]
    if quality:
        cases.append(Case(prefix+'.format','Repository Rust formatting',lambda r:r.process(r.cargo(['fmt','--all','--','--check'],toolchain))))
    for config,flags in (('default',[]),('movable',['--features','fsr-sdk/movable']),
                         ('no-default',['--no-default-features']),('no-default-movable',['--no-default-features','--features','fsr-sdk/movable']),
                         ('utils',['--features','fsr-sdk/utils']),('movable-utils',['--features','fsr-sdk/movable,fsr-sdk/utils']),
                         ('no-default-utils',['--no-default-features','--features','fsr-sdk/utils'])):
        for command in ('check','test','clippy','doc') if quality else ('check','test'):
            def action(r,command=command,flags=flags):
                args=[command,'--workspace']+flags
                if command in ('check','clippy'): args+=['--all-targets']
                if command=='doc': args+=['--no-deps']
                if command=='clippy': args+=['--','-D','warnings']
                r.process(r.cargo(args,toolchain))
            cases.append(Case(prefix+'.'+config+'.'+command,'Workspace '+command+' ('+config+')',action))
    return cases


def infrastructure():
    def tests(r):
        try:
            r.process([sys.executable,'-m','unittest','discover','-s','tools/verification/tests','-v'])
        except Rejected:
            # CI must show the assertion as well as retaining the full evidence.
            command = r.current['commands'][-1]
            output = (r.directory/command['stderr']).read_text(encoding='utf-8', errors='replace')
            print('Infrastructure test stderr tail (full log: '+command['stderr']+'):\n'+
                  r.sanitizer.text(output[-16000:]), file=sys.stderr, flush=True)
            raise

    def archive(r):
        report=r.directory/'cases'/r.current['id']/'archive-checks.json'
        args=[sys.executable,'tools/verification/archive/check.py','--report',report]
        if os.name=='nt': args+=['--require-powershell']
        try:
            r.process(args)
        finally:
            if report.is_file(): r.note(archive_checks=json.loads(report.read_text(encoding='utf-8')))
    return [Case('infrastructure.tests','Watchdog, negative-control, completeness and evidence-integrity tests',
                 tests),
            Case('archive.integrity','Frozen historical scripts, references and dependency index',
                 archive, expectation={'evaluation':'passed','contract':'Portable archive integrity and syntax; PowerShell syntax required on Windows',
                                       'powershell_syntax_required':os.name=='nt'})]


def compose(name, *, performance_plan=None, artifacts=False):
    if name not in PROFILES: raise ValueError('Unknown profile: '+name)
    name = PROFILE_ALIASES.get(name, name)
    if name=='fast': cases=rust_cases()[:1]+[case for case in rust_cases() if case.id=='rust.default.test']
    elif name=='ci':
        cases= (abi_cases() if os.name=='nt' else [])+infrastructure()+rust_cases()+consumer_profile()+example_cases()
    elif name=='msrv':
        cases=rust_cases('msrv',quality=False,toolchain='1.88.0')+consumer_profile('consumer-msrv',toolchain='1.88.0')+example_cases('consumer-msrv','1.88.0')
    elif name=='abi': cases=abi_cases()
    elif name=='consumers': cases=consumer_profile()+example_cases()
    elif name=='native-callbacks':
        from public_callbacks import cases as callback_cases
        cases=callback_cases()
    elif name=='native-upscaler': cases=native_setup()+regression_cases()
    elif name=='windows-upscaler-followup':
        source=(infrastructure()+rust_cases()+consumer_profile()+example_cases()+
                rust_cases('msrv',quality=False,toolchain='1.88.0')+
                consumer_profile('consumer-msrv',toolchain='1.88.0')+
                example_cases('consumer-msrv','1.88.0'))
        gate=Case('followup.source','Source/consumer/MSRV checks precede GPU execution',
            lambda r:r.note(source_checks_passed=True),tuple(case.id for case in source))
        native=regression_cases()
        native[0].requires+=('followup.source',)
        cases=native_setup()+source+[gate]+native
    elif name in ('experiment-reactive-mask', 'experiment-public-reactive-mask', 'experiment-selected-reactive-mask'):
        from reactive_mask import cases as reactive_cases
        cases=reactive_cases(public=name!='experiment-reactive-mask', selected=name=='experiment-selected-reactive-mask')
    elif name in ('experiment-variable-output','native-variable-output'):
        from variable_output import cases as output_cases
        cases=output_cases(public=name=='native-variable-output')
    elif name=='experiment-motion-scale':
        from motion_scale import cases as motion_cases
        cases=motion_cases()
    elif name=='native-motion-scale':
        from public_motion import cases as public_motion_cases
        cases=public_motion_cases()
    elif name=='native-resource-formats':
        from resource_formats import cases as resource_cases
        cases=resource_cases()
    elif name=='native-bgra-srgb':
        from bgra_srgb_public import cases as bgra_cases
        cases=bgra_cases()
    elif name in ('native-formats','native-depth-formats'):
        from typed_formats import cases as format_cases
        cases=format_cases(depth=name=='native-depth-formats')
    elif name=='experiment-provider-setup':
        namespace='provider-setup'
        build=build_case(namespace,release=True)
        cases=native_setup(optimized=True)+[build,child_case(namespace,'queries',
            'upscaler::dx12::tests::provider_setup::native_provider_setup_cost',
            'Provider setup timing: PASSED blocks=32 samples=64',(build.id,),scope='native-query')]
    elif name=='experiment-performance':
        from performance import cases as performance_cases
        cases=performance_cases()
    elif name in ('experiment-provider-performance', 'experiment-provider-performance-build'):
        from provider_performance import cases as provider_performance_cases
        cases=provider_performance_cases(build_only=name.endswith('-build'))
    elif name=='experiment-cpu-comparison':
        from cpu_comparison import cases as cpu_comparison_cases
        cases=cpu_comparison_cases()
    elif name=='experiment-gpu-timing':
        from gpu_timing import cases as gpu_timing_cases
        cases=gpu_timing_cases()
    elif name=='experiment-active-timing':
        from active_timing import cases as active_timing_cases
        cases=active_timing_cases()
    elif name=='experiment-feature-comparison':
        from active_timing import cases as active_timing_cases
        cases=active_timing_cases(include_features=True)
    elif name=='release-performance-reconfirm':
        from performance_characterization import cases as characterization_cases, RECONFIRM_PROTOCOL
        cases=characterization_cases(confirmation=True, plan_path=performance_plan, protocol_path=RECONFIRM_PROTOCOL)
    elif name in ('experiment-performance-pilot', 'experiment-performance-confirm'):
        from performance_characterization import cases as characterization_cases
        cases=characterization_cases(confirmation=name.endswith('-confirm'), plan_path=performance_plan)
    elif name=='experiment-color-signal':
        from color_signal import cases as color_signal_cases
        cases=color_signal_cases()
    elif name=='experiment-color-signal-controls':
        from color_signal import cases as color_signal_cases
        cases=color_signal_cases(controls=True)
    elif name=='proton':
        from proton import cases as proton_cases
        cases=proton_cases()
    elif name=='proton-offline':
        from proton import cases as proton_cases
        from proton_runtime import controls, integrity_case
        base=proton_cases(offline=True)
        cases=base[:1]+controls()+base[1:]+[integrity_case()]
    elif name=='proton-timeout':
        from proton_timeout import cases as timeout_cases
        cases=timeout_cases()
    elif name=='proton-temporal-quality':
        from temporal_quality import cases as quality_cases
        cases=quality_cases(dict(namespace='experiment.temporal-quality',protocol='temporal-quality-protocol.json',
                                script='temporal_quality.py',analysis='temporal-quality',backend='proton'))
    elif name=='proton-temporal-long':
        from temporal_long import cases as long_cases
        cases=long_cases(backend='proton')
    elif name=='proton-temporal-reset':
        from temporal_reset import cases as reset_cases
        cases=reset_cases(confirm=True,backend='proton')
    elif name=='proton-temporal-jitter':
        from temporal_jitter import cases as jitter_cases
        cases=jitter_cases(confirm=True,backend='proton')
    elif name=='release-coverage':
        from release import coverage_cases
        cases=coverage_cases()
    elif name=='release-artifacts': cases=release_cases()
    elif name=='release-dry-run':
        from release import dry_run_case
        cases=release_cases()+[dry_run_case()]
    elif name=='release':
        cases=native_setup()+infrastructure()+rust_cases()+rust_cases('msrv',False,'1.88.0')+consumer_profile()+example_cases()+release_cases()+regression_cases()+regression_cases('artifact',True)
        from public_callbacks import cases as callback_cases
        cases += callback_cases(namespace='artifact-callbacks', registry=True, include_setup=False)
        # Final integrity runs after GPU builds too.
        integrity=next(case for case in cases if case.id=='release.integrity')
        cases.remove(integrity)
        integrity.requires=tuple(case.id for case in cases)
        cases.append(integrity)
    elif name=='experiment-temporal-calibration':
        from temporal_calibration import cases as temporal_calibration_cases
        cases=temporal_calibration_cases()
    elif name=='experiment-temporal-diagnosis':
        from temporal_diagnosis import cases as temporal_diagnosis_cases
        cases=temporal_diagnosis_cases()
    elif name=='experiment-temporal-quality':
        from temporal_quality import cases as temporal_quality_cases
        cases=temporal_quality_cases()
    elif name=='experiment-temporal-long':
        from temporal_long import cases as temporal_long_cases
        cases=temporal_long_cases()
    elif name in ('experiment-temporal-reset-explore','experiment-temporal-reset'):
        from temporal_reset import cases as temporal_reset_cases
        cases=temporal_reset_cases(confirm=name=='experiment-temporal-reset')
    elif name in ('experiment-temporal-jitter-explore','experiment-temporal-jitter'):
        from temporal_jitter import cases as temporal_jitter_cases
        cases=temporal_jitter_cases(confirm=name=='experiment-temporal-jitter')
    else: cases=experiment_cases(name.removeprefix('experiment-'))
    if artifacts and name not in ('release', 'release-artifacts', 'release-dry-run', 'release-coverage'):
        if name not in ARTIFACT_NATIVE_PROFILES:
            raise ValueError('Archive mode is unsupported for profile: '+name)
        from release import bind_native_archives
        cases=bind_native_archives(cases)
    cases=equipment()+cases
    # Equipment is an explicit prerequisite for every profile action.
    for case in cases[1:]:
        case.requires=tuple(dict.fromkeys(('equipment.toolchain',)+case.requires))
    return cases
