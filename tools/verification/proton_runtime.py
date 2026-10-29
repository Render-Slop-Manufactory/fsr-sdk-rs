# SPDX-License-Identifier: MPL-2.0
"""Explicit local runtime import, integrity checks and contained Proton launch."""
import errno
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import stat
import sys
import time

from execution import Blocked, Case, Rejected
from records import save, sha

MAX_BYTES = 4_000_000_000
MAX_ENTRIES = 100_000
MAX_SECONDS = 300
ROLES = ('proton', 'runtime', 'launcher')


def tree(root, *, mutable=False, started=None):
    """Hash regular bytes, modes and literal symlinks, without following links."""
    root = Path(root)
    if root.is_symlink() or not root.is_dir():
        raise Blocked('Missing runtime directory: ' + str(root))
    entries, pending, total = [], [root], 0
    start = time.monotonic() if started is None else started
    while pending:
        for path in sorted(pending.pop().iterdir()):
            name = path.relative_to(root).as_posix()
            if mutable and name == 'var':
                continue  # pressure-vessel's generated cache, never distribution input
            info = path.lstat()
            item = dict(path=name, mode=stat.S_IMODE(info.st_mode))
            if stat.S_ISLNK(info.st_mode):
                item.update(kind='link', target=os.readlink(path))
            elif stat.S_ISDIR(info.st_mode):
                item['kind'] = 'directory'
                pending.append(path)
            elif stat.S_ISREG(info.st_mode):
                total += info.st_size
                if total > MAX_BYTES:
                    raise ValueError('Runtime exceeds the local import byte limit')
                item.update(kind='file', bytes=info.st_size, sha256=sha(path))
            else:
                raise ValueError('Unsupported runtime node: ' + name)
            entries.append(item)
            if len(entries) > MAX_ENTRIES or time.monotonic() - start > MAX_SECONDS:
                raise ValueError('Runtime exceeds the local import entry/time limit')
    return sorted(entries, key=lambda item: item['path'])


def summary(entries):
    return dict(entries=len(entries), bytes=sum(item.get('bytes', 0) for item in entries),
                sha256=hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest())


def prerequisites(proton, runtime, umu, launcher):
    for root, names in ((proton, ('proton', 'version', 'toolmanifest.vdf', 'files/bin/wine')),
                        (runtime, ('umu', '_v2-entry-point', 'VERSIONS.txt', '.installed.ok'))):
        for name in names:
            if not (root / name).is_file():
                raise Blocked('Missing prepared runtime input: ' + str(root / name))
    # The first offline policy deliberately covers the observed x86_64 sniper stack.
    if '"1628350"' not in (proton / 'toolmanifest.vdf').read_text():
        raise Blocked('Offline runtime policy requires Proton using steamrt3/sniper')
    if not umu.is_file() or not os.access(umu, os.X_OK) or not umu.resolve().is_relative_to(launcher):
        raise Blocked('umu executable must belong to the declared launcher directory')


def prepare(proton, runtime, umu, launcher, output, provenance):
    started = time.monotonic()
    sources = dict(proton=Path(proton).resolve(), runtime=Path(runtime).resolve(),
                   launcher=Path(launcher).resolve())
    umu, output = Path(umu).resolve(), Path(output).resolve()
    if not provenance.strip():
        raise ValueError('Local acquisition provenance is required')
    if any(output.is_relative_to(root) or root.is_relative_to(output) for root in sources.values()):
        raise ValueError('Import destination overlaps a source')
    prerequisites(sources['proton'], sources['runtime'], umu, sources['launcher'])
    shim = sources['runtime'].parent / 'umu-shim'
    if not shim.is_file():
        raise Blocked('Missing prepared umu-shim')
    shim_hash = sha(shim)
    before = {role: tree(root, mutable=role == 'runtime', started=started) for role, root in sources.items()}
    if sum(summary(items)['bytes'] for items in before.values()) > MAX_BYTES:
        raise ValueError('Combined runtime exceeds the local import byte limit')
    if sum(len(items) for items in before.values()) > MAX_ENTRIES:
        raise ValueError('Combined runtime exceeds the local import entry limit')
    output.mkdir(parents=True, exist_ok=False)
    def copy(source, destination):
        if time.monotonic() - started > MAX_SECONDS:
            raise ValueError('Runtime import exceeded its time limit')
        return shutil.copy2(source, destination)
    shutil.copytree(sources['proton'], output / 'proton', symlinks=True, copy_function=copy)
    deployed = output / 'data/umu/steamrt3'
    shutil.copytree(sources['runtime'], deployed, symlinks=True, copy_function=copy,
                    ignore=lambda directory, names: ['var'] if Path(directory) == sources['runtime'] else [])
    shutil.copy2(shim, deployed.parent / 'umu-shim')
    roots = dict(proton=output / 'proton', runtime=deployed, launcher=sources['launcher'])
    for role in ROLES:
        if tree(roots[role], mutable=role == 'runtime', started=started) != before[role]:
            raise ValueError('Local import changed bytes or metadata: ' + role)
    if any(tree(root, mutable=role == 'runtime', started=started) != before[role] for role, root in sources.items()):
        raise ValueError('Local import source changed during preparation')
    if sha(shim) != shim_hash or sha(deployed.parent / 'umu-shim') != shim_hash:
        raise ValueError('umu-shim changed during local import')
    manifest = dict(schema_version=1, policy='local-steamrt3-v1', provenance=provenance,
                    umu=str(umu), shim_sha256=shim_hash,
                    roots={role: dict(path=str(root), inventory=before[role]) for role, root in roots.items()})
    path = output / 'runtime.json'
    save(path, manifest)
    return dict(manifest=str(path), sha256=sha(path), components={k: summary(v) for k, v in before.items()})


