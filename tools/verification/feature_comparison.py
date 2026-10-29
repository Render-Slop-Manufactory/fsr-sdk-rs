# SPDX-License-Identifier: MPL-2.0
"""Selected-feature comparison with independent full mask and HDR input checks."""
from functools import lru_cache
import json
import math
from pathlib import Path
import re
import struct

from active_timing import parse as parse_active, protocol as base_protocol, summarize as summarize_active
from execution import Rejected
from performance import distribution

HERE = Path(__file__).resolve().parent


def selected_protocol():
    return base_protocol() | json.loads((HERE/'feature-comparison-protocol.json').read_text())


def fnv(data):
    result = 14695981039346656037
    for byte in data:
        result = ((result ^ byte)*1099511628211) & (2**64-1)
    return result


@lru_cache(maxsize=8)
def mask_hashes(width, height):
    # Integer inequalities avoid reproducing the C++ floating-point generator.
    reactive = bytes(128 if 4*x+2 > width and 2*x+1 < width else 0 for x in range(width))
    transparency = b''.join(bytes([255 if 2*y+1 > height and 4*y+2 < 3*height else 0])*width
                            for y in range(height))
    return [fnv(reactive*height), fnv(transparency)]


def parse(log, *, arm, feature, debug=False):
    if feature not in ('masks', 'hdr'):
        raise Rejected('Unknown selected feature')
    measured = parse_active(log, arm=arm, mode='max', debug=debug, hdr=feature=='hdr', masks=feature=='masks')
    rows = re.findall(r'^FEATURE_VALID (\d+) (\d+) (\d+) (\d+) (\d+)$', log, re.M)
    frames = [row[0] for row in measured['checkpoints']]
    if len(rows) != len(frames) or sorted(int(row[0]) for row in rows) != frames:
        raise Rejected('Missing/duplicate selected-feature checkpoint')
    values = [list(map(int, row)) for row in sorted(rows, key=lambda row: int(row[0]))]
    expected_hashes = mask_hashes(*measured['query']['render']) if feature == 'masks' else [0, 0]
    for _, kind, max_bits, *hashes in values:
        if kind != (1 if feature == 'masks' else 2) or hashes != expected_hashes or max_bits >= 2**32:
            raise Rejected('Wrong feature identity/full mask hashes')
        maximum = struct.unpack('<f', struct.pack('<I', max_bits))[0]
        if not math.isfinite(maximum) or (feature == 'hdr' and not 1 < maximum < 64):
            raise Rejected('Invalid/clipped HDR checkpoint')
    for index in range(2):
        count = len(re.findall(r'^Supplied mask input readback: PASSED index='+str(index)+r'$', log, re.M))
        if count != (len(frames) if feature == 'masks' else 0):
            raise Rejected('Missing/extra full mask readback confirmation')
    measured.update(mode=feature, feature_checkpoints=values)
    return measured


def summarize(processes, spec):
    controls = summarize_active(processes, base_protocol() | {'blocks': spec['blocks']})
    blocks = []
    for block in range(1, spec['blocks']+1):
        values = {}
        for mode in spec['workloads']:
            values[mode] = {}
            for arm in ('cpp-minimal', 'rust-wrapper'):
                child = processes[f'active.block{block}.{mode}.{arm}']
                values[mode][arm] = {clock: distribution(child[key]) for clock, key in (('cpu', 'measured_ns'), ('gpu', 'gpu_ns'))}
                if mode in ('masks', 'hdr'):
                    baseline = processes[f'active.block{block}.max.{arm}']
                    for key in ('provider', 'adapter_driver', 'frequency_hz', 'gpu_frequency_hz', 'query', 'smaller', 'active_frames', 'jitter_bits'):
                        if child[key] != baseline[key]:
                            raise Rejected('Feature differs from matched SDR equipment/history: '+key)
                    if mode == 'masks' and (child['input_samples'] != baseline['input_samples'] or
                        [row[5] for row in child['checkpoints']] != [row[5] for row in baseline['checkpoints']]):
                        raise Rejected('Masked colour/depth/motion inputs differ from SDR control')
        blocks.append({'block': block, 'distributions': values,
            'wrapper_minus_cpp_median_ns': {mode: {clock: values[mode]['rust-wrapper'][clock]['median_ns']-values[mode]['cpp-minimal'][clock]['median_ns']
                                                  for clock in ('cpu', 'gpu')} for mode in spec['workloads']},
            'feature_minus_sdr_median_ns': {mode: {arm: {clock: values[mode][arm][clock]['median_ns']-values['max'][arm][clock]['median_ns']
                                                        for clock in ('cpu', 'gpu')} for arm in ('cpp-minimal', 'rust-wrapper')}
                                           for mode in ('masks', 'hdr')}})
    return {'protocol': spec, 'active_controls': controls, 'blocks': blocks, 'performance_threshold': None,
            'limit': 'Descriptive independent process blocks; correlated frames, uncontrolled clocks and diagnostic gaps; no speed-parity claim.'}
