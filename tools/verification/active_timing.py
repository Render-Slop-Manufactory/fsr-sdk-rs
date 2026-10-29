# SPDX-License-Identifier: MPL-2.0
"""Matched active-render transitions; invalid execution is never a speed score."""
import json
import math
from pathlib import Path
import re
import struct

from execution import Case, Rejected
from gpu_timing import parse as gpu_parse, compatible as gpu_compatible, SELECTOR
from native import native_setup, build_case, environment
from performance import distribution
from records import save, sha

HERE = Path(__file__).resolve().parent


def protocol():
    return json.loads((HERE / 'active-timing-protocol.json').read_text())


def size_at(mode, frame, maximum, smaller, debug=False):
    bounds = (1, 3) if debug else (64, 112)
    return smaller if mode == 'small' or (mode == 'transition' and bounds[0] <= frame < bounds[1]) else maximum


def bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def input_expected(frame, x, y, width, height, maximum_width, jitter, rgb_scale=1):
    if x >= width or y >= height:
        return [65504, 65504, 65504, 1, 1, 65504, 65504]
    jx, jy = jitter
    return [rgb_scale*(.1+.75*((x+.5-jx)/width-frame/maximum_width)),
            rgb_scale*(.1+.75*(y+.5-jy)/height), rgb_scale*.2, 1, .5, -1/maximum_width if frame else 0, 0]


def audit_inputs(rows, metadata, jitter_bits, maximum_width, checkpoints, rgb_scale=1):
    expected = {}
    for frame in checkpoints:
        w, h = metadata[frame][:2]
        points = [(0, 0), (w-1, h-1)]
        if w < maximum_width: points.append((w, h))
        jitter = [struct.unpack('<f', struct.pack('<I', value))[0] for value in jitter_bits[frame]]
        for x, y in points:
            expected[(frame, x, y)] = input_expected(frame, x, y, w, h, maximum_width, jitter, rgb_scale)
    if len(rows) != len(expected) or {tuple(row[:3]) for row in rows} != set(expected):
        raise Rejected('Missing/duplicate GPU input sample')
    for row in rows:
        values = []
        for channel, value in enumerate(row[3:]):
            if not 0 <= value < (2**32 if channel == 4 else 2**16):
                raise Rejected('Invalid GPU input sample bits')
            values.append(struct.unpack('<f' if channel == 4 else '<e',
                                        struct.pack('<I' if channel == 4 else '<H', value))[0])
        for channel, (actual, target) in enumerate(zip(values, expected[tuple(row[:3])])):
            # The fixture truncates normal binary16 mantissas. One relative
            # binary16 step plus 1e-6 for binary32 arithmetic bounds this oracle;
            # depth, alpha and inactive guards remain exact.
            tolerance = 0 if channel in (3, 4) or target == 65504 else abs(target)/1024+1e-6
            if not math.isfinite(actual) or abs(actual-target) > tolerance:
                raise Rejected('GPU input differs from independent scene/motion oracle')


def parse(log, *, arm, mode, debug=False, hdr=False, masks=False):
    spec = protocol()
    queries = re.findall(r'^ACTIVE_QUERY (\w+) (\d+) (\d+) (\d+)$', log, re.M)
    maximum = re.findall(r'^GPU_QUERY (\d+) (\d+) (\d+)$', log, re.M)
    if len(queries) != 1 or queries[0][0] != mode or mode not in spec['workloads'] or len(maximum) != 1:
        raise Rejected('Missing/duplicate/mismatched active-size query')
    rw, rh, phase = map(int, maximum[0])
    sw, sh, small_phase = map(int, queries[0][1:])
    if not (0 < sw < rw and 0 < sh < rh and 0 < small_phase <= 2147483647):
        raise Rejected('Invalid smaller size/jitter phase')
    frames = spec['debug_frames'] if debug else spec['frames']
    rows = re.findall(r'^ACTIVE_FRAME (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)$', log, re.M)
    if len(rows) != frames or {int(row[0]) for row in rows} != set(range(frames)):
        raise Rejected('Missing/duplicate active-size frame')
    metadata = [list(map(int, row[1:])) for row in sorted(rows, key=lambda row: int(row[0]))]
    phases = []
    for frame, row in enumerate(metadata):
        w, h = size_at(mode, frame, (rw, rh), (sw, sh), debug)
        count = small_phase if (w, h) == (sw, sh) else phase
        if row != [w, h, count, int(frame == 0), bits(w), bits(h)]:
            raise Rejected('Wrong active-size sequence/reset/motion scale')
        phases.append(count)
    result = gpu_parse(log, arm=arm, debug=debug, dynamic=True, jitter_phases=phases, hdr=hdr, masks=masks)
    rows = re.findall(r'^ACTIVE_VALID (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)$', log, re.M)
    checkpoints = spec['debug_checkpoints'] if debug else spec['checkpoints']
    if len(rows) != len(checkpoints) or sorted(int(row[0]) for row in rows) != checkpoints:
        raise Rejected('Missing/duplicate active-size checkpoint')
    audited = [list(map(int, row)) for row in sorted(rows, key=lambda row: int(row[0]))]
    for _, finite, written, errors, output_hash, input_hash in audited:
        expected_errors = 0 if debug else 2**64-1  # Existing helper's unavailable-info-queue marker.
        if finite != 1920*1080 or written != finite or errors != expected_errors or not (0 < output_hash < 2**64 and 0 < input_hash < 2**64):
            raise Rejected('Invalid active-size output/input checkpoint')
    if len(re.findall(r'^Active render input readback/guards: PASSED$', log, re.M)) != len(checkpoints):
        raise Rejected('Missing input guard/readback confirmation')
    rows = re.findall(r'^ACTIVE_INPUT (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)$', log, re.M)
    inputs = [list(map(int, row)) for row in rows]
    audit_inputs(inputs, metadata, result['jitter_bits'], rw, checkpoints, rgb_scale=8 if hdr else 1)
    result.update(mode=mode, smaller={'render': [sw, sh], 'phase': small_phase},
                  active_frames=metadata, checkpoints=audited, input_samples=inputs)
    return result


