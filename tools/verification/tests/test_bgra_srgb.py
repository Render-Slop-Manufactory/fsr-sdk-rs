# SPDX-License-Identifier: MPL-2.0
"""Independent acceptance-boundary controls for the view experiment."""
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from bgra_srgb import COLORS, compare_floats
from execution import Rejected


class ViewAcceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.path = Path(self.temporary.name) / 'readback'

    def write(self, value):
        self.path.write_bytes(struct.pack('<4f', *value) * 128)

    def test_srgb_approximation_within_half_code_is_accepted(self):
        # Independent hardware-style approximation: deliberately outside the
        # discarded 2e-5 float threshold but inside the specified code budget.
        self.write((0.029541015625, 0.2158203125, 0.74609375, 160 / 255))
        result = compare_floats(self.path, COLORS[:1], True)
        self.assertGreater(result['maximum_absolute_error'], 0.00002)
        self.assertLess(result['maximum_srgb_code_error'], 0.5)

    def test_unorm_signal_cannot_pass_as_srgb_decoded(self):
        self.write(tuple(c / 255 for c in COLORS[0]))
        with self.assertRaises(Rejected):
            compare_floats(self.path, COLORS[:1], True)

    def test_srgb_error_outside_half_code_is_rejected(self):
        self.write((0.029541015625, 0.2158203125, 0.76, 160 / 255))
        with self.assertRaises(Rejected):
            compare_floats(self.path, COLORS[:1], True)

    def test_alpha_is_not_srgb_decoded(self):
        self.write((0.029541015625, 0.2158203125, 0.74609375, 0.3515))
        with self.assertRaises(Rejected):
            compare_floats(self.path, COLORS[:1], True)

    def test_stale_nonfinite_and_partial_readbacks_reject(self):
        for data in (b'\xa5' * 2048, struct.pack('<4f', float('nan'), 0, 0, 0) * 128, b'\0' * 16):
            self.path.write_bytes(data)
            with self.assertRaises(Rejected):
                compare_floats(self.path, COLORS[:1], False)

    def test_swapped_channel_control_rejects_normal_expectation(self):
        c = COLORS[0]
        self.write((c[2] / 255, c[1] / 255, c[0] / 255, c[3] / 255))
        compare_floats(self.path, COLORS[:1], False, swapped=True)
        with self.assertRaises(Rejected):
            compare_floats(self.path, COLORS[:1], False)


if __name__ == '__main__':
    unittest.main()
