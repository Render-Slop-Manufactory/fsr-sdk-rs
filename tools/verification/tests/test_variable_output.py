# SPDX-License-Identifier: MPL-2.0
"""Controls for raw output geometry, independent readbacks and failed evidence."""
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from profiles import compose
from variable_output import allocation, evaluate, input_oracles, output_metrics, REGIONS, workloads, public_workloads, PUBLIC_SELECTOR


class OutputEvidence(unittest.TestCase):
    def test_public_profile_requires_both_owners_selected_providers_and_controls(self):
        cases=compose('native-variable-output')
        children=[c for c in cases if c.selector==PUBLIC_SELECTOR]
        self.assertEqual(len(public_workloads()),12)
        self.assertEqual(len(children),52)
        self.assertEqual(sum(c.expectation['control'] for c in children),4)
        self.assertEqual({c.expectation['provider_name'] for c in children},{'4.1.1','3.1.5'})
        for case in children:
            s=case.expectation['selection']
            self.assertEqual((s['allocation'],s['motion_allocation'],s['dynamic']),('maximum','maximum',True))
            if not case.expectation['control']:
                group='.'.join(case.id.split('.')[:3])
                self.assertIn(group+'.control',case.requires)
                if s['name']!='stable-max': self.assertIn(group+'.stable-max',case.requires)
        self.assertEqual({c.id.split('.')[2] for c in children},{'local','movable'})

    def test_bounded_profile_is_separate_and_requires_equipment_and_provider_controls(self):
        cases=compose('experiment-variable-output')
        self.assertEqual(len(workloads()),20)
        self.assertEqual(len({item['name'] for item in workloads()}),20)
        children=[case for case in cases if case.scope=='native-experiment']
        self.assertEqual(len(children),61)
        for case in children:
            if case.expectation['control']: continue
            self.assertIn('variable-output.control',case.requires)
            if case.expectation['selection']['name']!='stable-max':
                self.assertIn('.'.join(case.id.split('.')[:2])+'.stable-max',case.requires)
        for name in ('ci','msrv','native-upscaler','windows-upscaler-followup','release'):
            self.assertFalse(any(case.id.startswith('variable-output') for case in compose(name)))

    def fixture(self,directory,*,outside=False,control=False):
        selection=workloads()[3]
        lines=[]
        for phase,active in enumerate(REGIONS[selection['sequence']]):
            extent=allocation(selection['allocation'],active)
            desc=dict(phase=phase,active=active,allocation=extent,motion_allocation=(320,180),
                      motion_scale=(320,180),jitter=(0.25,-0.25),jitter_bits=(1048576000,3196059648),
                      reset=phase==0,dynamic=True,sharpen=False,motion_mode=0,control=control)
            for role,data in enumerate(input_oracles(desc)):
                (directory/f'phase-{phase}-role-{role}.bin').write_bytes(data)
            sentinel=struct.pack('<4H',*([0x7bff]*4)); written=struct.pack('<4H',*([0x3800]*4))
            content=b''.join((sentinel if control else written if outside or (x<active[0] and y<active[1]) else sentinel)
                             for y in range(extent[1]) for x in range(extent[0]))
            (directory/f'phase-{phase}-role-4.bin').write_bytes(content)
            frame=dict(output_metrics(content,active,extent),phase=phase,input_mismatches=0,debug_errors=0,removed=0,completed=1)
            lines += ['OUTPUT_DESCRIPTOR '+json.dumps(desc),'OUTPUT_FRAME '+json.dumps(frame)]
        lines += ['OUTPUT_RESULT '+json.dumps(dict(outcome='completed',frames=3,code=0,cleanup_ok=True)), '1 passed; 0 failed']
        return '\n'.join(lines),selection

    def test_padding_writes_are_accepted_and_input_corruption_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary)
            text,selection=self.fixture(directory,outside=True)
            command=dict(execution='completed',exit_code=0)
            result=evaluate(text,command,directory,selection)
            self.assertEqual(result['evaluation'],'passed')
            self.assertGreater(result['frames'][1]['metrics']['outside'],0)
            path=directory/'phase-0-role-2.bin'; content=bytearray(path.read_bytes()); content[0]^=1; path.write_bytes(content)
            with self.assertRaisesRegex(Rejected,'input oracle'):
                evaluate(text,command,directory,selection)

    def test_control_and_missing_or_wrong_geometry_cannot_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary)
            text,selection=self.fixture(directory,control=True)
            command=dict(execution='completed',exit_code=0)
            self.assertEqual(evaluate(text,command,directory,selection,control=True)['evaluation'],'passed')
            with self.assertRaises(Rejected): evaluate(text,command,directory,selection)
            with self.assertRaises(Rejected): evaluate(text.replace('"motion_scale": [320, 180]','"motion_scale": [640, 360]'),command,directory,selection,control=True)
            with self.assertRaises(Rejected): evaluate(text.replace('1 passed; 0 failed','0 passed; 0 failed'),command,directory,selection,control=True)

    def test_public_route_identity_is_required_and_rejection_cannot_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary)
            text,selection=self.fixture(directory,control=True)
            command=dict(execution='completed',exit_code=0)
            with self.assertRaisesRegex(Rejected,'route/family'):
                evaluate(text,command,directory,selection,control=True,public_family='local')
            route='OUTPUT_ROUTE '+json.dumps(dict(public=True,family='local'))+'\n'
            self.assertEqual(evaluate(route+text,command,directory,selection,control=True,public_family='local')['evaluation'],'passed')
            with self.assertRaisesRegex(Rejected,'route/family'):
                evaluate(route+text,command,directory,selection,control=True,public_family='movable')
            with self.assertRaisesRegex(Rejected,'route/family'):
                evaluate(route+route+text,command,directory,selection,control=True,public_family='local')
            rejected=route+'OUTPUT_RESULT '+json.dumps(dict(outcome='rejected',frames=0,code=7,cleanup_ok=True))+'\n1 passed; 0 failed'
            with self.assertRaisesRegex(Rejected,'recording was rejected'):
                evaluate(rejected,command,directory,selection,public_family='local')

    def test_render_pixel_motion_oracle_uses_active_render_units(self):
        desc=dict(phase=1,jitter_bits=(0,0),active=(511,287),motion_allocation=(320,180),motion_mode=3)
        motion=input_oracles(desc)[2]
        self.assertEqual(motion,struct.pack('<2e',-2.5,-1.40625)*(320*180))

    def test_rejection_is_observational_only_with_terminal_cleanup(self):
        selection=workloads()[0]
        desc=dict(phase=0,active=(640,360),allocation=(640,360),motion_allocation=(320,180),motion_scale=(320,180),
                  dynamic=True,sharpen=False,motion_mode=0,control=False,reset=True)
        with tempfile.TemporaryDirectory() as temporary:
            for cleanup in (True,False):
                text='OUTPUT_DESCRIPTOR '+json.dumps(desc)+'\nOUTPUT_RESULT '+json.dumps(dict(outcome='rejected',frames=0,code=7,cleanup_ok=cleanup))+'\n1 passed; 0 failed'
                if cleanup:
                    self.assertEqual(evaluate(text,dict(execution='completed',exit_code=0),Path(temporary),selection)['evaluation'],'observed')
                else:
                    with self.assertRaises(Rejected): evaluate(text,dict(execution='completed',exit_code=0),Path(temporary),selection)
            for execution,code in [('timed_out',None),('aborted',-1),('completed',1)]:
                with self.assertRaises(Rejected): evaluate('',dict(execution=execution,exit_code=code),Path(temporary),selection)

    def test_nonfinite_and_seeded_active_output_are_detected_independently(self):
        values=struct.pack('<4H',*([0x7bff]*4))
        self.assertEqual(output_metrics(values,(1,1),(1,1))['written'],0)
        values=struct.pack('<4H',0x7c00,0,0,0)
        self.assertEqual(output_metrics(values,(1,1),(1,1))['finite'],0)