def compatible(previous, current):
    gpu_compatible(previous, current)
    for key in ('mode', 'smaller', 'active_frames', 'checkpoints', 'input_samples'):
        if previous[key] != current[key]:
            raise Rejected('Unmatched active-size comparison: ' + key)


def summarize(processes, spec):
    blocks = []
    for block in range(1, spec['blocks'] + 1):
        for arm in ('cpp-minimal', 'rust-wrapper'):
            moving = processes[f'active.block{block}.transition.{arm}']
            for checkpoint in moving['checkpoints']:
                frame = checkpoint[0]
                mode = 'small' if spec['transition']['shrink'] <= frame < spec['transition']['grow'] else 'max'
                fixed = processes[f'active.block{block}.{mode}.{arm}']
                match = next(row for row in fixed['checkpoints'] if row[0] == frame)
                if (checkpoint[5] != match[5] or moving['jitter_bits'][frame] != fixed['jitter_bits'][frame]
                    or [row for row in moving['input_samples'] if row[0] == frame] !=
                       [row for row in fixed['input_samples'] if row[0] == frame]):
                    raise Rejected('Transition/stable control inputs differ at matched active size')
                if frame < spec['transition']['shrink'] and checkpoint[4] != match[4]:
                    raise Rejected('Identical pre-transition histories produce unmatched output')
        segments = {}
        for name, (start, end) in spec['segments'].items():
            values = {}
            for mode in spec['workloads']:
                values[mode] = {}
                for arm in ('cpp-minimal', 'rust-wrapper'):
                    child = processes[f'active.block{block}.{mode}.{arm}']
                    values[mode][arm] = {
                        'cpu': distribution([x*1e9/child['frequency_hz'] for x in child['ticks']['frame'][start:end]]),
                        'gpu': distribution([(row[3]-row[2])*1e9/child['gpu_frequency_hz'] for row in child['gpu_ticks'][start:end]])}
            control = 'small' if name.startswith('small') or name == 'first_shrink' else 'max'
            delta = {arm: {clock: values['transition'][arm][clock]['median_ns'] - values[control][arm][clock]['median_ns']
                           for clock in ('cpu', 'gpu')} for arm in ('cpp-minimal', 'rust-wrapper')}
            segments[name] = {'frames': [start, end], 'stable_control': control, 'distributions': values,
                              'transition_minus_stable_median_ns': delta,
                              'wrapper_minus_cpp_median_ns': {
                                  mode: {clock: values[mode]['rust-wrapper'][clock]['median_ns'] - values[mode]['cpp-minimal'][clock]['median_ns']
                                         for clock in ('cpu', 'gpu')} for mode in spec['workloads']}}
        blocks.append({'block': block, 'segments': segments})
    return {'protocol': spec, 'blocks': blocks, 'performance_threshold': None,
            'limit': 'One frame per transition per process; correlated frame samples are not independent process repetitions. Serial fences/checkpoint readbacks can affect subsequent clocks.'}


