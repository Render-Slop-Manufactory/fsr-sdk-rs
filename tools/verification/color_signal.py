# SPDX-License-Identifier: MPL-2.0
"""Explicit native signal experiments with an independent Decimal input oracle."""
from decimal import Decimal, localcontext
import json
import math
from pathlib import Path
import re
import struct

from execution import Blocked, Case, Rejected
from native import native_setup, build_case, environment
from records import save, sha

HERE = Path(__file__).resolve().parent
SELECTOR = 'upscaler::dx12::native_dispatch::native_color_signal'


def protocol():
    return json.loads((HERE / 'color-signal-protocol.json').read_text())


def encode(value, encoding):
    """Own scalar implementation of declared standard transfer equations."""
    if not math.isfinite(value) or value < 0:
        raise ValueError('Nonnegative finite signal required')
    if encoding == 'linear':
        return value
    if encoding == 'srgb':
        return 12.92 * value if value <= 0.0031308 else 1.055 * value ** (1 / 2.4) - 0.055
    if encoding != 'pq':
        raise ValueError('Unknown encoding')
    # PQ inverse EOTF: convention is 100 nits per relative linear unit.
    y = (value / 100) ** (2610 / 16384)
    return ((3424 / 4096 + (2413 / 128) * y) / (1 + (2392 / 128) * y)) ** (2523 / 32)


def decode_reference(code, encoding):
    """Independent high-precision inverse; no generator implementation is reused."""
    if not math.isfinite(code) or code < 0:
        raise ValueError('Nonnegative finite code required')
    with localcontext() as context:
        context.prec = 45
        x = Decimal(str(code))
        if encoding == 'linear':
            result = x
        elif encoding == 'srgb':
            result = x / Decimal('12.92') if x <= Decimal('0.04045') else (
                (x + Decimal('0.055')) / Decimal('1.055')) ** Decimal('2.4')
        elif encoding == 'pq':
            power = x ** (Decimal(32) / Decimal(2523))
            numerator = max(power - Decimal(3424) / Decimal(4096), Decimal(0))
            denominator = Decimal(2413) / Decimal(128) - Decimal(2392) / Decimal(128) * power
            if denominator <= 0:
                raise ValueError('Outside bounded PQ domain')
            result = Decimal(100) * (numerator / denominator) ** (Decimal(16384) / Decimal(2610))
        else:
            raise ValueError('Unknown encoding')
        return float(result)


def signal_codes(encoding, pre=1, packing='linear-before-encode'):
    levels = protocol()['levels']
    if packing == 'linear-before-encode':
        values = [encode(level * pre, encoding) for level in levels]
    elif packing == 'encoded-after-encode':
        values = [encode(level, encoding) * pre for level in levels]
    else:
        raise ValueError('Unknown packing')
    if not all(0 <= value <= 1 for value in values):
        raise ValueError('Probe codes must stay within [0,1]')
    return [struct.unpack('<e', struct.pack('<e', value))[0] for value in values]


def input_bytes(encoding, pre=1, packing='linear-before-encode'):
    codes = signal_codes(encoding, pre, packing)
    row = b''.join(struct.pack('<4e', code, code, code, 1) * 40 for code in codes)
    return row * 180


def audit_input(payload, encoding, pre, packing):
    """Verify quantized input against the declared scene via the independent inverse."""
    import numpy as np
    if len(payload) != 320 * 180 * 8:
        raise Rejected('Invalid input length')
    image = np.frombuffer(payload, dtype='<f2').reshape(180, 320, 4)
    if not np.isfinite(image).all() or not (image[:, :, 3] == 1).all():
        raise Rejected('Invalid input channels')
    recovered = []
    for index, level in enumerate(protocol()['levels']):
        patch = image[:, index * 40:(index + 1) * 40, :3]
        code = float(patch[0, 0, 0])
        if not (patch == code).all():
            raise Rejected('Input patch is not exact neutral constant')
        decoded = decode_reference(code / pre if packing == 'encoded-after-encode' else code, encoding)
        observed = decoded / pre if packing == 'linear-before-encode' else decoded
        if abs(observed - level) > max(1e-7, level * 0.012):
            raise Rejected('Independent inverse rejects declared signal/normalization')
        recovered.append(observed)
    return recovered


def output_statistics(payload):
    import numpy as np
    if len(payload) != 640 * 360 * 8:
        raise Rejected('Invalid output length')
    image = np.frombuffer(payload, dtype='<f2').reshape(360, 640, 4)
    if not np.isfinite(image).all() or (image[:, :, :3] == 65504).any():
        raise Rejected('Nonfinite or retained-sentinel RGB output')
    margin = protocol()['patch_interior_margin_output_pixels']
    patches = []
    for index in range(8):
        patch = image[margin:-margin, index * 80 + margin:(index + 1) * 80 - margin, :3].astype('float64')
        patches.append(dict(mean=patch.mean(axis=(0, 1)).tolist(), minimum=patch.min(axis=(0, 1)).tolist(),
                            maximum=patch.max(axis=(0, 1)).tolist()))
    return dict(patches=patches, rgb_min=float(image[:, :, :3].min()), rgb_max=float(image[:, :, :3].max()))


