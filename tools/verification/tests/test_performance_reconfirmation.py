# SPDX-License-Identifier: MPL-2.0
"""Fixed P08 follow-up controls; no native measurements are launched."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import performance_characterization as dispatch
import release_performance as release
from execution import Rejected
from profiles import compose
from records import save, sha, source_identity


class Reconfirmation(unittest.TestCase):
    def fixture(self, root):
        previous = root/'previous'
        run = previous/'confirmation'
        (run/'analysis').mkdir(parents=True)
        archive = root/'candidate.crate'
        archive.write_bytes(b'unchanged packaged sources')
        archives = {name: {'path': str(archive), 'sha256': sha(archive)}
                    for name in ('fsr-sdk', 'fsr-sdk-sys')}
        sdk = {'sdk/header.h': 'fixed-sdk'}
        save(previous/'status.json', {'status': 'completed', 'source_identity': 'old-producer',
            'confirmation_analysis': str(run/'analysis/performance-characterization.json'),
            'archives': archives, 'sdk_inputs': sdk})
        save(run/'inputs.json', {'identity': 'old-inputs',
            'files': [{'path': path, 'sha256': digest} for path, digest in sdk.items()]})
        save(run/'analysis/performance-characterization.json', {
            'phase': 'confirmation', 'protocol': dispatch.protocol(), 'equipment': {'provider': 'fixed'},
            'measurement_seconds': 1123.4226606,
            'findings': {mode: {'all': {'cpu': {'blocks': 26, 'block_sd_ns': 4562.510629}}}
                         for mode in dispatch.protocol()['workloads']}})
        return previous, run, archives

    def test_freezes_fresh_256_plan_validates_and_never_pools(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            previous, _, archives = self.fixture(root)
            path = root/'selection.json'
            with patch('release_performance.verify_run', return_value={'status': 'passed', 'inputs_unchanged': True}), \
                    patch('release_performance.candidates', return_value=archives):
                plan = release.prepare_reconfirmation(previous, path)
                with self.assertRaises(FileExistsError):
                    release.prepare_reconfirmation(previous, path)
            self.assertEqual(dispatch.load_selection(path, dispatch.RECONFIRM_PROTOCOL)['blocks'], 256)
            self.assertLess(plan['planned_margin_ns'], 500)
            self.assertAlmostEqual(plan['planned_margin_ns'], 470.752, places=2)
            self.assertLess(plan['predicted_measurement_seconds'], 18000)
            self.assertFalse(plan['planning_only']['pool_previous_samples'])
            self.assertNotIn('block_differences_ns', plan)
            with self.assertRaises(Rejected):
                dispatch.load_selection(path)  # Cannot silently reinterpret as the old protocol.
            cases = compose('release-performance-reconfirm', performance_plan=path)
            primary = next(case for case in cases if '.block' in case.id)
            runner = SimpleNamespace(state={'characterization.started': 0})
            with patch('performance_characterization.time.monotonic', return_value=18000):
                with self.assertRaisesRegex(Rejected, 'allowance exhausted'):
                    primary.action(runner)
            ids = set()
            for case in cases:
                self.assertTrue(set(case.requires) <= ids)
                self.assertNotIn(case.id, ids)
                ids.add(case.id)
            self.assertEqual(sum('.block' in name for name in ids), 2560)
            self.assertEqual(sum('.lifecycle' in name for name in ids), 32)
            self.assertEqual(sum('.supplement' in name and name != 'characterization.supplements' for name in ids), 45)
            for key, value in (('blocks', 258), ('seed', 0), ('cpu_budget_ns', 6000),
                               ('measurement_limit_seconds', 7200), ('schedule', []),
                               ('producer_identity', 'changed')):
                save(root/'bad.json', plan | {key: value})
                with self.subTest(key=key), self.assertRaises(Rejected):
                    dispatch.load_selection(root/'bad.json', dispatch.RECONFIRM_PROTOCOL)

    def test_invalid_evidence_archive_sdk_or_planning_cannot_freeze(self):
        for failure in ('evidence', 'archive', 'sdk', 'noise', 'runtime', 'count'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                previous, run, archives = self.fixture(root)
                checked = {'status': 'passed', 'inputs_unchanged': failure != 'evidence'}
                current = {name: item | {'sha256': 'changed'} for name, item in archives.items()} if failure == 'archive' else archives
                if failure == 'sdk':
                    save(run/'inputs.json', {'identity': 'changed', 'files': []})
                analysis_path = run/'analysis/performance-characterization.json'
                analysis = json.loads(analysis_path.read_text(encoding='utf-8'))
                if failure == 'noise': analysis['findings']['max']['all']['cpu']['block_sd_ns'] = 100000
                if failure == 'runtime': analysis['measurement_seconds'] = 100000
                if failure == 'count': analysis['findings']['max']['all']['cpu']['blocks'] = 25
                save(analysis_path, analysis)
                with patch('release_performance.verify_run', return_value=checked), \
                        patch('release_performance.candidates', return_value=current):
                    with self.assertRaises(Rejected):
                        release.prepare_reconfirmation(previous, root/'selection.json')
                self.assertFalse((root/'selection.json').exists())

    def test_fixed_batch_skips_pilot_and_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            # Exercise canonicalization on every host, without symlink privileges.
            spelling = Path(temporary)/'path spelling'
            spelling.mkdir()
            root = spelling/'..'
            plan_path = root/'plan.json'
            plan = {'blocks': 256, 'predicted_measurement_seconds': 16000}
            calls = []

            def profile(args):
                calls.append(args[1])
                self.assertEqual(Path(args[args.index('--performance-plan')+1]), plan_path.resolve())
                directory = Path(args[args.index('--output-root')+1])/('test-'+args[1]+'-attempt')
                directory.mkdir(parents=True)
                save(directory/'run.json', {'status': 'passed'})
                return 0

            with patch('performance_characterization.load_selection', return_value=plan) as loaded, \
                    patch('performance_characterization.selection') as adaptive, \
                    patch('run.main', side_effect=profile), contextlib.redirect_stdout(io.StringIO()):
                result = dispatch.run_batch('sdk', output_root=root, fixed_plan=plan_path)
            adaptive.assert_not_called()
            loaded.assert_called_once_with(plan_path.resolve(), dispatch.RECONFIRM_PROTOCOL)
            self.assertEqual(calls, ['release-performance-reconfirm'])
            status = json.loads((result/'status.json').read_text(encoding='utf-8'))
            self.assertEqual(status['status'], 'completed')
            self.assertEqual(len(status['stages']), 1)

    def test_preview_is_read_only_and_uses_the_fixed_protocol(self):
        output = io.StringIO()
        with patch('release_performance.run_batch') as run, contextlib.redirect_stdout(output):
            self.assertEqual(release.main(['plan', '--reconfirmation']), 0)
        run.assert_not_called()
        spec = json.loads(output.getvalue())
        self.assertEqual(spec['minimum_confirmation_blocks'], 256)
        self.assertEqual(spec['maximum_confirmation_blocks'], 256)
        self.assertEqual(spec['confirmation_measurement_limit_seconds'], 18000)
        self.assertEqual(spec['cpu_budget_ns'], 5000)

    def test_cli_selects_fixed_mode(self):
        with patch('release_performance.run_batch') as run:
            self.assertEqual(release.main(['reconfirm', '--planning-batch', 'prior', '--sdk-root', 'sdk']), 0)
        self.assertEqual(run.call_args.kwargs, {'planning_batch': 'prior'})


if __name__ == '__main__':
    unittest.main()
