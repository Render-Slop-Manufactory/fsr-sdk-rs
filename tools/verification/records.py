# SPDX-License-Identifier: MPL-2.0
"""Run identity, artefact inventories and exclusive evidence export."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import time
import uuid


def now():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    path = Path(path)
    temporary = path.with_suffix(path.suffix + '.pending')
    temporary.write_bytes((json.dumps(value, indent=2) + '\n').encode('utf-8'))
    # Windows readers/scanners can briefly deny replacement of a closed file.
    # Preserve the old complete record and retain pending bytes on final failure.
    for attempt in range(8):
        try:
            temporary.replace(path)
            return
        except PermissionError as error:
            if getattr(error, 'winerror', None) not in (5, 32, 33) or attempt == 7:
                raise
            time.sleep(.01 * 2**attempt)


def fresh(root, profile):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    name = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    path = root / f'{name}-{profile}-{uuid.uuid4().hex[:12]}'
    path.mkdir()  # Exclusive creation; never open a previous attempt for writing.
    return path.resolve()


def inventory(root):
    root = Path(root).resolve()
    result = []
    for path in sorted(root.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'Symlink is not a retained artefact: {path.name}')
        if path.is_file() and path != root / 'artifacts.json':
            result.append({'path': path.relative_to(root).as_posix(),
                           'bytes': path.stat().st_size, 'sha256': sha(path)})
    return result


def checked_path(root, name):
    path = (Path(root) / name).resolve()
    if not path.is_relative_to(Path(root).resolve()):
        raise ValueError('Artefact path escapes its run')
    return path


def verify_artifacts(root):
    root = Path(root)
    entries = json.loads((root / 'artifacts.json').read_text(encoding='utf-8'))
    names = set()
    for entry in entries:
        if entry['path'] in names:
            raise ValueError('Duplicate artefact inventory entry')
        names.add(entry['path'])
        path = checked_path(root, entry['path'])
        if not path.is_file() or path.stat().st_size != entry['bytes'] or sha(path) != entry['sha256']:
            raise ValueError(f'Artefact identity mismatch: {entry["path"]}')
    if {item['path'] for item in inventory(root)} != names:
        raise ValueError('Artefact inventory has missing or additional files')
    return entries


class Sanitizer:
    def __init__(self, root, run=None):
        self.paths = [(Path(root).resolve(), '<repo>'), (Path.home(), '<user-home>')]
        if run:
            self.paths.insert(0, (Path(run).resolve(), '<run>'))
        self.pointers = {}

    def text(self, value):
        value = str(value)
        for path, replacement in self.paths:
            # Wine also exposes Linux paths through DOS drive mappings. Retain
            # the drive label while normalizing the private Linux path suffix.
            for spelling in (str(path), path.as_posix(), path.as_posix().replace('/', '\\')):
                for _ in range(4):
                    value = value.replace(spelling, replacement)
                    spelling = spelling.replace('\\', '\\\\')
        value = re.sub(r'(?i)<user-home>[\\/]+AppData[\\/]+Local[\\/]+Packages[\\/]+sandbox\.\{[0-9a-f-]+\}[\\/]+AC[\\/]+Temp',
                       '<sandbox-temp>', value)
        # Only pointer-labelled values, never provider IDs, return codes or hashes.
        def pointer(match):
            # DXGI adapter device IDs are hardware identity, not COM pointers.
            # Preserve the named adapter line while still normalizing device
            # pointers in lifecycle/dispatch observations.
            line_start = value.rfind('\n', 0, match.start()) + 1
            if (re.fullmatch(r'(?i)device\s*[:=]\s*', match[1])
                    and re.search(r'(?i)\bvendor(?:_id)?=0x[0-9a-f]+\s+$',value[line_start:match.start()])):
                return match[0]
            bits = match[2].lower()
            if int(bits, 16) == 0:
                return match[0]
            label = self.pointers.setdefault(bits, f'<pointer-{len(self.pointers) + 1}>')
            return match[1] + label
        value = re.sub(r'(?i)((?:\b(?:handle|pointer|ptr|device|context|root|library|allocation|device_ptr|context_ptr|allocation_ptr)|\bDedicated descriptor storage:\s*base)\s*[:=]\s*)(0x[0-9a-f]+)', pointer, value)
        return re.sub(r'(?i)[a-z]:[\\/]+Users[\\/]+[^\\/\s"<>]+', '<user-home>', value)

    def value(self, value):
        if isinstance(value, (str, os.PathLike)):
            return self.text(value)
        if isinstance(value, (list, tuple)):
            return [self.value(item) for item in value]
        if isinstance(value, dict):
            return {key: self.value(item) for key, item in value.items()}
        return value


def export(root, destination, repository):
    root, destination = Path(root).resolve(), Path(destination).resolve()
    entries = verify_artifacts(root)
    from retention import CALIBRATION_PROFILES, calibration_diagnostic, compact_case
    calibration = json.loads((root / 'run.json').read_text(encoding='utf-8')).get('profile') in CALIBRATION_PROFILES
    if destination.is_relative_to(root) or root.is_relative_to(destination):
        raise ValueError('Evidence destination overlaps the source run')
    # Validate source before creating output; partial attempts are retained on failure.
    destination.mkdir(parents=True, exist_ok=False)
    sanitizer = Sanitizer(repository, root)
    for entry in entries:
        name = entry['path']
        # Logs, declared inputs, results, analyses and readbacks are exported.
        # Toolchain environment, build products and AMD distributions remain local.
        if not (name in ('run.json', 'plan.json', 'inputs.json', 'retention.json') or
                name.startswith(('cases/', 'analysis/', 'readbacks/'))):
            continue
        if calibration and calibration_diagnostic(name):
            continue
        if name.endswith(('.dll', '.exe', '.obj', '.lib', '.pdb')):
            continue
        # Full Cargo metadata includes unrelated author/contact fields; evaluations
        # retain selected registry provenance instead.
        if any(label in name for label in ('-metadata.', '-environment.')) and name.endswith(('.stdout.txt', '.stderr.txt')):
            continue
        source, target = checked_path(root, name), checked_path(destination, name)
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.suffix in ('.json', '.txt', '.csv'):
            if source.suffix == '.json':
                value = json.loads(source.read_text(encoding='utf-8'))
                if calibration and name.startswith('cases/') and name.endswith('/result.json'):
                    value = compact_case(value)
                save(target, sanitizer.value(value))
            else:
                target.write_bytes(sanitizer.text(source.read_text(encoding='utf-8')).encode('utf-8'))
        else:
            shutil.copyfile(source, target)
    save(destination / 'export.json', {'schema_version': 1, 'exported_utc': now(),
         'source_input_identity': json.loads((root / 'inputs.json').read_text(encoding='utf-8'))['identity'],
         'normalization': 'JSON/TXT/CSV use UTF-8/LF; repository/home/run paths and pointer-labelled non-null values normalized; nulls, provider IDs, native codes and hashes preserved.',
         'raw_readbacks_included': not calibration,
         'offline_image_reanalysis_available': not calibration,
         'source_status': json.loads((root / 'run.json').read_text(encoding='utf-8'))['status']})
    # Hash the final exported bytes, and preserve even verbatim/binary artefacts
    # when Git would otherwise detect and convert them as text.
    (destination / '.gitattributes').write_bytes(b'# Preserve exported byte identities across checkouts.\n* -text whitespace=cr-at-eol\n')
    save(destination / 'artifacts.json', inventory(destination))
    return destination


def source_identity(repository, extra=()):
    repository = Path(repository)
    paths = [repository / name for name in ('.gitattributes', 'Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md') if (repository / name).is_file()]
    for directory in ('crates', 'tools', '.github'):
        paths.extend(path for path in (repository / directory).rglob('*')
                     if path.is_file() and not path.is_symlink() and
                     not any(part in ('target', '__pycache__', 'archive') for part in path.relative_to(repository).parts))
    paths.extend(path for path in (repository / 'docs/release').glob('*protocol.md'))
    entries = [{'path': path.relative_to(repository).as_posix(), 'sha256': sha(path)}
               for path in sorted(set(paths))]
    for label, path in extra:
        entries.append({'path': label, 'sha256': sha(path)})
    identity = hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest()
    return {'schema_version': 1, 'identity': identity, 'files': entries}
