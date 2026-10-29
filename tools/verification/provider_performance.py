# SPDX-License-Identifier: MPL-2.0
"""Small fixed provider A/B probe using the existing independent C++ consumer."""
import json
from pathlib import Path
import random
import time

from active_timing import parse
from execution import Case, Rejected
from native import environment, native_setup
from performance_characterization import estimate, process_summary, telemetry
from records import save, sha

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROTOCOL = HERE/'provider-performance-protocol.json'
REFERENCE = ROOT/'crates/fsr-sdk/tests/native/performance_reference.h'


def protocol():
    return json.loads(PROTOCOL.read_text(encoding='utf-8'))


def schedule(spec):
    rng = random.Random(spec['seed'])
    rows = []
    for _ in range(spec['blocks']//2):
        order = list(spec['arms'])
        rng.shuffle(order)
        rows.extend((order, order[::-1]))
    return rows


def probe_source(reference):
    """Change provider selection only; retain original timing and workload code."""
    selection = r'''
    const auto provider_arm=environment(L"FSR_SDK_PROVIDER_ARM");
    require(provider_arm==L"default" || provider_arm==L"explicit-4.1.1","provider arm");
    const bool explicit_provider=provider_arm==L"explicit-4.1.1";
    constexpr uint64_t old_provider=17700776140664213505ull;
    const uint64_t expected_provider=explicit_provider ? old_provider : 17700776142811697154ull;
    uint64_t probe_count=0;
    ffxQueryDescGetVersions available{};
    available.header.type=FFX_API_QUERY_DESC_TYPE_GET_VERSIONS;
    available.createDescType=FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE;
    available.device=device.Get(); available.outputCount=&probe_count;
    require(query(nullptr,&available.header)==0 && probe_count>0 && probe_count<=32,"provider count");
    const auto probe_capacity=probe_count;
    std::vector<uint64_t> probe_ids(static_cast<size_t>(probe_capacity));
    std::vector<const char*> probe_names(static_cast<size_t>(probe_capacity));
    available.versionIds=probe_ids.data(); available.versionNames=probe_names.data();
    require(query(nullptr,&available.header)==0 && probe_count>0 && probe_count<=probe_capacity,"provider fill");
    bool old_available=false;
    for (size_t i=0;i<static_cast<size_t>(probe_count);++i) {
        require(probe_names[i]!=nullptr,"provider name");
        std::fprintf(stderr,"PROVIDER_AVAILABLE %llu %s\n",static_cast<unsigned long long>(probe_ids[i]),probe_names[i]);
        old_available=old_available || probe_ids[i]==old_provider;
    }
    require(old_available,"explicit provider 4.1.1 unavailable; no fallback");
    ffxOverrideVersion provider_override{};
    provider_override.header.type=FFX_API_DESC_TYPE_OVERRIDE_VERSION;
    provider_override.versionId=old_provider;
    std::fprintf(stderr,"PROVIDER_ARM %ls\n",provider_arm.c_str());
'''
    replacements = (
        ('    const auto load_ticks=performance_clock()-load_start;',
         '    const auto load_ticks=performance_clock()-load_start;\n'+selection),
        ('        device_query.device=device.Get();',
         '        device_query.device=device.Get();\n'
         '        device_query.header.pNext=explicit_provider ? &provider_override.header : nullptr;'),
        ('    backend.device=device.Get();',
         '    backend.device=device.Get();\n'
         '    backend.header.pNext=explicit_provider ? &provider_override.header : nullptr;'),
        ('    std::fprintf(stderr,"CPU_PROVIDER %llu %s\\n",',
         '    require(provider.versionId==expected_provider,"actual provider mismatch; no fallback");\n'
         '    std::fprintf(stderr,"CPU_PROVIDER %llu %s\\n",'))
    for before, after in replacements:
        if reference.count(before) != 1:
            raise Rejected('Provider probe reference changed; review the source adaptation')
        reference = reference.replace(before, after)
    return '#define FSR_GPU_TIMING\nconstexpr bool gpu_timing = true;\n'+reference


def matched(previous, current):
    # Provider algorithms may differ in output bits. Both outputs are independently
    # validated, while descriptors, scene inputs, clocks and frame history match.
    for key in ('spec', 'adapter_driver', 'frequency_hz', 'gpu_frequency_hz',
                'query', 'smaller', 'jitter_bits', 'active_frames', 'input_samples'):
        if previous[key] != current[key]:
            raise Rejected('Provider A/B inputs/equipment differ: '+key)
    if [(r[0], r[5]) for r in previous['checkpoints']] != [(r[0], r[5]) for r in current['checkpoints']]:
        raise Rejected('Provider A/B full input fingerprints differ')


def summarize(processes, spec):
    expected = {f'{block}.{arm}' for block in range(1, spec['blocks']+1) for arm in spec['arms']}
    if set(processes) != expected:
        raise Rejected('Incomplete or extra provider A/B processes')
    metrics = {}
    for metric in ('cpu', 'gpu', 'first_dispatch_cpu', 'load', 'provider_query',
                   'first_create', 'warm_create', 'used_destroy', 'unused_destroy'):
        values = {}
        for arm in spec['arms']:
            values[arm] = []
            for block in range(1, spec['blocks']+1):
                summary = processes[f'{block}.{arm}']
                value = (summary['segments']['all'][metric]['mean_ns'] if metric in ('cpu', 'gpu') else
                         summary['first_dispatch_cpu_ns'] if metric == 'first_dispatch_cpu' else
                         summary['lifecycle'][metric]['mean_ns'])
                values[arm].append(value)
        metrics[metric] = {'process_means_ns': values, 'default_minus_explicit': estimate([
            default-old for default, old in zip(values['default'], values['explicit-4.1.1'], strict=True)])}
    return {'protocol': spec, 'metrics': metrics, 'speed_gate': None,
            'limit': spec['limits'], 'inference': spec['analysis']}


def cases(build_only=False):
    spec = protocol()

    def freeze(r):
        if sha(REFERENCE) != spec['base_reference_sha256']:
            raise Rejected('Frozen C++ reference identity changed')
        save(r.directory/'protocol.json', spec)
        save(r.directory/'schedule.json', schedule(spec))

    def build(r):
        stage, headers = r.state['native_stage'], r.state['headers']
        source = stage/'provider_reference.cpp'
        source.write_text(probe_source(REFERENCE.read_text(encoding='utf-8')), encoding='utf-8', newline='\n')
        executable = stage/'provider-reference.exe'
        r.process(['cl', '/nologo', '/std:c++17', '/O2', '/W4', '/WX', '/MT', '/EHsc',
            f'/I{REFERENCE.parent}', f'/I{headers/"api/include/dx12"}', f'/I{headers/"upscalers/include"}',
            source, f'/Fo{stage/"provider_reference.obj"}', f'/Fe{executable}',
            '/link', stage/'gpu_helper.lib', 'd3d12.lib'])
        r.state['provider-performance.cpp'] = executable
        r.note(probe_source_sha256=sha(source), executable_sha256=sha(executable),
               base_reference_sha256=sha(REFERENCE), compile_only=True)

    result = native_setup(optimized=True, gpu_timing=True)+[
        Case('provider-performance.protocol', 'Freeze eight-pair provider A/B protocol', freeze, ('native.helper',)),
        Case('provider-performance.build', 'Compile independent C++ provider selection probe', build,
             ('provider-performance.protocol',), scope='windows-compilation')]
    if build_only:
        return result
    previous = ('provider-performance.build',)

    def child(label, arm, requires, debug=False):
        case_id = 'provider-performance.'+label+'.'+arm

        def action(r):
            if not debug:
                r.state.setdefault('provider-performance.started', time.monotonic())
                if time.monotonic()-r.state['provider-performance.started'] >= spec['measurement_limit_seconds']:
                    raise Rejected('Provider A/B allowance exhausted; incomplete run cannot pass')
            output = r.directory/'readbacks'/case_id/'final.rgba16f'
            output.parent.mkdir()
            env = environment(r)
            env.update(FSR_SDK_PROVIDER_ARM=arm, FSR_SDK_CPU_ARM='cpp-minimal',
                FSR_SDK_PERFORMANCE_DEBUG=str(int(debug)), FSR_SDK_CPU_PREINSPECT='0',
                FSR_SDK_CPU_OUTPUT=output, FSR_SDK_ACTIVE_TIMING='max', FSR_SDK_FEATURE_TIMING=None,
                FSR_SDK_CHARACTERIZATION='timing' if debug else 'lifecycle')
            _, log, _ = r.process([r.state['provider-performance.cpp']], cwd=r.state['native_stage'], env=env,
                timeout=spec['child_timeout_seconds'], markers=('CPU_COMPARISON PASSED cpp-minimal',))
            measured = parse(log, arm='cpp-minimal', mode='max', debug=debug)
            if measured['provider']['id'] != spec['arms'][arm]:
                raise Rejected('Provider A/B identity mismatch; no fallback')
            measured.update(telemetry(log, 'timing' if debug else 'lifecycle', 'cpp-minimal',
                                      frames=4 if debug else spec['frames']))
            if not output.is_file() or output.stat().st_size != 1920*1080*8:
                raise Rejected('Invalid provider A/B readback payload')
            cohort = 'provider-performance.inputs.'+str(debug)
            if cohort in r.state:
                matched(r.state[cohort], measured)
            else:
                r.state[cohort] = measured
            measured['output_sha256'] = sha(output)
            r.state[case_id] = measured
            save(r.directory/'cases'/case_id/'samples.json', measured)
            r.note(provider=measured['provider'], adapter_driver=measured['adapter_driver'],
                   output_sha256=measured['output_sha256'], debug_control=debug)

        return Case(case_id, 'Matched C++ '+arm+(' debug control' if debug else ' GPU/context measurement'),
                    action, requires, 'provider-reference.exe', 'native-experiment')

    for arm in spec['arms']:
        case = child('control', arm, previous, debug=True)
        result.append(case)
        previous = (case.id,)
    primary = {}
    for block, order in enumerate(schedule(spec), 1):
        for arm in order:
            case = child(f'block{block}', arm, previous)
            result.append(case)
            primary[f'{block}.{arm}'] = case.id
            previous = (case.id,)

    def analyze(r):
        analysis = summarize({key: process_summary(r.state[name]) for key, name in primary.items()}, spec)
        analysis['elapsed_primary_seconds'] = time.monotonic()-r.state['provider-performance.started']
        analysis['schedule'] = schedule(spec)
        analysis['equipment'] = {key: r.state['provider-performance.inputs.False'][key]
                                 for key in ('adapter_driver', 'frequency_hz', 'gpu_frequency_hz', 'query', 'smaller')}
        (r.directory/'analysis').mkdir()
        save(r.directory/'analysis/provider-performance.json', analysis)
        r.note(paired_blocks=spec['blocks'], interpretation='Descriptive provider comparison; no release verdict')

    result.append(Case('provider-performance.analysis', 'Paired default-minus-4.1.1 GPU/context estimates',
                       analyze, tuple(primary.values()), scope='offline-analysis'))
    return result
