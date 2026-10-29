# SPDX-License-Identifier: MPL-2.0
"""Reject missing/duplicate benchmark evidence, independently of timing values."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from performance import samples, validate


class PerformanceEvidence(unittest.TestCase):
    def test_duplicate_and_nonfinite_rejected(self):
        with self.assertRaises(Rejected):
            samples('PERF_SAMPLE native 32 raw=1\nPERF_SAMPLE native 32 raw=2\n')
        with self.assertRaises(Rejected):
            samples('PERF_SAMPLE native 32 raw=1e999\n')

    def test_complete_matrix_required(self):
        log = ''.join(f'PERF_SAMPLE native {i} raw=100 wrapper=200 inspect=50\n' for i in range(32,160))
        log += ''.join(f'PERF_SAMPLE native_lifecycle {i} create=300 destroy=400\n' for i in range(8,39))
        matrix = samples(log)
        validate(matrix, False)
        del matrix['native.raw']['32']
        with self.assertRaises(Rejected):
            validate(matrix, False)

    def test_changed_indices_rejected(self):
        log = ''.join(f'PERF_SAMPLE controlled {i} empty=1 raw=2 inspect=3 wrapper=4\n' for i in range(31))
        log += ''.join(f'PERF_SAMPLE controlled_lifecycle {i} create=5 destroy=6\n' for i in range(31))
        matrix = samples(log)
        validate(matrix, True)
        matrix['controlled.raw']['31'] = matrix['controlled.raw'].pop('0')
        with self.assertRaises(Rejected):
            validate(matrix, True)