def hypotheses(stats, encoding, pre, exposure, packing):
    settings = protocol()
    codes = signal_codes(encoding, pre, packing)
    decoded = [decode_reference(value, encoding) for value in codes]
    candidates = {'supplied-code-domain': codes, 'decoded-relative-linear': decoded,
                  'code-divided-by-pre': [value / pre for value in codes],
                  'decoded-then-unpreexposed': [value / pre for value in decoded],
                  'code-times-exposure': [value * exposure for value in codes]}
    if encoding == 'pq':
        candidates['decoded-nits'] = [value * 100 for value in decoded]
        candidates['decoded-normalized-10000-nits'] = [value / 100 for value in decoded]
    result = {}
    for name, values in candidates.items():
        errors = [abs(channel - expected) for patch, expected in zip(stats['patches'], values)
                  for channel in patch['mean']]
        budgets = [max(settings['hypothesis_absolute_tolerance'], abs(value) * settings['hypothesis_relative_tolerance'])
                   for value in values for _ in range(3)]
        result[name] = dict(expected_patch_values=values, maximum_mean_error=max(errors),
                            matches_declared_budget=all(error <= budget for error, budget in zip(errors, budgets)))
    return result


def arms():
    result = []
    for encoding in ('linear', 'srgb', 'pq'):
        result.append(dict(name='neutral-' + encoding, encoding=encoding, hint=encoding, pre=[1] * 3,
                           exposure=[1] * 3, packing='linear-before-encode', automatic=False, phase='neutral'))
        if encoding != 'linear':
            result.append(dict(result[-1], name='neutral-' + encoding + '-unspecified', hint='unspecified'))
    for encoding in ('linear', 'srgb', 'pq'):
        base = dict(encoding=encoding, hint=encoding, packing='linear-before-encode', automatic=False, phase='exposure')
        for packing in ('linear-before-encode', 'encoded-after-encode'):
            if encoding == 'linear' and packing == 'encoded-after-encode':
                continue
            result.append(dict(base, name='pre-' + encoding + '-' + packing, pre=[0.5, 1, 2],
                               exposure=[1] * 3, packing=packing))
        result.append(dict(base, name='exposure-' + encoding, pre=[1] * 3, exposure=[0.25, 1, 4]))
        result.append(dict(base, name='automatic-' + encoding, pre=[1] * 3, exposure=[1] * 3, automatic=True))
    return result


def control_arms():
    result = []
    for encoding in ('linear', 'srgb', 'pq'):
        for packing in ('linear-before-encode', 'encoded-after-encode'):
            if encoding == 'linear' and packing == 'encoded-after-encode':
                continue
            base = dict(encoding=encoding, hint=encoding, packing=packing, automatic=False,
                        exposure=[1] * 3, phase='control')
            for pre in (0.5, 2):
                result.append(dict(base, name=f'fixed-{encoding}-{packing}-{pre}', pre=[pre] * 3))
            if encoding != 'linear':
                result.append(dict(base, name=f'reset-{encoding}-{packing}', pre=[0.5, 1, 2], reset=[True] * 3))
    return result


def audit_metadata(log, spec):
    """Validate current or recorded legacy labels with the same strict checks."""
    prefix = r'(?:COLOR|COLOUR)'
    if len(re.findall(r'^' + prefix + r'_SIGNAL PASSED frames=3$', log, re.M)) != 1:
        raise Rejected('Missing/duplicate signal completion marker')
    context = re.findall(r'^' + prefix + r'_CONTEXT flags=(\d+) encoding=(\w+) automatic=(true|false)', log, re.M)
    flags = 1 + (256 if spec['hint'] != 'linear' else 0) + (32 if spec['automatic'] else 0)
    if context != [(str(flags), spec['hint'], str(spec['automatic']).lower())]:
        raise Rejected('Creation metadata differs from declared arm')
    frame_metadata = re.findall(r'^' + prefix + r'_FRAME index=(\d+) pre=([\d.]+) exposure=([\d.]+) reset=(true|false) hint=(\w+)$', log, re.M)
    expected_hint = {'linear': 'Unspecified', 'unspecified': 'Unspecified', 'srgb': 'Srgb', 'pq': 'Pq'}[spec['hint']]
    resets = spec.get('reset', [True, False, False])
    expected_frames = [(str(i), float(spec['pre'][i]), float(spec['exposure'][i]), str(resets[i]).lower(), expected_hint) for i in range(3)]
    if [(i, float(p), float(e), reset, hint) for i, p, e, reset, hint in frame_metadata] != expected_frames:
        raise Rejected('Frame metadata differs from declared arm')
    if log.count('HDR input readback/guards: PASSED') != 3:
        raise Rejected('Missing full GPU input readback checks')
    providers = re.findall(r'^Upscaler provider: id=(0x[0-9a-f]+); name=(.+)$', log, re.M)
    if len(providers) != 1:
        raise Rejected('Missing/duplicate provider identity')
    return providers[0]


