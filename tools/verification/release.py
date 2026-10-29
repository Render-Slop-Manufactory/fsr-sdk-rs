# SPDX-License-Identifier: MPL-2.0
"""Exact Cargo archives, isolated registry consumers and packaged test inputs."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import shutil
import tarfile
import tomllib

from consumers import consumer_profile
from execution import Blocked, Case, Rejected
from records import sha


def index_path(name):
    if len(name) == 1:
        return Path('1') / name
    if len(name) == 2:
        return Path('2') / name
    if len(name) == 3:
        return Path('3') / name[0] / name
    return Path(name[:2]) / name[2:4] / name


def archive_files(archive, prefix):
    """Read before extracting; reject both POSIX and Windows escape spellings."""
    files = {}
    folded = set()
    with tarfile.open(archive) as stream:
        for member in stream.getmembers():
            path, windows = PurePosixPath(member.name), PureWindowsPath(member.name)
            if (not member.isfile() or path.is_absolute() or windows.drive or windows.root or
                '\\' in member.name or ':' in member.name or '..' in path.parts or
                not path.parts or path.parts[0] != prefix or len(path.parts) < 2 or
                member.name != path.as_posix()):
                raise Rejected('Unsafe or non-file archive member: ' + member.name)
            inner = PurePosixPath(*path.parts[1:]).as_posix()
            if inner.casefold() in folded:
                raise Rejected('Duplicate archive member: ' + member.name)
            folded.add(inner.casefold())
            for part in path.parts:
                if (part.rstrip(' .') != part or part.split('.')[0].upper() in
                    {'CON','PRN','AUX','NUL',*(f'COM{i}' for i in range(1,10)),*(f'LPT{i}' for i in range(1,10))}):
                    raise Rejected('Nonportable Windows archive member: '+member.name)
            if member.size > 64 * 1024 * 1024:
                raise Rejected('Unexpectedly large package member')
            files[inner] = stream.extractfile(member).read()
    if not files:
        raise Rejected('Empty package archive')
    return files


def registry(r):
    versions = {name:tomllib.loads((r.repository / 'crates' / name / 'Cargo.toml').read_text())['package']['version']
                for name in ('fsr-sdk','fsr-sdk-sys')}
    archives = r.state['archives']
    isolation = r.directory / 'isolation'
    packages, registry_dir, cargo_home = isolation / 'packages', isolation / 'registry', isolation / 'cargo-home'
    for path in (packages, registry_dir / 'index', cargo_home):
        path.mkdir(parents=True)
    (packages / 'Cargo.toml').write_text('[workspace]\nresolver = "3"\nmembers = '+json.dumps([f'{name}-{version}' for name,version in versions.items()])+'\n')
    cache = Path(r.env.get('CARGO_HOME', str(Path.home()/'.cargo')))
    lock = tomllib.loads((r.repository/'Cargo.lock').read_text())
    inventories, hashes, registry_inputs = [], {}, []
    for item in lock['package']:
        name, version = item['name'], item['version']
        filename = f'{name}-{version}.crate'
        if name in versions:
            if version != versions[name]:
                raise Rejected('Workspace lock/version mismatch')
            archive = archives[name]
            files = archive_files(archive, f'{name}-{version}')
            manifest = tomllib.loads(files['Cargo.toml'].decode())
            if manifest['package']['name'] != name or manifest['package']['version'] != version:
                raise Rejected('Archive package identity mismatch')
            if any('path' in dep for dep in manifest.get('dependencies',{}).values() if isinstance(dep,dict)):
                raise Rejected('Normalized archive retains path dependency')
            if files.get('LICENSE') != (r.repository/'LICENSE').read_bytes():
                raise Rejected('Project licence missing or altered')
            if name == 'fsr-sdk-sys' and files.get('LICENSE-AMD') != (r.repository/'crates/fsr-sdk-sys/LICENSE-AMD').read_bytes():
                raise Rejected('AMD licence missing or altered')
            if name == 'fsr-sdk':
                if 'movable' not in manifest['features'] or manifest['dependencies']['fsr-sdk-sys']['version'] != versions['fsr-sdk-sys']:
                    raise Rejected('Packaged wrapper feature/dependency contract changed')
                if manifest['package']['metadata']['docs']['rs']['features'] != ['movable', 'utils']:
                    raise Rejected('Packaged documentation feature contract changed')
            inventory = []
            for inner, content in files.items():
                path = Path(inner)
                if path.suffix.lower() in ('.dll','.exe','.pdb','.obj','.lib','.ps1') or any(part in ('external','target','.git','.cargo','.aws') for part in path.parts):
                    raise Rejected('Unwanted deployment/build material: '+inner)
                if inner not in ('Cargo.toml','Cargo.toml.orig','Cargo.lock','.cargo_vcs_info.json'):
                    source = r.repository/'crates'/name/inner
                    if not source.is_file() or source.read_bytes() != content:
                        raise Rejected('Packaged source differs from selected checkout: '+inner)
                destination = packages/f'{name}-{version}'/inner
                destination.parent.mkdir(parents=True,exist_ok=True)
                destination.write_bytes(content)
                inventory.append({'file':inner,'bytes':len(content),'sha256':hashlib.sha256(content).hexdigest()})
            index_source = r.state['package_dir']/'tmp-registry/index'/index_path(name)
            if index_source.is_file() and not r.options.wrapper_archive:
                index = json.loads(index_source.read_text())
            else:
                # Cargo's normalized manifest determines the local index entry.
                dependencies = []
                groups = [(None, manifest)] + list(manifest.get('target',{}).items())
                for target, group in groups:
                    for kind,key in (('normal','dependencies'),('dev','dev-dependencies'),('build','build-dependencies')):
                        for dep, detail in group.get(key,{}).items():
                            detail = {'version':detail} if isinstance(detail,str) else detail
                            dependencies.append({'name':dep,'req':detail['version'],'features':detail.get('features',[]),
                                'optional':detail.get('optional',False),'default_features':detail.get('default-features',True),
                                'target':target,'kind':kind,'registry':None,'package':detail.get('package')})
                index = {'name':name,'vers':version,'deps':dependencies,'cksum':sha(archive),'features':{},
                         'features2':manifest.get('features',{}),'yanked':False,'v':2,'rust_version':manifest['package']['rust-version']}
            hashes[name] = sha(archive)
            inventories.append({'archive':filename,'sha256':hashes[name],'files':inventory})
        else:
            candidates = list((cache/'registry/cache').glob('*/'+filename))
            candidates = [path for path in candidates if sha(path)==item['checksum']]
            if not candidates:
                raise Blocked('Dependency cache lacks checksum-verified '+filename+'; populate it with cargo fetch first')
            archive = candidates[0]
            indexes = list((cache/'registry/index').glob('*/.cache/'+index_path(name).as_posix()))
            entries = [json.loads(piece) for path in indexes for piece in path.read_bytes().split(b'\0') if piece.startswith(b'{')]
            index = next((entry for entry in entries if entry['vers']==version and entry['cksum']==sha(archive)),None)
            if not index:
                raise Blocked('Dependency cache lacks verified index entry: '+filename)
        if index['cksum'] != sha(archive):
            raise Rejected('Registry index checksum does not identify selected archive')
        shutil.copyfile(archive,registry_dir/filename)
        destination = registry_dir/'index'/index_path(name)
        destination.parent.mkdir(parents=True,exist_ok=True)
        destination.write_text(json.dumps(index,separators=(',',':'))+'\n')
        registry_inputs.append({'archive':filename,'sha256':sha(archive)})
    (cargo_home/'config.toml').write_text('[source.crates-io]\nreplace-with = "verification-artifacts"\n[source.verification-artifacts]\nlocal-registry = '+json.dumps(registry_dir.as_posix())+'\n')
    env = {'CARGO_HOME':str(cargo_home),'CARGO_TARGET_DIR':str(r.directory/'artifact-build')}
    r.state.update(packages=packages,registry_env=env,versions=versions,archive_hashes=hashes)
    r.process(r.cargo(['generate-lockfile','--manifest-path',packages/'Cargo.toml']),env=env)
    r.note(archives=inventories,registry_inputs=registry_inputs,unchanged_extracted_bytes=True)


def package_cases():
    def package(r):
        directory = r.directory/'package-build'
        r.state['package_dir'] = directory/'package'
        if ((r.options.wrapper_archive is not None or r.options.sys_archive is not None) and
                (not r.options.wrapper_archive or not r.options.sys_archive)):
            raise Rejected('Supply both selected package archives together')
        if r.options.wrapper_archive:
            r.state['archives'] = {'fsr-sdk':Path(r.options.wrapper_archive).resolve(),'fsr-sdk-sys':Path(r.options.sys_archive).resolve()}
        else:
            r.process(r.cargo(['package','--workspace','--allow-dirty']),env={'CARGO_TARGET_DIR':directory})
            r.state['archives'] = {name:directory/'package'/f'{name}-{tomllib.loads((r.repository/"crates"/name/"Cargo.toml").read_text())["package"]["version"]}.crate'
                                  for name in ('fsr-sdk','fsr-sdk-sys')}
        r.note(archives=[{'file':path.name,'sha256':sha(path)} for path in r.state['archives'].values()],published=False)
    cases = [Case('release.package','Create or select the exact pair of Cargo archives',package),
             Case('release.registry','Validate every archive byte and construct an isolated local registry',registry,('release.package',))]
    return cases


def registry_example_cases(full_contracts=True):
    from examples import example_cases
    cases = []
    for prefix, toolchain in (('release.consumer', None), ('release.consumer-msrv', '1.88.0')):
        consumers = consumer_profile(prefix, ('release.registry',), registry=True, toolchain=toolchain)
        if not full_contracts:
            consumers = [case for case in consumers if case.id in (prefix+'.setup', prefix+'.metadata')]
        cases += consumers + example_cases(prefix, toolchain, registry=True)
    return cases


def archive_integrity_case(requires):
    def integrity(r):
        verify_archive_integrity(r)
    return Case('release.integrity', 'Recheck selected archives and extracted bytes after all checks', integrity, requires)


def verify_archive_integrity(r):
    for name, archive in r.state['archives'].items():
        if sha(archive) != r.state['archive_hashes'][name]:
            raise Rejected('Selected archive was mutated')
        files = archive_files(archive, f'{name}-{r.state["versions"][name]}')
        directory = r.state['packages']/f'{name}-{r.state["versions"][name]}'
        paths = list(directory.rglob('*'))
        if (any(path.is_symlink() for path in paths) or
                {path.relative_to(directory).as_posix() for path in paths if path.is_file()} != set(files) or
                any((directory/inner).read_bytes() != content for inner, content in files.items())):
            raise Rejected('Extracted archive bytes/inventory changed during checks')
    r.note(selected_archives_unchanged=True, extracted_bytes_unchanged=True)


def release_cases():
    cases = package_cases()
    for toolchain in (None,'1.88.0'):
        for config, flags in (('default',[]),('movable',['--features','fsr-sdk/movable']),
                              ('no-default',['--no-default-features']),('no-default-movable',['--no-default-features','--features','fsr-sdk/movable']),
                              ('utils',['--features','fsr-sdk/utils']),('movable-utils',['--features','fsr-sdk/movable,fsr-sdk/utils']),
                              ('no-default-utils',['--no-default-features','--features','fsr-sdk/utils'])):
            for command in ('check','test','clippy','doc') if toolchain is None else ('check','test'):
                def check(r, command=command,flags=flags,toolchain=toolchain):
                    args = [command,'--manifest-path',r.state['packages']/'Cargo.toml','--workspace']+flags
                    if command in ('check','clippy'): args += ['--all-targets']
                    if command == 'doc': args += ['--no-deps']
                    if command == 'clippy': args += ['--','-D','warnings']
                    r.process(r.cargo(args,toolchain),env=r.state['registry_env'])
                cases.append(Case(f'release.artifact.{toolchain or "current"}.{config}.{command}',
                                  'Exact extracted package '+command+' ('+config+')',check,('release.registry',)))
    cases += registry_example_cases()
    cases.append(archive_integrity_case(tuple(case.id for case in cases)))
    return cases


def bind_native_archives(cases):
    """Bind supported native/Proton builders to a verified exact package pair."""
    first = next((i for i, case in enumerate(cases)
                  if case.id in ('windows.environment', 'proton.equipment')), None)
    if first is None:
        raise Rejected('Archive-native mode requires a Windows or Proton native profile')
    prerequisite = cases[first].id
    packages = package_cases()
    packages[0].requires += (prerequisite,)

    def bind(r):
        r.state['artifact_native'] = True
        r.note(archive_hashes=r.state['archive_hashes'], registry_artifact=True)

    gate = Case('artifact.bind', 'Use verified extracted packages for native builds', bind, ('release.registry',))
    for case in cases[first+1:]:
        case.requires += (gate.id,)
    result = cases[:first+1]+packages+[gate]+cases[first+1:]
    result.append(archive_integrity_case(tuple(case.id for case in result)))
    return result


def coverage_cases():
    from native import native_setup, regression_cases
    from public_callbacks import cases as callback_cases
    cases = native_setup()+package_cases()+registry_example_cases(full_contracts=False)
    cases += regression_cases('artifact', registry=True)
    cases += callback_cases(namespace='artifact-callbacks', registry=True, include_setup=False)
    cases.append(archive_integrity_case(tuple(case.id for case in cases)))
    return cases


def dry_run_case():
    def simulate(r):
        if not r.options.online:
            raise Blocked('Cargo publication simulation requires --online registry access; no uploads are permitted')
        target=r.directory/'dry-run-build'
        r.process(r.cargo(['publish','--workspace','--dry-run','--allow-dirty']),
                  env={'CARGO_TARGET_DIR':target},markers=('aborting upload due to dry run',))
        for name,version in r.state['versions'].items():
            filename=f'{name}-{version}.crate'
            generated=target/'package'/filename
            if not generated.is_file():
                generated=target/'package/tmp-crate'/filename
            if not generated.is_file():
                raise Rejected('Cargo did not retain the dry-run archive: '+name)
            if sha(generated)!=r.state['archive_hashes'][name]:
                raise Rejected('Dry-run package bytes differ from selected archive: '+name)
        r.note(uploaded=False,selected_archive_bytes_match=True)
    return Case('release.dry-run','Cargo publication simulation of selected archive bytes; no upload',simulate,
                ('release.integrity',),expectation={'exit':0,'required_marker':'aborting upload due to dry run','upload':False})
