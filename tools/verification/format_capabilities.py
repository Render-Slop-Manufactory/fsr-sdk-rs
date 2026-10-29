# SPDX-License-Identifier: MPL-2.0
"""Pinned helper mappings and actual adapter format bits; no SDK dispatch."""
import argparse
import json
from pathlib import Path
from execution import Case, Rejected, Runner
from native import abi_cases
from records import fresh, save

ROOT = Path(__file__).resolve().parents[2]


def cases():
    def compile_probe(r):
        r.process(['cl', '/nologo', '/std:c++17', '/EHsc', '/W4', '/WX',
                   '/I' + str(r.state['headers'] / 'api/include'),
                   ROOT / 'tools/verification/format_capabilities.cpp',
                   '/Fe' + str(r.directory / 'format-capabilities.exe'),
                   '/Fo' + str(r.directory / 'format-capabilities.obj'),
                   '/link', 'd3d12.lib', 'dxgi.lib'])

    def execute(r):
        out, _, _ = r.process([r.directory / 'format-capabilities.exe', str(r.options.adapter_index)])
        rows = [json.loads(line) for line in out.splitlines() if line.startswith('{')]
        if len(rows) != 116 or [x['format'] for x in rows[1:]] != list(range(1, 116)):
            raise Rejected('Incomplete DXGI format query inventory')
        save(r.directory / 'capabilities.json', rows)
        r.note(capabilities=rows)

    return abi_cases()[:2] + [
        Case('formats.compile', 'Compile pinned helper capability inventory', compile_probe,
             ('windows.environment', 'abi.headers'), scope='windows-compilation'),
        Case('formats.query', 'Explicit adapter native helper mappings and device bits', execute,
             ('formats.compile',), scope='native-query')]


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--sdk-root', required=True)
    p.add_argument('--adapter-index', type=int, required=True)
    options = p.parse_args()
    options.online = False; options.toolchain = None
    directory = fresh(ROOT / 'target/verification', 'format-capabilities')
    print(directory, flush=True)
    raise SystemExit(Runner(ROOT, directory, options).execute(cases(), 'format-capabilities', require_passed=True))
