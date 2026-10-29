# SPDX-License-Identifier: MPL-2.0
"""Independent reference, metric degeneracy and protocol isolation checks."""
import importlib.util
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Existing NumPy required for diagnostic oracle')
class DiagnosisTests(unittest.TestCase):
    def test_exact_area_integral_against_dense_independent_quadrature(self):
        import numpy as np
        from temporal_diagnosis import integral_color
        # A partially covered silhouette pixel; integrate FG/BG over their
        # actual subrectangles rather than interpolating the low-res input.
        samples=1000; coords=(np.arange(samples)+.5)/samples
        x,y=np.meshgrid(100+coords*.125,60+coords*.5)
        actual=integral_color(2,True,np.array(100.),np.array(100.125),np.array(60.),np.array(60.5),99.75,50)/(.125*.5)
        expected=np.array([(.65+.15*np.sin(.4*(x-99.75))).mean(),
                           (.2+.1*np.sin(.3*(y-50))).mean(),
                           (.15+.08*np.sin(.23*(x-99.75)+.19*(y-50))).mean()])
        np.testing.assert_allclose(actual,expected,rtol=0,atol=1e-9)
        empty=integral_color(2,False,np.array(1.),np.array(0.),np.array(1.),np.array(2.))
        np.testing.assert_array_equal(empty,[0,0,0])

    def test_new_flat_reference_matches_original_exactly(self):
        import numpy as np
        from temporal_diagnosis import references
        from temporal_calibration import references as original
        for frame in (0,1,32,63):
            new=references(dict(scene=1,frame=frame,velocity=[.375,.25]))
            old=original(dict(scene=1,frame=frame,camera_cut=False))
            np.testing.assert_array_equal(new[0],old[0])
            np.testing.assert_array_equal(new[1],old[1])
            for region in old[2]: np.testing.assert_array_equal(new[2][region],old[2][region])

    def test_motion_and_jitter_inputs_do_not_change_color_depth(self):
        from temporal_diagnosis import expected_inputs
        meta=dict(scene=2,frame=32,velocity=[.375,.25],motion=0)
        a=expected_inputs(meta); b=expected_inputs(dict(meta,motion=1)); c=expected_inputs(dict(meta,motion=2))
        self.assertEqual(a[:691200],b[:691200]); self.assertEqual(a[:691200],c[:691200])
        self.assertNotEqual(a[691200:-4],b[691200:-4]); self.assertNotEqual(a[691200:-4],c[691200:-4])

    def test_wrong_synthetic_image_can_have_perfect_leakage(self):
        from temporal_diagnosis import metric_controls
        results=metric_controls()['results']; missing=results['constant_background']
        self.assertEqual(missing['background_leakage']['mae'],0)
        self.assertGreater(missing['point']['foreground']['mae'],.1)
        self.assertEqual(results['constant_foreground']['foreground_temporal_residual']['mae'],0)
        self.assertGreater(results['constant_foreground']['point']['background']['mae'],.1)

    def test_diagnostic_matrix_is_explicit_and_excluded_from_routine_profiles(self):
        from temporal_diagnosis import arms
        from profiles import compose
        from retention import calibration_diagnostic
        self.assertEqual(len(arms()),19)
        for name in ('fast','ci','native-upscaler','release'):
            self.assertFalse(any('temporal-diagnosis' in case.id for case in compose(name)))
        self.assertTrue(calibration_diagnostic('readbacks/experiment.temporal-diagnosis.r1/frame.inputs.gz'))
        self.assertFalse(calibration_diagnostic('analysis/temporal-diagnosis/summary.json'))


if __name__=='__main__': unittest.main()
