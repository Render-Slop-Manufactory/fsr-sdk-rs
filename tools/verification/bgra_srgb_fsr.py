# SPDX-License-Identifier: MPL-2.0
"""Bounded direct-SDK BGRA/sRGB main and reactive-generation experiments."""
import argparse
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import struct

from execution import Case, Rejected, Runner
from native import build_case, environment, native_setup
from records import fresh, save
from typed_formats import BASE, STRIDE, SELECTOR, input_bytes as ordinary_input, output_check
from motion_scale import PROVIDERS

ROOT = Path(__file__).resolve().parents[2]
CANDIDATES = (87, 29, 91)
STRIDES = STRIDE | {87: 4, 29: 4, 91: 4}


def linear(code, domain):
    x = code / 255.0
    return x if domain == 'unorm' else x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4


def color_codes(x, y, phase, opaque, zero, altered=False):
    if zero:
        return (0, 0, 0, 255)
    c = (48, 128, 224, 255) if ((x - phase * 4) // 16 + y // 16) % 2 == 0 else (128, 64, 16, 255)
    if not opaque and 80 <= x < 240 and 45 <= y < 135:
        c = (c[0] + 64, *c[1:])
    return (c[2], c[1], c[0], c[3]) if altered else c


@lru_cache(maxsize=96)
def colors(fmt, phase, domain, opaque, zero, altered):
    result = bytearray()
    for y in range(180):
        for x in range(320):
            c = color_codes(x, y, phase, opaque, zero, altered)
            if fmt in (28, 29, 87, 91):
                result.extend((c[2], c[1], c[0], c[3]) if fmt in (87, 91) else c)
            else:
                values = tuple(linear(v, domain) for v in c[:3]) + (1.0,)
                result.extend(struct.pack('<4f' if fmt == 2 else '<4e', *values))
    return bytes(result)


def inputs(s, phase, role):
    fmt = s['formats'][role]
    if role in (0, 5):
        return colors(fmt, phase, s['domain'], role == 5, s['zero'], s['alter'])
    if role == 4:
        if fmt == 10:
            return struct.pack('<4e', 1, 0, 1, 0.25) * 640 * 360
        return bytes((255, 0, 255, 64)) * 640 * 360
    return ordinary_input(s, phase, role)


def workloads():
    base = dict(formats=list(BASE), uav=0, generate=False, dynamic=False, reversed=False,
                hdr=False, sharpen=False, control=False, zero=False, alter=False, domain='unorm')
    # Float32 references preserve independently decoded input precision.
    base['formats'][0] = 2
    result = [dict(base, name='control', control=True), dict(base, name='linear-reference'),
              dict(base, name='changed-input', alter=True), dict(base, name='decoded-reference', domain='srgb')]
    for fmt in (28, *CANDIDATES):
        fs = list(base['formats']); fs[0] = fmt
        result.append(dict(base, name=f'input-{fmt}', formats=fs, domain='srgb' if fmt in (29, 91) else 'unorm'))
        for sharp in (False, True):
            fs = list(base['formats']); fs[4] = fmt
            result.append(dict(base, name=f'output-{fmt}-sharp{int(sharp)}', formats=fs, sharpen=sharp))
    # Generator inputs are varied separately, including equal-black controls.
    for slot in (0, 5):
        for fmt in CANDIDATES:
            fs = list(BASE); fs[0] = fs[5] = 2; fs[slot] = fmt
            domain = 'srgb' if fmt in (29, 91) else 'unorm'
            for zero in (False, True):
                result.append(dict(base, name=f'generate-{slot}-{fmt}-zero{int(zero)}',
                                   formats=fs, domain=domain, generate=True, zero=zero))
    return result


def canonical_output(data, fmt):
    if len(data) != 640 * 360 * STRIDES[fmt]:
        raise Rejected('Incomplete output readback')
    if fmt in (87, 91):
        result = bytearray(data)
        result[0::4], result[2::4] = data[2::4], data[0::4]
        return bytes(result)
    return data


def mask_check(data, s, phase):
    if len(data) != 320 * 180:
        raise Rejected('Incomplete mask readback')
    width, height = (160, 90) if phase == 1 else (320, 180)
    written = 0
    for y in range(height):
        for x in range(width):
            actual = data[y * 320 + x]
            if actual == 0x55:
                raise Rejected(f'Generated active mask retained its seed at ({x},{y})')
            written += 1
    return dict(written=written, active_pixels=width * height,
                sha256=hashlib.sha256(data).hexdigest())


def evaluate(text, command, directory, s, provider, mode='reference'):
    if command['execution'] != 'completed' or command['exit_code'] != 0 or text.count('1 passed; 0 failed') != 1:
        raise Rejected('Direct SDK child did not complete')
    parse = lambda prefix: [json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    identity = [line for line in text.splitlines() if line.startswith('Format provider:')]
    if len(identity) != 1 or f'ProviderId({PROVIDERS[provider]})' not in identity[0] or f'"{provider}"' not in identity[0]:
        raise Rejected('Wrong selected provider')
    if parse('FORMAT_RESULT ') != [dict(frames=3, cleanup_ok=True)]:
        raise Rejected('Missing explicit context cleanup')
    if parse('FORMAT_FRAME ') != [dict(phase=p, debug_errors=0, removed=0) for p in range(3)]:
        raise Rejected('Incomplete frame/fence/debug evidence')
    caps = parse('FORMAT_CAP ')
    if len(caps) != 21:
        raise Rejected('Incomplete resource capability inventory')
    result = []
    for phase in range(3):
        hashes = []
        for role in range(7):
            cap = caps[phase * 7 + role]
            if cap['role'] != role or cap['format'] != s['formats'][role]:
                raise Rejected('Wrong allocation format/role')
            data = (directory / f'{mode}-{phase}-{role}.bin').read_bytes()
            if len(data) != len(inputs(s, phase, role)):
                raise Rejected('Wrong resource byte count')
            if role != 4 and not (role == 6 and s['generate']):
                if data != inputs(s, phase, role):
                    raise Rejected('Uploaded resource was changed or wrongly copied')
            hashes.append(hashlib.sha256(data).hexdigest())
        raw = (directory / f'{mode}-{phase}-4.bin').read_bytes()
        fmt = s['formats'][4]
        if fmt == 10:
            metrics = output_check(raw, inputs(s, phase, 4), 10, (640, 360), s['control'])
        else:
            canonical = canonical_output(raw, fmt)
            changed = sum(canonical[p:p + 4] != bytes((255, 0, 255, 64)) for p in range(0, len(canonical), 4))
            if changed != 640 * 360:
                raise Rejected('Incomplete output writes')
            metrics = dict(written=changed, sha256=hashlib.sha256(raw).hexdigest())
        entry = dict(phase=phase, resource_sha256=hashes, output=metrics)
        if s['generate']:
            entry['mask'] = mask_check((directory / f'{mode}-{phase}-6.bin').read_bytes(), s, phase)
        result.append(entry)
    return dict(frames=result, capabilities=caps)


def cases():
    result = native_setup(format_probe=True)
    build = build_case('bgra-fsr'); result.append(build)
    selections = workloads()
    for provider in ('3.1.5', '4.1.1'):
        chosen = [s for s in selections if not s['generate'] or provider == '3.1.5']
        for s in chosen:
            name = 'bgra-fsr.' + provider + '.' + s['name']
            def run(r, s=s, provider=provider, name=name):
                observations = []
                for repeat in range(2):
                    directory = r.directory / 'readbacks' / name / str(repeat)
                    directory.mkdir(parents=True)
                    for phase in range(3):
                        for role in range(7):
                            (directory / f'input-{phase}-{role}.bin').write_bytes(inputs(s, phase, role))
                    env = environment(r)
                    settings = dict(PROVIDER=provider, FAMILY='local', MODE='reference', DIR=directory,
                                    FORMATS=','.join(map(str, s['formats'])), UAV=0, GENERATE=int(s['generate']),
                                    DYNAMIC=0, REVERSED=0, HDR=0, SHARPEN=int(s['sharpen']), CONTROL=int(s['control']))
                    env.update({'FSR_SDK_TEST_FORMAT_' + k: str(v) for k, v in settings.items()})
                    out, err, command = r.process([r.state[build.id], '--ignored', '--exact', SELECTOR,
                                                   '--nocapture', '--test-threads=1'],
                                                  env=env, cwd=r.state['native_stage'], observe=True, timeout=180)
                    observations.append(evaluate(out + err, command, directory, s, provider))
                r.note(selection=s, provider=provider, repeats=observations)
                r.current['evaluation'] = 'passed'
            result.append(Case(name, 'Direct pinned C++ SDK ' + s['name'] + ' on ' + provider, run,
                               (build.id,), SELECTOR, 'native-gpu'))

    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--native', action='store_true', required=True)
    p.add_argument('--adapter-index', type=int, required=True)
    p.add_argument('--sdk-root', required=True)
    options = p.parse_args()
    options.sdk_root = str(Path(options.sdk_root).resolve())
    options.online = False; options.toolchain = None
    directory = fresh(ROOT / 'target/verification', 'bgra-srgb-fsr')
    print(directory, flush=True)
    return Runner(ROOT, directory, options).execute(cases(), 'bgra-srgb-fsr', require_passed=True)


if __name__ == '__main__':
    raise SystemExit(main())
