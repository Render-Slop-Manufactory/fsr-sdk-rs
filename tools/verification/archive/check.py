# SPDX-License-Identifier: MPL-2.0
"""Read-only integrity/navigation/syntax check; never execute archived scripts."""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
from urllib.parse import unquote

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[2]
LINK = re.compile(r'(?<!!)\[[^\]]*\]\(([^)\s]+)\)')


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def contained(base, name):
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and '..' not in relative.parts,
            f'Invalid relative archive path: {name}')
    path = base.joinpath(*relative.parts).resolve()
    require(path.is_relative_to(base.resolve()), f'Escaping archive path: {name}')
    return path


def check_links(path):
    for match in LINK.finditer(path.read_text(encoding='utf-8')):
        target = unquote(match[1]).split('#', 1)[0]
        if not target or '://' in target or target.startswith('mailto:'):
            continue
        require((path.parent / target).exists(),
                f'Broken archive link in {path.relative_to(ROOT).as_posix()}: {target}')


def check_archive(archive, require_powershell=False):
    inventory = json.loads((archive / 'manifest.json').read_text(encoding='utf-8'))
    require(inventory['schema_version'] == 1, 'Unsupported archive manifest version')
    require(inventory['archived_on'] == archive.name, 'Archive date does not match directory')
    entries = inventory['entries']
    originals = {row['original'] for row in entries}
    require(len(originals) == len(entries), 'Duplicate original paths')
    expected_files = set()
    python_count = 0
    json_count = 0
    powershell_count = 0
    retired = 0
    for row in entries:
        original = contained(ROOT, row['original'])
        require(row['archived'] == 'files/' + row['original'] + '.txt',
                f'Layout mismatch: {row["original"]}')
        payload = contained(archive, row['archived'])
        require(payload.is_file(), f'Missing payload: {row["archived"]}')
        expected_files.add(payload)
        data = payload.read_bytes()
        require(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'],
                f'Payload identity changed: {row["original"]}')
        require(row['group'] in inventory['groups'], f'Unknown group: {row["group"]}')
        require(row['disposition'] in ('retired', 'context-snapshot'), 'Unknown disposition')
        if row['disposition'] == 'retired':
            retired += 1
            require(not original.exists(), f'Retired entry point still exists: {row["original"]}')
        for dependency in row['depends_on']:
            require(dependency in originals, f'Missing direct dependency snapshot: {dependency}')
        for name in row['referenced_by']:
            document = contained(ROOT, name)
            require(document.is_file(), f'Missing referring document: {name}')
            if row['disposition'] == 'retired':
                text = document.read_text(encoding='utf-8')
                navigation = os.path.relpath(archive / 'README.md', document.parent).replace('\\', '/')
                require(navigation in text, f'Missing archive navigation in {name}')
        if row['original'].endswith('.py'):
            ast.parse(data.decode('utf-8-sig'), filename=row['original'])
            python_count += 1
        elif row['original'].endswith('.json'):
            json.loads(data.decode('utf-8-sig'))
            json_count += 1
        elif row['original'].endswith('.ps1'):
            powershell_count += 1
    actual_files = {path.resolve() for path in (archive / 'files').rglob('*') if path.is_file()}
    require(actual_files == expected_files, 'Payload inventory is incomplete or has extra files')
    for group in inventory['groups'].values():
        for name in group['repository_inputs']:
            # Intake context may later be replaced by current orchestration.
            require(contained(ROOT, name).exists() or name in originals,
                    f'Missing documented repository context or snapshot: {name}')
    check_links(archive / 'README.md')

    result = {'archive': archive.name,
              'integrity': {'execution': 'completed', 'evaluation': 'passed',
                            'retired_scripts': retired, 'context_snapshots': len(entries)-retired},
              'python_syntax': {'execution': 'completed', 'evaluation': 'passed', 'sources': python_count},
              'json_syntax': {'execution': 'completed', 'evaluation': 'passed', 'sources': json_count}}
    print(f'{archive.name}: {retired} retired scripts, {len(entries)-retired} context snapshots; '
          f'hashes, inventory, dependencies and navigation passed; '
          f'{python_count} Python and {json_count} JSON sources parsed')
    result['powershell_syntax'] = check_powershell(archive, powershell_count, require_powershell)
    syntax = result['powershell_syntax']
    print(f'{archive.name}: PowerShell syntax: {syntax["execution"]}, {syntax["evaluation"]}; '
          f'{syntax["sources"]} sources; required={syntax["required"]}' +
          (f'; {syntax["reason"]}' if 'reason' in syntax else ''))
    return result


def check_powershell(archive, count, required):
    # Parse source only. ParseFile does not execute commands, load DLLs or run probes.
    pwsh = shutil.which('pwsh')
    result = {'sources': count, 'required': required}
    if pwsh is None:
        return dict(result, execution='not_run', evaluation='not_evaluated',
                    reason='PowerShell 7 parser (pwsh) unavailable')
    parser = """
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'PowerShell 7 parser required' }
$archiveRoot = $env:FSR_VERIFICATION_ARCHIVE_CHECK_ROOT
$manifest = Get-Content -LiteralPath (Join-Path $archiveRoot 'manifest.json') -Raw | ConvertFrom-Json
$count = 0
foreach ($entry in $manifest.entries) {
    if (-not $entry.original.EndsWith('.ps1')) { continue }
    $tokens = $null
    $parseErrors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile(
        (Join-Path $archiveRoot $entry.archived), [ref]$tokens, [ref]$parseErrors)
    if ($parseErrors.Count) {
        Write-Output ('Syntax error in frozen source: ' + $entry.original)
        $parseErrors | ForEach-Object { Write-Output $_.Message }
        exit 1
    }
    $count++
}
Write-Output $count
"""
    environment = dict(os.environ, FSR_VERIFICATION_ARCHIVE_CHECK_ROOT=str(archive))
    try:
        parsed = subprocess.run([pwsh, '-NoProfile', '-NonInteractive', '-Command', parser],
                                capture_output=True, text=True, env=environment, timeout=30)
    except subprocess.TimeoutExpired:
        return dict(result, execution='timed_out', evaluation='failed', reason='PowerShell parser timed out')
    except OSError as error:
        return dict(result, execution='blocked', evaluation='failed', reason=str(error))
    if parsed.returncode != 0:
        return dict(result, execution='completed', evaluation='failed', reason=parsed.stdout + parsed.stderr)
    if parsed.stdout.strip() != str(count):
        return dict(result, execution='completed', evaluation='failed', reason='PowerShell parsed-source count mismatch')
    return dict(result, execution='completed', evaluation='passed', parser=pwsh)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require-powershell', action='store_true',
                        help='Fail if the PowerShell 7 syntax parser is unavailable')
    parser.add_argument('--report', type=Path, help='Write separate check outcomes to a new JSON file')
    options = parser.parse_args(argv)
    check_links(BASE / 'README.md')
    archives = sorted(path.parent for path in BASE.glob('*/manifest.json'))
    require(bool(archives), 'No indexed archives found')
    results = [check_archive(archive, options.require_powershell) for archive in archives]
    if options.report:
        with options.report.open('x', encoding='utf-8') as output:
            json.dump({'schema_version': 1, 'archives': results}, output, indent=2)
            output.write('\n')
    for result in results:
        syntax = result['powershell_syntax']
        require(syntax['evaluation'] != 'failed', f'{result["archive"]}: PowerShell syntax check failed')
        require(not syntax['required'] or syntax['evaluation'] == 'passed',
                f'{result["archive"]}: required PowerShell syntax check was not executed')


if __name__ == '__main__':
    main()