def cases(include_features=False):
    spec = protocol()
    if include_features:
        from feature_comparison import selected_protocol
        spec = selected_protocol()
    build = build_case('active-timing', release=True)

    def cpp_build(r):
        stage, headers = r.state['native_stage'], r.state['headers']
        executable = stage / 'active-reference.exe'
        r.process(['cl', '/nologo', '/std:c++17', '/O2', '/W4', '/WX', '/MT', '/EHsc',
                   f'/I{headers/"api/include/dx12"}', f'/I{headers/"upscalers/include"}',
                   r.repository/'crates/fsr-sdk/tests/native/gpu_reference.cpp',
                   f'/Fo{stage/"active_reference.obj"}', f'/Fe{executable}',
                   '/link', stage/'gpu_helper.lib', 'd3d12.lib'])
        r.state['active.cpp'] = executable
        r.note(executable_sha256=sha(executable), optimizer='/O2 /MT',
               independence='Standalone pinned-header C++ SDK consumer; shared equipment contains no AMD calls')

    result = native_setup(optimized=True, gpu_timing=True) + [build,
        Case('active.build.cpp', 'Independent optimized active-size C++ consumer', cpp_build, ('native.helper',), scope='windows-compilation')]
    previous = (build.id, 'active.build.cpp')
    children = []

    def child(name, mode, arm, requires, debug=False):
        case_id = 'active.' + name
        feature = mode if mode in ('masks', 'hdr') else None

        def action(r):
            output = r.directory/'readbacks'/case_id/'final.rgba16f'
            output.parent.mkdir()
            env = environment(r)
            env.update(FSR_SDK_CPU_ARM=arm, FSR_SDK_PERFORMANCE_DEBUG=str(int(debug)),
                       FSR_SDK_CPU_PREINSPECT='0', FSR_SDK_CPU_OUTPUT=output, FSR_SDK_ACTIVE_TIMING='max' if feature else mode)
            if feature: env['FSR_SDK_FEATURE_TIMING'] = feature
            args = [r.state['active.cpp']] if arm == 'cpp-minimal' else [r.state[build.id],
                '--ignored', '--exact', SELECTOR, '--nocapture', '--test-threads=1']
            _, stderr, _ = r.process(args, cwd=r.state['native_stage'], env=env,
                                    markers=('CPU_COMPARISON PASSED '+arm,), timeout=240)
            if feature:
                from feature_comparison import parse as parse_feature
                measured = parse_feature(stderr, arm=arm, feature=feature, debug=debug)
            else:
                measured = parse(stderr, arm=arm, mode=mode, debug=debug)
            if not output.is_file() or output.stat().st_size != 1920*1080*8:
                raise Rejected('Missing/invalid active-size final output')
            measured['output_sha256'] = sha(output)
            key = 'active.reference.' + mode + ('.debug' if debug else '.timed')
            if key in r.state:
                compatible(r.state[key], measured)
                if feature and r.state[key]['feature_checkpoints'] != measured['feature_checkpoints']:
                    raise Rejected('Unmatched feature checkpoint hashes/range')
            else:
                r.state[key] = measured
            # Equipment and creation must match across workloads too. Histories,
            # per-frame jitter and outputs intentionally differ between workloads.
            equipment = {key: measured[key] for key in ('provider', 'spec', 'adapter_driver', 'frequency_hz', 'gpu_frequency_hz', 'query', 'smaller')}
            if feature:
                from cpu_comparison import expected_spec
                equipment['spec'] = expected_spec(tuple(measured['query']['render']), (1920, 1080), dynamic=True)
            if 'active.equipment' in r.state and r.state['active.equipment'] != equipment:
                raise Rejected('Unmatched equipment/allocations across workloads')
            r.state['active.equipment'] = equipment
            r.state[case_id] = measured
            save(r.directory/'cases'/case_id/'samples.json', measured)
            r.note(mode=mode, provider=measured['provider'], query=measured['query'], smaller=measured['smaller'],
                   checkpoints=measured['checkpoints'], cpu=distribution(measured['measured_ns']),
                   gpu=distribution(measured['gpu_ns']), empty_gpu=distribution(measured['empty_gpu_ns']))

        return Case(case_id, mode+' '+arm+(' debug controls' if debug else ' active-size timing'), action, requires,
                    SELECTOR if arm != 'cpp-minimal' else 'active-reference.exe', 'native-experiment',
                    {'debug_layer': debug, 'frames': 4 if debug else 160, 'warmup': 0 if debug else 32,
                     'watchdog_seconds': 240, 'contract': 'Matched valid execution; descriptive performance without speed gate'})

    for mode in spec['workloads']:
        for arm in ('cpp-minimal', 'rust-wrapper'):
            case = child('diagnostics.'+mode+'.'+arm, mode, arm, previous, True)
            result.append(case)
            previous = (case.id,)
    for block in range(1, spec['blocks']+1):
        modes = spec['workloads'] if block % 2 else list(reversed(spec['workloads']))
        arms = ('cpp-minimal', 'rust-wrapper') if block % 2 else ('rust-wrapper', 'cpp-minimal')
        for mode in modes:
            for arm in arms:
                case = child(f'block{block}.{mode}.{arm}', mode, arm, previous)
                result.append(case)
                children.append(case.id)
                previous = (case.id,)

    def analyze(r):
        if include_features:
            from feature_comparison import summarize as summarize_features
            summary = summarize_features({name: r.state[name] for name in children}, spec)
        else:
            summary = summarize({name: r.state[name] for name in children}, spec)
        (r.directory/'analysis').mkdir(exist_ok=True)
        save(r.directory/'analysis'/('feature-comparison.json' if include_features else 'active-timing.json'), summary)
        r.note(blocks=spec['blocks'], independent_timed_processes=len(children), performance_threshold=None)

    result.append(Case('active.analysis', 'Selected-feature CPU/GPU and active-control comparisons' if include_features else
                       'Stratified CPU/GPU transition and stable-control comparisons', analyze, tuple(children), scope='offline-analysis'))
    return result
