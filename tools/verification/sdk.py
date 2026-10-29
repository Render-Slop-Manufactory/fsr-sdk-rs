# SPDX-License-Identifier: MPL-2.0
"""Explicit, pinned maintainer SDK acquisition; never used by Cargo builds."""
from contextlib import contextmanager
import hashlib
import http.client
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
import zlib

from native import DLL_HASHES, header_sha
from records import now, save, sha

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
VERSION = 'v2.3.0'
ASSET = 'FidelityFX-Samples-v2.3.0-prebuilt.zip'
URL = f'https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/download/{VERSION}/{ASSET}'
ARCHIVE_SHA = 'f90890b9323bb2f4f2404ac4cdc9395e8495ecdac6f7aa0bcdf1ad1848422273'
DEFAULT_ROOT = ROOT / 'external/FidelityFX-SDK' / VERSION
SDK_PATH = 'Kits/FidelityFX/'
DLL_PATH = 'Samples/Upscalers/FidelityFX_FSR/dx12/x64/Release/'
NOTICE_HASHES = {
    'docs/license.md': '6757f2fec461238da6f84f83536ec195e8c036b1773a06f6f34301b9ed8c0963',
    '3rdpartynotice.md': '94401a90486d1b53b528bb7f845a7b8daf7227b60eb7c0fc3b8c5571ba202a27',
}
MAX_ARCHIVE = 200_000_000
MAX_SOURCE = 1_000_000
MAX_ENTRY = 64_000_000
MAX_EXPANDED = 512_000_000
MAX_ENTRIES = 2048
MAX_RATIO = 200
CHUNK = 64 * 1024
METADATA = 'provenance.json'
WINDOWS = os.name == 'nt'


def sources():
    manifest = json.loads((ROOT / 'tools/native-headers.json').read_text(encoding='utf-8'))
    files = {SDK_PATH + item['path']: item['sha256'] for item in manifest['headers']}
    files.update(NOTICE_HASHES)
    return manifest['commit'], files


def file_pins():
    _, files = sources()
    return {**files, **{SDK_PATH + 'signedbin/' + name: digest for name, digest in DLL_HASHES.items()}}


def no_links(path):
    """Reject symlinks and Windows junctions/reparse points, including parents."""
    path = Path(path).absolute()
    for part in reversed((path, *path.parents)):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise ValueError('Link/reparse point is not an SDK input or destination: ' + str(part))
    return path


def regular_file(path, limit):
    path = no_links(path)
    info = path.stat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > limit:
        raise ValueError('Expected a bounded, unlinked regular file: ' + str(path))
    return path


def copy_stream(source, destination, limit, expected_length=None, deadline=None):
    digest, count = hashlib.sha256(), 0
    with Path(destination).open('xb') as output:
        while True:
            if deadline is not None and time.monotonic() > deadline:
                raise ValueError('SDK transfer exceeded its time limit')
            data = source.read(CHUNK)
            if not data:
                break
            count += len(data)
            if count > limit:
                raise ValueError('SDK transfer exceeded its byte limit')
            output.write(data)
            digest.update(data)
        output.flush()
        os.fsync(output.fileno())
    if expected_length is not None and count != expected_length:
        raise ValueError('Incomplete SDK transfer')
    return digest.hexdigest()


class PinnedRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, url):
        check_url(url)
        return super().redirect_request(request, response, code, message, headers, url)


def check_url(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme != 'https' or parsed.hostname not in
            {'github.com', 'release-assets.githubusercontent.com', 'raw.githubusercontent.com'} or
            parsed.username or parsed.password or parsed.port not in (None, 443)):
        raise ValueError('SDK download redirected outside the permitted HTTPS origins')


def download(url, destination, expected, limit):
    check_url(url)
    request = urllib.request.Request(url, headers={'User-Agent': 'fsr-sdk-rs-sdk-acquisition',
                                                  'Accept-Encoding': 'identity'})
    try:
        with urllib.request.build_opener(PinnedRedirects()).open(request, timeout=30) as response:
            check_url(response.geturl())
            length = response.headers.get('Content-Length')
            length = int(length) if length is not None else None
            if length is not None and not 0 <= length <= limit:
                raise ValueError('SDK response exceeds its byte limit')
            digest = copy_stream(response, destination, limit, length, time.monotonic() + 180)
    except (urllib.error.URLError, OSError, http.client.HTTPException) as error:
        raise ValueError('SDK download failed: ' + str(error)) from error
    if digest != expected:
        raise ValueError(f'SDK SHA-256 mismatch: expected {expected}, observed {digest}')
    return digest


def acquire_archive(local, target):
    if local is None:
        return download(URL, target, ARCHIVE_SHA, MAX_ARCHIVE)
    with regular_file(local, MAX_ARCHIVE).open('rb') as stream:
        digest = copy_stream(stream, target, MAX_ARCHIVE)
    if digest != ARCHIVE_SHA:
        raise ValueError(f'Archive SHA-256 mismatch: expected {ARCHIVE_SHA}, observed {digest}')
    return digest


