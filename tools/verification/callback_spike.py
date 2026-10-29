# SPDX-License-Identifier: MPL-2.0
"""Bounded callback aggregate ABI, native routing and allocation lifecycle spike."""

import argparse
from collections import Counter
import json
from pathlib import Path

from execution import Case, Rejected, Runner
from motion_scale import PROVIDERS
from native import native_setup
from records import fresh, sha

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def rows(text, prefix):
    return [json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]


def cases():
    result = native_setup()

    def abi_build(r):
        directory = r.directory / 'callback-abi'
        directory.mkdir()
        for name, toolchain in [('stable', []), ('msrv', ['+1.88.0'])]:
            r.process(['rustc', *toolchain, '--version', '--verbose'])
            output = directory / f'callback_abi_{name}.dll'
            r.process(['rustc', *toolchain, '--edition=2024', '--crate-type=cdylib',
                       '--crate-name', f'callback_abi_{name}', '-Dwarnings', '-Copt-level=1',
                       HERE / 'callback_abi.rs', '-o', output])
            r.note(**{name + '_dll_sha256': sha(output)})
        headers = r.state['headers']
        r.process(['cl', '/nologo', '/std:c++17', '/EHsc', '/W4', '/WX', '/MT', '/DCALLBACK_ABI_ONLY',
                   '/I' + str(headers / 'api/include'), '/I' + str(headers / 'upscalers/include'),
                   HERE / 'callback_spike.cpp', '/Fe' + str(directory / 'callback-abi.exe'),
                   '/Fo' + str(directory / 'callback-abi.obj')])
        r.state['callback_abi'] = directory

    result.append(Case('callbacks.build-abi', 'Build independent MSVC/Rust callback aggregate fixtures', abi_build,
                       ('windows.environment', 'abi.headers'), scope='windows-compilation'))
    for name in ('stable', 'msrv'):
        def exchange(r, name=name):
            directory = r.state['callback_abi']
            out, _, _ = r.process([directory / 'callback-abi.exe', 'abi', directory / f'callback_abi_{name}.dll'])
            expected = dict(size=56, alignment=8, handle_offset=48, exchanges_each_direction=4, negative_controls=4)
            if rows(out, 'ABI ') != [expected]:
                raise Rejected('Incomplete aggregate exchange or negative controls')
            r.note(toolchain=name, aggregate=expected)
        result.append(Case('callbacks.abi.' + name, 'By-value aggregate exchange and corruption controls: ' + name,
                           exchange, ('callbacks.build-abi',), scope='native-abi'))

    def native_build(r):
        stage, headers = r.state['native_stage'], r.state['headers']
        r.process(['cl', '/nologo', '/std:c++17', '/EHsc', '/W4', '/WX', '/MT',
                   '/I' + str(headers / 'api/include'), '/I' + str(headers / 'upscalers/include'),
                   HERE / 'callback_spike.cpp', '/Fe' + str(stage / 'callback-spike.exe'),
                   '/Fo' + str(stage / 'callback-spike.obj'), '/link', stage / 'gpu_helper.lib', 'd3d12.lib'])
        r.note(executable_sha256=sha(stage / 'callback-spike.exe'))

    result.append(Case('callbacks.build-native', 'Compile direct SDK callback instrumentation', native_build,
                       ('native.helper',), scope='windows-compilation'))
    selections = [('global', '4.1.1')] + [(mode, provider) for provider in ('3.1.5', '4.1.1')
                                         for mode in ('baseline', 'shared')]
    for mode, provider in selections:
        def execute(r, mode=mode, provider=provider):
            stage = r.state['native_stage']
            out, _, _ = r.process([stage / 'callback-spike.exe', mode,
                                  stage / 'amd_fidelityfx_loader_dx12.dll', str(PROVIDERS[provider]),
                                  str(r.options.adapter_index)], cwd=stage, timeout=180,
                                 markers=['COMPLETE'])
            events = rows(out, 'EVENT ')
            if not events or [event['seq'] for event in events] != list(range(1, len(events) + 1)):
                raise Rejected('Missing or discontinuous event inventory')
            if mode == 'global':
                queries = rows(out, 'QUERY ')
                configured = rows(out, 'CONFIGURE ')
                if len(queries) != 4 or any(query['code'] not in (2, 4) for query in queries):
                    raise Rejected('Global diagnostic control did not return four unsupported-query errors')
                delivered = {phase: [event['set'] for event in events if event['kind'] == 'message'
                                     and event['phase'] == phase] for phase in
                             ('global-upscale', 'global-other-effect', 'global-deprecated', 'global-null')}
                if len(configured) != 4:
                    raise Rejected('Incomplete global configuration inventory')
                r.note(queries=queries, configurations=configured, delivery=delivered,
                       diagnostic_stimulus_delivered=any(delivered.values()),
                       routing_matches_source=delivered == {'global-upscale': [1], 'global-other-effect': [2],
                                                           'global-deprecated': [1], 'global-null': []})
            else:
                contexts, frames = (1, 2) if mode == 'baseline' else (4, 6)
                identities = rows(out, 'PROVIDER ')
                if len(identities) != contexts or any(identity != {'id': PROVIDERS[provider], 'name': provider}
                                                       for identity in identities):
                    raise Rejected('Selected provider identity mismatch')
                created, destroyed, completed = rows(out, 'CREATE '), rows(out, 'DESTROY '), rows(out, 'FRAME ')
                if len(created) != contexts or len(destroyed) != contexts or len(completed) != frames:
                    raise Rejected('Incomplete context/frame/cleanup inventory')
                if any(item['code'] for item in created + destroyed):
                    raise Rejected('Native lifecycle error')
                if any(item['errors'] or item['removed'] or item['finite'] != 230400
                       or item['written'] != 230400 for item in completed):
                    raise Rejected('GPU integration failure')
                balance = rows(out, 'RESULT ')
                if len(balance) != 1 or any(balance[0][key] for key in ('host_live', 'resource_live', 'heap_live')):
                    raise Rejected('Live instrumented allocation at successful teardown')
                counts = Counter(event['kind'] for event in events)
                # Route absence is recorded explicitly, never counted as an
                # invoked callback proof merely because the context succeeded.
                coverage = {kind: 'observed' if counts[kind] else 'not_observed' for kind in
                            ('host_alloc', 'host_free', 'resource_alloc', 'resource_free', 'heap_alloc', 'heap_free', 'cb_alloc', 'message')}
                r.note(provider=provider, mode=mode, contexts=contexts, frames=completed, balance=balance[0],
                       live_global_configure=rows(out, 'CONFIGURE '),
                       callback_counts=dict(counts), callback_coverage=coverage,
                       cross_set_frees=sum(event['kind'] in ('heap_free', 'resource_free') and event['allocation_set']
                                          not in (0, event['set']) for event in events))
            r.note(events=events)
        result.append(Case(f'callbacks.{mode}.{provider}', 'Bounded callback observations: ' + mode + ' ' + provider,
                           execute, ('callbacks.build-native', 'callbacks.abi.stable', 'callbacks.abi.msrv'),
                           scope='native-configure' if mode == 'global' else 'native-gpu'))
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sdk-root', required=True)
    parser.add_argument('--adapter-index', required=True, type=int)
    parser.add_argument('--native', required=True, action='store_true')
    options = parser.parse_args()
    options.online = False
    options.toolchain = None
    directory = fresh(ROOT / 'target/verification', 'callback-spike')
    print(directory, flush=True)
    raise SystemExit(Runner(ROOT, directory, options).execute(cases(), 'callback-spike', require_passed=True))
