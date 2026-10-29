# SPDX-License-Identifier: MPL-2.0
"""Negative controls for accepting GPU evidence versus mere native acknowledgement."""
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from reactive_mask import REGIONS, evaluate_probe
from profiles import compose


class ReactiveEvidence(unittest.TestCase):
    def test_selected_providers_require_their_own_equipment_controls(self):
        cases = compose('experiment-selected-reactive-mask')
        children = [case for case in cases if case.id.startswith(('reactive-selected.fsr2.', 'reactive-selected.fsr3.'))]
        self.assertEqual(len(children), 72)
        self.assertEqual({case.expectation['provider_name'] for case in children}, {'3.1.5', '2.3.4'})
        for case in children:
            if not case.expectation['control']:
                self.assertIn(case.id.rsplit('.', 1)[0] + '.control', case.requires)
                self.assertEqual(len(case.requires), 3)  # Toolchain, build, own control.

    def test_public_families_have_separate_controls_and_independent_flag_children(self):
        cases = compose('experiment-public-reactive-mask')
        for family in ('local', 'movable'):
            prefix = 'reactive-public.' + family + '.'
            children = [case for case in cases if case.id.startswith(prefix)]
            self.assertEqual(len(children), 18)
            for case in children:
                self.assertEqual(case.selector, 'upscaler::dx12::native_dispatch::public_reactive_mask_probe')
                self.assertEqual(case.expectation['family'], family)
                if not case.expectation['control']:
                    self.assertIn(prefix + 'control', case.requires)
                    self.assertFalse(any('flags-' in name for name in case.requires))

    def test_raw_children_are_independent_after_equipment_control(self):
        cases=compose('experiment-reactive-mask')
        flags=[case for case in cases if case.id.startswith('reactive-probe.flags-')]
        self.assertEqual(len(flags),16)
        self.assertEqual({case.expectation['flags'] for case in flags},set(range(16)))
        for case in flags:
            self.assertIn('reactive-probe.control',case.requires)
            self.assertFalse(any(name.startswith('reactive-probe.flags-') for name in case.requires))
        for name in ('ci','msrv','native-upscaler','windows-upscaler-followup','release'):
            self.assertFalse(any('reactive-probe' in case.id for case in compose(name)))

    def evidence(self, *, control=False, alias=False):
        summary=dict(outcome='control' if control else 'generated',flags=0,alias=alias,control=control,
                     frames=3,native_code=0,cleanup_ok=True)
        frames=[dict(phase=i,width=w,height=h,changed=0 if control else w*h,
                     zeros=0 if control else w*h if alias else w*h*3//4,outside_changed=0,
                     minimum=227 if control else 0,maximum=227 if control else 0 if alias else 128)
                for i,(w,h) in enumerate(REGIONS)]
        return summary,frames

    def evaluate(self,summary,frames,**changes):
        text='\n'.join('REACTIVE_FRAME '+json.dumps(f) for f in frames)
        text+='\nREACTIVE_PROBE '+json.dumps(summary)+'\n1 passed; 0 failed'
        return evaluate_probe(text,dict(execution='completed',exit_code=0,**changes),
                              flags=0,alias=summary['alias'],control=summary['control'])

    def test_write_and_equipment_controls_are_distinct(self):
        for control,alias in [(False,False),(True,False),(False,True)]:
            result=self.evaluate(*self.evidence(control=control,alias=alias))
            self.assertEqual(result['evaluation'],'passed')
            self.assertEqual(result['outcome'],'control' if control else 'generated')

    def test_native_rejection_never_passes_and_cleanup_failure_fails(self):
        for cleanup in [True,False]:
            summary,frames=self.evidence()
            summary.update(outcome='rejected',frames=1,native_code=7,cleanup_ok=cleanup)
            result=self.evaluate(summary,frames[:1])
            self.assertEqual(result['evaluation'],'observed' if cleanup else 'failed')
            self.assertEqual(result['cleanup_ok'],cleanup)

    def test_zero_code_and_missing_or_duplicate_frames_cannot_pass(self):
        summary,frames=self.evidence()
        for bad_summary,bad_frames in [(dict(summary,outcome='rejected'),frames),
                                      (summary,frames[:2]),(summary,frames+[frames[0]]),
                                      (dict(summary,cleanup_ok=False),frames)]:
            with self.assertRaises(Rejected): self.evaluate(bad_summary,bad_frames)

    def test_gpu_controls_reject_unwritten_mask_bad_region_and_nonzero_alias(self):
        for update in [dict(changed=0),dict(outside_changed=1),dict(width=319),
                       dict(zeros=-1),dict(minimum=-1),dict(changed=True)]:
            summary,frames=self.evidence()
            frames[0].update(update)
            with self.assertRaises(Rejected): self.evaluate(summary,frames)
        summary,frames=self.evidence(alias=True)
        frames[0]['maximum']=1
        with self.assertRaises(Rejected): self.evaluate(summary,frames)

    def test_crash_timeout_nonzero_exit_and_missing_test_are_not_observations(self):
        for execution,exit_code in [('timed_out',None),('aborted',-1),('completed',1)]:
            with self.assertRaises(Rejected):
                evaluate_probe('',dict(execution=execution,exit_code=exit_code),flags=0,alias=False,control=False)