def zip_members(archive):
    entries, seen, total = {}, set(), 0
    infos = archive.infolist()
    if len(infos) > MAX_ENTRIES:
        raise ValueError('ZIP entry count exceeds its limit')
    for entry in infos:
        name = entry.filename.rstrip('/')
        path = PurePosixPath(name)
        if (entry.orig_filename != entry.filename or entry.filename not in (name, name + '/') or
                not path.parts or len(name) > 512 or path.is_absolute() or path.as_posix() != name or
                any(not re.fullmatch(r'[A-Za-z0-9 _.-]+', part) or part in ('.', '..') or
                    part.endswith((' ', '.')) or
                    re.fullmatch(r'(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?', part, re.I)
                    for part in path.parts)):
            raise ValueError('Unsafe ZIP path: ' + entry.filename)
        folded = name.casefold()
        if folded in seen:
            raise ValueError('Duplicate/case-colliding ZIP path: ' + name)
        seen.add(folded)
        kind = stat.S_IFMT(entry.external_attr >> 16)
        if (kind not in (0, stat.S_IFREG, stat.S_IFDIR) or
                (kind == stat.S_IFDIR and not entry.is_dir()) or
                (kind == stat.S_IFREG and entry.is_dir()) or
                entry.external_attr & 0x400 or
                (entry.external_attr & 0x10 and not entry.is_dir()) or entry.flag_bits & 1 or
                entry.compress_type not in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED)):
            raise ValueError('Unsupported ZIP entry type: ' + name)
        total += entry.file_size
        if (entry.file_size > MAX_ENTRY or total > MAX_EXPANDED or
                entry.file_size > MAX_RATIO * max(1, entry.compress_size) or
                (entry.is_dir() and entry.file_size)):
            raise ValueError('ZIP expansion limit exceeded: ' + name)
        entries[name] = entry
    # A file must never also be an ancestor of another member, even with different case.
    files = {name.casefold() for name, entry in entries.items() if not entry.is_dir()}
    for name in entries:
        if any(parent.as_posix().casefold() in files for parent in PurePosixPath(name).parents):
            raise ValueError('ZIP file/directory collision: ' + name)
    return entries


def extract(archive_path, stage):
    try:
        with zipfile.ZipFile(archive_path) as archive:
            entries = zip_members(archive)
            for name, expected in DLL_HASHES.items():
                entry = entries.get(DLL_PATH + name)
                if entry is None or entry.is_dir():
                    raise ValueError('Missing required DLL: ' + name)
                target = stage / SDK_PATH / 'signedbin' / name
                target.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(entry) as stream:
                    digest = copy_stream(stream, target, MAX_ENTRY, entry.file_size)
                if digest != expected:
                    raise ValueError(f'DLL SHA-256 mismatch: {name}; expected {expected}, observed {digest}')
    except (zipfile.BadZipFile, EOFError, RuntimeError, zlib.error) as error:
        raise ValueError('Invalid or incomplete SDK ZIP: ' + str(error)) from error


def acquire_sources(stage, local=None):
    commit, files = sources()
    for name, expected in files.items():
        target = stage / name
        target.parent.mkdir(parents=True, exist_ok=True)
        if local is None:
            url = f'https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/{commit}/{name}'
            download(url, target, expected, MAX_SOURCE)
        else:
            with regular_file(Path(local) / name, MAX_SOURCE).open('rb') as stream:
                digest = copy_stream(stream, target, MAX_SOURCE)
            if digest != expected:
                raise ValueError('Pinned source SHA-256 mismatch: ' + name)


def verify_files(root, metadata=True):
    root = no_links(root)
    if not root.is_dir():
        raise ValueError('SDK installation directory missing: ' + str(root))
    pins = file_pins()
    actual = {}
    expected_names = set(pins) | ({METADATA} if metadata else set())
    expected_dirs = {parent.as_posix() for name in expected_names for parent in PurePosixPath(name).parents
                     if parent.as_posix() != '.'}
    for directory, dirs, files in os.walk(root, followlinks=False):
        for name in dirs:
            path = no_links(Path(directory) / name)
            if path.relative_to(root).as_posix() not in expected_dirs:
                raise ValueError('Unexpected SDK directory: ' + str(path))
        for name in files:
            path = regular_file(Path(directory) / name, MAX_ENTRY)
            relative = path.relative_to(root).as_posix()
            if relative not in expected_names:
                raise ValueError('Unexpected installed file: ' + relative)
            if relative != METADATA:
                actual[relative] = sha(path)
    if actual != pins:
        changed = sorted(name for name, expected in pins.items() if actual.get(name) != expected)
        raise ValueError('Missing or changed pinned SDK files: ' + ', '.join(changed))
    manifest = json.loads((ROOT / 'tools/native-headers.json').read_text(encoding='utf-8'))
    for header in manifest['headers']:
        if header_sha(root / SDK_PATH / header['path']) != header['sha256']:
            raise ValueError('ABI header pin mismatch: ' + header['path'])
    if metadata:
        info = json.loads(regular_file(root / METADATA, MAX_SOURCE).read_text(encoding='utf-8'))
        if not isinstance(info, dict):
            raise ValueError('Invalid SDK provenance object')
        if ('license_notices' in info and 'licence_notices' in info and
                info['license_notices'] != info['licence_notices']):
            raise ValueError('Conflicting SDK provenance: license_notices and licence_notices')
        if info.get('license_notices', info.get('licence_notices')) != NOTICE_HASHES:
            raise ValueError('Inconsistent SDK provenance: license_notices')
        commit, _ = sources()
        for key, expected in {'version': VERSION, 'url': URL, 'asset': ASSET, 'commit': commit,
                              'archive_expected_sha256': ARCHIVE_SHA, 'archive_sha256': ARCHIVE_SHA,
                              'files': pins}.items():
            if info.get(key) != expected:
                raise ValueError('Inconsistent SDK provenance: ' + key)
    return actual


