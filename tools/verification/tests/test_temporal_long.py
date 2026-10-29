# SPDX-License-Identifier: MPL-2.0
"""Duration boundaries, continuous inputs and time-local failure discrimination."""
import copy
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))


class LongPlanTests(unittest.TestCase):
    def test_default_plan_preserved_and_long_plan_bounded(self):
        from temporal_diagnosis import frame_plan, protocol as diagnosis
        from temporal_long import protocol, windows
        self.assertEqual(frame_plan()[1],[0,1,31,32,63])
        spec=protocol(); full,inputs=frame_plan(spec)
        self.assertEqual(len(full),256); self.assertEqual(len(inputs),17)
        self.assertEqual([f for start,end in windows(spec) for f in range(start,end+1)],list(range(32,256)))
        for count in (32,65,257,288,64.0,True):
            with self.assertRaises(ValueError): frame_plan(dict(diagnosis(),frames=count))
        with self.assertRaises(ValueError): frame_plan(dict(spec,input_only_frames=inputs[:-1]))
        with self.assertRaises(ValueError): windows(dict(spec,windows=[[32,63],[96,255]]))
        fast=copy.deepcopy(spec); fast['scenes'][0]['velocity']=[2,1]
        with self.assertRaises(ValueError): windows(fast)

    def test_long_profile_and_retention_are_explicit(self):
        from profiles import compose
        from retention import calibration_diagnostic
        from temporal_quality import protocol as quality
        from temporal_long import protocol
        self.assertEqual(protocol()['native_budgets'],quality()['native_budgets'])
        self.assertEqual(protocol()['minimum_motion_ratio'],1.1)
        cases=compose('experiment-temporal-long'); ids={c.id for c in cases}
        self.assertTrue({'experiment.temporal-long.'+name for name in ('evaluator','input-only','r1','r2','r3','audit')}<=ids)
        for name in ('fast','ci','release','native-upscaler'):
            self.assertFalse(any('temporal-long' in c.id for c in compose(name)))
        self.assertTrue(calibration_diagnostic('readbacks/experiment.temporal-long.r1/frame.inputs.gz'))
        self.assertFalse(calibration_diagnostic('analysis/temporal-long/summary.json'))


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Existing NumPy required')
class LongEvaluatorTests(unittest.TestCase):
    def test_object_and_regions_remain_visible_throughout(self):
        import numpy as np
        from temporal_diagnosis import references
        from temporal_quality import age_step
        age=np.full((360,640),999); previous=None
        for frame in range(256):
            ref=references(dict(scene=2,frame=frame,velocity=[.375,.25]))
            if previous is not None: age=age_step(age,ref[4],previous)
            if frame>=32:
                for mask in ref[2].values(): self.assertGreater(int(mask.sum()),0)
                self.assertGreater(int((ref[2]['background']&(age>=2)&(age<=5)).sum()),0)
            previous=ref[4]

    def test_motion_does_not_restart_at_repeated_jitter_phase(self):
        import numpy as np
        from temporal_calibration import truncate_half
        from temporal_diagnosis import expected_inputs
        a=expected_inputs(dict(scene=2,frame=32,velocity=[.375,.25],motion=0))
        b=expected_inputs(dict(scene=2,frame=64,velocity=[.375,.25],motion=0))
        self.assertNotEqual(a[:460800],b[:460800])
        for frame in (32,64,128,224,255):
            raw=expected_inputs(dict(scene=2,frame=frame,velocity=[.375,.25],motion=0))
            motion=np.frombuffer(raw[691200:-4],dtype='<f2').reshape(180,320,2)
            y=int(70+.25*frame); x=int(120+.375*frame)
            expected=truncate_half(np.array([-.375/320,-.25/180])).view('<f2')
            np.testing.assert_array_equal(motion[y,x],expected)

    def test_unplanned_resets_and_incomplete_native_inventory_fail(self):
        from temporal_calibration import jitter_table
        from temporal_diagnosis import arms, validate
        from temporal_long import protocol
        spec=protocol(); arm=arms(spec)[0]; j=jitter_table()[0]
        meta=dict(arm=arm['name'],frame=64,scene=2,velocity=[.375,.25],motion=0,
                  image_jitter=j,dispatch_jitter=j,reset=True)
        with patch('temporal_diagnosis.rows',return_value=[meta]):
            with self.assertRaisesRegex(ValueError,'reset'): validate('.',False,arms(spec),spec)
        with patch('temporal_diagnosis.rows',return_value=[]):
            with self.assertRaisesRegex(ValueError,'Incomplete'): validate('.',False,arms(spec),spec)

    def test_late_failure_cannot_hide_in_passing_whole_run_average(self):
        from temporal_long import evaluate_block, protocol
        from temporal_quality import evaluate, pooled_metrics
        spec=protocol(); data={'textured-slow-'+name:[] for name in ('correct','reverse')}
        for frame in range(32,256):
            for name,measurements in data.items():
                factor=2 if name.endswith('reverse') else (1.1 if frame>=224 else .5)
                scores={metric:dict(pixels=10,mae=limit*factor,rmse=limit*factor,signed_mean=0)
                        for metric,limit in spec['native_budgets'].items()}
                scores['texture']=dict(pixels=10,numerator=.01,denominator=1,nrmse=.1)
                measurements.append(dict(frame=frame,scores=scores))
        pooled=pooled_metrics([m['scores'] for m in data['textured-slow-correct']])
        self.assertTrue(evaluate(pooled,spec['native_budgets'],pooled=True)['passed'])
        result=evaluate_block(data,spec)
        self.assertFalse(result['passed']); self.assertTrue(all(w['passed'] for w in result['windows'][:-1]))
        self.assertFalse(result['windows'][-1]['budget']['passed'])
        for changed in (data['textured-slow-correct'][:-1],data['textured-slow-correct'][::-1]):
            bad=dict(data); bad['textured-slow-correct']=changed
            with self.assertRaises(ValueError): evaluate_block(bad,spec)


if __name__=='__main__': unittest.main()
