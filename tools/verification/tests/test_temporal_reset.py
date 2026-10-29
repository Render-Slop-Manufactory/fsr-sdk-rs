# SPDX-License-Identifier: MPL-2.0
"""Cut geometry, paired-input plans and falsifiable history-reset predictions."""
import copy
import importlib.util
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))


class ResetPlanTests(unittest.TestCase):
    def test_paired_histories_have_exact_counts_and_only_intended_resets(self):
        from temporal_diagnosis import arms, frame_plan
        from temporal_reset import protocol
        spec=protocol(); full,inputs=frame_plan(spec); planned=arms(spec)
        self.assertEqual(sum(len(full)-a['start'] for a in planned),320)
        self.assertEqual(sum(f>=a['start'] for a in planned for f in inputs),25)
        self.assertEqual([(a['start'],a['reset_frame']) for a in planned],[(0,64),(0,-1),(64,64)])
        self.assertEqual([f for start,end in spec['windows'] for f in range(start,end+1)],list(range(64)))
        from temporal_diagnosis import HERE
        import json
        confirm=json.loads((HERE/'temporal-reset-confirm-protocol.json').read_text())
        self.assertEqual(confirm,protocol(True))

    def test_profiles_remain_separate_from_routine_gates(self):
        from profiles import compose
        from retention import calibration_diagnostic
        for name in ('ci','release','native-upscaler'):
            self.assertFalse(any('temporal-reset' in c.id for c in compose(name)))
        self.assertIn('experiment.temporal-reset.criteria',{c.id for c in compose('experiment-temporal-reset')})
        self.assertNotIn('experiment.temporal-reset-explore.criteria',{c.id for c in compose('experiment-temporal-reset-explore')})
        for ns in ('experiment.temporal-reset','experiment.temporal-reset-explore'):
            self.assertTrue(calibration_diagnostic(f'readbacks/{ns}.r1/frame.rgba16f.gz'))
        self.assertFalse(calibration_diagnostic('analysis/temporal-reset/summary.json'))

    def test_equal_controls_do_not_claim_causal_reset_sensitivity(self):
        from temporal_reset import verdict
        criteria=dict(native_budgets={'ghost':.01},pair_maximum=1e-6,minimum_history_difference=1e-4)
        block=dict(windows=[dict(frames=[64,71],arms={a:{'ghost':dict(pixel_frames=100,mae=.001)} for a in ('reset','fresh')})],
                   pairs=[dict(reset_fresh_maximum=0)],early_reset_mae=0,early_no_reset_mae=0)
        self.assertFalse(verdict(block,criteria)['passed'])
        block['early_no_reset_mae']=.002
        self.assertTrue(verdict(block,criteria)['passed'])
        late=copy.deepcopy(block); late['pairs'].append(dict(reset_fresh_maximum=.01))
        self.assertFalse(verdict(late,criteria)['recovery'])
        bad=copy.deepcopy(block); bad['windows'][0]['arms']['reset']['ghost']['pixel_frames']=0
        self.assertFalse(verdict(bad,criteria)['passed'])


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Existing NumPy required')
class ResetOracleTests(unittest.TestCase):
    def test_camera_projection_and_background_world_texture(self):
        import numpy as np
        from temporal_calibration import projected_rectangle
        from temporal_diagnosis import rectangle, references
        for frame in (0,63,64,127):
            meta=dict(scene=2,frame=frame,velocity=[.375,.25],cut_frame=64)
            np.testing.assert_allclose(rectangle(meta),projected_rectangle(frame,cut=frame>=64),rtol=0,atol=1e-12)
            l,t,r,b=rectangle(meta); self.assertTrue(4<l<r<316 and 4<t<b<176)
        ref=references(dict(scene=2,frame=64,velocity=[.375,.25],cut_frame=64))
        x,y=(500+.5)/2+24,(80+.5)/2-12
        np.testing.assert_allclose(ref[0][80,500],[.25+.1*np.sin(.35*x),.3+.1*np.sin(.27*y),.35+.1*np.sin(.21*x+.17*y)],rtol=0,atol=1e-14)

    def test_cut_motion_for_both_depth_planes_and_absolute_jitter(self):
        import numpy as np
        from temporal_calibration import truncate_half
        from temporal_diagnosis import expected_inputs
        def vectors(frame):
            data=expected_inputs(dict(scene=2,frame=frame,velocity=[.375,.25],motion=0,cut_frame=64))
            return np.frombuffer(data[691200:-4],dtype='<f2').reshape(180,320,2)
        cut=vectors(64); after=vectors(65)
        np.testing.assert_array_equal(cut[130,50],truncate_half(np.array([(96-.375)/320,(-48-.25)/180])).view('<f2'))
        np.testing.assert_array_equal(cut[40,250],truncate_half(np.array([24/320,-12/180])).view('<f2'))
        np.testing.assert_array_equal(after[130,50],truncate_half(np.array([-.375/320,-.25/180])).view('<f2'))
        np.testing.assert_array_equal(after[40,250],[0,0])

    def test_old_history_contamination_is_detected_and_disabled_metric_rejected(self):
        from temporal_reset import ghost_controls
        self.assertTrue(ghost_controls()['passed'])
        self.assertFalse(ghost_controls(lambda values,mask:dict(pixels=int(mask.sum()),mae=0))['passed'])

    def test_calibration_cannot_relax_limits_past_known_defects(self):
        from temporal_reset import criteria_controls, METRICS
        self.assertTrue(criteria_controls(dict(native_budgets={k:1e-6 for k in METRICS}))['passed'])
        self.assertFalse(criteria_controls(dict(native_budgets={k:1 for k in METRICS}))['passed'])


if __name__=='__main__': unittest.main()
