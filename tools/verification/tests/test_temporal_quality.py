# SPDX-License-Identifier: MPL-2.0
"""Validate evaluator discrimination, dose response and bounded claim selection."""
import importlib.util
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Existing NumPy required for image evaluator')
class QualityTests(unittest.TestCase):
    def test_known_defects_are_rejected_by_the_responsible_property(self):
        from temporal_quality import synthetic_controls
        result=synthetic_controls()
        self.assertTrue(result['passed'],result)
        self.assertEqual(len(result['controls']),13)
        self.assertTrue(all(result['severity_monotonic'].values()))
        missing=next(c for c in result['controls'] if c['control']=='missing-object')
        self.assertLess(missing['scores']['disocclusion']['mae'],.002)
        self.assertFalse(missing['verdict']['passed'])
        self.assertFalse(missing['verdict']['properties']['foreground']['passed'])

    def test_disabled_evaluator_is_detected_even_when_every_image_passes(self):
        from temporal_quality import frame_metrics, synthetic_controls
        def disabled(*args):
            result=frame_metrics(*args)
            for name,item in result.items():
                item['nrmse' if name=='texture' else 'mae']=0
            return result
        result=synthetic_controls(disabled)
        self.assertFalse(result['passed'])
        self.assertFalse(any(result['severity_monotonic'].values()))
        self.assertTrue(all(c['verdict']['passed'] for c in result['controls']))

    def test_empty_nonfinite_and_zero_reference_controls_do_not_pass(self):
        from temporal_quality import evaluate, sensitivity
        for value in (None,float('nan'),float('inf')):
            self.assertFalse(evaluate({'edge':dict(pixels=10,mae=value)},{'edge':1})['passed'])
        self.assertFalse(evaluate({'edge':dict(pixels=0,mae=0)},{'edge':1})['passed'])
        self.assertFalse(sensitivity({'temporal':dict(pixel_frames=10,mae=0)},
                                     {'temporal':dict(pixel_frames=10,mae=1)},'temporal')['passed'])

    def test_full_factorial_cells_and_independent_confirmations_are_preserved(self):
        from temporal_diagnosis import arms
        from temporal_quality import protocol
        from profiles import compose
        from retention import calibration_diagnostic
        planned=arms(protocol())
        cells={(a['scene'],tuple(a['velocity']),a['motion']) for a in planned if not a['reset_all']}
        self.assertEqual(len(cells),8)
        self.assertEqual(len(planned),9)
        cases=compose('experiment-temporal-quality')
        for name in ('r1','r2','r3'):
            self.assertIn('experiment.temporal-quality.'+name,{c.id for c in cases})
        for name in ('ci','release','native-upscaler'):
            self.assertFalse(any('temporal-quality' in c.id for c in compose(name)))
        self.assertTrue(calibration_diagnostic('readbacks/experiment.temporal-quality.r3/arm.rgba16f.gz'))
        self.assertFalse(calibration_diagnostic('analysis/temporal-quality/summary.json'))


if __name__=='__main__': unittest.main()
