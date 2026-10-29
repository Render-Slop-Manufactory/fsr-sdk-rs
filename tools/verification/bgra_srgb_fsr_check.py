# SPDX-License-Identifier: MPL-2.0
"""Check recorded direct-SDK integration, without image/algorithm comparisons."""
import argparse
import json
from pathlib import Path

from bgra_srgb_fsr import evaluate, workloads
from records import fresh, save, sha
from run import verify_run

ROOT = Path(__file__).resolve().parents[2]


def check(run):
    original = verify_run(run)
    if not original['inputs_unchanged']:
        raise ValueError('Native attempt inputs changed during execution')
    frozen = {p['path']: p['sha256'] for p in json.loads((run / 'inputs.json').read_text())['files']}
    equipment = ['crates/fsr-sdk/tests/native/typed_reference.cpp',
                 'crates/fsr-sdk/tests/native/typed_formats.h',
                 'crates/fsr-sdk/tests/native/typed_formats.rs',
                 'crates/fsr-sdk/tests/native/gpu.cpp', 'tools/verification/native.py']
    for p in equipment:
        if frozen.get(p) != sha(ROOT / p):
            raise ValueError('Native equipment differs from recorded source: ' + p)
    setup = {k: v for k, v in original['cases'].items() if not k.startswith('bgra-fsr.') or k == 'bgra-fsr.build.fsr_sdk'}
    if len(setup) != 9 or any(v != 'passed' for v in setup.values()):
        raise ValueError('Native equipment prerequisites did not pass')
    observed = []
    for provider in ('3.1.5', '4.1.1'):
        for selection in workloads():
            if selection['generate'] and provider != '3.1.5':
                continue
            name = 'bgra-fsr.' + provider + '.' + selection['name']
            case = json.loads((run / 'cases' / name / 'result.json').read_text())
            if case['evaluation'] != 'passed' or len(case['commands']) != 2:
                raise ValueError('Incomplete native workload: ' + name)
            if case['observations']['selection'] != selection:
                raise ValueError('Workload selection differs from recorded inputs: ' + name)
            repeats = []
            for repeat, command in enumerate(case['commands']):
                settings = command['environment_overrides']
                if settings['FSR_SDK_TEST_FORMAT_MODE'] != 'reference' or settings['FSR_SDK_TEST_FORMAT_PROVIDER'] != provider:
                    raise ValueError('Wrong native route/provider: ' + name)
                text = (run / command['stdout']).read_text(encoding='utf-8', errors='replace')
                text += (run / command['stderr']).read_text(encoding='utf-8', errors='replace')
                directory = run / 'readbacks' / name / str(repeat)
                repeats.append(evaluate(text, command, directory, selection, provider))
            observed.append(dict(case=name, provider=provider, selection=selection, repeats=repeats))
    return dict(schema_version=1, scope='SDK integration only; no image, reconstruction or algorithm comparisons',
                original_status=original['status'], original_input_identity=original['input_identity'],
                original_run_sha256=sha(run / 'run.json'), status='passed', setup=setup,
                native_equipment=[dict(path=p, sha256=frozen[p]) for p in equipment], workloads=observed,
                workload_count=len(observed), children=2 * len(observed), submitted_frames=6 * len(observed))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run', type=Path, required=True)
    options = p.parse_args()
    output = fresh(ROOT / 'target/verification', 'bgra-srgb-sdk-check')
    print(output, flush=True)
    result = check(options.run.resolve())
    result['checker_sha256'] = sha(Path(__file__))
    save(output / 'integration.json', result)
    print(json.dumps({k: result[k] for k in ('scope', 'status', 'workload_count', 'children', 'submitted_frames')}, indent=2))


if __name__ == '__main__':
    main()
