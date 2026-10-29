# SPDX-License-Identifier: MPL-2.0
"""Independent raw/public probes; acknowledgement alone never proves mask writes."""
import json

from execution import Case, Rejected
from native import build_case, environment, native_setup

REGIONS = ((320, 180), (256, 128), (320, 180))


def evaluate_probe(text, command, *, flags, alias, control):
    if (command['execution'] != 'completed' or command['exit_code'] != 0 or
            text.count('1 passed; 0 failed') != 1):
        raise Rejected('Reactive probe did not complete its exact native test')
    try:
        summaries = [json.loads(line.removeprefix('REACTIVE_PROBE ')) for line in text.splitlines()
                     if line.startswith('REACTIVE_PROBE ')]
        frames = [json.loads(line.removeprefix('REACTIVE_FRAME ')) for line in text.splitlines()
                  if line.startswith('REACTIVE_FRAME ')]
        if len(summaries) != 1 or not all(type(item) is dict for item in summaries + frames):
            raise ValueError('Exactly one structured completion is required')
        result = summaries[0]
        if (type(result['flags']) is not int or result['flags'] != flags or
                result['alias'] is not alias or result['control'] is not control or
                type(result['cleanup_ok']) is not bool):
            raise ValueError('Probe selection/cleanup identity mismatch')
        count = result['frames']
        code = result['native_code']
        if type(count) is not int or type(code) is not int or not 0 <= code <= 0xffffffff:
            raise ValueError('Invalid frame count or native code')
        if result['outcome'] == 'rejected':
            if control or code == 0 or not 0 <= count < 3:
                raise ValueError('Rejection must identify an actual nonzero native return')
            evaluation = 'observed' if result['cleanup_ok'] else 'failed'
        else:
            expected = 'control' if control else 'generated'
            if result['outcome'] != expected or code != 0 or count != 3 or not result['cleanup_ok']:
                raise ValueError('Incomplete generation/control or cleanup')
            evaluation = 'passed'
        if len(frames) != count:
            raise ValueError('Missing or duplicate completed GPU frame')
        for phase, frame in enumerate(frames):
            width, height = REGIONS[phase]
            if any(type(frame[key]) is not int for key in
                   ('phase', 'width', 'height', 'changed', 'zeros', 'outside_changed', 'minimum', 'maximum')):
                raise ValueError('Invalid metric type')
            pixels = width * height
            if (frame['phase'], frame['width'], frame['height']) != (phase, width, height):
                raise ValueError('Frame geometry/order mismatch')
            if not (0 <= frame['zeros'] <= pixels and 0 <= frame['changed'] <= pixels and
                    0 <= frame['minimum'] <= frame['maximum'] <= 255) or frame['outside_changed'] != 0:
                raise ValueError('Invalid mask metrics or writes outside active region')
            if control:
                if (frame['changed'], frame['zeros'], frame['minimum'], frame['maximum']) != (0, 0, 227, 227):
                    raise ValueError('No-generator control lost its seed')
            elif frame['changed'] <= pixels // 2:
                raise ValueError('Mask stayed at the upload sentinel')
            elif alias and (frame['zeros'] != pixels or frame['maximum'] != 0):
                raise ValueError('Identical-colour control did not produce zero reactivity')
            elif flags == 0 and not alias and frame['maximum'] == 0:
                raise ValueError('Untransformed distinct colours produced no reactive signal')
    except (ValueError, KeyError, TypeError) as error:
        raise Rejected('Invalid reactive probe evidence: ' + str(error)) from error
    return dict(result, evaluation=evaluation, measurements=frames)


def cases(*, public=False, selected=False):
    if selected and not public:
        raise ValueError('Selected provider probe requires the public route')
    namespace = 'reactive-selected' if selected else 'reactive-public' if public else 'reactive-probe'
    selector = 'upscaler::dx12::native_dispatch::' + ('public_reactive_mask_probe' if public else 'reactive_mask_probe')
    def abi(r):
        root = r.state['headers']
        r.process(['cl', '/nologo', '/std:c++17', '/W4', '/WX', '/c',
            f'/I{root / "api/include"}', f'/I{root / "upscalers/include"}',
            r.repository / 'crates/fsr-sdk/tests/native/reactive_mask_abi.cpp',
            f'/Fo{r.directory / "reactive-probe.obj"}'])
    result = native_setup(reactive_probe=True)
    result.append(Case(namespace + '.abi', 'Independent pinned-header probe-only layout/flags', abi,
        ('windows.environment', 'abi.headers'), scope='windows-compilation'))
    build = build_case(namespace)
    build.requires += (namespace + '.abi',)
    result.append(build)
    selections = [('control', 0, False, True), ('identical', 0, True, False)] + [
        (f'flags-{flags:02d}', flags, False, False) for flags in range(16)]
    providers = (('fsr3', '3.1.5'), ('fsr2', '2.3.4')) if selected else (('', None),)
    for provider_tag, provider_name, family, (name, flags, alias, control) in [
            (tag, provider, family, selection) for tag, provider in providers
            for family in (('local', 'movable') if public else ('raw',))
            for selection in selections]:
        prefix = namespace + ('.' + provider_tag if selected else '') + ('.' + family if public else '')
        def run(r, flags=flags, alias=alias, control=control, family=family, provider_name=provider_name):
            variables = environment(r)
            variables.update(FSR_SDK_TEST_REACTIVE_FLAGS=str(flags), FSR_SDK_TEST_REACTIVE_ALIAS=str(int(alias)),
                             FSR_SDK_TEST_REACTIVE_CONTROL=str(int(control)),
                             FSR_SDK_TEST_REACTIVE_FAMILY=family)
            if provider_name:
                variables['FSR_SDK_TEST_REACTIVE_PROVIDER'] = provider_name
            out, err, command = r.process([r.state[build.id], '--ignored', '--exact',
                selector, '--nocapture', '--test-threads=1'],
                cwd=r.state['native_stage'], env=variables, observe=True, timeout=180)
            if public and ('Reactive public route: family=' + family) not in err.splitlines():
                raise Rejected('Public generator family route marker missing')
            if provider_name and ('Reactive requested provider: name=' + provider_name) not in err.splitlines():
                raise Rejected('Explicit generator provider route marker missing')
            evidence = evaluate_probe(out + err, command, flags=flags, alias=alias, control=control)
            r.note(reactive_probe=evidence, public_generator_exercised=public, family=family, provider_name=provider_name)
            r.current['evaluation'] = evidence['evaluation']
        prerequisites = (build.id,) if control else (build.id, prefix + '.control')
        result.append(Case(prefix + '.' + name, ('Public ' + family if public else 'Raw') + ' reactive probe ' + name, run, prerequisites,
            selector, 'native-experiment',
            {'contract':'GPU evidence required for generation; native rejection remains observed',
             'flags':flags, 'alias':alias, 'control':control, 'family':family,
             'provider_name':provider_name, 'watchdog_seconds':180}))
    return result