def verify(path, checksum):
    path = Path(path)
    if not path.is_file():
        raise Blocked('Missing runtime manifest: ' + str(path))
    if sha(path) != checksum:
        raise Rejected('Runtime manifest checksum mismatch')
    manifest = json.loads(path.read_text())
    if manifest.get('schema_version') != 1 or manifest.get('policy') != 'local-steamrt3-v1':
        raise Rejected('Unknown offline runtime manifest policy')
    roots = {role: Path(manifest['roots'][role]['path']).resolve() for role in ROLES}
    umu = Path(manifest['umu']).resolve()
    prerequisites(roots['proton'], roots['runtime'], umu, roots['launcher'])
    if sha(roots['runtime'].parent / 'umu-shim') != manifest['shim_sha256']:
        raise Rejected('Prepared umu-shim changed')
    observations = {}
    for role in ROLES:
        actual = tree(roots[role], mutable=role == 'runtime')
        if actual != manifest['roots'][role]['inventory']:
            raise Rejected('Prepared runtime integrity mismatch: ' + role)
        observations[role] = summary(actual)
    return manifest, roots, observations


def contained(arguments, environment, *, offline=False):
    wrapper = shutil.which('bwrap', path=environment.get('PATH'))
    if not wrapper:
        raise Blocked('Contained Proton execution requires bubblewrap')
    command = [wrapper, '--unshare-user', '--unshare-pid', '--die-with-parent']
    if offline:
        command += ['--unshare-net']
    command += ['--bind', '/', '/', '--dev-bind', '/dev', '/dev', '--proc', '/proc', '--',
                sys.executable, Path(__file__).resolve()]
    if offline:
        command += ['--network-exec', os.readlink('/proc/self/ns/net')]
    else:
        command += ['--process-exec']
    return command + [os.readlink('/proc/self/ns/pid'), '--'] + list(arguments)


def isolated(arguments, environment):
    return contained(arguments, environment, offline=True)


def process_proof(parent):
    if os.readlink('/proc/self/ns/pid') == parent:
        raise Rejected('Proton PID namespace was not isolated')
    print('PROCESS CONTAINMENT: ' + json.dumps(dict(distinct_pid_namespace=True,
          parent_death_policy='--die-with-parent')), flush=True)


def network_proof(parent):
    current = os.readlink('/proc/self/ns/net')
    if current == parent:
        raise Rejected('Offline network namespace was not isolated')
    interfaces = socket.if_nameindex()
    if any(name != 'lo' for _, name in interfaces):
        raise Rejected('Offline network namespace was not isolated')
    errors = []
    for family, address in ((socket.AF_INET, ('192.0.2.1', 9)),
                             (socket.AF_INET6, ('2001:db8::1', 9))):
        with socket.socket(family, socket.SOCK_DGRAM) as connection:
            try:
                connection.connect(address)
            except OSError as error:
                if error.errno != errno.ENETUNREACH:
                    raise
                errors.append(error.errno)
            else:
                raise Rejected('Network route exists inside offline namespace')
    print('OFFLINE NETWORK: ' + json.dumps(dict(distinct_namespace=True,
          interfaces=[name for _, name in interfaces], unreachable_errno=errors)), flush=True)


def controls():
    def missing(r, role):
        directory = r.directory / 'cases' / r.current['id']
        manifest = Path(r.options.runtime_manifest)
        checksum = r.options.runtime_sha256
        if role == 'manifest':
            manifest = directory / 'absent.json'
        else:
            data = json.loads(manifest.read_text())
            empty = directory / 'empty-runtime'; empty.mkdir()
            data['roots'][role]['path'] = str(empty)
            manifest = directory / 'missing-input.json'; save(manifest, data)
            checksum = sha(manifest)
        args = [sys.executable, r.repository / 'tools/verification/run.py', 'runtime', 'verify',
                '--manifest', manifest, '--sha256', checksum]
        out, err, command = r.process(isolated(args, r.env), observe=True, timeout=90)
        if command['exit_code'] != 1 or 'Missing ' not in out + err:
            raise Rejected('Missing-input control did not reject before launch')
        if role != 'manifest' and any(empty.iterdir()):
            raise Rejected('Missing-input control acquired or repaired runtime files')
        r.note(missing=role, exit=command['exit_code'], runtime_acquired=False,
               network_proof=out.splitlines()[0])
        r.current['evaluation'] = 'passed'
    return [Case('proton.offline.missing-' + role, 'Missing ' + role + ' fails without downloads',
                 lambda r, role=role: missing(r, role), ('proton.equipment',))
            for role in ('manifest', 'proton', 'runtime')]


def integrity_case():
    def check(r):
        _, _, observed = verify(r.options.runtime_manifest, r.options.runtime_sha256)
        r.note(runtime=observed, runtime_unchanged=True)
    return Case('proton.offline.integrity', 'Prepared distribution bytes unchanged after offline GPU execution',
                check, ('proton.dispatch',))


if __name__ == '__main__':
    if sys.argv[1:2] == ['--network-exec'] and sys.argv[4:5] == ['--']:
        network_proof(sys.argv[2])
        process_proof(sys.argv[3])
        arguments = sys.argv[5:]
    elif sys.argv[1:2] == ['--process-exec'] and sys.argv[3:4] == ['--']:
        process_proof(sys.argv[2])
        arguments = sys.argv[4:]
    else:
        raise SystemExit('Use run.py runtime or a named verification profile')
    os.execv(arguments[0], arguments)
