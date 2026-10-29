# SPDX-License-Identifier: MPL-2.0
"""Randomised native characterisation, frozen selection and block-level inference."""
import argparse
import contextlib
import json
import math
from pathlib import Path
import random
import re
import statistics
import sys
import time

from active_timing import compatible, parse as parse_active, protocol as active_protocol
from execution import Case, Rejected
from feature_comparison import parse as parse_feature, summarize as summarize_features, selected_protocol
from gpu_timing import SELECTOR
from native import build_case, environment, native_setup
from records import fresh, now, save, sha, source_identity, verify_artifacts

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROTOCOL = HERE / 'performance-characterization-protocol.json'
RECONFIRM_PROTOCOL = HERE / 'performance-reconfirmation-protocol.json'
ARMS = ('cpp-minimal', 'rust-wrapper')


def protocol(path=None):
    return json.loads(Path(path or PROTOCOL).read_text(encoding='utf-8'))


def _beta_fraction(a, b, x):
    # Continued fraction for the regularised incomplete beta; no new dependency.
    c, d = 1., 1.-(a+b)*x/(a+1.)
    d = 1./max(d, 1e-300)
    result = d
    for m in range(1, 301):
        for numerator in (m*(b-m)*x/((a+2*m-1)*(a+2*m)),
                          -(a+m)*(a+b+m)*x/((a+2*m)*(a+2*m+1))):
            d = 1.+numerator*d
            c = 1.+numerator/c
            if abs(d) < 1e-300: d = 1e-300
            if abs(c) < 1e-300: c = 1e-300
            d = 1./d
            delta = d*c
            result *= delta
        if abs(delta-1.) < 3e-14:
            return result
    raise ValueError('Incomplete beta did not converge')


def _beta(a, b, x):
    if x <= 0: return 0.
    if x >= 1: return 1.
    front = math.exp(math.lgamma(a+b)-math.lgamma(a)-math.lgamma(b)+a*math.log(x)+b*math.log1p(-x))
    if x < (a+1)/(a+b+2):
        return front*_beta_fraction(a, b, x)/a
    return 1.-front*_beta_fraction(b, a, 1.-x)/b


def t_quantile(probability, degrees):
    if not .5 < probability < 1 or degrees < 1:
        raise ValueError('Invalid Student t quantile arguments')
    def cdf(value):
        return 1.-.5*_beta(degrees/2, .5, degrees/(degrees+value*value))
    lo, hi = 0., 1.
    while cdf(hi) < probability: hi *= 2
    for _ in range(80):
        middle = (lo+hi)/2
        if cdf(middle) < probability: lo = middle
        else: hi = middle
    return (lo+hi)/2


def estimate(values, budget=None):
    if len(values) < 2 or any(not math.isfinite(x) for x in values):
        raise Rejected('Inference requires finite independent block differences')
    mean = statistics.mean(values)
    sd = statistics.stdev(values)
    se = sd/math.sqrt(len(values))
    margin = t_quantile(.95, len(values)-1)*se
    two_sided = t_quantile(.975, len(values)-1)*se
    lower, upper = mean-margin, mean+margin
    verdict = None if budget is None else ('within-budget' if upper < budget else
        'exceeds-budget' if lower > budget else 'inconclusive')
    return {'blocks': len(values), 'mean_difference_ns': mean, 'block_sd_ns': sd,
            'lower_one_sided_95_ns': lower, 'upper_one_sided_95_ns': upper,
            'two_sided_95_ns': [mean-two_sided, mean+two_sided], 'budget_ns': budget,
            'verdict': verdict, 'block_differences_ns': values}


def schedule(blocks, seed, spec=None):
    rng = random.Random(seed)
    workloads = (spec or protocol())['workloads']
    # Each two-block group uses opposite arm positions for every workload.
    orders = []
    for block in range(blocks):
        if block % 2 == 0:
            positions = {mode: rng.randrange(2) for mode in workloads}
        modes = rng.sample(workloads, len(workloads))
        orders.append([(mode, arm) for mode in modes for arm in
                       (ARMS if positions[mode] == block % 2 else ARMS[::-1])])
    return orders


