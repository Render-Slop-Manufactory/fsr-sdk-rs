# SPDX-License-Identifier: MPL-2.0
"""Consumer-visible contracts, shared by source and registry-artefact profiles."""
import json
import os
import tomllib
from pathlib import Path

from execution import Case, Rejected
from records import save

HERE = Path(__file__).resolve().parent


def registry_provenance(data,cargo_home):
    members=set(data['workspace_members'])
    if len(members)!=1:
        raise Rejected('Expected exactly one downstream workspace member')
    packages=[]
    for item in data['packages']:
        if item['id'] in members:
            continue
        if (not item['source'] or not item['source'].startswith('registry+') or
            not Path(item['manifest_path']).is_relative_to(Path(cargo_home)) or
            any(dep.get('path') for dep in item['dependencies'])):
            raise Rejected('Dependency escaped isolated registry: '+item['name'])
        packages.append({key:item[key] for key in ('name','version','source','manifest_path')})
    return packages

BOUNDARIES = {
    'runtime': 'fsr_sdk::runtime', 'callbacks': 'fsr_sdk::upscaler::dx12::interop', 'dx12-module': 'fsr_sdk::upscaler::dx12',
    'setup': 'fsr_sdk::upscaler::dx12::UpscalerSetup',
    'movable-setup': 'fsr_sdk::upscaler::dx12::MovableUpscalerSetup',
    'legacy-upscaler': 'fsr_sdk::upscaler::Upscaler',
    'exposure-input': 'fsr_sdk::upscaler::dx12::ExposureInput',
    'generator-resources': 'fsr_sdk::upscaler::dx12::ReactiveMaskResources',
    'upscale-builder': 'fsr_sdk::upscaler::dx12::UpscaleResourcesBuilder',
    'generator-builder': 'fsr_sdk::upscaler::dx12::ReactiveMaskResourcesBuilder',
    'dispatch-resources': 'fsr_sdk::upscaler::UpscaleResources',
    'legacy-movable': 'fsr_sdk::upscaler::MovableUpscaler',
    'raw-loader': 'fsr_sdk_sys::loader', 'raw-dx12': 'fsr_sdk_sys::dx12',
}
TREES = [
    ('fsr-sdk-sys', 'defaults', [], False),
    ('fsr-sdk-sys', 'no-defaults', ['--no-default-features'], False),
    ('fsr-sdk-sys', 'dx12', ['--no-default-features', '--features', 'dx12'], True),
    ('fsr-sdk', 'defaults', [], True),
    ('fsr-sdk', 'no-defaults', ['--no-default-features'], False),
    ('fsr-sdk', 'dx12', ['--no-default-features', '--features', 'dx12'], True),
    ('fsr-sdk', 'movable', ['--features', 'movable'], True),
    ('fsr-sdk', 'no-defaults-movable', ['--no-default-features', '--features', 'movable'], False),
    ('fsr-sdk', 'utils', ['--no-default-features', '--features', 'utils'], False),
    ('fsr-sdk', 'dx12-utils', ['--no-default-features', '--features', 'dx12,utils'], True),
    ('fsr-sdk', 'dx12-movable-utils', ['--no-default-features', '--features', 'dx12,movable,utils'], True),
]


def require_native_dependencies(output, *, loader, windows):
    for name, expected in (('libloading', loader), ('windows', windows)):
        present = any(line.startswith(name + ' v') for line in output.splitlines())
        if present != expected:
            raise Rejected(f'Unexpected production {name} dependency (expected={expected})')


def source_cases(windows):
    cases = json.loads((HERE / 'consumer-cases.json').read_text(encoding='utf-8'))
    cases = [case for case in cases if case['platform'] == 'any' or windows]
    features = ['', 'movable', 'utils', 'movable,utils'] + ([] if windows else ['dx12', 'dx12,movable', 'dx12,utils', 'dx12,movable,utils'])
    for feature in features:
        for name, imported in BOUNDARIES.items():
            cases.append({'id': f'native-disabled-{name}-{feature.replace(",", "-") or "none"}',
                'features': feature, 'diagnostic': 'E0432',
                'source': f'use {imported}; fn main() {{}}'})
    if windows:
        for name, ty, bound in (
            ('movable-runtime-not-sync', 'fsr_sdk::runtime::MovableRuntime', 'Sync'),
            ('movable-upscaler-not-sync', 'fsr_sdk::upscaler::MovableUpscaler', 'Sync'),
            ('local-runtime-not-send', 'fsr_sdk::runtime::Runtime', 'Send'),
            ('local-upscaler-not-send', 'fsr_sdk::upscaler::Upscaler', 'Send')):
            cases.append({'id': name, 'features': 'dx12,movable', 'diagnostic': 'E0277',
                'source': f'use {ty}; fn check<T: {bound}>() {{}} fn main() {{ check::<{ty.split("::")[-1]}>(); }}'})
    return cases