def signatures(root, strict=False):
    method = 'Windows Get-AuthenticodeSignature' + (' + SignTool + OS online chain policy' if strict else '')
    if not WINDOWS or not shutil.which('pwsh'):
        result = {'status': 'not_run', 'method': method, 'reason': 'Windows and PowerShell 7 required'}
        if strict:
            raise ValueError('Strict Authenticode verification unavailable: ' + result['reason'])
        return result
    args = ['pwsh', '-NoProfile', '-NonInteractive', '-File', str(HERE / 'windows.ps1'),
            '-Mode', 'signatures', '-Directory', str(root / SDK_PATH / 'signedbin')]
    if strict:
        args.append('-Strict')
    try:
        process = subprocess.run(args, capture_output=True, text=True, timeout=180, check=True)
        records = json.loads(process.stdout)
    except (subprocess.SubprocessError, OSError, ValueError) as error:
        raise ValueError('Authenticode verification failed or incomplete: ' + str(error)) from error
    if (not isinstance(records, list) or len(records) != 2 or
            any(not isinstance(item, dict) or not isinstance(item.get('signer'), str) for item in records) or
            {item.get('file') for item in records} != set(DLL_HASHES) or
            any(item.get('status') != 'Valid' or 'Advanced Micro Devices' not in item.get('signer', '')
                for item in records)):
        raise ValueError('AMD Authenticode signature or publisher check failed')
    if strict and any(item.get('strict_passed') is not True for item in records):
        raise ValueError('Strict Authenticode trust/timestamp/revocation verification incomplete: ' +
                         json.dumps(records))
    return {'status': 'passed' if strict else 'windows_policy_valid', 'method': method,
            'revocation': 'OS online policy' if strict else 'not independently established',
            'checked_at': now(), 'files': records}


def verify(root=DEFAULT_ROOT, strict=False):
    files = verify_files(root)
    trust = signatures(Path(root).absolute(), True) if strict else {
        'status': 'not_run', 'reason': 'Offline hash verification only; use --strict for Windows trust checks'}
    # Do not repeat the installation's historical signature result as a current trust claim.
    return {'version': VERSION, 'root': str(Path(root).absolute()), 'integrity': 'passed',
            'files': files, 'signatures': trust}


@contextmanager
def installation_lock(root):
    no_links(root.parent)
    root.parent.mkdir(parents=True, exist_ok=True)
    lock = root.parent / ('.' + root.name + '.sdk-lock')
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError as error:
        raise ValueError('SDK installation busy or interrupted; inspect and explicitly remove ' + str(lock)) from error
    os.close(descriptor)
    try:
        yield
    finally:
        lock.unlink()


def install(root=DEFAULT_ROOT, archive=None, source_root=None, strict=False):
    root = no_links(root)
    with installation_lock(root):
        if root.exists():
            return verify(root, strict)  # No downloads or silent repair of existing files.
        with tempfile.TemporaryDirectory(prefix='.' + root.name + '.sdk-', dir=root.parent) as temporary:
            workspace = Path(temporary)
            stage = workspace / 'stage'
            stage.mkdir()
            digest = acquire_archive(archive, workspace / ASSET)
            extract(workspace / ASSET, stage)
            acquire_sources(stage, source_root)
            files = verify_files(stage, metadata=False)
            trust = signatures(stage, strict)
            commit, _ = sources()
            save(stage / METADATA, {'version': VERSION, 'url': URL, 'asset': ASSET,
                'commit': commit, 'installed_at': now(), 'archive_expected_sha256': ARCHIVE_SHA,
                'archive_sha256': digest, 'archive_bytes': (workspace / ASSET).stat().st_size,
                'files': files, 'license_notices': NOTICE_HASHES, 'signatures': trust,
                'dll_archive_members': [DLL_PATH + name for name in DLL_HASHES]})
            verify_files(stage)
            no_links(root)
            if root.exists():
                raise ValueError('SDK destination appeared during installation; refusing replacement')
            # Same-volume rename publishes only a completely checked tree. The sibling lock
            # serializes this tool; parent directories must remain under the caller's control.
            stage.rename(root)
    return {'version': VERSION, 'root': str(root), 'integrity': 'passed', 'files': files, 'signatures': trust}
