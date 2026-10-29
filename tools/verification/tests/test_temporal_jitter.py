# SPDX-License-Identifier: MPL-2.0
"""Independent phase scheduling, identifiable measurement and failed predictions."""
import copy
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))


class JitterPlanTests(unittest.TestCase):
    def test_start_phase_and_wrong_dispatch_are_distinct(self):
        from temporal_jitter import protocol
        from temporal_diagnosis import arms, frame_plan
        spec=protocol(); planned=arms(spec); full,inputs=frame_plan(spec)
        self.assertEqual(len(planned),30)
        self.assertEqual(len(full)*len(planned),3840)
        self.assertEqual(len(inputs)*len(planned),990)
        for phase in (0,8,16,24):
            self.assertTrue(all(f in inputs for f in full if (f+phase)%32 in (0,31)))
        self.assertEqual(spec['windows'],[[32,63],[64,95],[96,127]])
        self.assertEqual([a['jitter_phase'] for a in planned if a['name'].endswith('-correct')],[0,8,16,24,0])
        for group in {a['group'] for a in planned}:
            values=[a for a in planned if a['group']==group]
            self.assertTrue(all(a['motion']==0 and not a['reset_all'] for a in values))
            self.assertEqual({a['dispatch_lag'] for a in values},{0,1})
        confirm=protocol(True); expected=copy.deepcopy(spec)
        expected.update(confirmation=True,repeats=3,order_seeds=[419,827,1237])
        self.assertEqual(confirm,expected)

    def test_profiles_and_compaction_keep_experiments_separate(self):
        from profiles import compose
        from retention import calibration_diagnostic
        for name in ('ci','release','native-upscaler'):
            self.assertFalse(any('temporal-jitter' in c.id for c in compose(name)))
        self.assertIn('experiment.temporal-jitter.criteria',{c.id for c in compose('experiment-temporal-jitter')})
        self.assertTrue(calibration_diagnostic('readbacks/experiment.temporal-jitter.r1/frame.inputs.gz'))
        self.assertTrue(calibration_diagnostic('analysis/temporal-jitter/r1-trajectories.json'))
        self.assertFalse(calibration_diagnostic('analysis/temporal-jitter/summary.json'))

    def test_cached_compression_still_compares_every_raw_byte(self):
        from temporal_calibration import pack, payload
        with tempfile.TemporaryDirectory() as name:
            root=Path(name); cache={}; raw=b'known full raw payload'*100
            first=root/'a'; first.mkdir(); (first/'frame.inputs').write_bytes(raw)
            pack(first,cache=cache)
            second=root/'b'; second.mkdir(); (second/'frame.inputs').write_bytes(raw)
            with patch('temporal_calibration.gzip.compress',side_effect=AssertionError('Cache should reuse compressed bytes')):
                pack(second,cache=cache)
            self.assertEqual(payload(second/'frame.inputs.gz'),raw)
            next(iter(cache.values())).write_bytes(b'corrupt gzip')
            third=root/'c'; third.mkdir(); source=third/'frame.inputs'; source.write_bytes(raw)
            with self.assertRaises((OSError,ValueError,EOFError)): pack(third,cache=cache)
            self.assertEqual(source.read_bytes(),raw)


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Existing NumPy required')
class JitterMeasurementTests(unittest.TestCase):
    def test_translations_are_distinct_from_bias_and_gain(self):
        from temporal_jitter import position_controls
        self.assertTrue(position_controls()['passed'])
        self.assertFalse(position_controls(lambda _:dict(valid=True,x=0,y=0))['passed'])

    def test_single_axis_controls_detect_a_coupled_estimator(self):
        from temporal_jitter import PositionModel, position_controls
        def factory(meta,region):
            model=PositionModel(meta,region); original=model.measure
            def coupled(output):
                result=original(output)
                result['x'],result['y']=-result['y'],-result['x']
                return result
            model.measure=coupled
            return model
        # Anti-diagonal-only translations cannot detect this particular fault.
        controls=position_controls(model_factory=factory)
        self.assertTrue(all(c['passed'] for c in controls['controls'] if c['kind']!='axis-shift'))
        self.assertFalse(controls['passed'])

    def test_cached_oracles_equal_direct_bytes_for_all_physical_inputs(self):
        from temporal_diagnosis import expected_inputs, input_oracle_cache
        oracle=input_oracle_cache()
        for velocity in ([0,0],[.375,.25]):
            for phase in (0,8,16,24):
                for frame in (0,1,31,32,64,127):
                    meta=dict(scene=2,frame=frame,velocity=velocity,motion=0,jitter_phase=phase)
                    self.assertEqual(oracle(meta),expected_inputs(meta))
        for frame in (0,63,64,65,127):
            meta=dict(scene=2,frame=frame,velocity=[0,0],motion=0,jitter_phase=16,cut_frame=64)
            self.assertEqual(oracle(meta),expected_inputs(meta))

    def test_unidentifiable_or_nonfinite_images_do_not_pass(self):
        import numpy as np
        from temporal_jitter import PositionModel, pool_position
        model=PositionModel(dict(scene=2,frame=40,velocity=[0,0]),'background')
        constant=model.measure(np.full((360,640,3),.3))
        self.assertFalse(constant['valid'])
        with self.assertRaises(ValueError): pool_position([constant])
        with self.assertRaises(ValueError): model.measure(np.full((360,640,3),np.nan))
        with self.assertRaises(ValueError): pool_position([])

    def test_independent_inputs_follow_image_phase_only(self):
        import numpy as np
        from temporal_calibration import jitter_table, truncate_half
        from temporal_diagnosis import expected_inputs
        meta=dict(scene=2,frame=31,velocity=[0,0],motion=0,jitter_phase=8)
        raw=expected_inputs(meta); out=np.frombuffer(raw[:460800],dtype='<f2').reshape(180,320,4)
        jx,jy=jitter_table()[7]; x,y=250.5-jx,120.5-jy
        oracle=truncate_half(np.array([.25+.1*np.sin(.35*x),.3+.1*np.sin(.27*y),.35+.1*np.sin(.21*x+.17*y),1])).view('<f2')
        np.testing.assert_array_equal(out[120,250],oracle)
        self.assertEqual(raw,expected_inputs(dict(meta,dispatch_lag=1)))
        self.assertNotEqual(raw,expected_inputs(dict(meta,jitter_phase=0)))
        motion=np.frombuffer(raw[691200:-4],dtype='<u2')
        np.testing.assert_array_equal(motion,np.zeros_like(motion))

    def test_known_defects_cannot_be_calibrated_away(self):
        from temporal_jitter import ALIGNMENT, QUALITY, criteria_controls
        criteria=dict(groups={group:dict(alignment={k:1e-6 for k in ALIGNMENT},quality={k:1e-6 for k in QUALITY})
            for group in ('stationary','moving')})
        self.assertTrue(criteria_controls(criteria)['passed'])
        criteria['groups']['stationary']['alignment']['x_rms']=.125
        self.assertFalse(criteria_controls(criteria)['passed'])
        criteria['groups']['stationary']['alignment']['x_rms']=1e-6
        criteria['groups']['moving']['quality']['temporal']=float('nan')
        self.assertFalse(criteria_controls(criteria)['passed'])

    def test_equal_outputs_cannot_claim_sensitivity_and_late_failure_counts(self):
        from temporal_jitter import ALIGNMENT, QUALITY, CONTROLS, protocol, verdict
        spec=protocol(); block=dict(windows={})
        criteria=dict(groups={g:dict(alignment={k:.01 for k in ALIGNMENT},quality={k:.01 for k in QUALITY}) for g in ('stationary','moving')})
        for scene in spec['scenes']:
            for phase in scene['jitter_phases']:
                for start,end in spec['windows']:
                    value=dict(position={r:{k:.001 for k in ALIGNMENT} for r in ('background','foreground')},
                        quality={k:dict(pixel_frames=100,mae=.001,nrmse=.001) for k in QUALITY})
                    block['windows'][f'{scene["name"]}-p{phase:02}:{start}-{end}']={c:copy.deepcopy(value) for c in ['correct']+list(CONTROLS)}
        self.assertFalse(verdict(block,spec,criteria)['passed'])
        for window in block['windows'].values():
            for control in CONTROLS:
                for region in ('background','foreground'): window[control]['position'][region]={k:.1 for k in ALIGNMENT}
        self.assertTrue(verdict(block,spec,criteria)['passed'])
        last=block['windows']['moving-p00:96-127']['correct']
        last['position']['foreground']['x_rms']=.02
        self.assertFalse(verdict(block,spec,criteria)['passed'])


if __name__=='__main__': unittest.main()
