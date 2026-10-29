# SPDX-License-Identifier: MPL-2.0
"""Negative controls for the public native scale/jitter verification contract."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from motion_scale import PROVIDERS
from profiles import compose
from public_motion import SELECTOR, assess, descriptor, encoding, evaluate, input_oracles, workloads
from variable_output import output_metrics


class PublicMotionEvidence(unittest.TestCase):
    def test_matrix_requires_both_families_providers_controls_and_all_cells(self):
        cases=compose('native-motion-scale')
        self.assertEqual(len(cases),102)
        children=[c for c in cases if c.selector==SELECTOR]
        self.assertEqual(len(children),88)
        self.assertEqual(sum(c.expectation['control'] for c in children),4)
        self.assertEqual({c.expectation['family'] for c in children},{'local','movable'})
        self.assertEqual({c.expectation['provider_name'] for c in children},set(PROVIDERS))
        for case in children:
            if not case.expectation['control']:
                self.assertIn('.'.join(case.id.split('.')[:3])+'.control',case.requires)
        for name in ('ci','msrv','release','native-upscaler'):
            self.assertFalse(any(c.selector==SELECTOR for c in compose(name)))

    def test_jitter_is_encoded_before_axis_scaling_and_wrong_case_is_distinct(self):
        items={s['name']:s for s in workloads()}
        self.assertEqual(encoding(items['render-jitter-reference'],1)['stored'],(-3.5,-2.5))
        self.assertEqual(encoding(items['render-jitter-mixed'],1)['stored'],(1.75,-0.625))
        self.assertEqual(encoding(items['render-jitter-jitter-wrong'],1)['stored'],(2.5,-1.0))
        for phase in range(3):
            self.assertEqual(input_oracles(items['render-mixed'],phase),input_oracles(items['render-wrong-x'],phase))
            self.assertEqual(input_oracles(items['render-mixed'],phase),input_oracles(items['render-wrong-y'],phase))
        self.assertEqual(encoding(items['dynamic-uav-mixed'],1)['active'],(512,288))
        self.assertEqual(encoding(items['dynamic-uav-mixed'],1)['motion_allocation'],(640,360))
        self.assertEqual(encoding(items['dynamic-uav-mixed'],1)['scale'],(-2.0,4.0))
        self.assertFalse(descriptor(items['dynamic-uav-clear'],1,'4.1.1','local',False)['override'])

    def test_insensitive_controls_and_bad_jitter_equivalence_cannot_pass(self):
        plain=struct.pack('<4e',0.5,0.5,0.5,1.0)*(640*360)
        changed=struct.pack('<4e',0.75,0.5,0.5,1.0)+plain[8:]
        sensitive=lambda name,p: changed if p and any(name.endswith(v) for v in ('wrong-x','wrong-y','jitter-wrong')) else plain
        self.assertTrue(assess(sensitive)['passed'])
        self.assertFalse(assess(lambda name,p:plain)['passed'])
        failed=assess(lambda name,p:changed if name=='render-jitter-mixed' and p else sensitive(name,p))
        self.assertFalse(failed['passed'])
        self.assertFalse(failed['criteria']['render-jitter-mixed'])

    def test_independent_audit_rejects_bits_family_capability_fence_and_input_corruption(self):
        selection=next(s for s in workloads() if s['name']=='render-jitter-mixed')
        command=dict(execution='completed',exit_code=0)
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary)
            lines=['1 passed; 0 failed',f'Public motion provider: ProviderInfo {{ id: ProviderId({PROVIDERS["4.1.1"]}), name: "4.1.1" }}']
            for phase in range(3):
                desc=descriptor(selection,phase,'4.1.1','local',False)
                for role,data in enumerate(input_oracles(selection,phase)):
                    (directory/f'phase-{phase}-role-{role}.bin').write_bytes(data)
                output=struct.pack('<4e',0.5,0.5,0.5,1.0)*(640*360)
                (directory/f'phase-{phase}-role-4.bin').write_bytes(output)
                frame=dict(output_metrics(output,(640,360),(640,360)),phase=phase,input_mismatches=0,debug_errors=0,removed=0,completed=1)
                lines += ['PUBLIC_MOTION_CAPS '+json.dumps(dict(support1=0x320,support2=0)),
                          'PUBLIC_MOTION_DESCRIPTOR '+json.dumps(desc),'PUBLIC_MOTION_FRAME '+json.dumps(frame)]
            lines += ['PUBLIC_MOTION_RESULT '+json.dumps(dict(outcome='completed',frames=3,cleanup_ok=True))]
            text='\n'.join(lines)
            self.assertEqual(len(evaluate(text,command,directory,selection,'4.1.1','local')['frames']),3)
            for modified in (text+'\n'+lines[-1],text.replace('"family": "local"','"family": "movable"'),
                             text.replace('"support1": 800','"support1": 0'),text.replace('"completed": 1','"completed": 0'),
                             text.replace('"scale_bits": [3221225472, 1082130432]','"scale_bits": [0, 0]'),
                             text.replace('"override": true','"override": false')):
                with self.assertRaises(Rejected): evaluate(modified,command,directory,selection,'4.1.1','local')
            (directory/'phase-1-role-2.bin').write_bytes(b'corrupt')
            with self.assertRaises(Rejected): evaluate(text,command,directory,selection,'4.1.1','local')


if __name__=='__main__': unittest.main()
