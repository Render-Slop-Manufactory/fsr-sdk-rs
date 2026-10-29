# SPDX-License-Identifier: MPL-2.0
"""Bounded Linux cross-build/Proton proof using explicitly prepared local tools."""
import json
import os
from pathlib import Path, PureWindowsPath
import platform
import re
import shutil
import sys

from execution import Blocked, Case, Rejected
from native import DLL_HASHES
from records import fresh, sha

TARGET = 'x86_64-pc-windows-msvc'


def arguments(parser):
    parser.add_argument('--proton', help='Concrete installed Proton directory; no automatic Proton acquisition')
    parser.add_argument('--umu', default='umu-run', help='Installed umu-run executable')
    parser.add_argument('--cargo-xwin', default='cargo-xwin', help='Installed cargo-xwin executable')
    parser.add_argument('--vulkaninfo', default='vulkaninfo', help='Host-driver-compatible vulkaninfo executable')
    parser.add_argument('--xwin-cache', help='Explicitly prepared cargo-xwin cache with xwin/DONE')
    parser.add_argument('--allow-launcher-network', action='store_true',
                        help='Acknowledge that umu may contact upstream or acquire its runtime')
    parser.add_argument('--vulkan-validation-manifest',
                        help='Require Vulkan core/synchronization validation using this explicit layer manifest')


def tool(value, path=None):
    found = shutil.which(value, path=path)
    if not found:
        raise Blocked('Missing development tool: ' + value)
    return Path(found).resolve()


def clean_environment(environment):
    """Inherited game controls must not select a different execution path."""
    result = dict(environment)
    for key in tuple(result):
        if (key.startswith(('STEAM_COMPAT_', 'VKD3D_', 'DXVK_', 'FSR_SDK_', 'FSR4_', 'XWIN_', 'PROTON_', 'UMU_',
                            'VK_LAYER_', 'VK_LOADER_', 'VK_VALIDATION_', 'VVL_'))
                or key in ('WINEDLLOVERRIDES', 'UMU_NO_PROTON', 'UMU_NO_RUNTIME',
                           'VK_INSTANCE_LAYERS',
                           'RUNTIMEPATH', 'PROTON_USE_WINED3D', 'WINEPREFIX', 'PROTONPATH', 'DXIL_SPIRV_CONFIG',
                           'CARGO_BUILD_TARGET', 'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER')):
            result.pop(key)
    return result


def external_inputs(options):
    """Freeze concrete launcher/component and cross-toolchain inputs before cases."""
    inputs = []
    if options.vulkan_validation_manifest:
        from vulkan_validation import inputs as validation_inputs
        try:
            manifest, library = validation_inputs(options.vulkan_validation_manifest)
        except Blocked as error:
            raise ValueError(str(error)) from error
        inputs.extend([('vulkan-validation/manifest.json', manifest),
                       ('vulkan-validation/layer.so', library)])
    if options.proton:
        root = Path(options.proton).expanduser().resolve()
        for name in ('proton', 'version', 'toolmanifest.vdf', 'files/bin/wine',
                     'files/bin/wine64', 'files/bin/wineserver', 'contrib/amdxcffx64.dll',
                     'files/lib/wine/x86_64-windows/amdxc64.dll',
                     'files/lib/wine/x86_64-windows/amd_ags_x64.dll',
                     'files/lib/wine/x86_64-unix/amd_ags_x64.so'):
            path = root / name
            if path.is_file():
                inputs.append(('proton/' + name, path))
        for path in translators(root).values():
            inputs.append(('proton/' + path.relative_to(root).as_posix(), path))
    if options.xwin_cache:
        root = Path(options.xwin_cache).expanduser().resolve() / 'xwin'
        for path in sorted(root.rglob('*')):
            # xwin's case aliases are symlinks; hash canonical payloads once.
            if path.is_file() and not path.is_symlink():
                inputs.append(('cross-toolchain/' + path.relative_to(root).as_posix(), path))
    for name in ('cargo_xwin', 'umu', 'vulkaninfo'):
        found = shutil.which(getattr(options, name))
        if found:
            inputs.append(('tools/' + name, Path(found).resolve()))
    for name in ('clang-cl', 'lld-link'):
        found = shutil.which(name)
        if found:
            inputs.append(('tools/' + name, Path(found).resolve()))
    return inputs


