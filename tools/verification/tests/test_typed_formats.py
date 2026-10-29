# SPDX-License-Identifier: MPL-2.0
"""Independent packed-format examples and rejection controls for native acceptance."""
import math
import struct
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from execution import Rejected
from typed_formats import BASE, COLORS, DEPTH, decode, depth_audit, depth_workloads, input_bytes, output_check, pixel, workloads


class TypedFormats(unittest.TestCase):
    def test_depth_inventory_and_shader_plane_rejection(self):
        import json
        selections=depth_workloads()
        for fmt,(dsv,srv,typed) in DEPTH.items():
            for reversed_depth in (False,True):
                self.assertTrue(any(s['formats'][1]==fmt and s['reversed']==reversed_depth for s in selections))
            selection=next(s for s in selections if s['formats'][1]==fmt)
            text='\n'.join('FORMAT_DEPTH '+json.dumps(dict(allocation=fmt,dsv=dsv,srv=srv,casting=1)) for _ in range(3))
            with TemporaryDirectory() as temp:
                directory=Path(temp)
                for phase in range(3):
                    (directory/f'public-{phase}-depth-sampled.bin').write_bytes(input_bytes(selection,phase,1))
                    if fmt in (19,20): (directory/f'public-{phase}-stencil.bin').write_bytes(b'\x5a'*(320*180))
                depth_audit(text,directory,selection,'public')
                with self.assertRaises(Rejected): depth_audit('',directory,selection,'public')
                if typed:
                    with self.assertRaises(Rejected): depth_audit(text.replace('"casting": 1','"casting": 0'),directory,selection,'public')
                if fmt in (19,20):
                    (directory/'public-0-stencil.bin').write_bytes(b'\0'*(320*180))
                    with self.assertRaises(Rejected): depth_audit(text,directory,selection,'public')
                    (directory/'public-0-stencil.bin').write_bytes(b'\x5a'*(320*180))
                (directory/'public-0-depth-sampled.bin').write_bytes(b'\0'*(320*180*4))
                with self.assertRaises(Rejected): depth_audit(text,directory,selection,'public')

    def test_known_packed_channel_values(self):
        packed=960 | (896<<11) | (416<<22)
        self.assertEqual(decode(26,struct.pack('<I',packed)),(1,0.5,0.25))
        self.assertEqual(pixel(26,(1,0.5,0.25,1)),struct.pack('<I',packed))
        self.assertEqual(decode(28,bytes((0,51,102,255))),(0,0.2,0.4,1))
        self.assertEqual(decode(24,struct.pack('<I',1023|(3<<30))),(1,0,0,1))
        self.assertTrue(math.isnan(decode(26,struct.pack('<I',1985))[0]))

    def test_incomplete_nonfinite_and_changed_control_cannot_pass(self):
        seed=struct.pack('<4e',1,0,1,0.25)*640*360
        valid=struct.pack('<4e',0.25,0.5,0.75,1)*640*360
        self.assertEqual(output_check(valid,seed,10,(640,360),False)['written'],640*360)
        for data,control in ((seed[:8]+valid[8:],False),(valid,True),
                             (struct.pack('<4e',math.nan,0,0,1)+valid[8:],False),(valid[:-1],False)):
            with self.assertRaises(Rejected): output_check(data,seed,10,(640,360),control)
        self.assertEqual(output_check(seed,seed,10,(640,360),True)['written'],0)

    def test_inventory_accounts_for_every_selected_role_and_operation(self):
        main=workloads();all_work=workloads(generator=True)
        self.assertFalse(any(s['generate'] for s in main))
        for fmt in COLORS:
            for slot in (0,4):
                self.assertTrue(any(s['formats'][slot]==fmt and not s['generate'] for s in main))
            for slot in (0,5):
                self.assertTrue(any(s['formats'][slot]==fmt and s['generate'] and s['zero'] for s in all_work))
                self.assertTrue(any(s['formats'][slot]==fmt and s['generate'] and s['uav']&(1<<slot) for s in all_work))
        self.assertTrue(any(s['formats'][1]==54 and s['reversed'] for s in main))
        self.assertEqual(len({s['name'] for s in all_work}),len(all_work))
        self.assertEqual(BASE[3],41)
        self.assertEqual(BASE[6],61)


if __name__=='__main__': unittest.main()
