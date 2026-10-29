# SPDX-License-Identifier: MPL-2.0
"""Bounded P01 CPU timing protocol; P02 GPU/C++ parity remains separate."""
import re
from execution import Case, Rejected
from native import native_setup, build_case, environment
from records import save

SELECTOR = 'upscaler::dx12::tests::performance::'


def samples(log):
    result = {}
    for group, index, fields in re.findall(r'^PERF_SAMPLE (\w+) (\d+) (.+)$', log, re.M):
        for arm, value in re.findall(r'(\w+)=([\d.eE+-]+)', fields):
            number = float(value)
            if not 0 <= number < float('inf'):
                raise Rejected('Invalid timing sample')
            key = group + '.' + arm
            values = result.setdefault(key, {})
            if index in values:
                raise Rejected('Duplicate timing sample')
            values[index] = number
    return result


def distribution(values):
    ordered = sorted(values)
    return {'samples': len(ordered), 'min_ns': ordered[0],
            'median_ns': ordered[len(ordered)//2],
            'p95_ns': ordered[int((len(ordered)-1)*.95)], 'max_ns': ordered[-1]}


def validate(measurements, controlled):
    expected = ({'controlled.'+arm:set(map(str,range(31))) for arm in ('empty','raw','inspect','wrapper')} |
                {'controlled_lifecycle.'+arm:set(map(str,range(31))) for arm in ('create','destroy')}) if controlled else (
                {'native.'+arm:set(map(str,range(32,160))) for arm in ('raw','wrapper','inspect')} |
                {'native_lifecycle.'+arm:set(map(str,range(8,39))) for arm in ('create','destroy')})
    if set(measurements) != set(expected) or any(set(measurements[key]) != indices for key,indices in expected.items()):
        raise Rejected('Incomplete performance sample matrix')


def cases():
    namespace = 'performance'
    build = build_case(namespace, release=True)
    result = native_setup(optimized=True) + [build]
    previous = (build.id,)

    def child(name, selector, debug, requires, collect=False):
        def action(r):
            env = environment(r)
            env['FSR_SDK_PERFORMANCE_DEBUG'] = str(int(debug))
            stdout, stderr, _ = r.process([
                r.state[build.id], '--ignored', '--exact', SELECTOR + selector,
                '--nocapture', '--test-threads=1'], cwd=r.state['native_stage'],
                env=env, markers=('PERF ' + ('controlled' if selector.startswith('controlled') else 'native') + ': PASSED',
                                  '1 passed; 0 failed'), timeout=300)
            if collect:
                measurements = samples(stderr)
                validate(measurements, selector.startswith('controlled'))
                summaries = {key:distribution(list(values.values())) for key,values in measurements.items()}
                group = 'controlled' if selector.startswith('controlled') else 'native'
                wrapper, raw = measurements[group+'.wrapper'], measurements[group+'.raw']
                summaries[group+'.paired_delta'] = distribution([wrapper[index]-raw[index] for index in wrapper])
                if selector.startswith('controlled'):
                    if not re.search(r'^PERF_COUNTS dispatch allocations=\[0, 0, 0, 0\]', stderr, re.M):
                        raise Rejected('Expected no Rust allocation in warmed controlled dispatch')
                save(r.directory / 'cases' / r.current['id'] / 'samples.json', measurements)
                r.note(distributions=summaries, unit='ns', allocation_scope='Rust test executable, current thread; native heaps excluded',
                       observations=[line for line in stderr.splitlines() if line.startswith(('PERF_COUNTS','PERF provider:', 'Adapter:', 'Driver query:'))])
        return Case(namespace+'.'+name, name+' CPU baseline protocol', action, requires,
                    SELECTOR+selector, 'native-experiment',
                    {'contract':'Protocol completion, not a performance threshold or GPU-time claim',
                     'optimized':True,'debug_layer':debug,'watchdog_seconds':300})

    control = child('diagnostics', 'native_cpu_baseline', True, previous)
    result.append(control)
    previous = (control.id,)
    for repeat in range(1,4):
        controlled = child(f'controlled.r{repeat}', 'controlled_cpu_baseline', False, previous, True)
        native = child(f'native.r{repeat}', 'native_cpu_baseline', False, (controlled.id,), True)
        result.extend((controlled,native))
        previous = (native.id,)
    return result