def translators(root):
    """Identify both current and older Proton translator layouts explicitly."""
    result = {}
    for name, component in (('d3d12.dll', 'vkd3d-proton'), ('d3d12core.dll', 'vkd3d-proton'),
                            ('dxgi.dll', 'dxvk')):
        for directory in ('files/lib/wine/' + component + '/x86_64-windows',
                          'files/lib64/wine/' + component):
            path = root / directory / name
            if path.is_file():
                result[name] = path
                break
    return result


def wine_drives(prefix):
    """Resolve only the isolated prefix's actual DOS drive mappings."""
    return {path.name.casefold(): path.resolve(strict=True)
            for path in (prefix / 'dosdevices').iterdir()
            if re.fullmatch(r'[a-zA-Z]:', path.name) and path.is_symlink()}


def windows_path(path, drives):
    """Translate a local input/output using this prefix's real drive mappings."""
    path = Path(path).resolve()
    for drive, root in sorted(drives.items(), key=lambda item: len(item[1].parts), reverse=True):
        if path.is_relative_to(root):
            return str(PureWindowsPath(drive + '\\', *path.relative_to(root).parts))
    raise Blocked('No prefix drive maps the requested local path: ' + str(path))


def loaded_modules(logs, stage, drives=None):
    """Require Wine's observed native module paths to match the staged AMD bytes."""
    paths = re.findall(r'Loaded L"([^"]*amd_fidelityfx_[^"]*\.dll)"[^\n]*: native',
                       logs, flags=re.IGNORECASE)
    expected = ('Z:' + str(stage).replace('/', '\\')).casefold()
    observed = {}
    for path in paths:
        path = path.replace('\\\\', '\\')
        name = path.rsplit('\\', 1)[-1].casefold()
        if name not in DLL_HASHES:
            continue
        if drives is None:
            matches = path.rsplit('\\', 1)[0].casefold() == expected
        else:
            windows = PureWindowsPath(path)
            root = drives.get(windows.drive.casefold())
            matches = (windows.is_absolute() and root is not None
                       and root.joinpath(*windows.parts[1:]).parent.resolve() == stage.resolve())
        if not matches:
            raise Rejected('AMD module resolved outside the staged application directory: ' + path)
        observed[name] = {'loaded_path': path, 'sha256': sha(stage / name)}
    if set(observed) != set(DLL_HASHES):
        raise Rejected('Wine logs did not identify both loaded AMD module paths')
    return observed