def cases(controls=False):
    # Existing case IDs are stable evidence identifiers, independent of naming.
    build = build_case('colour')
    selected = control_arms() if controls else arms()
    def cpu(r):
        try:
            import numpy
        except ImportError as error:
            raise Blocked('Existing NumPy required for colour-signal analysis') from error
        r.process([__import__('sys').executable, '-m', 'unittest', 'discover', '-s',
                   'tools/verification/tests', '-p', 'test_color_signal.py', '-v'])
        r.note(numpy=numpy.__version__, protocol=protocol())
    result = native_setup() + [Case('colour.cpu-oracle', 'Independent transfer/input and output negative controls', cpu), build]
    children = []
    neutral = []
    for spec in selected:
        for repetition in range(1, 4):
            name = f'colour.{spec["name"]}.r{repetition}'
            def child(r, spec=spec, name=name):
                directory = r.directory / 'readbacks' / name
                inputs, outputs = directory / 'input', directory / 'output'
                inputs.mkdir(parents=True); outputs.mkdir()
                input_records = []
                for frame, pre in enumerate(spec['pre']):
                    path = inputs / f'frame-{frame}.rgba16f'
                    payload = input_bytes(spec['encoding'], pre, spec['packing'])
                    path.write_bytes(payload)
                    recovered = audit_input(payload, spec['encoding'], pre, spec['packing'])
                    input_records.append(dict(frame=frame, sha256=sha(path), recovered_scene=recovered))
                env = environment(r)
                env.update(FSR_SDK_COLOR_ENCODING=spec['hint'], FSR_SDK_COLOR_AUTOMATIC=str(int(spec['automatic'])),
                           FSR_SDK_COLOR_PRE_EXPOSURE=','.join(map(str, spec['pre'])),
                           FSR_SDK_COLOR_EXPOSURE=','.join(map(str, spec['exposure'])),
                           FSR_SDK_COLOR_RESET=','.join(str(int(value)) for value in spec.get('reset', [True, False, False])),
                           FSR_SDK_COLOR_INPUT=inputs, FSR_SDK_COLOR_OUTPUT=outputs)
                _, log, _ = r.process([r.state[build.id], '--ignored', '--exact', SELECTOR, '--nocapture', '--test-threads=1'],
                                     cwd=r.state['native_stage'], env=env, markers=('1 passed; 0 failed',), timeout=180)
                provider = audit_metadata(log, spec)
                previous = r.state.setdefault('color.provider', provider)
                if previous != provider:
                    raise Rejected('Provider changed between probe arms')
                frames = []
                if len(list(outputs.glob('*.rgba16f'))) != 3:
                    raise Rejected('Unexpected output inventory')
                for i, pre in enumerate(spec['pre']):
                    path = outputs / f'frame-{i}.rgba16f'
                    stats = output_statistics(path.read_bytes())
                    frames.append(dict(frame=i, pre=pre, exposure=spec['exposure'][i], sha256=sha(path), statistics=stats,
                                       hypotheses=hypotheses(stats, spec['encoding'], pre, spec['exposure'][i], spec['packing'])))
                measurement = dict(arm=spec, provider=provider, inputs=input_records, frames=frames)
                save(r.directory / 'cases' / name / 'measurement.json', measurement)
                r.state[name] = measurement
                r.note(provider=provider, source='independent RGBA16F files; all GPU inputs checked after fencing',
                       signal_hypotheses=[{key: value['matches_declared_budget'] for key, value in frame['hypotheses'].items()} for frame in frames],
                       interpretation='Integration passed; numerical hypotheses are observations, not a blanket semantic acceptance')
            requires = (build.id, 'colour.cpu-oracle') + (tuple(neutral) if spec['phase'] != 'neutral' else ())
            result.append(Case(name, 'Declared colour signal and fenced readback: ' + spec['name'], child, requires,
                               SELECTOR, 'native-experiment', {'frames': 3, 'watchdog_seconds': 180, 'specification': spec,
                               'contract': 'Native integration required; frozen numerical predictions reported separately, including failures'}))
            children.append(name)
            if spec['phase'] == 'neutral':
                neutral.append(name)
    def analyze(r):
        analysis = dict(protocol=protocol(), controls_protocol=json.loads((HERE / 'color-signal-controls-protocol.json').read_text()) if controls else None,
                        measurements=[r.state[name] for name in children],
                        conclusion_policy='No internal operation ordering or universal encoding contract follows from output invariance alone')
        directory = r.directory / 'analysis'; directory.mkdir(exist_ok=True)
        save(directory / 'color-signal.json', analysis)
        r.note(arms=len(selected), processes=len(children), frames=len(children) * 3,
               semantic_acceptance='Not inferred from completed integration', internal_order='Requires an observable discriminant beyond restored output')
    result.append(Case('colour.analysis', 'Retain independent signal-domain predictions and all failures', analyze, tuple(children), scope='offline-analysis'))
    return result