def telemetry(log, mode, arm, frames=160):
    modes = re.findall(r'^CHAR_MODE (\w+)$', log, re.M)
    if modes != [mode]: raise Rejected('Missing/duplicate/wrong characterisation mode')
    result = {}
    expected_labels = {'CHAR_MODE'}
    if mode == 'preparation':
        expected_labels.add('CHAR_PREPARATION')
        rows = re.findall(r'^CHAR_PREPARATION (\d+) (\d+)$', log, re.M)
        if len(rows) != frames or {int(row[0]) for row in rows} != set(range(frames)):
            raise Rejected('Incomplete preparation measurements')
        result['preparation_ticks'] = [int(x[1]) for x in sorted(rows, key=lambda x: int(x[0]))]
    elif mode == 'allocations':
        expected_labels.add('CHAR_ALLOCATIONS')
        rows = re.findall(r'^CHAR_ALLOCATIONS (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)$', log, re.M)
        if len(rows) != frames or {int(row[0]) for row in rows} != set(range(frames)):
            raise Rejected('Incomplete allocation measurements')
        result['allocations'] = [list(map(int, x[1:])) for x in sorted(rows, key=lambda x: int(x[0]))]
    elif mode == 'lifecycle':
        expected_labels.add('CHAR_LIFECYCLE')
        rows = re.findall(r'^CHAR_LIFECYCLE (\w+) (\d+) (\d+)$', log, re.M)
        expected = {(name, 0) for name in ('load', 'provider_query', 'first_create', 'used_destroy')}
        expected |= {(name, i) for name in ('warm_create', 'unused_destroy') for i in range(8)}
        if len(rows) != len(expected) or {(name, int(i)) for name, i, _ in rows} != expected:
            raise Rejected('Incomplete lifecycle measurements')
        result['lifecycle_ticks'] = {name: [int(value) for kind, _, value in
            sorted(rows, key=lambda x: int(x[1])) if kind == name] for name, _ in expected}
    elif mode not in ('timing', 'raw'):
        raise Rejected('Unknown characterisation mode')
    lines = [line for line in log.splitlines() if line.startswith('CHAR_')]
    if any(line.split()[0] not in expected_labels for line in lines):
        raise Rejected('Unexpected instrumentation in timing process')
    expected_count = 1 + (frames if mode in ('preparation', 'allocations') else 20 if mode == 'lifecycle' else 0)
    if len(lines) != expected_count or (mode == 'raw') != (arm == 'rust-raw'):
        raise Rejected('Malformed/extra characterisation telemetry or wrong arm')
    return result


def describe(values):
    ordered = sorted(values)
    return {'mean_ns': statistics.mean(values), 'median_ns': statistics.median(values),
            'p95_ns': ordered[math.ceil(.95*len(ordered))-1], 'samples': len(values)}


def process_summary(child):
    frequency = child['frequency_hz']
    cpu = [tick*1e9/frequency for tick in child['ticks']['frame']]
    gpu = [(row[3]-row[2])*1e9/child['gpu_frequency_hz'] for row in child['gpu_ticks']]
    segments = {'all': [32, 160], **active_protocol()['segments']}
    result = {'segments': {segment: {'cpu': describe(cpu[start:end]), 'gpu': describe(gpu[start:end])}
                          for segment, (start, end) in segments.items()},
              'first_dispatch_cpu_ns': cpu[0], 'first_dispatch_gpu_ns': gpu[0],
              'warmup_diagnostic_cpu_ns': [statistics.mean(cpu[a:b]) for a, b in ((0,16),(16,32),(32,48),(144,160))]}
    for label, key in (('preparation', 'preparation_ticks'), ('lifecycle', 'lifecycle_ticks')):
        if key in child:
            values = child[key]
            result[label] = ({name: describe([x*1e9/frequency for x in ticks]) for name, ticks in values.items()}
                             if isinstance(values, dict) else describe([x*1e9/frequency for x in values[32:]]))
    if 'allocations' in child:
        result['allocations'] = {'measured_dispatches': 128,
            'totals_alloc_dealloc_zeroed_realloc_bytes': [sum(row[i] for row in child['allocations'][32:]) for i in range(5)],
            'first_shrink': child['allocations'][64], 'first_grow': child['allocations'][112]}
    return result


