# SPDX-License-Identifier: MPL-2.0
"""Bounded raw motion-format/scale probe with independent readback oracles."""
from functools import lru_cache
import hashlib
import json
import struct

from execution import Case, Rejected
from native import build_case, environment, native_setup
from variable_output import output_metrics

SELECTOR = 'upscaler::dx12::native_dispatch::motion_scale_probe'
PROVIDERS = {'4.1.1': 17700776140664213505, '3.1.5': 17700776140660019205}


def bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def workloads():
    result = []
    for axis in range(3):
        base = (-4.0 if axis != 1 else 0.0, -2.0 if axis != 0 else 0.0)
        for arm, fmt, scale, stored in (
            ('half', 0, (1.0, 1.0), base),
            ('float', 1, (1.0, 1.0), base),
            ('asymmetric', 1, (2.0, 4.0), (base[0]/2, base[1]/4)),
            ('signed', 1, (-2.0, -4.0), (-base[0]/2, -base[1]/4)),
            ('wrong', 1, (2.0, 4.0), (-base[0]/2, -base[1]/4)),
        ):
            result.append(dict(name=f'{axis}-{arm}', axis=axis, arm=arm, format=fmt, scale=scale, stored=stored))
    return result


@lru_cache(maxsize=12)
def color_input(axis, phase):
    pixels = []
    for y in range(180):
        v = (y + 180 - (2*phase if axis != 0 else 0)) % 180
        for x in range(320):
            u = (x + 320 - (4*phase if axis != 1 else 0)) % 320
            pixels.append(struct.pack('<4e', (1+(u//4)%7)/8, (1+(v//3)%7)/8,
                                      0.75 if (u//8+v//8)%2 else 0.25, 1.0))
    return b''.join(pixels)


def input_oracles(selection, phase):
    stored = selection['stored'] if phase else (0.0, 0.0)
    motion = struct.pack('<2f' if selection['format'] else '<2e', *stored) * (320*180)
    return (color_input(selection['axis'], phase), struct.pack('<f', 0.25)*(320*180),
            motion, struct.pack('<f', 1.0))


def evaluate(text, command, directory, selection, provider, control=False):
    if command['execution'] != 'completed' or command['exit_code'] != 0 or text.count('1 passed; 0 failed') != 1:
        raise Rejected('Motion child did not complete its exact test')
    parse = lambda prefix: [json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    identities = [line for line in text.splitlines() if line.startswith('Motion probe provider:')]
    if len(identities) != 1 or f'ProviderId({PROVIDERS[provider]})' not in identities[0] or f'"{provider}"' not in identities[0]:
        raise Rejected('Missing or wrong selected motion provider identity')
    if parse('MOTION_RESULT ') != [dict(outcome='completed', frames=4, code=0, cleanup_ok=True)]:
        raise Rejected('Motion native rejection, cleanup failure or incomplete recording')
    descs, frames = parse('MOTION_DESCRIPTOR '), parse('MOTION_FRAME ')
    if len(descs) != 4 or len(frames) != 4:
        raise Rejected('Missing or duplicate motion descriptors/readbacks')
    evidence = []
    for phase, (desc, frame) in enumerate(zip(descs, frames)):
        expected = dict(phase=phase, provider=provider, axis=selection['axis'], format=selection['format'],
                        scale_bits=list(map(bits, selection['scale'])), stored_bits=list(map(bits, selection['stored'])),
                        reset=phase == 0, control=control)
        if desc != expected:
            raise Rejected('Motion selection/descriptor bit mismatch')
        if frame['phase'] != phase or frame['completed'] != 1 or any(frame[k] for k in ('input_mismatches', 'debug_errors', 'removed')):
            raise Rejected('Motion GPU diagnostics, input mutation or incomplete fence')
        inputs = input_oracles(selection, phase)
        for role, data in enumerate(inputs):
            if (directory/f'phase-{phase}-role-{role}.bin').read_bytes() != data:
                raise Rejected(f'Independent motion input oracle mismatch: phase={phase} role={role}')
        output = (directory/f'phase-{phase}-role-4.bin').read_bytes()
        if len(output) != 640*360*8:
            raise Rejected('Incorrect motion output readback size')
        metrics = output_metrics(output, (640,360), (640,360))
        if any(frame[key] != value for key, value in metrics.items()):
            raise Rejected('Motion helper/independent output metrics disagree')
        if metrics['finite'] != 640*360 or metrics['written'] != (0 if control else 640*360):
            raise Rejected('Motion output seed/finite/write criterion failed')
        evidence.append(dict(descriptor=desc, metrics=frame, output_sha256=hashlib.sha256(output).hexdigest(),
                             input_sha256=[hashlib.sha256(data).hexdigest() for data in inputs]))
    if {p.name for p in directory.iterdir()} != {f'phase-{p}-role-{r}.bin' for p in range(4) for r in range(5)}:
        raise Rejected('Missing or extra motion readback payload')
    return dict(frames=evidence, control=control, provider_name=provider, selection=selection)


def compare_frames(reference, candidate):
    if len(reference) != len(candidate) or len(reference) != 640*360*8:
        raise Rejected('Comparison payload size mismatch')
    changed = sum(reference[i:i+8] != candidate[i:i+8] for i in range(0, len(reference), 8))
    return dict(equal=changed == 0, changed_pixels=changed,
                reference_sha256=hashlib.sha256(reference).hexdigest(), candidate_sha256=hashlib.sha256(candidate).hexdigest())


def assess(readback):
    comparisons = []
    hypotheses = {}
    for axis in range(3):
        for arm in ('float', 'asymmetric', 'signed', 'wrong'):
            frames = [compare_frames(readback(axis, 'half', p), readback(axis, arm, p)) for p in range(4)]
            for phase, values in enumerate(frames):
                comparisons.append(dict(axis=axis, candidate=arm, phase=phase, **values))
            if arm == 'wrong':
                hypotheses[f'axis-{axis}-wrong-sign-sensitive'] = frames[0]['equal'] and any(not f['equal'] for f in frames[1:])
            else:
                hypotheses[f'axis-{axis}-{arm}-equivalent'] = all(f['equal'] for f in frames)
    return dict(comparisons=comparisons, hypotheses=hypotheses, hypotheses_passed=all(hypotheses.values()))


def cases():
    namespace = 'motion-scale'
    result = native_setup(motion_probe=True)
    build = build_case(namespace)
    result.append(build)
    control_selection = workloads()[0]
    selections = [('control', '4.1.1', control_selection, True)] + [
        (tag, provider, selection, False) for tag, provider in (('fsr4','4.1.1'), ('fsr3','3.1.5')) for selection in workloads()]
    for tag, provider, selection, control in selections:
        name = namespace+'.'+tag+('' if control else '.'+selection['name'])
        def run(r, name=name, provider=provider, selection=selection, control=control):
            directory = r.directory/'readbacks'/name
            directory.mkdir(parents=True)
            env = environment(r)
            settings = dict(PROVIDER=provider, AXIS=selection['axis'], FORMAT=selection['format'],
                            SCALE=','.join(map(str, selection['scale'])), STORED=','.join(map(str, selection['stored'])),
                            CONTROL=int(control), DIR=directory)
            env.update({'FSR_SDK_TEST_MOTION_'+key: str(value) for key, value in settings.items()})
            out, err, command = r.process([r.state[build.id], '--ignored', '--exact', SELECTOR, '--nocapture', '--test-threads=1'],
                                         cwd=r.state['native_stage'], env=env, observe=True, timeout=180)
            r.note(raw_result_lines=[line for line in (out+err).splitlines() if line.startswith(('MOTION_', 'Motion probe provider:', 'Adapter:', 'Driver query:'))])
            r.note(motion_scale=evaluate(out+err, command, directory, selection, provider, control))
            r.current['evaluation'] = 'passed'
        requires = (build.id,) if control else (build.id, namespace+'.control')
        result.append(Case(name, 'Raw motion format/scale '+tag+' '+selection['name'], run, requires,
                           SELECTOR, 'native-experiment', dict(provider_name=provider, selection=selection, control=control, watchdog_seconds=180)))
    for tag, provider in (('fsr4','4.1.1'), ('fsr3','3.1.5')):
        def analyze(r, tag=tag, provider=provider):
            def read(axis, arm, phase):
                return (r.directory/'readbacks'/f'{namespace}.{tag}.{axis}-{arm}'/f'phase-{phase}-role-4.bin').read_bytes()
            assessment = assess(read)
            r.note(provider_name=provider, motion_comparisons=assessment)
            if not assessment['hypotheses_passed']:
                r.current['evaluation'] = 'observed'
        result.append(Case(namespace+'.'+tag+'.compare', 'Format/scale equivalence and independent-axis sign sensitivity', analyze,
                           tuple(namespace+'.'+tag+'.'+s['name'] for s in workloads()), scope='native-analysis',
                           expectation={'contract': 'Frozen exact-equivalence hypotheses; wrong-sign controls must change a post-reset output'}))
    return result
