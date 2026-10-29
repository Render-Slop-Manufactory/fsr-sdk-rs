# SPDX-License-Identifier: MPL-2.0
"""Signal, mask and inventory rejection controls for direct SDK experiments."""
import unittest

from bgra_srgb_fsr import canonical_output, color_codes, colors, linear, mask_check, workloads
from execution import Rejected


class BgraFsrAcceptance(unittest.TestCase):
    def test_public_profile_covers_owners_roles_and_uav_inputs(self):
        from bgra_srgb_public import workloads as public_workloads
        from profiles import compose
        selections = public_workloads()
        self.assertEqual(len(selections), 32)
        self.assertEqual(len({s['name'] for s in selections}), 32)
        cases = compose('native-bgra-srgb')
        self.assertEqual(len(cases), 102)
        for provider in ('3.1.5', '4.1.1'):
            for family in ('local', 'movable'):
                prefix = 'bgra-public.' + provider + '.' + family + '.'
                self.assertEqual(sum(c.id.startswith(prefix) for c in cases), 32 if provider == '3.1.5' else 14)
        for role in (0, 5):
            for fmt in (87, 29, 91):
                self.assertTrue(any(s['generate'] and s['formats'][role] == fmt and s['uav'] == 1 << role for s in selections))

    def test_physical_channel_order_and_linear_reference(self):
        rgba = colors(28, 0, 'unorm', False, False, False)
        bgra = colors(87, 0, 'unorm', False, False, False)
        self.assertEqual(rgba[:4], bytes((48, 128, 224, 255)))
        self.assertEqual(bgra[:4], bytes((224, 128, 48, 255)))
        self.assertAlmostEqual(linear(128, 'srgb'), 0.21586050011389926)
        self.assertAlmostEqual(linear(128, 'unorm'), 128 / 255)

    def test_output_channel_canonicalization_detects_swap(self):
        expected = bytes((48, 128, 224, 255)) * 640 * 360
        bgra = bytes((224, 128, 48, 255)) * 640 * 360
        self.assertEqual(canonical_output(bgra, 87), expected)
        self.assertNotEqual(canonical_output(expected, 87), expected)
        with self.assertRaises(Rejected):
            canonical_output(bgra[:-1], 87)

    def test_generator_checks_writes_without_choosing_mask_values(self):
        s = dict(zero=True, domain='unorm')
        for value in (0, 1, 64, 255):
            self.assertEqual(mask_check(bytes([value]) * 320 * 180, s, 1)['written'], 160 * 90)
        with self.assertRaises(Rejected):
            mask_check(bytes([0x55]) * 320 * 180, s, 1)

    def test_generator_only_requires_active_writes(self):
        s = dict(zero=False, domain='unorm')
        data = bytearray([0x55]) * 320 * 180
        for y in range(90):
            data[y * 320:y * 320 + 160] = bytes([123]) * 160
        self.assertEqual(mask_check(data, s, 1)['written'], 160 * 90)
        with self.assertRaises(Rejected):
            mask_check(data, s, 0)

    def test_inventory_separates_roles_and_keeps_zero_controls(self):
        selections = workloads()
        self.assertEqual(len(selections), 28)
        self.assertEqual(len({s['name'] for s in selections}), 28)
        for role in (0, 5):
            for fmt in (87, 29, 91):
                self.assertEqual(sum(s['generate'] and s['formats'][role] == fmt for s in selections), 2)
        self.assertEqual(color_codes(80, 45, 0, False, False)[0] - color_codes(80, 45, 0, True, False)[0], 64)


if __name__ == '__main__':
    unittest.main()