def summarize(processes, blocks, phase, spec=None):
    spec = spec or protocol()
    # Reuse the independent cross-workload input/history/mask/HDR audits too.
    base = selected_protocol() | {'blocks': blocks}
    summarize_features({f'active.block{b}.{mode}.{arm}': processes[f'characterization.block{b}.{mode}.{arm}']
        for b in range(1, blocks+1) for mode in base['workloads'] for arm in ARMS}, base)
    summaries = {name: process_summary(child) for name, child in processes.items()}
    findings = {}
    for mode in spec['workloads']:
        findings[mode] = {}
        for segment in ('all', *active_protocol()['segments']):
            findings[mode][segment] = {}
            for clock in ('cpu', 'gpu'):
                differences = [summaries[f'characterization.block{b}.{mode}.rust-wrapper']['segments'][segment][clock]['mean_ns']-
                    summaries[f'characterization.block{b}.{mode}.cpp-minimal']['segments'][segment][clock]['mean_ns']
                    for b in range(1, blocks+1)]
                budget = spec['cpu_budget_ns'] if phase == 'confirmation' and segment == 'all' and clock == 'cpu' else None
                findings[mode][segment][clock] = estimate(differences, budget)
    verdicts = [findings[mode]['all']['cpu']['verdict'] for mode in findings]
    overall = None if phase == 'pilot' else ('within-budget' if all(x == 'within-budget' for x in verdicts) else
        'exceeds-budget' if 'exceeds-budget' in verdicts else 'inconclusive')
    return {'phase': phase, 'protocol': spec, 'findings': findings, 'process_summaries': summaries,
            'overall_cpu_verdict': overall, 'inference_limit': 'Pointwise model-based bounds across process blocks; no universal, every-frame or GPU-equivalence claim.'}


def selection(pilot, output):
    pilot = Path(pilot)
    verify_artifacts(pilot)
    run = json.loads((pilot/'run.json').read_text())
    if run['profile'] != 'experiment-performance-pilot' or run['status'] != 'passed' or not run['inputs_unchanged']:
        raise Rejected('Selection requires a complete passed pilot')
    result = json.loads((pilot/'analysis/performance-characterization.json').read_text())
    spec = protocol()
    if result['phase'] != 'pilot' or result['protocol'] != spec:
        raise Rejected('Pilot protocol does not match current protocol')
    inputs = json.loads((pilot/'inputs.json').read_text())
    local = source_identity(ROOT)
    actual = {entry['path']: entry['sha256'] for entry in inputs['files'] if not entry['path'].startswith('sdk/')}
    if actual != {entry['path']: entry['sha256'] for entry in local['files']}:
        raise Rejected('Source changed since pilot; run a fresh pilot')
    seconds = result['measurement_seconds']/spec['pilot_blocks']
    capacity = min(spec['maximum_confirmation_blocks'], int(spec['confirmation_measurement_limit_seconds'] /
        (seconds*spec['runtime_planning_multiplier'])))
    capacity -= capacity % 2
    if capacity < spec['minimum_confirmation_blocks']:
        raise Rejected('Minimum confirmation cannot fit the measurement allowance')
    sd = max(result['findings'][mode]['all']['cpu']['block_sd_ns'] for mode in spec['workloads'])
    count = capacity
    for n in range(spec['minimum_confirmation_blocks'], capacity+1, 2):
        if t_quantile(.95, n-1)*sd/math.sqrt(n) <= spec['planning_half_width_ns']:
            count = n
            break
    plan = {'schema_version': 1, 'protocol_sha256': sha(PROTOCOL), 'producer_identity': local['identity'],
            'pilot_input_identity': inputs['identity'], 'pilot_analysis_sha256': sha(pilot/'analysis/performance-characterization.json'),
            'blocks': count, 'seed': spec['confirmation_seed'], 'worst_pilot_sd_ns': sd,
            'planned_margin_ns': t_quantile(.95, count-1)*sd/math.sqrt(count),
            'planned_precision_feasible': t_quantile(.95, count-1)*sd/math.sqrt(count) <= spec['planning_half_width_ns'],
            'predicted_measurement_seconds': count*seconds*spec['runtime_planning_multiplier'],
            'measurement_limit_seconds': spec['confirmation_measurement_limit_seconds'],
            'pilot_equipment': result['equipment'], 'schedule': schedule(count, spec['confirmation_seed']),
            'analysis': spec['analysis'], 'cpu_budget_ns': spec['cpu_budget_ns']}
    output = Path(output)
    # Exclusive creation freezes this selection; later choices need a new path.
    with output.open('x', encoding='utf-8', newline='\n') as stream:
        stream.write(json.dumps(plan, indent=2)+'\n')
    return plan


