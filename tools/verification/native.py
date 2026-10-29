# SPDX-License-Identifier: MPL-2.0
"""Windows ABI/native equipment and effect-specific bounded case profiles."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import urllib.request

from execution import Blocked, Case, Rejected
from records import save, sha

HERE = Path(__file__).resolve().parent
DLL_HASHES = {
    'amd_fidelityfx_loader_dx12.dll': 'e2d85aa05a9bd9ed8b38935fdf5199372cca6f74c12015143bb6f945ee1608aa',
    'amd_fidelityfx_upscaler_dx12.dll': 'd0dcccc74a43c44ba435b7a369b456e0970d8a4464e4bd683119b374f2c9fb46',
}


def header_sha(path):
    """The existing ABI pin contract normalizes text line endings."""
    content = Path(path).read_text(encoding='utf-8').replace('\r\n', '\n').encode()
    return hashlib.sha256(content).hexdigest()


def windows_environment(r):
    if os.name != 'nt':
        raise Blocked('Windows x64/MSVC required')
    output, _, _ = r.process(['pwsh', '-NoProfile', '-NonInteractive', '-File', HERE / 'windows.ps1'], name='environment')
    try:
        environment = json.loads(output)
    except ValueError as error:
        raise Blocked('Developer-shell environment was not valid JSON') from error
    # Keep the acquired environment local; never export its potentially secret values.
    environment = {key.upper(): value for key, value in environment.items() if not key.upper().startswith('FSR_SDK_')}
    r.env.update(environment)
    r.env.update(CARGO_INCREMENTAL='0', CARGO_TERM_COLOR='never', RUST_BACKTRACE='1', RUSTDOCFLAGS='-D warnings')
    r.note(msvc=environment.get('VCTOOLSVERSION'), windows_sdk=environment.get('WINDOWSSDKVERSION'),
           target_arch=environment.get('VSCMD_ARG_TGT_ARCH'))


def abi_cases():
    def headers(r):
        manifest = json.loads((r.repository / 'tools/native-headers.json').read_text(encoding='utf-8'))
        root = Path(r.options.sdk_root).resolve() if r.options.sdk_root else r.directory / 'headers'
        if not r.options.sdk_root:
            if not r.options.online:
                raise Blocked('Local SDK headers required offline; --online permits pinned header acquisition')
            for header in manifest['headers']:
                target = root / header['path']
                target.parent.mkdir(parents=True, exist_ok=True)
                url = f'https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/{manifest["commit"]}/Kits/FidelityFX/{header["path"]}'
                with urllib.request.urlopen(url, timeout=30) as response:
                    target.write_bytes(response.read())
        for header in manifest['headers']:
            target = root / header['path']
            if not target.is_file():
                raise Blocked('Missing pinned header: ' + header['path'])
            if header_sha(target) != header['sha256']:
                raise Rejected('Pinned header identity mismatch: ' + header['path'])
        r.state['headers'] = root
        r.note(sdk_commit=manifest['commit'], headers=manifest['headers'])
    cases = [Case('windows.environment', 'x64 Windows developer-tool environment', windows_environment),
             Case('abi.headers', 'Four hash-verified pinned SDK headers', headers)]
    for label, source in (('common', 'native_abi.cpp'), ('creation', 'context_lifecycle/native_abi.cpp'), ('dispatch', 'dispatch_native_abi.cpp'), ('query', 'query_native_abi.cpp')):
        def compile_probe(r, label=label, source=source):
            root = r.state['headers']
            r.process(['cl', '/nologo', '/std:c++17', '/W4', '/WX', '/c',
                f'/I{root / "api/include"}', f'/I{root / "upscalers/include"}',
                r.repository / 'crates/fsr-sdk-sys/tests' / source,
                f'/Fo{r.directory / (label + ".obj")}'])
        cases.append(Case('abi.' + label, 'Independent native ' + label + ' ABI assertions', compile_probe,
                          ('windows.environment', 'abi.headers'), scope='windows-compilation'))
    return cases


def native_setup(optimized=False, gpu_timing=False, reactive_probe=False, output_probe=False, motion_probe=False, format_probe=False):
    if motion_probe:
        output_probe = True
    if output_probe and (gpu_timing or reactive_probe):
        raise ValueError('Variable-output equipment needs its own helper build')
    if reactive_probe and gpu_timing:
        raise ValueError('Reactive and timing helpers have different workload contracts')
    def sdk(r):
        if not r.options.native:
            raise Blocked('Native execution requires explicit --native opt-in')
        if not r.options.sdk_root:
            raise Blocked('Native execution requires a local --sdk-root deployment')
        stage = r.directory / 'native'
        stage.mkdir()
        records = []
        for name, expected in DLL_HASHES.items():
            source = Path(r.options.sdk_root) / 'signedbin' / name
            if not source.is_file():
                raise Blocked('Missing local native deployment: ' + name)
            if sha(source) != expected:
                raise Rejected('Pinned native artefact identity mismatch: ' + name)
            target = stage / name
            shutil.copyfile(source, target)
            records.append({'file': name, 'sha256': sha(target)})
        output, _, _ = r.process(['pwsh', '-NoProfile', '-NonInteractive', '-File', HERE / 'windows.ps1',
            '-Mode', 'signatures', '-Directory', stage])
        signatures = json.loads(output)
        if len(signatures) != 2 or any(item['status'] != 'Valid' or 'Advanced Micro Devices' not in item['signer'] for item in signatures):
            raise Rejected('Staged AMD signatures invalid')
        r.state['native_stage'] = stage
        r.note(artifacts=records, signatures=signatures, adapter_index=r.options.adapter_index)
    def helper(r):
        stage, root = r.state['native_stage'], r.state['headers']
        r.process(['cl', '/nologo', '/std:c++17', '/W4', '/WX', '/MT', '/LD'] + (['/O2'] if optimized else []) + (['/DFSR_GPU_TIMING'] if gpu_timing else []) + (['/DFSR_REACTIVE_PROBE'] if reactive_probe else []) + (['/DFSR_OUTPUT_PROBE'] if output_probe else []) + (['/DFSR_MOTION_PROBE'] if motion_probe else []) + (['/DFSR_FORMAT_PROBE', f'/I{root / "upscalers/include"}', r.repository / 'crates/fsr-sdk/tests/native/typed_reference.cpp'] if format_probe else []) + [
            f'/I{root / "api/include/dx12"}', r.repository / 'crates/fsr-sdk/tests/native/gpu.cpp',
            r.repository / 'crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp', f'/Fo{stage}{os.sep}',
            '/link', f'/OUT:{stage / "gpu_helper.dll"}', f'/IMPLIB:{stage / "gpu_helper.lib"}', 'd3d12.lib', 'dxgi.lib'] + (['d3dcompiler.lib'] if format_probe else []))
        r.note(helper_sha256=sha(stage / 'gpu_helper.dll'), reactive_probe=reactive_probe, output_probe=output_probe, motion_probe=motion_probe, format_probe=format_probe)
    return abi_cases() + [Case('native.sdk', 'Pinned, signed local loader/provider deployment', sdk, ('windows.environment',)),
        Case('native.helper', 'DX12 submission, fencing and readback equipment', helper,
             ('native.sdk', 'abi.common', 'abi.creation', 'abi.dispatch', 'abi.query'), scope='windows-compilation')]


def build_case(namespace='native', registry=False, target='fsr_sdk', release=False):
    case_id = namespace + '.build.' + target
    def build(r):
        selected_archive = registry or r.state.get('artifact_native', False)
        if selected_archive:
            package = 'fsr-sdk' if target == 'fsr_sdk' else 'fsr-sdk-sys'
            directory = r.state['packages'] / f'{package}-{r.state["versions"][package]}'
            environment = r.state['registry_env']
        else:
            directory = r.repository / 'crates' / ('fsr-sdk' if target == 'fsr_sdk' else 'fsr-sdk-sys')
            environment = {}
        args = ['test', '--manifest-path', directory / 'Cargo.toml', '--no-run', '--message-format=json',
                '--features', 'dx12,movable' if target == 'fsr_sdk' else 'dx12']
        args += ['--lib'] if target == 'fsr_sdk' else ['--test', target]
        if release:
            args += ['--release']
        output, _, _ = r.process(r.cargo(args), env=environment)
        artifacts = [json.loads(line) for line in output.splitlines() if line.startswith('{')]
        matching = [item for item in artifacts if item.get('reason') == 'compiler-artifact' and
                    item['target']['name'] == target and item.get('executable')]
        if len(matching) != 1:
            raise Rejected('Exact test executable not reported by Cargo')
        stage = r.state['native_stage']
        executable = stage / f'{namespace}-{target}.exe'
        shutil.copyfile(matching[0]['executable'], executable)
        r.state[case_id] = executable
        r.note(executable_sha256=sha(executable), manifest=directory / 'Cargo.toml', registry_artifact=selected_archive,
               archive_hashes=r.state.get('archive_hashes') if selected_archive else None)
    prerequisites = ('native.helper',) + (('release.registry',) if registry else ())
    return Case(case_id, 'Discover and stage exact ' + target + ' test executable', build, prerequisites, scope='windows-compilation')


def environment(r):
    stage = r.state['native_stage']
    result = {'FSR_SDK_TEST_DLL': stage / 'amd_fidelityfx_loader_dx12.dll',
              'FSR_SDK_TEST_GPU_DLL': stage / 'gpu_helper.dll', 'FSR_SDK_TEST_DEVICE_DLL': stage / 'gpu_helper.dll',
              'FSR_SDK_TEST_PROVIDER_DLL': stage / 'amd_fidelityfx_upscaler_dx12.dll'}
    if r.options.adapter_index is not None:
        result['FSR_SDK_TEST_ADAPTER_INDEX'] = str(r.options.adapter_index)
    return result


def child_case(namespace, name, selector, marker, requires, *, output=None, payloads=None,
               size=None, variables=None, observe=False, timeout=180, scope='native-gpu'):
    case_id = namespace + '.' + name
    def run(r):
        env = environment(r)
        if variables:
            env.update(variables)
        images = r.directory / 'readbacks' / case_id
        if output:
            images.mkdir()
            env[output] = images
        executable = r.state[namespace + '.build.fsr_sdk']
        stdout, stderr, command = r.process([executable, '--ignored', '--exact', selector,
            '--nocapture', '--test-threads=1'], cwd=r.state['native_stage'], env=env,
            markers=(marker, '1 passed; 0 failed') if marker else (), observe=observe, timeout=timeout)
        r.note(execution=command['execution'], exit_code=command['exit_code'],
               adapter_provider_observations=[line for line in stderr.splitlines()
                   if line.startswith(('Adapter:', 'Driver query:', 'Provider:', 'Upscaler provider:'))])
        if payloads is not None:
            suffix = '.rgba16f' if size == 1843200 else '.red.u16le'
            files = sorted(images.glob('*' + suffix))
            if len(files) != payloads or any(path.stat().st_size != size for path in files):
                raise Rejected('Readback count or byte-size expectation failed')
            r.note(readbacks=[{'file': path.name, 'bytes': path.stat().st_size, 'sha256': sha(path)} for path in files])
    return Case(case_id, name.replace('-', ' '), run, requires, selector, scope,
                {'evaluation':'observed' if observe else 'passed','exit':'any' if observe else 0,
                 'markers':[marker,'1 passed; 0 failed'] if marker else [],'watchdog_seconds':timeout,
                 'environment':variables or {},'readback_count':payloads,'readback_bytes_each':size})


def regression_cases(namespace='native', registry=False, sharpening=True, movable=True):
    definitions = {item['legacy_name']: item for item in json.loads((HERE / 'native-cases.json').read_text(encoding='utf-8'))}
    cases = [build_case(namespace, registry)]
    base = []
    for legacy, name in (('public-queries', 'queries.public'), ('utility-queries', 'queries.utilities'),
                         ('native-explicit', 'lifecycle.explicit'), ('native-drop', 'lifecycle.drop'),
                         ('gate-a', 'dispatch.single'), ('pre-exposure', 'dispatch.pre-exposure'),
                         ('depth-motion', 'dispatch.depth-motion'), ('automatic-exposure', 'dispatch.automatic-exposure'),
                         ('linear-hdr', 'dispatch.linear-hdr'),
                         ('supplied-masks', 'dispatch.supplied-masks'), ('active-render', 'dispatch.active-render'),
                         ('configure-dispatch', 'dispatch.configure'),
                         ('selected-provider-dispatch', 'dispatch.selected-provider'),
                         ('gate-b', 'dispatch.temporal'), ('gate-c', 'dispatch.semantics')):
        definition = definitions[legacy]
        requires = (namespace + '.build.fsr_sdk',) if name == 'queries.public' else (namespace + '.queries.public',)
        cases.append(child_case(namespace, name, definition['selector'], definition['marker'], requires,
                                scope='native-query' if name == 'queries.utilities' else 'native-gpu'))
        base.append(namespace + '.' + name)
    if movable:
        for legacy, name in (('public-movable', 'movable.disabled'), ('public-movable-sharpening', 'movable.sharpening')):
            definition = definitions[legacy]
            cases.append(child_case(namespace, name, definition['selector'], definition['marker'], tuple(base),
                                    variables={'FSR_SDK_TEST_CONTEXT_TRACE': '1'}))
    if sharpening:
        previous = tuple(base)
        for repeat in range(1, 4):
            name = f'sharpening.integration.r{repeat}'
            cases.append(child_case(namespace, name, 'upscaler::dx12::native_dispatch::native_sharpening_integration',
                'Sharpening integration: PASSED outputs=48', previous, output='FSR_SDK_TEST_SHARPENING_INTEGRATION_OUTPUT', payloads=48, size=460800))
            previous = (namespace + '.' + name,)
        def audit(r):
            from analysis import audit_sharpening_integration
            result = audit_sharpening_integration(r.directory, namespace)
            target = r.directory / 'analysis'
            target.mkdir(exist_ok=True)
            save(target / (namespace + '-sharpening.json'), result)
            r.note(outputs=result['outputs'], comparisons=len(result['comparisons']), image_thresholds=None)
        cases.append(Case(namespace + '.sharpening.audit', 'Independent offline half-float/frame-observation audit', audit,
            tuple(namespace + f'.sharpening.integration.r{repeat}' for repeat in range(1, 4)), scope='offline-analysis'))
    return cases


def experiment_cases(kind):
    namespace = 'experiment.' + kind
    cases = native_setup()
    if kind in ('dimensions', 'allocation', 'input-lifetime', 'provider', 'shared-runtime'):
        target = {'dimensions': 'context_dimensions', 'allocation': 'context_lifecycle',
                  'input-lifetime': 'context_lifecycle', 'provider': 'query', 'shared-runtime': 'shared_runtime'}[kind]
        build = build_case(namespace, target=target)
        cases.append(build)
        if kind == 'input-lifetime':
            cases += regression_cases(namespace, sharpening=False, movable=False)
        def probe(name, selector, variables, observe=False, markers=(), requires=None, paths=None):
            def action(r):
                env = environment(r)
                env.update(variables)
                stage = r.state['native_stage']
                cwd = stage
                exe = r.state[build.id]
                if paths:
                    # Explicit app/runtime/cwd layouts, independent per case.
                    layout = stage / name
                    for directory in ('app', 'runtime', 'cwd'):
                        (layout / directory).mkdir(parents=True)
                    exe = layout / 'app/query.exe'
                    shutil.copyfile(r.state[build.id], exe)
                    loader_dir, provider_dir = paths
                    shutil.copyfile(stage / 'amd_fidelityfx_loader_dx12.dll', layout / loader_dir / 'amd_fidelityfx_loader_dx12.dll')
                    if provider_dir:
                        shutil.copyfile(stage / 'amd_fidelityfx_upscaler_dx12.dll', layout / provider_dir / 'amd_fidelityfx_upscaler_dx12.dll')
                    env['FSR_SDK_TEST_DLL'] = layout / loader_dir / 'amd_fidelityfx_loader_dx12.dll'
                    cwd = layout / 'cwd'
                out, err, command = r.process([exe, '--ignored', '--exact', selector, '--nocapture', '--test-threads=1'],
                    cwd=cwd, env=env, observe=observe, markers=markers, timeout=60)
                if kind == 'allocation' and name == 'control' and not re.search(r'SUMMARY phase=after_create .*attempts=3\b', err):
                    raise Rejected('Bounded allocation protocol requires three observed callback allocations')
                if kind == 'input-lifetime' and name == 'retain' and not re.search(r'Create: return_code=[1-9][0-9]*;',err):
                    raise Rejected('Returned-error control did not produce a nonzero native error')
                r.note(execution_outcome=command['execution'], exit_code=command['exit_code'],
                       claim='raw observation; no native cleanup guarantee' if observe else 'required control')
            return Case(namespace + '.' + name, name + ' raw probe', action, requires or (build.id,), selector, 'native-experiment',
                {'evaluation':'observed' if observe else 'passed','exit':'any' if observe else 0,
                 'markers':list(markers),'environment':variables,'watchdog_seconds':60,'deployment_layout':paths})
        if kind == 'dimensions':
            cases.append(probe('control', 'signed_runtime_dimensions', {'FSR_SDK_DIMENSION_CASE': 'C0'}, markers=('1 passed; 0 failed',)))
            for name in ('Z1', 'Z2', 'Z3', 'Z4', 'T1', 'N1', 'E1', 'D1', 'B1', 'B2', 'B3'):
                cases.append(probe(name, 'signed_runtime_dimensions', {'FSR_SDK_DIMENSION_CASE': name}, observe=True,
                                   requires=(namespace + '.control',)))
        elif kind == 'allocation':
            cases.append(probe('control', 'probes_create_allocation_failure', {'FSR_SDK_FAIL_AT': '0'},
                               markers=('all_freed_once=true', 'Destroy: return_code=0;')))
            for index in range(1, 5):
                cases.append(probe(f'failure-{index}', 'probes_create_allocation_failure', {'FSR_SDK_FAIL_AT': str(index)},
                                   observe=index < 4, markers=() if index < 4 else ('all_freed_once=true',), requires=(namespace + '.control',)))
        elif kind == 'input-lifetime':
            for name in ('retain', 'protect', 'cleanup'):
                cases.append(probe(name, 'probes_create_error_input_lifetime', {'FSR_SDK_INPUT_LIFETIME': name},
                    observe=name != 'retain', markers=('Observation completed:',) if name == 'retain' else (),
                    requires=(namespace + '.lifecycle.explicit',) + ((namespace + '.retain',) if name != 'retain' else ())))
        elif kind == 'provider':
            for name, layout in (('a', ('app', 'app')), ('b', ('runtime', 'runtime')), ('c', ('runtime', 'app')), ('d', ('runtime', None))):
                cases.append(probe(name, 'queries_upscaler_version_count_without_context',
                    {'FSR_SDK_RESOLUTION_DIAGNOSTICS': '1', 'RUST_BACKTRACE': '0'}, observe=name in ('b', 'd'),
                    markers=('return_code=0, upscaler_version_count=',) if name in ('a', 'c') else (),
                    requires=(build.id,) if name == 'a' else (namespace + '.a',), paths=layout))
        else:
            cases.append(probe('sequential', 'sequential_shared_runtime_lifetime', {}, markers=('1 passed; 0 failed',)))
        return cases
    cases += regression_cases(namespace, sharpening=False, movable=False)
    prerequisite = tuple(namespace + '.' + name for name in ('dispatch.single', 'dispatch.temporal', 'dispatch.semantics'))
    if kind == 'gbv':
        selector = 'upscaler::dx12::native_dispatch::native_dispatch_readback'
        cases.append(child_case(namespace, 'control', selector, 'DX12 control readback:', prerequisite,
            variables={'FSR_SDK_TEST_GBV': '1', 'FSR_SDK_TEST_GPU_CONTROL': '1'}, timeout=45))
        cases.append(child_case(namespace, 'dispatch', selector, None, (namespace + '.control',),
            variables={'FSR_SDK_TEST_GBV': '1'}, observe=True, timeout=90))
    elif kind == 'recording':
        for mode, counts in (('same', (1, 2, 5, 8)), ('sibling', (2, 5))):
            for count in counts:
                cases.append(child_case(namespace, f'{mode}-{count}', 'upscaler::dx12::recording_ahead::native_recording_ahead_spike',
                    'GPU readback', prerequisite, variables={'FSR_SDK_TEST_RECORD_MODE': mode, 'FSR_SDK_TEST_RECORD_COUNT': str(count)}))
    elif kind == 'sharpening':
        for repeat in range(1, 4):
            cases.append(child_case(namespace, f'r{repeat}', 'upscaler::dx12::native_dispatch::native_sharpening_comparisons',
                'Sharpening gate: PASSED', prerequisite, output='FSR_SDK_TEST_SHARPENING_OUTPUT'))
        def audit(r):
            from analysis import audit_sharpening
            result=audit_sharpening(r.directory,namespace)
            target=r.directory/'analysis'
            target.mkdir(exist_ok=True)
            save(target/'original-sharpening.json',result)
            r.note(outputs=len(result['measurements']),original_criteria=result['criteria'])
        # The auditor runs even if the original image-effect child gate failed.
        cases.append(Case(namespace+'.audit','Independent original image-effect metrics, including failed children',audit,prerequisite,scope='offline-analysis'))
    elif kind in ('transfer', 'transfer-cleanup'):
        definitions = json.loads((HERE / 'native-cases.json').read_text(encoding='utf-8'))
        for item in definitions:
            if kind in item['groups']:
                cases.append(child_case(namespace, item['legacy_name'], item['selector'], item['marker'], prerequisite))
        if kind == 'transfer-cleanup':
            baseline = tuple(case.id for case in cases if case.scope == 'native-gpu')
            matrix = [
                ('create-after-transfer', 'same_thread_runtime_transfer_before_context_creation', 'runtime_transfer_before_context_creation_on_another_thread', 1),
                ('history', 'same_thread_continuous_history_across_ownership_handoffs', 'continuous_history_across_worker_thread_transfers', 6),
                ('independent-siblings', 'same_thread_independent_siblings_share_cleanup_coordination', 'independent_siblings_share_cleanup_coordination_across_threads', 5),
                ('unwind', 'same_thread_automatic_context_cleanup_during_unwind', 'transferred_context_automatic_cleanup_during_unwind', 1),
                ('sibling-unwind', 'same_thread_sibling_cleanup_during_unwind_preserves_survivor', 'transferred_sibling_cleanup_during_unwind_preserves_survivor', 3)]
            for number, (label, control, transfer, count) in enumerate(matrix, 1):
                for arm, selector in (('control', control), ('transfer', transfer)):
                    for repeat in range(1, 4):
                        cases.append(child_case(namespace, f'{label}-{arm}-r{repeat}',
                            'upscaler::dx12::native_dispatch::thread_transfers::' + selector,
                            f'case={number} transfer={str(arm == "transfer").lower()}: PASSED', baseline,
                            output='FSR_SDK_TEST_TRANSFER_OUTPUT', payloads=count, size=1843200,
                            variables={'FSR_SDK_TEST_CONTEXT_TRACE': '1'}))
            def compare(r):
                from analysis import audit_transfer
                trace=audit_transfer(r.directory,namespace,matrix)
                comparisons = []
                for label, *_ in matrix:
                    arms = {}
                    for arm in ('control', 'transfer'):
                        arms[arm] = [[{'file': path.name, 'sha256': sha(path)} for path in sorted(
                            (r.directory / 'readbacks' / f'{namespace}.{label}-{arm}-r{repeat}').glob('*.rgba16f'))] for repeat in range(1, 4)]
                    stable = all(payload == arms['control'][0] for payload in arms['control'])
                    comparisons.append({'case': label, 'controls_repeatable': stable,
                        'transfers_repeatable': all(payload == arms['transfer'][0] for payload in arms['transfer']),
                        'all_match_control': all(payload == arms['control'][0] for payload in arms['transfer']),
                        'diagnostic_only': True})
                target=r.directory/'analysis'
                target.mkdir(exist_ok=True)
                save(target/'transfer-cleanup.json',dict(trace,payload_comparisons=comparisons))
                r.note(payload_comparisons=comparisons,cleanup=trace, provider_termination='not established')
            cases.append(Case(namespace + '.payload-comparison', 'Selected control/transfer payload comparison', compare,
                tuple(case.id for case in cases if '.r' in case.id or re.search(r'-r[123]$', case.id)), scope='offline-analysis'))
    return cases
