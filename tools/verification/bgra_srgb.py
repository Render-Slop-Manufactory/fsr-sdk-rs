# SPDX-License-Identifier: MPL-2.0
"""Opt-in independent BGRA/sRGB DX12 probe; no FSR runtime or public API change."""
import argparse
import json
import math
from pathlib import Path
import struct

from execution import Case, Rejected, Runner
from native import windows_environment
from records import fresh, save

ROOT = Path(__file__).resolve().parents[2]
FORMATS = {'rgba': (28, 28, 28, 10), 'bgra': (87, 87, 87, 14),
           'rgba_srgb': (29, 29, 28, 12), 'bgra_srgb': (91, 91, 87, 15),
           'rgba_typeless': (27, 28, 28, 9), 'bgra_typeless': (90, 87, 87, 13)}
COLORS = ((48, 128, 224, 160), (192, 64, 16, 127))
TOLERANCE = 0.00002


def decode(value):
    value /= 255.0
    return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4


def expected(color, srgb):
    return tuple(decode(c) if srgb and i < 3 else c / 255.0 for i, c in enumerate(color))


def srgb_code(value):
    """Ideal inverse transfer without integer rounding, in 8-bit code units."""
    return 255.0 * (12.92 * value if value <= 0.0031308 else 1.055 * value ** (1.0 / 2.4) - 0.055)


