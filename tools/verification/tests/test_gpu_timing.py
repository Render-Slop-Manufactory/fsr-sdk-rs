# SPDX-License-Identifier: MPL-2.0
"""Reject invalid timing evidence independently of the observed speed."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from cpu_comparison import expected_spec
from execution import Rejected
from gpu_timing import compatible, parse
from profiles import compose


def log(debug=False):
    frames=4 if debug else 160
    return ('Adapter: test GPU\nDriver query: version=1\nCPU_CLOCK 10000000\nCPU_PROVIDER 1234 test\n'+
            'CPU_SPEC '+json.dumps(expected_spec((1280,720),(1920,1080)))+'\nGPU_QUERY 1280 720 18\n'+
            ''.join(f'CPU_FRAME {i} 400\n' for i in range(frames))+
            ''.join(f'CPU_EMPTY {i} 0\n' for i in range(256))+
            ''.join(f'GPU_FRAME {i} {1000+i*1000} {1002+i*1000} {1003+i*1000} {1103+i*1000} 25000000\nGPU_JITTER {i} 0 0\n' for i in range(frames))+
            f'CPU_VALID sentinel=0 finite=2073600 written=2073600 removed=0 debug={str(debug).lower()} preinspect=false\n'+
            'CPU_COMPARISON PASSED cpp-minimal\n')


class GpuTimingEvidence(unittest.TestCase):
    def test_separate_clock_conversion_and_warmup(self):
        measured=parse(log(),arm='cpp-minimal')
        self.assertEqual(measured['measured_ns'],[40000]*128)
        self.assertEqual(measured['gpu_ns'],[4000]*128)
        self.assertEqual(measured['empty_gpu_ns'],[80]*160)
        self.assertEqual(len(measured['gpu_ticks']),160)
        self.assertEqual(len(parse(log(True),arm='cpp-minimal',debug=True)['gpu_ns']),4)

    def test_incomplete_duplicate_and_invalid_queries_rejected(self):
        for text in (log()+'GPU_QUERY 1280 720 18\n',log().replace('GPU_QUERY 1280 720 18','GPU_QUERY 0 720 18'),
                     log().replace('GPU_JITTER 3 0 0\n',''),log()+'GPU_JITTER 3 0 0\n',
                     log().replace('GPU_FRAME 0 1000 1002 1003 1103 25000000\n','')):
            with self.assertRaises(Rejected): parse(text,arm='cpp-minimal')

    def test_clocks_order_empty_dispatch_and_jitter_rejected(self):
        for old,new in (('25000000','0'),
                        ('1000 1002 1003 1103 25000000','1000 1002 1003 1103 25000001'),
                        ('1000 1002 1003 1103','1000 999 1003 1103'),
                        ('1000 1002 1003 1103','1000 1002 1003 1003'),
                        ('2000 2002 2003 2103','1 2 3 4'),
                        ('GPU_JITTER 0 0 0','GPU_JITTER 0 2139095040 0'),
                        ('GPU_JITTER 18 0 0','GPU_JITTER 18 1 0')):
            with self.subTest(new=new),self.assertRaises(Rejected):
                parse(log().replace(old,new),arm='cpp-minimal')

    def test_cross_consumer_and_output_mismatch_rejected(self):
        original=parse(log(),arm='cpp-minimal'); original['output_sha256']='a'*64
        compatible(original,copy.deepcopy(original))
        for key,value in (('gpu_frequency_hz',1),('query',{'render':[640,360],'phase':18}),
                          ('jitter_bits',[[1,0]]*160),('output_sha256','b'*64)):
            changed=copy.deepcopy(original); changed[key]=value
            with self.assertRaises(Rejected): compatible(original,changed)

    def test_profile_orders_and_dependencies(self):
        protocol=json.loads((Path(__file__).resolve().parents[1]/'gpu-timing-protocol.json').read_text())
        self.assertEqual(protocol['frames']-protocol['warmup'],128)
        for position in range(2):
            self.assertEqual(sorted(order[position] for order in protocol['process_orders']),
                             ['cpp-minimal']*2+['rust-wrapper']*2)
        cases=compose('experiment-gpu-timing')
        known=set()
        for case in cases:
            self.assertTrue(set(case.requires)<=known)
            self.assertNotIn(case.id,known)
            known.add(case.id)
        self.assertEqual(len(cases),22)
