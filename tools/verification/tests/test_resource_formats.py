# SPDX-License-Identifier: MPL-2.0
"""Independent format packing and complete operation/owner case inventory."""
import math
import struct
import unittest
from resource_formats import encode, inputs, workloads, cases


class ResourceFormats(unittest.TestCase):
    def test_exact_packed_representations(self):
        self.assertEqual(encode(31,(0.125,0.25,0.5,1)), bytes((16,32,64,127)))
        self.assertEqual(encode(67,(0.125,0.25,0.5,1)), struct.pack('<I',64 | 128<<9 | 256<<18 | 15<<27))
        self.assertEqual(encode(6,(0.125,0.25,0.5,1)), struct.pack('<3f',0.125,0.25,0.5))

    def test_typeless_payloads_match_fixed_views(self):
        values=(0.125,0.25,0.5,1)
        for allocation,view in ((1,2),(9,10),(23,24),(27,28),(90,87),(5,6),(15,16),(33,34),(53,54),(39,41),(60,61)):
            self.assertEqual(encode(allocation,values),encode(view,values))

    def test_depth_payload_matches_representable_unorm(self):
        for fmt,denominator in ((55,65535),(44,16777215),(45,16777215)):
            s=next(s for s in workloads() if s['name']==f'role-1-{fmt}-uav0')
            raw=inputs(s,0,1)
            self.assertEqual(len(raw),320*180*4)
            left,right=struct.unpack_from('<f',raw,0)[0],struct.unpack_from('<f',raw,160*4)[0]
            expected=[struct.unpack('<f',struct.pack('<f',round(v*denominator)/denominator))[0] for v in (0.25,0.75)]
            self.assertEqual([left,right],expected)

    def test_independent_motion_exposure_and_mask_roles(self):
        selections=workloads()
        for fmt in (15,33):
            s=next(s for s in selections if s['name']==f'role-2-{fmt}-uav1')
            self.assertEqual(inputs(s,1,2)[:8 if fmt==15 else 4],struct.pack('<2f' if fmt==15 else '<2e',-4,0))
        for mask in (1,2):
            self.assertTrue(any(s['mask']==mask and s['formats'][6]==60 for s in selections))
        for fmt in (5,6):
            self.assertFalse(any(s['formats'][4]==fmt or s['formats'][0]==fmt and s['uav']&1 for s in selections))

    def test_public_case_inventory(self):
        selections=workloads()
        self.assertEqual(len({s['name'] for s in selections}),len(selections))
        native=[c for c in cases() if c.scope=='native-gpu']
        expected=sum(2*(1 if s['generate'] else 2) for s in selections)
        self.assertEqual(len(native),expected)
        for s in selections:
            for provider in ('3.1.5','4.1.1'):
                if s['generate'] and provider!='3.1.5': continue
                for family in ('local','movable'):
                    self.assertTrue(any(c.id==f'resource-formats.{provider}.{family}.'+s['name'] for c in native))


if __name__=='__main__': unittest.main()
