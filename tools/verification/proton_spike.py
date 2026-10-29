# SPDX-License-Identifier: MPL-2.0
"""Experimental Windows preparation/Linux execution split; no production profile."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import sys

sys.dont_write_bytecode = True
from execution import Blocked, Case, Runner
from records import fresh, inventory, save, sha, Sanitizer

ROOT = Path(__file__).resolve().parents[2]
PAYLOADS = ('native-fsr_sdk.exe', 'gpu_helper.dll',
            'amd_fidelityfx_loader_dx12.dll', 'amd_fidelityfx_upscaler_dx12.dll')


def check_sources(repository, inputs):
    """A previously verified executable must still match the current crate inputs."""
    expected = {item['path']: item['sha256'] for item in inputs['files']
                if item['path'].startswith('crates/') or item['path'] in ('Cargo.toml', 'Cargo.lock')}
    actual_names = {path.relative_to(repository).as_posix()
                    for path in (repository / 'crates').rglob('*')
                    if path.is_file() and not path.is_symlink()
                    and not any(part in ('target', '__pycache__', 'archive')
                                for part in path.relative_to(repository).parts)}
    if actual_names != {name for name in expected if name.startswith('crates/')}:
        raise ValueError('Crate input inventory changed; create a fresh native attempt')
    for name, digest in expected.items():
        if sha(repository / name) != digest:
            raise ValueError('Verified build input changed: ' + name)


def verify_bundle(bundle):
    manifest = json.loads((bundle / 'bundle.json').read_text(encoding='utf-8'))
    if manifest.get('schema_version') != 1:
        raise ValueError('Unsupported spike bundle schema')
    actual = [item for item in inventory(bundle) if item['path'] != 'bundle.json']
    if actual != manifest['files']:
        raise ValueError('Spike bundle has modified, missing or additional files')
    return manifest


def prepare(attempt, sdk_root, destination):
    # Repository-only imports: the portable execution bundle does not need them.
    from run import verify_run
    from sdk import verify_files
    summary = verify_run(attempt)
    if summary['profile'] != 'native-upscaler' or summary['status'] != 'passed':
        raise ValueError('A complete passed native-upscaler attempt is required')
    inputs = json.loads((attempt / 'inputs.json').read_text(encoding='utf-8'))
    check_sources(ROOT, inputs)
    verify_files(sdk_root)
    stage = attempt / 'native'
    for name in PAYLOADS[2:]:
        if sha(stage / name) != sha(sdk_root / 'Kits/FidelityFX/signedbin' / name):
            raise ValueError('Verified attempt and selected SDK differ: ' + name)
    if destination.exists() or destination.is_relative_to(attempt) or attempt.is_relative_to(destination):
        raise ValueError('Bundle destination must be new and separate from the attempt')
    destination.mkdir(parents=True, exist_ok=False)
    app = destination / 'app'
    app.mkdir()
    for name in PAYLOADS:
        shutil.copyfile(stage / name, app / name)
    tools = destination / 'tools/verification'
    tools.mkdir(parents=True)
    for name in ('proton_spike.py', 'execution.py', 'records.py'):
        shutil.copyfile(ROOT / 'tools/verification' / name, tools / name)
    notices = destination / 'notices'
    notices.mkdir()
    for name, source in (('LICENSE', ROOT / 'LICENSE'),
                         ('AMD-license.md', sdk_root / 'docs/license.md'),
                         ('AMD-3rdpartynotice.md', sdk_root / '3rdpartynotice.md')):
        shutil.copyfile(source, notices / name)
    sanitizer = Sanitizer(ROOT, attempt)
    cases = {}
    for case_id in ('equipment.toolchain', 'windows.environment', 'native.sdk',
                    'native.helper', 'native.build.fsr_sdk', 'native.queries.public',
                    'native.dispatch.single'):
        cases[case_id] = sanitizer.value(json.loads(
            (attempt / 'cases' / case_id / 'result.json').read_text(encoding='utf-8')))
    provenance = {'source_inputs': sanitizer.value(inputs), 'windows_attempt': summary,
                  'windows_cases': cases, 'sdk_provenance': sanitizer.value(json.loads(
                      (sdk_root / 'provenance.json').read_text(encoding='utf-8')))}
    save(destination / 'windows-provenance.json', provenance)
    (destination / 'README.txt').write_text(
        'Private experimental input; do not publish this AMD binary bundle.\n'
        'On Linux, Python 3.11+ can verify it without launching Proton:\n'
        'python3 tools/verification/proton_spike.py inspect --bundle .\n'
        'See the repository spike documentation before using the run command.\n'
        'Windows compilation is not Linux cross-compilation or Proton evidence.\n', encoding='utf-8')
    manifest = {'schema_version': 1, 'source_attempt_identity': summary['input_identity'],
                'build_host': 'Windows x64/MSVC',
                'scope': 'Public queries followed by unchanged Gate A; no Linux cross-build claim',
                'cases': [{'id': case_id, 'selector': cases[case_id]['selector'],
                           'markers': cases[case_id]['expectation']['markers']}
                          for case_id in ('native.queries.public', 'native.dispatch.single')],
                'files': inventory(destination)}
    save(destination / 'bundle.json', manifest)
    verify_bundle(destination)
    return destination


def run_bundle(options):
    if sys.platform != 'linux':
        raise ValueError('Execution requires Linux; Windows supports prepare/inspect only')
    if not options.native or not options.allow_launcher_network:
        raise ValueError('Use --native --allow-launcher-network explicitly: umu may fetch/update its runtime')
    manifest = verify_bundle(options.bundle)
    for output in (options.output_root, options.prefix_root):
        if output.is_relative_to(options.bundle) or options.bundle.is_relative_to(output):
            raise ValueError('Results and prefixes must stay outside the immutable bundle')
    if not options.umu.is_file() or not os.access(options.umu, os.X_OK):
        raise ValueError('Supply the installed executable umu-run path')
    if not (options.proton / 'proton').is_file():
        raise ValueError('Supply a concrete installed Proton directory; mutable labels are not accepted')
    directory = fresh(options.output_root, 'proton-spike')
    print('Attempt: ' + str(directory), flush=True)
    runner = Runner(ROOT, directory, options)
    stage = directory / 'native'
    shutil.copytree(options.bundle / 'app', stage)
    prefix = fresh(options.prefix_root, 'proton-spike')
    runner.state['external_inputs'] = [('bundle/' + item['path'], options.bundle / item['path'])
                                       for item in manifest['files']]
    runner.state['external_inputs'] += [('bundle/manifest', options.bundle / 'bundle.json'),
                                       ('launcher', options.umu), ('proton/launcher', options.proton / 'proton')]
    runner.env.update(GAMEID='umu-default', STORE='none', PROTONPATH=str(options.proton),
                      WINEPREFIX=str(prefix), PROTONFIXES_DISABLE='1',
                      PROTON_VERB='waitforexitandrun', UMU_LOG='1', UMU_RUNTIME_UPDATE='0',
                      PROTON_LOG='1', PROTON_LOG_DIR=str(directory))
    # Do not inherit bypasses, DLL overrides or compatibility settings from a game.
    for key in tuple(runner.env):
        if (key.startswith(('STEAM_COMPAT_', 'VKD3D_', 'DXVK_'))
                or key in ('WINEDLLOVERRIDES', 'UMU_NO_PROTON', 'UMU_NO_RUNTIME',
                           'RUNTIMEPATH', 'PROTON_USE_WINED3D')):
            runner.env.pop(key)

    def equipment(r):
        r.note(host=platform.platform(), python=sys.version,
               umu_label=options.umu_version, launcher_sha256=sha(options.umu),
               proton_label=options.proton_version, proton_launcher_sha256=sha(options.proton / 'proton'),
               prefix=prefix, bundle=manifest['source_attempt_identity'],
               validation='Unchanged Windows Gate A requires the DirectX debug layer',
               limitations='No Linux cross-build, signature-policy equivalence or full runtime integrity claim. '
                           'Operator version labels are descriptive; launcher logs must corroborate actual versions.')
        release = Path('/etc/os-release')
        if release.is_file():
            r.note(os_release=release.read_text())
        tool = shutil.which('vulkaninfo')
        if not tool:
            raise Blocked('vulkaninfo is required to record the actual GPU/driver before AMD execution')
        r.process([tool, '--summary'], timeout=30, name='vulkan')

    cases = [Case('proton.equipment', 'Linux Vulkan/GPU identity and declared local launcher', equipment)]
    for entry in manifest['cases']:
        case_id = 'proton.' + entry['id']
        previous = cases[-1].id

        def child(r, entry=entry):
            environment = {'FSR_SDK_TEST_DLL': 'amd_fidelityfx_loader_dx12.dll',
                           'FSR_SDK_TEST_PROVIDER_DLL': 'amd_fidelityfx_upscaler_dx12.dll',
                           'FSR_SDK_TEST_DEVICE_DLL': 'gpu_helper.dll', 'FSR_SDK_TEST_GPU_DLL': 'gpu_helper.dll'}
            if options.adapter_index is not None:
                environment['FSR_SDK_TEST_ADAPTER_INDEX'] = str(options.adapter_index)
            r.process([options.umu, stage / 'native-fsr_sdk.exe', '--ignored', '--exact',
                       entry['selector'], '--nocapture', '--test-threads=1'],
                      cwd=stage, env=environment, timeout=180, markers=entry['markers'])

        cases.append(Case(case_id, 'Experimental Proton execution of ' + entry['id'], child,
                          (previous,), entry['selector'], 'proton-gpu',
                          {'evaluation': 'passed', 'exit': 0, 'markers': entry['markers'],
                           'watchdog_seconds': 180, 'windows_debug_layer_required': True}))
    result = runner.execute(cases, 'proton-spike')
    print('Result: ' + str(directory / 'run.json'), flush=True)
    return result


def parser():
    cli = argparse.ArgumentParser(description=__doc__)
    commands = cli.add_subparsers(dest='command', required=True)
    build = commands.add_parser('prepare', help='Package an integrity-verified Windows native attempt')
    build.add_argument('--run', required=True, type=Path)
    build.add_argument('--sdk-root', required=True, type=Path, help='D017 version installation root, parent of Kits')
    build.add_argument('--output', required=True, type=Path)
    inspect = commands.add_parser('inspect', help='Check complete bundle identity without executing anything')
    inspect.add_argument('--bundle', required=True, type=Path)
    execute = commands.add_parser('run', help='Experimental Linux-only execution; existing local umu/Proton required')
    execute.add_argument('--bundle', required=True, type=Path)
    execute.add_argument('--umu', required=True, type=Path)
    execute.add_argument('--umu-version', required=True)
    execute.add_argument('--proton', required=True, type=Path)
    execute.add_argument('--proton-version', required=True)
    execute.add_argument('--native', action='store_true')
    execute.add_argument('--allow-launcher-network', action='store_true')
    execute.add_argument('--adapter-index', type=int)
    execute.add_argument('--output-root', type=Path, default=ROOT.parent / 'proton-spike-results/verification')
    execute.add_argument('--prefix-root', type=Path, default=ROOT.parent / 'proton-spike-results/prefixes')
    return cli


def main():
    options = parser().parse_args()
    for name in ('run', 'sdk_root', 'output', 'bundle', 'umu', 'proton', 'output_root', 'prefix_root'):
        if hasattr(options, name):
            setattr(options, name, getattr(options, name).expanduser().resolve())
    try:
        if options.command == 'prepare':
            if os.name != 'nt':
                raise ValueError('Initial preparation is Windows-only')
            print(prepare(options.run, options.sdk_root, options.output))
            return 0
        if options.command == 'inspect':
            manifest = verify_bundle(options.bundle)
            print(json.dumps({'integrity': 'passed', 'build_host': manifest['build_host'],
                              'scope': manifest['scope'], 'cases': manifest['cases']}, indent=2))
            return 0
        if options.adapter_index is not None and options.adapter_index < 0:
            raise ValueError('Adapter index must be nonnegative')
        return run_bundle(options)
    except (OSError, ValueError, KeyError) as error:
        print('Spike error: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
