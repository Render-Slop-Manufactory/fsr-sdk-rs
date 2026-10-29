# SPDX-License-Identifier: MPL-2.0
"""Negative controls for motion evidence and scale-consumption hypotheses."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from motion_scale import SELECTOR, PROVIDERS, assess, bits, evaluate, input_oracles, workloads
from profiles import compose


class MotionEvidence(unittest.TestCase):
    def test_profile_is_bounded_opt_in_and_comparison_requires_every_child(self):
        cases = compose('experiment-motion-scale')
        children = [c for c in cases if c.selector == SELECTOR]
        self.assertEqual(len(children), 31)
        self.assertEqual(sum(c.expectation['control'] for c in children), 1)
        self.assertEqual({c.expectation['provider_name'] for c in children}, set(PROVIDERS))
        for case in children:
            if not case.expectation['control']:
                self.assertIn('motion-scale.control', case.requires)
        comparisons = [c for c in cases if c.scope == 'native-analysis']
        self.assertEqual([len([p for p in c.requires if p.startswith('motion-scale.')]) for c in comparisons], [15,15])
        for name in ('ci','msrv','native-upscaler','windows-upscaler-followup','release'):
            self.assertFalse(any(c.id.startswith('motion-scale') for c in compose(name)))

    def test_oracle_distinguishes_width_sign_and_equivalent_products(self):
        selections = workloads()
        for axis in range(3):
            arms = selections[axis*5:axis*5+5]
            expected = tuple(a*b for a,b in zip(arms[0]['stored'], arms[0]['scale']))
            for arm in arms[1:4]:
                self.assertEqual(tuple(a*b for a,b in zip(arm['stored'], arm['scale'])), expected)
            self.assertEqual(input_oracles(arms[3], 1), input_oracles(arms[4], 1))
            self.assertEqual(len(input_oracles(arms[0], 1)[2]), 320*180*4)
            self.assertEqual(len(input_oracles(arms[1], 1)[2]), 320*180*8)
        self.assertNotEqual(input_oracles(selections[0], 1)[0], input_oracles(selections[5], 1)[0])

    def test_insensitive_outputs_and_sign_or_equivalence_errors_remain_observed(self):
        plain = struct.pack('<4e', 0.5,0.5,0.5,1.0)*(640*360)
        changed = struct.pack('<4e', 0.75,0.5,0.5,1.0)+plain[8:]
        self.assertFalse(assess(lambda axis,arm,phase: plain)['hypotheses_passed'])
        self.assertTrue(assess(lambda axis,arm,phase: changed if arm == 'wrong' and phase else plain)['hypotheses_passed'])
        failed = assess(lambda axis,arm,phase: changed if arm in ('wrong','signed') and phase else plain)
        self.assertFalse(failed['hypotheses_passed'])
        self.assertFalse(failed['hypotheses']['axis-0-signed-equivalent'])

    def test_evidence_rejects_wrong_bits_identity_input_or_duplicate_completion(self):
        selection = workloads()[1]
        command = dict(execution='completed', exit_code=0)
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            lines = ['1 passed; 0 failed', f'Motion probe provider: ProviderInfo {{ id: ProviderId({PROVIDERS["4.1.1"]}), name: "4.1.1" }}']
            for phase in range(4):
                desc = dict(phase=phase, provider='4.1.1', axis=0, format=1,
                            scale_bits=list(map(bits,selection['scale'])), stored_bits=list(map(bits,selection['stored'])),
                            reset=phase==0, control=False)
                for role,data in enumerate(input_oracles(selection,phase)):
                    (directory/f'phase-{phase}-role-{role}.bin').write_bytes(data)
                (directory/f'phase-{phase}-role-4.bin').write_bytes(struct.pack('<4e',0.5,0.5,0.5,1.0)*(640*360))
                frame = dict(phase=phase, finite=640*360, written=640*360, outside=0, right=640, bottom=360,
                             input_mismatches=0, debug_errors=0, removed=0, completed=1)
                lines += ['MOTION_DESCRIPTOR '+json.dumps(desc), 'MOTION_FRAME '+json.dumps(frame)]
            lines += ['MOTION_RESULT '+json.dumps(dict(outcome='completed', frames=4, code=0, cleanup_ok=True))]
            text = '\n'.join(lines)
            self.assertEqual(len(evaluate(text,command,directory,selection,'4.1.1')['frames']),4)
            for modified in (text+'\n'+lines[-1], text.replace('ProviderId(', 'WrongId('),
                             text.replace('"scale_bits": [1065353216, 1065353216]', '"scale_bits": [0, 0]'),
                             text.replace('"completed": 1', '"completed": 0')):
                with self.assertRaises(Rejected):
                    evaluate(modified,command,directory,selection,'4.1.1')
            (directory/'phase-1-role-2.bin').write_bytes(b'wrong')
            with self.assertRaises(Rejected):
                evaluate(text,command,directory,selection,'4.1.1')


if __name__ == '__main__':
    unittest.main()
