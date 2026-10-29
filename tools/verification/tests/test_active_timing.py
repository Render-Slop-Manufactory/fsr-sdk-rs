# SPDX-License-Identifier: MPL-2.0
"""Active-size evidence must reject wrong histories even when timings look good."""
import copy
import json
import struct
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from active_timing import bits, compatible, input_expected, parse, protocol, size_at, summarize
from cpu_comparison import expected_spec
from execution import Rejected
from profiles import compose
from test_gpu_timing import log as gpu_log


def log(mode='transition', debug=False):
    spec = protocol()
    text = gpu_log(debug).replace(json.dumps(expected_spec((1280, 720), (1920, 1080))),
                                json.dumps(expected_spec((1280, 720), (1920, 1080), dynamic=True)))
    text += f'ACTIVE_QUERY {mode} 960 540 32\n'
    for frame in range(4 if debug else 160):
        w, h = size_at(mode, frame, (1280, 720), (960, 540), debug)
        phase = 32 if w == 960 else 18
        text += f'ACTIVE_FRAME {frame} {w} {h} {phase} {int(frame == 0)} {bits(w)} {bits(h)}\n'
    for frame in spec['debug_checkpoints'] if debug else spec['checkpoints']:
        errors = 0 if debug else 2**64-1
        text += f'ACTIVE_VALID {frame} 2073600 2073600 {errors} 12345 67890\nActive render input readback/guards: PASSED\n'
        w, h = size_at(mode, frame, (1280, 720), (960, 540), debug)
        points = [(0, 0), (w-1, h-1)] + ([(w, h)] if w < 1280 else [])
        for x, y in points:
            values = input_expected(frame, x, y, w, h, 1280, (0, 0))
            values = [bits(value) if channel == 4 else struct.unpack('<H', struct.pack('<e', value))[0]
                      for channel, value in enumerate(values)]
            text += f'ACTIVE_INPUT {frame} {x} {y} ' + ' '.join(map(str, values)) + '\n'
    return text


