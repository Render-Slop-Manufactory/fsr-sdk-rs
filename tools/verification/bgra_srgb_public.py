# SPDX-License-Identifier: MPL-2.0
"""Both public owners: BGRA/sRGB resource contracts and fenced SDK integration."""
from bgra_srgb_fsr import CANDIDATES, SELECTOR, evaluate, inputs, workloads as direct_workloads
from execution import Case
from native import build_case, environment, native_setup


def workloads():
    result = []
    for selection in direct_workloads():
        name = selection['name']
        if name in ('changed-input', 'decoded-reference', 'input-28') or name.startswith('output-28-'):
            continue
        result.append(selection)
        if not selection['zero']:
            for role in (0, 5):
                if selection['formats'][role] in CANDIDATES and (role == 0 or selection['generate']):
                    result.append(dict(selection, name=name + '-uav', uav=1 << role))
    return result


def cases():
    result = native_setup(format_probe=True)
    build = build_case('bgra-public')
    result.append(build)
    for provider in ('3.1.5', '4.1.1'):
        for family in ('local', 'movable'):
            group = 'bgra-public.' + provider + '.' + family
            for selection in workloads():
                if selection['generate'] and provider != '3.1.5':
                    continue
                name = group + '.' + selection['name']

                def run(r, selection=selection, provider=provider, family=family, name=name):
                    directory = r.directory / 'readbacks' / name
                    directory.mkdir(parents=True)
                    for phase in range(3):
                        for role in range(7):
                            (directory / f'input-{phase}-{role}.bin').write_bytes(inputs(selection, phase, role))
                    settings = dict(PROVIDER=provider, FAMILY=family, MODE='public', DIR=directory,
                                    FORMATS=','.join(map(str, selection['formats'])), UAV=selection['uav'],
                                    GENERATE=int(selection['generate']), DYNAMIC=0, REVERSED=0, HDR=0,
                                    SHARPEN=int(selection['sharpen']), CONTROL=int(selection['control']))
                    env = environment(r)
                    env.update({'FSR_SDK_TEST_FORMAT_' + k: str(v) for k, v in settings.items()})
                    out, err, command = r.process([r.state[build.id], '--ignored', '--exact', SELECTOR,
                                                  '--nocapture', '--test-threads=1'],
                                                 env=env, cwd=r.state['native_stage'], observe=True, timeout=180)
                    observations = evaluate(out + err, command, directory, selection, provider, mode='public')
                    r.note(selection=selection, provider=provider, family=family, integration=observations)
                    r.current['evaluation'] = 'passed'

                result.append(Case(name, 'Public BGRA/sRGB ' + family + ' ' + selection['name'], run,
                                   (build.id,) if selection['control'] else (build.id, group + '.control'),
                                   SELECTOR, 'native-gpu'))
    return result