def consumer_profile(prefix='consumer', requires=(), registry=False, toolchain=None):
    setup_id = prefix + '.setup'
    def setup(r):
        directory = r.directory / prefix
        (directory / 'src').mkdir(parents=True)
        versions = r.state.get('versions', {})
        if registry:
            dependencies = '\n'.join(f'{name} = {{ version = "={versions[name]}", default-features = false }}'
                                     for name in ('fsr-sdk', 'fsr-sdk-sys'))
            env = r.state['registry_env']
            packages = r.state['packages']
        else:
            dependencies = '\n'.join(f'{name} = {{ path = {json.dumps((r.repository / "crates" / name).as_posix())}, default-features = false }}'
                                     for name in ('fsr-sdk', 'fsr-sdk-sys'))
            env = {}
            packages = r.repository / 'crates'
        manifest = ('# SPDX-License-Identifier: MPL-2.0\n[package]\nname = "verification-consumer"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n'
            '[features]\ndefault = []\ndx12 = ["fsr-sdk/dx12"]\nmovable = ["fsr-sdk/movable"]\nutils = ["fsr-sdk/utils"]\n[dependencies]\n' + dependencies + '\n')
        (directory / 'Cargo.toml').write_text(manifest, encoding='utf-8')
        (directory / 'src/main.rs').write_text('fn main() {}\n', encoding='utf-8')
        r.state[prefix] = {'directory': directory, 'env': env, 'packages': packages}
        r.process(r.cargo(['generate-lockfile', '--manifest-path', directory / 'Cargo.toml'], toolchain), env=env)
        r.note(manifest=manifest)
    cases = [Case(setup_id, 'Isolated downstream consumer with explicit dependency provenance', setup, requires)]
    for definition in source_cases(os.name == 'nt'):
        def compile_case(r, definition=definition):
            state = r.state[prefix]
            # Immutable, distinct targets prevent Cargo's timestamp freshness
            # checks from reusing an earlier snippet's successful compilation.
            binary = 'case-' + definition['id']
            source = state['directory'] / 'src/bin' / (binary + '.rs')
            source.parent.mkdir(exist_ok=True)
            with source.open('x', encoding='utf-8', newline='\n') as stream:
                stream.write(definition['source'] + '\n')
            args = ['check', '--manifest-path', state['directory'] / 'Cargo.toml', '--bin', binary]
            if definition['features']:
                args += ['--features', definition['features']]
            if definition.get('lint'):
                args += ['--message-format=json']
            r.note(source=definition['source'], binary=binary, features=definition['features'],
                expected_diagnostic=definition['diagnostic'], expected_lint=definition.get('lint'))
            r.process(r.cargo(args, toolchain), env=state['env'], diagnostic=definition['diagnostic'], lint=definition.get('lint'))
        cases.append(Case(prefix + '.' + definition['id'], 'Consumer ' + definition['id'], compile_case, (setup_id,),
            expectation={'exit':'nonzero' if definition['diagnostic'] or definition.get('lint') else 0,
                'diagnostic':definition['diagnostic'], 'lint':definition.get('lint'), 'features':definition['features']}))
    for target in ('x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'):
        for feature in ('', 'movable', 'dx12', 'dx12,movable', 'utils', 'dx12,utils', 'dx12,movable,utils'):
            def tree(r, target=target, feature=feature):
                state = r.state[prefix]
                args = ['tree', '--manifest-path', state['directory'] / 'Cargo.toml', '--target', target, '--edges', 'normal', '--prefix', 'none']
                if feature:
                    args += ['--features', feature]
                output, _, _ = r.process(r.cargo(args, toolchain), env=state['env'])
                expected = target.endswith('windows-msvc') and 'dx12' in feature.split(',')
                require_native_dependencies(output, loader=expected, windows=expected)
                r.note(target=target, expected_loader=expected, expected_windows=expected)
            cases.append(Case(f'{prefix}.tree.consumer.{target}.{feature.replace(",", "-") or "none"}',
                'Consumer normal dependency graph', tree, (setup_id,)))
        for package, config, flags, windows_loader in TREES:
            def package_tree(r, target=target, package=package, flags=flags, windows_loader=windows_loader):
                state = r.state[prefix]
                directory = state['packages'] / (f'{package}-{r.state["versions"][package]}' if registry else package)
                args = ['tree', '--manifest-path', directory / 'Cargo.toml', '-p', package, '--target', target, '--edges', 'normal', '--prefix', 'none'] + flags
                output, _, _ = r.process(r.cargo(args, toolchain), env=state['env'])
                expected = target.endswith('windows-msvc') and windows_loader
                expected_windows = expected and package == 'fsr-sdk'
                require_native_dependencies(output, loader=expected, windows=expected_windows)
                r.note(target=target, expected_loader=expected, expected_windows=expected_windows)
            cases.append(Case(f'{prefix}.tree.{package}.{target}.{config}', 'Real package defaults and feature dependency graph', package_tree, (setup_id,)))
    for package, features in (('fsr-sdk', 'dx12,movable,utils'), ('fsr-sdk-sys', 'dx12')):
        def supported_features(r, package=package, features=features):
            state = r.state[prefix]
            directory = state['packages'] / (f'{package}-{r.state["versions"][package]}' if registry else package)
            manifest = tomllib.loads((directory / 'Cargo.toml').read_text(encoding='utf-8'))
            expected = ['fsr-sdk-sys/vulkan'] if package == 'fsr-sdk' else []
            if manifest.get('features', {}).get('vulkan') != expected:
                raise Rejected('Reserved Vulkan feature contract changed')
            r.process(r.cargo(['check', '--manifest-path', directory / 'Cargo.toml',
                '-p', package, '--all-targets', '--no-default-features', '--features', features], toolchain), env=state['env'])
        supported_id = f'{prefix}.supported-features.{package}'
        cases.append(Case(supported_id,
            'Implemented package features compile; reserved Vulkan feature is retained', supported_features, (setup_id,)))
        for config, flags in (('vulkan', ['--no-default-features', '--features', 'vulkan']),
                              ('all-features', ['--all-features'])):
            def unsupported(r, package=package, flags=flags):
                state = r.state[prefix]
                directory = state['packages'] / (f'{package}-{r.state["versions"][package]}' if registry else package)
                output, errors, command = r.process(r.cargo(['check', '--manifest-path', directory / 'Cargo.toml',
                    '-p', package, '--all-targets'] + flags, toolchain), env=state['env'], observe=True)
                if (command['execution'] != 'completed' or command['exit_code'] == 0 or
                        'error: the `vulkan` feature is intentionally unsupported' not in output + errors):
                    raise Rejected('Unsupported Vulkan compile diagnostic absent')
                r.current['evaluation'] = 'passed'
            cases.append(Case(f'{prefix}.unsupported-{config}.{package}',
                'Reserved Vulkan feature produces its deliberate compile error', unsupported, (supported_id,),
                expectation={'exit': 'nonzero', 'diagnostic': 'the `vulkan` feature is intentionally unsupported',
                             'features': config}))
    def metadata(r):
        state = r.state[prefix]
        # Leave a valid source after negative controls.
        (state['directory'] / 'src/main.rs').write_text('fn main() {}\n', encoding='utf-8')
        output, _, _ = r.process(r.cargo(['metadata', '--manifest-path', state['directory'] / 'Cargo.toml', '--features', 'dx12,movable,utils', '--format-version=1'], toolchain), env=state['env'], name='metadata')
        data = json.loads(output)
        selected = [{key: item[key] for key in ('name', 'version', 'source', 'manifest_path')}
                    for item in data['packages'] if item['name'].startswith('fsr-sdk')]
        if registry:
            all_packages=registry_provenance(data,state['env']['CARGO_HOME'])
            r.note(all_registry_packages=all_packages)
            lock = tomllib.loads((state['directory'] / 'Cargo.lock').read_text(encoding='utf-8'))
            for item in lock['package']:
                if item['name'].startswith('fsr-sdk') and item['checksum'] != r.state['archive_hashes'][item['name']]:
                    raise Rejected('Consumer did not resolve the selected archive bytes')
        r.note(registry_only=registry, packages=selected)
    cases.append(Case(prefix + '.metadata', 'Source IDs and selected package provenance', metadata, (setup_id,)))
    return cases