class ActiveTimingEvidence(unittest.TestCase):
    def test_controls_and_transition_sequences(self):
        for mode in ('max', 'small', 'transition'):
            for debug in (False, True):
                result = parse(log(mode, debug), arm='cpp-minimal', mode=mode, debug=debug)
                self.assertEqual(result['spec']['creation'][4], 64)
                self.assertEqual(len(result['checkpoints']), 4 if debug else 6)
        result = parse(log(), arm='cpp-minimal', mode='transition')
        self.assertEqual([result['active_frames'][i][:2] for i in (63, 64, 111, 112)],
                         [[1280, 720], [960, 540], [960, 540], [1280, 720]])

    def test_wrong_size_scale_reset_and_flags_rejected(self):
        for old, new in (
            ('ACTIVE_FRAME 64 960 540 32 0', 'ACTIVE_FRAME 64 960 540 32 1'),
            ('ACTIVE_FRAME 64 960 540', 'ACTIVE_FRAME 64 1280 720'),
            (f'0 {bits(960)} {bits(540)}', f'0 {bits(1280)} {bits(720)}'),
            ('ACTIVE_QUERY transition 960 540 32', 'ACTIVE_QUERY transition 1280 540 32'),
            ('ACTIVE_FRAME 63 1280 720 18', 'ACTIVE_FRAME 63 1280 720 0'),
            ('"creation": [65536, 65547, 16781313, 2, 64,', '"creation": [65536, 65547, 16781313, 2, 0,')
        ):
            changed = log().replace(old, new)
            self.assertNotEqual(changed, log())
            with self.subTest(new=new), self.assertRaises(Rejected):
                parse(changed, arm='cpp-minimal', mode='transition')

    def test_missing_duplicate_guards_and_bad_output_rejected(self):
        base = log()
        for text in (base+'ACTIVE_QUERY transition 960 540 32\n',
                     base+'ACTIVE_FRAME 0 1280 720 18 1 0 0\n',
                     base.replace(f'ACTIVE_VALID 64 2073600 2073600 {2**64-1} 12345 67890\n', ''),
                     base.replace(f'ACTIVE_VALID 64 2073600 2073600 {2**64-1}', 'ACTIVE_VALID 64 2073600 2073600 1'),
                     base.replace('Active render input readback/guards: PASSED\n', '', 1),
                     base.replace('ACTIVE_VALID 64 2073600', 'ACTIVE_VALID 64 1')):
            with self.assertRaises(Rejected): parse(text, arm='cpp-minimal', mode='transition')

    def test_jitter_repetition_uses_current_size_phase(self):
        base = log()
        # Frame 64 restarts index 0 of the 32-sample phase, not index 10 of 18.
        changed = base.replace('GPU_JITTER 64 0 0', 'GPU_JITTER 64 1 0')
        with self.assertRaises(Rejected): parse(changed, arm='cpp-minimal', mode='transition')
        changed = base.replace('GPU_JITTER 112 0 0', 'GPU_JITTER 112 1 0')
        with self.assertRaises(Rejected): parse(changed, arm='cpp-minimal', mode='transition')

    def test_cross_consumer_input_and_output_audits_match(self):
        result = parse(log(), arm='cpp-minimal', mode='transition')
        result['output_sha256'] = 'a'*64
        compatible(result, copy.deepcopy(result))
        for key in ('checkpoints', 'active_frames', 'smaller'):
            changed = copy.deepcopy(result)
            changed[key] = []
            with self.assertRaises(Rejected): compatible(result, changed)

    def test_analysis_keeps_first_frames_separate_and_signed(self):
        processes = {}
        for block in range(1, 5):
            for mode in ('max', 'small', 'transition'):
                for arm in ('cpp-minimal', 'rust-wrapper'):
                    result = parse(log(mode), arm='cpp-minimal', mode=mode)
                    if mode == 'transition': result['ticks']['frame'][64] = 100
                    processes[f'active.block{block}.{mode}.{arm}'] = result
        summary = summarize(processes, protocol())
        shrink = summary['blocks'][0]['segments']['first_shrink']
        self.assertEqual(shrink['stable_control'], 'small')
        self.assertEqual(shrink['distributions']['transition']['rust-wrapper']['cpu']['samples'], 1)
        self.assertEqual(shrink['transition_minus_stable_median_ns']['rust-wrapper']['cpu'], -30000)
        self.assertIsNone(summary['performance_threshold'])
        changed = copy.deepcopy(processes)
        changed['active.block1.transition.cpp-minimal']['checkpoints'][2][5] += 1
        with self.assertRaises(Rejected): summarize(changed, protocol())
        changed = copy.deepcopy(processes)
        changed['active.block1.transition.cpp-minimal']['checkpoints'][1][4] += 1
        with self.assertRaises(Rejected): summarize(changed, protocol())

    def test_independent_oracle_rejects_wrong_motion_guard_and_scene(self):
        base = log()
        line = next(line for line in base.splitlines() if line.startswith('ACTIVE_INPUT 64 0 0 '))
        fields = line.split()
        for index, value in ((9, '0'), (9, str(int(fields[9]) ^ 32768)), (4, '0'), (8, '0')):
            wrong = fields.copy()
            wrong[index] = value
            with self.subTest(index=index), self.assertRaises(Rejected):
                parse(base.replace(line, ' '.join(wrong)), arm='cpp-minimal', mode='transition')
        guard = next(line for line in base.splitlines() if line.startswith('ACTIVE_INPUT 64 960 540 '))
        with self.assertRaises(Rejected):
            parse(base.replace(guard, guard.replace('31743', '0', 1)), arm='cpp-minimal', mode='transition')

    def test_profile_has_balanced_blocks_and_no_ordinary_profile_selection(self):
        cases = compose('experiment-active-timing')
        known = set()
        for case in cases:
            self.assertTrue(set(case.requires) <= known)
            self.assertNotIn(case.id, known)
            known.add(case.id)
        self.assertEqual(len(cases), 42)
        self.assertEqual(sum('.diagnostics.' in case.id for case in cases), 6)
        self.assertEqual(sum('.block' in case.id for case in cases), 24)
        for name in ('ci', 'msrv', 'native-upscaler', 'release'):
            self.assertFalse(any(case.id.startswith('active.') for case in compose(name)))
