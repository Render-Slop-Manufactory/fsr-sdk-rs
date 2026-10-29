# SPDX-License-Identifier: MPL-2.0
"""Analytic checks of the independent calibration oracle and failure boundaries."""
import gzip
import importlib.util
import argparse
import contextlib
import io
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from profiles import compose
from temporal_calibration import depth, jitter_table, pack, payload, projected_rectangle
from records import Sanitizer
from records import export, fresh, verify_artifacts
from execution import Case, Runner
from retention import compact_calibration
from repository_size import evaluate, TOTAL_LIMIT, FILE_LIMIT
from run import verify_run


class ProtocolTests(unittest.TestCase):
    def retention_fixture(self, root, fail=False):
        repository = root/'repo'; repository.mkdir()
        (repository/'Cargo.toml').write_text('[workspace]\n')
        runner = Runner(repository, fresh(root/'runs','calibration'),
                        argparse.Namespace(toolchain=None,online=False))
        def data(r):
            directory = r.directory/'readbacks/experiment.temporal-calibration.r1'
            directory.mkdir()
            (directory/'frame.rgba16f.gz').write_bytes(gzip.compress(b'raw pixels'))
            (directory/'frames.jsonl').write_text('{}\n')
            analysis = r.directory/'analysis/temporal-calibration'; analysis.mkdir(parents=True)
            (analysis/'r1-a-correct.json').write_text('{}')
            (analysis/'diagnostic-summary.json').write_text('{"result":"measured"}')
            r.note(payloads=[{'file':'frame.rgba16f.gz','decoded_bytes':10}])
            if fail: raise ValueError('audit failed')
        with contextlib.redirect_stdout(io.StringIO()):
            runner.execute([Case('capture','capture',data)],'experiment-temporal-calibration')
        return runner

    def test_success_and_failure_compaction_preserve_outcomes_and_integrity(self):
        for fail in (False, True):
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as temporary:
                runner = self.retention_fixture(Path(temporary), fail)
                root = runner.directory
                self.assertEqual(compact_calibration(root),2)
                self.assertFalse((root/'readbacks/experiment.temporal-calibration.r1/frame.rgba16f.gz').exists())
                self.assertTrue((root/'analysis/temporal-calibration/diagnostic-summary.json').exists())
                self.assertTrue((root/'readbacks/experiment.temporal-calibration.r1/frames.jsonl').exists())
                result = json.loads((root/'cases/capture/result.json').read_text())
                self.assertNotIn('payloads',result['observations'])
                self.assertEqual(result['observations']['payload_inventory']['files'],1)
                self.assertEqual(result['evaluation'],'failed' if fail else 'passed')
                verify_artifacts(root)
                verify_run(root)

    def test_compaction_rejects_tampering_before_deletion(self):
        with tempfile.TemporaryDirectory() as temporary:
            runner = self.retention_fixture(Path(temporary))
            raw = runner.directory/'readbacks/experiment.temporal-calibration.r1/frame.rgba16f.gz'
            raw.write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError,'identity mismatch'):
                compact_calibration(runner.directory)
            self.assertTrue(raw.exists())

    def test_export_of_full_local_calibration_is_compact_without_mutating_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            runner = self.retention_fixture(Path(temporary))
            destination = Path(temporary)/'export'
            export(runner.directory,destination,runner.repository)
            self.assertFalse((destination/'readbacks/experiment.temporal-calibration.r1/frame.rgba16f.gz').exists())
            self.assertTrue((runner.directory/'readbacks/experiment.temporal-calibration.r1/frame.rgba16f.gz').exists())
            self.assertFalse(json.loads((destination/'export.json').read_text())['raw_readbacks_included'])
            verify_artifacts(destination)

    def test_budget_rejects_many_small_files_and_generated_payloads(self):
        self.assertEqual(evaluate([('crates/source.rs',100)])[1],[])
        self.assertTrue(evaluate([('docs/large.json',FILE_LIMIT+1)])[1])
        self.assertTrue(evaluate([(f'docs/{i}.json',100_000) for i in range(TOTAL_LIMIT//100_000+1)])[1])
        self.assertTrue(evaluate([('docs/research/frame.inputs.gz',10)])[1])
        self.assertTrue(evaluate([('docs/research/sources/Cargo.toml',10)])[1])

    def test_pooled_results_weight_pixels_and_preserve_empty_regions(self):
        from temporal_calibration import pool
        samples = [dict(pixels=1,mae=1,rmse=1,signed_mean=1),
                   dict(pixels=3,mae=3,rmse=3,signed_mean=-3)]
        result = pool(samples)
        self.assertEqual(result['pixel_frames'],4)
        self.assertEqual(result['mae'],2.5)
        self.assertAlmostEqual(result['rmse'],7**.5)
        self.assertEqual(result['signed_mean'],-2)
        self.assertEqual(pool([dict(pixels=0,mae=None,rmse=None,signed_mean=None)])['mae'],None)

    def test_profile_is_excluded_from_ordinary_gates(self):
        for name in ('ci','release','native-upscaler','fast'):
            self.assertFalse(any('temporal-calibration' in case.id for case in compose(name)))
        cases=compose('experiment-temporal-calibration')
        preceding=set()
        for case in cases:
            self.assertTrue(set(case.requires)<=preceding)
            preceding.add(case.id)
        self.assertLess([c.id for c in cases].index('experiment.temporal-calibration.input-only'),
                        [c.id for c in cases].index('experiment.temporal-calibration.dispatch.single'))

    def test_depth_projection_and_continuous_motion(self):
        self.assertAlmostEqual(depth(.5),0)
        self.assertAlmostEqual(depth(20),1)
        self.assertAlmostEqual(depth(2),10/13)
        self.assertAlmostEqual(depth(8),25/26)
        a,b=projected_rectangle(31),projected_rectangle(32)
        self.assertEqual(tuple(y-x for x,y in zip(a,b)),(.375,.25,.375,.25))
        a,b=projected_rectangle(37),projected_rectangle(37,True)
        self.assertEqual(tuple(y-x for x,y in zip(a,b)),(-96,48,-96,48))
        for frame in range(64):
            l,t,r,b=projected_rectangle(frame,True)
            self.assertTrue(0<l<r<320 and 0<t<b<180)

    def test_jitter_is_saved_binary32_and_not_zero(self):
        j=jitter_table()
        self.assertEqual(len(j),32)
        self.assertEqual(j[0],[0,struct.unpack('<f',struct.pack('<f',-1/6))[0]])
        self.assertTrue(all(abs(x)<.5 and abs(y)<.5 and (x,y)!=(0,0) for x,y in j))

    def test_projection_jitter_shifts_image_in_reported_direction(self):
        focal=90/(3**.5/3)
        world=(.4,.2,2)
        ndc=(focal*world[0]/world[2]/160,focal*world[1]/world[2]/90)
        unjittered=(160*(1+ndc[0]),90*(1-ndc[1]))
        for jx,jy in jitter_table():
            jittered=(160*(1+ndc[0]+2*jx/320),90*(1-ndc[1]+2*jy/180))
            self.assertAlmostEqual(jittered[0]-unjittered[0],jx)
            self.assertAlmostEqual(jittered[1]-unjittered[1],jy)

    def test_export_preserves_dxgi_device_id_and_normalizes_com_pointer(self):
        value='Adapter: AMD; vendor=0x1002 device=0x7590\nSelected-device AMD check: vendor=0x1002 device=0x7590\nDevice: device=0x12345678'
        sanitized=Sanitizer(Path.cwd()).text(value)
        self.assertIn('device=0x7590',sanitized)
        self.assertIn('Device: device=<pointer-1>',sanitized)
        self.assertIn('Selected-device AMD check: vendor=0x1002 device=0x7590',sanitized)

    def test_export_normalizes_sandbox_temporary_paths(self):
        value=str(Path.home())+'/AppData/Local/Packages/sandbox.{12345678-1234-1234-1234-123456789abc}/AC/Temp/fixture'
        self.assertEqual(Sanitizer(Path.cwd()).text(value),'<sandbox-temp>/fixture')

    def test_pack_is_lossless_and_preserves_decoded_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary); data=struct.pack('<eeee',.125,.25,.375,1)*50
            (root/'sample.rgba16f').write_bytes(data)
            record=pack(root)[0]
            self.assertEqual(payload(root/record['file']),data)
            self.assertEqual(record['decoded_bytes'],len(data))
            self.assertFalse((root/'sample.rgba16f').exists())


@unittest.skipUnless(importlib.util.find_spec('numpy'),'Experiment oracle requires existing NumPy')
class OracleTests(unittest.TestCase):
    def test_half_decoder_and_truncation_are_distinct(self):
        import numpy as np
        from temporal_calibration import truncate_half
        values=np.array([1,.125,-.375/320,-.25/180,0])
        bits=truncate_half(values)
        decoded=np.array([struct.unpack('<e',struct.pack('<H',int(b)))[0] for b in bits])
        self.assertTrue(np.all(np.abs(decoded)<=np.abs(values)))
        self.assertAlmostEqual(decoded[2]*320,-.374755859375)
        with self.assertRaises(ValueError): truncate_half([1e-8])

    def test_area_reference_and_empty_metrics(self):
        import numpy as np
        from temporal_calibration import references, stats
        p,a,regions,_,fg=references(dict(scene=1,frame=1,camera_cut=False))
        # Left=100.375. Display pixel x=200 spans [100,100.5];
        # overlap .125/.5 = .25 at a vertical interior position.
        self.assertFalse(fg[140,200])
        self.assertTrue(np.array_equal(p[140,200],[.125,.25,.375]))
        self.assertTrue(np.array_equal(a[140,200],np.array([.125,.25,.375])+.25*np.array([.75,-.125,-.3125])))
        self.assertEqual(stats([])['mae'],None)
        self.assertAlmostEqual(stats([1,-1,0],1)['rmse'],(2/3)**.5)

    def test_true_flow_reprojection_on_affine_field(self):
        import numpy as np
        from temporal_calibration import reproject
        x,y=np.meshgrid(np.arange(640),np.arange(360))
        field=np.stack((x+2*y,x-y,x*0+1),axis=2)
        warped,mask=reproject(field,np.ones((360,640),bool))
        self.assertTrue(mask[100,100])
        self.assertTrue(np.array_equal(warped[100,100],[298.25,-.25,1]))

    def test_unwritten_channel_and_incomplete_readback_fail(self):
        from temporal_calibration import image
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary)/'frame.rgba16f.gz'
            path.write_bytes(gzip.compress(b'bad'))
            with self.assertRaises(ValueError): image(path)
            path.write_bytes(gzip.compress(struct.pack('<eeee',.1,.2,.3,65504)*640*360))
            with self.assertRaises(ValueError): image(path)


if __name__=='__main__': unittest.main()
