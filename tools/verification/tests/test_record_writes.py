# SPDX-License-Identifier: MPL-2.0
"""Windows replacement retries preserve complete records and real failures."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from records import save


class RecordWrites(unittest.TestCase):
    def test_transient_windows_denial_retries_same_complete_pending_file(self):
        for code in (5, 32, 33):
            with self.subTest(winerror=code), tempfile.TemporaryDirectory() as directory:
                target = Path(directory)/'result.json'
                target.write_text('{"old": true}', encoding='utf-8')
                denied = PermissionError('Temporary Windows replacement denial')
                denied.winerror = code
                replace = Path.replace
                attempts = []

                def transient(source, destination):
                    attempts.append(source.read_bytes())
                    if len(attempts) == 1:
                        self.assertEqual(json.loads(target.read_text()), {'old': True})
                        raise denied
                    return replace(source, destination)

                with patch.object(Path, 'replace', autospec=True, side_effect=transient), patch('records.time.sleep') as sleep:
                    save(target, {'new': 'colour'})
                self.assertEqual(attempts, [b'{\n  "new": "colour"\n}\n']*2)
                self.assertEqual(json.loads(target.read_text()), {'new': 'colour'})
                self.assertFalse(target.with_suffix('.json.pending').exists())
                sleep.assert_called_once_with(.01)

    def test_persistent_windows_denial_is_bounded_and_preserves_old_record(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)/'result.json'
            target.write_text('{"old": true}', encoding='utf-8')
            denied = PermissionError('Persistent Windows access denial')
            denied.winerror = 5
            with patch.object(Path, 'replace', side_effect=denied) as replace, patch('records.time.sleep') as sleep:
                with self.assertRaises(PermissionError): save(target, {'new': True})
            self.assertEqual(replace.call_count, 8)
            self.assertEqual(sleep.call_count, 7)
            self.assertAlmostEqual(sum(call.args[0] for call in sleep.call_args_list), 1.27)
            self.assertEqual(json.loads(target.read_text()), {'old': True})
            self.assertEqual(json.loads(target.with_suffix('.json.pending').read_text()), {'new': True})

    def test_other_permission_errors_are_not_retried(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(Path, 'replace', side_effect=PermissionError('Denied')) as replace, patch('records.time.sleep') as sleep:
                with self.assertRaises(PermissionError): save(Path(directory)/'result.json', {})
            replace.assert_called_once()
            sleep.assert_not_called()

    def test_non_permission_errors_are_not_retried(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(Path, 'replace', side_effect=OSError('Storage error')) as replace, patch('records.time.sleep') as sleep:
                with self.assertRaises(OSError): save(Path(directory)/'result.json', {})
            replace.assert_called_once()
            sleep.assert_not_called()


if __name__ == '__main__': unittest.main()
