# SPDX-License-Identifier: MPL-2.0
"""LF generation and exact retained bytes across Git/platform conversions."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from records import export, inventory, save, source_identity, verify_artifacts

ROOT = Path(__file__).resolve().parents[3]
HISTORICAL = (
    ('2026-10-11-exp-temporal-quality-metric-diagnosis', 12),
    ('2026-10-11-exp-temporal-quality-validation', 12),
    ('2026-10-11-exp-temporal-long-sequence', 11),
    ('2026-10-11-exp-temporal-reset-recovery', 21),
    ('2026-10-12-exp-temporal-jitter-phase', 21),
)


class LineEndingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_json_save_has_platform_independent_utf8_lf_bytes(self):
        path = self.root / 'record.json'
        value = {'message': 'caf\u00e9\nsecond line', 'value': 7}
        save(path, value)
        expected = (json.dumps(value, indent=2) + '\n').encode('utf-8')
        self.assertEqual(path.read_bytes(), expected)
        self.assertNotIn(b'\r', path.read_bytes())
        self.assertFalse(path.with_suffix('.json.pending').exists())

    def test_line_ending_policy_is_part_of_source_identity(self):
        (self.root / '.gitattributes').write_bytes(b'* text=auto eol=lf\n')
        first = source_identity(self.root)
        self.assertEqual([entry['path'] for entry in first['files']], ['.gitattributes'])
        (self.root / '.gitattributes').write_bytes(b'* -text\n')
        self.assertNotEqual(source_identity(self.root)['identity'], first['identity'])

    def test_historical_evidence_matches_original_sizes_and_hashes(self):
        for name, count in HISTORICAL:
            with self.subTest(record=name):
                directory = ROOT / 'docs/research/records' / name
                if (directory / 'artifacts.json').exists():
                    entries = verify_artifacts(directory)
                else:
                    entries = json.loads((directory / 'evidence.json').read_text(encoding='utf-8'))['artefacts']
                    self.assertEqual({p.name for p in directory.iterdir()},
                                     {entry['path'] for entry in entries} | {'evidence.json'})
                    for entry in entries:
                        raw = (directory / entry['path']).read_bytes()
                        self.assertEqual(len(raw), entry['bytes'], entry['path'])
                        self.assertEqual(hashlib.sha256(raw).hexdigest(), entry['sha256'], entry['path'])
                self.assertEqual(len(entries), count)

    def test_export_and_historical_bytes_survive_git_round_trips(self):
        # Ignore user/system Git settings; exercise Windows-style conversions
        # on every host without changing the real repository or its index.
        environment = {key: value for key, value in os.environ.items() if not key.startswith('GIT_')}
        environment.update(GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull, GIT_ATTR_NOSYSTEM='1')
        for mode in ('true', 'false', 'input'):
            with self.subTest(autocrlf=mode):
                checkout = self.root / mode
                checkout.mkdir()

                def git(*args):
                    return subprocess.run(
                        ['git', '-C', str(checkout), '-c', 'core.autocrlf=' + mode,
                         '-c', 'core.eol=crlf', '-c', 'core.safecrlf=false',
                         '-c', 'core.attributesFile=' + os.devnull, *args],
                        env=environment, check=True, capture_output=True).stdout

                git('init', '--quiet')
                for relative in ('.gitattributes', 'docs/research/records/.gitattributes',
                                 'tools/verification/archive/.gitattributes'):
                    target = checkout / relative
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes((ROOT / relative).read_bytes())

                preserved = {}
                for name, _ in HISTORICAL:
                    preserved['docs/research/records/' + name + '/checks.json'] = b'{"value": 7}\r\n'
                preserved['tools/verification/archive/2026-10-10/files/probe.ps1.txt'] = b'Write-Output 7\r\n'
                for relative, raw in preserved.items():
                    target = checkout / relative
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(raw)
                (checkout / 'source.rs').write_bytes(b'// source\r\n')

                source = self.root / ('attempt-' + mode)
                (source / 'cases/example').mkdir(parents=True)
                (source / 'readbacks').mkdir()
                save(source / 'run.json', {'profile': 'fixture', 'status': 'passed'})
                save(source / 'inputs.json', {'identity': 'fixture'})
                raw_log = 'caf\u00e9\r\nsecond line\r\n'.encode('utf-8')
                (source / 'cases/example/stdout.txt').write_bytes(raw_log)
                (source / 'cases/example/samples.csv').write_bytes(b'value\r\n7\r\n')
                # A text-looking verbatim payload would be converted by Git's
                # auto detection without the export's byte-preservation rule.
                (source / 'readbacks/raw.payload').write_bytes(b'raw\r\ncontent\r\n')
                (source / 'readbacks/binary.bin').write_bytes(b'\x00\r\n\xff')
                save(source / 'artifacts.json', inventory(source))
                original = {p.relative_to(source).as_posix(): p.read_bytes()
                            for p in source.rglob('*') if p.is_file()}
                destination = export(source, checkout / 'evidence', checkout)
                verify_artifacts(destination)
                exported = {p.relative_to(destination).as_posix(): p.read_bytes()
                            for p in destination.rglob('*') if p.is_file()}
                self.assertEqual(exported['cases/example/stdout.txt'], raw_log.replace(b'\r\n', b'\n'))
                self.assertEqual(exported['cases/example/samples.csv'], b'value\n7\n')
                for relative, raw in exported.items():
                    if relative.endswith(('.json', '.txt', '.csv')):
                        self.assertNotIn(b'\r', raw, relative)

                git('add', '--all')
                self.assertEqual(git('show', ':source.rs'), b'// source\n')
                # Force a real fresh checkout; checkout-index may otherwise
                # retain an existing file whose cached stat still matches.
                for relative in git('ls-files', '-z').decode('utf-8').split('\0'):
                    if relative:
                        (checkout / relative).unlink()
                git('checkout-index', '--all', '--force')
                self.assertEqual((checkout / 'source.rs').read_bytes(), b'// source\n')
                for relative, raw in preserved.items():
                    self.assertEqual(git('show', ':' + relative), raw)
                    self.assertEqual((checkout / relative).read_bytes(), raw)
                for relative, raw in exported.items():
                    self.assertEqual(git('show', ':evidence/' + relative), raw)
                    self.assertEqual((destination / relative).read_bytes(), raw)
                verify_artifacts(destination)
                self.assertEqual({p.relative_to(source).as_posix(): p.read_bytes()
                                  for p in source.rglob('*') if p.is_file()}, original)
                verify_artifacts(source)
                # Corruption is still rejected after a successful round trip.
                (destination / 'cases/example/stdout.txt').write_bytes(b'changed\n')
                with self.assertRaisesRegex(ValueError, 'identity mismatch'):
                    verify_artifacts(destination)


if __name__ == '__main__':
    unittest.main()
