# SPDX-License-Identifier: MPL-2.0
"""Independent oracle and known-defect controls for the bounded colour probe."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from color_signal import (HERE, SELECTOR, arms, audit_input, audit_metadata, control_arms,
                          decode_reference, encode, hypotheses, input_bytes,
                          output_statistics, protocol)
from execution import Rejected
try:
    import numpy as np
except ImportError:
    np = None


class TransferTests(unittest.TestCase):
    def test_standard_anchors_and_independent_inverse(self):
        self.assertAlmostEqual(encode(0.0031308, 'srgb'), 0.040449936, places=10)
        self.assertAlmostEqual(encode(0.18, 'srgb'), 0.46135612950044164, places=12)
        self.assertAlmostEqual(encode(1, 'pq'), 0.508078421517399, places=12)
        self.assertAlmostEqual(encode(10, 'pq'), 0.751827096247041, places=12)
        self.assertAlmostEqual(encode(100, 'pq'), 1, places=12)
        self.assertAlmostEqual(decode_reference(0.5, 'pq'), 0.92245708994065, places=10)
        for encoding in ('linear', 'srgb', 'pq'):
            for level in protocol()['levels'] + [1, 2]:
                self.assertAlmostEqual(decode_reference(encode(level, encoding), encoding), level, places=10)

    def test_outside_declared_domain_and_unknown_encodings_reject(self):
        for value in (-1, float('nan'), float('inf')):
            with self.assertRaises(ValueError): encode(value, 'srgb')
            with self.assertRaises(ValueError): decode_reference(value, 'pq')
        with self.assertRaises(ValueError): encode(0.2, 'unknown')
        with self.assertRaises(ValueError): decode_reference(0.2, 'unknown')

    def test_protocol_is_bounded_and_distinguishes_packing(self):
        self.assertEqual(protocol()['pq_reference_unit_nits'], 100)
        self.assertEqual((protocol()['frames'], protocol()['repetitions']), (3, 3))
        selected = arms()
        self.assertEqual((len(selected), len({arm['name'] for arm in selected})), (16, 16))
        self.assertEqual(sum(arm['phase'] == 'neutral' for arm in selected), 5)
        self.assertNotEqual(input_bytes('srgb', 2, 'linear-before-encode'), input_bytes('srgb', 2, 'encoded-after-encode'))
        self.assertEqual(input_bytes('linear', 2, 'linear-before-encode'), input_bytes('linear', 2, 'encoded-after-encode'))

    def test_followup_separates_static_metadata_from_resetting_transitions(self):
        selected = control_arms()
        self.assertEqual((len(selected), len({arm['name'] for arm in selected})), (14, 14))
        reset = [arm for arm in selected if 'reset' in arm]
        self.assertEqual(len(reset), 4)
        self.assertTrue(all(arm['reset'] == [True] * 3 and arm['pre'] == [0.5, 1, 2] for arm in reset))
        self.assertTrue(all(len(set(arm['pre'])) == 1 for arm in selected if 'reset' not in arm))


class CompatibilityTests(unittest.TestCase):
    def metadata(self, spec, prefix='COLOR'):
        flags = 1 + (256 if spec['hint'] != 'linear' else 0) + (32 if spec['automatic'] else 0)
        hint = {'linear': 'Unspecified', 'unspecified': 'Unspecified', 'srgb': 'Srgb', 'pq': 'Pq'}[spec['hint']]
        lines = [f'{prefix}_CONTEXT flags={flags} encoding={spec["hint"]} automatic={str(spec["automatic"]).lower()}',
                 'Upscaler provider: id=0x1; name=Fixture']
        for i, reset in enumerate(spec.get('reset', [True, False, False])):
            lines += [f'{prefix}_FRAME index={i} pre={spec["pre"][i]} exposure={spec["exposure"][i]} reset={str(reset).lower()} hint={hint}',
                      'HDR input readback/guards: PASSED']
        return '\n'.join(lines + [f'{prefix}_SIGNAL PASSED frames=3'])

    def test_current_and_legacy_logs_validate_all_declared_arms(self):
        for spec in arms() + control_arms():
            for prefix in ('COLOR', 'COLOUR'):
                with self.subTest(arm=spec['name'], prefix=prefix):
                    self.assertEqual(audit_metadata(self.metadata(spec, prefix), spec), ('0x1', 'Fixture'))

    def test_alias_support_does_not_accept_missing_duplicate_or_wrong_metadata(self):
        spec = arms()[0]
        good = self.metadata(spec)
        for broken in (good.replace('COLOR_SIGNAL PASSED frames=3', ''),
                       good + '\nCOLOUR_SIGNAL PASSED frames=3',
                       good + '\nCOLOUR_CONTEXT flags=1 encoding=linear automatic=false',
                       good.replace('flags=1 ', 'flags=257 '),
                       good.replace('index=0 ', 'index=1 '),
                       good.replace('reset=true', 'reset=false'),
                       good.replace('Upscaler provider:', 'Unknown provider:'),
                       good.replace('HDR input readback/guards: PASSED', '', 1)):
            with self.subTest(log=broken), self.assertRaises(Rejected):
                audit_metadata(broken, spec)

    def test_profile_aliases_have_identical_plans_and_preserve_stable_case_ids(self):
        from profiles import compose
        from run import parser
        for suffix, count in (('', 60), ('-controls', 54)):
            canonical = 'experiment-color-signal' + suffix
            legacy = 'experiment-colour-signal' + suffix
            current = [case.plan() for case in compose(canonical)]
            self.assertEqual(current, [case.plan() for case in compose(legacy)])
            self.assertEqual(len(current), count)
            seen = set()
            for case in current:
                self.assertTrue(set(case['requires']) <= seen)
                seen.add(case['id'])
                if case['id'].startswith('colour.') and case['selector']:
                    self.assertEqual(case['selector'], SELECTOR)
            self.assertIn('colour.analysis', seen)
            for name in (canonical, legacy):
                self.assertEqual(parser().parse_args(['plan', name]).profile, name)

    def test_protocol_aliases_preserve_frozen_bytes_and_base_reference(self):
        import json
        for suffix in ('', '-controls'):
            self.assertEqual((HERE / f'color-signal{suffix}-protocol.json').read_bytes(),
                             (HERE / f'colour-signal{suffix}-protocol.json').read_bytes())
        controls = json.loads((HERE / 'color-signal-controls-protocol.json').read_text())
        self.assertEqual(json.loads((HERE / controls['base_protocol']).read_text()), protocol())

    def test_legacy_module_forwards_to_the_same_implementation(self):
        import color_signal
        import colour_signal
        self.assertIs(colour_signal.cases, color_signal.cases)
        self.assertIs(colour_signal.audit_input, color_signal.audit_input)


@unittest.skipIf(np is None, 'Existing NumPy required for colour payload controls')
class PayloadTests(unittest.TestCase):
    def output(self, encoding):
        source = np.frombuffer(input_bytes(encoding), dtype='<f2').reshape(180, 320, 4)
        return np.repeat(np.repeat(source, 2, axis=0), 2, axis=1).tobytes()

    def test_independent_input_oracle_checks_both_packings_and_quantization(self):
        for encoding in ('linear', 'srgb', 'pq'):
            for packing in ('linear-before-encode', 'encoded-after-encode'):
                for pre in (0.5, 1, 2):
                    audit_input(input_bytes(encoding, pre, packing), encoding, pre, packing)
        with self.assertRaises(Rejected): audit_input(input_bytes('srgb'), 'pq', 1, 'linear-before-encode')
        with self.assertRaises(Rejected): audit_input(input_bytes('pq', 2), 'pq', 1, 'linear-before-encode')
        changed = bytearray(input_bytes('srgb')); changed[100:102] = b'\0\0'
        with self.assertRaises(Rejected): audit_input(changed, 'srgb', 1, 'linear-before-encode')

    def test_output_hypotheses_distinguish_code_and_decoded_domains(self):
        for encoding in ('srgb', 'pq'):
            measured = output_statistics(self.output(encoding))
            checks = hypotheses(measured, encoding, 1, 1, 'linear-before-encode')
            self.assertTrue(checks['supplied-code-domain']['matches_declared_budget'])
            self.assertFalse(checks['decoded-relative-linear']['matches_declared_budget'])
            wrong = output_statistics(self.output('linear'))
            self.assertFalse(hypotheses(wrong, encoding, 1, 1, 'linear-before-encode')['supplied-code-domain']['matches_declared_budget'])

    def test_nonfinite_sentinel_and_truncated_outputs_fail_outside_sampled_interiors(self):
        payload = self.output('srgb')
        for value in (float('nan'), 65504):
            image = np.frombuffer(payload, dtype='<f2').copy()
            image[0] = value
            with self.assertRaises(Rejected): output_statistics(image.tobytes())
        with self.assertRaises(Rejected): output_statistics(payload[:-2])
        zeros = np.zeros((360, 640, 4), dtype='<f2')
        self.assertFalse(hypotheses(output_statistics(zeros.tobytes()), 'srgb', 1, 1,
                                    'linear-before-encode')['supplied-code-domain']['matches_declared_budget'])


if __name__ == '__main__':
    unittest.main()
