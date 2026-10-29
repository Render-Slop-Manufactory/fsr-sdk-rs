# SPDX-License-Identifier: MPL-2.0
"""Portable archive integrity and explicit PowerShell syntax outcomes."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from archive import check
from profiles import infrastructure


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.base = self.root / 'tools/verification/archive'
        self.archive = self.base / '2026-10-10'
        self.archive.mkdir(parents=True)
        (self.base / 'README.md').write_text('[Archive](2026-10-10/README.md)', encoding='utf-8')
        (self.archive / 'README.md').write_text('Fixture archive', encoding='utf-8')
        self.manifest = {'schema_version': 1, 'archived_on': self.archive.name,
                         'groups': {'fixture': {'repository_inputs': []}}, 'entries': []}
        for original, data in (('scripts/probe.py', b'raise RuntimeError("must not execute")\n'),
                               ('scripts/probe.ps1', b'throw "must not execute"\n'),
                               ('scripts/config.json', b'{"fixture": true}\n')):
            payload = self.archive / ('files/' + original + '.txt')
            payload.parent.mkdir(parents=True, exist_ok=True)
            payload.write_bytes(data)
            self.manifest['entries'].append({'original': original, 'archived': 'files/' + original + '.txt',
                'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'group': 'fixture',
                'disposition': 'retired', 'depends_on': [], 'referenced_by': []})
        self.save_manifest()
        self.output = io.StringIO()
        self.enterContext(contextlib.redirect_stdout(self.output))
        self.enterContext(patch.object(check, 'ROOT', self.root))
        self.enterContext(patch.object(check, 'BASE', self.base))

    def save_manifest(self):
        (self.archive / 'manifest.json').write_text(json.dumps(self.manifest), encoding='utf-8')

    def run_check(self, *args):
        report = self.root / 'report.json'
        check.main(['--report', str(report), *args])
        return json.loads(report.read_text(encoding='utf-8'))['archives'][0]

    def test_missing_parser_is_reported_without_claiming_syntax_pass(self):
        with patch.object(check.shutil, 'which', return_value=None), patch.object(check.subprocess, 'run') as parser:
            result = self.run_check()
        parser.assert_not_called()
        self.assertEqual(result['integrity']['evaluation'], 'passed')
        self.assertEqual(result['python_syntax']['sources'], 1)
        self.assertEqual(result['json_syntax']['sources'], 1)
        syntax = result['powershell_syntax']
        self.assertEqual(syntax['execution'], 'not_run')
        self.assertEqual(syntax['evaluation'], 'not_evaluated')
        self.assertIn('unavailable', syntax['reason'])
        self.assertFalse(syntax['required'])
        self.assertIn('PowerShell syntax: not_run, not_evaluated', self.output.getvalue())

    def test_strict_missing_parser_fails_and_retains_report(self):
        with patch.object(check.shutil, 'which', return_value=None):
            with self.assertRaisesRegex(SystemExit, 'required PowerShell syntax check was not executed'):
                self.run_check('--require-powershell')
        syntax = json.loads((self.root / 'report.json').read_text())['archives'][0]['powershell_syntax']
        self.assertTrue(syntax['required'])
        self.assertEqual(syntax['execution'], 'not_run')
        self.assertNotEqual(syntax['evaluation'], 'passed')

    def test_parser_success_and_failure_are_distinct(self):
        for exit_code, stdout in ((0, '1\n'), (1, 'Syntax error in frozen source'), (0, '0\n')):
            with self.subTest(exit_code=exit_code, stdout=stdout):
                parsed = subprocess.CompletedProcess([], exit_code, stdout, '')
                with patch.object(check.shutil, 'which', return_value='pwsh'), patch.object(check.subprocess, 'run', return_value=parsed):
                    result = check.check_archive(self.archive, require_powershell=True)
                self.assertEqual(result['powershell_syntax']['evaluation'],
                                 'passed' if exit_code == 0 and stdout == '1\n' else 'failed')

    def test_parser_failure_rejects_default_mode_and_retains_report(self):
        parsed = subprocess.CompletedProcess([], 1, 'Syntax error in frozen source', '')
        with patch.object(check.shutil, 'which', return_value='pwsh'), patch.object(check.subprocess, 'run', return_value=parsed):
            with self.assertRaisesRegex(SystemExit, 'PowerShell syntax check failed'):
                self.run_check()
        syntax = json.loads((self.root / 'report.json').read_text())['archives'][0]['powershell_syntax']
        self.assertEqual(syntax['evaluation'], 'failed')
        self.assertIn('Syntax error', syntax['reason'])

    def test_unusable_parser_is_a_failure(self):
        for error, execution in ((OSError('parser unavailable'), 'blocked'),
                                 (subprocess.TimeoutExpired('pwsh', 30), 'timed_out')):
            with self.subTest(execution=execution):
                with patch.object(check.shutil, 'which', return_value='pwsh'), patch.object(check.subprocess, 'run', side_effect=error):
                    result = check.check_archive(self.archive)
                self.assertEqual(result['powershell_syntax']['execution'], execution)
                self.assertEqual(result['powershell_syntax']['evaluation'], 'failed')

    def test_corrupt_payload_is_rejected_without_parser(self):
        payload = self.archive / self.manifest['entries'][1]['archived']
        payload.write_bytes(b'changed')
        with patch.object(check.shutil, 'which', return_value=None):
            with self.assertRaisesRegex(SystemExit, 'Payload identity changed'):
                self.run_check()

    def test_extra_payload_is_rejected_without_parser(self):
        (self.archive / 'files/extra.txt').write_text('extra')
        with patch.object(check.shutil, 'which', return_value=None):
            with self.assertRaisesRegex(SystemExit, 'Payload inventory'):
                self.run_check()

    def test_portable_syntax_is_checked_without_parser(self):
        row = self.manifest['entries'][0]
        data = b'if (\n'
        (self.archive / row['archived']).write_bytes(data)
        row.update(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
        self.save_manifest()
        with patch.object(check.shutil, 'which', return_value=None):
            with self.assertRaises(SyntaxError):
                self.run_check()

    def test_report_is_not_overwritten(self):
        report = self.root / 'report.json'
        report.write_text('existing report', encoding='utf-8')
        with patch.object(check.shutil, 'which', return_value=None):
            with self.assertRaises(FileExistsError):
                self.run_check()
        self.assertEqual(report.read_text(encoding='utf-8'), 'existing report')

    @unittest.skipUnless(shutil.which('pwsh'), 'PowerShell parser unavailable')
    def test_real_parser_parses_without_executing_frozen_source(self):
        result = self.run_check('--require-powershell')
        self.assertEqual(result['powershell_syntax']['evaluation'], 'passed')

    @unittest.skipUnless(shutil.which('pwsh'), 'PowerShell parser unavailable')
    def test_real_parser_rejects_invalid_frozen_source(self):
        row = self.manifest['entries'][1]
        data = b'if (\n'
        (self.archive / row['archived']).write_bytes(data)
        row.update(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
        self.save_manifest()
        with self.assertRaisesRegex(SystemExit, 'PowerShell syntax check failed'):
            self.run_check('--require-powershell')

    def test_windows_profile_requires_parser_and_retains_failed_subcheck(self):
        for platform in ('nt', 'posix'):
            with self.subTest(platform=platform):
                report = self.root / f'{platform}/cases/archive.integrity/archive-checks.json'
                report.parent.mkdir(parents=True)
                expected = {'schema_version': 1, 'archives': [{'powershell_syntax':
                            {'execution': 'not_run', 'evaluation': 'not_evaluated', 'reason': 'parser unavailable'}}]}
                report.write_text(json.dumps(expected), encoding='utf-8')
                class Runner:
                    directory = report.parents[2]
                    current = {'id': 'archive.integrity'}
                    def process(self, args):
                        self.args = args
                        raise RuntimeError('required parser unavailable')
                    def note(self, **observations):
                        self.observations = observations
                runner = Runner()
                with patch('profiles.os.name', platform):
                    case = infrastructure()[1]
                    with self.assertRaises(RuntimeError):
                        case.action(runner)
                self.assertEqual('--require-powershell' in runner.args, platform == 'nt')
                self.assertEqual(case.expectation['powershell_syntax_required'], platform == 'nt')
                self.assertEqual(runner.observations['archive_checks'], expected)


if __name__ == '__main__':
    unittest.main()
