# SPDX-License-Identifier: MPL-2.0
"""Check the checkout budget, including new files that Git does not ignore."""
from pathlib import Path
import subprocess
import sys


TOTAL_LIMIT = 50_000_000
FILE_LIMIT = 1_000_000


def evaluate(entries):
    errors = []
    total = sum(size for _, size in entries)
    if total > TOTAL_LIMIT:
        errors.append(f'Total {total:,} bytes exceeds {TOTAL_LIMIT:,}')
    for name, size in entries:
        if size > FILE_LIMIT:
            errors.append(f'{name}: {size:,} bytes exceeds {FILE_LIMIT:,}')
        if name.startswith('docs/') and (name.endswith(('.inputs', '.rgba16f', '.gz')) or '/sources/' in name):
            errors.append(f'{name}: generated diagnostics belong under ignored target/, not in documentation')
    return total, errors


def check(root):
    root = Path(root).resolve()
    result = subprocess.run(['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'],
                            cwd=root, check=True, stdout=subprocess.PIPE)
    entries = []
    for name in sorted(set(result.stdout.decode('utf-8').split('\0')) - {''}):
        path = root / name
        if path.is_symlink() or not path.resolve().is_relative_to(root):
            raise ValueError(f'File escapes checkout: {name}')
        if path.is_file():  # Tracked deletions are absent from the candidate checkout.
            entries.append((name, path.stat().st_size))
    return evaluate(entries)


if __name__ == '__main__':
    try:
        total, errors = check(Path(__file__).resolve().parents[2])
        print(f'Repository file contents: {total:,} bytes ({total/1_000_000:.2f} MB)')
        for error in errors:
            print(error, file=sys.stderr)
        sys.exit(bool(errors))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(2)