def compare_floats(path, colors, srgb, swapped=False):
    data = path.read_bytes()
    if len(data) != 128 * 16:
        raise Rejected('Incomplete float readback: ' + path.name)
    maximum = 0.0
    maximum_code_error = 0.0
    for i, values in enumerate(struct.iter_unpack('<4f', data)):
        p = i % 64
        color = colors[((p // 8 + p % 8) & 1) if len(colors) == 2 else 0]
        if swapped:
            color = (color[2], color[1], color[0], color[3])
        reference = expected(color, srgb)
        if not all(math.isfinite(x) for x in values):
            raise Rejected('Non-finite float readback: ' + path.name)
        for channel, (actual, ideal) in enumerate(zip(values, reference)):
            error = abs(actual - ideal)
            maximum = max(maximum, error)
            if srgb and channel < 3:
                # Direct3D functional specification 3.2.3.7: 0.5 ULP on the
                # SRGB side, not float32 ULP. Endpoints must remain exact.
                code_error = abs(srgb_code(actual) - color[channel])
                maximum_code_error = max(maximum_code_error, code_error)
                if code_error > 0.5 or (color[channel] in (0, 255) and actual != ideal):
                    raise Rejected(f'{path.name}: sRGB code error {code_error} exceeds 0.5')
            elif error > TOLERANCE:
                raise Rejected(f'{path.name}: error {error} exceeds frozen {TOLERANCE}')
    return {'maximum_absolute_error': maximum, 'maximum_srgb_code_error': maximum_code_error}


def cases(options):
    sdk = Path(options.sdk_root).resolve()

    def compile_probe(r):
        import hashlib
        pins = json.loads((ROOT / 'tools/native-headers.json').read_text(encoding='utf-8'))
        for item in pins['headers'][:3]:
            raw = (sdk / item['path']).read_bytes().replace(b'\r\n', b'\n')
            if hashlib.sha256(raw).hexdigest() != item['sha256']:
                raise Rejected('Pinned header mismatch: ' + item['path'])
        r.note(sdk_commit=pins['commit'], amd_runtime_loaded=False)
        r.process(['cl', '/nologo', '/std:c++17', '/EHsc', '/W4', '/WX',
                   '/I' + str(sdk / 'api/include'), ROOT / 'tools/verification/bgra_srgb_probe.cpp',
                   '/Fe' + str(r.directory / 'view-probe.exe'), '/Fo' + str(r.directory / 'view-probe.obj'),
                   '/link', 'd3d12.lib', 'dxgi.lib', 'd3dcompiler.lib'], timeout=120)

    def execute_probe(r):
        output, _, _ = r.process([r.directory / 'view-probe.exe', str(options.adapter_index)],
                                 cwd=r.directory, timeout=90, name='gpu')
        rows = [json.loads(line) for line in output.splitlines() if line.startswith('{')]
        if not rows or rows[-1] != {'kind': 'finished', 'fenced': True, 'unexpected_debug_errors': 0}:
            raise Rejected('Probe did not complete fencing and diagnostic checks')
        r.state['rows'] = rows
        save(r.directory / 'observations.json', rows)

    def evaluate(r):
        rows = r.state['rows']
        inputs = [x for x in rows if x['kind'] == 'input']
        outputs = [x for x in rows if x['kind'] == 'output']
        if {(x['name'], x['swapped']) for x in inputs} != {(x, False) for x in FORMATS} | {('bgra', True)} or len(inputs) != 7:
            raise Rejected('Input case inventory mismatch')
        if {x['name'] for x in outputs} != set(FORMATS) or len(outputs) != 6:
            raise Rejected('Output case inventory mismatch')
        measurements = []
        for item in inputs:
            name, swapped = item['name'], item['swapped']
            allocation, srv, _, native = FORMATS[name]
            if (item['allocation'], item['srv'], item['native_format'], item['native_usage']) != (allocation, srv, native, 0):
                raise Rejected('Header/allocation mapping mismatch: ' + name)
            suffix = '_swapped' if swapped else ''
            upload = (r.directory / (name + suffix + '.upload')).read_bytes()
            pitch = item['row_pitch']
            if pitch < 32 or pitch % 256 or len(upload) < pitch * 7 + 32:
                raise Rejected('Invalid upload footprint: ' + name)
            for y in range(8):
                for x in range(8):
                    c = COLORS[(x + y) & 1]
                    if name.startswith('bgra') != swapped:
                        c = (c[2], c[1], c[0], c[3])
                    if upload[y * pitch + x * 4:y * pitch + x * 4 + 4] != bytes(c):
                        raise Rejected('Independent upload-byte audit failed: ' + name)
            errors = compare_floats(r.directory / (name + suffix + '.floats'), COLORS, name.endswith('srgb'), swapped)
            measurements.append({'case': name + suffix, **errors, 'status': 'passed'})
        # The deliberately swapped upload must fail the ordinary channel expectation.
        try:
            compare_floats(r.directory / 'bgra_swapped.floats', COLORS, False)
        except Rejected:
            pass
        else:
            raise Rejected('R/B-swap negative control was not detected')
        for item in outputs:
            name = item['name']
            if item['status'] != 'written':
                if item['status'] not in ('allocation_rejected', 'unsupported'):
                    raise Rejected('Unknown output outcome')
                measurements.append({'case': name + '_output', **item})
                continue
            if (item['allocation'], item['srv'], item['uav']) != FORMATS[name][:3]:
                raise Rejected('Output-view mapping mismatch: ' + name)
            c = COLORS[0]
            if name.startswith('bgra'):
                c = (c[2], c[1], c[0], c[3])
            stored = (r.directory / (name + '.stores')).read_bytes()
            if stored != bytes(c) * 64:
                raise Rejected('UAV storage is not the exact expected UNORM bytes: ' + name)
            errors = compare_floats(r.directory / (name + '.stored_floats'), COLORS[:1], name.endswith('srgb'))
            measurements.append({'case': name + '_output', 'status': 'passed', **errors})
        if next(x for x in outputs if x['name'] == 'rgba')['status'] != 'written':
            raise Rejected('Mandatory baseline output did not write')
        save(r.directory / 'evaluation.json', {'unorm_absolute_tolerance': TOLERANCE, 'srgb_code_tolerance': 0.5, 'measurements': measurements,
                                             'rb_swap_negative_control': 'rejected_as_expected'})
        r.note(measurements=measurements, rb_swap_negative_control='rejected_as_expected')
        # A completed observation can pass without declaring rejected routes supported.
        if any(x['status'] != 'written' for x in outputs):
            r.current['evaluation'] = 'observed'

    return [Case('windows.environment', 'x64 MSVC environment', windows_environment),
            Case('views.compile', 'Independent shaders and pinned public helper compile', compile_probe, ('windows.environment',), scope='windows-compilation'),
            Case('views.execute', 'Explicit hardware DX12 read/store probes and fencing', execute_probe, ('views.compile',), scope='gpu-experiment'),
            Case('views.evaluate', 'Independent CPU byte/sRGB/channel checks; unsupported outputs remain observations', evaluate, ('views.execute',), scope='gpu-experiment')]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', action='store_true', required=True)
    parser.add_argument('--adapter-index', type=int, required=True)
    parser.add_argument('--sdk-root', required=True, help='Pinned Kits/FidelityFX directory')
    options = parser.parse_args()
    if options.adapter_index < 0:
        parser.error('Adapter index must be non-negative')
    directory = fresh(ROOT / 'target/verification', 'bgra-srgb-views')
    print(directory, flush=True)
    runner = Runner(ROOT, directory, options)
    return runner.execute(cases(options), 'bgra-srgb-views')


if __name__ == '__main__':
    raise SystemExit(main())