def load_selection(path, protocol_path=None):
    if not path: raise ValueError('Confirmation requires --performance-plan with a frozen selection')
    plan = json.loads(Path(path).read_text(encoding='utf-8'))
    spec = protocol(protocol_path)
    if plan['protocol_sha256'] != sha(Path(protocol_path or PROTOCOL)) or plan['producer_identity'] != source_identity(ROOT)['identity']:
        raise Rejected('Frozen selection source/protocol identity mismatch')
    n = plan['blocks']
    if type(n) is not int or n % 2 or not spec['minimum_confirmation_blocks'] <= n <= spec['maximum_confirmation_blocks']:
        raise Rejected('Invalid frozen confirmation count')
    if (plan['seed'] != spec['confirmation_seed'] or plan['cpu_budget_ns'] != spec['cpu_budget_ns'] or plan['measurement_limit_seconds'] != spec['confirmation_measurement_limit_seconds']
        or plan['schedule'] != [[list(pair) for pair in row] for row in schedule(n, spec['confirmation_seed'], spec)]
        or plan['analysis'] != spec['analysis']):
        raise Rejected('Frozen confirmation boundaries/order/analysis mismatch')
    return plan


def cases(confirmation=False, plan_path=None, protocol_path=None):
    spec = protocol(protocol_path)
    plan = load_selection(plan_path, protocol_path) if confirmation else None
    blocks = plan['blocks'] if confirmation else spec['pilot_blocks']
    orders = plan['schedule'] if confirmation else schedule(blocks, spec['pilot_seed'], spec)
    build = build_case('characterization', release=True)

    def cpp_build(r):
        stage, headers = r.state['native_stage'], r.state['headers']
        executable = stage/'characterization-reference.exe'
        r.process(['cl', '/nologo', '/std:c++17', '/O2', '/W4', '/WX', '/MT', '/EHsc',
            f'/I{headers/"api/include/dx12"}', f'/I{headers/"upscalers/include"}',
            r.repository/'crates/fsr-sdk/tests/native/gpu_reference.cpp', f'/Fo{stage/"characterization_reference.obj"}',
            f'/Fe{executable}', '/link', stage/'gpu_helper.lib', 'd3d12.lib'])
        r.state['characterization.cpp'] = executable
        r.note(executable_sha256=sha(executable), independent_native_consumer=True)

    result = native_setup(optimized=True, gpu_timing=True)+[build,
        Case('characterization.build.cpp', 'Independent optimized C++ consumer', cpp_build, ('native.helper',), scope='windows-compilation')]
    previous = (build.id, 'characterization.build.cpp')
    primary = []
    supplemental = []

    def child(name, workload, arm, requires, mode='timing', debug=False):
        case_id = 'characterization.'+name
        feature = workload if workload in ('masks', 'hdr') else None

        def action(r):
            if name.startswith('block'):
                r.state.setdefault('characterization.started', time.monotonic())
                if confirmation and time.monotonic()-r.state['characterization.started'] >= spec['confirmation_measurement_limit_seconds']:
                    raise Rejected('Confirmation measurement allowance exhausted; incomplete result cannot pass')
            output = r.directory/'readbacks'/case_id/'final.rgba16f'
            output.parent.mkdir()
            env = environment(r)
            env.update(FSR_SDK_CPU_ARM=arm, FSR_SDK_PERFORMANCE_DEBUG=str(int(debug)), FSR_SDK_CPU_PREINSPECT='0',
                FSR_SDK_CPU_OUTPUT=output, FSR_SDK_ACTIVE_TIMING='max' if feature else workload,
                FSR_SDK_CHARACTERIZATION=mode, FSR_SDK_FEATURE_TIMING=feature)
            args = [r.state['characterization.cpp']] if arm == 'cpp-minimal' else [r.state[build.id],
                '--ignored', '--exact', SELECTOR, '--nocapture', '--test-threads=1']
            _, log, command = r.process(args, cwd=r.state['native_stage'], env=env,
                markers=('CPU_COMPARISON PASSED '+arm,), timeout=240)
            measured = (parse_feature(log, arm=arm, feature=feature, debug=debug) if feature else
                        parse_active(log, arm=arm, mode=workload, debug=debug))
            measured.update(telemetry(log, mode, arm, frames=4 if debug else 160))
            if not output.is_file() or output.stat().st_size != 1920*1080*8:
                raise Rejected('Invalid final output payload')
            measured['output_sha256'] = sha(output)
            key = 'characterization.reference.'+workload+('.debug' if debug else '.timed')
            if key in r.state:
                compatible(r.state[key], measured)
                if feature and measured['feature_checkpoints'] != r.state[key]['feature_checkpoints']:
                    raise Rejected('Feature output/range mismatch')
            else: r.state[key] = measured
            equipment = {key: measured[key] for key in ('provider','adapter_driver','frequency_hz','gpu_frequency_hz','query','smaller')}
            if 'characterization.equipment' in r.state and r.state['characterization.equipment'] != equipment:
                raise Rejected('Equipment changed between workloads/processes')
            if confirmation and equipment != plan['pilot_equipment']:
                raise Rejected('Confirmation equipment differs from pilot')
            r.state['characterization.equipment'] = equipment
            measured['elapsed_seconds'] = command['elapsed_seconds']
            r.state[case_id] = measured
            save(r.directory/'cases'/case_id/'samples.json', measured)
            r.note(workload=workload, mode=mode, arm=arm, output_sha256=measured['output_sha256'],
                   summary=process_summary(measured) if not debug else {'debug_control': True})

        return Case(case_id, workload+' '+arm+' '+mode+(' debug control' if debug else ''), action, requires,
            SELECTOR if arm != 'cpp-minimal' else 'characterization-reference.exe', 'native-experiment',
            {'mode': mode, 'debug': debug, 'frames': 4 if debug else 160, 'watchdog_seconds': 240,
             'contract': 'Matched valid execution and complete evidence; speed outcome is separate from execution acceptance'})

    for workload in spec['workloads']:
        for arm in ARMS:
            case = child('diagnostics.'+workload+'.'+arm, workload, arm, previous, debug=True)
            result.append(case); previous = (case.id,)
    for block, order in enumerate(orders, 1):
        for workload, arm in order:
            case = child(f'block{block}.{workload}.{arm}', workload, arm, previous)
            result.append(case); primary.append(case.id); previous = (case.id,)

    def analyze(r):
        analysis = summarize({name: r.state[name] for name in primary}, blocks, 'confirmation' if confirmation else 'pilot', spec)
        analysis['measurement_seconds'] = time.monotonic()-r.state['characterization.started']
        analysis['equipment'] = r.state['characterization.equipment']
        analysis['selection'] = plan
        analysis['orders'] = orders
        (r.directory/'analysis').mkdir(exist_ok=True)
        save(r.directory/'analysis/performance-characterization.json', analysis)
        r.note(overall_cpu_verdict=analysis['overall_cpu_verdict'], blocks=blocks,
               performance_outcome_is_not_execution_acceptance=True)
    result.append(Case('characterization.analysis', 'Independent input audits and paired process-mean confidence bounds', analyze,
                       tuple(primary), scope='offline-analysis'))
    previous = ('characterization.analysis',)
    if confirmation:
        for repeat in range(1, 4):
            for workload in spec['workloads']:
                for mode in ('preparation', 'raw', 'allocations'):
                    arm = 'rust-raw' if mode == 'raw' else 'rust-wrapper'
                    case = child(f'supplement{repeat}.{workload}.{mode}', workload, arm, previous, mode)
                    result.append(case); supplemental.append(case.id); previous = (case.id,)
        for repeat in range(1, 9):
            for workload in ('max', 'hdr'):
                for arm in ARMS if repeat % 2 else ARMS[::-1]:
                    case = child(f'lifecycle{repeat}.{workload}.{arm}', workload, arm, previous, 'lifecycle')
                    result.append(case); supplemental.append(case.id); previous = (case.id,)

        def supplements(r):
            summaries = {name: process_summary(r.state[name]) for name in supplemental}
            lifecycle = {}
            for workload in ('max', 'hdr'):
                lifecycle[workload] = {}
                for metric in ('load','provider_query','first_create','used_destroy','warm_create','unused_destroy'):
                    differences = [summaries[f'characterization.lifecycle{b}.{workload}.rust-wrapper']['lifecycle'][metric]['mean_ns']-
                        summaries[f'characterization.lifecycle{b}.{workload}.cpp-minimal']['lifecycle'][metric]['mean_ns'] for b in range(1,9)]
                    lifecycle[workload][metric] = estimate(differences)
            counts = [value['allocations']['totals_alloc_dealloc_zeroed_realloc_bytes'] for value in summaries.values() if 'allocations' in value]
            save(r.directory/'analysis/performance-supplements.json', {'processes': summaries,
                'lifecycle_paired_differences': lifecycle, 'zero_observed_rust_allocation_events': all(row[0]+row[2]+row[3] == 0 for row in counts),
                'limit': 'Rust executable calling-thread allocation requests only; no native or driver heap claim. Raw mode retains an unused wrapper context for provider queries and is diagnostic only.'})
            r.note(supplemental_processes=len(supplemental), allocation_processes=len(counts))
        result.append(Case('characterization.supplements', 'Separate allocation/preparation/raw and matched lifecycle observations', supplements,
                           tuple(supplemental), scope='offline-analysis'))
    return result


