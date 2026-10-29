# SPDX-License-Identifier: MPL-2.0
"""P08 orchestration and candidate checks; every native launch is mocked."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import release_performance as release
from execution import Rejected
from records import save, sha, source_identity


def setup_log():
    return 'Provider setup identity: fixture-provider\n' + ''.join(
        f'PROVIDER_SETUP,{block},{arm},{100+block}\n'
        for block in range(32) for arm in ('default', 'selected'))


class Protocol(unittest.TestCase):
    def test_plan_is_read_only_and_retains_selected_budget(self):
        output = io.StringIO()
        with patch('release_performance.run_batch') as run, contextlib.redirect_stdout(output):
            self.assertEqual(release.main(['plan']), 0)
        run.assert_not_called()
        spec = json.loads(output.getvalue())
        self.assertEqual(spec['cpu_budget_ns'], 5000)
        self.assertEqual(spec['setup_processes'], 3)
        self.assertEqual(spec['workloads'], ['max', 'small', 'transition', 'masks', 'hdr'])

    def test_rejects_a_changed_base_protocol(self):
        with patch('release_performance.dispatch.protocol', return_value={'workloads': [], 'cpu_budget_ns': 5000}):
            with self.assertRaises(Rejected):
                release.protocol()

    def test_setup_requires_exact_paired_samples_and_identity(self):
        log = setup_log()
        self.assertEqual(len(release.setup_samples(log, 32)['samples_ns']['default']), 32)
        for bad in (log+'PROVIDER_SETUP,0,default,100\n',
                    log.replace('PROVIDER_SETUP,4,selected,104\n', ''),
                    log.replace('Provider setup identity: fixture-provider\n', '')):
            with self.subTest(log=bad[-60:]), self.assertRaises(Rejected):
                release.setup_samples(bad, 32)


class CandidateSources(unittest.TestCase):
    def test_archives_must_match_current_source_and_cargo_inventory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archives = {}
            contents = {}
            for name, version in (('fsr-sdk', '0.3.0'), ('fsr-sdk-sys', '0.1.2')):
                source = root/'crates'/name
                (source/'src').mkdir(parents=True)
                manifest = f'[package]\nname="{name}"\nversion="{version}"\n'
                files = {'Cargo.toml.orig': manifest, 'Cargo.toml': manifest,
                         'src/lib.rs': '// measured source\n', 'LICENSE': 'licence\n'}
                (source/'Cargo.toml').write_text(manifest, encoding='utf-8', newline='\n')
                for inner in ('src/lib.rs', 'LICENSE'):
                    (source/inner).write_text(files[inner], encoding='utf-8', newline='\n')
                path = root/(name+'.crate')
                with tarfile.open(path, 'w:gz') as archive:
                    for inner, text in files.items():
                        data = text.encode()
                        entry = tarfile.TarInfo(f'{name}-{version}/{inner}')
                        entry.size = len(data)
                        archive.addfile(entry, io.BytesIO(data))
                archives[name] = path
                contents[name] = files

            def listed(args, **kwargs):
                name = args[args.index('-p')+1]
                return SimpleNamespace(stdout='\n'.join(contents[name])+'\n')

            with patch.object(release, 'ROOT', root), patch('release_performance.subprocess.run', side_effect=listed):
                selected = release.candidates(root, archives['fsr-sdk'], archives['fsr-sdk-sys'])
                self.assertEqual(selected['fsr-sdk']['sha256'], sha(archives['fsr-sdk']))
                (root/'crates/fsr-sdk/src/lib.rs').write_text('// changed\n', encoding='utf-8', newline='\n')
                with self.assertRaises(Rejected):
                    release.candidates(root, archives['fsr-sdk'], archives['fsr-sdk-sys'])
                (root/'crates/fsr-sdk/src/lib.rs').write_text('// measured source\n', encoding='utf-8', newline='\n')
                contents['fsr-sdk']['src/missing.rs'] = '// newly included source\n'
                with self.assertRaises(Rejected):
                    release.candidates(root, archives['fsr-sdk'], archives['fsr-sdk-sys'])

    def test_requires_both_supplied_archives(self):
        with self.assertRaises(Rejected):
            release.candidates(Path('.'), 'wrapper.crate', None)


class UnattendedBatch(unittest.TestCase):
    def exercise(self, failure=None, verdict='within-budget', reconfirm=False):
        with tempfile.TemporaryDirectory() as temporary:
            # Match Windows aliases and other non-canonical input paths on every host.
            spelling = Path(temporary)/'path spelling'
            spelling.mkdir()
            root = spelling/'..'
            (root/'Cargo.toml').write_text('[workspace]\n', encoding='utf-8', newline='\n')
            archive = root/'candidate.crate'
            archive.write_bytes(b'candidate')
            selected = {'fixture': {'path': str(archive), 'sha256': sha(archive)}}
            calls = []

            def fixture_run(path, sdk='fixed-sdk'):
                path.mkdir(parents=True)
                save(path/'run.json', {'status': 'passed', 'inputs_unchanged': True})
                inputs = source_identity(root)
                inputs['files'].append({'path': 'sdk/header.h', 'sha256': sdk})
                save(path/'inputs.json', inputs)
                return path

            def profile(args):
                calls.append('setup')
                output = Path(args[args.index('--output-root')+1])
                path = fixture_run(output/'attempt')
                case = path/'cases/provider-setup.queries'
                case.mkdir(parents=True)
                (case/'stderr.txt').write_text(setup_log(), encoding='utf-8', newline='\n')
                save(case/'result.json', {'commands': [{'stderr': 'cases/provider-setup.queries/stderr.txt'}]})
                if failure == 'source':
                    (root/'Cargo.toml').write_text('[workspace]\n# changed\n', encoding='utf-8', newline='\n')
                if failure == 'archive':
                    archive.write_bytes(b'changed')
                return int(failure == 'setup')

            def dispatch(sdk, adapter, output, fixed_plan=None):
                calls.append('dispatch')
                self.assertEqual(bool(fixed_plan), reconfirm)
                self.assertEqual(output, (root/'batches').resolve())
                if failure == 'dispatch':
                    raise Rejected('Dispatch failed')
                batch = output/'batch'
                pilot = fixture_run(batch/'pilot')
                confirmation = fixture_run(batch/'confirmation', 'changed-sdk' if failure == 'sdk' else 'fixed-sdk')
                analysis = confirmation/'analysis.json'
                save(analysis, {'protocol': release.dispatch.protocol(release.dispatch.RECONFIRM_PROTOCOL if reconfirm else None), 'overall_cpu_verdict': verdict})
                save(batch/'status.json', {'status': 'completed', 'stages': {
                    'pilot': {'run': str(pilot)}, 'confirmation': {'run': str(confirmation)}},
                    'confirmation_analysis': str(analysis)})
                return batch

            with patch.object(release, 'ROOT', root), patch('release_performance.candidates', return_value=selected), \
                    patch('release_performance.prepare_reconfirmation', return_value={
                        'blocks': 256, 'protocol_sha256': 'fixed',
                        'planning_only': {'sdk_inputs': {'sdk/header.h': 'fixed-sdk'}, 'pool_previous_samples': False}}), \
                    patch('run.main', side_effect=profile), patch('release_performance.dispatch.run_batch', side_effect=dispatch), \
                    patch('release_performance.verify_run', side_effect=lambda p: json.loads((p/'run.json').read_text(encoding='utf-8'))), \
                    contextlib.redirect_stdout(io.StringIO()):
                if failure:
                    with self.assertRaises(Rejected):
                        release.run_batch('sdk', output_root=root/'batches', planning_batch=root/'old' if reconfirm else None)
                else:
                    release.run_batch('sdk', output_root=root/'batches', planning_batch=root/'old' if reconfirm else None)
            status = json.loads(next((root/'batches').glob('*-release-performance-*/status.json')).read_text(encoding='utf-8'))
            return status, calls

    def test_serial_batch_retains_results_and_requires_release_review(self):
        status, calls = self.exercise()
        self.assertEqual(calls, ['setup', 'setup', 'setup', 'dispatch'])
        self.assertEqual(status['status'], 'completed')
        self.assertEqual(status['cpu_verdict'], 'within-budget')
        self.assertEqual(status['release_disposition'], 'review-required')
        self.assertEqual(len(status['stages']), 3)

    def test_reconfirmation_uses_fixed_plan_and_remains_review_required(self):
        status, calls = self.exercise(reconfirm=True)
        self.assertEqual(calls, ['setup', 'setup', 'setup', 'dispatch'])
        self.assertEqual(status['reconfirmation']['blocks'], 256)
        self.assertEqual(status['dispatch_mode'], 'independent-fixed-confirmation')
        self.assertEqual(status['protocol'], release.protocol())
        self.assertEqual(status['protocol_sha256'], sha(release.PROTOCOL))
        self.assertFalse(status['reconfirmation']['planning_only']['pool_previous_samples'])
        self.assertEqual(status['release_disposition'], 'review-required')

    def test_failure_or_changed_inputs_never_look_completed(self):
        for failure in ('setup', 'source', 'archive', 'dispatch', 'sdk'):
            with self.subTest(failure=failure):
                status, calls = self.exercise(failure)
                self.assertEqual(status['status'], 'failed')
                if failure in ('setup', 'source', 'archive'):
                    self.assertNotIn('dispatch', calls)

    def test_speed_outcome_is_not_a_release_approval(self):
        for verdict in ('inconclusive', 'exceeds-budget'):
            status, _ = self.exercise(verdict=verdict)
            self.assertEqual(status['status'], 'completed')
            self.assertEqual(status['cpu_verdict'], verdict)
            self.assertEqual(status['release_disposition'], 'review-required')


if __name__ == '__main__':
    unittest.main()