def invoke(r, arguments, *, environment=None, markers=(), validation=False, timeout=180):
    """Proton redirects child output into its own log, not the launcher pipes."""
    log_directory = r.directory / 'cases' / r.current['id'] / f'proton-{len(r.current["commands"]):02d}'
    log_directory.mkdir()
    variables = dict(environment or {}, PROTON_LOG_DIR=log_directory)
    validation_log = None
    if validation and r.state.get('validation_manifest'):
        from vulkan_validation import environment as validation_environment
        controls, validation_log = validation_environment(r.state['validation_manifest'], log_directory)
        variables.update(controls)
    args = [r.state['umu']] + list(arguments)
    if r.state.get('offline_runtime'):
        from proton_runtime import isolated
        args = isolated(args, r.env)
    elif r.state.get('proton_containment'):
        from proton_runtime import contained
        args = contained(args, r.env)
    try:
        out, err, command = r.process(args, env=variables,
                                     cwd=r.state['native_stage'], timeout=timeout)
    finally:
        command = r.current['commands'][-1]
        r.current['observations'].setdefault('proton_commands', []).append({
            'logs': r.sanitizer.value(log_directory), 'required_markers': list(markers),
            'exit': command.get('exit_code'), 'execution': command['execution']})
        if validation_log:
            from vulkan_validation import observation
            captured = '\n'.join(path.read_text(errors='replace')
                                 for path in log_directory.glob('steam-*.log'))
            for key in ('stdout', 'stderr'):
                captured += '\n' + (r.directory / command[key]).read_text(errors='replace')
            validation_log.write_text(captured)
            result = observation(validation_log)
            r.note(vulkan_validation=r.sanitizer.value(result))
    logs = '\n'.join(path.read_text(errors='replace') for path in log_directory.glob('steam-*.log'))
    text = out + err + logs
    if r.state.get('offline_runtime') or r.state.get('proton_containment'):
        proofs = [line for line in out.splitlines() if line.startswith('PROCESS CONTAINMENT: ')]
        if len(proofs) != 1:
            raise Rejected('Proton launcher did not report its PID isolation proof')
        r.current['observations'].setdefault('process_containment', []).append(json.loads(proofs[0].split(': ', 1)[1]))
    if r.state.get('offline_runtime'):
        proofs = [line for line in out.splitlines() if line.startswith('OFFLINE NETWORK: ')]
        if len(proofs) != 1:
            raise Rejected('Offline launcher did not report its network isolation proof')
        r.current['observations'].setdefault('offline_network', []).append(json.loads(proofs[0].split(': ', 1)[1]))
    if validation_log:
        from vulkan_validation import require
        require(result)
    if not all(marker in text for marker in markers):
        raise Rejected('Proton child output did not contain every required marker')
    return text


