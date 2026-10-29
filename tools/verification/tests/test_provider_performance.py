# SPDX-License-Identifier: MPL-2.0
"""Provider A/B evidence controls; native execution is never launched here."""
import copy
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
import provider_performance as probe
from profiles import compose
from test_active_timing import log


class ProviderPerformance(unittest.TestCase):
    def test_schedule_preserves_pairs_and_balances_adjacent_blocks(self):
        spec = probe.protocol()
        orders = probe.schedule(spec)
        self.assertEqual(len(orders), 8)
        self.assertEqual(orders, probe.schedule(spec))
        for left, right in zip(orders[::2], orders[1::2], strict=True):
            self.assertEqual(left, right[::-1])
            self.assertEqual(set(left), set(spec['arms']))

    def test_probe_adapts_both_query_and_create_and_rejects_source_drift(self):
        source = probe.REFERENCE.read_text(encoding='utf-8')
        output = probe.probe_source(source)
        self.assertIn('device_query.header.pNext=explicit_provider', output)
        self.assertIn('backend.header.pNext=explicit_provider', output)
        self.assertIn('provider.versionId==expected_provider', output)
        self.assertIn('old_available', output)
        with self.assertRaises(Rejected):
            probe.probe_source(source.replace('backend.device=device.Get();', 'backend.device=nullptr;'))

    def test_provider_output_bits_may_differ_but_inputs_and_equipment_must_match(self):
        from active_timing import parse
        base = parse(log('max'), arm='cpp-minimal', mode='max')
        other = copy.deepcopy(base)
        other['provider']['id'] += 1
        other['output_sha256'] = 'different'
        for row in other['checkpoints']:
            row[4] += 1
        probe.matched(base, other)
        for key in ('jitter_bits', 'spec', 'adapter_driver'):
            bad = copy.deepcopy(other)
            bad[key] = None
            with self.subTest(key=key), self.assertRaises(Rejected):
                probe.matched(base, bad)
        other['checkpoints'][0][5] += 1
        with self.assertRaises(Rejected):
            probe.matched(base, other)

    def test_whole_graph_and_build_only_graph_are_explicit(self):
        for name in ('experiment-provider-performance', 'experiment-provider-performance-build'):
            known = set()
            for case in compose(name):
                self.assertNotIn(case.id, known)
                self.assertTrue(set(case.requires) <= known)
                known.add(case.id)
            self.assertIn('provider-performance.build', known)
            self.assertEqual(sum('.block' in key for key in known), 0 if name.endswith('-build') else 16)
            if name.endswith('-build'):
                self.assertFalse(any(case.scope == 'native-experiment' for case in compose(name)))

    def test_pair_estimates_use_eight_process_means_and_reject_missing_pairs(self):
        spec = probe.protocol()
        processes = {}
        for block in range(1, 9):
            for arm in spec['arms']:
                value = 1000+block+(100 if arm == 'default' else 0)
                processes[f'{block}.{arm}'] = {
                    'segments': {'all': {clock: {'mean_ns': value} for clock in ('cpu', 'gpu')}},
                    'first_dispatch_cpu_ns': value,
                    'lifecycle': {metric: {'mean_ns': value} for metric in
                        ('load', 'provider_query', 'first_create', 'warm_create', 'used_destroy', 'unused_destroy')}}
        analysis = probe.summarize(processes, spec)
        self.assertIsNone(analysis['speed_gate'])
        for metric in analysis['metrics'].values():
            self.assertEqual(metric['default_minus_explicit']['blocks'], 8)
            self.assertEqual(metric['default_minus_explicit']['mean_difference_ns'], 100)
        processes.pop('8.default')
        with self.assertRaises(Rejected):
            probe.summarize(processes, spec)

    def test_native_identity_mismatch_cannot_silently_fall_back(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'readbacks').mkdir()
            runner = SimpleNamespace(directory=root, options=SimpleNamespace(adapter_index=0),
                state={'native_stage': root, 'provider-performance.cpp': root/'probe.exe'},
                process=lambda *args, **kwargs: ('', log('max'), {}))
            case = next(case for case in probe.cases() if case.id == 'provider-performance.block1.default')
            with self.assertRaisesRegex(Rejected, 'identity mismatch'):
                case.action(runner)

    def test_watchdog_rejects_before_launching_another_child(self):
        case = next(case for case in probe.cases() if '.block' in case.id)
        runner = SimpleNamespace(state={'provider-performance.started': 0})
        with patch('provider_performance.time.monotonic', return_value=600):
            with self.assertRaisesRegex(Rejected, 'allowance exhausted'):
                case.action(runner)


if __name__ == '__main__':
    unittest.main()