def run_batch(sdk_root, adapter_index=0, output_root=None, fixed_plan=None):
    """Run existing profiles sequentially, logging without an agent watching."""
    from run import main as run_profile

    directory = fresh(output_root or ROOT/'target/performance-batches', 'native-performance')
    index = {'status': 'running', 'started_utc': now(), 'stages': {},
             'limits': ('Independent fixed confirmation, previous data for planning only' if fixed_plan else
                        'Pilot plus selected confirmation')+'; complete evidence and speed verdict are separate.'}
    status = directory/'status.json'
    save(status, index)
    print('Batch: '+str(directory), flush=True)

    def phase(profile, extra=()):
        log = directory/(profile+'.log')
        print('Starting '+profile+'; log: '+str(log), flush=True)
        with log.open('x', encoding='utf-8', newline='\n') as stream:
            with contextlib.redirect_stdout(stream), contextlib.redirect_stderr(stream):
                code = run_profile(['run', profile, '--native', '--require-passed',
                    '--sdk-root', str(Path(sdk_root).resolve()), '--adapter-index', str(adapter_index),
                    '--output-root', str(directory/'runs'), *extra])
        matches = list((directory/'runs').glob('*-'+profile+'-*/run.json'))
        index['stages'][profile] = {'exit_code': code, 'log': str(log),
            'run': str(matches[0].parent) if len(matches) == 1 else None}
        save(status, index)
        if code != 0 or len(matches) != 1:
            raise Rejected(profile+' did not complete successfully; see '+str(log))
        if json.loads(matches[0].read_text())['status'] != 'passed':
            raise Rejected('Profile lacks passed evidence: '+profile)
        return matches[0].parent

    try:
        if fixed_plan:
            plan_path = Path(fixed_plan).resolve()
            plan = load_selection(plan_path, RECONFIRM_PROTOCOL)
            confirmation_profile = 'release-performance-reconfirm'
        else:
            pilot = phase('experiment-performance-pilot')
            plan_path = directory/'selection.json'
            plan = selection(pilot, plan_path)
            confirmation_profile = 'experiment-performance-confirm'
        index['selection'] = {'path': str(plan_path), 'blocks': plan['blocks'],
                              'predicted_measurement_seconds': plan['predicted_measurement_seconds']}
        save(status, index)
        print('Frozen confirmation: '+str(plan['blocks'])+' blocks', flush=True)
        confirmation = phase(confirmation_profile, ['--performance-plan', str(plan_path)])
        index.update(status='completed', confirmation_analysis=str(confirmation/'analysis/performance-characterization.json'),
                     supplemental_analysis=str(confirmation/'analysis/performance-supplements.json'))
    except BaseException as error:
        index.update(status='interrupted' if isinstance(error, KeyboardInterrupt) else 'failed', reason=str(error))
        raise
    finally:
        index['finished_utc'] = now()
        save(status, index)
    print('Completed; results: '+str(status), flush=True)
    return directory


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest='command', required=True)
    select = actions.add_parser('select', help='Freeze a confirmation plan from a passed pilot')
    select.add_argument('--pilot', required=True)
    select.add_argument('--output', required=True)
    run = actions.add_parser('run', help='Unattended pilot, selection and full confirmation; no agent required')
    run.add_argument('--sdk-root', required=True)
    run.add_argument('--adapter-index', type=int, default=0)
    run.add_argument('--output-root', help='Exclusive batch folders; default target/performance-batches')
    args = parser.parse_args()
    try:
        if args.command == 'select':
            print(json.dumps(selection(args.pilot, args.output), indent=2))
        else:
            if args.adapter_index < 0: raise ValueError('Adapter index must be nonnegative')
            run_batch(args.sdk_root, args.adapter_index, args.output_root)
    except (Rejected, ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