def cases(regression=False, offline=False):
    def equipment(r):
        if sys.platform != 'linux':
            raise Blocked('Proton execution requires a Linux host')
        if not r.options.native:
            raise Blocked('Proton AMD execution requires explicit --native')
        if offline:
            from proton_runtime import verify
            if r.options.online or r.options.allow_launcher_network:
                raise Blocked('Offline profile forbids --online and --allow-launcher-network')
            if not r.options.runtime_manifest or not r.options.runtime_sha256:
                raise Blocked('Offline profile requires --runtime-manifest and --runtime-sha256')
            manifest, roots, identity = verify(r.options.runtime_manifest, r.options.runtime_sha256)
            if r.options.proton and Path(r.options.proton).resolve() != roots['proton']:
                raise Blocked('--proton differs from the prepared offline runtime')
            r.options.proton = str(roots['proton'])
            r.options.umu = manifest['umu']
            r.state['offline_runtime'] = True
            r.note(offline_runtime=identity, provenance=manifest['provenance'],
                   manifest_sha256=r.options.runtime_sha256,
                   runtime_versions=(roots['runtime'] / 'VERSIONS.txt').read_text(),
                   launcher_package=roots['launcher'])
        elif not r.options.allow_launcher_network:
            raise Blocked('Use --allow-launcher-network: umu may acquire/update its runtime')
        if (regression or offline) and not r.options.vulkan_validation_manifest:
            raise Blocked('Proton regression requires --vulkan-validation-manifest')
        if not r.options.proton or not r.options.xwin_cache or not r.options.sdk_root:
            raise Blocked('Supply --proton, --xwin-cache and a D017 --sdk-root')
        proton = Path(r.options.proton).expanduser().resolve()
        cache = Path(r.options.xwin_cache).expanduser().resolve()
        if not (proton / 'proton').is_file() or not (proton / 'version').is_file():
            raise Blocked('A concrete installed Proton distribution and version file are required')
        if not (cache / 'xwin/DONE').is_file():
            raise Blocked('Missing prepared xwin cache; run the explicit cargo-xwin cache bootstrap first')
        translator_files = translators(proton)
        if len(translator_files) != 3:
            raise Blocked('Cannot identify Proton D3D12/DXGI translator payloads')
        r.env = clean_environment(r.env)
        if offline:
            r.env.update(UMU_FOLDERS_PATH=str(roots['runtime'].parents[1]),
                         UMU_HTTP_RETRIES='0', UMU_HTTP_TIMEOUT='3', PYTHONDONTWRITEBYTECODE='1')
            for key in tuple(r.env):
                if key.lower() in ('http_proxy', 'https_proxy', 'all_proxy', 'no_proxy'):
                    r.env.pop(key)
        if r.options.vulkan_validation_manifest:
            from vulkan_validation import inputs as validation_inputs
            manifest, library = validation_inputs(r.options.vulkan_validation_manifest)
            r.state['validation_manifest'] = manifest
            r.note(vulkan_layer={'manifest': manifest, 'library': library,
                                'sha256': sha(library), 'gpu_assisted': False})
        r.state.update(proton=proton, xwin_cache=cache, translators=translator_files,
                       proton_containment=True,
                       umu=tool(r.options.umu, r.env.get('PATH')),
                       cargo_xwin=tool(r.options.cargo_xwin, r.env.get('PATH')),
                       clang=tool('clang-cl', r.env.get('PATH')),
                       linker=tool('lld-link', r.env.get('PATH')))
        for name in ('cargo_xwin', 'clang', 'linker'):
            out, _, _ = r.process([r.state[name], '--version'], name=name)
            r.note(**{name: {'version': out, 'sha256': sha(r.state[name])}})
        r.process([r.options.vulkaninfo, '--summary'], timeout=30, name='vulkan')
        r.note(host=platform.platform(), os_release=Path('/etc/os-release').read_text(),
               proton_version=(proton / 'version').read_text().strip(), target=TARGET,
               validation={'required': 'Pinned SDK, module paths, public queries, shared single-frame numerical checks, dispatch/fence/readback, no device removal and teardown',
                           'directx_debug_layer': 'Availability observed separately; unavailable count remains UINT64_MAX. This is not the Windows Gate A case.',
                           'gpu_based_validation': 'Not enabled; not claimed',
                           'vulkan_validation': ('Core and synchronization required in both GPU children'
                                                 if r.state.get('validation_manifest') else 'Not enabled; not claimed'),
                           'windows_signature_policy': 'Unavailable on Linux; hash pins are the explicit trust input'},
               acquisition=('Prepared checksum-selected local runtime; every umu launch has an isolated network namespace.'
                            if offline else 'Tools, Rust target, Microsoft cache and AMD SDK must already be prepared. '
                           'umu runtime acquisition is separately acknowledged; no full offline-runtime claim.'))

    def sdk(r):
        from sdk import verify_files
        root = Path(r.options.sdk_root).expanduser().resolve()
        pins = verify_files(root.parents[1])
        stage = r.directory / 'native'
        stage.mkdir()
        for name, expected in DLL_HASHES.items():
            source = root / 'signedbin' / name
            if sha(source) != expected:
                raise Rejected('Pinned AMD payload changed: ' + name)
            shutil.copyfile(source, stage / name)
        r.state.update(native_stage=stage, headers=root)
        # Capture Linux signature diagnostics independently of the frozen hash
        # trust input. An unavailable Windows chain policy never becomes a pass.
        signatures = []
        sign = shutil.which('osslsigncode', path=r.env.get('PATH'))
        for name in DLL_HASHES:
            if sign:
                out, err, command = r.process([sign, 'verify', '-in', stage / name],
                                              observe=True, timeout=30, name='signature')
                signatures.append({'file': name, 'method': 'osslsigncode verify',
                                   'exit': command['exit_code'], 'diagnostics': out + err,
                                   'windows_policy_equivalence': False})
            else:
                signatures.append({'file': name, 'status': 'not_run',
                                   'reason': 'Optional osslsigncode unavailable',
                                   'windows_policy_equivalence': False})
        r.current['evaluation'] = 'passed'  # This case requires hash integrity and observation, not chain-policy success.
        r.note(files=pins, signatures=signatures, trust_input='D017 pinned bytes; Windows strict trust not evaluated')

    def helper(r):
        root, stage = r.state['headers'], r.state['native_stage']
        cache = r.state['xwin_cache'] / 'xwin'
        includes = [cache / name for name in ('crt/include', 'sdk/include/ucrt',
                                              'sdk/include/um', 'sdk/include/shared', 'sdk/include/winrt')]
        libs = [cache / name for name in ('crt/lib/x86_64', 'sdk/lib/um/x86_64', 'sdk/lib/ucrt/x86_64')]
        flags = ['--target=' + TARGET, '/nologo', '/std:c++17', '/EHsc', '/W4', '/WX', '/MT', '/LD']
        for path in includes:
            flags.extend(['/imsvc', path])
        if r.state.get('artifact_native'):
            wrapper = r.state['packages']/f'fsr-sdk-{r.state["versions"]["fsr-sdk"]}'
            raw = r.state['packages']/f'fsr-sdk-sys-{r.state["versions"]["fsr-sdk-sys"]}'
        else:
            wrapper, raw = r.repository/'crates/fsr-sdk', r.repository/'crates/fsr-sdk-sys'
        flags += [f'/I{root / "api/include/dx12"}',
                  wrapper / 'tests/native/gpu.cpp',
                  raw / 'tests/context_lifecycle/device.cpp',
                  f'/Fo{stage}{os.sep}', '-fuse-ld=lld', '/link',
                  f'/OUT:{stage / "gpu_helper.dll"}', f'/IMPLIB:{stage / "gpu_helper.lib"}']
        flags += [f'/LIBPATH:{path}' for path in libs] + ['d3d12.lib', 'dxgi.lib']
        r.process([r.state['clang']] + flags, timeout=180)
        r.note(helper_sha256=sha(stage / 'gpu_helper.dll'),
               wrapper_source=wrapper, raw_source=raw, registry_artifact=bool(r.state.get('artifact_native')))

    def build(r):
        artifact = bool(r.state.get('artifact_native'))
        directory = r.state['packages'] if artifact else r.repository
        arguments = r.cargo(['test', '--manifest-path', directory/'Cargo.toml', '-p', 'fsr-sdk', '--lib', '--features', 'dx12,movable',
                             '--target', TARGET, '--no-run', '--message-format=json'])
        # Invoke the tool as a Cargo subcommand without relying on ~/.cargo/bin
        # being inherited by a desktop application's PATH.
        arguments = [r.state['cargo_xwin'], 'test'] + arguments[arguments.index('test') + 1:]
        environment = (dict(r.state['registry_env']) if artifact else {}) | {'XWIN_CACHE_DIR': r.state['xwin_cache'], 'XWIN_ARCH': 'x86_64',
                       'XWIN_HTTP_RETRIES': '0',
                       'RUSTUP_TOOLCHAIN': r.options.toolchain or r.env.get('RUSTUP_TOOLCHAIN') or 'stable'}
        out, _, _ = r.process(arguments, cwd=directory, env=environment, timeout=600)
        artifacts = [json.loads(line) for line in out.splitlines() if line.startswith('{')]
        matching = [item for item in artifacts if item.get('reason') == 'compiler-artifact'
                    and item['target']['name'] == 'fsr_sdk' and item.get('executable')]
        if len(matching) != 1:
            raise Rejected('Cargo did not report exactly one Windows test executable')
        executable = r.state['native_stage'] / 'proton-fsr_sdk.exe'
        shutil.copyfile(matching[0]['executable'], executable)
        r.state['executable'] = executable
        r.note(executable_sha256=sha(executable), manifest=directory/'Cargo.toml',
               registry_artifact=artifact, archive_hashes=r.state.get('archive_hashes') if artifact else None)

    def launch(r):
        prefix = fresh(r.repository / 'target/proton-prefixes', 'proton')
        r.state['prefix'] = prefix
        r.env.update(GAMEID='umu-default', STORE='none', PROTONPATH=str(r.state['proton']),
                     WINEPREFIX=str(prefix), PROTONFIXES_DISABLE='1', UMU_LOG='info',
                     PROTON_VERB='waitforexitandrun', UMU_RUNTIME_UPDATE='0',
                     PROTON_LOG='1', PROTON_LOG_DIR=str(r.directory), WINEDEBUG='+loaddll')
        # Normal launch prepares the fresh prefix and Proton's DX12 DLLs. Its
        # Steam shim detaches a console, so it is not our test-output oracle.
        invoke(r, [r.state['executable'], '--list'])
        copied = {}
        for name, source in r.state['translators'].items():
            deployed = prefix / 'drive_c/windows/system32' / name
            if not deployed.is_file() or sha(deployed) != sha(source):
                raise Rejected('Prefix translator does not match selected Proton payload: ' + name)
            copied[name] = {'source': source, 'prefix_sha256': sha(deployed)}
        r.env['PROTON_VERB'] = 'runinprefix'
        # runinprefix deliberately skips Proton's setup_prefix(), which also
        # selects these native translator DLLs. Restore that exact selection
        # explicitly rather than accidentally using Wine's builtin D3D12.
        r.env['WINEDLLOVERRIDES'] = 'd3d12=n;d3d12core=n;dxgi=n'
        invoke(r, [r.state['executable'], '--list'],
               markers=('upscaler::dx12::tests::queries::native_public_queries: test',))
        r.note(prefix=prefix, translator_overrides=r.env['WINEDLLOVERRIDES'], translators=copied,
               claim='Windows Rust executable launched; no AMD calls in --list')

    definitions = {item['legacy_name']: item for item in json.loads(
        (Path(__file__).parent / 'native-cases.json').read_text())}
    result = [Case('proton.equipment', 'Linux Vulkan identity and prepared cross/launch tools', equipment),
              Case('proton.sdk', 'D017 pinned deployment and separate signature observations', sdk,
                   ('proton.equipment',)),
              Case('proton.helper', 'Linux-hosted MSVC ABI C++ helper build', helper,
                   ('proton.sdk',), scope='linux-cross-compilation'),
              Case('proton.build', 'Linux-hosted Windows Rust test build', build,
                   ('proton.sdk',), scope='linux-cross-compilation'),
              Case('proton.launch', 'Proton starts the exact Windows test executable', launch,
                   ('proton.helper', 'proton.build'))]
    for legacy, name in (('public-queries', 'queries'), ('gate-a', 'dispatch')):
        definition = dict(definitions[legacy])
        if name == 'dispatch':
            definition.update(selector='upscaler::dx12::native_dispatch::proton_dispatch_readback',
                              marker='Proton single-frame: PASSED')

        def child(r, definition=definition):
            environment = {'FSR_SDK_TEST_DLL': 'amd_fidelityfx_loader_dx12.dll',
                           'FSR_SDK_TEST_PROVIDER_DLL': 'amd_fidelityfx_upscaler_dx12.dll',
                           'FSR_SDK_TEST_DEVICE_DLL': 'gpu_helper.dll', 'FSR_SDK_TEST_GPU_DLL': 'gpu_helper.dll'}
            if r.options.adapter_index is not None:
                environment['FSR_SDK_TEST_ADAPTER_INDEX'] = str(r.options.adapter_index)
            text = invoke(r, [r.state['executable'], '--ignored', '--exact', definition['selector'],
                              '--nocapture', '--test-threads=1'], environment=environment,
                          markers=(definition['marker'], '1 passed; 0 failed'), validation=True)
            drives = wine_drives(r.state['prefix'])
            r.note(loaded_modules=loaded_modules(text, r.state['native_stage'], drives),
                   wine_drives=drives)
            r.note(adapter_provider_metrics=[line for line in text.splitlines()
                                            if line.startswith(('Adapter:', 'Public queries:', 'Local queries:', 'Movable queries:', 'Upscaler provider:', 'Selected-device',
                                                                'Proton ', 'Gate A ', 'GPU readback:'))])
        result.append(Case('proton.' + name, 'Shared ' + legacy + ' workload under explicit Proton validation contract', child,
                           (result[-1].id,), definition['selector'], 'proton-gpu',
                           {'evaluation': 'passed', 'exit': 0,
                            'markers': [definition['marker'], '1 passed; 0 failed'],
                            'watchdog_seconds': 180, 'windows_case_unchanged': name == 'queries',
                            'directx_diagnostics_required': False}))
    return result
