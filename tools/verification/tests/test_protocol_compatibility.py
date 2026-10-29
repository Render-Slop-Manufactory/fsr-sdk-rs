# SPDX-License-Identifier: MPL-2.0
"""Current native labels and frozen log formats retain the same audit gates."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from analysis import audit_sharpening_integration, audit_transfer
from records import save


class ProtocolCompatibility(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def child(self, case, lines):
        directory = self.root / 'cases' / case
        directory.mkdir(parents=True, exist_ok=True)
        stdout = directory / 'stdout.txt'
        stderr = directory / 'stderr.txt'
        stdout.write_text('1 passed; 0 failed\n', encoding='utf-8')
        stderr.write_text('\n'.join(lines) + '\n', encoding='utf-8')
        save(directory / 'result.json', {'commands': [{
            'execution': 'completed', 'exit_code': 0,
            'stdout': stdout.relative_to(self.root).as_posix(),
            'stderr': stderr.relative_to(self.root).as_posix(),
        }]})
        return stderr

    def sharpening(self, legacy=False):
        label = 'M9' if legacy else 'SHARPENING'
        for repeat in range(1, 4):
            case = f'native.sharpening.integration.r{repeat}'
            directory = self.root / 'readbacks' / case
            directory.mkdir(parents=True, exist_ok=True)
            lines = []
            for scene in range(2):
                for arm in ('switch-a', 'switch-b', 'disabled'):
                    for frame in range(8):
                        # These tests exercise log compatibility; pixel decoding
                        # and independent metric calculations have their own tests.
                        (directory / f'scene-{scene}-{arm}-frame-{frame}.red.u16le').write_bytes(b'\x00\x3c')
                        lines.append(f'{label}_FRAME scene={scene} arm={arm} frame={frame} '
                                     'finish_hr=0x0 fence=1 width=640 height=360 finite=230400 '
                                     'written=230400 debug_errors=0 device_removed=0x0')
                for frame in range(8):
                    for pair in ('a-b', 'a-disabled'):
                        if pair == 'a-disabled' and frame < 3:
                            continue
                        lines.append(f'{label}_COMPARE scene={scene} frame={frame} pair={pair} '
                                     'roi_bit_equal=true roi_bit_mismatches=0 roi_mae_f64=0')
            lines.append(('M9' if legacy else 'Sharpening') + ' integration: PASSED outputs=48')
            self.child(case, lines)

    def audit_sharpening(self):
        # Isolate protocol/negative-control coverage from expensive image arrays.
        with patch('analysis.red', return_value=None), patch('analysis.compare', return_value={
                'roi_bit_equal': True, 'roi_bit_mismatches': 0, 'roi_mae_f64': 0}):
            return audit_sharpening_integration(self.root)

    def test_sharpening_accepts_current_and_historical_logs_without_mutating_them(self):
        for legacy in (False, True):
            with self.subTest(legacy=legacy):
                self.sharpening(legacy)
                before = {p: p.read_bytes() for p in self.root.rglob('stderr.txt')}
                result = self.audit_sharpening()
                self.assertEqual((result['outputs'], len(result['frames']), len(result['comparisons'])),
                                 (144, 144, 78))
                self.assertEqual({p: p.read_bytes() for p in before}, before)

    def test_sharpening_rejects_duplicate_legacy_and_current_observations(self):
        self.sharpening()
        path = self.root / 'cases/native.sharpening.integration.r1/stderr.txt'
        text = path.read_text(encoding='utf-8')
        path.write_text(text + text.splitlines()[0].replace('SHARPENING_', 'M9_') + '\n', encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'duplicate frame'):
            self.audit_sharpening()

    def test_sharpening_rejects_changed_frame_or_comparison_evidence_in_both_formats(self):
        for legacy in (False, True):
            for old, new in (('fence=1', 'fence=0'), ('roi_bit_mismatches=0', 'roi_bit_mismatches=1')):
                with self.subTest(legacy=legacy, field=old):
                    self.sharpening(legacy)
                    path = self.root / 'cases/native.sharpening.integration.r1/stderr.txt'
                    path.write_text(path.read_text(encoding='utf-8').replace(old, new, 1), encoding='utf-8')
                    with self.assertRaises(ValueError):
                        self.audit_sharpening()

    def transfer(self, legacy=False):
        matrix = [(label, '', '', count) for label, count in (
            ('create-after-transfer', 1), ('history', 6), ('independent-siblings', 5),
            ('unwind', 1), ('sibling-unwind', 3))]
        frames = dict(zip((row[0] for row in matrix), (1, 64, 17, 1, 3)))
        prefix = 'D013-2' if legacy else 'Transfer cleanup'
        destroy = 'D013' if legacy else 'Context'
        for label, _, _, _ in matrix:
            for arm in ('control', 'transfer'):
                for repeat in range(1, 4):
                    lines = [f'{prefix} fenced context=0 frame={frame} completed=1 metrics='
                             'finite_pixels: 230400, changed_pixels: 230400, debug_errors: 0, device_removed: 0'
                             for frame in range(frames[label])]
                    lines += ['residency checkpoint=1-live helper=true',
                              f'{destroy} native destroy: thread=ThreadId(1) code=0x00000000']
                    lines += [f'residency checkpoint={checkpoint}-done runtime_strong=Some(0) '
                              'loader=false provider=false helper=false' for checkpoint in (5, 6)]
                    self.child(f'experiment.transfer-cleanup.{label}-{arm}-r{repeat}', lines)
        return matrix

    def test_transfer_accepts_both_formats_with_the_same_complete_frame_matrix(self):
        for legacy in (False, True):
            with self.subTest(legacy=legacy):
                matrix = self.transfer(legacy)
                result = audit_transfer(self.root, 'experiment.transfer-cleanup', matrix)
                self.assertEqual(len(result['outcomes']), 30)
                self.assertEqual(sum(row['fenced_frames'] for row in result['outcomes']), 516)

    def test_transfer_rejects_failed_cleanup_and_incomplete_frames_in_both_formats(self):
        for legacy in (False, True):
            for old, new in (('completed=1', 'completed=0'),
                             ('code=0x00000000', 'code=0x00000001'),
                             ('helper=false', 'helper=true')):
                with self.subTest(legacy=legacy, field=old):
                    matrix = self.transfer(legacy)
                    path = self.root / 'cases/experiment.transfer-cleanup.history-transfer-r1/stderr.txt'
                    path.write_text(path.read_text(encoding='utf-8').replace(old, new, 1), encoding='utf-8')
                    with self.assertRaises(ValueError):
                        audit_transfer(self.root, 'experiment.transfer-cleanup', matrix)
