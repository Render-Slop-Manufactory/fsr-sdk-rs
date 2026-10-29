# SPDX-License-Identifier: MPL-2.0
"""Performance evidence must reject invalid comparisons, independently of speed."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from cpu_comparison import compatible, expected_spec, parse
from execution import Rejected


def log():
    return ('Adapter: test GPU; index=0\nDriver query: version=1\nCPU_CLOCK 10000000\nCPU_PROVIDER 1234 test\nCPU_SPEC '+json.dumps(expected_spec())+'\n'+
            ''.join(f'CPU_FRAME {index} 400\n' for index in range(160))+
            ''.join(f'CPU_EMPTY {index} 0\n' for index in range(256))+
            'CPU_VALID sentinel=0 finite=230400 written=230400 removed=0 debug=false preinspect=false\n'+
            'CPU_COMPARISON PASSED cpp-minimal\n')


class CpuComparisonEvidence(unittest.TestCase):
    def test_complete_samples_and_timer_units(self):
        result=parse(log(),arm='cpp-minimal')
        self.assertEqual(len(result['measured_ns']),128)
        self.assertEqual(result['measured_ns'][0],40000)
        self.assertEqual(result['empty_ns'],[0]*256)

    def test_duplicate_missing_samples_and_clock_rejected(self):
        for changed in (log()+'CPU_FRAME 32 400\n',log().replace('CPU_FRAME 32 400\n',''),
                        log().replace('CPU_CLOCK 10000000','CPU_CLOCK 0')):
            with self.subTest(changed=changed[-80:]),self.assertRaises(Rejected):
                parse(changed,arm='cpp-minimal')

    def test_descriptor_arm_and_validity_mismatches_rejected(self):
        altered=expected_spec(); altered['shape'][3]=256
        for changed in (log().replace(json.dumps(expected_spec()),json.dumps(altered)),
                        log().replace('PASSED cpp-minimal','PASSED rust-raw'),
                        log().replace('written=230400','written=0')):
            with self.assertRaises(Rejected): parse(changed,arm='cpp-minimal')

    def test_provider_clock_and_output_mismatches_rejected(self):
        original=parse(log(),arm='cpp-minimal'); original['output_sha256']='a'*64
        compatible(original,copy.deepcopy(original))
        for key,value in (('provider',{'id':1235,'name':'test'}),('adapter_driver',{'adapter':'other','driver':'version=1'}),('frequency_hz',1),('output_sha256','b'*64)):
            changed=copy.deepcopy(original); changed[key]=value
            with self.assertRaises(Rejected): compatible(original,changed)

    def test_process_orders_are_balanced(self):
        protocol=json.loads((Path(__file__).resolve().parents[1]/'cpu-comparison-protocol.json').read_text())
        orders=protocol['process_orders']
        self.assertEqual(len(orders),6)
        self.assertEqual(len({tuple(order) for order in orders}),6)
        for position in range(3):
            self.assertEqual(sorted(order[position] for order in orders),
                sorted(['cpp-minimal','rust-raw','rust-wrapper']*2))
