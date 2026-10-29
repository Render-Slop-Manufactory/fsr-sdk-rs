# SPDX-License-Identifier: MPL-2.0
"""Selected-feature evidence rejects missing masks, wrong HDR and bad controls."""
import copy
import json
from pathlib import Path
import re
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from active_timing import parse as parse_active
from cpu_comparison import expected_spec
from execution import Rejected
from feature_comparison import fnv, mask_hashes, parse, selected_protocol, summarize
from profiles import compose
from test_active_timing import log as active_log


def log(feature, debug=False):
    result = active_log('max', debug)
    result = result.replace(json.dumps(expected_spec((1280, 720), (1920, 1080), dynamic=True)),
        json.dumps(expected_spec((1280, 720), (1920, 1080), dynamic=True, hdr=feature=='hdr', masks=feature=='masks')))
    if feature == 'hdr':
        def scale(match):
            fields = match[0].split()
            for index in (4, 5, 6):
                value = struct.unpack('<e', struct.pack('<H', int(fields[index])))[0]*8
                fields[index] = str(struct.unpack('<H', struct.pack('<e', value))[0])
            return ' '.join(fields)
        result = re.sub(r'^ACTIVE_INPUT .+$', scale, result, flags=re.M)
    for frame in ([0, 1, 2, 3] if debug else [31, 63, 64, 111, 112, 159]):
        hashes = mask_hashes(1280, 720) if feature == 'masks' else [0, 0]
        maximum = struct.unpack('<I', struct.pack('<f', 6.4 if feature=='hdr' else .8))[0]
        result += f'FEATURE_VALID {frame} {1 if feature=="masks" else 2} {maximum} {hashes[0]} {hashes[1]}\n'
        if feature == 'masks':
            result += 'Supplied mask input readback: PASSED index=0\nSupplied mask input readback: PASSED index=1\n'
    return result


class FeatureEvidence(unittest.TestCase):
    def test_full_mask_integer_oracle_and_known_hash(self):
        self.assertEqual(fnv(b'hello'), 0xa430d84680aabd0b)
        self.assertEqual(mask_hashes(4, 4), [fnv(bytes([0,128,0,0])*4),
            fnv(bytes([0]*8+[255]*4+[0]*4))])

    def test_both_features_and_history_lengths(self):
        for feature in ('masks', 'hdr'):
            for debug in (False, True):
                value = parse(log(feature, debug), arm='cpp-minimal', feature=feature, debug=debug)
                self.assertEqual(value['mode'], feature)
                self.assertEqual(value['spec']['creation'][4], 65 if feature=='hdr' else 64)
                self.assertEqual(len(value['feature_checkpoints']), 4 if debug else 6)

    def test_canonical_flags_and_mask_presence_are_required(self):
        for feature in ('masks', 'hdr'):
            old = json.dumps(expected_spec((1280,720),(1920,1080),dynamic=True,hdr=feature=='hdr',masks=feature=='masks'))
            changed = log(feature).replace(old, json.dumps(expected_spec((1280,720),(1920,1080),dynamic=True)))
            with self.assertRaises(Rejected): parse(changed,arm='cpp-minimal',feature=feature)

    def test_missing_wrong_hash_and_unverified_masks_fail(self):
        base = log('masks')
        line = next(line for line in base.splitlines() if line.startswith('FEATURE_VALID 64 '))
        wrong = line.split(); wrong[-1] = '1'
        for text in (base.replace(line+'\n',''), base+line+'\n', base.replace(line,' '.join(wrong)),
                     base.replace('Supplied mask input readback: PASSED index=0\n','',1)):
            with self.assertRaises(Rejected): parse(text,arm='cpp-minimal',feature='masks')

    def test_sdr_input_clipping_and_nonfinite_hdr_fail(self):
        base = log('hdr')
        for maximum in (1., float('inf')):
            old = str(struct.unpack('<I',struct.pack('<f',6.4))[0])
            new = str(struct.unpack('<I',struct.pack('<f',maximum))[0])
            with self.assertRaises(Rejected): parse(base.replace(old,new),arm='cpp-minimal',feature='hdr')
        old = next(line for line in base.splitlines() if line.startswith('ACTIVE_INPUT 64 0 0 '))
        wrong = next(line for line in active_log('max').splitlines() if line.startswith('ACTIVE_INPUT 64 0 0 '))
        with self.assertRaises(Rejected): parse(base.replace(old,wrong),arm='cpp-minimal',feature='hdr')

    def test_summary_retains_controls_and_signed_feature_differences(self):
        processes = {}
        for block in range(1, 5):
            for mode in selected_protocol()['workloads']:
                value = parse(log(mode),arm='cpp-minimal',feature=mode) if mode in ('masks','hdr') else parse_active(active_log(mode),arm='cpp-minimal',mode=mode)
                value['output_sha256'] = 'a'*64
                for arm in ('cpp-minimal','rust-wrapper'):
                    processes[f'active.block{block}.{mode}.{arm}'] = copy.deepcopy(value)
        summary = summarize(processes,selected_protocol())
        self.assertEqual(len(summary['active_controls']['blocks']),4)
        self.assertIsNone(summary['performance_threshold'])
        self.assertEqual(summary['blocks'][0]['feature_minus_sdr_median_ns']['masks']['rust-wrapper']['gpu'],0)
        processes['active.block1.masks.rust-wrapper']['checkpoints'][0][5] += 1
        with self.assertRaises(Rejected): summarize(processes,selected_protocol())

    def test_profile_balances_features_without_duplicate_setup(self):
        cases = compose('experiment-feature-comparison')
        self.assertEqual(len(cases),62)
        self.assertEqual(sum(case.id=='native.helper' for case in cases),1)
        known = set()
        for case in cases:
            self.assertTrue(set(case.requires)<=known)
            self.assertNotIn(case.id,known)
            known.add(case.id)
        for mode in ('masks','hdr'):
            self.assertEqual(sum(f'.{mode}.' in case.id for case in cases),10)
        self.assertEqual(len(compose('experiment-active-timing')),42)
